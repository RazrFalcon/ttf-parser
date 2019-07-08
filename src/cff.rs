// Useful links:
// http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf
// http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5177.Type2.pdf
// https://github.com/opentypejs/opentype.js/blob/master/src/tables/cff.js

use std::ops::Range;

use crate::parser::{Stream, TryFromData, SafeStream, TrySlice};
use crate::{Font, GlyphId, TableName, OutlineBuilder, Rect, Result, Error};

// Limits according to the Adobe Technical Note #5176, chapter 4 DICT Data.
const MAX_OPERANDS_LEN: usize = 48;

// Limits according to the Adobe Technical Note #5177 Appendix B.
const STACK_LIMIT: u8 = 10;
const MAX_ARGUMENTS_STACK_LEN: usize = 48;

const TWO_BYTE_OPERATOR_MARK: u8 = 12;


/// A list of errors that can occur during a CFF table parsing.
#[derive(Clone, Copy, Debug)]
pub enum CFFError {
    /// An invalid operand occurred.
    InvalidOperand,

    /// An invalid operator occurred.
    InvalidOperator,

    /// An unsupported operator occurred.
    UnsupportedOperator,

    /// Failed to parse a float number.
    InvalidFloat,

    /// The `OffSize` value must be in 1..4 range.
    ///
    /// Adobe Technical Note #5176, Table 2 CFF Data Types
    InvalidOffsetSize,

    /// Subroutines nesting is limited by 10.
    ///
    /// Adobe Technical Note #5177 Appendix B.
    NestingLimitReached,

    /// An arguments stack size is limited by 48 values.
    ///
    /// Adobe Technical Note #5177 Appendix B.
    ArgumentsStackLimitReached,

    /// Each operand expects a specific amount of arguments on the stack.
    ///
    /// Usually indicates an implementation error and should not occur on valid fonts.
    InvalidArgumentsStackLength,
}

impl std::fmt::Display for CFFError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CFFError::InvalidOperand => {
                write!(f, "an invalid operand occurred")
            }
            CFFError::InvalidOperator => {
                write!(f, "an invalid operator occurred")
            }
            CFFError::UnsupportedOperator => {
                write!(f, "an unsupported operator occurred")
            }
            CFFError::InvalidFloat => {
                write!(f, "failed to parse a float number")
            }
            CFFError::InvalidOffsetSize => {
                write!(f, "OffSize with an invalid value occurred")
            }
            CFFError::NestingLimitReached => {
                write!(f, "subroutines nesting limit reached")
            }
            CFFError::ArgumentsStackLimitReached => {
                write!(f, "arguments stack limit reached")
            }
            CFFError::InvalidArgumentsStackLength => {
                write!(f, "an invalid amount of items are in an arguments stack")
            }
        }
    }
}

impl std::error::Error for CFFError {}


impl<'a> Font<'a> {
    pub(crate) fn cff_glyph_outline(
        &self,
        glyph_id: GlyphId,
        builder: &mut impl OutlineBuilder,
    ) -> Result<Rect> {
        let data = self.table_data(TableName::CompactFontFormat)?;
        let mut s = Stream::new(data);

        // Parse Header.
        let major: u8 = s.read()?;
        s.skip::<u8>(); // minor
        let header_size: u8 = s.read()?;
        s.skip::<u8>(); // Absolute offset

        if major != 1 {
            return Err(Error::UnsupportedTableVersion(TableName::CompactFontFormat, major as u16));
        }

        // Jump to Name INDEX. It's not necessarily right after the header.
        if header_size > s.offset() as u8 {
            s.skip_len(header_size as u32 - s.offset() as u32);
        }

        // Skip Name INDEX.
        skip_index(&mut s)?;

        let top_dict = parse_top_dict(&mut s)?;

        let private_dict = match top_dict.private_dict_range.clone() {
            Some(range) => {
                let range = range.start as usize .. range.end as usize;
                let mut s = Stream::new(data.try_slice(range)?);
                parse_private_dict(&mut s)?
            }
            _ => {
                return Err(Error::NoGlyph);
            }
        };

        // Skip String INDEX.
        skip_index(&mut s)?;

        // Parse Global Subroutines INDEX.
        let global_subrs = parse_index(&mut s)?;

        let mut local_subrs = DataIndex::create_empty();
        match (top_dict.private_dict_range, private_dict.subroutines_offset) {
            (Some(private_dict_range), Some(subroutines_offset)) => {
                // 'The local subroutines offset is relative to the beginning
                // of the Private DICT data.'
                if let Some(start) = private_dict_range.start.checked_add(subroutines_offset) {
                    let data = data.try_slice(start as usize..data.len())?;
                    let mut s = Stream::new(data);
                    local_subrs = parse_index(&mut s)?;
                }
            }
            _ => {}
        }

        if let Some(offset) = top_dict.char_strings_offset {
            let start = offset as usize;
            let mut s = Stream::new(data.try_slice(start..data.len())?);
            parse_char_string(global_subrs, local_subrs, glyph_id, &mut s, builder)
        } else {
            Err(Error::NoGlyph)
        }
    }
}

#[derive(Clone, Default)]
struct TopDict {
    char_strings_offset: Option<u32>,
    private_dict_range: Option<Range<u32>>,
}

fn parse_top_dict(s: &mut Stream) -> Result<TopDict> {
    let mut dict = TopDict::default();

    let index = parse_index(s)?;

    // The Top DICT INDEX should have only one dictionary.
    let data = match index.get(0) {
        Some(v) => v,
        None => return Ok(dict),
    };

    let mut s = Stream::new(data);
    let mut dict_parser = DictionaryParser::new(&mut s);
    while let Some(result) = dict_parser.parse_next() {
        let operator = result?;
        let operands = dict_parser.operands();

        // Adobe Technical Note #5176, Table 9 Top DICT Operator Entries
        match operator.value() {
            17 if operands.len() == 1 => {
                dict.char_strings_offset = Some(operands[0].as_i32() as u32);
            }
            18 if operands.len() == 2 => {
                let start = operands[1].as_i32() as u32;
                let len = operands[0].as_i32() as u32;

                if let Some(end) = start.checked_add(len) {
                    dict.private_dict_range = Some(start..end);
                }
            }
            1206 if operands.len() == 1 => { // CharstringType
                // Check that `charstring` type is set to 2. We do not support other formats.
                // Usually, it will not be set at all, therefore defaults to 2.
                if operands[0].as_i32() != 2 {
                    return Err(Error::NoGlyph);
                }
            }
            _ => {}
        }
    }

    Ok(dict)
}

#[derive(Clone, Copy, Default)]
struct PrivateDict {
    subroutines_offset: Option<u32>,
}

fn parse_private_dict<'a>(s: &'a mut Stream<'a>) -> Result<PrivateDict> {
    let mut dict = PrivateDict::default();

    let mut dict_parser = DictionaryParser::new(s);
    while let Some(result) = dict_parser.parse_next() {
        let operator = result?;
        let operands = dict_parser.operands();

        // Adobe Technical Note #5176, Table 23 Private DICT Operators
        if operator.value() == 19 && operands.len() == 1 {
            dict.subroutines_offset = Some(operands[0].as_i32() as u32);
            break;
        }
    }

    Ok(dict)
}

struct CharStringParserContext<'a> {
    global_subrs: DataIndex<'a>,
    local_subrs: DataIndex<'a>,
    is_first_move_to: bool,
    width_parsed: bool,
    stems_len: u32,
}

fn parse_char_string(
    global_subrs: DataIndex,
    local_subrs: DataIndex,
    glyph_id: GlyphId,
    s: &mut Stream,
    builder: &mut impl OutlineBuilder,
) -> Result<Rect> {
    let char_strings = parse_index(s)?;
    let data = char_strings.get(glyph_id.0).ok_or(Error::NoGlyph)?;

    let mut ctx = CharStringParserContext {
        global_subrs,
        local_subrs,
        is_first_move_to: true,
        width_parsed: false,
        stems_len: 0,
    };

    let mut inner_builder = Builder {
        builder,
        bbox: RectF {
            x_min: std::f32::MAX,
            y_min: std::f32::MAX,
            x_max: std::f32::MIN,
            y_max: std::f32::MIN,
        }
    };

    let mut stack = ArgumentsStack::new();
    let _ = _parse_char_string(&mut ctx, data, 0.0, 0.0, &mut stack, 0, &mut inner_builder)?;

    let bbox = inner_builder.bbox;
    Ok(Rect {
        x_min: bbox.x_min as i16,
        y_min: bbox.y_min as i16,
        x_max: bbox.x_max as i16,
        y_max: bbox.y_max as i16,
    })
}


struct RectF {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

trait OutlineBuilderInner {
    fn update_bbox(&mut self, x: f32, y: f32);
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);
    fn close(&mut self);
}

struct Builder<'a, T: OutlineBuilder> {
    builder: &'a mut T,
    bbox: RectF,
}

impl<'a, T: OutlineBuilder> OutlineBuilderInner for Builder<'a, T> {
    #[inline]
    fn update_bbox(&mut self, x: f32, y: f32) {
        self.bbox.x_min = self.bbox.x_min.min(x);
        self.bbox.y_min = self.bbox.y_min.min(y);

        self.bbox.x_max = self.bbox.x_max.max(x);
        self.bbox.y_max = self.bbox.y_max.max(y);
    }

    #[inline]
    fn move_to(&mut self, x: f32, y: f32) {
        self.update_bbox(x, y);
        self.builder.move_to(x, y);
    }

    #[inline]
    fn line_to(&mut self, x: f32, y: f32) {
        self.update_bbox(x, y);
        self.builder.line_to(x, y);
    }

    #[inline]
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.update_bbox(x1, y1);
        self.update_bbox(x2, y2);
        self.update_bbox(x, y);
        self.builder.curve_to(x1, y1, x2, y2, x, y);
    }

    #[inline]
    fn close(&mut self) {
        self.builder.close();
    }
}


fn _parse_char_string<T: OutlineBuilder>(
    ctx: &mut CharStringParserContext,
    char_string: &[u8],
    mut x: f32,
    mut y: f32,
    stack: &mut ArgumentsStack,
    depth: u8,
    builder: &mut Builder<T>,
) -> Result<(f32, f32)> {
    let mut s = Stream::new(char_string);

    while !s.at_end() {
        let op: u8 = s.read()?;
        match op {
            0 | 2 | 9 | 13 | 15 | 16 | 17 => {
                // Reserved.
                return Err(CFFError::InvalidOperator.into());
            }
            1 | 3 | 18 | 23 => {
                // |- y dy {dya dyb}* hstem (1) |-
                // |- x dx {dxa dxb}* vstem (3) |-
                // |- y dy {dya dyb}* hstemhm (18) |-
                // |- x dx {dxa dxb}* vstemhm (23) |-

                // If the stack length is uneven, than the first value is a `width`.
                let len = if stack.len().is_odd() && !ctx.width_parsed {
                    ctx.width_parsed = true;
                    stack.len() - 1
                } else {
                    stack.len()
                };

                ctx.stems_len += len as u32 >> 1;

                // We are ignoring the hint operators.
                stack.clear();
            }
            4 => {
                // |- dy1 vmoveto (4) |-

                let mut i = 0;
                if stack.len() == 2 && !ctx.width_parsed {
                    i += 1;
                    ctx.width_parsed = true;
                } else if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                y += stack.at(i);
                builder.move_to(x, y);

                stack.clear();
            }
            5 => {
                // |- {dxa dya}+ rlineto (5) |-

                if stack.len().is_odd() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() {
                    x += stack.at(i + 0);
                    y += stack.at(i + 1);
                    builder.line_to(x, y);
                    i += 2;
                }

                stack.clear();
            }
            6 => {
                // |- dx1 {dya dxb}* hlineto (6) |-
                // |-     {dxa dyb}+ hlineto (6) |-

                let mut i = 0;
                while i < stack.len() {
                    x += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);

                    if i == stack.len() {
                        break;
                    }

                    y += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);
                }

                stack.clear();
            }
            7 => {
                // |- dy1 {dxa dyb}* vlineto (7) |-
                // |-     {dya dxb}+ vlineto (7) |-

                let mut i = 0;
                while i < stack.len() {
                    y += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);

                    if i == stack.len() {
                        break;
                    }

                    x += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);
                }

                stack.clear();
            }
            8 => {
                // |- {dxa dya dxb dyb dxc dyc}+ rrcurveto (8) |-

                if stack.len() % 6 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() {
                    let x1 = x + stack.at(i + 0);
                    let y1 = y + stack.at(i + 1);
                    let x2 = x1 + stack.at(i + 2);
                    let y2 = y1 + stack.at(i + 3);
                    x = x2 + stack.at(i + 4);
                    y = y2 + stack.at(i + 5);

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 6;
                }

                stack.clear();
            }
            10 => {
                // subr# callsubr (10) –

                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if depth == STACK_LIMIT {
                    return Err(CFFError::NestingLimitReached.into());
                }

                let subroutine_bias = calc_subroutine_bias(ctx.local_subrs.len() as u16);
                let index = stack.pop() as i32 + subroutine_bias as i32;
                let char_string = ctx.local_subrs.get(index as u16).ok_or(Error::NoGlyph)?;
                let pos = _parse_char_string(ctx, char_string, x, y, stack, depth + 1, builder)?;
                x = pos.0;
                y = pos.1;
            }
            11 => {
                // – return (11) –
                break;
            }
            TWO_BYTE_OPERATOR_MARK => {
                // flex
                let op2: u8 = s.read()?;
                match op2 {
                    34 => {
                        // |- dx1 dx2 dy2 dx3 dx4 dx5 dx6 hflex (12 34) |-

                        if stack.len() != 7 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y;
                        let dx2 = dx1 + stack.at(1);
                        let dy2 = dy1 + stack.at(2);
                        let dx3 = dx2 + stack.at(3);
                        let dy3 = dy2;
                        let dx4 = dx3 + stack.at(4);
                        let dy4 = dy2;
                        let dx5 = dx4 + stack.at(5);
                        let dy5 = y;
                        x = dx5 + stack.at(6);
                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    35 => {
                        // |- dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 dx6 dy6 fd flex (12 35) |-

                        if stack.len() != 13 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y + stack.at(1);
                        let dx2 = dx1 + stack.at(2);
                        let dy2 = dy1 + stack.at(3);
                        let dx3 = dx2 + stack.at(4);
                        let dy3 = dy2 + stack.at(5);
                        let dx4 = dx3 + stack.at(6);
                        let dy4 = dy3 + stack.at(7);
                        let dx5 = dx4 + stack.at(8);
                        let dy5 = dy4 + stack.at(9);
                        x = dx5 + stack.at(10);
                        y = dy5 + stack.at(11);
                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    36 => {
                        // |- dx1 dy1 dx2 dy2 dx3 dx4 dx5 dy5 dx6 hflex1 (12 36) |-

                        if stack.len() != 9 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y + stack.at(1);
                        let dx2 = dx1 + stack.at(2);
                        let dy2 = dy1 + stack.at(3);
                        let dx3 = dx2 + stack.at(4);
                        let dy3 = dy2;
                        let dx4 = dx3 + stack.at(5);
                        let dy4 = dy2;
                        let dx5 = dx4 + stack.at(6);
                        let dy5 = dy4 + stack.at(7);
                        x = dx5 + stack.at(8);
                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    37 => {
                        // |- dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 d6 flex1 (12 37) |-

                        if stack.len() != 11 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y + stack.at(1);
                        let dx2 = dx1 + stack.at(2);
                        let dy2 = dy1 + stack.at(3);
                        let dx3 = dx2 + stack.at(4);
                        let dy3 = dy2 + stack.at(5);
                        let dx4 = dx3 + stack.at(6);
                        let dy4 = dy3 + stack.at(7);
                        let dx5 = dx4 + stack.at(8);
                        let dy5 = dy4 + stack.at(9);

                        if (dx5 - x).abs() > (dy5 - y).abs() {
                            x = dx5 + stack.at(10);
                        } else {
                            y = dy5 + stack.at(10);
                        }

                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    _ => {
                        return Err(CFFError::UnsupportedOperator.into());
                    }
                }
            }
            14 => {
                // – endchar (14) |–

                if !stack.is_empty() && !ctx.width_parsed {
                    stack.clear();
                    ctx.width_parsed = true;
                }

                if !ctx.is_first_move_to {
                    ctx.is_first_move_to = true;
                    builder.close();
                }
            }
            19 | 20 => {
                // |- hintmask (19 + mask) |-
                // |- cntrmask (20 + mask) |-

                let mut len = stack.len();

                // We are ignoring the hint operators.
                stack.clear();

                // If the stack length is uneven, than the first value is a `width`.
                if len.is_odd() && !ctx.width_parsed {
                    len -= 1;
                    ctx.width_parsed = true;
                }

                ctx.stems_len += len as u32 >> 1;

                s.skip_len((ctx.stems_len + 7) >> 3);
            }
            21 => {
                // |- dx1 dy1 rmoveto (21) |-

                let mut i = 0;
                if stack.len() == 3 && !ctx.width_parsed {
                    i += 1;
                    ctx.width_parsed = true;
                } else if stack.len() != 2 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                x += stack.at(i + 0);
                y += stack.at(i + 1);
                builder.move_to(x, y);

                stack.clear();
            }
            22 => {
                // |- dx1 hmoveto (22) |-

                let mut i = 0;
                if stack.len() == 2 && !ctx.width_parsed {
                    i += 1;
                    ctx.width_parsed = true;
                } else if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                x += stack.at(i);
                builder.move_to(x, y);

                stack.clear();
            }
            24 => {
                // |- {dxa dya dxb dyb dxc dyc}+ dxd dyd rcurveline (24) |-

                if stack.len() < 8 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if (stack.len() - 2) % 6 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() - 2 {
                    let x1 = x + stack.at(i + 0);
                    let y1 = y + stack.at(i + 1);
                    let x2 = x1 + stack.at(i + 2);
                    let y2 = y1 + stack.at(i + 3);
                    x = x2 + stack.at(i + 4);
                    y = y2 + stack.at(i + 5);

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 6;
                }

                x += stack.at(i + 0);
                y += stack.at(i + 1);
                builder.line_to(x, y);

                stack.clear();
            }
            25 => {
                // |- {dxa dya}+ dxb dyb dxc dyc dxd dyd rlinecurve (25) |-

                if stack.len() < 8 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if (stack.len() - 6).is_odd() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() - 6 {
                    x += stack.at(i + 0);
                    y += stack.at(i + 1);

                    builder.line_to(x, y);
                    i += 2;
                }

                let x1 = x + stack.at(i + 0);
                let y1 = y + stack.at(i + 1);
                let x2 = x1 + stack.at(i + 2);
                let y2 = y1 + stack.at(i + 3);
                x = x2 + stack.at(i + 4);
                y = y2 + stack.at(i + 5);
                builder.curve_to(x1, y1, x2, y2, x, y);

                stack.clear();
            }
            26 => {
                // |- dx1? {dya dxb dyb dyc}+ vvcurveto (26) |-

                let mut i = 0;

                // The odd argument count indicates an X position.
                if stack.len().is_odd() {
                    x += stack.at(0);
                    i += 1;
                }

                if (stack.len() - i) % 4 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                while i < stack.len() {
                    let x1 = x;
                    let y1 = y + stack.at(i + 0);
                    let x2 = x1 + stack.at(i + 1);
                    let y2 = y1 + stack.at(i + 2);
                    x = x2;
                    y = y2 + stack.at(i + 3);

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 4;
                }

                stack.clear();
            }
            27 => {
                // |- dy1? {dxa dxb dyb dxc}+ hhcurveto (27) |-

                let mut i = 0;

                // The odd argument count indicates an Y position.
                if stack.len().is_odd() {
                    y += stack.at(0);
                    i += 1;
                }

                if (stack.len() - i) % 4 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                while i < stack.len() {
                    let x1 = x + stack.at(i + 0);
                    let y1 = y;
                    let x2 = x1 + stack.at(i + 1);
                    let y2 = y1 + stack.at(i + 2);
                    x = x2 + stack.at(i + 3);
                    y = y2;

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 4;
                }

                stack.clear();
            }
            28 => {
                let b1 = s.read::<u8>()? as i32;
                let b2 = s.read::<u8>()? as i32;
                let n = ((b1 << 24) | (b2 << 16)) >> 16;
                debug_assert!((-32768..=32767).contains(&n));
                stack.push(n as f32)?;
            }
            29 => {
                // globalsubr# callgsubr (29) –

                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if depth == STACK_LIMIT {
                    return Err(CFFError::NestingLimitReached.into());
                }

                let subroutine_bias = calc_subroutine_bias(ctx.global_subrs.len() as u16);
                let index = stack.pop() as i32 + subroutine_bias as i32;
                let char_string = ctx.global_subrs.get(index as u16).ok_or(Error::NoGlyph)?;
                let pos = _parse_char_string(ctx, char_string, x, y, stack, depth + 1, builder)?;
                x = pos.0;
                y = pos.1;
            }
            30 => {
                // |- dy1 dx2 dy2 dx3 {dxa dxb dyb dyc dyd dxe dye dxf}* dyf? vhcurveto (30) |-
                // |-                 {dya dxb dyb dxc dxd dxe dye dyf}+ dxf? vhcurveto (30) |-

                if stack.len() < 4 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                stack.reverse();
                while !stack.is_empty() {
                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x;
                    let y1 = y + stack.pop();
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    x = x2 + stack.pop();
                    y = y2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                    if stack.is_empty() {
                        break;
                    }

                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x + stack.pop();
                    let y1 = y;
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    y = y2 + stack.pop();
                    x = x2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                }

                debug_assert!(stack.is_empty());
            }
            31 => {
                // |- dx1 dx2 dy2 dy3 {dya dxb dyb dxc dxd dxe dye dyf}* dxf? hvcurveto (31) |-
                // |-                 {dxa dxb dyb dyc dyd dxe dye dxf}+ dyf? hvcurveto (31) |-

                if stack.len() < 4 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                stack.reverse();
                while !stack.is_empty() {
                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x + stack.pop();
                    let y1 = y;
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    y = y2 + stack.pop();
                    x = x2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                    if stack.is_empty() {
                        break;
                    }

                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x;
                    let y1 = y + stack.pop();
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    x = x2 + stack.pop();
                    y = y2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                }

                debug_assert!(stack.is_empty());
            }
            32...246 => {
                let n = op as i32 - 139;
                debug_assert!((-107..=107).contains(&n));
                stack.push(n as f32)?;
            }
            247...250 => {
                let b1 = s.read::<u8>()? as i32;
                let n = (op as i32 - 247) * 256 + b1 + 108;
                debug_assert!((108..=1131).contains(&n));
                stack.push(n as f32)?;
            }
            251...254 => {
                let b1 = s.read::<u8>()? as i32;
                let n = -(op as i32 - 251) * 256 - b1 - 108;
                debug_assert!((-1131..=-108).contains(&n));
                stack.push(n as f32)?;
            }
            255 => {
                let n = s.read::<u32>()? as i32 as f32 / 65536.0;
                stack.push(n)?;
            }
        }
    }

    Ok((x, y))
}

// Adobe Technical Note #5176, Chapter 16 "Local / Global Subrs INDEXes"
#[inline]
fn calc_subroutine_bias(len: u16) -> u16 {
    if len < 1240 {
        107
    } else if len < 33900 {
        1131
    } else {
        32768
    }
}

fn parse_index<'a>(s: &mut Stream<'a>) -> Result<DataIndex<'a>> {
    let count: u16 = s.read()?;
    if count != 0 && count != std::u16::MAX {
        let offset_size: OffsetSize = s.try_read()?;
        let offsets = parse_var_offsets(s, count + 1, offset_size)?;
        match offsets.last() {
            Some(last_offset) => {
                let data = s.read_bytes(last_offset)?;
                Ok(DataIndex { data, offsets })
            }
            None => {
                Ok(DataIndex::create_empty())
            }
        }
    } else {
        Ok(DataIndex::create_empty())
    }
}

fn skip_index(s: &mut Stream) -> Result<()> {
    let count: u16 = s.read()?;
    if count != 0 && count != std::u16::MAX {
        let offset_size: OffsetSize = s.try_read()?;
        let offsets = parse_var_offsets(s, count + 1, offset_size)?;
        if let Some(last_offset) = offsets.last() {
            s.skip_len(last_offset);
        }
    }

    Ok(())
}

fn parse_var_offsets<'a>(
    s: &mut Stream<'a>,
    count: u16,
    offset_size: OffsetSize,
) -> Result<VarOffsets<'a>> {
    let offsets_len = count as u32 * offset_size as u32;
    Ok(VarOffsets {
        data: &s.read_bytes(offsets_len)?,
        offset_size,
    })
}


#[derive(Clone, Copy)]
struct VarOffsets<'a> {
    data: &'a [u8],
    offset_size: OffsetSize,
}

impl<'a> VarOffsets<'a> {
    fn get(&self, index: u16) -> Option<u32> {
        if index < self.len() {
            let start = index as usize * self.offset_size as usize;
            let end = start + self.offset_size as usize;
            let data = self.data.try_slice(start..end).ok()?;

            let mut s = SafeStream::new(data);
            let n = match self.offset_size {
                OffsetSize::Size1 => s.read::<u8>() as u32,
                OffsetSize::Size2 => s.read::<u16>() as u32,
                OffsetSize::Size3 => s.read_u24(),
                OffsetSize::Size4 => s.read::<u32>(),
            };

            // Offset must be positive.
            if n == 0 {
                return None;
            }

            // Offsets are offset by one byte in the font,
            // so we have to shift them back.
            Some(n - 1)
        } else {
            None
        }
    }

    #[inline]
    fn last(&self) -> Option<u32> {
        if self.len() != 0 {
            self.get(self.len() - 1)
        } else {
            None
        }
    }

    #[inline]
    fn len(&self) -> u16 {
        self.data.len() as u16 / self.offset_size as u16
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


#[derive(Clone, Copy)]
struct DataIndex<'a> {
    data: &'a [u8],
    offsets: VarOffsets<'a>,
}

impl<'a> DataIndex<'a> {
    fn create_empty() -> Self {
        DataIndex {
            data: b"",
            offsets: VarOffsets { data: b"", offset_size: OffsetSize::Size1 },
        }
    }

    #[inline]
    fn len(&self) -> u16 {
        if !self.offsets.is_empty() {
            // Last offset points to the byte after the `Object data`.
            // We should skip it.
            self.offsets.len() - 1
        } else {
            0
        }
    }

    fn get(&self, index: u16) -> Option<&'a [u8]> {
        // Check for overflow first.
        if index == std::u16::MAX {
            None
        } else if index + 1 < self.offsets.len() {
            let start = self.offsets.get(index)? as usize;
            let end = self.offsets.get(index + 1)? as usize;
            let data = self.data.try_slice(start..end).ok()?;
            Some(data)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for DataIndex<'a> {
    type Item = &'a [u8];
    type IntoIter = DataIndexIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        DataIndexIter {
            data: self,
            index: 0,
        }
    }
}

struct DataIndexIter<'a> {
    data: DataIndex<'a>,
    index: u16,
}

impl<'a> Iterator for DataIndexIter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.data.get(index)
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum OffsetSize {
    Size1 = 1,
    Size2 = 2,
    Size3 = 3,
    Size4 = 4,
}

impl TryFromData for OffsetSize {
    #[inline]
    fn try_parse(s: &mut SafeStream) -> Result<Self> {
        let n: u8 = s.read();
        match n {
            1 => Ok(OffsetSize::Size1),
            2 => Ok(OffsetSize::Size2),
            3 => Ok(OffsetSize::Size3),
            4 => Ok(OffsetSize::Size4),
            _ => Err(CFFError::InvalidOffsetSize.into()),
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct Operator(u16);

impl Operator {
    #[inline]
    fn value(&self) -> u16 { self.0 }
}


struct DictionaryParser<'a> {
    stream: &'a mut Stream<'a>,
    operands: [Number; MAX_OPERANDS_LEN], // 192B
    operands_len: u8,
}

impl<'a> DictionaryParser<'a> {
    fn new(stream: &'a mut Stream<'a>) -> Self {
        DictionaryParser {
            stream,
            operands: [Number::Integer(0); MAX_OPERANDS_LEN],
            operands_len: 0,
        }
    }

    fn parse_next(&mut self) -> Option<Result<Operator>> {
        self.operands_len = 0;
        while !self.stream.at_end() {
            let b: u8 = self.stream.read().ok()?;
            // 0..=21 bytes are operators.
            if b <= 21 {
                let mut operator = b as u16;

                // Check that operator is two byte long.
                if b == TWO_BYTE_OPERATOR_MARK {
                    // Use a 1200 'prefix' to make two byte operators more readable.
                    // 12 3 => 1203
                    operator = 1200 + self.stream.read::<u8>().ok()? as u16;
                }

                return Some(Ok(Operator(operator)));
            } else {
                let op = match parse_number(b, &mut self.stream) {
                    Ok(op) => op,
                    Err(e) => {
                        // Jump to the end of the stream,
                        // so the next `parse_next()` will return `None`.
                        self.stream.jump_to_end();
                        return Some(Err(e));
                    }
                };

                self.operands[self.operands_len as usize] = op;
                self.operands_len += 1;

                if self.operands_len >= MAX_OPERANDS_LEN as u8 {
                    break;
                }
            }
        }

        None
    }

    fn operands(&self) -> &[Number] {
        &self.operands[..self.operands_len as usize]
    }
}

// Adobe Technical Note #5177, Table 3 Operand Encoding
fn parse_number(b0: u8, s: &mut Stream) -> Result<Number> {
    match b0 {
        28 => {
            let n = s.read::<u16>()? as i32;
            Ok(Number::Integer(n))
        }
        29 => {
            let n = s.read::<u32>()? as i32;
            Ok(Number::Integer(n))
        }
        30 => {
            parse_float(s)
        }
        32...246 => {
            let n = b0 as i32 - 139;
            Ok(Number::Integer(n))
        }
        247...250 => {
            let b1 = s.read::<u8>()? as i32;
            let n = (b0 as i32 - 247) * 256 + b1 + 108;
            Ok(Number::Integer(n))
        }
        251...254 => {
            let b1 = s.read::<u8>()? as i32;
            let n = -(b0 as i32 - 251) * 256 - b1 - 108;
            Ok(Number::Integer(n))
        }
        _ => Err(CFFError::InvalidOperand.into()),
    }
}

fn parse_float(s: &mut Stream) -> Result<Number> {
    const STACK_LEN: usize = 64;
    const END_OF_NUMBER: u8 = 0xf;

    let mut data = [0u8; STACK_LEN];
    let mut idx = 0;

    // Adobe Technical Note #5176, Table 5 Nibble Definitions
    let mut lookup = |n: u8| -> Result<()> {
        if idx == STACK_LEN {
            return Err(CFFError::InvalidFloat.into());
        }

        match n {
            0...9 => {
                data[idx] = b'0' + n;
            }
            10 => {
                data[idx] = b'.';
            }
            11 => {
                data[idx] = b'E';
            }
            12 => {
                if idx + 1 == STACK_LEN {
                    return Err(CFFError::InvalidFloat.into());
                }

                data[idx] = b'E';
                idx += 1;
                data[idx] = b'-';
            }
            13 => {
                return Err(CFFError::InvalidFloat.into());
            }
            14 => {
                data[idx] = b'-';
            }
            _ => {
                return Err(CFFError::InvalidFloat.into());
            }
        }

        idx += 1;
        Ok(())
    };

    loop {
        let b1: u8 = s.read()?;
        let nibble1 = b1 >> 4;
        let nibble2 = b1 & 15;

        if nibble1 == END_OF_NUMBER {
            break;
        }

        lookup(nibble1)?;

        if nibble2 == END_OF_NUMBER {
            break;
        }

        lookup(nibble2)?;
    }

    let s = std::str::from_utf8(&data[..idx]).map_err(|_| CFFError::InvalidFloat)?;
    let n = s.parse().map_err(|_| CFFError::InvalidFloat)?;
    Ok(Number::Float(n))
}


#[derive(Clone, Copy, Debug)]
enum Number {
    Integer(i32),
    Float(f32),
}

impl Number {
    #[inline]
    fn as_i32(&self) -> i32 {
        match *self {
            Number::Integer(n) => n,
            Number::Float(n) => n as i32,
        }
    }
}


struct ArgumentsStack {
    data: [f32; MAX_ARGUMENTS_STACK_LEN], // 192B
    len: usize,
}

impl ArgumentsStack {
    fn new() -> Self {
        ArgumentsStack {
            data: [0.0; MAX_ARGUMENTS_STACK_LEN],
            len: 0,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn push(&mut self, n: f32) -> Result<()> {
        if self.len == MAX_ARGUMENTS_STACK_LEN {
            Err(CFFError::ArgumentsStackLimitReached.into())
        } else {
            self.data[self.len] = n;
            self.len += 1;
            Ok(())
        }
    }

    #[inline]
    fn at(&self, index: usize) -> f32 {
        self.data[index]
    }

    #[inline]
    fn pop(&mut self) -> f32 {
        debug_assert!(!self.is_empty());
        self.len -= 1;
        self.data[self.len]
    }

    #[inline]
    fn reverse(&mut self) {
        if self.is_empty() {
            return;
        }

        // Reverse only the actual data and not the whole stack.
        let (first, _) = self.data.split_at_mut(self.len);
        first.reverse();
    }

    #[inline]
    fn clear(&mut self) {
        self.len = 0;
    }
}

impl<'a> std::fmt::Debug for ArgumentsStack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list().entries(&self.data[..self.len]).finish()
    }
}


trait IsEven {
    fn is_even(&self) -> bool;
    fn is_odd(&self) -> bool;
}

impl IsEven for usize {
    #[inline]
    fn is_even(&self) -> bool { (*self) & 1 == 0 }

    #[inline]
    fn is_odd(&self) -> bool { !self.is_even() }
}

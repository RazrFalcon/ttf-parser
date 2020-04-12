// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2
// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2charstr

use core::convert::TryFrom;
use core::ops::Range;

use crate::{GlyphId, OutlineBuilder, Rect, BBox, NormalizedCoord};
use crate::parser::{Stream, Fixed, NumFrom, TryNumFrom};
use crate::var_store::*;
use crate::cff::{
    Builder, DataIndex, IsEven, Operator, ArgumentsStack, CFFError,
    calc_subroutine_bias, f32_abs, parse_number, skip_number, parse_index_impl,
    is_dict_one_byte_op, conv_subroutine_index
};

// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2#7-top-dict-data
// 'Operators in DICT may be preceded by up to a maximum of 513 operands.'
const MAX_OPERANDS_LEN: usize = 513;

// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2charstr#appendix-b-cff2-charstring-implementation-limits
const STACK_LIMIT: u8 = 10;
const MAX_ARGUMENTS_STACK_LEN: usize = 513;

const TWO_BYTE_OPERATOR_MARK: u8 = 12;

// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2charstr#4-charstring-operators
mod operator {
    pub const HORIZONTAL_STEM: u8           = 1;
    pub const VERTICAL_STEM: u8             = 3;
    pub const VERTICAL_MOVE_TO: u8          = 4;
    pub const LINE_TO: u8                   = 5;
    pub const HORIZONTAL_LINE_TO: u8        = 6;
    pub const VERTICAL_LINE_TO: u8          = 7;
    pub const CURVE_TO: u8                  = 8;
    pub const CALL_LOCAL_SUBROUTINE: u8     = 10;
    pub const VS_INDEX: u8                  = 15;
    pub const BLEND: u8                     = 16;
    pub const HORIZONTAL_STEM_HINT_MASK: u8 = 18;
    pub const HINT_MASK: u8                 = 19;
    pub const COUNTER_MASK: u8              = 20;
    pub const MOVE_TO: u8                   = 21;
    pub const HORIZONTAL_MOVE_TO: u8        = 22;
    pub const VERTICAL_STEM_HINT_MASK: u8   = 23;
    pub const CURVE_LINE: u8                = 24;
    pub const LINE_CURVE: u8                = 25;
    pub const VV_CURVE_TO: u8               = 26;
    pub const HH_CURVE_TO: u8               = 27;
    pub const SHORT_INT: u8                 = 28;
    pub const CALL_GLOBAL_SUBROUTINE: u8    = 29;
    pub const VH_CURVE_TO: u8               = 30;
    pub const HV_CURVE_TO: u8               = 31;
    pub const HFLEX: u8                     = 34;
    pub const FLEX: u8                      = 35;
    pub const HFLEX1: u8                    = 36;
    pub const FLEX1: u8                     = 37;
    pub const FIXED_16_16: u8               = 255;
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2#table-9-top-dict-operator-entries
mod top_dict_operator {
    pub const CHAR_STRINGS_OFFSET: u16      = 17;
    pub const VARIATION_STORE_OFFSET: u16   = 24;
    pub const FONT_DICT_INDEX_OFFSET: u16   = 1236;
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2#table-10-font-dict-operator-entries
mod font_dict_operator {
    pub const PRIVATE_DICT_SIZE_AND_OFFSET: u16 = 18;
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cff2#table-16-private-dict-operators
mod private_dict_operator {
    pub const LOCAL_SUBROUTINES_OFFSET: u16 = 19;
}


#[derive(Clone, Copy, Default)]
pub struct Metadata<'a> {
    global_subrs: DataIndex<'a>,
    local_subrs: DataIndex<'a>,
    char_strings: DataIndex<'a>,
    item_variation_store: ItemVariationStore<'a>,
}

pub(crate) fn parse_metadata(data: &[u8]) -> Option<Metadata> {
    let mut s = Stream::new(data);

    // Parse Header.
    let major: u8 = s.read()?;
    s.skip::<u8>(); // minor
    let header_size: u8 = s.read()?;
    let top_dict_length: u16 = s.read()?;

    if major != 2 {
        return None;
    }

    // Jump to Top DICT. It's not necessarily right after the header.
    if header_size > 5 {
        s.advance(usize::from(header_size) - 5);
    }

    let top_dict_data = s.read_bytes(usize::from(top_dict_length))?;
    let top_dict = parse_top_dict(top_dict_data)?;

    let mut metadata = Metadata::default();

    // Parse Global Subroutines INDEX.
    metadata.global_subrs = parse_index(&mut s)?;

    metadata.char_strings = {
        let mut s = Stream::new_at(data, top_dict.char_strings_offset)?;
        parse_index(&mut s)?
    };

    if let Some(offset) = top_dict.variation_store_offset {
        let mut s = Stream::new_at(data, offset)?;
        s.skip::<u16>(); // length
        metadata.item_variation_store = ItemVariationStore::parse(s)?;
    }

    // TODO: simplify
    if let Some(offset) = top_dict.font_dict_index_offset {
        let mut s = Stream::new_at(data, offset)?;
        'outer: for font_dict_data in parse_index(&mut s)? {
            if let Some(private_dict_range) = parse_font_dict(font_dict_data) {
                // 'Private DICT size and offset, from start of the CFF2 table.'
                let private_dict_data = data.get(private_dict_range.clone())?;
                if let Some(subroutines_offset) = parse_private_dict(private_dict_data) {
                    // 'The local subroutines offset is relative to the beginning
                    // of the Private DICT data.'
                    if let Some(start) = private_dict_range.start.checked_add(subroutines_offset) {
                        let data = data.get(start..data.len())?;
                        let mut s = Stream::new(data);
                        metadata.local_subrs = parse_index(&mut s)?;
                        break 'outer;
                    }
                }
            }
        }
    }

    Some(metadata)
}


pub(crate) fn outline(
    metadata: &Metadata,
    coordinates: &[NormalizedCoord],
    glyph_id: GlyphId,
    builder: &mut dyn OutlineBuilder,
) -> Option<Rect> {
    let data = metadata.char_strings.get(glyph_id.0)?;
    parse_char_string(data, metadata, coordinates, builder).ok()
}

#[derive(Clone, Copy, Default)]
struct TopDictData {
    char_strings_offset: usize,
    font_dict_index_offset: Option<usize>,
    variation_store_offset: Option<usize>,
}

fn parse_top_dict(data: &[u8]) -> Option<TopDictData> {
    let mut dict_data = TopDictData::default();

    // TODO: simplify
    let mut dict_parser = DictionaryParser::new(data);
    while let Some(operator) = dict_parser.parse_next() {
        if operator.get() == top_dict_operator::CHAR_STRINGS_OFFSET {
            dict_parser.parse_operands()?;
            let operands = dict_parser.operands();

            if operands.len() == 1 {
                dict_data.char_strings_offset = usize::try_from(operands[0]).ok()?;
            }
        } else if operator.get() == top_dict_operator::FONT_DICT_INDEX_OFFSET {
            dict_parser.parse_operands()?;
            let operands = dict_parser.operands();

            if operands.len() == 1 {
                dict_data.font_dict_index_offset = usize::try_from(operands[0]).ok();
            }
        } else if operator.get() == top_dict_operator::VARIATION_STORE_OFFSET {
            dict_parser.parse_operands()?;
            let operands = dict_parser.operands();

            if operands.len() == 1 {
                dict_data.variation_store_offset = usize::try_from(operands[0]).ok();
            }
        }
    }

    // Must be set, otherwise there are nothing to parse.
    if dict_data.char_strings_offset == 0 {
        return None;
    }

    Some(dict_data)
}

fn parse_font_dict(data: &[u8]) -> Option<Range<usize>> {
    let mut private_dict_range = None;

    let mut dict_parser = DictionaryParser::new(data);
    while let Some(operator) = dict_parser.parse_next() {
        if operator.get() == font_dict_operator::PRIVATE_DICT_SIZE_AND_OFFSET {
            dict_parser.parse_operands()?;
            let operands = dict_parser.operands();

            if operands.len() == 2 {
                let len = usize::try_from(operands[0]).ok()?;
                let start = usize::try_from(operands[1]).ok()?;
                let end = start.checked_add(len)?;
                private_dict_range = Some(start..end);
            }

            break;
        }
    }

    private_dict_range
}

fn parse_private_dict(data: &[u8]) -> Option<usize> {
    let mut subroutines_offset = None;
    let mut dict_parser = DictionaryParser::new(data);
    while let Some(operator) = dict_parser.parse_next() {
        if operator.get() == private_dict_operator::LOCAL_SUBROUTINES_OFFSET {
            dict_parser.parse_operands()?;
            let operands = dict_parser.operands();

            if operands.len() == 1 {
                subroutines_offset = usize::try_from(operands[0]).ok();
            }

            break;
        }
    }

    subroutines_offset
}

/// CFF2 allows up to 65535 scalars, but an average font will have 3-5.
/// So 64 is more than enough.
const SCALARS_MAX: u8 = 64;

#[derive(Clone, Copy)]
pub struct Scalars {
    d: [f32; SCALARS_MAX as usize], // 256B
    len: u8,
}

impl Default for Scalars {
    fn default() -> Self {
        Scalars {
            d: [0.0; SCALARS_MAX as usize],
            len: 0,
        }
    }
}

impl Scalars {
    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn at(&self, i: u8) -> f32 {
        if i < self.len {
            self.d[usize::from(i)]
        } else {
            0.0
        }
    }

    pub fn push(&mut self, n: f32) -> Option<()> {
        if self.len < SCALARS_MAX {
            self.d[usize::from(self.len)] = n;
            self.len += 1;
            Some(())
        } else {
            None
        }
    }
}

struct CharStringParserContext<'a> {
    metadata: &'a Metadata<'a>,
    coordinates: &'a [NormalizedCoord],
    is_first_move_to: bool,
    has_move_to: bool,
    scalars: Scalars,
    had_vsindex: bool,
    had_blend: bool,
    stems_len: u32,
}

impl CharStringParserContext<'_> {
    fn update_scalars(&mut self, index: u16) -> Result<(), CFFError> {
        self.scalars.clear();

        let indices = self.metadata.item_variation_store.region_indices(index)
            .ok_or(CFFError::InvalidItemVariationDataIndex)?;
        for index in indices {
            let scalar = self.metadata.item_variation_store.regions
                .evaluate_region(index, self.coordinates);
            self.scalars.push(scalar)
                .ok_or(CFFError::BlendRegionsLimitReached)?;
        }

        Ok(())
    }
}

fn parse_char_string(
    data: &[u8],
    metadata: &Metadata,
    coordinates: &[NormalizedCoord],
    builder: &mut dyn OutlineBuilder,
) -> Result<Rect, CFFError> {
    let mut ctx = CharStringParserContext {
        metadata,
        coordinates,
        is_first_move_to: true,
        has_move_to: false,
        scalars: Scalars::default(),
        had_vsindex: false,
        had_blend: false,
        stems_len: 0,
    };

    // Load scalars at default index.
    ctx.update_scalars(0)?;

    let mut inner_builder = Builder {
        builder,
        bbox: BBox::new(),
    };

    let mut stack = ArgumentsStack {
        data: &mut [0.0; MAX_ARGUMENTS_STACK_LEN], // 2052B
        len: 0,
        max_len: MAX_ARGUMENTS_STACK_LEN,
    };
    let _ = _parse_char_string(&mut ctx, data, 0.0, 0.0, &mut stack, 0, &mut inner_builder)?;

    let bbox = inner_builder.bbox;

    // Check that bbox was changed.
    if bbox.is_default() {
        return Err(CFFError::ZeroBBox);
    }

    bbox.to_rect().ok_or(CFFError::BboxOverflow)
}

// TODO: It would be great to merge this with CFF, but we need const generics first.
//       And still, we can merge only flex and path operators,
//       since CFF2 doesn't have advance width as a first (optional) argument.
//       On the other hand, other small CFF and CFF2 differences may lead
//       to a more complicated code, so maybe some code duplication would not hurt.
fn _parse_char_string(
    ctx: &mut CharStringParserContext,
    char_string: &[u8],
    mut x: f32,
    mut y: f32,
    stack: &mut ArgumentsStack,
    depth: u8,
    builder: &mut Builder,
) -> Result<(f32, f32), CFFError> {
    let mut s = Stream::new(char_string);
    while !s.at_end() {
        let op: u8 = s.read().ok_or(CFFError::ReadOutOfBounds)?;
        match op {
            0 | 2 | 9 | 11 | 13 | 14 | 17 => {
                // Reserved.
                return Err(CFFError::InvalidOperator);
            }
            operator::HORIZONTAL_STEM |
            operator::VERTICAL_STEM |
            operator::HORIZONTAL_STEM_HINT_MASK |
            operator::VERTICAL_STEM_HINT_MASK => {
                // y dy {dya dyb}* hstem
                // x dx {dxa dxb}* vstem
                // y dy {dya dyb}* hstemhm
                // x dx {dxa dxb}* vstemhm

                ctx.stems_len += stack.len() as u32 >> 1;

                // We are ignoring the hint operators.
                stack.clear();
            }
            operator::VERTICAL_MOVE_TO => {
                // dy1

                if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                ctx.has_move_to = true;

                y += stack.at(0);
                builder.move_to(x, y);

                stack.clear();
            }
            operator::LINE_TO => {
                // {dxa dya}+

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.len().is_odd() {
                    return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::HORIZONTAL_LINE_TO => {
                // dx1 {dya dxb}*
                //     {dxa dyb}+

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

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
            operator::VERTICAL_LINE_TO => {
                // dy1 {dxa dyb}*
                //     {dya dxb}+

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

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
            operator::CURVE_TO => {
                // {dxa dya dxb dyb dxc dyc}+

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.len() % 6 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::CALL_LOCAL_SUBROUTINE => {
                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if depth == STACK_LIMIT {
                    return Err(CFFError::NestingLimitReached);
                }

                let subroutine_bias = calc_subroutine_bias(ctx.metadata.local_subrs.len());
                let index = conv_subroutine_index(stack.pop(), subroutine_bias)?;
                let char_string = ctx.metadata.local_subrs.get(index)
                    .ok_or(CFFError::InvalidSubroutineIndex)?;
                let pos = _parse_char_string(ctx, char_string, x, y, stack, depth + 1, builder)?;
                x = pos.0;
                y = pos.1;
            }
            TWO_BYTE_OPERATOR_MARK => {
                // flex
                let op2: u8 = s.read().ok_or(CFFError::ReadOutOfBounds)?;
                match op2 {
                    operator::HFLEX => {
                        // dx1 dx2 dy2 dx3 dx4 dx5 dx6

                        if !ctx.has_move_to {
                            return Err(CFFError::MissingMoveTo);
                        }

                        if stack.len() != 7 {
                            return Err(CFFError::InvalidArgumentsStackLength);
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
                    operator::FLEX => {
                        // dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 dx6 dy6 fd

                        if !ctx.has_move_to {
                            return Err(CFFError::MissingMoveTo);
                        }

                        if stack.len() != 13 {
                            return Err(CFFError::InvalidArgumentsStackLength);
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
                    operator::HFLEX1 => {
                        // dx1 dy1 dx2 dy2 dx3 dx4 dx5 dy5 dx6

                        if !ctx.has_move_to {
                            return Err(CFFError::MissingMoveTo);
                        }

                        if stack.len() != 9 {
                            return Err(CFFError::InvalidArgumentsStackLength);
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
                    operator::FLEX1 => {
                        // dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 d6

                        if !ctx.has_move_to {
                            return Err(CFFError::MissingMoveTo);
                        }

                        if stack.len() != 11 {
                            return Err(CFFError::InvalidArgumentsStackLength);
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

                        if f32_abs(dx5 - x) > f32_abs(dy5 - y) {
                            x = dx5 + stack.at(10);
                        } else {
                            y = dy5 + stack.at(10);
                        }

                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    _ => {
                        return Err(CFFError::UnsupportedOperator);
                    }
                }
            }
            operator::VS_INDEX => {
                // |- ivs vsindex (15) |-

                // `vsindex` must precede the first `blend` operator, and may occur only once.
                if ctx.had_blend || ctx.had_vsindex {
                    // TODO: maybe add a custom error
                    return Err(CFFError::InvalidOperator);
                }

                if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                let index = u16::try_num_from(stack.pop())
                    .ok_or(CFFError::InvalidItemVariationDataIndex)?;
                ctx.update_scalars(index)?;

                ctx.had_vsindex = true;

                stack.clear();
            }
            operator::BLEND => {
                // num(0)..num(n-1), delta(0,0)..delta(k-1,0),
                // delta(0,1)..delta(k-1,1) .. delta(0,n-1)..delta(k-1,n-1)
                // n blend (16) val(0)..val(n-1)

                ctx.had_blend = true;

                let n = u16::try_num_from(stack.pop())
                    .ok_or(CFFError::InvalidNumberOfBlendOperands)?;
                let k = ctx.scalars.len();

                let len = usize::from(n) * (usize::from(k) + 1);
                if stack.len() < len {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                let start = stack.len() - len;
                for i in (0..n).rev() {
                    for j in 0..k {
                        let delta = stack.pop();
                        stack.data[start + usize::from(i)] += delta * ctx.scalars.at(k - j - 1);
                    }
                }
            }
            operator::HINT_MASK | operator::COUNTER_MASK => {
                ctx.stems_len += stack.len() as u32 >> 1;
                s.advance(usize::num_from((ctx.stems_len + 7) >> 3));

                // We are ignoring the hint operators.
                stack.clear();
            }
            operator::MOVE_TO => {
                // dx1 dy1

                if stack.len() != 2 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                ctx.has_move_to = true;

                x += stack.at(0);
                y += stack.at(1);
                builder.move_to(x, y);

                stack.clear();
            }
            operator::HORIZONTAL_MOVE_TO => {
                // dx1

                if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                ctx.has_move_to = true;

                x += stack.at(0);
                builder.move_to(x, y);

                stack.clear();
            }
            operator::CURVE_LINE => {
                // {dxa dya dxb dyb dxc dyc}+ dxd dyd

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.len() < 8 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if (stack.len() - 2) % 6 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::LINE_CURVE => {
                // {dxa dya}+ dxb dyb dxc dyc dxd dyd

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.len() < 8 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if (stack.len() - 6).is_odd() {
                    return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::VV_CURVE_TO => {
                // dx1? {dya dxb dyb dyc}+

                let mut i = 0;

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                // The odd argument count indicates an X position.
                if stack.len().is_odd() {
                    x += stack.at(0);
                    i += 1;
                }

                if (stack.len() - i) % 4 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::HH_CURVE_TO => {
                // dy1? {dxa dxb dyb dxc}+

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                let mut i = 0;

                // The odd argument count indicates an Y position.
                if stack.len().is_odd() {
                    y += stack.at(0);
                    i += 1;
                }

                if (stack.len() - i) % 4 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::SHORT_INT => {
                let n = s.read::<i16>().ok_or(CFFError::ReadOutOfBounds)?;
                stack.push(f32::from(n))?;
            }
            operator::CALL_GLOBAL_SUBROUTINE => {
                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                if depth == STACK_LIMIT {
                    return Err(CFFError::NestingLimitReached);
                }

                let subroutine_bias = calc_subroutine_bias(ctx.metadata.global_subrs.len());
                let index = conv_subroutine_index(stack.pop(), subroutine_bias)?;
                let char_string = ctx.metadata.global_subrs.get(index)
                    .ok_or(CFFError::InvalidSubroutineIndex)?;
                let pos = _parse_char_string(ctx, char_string, x, y, stack, depth + 1, builder)?;
                x = pos.0;
                y = pos.1;
            }
            operator::VH_CURVE_TO => {
                // dy1 dx2 dy2 dx3 {dxa dxb dyb dyc dyd dxe dye dxf}* dyf?
                //                 {dya dxb dyb dxc dxd dxe dye dyf}+ dxf?

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.len() < 4 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                stack.reverse();
                while !stack.is_empty() {
                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength);
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
                        return Err(CFFError::InvalidArgumentsStackLength);
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
            operator::HV_CURVE_TO => {
                // dx1 dx2 dy2 dy3 {dya dxb dyb dxc dxd dxe dye dyf}* dxf?
                //                 {dxa dxb dyb dyc dyd dxe dye dxf}+ dyf?

                if !ctx.has_move_to {
                    return Err(CFFError::MissingMoveTo);
                }

                if stack.len() < 4 {
                    return Err(CFFError::InvalidArgumentsStackLength);
                }

                stack.reverse();
                while !stack.is_empty() {
                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength);
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
                        return Err(CFFError::InvalidArgumentsStackLength);
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
            32..=246 => {
                let n = i16::from(op) - 139;
                stack.push(f32::from(n))?;
            }
            247..=250 => {
                let b1: u8 = s.read().ok_or(CFFError::ReadOutOfBounds)?;
                let n = (i16::from(op) - 247) * 256 + i16::from(b1) + 108;
                debug_assert!((108..=1131).contains(&n));
                stack.push(f32::from(n))?;
            }
            251..=254 => {
                let b1: u8 = s.read().ok_or(CFFError::ReadOutOfBounds)?;
                let n = -(i16::from(op) - 251) * 256 - i16::from(b1) - 108;
                debug_assert!((-1131..=-108).contains(&n));
                stack.push(f32::from(n))?;
            }
            operator::FIXED_16_16 => {
                let n = s.read::<Fixed>().ok_or(CFFError::ReadOutOfBounds)?;
                stack.push(n.0)?;
            }
        }
    }

    Ok((x, y))
}

fn parse_index<'a>(s: &mut Stream<'a>) -> Option<DataIndex<'a>> {
    // Unlike in CFF, in CFF2 `count` us u32 and not u16.
    let count: u32 = s.read()?;
    if count != 0 && count != core::u32::MAX {
        parse_index_impl(count, s)
    } else {
        Some(DataIndex::default())
    }
}


struct DictionaryParser<'a> {
    data: &'a [u8],
    // The current offset.
    offset: usize,
    // Offset to the last operands start.
    operands_offset: usize,
    // Actual operands.
    operands: [i32; MAX_OPERANDS_LEN], // 2052B
    // An amount of operands in the `operands` array.
    operands_len: u16,
}

impl<'a> DictionaryParser<'a> {
    #[inline]
    fn new(data: &'a [u8]) -> Self {
        DictionaryParser {
            data,
            offset: 0,
            operands_offset: 0,
            operands: [0; MAX_OPERANDS_LEN],
            operands_len: 0,
        }
    }

    #[inline(never)]
    fn parse_next(&mut self) -> Option<Operator> {
        let mut s = Stream::new_at(self.data, self.offset)?;
        self.operands_offset = self.offset;
        while !s.at_end() {
            let b: u8 = s.read()?;
            if is_dict_one_byte_op(b) {
                let mut operator = u16::from(b);

                // Check that operator is two byte long.
                if b == TWO_BYTE_OPERATOR_MARK {
                    // Use a 1200 'prefix' to make two byte operators more readable.
                    // 12 3 => 1203
                    operator = 1200 + u16::from(s.read::<u8>()?);
                }

                self.offset = s.offset();
                return Some(Operator(operator));
            } else {
                skip_number(b, &mut s)?;
            }
        }

        None
    }

    /// Parses operands of the current operator.
    ///
    /// In the DICT structure, operands are defined before an operator.
    /// So we are trying to find an operator first and the we can actually parse the operands.
    ///
    /// Since this methods is pretty expensive and we do not care about most of the operators,
    /// we can speed up parsing by parsing operands only for required operators.
    ///
    /// We still have to "skip" operands during operators search (see `skip_number()`),
    /// but it's still faster that a naive method.
    fn parse_operands(&mut self) -> Option<()> {
        let mut s = Stream::new(self.data.get(self.operands_offset..)?);
        self.operands_len = 0;
        while !s.at_end() {
            let b: u8 = s.read()?;
            if is_dict_one_byte_op(b) {
                break;
            } else {
                let op = parse_number(b, &mut s)?;
                self.operands[usize::from(self.operands_len)] = op;
                self.operands_len += 1;

                if self.operands_len >= MAX_OPERANDS_LEN as u16 {
                    break;
                }
            }
        }

        Some(())
    }

    #[inline]
    fn operands(&self) -> &[i32] {
        &self.operands[..usize::from(self.operands_len)]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer;
    use writer::TtfType::*;
    use crate::cff::parse_index_impl;

    #[test]
    fn index_data_offsets_len_overflow() {
        let data = writer::convert(&[
            UInt8(4), // offset size
            // other data doesn't matter
        ]);

        assert!(parse_index_impl(std::u32::MAX / 2, &mut Stream::new(&data)).is_none());
    }
}

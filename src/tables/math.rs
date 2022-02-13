//! A [MATH Table](https://docs.microsoft.com/en-us/typography/opentype/spec/math) implementation.

use crate::GlyphId;
use crate::gpos::Device;
use crate::opentype_layout::Coverage;
use crate::parser::{
    Offset16, Stream,
    FromSlice, FromData, Offset,
    LazyArray16, LazyOffsetArray16,
};

/// A [MATH Table](https://docs.microsoft.com/en-us/typography/opentype/spec/math).
#[derive(Clone)]
pub struct Table<'a> {
    /// MathConstants Table, math positioning constants.
    pub math_constants: MathConstants<'a>,
    /// MathGlyphInfo Table, per-glyph positioning information.
    pub math_glyph_info: MathGlyphInfo<'a>,
    /// MathVariants Table, glyph in different sizes and infinitely stretchable shapes.
    pub math_variants: MathVariants<'a>,
}

impl<'a> core::fmt::Debug for Table<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Table {{ ... }}")
    }
}

#[derive(Debug, Copy, Clone)]
struct RawMathValueRecord {
    value: i16,
    device_table_offset: Option<Offset16>,
}

impl FromData for RawMathValueRecord {
    const SIZE: usize = 4;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let value = s.read::<i16>()?;
        let device_table_offset = s.read::<Option<Offset16>>()?;
        Some(RawMathValueRecord { value, device_table_offset })
    }
}

impl RawMathValueRecord {
    fn get(self, data: &[u8]) -> MathValueRecord {
        let device_table = self.device_table_offset
            .and_then(|offset| data.get(offset.to_usize()..))
            .and_then(Device::parse);
        MathValueRecord { value: self.value, device_correction: device_table }
    }
}

/// A [MathValueRecord](https://docs.microsoft.com/en-us/typography/opentype/spec/math#mathvaluerecord).
#[derive(Default, Debug, Copy, Clone)]
pub struct MathValueRecord<'a> {
    /// The X or Y value in design units.
    pub value: i16,
    /// Device corrections for this value.
    pub device_correction: Option<Device<'a>>,
}

impl<'a> MathValueRecord<'a> {
    fn parse(data: &'a [u8], parent: &'a [u8]) -> Option<Self> {
        Some(RawMathValueRecord::parse(data)?.get(parent))
    }
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let major_version = s.read::<u16>()? as u8;
        let minor_version = s.read::<u16>()? as u8;
        // handle the version implicitly, we only recognize version 1.0
        if [major_version, minor_version] != [1, 0] { return None; }
        Some(Table {
            math_constants: s.parse_at_offset16::<MathConstants>(data)?,
            math_glyph_info: s.parse_at_offset16::<MathGlyphInfo>(data)?,
            math_variants: s.parse_at_offset16::<MathVariants>(data)?,
        })
    }
}

/// Constants for math positioning.
#[derive(Clone)]
pub struct MathConstants<'a> {
    data: &'a [u8],
}

impl<'a> core::fmt::Debug for MathConstants<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MathConstants {{ ... }}")
    }
}

impl<'a> FromSlice<'a> for MathConstants<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> { Some(MathConstants { data }) }
}

const SCRIPT_PERCENT_SCALE_DOWN_OFFSET: usize = 0;
const SCRIPT_SCRIPT_PERCENT_SCALE_DOWN_OFFSET: usize = 2;
const DELIMITED_SUB_FORMULA_MIN_HEIGHT_OFFSET: usize = 4;
const DISPLAY_OPERATOR_MIN_HEIGHT_OFFSET: usize = 6;
const MATH_LEADING_OFFSET: usize = 8;
const AXIS_HEIGHT_OFFSET: usize = 12;
const ACCENT_BASE_HEIGHT_OFFSET: usize = 16;
const FLATTENED_ACCENT_BASE_HEIGHT_OFFSET: usize = 20;
const SUBSCRIPT_SHIFT_DOWN_OFFSET: usize = 24;
const SUBSCRIPT_TOP_MAX_OFFSET: usize = 28;
const SUBSCRIPT_BASELINE_DROP_MIN_OFFSET: usize = 32;
const SUPERSCRIPT_SHIFT_UP_OFFSET: usize = 36;
const SUPERSCRIPT_SHIFT_UP_CRAMPED_OFFSET: usize = 40;
const SUPERSCRIPT_BOTTOM_MIN_OFFSET: usize = 44;
const SUPERSCRIPT_BASELINE_DROP_MAX_OFFSET: usize = 48;
const SUB_SUPERSCRIPT_GAP_MIN_OFFSET: usize = 52;
const SUPERSCRIPT_BOTTOM_MAX_WITH_SUBSCRIPT_OFFSET: usize = 56;
const SPACE_AFTER_SCRIPT_OFFSET: usize = 60;
const UPPER_LIMIT_GAP_MIN_OFFSET: usize = 64;
const UPPER_LIMIT_BASELINE_RISE_MIN_OFFSET: usize = 68;
const LOWER_LIMIT_GAP_MIN_OFFSET: usize = 72;
const LOWER_LIMIT_BASELINE_DROP_MIN_OFFSET: usize = 76;
const STACK_TOP_SHIFT_UP_OFFSET: usize = 80;
const STACK_TOP_DISPLAY_STYLE_SHIFT_UP_OFFSET: usize = 84;
const STACK_BOTTOM_SHIFT_DOWN_OFFSET: usize = 88;
const STACK_BOTTOM_DISPLAY_STYLE_SHIFT_DOWN_OFFSET: usize = 92;
const STACK_GAP_MIN_OFFSET: usize = 96;
const STACK_DISPLAY_STYLE_GAP_MIN_OFFSET: usize = 100;
const STRETCH_STACK_TOP_SHIFT_UP_OFFSET: usize = 104;
const STRETCH_STACK_BOTTOM_SHIFT_DOWN_OFFSET: usize = 108;
const STRETCH_STACK_GAP_ABOVE_MIN_OFFSET: usize = 112;
const STRETCH_STACK_GAP_BELOW_MIN_OFFSET: usize = 116;
const FRACTION_NUMERATOR_SHIFT_UP_OFFSET: usize = 120;
const FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP_OFFSET: usize = 124;
const FRACTION_DENOMINATOR_SHIFT_DOWN_OFFSET: usize = 128;
const FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN_OFFSET: usize = 132;
const FRACTION_NUMERATOR_GAP_MIN_OFFSET: usize = 136;
const FRACTION_NUM_DISPLAY_STYLE_GAP_MIN_OFFSET: usize = 140;
const FRACTION_RULE_THICKNESS_OFFSET: usize = 144;
const FRACTION_DENOMINATOR_GAP_MIN_OFFSET: usize = 148;
const FRACTION_DENOM_DISPLAY_STYLE_GAP_MIN_OFFSET: usize = 152;
const SKEWED_FRACTION_HORIZONTAL_GAP_OFFSET: usize = 156;
const SKEWED_FRACTION_VERTICAL_GAP_OFFSET: usize = 160;
const OVERBAR_VERTICAL_GAP_OFFSET: usize = 164;
const OVERBAR_RULE_THICKNESS_OFFSET: usize = 168;
const OVERBAR_EXTRA_ASCENDER_OFFSET: usize = 172;
const UNDERBAR_VERTICAL_GAP_OFFSET: usize = 176;
const UNDERBAR_RULE_THICKNESS_OFFSET: usize = 180;
const UNDERBAR_EXTRA_DESCENDER_OFFSET: usize = 184;
const RADICAL_VERTICAL_GAP_OFFSET: usize = 188;
const RADICAL_DISPLAY_STYLE_VERTICAL_GAP_OFFSET: usize = 192;
const RADICAL_RULE_THICKNESS_OFFSET: usize = 196;
const RADICAL_EXTRA_ASCENDER_OFFSET: usize = 200;
const RADICAL_KERN_BEFORE_DEGREE_OFFSET: usize = 204;
const RADICAL_KERN_AFTER_DEGREE_OFFSET: usize = 208;
const RADICAL_DEGREE_BOTTOM_RAISE_PERCENT_OFFSET: usize = 212;

impl<'a> MathConstants<'a> {
    /// Percentage of scaling down for level 1 superscripts and subscripts. Suggested value: 80%.
    #[inline]
    pub fn script_percent_scale_down(&self) -> i16 {
        Stream::read_at(self.data, SCRIPT_PERCENT_SCALE_DOWN_OFFSET).unwrap_or(0)
    }

    /// Percentage of scaling down for level 2 (scriptScript) superscripts and subscripts.
    /// Suggested value: 60%.
    #[inline]
    pub fn script_script_percent_scale_down(&self) -> i16 {
        Stream::read_at(self.data, SCRIPT_SCRIPT_PERCENT_SCALE_DOWN_OFFSET).unwrap_or(0)
    }

    /// Minimum height required for a delimited expression (contained within parentheses, etc.) to
    /// be treated as a sub-formula. Suggested value: normal line height × 1.5.
    #[inline]
    pub fn delimited_sub_formula_min_height(&self) -> u16 {
        Stream::read_at(self.data, DELIMITED_SUB_FORMULA_MIN_HEIGHT_OFFSET).unwrap_or(0)
    }

    /// Minimum height of n-ary operators (such as integral and summation) for formulas in display
    /// mode (that is, appearing as standalone page elements, not embedded inline within text).
    #[inline]
    pub fn display_operator_min_height(&self) -> u16 {
        Stream::read_at(self.data, DISPLAY_OPERATOR_MIN_HEIGHT_OFFSET).unwrap_or(0)
    }

    /// White space to be left between math formulas to ensure proper line spacing. For example,
    /// for applications that treat line gap as a part of line ascender, formulas with ink going
    /// above (os2.sTypoAscender + os2.sTypoLineGap - MathLeading) or with ink going below
    /// os2.sTypoDescender will result in increasing line height.
    #[inline]
    pub fn math_leading(&self) -> MathValueRecord<'a> {
        self.data.get(MATH_LEADING_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Axis height of the font.
    ///
    /// In math typesetting, the term axis refers to a horizontal reference line used for
    /// positioning elements in a formula. The math axis is similar to but distinct from the
    /// baseline for regular text layout. For example, in a simple equation, a minus symbol or
    /// fraction rule would be on the axis, but a string for a variable name would be set on a
    /// baseline that is offset from the axis. The `axis_height` value determines the amount of
    /// that offset.
    #[inline]
    pub fn axis_height(&self) -> MathValueRecord<'a> {
        self.data.get(AXIS_HEIGHT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum (ink) height of accent base that does not require raising the accents. Suggested:
    /// x‑height of the font (os2.sxHeight) plus any possible overshots.
    #[inline]
    pub fn accent_base_height(&self) -> MathValueRecord<'a> {
        self.data.get(ACCENT_BASE_HEIGHT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum (ink) height of accent base that does not require flattening the accents.
    /// Suggested: cap height of the font (os2.sCapHeight).
    #[inline]
    pub fn flattened_accent_base_height(&self) -> MathValueRecord<'a> {
        self.data.get(FLATTENED_ACCENT_BASE_HEIGHT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// The standard shift down applied to subscript elements. Positive for moving in the downward
    /// direction. Suggested: os2.ySubscriptYOffset.
    #[inline]
    pub fn subscript_shift_down(&self) -> MathValueRecord<'a> {
        self.data.get(SUBSCRIPT_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum allowed height of the (ink) top of subscripts that does not require moving
    /// subscripts further down. Suggested: 4/5 x-height.
    #[inline]
    pub fn subscript_top_max(&self) -> MathValueRecord<'a> {
        self.data.get(SUBSCRIPT_TOP_MAX_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum allowed drop of the baseline of subscripts relative to the (ink) bottom of the
    /// base. Checked for bases that are treated as a box or extended shape. Positive for
    /// subscript baseline dropped below the base bottom.
    #[inline]
    pub fn subscript_baseline_drop_min(&self) -> MathValueRecord<'a> {
        self.data.get(SUBSCRIPT_BASELINE_DROP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to superscript elements. Suggested: os2.ySuperscriptYOffset.
    #[inline]
    pub fn superscript_shift_up(&self) -> MathValueRecord<'a> {
        self.data.get(SUPERSCRIPT_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift of superscripts relative to the base, in cramped style.
    #[inline]
    pub fn superscript_shift_up_cramped(&self) -> MathValueRecord<'a> {
        self.data.get(SUPERSCRIPT_SHIFT_UP_CRAMPED_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum allowed height of the (ink) bottom of superscripts that does not require moving
    /// subscripts further up. Suggested: ¼ x-height.
    #[inline]
    pub fn superscript_bottom_min(&self) -> MathValueRecord<'a> {
        self.data.get(SUPERSCRIPT_BOTTOM_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum allowed drop of the baseline of superscripts relative to the (ink) top of the
    /// base. Checked for bases that are treated as a box or extended shape. Positive for
    /// superscript baseline below the base top.
    #[inline]
    pub fn superscript_baseline_drop_max(&self) -> MathValueRecord<'a> {
        self.data.get(SUPERSCRIPT_BASELINE_DROP_MAX_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the superscript and subscript ink. Suggested: 4 × default rule
    /// thickness.
    #[inline]
    pub fn sub_superscript_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(SUB_SUPERSCRIPT_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// The maximum level to which the (ink) bottom of superscript can be pushed to increase the
    /// gap between superscript and subscript, before subscript starts being moved down.
    /// Suggested: 4/5 x-height.
    #[inline]
    pub fn superscript_bottom_max_with_subscript(&self) -> MathValueRecord<'a> {
        self.data.get(SUPERSCRIPT_BOTTOM_MAX_WITH_SUBSCRIPT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space to be added after each subscript and superscript. Suggested: 0.5 pt for
    /// a 12 pt font. (Note that, in some math layout implementations, a constant value, such as
    /// 0.5 pt, may be used for all text sizes. Some implementations may use a constant ratio of
    /// text size, such as 1/24 of em.)
    #[inline]
    pub fn space_after_script(&self) -> MathValueRecord<'a> {
        self.data.get(SPACE_AFTER_SCRIPT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the (ink) bottom of the upper limit, and the (ink) top of the base
    /// operator.
    #[inline]
    pub fn upper_limit_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(UPPER_LIMIT_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum distance between baseline of upper limit and (ink) top of the base operator.
    #[inline]
    pub fn upper_limit_baseline_rise_min(&self) -> MathValueRecord<'a> {
        self.data.get(UPPER_LIMIT_BASELINE_RISE_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between (ink) top of the lower limit, and (ink) bottom of the base operator.
    #[inline]
    pub fn lower_limit_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(LOWER_LIMIT_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum distance between baseline of the lower limit and (ink) bottom of the base
    /// operator.
    #[inline]
    pub fn lower_limit_baseline_drop_min(&self) -> MathValueRecord<'a> {
        self.data.get(LOWER_LIMIT_BASELINE_DROP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the top element of a stack.
    #[inline]
    pub fn stack_top_shift_up(&self) -> MathValueRecord<'a> {
        self.data.get(STACK_TOP_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the top element of a stack in display style.
    #[inline]
    pub fn stack_top_display_style_shift_up(&self) -> MathValueRecord<'a> {
        self.data.get(STACK_TOP_DISPLAY_STYLE_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the bottom element of a stack. Positive for moving in the
    /// downward direction.
    #[inline]
    pub fn stack_bottom_shift_down(&self) -> MathValueRecord<'a> {
        self.data.get(STACK_BOTTOM_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the bottom element of a stack in display style. Positive
    /// for moving in the downward direction.
    #[inline]
    pub fn stack_bottom_display_style_shift_down(&self) -> MathValueRecord<'a> {
        self.data.get(STACK_BOTTOM_DISPLAY_STYLE_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between (ink) bottom of the top element of a stack, and the (ink) top of the
    /// bottom element. Suggested: 3 × default rule thickness.
    #[inline]
    pub fn stack_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(STACK_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between (ink) bottom of the top element of a stack, and the (ink) top of the
    /// bottom element in display style. Suggested: 7 × default rule thickness.
    #[inline]
    pub fn stack_display_style_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(STACK_DISPLAY_STYLE_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the top element of the stretch stack.
    #[inline]
    pub fn stretch_stack_top_shift_up(&self) -> MathValueRecord<'a> {
        self.data.get(STRETCH_STACK_TOP_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the bottom element of the stretch stack. Positive for
    /// moving in the downward direction.
    #[inline]
    pub fn stretch_stack_bottom_shift_down(&self) -> MathValueRecord<'a> {
        self.data.get(STRETCH_STACK_BOTTOM_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the ink of the stretched element, and the (ink) bottom of the element
    /// above. Suggested: same value as [`MathConstants::upper_limit_gap_min`].
    #[inline]
    pub fn stretch_stack_gap_above_min(&self) -> MathValueRecord<'a> {
        self.data.get(STRETCH_STACK_GAP_ABOVE_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the ink of the stretched element, and the (ink) top of the element
    /// below. Suggested: same value as [`MathConstants::lower_limit_gap_min`].
    #[inline]
    pub fn stretch_stack_gap_below_min(&self) -> MathValueRecord<'a> {
        self.data.get(STRETCH_STACK_GAP_BELOW_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the numerator.
    #[inline]
    pub fn fraction_numerator_shift_up(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_NUMERATOR_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the numerator in display style. Suggested: same value as
    /// [`MathConstants::stack_top_display_style_shift_up`].
    #[inline]
    pub fn fraction_numerator_display_style_shift_up(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the denominator. Positive for moving in the downward direction.
    #[inline]
    pub fn fraction_denominator_shift_down(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_DENOMINATOR_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the denominator in display style. Positive for moving in the
    /// downward direction. Suggested: same value as [`MathConstants::stack_bottom_display_style_shift_down`].
    #[inline]
    pub fn fraction_denominator_display_style_shift_down(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) bottom of the numerator and the ink of the
    /// fraction bar. Suggested: default rule thickness.
    #[inline]
    pub fn fraction_numerator_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_NUMERATOR_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) bottom of the numerator and the ink of the
    /// fraction bar in display style. Suggested: 3 × default rule thickness.
    #[inline]
    pub fn fraction_num_display_style_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_NUM_DISPLAY_STYLE_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of the fraction bar. Suggested: default rule thickness.
    #[inline]
    pub fn fraction_rule_thickness(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) top of the denominator and the ink of the fraction
    /// bar. Suggested: default rule thickness.
    #[inline]
    pub fn fraction_denominator_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_DENOMINATOR_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) top of the denominator and the ink of the fraction
    /// bar in display style. Suggested: 3 × default rule thickness.
    #[inline]
    pub fn fraction_denom_display_style_gap_min(&self) -> MathValueRecord<'a> {
        self.data.get(FRACTION_DENOM_DISPLAY_STYLE_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Horizontal distance between the top and bottom elements of a skewed fraction.
    #[inline]
    pub fn skewed_fraction_horizontal_gap(&self) -> MathValueRecord<'a> {
        self.data.get(SKEWED_FRACTION_HORIZONTAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Vertical distance between the ink of the top and bottom elements of a skewed fraction.
    #[inline]
    pub fn skewed_fraction_vertical_gap(&self) -> MathValueRecord<'a> {
        self.data.get(SKEWED_FRACTION_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Distance between the overbar and the (ink) top of he base. Suggested: 3 × default rule
    /// thickness.
    #[inline]
    pub fn overbar_vertical_gap(&self) -> MathValueRecord<'a> {
        self.data.get(OVERBAR_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of overbar. Suggested: default rule thickness.
    #[inline]
    pub fn overbar_rule_thickness(&self) -> MathValueRecord<'a> {
        self.data.get(OVERBAR_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space reserved above the overbar. Suggested: default rule thickness.
    #[inline]
    pub fn overbar_extra_ascender(&self) -> MathValueRecord<'a> {
        self.data.get(OVERBAR_EXTRA_ASCENDER_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Distance between underbar and (ink) bottom of the base. Suggested: 3 × default rule
    /// thickness.
    #[inline]
    pub fn underbar_vertical_gap(&self) -> MathValueRecord<'a> {
        self.data.get(UNDERBAR_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of underbar. Suggested: default rule thickness.
    #[inline]
    pub fn underbar_rule_thickness(&self) -> MathValueRecord<'a> {
        self.data.get(UNDERBAR_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space reserved below the underbar. Always positive. Suggested: default rule
    /// thickness.
    #[inline]
    pub fn underbar_extra_descender(&self) -> MathValueRecord<'a> {
        self.data.get(UNDERBAR_EXTRA_DESCENDER_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Space between the (ink) top of the expression and the bar over it. Suggested: 1¼ default
    /// rule thickness.
    #[inline]
    pub fn radical_vertical_gap(&self) -> MathValueRecord<'a> {
        self.data.get(RADICAL_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Space between the (ink) top of the expression and the bar over it. Suggested: default rule
    /// thickness + ¼ x-height.
    #[inline]
    pub fn radical_display_style_vertical_gap(&self) -> MathValueRecord<'a> {
        self.data.get(RADICAL_DISPLAY_STYLE_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of the radical rule. This is the thickness of the rule in designed or
    /// constructed radical signs. Suggested: default rule thickness.
    #[inline]
    pub fn radical_rule_thickness(&self) -> MathValueRecord<'a> {
        self.data.get(RADICAL_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space reserved above the radical. Suggested: same value as
    /// [`MathConstants::radical_rule_thickness`].
    #[inline]
    pub fn radical_extra_ascender(&self) -> MathValueRecord<'a> {
        self.data.get(RADICAL_EXTRA_ASCENDER_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra horizontal kern before the degree of a radical, if such is present. Suggested: 5/18
    /// of em.
    #[inline]
    pub fn radical_kern_before_degree(&self) -> MathValueRecord<'a> {
        self.data.get(RADICAL_KERN_BEFORE_DEGREE_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Negative kern after the degree of a radical, if such is present. Suggested: −10/18 of em.
    #[inline]
    pub fn radical_kern_after_degree(&self) -> MathValueRecord<'a> {
        self.data.get(RADICAL_KERN_AFTER_DEGREE_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Height of the bottom of the radical degree, if such is present, in proportion to the
    /// ascender of the radical sign. Suggested: 60%.
    #[inline]
    pub fn radical_degree_bottom_raise_percent(&self) -> i16 {
        Stream::read_at(self.data, RADICAL_DEGREE_BOTTOM_RAISE_PERCENT_OFFSET).unwrap_or(0)
    }
}

/// The MathGlyphInfo table contains positioning information that is defined on per-glyph basis.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MathGlyphInfo<'a> {
    /// Contains information on italics correction values.
    pub math_italics_correction_info: MathValueTable<'a>,
    /// Contains horizontal positions for attaching mathematical accents.
    pub math_top_accent_attachment: MathValueTable<'a>,
    /// The glyphs covered by this table are to be considered extended shapes.
    pub extended_shape_coverage: Option<Coverage<'a>>,
    /// Provides per-glyph information for mathematical kerning.
    pub math_kern_info: Option<MathKernInfo<'a>>,
}

impl<'a> FromSlice<'a> for MathGlyphInfo<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(MathGlyphInfo {
            math_italics_correction_info: s.parse_at_offset16::<MathValueTable>(data)?,
            math_top_accent_attachment: s.parse_at_offset16::<MathValueTable>(data)?,
            extended_shape_coverage: s.parse_at_offset16::<Coverage>(data),
            math_kern_info: s.parse_at_offset16::<MathKernInfo>(data),
        })
    }
}

/// Mapping from glyph to values.
#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
#[derive(Clone)]
pub struct MathValueTable<'a> {
    pub coverage: Coverage<'a>,
    values: MathValueArray<'a>,
}

impl<'a> MathValueTable<'a> {
    /// Checks that glyph is present.
    pub fn contains(&self, glyph: GlyphId) -> bool {
        self.coverage.contains(glyph)
    }

    /// Returns the [`MathValueRecord`] of the glyph or `None` if it is not covered.
    pub fn get(&self, glyph: GlyphId) -> Option<MathValueRecord<'a>> {
        let index = self.coverage.get(glyph)?;
        self.values.get(index)
    }
}

impl<'a> FromSlice<'a> for MathValueTable<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let coverage = s.parse_at_offset16::<Coverage>(data)?;
        let count = s.read::<u16>()?;
        let entries = s.read_array16(count)?;
        Some(MathValueTable { coverage, values: MathValueArray { entries, data } })
    }
}

#[derive(Clone)]
struct MathValueArray<'a> {
    entries: LazyArray16<'a, RawMathValueRecord>,
    data: &'a [u8], // for locating device tables
}

impl<'a> MathValueArray<'a> {
    fn get(&self, index: u16) -> Option<MathValueRecord<'a>> {
        Some(self.entries.get(index)?.get(self.data))
    }
}

impl<'a> Stream<'a> {
    fn parse_at_offset16<T: FromSlice<'a>>(&mut self, base: &'a [u8]) -> Option<T> {
        self.read_at_offset16(base).and_then(T::parse)
    }
}

/// See <https://docs.microsoft.com/en-us/typography/opentype/spec/math#mathkerninfo-table>.
#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
#[derive(Clone)]
pub struct MathKernInfo<'a> {
    pub coverage: Coverage<'a>,
    values: MathKernInfoArray<'a>,
}

impl<'a> MathKernInfo<'a> {
    /// Checks that glyph is present.
    pub fn contains(&self, glyph: GlyphId) -> bool {
        self.coverage.contains(glyph)
    }

    /// Returns the [`MathKernInfoRecord`] of the glyph or `None` if it is not covered.
    pub fn get(&self, glyph: GlyphId) -> Option<MathKernInfoRecord<'a>> {
        let index = self.coverage.get(glyph)?;
        self.values.get(index)
    }
}

impl<'a> FromSlice<'a> for MathKernInfo<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let coverage = s.parse_at_offset16::<Coverage>(data)?;
        let count = s.read::<u16>()?;
        let entries = s.read_array16(count)?;
        Some(MathKernInfo { coverage, values: MathKernInfoArray { entries, data } })
    }
}

#[derive(Clone)]
struct MathKernInfoArray<'a> {
    entries: LazyArray16<'a, RawMathKernInfoRecord>,
    data: &'a [u8], // for locating device tables
}

impl<'a> MathKernInfoArray<'a> {
    fn get(&self, index: u16) -> Option<MathKernInfoRecord<'a>> {
        Some(self.entries.get(index)?.get(self.data))
    }
}

#[derive(Default, Debug, Copy, Clone)]
struct RawMathKernInfoRecord {
    top_right: Option<Offset16>,
    top_left: Option<Offset16>,
    bottom_right: Option<Offset16>,
    bottom_left: Option<Offset16>,
}

impl FromData for RawMathKernInfoRecord {
    const SIZE: usize = 8;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(RawMathKernInfoRecord {
            top_right: s.read::<Option<Offset16>>()?,
            top_left: s.read::<Option<Offset16>>()?,
            bottom_right: s.read::<Option<Offset16>>()?,
            bottom_left: s.read::<Option<Offset16>>()?,
        })
    }
}

impl RawMathKernInfoRecord {
    fn get<'a>(&self, data: &'a [u8]) -> MathKernInfoRecord<'a> {
        let parse_field = |offset: Option<Offset16>| offset
            .and_then(|offset| data.get(offset.to_usize()..))
            .and_then(MathKern::parse);
        MathKernInfoRecord {
            top_right: parse_field(self.top_right),
            top_left: parse_field(self.top_left),
            bottom_right: parse_field(self.bottom_right),
            bottom_left: parse_field(self.bottom_left),
        }
    }
}

/// Each MathKernInfoRecord points to up to four kern tables for each of the corners around the
/// glyph. If no kern table is provided for a corner, a kerning amount of zero is assumed.
#[allow(missing_debug_implementations)]
#[allow(missing_docs)]
#[derive(Clone)]
pub struct MathKernInfoRecord<'a> {
    pub top_right: Option<MathKern<'a>>,
    pub top_left: Option<MathKern<'a>>,
    pub bottom_right: Option<MathKern<'a>>,
    pub bottom_left: Option<MathKern<'a>>,
}

/// The MathKern table provides kerning amounts for different heights in a glyph’s vertical extent.
/// An array of kerning values is provided, each of which applies to a height range. A corresponding
/// array of heights indicate the transition points between consecutive ranges.
///
/// Correction heights for each glyph are relative to the glyph baseline, with positive height
/// values above the baseline, and negative height values below the baseline. The correctionHeights
/// array is sorted in increasing order, from lowest to highest.
///
/// The kerning value corresponding to a particular height is determined by finding two consecutive
/// entries in the correctionHeight array such that the given height is greater than or equal to
/// the first entry and less than the second entry. The index of the second entry is used to look
/// up a kerning value in the kernValues array. If the given height is less than the first entry
/// in the correctionHeights array, the first kerning value (index 0) is used. For a height that is
/// greater than or equal to the last entry in the correctionHeights array, the last entry is used.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MathKern<'a> {
    data: &'a [u8],
    correction_height: LazyArray16<'a, RawMathValueRecord>,
    kern_values: LazyArray16<'a, RawMathValueRecord>,
}

impl<'a> MathKern<'a> {
    /// Number of heights as split points.
    pub fn height_count(&self) -> u16 { self.correction_height.len() }
    /// Number of kern values. It is always greater than number of heights by one.
    pub fn kern_count(&self) -> u16 { self.kern_values.len() }
    /// Get the height value at some specific index.
    pub fn get_height(&self, index: u16) -> Option<MathValueRecord> {
        Some(self.correction_height.get(index)?.get(self.data))
    }
    /// Get the kern value at some specific index.
    pub fn get_kern_value(&self, index: u16) -> Option<MathValueRecord> {
        Some(self.kern_values.get(index)?.get(self.data))
    }
}

impl<'a> FromSlice<'a> for MathKern<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let correction_height = s.read_array16(count)?;
        let kern_values = s.read_array16(count + 1)?;
        Some(MathKern { data, correction_height, kern_values })
    }
}

/// Used for selecting glyph variants at correct size, or constructing stretchable shapes.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MathVariants<'a> {
    /// Minimum overlap of connecting glyphs during glyph construction, in design units.
    ///
    /// `min_connector_overlap` defines by how much two glyphs need to overlap with each other when
    /// used to construct a larger shape. Each glyph to be used as a building block in constructing
    /// extended shapes will have a straight part at either or both ends. This connector part is
    /// used to connect that glyph to other glyphs in the assembly. These connectors need to overlap
    /// to compensate for rounding errors and hinting corrections at a lower resolution. The
    /// `min_connector_overlap` value tells how much overlap is necessary for this particular font.
    pub min_connector_overlap: u16,
    /// Coverage table for shapes growing in the vertical direction.
    pub vert_glyph_coverage: Coverage<'a>,
    /// Coverage table for shapes growing in the horizontal direction.
    pub horiz_glyph_coverage: Coverage<'a>,
    /// For shapes growing in the vertical direction.
    pub vert_glyph_construction: LazyOffsetArray16<'a, MathGlyphConstruction<'a>>,
    /// For shapes growing in the horizontal direction.
    pub horiz_glyph_construction: LazyOffsetArray16<'a, MathGlyphConstruction<'a>>,
}

impl<'a> FromSlice<'a> for MathVariants<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let min_connector_overlap = s.read::<u16>()?;
        let vert_glyph_coverage = s.parse_at_offset16::<Coverage>(data)?;
        let horiz_glyph_coverage = s.parse_at_offset16::<Coverage>(data)?;
        let vert_glyph_count = s.read::<u16>()?;
        let horiz_glyph_count = s.read::<u16>()?;
        let vert_glyph_construction_offsets = s.read_array16(vert_glyph_count)?;
        let horiz_glyph_construction_offsets = s.read_array16(horiz_glyph_count)?;
        Some(MathVariants {
            min_connector_overlap,
            vert_glyph_coverage,
            horiz_glyph_coverage,
            vert_glyph_construction: LazyOffsetArray16::new(data, vert_glyph_construction_offsets),
            horiz_glyph_construction: LazyOffsetArray16::new(data, horiz_glyph_construction_offsets),
        })
    }
}

/// The `MathGlyphConstruction` table provides information on finding or assembling extended
/// variants for one particular glyph. It can be used for shapes that grow in either horizontal or
/// vertical directions.
///
/// Note that it is quite possible that both the [`GlyphAssembly`] table and some variants are
/// defined for a particular glyph. For example, the font may provide several variants of curly
/// braces with different sizes, and also a general mechanism for constructing larger versions of
/// curly braces by stacking parts found in the glyph set. First, an attempt is made to find glyph
/// among provided variants. If the required size is larger than any of the glyph variants
/// provided, however, then the general mechanism can be employed to typeset the curly braces as a
/// glyph assembly.
#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MathGlyphConstruction<'a> {
    pub glyph_assembly: Option<GlyphAssembly<'a>>,
    pub variants: LazyArray16<'a, MathGlyphVariantRecord>,
}

impl<'a> FromSlice<'a> for MathGlyphConstruction<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let glyph_assembly = s.parse_at_offset16::<GlyphAssembly>(data);
        let variant_count = s.read::<u16>()?;
        let variants = s.read_array16(variant_count)?;
        Some(MathGlyphConstruction { glyph_assembly, variants })
    }
}

/// Description of math glyph variants.
#[derive(Debug, Default, Copy, Clone)]
pub struct MathGlyphVariantRecord {
    /// Glyph ID for the variant.
    pub variant_glyph: GlyphId,
    /// Advance width/height, in design units, of the variant, in the direction of requested glyph
    /// extension.
    pub advance_measurement: u16,
}

impl FromData for MathGlyphVariantRecord {
    const SIZE: usize = 4;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let variant_glyph = s.read::<GlyphId>()?;
        let advance_measurement = s.read::<u16>()?;
        Some(MathGlyphVariantRecord { variant_glyph, advance_measurement })
    }
}

/// How the shape for this glyph can be assembled from parts found in the glyph set of the font.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct GlyphAssembly<'a> {
    /// Italics correction of this GlyphAssembly. Should not depend on the assembly size.
    pub italics_correction: MathValueRecord<'a>,
    /// Array of part records, from left to right (for assemblies that extend horizontally) or
    /// bottom to top (for assemblies that extend vertically).
    pub glyph_parts: LazyArray16<'a, GlyphPartRecord>,
}

impl<'a> FromSlice<'a> for GlyphAssembly<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let italics_correction = s.read::<RawMathValueRecord>()?.get(data);
        let glyph_part_count = s.read::<u16>()?;
        let glyph_parts = s.read_array16(glyph_part_count)?;
        Some(GlyphAssembly { italics_correction, glyph_parts })
    }
}

/// Description of glyph parts to be assembled.
#[allow(missing_debug_implementations)]
#[derive(Copy, Clone)]
pub struct GlyphPartRecord {
    /// Glyph ID for the part.
    pub glyph_id: GlyphId,
    /// Lengths of the connectors on the start of the glyph.
    pub start_connector_length: u16,
    /// Lengths of the connectors on the end of the glyph.
    pub end_connector_length: u16,
    /// The full advance of the part. It is also used to determine the measurement of the result
    /// by using the following formula:
    ///
    /// _Size of Assembly = Offset of the Last Part + Full Advance of the Last Part_
    pub full_advance: u16,
    /// Part qualifiers.
    pub part_flags: PartFlags,
}

impl FromData for GlyphPartRecord {
    const SIZE: usize = 10;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(GlyphPartRecord {
            glyph_id: s.read::<GlyphId>()?,
            start_connector_length: s.read::<u16>()?,
            end_connector_length: s.read::<u16>()?,
            full_advance: s.read::<u16>()?,
            part_flags: s.read::<PartFlags>()?,
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct PartFlags(pub u16);

impl PartFlags {
    /// If set, the part can be skipped or repeated.
    #[inline]
    pub fn extender(self) -> bool { self.0 & 0x0001 != 0 }
}

impl FromData for PartFlags {
    const SIZE: usize = 2;

    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(PartFlags)
    }
}

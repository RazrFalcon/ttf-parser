//! A [MATH Table](https://docs.microsoft.com/en-us/typography/opentype/spec/math) implementation.

use crate::gpos::Device;
use crate::parser::{Offset, Offset16, Stream};

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Table<'a> {
    pub major_version: u8,
    pub minor_version: u8,
    pub math_constants: Option<MathConstants<'a>>,
    // pub math_glyph_info: MathGlyphInfo<'a>,
    // pub math_variants: MathVariants<'a>,
}

#[allow(missing_debug_implementations)]
#[derive(Default, Debug, Copy, Clone)]
pub struct MathValueRecord<'a> {
    value: i16,
    device_table: Option<Device<'a>>,
}

impl<'a> MathValueRecord<'a> {
    pub fn get(&self) -> i16 { self.value }
    fn parse(data: &'a [u8], parent: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let value = s.read::<i16>()?;
        let device_table = s.read::<Option<Offset16>>()?
            .and_then(|offset| parent.get(offset.to_usize()..))
            .and_then(Device::parse);
        Some(MathValueRecord { value, device_table })
    }
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let major_version = s.read::<u16>()? as u8;
        let minor_version = s.read::<u16>()? as u8;
        let math_constants = s.read::<Option<Offset16>>()?
            .and_then(|offset| data.get(offset.to_usize()..))
            .map(|data| MathConstants { data });
        let math_glyph_info_offset = s.read::<u16>()?;
        let math_variants_offset = s.read::<u16>()?;
        Some(Table {
            major_version,
            minor_version,
            math_constants,
            // math_glyph_info: todo!(),
            // math_variants: todo!(),
        })
    }
}

/// The MathConstants table defines a number of constants required to properly position elements of
/// mathematical formulas. These constants belong to several groups of semantically-related values,
/// such as values for positioning of accents, positioning of superscripts and subscripts, and
/// positioning of elements of fractions. The table also contains general-use constants that may
/// affect all parts of the formula, such as axis height and math leading. Note that most of the
/// constants deal with aspects of vertical positioning.
///
/// In most cases, values in the MathConstants table are assumed to be positive. For example, for
/// descenders and shift-down values a positive constant signifies movement in a downwards
/// direction. Most values in the MathConstants table are represented by a MathValueRecord, which
/// allows the font designer to supply device corrections to those values when necessary.
///
/// For values that pertain to layout interaction between a base and dependent elements (e.g.,
/// superscripts or limits), the specific value used is taken from the font associated with the
/// base, and the size of the value is relative to the size of the base.
///
/// The following naming convention are used for fields in the MathConstants table:
///
/// - Height — Specifies a distance from the main baseline.
/// - Kern — Represents a fixed amount of empty space to be introduced.
/// - Gap — Represents an amount of empty space that may need to be increased to meet certain criteria.
/// - Drop and Rise — Specifies the relationship between measurements of two elements to be
///   positioned relative to each other (but not necessarily in a stack-like manner) that must meet
///   certain criteria. For a Drop, one of the positioned elements has to be moved down to satisfy
///   those criteria; for a Rise, the movement is upwards.
/// - Shift — Defines a vertical shift applied to an element sitting on a baseline. Note that the
///   value is an amount of adjustment to the position of an element, not the resulting distance
///   from the baseline or other reference line.
/// - Dist — Defines a distance between baselines of two elements.
///
/// The descriptions for several fields refer to default rule thickness. Layout engines control how
/// rules are drawn and how their thickness is set. It is recommended that rules have the same
/// thickness as a minus sign, low line, or a similar font value such as `OS/2.yStrikeoutSize`. For
/// fields that are described in reference to default rule thickness, one of these should be assumed.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MathConstants<'a> {
    data: &'a [u8],
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
    pub fn math_leading(&self) -> MathValueRecord {
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
    pub fn axis_height(&self) -> MathValueRecord {
        self.data.get(AXIS_HEIGHT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum (ink) height of accent base that does not require raising the accents. Suggested:
    /// x‑height of the font (os2.sxHeight) plus any possible overshots.
    #[inline]
    pub fn accent_base_height(&self) -> MathValueRecord {
        self.data.get(ACCENT_BASE_HEIGHT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum (ink) height of accent base that does not require flattening the accents.
    /// Suggested: cap height of the font (os2.sCapHeight).
    #[inline]
    pub fn flattened_accent_base_height(&self) -> MathValueRecord {
        self.data.get(FLATTENED_ACCENT_BASE_HEIGHT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// The standard shift down applied to subscript elements. Positive for moving in the downward
    /// direction. Suggested: os2.ySubscriptYOffset.
    #[inline]
    pub fn subscript_shift_down(&self) -> MathValueRecord {
        self.data.get(SUBSCRIPT_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum allowed height of the (ink) top of subscripts that does not require moving
    /// subscripts further down. Suggested: 4/5 x-height.
    #[inline]
    pub fn subscript_top_max(&self) -> MathValueRecord {
        self.data.get(SUBSCRIPT_TOP_MAX_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum allowed drop of the baseline of subscripts relative to the (ink) bottom of the
    /// base. Checked for bases that are treated as a box or extended shape. Positive for
    /// subscript baseline dropped below the base bottom.
    #[inline]
    pub fn subscript_baseline_drop_min(&self) -> MathValueRecord {
        self.data.get(SUBSCRIPT_BASELINE_DROP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to superscript elements. Suggested: os2.ySuperscriptYOffset.
    #[inline]
    pub fn superscript_shift_up(&self) -> MathValueRecord {
        self.data.get(SUPERSCRIPT_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift of superscripts relative to the base, in cramped style.
    #[inline]
    pub fn superscript_shift_up_cramped(&self) -> MathValueRecord {
        self.data.get(SUPERSCRIPT_SHIFT_UP_CRAMPED_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum allowed height of the (ink) bottom of superscripts that does not require moving
    /// subscripts further up. Suggested: ¼ x-height.
    #[inline]
    pub fn superscript_bottom_min(&self) -> MathValueRecord {
        self.data.get(SUPERSCRIPT_BOTTOM_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Maximum allowed drop of the baseline of superscripts relative to the (ink) top of the
    /// base. Checked for bases that are treated as a box or extended shape. Positive for
    /// superscript baseline below the base top.
    #[inline]
    pub fn superscript_baseline_drop_max(&self) -> MathValueRecord {
        self.data.get(SUPERSCRIPT_BASELINE_DROP_MAX_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the superscript and subscript ink. Suggested: 4 × default rule
    /// thickness.
    #[inline]
    pub fn sub_superscript_gap_min(&self) -> MathValueRecord {
        self.data.get(SUB_SUPERSCRIPT_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// The maximum level to which the (ink) bottom of superscript can be pushed to increase the
    /// gap between superscript and subscript, before subscript starts being moved down.
    /// Suggested: 4/5 x-height.
    #[inline]
    pub fn superscript_bottom_max_with_subscript(&self) -> MathValueRecord {
        self.data.get(SUPERSCRIPT_BOTTOM_MAX_WITH_SUBSCRIPT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space to be added after each subscript and superscript. Suggested: 0.5 pt for
    /// a 12 pt font. (Note that, in some math layout implementations, a constant value, such as
    /// 0.5 pt, may be used for all text sizes. Some implementations may use a constant ratio of
    /// text size, such as 1/24 of em.)
    #[inline]
    pub fn space_after_script(&self) -> MathValueRecord {
        self.data.get(SPACE_AFTER_SCRIPT_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the (ink) bottom of the upper limit, and the (ink) top of the base
    /// operator.
    #[inline]
    pub fn upper_limit_gap_min(&self) -> MathValueRecord {
        self.data.get(UPPER_LIMIT_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum distance between baseline of upper limit and (ink) top of the base operator.
    #[inline]
    pub fn upper_limit_baseline_rise_min(&self) -> MathValueRecord {
        self.data.get(UPPER_LIMIT_BASELINE_RISE_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between (ink) top of the lower limit, and (ink) bottom of the base operator.
    #[inline]
    pub fn lower_limit_gap_min(&self) -> MathValueRecord {
        self.data.get(LOWER_LIMIT_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum distance between baseline of the lower limit and (ink) bottom of the base
    /// operator.
    #[inline]
    pub fn lower_limit_baseline_drop_min(&self) -> MathValueRecord {
        self.data.get(LOWER_LIMIT_BASELINE_DROP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the top element of a stack.
    #[inline]
    pub fn stack_top_shift_up(&self) -> MathValueRecord {
        self.data.get(STACK_TOP_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the top element of a stack in display style.
    #[inline]
    pub fn stack_top_display_style_shift_up(&self) -> MathValueRecord {
        self.data.get(STACK_TOP_DISPLAY_STYLE_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the bottom element of a stack. Positive for moving in the
    /// downward direction.
    #[inline]
    pub fn stack_bottom_shift_down(&self) -> MathValueRecord {
        self.data.get(STACK_BOTTOM_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the bottom element of a stack in display style. Positive
    /// for moving in the downward direction.
    #[inline]
    pub fn stack_bottom_display_style_shift_down(&self) -> MathValueRecord {
        self.data.get(STACK_BOTTOM_DISPLAY_STYLE_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between (ink) bottom of the top element of a stack, and the (ink) top of the
    /// bottom element. Suggested: 3 × default rule thickness.
    #[inline]
    pub fn stack_gap_min(&self) -> MathValueRecord {
        self.data.get(STACK_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between (ink) bottom of the top element of a stack, and the (ink) top of the
    /// bottom element in display style. Suggested: 7 × default rule thickness.
    #[inline]
    pub fn stack_display_style_gap_min(&self) -> MathValueRecord {
        self.data.get(STACK_DISPLAY_STYLE_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the top element of the stretch stack.
    #[inline]
    pub fn stretch_stack_top_shift_up(&self) -> MathValueRecord {
        self.data.get(STRETCH_STACK_TOP_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the bottom element of the stretch stack. Positive for
    /// moving in the downward direction.
    #[inline]
    pub fn stretch_stack_bottom_shift_down(&self) -> MathValueRecord {
        self.data.get(STRETCH_STACK_BOTTOM_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the ink of the stretched element, and the (ink) bottom of the element
    /// above. Suggested: same value as upperLimitGapMin.
    #[inline]
    pub fn stretch_stack_gap_above_min(&self) -> MathValueRecord {
        self.data.get(STRETCH_STACK_GAP_ABOVE_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum gap between the ink of the stretched element, and the (ink) top of the element
    /// below. Suggested: same value as lowerLimitGapMin.
    #[inline]
    pub fn stretch_stack_gap_below_min(&self) -> MathValueRecord {
        self.data.get(STRETCH_STACK_GAP_BELOW_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the numerator.
    #[inline]
    pub fn fraction_numerator_shift_up(&self) -> MathValueRecord {
        self.data.get(FRACTION_NUMERATOR_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift up applied to the numerator in display style. Suggested: same value as
    /// stackTopDisplayStyleShiftUp.
    #[inline]
    pub fn fraction_numerator_display_style_shift_up(&self) -> MathValueRecord {
        self.data.get(FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the denominator. Positive for moving in the downward
    /// direction.
    #[inline]
    pub fn fraction_denominator_shift_down(&self) -> MathValueRecord {
        self.data.get(FRACTION_DENOMINATOR_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Standard shift down applied to the denominator in display style. Positive for moving in
    /// the downward direction. Suggested: same value as stackBottomDisplayStyleShiftDown.
    #[inline]
    pub fn fraction_denominator_display_style_shift_down(&self) -> MathValueRecord {
        self.data.get(FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) bottom of the numerator and the ink of the
    /// fraction bar. Suggested: default rule thickness.
    #[inline]
    pub fn fraction_numerator_gap_min(&self) -> MathValueRecord {
        self.data.get(FRACTION_NUMERATOR_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) bottom of the numerator and the ink of the
    /// fraction bar in display style. Suggested: 3 × default rule thickness.
    #[inline]
    pub fn fraction_num_display_style_gap_min(&self) -> MathValueRecord {
        self.data.get(FRACTION_NUM_DISPLAY_STYLE_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of the fraction bar. Suggested: default rule thickness.
    #[inline]
    pub fn fraction_rule_thickness(&self) -> MathValueRecord {
        self.data.get(FRACTION_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) top of the denominator and the ink of the fraction
    /// bar. Suggested: default rule thickness.
    #[inline]
    pub fn fraction_denominator_gap_min(&self) -> MathValueRecord {
        self.data.get(FRACTION_DENOMINATOR_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Minimum tolerated gap between the (ink) top of the denominator and the ink of the fraction
    /// bar in display style. Suggested: 3 × default rule thickness.
    #[inline]
    pub fn fraction_denom_display_style_gap_min(&self) -> MathValueRecord {
        self.data.get(FRACTION_DENOM_DISPLAY_STYLE_GAP_MIN_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Horizontal distance between the top and bottom elements of a skewed fraction.
    #[inline]
    pub fn skewed_fraction_horizontal_gap(&self) -> MathValueRecord {
        self.data.get(SKEWED_FRACTION_HORIZONTAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Vertical distance between the ink of the top and bottom elements of a skewed fraction.
    #[inline]
    pub fn skewed_fraction_vertical_gap(&self) -> MathValueRecord {
        self.data.get(SKEWED_FRACTION_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Distance between the overbar and the (ink) top of he base. Suggested: 3 × default rule
    /// thickness.
    #[inline]
    pub fn overbar_vertical_gap(&self) -> MathValueRecord {
        self.data.get(OVERBAR_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of overbar. Suggested: default rule thickness.
    #[inline]
    pub fn overbar_rule_thickness(&self) -> MathValueRecord {
        self.data.get(OVERBAR_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space reserved above the overbar. Suggested: default rule thickness.
    #[inline]
    pub fn overbar_extra_ascender(&self) -> MathValueRecord {
        self.data.get(OVERBAR_EXTRA_ASCENDER_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Distance between underbar and (ink) bottom of the base. Suggested: 3 × default rule
    /// thickness.
    #[inline]
    pub fn underbar_vertical_gap(&self) -> MathValueRecord {
        self.data.get(UNDERBAR_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of underbar. Suggested: default rule thickness.
    #[inline]
    pub fn underbar_rule_thickness(&self) -> MathValueRecord {
        self.data.get(UNDERBAR_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space reserved below the underbar. Always positive. Suggested: default rule
    /// thickness.
    #[inline]
    pub fn underbar_extra_descender(&self) -> MathValueRecord {
        self.data.get(UNDERBAR_EXTRA_DESCENDER_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Space between the (ink) top of the expression and the bar over it. Suggested: 1¼ default
    /// rule thickness.
    #[inline]
    pub fn radical_vertical_gap(&self) -> MathValueRecord {
        self.data.get(RADICAL_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Space between the (ink) top of the expression and the bar over it. Suggested: default rule
    /// thickness + ¼ x-height.
    #[inline]
    pub fn radical_display_style_vertical_gap(&self) -> MathValueRecord {
        self.data.get(RADICAL_DISPLAY_STYLE_VERTICAL_GAP_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Thickness of the radical rule. This is the thickness of the rule in designed or
    /// constructed radical signs. Suggested: default rule thickness.
    #[inline]
    pub fn radical_rule_thickness(&self) -> MathValueRecord {
        self.data.get(RADICAL_RULE_THICKNESS_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra white space reserved above the radical. Suggested: same value as
    /// radicalRuleThickness.
    #[inline]
    pub fn radical_extra_ascender(&self) -> MathValueRecord {
        self.data.get(RADICAL_EXTRA_ASCENDER_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Extra horizontal kern before the degree of a radical, if such is present. Suggested: 5/18
    /// of em.
    #[inline]
    pub fn radical_kern_before_degree(&self) -> MathValueRecord {
        self.data.get(RADICAL_KERN_BEFORE_DEGREE_OFFSET..)
            .and_then(|data| MathValueRecord::parse(data, self.data))
            .unwrap_or_default()
    }

    /// Negative kern after the degree of a radical, if such is present. Suggested: −10/18 of em.
    #[inline]
    pub fn radical_kern_after_degree(&self) -> MathValueRecord {
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

// pub struct MathGlyphInfo<'a> {}

// pub struct MathVariants<'a> {}

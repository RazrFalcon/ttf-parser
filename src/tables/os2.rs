// https://docs.microsoft.com/en-us/typography/opentype/spec/os2

use crate::LineMetrics;
use crate::parser::Stream;


const WEIGHT_CLASS_OFFSET: usize = 4;
const WIDTH_CLASS_OFFSET: usize = 6;
const Y_SUBSCRIPT_X_SIZE_OFFSET: usize = 10;
const Y_SUPERSCRIPT_X_SIZE_OFFSET: usize = 18;
const Y_STRIKEOUT_SIZE_OFFSET: usize = 26;
const Y_STRIKEOUT_POSITION_OFFSET: usize = 28;
const FS_SELECTION_OFFSET: usize = 62;
const TYPO_ASCENDER_OFFSET: usize = 68;
const TYPO_DESCENDER_OFFSET: usize = 70;
const TYPO_LINE_GAP_OFFSET: usize = 72;
const WIN_ASCENT: usize = 74;
const WIN_DESCENT: usize = 76;
const X_HEIGHT_OFFSET: usize = 86;
const CAP_HEIGHT_OFFSET: usize = 88;


/// A font [weight](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#usweightclass).
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
#[allow(missing_docs)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Other(u16),
}

impl Weight {
    /// Returns a numeric representation of a weight.
    #[inline]
    pub fn to_number(self) -> u16 {
        match self {
            Weight::Thin        => 100,
            Weight::ExtraLight  => 200,
            Weight::Light       => 300,
            Weight::Normal      => 400,
            Weight::Medium      => 500,
            Weight::SemiBold    => 600,
            Weight::Bold        => 700,
            Weight::ExtraBold   => 800,
            Weight::Black       => 900,
            Weight::Other(n)    => n,
        }
    }
}

impl From<u16> for Weight {
    #[inline]
    fn from(value: u16) -> Self {
        match value {
            100 => Weight::Thin,
            200 => Weight::ExtraLight,
            300 => Weight::Light,
            400 => Weight::Normal,
            500 => Weight::Medium,
            600 => Weight::SemiBold,
            700 => Weight::Bold,
            800 => Weight::ExtraBold,
            900 => Weight::Black,
            _   => Weight::Other(value),
        }
    }
}

impl Default for Weight {
    #[inline]
    fn default() -> Self {
        Weight::Normal
    }
}


/// A font [width](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#uswidthclass).
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
#[allow(missing_docs)]
pub enum Width {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl Width {
    /// Returns a numeric representation of a width.
    #[inline]
    pub fn to_number(self) -> u16 {
        match self {
            Width::UltraCondensed   => 1,
            Width::ExtraCondensed   => 2,
            Width::Condensed        => 3,
            Width::SemiCondensed    => 4,
            Width::Normal           => 5,
            Width::SemiExpanded     => 6,
            Width::Expanded         => 7,
            Width::ExtraExpanded    => 8,
            Width::UltraExpanded    => 9,
        }
    }
}

impl Default for Width {
    #[inline]
    fn default() -> Self {
        Width::Normal
    }
}


/// A script metrics used by subscript and superscript.
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct ScriptMetrics {
    /// Horizontal font size.
    pub x_size: i16,

    /// Vertical font size.
    pub y_size: i16,

    /// X offset.
    pub x_offset: i16,

    /// Y offset.
    pub y_offset: i16,
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/os2#fsselection
#[derive(Clone, Copy)]
struct SelectionFlags(u16);

impl SelectionFlags {
    #[inline] fn italic(self) -> bool { self.0 & (1 << 0) != 0 }
    #[inline] fn bold(self) -> bool { self.0 & (1 << 5) != 0 }
    #[inline] fn regular(self) -> bool { self.0 & (1 << 6) != 0 }
    #[inline] fn use_typo_metrics(self) -> bool { self.0 & (1 << 7) != 0 }
    #[inline] fn oblique(self) -> bool { self.0 & (1 << 9) != 0 }
}


#[derive(Clone, Copy)]
pub(crate) struct Table<'a> {
    version: u8,
    data: &'a [u8],
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version: u16 = s.read()?;

        let table_len = match version {
            0 => 78,
            1 => 86,
            2 => 96,
            3 => 96,
            4 => 96,
            5 => 100,
            _ => return None,
        };

        if data.len() != table_len {
            return None;
        }

        Some(Table {
            version: version as u8,
            data,
        })
    }

    #[inline]
    pub fn weight(&self) -> Weight {
        Weight::from(Stream::read_at::<u16>(self.data, WEIGHT_CLASS_OFFSET).unwrap_or(0))
    }

    #[inline]
    pub fn width(&self) -> Width {
        match Stream::read_at::<u16>(self.data, WIDTH_CLASS_OFFSET).unwrap_or(0) {
            1 => Width::UltraCondensed,
            2 => Width::ExtraCondensed,
            3 => Width::Condensed,
            4 => Width::SemiCondensed,
            5 => Width::Normal,
            6 => Width::SemiExpanded,
            7 => Width::Expanded,
            8 => Width::ExtraExpanded,
            9 => Width::UltraExpanded,
            _ => Width::Normal,
        }
    }

    #[inline]
    pub fn subscript_metrics(&self) -> ScriptMetrics {
        let mut s = Stream::new_at(self.data, Y_SUBSCRIPT_X_SIZE_OFFSET).unwrap_or_default();
        ScriptMetrics {
            x_size: s.read::<i16>().unwrap_or(0),
            y_size: s.read::<i16>().unwrap_or(0),
            x_offset: s.read::<i16>().unwrap_or(0),
            y_offset: s.read::<i16>().unwrap_or(0),
        }
    }

    #[inline]
    pub fn superscript_metrics(&self) -> ScriptMetrics {
        let mut s = Stream::new_at(self.data, Y_SUPERSCRIPT_X_SIZE_OFFSET).unwrap_or_default();
        ScriptMetrics {
            x_size: s.read::<i16>().unwrap_or(0),
            y_size: s.read::<i16>().unwrap_or(0),
            x_offset: s.read::<i16>().unwrap_or(0),
            y_offset: s.read::<i16>().unwrap_or(0),
        }
    }

    #[inline]
    pub fn strikeout_metrics(&self) -> LineMetrics {
        LineMetrics {
            thickness: Stream::read_at::<i16>(self.data, Y_STRIKEOUT_SIZE_OFFSET).unwrap_or(0),
            position: Stream::read_at::<i16>(self.data, Y_STRIKEOUT_POSITION_OFFSET).unwrap_or(0),
        }
    }

    #[inline]
    fn fs_selection(&self) -> u16 {
        Stream::read_at::<u16>(self.data, FS_SELECTION_OFFSET).unwrap_or(0)
    }

    #[inline]
    pub fn is_regular(&self) -> bool {
        SelectionFlags(self.fs_selection()).regular()
    }

    #[inline]
    pub fn is_italic(&self) -> bool {
        SelectionFlags(self.fs_selection()).italic()
    }

    #[inline]
    pub fn is_bold(&self) -> bool {
        SelectionFlags(self.fs_selection()).bold()
    }

    #[inline]
    pub fn is_oblique(&self) -> bool {
        if self.version < 4 {
            false
        } else {
            SelectionFlags(self.fs_selection()).oblique()
        }
    }

    #[inline]
    pub fn is_use_typo_metrics(&self) -> bool {
        if self.version < 4 {
            false
        } else {
            SelectionFlags(self.fs_selection()).use_typo_metrics()
        }
    }

    #[inline]
    pub fn typo_ascender(&self) -> i16 {
        Stream::read_at::<i16>(self.data, TYPO_ASCENDER_OFFSET).unwrap_or(0)
    }

    #[inline]
    pub fn typo_descender(&self) -> i16 {
        Stream::read_at::<i16>(self.data, TYPO_DESCENDER_OFFSET).unwrap_or(0)
    }

    #[inline]
    pub fn typo_line_gap(&self) -> i16 {
        Stream::read_at::<i16>(self.data, TYPO_LINE_GAP_OFFSET).unwrap_or(0)
    }

    #[inline]
    pub fn windows_ascender(&self) -> i16 {
        Stream::read_at::<i16>(self.data, WIN_ASCENT).unwrap_or(0)
    }

    #[inline]
    pub fn windows_descender(&self) -> i16 {
        // Should be negated.
        -Stream::read_at::<i16>(self.data, WIN_DESCENT).unwrap_or(0)
    }

    #[inline]
    pub fn x_height(&self) -> Option<i16> {
        if self.version < 2 {
            None
        } else {
            Stream::read_at::<i16>(self.data, X_HEIGHT_OFFSET)
        }
    }

    #[inline]
    pub fn cap_height(&self) -> Option<i16> {
        if self.version < 2 {
            None
        } else {
            Stream::read_at::<i16>(self.data, CAP_HEIGHT_OFFSET)
        }
    }
}

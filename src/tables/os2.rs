// https://docs.microsoft.com/en-us/typography/opentype/spec/os2

use crate::LineMetrics;
use crate::parser::Stream;
use crate::raw::os_2 as raw;


/// A font [weight](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#usweightclass).
#[derive(Clone, Copy, PartialEq, Debug)]
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
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
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
#[derive(Clone, Copy, PartialEq, Debug)]
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
    pub table: raw::Table<'a>,
}

impl<'a> core::ops::Deref for Table<'a> {
    type Target = raw::Table<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.table
    }
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
            table: raw::Table::new(&data[0..78]),
        })
    }

    #[inline]
    pub fn weight(&self) -> Weight {
        Weight::from(self.us_weight_class())
    }

    #[inline]
    pub fn width(&self) -> Width {
        match self.us_width_class() {
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
    pub(crate) fn is_use_typo_metrics(&self) -> bool {
        if self.version < 4 {
            false
        } else {
            SelectionFlags(self.fs_selection()).use_typo_metrics()
        }
    }

    #[inline]
    pub fn x_height(&self) -> Option<i16> {
        if self.version < 2 {
            None
        } else {
            // We cannot use SafeStream here, because x height is an optional data.
            Stream::read_at(self.data, raw::SX_HEIGHT_OFFSET)
        }
    }

    #[inline]
    pub fn strikeout_metrics(&self) -> LineMetrics {
        LineMetrics {
            thickness: self.y_strikeout_size(),
            position: self.y_strikeout_position(),
        }
    }

    #[inline]
    pub fn subscript_metrics(&self) -> ScriptMetrics {
        ScriptMetrics {
            x_size: self.y_subscript_x_size(),
            y_size: self.y_subscript_y_size(),
            x_offset: self.y_subscript_x_offset(),
            y_offset: self.y_subscript_y_offset(),
        }
    }

    #[inline]
    pub fn superscript_metrics(&self) -> ScriptMetrics {
        ScriptMetrics {
            x_size: self.y_superscript_x_size(),
            y_size: self.y_superscript_y_size(),
            x_offset: self.y_superscript_x_offset(),
            y_offset: self.y_superscript_y_offset(),
        }
    }
}

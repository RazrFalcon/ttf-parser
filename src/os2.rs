//! The [OS/2](https://docs.microsoft.com/en-us/typography/opentype/spec/os2)
//! table parsing primitives.

use std::convert::TryFrom;

use crate::stream::{Stream, FromData};
use crate::LineMetrics;


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
    Other(u16), // TODO: do we really need this?
}

impl Weight {
    /// Returns a numeric representation of a weight.
    pub fn to_number(&self) -> u16 {
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
            Weight::Other(n)    => *n,
        }
    }
}

impl From<u16> for Weight {
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
    pub fn to_number(&self) -> u16 {
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

impl TryFrom<u16> for Width {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Width::UltraCondensed),
            2 => Ok(Width::ExtraCondensed),
            3 => Ok(Width::Condensed),
            4 => Ok(Width::SemiCondensed),
            5 => Ok(Width::Normal),
            6 => Ok(Width::SemiExpanded),
            7 => Ok(Width::Expanded),
            8 => Ok(Width::ExtraExpanded),
            9 => Ok(Width::UltraExpanded),
            _ => Err(()),
        }
    }
}

impl Default for Width {
    fn default() -> Self {
        Width::Normal
    }
}


/// A script metrics used by subscript and superscript.
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

impl FromData for ScriptMetrics {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        ScriptMetrics {
            x_size: s.read_i16(),
            y_size: s.read_i16(),
            x_offset: s.read_i16(),
            y_offset: s.read_i16(),
        }
    }
}


/// Handle to a OS/2 table.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Table<'a> {
    pub(crate) data: &'a [u8],
}

impl<'a> Table<'a> {
    /// Parses font's weight.
    pub fn weight(&self) -> Weight {
        const US_WEIGHT_CLASS_OFFSET: usize = 4;

        let n: u16 = Stream::read_at(self.data, US_WEIGHT_CLASS_OFFSET);
        Weight::from(n)
    }

    /// Parses font's width.
    ///
    /// Returns `None` when value is out of 1..9 range.
    pub fn width(&self) -> Option<Width> {
        const US_WIDTH_CLASS_OFFSET: usize = 6;

        let n: u16 = Stream::read_at(self.data, US_WIDTH_CLASS_OFFSET);
        Width::try_from(n).ok()
    }

    /// Checks that font is marked as *Regular*.
    pub fn is_regular(&self) -> bool {
        const REGULAR_FLAG: u16 = 6;
        (self.get_fs_selection() >> REGULAR_FLAG) & 1 == 1
    }

    /// Checks that font is marked as *Italic*.
    pub fn is_italic(&self) -> bool {
        const ITALIC_FLAG: u16 = 0;
        (self.get_fs_selection() >> ITALIC_FLAG) & 1 == 1
    }

    /// Checks that font is marked as *Bold*.
    pub fn is_bold(&self) -> bool {
        const BOLD_FLAG: u16 = 5;
        (self.get_fs_selection() >> BOLD_FLAG) & 1 == 1
    }

    /// Checks that font is marked as *Oblique*.
    ///
    /// Available only in OS/2 table version >= 4.
    pub fn is_oblique(&self) -> Option<bool> {
        const VERSION_OFFSET: usize = 0;

        let version: u16 = Stream::read_at(self.data, VERSION_OFFSET);

        if version < 4 {
            return None;
        }

        const OBLIQUE_FLAG: u16 = 9;
        Some((self.get_fs_selection() >> OBLIQUE_FLAG) & 1 == 1)
    }

    fn get_fs_selection(&self) -> u16 {
        const FS_SELECTION_OFFSET: usize = 62;
        Stream::read_at(self.data, FS_SELECTION_OFFSET)
    }

    /// Parses font's X height.
    ///
    /// Available only in OS/2 table version >= 2.
    pub fn x_height(&self) -> Option<i16> {
        const VERSION_OFFSET: usize = 0;
        const SX_HEIGHT_OFFSET: usize = 86;

        let version: u16 = Stream::read_at(self.data, VERSION_OFFSET);
        if version < 2 {
            return None;
        }

        Some(Stream::read_at(self.data, SX_HEIGHT_OFFSET))
    }

    /// Parses font's strikeout metrics.
    pub fn strikeout_metrics(&self) -> LineMetrics {
        const Y_STRIKEOUT_SIZE_OFFSET: usize = 26;
        const Y_STRIKEOUT_POSITION_OFFSET: usize = 28;

        LineMetrics {
            position:  Stream::read_at(self.data, Y_STRIKEOUT_SIZE_OFFSET),
            thickness: Stream::read_at(self.data, Y_STRIKEOUT_POSITION_OFFSET),
        }
    }

    /// Parses font's subscript metrics.
    pub fn subscript_metrics(&self) -> ScriptMetrics {
        const Y_SUBSCRIPT_XSIZE_OFFSET: usize = 10;
        Stream::read_at(self.data, Y_SUBSCRIPT_XSIZE_OFFSET)
    }

    /// Parses font's superscript metrics.
    pub fn superscript_metrics(&self) -> ScriptMetrics {
        const Y_SUPERSCRIPT_XSIZE_OFFSET: usize = 18;
        Stream::read_at(self.data, Y_SUPERSCRIPT_XSIZE_OFFSET)
    }
}

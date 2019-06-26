use core::convert::TryFrom;

use crate::parser::{Stream, FromData};
use crate::{Font, TableName, LineMetrics, Result, Error};


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
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
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
            _ => Err(Error::InvalidFontWidth(value)),
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
            x_size: s.read(),
            y_size: s.read(),
            x_offset: s.read(),
            y_offset: s.read(),
        }
    }
}


impl<'a> Font<'a> {
    /// Parses font's weight.
    pub fn weight(&self) -> Result<Weight> {
        const US_WEIGHT_CLASS_OFFSET: usize = 4;

        let data = self.table_data(TableName::WindowsMetrics)?;
        let n: u16 = Stream::read_at(data, US_WEIGHT_CLASS_OFFSET);
        Ok(Weight::from(n))
    }

    /// Parses font's width.
    pub fn width(&self) -> Result<Width> {
        const US_WIDTH_CLASS_OFFSET: usize = 6;

        let data = self.table_data(TableName::WindowsMetrics)?;
        let n: u16 = Stream::read_at(data, US_WIDTH_CLASS_OFFSET);
        Width::try_from(n)
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
    pub fn is_oblique(&self) -> bool {
        const VERSION_OFFSET: usize = 0;

        let data = match self.table_data(TableName::WindowsMetrics) {
            Ok(data) => data,
            Err(_) => return false,
        };

        let version: u16 = Stream::read_at(data, VERSION_OFFSET);
        if version < 4 {
            return false;
        }

        const OBLIQUE_FLAG: u16 = 9;
        (self.get_fs_selection() >> OBLIQUE_FLAG) & 1 == 1
    }

    fn get_fs_selection(&self) -> u16 {
        const FS_SELECTION_OFFSET: usize = 62;
        match self.table_data(TableName::WindowsMetrics) {
            Ok(data) => Stream::read_at(data, FS_SELECTION_OFFSET),
            Err(_) => 0,
        }
    }

    /// Parses font's X height.
    ///
    /// Available only in OS/2 table version >= 2.
    pub fn x_height(&self) -> Option<i16> {
        const VERSION_OFFSET: usize = 0;
        const SX_HEIGHT_OFFSET: usize = 86;

        let data = self.table_data(TableName::WindowsMetrics).ok()?;
        let version: u16 = Stream::read_at(data, VERSION_OFFSET);
        if version < 2 {
            return None;
        }

        Some(Stream::read_at(data, SX_HEIGHT_OFFSET))
    }

    /// Parses font's strikeout metrics.
    pub fn strikeout_metrics(&self) -> Result<LineMetrics> {
        const Y_STRIKEOUT_SIZE_OFFSET: usize = 26;
        const Y_STRIKEOUT_POSITION_OFFSET: usize = 28;

        let data = self.table_data(TableName::WindowsMetrics)?;
        Ok(LineMetrics {
            position:  Stream::read_at(data, Y_STRIKEOUT_POSITION_OFFSET),
            thickness: Stream::read_at(data, Y_STRIKEOUT_SIZE_OFFSET),
        })
    }

    /// Parses font's subscript metrics.
    pub fn subscript_metrics(&self) -> Result<ScriptMetrics> {
        const Y_SUBSCRIPT_XSIZE_OFFSET: usize = 10;
        let data = self.table_data(TableName::WindowsMetrics)?;
        Ok(Stream::read_at(data, Y_SUBSCRIPT_XSIZE_OFFSET))
    }

    /// Parses font's superscript metrics.
    pub fn superscript_metrics(&self) -> Result<ScriptMetrics> {
        const Y_SUPERSCRIPT_XSIZE_OFFSET: usize = 18;
        let data = self.table_data(TableName::WindowsMetrics)?;
        Ok(Stream::read_at(data, Y_SUPERSCRIPT_XSIZE_OFFSET))
    }
}

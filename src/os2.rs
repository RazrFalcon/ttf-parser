//! The [OS/2](https://docs.microsoft.com/en-us/typography/opentype/spec/os2)
//! table parsing primitives.

use std::convert::TryFrom;

use crate::stream::Stream;
use crate::{Font, LineMetrics};


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


impl<'a> Font<'a> {
    /// Parses font's weight set in the OS/2 table.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn weight(&self) -> Option<Weight> {
        const US_WEIGHT_CLASS_OFFSET: usize = 4;

        let n: u16 = Stream::read_at(&self.data[self.os_2?.range()], US_WEIGHT_CLASS_OFFSET);
        Some(Weight::from(n))
    }

    /// Parses font's width set in the OS/2 table.
    ///
    /// Returns `None` when OS/2 table is not present.
    /// Or when value is out of 1..9 range.
    pub fn width(&self) -> Option<Width> {
        const US_WIDTH_CLASS_OFFSET: usize = 6;

        let n: u16 = Stream::read_at(&self.data[self.os_2?.range()], US_WIDTH_CLASS_OFFSET);
        Width::try_from(n).ok()
    }

    /// Checks that font is marked as *Regular* in the OS/2 table.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn is_regular(&self) -> Option<bool> {
        const REGULAR_FLAG: u16 = 6;
        Some((self.get_fs_selection()? >> REGULAR_FLAG) & 1 == 1)
    }

    /// Checks that font is marked as *Italic* in the OS/2 table.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn is_italic(&self) -> Option<bool> {
        const ITALIC_FLAG: u16 = 0;
        Some((self.get_fs_selection()? >> ITALIC_FLAG) & 1 == 1)
    }

    /// Checks that font is marked as *Bold* in the OS/2 table.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn is_bold(&self) -> Option<bool> {
        const BOLD_FLAG: u16 = 5;
        Some((self.get_fs_selection()? >> BOLD_FLAG) & 1 == 1)
    }

    /// Checks that font is marked as *Oblique* in the OS/2 table.
    ///
    /// Available only in OS/2 table version >= 4.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn is_oblique(&self) -> Option<bool> {
        const VERSION_OFFSET: usize = 0;

        let version: u16 = Stream::read_at(&self.data[self.os_2?.range()], VERSION_OFFSET);

        if version < 4 {
            return None;
        }

        const OBLIQUE_FLAG: u16 = 9;
        Some((self.get_fs_selection()? >> OBLIQUE_FLAG) & 1 == 1)
    }

    fn get_fs_selection(&self) -> Option<u16> {
        const FS_SELECTION_OFFSET: usize = 62;
        let n: u16 = Stream::read_at(&self.data[self.os_2?.range()], FS_SELECTION_OFFSET);
        Some(n)
    }

    /// Parses font's X height set in the OS/2 table.
    ///
    /// Available only in OS/2 table version >= 2.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn x_height(&self) -> Option<i16> {
        const VERSION_OFFSET: usize = 0;
        const SX_HEIGHT_OFFSET: usize = 86;

        let data = &self.data[self.os_2?.range()];
        let version: u16 = Stream::read_at(data, VERSION_OFFSET);
        if version < 2 {
            return None;
        }

        let n: i16 = Stream::read_at(data, SX_HEIGHT_OFFSET);
        Some(n)
    }

    /// Parses font's strikeout metrics set in the OS/2 table.
    ///
    /// Returns `None` when OS/2 table is not present.
    pub fn strikeout(&self) -> Option<LineMetrics> {
        const Y_STRIKEOUT_SIZE_OFFSET: usize = 26;
        const Y_STRIKEOUT_POSITION_OFFSET: usize = 28;

        let data = &self.data[self.os_2?.range()];
        Some(LineMetrics {
            position:  Stream::read_at(data, Y_STRIKEOUT_SIZE_OFFSET),
            thickness: Stream::read_at(data, Y_STRIKEOUT_POSITION_OFFSET),
        })
    }
}

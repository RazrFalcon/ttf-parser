// https://docs.microsoft.com/en-us/typography/opentype/spec/os2

use crate::parser::Stream;
use crate::{Font, LineMetrics};
use crate::raw::os_2 as raw;


macro_rules! try_opt_or {
    ($value:expr, $ret:expr) => {
        match $value {
            Some(v) => v,
            None => return $ret,
        }
    };
}


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

impl Default for Width {
    #[inline]
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


// We already checked that OS/2 table has a valid length,
// so it's safe to use `SafeStream`.

impl<'a> Font<'a> {
    /// Parses font's weight.
    ///
    /// Returns `Weight::Normal` when OS/2 table is not present.
    #[inline]
    pub fn weight(&self) -> Weight {
        let table = try_opt_or!(self.os_2, Weight::default());
        Weight::from(table.us_weight_class())
    }

    /// Parses font's width.
    ///
    /// Returns `Width::Normal` when OS/2 table is not present or when value is invalid.
    #[inline]
    pub fn width(&self) -> Width {
        let table = try_opt_or!(self.os_2, Width::default());
        match table.us_width_class() {
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

    /// Checks that font is marked as *Regular*.
    ///
    /// Returns `false` when OS/2 table is not present.
    #[inline]
    pub fn is_regular(&self) -> bool {
        const REGULAR_FLAG: u16 = 6;
        self.get_fs_selection(REGULAR_FLAG)
    }

    /// Checks that font is marked as *Italic*.
    ///
    /// Returns `false` when OS/2 table is not present.
    #[inline]
    pub fn is_italic(&self) -> bool {
        const ITALIC_FLAG: u16 = 0;
        self.get_fs_selection(ITALIC_FLAG)
    }

    /// Checks that font is marked as *Bold*.
    ///
    /// Returns `false` when OS/2 table is not present.
    #[inline]
    pub fn is_bold(&self) -> bool {
        const BOLD_FLAG: u16 = 5;
        self.get_fs_selection(BOLD_FLAG)
    }

    /// Checks that font is marked as *Oblique*.
    ///
    /// Returns `None` when OS/2 table is not present or when its version is < 4.
    #[inline]
    pub fn is_oblique(&self) -> bool {
        let table = try_opt_or!(self.os_2, false);
        if table.version() < 4 {
            return false;
        }

        const OBLIQUE_FLAG: u16 = 9;
        self.get_fs_selection(OBLIQUE_FLAG)
    }

    #[inline]
    pub(crate) fn is_use_typo_metrics(&self) -> bool {
        let table = try_opt_or!(self.os_2, false);
        if table.version() < 4 {
            return false;
        }

        const USE_TYPO_METRICS: u16 = 7;
        self.get_fs_selection(USE_TYPO_METRICS)
    }

    #[inline]
    fn get_fs_selection(&self, bit: u16) -> bool {
        let table = try_opt_or!(self.os_2, false);
        (table.fs_selection() >> bit) & 1 == 1
    }

    /// Parses font's X height.
    ///
    /// Returns `None` when OS/2 table is not present or when its version is < 2.
    #[inline]
    pub fn x_height(&self) -> Option<i16> {
        let table = self.os_2?;
        if table.version() < 2 {
            return None;
        }

        // We cannot use SafeStream here, because X height is an optional data.
        Stream::read_at(table.data, raw::SX_HEIGHT_OFFSET).ok()
    }

    /// Parses font's strikeout metrics.
    ///
    /// Returns `None` when OS/2 table is not present.
    #[inline]
    pub fn strikeout_metrics(&self) -> Option<LineMetrics> {
        let table = self.os_2?;
        Some(LineMetrics {
            thickness: table.y_strikeout_size(),
            position: table.y_strikeout_position(),
        })
    }

    /// Parses font's subscript metrics.
    ///
    /// Returns `None` when OS/2 table is not present.
    #[inline]
    pub fn subscript_metrics(&self) -> Option<ScriptMetrics> {
        let table = self.os_2?;
        Some(ScriptMetrics {
            x_size: table.y_subscript_x_size(),
            y_size: table.y_subscript_y_size(),
            x_offset: table.y_subscript_x_offset(),
            y_offset: table.y_subscript_y_offset(),
        })
    }

    /// Parses font's superscript metrics.
    ///
    /// Returns `None` when OS/2 table is not present.
    #[inline]
    pub fn superscript_metrics(&self) -> Option<ScriptMetrics> {
        let table = self.os_2?;
        Some(ScriptMetrics {
            x_size: table.y_superscript_x_size(),
            y_size: table.y_superscript_y_size(),
            x_offset: table.y_superscript_x_offset(),
            y_offset: table.y_superscript_y_offset(),
        })
    }
}

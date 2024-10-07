//! A [PANOSE classification](
//! https://learn.microsoft.com/en-us/typography/opentype/spec/os2#panose) implementation.
//!
//! The underlying specification is located at: <https://monotype.github.io/panose/pan1.htm>

use crate::{parser::Stream, FromData};

/// The type of diagonal stems and letterform termination used by a font face.
///
/// <https://monotype.github.io/panose/pan2.htm#_Toc380547277>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ArmStyle {
    AnyFit = 0,
    NoFit = 1,
    StraightHorizontal = 2,
    StraightWedge = 3,
    StraightVertical = 4,
    StraightSingleSerif = 5,
    StraightDoubleSerif = 6,
    NonStraightHorizontal = 7,
    NonStraightWedge = 8,
    NonStraightVertical = 9,
    NonStraightSingleSerif = 10,
    NonStraightDoubleSerif = 11,
}

impl FromData for ArmStyle {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::StraightHorizontal),
            3 => Some(Self::StraightWedge),
            4 => Some(Self::StraightVertical),
            5 => Some(Self::StraightSingleSerif),
            6 => Some(Self::StraightDoubleSerif),
            7 => Some(Self::NonStraightHorizontal),
            8 => Some(Self::NonStraightWedge),
            9 => Some(Self::NonStraightVertical),
            10 => Some(Self::NonStraightSingleSerif),
            11 => Some(Self::NonStraightDoubleSerif),
            _ => None,
        }
    }
}

/// The ratio between the thickest and thinnest parts of the uppercase letter "O".
///
/// <https://monotype.github.io/panose/pan2.htm#_Toc380547263>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Contrast {
    AnyFit = 0,
    NoFit = 1,
    None = 2,
    VeryLow = 3,
    Low = 4,
    MediumLow = 5,
    Medium = 6,
    MediumHigh = 7,
    High = 8,
    VeryHigh = 9,
    // The following are Latin Symbol specific?
    HorizontalLow = 10,
    HorizontalMedium = 11,
    HorizontalHigh = 12,
    Broken = 13,
}

impl FromData for Contrast {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::None),
            3 => Some(Self::VeryLow),
            4 => Some(Self::Low),
            5 => Some(Self::MediumLow),
            6 => Some(Self::Medium),
            7 => Some(Self::MediumHigh),
            8 => Some(Self::High),
            9 => Some(Self::VeryHigh),
            10 => Some(Self::HorizontalLow),
            11 => Some(Self::HorizontalMedium),
            12 => Some(Self::HorizontalHigh),
            13 => Some(Self::Broken),
            _ => None,
        }
    }
}

/// The "look and feel" of a [`LatinDecorative`](Panose::LatinDecorative) face.
///
/// <https://monotype.github.io/panose/pan4.htm#_Toc380547360>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DecorationClass {
    AnyFit = 0,
    NoFit = 1,
    Derivative = 2,
    NonStandardTopology = 3,
    NonStandardElements = 4,
    NonStandardAspect = 5,
    Initials = 6,
    Cartoon = 7,
    PictureStems = 8,
    Ornamented = 9,
    TextAndBackground = 10,
    Collage = 11,
    Montage = 12,
}

impl FromData for DecorationClass {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::Derivative),
            3 => Some(Self::NonStandardTopology),
            4 => Some(Self::NonStandardElements),
            5 => Some(Self::NonStandardAspect),
            6 => Some(Self::Initials),
            7 => Some(Self::Cartoon),
            8 => Some(Self::PictureStems),
            9 => Some(Self::Ornamented),
            10 => Some(Self::TextAndBackground),
            11 => Some(Self::Collage),
            12 => Some(Self::Montage),
            _ => None,
        }
    }
}

/// The ratio between the width and the height of a [`LatinDecorative`](Panose::LatinDecorative)
/// face.
///
/// <https://monotype.github.io/panose/pan4.htm#_Toc380547371>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DecorativeAspectRatio {
    AnyFit = 0,
    NoFit = 1,
    SuperCondensed = 2,
    VeryCondensed = 3,
    Condensed = 4,
    Normal = 5,
    Extended = 6,
    VeryExtended = 7,
    SuperExtended = 8,
    Monospaced = 9,
}

impl FromData for DecorativeAspectRatio {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::SuperCondensed),
            3 => Some(Self::VeryCondensed),
            4 => Some(Self::Condensed),
            5 => Some(Self::Normal),
            6 => Some(Self::Extended),
            7 => Some(Self::VeryExtended),
            8 => Some(Self::SuperExtended),
            9 => Some(Self::Monospaced),
            _ => None,
        }
    }
}

/// The range of characters available in a [`LatinDecorative`](Panose::LatinDecorative) face.
///
/// <https://monotype.github.io/panose/pan4.htm#_Toc380547399>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DecorativeRange {
    AnyFit = 0,
    NoFit = 1,
    Extended = 2,
    Literals = 3,
    NoLowerCase = 4,
    SmallCaps = 5,
}

impl FromData for DecorativeRange {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::Extended),
            3 => Some(Self::Literals),
            4 => Some(Self::NoLowerCase),
            5 => Some(Self::SmallCaps),
            _ => None,
        }
    }
}

/// So-called unusual characteristics of a [`LatinDecorative`](Panose::LatinDecorative) face.
///
/// <https://monotype.github.io/panose/pan4.htm#_Toc380547395>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DecorativeTopology {
    AnyFit = 0,
    NoFit = 1,
    Standard = 2,
    Square = 3,
    MultipleSegment = 4,
    DecoMidlines = 5,
    UnevenWeighting = 6,
    DiverseArms = 7,
    DiverseForms = 8,
    LombardicForms = 9,
    UppercaseInLowercase = 10,
    Implied = 11,
    Horseshoe = 12,
    Cursive = 13,
    Blackletter = 14,
    SwashVariance = 15,
}

impl FromData for DecorativeTopology {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::Standard),
            3 => Some(Self::Square),
            4 => Some(Self::MultipleSegment),
            5 => Some(Self::DecoMidlines),
            6 => Some(Self::UnevenWeighting),
            7 => Some(Self::DiverseArms),
            8 => Some(Self::DiverseForms),
            9 => Some(Self::LombardicForms),
            10 => Some(Self::UppercaseInLowercase),
            11 => Some(Self::Implied),
            12 => Some(Self::Horseshoe),
            13 => Some(Self::Cursive),
            14 => Some(Self::Blackletter),
            15 => Some(Self::SwashVariance),
            _ => None,
        }
    }
}

/// Describes the fill and outline of a [`LatinDecorative`](Panose::LatinDecorative) face.
///
/// <https://monotype.github.io/panose/pan4.htm#_Toc380547387>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DecorativeTreatment {
    AnyFit = 0,
    NoFit = 1,
    None = 2,
    White = 3,
    Pattern = 4,
    Complex = 5,
    Shaped = 6,
    Distressed = 7,
}

impl FromData for DecorativeTreatment {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::None),
            3 => Some(Self::White),
            4 => Some(Self::Pattern),
            5 => Some(Self::Complex),
            6 => Some(Self::Shaped),
            7 => Some(Self::Distressed),
            _ => None,
        }
    }
}

/// Treatment of the ends of characters in a [`LatinHandwritten`](Panose::LatinHandwritten) face.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547346>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Finial {
    AnyFit = 0,
    NoFit = 1,
    NoFinialNoLoop = 2,
    NoFinialClosedLoop = 3,
    NoFinialOpenLoop = 4,
    SharpFinialNoLoop = 5,
    SharpFinialClosedLoop = 6,
    SharpFinialOpenLoop = 7,
    TaperedFinialNoLoop = 8,
    TaperedFinialClosedLoop = 9,
    TaperedFinialOpenLoop = 10,
    RoundFinialNoLoop = 11,
    RoundFinialClosedLoop = 12,
    RoundFinialOpenLoop = 13,
}

impl FromData for Finial {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::NoFinialNoLoop),
            3 => Some(Self::NoFinialClosedLoop),
            4 => Some(Self::NoFinialOpenLoop),
            5 => Some(Self::SharpFinialNoLoop),
            6 => Some(Self::SharpFinialClosedLoop),
            7 => Some(Self::SharpFinialOpenLoop),
            8 => Some(Self::TaperedFinialNoLoop),
            9 => Some(Self::TaperedFinialClosedLoop),
            10 => Some(Self::TaperedFinialOpenLoop),
            11 => Some(Self::RoundFinialNoLoop),
            12 => Some(Self::RoundFinialClosedLoop),
            13 => Some(Self::RoundFinialOpenLoop),
            _ => None,
        }
    }
}

/// Classification of a [`LatinHandwritten`](Panose::LatinHandwritten) based on the tails of
/// connecting strokes and the slope of the verticals.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547340>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum HandForm {
    AnyFit = 0,
    NoFit = 1,
    UprightNoWrap = 2,
    UprightSomeWrap = 3,
    UprightMoreWrap = 4,
    UprightExtremeWrap = 5,
    ObliqueNoWrap = 6,
    ObliqueSomeWrap = 7,
    ObliqueMoreWrap = 8,
    ObliqueExtremeWrap = 9,
    ExaggeratedNoWrap = 10,
    ExaggeratedSomeWrap = 11,
    ExaggeratedMoreWrap = 12,
    ExaggeratedExtremeWrap = 13,
}

impl FromData for HandForm {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::UprightNoWrap),
            3 => Some(Self::UprightSomeWrap),
            4 => Some(Self::UprightMoreWrap),
            5 => Some(Self::UprightExtremeWrap),
            6 => Some(Self::ObliqueNoWrap),
            7 => Some(Self::ObliqueSomeWrap),
            8 => Some(Self::ObliqueMoreWrap),
            9 => Some(Self::ObliqueExtremeWrap),
            10 => Some(Self::ExaggeratedNoWrap),
            11 => Some(Self::ExaggeratedSomeWrap),
            12 => Some(Self::ExaggeratedMoreWrap),
            13 => Some(Self::ExaggeratedExtremeWrap),
            _ => None,
        }
    }
}

/// Ratio between the width and the height of the capital "O" character in a
/// [`LatinHandwritten`](Panose::LatinHandwritten) face.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547324>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum HandwrittenAspectRatio {
    AnyFit = 0,
    NoFit = 1,
    VeryCondensed = 2,
    Condensed = 3,
    Normal = 4,
    Expanded = 5,
    VeryExpanded = 6,
}

impl FromData for HandwrittenAspectRatio {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::VeryCondensed),
            3 => Some(Self::Condensed),
            4 => Some(Self::Normal),
            5 => Some(Self::Expanded),
            6 => Some(Self::VeryExpanded),
            _ => None,
        }
    }
}

/// Broad classification of a [`LatinHandwritten`](Panose::LatinHandwritten) face based on
/// letterforms and connections between letters.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547336>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum HandwrittenTopology {
    AnyFit = 0,
    NoFit = 1,
    RomanDisconnected = 2,
    RomanTrailing = 3,
    RomanConnected = 4,
    CursiveDisconnected = 5,
    CursiveTrailing = 6,
    CursiveConnected = 7,
    BlackletterDisconnected = 8,
    BlackletterTrailing = 9,
    BlackletterConnected = 10,
}

impl FromData for HandwrittenTopology {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::RomanDisconnected),
            3 => Some(Self::RomanTrailing),
            4 => Some(Self::RomanConnected),
            5 => Some(Self::CursiveDisconnected),
            6 => Some(Self::CursiveTrailing),
            7 => Some(Self::CursiveConnected),
            8 => Some(Self::BlackletterDisconnected),
            9 => Some(Self::BlackletterTrailing),
            10 => Some(Self::BlackletterConnected),
            _ => None,
        }
    }
}

/// Roundness and skew of letterforms in a [`LatinText`](Panose::LatinText) face.
///
/// <https://monotype.github.io/panose/pan2.htm#_Toc380547284>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Letterform {
    AnyFit = 0,
    NoFit = 1,
    NormalContact = 2,
    NormalWeighted = 3,
    NormalBoxed = 4,
    NormalFlattened = 5,
    NormalRounded = 6,
    NormalOffCenter = 7,
    NormalSquare = 8,
    ObliqueContact = 9,
    ObliqueWeighted = 10,
    ObliqueBoxed = 11,
    ObliqueFlattened = 12,
    ObliqueRounded = 13,
    ObliqueOffCenter = 14,
    ObliqueSquare = 15,
}

impl FromData for Letterform {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::NormalContact),
            3 => Some(Self::NormalWeighted),
            4 => Some(Self::NormalBoxed),
            5 => Some(Self::NormalFlattened),
            6 => Some(Self::NormalRounded),
            7 => Some(Self::NormalOffCenter),
            8 => Some(Self::NormalSquare),
            9 => Some(Self::ObliqueContact),
            10 => Some(Self::ObliqueWeighted),
            11 => Some(Self::ObliqueBoxed),
            12 => Some(Self::ObliqueFlattened),
            13 => Some(Self::ObliqueRounded),
            14 => Some(Self::ObliqueOffCenter),
            15 => Some(Self::ObliqueSquare),
            _ => None,
        }
    }
}

/// Style of character outline in a [`LatinDecorative`](Panose::LatinDecorative) face.
///
/// <https://monotype.github.io/panose/pan4.htm#_Toc380547391>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Lining {
    AnyFit = 0,
    NoFit = 1,
    None = 2,
    Inline = 3,
    Outline = 4,
    Engraved = 5,
    Shadow = 6,
    Relief = 7,
    Backdrop = 8,
}

impl FromData for Lining {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::None),
            3 => Some(Self::Inline),
            4 => Some(Self::Outline),
            5 => Some(Self::Engraved),
            6 => Some(Self::Shadow),
            7 => Some(Self::Relief),
            8 => Some(Self::Backdrop),
            _ => None,
        }
    }
}

/// Classification based on the diagonal stem apexes location of midline on uppercase characters in
/// a [`LatinText`](Panose::LatinText) face.
///
/// <https://monotype.github.io/panose/pan2.htm#Sec2Midline>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Midline {
    AnyFit = 0,
    NoFit = 1,
    StandardTrimmed = 2,
    StandardPointed = 3,
    StandardSerifed = 4,
    HighTrimmed = 5,
    HighPointed = 6,
    HighSerifed = 7,
    ConstantTrimmed = 8,
    ConstantPointed = 9,
    ConstantSerifed = 10,
    LowTrimmed = 11,
    LowPointed = 12,
    LowSerifed = 13,
}

impl FromData for Midline {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::StandardTrimmed),
            3 => Some(Self::StandardPointed),
            4 => Some(Self::StandardSerifed),
            5 => Some(Self::HighTrimmed),
            6 => Some(Self::HighPointed),
            7 => Some(Self::HighSerifed),
            8 => Some(Self::ConstantTrimmed),
            9 => Some(Self::ConstantPointed),
            10 => Some(Self::ConstantSerifed),
            11 => Some(Self::LowTrimmed),
            12 => Some(Self::LowPointed),
            13 => Some(Self::LowSerifed),
            _ => None,
        }
    }
}

/// Classification based on the width of characters relative to other characters as well as their
/// aspect ratio in a [`LatinText`](Panose::LatinText) face.
///
/// <https://monotype.github.io/panose/pan2.htm#_Toc380547256>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Proportion {
    AnyFit = 0,
    NoFit = 1,
    OldStyle = 2,
    Modern = 3,
    EvenWidth = 4,
    Extended = 5,
    Condensed = 6,
    VeryExtended = 7,
    VeryCondensed = 8,
    Monospaced = 9,
}

impl FromData for Proportion {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::OldStyle),
            3 => Some(Self::Modern),
            4 => Some(Self::EvenWidth),
            5 => Some(Self::Extended),
            6 => Some(Self::Condensed),
            7 => Some(Self::VeryExtended),
            8 => Some(Self::VeryCondensed),
            9 => Some(Self::Monospaced),
            _ => None,
        }
    }
}

/// Appearance of the serifs in a [`LatinText`](Panose::LatinText) or
/// [`LatinDecorative`](Panose::LatinDecorative) face.
///
/// <https://monotype.github.io/panose/pan2.htm#Sec2SerifStyle>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Serif {
    Any = 0,
    NoFit = 1,
    Cove = 2,
    ObtuseCove = 3,
    SquareCove = 4,
    ObtuseSquareCove = 5,
    Square = 6,
    Thin = 7,
    Oval = 8,
    Exaggerated = 9,
    Triangle = 10,
    NormalSans = 11,
    ObtuseSans = 12,
    PerpendicularSans = 13,
    Flared = 14,
    Rounded = 15,
    // The following is Latin Symbol specific?
    Script = 16,
}

impl FromData for Serif {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::Any),
            1 => Some(Self::NoFit),
            2 => Some(Self::Cove),
            3 => Some(Self::ObtuseCove),
            4 => Some(Self::SquareCove),
            5 => Some(Self::ObtuseSquareCove),
            6 => Some(Self::Square),
            7 => Some(Self::Thin),
            8 => Some(Self::Oval),
            9 => Some(Self::Exaggerated),
            10 => Some(Self::Triangle),
            11 => Some(Self::NormalSans),
            12 => Some(Self::ObtuseSans),
            13 => Some(Self::PerpendicularSans),
            14 => Some(Self::Flared),
            15 => Some(Self::Rounded),
            16 => Some(Self::Script),
            _ => None,
        }
    }
}

/// Whether a [`LatinHandwritten`](Panose::LatinHandwritten) or [`LatinSymbol`](Panose::LatinSymbol)
/// face is monospaced or proportional.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547321>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Spacing {
    AnyFit = 0,
    NoFit = 1,
    Proportional = 2,
    Monospaced = 3,
}

impl FromData for Spacing {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::Proportional),
            3 => Some(Self::Monospaced),
            _ => None,
        }
    }
}

/// Contrast of stem thickness transitions in a [`LatinText`](Panose::LatinText) face.
///
/// <https://monotype.github.io/panose/pan2.htm#_Toc380547270>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum StrokeVariation {
    AnyFit = 0,
    NoFit = 1,
    NoVariation = 2,
    GradualDiagonal = 3,
    GradualTransitional = 4,
    GradualVertical = 5,
    GradualHorizontal = 6,
    RapidVertical = 7,
    RapidHorizontal = 8,
    InstantVertical = 9,
    InstantHorizontal = 10,
}

impl FromData for StrokeVariation {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::NoVariation),
            3 => Some(Self::GradualDiagonal),
            4 => Some(Self::GradualTransitional),
            5 => Some(Self::GradualVertical),
            6 => Some(Self::GradualHorizontal),
            7 => Some(Self::RapidVertical),
            8 => Some(Self::RapidHorizontal),
            9 => Some(Self::InstantVertical),
            10 => Some(Self::InstantHorizontal),
            _ => None,
        }
    }
}

/// Height of character divided by the black width.
///
/// <https://monotype.github.io/panose/pan5.htm#_Toc380547418>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum SymbolAspectRatio {
    AnyFit = 0,
    NoFit = 1,
    NoWidth = 2,
    ExceptionallyWide = 3,
    SuperWide = 4,
    VeryWide = 5,
    Wide = 6,
    Normal = 7,
    Narrow = 8,
    VeryNarrow = 9,
}

impl FromData for SymbolAspectRatio {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::NoWidth),
            3 => Some(Self::ExceptionallyWide),
            4 => Some(Self::SuperWide),
            5 => Some(Self::VeryWide),
            6 => Some(Self::Wide),
            7 => Some(Self::Normal),
            8 => Some(Self::Narrow),
            9 => Some(Self::VeryNarrow),
            _ => None,
        }
    }
}

/// Type of symbols included in a [`LatinSymbol`](Panose::LatinSymbol) face.
///
/// <https://monotype.github.io/panose/pan5.htm#_Toc380547406>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum SymbolKind {
    AnyFit = 0,
    NoFit = 1,
    Montages = 2,
    Pictures = 3,
    Shapes = 4,
    Scientific = 5,
    Music = 6,
    Expert = 7,
    Patterns = 8,
    Borders = 9,
    Icons = 10,
    Logos = 11,
    IndustrySpecific = 12,
}

impl FromData for SymbolKind {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::Montages),
            3 => Some(Self::Pictures),
            4 => Some(Self::Shapes),
            5 => Some(Self::Scientific),
            6 => Some(Self::Music),
            7 => Some(Self::Expert),
            8 => Some(Self::Patterns),
            9 => Some(Self::Borders),
            10 => Some(Self::Icons),
            11 => Some(Self::Logos),
            12 => Some(Self::IndustrySpecific),
            _ => None,
        }
    }
}

/// Kind of handwriting tool a [`LatinHandwritten`](Panose::LatinHandwritten) face is intended to
/// emulate.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547310>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ToolKind {
    AnyFit = 0,
    NoFit = 1,
    FlatNib = 2,
    PressurePoint = 3,
    Engraved = 4,
    Ball = 5,
    Brush = 6,
    Rough = 7,
    FeltPenBrush = 8,
    WildBrush = 9,
}

impl FromData for ToolKind {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::FlatNib),
            3 => Some(Self::PressurePoint),
            4 => Some(Self::Engraved),
            5 => Some(Self::Ball),
            6 => Some(Self::Brush),
            7 => Some(Self::Rough),
            8 => Some(Self::FeltPenBrush),
            9 => Some(Self::WildBrush),
            _ => None,
        }
    }
}

/// Ratio of stroke thickness to the height of the uppercase "E" character.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547314>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Weight {
    AnyFit = 0,
    NoFit = 1,
    VeryLight = 2,
    Light = 3,
    Thin = 4,
    Book = 5,
    Medium = 6,
    Demi = 7,
    Bold = 8,
    Heavy = 9,
    Black = 10,
    ExtraBlack = 11,
}

impl FromData for Weight {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::VeryLight),
            3 => Some(Self::Light),
            4 => Some(Self::Thin),
            5 => Some(Self::Book),
            6 => Some(Self::Medium),
            7 => Some(Self::Demi),
            8 => Some(Self::Bold),
            9 => Some(Self::Heavy),
            10 => Some(Self::Black),
            11 => Some(Self::ExtraBlack),
            _ => None,
        }
    }
}

/// Relative size of lowercase characters in a [`LatinHandwritten`](Panose::LatinHandwritten) face.
///
/// <https://monotype.github.io/panose/pan3.htm#_Toc380547350>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum XAscent {
    AnyFit = 0,
    NoFit = 1,
    VeryLow = 2,
    Low = 3,
    Medium = 4,
    High = 5,
    VeryHigh = 6,
}

impl FromData for XAscent {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::VeryLow),
            3 => Some(Self::Low),
            4 => Some(Self::Medium),
            5 => Some(Self::High),
            6 => Some(Self::VeryHigh),
            _ => None,
        }
    }
}

/// Relative size of lowercase characters in a [`LatinText`](Panose::LatinText) face.
///
/// <https://monotype.github.io/panose/pan2.htm#_Toc380547299>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum XHeight {
    AnyFit = 0,
    NoFit = 1,
    ConstantSmall = 2,
    ConstantStandard = 3,
    ConstantLarge = 4,
    DuckingSmall = 5,
    DuckingStandard = 6,
    DuckingLarge = 7,
}

impl FromData for XHeight {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            0 => Some(Self::AnyFit),
            1 => Some(Self::NoFit),
            2 => Some(Self::ConstantSmall),
            3 => Some(Self::ConstantStandard),
            4 => Some(Self::ConstantLarge),
            5 => Some(Self::DuckingSmall),
            6 => Some(Self::DuckingStandard),
            7 => Some(Self::DuckingLarge),
            _ => None,
        }
    }
}

/// A [PANOSE classification](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/os2#panose).
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Panose {
    AnyFit,
    NoFit,
    LatinText {
        serif: Serif,
        weight: Weight,
        proportion: Proportion,
        contrast: Contrast,
        stroke: StrokeVariation,
        arm: ArmStyle,
        letterform: Letterform,
        midline: Midline,
        x_height: XHeight,
    },
    LatinHandwritten {
        tool_kind: ToolKind,
        weight: Weight,
        spacing: Spacing,
        aspect_ratio: HandwrittenAspectRatio,
        contrast: Contrast,
        topology: HandwrittenTopology,
        form: HandForm,
        finials: Finial,
        x_ascent: XAscent,
    },
    LatinDecorative {
        decoration_class: DecorationClass,
        weight: Weight,
        aspect_ratio: DecorativeAspectRatio,
        contrast: Contrast,
        serif: Serif,
        treatment: DecorativeTreatment,
        lining: Lining,
        topology: DecorativeTopology,
        range_of_characters: DecorativeRange,
    },
    LatinSymbol {
        symbol_kind: SymbolKind,
        weight: Weight,
        spacing: Spacing,
        // This should always be 1
        contrast: u8,
        aspect_94: SymbolAspectRatio,
        aspect_119: SymbolAspectRatio,
        aspect_157: SymbolAspectRatio,
        aspect_163: SymbolAspectRatio,
        aspect_211: SymbolAspectRatio,
    },
}

impl Panose {
    /// Returns true if the weight is set to bold.
    pub fn is_bold(&self) -> bool {
        match self {
            Panose::AnyFit => false,
            Panose::NoFit => false,
            Panose::LatinText { weight, .. }
            | Panose::LatinHandwritten { weight, .. }
            | Panose::LatinDecorative { weight, .. }
            | Panose::LatinSymbol { weight, .. } => weight == &Weight::Bold,
        }
    }

    /// Returns true if the letter form is set to oblique.
    pub fn is_italic(&self) -> bool {
        match self {
            Panose::LatinText { letterform, .. } => match letterform {
                Letterform::ObliqueContact
                | Letterform::ObliqueWeighted
                | Letterform::ObliqueBoxed
                | Letterform::ObliqueFlattened
                | Letterform::ObliqueRounded
                | Letterform::ObliqueOffCenter
                | Letterform::ObliqueSquare => true,
                _ => false,
            },
            Panose::LatinHandwritten { form, .. } => match form {
                HandForm::ObliqueNoWrap
                | HandForm::ObliqueSomeWrap
                | HandForm::ObliqueMoreWrap
                | HandForm::ObliqueExtremeWrap => true,
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns true if the aspect ratio is set to monospaced.
    pub fn is_monospaced(&self) -> bool {
        match self {
            Panose::LatinText { proportion, .. } => proportion == &Proportion::Monospaced,
            Panose::LatinHandwritten { spacing, .. } => spacing == &Spacing::Monospaced,
            Panose::LatinDecorative { aspect_ratio, .. } => {
                aspect_ratio == &DecorativeAspectRatio::Monospaced
            }
            Panose::LatinSymbol { spacing, .. } => spacing == &Spacing::Monospaced,
            _ => false,
        }
    }

    /// Returns an approximate mapping of the PANOSE weight to the OS/2 weight class.
    pub fn weight(&self) -> crate::Weight {
        match self {
            Panose::LatinText { weight, .. }
            | Panose::LatinHandwritten { weight, .. }
            | Panose::LatinDecorative { weight, .. }
            | Panose::LatinSymbol { weight, .. } => match weight {
                Weight::VeryLight => crate::Weight::Thin,
                Weight::Light => crate::Weight::ExtraLight,
                Weight::Thin => crate::Weight::Light,
                Weight::Book => crate::Weight::Normal,
                Weight::Medium => crate::Weight::Normal,
                Weight::Demi => crate::Weight::SemiBold,
                Weight::Bold => crate::Weight::Bold,
                Weight::Heavy => crate::Weight::ExtraBold,
                Weight::Black => crate::Weight::Black,
                Weight::ExtraBlack => crate::Weight::Black,
                _ => crate::Weight::Normal,
            },
            _ => crate::Weight::Normal,
        }
    }
}

impl FromData for Panose {
    const SIZE: usize = 10;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let panose_kind = s.read::<u8>()?;

        match panose_kind {
            0 => {
                s.advance(9);
                Some(Panose::AnyFit)
            }
            1 => {
                s.advance(9);
                Some(Panose::NoFit)
            }
            2 => Some(Panose::LatinText {
                serif: s.read()?,
                weight: s.read()?,
                proportion: s.read()?,
                contrast: s.read()?,
                stroke: s.read()?,
                arm: s.read()?,
                letterform: s.read()?,
                midline: s.read()?,
                x_height: s.read()?,
            }),
            3 => Some(Panose::LatinHandwritten {
                tool_kind: s.read()?,
                weight: s.read()?,
                spacing: s.read()?,
                aspect_ratio: s.read()?,
                contrast: s.read()?,
                topology: s.read()?,
                form: s.read()?,
                finials: s.read()?,
                x_ascent: s.read()?,
            }),
            4 => Some(Panose::LatinDecorative {
                decoration_class: s.read()?,
                weight: s.read()?,
                aspect_ratio: s.read()?,
                contrast: s.read()?,
                serif: s.read()?,
                treatment: s.read()?,
                lining: s.read()?,
                topology: s.read()?,
                range_of_characters: s.read()?,
            }),
            5 => Some(Panose::LatinSymbol {
                symbol_kind: s.read()?,
                weight: s.read()?,
                spacing: s.read()?,
                contrast: s.read()?,
                aspect_94: s.read()?,
                aspect_119: s.read()?,
                aspect_157: s.read()?,
                aspect_163: s.read()?,
                aspect_211: s.read()?,
            }),
            _ => None,
        }
    }
}

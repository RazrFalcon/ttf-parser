//! An [Embedded Bitmap Location Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/eblc) implementation.
use super::cblc;

// CBLC is defined as a backward compatible extension to EBLC, so any valid EBLC is also a valid
// CBLC. Thus, we can just re-use the CBLC table parsing code for EBLC.

/// An [Embedded Bitmap Location Table](
/// https://docs.microsoft.com/en-us/typography/opentype/spec/eblc).
#[derive(Clone, Copy)]
pub struct Table<'a>(pub(crate) cblc::Table<'a>);

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        cblc::Table::parse(data).map(Self)
    }
}

impl core::fmt::Debug for Table<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
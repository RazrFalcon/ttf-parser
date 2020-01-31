// https://docs.microsoft.com/en-us/typography/opentype/spec/gpos

use crate::{Font, Result};
use crate::ggg::*;
use crate::raw;


impl<'a> Font<'a> {
    /// Returns a reference to a [Glyph Positioning Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos).
    pub fn positioning_table(&self) -> Result<PositioningTable<'a>> {
        Ok(PositioningTable { data: self.gpos? })
    }
}


/// A reference to a [Glyph Positioning Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos).
#[derive(Clone, Copy)]
pub struct PositioningTable<'a> {
    data: raw::gsubgpos::Table<'a>,
}

impl<'a> GlyphPosSubTable for PositioningTable<'a> {
    fn scripts(&self) -> Result<Scripts> {
        parse_scripts(self.data)
    }

    fn features(&self) -> Result<Features> {
        parse_features(self.data)
    }

    fn lookups(&self) -> Result<Lookups> {
        parse_lookups(self.data)
    }

    fn feature_variations(&self) -> Result<FeatureVariations> {
        parse_feature_variations(self.data)
    }
}

impl core::fmt::Debug for PositioningTable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PositioningTable()")
    }
}

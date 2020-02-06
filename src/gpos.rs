// https://docs.microsoft.com/en-us/typography/opentype/spec/gpos

use crate::Font;
use crate::ggg::*;


impl<'a> Font<'a> {
    /// Returns a reference to a [Glyph Positioning Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos).
    pub fn positioning_table(&self) -> Option<PositioningTable<'a>> {
        self.gpos.map(|table| PositioningTable { table })
    }
}


/// A reference to a [Glyph Positioning Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos).
#[derive(Clone, Copy)]
pub struct PositioningTable<'a> {
    table: GsubGposTable<'a>,
}

impl<'a> GlyphPosSubTable for PositioningTable<'a> {
    fn scripts(&self) -> Scripts {
        self.table.script
    }

    fn features(&self) -> Features {
        self.table.features
    }

    fn lookups(&self) -> Lookups {
        self.table.lookups
    }

    fn feature_variations(&self) -> FeatureVariations {
        self.table.feature_variations
    }
}

impl core::fmt::Debug for PositioningTable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PositioningTable()")
    }
}

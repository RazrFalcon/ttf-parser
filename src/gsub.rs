// https://docs.microsoft.com/en-us/typography/opentype/spec/gsub

use crate::Font;
use crate::ggg::*;


impl<'a> Font<'a> {
    /// Returns a reference to a [Glyph Substitution Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gsub).
    pub fn substitution_table(&self) -> Option<SubstitutionTable<'a>> {
        self.gsub.map(|table| SubstitutionTable { table })
    }
}


/// A reference to a [Glyph Substitution Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gsub).
#[derive(Clone, Copy)]
pub struct SubstitutionTable<'a> {
    table: GsubGposTable<'a>,
}

impl<'a> GlyphPosSubTable for SubstitutionTable<'a> {
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

impl core::fmt::Debug for SubstitutionTable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SubstitutionTable()")
    }
}

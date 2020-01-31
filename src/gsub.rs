// https://docs.microsoft.com/en-us/typography/opentype/spec/gsub

use crate::{Font, Result};
use crate::ggg::*;
use crate::raw;


impl<'a> Font<'a> {
    /// Returns a reference to a [Glyph Substitution Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gsub).
    pub fn substitution_table(&self) -> Result<SubstitutionTable<'a>> {
        Ok(SubstitutionTable { data: self.gsub? })
    }
}


/// A reference to a [Glyph Substitution Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gsub).
#[derive(Clone, Copy)]
pub struct SubstitutionTable<'a> {
    data: raw::gsubgpos::Table<'a>,
}

impl<'a> GlyphPosSubTable for SubstitutionTable<'a> {
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

impl core::fmt::Debug for SubstitutionTable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SubstitutionTable()")
    }
}

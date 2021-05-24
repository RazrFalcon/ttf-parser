// https://docs.microsoft.com/en-us/typography/opentype/spec/gdef

use crate::GlyphId;
use crate::parser::{Stream, Offset, Offset16, Offset32, LazyArray16};
use crate::ggg::{Class, ClassDefinitionTable, CoverageTable};

#[cfg(feature = "variable-fonts")] use crate::NormalizedCoordinate;
#[cfg(feature = "variable-fonts")] use crate::var_store::ItemVariationStore;


/// A [glyph class](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
#[allow(missing_docs)]
pub enum GlyphClass {
    Base      = 1,
    Ligature  = 2,
    Mark      = 3,
    Component = 4,
}


#[derive(Clone, Copy, Default)]
pub struct Table<'a> {
    glyph_classes: Option<ClassDefinitionTable<'a>>,
    mark_attach_classes: Option<ClassDefinitionTable<'a>>,
    mark_glyph_coverage_offsets: Option<(&'a [u8], LazyArray16<'a, Offset32>)>,
    #[cfg(feature = "variable-fonts")] variation_store: Option<ItemVariationStore<'a>>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version: u32 = s.read()?;
        if !(version == 0x00010000 || version == 0x00010002 || version == 0x00010003) {
            return None;
        }

        let glyph_class_def_offset: Option<Offset16> = s.read()?;
        s.skip::<Offset16>(); // attachListOffset
        s.skip::<Offset16>(); // ligCaretListOffset
        let mark_attach_class_def_offset: Option<Offset16> = s.read()?;

        let mut mark_glyph_sets_def_offset: Option<Offset16> = None;
        if version > 0x00010000 {
            mark_glyph_sets_def_offset = s.read()?;
        }

        #[allow(unused_mut)]
        #[allow(unused_variables)]
        let mut var_store_offset: Option<Offset32> = None;

        #[cfg(feature = "variable-fonts")]
        {
            if version > 0x00010002 {
                var_store_offset = s.read();
            }
        }

        let mut table = Table::default();

        if let Some(offset) = glyph_class_def_offset {
            if let Some(subdata) = data.get(offset.to_usize()..) {
                table.glyph_classes = Some(ClassDefinitionTable::new(subdata));
            }
        }

        if let Some(offset) = mark_attach_class_def_offset {
            if let Some(subdata) = data.get(offset.to_usize()..) {
                table.mark_attach_classes = Some(ClassDefinitionTable::new(subdata));
            }
        }

        if let Some(offset) = mark_glyph_sets_def_offset {
            if let Some(subdata) = data.get(offset.to_usize()..) {
                let mut s = Stream::new(subdata);
                let format: u16 = s.read()?;
                if format == 1 {
                    if let Some(count) = s.read::<u16>() {
                        if let Some(array) = s.read_array16::<Offset32>(count) {
                            table.mark_glyph_coverage_offsets = Some((subdata, array));
                        }
                    }
                }
            }
        }

        #[cfg(feature = "variable-fonts")]
        {
            if let Some(offset) = var_store_offset {
                if let Some(subdata) = data.get(offset.to_usize()..) {
                    let s = Stream::new(subdata);
                    table.variation_store = ItemVariationStore::parse(s);
                }
            }
        }

        Some(table)
    }

    #[inline]
    pub fn has_glyph_classes(&self) -> bool {
        self.glyph_classes.is_some()
    }

    #[inline]
    pub fn glyph_class(&self, glyph_id: GlyphId) -> Option<GlyphClass> {
        match self.glyph_classes?.get(glyph_id).0 {
            1 => Some(GlyphClass::Base),
            2 => Some(GlyphClass::Ligature),
            3 => Some(GlyphClass::Mark),
            4 => Some(GlyphClass::Component),
            _ => None,
        }
    }

    #[inline]
    pub fn glyph_mark_attachment_class(&self, glyph_id: GlyphId) -> Class {
        self.mark_attach_classes
            .map(|def| def.get(glyph_id))
            .unwrap_or(Class(0))
    }

    #[inline]
    pub fn is_mark_glyph(&self, glyph_id: GlyphId, set_index: Option<u16>) -> bool {
        is_mark_glyph_impl(self, glyph_id, set_index).is_some()
    }

    #[cfg(feature = "variable-fonts")]
    #[inline]
    pub fn variation_delta(
        &self,
        outer_index: u16,
        inner_index: u16,
        coordinates: &[NormalizedCoordinate],
    ) -> Option<f32> {
        self.variation_store
            .and_then(|store| store.parse_delta(outer_index, inner_index, coordinates))
    }
}

#[inline(never)]
fn is_mark_glyph_impl(
    table: &Table,
    glyph_id: GlyphId,
    set_index: Option<u16>,
) -> Option<()> {
    let (data, offsets) = table.mark_glyph_coverage_offsets?;

    if let Some(set_index) = set_index {
        if let Some(offset) = offsets.get(set_index) {
            let table = CoverageTable::new(data.get(offset.to_usize()..)?);
            if table.contains(glyph_id) {
                return Some(());
            }
        }
    } else {
        for offset in offsets {
            let table = CoverageTable::new(data.get(offset.to_usize()..)?);
            if table.contains(glyph_id) {
                return Some(());
            }
        }
    }

    None
}

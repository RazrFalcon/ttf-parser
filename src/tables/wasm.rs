//! A [WASM table](https://github.com/harfbuzz/harfbuzz/blob/main/docs/wasm-shaper.md) experimental implementation

#![allow(unused)] // for the time being

use crate::parser::{Offset as _, Offset32, Stream};

#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    pub wasm_data: &'a [u8],
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        // let mut s = Stream::new(data);
        // s.skip::<u16>(); // version
        // let doc_list_offset = s.read::<Option<Offset32>>()??;

        // let mut s = Stream::new_at(data, doc_list_offset.to_usize())?;
        // let count = s.read::<u16>()?;

        Some(Table { wasm_data: data })
    }
}

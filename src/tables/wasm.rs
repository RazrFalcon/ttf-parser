//! A [WASM table](https://github.com/harfbuzz/harfbuzz/blob/main/docs/wasm-shaper.md) experimental implementation

#![allow(unused)] // for the time being

use crate::parser::{Offset as _, Offset32, Stream};

/// A WASM module. Contains the WASM Raw Data
#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    /// Raw WASM data in WASM format.
    pub wasm_data: &'a [u8],
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        Some(Table { wasm_data: data })
    }
}

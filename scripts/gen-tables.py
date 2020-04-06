#!/usr/bin/env python3

import re
from typing import List


def to_snake_case(name: str) -> str:
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


class TtfType:
    def to_rust(self) -> str:
        raise NotImplementedError()

    def size(self) -> int:
        return 0

    def print(self, offset: int) -> None:
        raise NotImplementedError()


class TtfUInt8(TtfType):
    def to_rust(self) -> str:
        return 'u8'

    def size(self) -> int:
        return 1

    def print(self, offset: int) -> None:
        print(f'self.data[{offset}]')


class TtfInt32(TtfType):
    def to_rust(self) -> str:
        return 'i32'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'i32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f'])')


class TtfUInt16(TtfType):
    def to_rust(self) -> str:
        return 'u16'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]])')


class TtfNonZeroUInt16(TtfType):
    def to_rust(self) -> str:
        return 'Option<NonZeroU16>'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'NonZeroU16::new(u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]))')


class TtfInt16(TtfType):
    def to_rust(self) -> str:
        return 'i16'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'i16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]])')


class TtfUInt24(TtfType):
    def to_rust(self) -> str:
        return 'u32'

    def size(self) -> int:
        return 3

    def print(self, offset: int) -> None:
        print(f'u32::from_be_bytes(['
              f'    0, self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}]'
              f'])')


class TtfUInt32(TtfType):
    def to_rust(self) -> str:
        return 'u32'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'u32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f'])')


class TtfFWORD(TtfInt16):
    pass


class TtfUFWORD(TtfUInt16):
    pass


class TtfOffset16(TtfType):
    def to_rust(self) -> str:
        return 'Offset16'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'Offset16(u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]))')


class TtfOptionalOffset16(TtfType):
    def to_rust(self) -> str:
        return 'Option<Offset16>'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'let n = u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]);')
        print('if n != 0 { Some(Offset16(n)) } else { None }')


class TtfOffset32(TtfType):
    def to_rust(self) -> str:
        return 'Offset32'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'Offset32(u32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f']))')


class TtfOptionalOffset32(TtfType):
    def to_rust(self) -> str:
        return 'Option<Offset32>'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'let n = u32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f']);')
        print('if n != 0 { Some(Offset32(n)) } else { None }')


class TtfGlyphId(TtfType):
    def to_rust(self) -> str:
        return 'GlyphId'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'GlyphId(u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]))')


class TtfGlyphIdRangeInclusive(TtfType):
    def to_rust(self) -> str:
        return 'RangeInclusive<GlyphId>'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'GlyphId(u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]))'
              f'..=GlyphId(u16::from_be_bytes([self.data[{offset+2}], self.data[{offset + 3}]]))')


class TtfTag(TtfType):
    def to_rust(self) -> str:
        return 'Tag'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'Tag(u32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f']))')


class TtfFixed(TtfType):
    def to_rust(self) -> str:
        return 'f32'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'i32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f']) as f32 / 65536.0')


# unsupported
class TtfLongDateTime(TtfType):
    def size(self) -> int:
        return 8


# unsupported
class TtfPanose(TtfType):
    def size(self) -> int:
        return 10


class TtfF2DOT14(TtfType):
    def to_rust(self) -> str:
        return 'f32'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'i16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]) as f32 / 16384.0')


class TableRow:
    ttf_type: TtfType
    name: str

    def __init__(self, ttf_type: TtfType, name: str):
        self.ttf_type = ttf_type
        self.name = name


# https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
TABLE_RECORD = [
    TableRow(TtfTag(),      'tableTag'),
    TableRow(TtfUInt32(),   'checkSum'),
    TableRow(TtfOffset32(), 'offset'),
    TableRow(TtfUInt32(),   'length'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/head
HEAD_TABLE = [
    TableRow(TtfUInt16(),       'majorVersion'),
    TableRow(TtfUInt16(),       'minorVersion'),
    TableRow(TtfFixed(),        'fontRevision'),
    TableRow(TtfUInt32(),       'checkSumAdjustment'),
    TableRow(TtfUInt32(),       'magicNumber'),
    TableRow(TtfUInt16(),       'flags'),
    TableRow(TtfUInt16(),       'unitsPerEm'),
    TableRow(TtfLongDateTime(), 'created'),
    TableRow(TtfLongDateTime(), 'modified'),
    TableRow(TtfInt16(),        'xMin'),
    TableRow(TtfInt16(),        'yMin'),
    TableRow(TtfInt16(),        'xMax'),
    TableRow(TtfInt16(),        'yMax'),
    TableRow(TtfUInt16(),       'macStyle'),
    TableRow(TtfUInt16(),       'lowestRecPPEM'),
    TableRow(TtfInt16(),        'fontDirectionHint'),
    TableRow(TtfInt16(),        'indexToLocFormat'),
    TableRow(TtfInt16(),        'glyphDataFormat'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/hhea
HHEA_TABLE = [
    TableRow(TtfUInt16(),           'majorVersion'),
    TableRow(TtfUInt16(),           'minorVersion'),
    TableRow(TtfFWORD(),            'ascender'),
    TableRow(TtfFWORD(),            'descender'),
    TableRow(TtfFWORD(),            'lineGap'),
    TableRow(TtfUFWORD(),           'advanceWidthMax'),
    TableRow(TtfFWORD(),            'minLeftSideBearing'),
    TableRow(TtfFWORD(),            'minRightSideBearing'),
    TableRow(TtfFWORD(),            'xMaxExtent'),
    TableRow(TtfInt16(),            'caretSlopeRise'),
    TableRow(TtfInt16(),            'caretSlopeRun'),
    TableRow(TtfInt16(),            'caretOffset'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'metricDataFormat'),
    TableRow(TtfNonZeroUInt16(),    'numberOfHMetrics'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx
HMTX_METRICS = [
    TableRow(TtfUInt16(),   'advanceWidth'),
    TableRow(TtfInt16(),    'lsb'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/vhea#table-format
VHEA_TABLE = [
    TableRow(TtfFixed(),            'version'),
    TableRow(TtfInt16(),            'ascender'),
    TableRow(TtfInt16(),            'descender'),
    TableRow(TtfInt16(),            'lineGap'),
    TableRow(TtfInt16(),            'advanceHeightMax'),
    TableRow(TtfInt16(),            'minTopSideBearing'),
    TableRow(TtfInt16(),            'minBottomSideBearing'),
    TableRow(TtfInt16(),            'yMaxExtent'),
    TableRow(TtfInt16(),            'caretSlopeRise'),
    TableRow(TtfInt16(),            'caretSlopeRun'),
    TableRow(TtfInt16(),            'caretOffset'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'reserved'),
    TableRow(TtfInt16(),            'metricDataFormat'),
    TableRow(TtfNonZeroUInt16(),    'numOfLongVerMetrics'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
NAME_RECORD_TABLE = [
    TableRow(TtfUInt16(),   'platformID'),
    TableRow(TtfUInt16(),   'encodingID'),
    TableRow(TtfUInt16(),   'languageID'),
    TableRow(TtfUInt16(),   'nameID'),
    TableRow(TtfUInt16(),   'length'),
    TableRow(TtfUInt16(),   'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/kern
# In the kern table, coverage is stored as uint16, but we are using two uint8 to simply the code.
KERN_COVERAGE = [
    TableRow(TtfUInt8(),    'coverage'),
    TableRow(TtfUInt8(),    'format'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/kern
# In the kern table, a kerning pair is stored as two uint16, but we are using one uint32
# so we can use binary search.
KERNING_RECORD = [
    TableRow(TtfUInt32(),   'pair'),
    TableRow(TtfInt16(),    'value'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#encoding-records-and-encodings
CMAP_ENCODING_RECORD = [
    TableRow(TtfUInt16(),   'platformID'),
    TableRow(TtfUInt16(),   'encodingID'),
    TableRow(TtfOffset32(), 'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
CMAP_SUB_HEADER_RECORD = [
    TableRow(TtfUInt16(),   'firstCode'),
    TableRow(TtfUInt16(),   'entryCount'),
    TableRow(TtfInt16(),    'idDelta'),
    TableRow(TtfUInt16(),   'idRangeOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
CMAP_SEQUENTIAL_MAP_GROUP_RECORD = [
    TableRow(TtfUInt32(),   'startCharCode'),
    TableRow(TtfUInt32(),   'endCharCode'),
    TableRow(TtfUInt32(),   'startGlyphID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#default-uvs-table
CMAP_UNICODE_RANGE_RECORD = [
    TableRow(TtfUInt24(),   'startUnicodeValue'),
    TableRow(TtfUInt8(),    'additionalCount'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#non-default-uvs-table
CMAP_UVS_MAPPING_RECORD = [
    TableRow(TtfUInt24(),   'unicodeValue'),
    TableRow(TtfGlyphId(),  'glyphID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences
CMAP_VARIATION_SELECTOR_RECORD = [
    TableRow(TtfUInt24(),           'varSelector'),
    TableRow(TtfOptionalOffset32(), 'defaultUVSOffset'),
    TableRow(TtfOptionalOffset32(), 'nonDefaultUVSOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/os2#os2-table-formats
OS_2_TABLE = [
    TableRow(TtfUInt16(),   'version'),
    TableRow(TtfInt16(),    'xAvgCharWidth'),
    TableRow(TtfUInt16(),   'usWeightClass'),
    TableRow(TtfUInt16(),   'usWidthClass'),
    TableRow(TtfUInt16(),   'fsType'),
    TableRow(TtfInt16(),    'ySubscriptXSize'),
    TableRow(TtfInt16(),    'ySubscriptYSize'),
    TableRow(TtfInt16(),    'ySubscriptXOffset'),
    TableRow(TtfInt16(),    'ySubscriptYOffset'),
    TableRow(TtfInt16(),    'ySuperscriptXSize'),
    TableRow(TtfInt16(),    'ySuperscriptYSize'),
    TableRow(TtfInt16(),    'ySuperscriptXOffset'),
    TableRow(TtfInt16(),    'ySuperscriptYOffset'),
    TableRow(TtfInt16(),    'yStrikeoutSize'),
    TableRow(TtfInt16(),    'yStrikeoutPosition'),
    TableRow(TtfInt16(),    'sFamilyClass'),
    TableRow(TtfPanose(),   'panose'),
    TableRow(TtfUInt32(),   'ulUnicodeRange1'),
    TableRow(TtfUInt32(),   'ulUnicodeRange2'),
    TableRow(TtfUInt32(),   'ulUnicodeRange3'),
    TableRow(TtfUInt32(),   'ulUnicodeRange4'),
    TableRow(TtfTag(),      'achVendID'),
    TableRow(TtfUInt16(),   'fsSelection'),
    TableRow(TtfUInt16(),   'usFirstCharIndex'),
    TableRow(TtfUInt16(),   'usLastCharIndex'),
    TableRow(TtfInt16(),    'sTypoAscender'),
    TableRow(TtfInt16(),    'sTypoDescender'),
    TableRow(TtfInt16(),    'sTypoLineGap'),
    TableRow(TtfUInt16(),   'usWinAscent'),
    TableRow(TtfUInt16(),   'usWinDescent'),
    TableRow(TtfUInt32(),   'ulCodePageRange1'),
    TableRow(TtfUInt32(),   'ulCodePageRange2'),
    TableRow(TtfInt16(),    'sxHeight'),
    TableRow(TtfInt16(),    'sCapHeight'),
    TableRow(TtfUInt16(),   'usDefaultChar'),
    TableRow(TtfUInt16(),   'usBreakChar'),
    TableRow(TtfUInt16(),   'usMaxContext'),
    TableRow(TtfUInt16(),   'usLowerOpticalPointSize'),
    TableRow(TtfUInt16(),   'usUpperOpticalPointSize'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/gdef
GDEF_TABLE = [
    TableRow(TtfUInt16(),           'majorVersion'),
    TableRow(TtfUInt16(),           'minorVersion'),
    TableRow(TtfOptionalOffset16(), 'glyphClassDefOffset'),
    TableRow(TtfOptionalOffset16(), 'attachListOffset'),
    TableRow(TtfOptionalOffset16(), 'ligCaretListOffset'),
    TableRow(TtfOptionalOffset16(), 'markAttachClassDefOffset'),
    TableRow(TtfOptionalOffset16(), 'markGlyphSetsDefOffset'),
    TableRow(TtfOptionalOffset32(), 'itemVarStoreOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table-format-2
GDEF_CLASS_RANGE_RECORD = [
    TableRow(TtfGlyphIdRangeInclusive(),    'range'),
    TableRow(TtfUInt16(),                   'class'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-format-2
GDEF_RANGE_RECORD = [
    TableRow(TtfGlyphIdRangeInclusive(),    'range'),
    TableRow(TtfUInt16(),                   'startCoverageIndex'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/avar#table-formats
AVAR_AXIS_VALUE_MAP_RECORD = [
    TableRow(TtfInt16(),    'fromCoordinate'),  # Actually F2DOT14.
    TableRow(TtfInt16(),    'toCoordinate'),    #
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/fvar#variationaxisrecord
FVAR_VARIATION_AXIS_RECORD = [
    TableRow(TtfTag(),      'axisTag'),
    TableRow(TtfFixed(),    'minValue'),
    TableRow(TtfFixed(),    'defValue'),
    TableRow(TtfFixed(),    'maxValue'),
    TableRow(TtfUInt16(),   'flags'),
    TableRow(TtfUInt16(),   'axisNameID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/vorg#vertical-origin-table-format
VERT_ORIGIN_Y_METRICS = [
    TableRow(TtfGlyphId(),  'glyphIndex'),
    TableRow(TtfInt16(),    'vertOriginY'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/mvar
MVAR_VALUE_RECORD = [
    TableRow(TtfTag(),      'valueTag'),
    TableRow(TtfUInt16(),   'deltaSetOuterIndex'),
    TableRow(TtfUInt16(),   'deltaSetInnerIndex'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#variation-regions
VARIATION_STORE_REGION_AXIS_COORDINATES_RECORD = [
    TableRow(TtfInt16(),    'startCoord'),  #
    TableRow(TtfInt16(),    'peakCoord'),   # Use i16 instead of F2DOT14 to simplify calculations.
    TableRow(TtfInt16(),    'endCoord'),    #
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/svg#svg-document-list
SVG_DOC_RECORD = [
    TableRow(TtfGlyphId(),          'startGlyphID'),
    TableRow(TtfGlyphId(),          'endGlyphID'),
    TableRow(TtfOptionalOffset32(), 'svgDocOffset'),
    TableRow(TtfUInt32(),           'svgDocLength'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/post
POST_TABLE = [
    TableRow(TtfFixed(),    'version'),
    TableRow(TtfFixed(),    'italicAngle'),
    TableRow(TtfFWORD(),    'underlinePosition'),
    TableRow(TtfFWORD(),    'underlineThickness'),
    TableRow(TtfUInt32(),   'isFixedPitch'),
    TableRow(TtfUInt32(),   'minMemType42'),
    TableRow(TtfUInt32(),   'maxMemType42'),
    TableRow(TtfUInt32(),   'minMemType1'),
    TableRow(TtfUInt32(),   'maxMemType1'),
]


def print_struct(name: str, size: int) -> None:
    print('#[derive(Clone, Copy)]')
    print(f'pub struct {name} {{ data: [u8; {size}] }}')


def print_struct_size(size: int) -> None:
    print(f'pub const SIZE: usize = {size};')


def print_constructor(name: str) -> None:
    print('#[inline(always)]')
    print('pub fn new(input: &[u8]) -> Option<Self> {')
    print('    use core::convert::TryInto;')
    print(f'    input.try_into().ok().map(|data| {name} {{ data }})')
    print('}')


def print_method(spec_name: str, ttf_type: TtfType, offset: int) -> None:
    fn_name = to_snake_case(spec_name)
    rust_type = ttf_type.to_rust()

    print('    #[inline(always)]')
    print(f'    pub fn {fn_name}(&self) -> {rust_type} {{')
    ttf_type.print(offset)
    print('    }')


def print_impl_from_data(name: str) -> None:
    print(f'impl FromData for {name} {{')
    print(f'    const SIZE: usize = {name}::SIZE;')
    print()
    print('    #[inline]')
    print('    fn parse(data: &[u8]) -> Option<Self> {')
    print('        Self::new(data)')
    print('    }')
    print('}')


def find_struct_size(table: List[TableRow]) -> int:
    struct_size = 0
    for row in table:
        struct_size += row.ttf_type.size()

    return struct_size


# Structs smaller than 16 bytes is more efficient to store as owned.
def generate_table(table: List[TableRow], struct_name: str) -> None:
    struct_size = find_struct_size(table)

    print_struct(struct_name, struct_size)
    print()
    print(f'impl {struct_name} {{')
    print_struct_size(struct_size)
    print()
    print_constructor(struct_name)
    print()

    offset = 0
    for row in table:
        print_method(row.name, row.ttf_type, offset)
        print()

        offset += row.ttf_type.size()

    print('}')

    print()
    print_impl_from_data(struct_name)


def table_field_offset(table: List[TableRow], field: str) -> None:
    offset = 0
    for row in table:
        if row.name == field:
            print(f'pub const {to_snake_case(row.name).upper()}_OFFSET: usize = {offset};')
            return

        offset += row.ttf_type.size()

    raise ValueError('unknown field')


print('// This file is autogenerated by scripts/gen-tables.py')
print('// Do not edit it!')
print()
print('// All structs in this module use fixed-size arrays,')
print('// so Rust compiler will check bounds at compile time.')
print()
print('#![allow(dead_code)]')
print()
print('use crate::Tag;')
print('use crate::parser::{FromData, Offset32};')
print()
generate_table(TABLE_RECORD, 'TableRecord')
print()
print('pub mod head {')
print(f'pub const TABLE_SIZE: usize = {find_struct_size(HEAD_TABLE)};')
table_field_offset(HEAD_TABLE, 'unitsPerEm')
table_field_offset(HEAD_TABLE, 'indexToLocFormat')
print('}')
print()
print('pub mod hhea {')
print(f'pub const TABLE_SIZE: usize = {find_struct_size(HHEA_TABLE)};')
table_field_offset(HHEA_TABLE, 'ascender')
table_field_offset(HHEA_TABLE, 'descender')
table_field_offset(HHEA_TABLE, 'lineGap')
table_field_offset(HHEA_TABLE, 'numberOfHMetrics')
print('}')
print()
print('pub mod hmtx {')
print('use crate::parser::FromData;')
print()
generate_table(HMTX_METRICS, 'HorizontalMetrics')
print('}')
print()
print('pub mod vhea {')
print(f'pub const TABLE_SIZE: usize = {find_struct_size(VHEA_TABLE)};')
table_field_offset(VHEA_TABLE, 'ascender')
table_field_offset(VHEA_TABLE, 'descender')
table_field_offset(VHEA_TABLE, 'lineGap')
table_field_offset(VHEA_TABLE, 'numOfLongVerMetrics')
print('}')
print()
print('pub mod post {')
print(f'pub const TABLE_SIZE: usize = {find_struct_size(POST_TABLE)};')
table_field_offset(POST_TABLE, 'underlinePosition')
table_field_offset(POST_TABLE, 'underlineThickness')
print('}')
print()
print('pub mod cmap {')
print('use crate::GlyphId;')
print('use crate::parser::{FromData, Offset32};')
print()
generate_table(CMAP_ENCODING_RECORD, 'EncodingRecord')
print()
generate_table(CMAP_SUB_HEADER_RECORD, 'SubHeaderRecord')
print()
generate_table(CMAP_SEQUENTIAL_MAP_GROUP_RECORD, 'SequentialMapGroup')
print()
generate_table(CMAP_UNICODE_RANGE_RECORD, 'UnicodeRangeRecord')
print()
generate_table(CMAP_UVS_MAPPING_RECORD, 'UVSMappingRecord')
print()
generate_table(CMAP_VARIATION_SELECTOR_RECORD, 'VariationSelectorRecord')
print('}')
print()
print('pub mod os_2 {')
table_field_offset(OS_2_TABLE, 'usWeightClass')
table_field_offset(OS_2_TABLE, 'usWidthClass')
table_field_offset(OS_2_TABLE, 'ySubscriptXSize')
table_field_offset(OS_2_TABLE, 'ySuperscriptXSize')
table_field_offset(OS_2_TABLE, 'yStrikeoutSize')
table_field_offset(OS_2_TABLE, 'yStrikeoutPosition')
table_field_offset(OS_2_TABLE, 'fsSelection')
table_field_offset(OS_2_TABLE, 'sTypoAscender')
table_field_offset(OS_2_TABLE, 'sTypoDescender')
table_field_offset(OS_2_TABLE, 'sTypoLineGap')
table_field_offset(OS_2_TABLE, 'sxHeight')
print('}')
print()
print('pub mod name {')
print('use crate::parser::FromData;')
print()
generate_table(NAME_RECORD_TABLE, 'NameRecord')
print('}')
print()
print('pub mod kern {')
print('use crate::parser::FromData;')
print()
generate_table(KERN_COVERAGE, 'Coverage')
print()
generate_table(KERNING_RECORD, 'KerningRecord')
print('}')
print()
print('pub mod gdef {')
print('use core::ops::RangeInclusive;')
print('use crate::GlyphId;')
print('use crate::parser::FromData;')
print()
generate_table(GDEF_CLASS_RANGE_RECORD, 'ClassRangeRecord')
print()
generate_table(GDEF_RANGE_RECORD, 'RangeRecord')
print('}')
print()
print('pub mod avar {')
print('use crate::parser::FromData;')
print()
generate_table(AVAR_AXIS_VALUE_MAP_RECORD, 'AxisValueMapRecord')
print('}')
print()
print('pub mod fvar {')
print('use crate::Tag;')
print('use crate::parser::FromData;')
print()
generate_table(FVAR_VARIATION_AXIS_RECORD, 'VariationAxisRecord')
print('}')
print()
print('pub mod vorg {')
print('use crate::GlyphId;')
print('use crate::parser::FromData;')
print()
generate_table(VERT_ORIGIN_Y_METRICS, 'VertOriginYMetrics')
print('}')
print()
print('pub mod mvar {')
print('use crate::Tag;')
print('use crate::parser::FromData;')
print()
generate_table(MVAR_VALUE_RECORD, 'ValueRecord')
print('}')
print()
print('pub mod var_store {')
print('use crate::parser::FromData;')
print()
generate_table(VARIATION_STORE_REGION_AXIS_COORDINATES_RECORD, 'RegionAxisCoordinatesRecord')
print('}')
print()
print('pub mod svg {')
print('use crate::GlyphId;')
print('use crate::parser::{FromData, Offset32};')
print()
generate_table(SVG_DOC_RECORD, 'SvgDocumentRecord')
print('}')
print()

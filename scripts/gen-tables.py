#!/usr/bin/env python3

import re
from typing import List


def to_snake_case(name: str) -> str:
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


class TTFType:
    def to_rust(self) -> str:
        raise NotImplementedError()

    def size(self) -> int:
        return 0

    def print(self, offset: int) -> None:
        raise NotImplementedError()


class TTF_UInt8(TTFType):
    def to_rust(self) -> str:
        return 'u8'

    def size(self) -> int:
        return 1

    def print(self, offset: int) -> None:
        print(f'self.data[{offset}]')


class TTF_UInt16(TTFType):
    def to_rust(self) -> str:
        return 'u16'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]])')


class TTF_Int16(TTFType):
    def to_rust(self) -> str:
        return 'i16'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'i16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]])')


class TTF_UInt24(TTFType):
    def to_rust(self) -> str:
        return 'u32'

    def size(self) -> int:
        return 3

    def print(self, offset: int) -> None:
        print(f'(self.data[{offset}] as u32) << 16 | (self.data[{offset + 1}] as u32) << 8 '
              f'| self.data[{offset + 2}] as u32')


class TTF_UInt32(TTFType):
    def to_rust(self) -> str:
        return 'u32'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'u32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f'])')


class TTF_FWORD(TTF_Int16):
    pass


class TTF_UFWORD(TTF_UInt16):
    pass


class TTF_Offset32(TTF_UInt32):
    pass


class TTF_Optional_Offset32(TTF_UInt32):
    def to_rust(self) -> str:
        return 'Option<u32>'

    def size(self) -> int:
        return 4

    def print(self, offset: int) -> None:
        print(f'let n = u32::from_be_bytes(['
              f'    self.data[{offset}], self.data[{offset + 1}], self.data[{offset + 2}], self.data[{offset + 3}]'
              f']);')
        print('if n != 0 { Some(n) } else { None }')


class TTF_GlyphId(TTFType):
    def to_rust(self) -> str:
        return 'GlyphId'

    def size(self) -> int:
        return 2

    def print(self, offset: int) -> None:
        print(f'GlyphId(u16::from_be_bytes([self.data[{offset}], self.data[{offset + 1}]]))')


# unsupported
class TTF_Tag(TTFType):
    def size(self) -> int:
        return 4


# unsupported
class TTF_Fixed(TTFType):
    def size(self) -> int:
        return 4


# unsupported
class TTF_LONGDATETIME(TTFType):
    def size(self) -> int:
        return 8


# unsupported
class TTF_Panose(TTFType):
    def size(self) -> int:
        return 10


class TableRow:
    enable: bool
    ttf_type: TTFType
    name: str

    def __init__(self, enable: bool, ttf_type: TTFType, name: str):
        self.enable = enable
        self.ttf_type = ttf_type
        self.name = name


# https://docs.microsoft.com/en-us/typography/opentype/spec/head
HEAD_TABLE = [
    TableRow(False, TTF_UInt16(),       'majorVersion'),
    TableRow(False, TTF_UInt16(),       'minorVersion'),
    TableRow(False, TTF_Fixed(),        'fontRevision'),
    TableRow(False, TTF_UInt32(),       'checkSumAdjustment'),
    TableRow(False, TTF_UInt32(),       'magicNumber'),
    TableRow(False, TTF_UInt16(),       'flags'),
    TableRow(True,  TTF_UInt16(),       'unitsPerEm'),
    TableRow(False, TTF_LONGDATETIME(), 'created'),
    TableRow(False, TTF_LONGDATETIME(), 'modified'),
    TableRow(False, TTF_Int16(),        'xMin'),
    TableRow(False, TTF_Int16(),        'yMin'),
    TableRow(False, TTF_Int16(),        'xMax'),
    TableRow(False, TTF_Int16(),        'yMax'),
    TableRow(False, TTF_UInt16(),       'macStyle'),
    TableRow(False, TTF_UInt16(),       'lowestRecPPEM'),
    TableRow(False, TTF_Int16(),        'fontDirectionHint'),
    TableRow(True,  TTF_Int16(),        'indexToLocFormat'),
    TableRow(False, TTF_Int16(),        'glyphDataFormat'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/os2#os2-table-formats
OS_2_TABLE_V0 = [
    TableRow(True,  TTF_UInt16(),   'version'),
    TableRow(False, TTF_Int16(),    'xAvgCharWidth'),
    TableRow(True,  TTF_UInt16(),   'usWeightClass'),
    TableRow(True,  TTF_UInt16(),   'usWidthClass'),
    TableRow(False, TTF_UInt16(),   'fsType'),
    TableRow(True,  TTF_Int16(),    'ySubscriptXSize'),
    TableRow(True,  TTF_Int16(),    'ySubscriptYSize'),
    TableRow(True,  TTF_Int16(),    'ySubscriptXOffset'),
    TableRow(True,  TTF_Int16(),    'ySubscriptYOffset'),
    TableRow(True,  TTF_Int16(),    'ySuperscriptXSize'),
    TableRow(True,  TTF_Int16(),    'ySuperscriptYSize'),
    TableRow(True,  TTF_Int16(),    'ySuperscriptXOffset'),
    TableRow(True,  TTF_Int16(),    'ySuperscriptYOffset'),
    TableRow(True,  TTF_Int16(),    'yStrikeoutSize'),
    TableRow(True,  TTF_Int16(),    'yStrikeoutPosition'),
    TableRow(False, TTF_Int16(),    'sFamilyClass'),
    TableRow(False, TTF_Panose(),   'panose'),
    TableRow(False, TTF_UInt32(),   'ulUnicodeRange1'),
    TableRow(False, TTF_UInt32(),   'ulUnicodeRange2'),
    TableRow(False, TTF_UInt32(),   'ulUnicodeRange3'),
    TableRow(False, TTF_UInt32(),   'ulUnicodeRange4'),
    TableRow(False, TTF_Tag(),      'achVendID'),
    TableRow(True,  TTF_UInt16(),   'fsSelection'),
    TableRow(False, TTF_UInt16(),   'usFirstCharIndex'),
    TableRow(False, TTF_UInt16(),   'usLastCharIndex'),
    TableRow(False, TTF_Int16(),    'sTypoAscender'),
    TableRow(False, TTF_Int16(),    'sTypoDescender'),
    TableRow(False, TTF_Int16(),    'sTypoLineGap'),
    TableRow(False, TTF_UInt16(),   'usWinAscent'),
    TableRow(False, TTF_UInt16(),   'usWinDescent'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/hhea
HHEA_TABLE = [
    TableRow(False, TTF_UInt16(),  'majorVersion'),
    TableRow(False, TTF_UInt16(),  'minorVersion'),
    TableRow(True,  TTF_FWORD(),   'ascender'),
    TableRow(True,  TTF_FWORD(),   'descender'),
    TableRow(True,  TTF_FWORD(),   'lineGap'),
    TableRow(False, TTF_UFWORD(),  'advanceWidthMax'),
    TableRow(False, TTF_FWORD(),   'minLeftSideBearing'),
    TableRow(False, TTF_FWORD(),   'minRightSideBearing'),
    TableRow(False, TTF_FWORD(),   'xMaxExtent'),
    TableRow(False, TTF_Int16(),   'caretSlopeRise'),
    TableRow(False, TTF_Int16(),   'caretSlopeRun'),
    TableRow(False, TTF_Int16(),   'caretOffset'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'metricDataFormat'),
    TableRow(True,  TTF_UInt16(),  'numberOfHMetrics'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx
HMTX_METRICS = [
    TableRow(True,  TTF_UInt16(),   'advanceWidth'),
    TableRow(True,  TTF_Int16(),    'lsb'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/vhea#table-format
VHEA_TABLE = [
    TableRow(False, TTF_Fixed(),   'version'),
    TableRow(False, TTF_Int16(),   'ascender'),
    TableRow(False, TTF_Int16(),   'descender'),
    TableRow(False, TTF_Int16(),   'lineGap'),
    TableRow(False, TTF_Int16(),   'advanceHeightMax'),
    TableRow(False, TTF_Int16(),   'minTopSideBearing'),
    TableRow(False, TTF_Int16(),   'minBottomSideBearing'),
    TableRow(False, TTF_Int16(),   'yMaxExtent'),
    TableRow(False, TTF_Int16(),   'caretSlopeRise'),
    TableRow(False, TTF_Int16(),   'caretSlopeRun'),
    TableRow(False, TTF_Int16(),   'caretOffset'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'reserved'),
    TableRow(False, TTF_Int16(),   'metricDataFormat'),
    TableRow(True,  TTF_UInt16(),  'numOfLongVerMetrics'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx#vertical-metrics-table-format
VMTX_METRICS = [
    TableRow(True,  TTF_UInt16(),   'advanceHeight'),
    TableRow(True,  TTF_Int16(),    'topSideBearing'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/post#header
POST_TABLE = [
    TableRow(False, TTF_Fixed(),   'version'),
    TableRow(False, TTF_Fixed(),   'italicAngle'),
    TableRow(True,  TTF_FWORD(),   'underlinePosition'),
    TableRow(True,  TTF_FWORD(),   'underlineThickness'),
    TableRow(False, TTF_UInt32(),  'isFixedPitch'),
    TableRow(False, TTF_UInt32(),  'minMemType42'),
    TableRow(False, TTF_UInt32(),  'maxMemType42'),
    TableRow(False, TTF_UInt32(),  'minMemType1'),
    TableRow(False, TTF_UInt32(),  'maxMemType1'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
NAME_RECORD_TABLE = [
    TableRow(True,  TTF_UInt16(),   'platformID'),
    TableRow(True,  TTF_UInt16(),   'encodingID'),
    TableRow(True,  TTF_UInt16(),   'languageID'),
    TableRow(True,  TTF_UInt16(),   'nameID'),
    TableRow(True,  TTF_UInt16(),   'length'),
    TableRow(True,  TTF_UInt16(),   'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#encoding-records-and-encodings
CMAP_ENCODING_RECORD = [
    TableRow(True,  TTF_UInt16(),   'platformID'),
    TableRow(True,  TTF_UInt16(),   'encodingID'),
    TableRow(True,  TTF_Offset32(), 'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
CMAP_SUB_HEADER_RECORD = [
    TableRow(True,  TTF_UInt16(),   'firstCode'),
    TableRow(True,  TTF_UInt16(),   'entryCount'),
    TableRow(True,  TTF_Int16(),    'idDelta'),
    TableRow(True,  TTF_UInt16(),   'idRangeOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
CMAP_SEQUENTIAL_MAP_GROUP_RECORD = [
    TableRow(True,  TTF_UInt32(),   'startCharCode'),
    TableRow(True,  TTF_UInt32(),   'endCharCode'),
    TableRow(True,  TTF_UInt32(),   'startGlyphID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#default-uvs-table
CMAP_UNICODE_RANGE_RECORD = [
    TableRow(True,  TTF_UInt24(),   'startUnicodeValue'),
    TableRow(True,  TTF_UInt8(),    'additionalCount'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#non-default-uvs-table
CMAP_UVS_MAPPING_RECORD = [
    TableRow(True,  TTF_UInt24(),   'unicodeValue'),
    TableRow(True,  TTF_GlyphId(),  'glyphID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences
CMAP_VARIATION_SELECTOR_RECORD = [
    TableRow(True,  TTF_UInt24(),               'varSelector'),
    TableRow(True,  TTF_Optional_Offset32(),    'defaultUVSOffset'),
    TableRow(True,  TTF_Optional_Offset32(),    'nonDefaultUVSOffset'),
]


def print_struct(name: str, size: int, owned: bool) -> None:
    print('#[derive(Clone, Copy)]')
    if owned:
        print(f'pub struct {name} {{ data: [u8; {size}] }}')
    else:
        print(f'pub struct {name}<\'a> {{ data: &\'a [u8; {size}] }}')


def print_struct_size(size: int) -> None:
    print(f'pub const SIZE: usize = {size};')


def print_constructor(name: str, size: int, owned: bool) -> None:
    print('#[inline(always)]')
    if owned:
        print('pub fn new(input: &[u8]) -> Self {')
        print('    let mut data = [0u8; Self::SIZE];')
        # Do not use `copy_from_slice`, because it's slower.
        print('    data.clone_from_slice(input);')
        print(f'    {name} {{ data }}')
        print('}')
    else:
        print('pub fn new(input: &\'a [u8]) -> Self {')
        print(f'    {name} {{ data: array_ref![input, {size}] }}')
        print('}')


def print_method(spec_name: str, ttf_type: TTFType, offset: int) -> None:
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
    print('    fn parse(data: &[u8]) -> Self {')
    print('        Self::new(data)')
    print('    }')
    print('}')


# Structs smaller than 16 bytes is more efficient to store as owned.
def generate_table(table: List[TableRow], struct_name: str, owned: bool = False,
                   impl_from_data: bool = False) -> None:
    struct_size = 0
    for row in table:
        struct_size += row.ttf_type.size()

    print_struct(struct_name, struct_size, owned)
    print()
    if owned:
        print(f'impl {struct_name} {{')
    else:
        print(f'impl<\'a> {struct_name}<\'a> {{')
    print_struct_size(struct_size)
    print()
    print_constructor(struct_name, struct_size, owned)
    print()

    offset = 0
    for row in table:
        if not row.enable:
            offset += row.ttf_type.size()
            continue

        print_method(row.name, row.ttf_type, offset)
        print()

        offset += row.ttf_type.size()

    print('}')

    if impl_from_data:
        print()
        print_impl_from_data(struct_name)


print('// This file is autogenerated by scripts/get-tables.py')
print('// Do not edit it!')
print()
print('// By using static arrays we can have compile-time guaranties that')
print('// we are not reading out-ouf-bounds.')
print('// Also, it removes bounds-checking overhead.')
print()
print('// Based on https://github.com/droundy/arrayref')
print('macro_rules! array_ref {')
print('    ($arr:expr, $len:expr) => {{')
print('        // Always check that the slice length is the same as `$len`.')
print('        assert_eq!($arr.len(), $len);')
print('        unsafe { &*($arr.as_ptr() as *const [_; $len]) }')
print('    }}')
print('}')
print()
print('pub mod head {')
generate_table(HEAD_TABLE, 'Table')
print('}')
print()
print('pub mod hhea {')
generate_table(HHEA_TABLE, 'Table')
print('}')
print()
print('pub mod hmtx {')
print('use crate::parser::FromData;')
print()
generate_table(HMTX_METRICS, 'HorizontalMetrics', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod vhea {')
generate_table(VHEA_TABLE, 'Table')
print('}')
print()
print('pub mod vmtx {')
print('use crate::parser::FromData;')
print()
generate_table(VMTX_METRICS, 'VerticalMetrics', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod post {')
generate_table(POST_TABLE, 'Table')
print('}')
print()
print('pub mod cmap {')
print('use crate::GlyphId;')
print('use crate::parser::FromData;')
print()
generate_table(CMAP_ENCODING_RECORD, 'EncodingRecord', owned=True, impl_from_data=True)
print()
generate_table(CMAP_SUB_HEADER_RECORD, 'SubHeaderRecord', owned=True, impl_from_data=True)
print()
generate_table(CMAP_SEQUENTIAL_MAP_GROUP_RECORD, 'SequentialMapGroup', owned=True, impl_from_data=True)
print()
generate_table(CMAP_UNICODE_RANGE_RECORD, 'UnicodeRangeRecord', owned=True, impl_from_data=True)
print()
generate_table(CMAP_UVS_MAPPING_RECORD, 'UVSMappingRecord', owned=True, impl_from_data=True)
print()
generate_table(CMAP_VARIATION_SELECTOR_RECORD, 'VariationSelectorRecord', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod os_2 {')
generate_table(OS_2_TABLE_V0, 'TableV0')
print('}')
print()
print('pub mod name {')
generate_table(NAME_RECORD_TABLE, 'NameRecord', owned=True)
print('}')
print()

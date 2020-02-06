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
        print(f'(self.data[{offset}] as u32) << 16 | (self.data[{offset + 1}] as u32) << 8 '
              f'| self.data[{offset + 2}] as u32')


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
        print('use core::convert::TryInto;')
        print('// Unwrap is safe, because an array and a slice have the same size.')
        print(f'Tag::from_bytes(&self.data[{offset}..{offset + self.size()}].try_into().unwrap())')


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
    enable: bool
    ttf_type: TtfType
    name: str

    def __init__(self, enable: bool, ttf_type: TtfType, name: str, optional: bool = False):
        self.enable = enable
        self.ttf_type = ttf_type
        self.name = name
        self.optional = optional


# https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
TTC_HEADER = [
    TableRow(True,  TtfTag(),      'ttcTag'),
    TableRow(False, TtfUInt16(),   'majorVersion'),
    TableRow(False, TtfUInt16(),   'minorVersion'),
    TableRow(True,  TtfUInt32(),   'numFonts'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
TABLE_RECORD = [
    TableRow(True,  TtfTag(),      'tableTag'),
    TableRow(False, TtfUInt32(),   'checkSum'),
    TableRow(True,  TtfOffset32(), 'offset'),
    TableRow(True,  TtfUInt32(),   'length'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/head
HEAD_TABLE = [
    TableRow(False, TtfUInt16(),       'majorVersion'),
    TableRow(False, TtfUInt16(),       'minorVersion'),
    TableRow(False, TtfFixed(),        'fontRevision'),
    TableRow(False, TtfUInt32(),       'checkSumAdjustment'),
    TableRow(False, TtfUInt32(),       'magicNumber'),
    TableRow(False, TtfUInt16(),       'flags'),
    TableRow(True,  TtfUInt16(),       'unitsPerEm'),
    TableRow(False, TtfLongDateTime(), 'created'),
    TableRow(False, TtfLongDateTime(), 'modified'),
    TableRow(False, TtfInt16(),        'xMin'),
    TableRow(False, TtfInt16(),        'yMin'),
    TableRow(False, TtfInt16(),        'xMax'),
    TableRow(False, TtfInt16(),        'yMax'),
    TableRow(False, TtfUInt16(),       'macStyle'),
    TableRow(False, TtfUInt16(),       'lowestRecPPEM'),
    TableRow(False, TtfInt16(),        'fontDirectionHint'),
    TableRow(True,  TtfInt16(),        'indexToLocFormat'),
    TableRow(False, TtfInt16(),        'glyphDataFormat'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/hhea
HHEA_TABLE = [
    TableRow(False, TtfUInt16(),        'majorVersion'),
    TableRow(False, TtfUInt16(),        'minorVersion'),
    TableRow(True,  TtfFWORD(),         'ascender'),
    TableRow(True,  TtfFWORD(),         'descender'),
    TableRow(True,  TtfFWORD(),         'lineGap'),
    TableRow(False, TtfUFWORD(),        'advanceWidthMax'),
    TableRow(False, TtfFWORD(),         'minLeftSideBearing'),
    TableRow(False, TtfFWORD(),         'minRightSideBearing'),
    TableRow(False, TtfFWORD(),         'xMaxExtent'),
    TableRow(False, TtfInt16(),         'caretSlopeRise'),
    TableRow(False, TtfInt16(),         'caretSlopeRun'),
    TableRow(False, TtfInt16(),         'caretOffset'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'metricDataFormat'),
    TableRow(True,  TtfNonZeroUInt16(), 'numberOfHMetrics'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx
HMTX_METRICS = [
    TableRow(True,  TtfUInt16(),   'advanceWidth'),
    TableRow(True,  TtfInt16(),    'lsb'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/vhea#table-format
VHEA_TABLE = [
    TableRow(False, TtfFixed(),         'version'),
    TableRow(True,  TtfInt16(),         'ascender'),
    TableRow(True,  TtfInt16(),         'descender'),
    TableRow(True,  TtfInt16(),         'lineGap'),
    TableRow(False, TtfInt16(),         'advanceHeightMax'),
    TableRow(False, TtfInt16(),         'minTopSideBearing'),
    TableRow(False, TtfInt16(),         'minBottomSideBearing'),
    TableRow(False, TtfInt16(),         'yMaxExtent'),
    TableRow(False, TtfInt16(),         'caretSlopeRise'),
    TableRow(False, TtfInt16(),         'caretSlopeRun'),
    TableRow(False, TtfInt16(),         'caretOffset'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'reserved'),
    TableRow(False, TtfInt16(),         'metricDataFormat'),
    TableRow(True,  TtfNonZeroUInt16(), 'numOfLongVerMetrics'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
NAME_RECORD_TABLE = [
    TableRow(True,  TtfUInt16(),   'platformID'),
    TableRow(True,  TtfUInt16(),   'encodingID'),
    TableRow(True,  TtfUInt16(),   'languageID'),
    TableRow(True,  TtfUInt16(),   'nameID'),
    TableRow(True,  TtfUInt16(),   'length'),
    TableRow(True,  TtfUInt16(),   'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#encoding-records-and-encodings
CMAP_ENCODING_RECORD = [
    TableRow(True,  TtfUInt16(),   'platformID'),
    TableRow(True,  TtfUInt16(),   'encodingID'),
    TableRow(True,  TtfOffset32(), 'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
CMAP_SUB_HEADER_RECORD = [
    TableRow(True,  TtfUInt16(),   'firstCode'),
    TableRow(True,  TtfUInt16(),   'entryCount'),
    TableRow(True,  TtfInt16(),    'idDelta'),
    TableRow(True,  TtfUInt16(),   'idRangeOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
CMAP_SEQUENTIAL_MAP_GROUP_RECORD = [
    TableRow(True,  TtfUInt32(),   'startCharCode'),
    TableRow(True,  TtfUInt32(),   'endCharCode'),
    TableRow(True,  TtfUInt32(),   'startGlyphID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#default-uvs-table
CMAP_UNICODE_RANGE_RECORD = [
    TableRow(True,  TtfUInt24(),   'startUnicodeValue'),
    TableRow(True,  TtfUInt8(),    'additionalCount'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#non-default-uvs-table
CMAP_UVS_MAPPING_RECORD = [
    TableRow(True,  TtfUInt24(),   'unicodeValue'),
    TableRow(True,  TtfGlyphId(),  'glyphID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences
CMAP_VARIATION_SELECTOR_RECORD = [
    TableRow(True,  TtfUInt24(),            'varSelector'),
    TableRow(True,  TtfOptionalOffset32(),  'defaultUVSOffset'),
    TableRow(True,  TtfOptionalOffset32(),  'nonDefaultUVSOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/os2#os2-table-formats
OS_2_TABLE = [
    TableRow(False, TtfUInt16(),   'version'),
    TableRow(False, TtfInt16(),    'xAvgCharWidth'),
    TableRow(True,  TtfUInt16(),   'usWeightClass'),
    TableRow(True,  TtfUInt16(),   'usWidthClass'),
    TableRow(False, TtfUInt16(),   'fsType'),
    TableRow(True,  TtfInt16(),    'ySubscriptXSize'),
    TableRow(True,  TtfInt16(),    'ySubscriptYSize'),
    TableRow(True,  TtfInt16(),    'ySubscriptXOffset'),
    TableRow(True,  TtfInt16(),    'ySubscriptYOffset'),
    TableRow(True,  TtfInt16(),    'ySuperscriptXSize'),
    TableRow(True,  TtfInt16(),    'ySuperscriptYSize'),
    TableRow(True,  TtfInt16(),    'ySuperscriptXOffset'),
    TableRow(True,  TtfInt16(),    'ySuperscriptYOffset'),
    TableRow(True,  TtfInt16(),    'yStrikeoutSize'),
    TableRow(True,  TtfInt16(),    'yStrikeoutPosition'),
    TableRow(False, TtfInt16(),    'sFamilyClass'),
    TableRow(False, TtfPanose(),   'panose'),
    TableRow(False, TtfUInt32(),   'ulUnicodeRange1'),
    TableRow(False, TtfUInt32(),   'ulUnicodeRange2'),
    TableRow(False, TtfUInt32(),   'ulUnicodeRange3'),
    TableRow(False, TtfUInt32(),   'ulUnicodeRange4'),
    TableRow(False, TtfTag(),      'achVendID'),
    TableRow(True,  TtfUInt16(),   'fsSelection'),
    TableRow(False, TtfUInt16(),   'usFirstCharIndex'),
    TableRow(False, TtfUInt16(),   'usLastCharIndex'),
    TableRow(True,  TtfInt16(),    'sTypoAscender'),
    TableRow(True,  TtfInt16(),    'sTypoDescender'),
    TableRow(True,  TtfInt16(),    'sTypoLineGap'),
    TableRow(False, TtfUInt16(),   'usWinAscent'),
    TableRow(False, TtfUInt16(),   'usWinDescent'),
    TableRow(False, TtfUInt32(),   'ulCodePageRange1', optional=True),
    TableRow(False, TtfUInt32(),   'ulCodePageRange2', optional=True),
    TableRow(False, TtfInt16(),    'sxHeight', optional=True),
    TableRow(False, TtfInt16(),    'sCapHeight', optional=True),
    TableRow(False, TtfUInt16(),   'usDefaultChar', optional=True),
    TableRow(False, TtfUInt16(),   'usBreakChar', optional=True),
    TableRow(False, TtfUInt16(),   'usMaxContext', optional=True),
    TableRow(False, TtfUInt16(),   'usLowerOpticalPointSize', optional=True),
    TableRow(False, TtfUInt16(),   'usUpperOpticalPointSize', optional=True),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/gdef
GDEF_TABLE = [
    TableRow(True,  TtfUInt16(),            'majorVersion'),
    TableRow(True,  TtfUInt16(),            'minorVersion'),
    TableRow(True,  TtfOptionalOffset16(),  'glyphClassDefOffset'),
    TableRow(False, TtfOptionalOffset16(),  'attachListOffset'),
    TableRow(False, TtfOptionalOffset16(),  'ligCaretListOffset'),
    TableRow(True,  TtfOptionalOffset16(),  'markAttachClassDefOffset'),
    TableRow(False, TtfOptionalOffset16(),  'markGlyphSetsDefOffset', optional=True),
    TableRow(False, TtfOptionalOffset32(),  'itemVarStoreOffset', optional=True),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table-format-2
GDEF_CLASS_RANGE_RECORD = [
    TableRow(True,  TtfGlyphIdRangeInclusive(), 'range'),
    TableRow(True,  TtfUInt16(),                'class'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-format-2
GDEF_RANGE_RECORD = [
    TableRow(True,  TtfGlyphIdRangeInclusive(), 'range'),
    TableRow(False, TtfUInt16(),                'startCoverageIndex'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/fvar#variationaxisrecord
FVAR_VARIATION_AXIS_RECORD = [
    TableRow(True,  TtfTag(),       'axisTag'),
    TableRow(True,  TtfInt32(),     'minValue'),      # This three values actually have type Fixed,
    TableRow(True,  TtfInt32(),     'defaultValue'),  # but we need to compare them later,
    TableRow(True,  TtfInt32(),     'maxValue'),      # and Rust doesn't implement Ord for f32.
    TableRow(True,  TtfUInt16(),    'flags'),
    TableRow(True,  TtfUInt16(),    'axisNameID'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/vorg#vertical-origin-table-format
VERT_ORIGIN_Y_METRICS = [
    TableRow(True,  TtfGlyphId(),  'glyphIndex'),
    TableRow(True,  TtfInt16(),    'vertOriginY'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/mvar
MVAR_VALUE_RECORD = [
    TableRow(True,  TtfTag(),      'valueTag'),
    TableRow(True,  TtfUInt16(),   'deltaSetOuterIndex'),
    TableRow(True,  TtfUInt16(),   'deltaSetInnerIndex'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#variation-regions
MVAR_REGION_AXIS_COORDINATES_RECORD = [
    TableRow(True,  TtfInt16(), 'startCoord'),  #
    TableRow(True,  TtfInt16(), 'peakCoord'),   # Use i16 instead of F2DOT14 to simplify calculations.
    TableRow(True,  TtfInt16(), 'endCoord'),    #
]

GSUB_GPOS_RECORD = [
    TableRow(True,  TtfTag(),       'tag'),
    TableRow(True,  TtfOffset16(),  'offset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#condition-table-format-1-font-variation-axis-range
GSUB_GPOS_CONDITION_TABLE = [
    TableRow(True,  TtfUInt16(),    'format'),
    TableRow(True,  TtfUInt16(),    'axisIndex'),
    TableRow(True,  TtfInt16(),     'filterRangeMinValue'),  # Use i16 instead of F2DOT14 to simplify calculations.
    TableRow(True,  TtfInt16(),     'filterRangeMaxValue'),  #
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#featurevariations-table
GSUB_GPOS_FEATURE_VARIATION_RECORD = [
    TableRow(True,  TtfOffset32(),  'conditionSetOffset'),
    TableRow(True,  TtfOffset32(),  'featureTableSubstitutionOffset'),
]

# https://docs.microsoft.com/en-us/typography/opentype/spec/post
POST_TABLE = [
    TableRow(False, TtfFixed(),     'version'),
    TableRow(False, TtfFixed(),     'italicAngle'),
    TableRow(True,  TtfFWORD(),     'underlinePosition'),
    TableRow(True,  TtfFWORD(),     'underlineThickness'),
    TableRow(False, TtfUInt32(),    'isFixedPitch'),
    TableRow(False, TtfUInt32(),    'minMemType42'),
    TableRow(False, TtfUInt32(),    'maxMemType42'),
    TableRow(False, TtfUInt32(),    'minMemType1'),
    TableRow(False, TtfUInt32(),    'maxMemType1'),
]


def print_struct(name: str, size: int, owned: bool) -> None:
    print('#[derive(Clone, Copy)]')
    if owned:
        print(f'pub struct {name} {{ data: [u8; {size}] }}')
    else:
        print(f'pub struct {name}<\'a> {{ data: &\'a [u8] }}')


def print_struct_size(size: int) -> None:
    print('#[allow(dead_code)]')
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
        print(f'    debug_assert_eq!(input.len(), {size});')
        print(f'    {name} {{ data: input }}')
        print('}')


def print_parse(size: int) -> None:
    print('#[inline(always)]')
    print('pub fn parse(input: &\'a [u8]) -> Option<Self> {')
    print(f'    if input.len() == {size} {{')
    print('        Some(Table {')
    print(f'            data: input,')
    print('        })')
    print('    } else {')
    print('        None')
    print('    }')
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
    print('    fn parse(data: &[u8]) -> Self {')
    print('        Self::new(data)')
    print('    }')
    print('}')


# Structs smaller than 16 bytes is more efficient to store as owned.
def generate_table(table: List[TableRow], struct_name: str, owned: bool = False,
                   impl_from_data: bool = False, parse: bool = False) -> None:
    struct_size = 0
    for row in table:
        if row.optional:
            break
        else:
            struct_size += row.ttf_type.size()

    print_struct(struct_name, struct_size, owned)
    print()
    if owned:
        print(f'impl {struct_name} {{')
    else:
        print(f'impl<\'a> {struct_name}<\'a> {{')

    if not parse:
        print_struct_size(struct_size)
        print()
        print_constructor(struct_name, struct_size, owned)
        print()
    else:
        print_parse(struct_size)
        print()

    offset = 0
    for row in table:
        if row.optional:
            break

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


def table_field_offset(table: List[TableRow], field: str) -> None:
    offset = 0
    for row in table:
        if row.name == field:
            print(f'pub const {to_snake_case(row.name).upper()}_OFFSET: usize = {offset};')
            return

        offset += row.ttf_type.size()

    raise ValueError('unknown field')


print('// This file is autogenerated by scripts/get-tables.py')
print('// Do not edit it!')
print()
print('use crate::Tag;')
print('use crate::parser::{FromData, Offset32};')
print()
generate_table(TTC_HEADER, 'TTCHeader')
print()
generate_table(TABLE_RECORD, 'TableRecord', owned=True, impl_from_data=True)
print()
print('pub mod head {')
generate_table(HEAD_TABLE, 'Table', parse=True)
print('}')
print()
print('pub mod hhea {')
print('use core::num::NonZeroU16;')
print()
generate_table(HHEA_TABLE, 'Table', parse=True)
print('}')
print()
print('pub mod hmtx {')
print('use crate::parser::FromData;')
print()
generate_table(HMTX_METRICS, 'HorizontalMetrics', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod vhea {')
print('use core::num::NonZeroU16;')
print()
generate_table(VHEA_TABLE, 'Table', parse=True)
print('}')
print()
print('pub mod post {')
generate_table(POST_TABLE, 'Table')
print('}')
print()
print('pub mod cmap {')
print('use crate::GlyphId;')
print('use crate::parser::{FromData, Offset32};')
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
table_field_offset(OS_2_TABLE, 'sxHeight')
print()
generate_table(OS_2_TABLE, 'Table')
print('}')
print()
print('pub mod name {')
generate_table(NAME_RECORD_TABLE, 'NameRecord', owned=True)
print('}')
print()
print('pub mod gdef {')
print('use core::ops::RangeInclusive;')
print('use crate::GlyphId;')
print('use crate::parser::FromData;')
print()
generate_table(GDEF_CLASS_RANGE_RECORD, 'ClassRangeRecord', owned=True, impl_from_data=True)
print()
generate_table(GDEF_RANGE_RECORD, 'RangeRecord', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod gsubgpos {')
print('use crate::Tag;')
print('use crate::parser::{Offset16, Offset32, FromData};')
print()
generate_table(GSUB_GPOS_RECORD, 'Record', owned=True, impl_from_data=True)
print()
generate_table(GSUB_GPOS_CONDITION_TABLE, 'Condition', owned=True, impl_from_data=True)
print()
generate_table(GSUB_GPOS_FEATURE_VARIATION_RECORD, 'FeatureVariationRecord', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod fvar {')
print('use crate::Tag;')
print('use crate::parser::FromData;')
print()
generate_table(FVAR_VARIATION_AXIS_RECORD, 'VariationAxisRecord', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod vorg {')
print('use crate::GlyphId;')
print('use crate::parser::FromData;')
print()
generate_table(VERT_ORIGIN_Y_METRICS, 'VertOriginYMetrics', owned=True, impl_from_data=True)
print('}')
print()
print('pub mod mvar {')
print('use crate::Tag;')
print('use crate::parser::FromData;')
print()
generate_table(MVAR_VALUE_RECORD, 'ValueRecord', owned=True, impl_from_data=True)
print()
generate_table(MVAR_REGION_AXIS_COORDINATES_RECORD, 'RegionAxisCoordinatesRecord', owned=True, impl_from_data=True)
print('}')
print()

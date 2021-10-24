mod format0 {
    use ttf_parser::{cmap, GlyphId};

    #[test]
    fn maps_not_all_256_codepoints() {
        let mut data = vec![
            0x00, 0x00, // format: 0
            0x01, 0x06, // subtable size: 262
            0x00, 0x00, // language ID: 0
        ];

        // Map (only) codepoint 0x40 to 100.
        data.extend(std::iter::repeat(0).take(256));
        data[6 + 0x40] = 100;

        let subtable = cmap::Subtable0::parse(&data).unwrap();

        assert_eq!(subtable.glyph_index(0 as char), None);
        assert_eq!(subtable.glyph_index(0x40 as char), Some(GlyphId(100)));
        assert_eq!(subtable.glyph_index(100 as char), None);

        let mut vec = vec![];
        subtable.codepoints(|c| vec.push(c));
        assert_eq!(vec, [0x40]);
    }
}

mod format2 {
    use ttf_parser::{cmap, GlyphId, parser::FromData};

    #[test]
    fn collect_codepoints() {
        let mut data = vec![
            0x00, 0x02, // format: 2
            0x02, 0x16, // subtable size: 534
            0x00, 0x00, // language ID: 0
        ];

        // Make only high byte 0x28 multi-byte.
        data.extend(std::iter::repeat(0x00).take(256 * u16::SIZE));
        data[6 + 0x28 * u16::SIZE + 1] = 0x08;

        data.extend(&[
            // First sub header (for single byte mapping)
            0x00, 0xFE, // first code: 254
            0x00, 0x02, // entry count: 2
            0x00, 0x00, // id delta: uninteresting
            0x00, 0x00, // id range offset: uninteresting
            // Second sub header (for high byte 0x28)
            0x00, 0x10, // first code: (0x28 << 8) + 0x10 = 10256,
            0x00, 0x03, // entry count: 3
            0x00, 0x00, // id delta: uninteresting
            0x00, 0x00, // id range offset: uninteresting
        ]);

        // Now only glyph ID's would follow. Not interesting for codepoints.

        let subtable = cmap::Subtable2::parse(&data).unwrap();

        let mut vec = vec![];
        subtable.codepoints(|c| vec.push(c));
        assert_eq!(vec, [10256, 10257, 10258, 254, 255]);
    }

    #[test]
    fn codepoint_at_range_end() {
        let mut data = vec![
            0x00, 0x02, // format: 2
            0x02, 0x14, // subtable size: 532
            0x00, 0x00, // language ID: 0
        ];

        // Only single bytes.
        data.extend(std::iter::repeat(0x00).take(256 * u16::SIZE));
        data.extend(&[
            // First sub header (for single byte mapping)
            0x00, 0x28, // first code: 40
            0x00, 0x02, // entry count: 2
            0x00, 0x00, // id delta: 0
            0x00, 0x02, // id range offset: 2
            // Glyph index
            0x00, 0x64, // glyph ID [0]: 100
            0x03, 0xE8, // glyph ID [1]: 1000
            0x03, 0xE8, // glyph ID [2]: 10000 (unused)
        ]);

        let subtable = cmap::Subtable2::parse(&data).unwrap();
        assert_eq!(subtable.glyph_index(39 as char), None);
        assert_eq!(subtable.glyph_index(40 as char), Some(GlyphId(100)));
        assert_eq!(subtable.glyph_index(41 as char), Some(GlyphId(1000)));
        assert_eq!(subtable.glyph_index(42 as char), None);
    }
}

mod format4 {
    use ttf_parser::{cmap, GlyphId};

    #[test]
    fn single_glyph() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x41 as char), Some(GlyphId(1)));
        assert_eq!(subtable.glyph_index(0x42 as char), None);
    }

    #[test]
    fn continuous_range() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x49, // char code [0]: 73
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x40 as char), None);
        assert_eq!(subtable.glyph_index(0x41 as char), Some(GlyphId(1)));
        assert_eq!(subtable.glyph_index(0x42 as char), Some(GlyphId(2)));
        assert_eq!(subtable.glyph_index(0x43 as char), Some(GlyphId(3)));
        assert_eq!(subtable.glyph_index(0x44 as char), Some(GlyphId(4)));
        assert_eq!(subtable.glyph_index(0x45 as char), Some(GlyphId(5)));
        assert_eq!(subtable.glyph_index(0x46 as char), Some(GlyphId(6)));
        assert_eq!(subtable.glyph_index(0x47 as char), Some(GlyphId(7)));
        assert_eq!(subtable.glyph_index(0x48 as char), Some(GlyphId(8)));
        assert_eq!(subtable.glyph_index(0x49 as char), Some(GlyphId(9)));
        assert_eq!(subtable.glyph_index(0x4A as char), None);
    }

    #[test]
    fn multiple_ranges() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x30, // subtable size: 48
            0x00, 0x00, // language ID: 0
            0x00, 0x08, // 2 x segCount: 8
            0x00, 0x04, // search range: 4
            0x00, 0x01, // entry selector: 1
            0x00, 0x04, // range shift: 4
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0x00, 0x45, // char code [1]: 69
            0x00, 0x49, // char code [2]: 73
            0xFF, 0xFF, // char code [3]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0x00, 0x43, // char code [1]: 67
            0x00, 0x47, // char code [2]: 71
            0xFF, 0xFF, // char code [3]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0xFF, 0xBF, // delta [1]: -65
            0xFF, 0xBE, // delta [2]: -66
            0x00, 0x01, // delta [3]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
            0x00, 0x00, // offset [2]: 0
            0x00, 0x00, // offset [3]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x40 as char), None);
        assert_eq!(subtable.glyph_index(0x41 as char), Some(GlyphId(1)));
        assert_eq!(subtable.glyph_index(0x42 as char), None);
        assert_eq!(subtable.glyph_index(0x43 as char), Some(GlyphId(2)));
        assert_eq!(subtable.glyph_index(0x44 as char), Some(GlyphId(3)));
        assert_eq!(subtable.glyph_index(0x45 as char), Some(GlyphId(4)));
        assert_eq!(subtable.glyph_index(0x46 as char), None);
        assert_eq!(subtable.glyph_index(0x47 as char), Some(GlyphId(5)));
        assert_eq!(subtable.glyph_index(0x48 as char), Some(GlyphId(6)));
        assert_eq!(subtable.glyph_index(0x49 as char), Some(GlyphId(7)));
        assert_eq!(subtable.glyph_index(0x4A as char), None);
    }

    #[test]
    fn unordered_ids() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x2A, // subtable size: 42
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x45, // char code [0]: 69
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0x00, 0x00, // delta [0]: 0
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x04, // offset [0]: 4
            0x00, 0x00, // offset [1]: 0
            // Glyph index array
            0x00, 0x01, // glyph ID [0]: 1
            0x00, 0x0A, // glyph ID [1]: 10
            0x00, 0x64, // glyph ID [2]: 100
            0x03, 0xE8, // glyph ID [3]: 1000
            0x27, 0x10, // glyph ID [4]: 10000
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x40 as char), None);
        assert_eq!(subtable.glyph_index(0x41 as char), Some(GlyphId(1)));
        assert_eq!(subtable.glyph_index(0x42 as char), Some(GlyphId(10)));
        assert_eq!(subtable.glyph_index(0x43 as char), Some(GlyphId(100)));
        assert_eq!(subtable.glyph_index(0x44 as char), Some(GlyphId(1000)));
        assert_eq!(subtable.glyph_index(0x45 as char), Some(GlyphId(10000)));
        assert_eq!(subtable.glyph_index(0x46 as char), None);
    }

    #[test]
    fn unordered_chars_and_ids() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x40, // subtable size: 64
            0x00, 0x00, // language ID: 0
            0x00, 0x0C, // 2 x segCount: 12
            0x00, 0x08, // search range: 8
            0x00, 0x02, // entry selector: 2
            0x00, 0x04, // range shift: 4
            // End character codes
            0x00, 0x50, // char code [0]: 80
            0x01, 0x00, // char code [1]: 256
            0x01, 0x50, // char code [2]: 336
            0x02, 0x00, // char code [3]: 512
            0x02, 0x50, // char code [4]: 592
            0xFF, 0xFF, // char code [5]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x50, // char code [0]: 80
            0x01, 0x00, // char code [1]: 256
            0x01, 0x50, // char code [2]: 336
            0x02, 0x00, // char code [3]: 512
            0x02, 0x50, // char code [4]: 592
            0xFF, 0xFF, // char code [5]: 65535
            // Deltas
            0xFF, 0xB1, // delta [0]: -79
            0xFF, 0x0A, // delta [1]: -246
            0xFF, 0x14, // delta [2]: -236
            0x01, 0xE8, // delta [3]: 488
            0x24, 0xC0, // delta [4]: 9408
            0x00, 0x01, // delta [5]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
            0x00, 0x00, // offset [2]: 0
            0x00, 0x00, // offset [3]: 0
            0x00, 0x00, // offset [4]: 0
            0x00, 0x00, // offset [5]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x40 as char),  None);
        assert_eq!(subtable.glyph_index(0x50 as char),  Some(GlyphId(1)));
        assert_eq!(subtable.glyph_index(std::char::from_u32(0x100).unwrap()), Some(GlyphId(10)));
        assert_eq!(subtable.glyph_index(std::char::from_u32(0x150).unwrap()), Some(GlyphId(100)));
        assert_eq!(subtable.glyph_index(std::char::from_u32(0x200).unwrap()), Some(GlyphId(1000)));
        assert_eq!(subtable.glyph_index(std::char::from_u32(0x250).unwrap()), Some(GlyphId(10000)));
        assert_eq!(subtable.glyph_index(std::char::from_u32(0x300).unwrap()), None);
    }

    #[test]
    fn no_end_codes() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 28
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x49, // char code [0]: 73
            // 0xFF, 0xFF, // char code [1]: 65535 <-- removed
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            // 0xFF, 0xFF, // char code [1]: 65535 <-- removed
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert!(cmap::Subtable4::parse(data).is_none());
    }

    #[test]
    fn invalid_segment_count() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x01, // 2 x segCount: 1 <-- must be more than 1
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert!(cmap::Subtable4::parse(data).is_none());
    }

    #[test]
    fn only_end_segments() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x02, // 2 x segCount: 2
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        // Should not loop forever.
        assert_eq!(subtable.glyph_index(0x41 as char), None);
    }

    #[test]
    fn invalid_length() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x10, // subtable size: 16 <-- the size should be 32, but we don't check it anyway
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x41 as char), Some(GlyphId(1)));
        assert_eq!(subtable.glyph_index(0x42 as char), None);
    }

    #[test]
    fn codepoint_out_of_range() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        // Format 4 support only u16 codepoints, so we have to bail immediately otherwise.
        assert_eq!(subtable.glyph_index(std::char::from_u32(0x1FFFF).unwrap()), None);
    }

    #[test]
    fn zero() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x2A, // subtable size: 42
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x45, // char code [0]: 69
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0x00, 0x00, // delta [0]: 0
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x04, // offset [0]: 4
            0x00, 0x00, // offset [1]: 0
            // Glyph index array
            0x00, 0x00, // glyph ID [0]: 0 <-- indicates missing glyph
            0x00, 0x0A, // glyph ID [1]: 10
            0x00, 0x64, // glyph ID [2]: 100
            0x03, 0xE8, // glyph ID [3]: 1000
            0x27, 0x10, // glyph ID [4]: 10000
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();
        assert_eq!(subtable.glyph_index(0x41 as char), None);
    }

    #[test]
    fn collect_codepoints() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x18, // subtable size: 24
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x22, // char code [0]: 34
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x1B, // char code [0]: 27
            0xFF, 0xFD, // char code [1]: 65533
            // Deltas
            0x00, 0x00, // delta [0]: 0
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x04, // offset [0]: 4
            0x00, 0x00, // offset [1]: 0
            // Glyph index array
            0x00, 0x00, // glyph ID [0]: 0
            0x00, 0x0A, // glyph ID [1]: 10
        ];

        let subtable = cmap::Subtable4::parse(data).unwrap();

        let mut vec = vec![];
        subtable.codepoints(|c| vec.push(c));
        assert_eq!(vec, [27, 28, 29, 30, 31, 32, 33, 34, 65533, 65534, 65535]);
    }
}

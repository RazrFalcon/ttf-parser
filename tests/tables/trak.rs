use ttf_parser::trak::Table;

#[test]
fn empty() {
    let data = &[
        0x00, 0x01, 0x00, 0x00, // version: 1
        0x00, 0x00, // format: 0
        0x00, 0x00, // horizontal data offset: NULL
        0x00, 0x00, // vertical data offset: NULL
        0x00, 0x00, // padding
    ];

    let table = Table::parse(data).unwrap();
    assert_eq!(table.horizontal.tracks.len(), 0);
    assert_eq!(table.horizontal.sizes.len(), 0);
    assert_eq!(table.vertical.tracks.len(), 0);
    assert_eq!(table.vertical.sizes.len(), 0);
}

#[test]
fn basic() {
    let data = &[
        0x00, 0x01, 0x00, 0x00, // version: 1
        0x00, 0x00, // format: 0
        0x00, 0x0C, // horizontal data offset: 12
        0x00, 0x00, // vertical data offset: NULL
        0x00, 0x00, // padding

        // TrackData
        0x00, 0x03, // number of tracks: 3
        0x00, 0x02, // number of sizes: 2
        0x00, 0x00, 0x00, 0x2C, // offset to size table: 44

        // TrackTableEntry [0]
        0xFF, 0xFF, 0x00, 0x00, // track: -1
        0x01, 0x00, // name index: 256
        0x00, 0x34, // offset of the two per-size tracking values: 52

        // TrackTableEntry [1]
        0x00, 0x00, 0x00, 0x00, // track: 0
        0x01, 0x02, // name index: 258
        0x00, 0x3C, // offset of the two per-size tracking values: 60

        // TrackTableEntry [2]
        0x00, 0x01, 0x00, 0x00, // track: 1
        0x01, 0x01, // name index: 257
        0x00, 0x38, // offset of the two per-size tracking values: 56

        // Size [0]
        0x00, 0x0C, 0x00, 0x00, // points: 12
        // Size [1]
        0x00, 0x18, 0x00, 0x00, // points: 24

        // Per-size tracking values.
        0xFF, 0xF1, // -15
        0xFF, 0xF9, // -7
        0x00, 0x32, // 50
        0x00, 0x14, // 20
        0x00, 0x00, // 0
        0x00, 0x00, // 0
    ];

    let table = Table::parse(data).unwrap();

    assert_eq!(table.horizontal.tracks.len(), 3);
    assert_eq!(table.horizontal.tracks.get(0).unwrap().value, -1.0);
    assert_eq!(table.horizontal.tracks.get(1).unwrap().value, 0.0);
    assert_eq!(table.horizontal.tracks.get(2).unwrap().value, 1.0);
    assert_eq!(table.horizontal.tracks.get(0).unwrap().name_index, 256);
    assert_eq!(table.horizontal.tracks.get(1).unwrap().name_index, 258);
    assert_eq!(table.horizontal.tracks.get(2).unwrap().name_index, 257);
    assert_eq!(table.horizontal.tracks.get(0).unwrap().values.len(), 2);
    assert_eq!(table.horizontal.tracks.get(0).unwrap().values.get(0).unwrap(), -15);
    assert_eq!(table.horizontal.tracks.get(0).unwrap().values.get(1).unwrap(), -7);
    assert_eq!(table.horizontal.tracks.get(1).unwrap().values.len(), 2);
    assert_eq!(table.horizontal.tracks.get(1).unwrap().values.get(0).unwrap(), 0);
    assert_eq!(table.horizontal.tracks.get(1).unwrap().values.get(1).unwrap(), 0);
    assert_eq!(table.horizontal.tracks.get(2).unwrap().values.len(), 2);
    assert_eq!(table.horizontal.tracks.get(2).unwrap().values.get(0).unwrap(), 50);
    assert_eq!(table.horizontal.tracks.get(2).unwrap().values.get(1).unwrap(), 20);
    assert_eq!(table.horizontal.sizes.len(), 2);
    assert_eq!(table.horizontal.sizes.get(0).unwrap().0, 12.0);
    assert_eq!(table.horizontal.sizes.get(1).unwrap().0, 24.0);

    assert_eq!(table.vertical.tracks.len(), 0);
    assert_eq!(table.vertical.sizes.len(), 0);
}

use ttf_parser::feat::Table;

#[test]
fn basic() {
    let data = &[
        0x00, 0x01, 0x00, 0x00, // version: 1
        0x00, 0x04, // number of features: 4
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // reserved

        // Feature Name [0]
        0x00, 0x00, // feature: 0
        0x00, 0x01, // number of settings: 1
        0x00, 0x00, 0x00, 0x3C, // offset to settings table: 60
        0x00, 0x00, // flags: none
        0x01, 0x04, // name index: 260

        // Feature Name [1]
        0x00, 0x01, // feature: 1
        0x00, 0x01, // number of settings: 1
        0x00, 0x00, 0x00, 0x40, // offset to settings table: 64
        0x00, 0x00, // flags: none
        0x01, 0x00, // name index: 256

        // Feature Name [2]
        0x00, 0x03, // feature: 3
        0x00, 0x03, // number of settings: 3
        0x00, 0x00, 0x00, 0x44, // offset to settings table: 68
        0x80, 0x00, // flags: exclusive
        0x01, 0x06, // name index: 262

        // Feature Name [3]
        0x00, 0x06, // feature: 6
        0x00, 0x01, // number of settings: 2
        0x00, 0x00, 0x00, 0x50, // offset to settings table: 80
        0xC0, 0x01, // flags: exclusive and other
        0x01, 0x02, // name index: 258

        // Setting Name [0]
        0x00, 0x00, // setting: 0
        0x01, 0x05, // name index: 261

        // Setting Name [1]
        0x00, 0x02, // setting: 2
        0x01, 0x01, // name index: 257

        // Setting Name [2]
        0x00, 0x00, // setting: 0
        0x01, 0x0C, // name index: 268
        0x00, 0x03, // setting: 3
        0x01, 0x08, // name index: 264
        0x00, 0x04, // setting: 4
        0x01, 0x09, // name index: 265

        // Setting Name [3]
        0x00, 0x00, // setting: 0
        0x01, 0x03, // name index: 259
        0x00, 0x00, // setting: 1
        0x01, 0x04, // name index: 260
    ];

    let table = Table::parse(data).unwrap();
    assert_eq!(table.names.len(), 4);

    let feature0 = table.names.get(0).unwrap();
    assert_eq!(feature0.feature, 0);
    assert_eq!(feature0.setting_names.len(), 1);
    assert_eq!(feature0.exclusive, false);
    assert_eq!(feature0.name_index, 260);

    let feature2 = table.names.get(2).unwrap();
    assert_eq!(feature2.feature, 3);
    assert_eq!(feature2.setting_names.len(), 3);
    assert_eq!(feature2.exclusive, true);

    assert_eq!(feature2.setting_names.get(1).unwrap().setting, 3);
    assert_eq!(feature2.setting_names.get(1).unwrap().name_index, 264);

    let feature3 = table.names.get(3).unwrap();
    assert_eq!(feature3.default_setting_index, 1);
    assert_eq!(feature3.exclusive, true);
}

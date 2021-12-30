use ttf_parser::feat::Table;
use crate::{convert, Unit::*};

#[test]
fn basic() {
    let data = convert(&[
        Fixed(1.0), // version: 1
        UInt16(4), // number of features: 4
        UInt16(0), // reserved
        UInt32(0), // reserved

        // Feature Name [0]
        UInt16(0), // feature: 0
        UInt16(1), // number of settings: 1
        UInt32(60), // offset to settings table: 60
        UInt16(0), // flags: none
        UInt16(260), // name index: 260

        // Feature Name [1]
        UInt16(1), // feature: 1
        UInt16(1), // number of settings: 1
        UInt32(64), // offset to settings table: 64
        UInt16(0), // flags: none
        UInt16(256), // name index: 256

        // Feature Name [2]
        UInt16(3), // feature: 3
        UInt16(3), // number of settings: 3
        UInt32(68), // offset to settings table: 68
        Raw(&[0x80, 0x00]), // flags: exclusive
        UInt16(262), // name index: 262

        // Feature Name [3]
        UInt16(6), // feature: 6
        UInt16(2), // number of settings: 2
        UInt32(80), // offset to settings table: 80
        Raw(&[0xC0, 0x01]), // flags: exclusive and other
        UInt16(258), // name index: 258

        // Setting Name [0]
        UInt16(0), // setting: 0
        UInt16(261), // name index: 261

        // Setting Name [1]
        UInt16(2), // setting: 2
        UInt16(257), // name index: 257

        // Setting Name [2]
        UInt16(0), // setting: 0
        UInt16(268), // name index: 268
        UInt16(3), // setting: 3
        UInt16(264), // name index: 264
        UInt16(4), // setting: 4
        UInt16(265), // name index: 265

        // Setting Name [3]
        UInt16(0), // setting: 0
        UInt16(259), // name index: 259
        UInt16(1), // setting: 1
        UInt16(260), // name index: 260
    ]);

    let table = Table::parse(&data).unwrap();
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

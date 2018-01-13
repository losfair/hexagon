use super::page_table;

#[test]
fn test_read_write() {
    let mut pt = page_table::PageTable::new();
    assert_eq!(pt.write_u64(0x00100000, 42), false);
    assert_eq!(pt.write_u64(0x00200000, 192), false);

    assert_eq!(pt.virtual_alloc(0x00100000), true);

    assert_eq!(pt.write_u64(0x00100000, 42), true);
    assert_eq!(pt.write_u64(0x00200000, 192), false);

    assert_eq!(pt.write_u64(0x00100008, 7889315787603), true);
    assert_eq!(pt.read_u64(0x00100000), Some(42));
    assert_eq!(pt.read_u64(0x00100008), Some(7889315787603));

    assert_eq!(pt.read_u64(0x001ffff8), Some(0));
    assert_eq!(pt.read_u64(0x001ffff9), None);

    assert_eq!(pt.virtual_alloc(0x00200000), true);

    assert_eq!(pt.write_u64(0x00200000, 192), true);
    assert_eq!(pt.read_u64(0x00100000), Some(42));
    assert_eq!(pt.read_u64(0x00200000), Some(192));
}

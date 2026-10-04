use super::*;

#[test]
fn chip_orders_keep_label_below_chevron() {
    let surface = 8;
    let label = label_order(surface);
    let chevron = chevron_order(surface);

    assert_eq!(label, 10);
    assert_eq!(chevron, 11);
    assert!(surface < label);
    assert!(label < chevron);
}

use super::*;

#[test]
fn scale_link_connector_paints_above_lobes() {
    let lobes = 50;

    assert!(lobes < scale_link_connector_order(lobes));
}

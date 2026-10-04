use super::*;

#[test]
fn receipt_identifiers_are_bounded_and_keep_a_hash_suffix() {
    let asset_id = "asset/".repeat(100);
    let binding_id = "binding/".repeat(100);

    let receipt = UiBindingExecutionReceipt::executed(&asset_id, &binding_id, 7, false, 11);

    assert!(receipt.asset_id.len() <= UI_BINDING_TELEMETRY_ASSET_ID_MAX_BYTES);
    assert!(receipt.binding_id.len() <= UI_BINDING_TELEMETRY_BINDING_ID_MAX_BYTES);
    assert!(receipt.asset_id.contains('~'));
    assert!(receipt.binding_id.contains('~'));
}

use super::*;

#[test]
fn inventory_move_round_trips_indices() {
    let payload = InventoryMovePayload { from: 2, to: 17 };
    let bytes = payload.encode();

    assert_eq!(bytes, [2, 0, 0, 0, 17, 0, 0, 0]);
    assert_eq!(InventoryMovePayload::decode(&bytes), Ok(payload));
}

#[test]
fn inventory_move_rejects_noncanonical_length() {
    assert!(InventoryMovePayload::decode(&[0; 7]).is_err());
    assert!(InventoryMovePayload::decode(&[0; 9]).is_err());
}

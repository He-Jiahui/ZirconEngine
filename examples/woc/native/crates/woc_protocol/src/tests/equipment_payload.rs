use super::*;

#[test]
fn roundtrips_automatic_and_aimed_equips() {
    for slot in [
        None,
        Some(EquipmentSlot::Helmet),
        Some(EquipmentSlot::Ring2),
    ] {
        let payload = EquipItemPayload {
            item_id: "cryptbone_helm".to_owned(),
            slot,
        };
        let encoded = payload.clone().encode().expect("payload encodes");
        assert_eq!(EquipItemPayload::decode(&encoded), Ok(payload));
    }
}

#[test]
fn unequip_requires_a_live_equipment_slot() {
    assert!(matches!(
        UnequipItemPayload::decode(&[0]),
        Err(ProtocolError::InvalidEquipmentSlot(0))
    ));
    assert_eq!(
        UnequipItemPayload::decode(&[EquipmentSlot::Feet.wire_code()]),
        Ok(UnequipItemPayload {
            slot: EquipmentSlot::Feet,
        })
    );
}

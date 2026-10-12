use super::SlotLease;

#[test]
fn graphics_surface_slot_lease_restores_value_on_drop() {
    let mut slot = Some(7);
    {
        let mut lease = SlotLease::take(&mut slot).expect("slot has value");
        *lease.value_mut() = 11;
    }

    assert_eq!(slot, Some(11));
}

#[test]
fn graphics_surface_slot_lease_restores_value_on_explicit_restore() {
    let mut slot = Some(3);
    let mut lease = SlotLease::take(&mut slot).expect("slot has value");
    *lease.value_mut() = 5;

    lease.restore();

    assert_eq!(slot, Some(5));
}

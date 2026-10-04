use super::*;
use zr_rhi::{
    DeviceGeneration, DeviceId, DiagnosticReadbackBudget, DiagnosticReadbackKind,
    DiagnosticReadbackTracker,
};

#[test]
fn product_router_moves_delivery_bytes_and_runs_callbacks_outside_the_router() {
    let source = include_str!("../product_diagnostic_delivery_router.rs");
    let collection = source
        .split("pub(super) fn collect_dispatches")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn record_callback_panic").next())
        .expect("router collection method");

    assert!(collection.contains("append_diagnostic_readback_deliveries"));
    assert!(collection.contains("self.pending.remove(&receipt.request())"));
    assert!(collection.contains("delivery.into_bytes()"));
    assert!(!collection.contains("callback)("));
}

#[test]
fn full_router_rejects_new_registration_without_orphaning_existing_owner() {
    let budget = DiagnosticReadbackBudget::default();
    let mut tracker =
        DiagnosticReadbackTracker::new(DeviceId::new(7), DeviceGeneration::initial(), budget);
    tracker.begin_frame(1).unwrap();
    let first_request = tracker
        .admit(DiagnosticReadbackKind::Buffer, 4)
        .expect("first request must be admitted");
    let second_request = tracker
        .admit(DiagnosticReadbackKind::Buffer, 4)
        .expect("second request must be admitted");
    let mut router = ProductDiagnosticDeliveryRouter::new(1);

    router
        .register(
            DiagnosticReadbackAdmission::Admitted(first_request),
            Box::new(|_| {}),
        )
        .unwrap();
    let error = router
        .register(
            DiagnosticReadbackAdmission::Admitted(second_request),
            Box::new(|_| {}),
        )
        .expect_err("full router must fail instead of dropping an unresolved callback");

    assert!(error.contains("pending-request budget"));
    assert!(router.pending.contains_key(&first_request));
    assert!(!router.pending.contains_key(&second_request));
    assert_eq!(router.orphan_delivery_count, 0);
}

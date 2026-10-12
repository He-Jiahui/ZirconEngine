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

#[test]
fn typed_dispatch_retains_the_actual_terminal_receipt_and_submission() {
    for terminal in [
        DiagnosticReadbackTerminal::Succeeded,
        DiagnosticReadbackTerminal::Shutdown,
    ] {
        let mut tracker = DiagnosticReadbackTracker::new(
            DeviceId::new(7),
            DeviceGeneration::initial(),
            DiagnosticReadbackBudget::default(),
        );
        tracker.begin_frame(41).unwrap();
        let request = tracker.admit(DiagnosticReadbackKind::Texture, 16).unwrap();
        let ticket = zr_rhi::SubmissionTicket::new(
            DeviceId::new(7),
            DeviceGeneration::initial(),
            zr_rhi::RenderQueueClass::Graphics,
            9,
        );
        tracker.bind_active_frame(ticket).unwrap();
        let receipt = tracker.terminalize(request, terminal).unwrap();
        let bytes = (terminal == DiagnosticReadbackTerminal::Succeeded).then(|| vec![1, 2, 3, 4]);
        let (send, receive) = std::sync::mpsc::channel();
        let mut router = ProductDiagnosticDeliveryRouter::new(1);
        assert!(router
            .register_with_receipt(
                DiagnosticReadbackAdmission::Admitted(request),
                Box::new(move |actual, payload| send.send((actual, payload)).unwrap()),
            )
            .unwrap());
        let callback = router.pending.remove(&request).unwrap();
        let dispatch =
            ProductDiagnosticDispatch::new_with_receipt(callback, receipt, bytes.clone());
        assert!(!dispatch.run());
        let (actual, payload) = receive.try_recv().unwrap();
        assert_eq!(actual, receipt);
        assert_eq!(actual.frame_key().unwrap().submission(), ticket);
        assert_eq!(actual.terminal(), terminal);
        assert_eq!(payload, bytes);
    }
}

#[test]
fn cancelling_one_receipt_callback_preserves_the_other_registered_owner() {
    let mut tracker = DiagnosticReadbackTracker::new(
        DeviceId::new(7),
        DeviceGeneration::initial(),
        DiagnosticReadbackBudget::default(),
    );
    tracker.begin_frame(41).unwrap();
    let first = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let second = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let mut router = ProductDiagnosticDeliveryRouter::new(2);
    let (first_send, first_receive) = std::sync::mpsc::channel();
    let (second_send, second_receive) = std::sync::mpsc::channel();
    router
        .register_with_receipt(
            DiagnosticReadbackAdmission::Admitted(first),
            Box::new(move |receipt, _| first_send.send(receipt).unwrap()),
        )
        .unwrap();
    router
        .register_with_receipt(
            DiagnosticReadbackAdmission::Admitted(second),
            Box::new(move |receipt, _| second_send.send(receipt).unwrap()),
        )
        .unwrap();
    router.cancel_receipt_callback(first);
    assert!(!router.pending.contains_key(&first));
    assert!(router.pending.contains_key(&second));
    assert_eq!(router.pending.len(), 1);
    assert_eq!(router.orphan_delivery_count, 0);
    assert!(matches!(
        first_receive.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
    let receipt = tracker
        .terminalize(second, DiagnosticReadbackTerminal::Cancelled)
        .unwrap();
    let callback = router.pending.remove(&second).unwrap();
    assert!(!ProductDiagnosticDispatch::new_with_receipt(callback, receipt, None).run());
    assert_eq!(second_receive.try_recv().unwrap(), receipt);
}

#[test]
fn central_dispatch_allows_router_reentry_and_accounts_callback_panics() {
    let mut tracker = DiagnosticReadbackTracker::new(
        DeviceId::new(7),
        DeviceGeneration::initial(),
        DiagnosticReadbackBudget::default(),
    );
    tracker.begin_frame(41).unwrap();
    let first = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let second = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let third = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
    let router = std::sync::Arc::new(std::sync::Mutex::new(ProductDiagnosticDeliveryRouter::new(
        3,
    )));
    let reentrant_router = std::sync::Arc::clone(&router);
    let (send, receive) = std::sync::mpsc::channel();
    {
        let mut locked = router.lock().unwrap();
        locked
            .register_with_receipt(
                DiagnosticReadbackAdmission::Admitted(first),
                Box::new(move |receipt, _| {
                    let mut locked = reentrant_router
                        .try_lock()
                        .expect("callback must run without router lock");
                    assert!(locked
                        .register_with_receipt(
                            DiagnosticReadbackAdmission::Admitted(third),
                            Box::new(|_, _| {})
                        )
                        .unwrap());
                    send.send(receipt).unwrap();
                }),
            )
            .unwrap();
        locked
            .register_with_receipt(
                DiagnosticReadbackAdmission::Admitted(second),
                Box::new(|_, _| panic!("intentional diagnostic callback panic")),
            )
            .unwrap();
    }
    let first_receipt = tracker
        .terminalize(first, DiagnosticReadbackTerminal::Cancelled)
        .unwrap();
    let second_receipt = tracker
        .terminalize(second, DiagnosticReadbackTerminal::Shutdown)
        .unwrap();
    let dispatches = {
        let mut locked = router.lock().unwrap();
        vec![
            ProductDiagnosticDispatch::new_with_receipt(
                locked.pending.remove(&first).unwrap(),
                first_receipt,
                None,
            ),
            ProductDiagnosticDispatch::new_with_receipt(
                locked.pending.remove(&second).unwrap(),
                second_receipt,
                None,
            ),
        ]
    };
    super::super::ui_surface_diagnostic_completion::run_dispatches(router.as_ref(), dispatches);
    assert_eq!(receive.try_recv().unwrap(), first_receipt);
    let locked = router.lock().unwrap();
    assert_eq!(locked.callback_panic_count, 1);
    assert!(locked.pending.contains_key(&third));
    assert_eq!(locked.pending.len(), 1);
    assert_eq!(locked.orphan_delivery_count, 0);
}

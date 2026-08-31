use super::*;

#[test]
fn platform_preference_storage_completed_small_reads_release_maximum_overlay_quotes() {
    let backend = Arc::new(MemoryBackend::default());
    let first_key = PreferenceKey::new("woc.input", "account-a").unwrap();
    let second_key = PreferenceKey::new("woc.input", "account-b").unwrap();
    backend
        .values
        .lock()
        .unwrap()
        .insert(first_key.clone(), Arc::from(&b"a"[..]));
    backend
        .values
        .lock()
        .unwrap()
        .insert(second_key.clone(), Arc::from(&b"b"[..]));
    let adapter = test_adapter(
        backend,
        PreferencePersistenceLimits {
            max_value_bytes: 1024,
            max_overlay_entries: 2,
            max_overlay_retained_bytes: 3400,
            max_lane_entries: 2,
            max_lane_retained_bytes: 64 * 1024,
        },
    );

    assert_eq!(
        wait_snapshot_terminal(&adapter, &first_key),
        PreferenceMutationTerminal::Durable
    );
    assert_eq!(
        adapter
            .snapshot(&second_key)
            .expect("the first small read must release its maximum reservation")
            .durability(),
        PreferenceDurabilityState::Pending
    );
    assert_eq!(
        wait_snapshot_terminal(&adapter, &second_key),
        PreferenceMutationTerminal::Durable
    );
    let diagnostics = adapter.diagnostics().overlay;
    assert_eq!(diagnostics.durable, 2);
    assert!(diagnostics.retained_bytes < 3400);
}

#[test]
fn platform_preference_storage_fence_preserves_backend_failure_kind() {
    for kind in [
        PreferenceStorageErrorKind::Denied,
        PreferenceStorageErrorKind::CapacityExceeded,
    ] {
        let backend = Arc::new(MemoryBackend::default());
        *backend.write_failure_kind.lock().unwrap() = Some(kind);
        let adapter = adapter(backend, 64);
        let key = PreferenceKey::new("woc.input", format!("failure-{kind:?}")).unwrap();
        let failed = adapter
            .submit_write(
                key,
                Arc::from(&b"value"[..]),
                PreferenceWorkDeadline::none(),
            )
            .unwrap();
        assert!(matches!(
            failed
                .ticket()
                .wait_until(Instant::now() + Duration::from_secs(2)),
            PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
        ));

        let fence = adapter.flush_fence(PreferenceWorkDeadline::none()).unwrap();
        let PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(failure)) =
            fence.wait_until(Instant::now() + Duration::from_secs(2))
        else {
            panic!("pre-fence backend failure must fail the fence");
        };
        assert_eq!(failure.kind(), kind);
    }
}

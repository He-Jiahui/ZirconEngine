use super::*;

#[test]
fn session_guard_persists_heartbeat_and_requires_explicit_residual_takeover() {
    let root = temporary_root("session-guard");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let mut guard = claim_acquired(&root, start);
    let initial = guard.record().clone();
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == initial
    ));
    assert!(matches!(
        SessionGuard::claim_at(&root, &test_session_admission(), start).unwrap(),
        SessionGuardAdmission::Active { .. }
    ));

    assert_eq!(
        guard
            .refresh_heartbeat_at(start + Duration::from_secs(1))
            .unwrap(),
        expected_lock_durability()
    );
    assert_eq!(guard.record().heartbeat_unix_millis(), 11_000);
    assert_eq!(guard.release().unwrap(), expected_lock_durability());
    assert!(guard.is_released());
    assert_eq!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Missing
    );
    remove_temporary_root(&root);
}

#[test]
fn session_guard_commits_a_generation_only_after_activation() {
    let root = temporary_root("session-guard-admission-lifecycle");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let mut guard = claim_acquired(&root, start);

    assert_eq!(
        guard.record().lifecycle(),
        ProjectSessionAdmissionLifecycleV1::Claimed
    );
    assert_eq!(guard.record().session_generation(), None);
    guard.mark_preflight_approved().unwrap();
    guard.begin_activation().unwrap();
    assert_eq!(
        guard.record().lifecycle(),
        ProjectSessionAdmissionLifecycleV1::Activating
    );
    assert_eq!(guard.record().session_generation(), None);

    guard.commit_ready().unwrap();
    assert_eq!(
        guard.record().lifecycle(),
        ProjectSessionAdmissionLifecycleV1::Ready
    );
    assert!(guard.record().session_generation().is_some());
    assert_eq!(guard.record().checked_epoch(), 4);

    guard.release().unwrap();
    remove_temporary_root(&root);
}

#[test]
fn session_guard_recovery_defer_preserves_the_record_after_releasing_ownership() {
    let root = temporary_root("session-guard-recovery-defer");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let mut guard = claim_acquired(&root, start);
    guard.mark_preflight_approved().unwrap();
    guard.begin_activation().unwrap();
    guard.commit_ready().unwrap();

    assert_eq!(
        guard.release_ownership_for_recovery().unwrap(),
        expected_lock_durability()
    );
    assert!(guard.is_released());
    assert_eq!(
        guard.record().lifecycle(),
        ProjectSessionAdmissionLifecycleV1::RecoveryRequired
    );
    let expected = guard.record().clone();
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == expected
    ));

    let claim = SessionGuard::claim(&root, &test_session_admission()).unwrap();
    assert!(matches!(
        claim,
        SessionGuardAdmission::Residual(ref residual) if residual.record() == &expected
    ));
    drop(claim);

    let mut cleanup = SessionGuard::replace_residual_at(
        &root,
        &expected,
        &test_session_admission(),
        std::time::SystemTime::now(),
    )
    .unwrap();
    cleanup.release().unwrap();
    remove_temporary_root(&root);
}

#[test]
fn session_guard_claim_reports_the_active_record_without_transferring_ownership() {
    let root = temporary_root("session-guard-active-claim");
    let mut guard = claim_acquired(&root, std::time::SystemTime::now());
    let expected = guard.record().clone();

    assert!(matches!(
        SessionGuard::claim(&root, &test_session_admission()).unwrap(),
        SessionGuardAdmission::Active {
            record: Some(record)
        } if record == expected
    ));
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == expected
    ));

    guard.release().unwrap();
    remove_temporary_root(&root);
}

#[test]
fn session_guard_claim_holds_a_residual_record_until_explicit_takeover() {
    let root = temporary_root("session-guard-residual-claim");
    let guard = claim_acquired(&root, std::time::SystemTime::now());
    let expected = guard.record().clone();
    drop(guard);

    let claim = SessionGuard::claim(&root, &test_session_admission()).unwrap();
    assert!(matches!(
        claim,
        SessionGuardAdmission::Residual(ref residual) if residual.record() == &expected
    ));
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == expected
    ));

    drop(claim);
    let mut replacement = SessionGuard::replace_residual_at(
        &root,
        &expected,
        &test_session_admission(),
        std::time::SystemTime::now(),
    )
    .unwrap();
    replacement.release().unwrap();
    remove_temporary_root(&root);
}

#[cfg(any(windows, unix))]
#[test]
fn session_guard_uses_the_physical_project_identity_for_directory_aliases() {
    let root = temporary_root("session-guard-project-alias");
    let physical = root.join("physical-project");
    let alias = root.join("project-alias");
    fs::create_dir_all(&physical).unwrap();
    create_project_directory_alias(&physical, &alias);

    let mut guard = claim_acquired(&alias, std::time::SystemTime::now());
    assert_eq!(
        guard.path(),
        physical.join(".zircon").join("session.lock").as_path()
    );
    assert!(matches!(
        SessionGuard::inspect(&physical).unwrap(),
        SessionLockInspection::Residual(_)
    ));
    guard.release().unwrap();
    remove_temporary_root(&root);
}

#[test]
fn residual_takeover_failure_keeps_the_selected_lock() {
    let root = temporary_root("session-guard-takeover-failure");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let guard = claim_acquired(&root, start);
    let expected = guard.record().clone();
    drop(guard);

    assert!(matches!(
        SessionGuard::replace_residual_at(
            &root,
            &expected,
            &test_session_admission(),
            std::time::UNIX_EPOCH - Duration::from_secs(1),
        ),
        Err(SessionGuardError::ClockBeforeUnixEpoch)
    ));
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == expected
    ));
    remove_temporary_root(&root);
}

#[test]
fn residual_takeover_persists_a_new_guard_record() {
    let root = temporary_root("session-guard-takeover-success");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let guard = claim_acquired(&root, start);
    let expected = guard.record().clone();
    drop(guard);

    let mut replacement = SessionGuard::replace_residual_at(
        &root,
        &expected,
        &test_session_admission(),
        start + Duration::from_secs(1),
    )
    .unwrap();
    let replacement_record = replacement.record().clone();
    assert_ne!(replacement_record, expected);
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == replacement_record
    ));

    assert_eq!(replacement.release().unwrap(), expected_lock_durability());
    remove_temporary_root(&root);
}

#[test]
fn concurrent_residual_takeover_keeps_exactly_one_live_guard() {
    let root = temporary_root("session-guard-concurrent-takeover");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let guard = claim_acquired(&root, start);
    let expected = Arc::new(guard.record().clone());
    drop(guard);

    let start_takeover = Arc::new(Barrier::new(3));
    let hold_results = Arc::new(Barrier::new(3));
    let (sender, receiver) = mpsc::channel();
    let first_root = root.clone();
    let first_expected = Arc::clone(&expected);
    let first_start_takeover = Arc::clone(&start_takeover);
    let first_hold_results = Arc::clone(&hold_results);
    let first_sender = sender.clone();
    let first = thread::spawn(move || {
        first_start_takeover.wait();
        let admission = test_session_admission();
        let result = SessionGuard::replace_residual_at(
            &first_root,
            &first_expected,
            &admission,
            start + Duration::from_secs(1),
        );
        first_sender.send(result.is_ok()).unwrap();
        first_hold_results.wait();
    });
    let second_root = root.clone();
    let second_expected = Arc::clone(&expected);
    let second_start_takeover = Arc::clone(&start_takeover);
    let second_hold_results = Arc::clone(&hold_results);
    let second = thread::spawn(move || {
        second_start_takeover.wait();
        let admission = test_session_admission();
        let result = SessionGuard::replace_residual_at(
            &second_root,
            &second_expected,
            &admission,
            start + Duration::from_secs(2),
        );
        sender.send(result.is_ok()).unwrap();
        second_hold_results.wait();
    });

    start_takeover.wait();
    let successes = [receiver.recv().unwrap(), receiver.recv().unwrap()]
        .into_iter()
        .filter(|success| *success)
        .count();
    assert_eq!(successes, 1);
    hold_results.wait();
    first.join().unwrap();
    second.join().unwrap();
    remove_temporary_root(&root);
}

#[test]
fn live_guard_rejects_takeover_before_heartbeat_and_release() {
    let root = temporary_root("session-guard-live-owner");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let residual = claim_acquired(&root, start);
    let expected = residual.record().clone();
    drop(residual);

    let mut guard = SessionGuard::replace_residual_at(
        &root,
        &expected,
        &test_session_admission(),
        start + Duration::from_secs(1),
    )
    .unwrap();
    let successor = guard.record().clone();
    assert!(matches!(
        SessionGuard::replace_residual_at(
            &root,
            &expected,
            &test_session_admission(),
            start + Duration::from_secs(2),
        ),
        Err(SessionGuardError::AlreadyHeld { .. })
    ));
    assert_eq!(
        guard
            .refresh_heartbeat_at(start + Duration::from_secs(3))
            .unwrap(),
        expected_lock_durability()
    );
    assert_eq!(guard.release().unwrap(), expected_lock_durability());
    assert!(matches!(
        SessionGuard::replace_residual_at(
            &root,
            &successor,
            &test_session_admission(),
            start + Duration::from_secs(4),
        ),
        Err(SessionGuardError::OwnershipLost { .. })
    ));
    remove_temporary_root(&root);
}

#[test]
fn session_guard_rejects_duplicate_persisted_record_fields() {
    let root = temporary_root("session-guard-duplicate-fields");
    let directory = root.join(".zircon");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("session.lock"),
        "version=1\nprocess_id=1\nprocess_id=2\ninstance_id=1-10-1\nheartbeat_unix_millis=10000\n",
    )
    .unwrap();

    assert!(matches!(
        SessionGuard::inspect(&root),
        Err(SessionGuardError::InvalidRecord { .. })
    ));
    remove_temporary_root(&root);
}

#[cfg(windows)]
#[test]
fn windows_session_guard_lease_uses_the_cross_session_namespace() {
    let root = temporary_root("session-guard-global-namespace");
    let name = super::super::session_guard::session_mutex_name_for_test(&root);

    assert!(name.starts_with("Global\\ZirconEngineProjectSession-"));
}

#[cfg(windows)]
fn create_project_directory_alias(target: &std::path::Path, alias: &std::path::Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, alias.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for project alias fixture");
    assert!(
        output.status.success(),
        "create project alias fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
fn create_project_directory_alias(target: &std::path::Path, alias: &std::path::Path) {
    std::os::unix::fs::symlink(target, alias).expect("create project alias fixture");
}

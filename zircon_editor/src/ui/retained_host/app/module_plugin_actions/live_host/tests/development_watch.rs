use notify::event::ModifyKind;

use super::*;

#[test]
fn development_watch_uses_the_editor_job_owner_without_a_private_worker() {
    let source = include_str!("../development_watch.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("development watch production source");

    assert!(production.contains("EditorJobSystem"));
    assert!(production.contains("JobTicket<String>"));
    assert!(production.contains("wake_host();"));
    for retired_owner in [
        "std::thread",
        "JoinHandle",
        "sync_channel",
        "RecvTimeoutError",
        ".join()",
    ] {
        assert!(
            !production.contains(retired_owner),
            "retired private worker owner remains: {retired_owner}"
        );
    }
}

#[test]
fn development_watch_schedule_coalesces_to_the_latest_change_time() {
    let start = Instant::now();
    let mut schedule = DevelopmentPluginWatchSchedule::default();
    schedule.record_change_at(start);
    schedule.record_change_at(start + Duration::from_millis(100));

    assert_eq!(
        schedule.take_due_at(start + Duration::from_millis(449)),
        None
    );
    assert_eq!(
        schedule.take_due_at(start + Duration::from_millis(450)),
        Some(start + Duration::from_millis(100))
    );
    assert_eq!(schedule.take_due_at(start + Duration::from_secs(1)), None);
}

#[test]
fn development_watch_filters_for_the_exact_loaded_artifact() {
    let artifact_path = PathBuf::from("project/native/plugin.dll");
    assert!(development_event_requests_reload(
        &Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![artifact_path.clone()],
            attrs: Default::default(),
        },
        &artifact_path,
    ));

    for path in [
        PathBuf::from("project/other/native/other.dll"),
        PathBuf::from("project/plugin.toml"),
        PathBuf::from("project/native/plugin.pdb"),
    ] {
        assert!(!development_event_requests_reload(
            &Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![path],
                attrs: Default::default(),
            },
            &artifact_path,
        ));
    }
    assert!(!development_event_requests_reload(
        &Event {
            kind: EventKind::Access(notify::event::AccessKind::Any),
            paths: vec![artifact_path.clone()],
            attrs: Default::default(),
        },
        &artifact_path,
    ));
}

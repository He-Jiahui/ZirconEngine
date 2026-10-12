use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::Duration;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

use super::*;
use crate::ui::host::{EditorPluginStatus, EditorPluginStatusReport};

fn report() -> EditorPluginStatusReport {
    EditorPluginStatusReport {
        plugins: ["native.demo", "native.other"]
            .into_iter()
            .map(|id| EditorPluginStatus {
                plugin_id: id.to_string(),
                display_name: id.to_string(),
                package_source: "native".to_string(),
                load_state: "loaded".to_string(),
                enabled: true,
                required: false,
                target_modes: Vec::new(),
                packaging: ExportPackagingStrategy::NativeDynamic,
                runtime_crate: None,
                editor_crate: None,
                runtime_capabilities: Vec::new(),
                editor_capabilities: Vec::new(),
                optional_features: Vec::new(),
                diagnostics: Vec::new(),
            })
            .collect(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn blocked_live_action_rejects_a_project_reopened_before_admission() {
    let transition = Arc::new(ProjectSessionTransitionGate::default());
    let old = ("project".into(), "instance".to_string(), 1);
    let reopened = ("project".into(), "instance".to_string(), 2);
    let active = Arc::new(Mutex::new(Some(old.clone())));
    let first_transition = transition.enter().unwrap();
    let entered_loader = Arc::new(AtomicBool::new(false));
    let (attempted_tx, attempted_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let worker_transition = Arc::clone(&transition);
    let worker_active = Arc::clone(&active);
    let worker_entered_loader = Arc::clone(&entered_loader);
    let worker = std::thread::spawn(move || {
        attempted_tx.send(()).unwrap();
        let result = execute_for_matching_project_session(
            &worker_transition,
            &old,
            || worker_active.lock().unwrap().clone(),
            || {
                worker_entered_loader.store(true, Ordering::SeqCst);
                Ok(())
            },
        );
        result_tx.send(result).unwrap();
    });

    attempted_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(result_rx.recv_timeout(Duration::from_millis(25)).is_err());
    *active.lock().unwrap() = Some(reopened);
    drop(first_transition);
    assert!(result_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap()
        .is_err());
    assert!(!entered_loader.load(Ordering::SeqCst));
    worker.join().unwrap();
}

#[test]
fn project_switch_waits_until_an_admitted_live_action_finishes() {
    let transition = Arc::new(ProjectSessionTransitionGate::default());
    let old = ("project".into(), "instance".to_string(), 1);
    let reopened = ("project".into(), "instance".to_string(), 2);
    let active = Arc::new(Mutex::new(Some(old.clone())));
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (switched_tx, switched_rx) = mpsc::channel();
    let worker_transition = Arc::clone(&transition);
    let worker_active = Arc::clone(&active);
    let worker = std::thread::spawn(move || {
        execute_for_matching_project_session(
            &worker_transition,
            &old,
            || worker_active.lock().unwrap().clone(),
            || {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(1)).unwrap();
                Ok(())
            },
        )
    });
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();

    let switch_transition = Arc::clone(&transition);
    let switch_active = Arc::clone(&active);
    let switch = std::thread::spawn(move || {
        let _transition = switch_transition.enter().unwrap();
        *switch_active.lock().unwrap() = Some(reopened);
        switched_tx.send(()).unwrap();
    });
    assert!(switched_rx.recv_timeout(Duration::from_millis(25)).is_err());
    release_tx.send(()).unwrap();
    assert!(worker.join().unwrap().is_ok());
    switched_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    switch.join().unwrap();
}

#[test]
fn successful_unload_replaces_only_the_target_live_state() {
    let previous = report();
    let next = project_live_status_report(
        &previous,
        "native.demo",
        "unload",
        &Ok("Plugin native.demo unloaded".to_string()),
        &Ok(false),
    );

    assert_eq!(previous.plugins[0].load_state, "loaded");
    assert_eq!(next.plugins[0].load_state, "unloaded");
    assert_eq!(next.plugins[1], previous.plugins[1]);
    assert!(next.plugins[0].diagnostics[0].contains("unload.completed"));
}

#[test]
fn failed_hot_reload_keeps_the_last_good_loaded_state_and_reports_failure() {
    let next = project_live_status_report(
        &report(),
        "native.demo",
        "hot_reload",
        &Err("replacement failed; last-good restored".to_string()),
        &Ok(true),
    );

    assert_eq!(next.plugins[0].load_state, "loaded");
    assert!(next.plugins[0].diagnostics[0].contains("hot_reload.failed"));
    assert!(next.plugins[0].diagnostics[0].contains("last-good restored"));
}

#[test]
fn failed_live_state_query_never_reuses_an_old_loaded_claim() {
    let next = project_live_status_report(
        &report(),
        "native.demo",
        "hot_reload",
        &Err("replacement failed".to_string()),
        &Err("live host unavailable".to_string()),
    );

    assert_eq!(next.plugins[0].load_state, "live state unknown");
    assert!(next.plugins[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("live host unavailable")));
}

#[test]
fn repeated_live_operations_replace_the_prior_operation_diagnostic() {
    let unloaded = project_live_status_report(
        &report(),
        "native.demo",
        "unload",
        &Ok("Plugin native.demo unloaded".to_string()),
        &Ok(false),
    );
    let reloaded = project_live_status_report(
        &unloaded,
        "native.demo",
        "hot_reload",
        &Ok("Plugin native.demo hot reloaded".to_string()),
        &Ok(true),
    );

    assert_eq!(reloaded.plugins[0].load_state, "loaded");
    assert_eq!(reloaded.plugins[0].diagnostics.len(), 1);
    assert!(reloaded.plugins[0].diagnostics[0].contains("hot_reload.completed"));
}

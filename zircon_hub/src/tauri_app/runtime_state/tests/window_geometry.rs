use std::{fs, path::PathBuf};

use super::{HubRuntimeSession, NormalWindowGeometry, WindowGeometrySample};
use crate::settings::{HubConfig, HubLanguage};
use crate::state::{HubMessage, TaskCancellationToken, TaskStatus};
use crate::tauri_app::runtime_state::ActiveBackgroundTask;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .expect("Hub window geometry tests require coordinator-managed CARGO_TARGET_DIR");
    let path = PathBuf::from(target).join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn normal_geometry_survives_other_hub_actions_and_duplicate_flushes() {
    let temp = temp_test_dir("zircon-hub-window-geometry");
    let config_path = temp.join("hub.toml");
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), temp.join("recent_projects.json"))
            .unwrap();
    let normal = NormalWindowGeometry {
        position_x: -1540,
        position_y: 120,
        width: 1280,
        height: 800,
    };
    let sample = WindowGeometrySample {
        normal: Some(normal),
        maximized: Some(false),
    };

    assert!(!session.persist_window_geometry(sample).unwrap());
    let saved = fs::read(&config_path).unwrap();
    assert!(!session.persist_window_geometry(sample).unwrap());
    assert_eq!(fs::read(&config_path).unwrap(), saved);

    session
        .apply_action(crate::tauri_app::HubActionRequest {
            action_id: "show-page".to_string(),
            target_id: Some("settings".to_string()),
            payload: None,
        })
        .unwrap();
    let restored = HubConfig::load(&config_path).unwrap().window;
    assert_eq!(restored.position_x, Some(normal.position_x));
    assert_eq!(restored.position_y, Some(normal.position_y));
    assert_eq!(restored.width, Some(normal.width));
    assert_eq!(restored.height, Some(normal.height));
    assert!(!restored.maximized);

    session
        .persist_window_geometry(WindowGeometrySample {
            normal: None,
            maximized: Some(true),
        })
        .unwrap();
    let maximized = HubConfig::load(&config_path).unwrap().window;
    assert_eq!(maximized.width, Some(1280));
    assert_eq!(maximized.height, Some(800));
    assert!(maximized.maximized);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_failure_preserves_previous_window_state_for_a_later_retry() {
    let temp = temp_test_dir("zircon-hub-window-geometry-retry");
    let config_path = temp.join("hub.toml");
    let mut config = HubConfig::default();
    config.settings.language = HubLanguage::English;
    config.save(&config_path).unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), temp.join("recent_projects.json"))
            .unwrap();
    let before = session.config.window.clone();
    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    let sample = WindowGeometrySample {
        normal: Some(NormalWindowGeometry {
            position_x: 140,
            position_y: 80,
            width: 1160,
            height: 740,
        }),
        maximized: Some(false),
    };

    let error = session.persist_window_geometry(sample).unwrap_err();
    assert_eq!(session.config.window, before);
    session.active_background_task = Some(ActiveBackgroundTask {
        cancellation: TaskCancellationToken::new(1),
        status: TaskStatus::running("Building", HubMessage::raw_text("Build in progress"))
            .with_cancellable()
            .with_task_id(1),
    });
    let failure = session.report_window_geometry_save_failure(&error);
    assert_eq!(failure.task_summary.label, "Building");
    assert!(failure.task_summary.cancellable);
    assert_eq!(failure.task_summary.task_id, 1);
    let visible_error = failure.window_close_save_error.unwrap();
    assert_eq!(visible_error.label, "Save Hub state failed");
    assert!(!visible_error.detail.is_empty());
    assert!(
        session
            .active_background_task
            .as_ref()
            .unwrap()
            .status
            .running
    );
    session.config_path = config_path.clone();
    assert!(session.persist_window_geometry(sample).unwrap());
    assert_eq!(session.view_model().task_summary.label, "Building");
    assert!(session.view_model().window_close_save_error.is_none());
    let restored = HubConfig::load(&config_path).unwrap().window;
    assert_eq!(restored.position_x, Some(140));
    assert_eq!(restored.width, Some(1160));
    fs::remove_dir_all(temp).unwrap();
}

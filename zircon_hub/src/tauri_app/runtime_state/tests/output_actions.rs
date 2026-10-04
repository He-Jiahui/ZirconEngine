use std::{fs, path::PathBuf};

use crate::{
    engines::{SourceBuildRecord, SourceEngineInstall},
    settings::{HubConfig, HubLanguage},
    state::{HubActionKind, HubActionRecord, HubActionStatus, HubMessage},
};

use super::super::{HubActionRequest, HubRuntimeSession};
use crate::tauri_app::action_request::HubAction;

#[test]
fn open_output_folder_payload_accepts_hub_receipt_selector() {
    let temp = temp_test_dir("zircon-hub-open-output-payload");
    let receipt_id = "42:package-project:Game";

    let action = HubActionRequest {
        action_id: "open-output-folder".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "receiptId": receipt_id
        })),
    }
    .parse()
    .expect("output folder payload should parse");
    let HubAction::OpenOutputFolder { payload, .. } = action else {
        panic!("open-output-folder should parse to open-output action");
    };
    let parsed_receipt = payload
        .expect("output folder payload should be present")
        .receipt_id
        .expect("output folder receipt should be present");

    assert_eq!(parsed_receipt, receipt_id);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_resolves_hub_receipt_without_path_fallback() {
    let temp = temp_test_dir("zircon-hub-open-output-history");
    let output_dir = temp.join("package-output");
    fs::create_dir_all(&output_dir).unwrap();
    let session = session_with_output_history(&temp, &output_dir);
    let history_id = super::action_history_id(&session.config.action_history[0]);

    let resolved = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some(history_id.clone()),
                capability: None,
                engine_id: None,
            },
        ))
        .expect("history id should resolve to recorded output dir");

    assert_eq!(
        resolved,
        crate::projects::normalize_project_root(&output_dir)
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_resolves_a_server_defined_output_capability() {
    let temp = temp_test_dir("zircon-hub-open-output-typed-output-dir");
    let output_dir = temp.join("package-output");
    fs::create_dir_all(&output_dir).unwrap();
    let mut session = session_with_output_history(&temp, &output_dir);
    session
        .config
        .settings
        .set_default_build_output_from_native_folder_picker(output_dir.clone());

    let resolved = session
        .resolve_output_folder(
            Some(crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: None,
                capability: Some(
                    crate::tauri_app::action_request::OpenOutputFolderCapability::DefaultBuildOutput,
                ),
                engine_id: None,
            }),
        )
        .expect("server-defined output capability should resolve");

    assert_eq!(
        resolved,
        crate::projects::normalize_project_root(&output_dir)
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_resolves_exact_successful_source_build_receipt() {
    let temp = temp_test_dir("zircon-hub-open-output-source-build-receipt");
    let output_dir = temp.join("source-build-output");
    fs::create_dir_all(&output_dir).unwrap();
    let mut session = session_with_output_history(&temp, &output_dir);
    session.config.engines.push(SourceEngineInstall {
        id: "source-test".to_string(),
        display_name: "Source Test".to_string(),
        source_dir: temp.join("source"),
        output_dir: output_dir.clone(),
        last_build_unix_ms: Some(77),
        build_history: vec![SourceBuildRecord {
            finished_unix_ms: 77,
            status: "success".to_string(),
            profile: "debug".to_string(),
            jobs: Some(1),
            output_dir: output_dir.clone(),
            detail: HubMessage::raw_text("built"),
            log_excerpt: HubMessage::empty(),
            command_line: Vec::new(),
        }],
    });
    let receipt_id = "source-build:source-test:77:0";

    let resolved = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some(receipt_id.to_string()),
                capability: None,
                engine_id: None,
            },
        ))
        .expect("an exact successful source-build receipt should resolve");

    assert_eq!(
        resolved,
        crate::projects::normalize_project_root(&output_dir)
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_source_engine_capability_requires_a_generated_or_trusted_root() {
    let temp = temp_test_dir("zircon-hub-open-output-source-capability");
    let output_dir = temp.join("source-engine-output");
    let trusted_root = temp.join("trusted-root");
    fs::create_dir_all(&output_dir).unwrap();
    fs::create_dir_all(&trusted_root).unwrap();
    let mut session = session_with_output_history(&temp, &output_dir);
    session.config.settings.default_build_output_dir = trusted_root;
    session.config.engines.push(SourceEngineInstall {
        id: "source-test".to_string(),
        display_name: "Source Test".to_string(),
        source_dir: temp.join("source"),
        output_dir: output_dir.clone(),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: None,
                capability: Some(
                    crate::tauri_app::action_request::OpenOutputFolderCapability::SourceEngineOutput,
                ),
                engine_id: Some("source-test".to_string()),
            },
        ))
        .expect_err("an unrecorded source engine output must not become a capability");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_webview_configured_default_root() {
    let temp = temp_test_dir("zircon-hub-open-output-unverified-default");
    let output_dir = temp.join("webview-configured-output");
    fs::create_dir_all(&output_dir).unwrap();
    let mut session = session_with_output_history(&temp, &output_dir);
    session
        .config
        .settings
        .set_default_build_output_from_webview(output_dir.clone());

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: None,
                capability: Some(
                    crate::tauri_app::action_request::OpenOutputFolderCapability::DefaultBuildOutput,
                ),
                engine_id: None,
            },
        ))
        .expect_err("a WebView path update must not become a shell capability");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_an_unrecorded_receipt() {
    let temp = temp_test_dir("zircon-hub-open-output-unrecorded");
    let recorded_output = temp.join("package-output");
    fs::create_dir_all(&recorded_output).unwrap();
    let session = session_with_output_history(&temp, &recorded_output);

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some("forged-receipt".to_string()),
                capability: None,
                engine_id: None,
            },
        ))
        .expect_err("a WebView receipt must be generated by Hub");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_legacy_success_path_outside_configured_roots() {
    let temp = temp_test_dir("zircon-hub-open-output-legacy-path");
    let trusted_root = temp.join("trusted-output");
    let legacy_output = temp.join("legacy-arbitrary-output");
    fs::create_dir_all(&trusted_root).unwrap();
    fs::create_dir_all(&legacy_output).unwrap();
    let mut session = session_with_output_history(&temp, &legacy_output);
    session
        .config
        .settings
        .set_default_build_output_from_native_folder_picker(trusted_root);
    let receipt_id = super::action_history_id(&session.config.action_history[0]);

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some(receipt_id),
                capability: None,
                engine_id: None,
            },
        ))
        .expect_err("legacy arbitrary output paths must not self-authorize");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_a_target_only_history_selector() {
    let temp = temp_test_dir("zircon-hub-open-output-target-injection");
    let recorded_output = temp.join("package-output");
    fs::create_dir_all(&recorded_output).unwrap();
    let mut session = session_with_output_history(&temp, &recorded_output);
    let history_id = super::action_history_id(&session.config.action_history[0]);

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: Some(history_id),
            payload: None,
        })
        .expect("a target-only request should be recorded as a recoverable rejection");

    assert_eq!(view_model.task_summary.label, "Action failed");
    assert!(view_model
        .task_summary
        .detail
        .contains("targetId is not accepted"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_stale_recorded_output() {
    let temp = temp_test_dir("zircon-hub-open-output-stale-history");
    let missing_output = temp.join("missing-output");
    let session = session_with_output_history(&temp, &missing_output);
    let receipt_id = super::action_history_id(&session.config.action_history[0]);

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some(receipt_id),
                capability: None,
                engine_id: None,
            },
        ))
        .expect_err("stale output receipts must not be handed to the shell");

    assert!(error.to_string().contains("does not exist"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_returns_canonical_recorded_path() {
    let temp = temp_test_dir("zircon-hub-open-output-canonical");
    let recorded_output = temp.join("package-output");
    fs::create_dir_all(&recorded_output).unwrap();
    let session = session_with_output_history(&temp, &recorded_output);
    let receipt_id = super::action_history_id(&session.config.action_history[0]);

    let resolved = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some(receipt_id),
                capability: None,
                engine_id: None,
            },
        ))
        .expect("a Hub receipt should resolve to a canonical output path");

    assert_eq!(
        resolved,
        crate::projects::normalize_project_root(&recorded_output)
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_parent_traversal_in_forged_receipt() {
    let temp = temp_test_dir("zircon-hub-open-output-parent-traversal");
    let recorded_output = temp.join("package-output");
    fs::create_dir_all(&recorded_output).unwrap();
    let session = session_with_output_history(&temp, &recorded_output);

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some("../attacker-output".to_string()),
                capability: None,
                engine_id: None,
            },
        ))
        .expect_err("parent traversal must not become a shell capability");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_unrecorded_child_receipt() {
    let temp = temp_test_dir("zircon-hub-open-output-child-injection");
    let recorded_output = temp.join("package-output");
    fs::create_dir_all(&recorded_output).unwrap();
    let session = session_with_output_history(&temp, &recorded_output);

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some("source-build:unrecorded-child:1:0".to_string()),
                capability: None,
                engine_id: None,
            },
        ))
        .expect_err("only exact Hub receipts should be shell capabilities");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_rejects_symlink_or_junction_alias() {
    let temp = temp_test_dir("zircon-hub-open-output-link-alias");
    let recorded_output = temp.join("package-output");
    let linked_output = temp.join("linked-output");
    fs::create_dir_all(&recorded_output).unwrap();
    if !create_directory_link(&linked_output, &recorded_output) {
        fs::remove_dir_all(temp).unwrap();
        return;
    }
    let mut session = session_with_output_history(&temp, &recorded_output);
    session.config.action_history[0].output_dir = Some(linked_output.clone());
    let receipt_id = super::action_history_id(&session.config.action_history[0]);

    let error = session
        .resolve_output_folder(Some(
            crate::tauri_app::action_request::OpenOutputFolderPayload {
                receipt_id: Some(receipt_id),
                capability: None,
                engine_id: None,
            },
        ))
        .expect_err("a receipt resolving through a reparse point must not reach the shell");

    assert!(error.to_string().contains("not a recorded Hub output"));
    fs::remove_dir_all(temp).unwrap();
}

#[cfg(windows)]
#[test]
fn open_output_folder_rejects_unc_and_device_paths() {
    let temp = temp_test_dir("zircon-hub-open-output-unc-device");

    for path in [
        PathBuf::from(r"\\server\share\package-output"),
        PathBuf::from(r"\\.\GLOBALROOT\Device\HarddiskVolumeShadowCopy1"),
    ] {
        assert!(
            super::validate_output_path_shape(&path).is_err(),
            "UNC and device paths must not be accepted as capabilities"
        );
    }

    fs::remove_dir_all(temp).unwrap();
}

#[cfg(windows)]
#[test]
fn open_output_folder_accepts_case_alias_of_recorded_root() {
    let temp = temp_test_dir("zircon-hub-open-output-case-alias");
    let recorded_output = temp.join("Package-Output");
    fs::create_dir_all(&recorded_output).unwrap();
    let mut alias_text = recorded_output.to_string_lossy().into_owned();
    let Some(index) = alias_text
        .char_indices()
        .find_map(|(index, character)| character.is_ascii_alphabetic().then_some(index))
    else {
        fs::remove_dir_all(temp).unwrap();
        return;
    };
    let character = alias_text[index..].chars().next().unwrap();
    alias_text.replace_range(
        index..index + character.len_utf8(),
        &character.to_ascii_lowercase().to_string(),
    );
    let alias = PathBuf::from(alias_text);
    assert!(
        super::trusted_root_contains(&recorded_output, &alias),
        "case-only aliases of recorded roots should compare by normalized identity"
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_missing_directory_is_recoverable_status() {
    let temp = temp_test_dir("zircon-hub-open-output-missing");
    let output_dir = temp.join("missing-output");
    let mut session = session_with_output_history(&temp, &output_dir);
    let receipt_id = super::action_history_id(&session.config.action_history[0]);

    session
        .apply_action(HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({"receiptId": receipt_id})),
        })
        .expect("missing output folder should be a recoverable Hub error");

    assert_eq!(session.task_status.label, "Open Output failed");
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert_eq!(
        session.config.action_history[0].action,
        HubActionKind::OpenOutput
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_missing_target_failure_localizes_task_summary() {
    let temp = temp_test_dir("zircon-hub-open-output-missing-target-localized");
    let output_dir = temp.join("package-output");
    let mut session = session_with_output_history(&temp, &output_dir);
    session.config.settings.language = HubLanguage::Chinese;

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("missing output target should be a recoverable Hub error");

    assert_eq!(view_model.task_summary.label, "打开输出失败");
    assert_eq!(view_model.task_summary.detail, "需要打开输出目标");
    assert_eq!(
        view_model.task_summary.recovery.as_deref(),
        Some("打开文件夹前先选择已记录的包、安装或构建输出")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_output_folder_missing_directory_failure_localizes_task_summary() {
    let temp = temp_test_dir("zircon-hub-open-output-missing-localized");
    let output_dir = temp.join("missing-output");
    let mut session = session_with_output_history(&temp, &output_dir);
    session.config.settings.language = HubLanguage::Chinese;
    let receipt_id = super::action_history_id(&session.config.action_history[0]);

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({"receiptId": receipt_id})),
        })
        .expect("missing output folder should localize the recoverable Hub error");

    assert_eq!(view_model.task_summary.label, "打开输出失败");
    assert_eq!(
        view_model.task_summary.detail,
        format!("输出文件夹不存在：{}", output_dir.to_string_lossy())
    );
    assert_eq!(
        view_model.task_summary.recovery.as_deref(),
        Some("重新运行构建、打包或安装工作流后再打开输出文件夹")
    );

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_output_history(
    temp: &std::path::Path,
    output_dir: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir =
        output_dir.parent().unwrap_or(output_dir).to_path_buf();
    config.action_history.push(HubActionRecord {
        finished_unix_ms: 42,
        action: HubActionKind::PackageProject,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::raw_text("Packaged Game"),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: Some(output_dir.to_path_buf()),
    });
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(windows)]
fn create_directory_link(link: &std::path::Path, target: &std::path::Path) -> bool {
    std::os::windows::fs::symlink_dir(target, link).is_ok()
}

#[cfg(unix)]
fn create_directory_link(link: &std::path::Path, target: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(not(any(windows, unix)))]
fn create_directory_link(_link: &std::path::Path, _target: &std::path::Path) -> bool {
    false
}

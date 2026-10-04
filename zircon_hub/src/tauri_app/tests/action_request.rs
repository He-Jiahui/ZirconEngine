use super::*;

#[test]
fn parses_create_project_payload_for_create_project_action() {
    let action = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": "E:/Projects",
            "template": "renderable-empty",
            "engineId": "engine"
        })),
    }
    .parse()
    .expect("create-project should parse a project payload");

    let HubAction::CreateProject { payload } = action else {
        panic!("create-project should parse to the create-project action variant");
    };
    assert_eq!(payload.name, "Game");
    assert_eq!(payload.template, "renderable-empty");
    assert_eq!(payload.engine_id.as_deref(), Some("engine"));
}

#[test]
fn parses_new_project_draft_payload_for_runtime_state_update() {
    let action = HubActionRequest {
        action_id: "update-new-project-draft".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Draft Game",
            "location": "E:/Drafts",
            "template": "renderable-empty",
            "engineId": "engine"
        })),
    }
    .parse()
    .expect("update-new-project-draft should parse a draft payload");

    let HubAction::UpdateNewProjectDraft { payload } = action else {
        panic!("update-new-project-draft should parse to the draft update variant");
    };
    assert_eq!(payload.name, "Draft Game");
    assert_eq!(payload.location, PathBuf::from("E:/Drafts"));
    assert_eq!(payload.template, "renderable-empty");
    assert_eq!(payload.engine_id.as_deref(), Some("engine"));
}

#[test]
fn parses_search_projects_typed_payload() {
    let action = HubActionRequest {
        action_id: "search-projects".to_string(),
        target_id: Some("archived query".to_string()),
        payload: Some(serde_json::json!({
            "query": "typed query"
        })),
    }
    .parse()
    .expect("search-projects should parse a typed search payload");

    let HubAction::SearchProjects { query } = action else {
        panic!("search-projects should parse to the search action variant");
    };
    assert_eq!(query, "typed query");
}

#[test]
fn parses_browse_settings_folder_payload_for_folder_action() {
    let action = HubActionRequest {
        action_id: "browse-settings-folder".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "field": "defaultProjectDir",
            "initialDir": "E:/Drafts",
            "settings": {
                "defaultProjectDir": "E:/Projects"
            }
        })),
    }
    .parse()
    .expect("browse-settings-folder should parse a folder payload");

    let HubAction::BrowseSettingsFolder { payload, .. } = action else {
        panic!("browse-settings-folder should parse to the browse folder action variant");
    };
    let payload = payload.expect("folder payload should be present");
    assert_eq!(payload.field.as_deref(), Some("defaultProjectDir"));
    assert_eq!(payload.initial_dir, Some(PathBuf::from("E:/Drafts")));
    assert!(payload.settings.is_some());
}

#[test]
fn parses_update_settings_draft_payload_for_draft_action() {
    let action = HubActionRequest {
        action_id: "update-settings-draft".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "settings": {
                "pythonPath": "",
                "language": "Chinese"
            }
        })),
    }
    .parse()
    .expect("update-settings-draft should parse a settings payload");

    let HubAction::UpdateSettingsDraft { payload } = action else {
        panic!("update-settings-draft should parse to the settings draft action variant");
    };
    assert_eq!(payload.python_path.as_deref(), Some(""));
    assert_eq!(payload.language.as_deref(), Some("Chinese"));
}

#[test]
fn parses_project_target_payload_for_background_project_actions() {
    let action = HubActionRequest {
        action_id: "package-project".to_string(),
        target_id: Some("fallback-project".to_string()),
        payload: Some(serde_json::json!({
            "projectId": "target-project",
            "projectPath": "E:/Projects/Target"
        })),
    }
    .parse()
    .expect("package-project should parse a typed project target payload");

    let HubAction::PackageProject { target_id, payload } = action else {
        panic!("package-project should parse to the package action variant");
    };
    assert_eq!(target_id.as_deref(), Some("fallback-project"));
    let payload = payload.expect("project target payload should be present");
    assert_eq!(payload.project_id.as_deref(), Some("target-project"));
    assert_eq!(
        payload.project_path,
        Some(PathBuf::from("E:/Projects/Target"))
    );
}

#[test]
fn parses_cancel_delete_project_target_payload() {
    let action = HubActionRequest {
        action_id: "cancel-delete".to_string(),
        target_id: Some("fallback-project".to_string()),
        payload: Some(serde_json::json!({
            "projectId": "target-project",
            "projectPath": "E:/Projects/Target"
        })),
    }
    .parse()
    .expect("cancel-delete should parse a typed project target payload");

    let HubAction::CancelDelete { target_id, payload } = action else {
        panic!("cancel-delete should parse to the cancel delete action variant");
    };
    assert_eq!(target_id.as_deref(), Some("fallback-project"));
    let payload = payload.expect("project target payload should be present");
    assert_eq!(payload.project_id.as_deref(), Some("target-project"));
    assert_eq!(
        payload.project_path,
        Some(PathBuf::from("E:/Projects/Target"))
    );
}

#[test]
fn parses_task_bound_background_cancellation_target() {
    let action = HubActionRequest {
        action_id: "cancel-background-task".to_string(),
        target_id: Some("42".to_string()),
        payload: None,
    }
    .parse()
    .expect("background cancellation should require a numeric task target");

    assert!(matches!(
        action,
        HubAction::CancelBackgroundTask { task_id: 42 }
    ));

    let error = HubActionRequest {
        action_id: "cancel-background-task".to_string(),
        target_id: Some("current".to_string()),
        payload: None,
    }
    .parse()
    .expect_err("symbolic task targets must not cancel an arbitrary worker");
    assert!(error
        .to_string()
        .contains("Task id must be an unsigned integer"));
}

#[test]
fn parses_open_output_folder_receipt_payload_for_output_action() {
    let action = HubActionRequest {
        action_id: "open-output-folder".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "receiptId": "123:package-project:Game"
        })),
    }
    .parse()
    .expect("open-output-folder should parse an output payload");

    let HubAction::OpenOutputFolder { payload, .. } = action else {
        panic!("open-output-folder should parse to the open-output action variant");
    };
    assert_eq!(
        payload
            .expect("output payload should be present")
            .receipt_id
            .as_deref(),
        Some("123:package-project:Game")
    );
}

#[test]
fn parses_open_output_root_capability_payload() {
    let action = HubActionRequest {
        action_id: "open-output-folder".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "capability": "source-engine-output",
            "engineId": "source-local"
        })),
    }
    .parse()
    .expect("source engine capability should parse");

    let HubAction::OpenOutputFolder { payload, .. } = action else {
        panic!("open-output-folder should parse to the open-output action variant");
    };
    let payload = payload.expect("output payload should be present");
    assert_eq!(
        payload.capability,
        Some(OpenOutputFolderCapability::SourceEngineOutput)
    );
    assert_eq!(payload.engine_id.as_deref(), Some("source-local"));
}

#[test]
fn open_output_folder_rejects_raw_path_payloads_at_the_ipc_boundary() {
    for payload in [
        serde_json::json!({ "outputDir": "E:/arbitrary" }),
        serde_json::json!({ "path": "E:/arbitrary" }),
    ] {
        let error = HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: Some(payload),
        }
        .parse()
        .expect_err("raw output paths must not be accepted as shell capabilities");

        assert!(error.to_string().contains("Invalid payload for Hub action"));
    }
}

#[test]
fn open_output_folder_requires_one_selector() {
    let error = HubActionRequest {
        action_id: "open-output-folder".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({})),
    }
    .parse()
    .expect_err("an output action must carry a receipt or capability");

    assert!(error
        .to_string()
        .contains("provide exactly one Hub receiptId or capability"));
}

#[test]
fn open_output_folder_rejects_target_id_selector_at_ipc_boundary() {
    let error = HubActionRequest {
        action_id: "open-output-folder".to_string(),
        target_id: Some("123:package-project:Game".to_string()),
        payload: None,
    }
    .parse()
    .expect_err("targetId must not be treated as an output receipt");

    assert!(error
        .to_string()
        .contains("targetId is not accepted; use receiptId or capability"));
}

#[test]
fn payload_budget_rejects_oversized_string_before_typed_deserialization() {
    let error = HubActionRequest {
        action_id: "search-projects".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "query": "x".repeat(MAX_PAYLOAD_BYTES)
        })),
    }
    .parse()
    .expect_err("oversized action payloads must fail closed");

    assert!(error.to_string().contains("payload exceeds the Hub budget"));
}

#[test]
fn payload_budget_rejects_excessive_nesting_without_recursive_walk() {
    let mut payload = serde_json::json!({ "query": "x" });
    for _ in 0..=MAX_PAYLOAD_DEPTH {
        payload = serde_json::json!({ "nested": payload });
    }

    let error = HubActionRequest {
        action_id: "search-projects".to_string(),
        target_id: None,
        payload: Some(payload),
    }
    .parse()
    .expect_err("deep action payloads must fail closed");

    assert!(error.to_string().contains("payload exceeds the Hub budget"));
}

#[test]
fn create_project_rejects_empty_name_with_recoverable_message() {
    let error = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "  ",
            "location": "E:/Projects",
            "template": "renderable-empty"
        })),
    }
    .parse()
    .expect_err("empty project names should be rejected");

    assert_eq!(error.to_string(), "Project name must not be empty");
}

#[test]
fn create_project_rejects_relative_location() {
    let error = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": "projects/Game",
            "template": "renderable-empty"
        })),
    }
    .parse()
    .expect_err("relative project locations should be rejected");

    assert_eq!(
        error.to_string(),
        "Project location must be an absolute path: projects/Game"
    );
}

#[test]
fn create_project_rejects_unknown_template_id() {
    let disabled_template = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": "E:/Projects",
            "template": "3d-scene"
        })),
    }
    .parse()
    .expect("disabled catalog templates should reach runtime as coming soon");
    assert!(matches!(
        disabled_template,
        HubAction::CreateProject { payload } if payload.template == "3d-scene"
    ));

    let error = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": "E:/Projects",
            "template": "not-a-template"
        })),
    }
    .parse()
    .expect_err("unknown templates should be rejected before runtime creation");

    assert_eq!(
        error.to_string(),
        "Unknown project template: not-a-template"
    );
}

#[test]
fn project_target_envelope_payload_is_rejected_after_hard_cutover() {
    let error = HubActionRequest {
        action_id: "package-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "project": {
                "projectId": "target-project"
            }
        })),
    }
    .parse()
    .expect_err("project target envelopes should be removed after hard cutover");

    assert!(error
        .to_string()
        .contains("Invalid payload for Hub action package-project"));
}

#[test]
fn missing_required_payload_is_rejected_with_action_id() {
    let error = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: None,
    }
    .parse()
    .expect_err("required payloads should include the action id in errors");

    assert_eq!(
        error.to_string(),
        "Payload is required for Hub action: create-project"
    );
}

#[test]
fn settings_payload_requires_settings_wrapper() {
    let error = HubActionRequest {
        action_id: "update-settings-draft".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "pythonPath": "python"
        })),
    }
    .parse()
    .expect_err("settings actions should require a settings wrapper");

    assert!(error
        .to_string()
        .contains("Invalid payload for Hub action update-settings-draft"));
}

#[test]
fn unknown_action_is_rejected_before_runtime_routing() {
    let error = HubActionRequest {
        action_id: "upload-to-cloud".to_string(),
        target_id: None,
        payload: None,
    }
    .parse()
    .expect_err("unknown actions should not reach runtime routing");

    assert_eq!(error.to_string(), "Unknown Hub action: upload-to-cloud");
}

use crate::settings::HubLanguage;
use crate::state::{
    EngineMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ProjectMessageId, ShellMessageId,
};

#[test]
fn action_history_row_localizes_action_status_message_and_recovery() {
    let record = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 1,
        action: HubActionKind::PackageProject,
        status: HubActionStatus::Failed,
        target: "Game".to_string(),
        detail: HubMessage::new(HubMessageId::Project(
            ProjectMessageId::NoRecentProjectToPackage,
        )),
        log_excerpt: HubMessage::empty(),
        recovery: Some(HubMessage::new(HubMessageId::Project(
            ProjectMessageId::SelectProjectBeforePackaging,
        ))),
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };

    let item = super::action_history_row(&record, 2, HubLanguage::Chinese);

    assert_eq!(item.action, "打包项目");
    assert_eq!(item.status, "失败");
    assert_eq!(item.detail, "没有可用于打包的最近项目");
    assert_eq!(item.recovery.as_deref(), Some("打包前先选择一个可用项目"));
    assert_eq!(item.detail_rows[0].title, "目标");
    assert_eq!(item.detail_rows[0].detail, "Game");
    assert_eq!(item.detail_rows[3].title, "恢复建议");
    assert_eq!(item.detail_rows[3].detail, "打包前先选择一个可用项目");
    assert_eq!(item.detail_rows[4].title, "命令");
    assert_eq!(item.detail_rows[4].detail, "没有记录命令");
}

#[test]
fn action_history_row_localizes_project_lifecycle_success_detail() {
    let created = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 1,
        action: HubActionKind::CreateProject,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::CreatedPath),
            ["C:\\Projects\\Game"],
        ),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };
    let imported = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 2,
        action: HubActionKind::ImportProject,
        status: HubActionStatus::Success,
        target: "Imported".to_string(),
        detail: HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::ImportedPath),
            ["C:\\Projects\\Imported"],
        ),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };

    let created_item = super::action_history_row(&created, 3, HubLanguage::Chinese);
    let imported_item = super::action_history_row(&imported, 3, HubLanguage::Chinese);

    assert_eq!(created_item.detail, "已创建 C:\\Projects\\Game");
    assert_eq!(imported_item.detail, "已导入 C:\\Projects\\Imported");
}

#[test]
fn action_history_row_renders_persisted_message_in_current_language() {
    let record = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 1,
        action: HubActionKind::CreateProject,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::CreatedPath),
            ["C:\\Projects\\Game"],
        ),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };

    let english_item = super::action_history_row(&record, 2, HubLanguage::English);
    let chinese_item = super::action_history_row(&record, 2, HubLanguage::Chinese);

    assert_eq!(english_item.detail, "Created C:\\Projects\\Game");
    assert_eq!(chinese_item.detail, "已创建 C:\\Projects\\Game");
}

#[test]
fn action_history_row_localizes_log_excerpt() {
    let record = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 1,
        action: HubActionKind::RemoveProject,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::new(HubMessageId::Project(ProjectMessageId::RemovedFromHub)),
        log_excerpt: HubMessage::new(HubMessageId::Project(ProjectMessageId::RemovedFromHub)),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };

    let item = super::action_history_row(&record, 2, HubLanguage::Chinese);

    assert_eq!(item.detail, "已从 Hub 最近项目列表移除");
    assert_eq!(item.log_excerpt, "已从 Hub 最近项目列表移除");
    assert_eq!(item.detail_rows[5].title, "日志");
    assert_eq!(item.detail_rows[5].detail, "已从 Hub 最近项目列表移除");
}

#[test]
fn action_history_row_localizes_open_output_success_detail() {
    let record = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 1,
        action: HubActionKind::OpenOutput,
        status: HubActionStatus::Success,
        target: "C:\\Packages\\Game".to_string(),
        detail: HubMessage::with_params(
            HubMessageId::Shell(ShellMessageId::OpenedPath),
            ["C:\\Packages\\Game"],
        ),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: Some(42),
        command_line: Vec::new(),
        output_dir: None,
    };

    let item = super::action_history_row(&record, 2, HubLanguage::Chinese);

    assert_eq!(item.action, "打开输出");
    assert_eq!(item.detail, "已打开 C:\\Packages\\Game");
}

#[test]
fn action_history_row_identity_survives_prepending_a_new_record() {
    let record = HubActionRecord {
        id: "persisted-old-id".to_string(),
        finished_unix_ms: 7,
        action: HubActionKind::CreateProject,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::new(HubMessageId::Project(ProjectMessageId::CreatedPath)),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };
    let old_row = super::action_history_row(&record, 7, HubLanguage::English);
    let mut history = vec![record];
    crate::state::push_action_record(
        &mut history,
        HubActionRecord {
            id: "persisted-new-id".to_string(),
            finished_unix_ms: 7,
            action: HubActionKind::CreateProject,
            status: HubActionStatus::Success,
            target: "Game".to_string(),
            detail: HubMessage::new(HubMessageId::Project(ProjectMessageId::CreatedPath)),
            log_excerpt: HubMessage::empty(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: None,
        },
    );
    let old_row_after_prepend = super::action_history_row(&history[1], 7, HubLanguage::English);

    assert_eq!(old_row.id, "persisted-old-id");
    assert_eq!(old_row_after_prepend.id, old_row.id);
    assert_ne!(history[0].id, history[1].id);
}

#[test]
fn action_history_detail_rows_include_backend_output_and_command_display() {
    let record = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 1,
        action: HubActionKind::BuildEditorRuntime,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::new(HubMessageId::Engine(
            EngineMessageId::StagedEditorRuntimePayload,
        )),
        log_excerpt: HubMessage::new(HubMessageId::Engine(
            EngineMessageId::StagedEditorRuntimePayload,
        )),
        recovery: None,
        process_id: None,
        command_line: vec![
            "python".to_string(),
            "tools/zircon_build.py".to_string(),
            "--targets".to_string(),
            "editor,runtime".to_string(),
        ],
        output_dir: Some("C:\\Builds\\Game".into()),
    };

    let item = super::action_history_row(&record, 2, HubLanguage::Chinese);

    assert_eq!(item.detail_rows[2].title, "输出");
    assert_eq!(item.detail_rows[2].detail, "C:\\Builds\\Game");
    assert_eq!(item.detail_rows[3].detail, "无需恢复");
    assert_eq!(
        item.detail_rows[4].detail,
        "python tools/zircon_build.py --targets editor,runtime"
    );
    assert_eq!(item.detail_rows[5].detail, "已暂存编辑器/运行时载荷");
}

#[test]
fn generated_action_history_ids_are_unique_for_identical_same_millisecond_actions() {
    let record = HubActionRecord {
        id: String::new(),
        finished_unix_ms: 7,
        action: HubActionKind::CreateProject,
        status: HubActionStatus::Success,
        target: "Game".to_string(),
        detail: HubMessage::new(HubMessageId::Project(ProjectMessageId::CreatedPath)),
        log_excerpt: HubMessage::empty(),
        recovery: None,
        process_id: None,
        command_line: Vec::new(),
        output_dir: None,
    };
    let mut history = Vec::new();
    crate::state::push_action_record(&mut history, record.clone());
    let old_row = super::action_history_row(&history[0], 7, HubLanguage::English);
    crate::state::push_action_record(&mut history, record);
    let new_row = super::action_history_row(&history[0], 7, HubLanguage::English);
    let old_row_after_prepend = super::action_history_row(&history[1], 7, HubLanguage::English);

    assert_eq!(history.len(), 2);
    assert!(!old_row.id.is_empty());
    assert!(!new_row.id.is_empty());
    assert_ne!(new_row.id, old_row.id);
    assert_eq!(old_row_after_prepend.id, old_row.id);
    assert_eq!(new_row.id, history[0].id);
    assert_eq!(old_row_after_prepend.id, history[1].id);
    assert_eq!(history[0].finished_unix_ms, history[1].finished_unix_ms);
    assert_eq!(history[0].action, history[1].action);
    assert_eq!(history[0].target, history[1].target);
}

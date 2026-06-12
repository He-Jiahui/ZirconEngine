use serde::Serialize;

use crate::settings::HubLanguage;
use crate::state::{HubActionRecord, HubActionStatus, HubSnapshot};

use super::{path_text_en, relative_time, HubTextBundle};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubActionHistoryItem {
    pub id: String,
    pub kind: String,
    pub action: String,
    pub status: String,
    pub tone: String,
    pub target: String,
    pub detail: String,
    pub log_excerpt: String,
    pub finished: String,
    pub recovery: Option<String>,
    pub process_id: Option<u32>,
    pub command_line: Vec<String>,
    pub output_dir: Option<String>,
    pub detail_rows: Vec<HubActionHistoryDetailRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubActionHistoryDetailRow {
    pub id: String,
    pub title: String,
    pub detail: String,
}

pub(crate) fn action_history_rows(
    snapshot: &HubSnapshot,
    now_ms: u64,
    language: HubLanguage,
) -> Vec<HubActionHistoryItem> {
    snapshot
        .action_history
        .iter()
        .map(|record| action_history_row(record, now_ms, language))
        .collect()
}

fn action_history_row(
    record: &HubActionRecord,
    now_ms: u64,
    language: HubLanguage,
) -> HubActionHistoryItem {
    let text = HubTextBundle::new(language);
    let finished = relative_time(now_ms, record.finished_unix_ms, language);
    let output_dir = record.output_dir.as_deref().map(path_text_en);
    let detail = text.render_message(&record.detail);
    let log_excerpt = text.render_message(&record.log_excerpt);
    let recovery = record
        .recovery
        .as_ref()
        .map(|recovery| text.render_message(recovery));
    let detail_rows = action_history_detail_rows(
        text,
        &record.target,
        &finished,
        output_dir.as_deref(),
        recovery.as_deref(),
        &record.command_line,
        &log_excerpt,
    );

    HubActionHistoryItem {
        id: format!(
            "{}:{}:{}",
            record.finished_unix_ms,
            record.action.id(),
            record.target
        ),
        kind: record.action.id().to_string(),
        action: text.action_label(record.action).to_string(),
        status: text.action_status_label(record.status).to_string(),
        tone: action_status_tone(record.status).to_string(),
        target: record.target.clone(),
        detail,
        log_excerpt,
        finished,
        recovery,
        process_id: record.process_id,
        command_line: record.command_line.clone(),
        output_dir,
        detail_rows,
    }
}

fn action_history_detail_rows(
    text: HubTextBundle,
    target: &str,
    finished: &str,
    output_dir: Option<&str>,
    recovery: Option<&str>,
    command_line: &[String],
    log_excerpt: &str,
) -> Vec<HubActionHistoryDetailRow> {
    vec![
        detail_row("target", text.pair("Target", "目标"), target),
        detail_row("finished", text.pair("Finished", "完成时间"), finished),
        detail_row(
            "output",
            text.pair("Output", "输出"),
            output_dir.unwrap_or_else(|| text.pair("No output directory", "没有输出目录")),
        ),
        detail_row(
            "recovery",
            text.pair("Recovery", "恢复建议"),
            recovery.unwrap_or_else(|| text.pair("No recovery needed", "无需恢复")),
        ),
        detail_row(
            "command",
            text.pair("Command", "命令"),
            &command_line_text(command_line, text),
        ),
        detail_row(
            "log",
            text.pair("Log", "日志"),
            if log_excerpt.is_empty() {
                text.pair("No log excerpt", "没有日志摘录")
            } else {
                log_excerpt
            },
        ),
    ]
}

fn detail_row(id: &str, title: &str, detail: &str) -> HubActionHistoryDetailRow {
    HubActionHistoryDetailRow {
        id: id.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
    }
}

fn command_line_text(command_line: &[String], text: HubTextBundle) -> String {
    if command_line.is_empty() {
        text.pair("No command recorded", "没有记录命令").to_string()
    } else {
        command_line.join(" ")
    }
}

fn action_status_tone(status: HubActionStatus) -> &'static str {
    match status {
        HubActionStatus::Success => "success",
        HubActionStatus::Failed => "error",
        HubActionStatus::Cancelled => "warning",
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::HubLanguage;
    use crate::state::{
        EngineMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
        ProjectMessageId, ShellMessageId,
    };

    #[test]
    fn action_history_row_localizes_action_status_message_and_recovery() {
        let record = HubActionRecord {
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
    fn action_history_detail_rows_include_backend_output_and_command_display() {
        let record = HubActionRecord {
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
}

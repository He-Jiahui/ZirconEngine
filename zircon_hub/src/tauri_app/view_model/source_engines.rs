use crate::engines::{SourceBuildRecord, SourceEngineInstall};
use crate::settings::HubLanguage;

use super::display::{path_text_en, relative_time};
use super::{HubSourceBuildHistoryItem, HubTextBundle};

pub(crate) fn source_build_history_rows(
    engine: &SourceEngineInstall,
    now_ms: u64,
    language: HubLanguage,
) -> Vec<HubSourceBuildHistoryItem> {
    let text = HubTextBundle::new(language);
    engine
        .build_history
        .iter()
        .enumerate()
        .map(|(index, record)| HubSourceBuildHistoryItem {
            id: format!(
                "source-build:{}:{}:{}",
                engine.id, record.finished_unix_ms, index
            ),
            status: source_build_status_label(&record.status, text).to_string(),
            status_tone: status_tone(&record.status).to_string(),
            profile: record.profile.clone(),
            jobs: record.jobs,
            detail: text.render_message(&record.detail),
            secondary_detail: source_build_history_secondary_detail(record, text, language),
            log_excerpt: text.render_message(&record.log_excerpt),
            command_line: record.command_line.clone(),
            output_dir: path_text_en(&record.output_dir),
            finished: relative_time(now_ms, record.finished_unix_ms, language),
        })
        .collect()
}

fn source_build_history_secondary_detail(
    record: &SourceBuildRecord,
    text: HubTextBundle,
    language: HubLanguage,
) -> String {
    let command = if record.command_line.is_empty() {
        text.pair("No command recorded", "没有记录命令").to_string()
    } else {
        record.command_line.join(" ")
    };
    let log_excerpt = if record.log_excerpt.is_empty() {
        text.pair("No log excerpt", "没有日志摘录").to_string()
    } else {
        text.render_message(&record.log_excerpt)
    };

    match language {
        HubLanguage::English => format!(
            "{}: {}; {}: {}",
            text.pair("Command", "命令"),
            command,
            text.pair("Log", "日志"),
            log_excerpt
        ),
        HubLanguage::Chinese => format!(
            "{}：{}；{}：{}",
            text.pair("Command", "命令"),
            command,
            text.pair("Log", "日志"),
            log_excerpt
        ),
    }
}

fn source_build_status_label(status: &str, text: HubTextBundle) -> &'static str {
    match status {
        "success" => text.pair("Success", "成功"),
        "failed" => text.pair("Failed", "失败"),
        _ => text.pair("Unknown", "未知"),
    }
}

fn status_tone(status: &str) -> &'static str {
    match status {
        "success" => "success",
        "failed" => "error",
        _ => "warning",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::engines::{SourceBuildRecord, SourceEngineInstall};
    use crate::settings::HubLanguage;
    use crate::state::{EngineMessageId, HubMessage, HubMessageId};

    #[test]
    fn source_build_history_rows_localize_detail_status_and_finished_time() {
        let engine = SourceEngineInstall {
            id: "source-local".to_string(),
            display_name: "Local Source".to_string(),
            source_dir: PathBuf::from("E:/Source/ZirconEngine"),
            output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
            last_build_unix_ms: Some(1_000),
            build_history: vec![SourceBuildRecord {
                finished_unix_ms: 1_000,
                status: "success".to_string(),
                profile: "debug".to_string(),
                jobs: Some(4),
                output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
                detail: HubMessage::new(HubMessageId::Engine(
                    EngineMessageId::StagedEditorRuntimePayload,
                )),
                log_excerpt: HubMessage::new(HubMessageId::Engine(
                    EngineMessageId::StagedEditorRuntimePayload,
                )),
                command_line: vec!["python".to_string(), "tools/zircon_build.py".to_string()],
            }],
        };

        let rows = super::source_build_history_rows(&engine, 1_000, HubLanguage::Chinese);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "成功");
        assert_eq!(rows[0].status_tone, "success");
        assert_eq!(rows[0].detail, "已暂存编辑器/运行时载荷");
        assert_eq!(
            rows[0].secondary_detail,
            "命令：python tools/zircon_build.py；日志：已暂存编辑器/运行时载荷"
        );
        assert_eq!(rows[0].log_excerpt, "已暂存编辑器/运行时载荷");
        assert_eq!(rows[0].finished, "刚刚");
        assert_eq!(rows[0].output_dir, "E:/Source/ZirconEngine/out");
    }
}

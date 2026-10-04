//! 将各源码引擎的已完成构建记录投影为历史行，保存命令、日志和输出的追溯入口。
//! 状态、诊断和完成时间在这里本地化；记录本身仍保留原始参数。

use crate::engines::{SourceBuildRecord, SourceEngineInstall};
use crate::settings::HubLanguage;

use super::display::{path_text_en, relative_time};
use super::{HubSourceBuildHistoryItem, HubTextBundle};

/// 按保存顺序投影单个引擎的构建历史；记录索引参与行身份，不用于重新执行构建。
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

// 命令和日志组成完整本地化诊断摘要；真正参数边界仍保留在独立数组字段。
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

// 旧记录以字符串保存状态，未知值仍可展示，避免历史加载因新状态失效。
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
#[path = "tests/source_engines.rs"]
mod tests;

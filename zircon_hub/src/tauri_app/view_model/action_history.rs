//! 将持久化终态记录投影为当前语言的历史项与详情行。
//! 结构化消息和稳定动作分类分别渲染；输出路径、参数数组和进程号保留供追溯。

use serde::Serialize;

use crate::settings::HubLanguage;
use crate::state::{HubActionRecord, HubActionStatus, HubSnapshot};

use super::{path_text_en, relative_time, HubTextBundle};

/// 跨端历史项的显示协议；稳定分类供页面筛选，已渲染文本供当前语言界面使用。
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

/// 后端拥有详情标签、标点和缺省文本；行编号只在单个历史项内唯一。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubActionHistoryDetailRow {
    pub id: String,
    pub title: String,
    pub detail: String,
}

/// 按保存的最近顺序投影，不在这里重新执行动作或重读日志文件。
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
        // BUG: [CR-HUBSTATE-0003] 同毫秒、同动作且同目标的两条记录得到重复编号，Web 唯一性校验会拒绝整个状态；证据：终态使用毫秒时钟且没有序列身份。
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

/// 集中形成各历史页共用的诊断展示，避免页面分别拼接本地化缺省文案。
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

// 此摘要仅用于阅读，不是可重新执行的 shell 命令；真实参数边界保留在数组字段中。
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
#[path = "tests/action_history.rs"]
mod tests;

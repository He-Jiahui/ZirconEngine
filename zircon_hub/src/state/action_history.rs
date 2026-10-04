//! 保存已结束动作的诊断记录，供配置恢复和操作历史投影使用；运行中的进度另由任务状态持有。
//! 记录中的结构化消息在显示时选择语言，路径、命令参数和进程号保留为执行时证据。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::HubMessage;

pub const ACTION_HISTORY_LIMIT: usize = 16;

/// 一次终态动作的持久化诊断证据；恢复配置后仍可按当前语言呈现。
/// 命令参数用于展示和追溯，不能仅凭历史记录重新执行动作。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubActionRecord {
    pub finished_unix_ms: u64,
    pub action: HubActionKind,
    pub status: HubActionStatus,
    pub target: String,
    pub detail: HubMessage,
    #[serde(default = "HubMessage::empty")]
    pub log_excerpt: HubMessage,
    #[serde(default)]
    pub recovery: Option<HubMessage>,
    #[serde(default)]
    pub process_id: Option<u32>,
    #[serde(default)]
    pub command_line: Vec<String>,
    #[serde(default)]
    pub output_dir: Option<PathBuf>,
}

/// 历史记录的稳定动作分类；持久化和前端分类使用其编号，显示文案由语言投影负责。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HubActionKind {
    CreateProject,
    ImportProject,
    RemoveProject,
    DeleteProject,
    BuildEditorRuntime,
    OpenEditor,
    PackageProject,
    InstallProject,
    OpenResource,
    OpenOutput,
}

/// 动作的终结结果；取消保留独立状态，供历史页区别于失败和成功。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HubActionStatus {
    Success,
    Failed,
    Cancelled,
}

impl HubActionKind {
    /// 持久化和视图分类使用的稳定编号；语言切换不应改变它。
    pub fn id(self) -> &'static str {
        match self {
            Self::CreateProject => "create-project",
            Self::ImportProject => "import-project",
            Self::RemoveProject => "remove-project",
            Self::DeleteProject => "delete-project",
            Self::BuildEditorRuntime => "build-editor-runtime",
            Self::OpenEditor => "open-editor",
            Self::PackageProject => "package-project",
            Self::InstallProject => "install-project",
            Self::OpenResource => "open-resource",
            Self::OpenOutput => "open-output",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CreateProject => "Create Project",
            Self::ImportProject => "Import Project",
            Self::RemoveProject => "Remove Project",
            Self::DeleteProject => "Delete Project",
            Self::BuildEditorRuntime => "Build editor/runtime",
            Self::OpenEditor => "Open Editor",
            Self::PackageProject => "Package Project",
            Self::InstallProject => "Install to Device",
            Self::OpenResource => "Open Resource",
            Self::OpenOutput => "Open Output",
        }
    }
}

impl HubActionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// 供结果处理判断是否成功；取消不应被计入成功。
    pub fn succeeded(self) -> bool {
        self == Self::Success
    }
}

/// 由动作终结路径调用，按调用顺序保留最近记录；调用者须先构造完整诊断信息。
/// 该历史是窗口内的有界摘要，完整构建日志由对应输出和报告保存。
pub fn push_action_record(history: &mut Vec<HubActionRecord>, record: HubActionRecord) {
    history.insert(0, record);
    history.truncate(ACTION_HISTORY_LIMIT);
}

#[cfg(test)]
#[path = "tests/action_history.rs"]
mod tests;

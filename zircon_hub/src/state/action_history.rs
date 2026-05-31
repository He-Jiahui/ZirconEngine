use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const ACTION_HISTORY_LIMIT: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubActionRecord {
    pub finished_unix_ms: u64,
    pub action: HubActionKind,
    pub status: HubActionStatus,
    pub target: String,
    pub detail: String,
    #[serde(default)]
    pub log_excerpt: String,
    #[serde(default)]
    pub recovery: Option<String>,
    #[serde(default)]
    pub process_id: Option<u32>,
    #[serde(default)]
    pub command_line: Vec<String>,
    #[serde(default)]
    pub output_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HubActionKind {
    BuildEditorRuntime,
    OpenEditor,
    PackageProject,
    InstallProject,
    OpenOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HubActionStatus {
    Success,
    Failed,
    Cancelled,
}

impl HubActionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::BuildEditorRuntime => "Build editor/runtime",
            Self::OpenEditor => "Open Editor",
            Self::PackageProject => "Package Project",
            Self::InstallProject => "Install to Device",
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

    pub fn succeeded(self) -> bool {
        self == Self::Success
    }
}

pub fn push_action_record(history: &mut Vec<HubActionRecord>, record: HubActionRecord) {
    history.insert(0, record);
    history.truncate(ACTION_HISTORY_LIMIT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_history_keeps_newest_records() {
        let mut history = Vec::new();

        for index in 0..20 {
            push_action_record(
                &mut history,
                HubActionRecord {
                    finished_unix_ms: index,
                    action: HubActionKind::OpenEditor,
                    status: HubActionStatus::Success,
                    target: format!("target {index}"),
                    detail: "opened".to_string(),
                    log_excerpt: String::new(),
                    recovery: None,
                    process_id: Some(index as u32),
                    command_line: Vec::new(),
                    output_dir: None,
                },
            );
        }

        assert_eq!(history.len(), ACTION_HISTORY_LIMIT);
        assert_eq!(history[0].target, "target 19");
        assert_eq!(history[ACTION_HISTORY_LIMIT - 1].target, "target 4");
    }
}

//! Stable editor-contribution payloads exchanged with plugin SDK and native hosts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{EditorCommandExecutionContract, EditorCommandId};

pub const SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1: &str =
    "zircon.editor.contribution-batch/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializedToolScopeKind {
    Editor,
    Project,
    Document,
    Window,
    Viewport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializedToolResourceChannelPolicy {
    Forbidden,
    Optional,
    Required,
}

/// 插件 loader 与 editor materializer 共用的带标签载荷；批次构造及批次反序列化统一规范顺序并校验 schema。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SerializedEditorContribution {
    View {
        id: String,
        schema: String,
        title: String,
        category: String,
    },
    Drawer {
        id: String,
        schema: String,
        display_name: String,
    },
    Menu {
        id: String,
        schema: String,
        command_id: String,
        root_id: String,
        root_label_key: String,
        group_ids: Vec<String>,
        group_label_keys: Vec<String>,
        leaf_label_key: String,
    },
    Command {
        id: String,
        schema: String,
        localization_bundle_id: String,
        label_key: String,
        description_key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        execution_contract: Option<EditorCommandExecutionContract>,
    },
    AssetType {
        id: String,
        schema: String,
        display_name: String,
        badge: String,
        icon_name: String,
        color_token: String,
        thumbnail_icon: String,
    },
    LocalizationBundle {
        id: String,
        schema: String,
        locales: BTreeMap<String, BTreeMap<String, String>>,
    },
    SettingsPage {
        id: String,
        schema: String,
        label_key: String,
        description_key: String,
        category_keys: Vec<String>,
    },
    ToolResourceKind {
        id: String,
        schema: String,
        supported_scopes: Vec<SerializedToolScopeKind>,
        channel_policy: SerializedToolResourceChannelPolicy,
    },
}

impl SerializedEditorContribution {
    pub const VIEW_SCHEMA: &str = "zircon.editor.view/1";
    pub const DRAWER_SCHEMA: &str = "zircon.editor.drawer/1";
    pub const MENU_SCHEMA: &str = "zircon.editor.menu/2";
    pub const COMMAND_SCHEMA: &str = "zircon.editor.command/3";
    pub const ASSET_TYPE_SCHEMA: &str = "zircon.editor.asset-type/1";
    pub const LOCALIZATION_BUNDLE_SCHEMA: &str = "zircon.editor.localization-bundle/1";
    pub const SETTINGS_PAGE_SCHEMA: &str = "zircon.editor.settings-page/2";
    pub const TOOL_RESOURCE_KIND_SCHEMA: &str = "zircon.editor.tool-resource-kind/1";

    pub fn key(&self) -> (&'static str, &str) {
        match self {
            Self::View { id, .. } => ("view", id),
            Self::Drawer { id, .. } => ("drawer", id),
            Self::Menu { id, .. } => ("menu", id),
            Self::Command { id, .. } => ("command", id),
            Self::AssetType { id, .. } => ("asset_type", id),
            Self::LocalizationBundle { id, .. } => ("localization_bundle", id),
            Self::SettingsPage { id, .. } => ("settings_page", id),
            Self::ToolResourceKind { id, .. } => ("tool_resource_kind", id),
        }
    }

    pub fn schema(&self) -> &str {
        match self {
            Self::View { schema, .. }
            | Self::Drawer { schema, .. }
            | Self::Menu { schema, .. }
            | Self::Command { schema, .. }
            | Self::AssetType { schema, .. }
            | Self::LocalizationBundle { schema, .. }
            | Self::SettingsPage { schema, .. }
            | Self::ToolResourceKind { schema, .. } => schema,
        }
    }

    pub fn expected_schema(&self) -> &'static str {
        match self {
            Self::View { .. } => Self::VIEW_SCHEMA,
            Self::Drawer { .. } => Self::DRAWER_SCHEMA,
            Self::Menu { .. } => Self::MENU_SCHEMA,
            Self::Command { .. } => Self::COMMAND_SCHEMA,
            Self::AssetType { .. } => Self::ASSET_TYPE_SCHEMA,
            Self::LocalizationBundle { .. } => Self::LOCALIZATION_BUNDLE_SCHEMA,
            Self::SettingsPage { .. } => Self::SETTINGS_PAGE_SCHEMA,
            Self::ToolResourceKind { .. } => Self::TOOL_RESOURCE_KIND_SCHEMA,
        }
    }

    fn validate_schema(&self) -> Result<(), SerializedContributionBatchError> {
        let expected = self.expected_schema();
        if self.schema() == expected {
            return Ok(());
        }
        let (kind, id) = self.key();
        Err(
            SerializedContributionBatchError::UnsupportedContributionSchema {
                kind,
                id: id.to_string(),
                actual: self.schema().to_string(),
                expected,
            },
        )
    }

    fn validate_command_ids(&self) -> Result<(), SerializedContributionBatchError> {
        let (kind, id) = match self {
            Self::Command { id, .. } => ("command", id),
            Self::Menu { command_id, .. } => ("menu", command_id),
            _ => return Ok(()),
        };
        EditorCommandId::parse(id).map_err(|error| {
            SerializedContributionBatchError::InvalidCommandId {
                kind,
                id: error.into_value(),
            }
        })?;
        Ok(())
    }

    fn canonicalize(&mut self) {
        if let Self::ToolResourceKind {
            supported_scopes, ..
        } = self
        {
            supported_scopes.sort_unstable();
            supported_scopes.dedup();
        }
    }

    fn validate_tool_resource_scopes(&self) -> Result<(), SerializedContributionBatchError> {
        if let Self::ToolResourceKind {
            id,
            supported_scopes,
            ..
        } = self
        {
            if supported_scopes.is_empty() {
                return Err(SerializedContributionBatchError::EmptyToolResourceScopes {
                    id: id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SerializedContributionBatch {
    package_id: String,
    contributions: Vec<SerializedEditorContribution>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSerializedContributionBatch {
    package_id: String,
    contributions: Vec<SerializedEditorContribution>,
}

impl<'de> Deserialize<'de> for SerializedContributionBatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawSerializedContributionBatch::deserialize(deserializer)?;
        Self::new(raw.package_id, raw.contributions).map_err(serde::de::Error::custom)
    }
}

impl SerializedContributionBatch {
    /// 作为批次构造与反序列化的共同入口，先归一化 scope、按 `(kind, id)` 排序，再验证 schema、命令 ID 和重复项。
    pub fn new(
        package_id: impl Into<String>,
        mut contributions: Vec<SerializedEditorContribution>,
    ) -> Result<Self, SerializedContributionBatchError> {
        for contribution in &mut contributions {
            contribution.canonicalize();
        }
        contributions.sort_unstable_by(|left, right| left.key().cmp(&right.key()));
        let mut previous_key = None;
        for contribution in &contributions {
            contribution.validate_schema()?;
            contribution.validate_command_ids()?;
            contribution.validate_tool_resource_scopes()?;
            let key = contribution.key();
            if previous_key == Some(key) {
                return Err(SerializedContributionBatchError::DuplicateContribution {
                    kind: key.0,
                    id: key.1.to_string(),
                });
            }
            previous_key = Some(key);
        }
        Ok(Self {
            package_id: package_id.into(),
            contributions,
        })
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn contributions(&self) -> &[SerializedEditorContribution] {
        &self.contributions
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializedContributionBatchError {
    DuplicateContribution {
        kind: &'static str,
        id: String,
    },
    UnsupportedContributionSchema {
        kind: &'static str,
        id: String,
        actual: String,
        expected: &'static str,
    },
    InvalidCommandId {
        kind: &'static str,
        id: String,
    },
    EmptyToolResourceScopes {
        id: String,
    },
}

impl std::fmt::Display for SerializedContributionBatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateContribution { kind, id } => {
                write!(
                    formatter,
                    "duplicate serialized editor {kind} contribution `{id}`"
                )
            }
            Self::UnsupportedContributionSchema {
                kind,
                id,
                actual,
                expected,
            } => write!(
                formatter,
                "serialized editor {kind} contribution `{id}` has schema `{actual}`; expected `{expected}`"
            ),
            Self::InvalidCommandId { kind, id } => write!(
                formatter,
                "serialized editor {kind} references invalid command id `{id}`"
            ),
            Self::EmptyToolResourceScopes { id } => write!(
                formatter,
                "serialized editor tool resource kind `{id}` must support at least one scope"
            ),
        }
    }
}

impl std::error::Error for SerializedContributionBatchError {}

#[cfg(test)]
#[path = "tests/editor_contribution.rs"]
mod tests;

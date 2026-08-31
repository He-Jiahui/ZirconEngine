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
mod tests {
    use std::collections::BTreeMap;

    use super::{
        SerializedContributionBatch, SerializedEditorContribution,
        SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
    };

    #[test]
    fn batch_sorts_contributions_and_rejects_duplicate_kind_id_pairs() {
        let batch = SerializedContributionBatch::new(
            "plugin.sample",
            vec![
                SerializedEditorContribution::View {
                    id: "view.z".to_string(),
                    schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                    title: "Z".to_string(),
                    category: "Sample".to_string(),
                },
                SerializedEditorContribution::Command {
                    id: "plugin.command.a".to_string(),
                    schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                    localization_bundle_id: "plugin.sample".to_string(),
                    label_key: "command.plugin.command.a.label".to_string(),
                    description_key: "command.plugin.command.a.description".to_string(),
                    execution_contract: None,
                },
            ],
        )
        .expect("distinct contributions should be accepted");
        assert_eq!(
            batch.contributions()[0].key(),
            ("command", "plugin.command.a")
        );

        let duplicate = SerializedContributionBatch::new(
            "plugin.sample",
            vec![
                SerializedEditorContribution::Drawer {
                    id: "drawer".to_string(),
                    schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
                    display_name: "One".to_string(),
                },
                SerializedEditorContribution::View {
                    id: "view.between".to_string(),
                    schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                    title: "Between".to_string(),
                    category: "Sample".to_string(),
                },
                SerializedEditorContribution::Drawer {
                    id: "drawer".to_string(),
                    schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
                    display_name: "Two".to_string(),
                },
            ],
        );
        assert!(duplicate.is_err());
    }

    #[test]
    fn batch_rejects_a_contribution_with_the_wrong_schema() {
        let batch = SerializedContributionBatch::new(
            "plugin.sample",
            vec![SerializedEditorContribution::SettingsPage {
                id: "settings.page".to_string(),
                schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
                label_key: "settings.sample.label".to_string(),
                description_key: "settings.sample.description".to_string(),
                category_keys: vec!["settings.category.plugins".to_string()],
            }],
        );

        assert!(batch.is_err());
    }

    #[test]
    fn batch_rejects_command_ids_outside_the_shared_host_grammar() {
        let command = SerializedContributionBatch::new(
            "plugin.sample",
            vec![SerializedEditorContribution::Command {
                id: "sample.command".to_string(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                localization_bundle_id: "plugin.sample".to_string(),
                label_key: "command.sample.command.label".to_string(),
                description_key: "command.sample.command.description".to_string(),
                execution_contract: None,
            }],
        );
        let menu = SerializedContributionBatch::new(
            "plugin.sample",
            vec![SerializedEditorContribution::Menu {
                id: "plugin.sample.menu".to_string(),
                schema: SerializedEditorContribution::MENU_SCHEMA.to_string(),
                command_id: "sample.command".to_string(),
                root_id: "tools".to_string(),
                root_label_key: "menu.tools.label".to_string(),
                group_ids: Vec::new(),
                group_label_keys: Vec::new(),
                leaf_label_key: "command.sample.command.label".to_string(),
            }],
        );

        assert!(command.is_err());
        assert!(menu.is_err());
    }

    #[test]
    fn command_schema_v3_roundtrips_a_versioned_execution_contract() {
        let contract = EditorCommandExecutionContract::new(
            crate::EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
            crate::EditorCommandResourceBudget::new(4096, 8192, 250).unwrap(),
        );
        let command = SerializedEditorContribution::Command {
            id: "plugin.sample.command".to_string(),
            schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
            localization_bundle_id: "plugin.sample".to_string(),
            label_key: "command.plugin.sample.command.label".to_string(),
            description_key: "command.plugin.sample.command.description".to_string(),
            execution_contract: Some(contract),
        };
        let encoded = serde_json::to_string(&command).expect("command should serialize");
        let decoded: SerializedEditorContribution =
            serde_json::from_str(&encoded).expect("command should deserialize");
        assert_eq!(decoded, command);
        assert!(encoded.contains("zircon.editor.command/3"));
    }

    #[test]
    fn settings_page_v1_literal_payload_is_rejected_by_the_hard_cut() {
        let payload = r#"{
            "kind":"settings_page",
            "id":"settings.page",
            "schema":"zircon.editor.settings-page/1",
            "display_name":"Settings",
            "category_path":"Plugin/Settings"
        }"#;

        assert!(serde_json::from_str::<SerializedEditorContribution>(payload).is_err());
    }

    #[test]
    fn settings_page_v2_and_package_bundle_roundtrip_without_literal_fields() {
        let bundle = SerializedEditorContribution::LocalizationBundle {
            id: "plugin.sample".to_string(),
            schema: SerializedEditorContribution::LOCALIZATION_BUNDLE_SCHEMA.to_string(),
            locales: BTreeMap::from([(
                "en".to_string(),
                BTreeMap::from([(
                    "plugin.sample.settings.label".to_string(),
                    "Sample".to_string(),
                )]),
            )]),
        };
        let page = SerializedEditorContribution::SettingsPage {
            id: "plugin.sample.settings".to_string(),
            schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
            label_key: "plugin.sample.settings.label".to_string(),
            description_key: "plugin.sample.settings.description".to_string(),
            category_keys: vec!["plugin.sample.category.settings".to_string()],
        };
        let batch = SerializedContributionBatch::new("plugin.sample", vec![page.clone(), bundle])
            .expect("V2 settings page and bundle should be accepted");

        let json = serde_json::to_string(&batch).unwrap();
        let decoded = serde_json::from_str::<SerializedContributionBatch>(&json).unwrap();
        assert_eq!(decoded, batch);
        assert!(
            SerializedContributionBatch::new("plugin.sample", vec![page.clone(), page]).is_err(),
            "page identity must remain duplicate-checked independently of presentation"
        );
    }

    #[test]
    fn tool_resource_kind_scopes_are_nonempty_canonical_and_roundtrip() {
        let resource = SerializedEditorContribution::ToolResourceKind {
            id: "plugin.sample.viewport-lock".to_string(),
            schema: SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA.to_string(),
            supported_scopes: vec![
                SerializedToolScopeKind::Viewport,
                SerializedToolScopeKind::Window,
                SerializedToolScopeKind::Viewport,
            ],
            channel_policy: SerializedToolResourceChannelPolicy::Optional,
        };
        let batch = SerializedContributionBatch::new("sample", vec![resource])
            .expect("tool resource declaration should canonicalize");
        assert!(matches!(
            &batch.contributions()[0],
            SerializedEditorContribution::ToolResourceKind {
                supported_scopes,
                ..
            } if supported_scopes == &[
                SerializedToolScopeKind::Window,
                SerializedToolScopeKind::Viewport,
            ]
        ));
        let json = serde_json::to_string(&batch).unwrap();
        assert_eq!(
            serde_json::from_str::<SerializedContributionBatch>(&json).unwrap(),
            batch
        );

        let empty = SerializedContributionBatch::new(
            "sample",
            vec![SerializedEditorContribution::ToolResourceKind {
                id: "plugin.sample.empty".to_string(),
                schema: SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA.to_string(),
                supported_scopes: Vec::new(),
                channel_policy: SerializedToolResourceChannelPolicy::Forbidden,
            }],
        );
        assert!(matches!(
            empty,
            Err(super::SerializedContributionBatchError::EmptyToolResourceScopes { .. })
        ));
    }
}

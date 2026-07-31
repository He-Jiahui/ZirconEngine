//! Stable editor-contribution payloads exchanged with plugin SDK and native hosts.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1: &str =
    "zircon.editor.contribution-batch/1";

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
        path: String,
        schema: String,
        command_id: String,
    },
    Command {
        id: String,
        schema: String,
        display_name: String,
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
    SettingsPage {
        id: String,
        schema: String,
        display_name: String,
        category_path: String,
    },
}

impl SerializedEditorContribution {
    pub const VIEW_SCHEMA: &str = "zircon.editor.view/1";
    pub const DRAWER_SCHEMA: &str = "zircon.editor.drawer/1";
    pub const MENU_SCHEMA: &str = "zircon.editor.menu/1";
    pub const COMMAND_SCHEMA: &str = "zircon.editor.command/1";
    pub const ASSET_TYPE_SCHEMA: &str = "zircon.editor.asset-type/1";
    pub const SETTINGS_PAGE_SCHEMA: &str = "zircon.editor.settings-page/1";

    pub fn key(&self) -> (&'static str, &str) {
        match self {
            Self::View { id, .. } => ("view", id),
            Self::Drawer { id, .. } => ("drawer", id),
            Self::Menu { path, .. } => ("menu", path),
            Self::Command { id, .. } => ("command", id),
            Self::AssetType { id, .. } => ("asset_type", id),
            Self::SettingsPage { id, .. } => ("settings_page", id),
        }
    }

    pub fn schema(&self) -> &str {
        match self {
            Self::View { schema, .. }
            | Self::Drawer { schema, .. }
            | Self::Menu { schema, .. }
            | Self::Command { schema, .. }
            | Self::AssetType { schema, .. }
            | Self::SettingsPage { schema, .. } => schema,
        }
    }

    pub fn expected_schema(&self) -> &'static str {
        match self {
            Self::View { .. } => Self::VIEW_SCHEMA,
            Self::Drawer { .. } => Self::DRAWER_SCHEMA,
            Self::Menu { .. } => Self::MENU_SCHEMA,
            Self::Command { .. } => Self::COMMAND_SCHEMA,
            Self::AssetType { .. } => Self::ASSET_TYPE_SCHEMA,
            Self::SettingsPage { .. } => Self::SETTINGS_PAGE_SCHEMA,
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
        contributions.sort_by(|left, right| left.key().cmp(&right.key()));
        let mut keys = BTreeSet::new();
        for contribution in &contributions {
            contribution.validate_schema()?;
            let key = contribution.key();
            if !keys.insert(key) {
                return Err(SerializedContributionBatchError::DuplicateContribution {
                    kind: key.0,
                    id: key.1.to_string(),
                });
            }
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
        }
    }
}

impl std::error::Error for SerializedContributionBatchError {}

#[cfg(test)]
mod tests {
    use super::{SerializedContributionBatch, SerializedEditorContribution};

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
                    id: "command.a".to_string(),
                    schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                    display_name: "A".to_string(),
                },
            ],
        )
        .expect("distinct contributions should be accepted");
        assert_eq!(batch.contributions()[0].key(), ("command", "command.a"));

        let duplicate = SerializedContributionBatch::new(
            "plugin.sample",
            vec![
                SerializedEditorContribution::Drawer {
                    id: "drawer".to_string(),
                    schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
                    display_name: "One".to_string(),
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
                display_name: "Settings".to_string(),
                category_path: "Plugin/Settings".to_string(),
            }],
        );

        assert!(batch.is_err());
    }
}

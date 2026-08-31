//! DTO-only editor contribution authoring for native plugin boundaries.

use std::collections::BTreeMap;

use zircon_runtime_interface::editor_contribution::{
    SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
};
use zircon_runtime_interface::{
    EditorCommandExecutionContract, SerializedContributionBatch, SerializedContributionBatchError,
    SerializedEditorContribution,
};

/// Collects one plugin package's editor contributions before host-side materialization.
#[derive(Clone, Debug)]
pub struct EditorContributionBuilder {
    package_id: String,
    contributions: Vec<SerializedEditorContribution>,
}

impl EditorContributionBuilder {
    pub fn new(package_id: impl Into<String>) -> Self {
        Self {
            package_id: package_id.into(),
            contributions: Vec::new(),
        }
    }

    pub fn view(
        mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        self.contributions.push(SerializedEditorContribution::View {
            id: id.into(),
            schema: SerializedEditorContribution::VIEW_SCHEMA.to_string(),
            title: title.into(),
            category: category.into(),
        });
        self
    }

    pub fn drawer(mut self, id: impl Into<String>, display_name: impl Into<String>) -> Self {
        self.contributions
            .push(SerializedEditorContribution::Drawer {
                id: id.into(),
                schema: SerializedEditorContribution::DRAWER_SCHEMA.to_string(),
                display_name: display_name.into(),
            });
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn menu<I, S, K>(
        mut self,
        id: impl Into<String>,
        command_id: impl Into<String>,
        root_id: impl Into<String>,
        root_label_key: impl Into<String>,
        groups: I,
        leaf_label_key: impl Into<String>,
    ) -> Self
    where
        I: IntoIterator<Item = (S, K)>,
        S: Into<String>,
        K: Into<String>,
    {
        let (group_ids, group_label_keys) = groups
            .into_iter()
            .map(|(id, key)| (id.into(), key.into()))
            .unzip();
        self.contributions.push(SerializedEditorContribution::Menu {
            id: id.into(),
            schema: SerializedEditorContribution::MENU_SCHEMA.to_string(),
            command_id: command_id.into(),
            root_id: root_id.into(),
            root_label_key: root_label_key.into(),
            group_ids,
            group_label_keys,
            leaf_label_key: leaf_label_key.into(),
        });
        self
    }

    pub fn command(
        mut self,
        id: impl Into<String>,
        label_key: impl Into<String>,
        description_key: impl Into<String>,
    ) -> Self {
        self.contributions
            .push(SerializedEditorContribution::Command {
                id: id.into(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                localization_bundle_id: self.package_id.clone(),
                label_key: label_key.into(),
                description_key: description_key.into(),
                execution_contract: None,
            });
        self
    }

    pub fn command_with_execution_contract(
        mut self,
        id: impl Into<String>,
        label_key: impl Into<String>,
        description_key: impl Into<String>,
        execution_contract: EditorCommandExecutionContract,
    ) -> Self {
        self.contributions
            .push(SerializedEditorContribution::Command {
                id: id.into(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                localization_bundle_id: self.package_id.clone(),
                label_key: label_key.into(),
                description_key: description_key.into(),
                execution_contract: Some(execution_contract),
            });
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn asset_type(
        mut self,
        id: impl Into<String>,
        display_name: impl Into<String>,
        badge: impl Into<String>,
        icon_name: impl Into<String>,
        color_token: impl Into<String>,
        thumbnail_icon: impl Into<String>,
    ) -> Self {
        self.contributions
            .push(SerializedEditorContribution::AssetType {
                id: id.into(),
                schema: SerializedEditorContribution::ASSET_TYPE_SCHEMA.to_string(),
                display_name: display_name.into(),
                badge: badge.into(),
                icon_name: icon_name.into(),
                color_token: color_token.into(),
                thumbnail_icon: thumbnail_icon.into(),
            });
        self
    }

    pub fn settings_page<I, S>(
        mut self,
        id: impl Into<String>,
        label_key: impl Into<String>,
        description_key: impl Into<String>,
        category_keys: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.contributions
            .push(SerializedEditorContribution::SettingsPage {
                id: id.into(),
                schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
                label_key: label_key.into(),
                description_key: description_key.into(),
                category_keys: category_keys.into_iter().map(Into::into).collect(),
            });
        self
    }

    pub fn localization_bundle(
        mut self,
        locales: BTreeMap<String, BTreeMap<String, String>>,
    ) -> Self {
        self.contributions
            .push(SerializedEditorContribution::LocalizationBundle {
                id: self.package_id.clone(),
                schema: SerializedEditorContribution::LOCALIZATION_BUNDLE_SCHEMA.to_string(),
                locales,
            });
        self
    }

    pub fn tool_resource_kind<I>(
        mut self,
        id: impl Into<String>,
        supported_scopes: I,
        channel_policy: SerializedToolResourceChannelPolicy,
    ) -> Self
    where
        I: IntoIterator<Item = SerializedToolScopeKind>,
    {
        self.contributions
            .push(SerializedEditorContribution::ToolResourceKind {
                id: id.into(),
                schema: SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA.to_string(),
                supported_scopes: supported_scopes.into_iter().collect(),
                channel_policy,
            });
        self
    }

    /// Delegates canonical ordering and duplicate validation to the shared DTO boundary.
    pub fn build(self) -> Result<SerializedContributionBatch, SerializedContributionBatchError> {
        SerializedContributionBatch::new(self.package_id, self.contributions)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime_interface::editor_contribution::{
        SerializedToolResourceChannelPolicy, SerializedToolScopeKind,
    };
    use zircon_runtime_interface::{
        EditorCommandResourceBudget, EditorCommandResultCodecId, SerializedEditorContribution,
    };

    use super::EditorContributionBuilder;

    #[test]
    fn build_returns_a_canonically_sorted_batch() {
        let batch = EditorContributionBuilder::new("plugin.sample")
            .view("sample.view", "Sample", "Sample")
            .command(
                "plugin.sample.command",
                "command.plugin.sample.command.label",
                "command.plugin.sample.command.description",
            )
            .build()
            .expect("distinct contributions should be accepted");

        assert_eq!(batch.package_id(), "plugin.sample");
        assert_eq!(
            batch.contributions()[0].key(),
            ("command", "plugin.sample.command")
        );
    }

    #[test]
    fn build_reuses_the_shared_duplicate_rejection() {
        let result = EditorContributionBuilder::new("plugin.sample")
            .drawer("sample.drawer", "First")
            .drawer("sample.drawer", "Second")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn build_reuses_the_shared_command_id_grammar() {
        let command = EditorContributionBuilder::new("plugin.sample")
            .command(
                "sample.command",
                "command.sample.command.label",
                "command.sample.command.description",
            )
            .build();
        let menu = EditorContributionBuilder::new("plugin.sample")
            .menu(
                "plugin.sample.menu",
                "sample.command",
                "tools",
                "menu.tools.label",
                std::iter::empty::<(&str, &str)>(),
                "command.sample.command.label",
            )
            .build();

        assert!(command.is_err());
        assert!(menu.is_err());
    }

    #[test]
    fn command_builder_can_declare_a_versioned_execution_contract() {
        let contract = zircon_runtime_interface::EditorCommandExecutionContract::new(
            EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
            EditorCommandResourceBudget::new(4096, 8192, 250).unwrap(),
        );
        let batch = EditorContributionBuilder::new("plugin.sample")
            .command_with_execution_contract(
                "plugin.sample.command",
                "command.plugin.sample.command.label",
                "command.plugin.sample.command.description",
                contract,
            )
            .build()
            .expect("contract-bearing command should build");

        assert!(matches!(
            &batch.contributions()[0],
            SerializedEditorContribution::Command {
                schema,
                execution_contract: Some(_),
                ..
            } if schema == SerializedEditorContribution::COMMAND_SCHEMA
        ));
    }

    #[test]
    fn settings_page_uses_its_package_bundle_and_locale_neutral_keys() {
        let batch = EditorContributionBuilder::new("fixture.editor")
            .localization_bundle(BTreeMap::from([(
                "en".to_string(),
                BTreeMap::from([
                    ("plugin.fixture.label".to_string(), "Fixture".to_string()),
                    (
                        "plugin.fixture.description".to_string(),
                        "Fixture settings".to_string(),
                    ),
                    ("plugin.category.plugins".to_string(), "Plugins".to_string()),
                    ("plugin.category.fixture".to_string(), "Fixture".to_string()),
                ]),
            )]))
            .settings_page(
                "plugin.fixture.editor.settings",
                "plugin.fixture.label",
                "plugin.fixture.description",
                ["plugin.category.plugins", "plugin.category.fixture"],
            )
            .build()
            .expect("locale-neutral settings contributions should build");

        assert!(matches!(
            &batch.contributions()[0],
            SerializedEditorContribution::LocalizationBundle { id, .. }
                if id == "fixture.editor"
        ));
        assert!(matches!(
            &batch.contributions()[1],
            SerializedEditorContribution::SettingsPage { label_key, category_keys, .. }
                if label_key == "plugin.fixture.label"
                    && category_keys == &["plugin.category.plugins", "plugin.category.fixture"]
        ));
    }

    #[test]
    fn menu_uses_typed_segment_ids_and_locale_neutral_keys() {
        let batch = EditorContributionBuilder::new("fixture.editor")
            .menu(
                "fixture.menu.command",
                "fixture.editor.command",
                "tools",
                "menu.tools.label",
                [("fixture", "menu.tools.fixture.label")],
                "command.fixture.editor.command.label",
            )
            .build()
            .expect("typed menu contribution should build");

        assert!(matches!(
            &batch.contributions()[0],
            SerializedEditorContribution::Menu {
                schema,
                command_id,
                root_id,
                root_label_key,
                group_ids,
                group_label_keys,
                leaf_label_key,
                ..
            } if schema == SerializedEditorContribution::MENU_SCHEMA
                && command_id == "fixture.editor.command"
                && root_id == "tools"
                && root_label_key == "menu.tools.label"
                && group_ids == &["fixture"]
                && group_label_keys == &["menu.tools.fixture.label"]
                && leaf_label_key == "command.fixture.editor.command.label"
        ));
    }

    #[test]
    fn tool_resource_kind_uses_the_shared_canonical_declaration() {
        let batch = EditorContributionBuilder::new("sample")
            .tool_resource_kind(
                "plugin.sample.viewport-lock",
                [
                    SerializedToolScopeKind::Viewport,
                    SerializedToolScopeKind::Window,
                    SerializedToolScopeKind::Viewport,
                ],
                SerializedToolResourceChannelPolicy::Required,
            )
            .build()
            .expect("tool resource kind contribution should build");

        assert!(matches!(
            &batch.contributions()[0],
            SerializedEditorContribution::ToolResourceKind {
                schema,
                supported_scopes,
                channel_policy: SerializedToolResourceChannelPolicy::Required,
                ..
            } if schema == SerializedEditorContribution::TOOL_RESOURCE_KIND_SCHEMA
                && supported_scopes == &[
                    SerializedToolScopeKind::Window,
                    SerializedToolScopeKind::Viewport,
                ]
        ));
    }
}

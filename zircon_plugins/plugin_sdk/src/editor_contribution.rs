//! DTO-only editor contribution authoring for native plugin boundaries.

use zircon_runtime_interface::{
    SerializedContributionBatch, SerializedContributionBatchError, SerializedEditorContribution,
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

    pub fn menu(mut self, path: impl Into<String>, command_id: impl Into<String>) -> Self {
        self.contributions.push(SerializedEditorContribution::Menu {
            path: path.into(),
            schema: SerializedEditorContribution::MENU_SCHEMA.to_string(),
            command_id: command_id.into(),
        });
        self
    }

    pub fn command(mut self, id: impl Into<String>, display_name: impl Into<String>) -> Self {
        self.contributions
            .push(SerializedEditorContribution::Command {
                id: id.into(),
                schema: SerializedEditorContribution::COMMAND_SCHEMA.to_string(),
                display_name: display_name.into(),
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

    pub fn settings_page(
        mut self,
        id: impl Into<String>,
        display_name: impl Into<String>,
        category_path: impl Into<String>,
    ) -> Self {
        self.contributions
            .push(SerializedEditorContribution::SettingsPage {
                id: id.into(),
                schema: SerializedEditorContribution::SETTINGS_PAGE_SCHEMA.to_string(),
                display_name: display_name.into(),
                category_path: category_path.into(),
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
    use super::EditorContributionBuilder;

    #[test]
    fn build_returns_a_canonically_sorted_batch() {
        let batch = EditorContributionBuilder::new("plugin.sample")
            .view("sample.view", "Sample", "Sample")
            .command("sample.command", "Sample Command")
            .build()
            .expect("distinct contributions should be accepted");

        assert_eq!(batch.package_id(), "plugin.sample");
        assert_eq!(
            batch.contributions()[0].key(),
            ("command", "sample.command")
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
}

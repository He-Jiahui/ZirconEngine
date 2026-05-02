use zircon_runtime_interface::ui::template::UiRootClassPolicy;

use super::{
    command_entry::tree_document_replay_bundle,
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
};

impl UiAssetEditorSession {
    pub fn selected_component_root_class_policy(&self) -> Option<UiRootClassPolicy> {
        let component_name = self.selected_local_component_name()?;
        self.last_valid_document
            .components
            .get(&component_name)
            .map(|component| component.contract.root_class_policy)
    }

    pub fn can_edit_selected_component_root_class_policy(&self) -> bool {
        self.diagnostics.is_empty() && self.selected_component_root_class_policy().is_some()
    }

    pub fn set_selected_component_root_class_policy(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(policy) = parse_root_class_policy(value.as_ref()) else {
            return Ok(false);
        };
        let Some(component_name) = self.selected_local_component_name() else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(component) = document.components.get_mut(&component_name) else {
            return Ok(false);
        };
        if component.contract.root_class_policy == policy {
            return Ok(false);
        }

        component.contract.root_class_policy = policy;
        let replay = tree_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Set Root Class Policy", replay)?;
        Ok(true)
    }
}

pub(super) fn root_class_policy_label(policy: UiRootClassPolicy) -> &'static str {
    match policy {
        UiRootClassPolicy::AppendOnly => "append_only",
        UiRootClassPolicy::Closed => "closed",
    }
}

fn parse_root_class_policy(value: &str) -> Option<UiRootClassPolicy> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "append_only" | "appendonly" => Some(UiRootClassPolicy::AppendOnly),
        "closed" => Some(UiRootClassPolicy::Closed),
        _ => None,
    }
}

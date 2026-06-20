use std::path::Path;

use super::super::super::RetainedEditorHost;
use super::super::action_ids::ModulePluginAction;
use zircon_runtime::asset::project::ProjectManifest;

mod feature;
mod plugin;

impl RetainedEditorHost {
    pub(super) fn dispatch_module_plugin_project_manifest_action(
        &mut self,
        project_root: &Path,
        manifest: &mut ProjectManifest,
        action: ModulePluginAction,
    ) -> Result<String, String> {
        match action {
            ModulePluginAction::SetEnabled { plugin_id, enabled } => {
                self.set_project_plugin_enabled(project_root, manifest, plugin_id, enabled)
            }
            ModulePluginAction::CyclePackaging { plugin_id } => {
                self.cycle_project_plugin_packaging(project_root, manifest, plugin_id)
            }
            ModulePluginAction::CycleTargetModes { plugin_id } => {
                self.cycle_project_plugin_target_modes(project_root, manifest, plugin_id)
            }
            ModulePluginAction::SetFeatureEnabled {
                plugin_id,
                feature_id,
                enabled,
            } => self.set_project_plugin_feature_enabled(
                project_root,
                manifest,
                plugin_id,
                feature_id,
                enabled,
            ),
            ModulePluginAction::EnableFeatureDependencies {
                plugin_id,
                feature_id,
            } => self.enable_project_plugin_feature_dependencies(
                project_root,
                manifest,
                plugin_id,
                feature_id,
            ),
            ModulePluginAction::Unload { .. } | ModulePluginAction::HotReload { .. } => {
                Err("live plugin action was routed to project manifest mutation".to_string())
            }
        }
    }
}

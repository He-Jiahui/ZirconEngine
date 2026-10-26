//! Capture script drafts through the same scoped Edit-world boundary as Add Component.
use crate::core::editing::command::{read_script_bindings, AddComponentScope};
use crate::core::editing::engine::EditCommandError;
use zircon_runtime::asset::SceneScriptBindingAsset;
use zircon_runtime::scene::NodeId;

use super::{EditorHostEventController, ScriptPackageChoice};

impl EditorHostEventController {
    pub(crate) fn inspector_script_binding_packages(
        &self,
        refresh: bool,
    ) -> Result<Vec<ScriptPackageChoice>, String> {
        self.enter_editor_manager()
            .map_err(|error| error.to_string())?
            .current_script_binding_packages(refresh)
    }

    pub(crate) fn inspector_script_binding_before(
        &self,
        scope: &AddComponentScope,
    ) -> Result<Option<Vec<SceneScriptBindingAsset>>, String> {
        self.with_inspector_component_add_context(Some(scope), |core, scope| {
            let [node] = scope.targets.as_slice() else {
                return Err(EditCommandError::InvariantViolation {
                    invariant: "script authoring requires one selected entity",
                });
            };
            core.with_scene(|scene| read_script_bindings(scene, *node))?
        })
    }

    pub(crate) fn inspector_script_binding_scope(
        &self,
        node: NodeId,
        before: &Option<Vec<SceneScriptBindingAsset>>,
    ) -> Result<AddComponentScope, String> {
        self.with_inspector_component_add_context(None, |core, scope| {
            if scope.targets.as_slice() != [node] || scope.primary != Some(node) {
                return Err(EditCommandError::InvariantViolation {
                    invariant: "script authoring requires the current single selection",
                });
            }
            if &core.with_scene(|scene| read_script_bindings(scene, node))?? != before {
                return Err(EditCommandError::InvariantViolation {
                    invariant: "script bindings changed since the inspector snapshot",
                });
            }
            Ok(scope)
        })
    }
}

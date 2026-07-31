use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_operation::EditorOperationPath;

use super::builtin_scene_mode::{SelectSceneMode, TransformSceneMode};
use super::{EditorSceneMode, SceneModeRegistration, SceneModeRegistry};

pub(crate) fn builtin_scene_mode_registry() -> SceneModeRegistry {
    let mut registry = SceneModeRegistry::default();
    register_builtin(
        &mut registry,
        "scene.select",
        "Select",
        "scene.mode.activate.select",
        || Box::new(SelectSceneMode::new()) as Box<dyn EditorSceneMode>,
    );
    register_builtin(
        &mut registry,
        "scene.transform",
        "Transform",
        "scene.mode.activate.transform",
        || Box::new(TransformSceneMode::new()) as Box<dyn EditorSceneMode>,
    );
    registry
}

fn register_builtin(
    registry: &mut SceneModeRegistry,
    id: &'static str,
    display_name: &'static str,
    operation: &'static str,
    factory: impl Fn() -> Box<dyn EditorSceneMode> + Send + Sync + 'static,
) {
    let descriptor = SceneModeDescriptor::new(
        id,
        display_name,
        "editor.scene",
        EditorOperationPath::parse(operation).expect("built-in scene mode operation is valid"),
    );
    registry
        .register(
            SceneModeRegistration::new(descriptor, factory).with_owner_id("zircon.editor.builtin"),
        )
        .expect("built-in scene mode identifiers are unique");
}

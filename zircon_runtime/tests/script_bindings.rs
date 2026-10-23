//! Built-in script storage must coexist with an owner-bound plugin catalog.

use serde_json::json;
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetUri, SceneAsset};
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::plugin::RuntimeExtensionRegistry;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::world::SceneComponentSerializer;
use zircon_runtime::scene::{SceneError, World};
use zircon_runtime_interface::project::RelPath;

const PLUGIN_TYPE: &str = "test.script_attachment.Tag";

fn bindings() -> serde_json::Value {
    json!([{
        "package": "orb_run", "module": "main", "enabled": true,
        "update": true, "fixed_update": false, "properties": {"role": "player"}
    }])
}

#[test]
fn script_bindings_attach_with_a_plugin_descriptor_without_admitting_unknown_types() {
    let mut world = World::empty();
    world
        .register_component_type(ComponentTypeDescriptor::new(
            PLUGIN_TYPE,
            "test.script_attachment",
            "Tag",
        ))
        .unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    assert!(world
        .set_dynamic_component(entity, "script.bindings", bindings())
        .expect("built-in bindings must remain writable with a nonempty plugin catalog"));
    assert_eq!(
        world.dynamic_component(entity, "script.bindings"),
        Some(&bindings())
    );
    assert!(matches!(
        world.set_dynamic_component(entity, "unknown.Component", json!({})),
        Err(SceneError::UnregisteredDynamicComponentType { .. })
    ));
    assert!(world.component_type_descriptor("script.bindings").is_none());
    assert!(
        !world
            .type_registry()
            .registration("script.bindings")
            .unwrap()
            .editor_visible
    );
}

#[test]
fn script_bindings_load_and_save_with_an_installed_plugin_provider() {
    let root = std::env::temp_dir().join(format!("zircon_g02_{}", uuid::Uuid::new_v4()));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "G02",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let mut extensions = RuntimeExtensionRegistry::default();
    let owner = extensions
        .intern_plugin_module("test.script_attachment.runtime")
        .unwrap();
    extensions
        .register_component_for_owner(
            owner,
            ComponentTypeDescriptor::new(PLUGIN_TYPE, "test.script_attachment", "Tag"),
        )
        .unwrap();
    extensions
        .register_scene_component_codec_for_owner(
            owner,
            SceneComponentSerializer {
                type_id: PLUGIN_TYPE,
                schema_id: "test.script_attachment.Tag.v1",
                schema_version: 1,
                provider_id: "test.script_attachment",
                capture: |_, _, _| Ok(None),
                instantiate: |_, _, _| Ok(None),
            },
        )
        .unwrap();
    extensions
        .apply_scene_component_codecs_to_project_manager(&project)
        .unwrap();
    let mut source = World::empty();
    let entity = source.spawn_node(NodeKind::Empty).unwrap();
    source
        .set_dynamic_component(entity, "script.bindings", bindings())
        .unwrap();
    let scene: SceneAsset = source.to_scene_asset(&project).unwrap();
    let loaded = World::from_scene_asset(&project, &scene)
        .expect("the provider is installed before built-in bindings are loaded");
    assert!(loaded.component_type_descriptor(PLUGIN_TYPE).is_some());
    assert_eq!(
        loaded.dynamic_component(entity, "script.bindings"),
        Some(&bindings())
    );
    assert_eq!(loaded.to_scene_asset(&project).unwrap(), scene);
    std::fs::remove_dir_all(root).unwrap();
}

use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, LevelSystem, NodeId, Scene};
use zircon_runtime_interface::reflect::{ReflectObjectAddress, ReflectReadRequest, ReflectedValue};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::EditorRuntimeGatewayHandle;

pub(super) const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
pub(super) const HIERARCHY_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";
pub(super) const LOCAL_TRANSFORM_TYPE_PATH: &str =
    "zircon_runtime::scene::components::LocalTransform";
pub(super) const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
pub(super) const WIND_ANCHOR_TYPE_PATH: &str = "weather.Component.WindAnchor";

pub(super) fn transaction_scene(
    scene: Scene,
    selected: NodeId,
) -> (LevelSystem, EditorTransactionEngine) {
    let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
    let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    context
        .bind_scene(
            level.clone(),
            SceneSelection::new(vec![selected], Some(selected)),
        )
        .unwrap();
    (level, EditorTransactionEngine::new(context))
}

pub(super) fn commit_command(transactions: &EditorTransactionEngine, command: EditorCommand) {
    let mut scope = transactions
        .begin("Set reflected scene field", HistoryContextId::Global)
        .expect("transaction should begin");
    scope.push(command).expect("command should apply");
    scope.commit().expect("transaction should commit");
}

pub(super) fn scene_with_cloud_layer() -> Scene {
    let mut scene = Scene::empty();
    scene
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic component descriptor should register");
    scene
}

pub(super) fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}

pub(super) fn wind_anchor_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(WIND_ANCHOR_TYPE_PATH, "weather", "Wind Anchor")
        .with_property("direction", "Vec3", true)
        .with_property("target", "Entity", true)
}

pub(super) fn read_reflected_field(
    scene: &Scene,
    entity: NodeId,
    type_path: &str,
    field_name: &str,
) -> ReflectedValue {
    let field_id = scene
        .reflect_schema(type_path)
        .expect("test reflected type should be registered")
        .type_info
        .fields
        .into_iter()
        .find(|field| field.name == field_name)
        .expect("test reflected field should be registered")
        .id;
    scene
        .reflect_read(ReflectReadRequest::new(
            ReflectObjectAddress::component(entity, type_path)
                .expect("test component address should be valid"),
            field_id,
        ))
        .expect("reflected field should be readable")
        .field
        .value
}

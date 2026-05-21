use zircon_runtime::scene::{EntityId, Scene, SceneEditorHierarchyRow, SceneEditorInspectorField};
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::scene::viewport::SceneViewportSettings;

use super::{
    SceneEditModeProjection, SceneHierarchyRow, SceneInspectorField, SceneInspectorFieldValue,
    SceneViewportStats, SceneViewportToolbarState,
};

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const HIERARCHY_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";
const LOCAL_TRANSFORM_TYPE_PATH: &str = "zircon_runtime::scene::components::LocalTransform";
const ACTIVE_SELF_TYPE_PATH: &str = "zircon_runtime::scene::components::ActiveSelf";
const ACTIVE_IN_HIERARCHY_TYPE_PATH: &str = "zircon_runtime::scene::components::ActiveInHierarchy";
const RENDER_LAYER_MASK_TYPE_PATH: &str = "zircon_runtime::scene::components::RenderLayerMask";
const MOBILITY_TYPE_PATH: &str = "zircon_runtime::core::framework::scene::Mobility";
const CAMERA_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::CameraComponent";
const MESH_RENDERER_TYPE_PATH: &str = "zircon_runtime::scene::components::MeshRenderer";
const AMBIENT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::AmbientLight";
const DIRECTIONAL_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::DirectionalLight";
const POINT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::PointLight";
const RECT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::RectLight";
const SPOT_LIGHT_TYPE_PATH: &str = "zircon_runtime::scene::components::SpotLight";
const RIGID_BODY_COMPONENT_TYPE_PATH: &str =
    "zircon_runtime::scene::components::RigidBodyComponent";

pub(crate) fn build_scene_edit_mode_projection(
    scene: &Scene,
    settings: &SceneViewportSettings,
    selected: Option<EntityId>,
    handle_drag_active: bool,
) -> SceneEditModeProjection {
    let runtime_projection = scene.editor_projection(selected);
    let selected_entity = runtime_projection.selected_entity;

    SceneEditModeProjection {
        selected_entity,
        hierarchy_rows: runtime_projection
            .hierarchy_rows
            .into_iter()
            .map(SceneHierarchyRow::from)
            .collect(),
        inspector_fields: runtime_projection
            .inspector_fields
            .into_iter()
            .filter_map(scene_inspector_field_from_runtime)
            .collect(),
        toolbar: build_toolbar_state(settings, selected_entity, handle_drag_active),
        stats: build_stats(scene, selected_entity),
    }
}

impl From<SceneEditorHierarchyRow> for SceneHierarchyRow {
    fn from(row: SceneEditorHierarchyRow) -> Self {
        Self {
            entity: row.entity,
            parent: row.parent,
            depth: row.depth,
            display_name: row.display_name,
            kind: row.kind,
            selected: row.selected,
            active_in_hierarchy: row.active_in_hierarchy,
            has_children: row.has_children,
        }
    }
}

fn scene_inspector_field_from_runtime(
    field: SceneEditorInspectorField,
) -> Option<SceneInspectorField> {
    let property_path = property_path_for_field(&field);
    let value = scene_inspector_value_from_reflected(&field)?;
    let editable = field.editable && property_path.is_some();
    Some(SceneInspectorField {
        component: component_label(&field).to_string(),
        label: field_label(&field),
        property_path,
        value,
        editable,
    })
}

fn scene_inspector_value_from_reflected(
    field: &SceneEditorInspectorField,
) -> Option<SceneInspectorFieldValue> {
    match &field.value {
        ReflectedValue::Bool(value) => Some(SceneInspectorFieldValue::Bool(*value)),
        ReflectedValue::Unsigned(value) => Some(SceneInspectorFieldValue::Unsigned(*value)),
        ReflectedValue::Scalar(value) => Some(SceneInspectorFieldValue::Scalar(*value)),
        ReflectedValue::String(value) => Some(SceneInspectorFieldValue::Text(value.clone())),
        ReflectedValue::Enum(value) => Some(SceneInspectorFieldValue::Enum(value.clone())),
        ReflectedValue::Vec2(value) => Some(SceneInspectorFieldValue::Vec2(*value)),
        ReflectedValue::Vec3(value) => Some(SceneInspectorFieldValue::Vec3(*value)),
        ReflectedValue::Vec4(value) if is_transform_rotation(field) => {
            Some(SceneInspectorFieldValue::Quaternion(*value))
        }
        ReflectedValue::Vec4(value) => Some(SceneInspectorFieldValue::Vec4(*value)),
        ReflectedValue::Quaternion(value) => Some(SceneInspectorFieldValue::Quaternion(*value)),
        ReflectedValue::Entity(entity) => Some(SceneInspectorFieldValue::Entity(*entity)),
        ReflectedValue::Resource(value) => Some(SceneInspectorFieldValue::Resource(value.clone())),
        ReflectedValue::Null
        | ReflectedValue::Integer(_)
        | ReflectedValue::List(_)
        | ReflectedValue::Map(_)
        | ReflectedValue::Json(_) => None,
    }
}

fn is_transform_rotation(field: &SceneEditorInspectorField) -> bool {
    field.component_type_path == LOCAL_TRANSFORM_TYPE_PATH && field.field_name == "rotation"
}

fn property_path_for_field(field: &SceneEditorInspectorField) -> Option<String> {
    if field.component_type_path == ACTIVE_IN_HIERARCHY_TYPE_PATH {
        return None;
    }
    let component = property_component_name(field);
    let property = property_field_name(field);
    Some(format!("{component}.{property}"))
}

fn property_component_name(field: &SceneEditorInspectorField) -> &str {
    match field.component_type_path.as_str() {
        NAME_TYPE_PATH => "Name",
        HIERARCHY_TYPE_PATH => "Hierarchy",
        LOCAL_TRANSFORM_TYPE_PATH => "Transform",
        ACTIVE_SELF_TYPE_PATH => "Active",
        RENDER_LAYER_MASK_TYPE_PATH => "RenderLayer",
        MOBILITY_TYPE_PATH => "Mobility",
        CAMERA_COMPONENT_TYPE_PATH => "Camera",
        MESH_RENDERER_TYPE_PATH => "MeshRenderer",
        AMBIENT_LIGHT_TYPE_PATH => "AmbientLight",
        DIRECTIONAL_LIGHT_TYPE_PATH => "DirectionalLight",
        POINT_LIGHT_TYPE_PATH => "PointLight",
        RECT_LIGHT_TYPE_PATH => "RectLight",
        SPOT_LIGHT_TYPE_PATH => "SpotLight",
        RIGID_BODY_COMPONENT_TYPE_PATH => "RigidBody",
        _ if field.plugin_owned => field.component_type_path.as_str(),
        _ => field.component_display_name.as_str(),
    }
}

fn property_field_name(field: &SceneEditorInspectorField) -> &str {
    match (
        field.component_type_path.as_str(),
        field.field_name.as_str(),
    ) {
        (ACTIVE_SELF_TYPE_PATH, "value") => "enabled",
        (RIGID_BODY_COMPONENT_TYPE_PATH, "body_type") => "kind",
        _ => field.field_name.as_str(),
    }
}

fn component_label(field: &SceneEditorInspectorField) -> &str {
    match field.component_type_path.as_str() {
        LOCAL_TRANSFORM_TYPE_PATH => "Transform",
        ACTIVE_SELF_TYPE_PATH | ACTIVE_IN_HIERARCHY_TYPE_PATH => "Active",
        RENDER_LAYER_MASK_TYPE_PATH => "Render Layer",
        CAMERA_COMPONENT_TYPE_PATH => "Camera",
        MESH_RENDERER_TYPE_PATH => "Mesh Renderer",
        AMBIENT_LIGHT_TYPE_PATH => "Ambient Light",
        DIRECTIONAL_LIGHT_TYPE_PATH => "Directional Light",
        POINT_LIGHT_TYPE_PATH => "Point Light",
        RECT_LIGHT_TYPE_PATH => "Rect Light",
        SPOT_LIGHT_TYPE_PATH => "Spot Light",
        RIGID_BODY_COMPONENT_TYPE_PATH => "Rigid Body",
        _ => field.component_display_name.as_str(),
    }
}

fn field_label(field: &SceneEditorInspectorField) -> String {
    match (
        field.component_type_path.as_str(),
        field.field_name.as_str(),
    ) {
        (ACTIVE_SELF_TYPE_PATH, "value") => "Enabled".to_string(),
        (ACTIVE_IN_HIERARCHY_TYPE_PATH, "value") => "Active In Hierarchy".to_string(),
        (RIGID_BODY_COMPONENT_TYPE_PATH, "body_type") => "Kind".to_string(),
        (CAMERA_COMPONENT_TYPE_PATH, "fov_y_radians") => "FOV Y".to_string(),
        (CAMERA_COMPONENT_TYPE_PATH, "z_near") => "Near Clip".to_string(),
        (CAMERA_COMPONENT_TYPE_PATH, "z_far") => "Far Clip".to_string(),
        _ => title_case_identifier(&field.field_display_name),
    }
}

fn title_case_identifier(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = first.to_ascii_uppercase().to_string();
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_toolbar_state(
    settings: &SceneViewportSettings,
    selected: Option<EntityId>,
    handle_drag_active: bool,
) -> SceneViewportToolbarState {
    SceneViewportToolbarState {
        tool: settings.tool,
        transform_space: settings.transform_space,
        projection_mode: settings.projection_mode,
        view_orientation: settings.view_orientation,
        display_mode: settings.display_mode,
        grid_mode: settings.grid_mode,
        preview_lighting: settings.preview_lighting,
        preview_skybox: settings.preview_skybox,
        gizmos_enabled: settings.gizmos_enabled,
        has_selection: selected.is_some(),
        can_frame_selection: selected.is_some(),
        handle_drag_active,
    }
}

fn build_stats(scene: &Scene, selected: Option<EntityId>) -> SceneViewportStats {
    let mut stats = SceneViewportStats {
        selected_entity: selected,
        ..SceneViewportStats::default()
    };
    for node in scene.node_records() {
        stats.node_count += 1;
        if scene.active_in_hierarchy(node.id) == Some(true) {
            stats.visible_node_count += 1;
        }
        if node.camera.is_some() {
            stats.camera_count += 1;
        }
        if node.mesh.is_some() {
            stats.mesh_count += 1;
        }
        if node.directional_light.is_some()
            || node.ambient_light.is_some()
            || node.point_light.is_some()
            || node.rect_light.is_some()
            || node.spot_light.is_some()
        {
            stats.light_count += 1;
        }
    }
    stats
}

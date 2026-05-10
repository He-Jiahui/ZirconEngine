use std::collections::{HashMap, HashSet};

use crate::scene::viewport::SceneViewportSettings;
use zircon_runtime::scene::components::{Mobility, NodeKind, SceneNode};
use zircon_runtime::scene::{EntityId, Scene};

use super::{
    SceneEditModeProjection, SceneHierarchyRow, SceneInspectorField, SceneInspectorFieldValue,
    SceneViewportStats, SceneViewportToolbarState,
};

pub(crate) fn build_scene_edit_mode_projection(
    scene: &Scene,
    settings: &SceneViewportSettings,
    selected: Option<EntityId>,
    handle_drag_active: bool,
) -> SceneEditModeProjection {
    let selected_entity = selected.filter(|entity| scene.contains_entity(*entity));
    let selected_node = selected_entity.and_then(|entity| scene.find_node(entity));

    SceneEditModeProjection {
        selected_entity,
        hierarchy_rows: build_hierarchy_rows(scene, selected_entity),
        inspector_fields: build_inspector_fields(scene, selected_node),
        toolbar: build_toolbar_state(settings, selected_entity, handle_drag_active),
        stats: build_stats(scene, selected_entity),
    }
}

fn build_hierarchy_rows(scene: &Scene, selected: Option<EntityId>) -> Vec<SceneHierarchyRow> {
    let nodes = scene.node_records();
    let node_by_entity = nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    let mut children_by_parent: HashMap<Option<EntityId>, Vec<EntityId>> = HashMap::new();
    for node in &nodes {
        children_by_parent
            .entry(node.parent)
            .or_default()
            .push(node.id);
    }

    let mut rows = Vec::new();
    let mut visited = HashSet::new();
    if let Some(roots) = children_by_parent.get(&None) {
        for root in roots {
            push_hierarchy_row(
                scene,
                &node_by_entity,
                &children_by_parent,
                selected,
                *root,
                0,
                &mut visited,
                &mut rows,
            );
        }
    }

    for node in &nodes {
        if !visited.contains(&node.id) {
            push_hierarchy_row(
                scene,
                &node_by_entity,
                &children_by_parent,
                selected,
                node.id,
                0,
                &mut visited,
                &mut rows,
            );
        }
    }

    rows
}

fn push_hierarchy_row(
    scene: &Scene,
    node_by_entity: &HashMap<EntityId, &SceneNode>,
    children_by_parent: &HashMap<Option<EntityId>, Vec<EntityId>>,
    selected: Option<EntityId>,
    entity: EntityId,
    depth: u32,
    visited: &mut HashSet<EntityId>,
    rows: &mut Vec<SceneHierarchyRow>,
) {
    if !visited.insert(entity) {
        return;
    }
    let Some(node) = node_by_entity.get(&entity).copied() else {
        return;
    };
    let children = children_by_parent.get(&Some(entity));
    rows.push(SceneHierarchyRow {
        entity,
        parent: node.parent,
        depth,
        display_name: node.name.clone(),
        kind: node_kind_label(&node.kind).to_string(),
        selected: selected == Some(entity),
        active_in_hierarchy: scene.active_in_hierarchy(entity).unwrap_or(false),
        has_children: children.is_some_and(|children| !children.is_empty()),
    });

    if let Some(children) = children {
        for child in children {
            push_hierarchy_row(
                scene,
                node_by_entity,
                children_by_parent,
                selected,
                *child,
                depth + 1,
                visited,
                rows,
            );
        }
    }
}

fn build_inspector_fields(
    scene: &Scene,
    selected_node: Option<SceneNode>,
) -> Vec<SceneInspectorField> {
    let Some(node) = selected_node.as_ref() else {
        return Vec::new();
    };

    let mut fields = Vec::new();
    push_field(
        &mut fields,
        "Name",
        "Name",
        Some("Name.value"),
        SceneInspectorFieldValue::Text(node.name.clone()),
        true,
    );
    push_field(
        &mut fields,
        "Hierarchy",
        "Parent",
        Some("Hierarchy.parent"),
        SceneInspectorFieldValue::Entity(node.parent),
        true,
    );
    push_field(
        &mut fields,
        "Transform",
        "Translation",
        Some("Transform.translation"),
        SceneInspectorFieldValue::Vec3(node.transform.translation.to_array()),
        true,
    );
    push_field(
        &mut fields,
        "Transform",
        "Rotation",
        Some("Transform.rotation"),
        SceneInspectorFieldValue::Quaternion(node.transform.rotation.to_array()),
        true,
    );
    push_field(
        &mut fields,
        "Transform",
        "Scale",
        Some("Transform.scale"),
        SceneInspectorFieldValue::Vec3(node.transform.scale.to_array()),
        true,
    );
    push_field(
        &mut fields,
        "Active",
        "Enabled",
        Some("Active.enabled"),
        SceneInspectorFieldValue::Bool(scene.active_self(node.id).unwrap_or(true)),
        true,
    );
    push_field(
        &mut fields,
        "Active",
        "Active In Hierarchy",
        None,
        SceneInspectorFieldValue::Bool(scene.active_in_hierarchy(node.id).unwrap_or(false)),
        false,
    );
    if let Some(mask) = scene.render_layer_mask(node.id) {
        push_field(
            &mut fields,
            "Render Layer",
            "Mask",
            Some("RenderLayer.mask"),
            SceneInspectorFieldValue::Unsigned(mask as u64),
            true,
        );
    }
    if let Some(mobility) = scene.mobility(node.id) {
        push_field(
            &mut fields,
            "Mobility",
            "Kind",
            Some("Mobility.kind"),
            SceneInspectorFieldValue::Enum(mobility_label(mobility).to_string()),
            true,
        );
    }

    if let Some(camera) = node.camera.as_ref() {
        push_field(
            &mut fields,
            "Camera",
            "FOV Y",
            Some("Camera.fov_y_radians"),
            SceneInspectorFieldValue::Scalar(camera.fov_y_radians),
            true,
        );
        push_field(
            &mut fields,
            "Camera",
            "Near Clip",
            Some("Camera.z_near"),
            SceneInspectorFieldValue::Scalar(camera.z_near),
            true,
        );
        push_field(
            &mut fields,
            "Camera",
            "Far Clip",
            Some("Camera.z_far"),
            SceneInspectorFieldValue::Scalar(camera.z_far),
            true,
        );
    }
    if let Some(mesh) = node.mesh.as_ref() {
        push_field(
            &mut fields,
            "Mesh Renderer",
            "Model",
            Some("MeshRenderer.model"),
            SceneInspectorFieldValue::Resource(mesh.model.id().to_string()),
            false,
        );
        push_field(
            &mut fields,
            "Mesh Renderer",
            "Material",
            Some("MeshRenderer.material"),
            SceneInspectorFieldValue::Resource(mesh.material.id().to_string()),
            false,
        );
        push_field(
            &mut fields,
            "Mesh Renderer",
            "Tint",
            Some("MeshRenderer.tint"),
            SceneInspectorFieldValue::Vec4(mesh.tint.to_array()),
            true,
        );
    }
    if let Some(light) = node.directional_light.as_ref() {
        push_field(
            &mut fields,
            "Directional Light",
            "Direction",
            Some("DirectionalLight.direction"),
            SceneInspectorFieldValue::Vec3(light.direction.to_array()),
            true,
        );
        push_field(
            &mut fields,
            "Directional Light",
            "Color",
            Some("DirectionalLight.color"),
            SceneInspectorFieldValue::Vec3(light.color.to_array()),
            true,
        );
        push_field(
            &mut fields,
            "Directional Light",
            "Intensity",
            Some("DirectionalLight.intensity"),
            SceneInspectorFieldValue::Scalar(light.intensity),
            true,
        );
    }
    if let Some(light) = node.point_light.as_ref() {
        push_field(
            &mut fields,
            "Point Light",
            "Color",
            Some("PointLight.color"),
            SceneInspectorFieldValue::Vec3(light.color.to_array()),
            true,
        );
        push_field(
            &mut fields,
            "Point Light",
            "Intensity",
            Some("PointLight.intensity"),
            SceneInspectorFieldValue::Scalar(light.intensity),
            true,
        );
        push_field(
            &mut fields,
            "Point Light",
            "Range",
            Some("PointLight.range"),
            SceneInspectorFieldValue::Scalar(light.range),
            true,
        );
    }
    if let Some(light) = node.spot_light.as_ref() {
        push_field(
            &mut fields,
            "Spot Light",
            "Direction",
            Some("SpotLight.direction"),
            SceneInspectorFieldValue::Vec3(light.direction.to_array()),
            true,
        );
        push_field(
            &mut fields,
            "Spot Light",
            "Color",
            Some("SpotLight.color"),
            SceneInspectorFieldValue::Vec3(light.color.to_array()),
            true,
        );
        push_field(
            &mut fields,
            "Spot Light",
            "Intensity",
            Some("SpotLight.intensity"),
            SceneInspectorFieldValue::Scalar(light.intensity),
            true,
        );
        push_field(
            &mut fields,
            "Spot Light",
            "Range",
            Some("SpotLight.range"),
            SceneInspectorFieldValue::Scalar(light.range),
            true,
        );
        push_field(
            &mut fields,
            "Spot Light",
            "Inner Angle",
            Some("SpotLight.inner_angle_radians"),
            SceneInspectorFieldValue::Scalar(light.inner_angle_radians),
            true,
        );
        push_field(
            &mut fields,
            "Spot Light",
            "Outer Angle",
            Some("SpotLight.outer_angle_radians"),
            SceneInspectorFieldValue::Scalar(light.outer_angle_radians),
            true,
        );
    }

    fields
}

fn push_field(
    fields: &mut Vec<SceneInspectorField>,
    component: &str,
    label: &str,
    property_path: Option<&str>,
    value: SceneInspectorFieldValue,
    editable: bool,
) {
    fields.push(SceneInspectorField {
        component: component.to_string(),
        label: label.to_string(),
        property_path: property_path.map(ToOwned::to_owned),
        value,
        editable: editable && property_path.is_some(),
    });
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
            || node.point_light.is_some()
            || node.spot_light.is_some()
        {
            stats.light_count += 1;
        }
    }
    stats
}

fn node_kind_label(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "Camera",
        NodeKind::Cube => "Cube",
        NodeKind::Mesh => "Mesh",
        NodeKind::DirectionalLight => "Directional Light",
        NodeKind::PointLight => "Point Light",
        NodeKind::SpotLight => "Spot Light",
    }
}

fn mobility_label(mobility: Mobility) -> &'static str {
    match mobility {
        Mobility::Dynamic => "dynamic",
        Mobility::Static => "static",
    }
}

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectFieldValue};

use crate::scene::components::{NodeKind, SceneNode};
use crate::scene::reflect::RuntimeTypeRegistration;
use crate::scene::{EntityId, World};

use super::{WorldInspectionField, WorldInspectionHierarchyRow};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldInspection {
    pub focused_entity: Option<EntityId>,
    pub hierarchy_rows: Vec<WorldInspectionHierarchyRow>,
    pub fields: Vec<WorldInspectionField>,
}

impl WorldInspection {
    pub fn from_world(world: &World, focused: Option<EntityId>) -> Self {
        let focused_entity = focused.filter(|entity| world.contains_entity(*entity));
        Self {
            focused_entity,
            hierarchy_rows: build_hierarchy_rows(world, focused_entity),
            fields: focused_entity
                .map(|entity| build_inspection_fields(world, entity))
                .unwrap_or_default(),
        }
    }
}

impl World {
    pub fn inspect_world(&self, focused: Option<EntityId>) -> WorldInspection {
        WorldInspection::from_world(self, focused)
    }
}

fn build_hierarchy_rows(
    world: &World,
    focused: Option<EntityId>,
) -> Vec<WorldInspectionHierarchyRow> {
    let nodes = world.node_records();
    let node_by_entity = nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    let mut children_by_parent: BTreeMap<Option<EntityId>, Vec<EntityId>> = BTreeMap::new();
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
                world,
                &node_by_entity,
                &children_by_parent,
                focused,
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
                world,
                &node_by_entity,
                &children_by_parent,
                focused,
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
    world: &World,
    node_by_entity: &HashMap<EntityId, &SceneNode>,
    children_by_parent: &BTreeMap<Option<EntityId>, Vec<EntityId>>,
    focused: Option<EntityId>,
    entity: EntityId,
    depth: u32,
    visited: &mut HashSet<EntityId>,
    rows: &mut Vec<WorldInspectionHierarchyRow>,
) {
    if !visited.insert(entity) {
        return;
    }
    let Some(node) = node_by_entity.get(&entity).copied() else {
        return;
    };
    let children = children_by_parent.get(&Some(entity));
    rows.push(WorldInspectionHierarchyRow {
        entity,
        parent: node.parent,
        depth,
        display_name: node.name.clone(),
        kind: node_kind_label(&node.kind).to_string(),
        focused: focused == Some(entity),
        active_in_hierarchy: world.active_in_hierarchy(entity).unwrap_or(false),
        has_children: children.is_some_and(|children| !children.is_empty()),
    });

    if let Some(children) = children {
        for child in children {
            push_hierarchy_row(
                world,
                node_by_entity,
                children_by_parent,
                focused,
                *child,
                depth + 1,
                visited,
                rows,
            );
        }
    }
}

fn build_inspection_fields(world: &World, entity: EntityId) -> Vec<WorldInspectionField> {
    let mut fields = Vec::new();
    for runtime in world.type_registry().iter() {
        if let Ok(component_fields) = reflected_component_fields(world, entity, runtime) {
            fields.extend(component_fields);
        }
    }
    fields.sort_by(|left, right| {
        left.component_display_name
            .cmp(&right.component_display_name)
            .then_with(|| left.field_display_name.cmp(&right.field_display_name))
            .then_with(|| left.component_type_path.cmp(&right.component_type_path))
            .then_with(|| left.field_name.cmp(&right.field_name))
    });
    fields
}

fn reflected_component_fields(
    world: &World,
    entity: EntityId,
    runtime: &RuntimeTypeRegistration,
) -> Result<Vec<WorldInspectionField>, zircon_runtime_interface::reflect::ReflectError> {
    let metadata = &runtime.registration;
    if !metadata.is_component || !metadata.editor_visible {
        return Ok(Vec::new());
    }
    let Some(adapter) = &runtime.component else {
        return Ok(Vec::new());
    };
    if !adapter.contains(world, entity) {
        return Ok(Vec::new());
    }

    let values = adapter
        .read_fields(world, entity)?
        .into_iter()
        .map(|field| (field.field_name.clone(), field))
        .collect::<HashMap<_, _>>();

    Ok(metadata
        .type_info
        .fields
        .iter()
        .filter(|field| field.editor_visible)
        .filter_map(|field| {
            values.get(&field.name).map(|value| {
                inspection_field_from_reflection(runtime, field, value, metadata.plugin_owned)
            })
        })
        .collect())
}

fn inspection_field_from_reflection(
    runtime: &RuntimeTypeRegistration,
    field: &ReflectFieldInfo,
    value: &ReflectFieldValue,
    plugin_owned: bool,
) -> WorldInspectionField {
    let metadata = &runtime.registration;
    WorldInspectionField {
        component_type_path: metadata.type_path.type_path.clone(),
        component_display_name: metadata.display_name.clone(),
        field_name: field.name.clone(),
        field_display_name: field.display_name.clone(),
        value_type_path: field.value_type_path.clone(),
        value: value.value.clone(),
        writable: field.editable,
        serializable: field.serializable,
        plugin_owned,
    }
}

fn node_kind_label(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "Camera",
        NodeKind::Cube => "Cube",
        NodeKind::Mesh => "Mesh",
        NodeKind::AmbientLight => "Ambient Light",
        NodeKind::DirectionalLight => "Directional Light",
        NodeKind::PointLight => "Point Light",
        NodeKind::RectLight => "Rect Light",
        NodeKind::SpotLight => "Spot Light",
    }
}

use super::*;
use crate::ui::template_runtime::{
    RetainedUiHostAdapter, RetainedUiHostNodeProjection, RetainedUiHostProjection,
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiSize,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

fn source_node(id: u64, path: &str, component: &str) -> UiTreeNode {
    let mut node = UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(path));
    node.template_metadata = Some(UiTemplateNodeMetadata {
        component: component.into(),
        source_path: Some("fixtures/normal-inventory.zui".into()),
        source_node_id: Some(path.into()),
        instance_path: Some(Vec::new()),
        attributes: BTreeMap::from([("label".into(), toml::Value::String(format!("Label {id}")))]),
        ..Default::default()
    });
    node
}

fn projection(surface: &UiSurface) -> RetainedUiHostProjection {
    let frame = surface.surface_frame();
    let nodes = frame
        .arranged_tree
        .nodes
        .iter()
        .map(|arranged| {
            let live = surface.tree.node(arranged.node_id).unwrap();
            let source = live.template_metadata.clone().unwrap_or_default();
            let mut projected = RetainedUiHostAdapter::build_node(&RetainedUiHostNodeProjection {
                node_id: arranged.node_path.0.clone(),
                surface_node_id: Some(arranged.node_id),
                has_workbench_icon_tooltip: false,
                parent_id: arranged
                    .parent
                    .and_then(|id| frame.arranged_tree.get(id))
                    .map(|node| node.node_path.0.clone()),
                component: source.component,
                control_id: source.control_id,
                source_path: source.source_path,
                source_node_id: source.source_node_id,
                instance_path: source.instance_path,
                parent_source_path: None,
                parent_source_node_id: None,
                parent_instance_path: None,
                frame: arranged.frame,
                clip_frame: Some(arranged.clip_frame),
                z_index: arranged.z_index,
                attributes: source.attributes,
                style_overrides: source.style_overrides,
                style_tokens: source.style_tokens,
                bindings: Vec::new(),
            });
            projected.source_surface_frame = Some(frame.clone());
            projected
        })
        .collect();
    RetainedUiHostProjection {
        source_surface_frame: Some(frame),
        document_id: "fixture.normal-inventory".into(),
        nodes,
    }
}

fn fixture(missing_metadata: bool) -> (UiSurface, RetainedUiHostProjection) {
    let mut surface = UiSurface::new(UiTreeId::new("normal-inventory-fixture"));
    surface
        .tree
        .insert_root(source_node(1, "Root", "Container"));
    let mut hidden = source_node(2, "Root/Hidden", "Container");
    hidden.state_flags.visible = false;
    surface.tree.insert_child(UiNodeId::new(1), hidden).unwrap();
    let mut decoration = source_node(3, "Root/Decoration", "Divider");
    if missing_metadata {
        decoration.template_metadata = None;
    }
    surface
        .tree
        .insert_child(UiNodeId::new(1), decoration)
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            source_node(4, "Root/Hidden/Label", "Label"),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(100.0, 80.0)).unwrap();
    let projected = projection(&surface);
    (surface, projected)
}

fn capture(surface: &UiSurface, projection: &RetainedUiHostProjection) -> NormalSurfaceInventory {
    capture_normal_surface_inventory("fixture", projection, surface, None, 1.25)
}

fn row<'a>(inventory: &'a NormalSurfaceInventory, path: &str) -> &'a Value {
    inventory.observation["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["nodeId"] == path)
        .unwrap()
}

#[test]
fn normal_inventory_keeps_hidden_and_control_less_nodes() {
    let (surface, projection) = fixture(false);
    let inventory = capture(&surface, &projection);
    assert_eq!(
        inventory.observation["inventoryAudit"]["complete"], true,
        "{}",
        inventory.observation
    );
    assert_eq!(
        inventory.observation["nodes"].as_array().unwrap().len(),
        surface.tree.nodes.len()
    );
    assert!(inventory.observation["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|node| node["controlId"].is_null()));
    assert_eq!(row(&inventory, "Root/Hidden")["selfVisible"], false);
    assert_eq!(
        row(&inventory, "Root/Hidden/Label")["ancestorVisible"],
        false
    );
    assert_eq!(row(&inventory, "Root/Hidden/Label")["visible"], false);
    assert_eq!(row(&inventory, "Root/Decoration")["component"], "Divider");
    assert_eq!(row(&inventory, "Root/Hidden/Label")["text"], "Label 4");
    let publication = NormalPresentInventory::new(vec![inventory]).observation();
    assert_eq!(publication["inventoryAudit"]["complete"], true);
    assert_eq!(publication["complete"], false);
}

#[test]
fn normal_inventory_retains_nodes_with_missing_source_metadata() {
    let (surface, projection) = fixture(true);
    let inventory = capture(&surface, &projection);
    assert_eq!(inventory.observation["nodes"].as_array().unwrap().len(), 4);
    assert!(row(&inventory, "Root/Decoration")["rawMetadata"].is_null());
    assert_eq!(inventory.observation["inventoryAudit"]["complete"], false);
    assert!(!inventory.observation["inventoryAudit"]["issues"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn normal_inventory_reports_raw_nodes_missing_from_the_arranged_frame() {
    let (mut surface, projection) = fixture(false);
    surface
        .tree
        .insert_root(source_node(5, "AddedWithoutLayout", "Label"));
    let inventory = capture(&surface, &projection);
    assert_eq!(inventory.observation["inventoryAudit"]["complete"], false);
    assert_eq!(inventory.observation["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(
        inventory.observation["unarrangedNodes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        inventory.observation["unarrangedNodes"][0]["nodeId"],
        "AddedWithoutLayout"
    );
}

#[test]
fn normal_inventory_reports_missing_and_duplicate_projection_rows() {
    let (surface, mut projection) = fixture(false);
    projection.nodes.push(projection.nodes[0].clone());
    let duplicate = capture(&surface, &projection);
    assert_eq!(duplicate.observation["inventoryAudit"]["complete"], false);
    assert_eq!(duplicate.observation["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(
        duplicate.observation["unmatchedProjectionNodes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    projection.nodes.pop();
    let missing_path = projection.nodes.pop().unwrap().node_id;
    let missing = capture(&surface, &projection);
    assert_eq!(missing.observation["inventoryAudit"]["complete"], false);
    assert!(row(&missing, &missing_path)["projected"].is_null());
}

#[test]
fn normal_inventory_rejects_a_different_projection_frame_anchor() {
    let (surface, mut projection) = fixture(false);
    let (other, _) = fixture(false);
    let other_frame = other.surface_frame();
    assert!(!Arc::ptr_eq(
        projection.source_surface_frame.as_ref().unwrap(),
        &other_frame
    ));
    projection.source_surface_frame = Some(other_frame.clone());
    let inventory = capture(&surface, &projection);
    assert_eq!(inventory.observation["inventoryAudit"]["complete"], false);
    assert!(Arc::ptr_eq(&inventory._source_frame, &other_frame));
}

#[test]
fn normal_inventory_allows_older_node_frame_handles_under_the_current_projection_anchor() {
    let (surface, mut projection) = fixture(false);
    let (other, _) = fixture(false);
    let older_frame = other.surface_frame();
    for node in &mut projection.nodes {
        node.source_surface_frame = Some(older_frame.clone());
    }
    let inventory = capture(&surface, &projection);
    assert_eq!(
        inventory.observation["inventoryAudit"]["complete"], true,
        "{}",
        inventory.observation
    );
}

#[test]
fn normal_inventory_preserves_the_frozen_frame_after_runtime_layout_changes() {
    let (mut surface, initial_projection) = fixture(false);
    let inventory = capture(&surface, &initial_projection);
    let original = inventory.observation.as_ref().clone();
    surface.tree.insert_root(source_node(5, "Later", "Label"));
    surface.compute_layout(UiSize::new(120.0, 90.0)).unwrap();
    let later_projection = projection(&surface);
    let later = capture(&surface, &later_projection);
    assert!(!Arc::ptr_eq(&inventory._source_frame, &later._source_frame));
    assert_eq!(inventory.observation.as_ref(), &original);
    assert_eq!(inventory.observation["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(later.observation["nodes"].as_array().unwrap().len(), 5);
}

#[test]
fn normal_inventory_detects_raw_geometry_and_root_drift() {
    let (mut surface, projection) = fixture(false);
    let frozen_bounds = projection
        .nodes
        .iter()
        .find(|node| node.node_id == "Root/Decoration")
        .unwrap()
        .frame;
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(3))
        .unwrap()
        .layout_cache
        .frame
        .x += 1.0;
    let geometry_drift = capture(&surface, &projection);
    assert_eq!(
        geometry_drift.observation["inventoryAudit"]["complete"],
        false
    );
    assert_eq!(
        row(&geometry_drift, "Root/Decoration")["bounds"],
        json!(frozen_bounds)
    );

    let (mut surface, projection) = fixture(false);
    surface.tree.roots.clear();
    let root_drift = capture(&surface, &projection);
    assert_eq!(root_drift.observation["inventoryAudit"]["complete"], false);
    assert_eq!(root_drift.observation["nodes"].as_array().unwrap().len(), 4);
}

fn inspector_repeat_fixture(owner_count: u64) -> (UiSurface, RetainedUiHostProjection) {
    use zircon_runtime::ui::surface::UiVirtualListItemKey;
    use zircon_runtime_interface::ui::layout::{
        UiContainerKind, UiScrollState, UiScrollableBoxConfig, UiVirtualListConfig,
    };

    let mut surface = UiSurface::new(UiTreeId::new("normal-inspector-repeat"));
    for owner_index in 0..owner_count {
        let owner_value = 1 + owner_index * 10;
        let owner_id = UiNodeId::new(owner_value);
        let owner_path = format!("Owner{owner_index}");
        let owner = source_node(owner_value, &owner_path, "ScrollableBox")
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 20.0,
                    overscan: 0,
                }),
                ..Default::default()
            }))
            .with_scroll_state(UiScrollState {
                offset: 0.0,
                viewport_extent: 40.0,
                content_extent: 60.0,
            });
        surface.tree.insert_root(owner);
        let mut changes = Vec::new();
        let report = surface
            .reconcile_virtual_list_materialization_with_keys(owner_id, 3, &mut changes, |index| {
                UiVirtualListItemKey::new(11 + index as u128)
            })
            .unwrap();
        assert_eq!(report.slot_capacity, 3);
        let mut slots = Vec::new();
        for slot in 0..report.slot_capacity {
            let value = owner_value + 1 + slot as u64;
            let id = UiNodeId::new(value);
            let mut node = source_node(value, &format!("{owner_path}/Slot{slot}"), "InputField");
            node.template_metadata.as_mut().unwrap().source_node_id = Some("prototype".into());
            surface.tree.insert_child(owner_id, node).unwrap();
            slots.push(id);
        }
        surface
            .register_virtual_list_slots(owner_id, &slots)
            .unwrap();
        for id in slots {
            let binding = surface.virtual_list_binding_for_node(owner_id, id).unwrap();
            let metadata = surface
                .tree
                .node_mut(id)
                .unwrap()
                .template_metadata
                .as_mut()
                .unwrap();
            metadata.attributes.insert(
                "inspector_property_item_key".into(),
                toml::Value::String(binding.item_key.get().to_string()),
            );
            metadata.attributes.insert(
                "inspector_property_field_id".into(),
                toml::Value::String(format!("field.{}", binding.item_key.get())),
            );
        }
    }
    surface.compute_layout(UiSize::new(100.0, 80.0)).unwrap();
    let projected = projection(&surface);
    (surface, projected)
}

#[test]
fn normal_inventory_distinguishes_real_inspector_items_and_owners() {
    let (surface, projection) = inspector_repeat_fixture(2);
    let inventory = capture(&surface, &projection);
    assert_eq!(
        inventory.observation["inventoryAudit"]["complete"], true,
        "{}",
        inventory.observation
    );
    let first = row(&inventory, "Owner0/Slot0");
    let next = row(&inventory, "Owner0/Slot1");
    let other = row(&inventory, "Owner1/Slot0");
    assert_eq!(first["sourceNodeId"], next["sourceNodeId"]);
    assert_ne!(first["itemKey"], next["itemKey"]);
    assert_eq!(first["itemKey"], other["itemKey"]);
    assert_ne!(first["parent"], other["parent"]);
    assert_eq!(first["inspectorRepeat"]["runtimeBindingCurrent"], true);
    let publication = NormalPresentInventory::new(vec![inventory]).observation();
    assert_eq!(publication["inventoryAudit"]["complete"], true);
    assert_eq!(publication["complete"], false);
}

#[test]
fn normal_inventory_rejects_duplicate_partial_and_mismatched_inspector_keys() {
    for invalid in ["11", "0012", "340282366920938463463374607431768211456", ""] {
        let (mut surface, _) = inspector_repeat_fixture(1);
        surface
            .tree
            .node_mut(UiNodeId::new(3))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert(
                "inspector_property_item_key".into(),
                toml::Value::String(invalid.into()),
            );
        let projected = projection(&surface);
        assert_eq!(
            capture(&surface, &projected).observation["inventoryAudit"]["complete"],
            false,
            "{invalid}"
        );
    }
    for remove in [false, true] {
        let (mut surface, _) = inspector_repeat_fixture(1);
        let metadata = surface
            .tree
            .node_mut(UiNodeId::new(3))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap();
        for key in ["inspector_property_item_key", "inspector_property_field_id"] {
            if remove {
                metadata.attributes.remove(key);
            } else {
                metadata
                    .attributes
                    .insert(key.into(), toml::Value::String(String::new()));
            }
        }
        let projected = projection(&surface);
        assert_eq!(
            capture(&surface, &projected).observation["inventoryAudit"]["complete"],
            false,
            "a current Runtime-bound row cannot lose both model identity fields"
        );
    }
    let (surface, mut projected) = inspector_repeat_fixture(1);
    projected
        .nodes
        .iter_mut()
        .find(|node| node.node_id == "Owner0/Slot0")
        .unwrap()
        .properties
        .insert(
            "inspector_property_field_id".into(),
            RetainedUiHostValue::String("wrong.field".into()),
        );
    assert_eq!(
        capture(&surface, &projected).observation["inventoryAudit"]["complete"],
        false
    );
}

#[test]
fn normal_inventory_rejects_a_rebound_slot_and_keeps_its_old_observation_frozen() {
    use zircon_runtime::ui::surface::UiVirtualListItemKey;
    let (mut surface, initial_projection) = inspector_repeat_fixture(1);
    let before = capture(&surface, &initial_projection);
    let frozen = before.observation.as_ref().clone();
    let old_binding = surface
        .virtual_list_binding_for_node(UiNodeId::new(1), UiNodeId::new(2))
        .unwrap();
    surface
        .reconcile_virtual_list_materialization_with_keys(
            UiNodeId::new(1),
            3,
            &mut Vec::new(),
            |index| UiVirtualListItemKey::new(111 + index as u128),
        )
        .unwrap();
    assert!(!surface.virtual_list_binding_is_current(UiNodeId::new(2), old_binding));
    let stale = capture(&surface, &initial_projection);
    assert_eq!(stale.observation["inventoryAudit"]["complete"], false);
    assert_eq!(
        row(&stale, "Owner0/Slot0")["inspectorRepeat"]["runtimeBindingCurrent"],
        false
    );
    assert_eq!(before.observation.as_ref(), &frozen);
    assert_eq!(row(&before, "Owner0/Slot0")["itemKey"], "11");
}

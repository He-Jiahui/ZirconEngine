use super::*;

#[test]
fn template_tree_builder_preserves_parent_owned_slot_contracts() {
    let document = UiTemplateLoader::load_toml_str(SLOT_CONTRACT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("slot.contract"), &instance).unwrap();

    assert_eq!(tree.slots.len(), 1);
    let slot = &tree.slots[0];
    let parent = tree.node(slot.parent_id).unwrap();
    let child = tree.node(slot.child_id).unwrap();
    assert_eq!(
        parent
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref()),
        Some("SlotParent")
    );
    assert_eq!(
        child
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref()),
        Some("PrimaryAction")
    );
    assert_eq!(slot.kind, UiSlotKind::Linear);
    assert_eq!(slot.padding.left, 4.0);
    assert_eq!(slot.padding.top, 6.0);
    assert_eq!(slot.padding.right, 8.0);
    assert_eq!(slot.padding.bottom, 10.0);
    assert_eq!(slot.alignment.horizontal, UiAlignment::Fill);
    assert_eq!(slot.alignment.vertical, UiAlignment::Center);
    let linear_sizing = slot.linear_sizing.expect("linear slot sizing");
    assert_eq!(linear_sizing.rule, UiLinearSlotSizeRule::StretchContent);
    assert_eq!(linear_sizing.value, 2.0);
    assert_eq!(linear_sizing.shrink_value, 0.5);
    assert_eq!(linear_sizing.min, 48.0);
    assert_eq!(linear_sizing.max, 160.0);
    assert_eq!(slot.order, 3);
    assert_eq!(slot.z_order, 0);
    assert_eq!(slot.dirty_revision, 0);
    assert_eq!(
        child.constraints.width,
        AxisConstraint {
            min: 96.0,
            max: 96.0,
            preferred: 96.0,
            priority: 0,
            weight: 1.0,
            stretch_mode: StretchMode::Fixed,
        }
    );
}

#[test]
fn template_tree_builder_preserves_overlay_slot_z_order_contracts() {
    let document = UiTemplateLoader::load_toml_str(OVERLAY_SLOT_CONTRACT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree = UiTemplateTreeBuilder::build_tree(UiTreeId::new("overlay.slot.contract"), &instance)
        .unwrap();

    let background_slot = tree
        .slots
        .iter()
        .find(|slot| {
            tree.node(slot.child_id)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("BackgroundLayer")
        })
        .expect("background overlay slot");
    let foreground_slot = tree
        .slots
        .iter()
        .find(|slot| {
            tree.node(slot.child_id)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ForegroundLayer")
        })
        .expect("foreground overlay slot");
    let foreground_node = tree.node(foreground_slot.child_id).unwrap();

    assert_eq!(background_slot.kind, UiSlotKind::Overlay);
    assert_eq!(background_slot.z_order, -4);
    assert_eq!(background_slot.order, 2);
    assert_eq!(background_slot.alignment.horizontal, UiAlignment::Fill);
    assert_eq!(background_slot.alignment.vertical, UiAlignment::Fill);
    assert_eq!(foreground_slot.kind, UiSlotKind::Overlay);
    assert_eq!(foreground_slot.z_order, 16);
    assert_eq!(foreground_slot.order, 1);
    assert_eq!(foreground_slot.padding.left, 4.0);
    assert_eq!(foreground_slot.padding.top, 6.0);
    assert_eq!(foreground_node.z_index, 99);
    assert_eq!(foreground_slot.linear_sizing, None);
}

#[test]
fn template_tree_builder_preserves_canvas_free_slot_placement_contracts() {
    let document =
        UiTemplateLoader::load_toml_str(CANVAS_FREE_SLOT_CONTRACT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("canvas.free.slot.contract"), &instance)
            .unwrap();

    assert_eq!(tree.slots.len(), 1);
    let slot = &tree.slots[0];
    let parent = tree.node(slot.parent_id).unwrap();
    let child = tree.node(slot.child_id).unwrap();
    let placement = slot.canvas_placement.expect("canvas/free slot placement");

    assert_eq!(
        parent
            .template_metadata
            .as_ref()
            .map(|metadata| metadata.component.as_str()),
        Some("Canvas")
    );
    assert_eq!(slot.kind, UiSlotKind::Free);
    assert_eq!(slot.order, 4);
    assert_eq!(placement.anchor.x, 1.0);
    assert_eq!(placement.anchor.y, 0.25);
    assert_eq!(placement.pivot.x, 1.0);
    assert_eq!(placement.pivot.y, 0.5);
    assert_eq!(placement.position.x, -24.0);
    assert_eq!(placement.position.y, 16.0);
    assert_eq!(placement.offset.left, 2.0);
    assert_eq!(placement.offset.top, 4.0);
    assert_eq!(placement.offset.right, 120.0);
    assert_eq!(placement.offset.bottom, 40.0);
    assert!(placement.auto_size);
    assert_eq!(child.anchor.x, 1.0);
    assert_eq!(child.pivot.x, 1.0);
    assert_eq!(child.position.x, -24.0);
}

#[test]
fn template_tree_builder_ignores_canvas_free_placement_on_non_free_slots() {
    let document =
        UiTemplateLoader::load_toml_str(NON_CANVAS_FREE_SLOT_PLACEMENT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree = UiTemplateTreeBuilder::build_tree(UiTreeId::new("linear.slot.contract"), &instance)
        .unwrap();

    assert_eq!(tree.slots.len(), 1);
    let slot = &tree.slots[0];
    let child = tree.node(slot.child_id).unwrap();

    assert_eq!(slot.kind, UiSlotKind::Linear);
    assert_eq!(slot.order, 4);
    assert_eq!(slot.canvas_placement, None);
    assert_eq!(child.anchor.x, 1.0);
    assert_eq!(child.pivot.x, 1.0);
    assert_eq!(child.position.x, -24.0);
}

#[test]
fn template_tree_builder_ignores_canvas_free_placement_on_space_slots() {
    let document = UiTemplateLoader::load_toml_str(SPACE_SLOT_PLACEMENT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("space.slot.contract"), &instance).unwrap();

    assert_eq!(tree.slots.len(), 1);
    let slot = &tree.slots[0];
    let parent = tree.node(slot.parent_id).unwrap();
    let child = tree.node(slot.child_id).unwrap();

    assert_eq!(parent.container, UiContainerKind::Space);
    assert_eq!(slot.kind, UiSlotKind::Free);
    assert_eq!(slot.canvas_placement, None);
    assert_eq!(child.anchor.x, 0.5);
    assert_eq!(child.position.x, 8.0);
}

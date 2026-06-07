use crate::ui::surface::{hit_test_surface_frame, UiSurface};
use crate::ui::template::{UiTemplateInstance, UiTemplateLoader, UiTemplateSurfaceBuilder};
use crate::ui::v2::{UiV2AssetLoader, UiV2DocumentCompiler, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAlignment, UiAlignment2D, UiContainerKind,
        UiFrame, UiLayoutEngineBackend, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
        UiLayoutEngineSupport, UiMargin, UiPoint, UiSize, UiSlot, UiSlotKind,
    },
    tree::{UiInputPolicy, UiTreeNode},
};

#[test]
fn block_box_arranges_visible_children_through_taffy_surface_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.block_box"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(2, "root/summary", fixed_constraints(80.0, 20.0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(3, "root/details", fixed_constraints(60.0, 30.0)),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(120.0, 80.0)).unwrap();
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("block root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Block);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);
    assert_eq!(frame.layout_engine_report.taffy_tree_node_count, 3);

    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("first block child should be arranged");
    let second = frame
        .arranged_tree
        .get(UiNodeId::new(3))
        .expect("second block child should be arranged");
    assert_eq!(first.frame, UiFrame::new(0.0, 0.0, 80.0, 20.0));
    assert_eq!(second.frame, UiFrame::new(0.0, 20.0, 60.0, 30.0));

    let second_render = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3))
        .expect("second block child should render from arranged frame");
    assert_eq!(second_render.frame, second.frame);

    let second_hit = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == UiNodeId::new(3))
        .expect("second block child should enter hit grid");
    assert_eq!(second_hit.frame, second.frame);

    let frame_hit = hit_test_surface_frame(&frame, UiPoint::new(12.0, 32.0));
    assert_eq!(surface.hit_test(UiPoint::new(12.0, 32.0)), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(
        frame_hit.path.bubble_route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
}

#[test]
fn block_box_template_contract_infers_and_parses_explicit_block_container() {
    let document = UiTemplateLoader::load_toml_str(
        r#"
version = 1

[root]
component = "BlockBox"
control_id = "BlockRoot"
children = [
    { component = "Block", control_id = "BlockAlias", attributes = { layout = { container = { kind = "Block" } } } }
]
"#,
    )
    .unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();
    let surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("runtime.ui.template.block_box"),
        &instance,
    )
    .unwrap();

    let root = node_by_control_id(&surface, "BlockRoot");
    let alias = node_by_control_id(&surface, "BlockAlias");
    assert_eq!(root.container, UiContainerKind::BlockBox);
    assert_eq!(alias.container, UiContainerKind::BlockBox);

    let alias_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == alias.node_id)
        .expect("block child should carry a parent-owned slot");
    assert_eq!(alias_slot.kind, UiSlotKind::Container);
}

#[test]
fn block_box_v2_contract_infers_and_parses_explicit_block_container() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/block_box.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "BlockBox"
control_id = "V2BlockRoot"
children = [{ node = "alias", slot = { layout = { padding = { top = 4.0 } } } }]

[nodes.alias]
component = "Panel"
control_id = "V2BlockAlias"
layout = { container = { kind = "Block" } }
"#,
    )
    .unwrap();
    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.block_box"),
        &document,
        &compiled,
    )
    .unwrap();

    let root = node_by_control_id(&surface, "V2BlockRoot");
    let alias = node_by_control_id(&surface, "V2BlockAlias");
    assert_eq!(root.container, UiContainerKind::BlockBox);
    assert_eq!(alias.container, UiContainerKind::BlockBox);

    let alias_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == alias.node_id)
        .expect("v2 block child should carry a parent-owned slot");
    assert_eq!(alias_slot.kind, UiSlotKind::Container);
    assert_eq!(alias_slot.padding, UiMargin::new(0.0, 4.0, 0.0, 0.0));
}

#[test]
fn block_box_native_path_uses_container_slots_for_padding() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.block_box.padding"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(2, "root/first", fixed_constraints(40.0, 10.0)),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Container)
            .with_padding(UiMargin::new(3.0, 5.0, 7.0, 11.0)),
    );

    surface.compute_layout(UiSize::new(100.0, 60.0)).unwrap();
    let frame = surface.surface_frame();
    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("block root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Block);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("block child should be arranged");
    assert_eq!(first.frame, UiFrame::new(3.0, 5.0, 40.0, 10.0));
}

#[test]
fn block_box_fallback_keeps_container_slots_when_taffy_rejects_alignment() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.block_box.fallback"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            pointer_node(2, "root/first", fixed_constraints(40.0, 10.0)),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(UiNodeId::new(1), UiNodeId::new(2), UiSlotKind::Container)
            .with_padding(UiMargin::new(3.0, 5.0, 7.0, 11.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::Start)),
    );

    surface.compute_layout(UiSize::new(100.0, 60.0)).unwrap();
    let frame = surface.surface_frame();
    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("block root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Block);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );

    let first = frame
        .arranged_tree
        .get(UiNodeId::new(2))
        .expect("block child should be arranged by Zircon fallback");
    assert_eq!(first.frame, UiFrame::new(28.0, 5.0, 40.0, 10.0));
}

fn pointer_node(id: u64, path: &str, constraints: BoxConstraints) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(path))
        .with_constraints(constraints)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: false,
        })
}

fn fixed_constraints(width: f32, height: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_constraint(width),
        height: fixed_constraint(height),
    }
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn node_by_control_id<'a>(surface: &'a UiSurface, control_id: &str) -> &'a UiTreeNode {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .expect("node with control id should exist")
}

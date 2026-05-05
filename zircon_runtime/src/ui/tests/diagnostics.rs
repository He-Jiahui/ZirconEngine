use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiRenderCommandKind, UiSurfaceDebugOptions},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn surface_debug_snapshot_reports_reflector_render_batch_and_hit_grid_stats() {
    let surface = diagnostic_surface();
    let snapshot = surface.debug_snapshot_with_options(&UiSurfaceDebugOptions {
        overdraw_sample_cell_size: 40.0,
    });

    assert_eq!(snapshot.tree_id, UiTreeId::new("runtime.ui.diagnostics"));
    assert_eq!(snapshot.roots, vec![UiNodeId::new(1)]);
    assert_eq!(snapshot.nodes.len(), 3);

    let front = snapshot
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(3))
        .expect("front node should be reflected");
    assert_eq!(front.control_id.as_deref(), Some("front.button"));
    assert_eq!(front.render_command_count, 1);
    assert_eq!(front.hit_entry_count, 1);
    assert!(front.hit_cell_count > 0);

    assert_eq!(snapshot.render.command_count, 3);
    assert_eq!(snapshot.render.group_count, 1);
    assert_eq!(snapshot.render.quad_count, 2);
    assert_eq!(snapshot.render.material_batch_count, 2);
    assert!(snapshot.render.estimated_draw_calls >= 2);
    assert!(snapshot
        .render
        .material_batches
        .iter()
        .any(|batch| batch.command_kind == UiRenderCommandKind::Quad
            && batch.break_reason == "kind=Quad;unclipped;opaque;style"
            && batch.command_count == 2
            && batch.node_ids.contains(&UiNodeId::new(2))
            && batch.node_ids.contains(&UiNodeId::new(3))));

    assert_eq!(snapshot.hit_test.entry_count, 2);
    assert!(snapshot.hit_test.occupied_cell_count > 0);
    assert!(snapshot.hit_test.max_entries_per_cell >= 2);

    assert!(snapshot.overdraw.covered_cells > 0);
    assert!(snapshot.overdraw.overdrawn_cells > 0);
    assert!(snapshot.overdraw.max_layers >= 2);
    assert!(snapshot.rebuild.arranged_rebuilt);
    assert!(snapshot.rebuild.hit_grid_rebuilt);
    assert!(snapshot.rebuild.render_rebuilt);
    assert_eq!(snapshot.rebuild.arranged_node_count, 3);
    assert_eq!(snapshot.rebuild.render_command_count, snapshot.render.command_count);
    assert_eq!(snapshot.rebuild.hit_grid_entry_count, snapshot.hit_test.entry_count);
    assert_eq!(snapshot.rebuild.hit_grid_cell_count, snapshot.hit_test.cell_count);
}

#[test]
fn surface_debug_snapshot_uses_surface_frame_as_single_spatial_source() {
    let surface = diagnostic_surface();
    let frame = surface.surface_frame();
    let snapshot = crate::ui::surface::debug_surface_frame(&frame);
    let hit = crate::ui::surface::hit_test_surface_frame(&frame, UiPoint::new(60.0, 20.0));

    let front = snapshot
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(3))
        .expect("front node should be reflected");

    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(front.frame, UiFrame::new(40.0, 0.0, 80.0, 40.0));
    assert_eq!(front.clip_frame, UiFrame::new(40.0, 0.0, 80.0, 40.0));
    assert_eq!(front.hit_entry_count, 1);
    assert_eq!(front.render_command_count, 1);
}

fn diagnostic_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.diagnostics"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                2,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 80.0, 40.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                3,
                "root/front",
                "front.button",
                UiFrame::new(40.0, 0.0, 80.0, 40.0),
                10,
            ),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn button_node(
    node_id: u64,
    node_path: &str,
    control_id: &str,
    frame: UiFrame,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(node_id), UiNodePath::new(node_path))
        .with_frame(frame)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            attributes: toml::from_str(
                r##"
text = "Run"
opacity = 1.0

[background]
color = "#224466"
"##,
            )
            .unwrap(),
            ..Default::default()
        })
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

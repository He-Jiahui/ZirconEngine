//! 验证工作台命中区域与已提交窗口几何的复用关系。

use super::*;

#[test]
fn workbench_hit_test_paths_reuse_pane_surfaces_and_borrow_committed_window_geometry() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_no_active_retained_files(&manifest.join("ui"));

    let retained_host_root = manifest.join("src").join("ui").join("retained_host");
    for path in collect_rust_files(&retained_host_root) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        for forbidden in [
            "HitTable",
            "hit_table",
            "PointerTable",
            "pointer_table",
            "ControlHitTable",
            "control_hit_table",
            "BusinessHitTable",
            "business_hit_table",
            "ManualHitTable",
            "manual_hit_table",
        ] {
            assert_does_not_contain(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("retained_host source"),
                &source,
                forbidden,
            );
        }
    }

    for (relative, required) in [
        (
            "src/ui/retained/menu_pointer/host_menu_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "route_intents: EditorRouteIntentMap",
            ][..],
        ),
        (
            "src/ui/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "register_handled_pointer_node",
                "route_intents.bind_node",
            ][..],
        ),
        (
            "src/ui/activity_rail_pointer/host_activity_rail_pointer_bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "route_intents: EditorRouteIntentMap",
            ][..],
        ),
        (
            "src/ui/retained/activity_rail_pointer/rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "insert_strip(",
            ][..],
        ),
        (
            "src/ui/retained/viewport_toolbar_pointer/bridge.rs",
            &[
                "surface: UiSurface",
                "dispatcher: UiPointerDispatcher",
                "route_intents: EditorRouteIntentMap",
            ][..],
        ),
        (
            "src/ui/retained/viewport_toolbar_pointer/rebuild_surface.rs",
            &[
                "UiSurface::new",
                "UiTreeNode::new",
                "surface.rebuild()",
                "register_handled_pointer_node",
                "route_intents.bind_node",
            ][..],
        ),
        (
            "src/ui/retained/shell_pointer/bridge.rs",
            &[
                "drag_surface: UiSurface",
                "drag_dispatcher: UiPointerDispatcher",
                "drag_route_intents: EditorRouteIntentMap",
                "resize_surface: UiSurface",
                "resize_dispatcher: UiPointerDispatcher",
                "resize_route_intents: EditorRouteIntentMap",
                ".dispatch_input_event(",
            ][..],
        ),
        (
            "src/ui/retained/contract/surface_hit_test/surface_frame.rs",
            &["UiSurfaceFrame", "hit_test_host_surface_frame"][..],
        ),
        (
            "src/ui/retained/contract/surface_hit_test/node.rs",
            &[
                "hit_test_workbench_window_template_node_with_index",
                "HostWorkbenchHitIndex",
            ][..],
        ),
        (
            "src/ui/retained/contract/surface_hit_test/node/hit.rs",
            &[
                "index.candidate_rows(x, y)",
                "index.record_candidate_visit()",
            ][..],
        ),
        (
            "src/ui/retained/contract/node/surface_frame_builder/surface.rs",
            &["surface.surface_frame()"][..],
        ),
    ] {
        let source = source_file(&[relative]);
        for marker in required {
            assert_contains(relative, &source, marker);
        }
    }

    for relative in [
        "src/ui/document_tab_pointer/host_document_tab_pointer_bridge.rs",
        "src/ui/drawer_header_pointer/host_drawer_header_pointer_bridge.rs",
        "src/ui/retained/host_page_pointer/host_page_pointer_bridge.rs",
        "src/ui/retained/hierarchy_pointer/hierarchy_pointer_bridge.rs",
        "src/ui/retained/detail_pointer/scroll_surface_pointer_bridge.rs",
        "src/ui/retained/welcome_recent_pointer/bridge.rs",
    ] {
        let source = source_file(&[relative]);
        for forbidden in ["UiSurface", "UiPointerDispatcher", "EditorRouteIntentMap"] {
            assert_does_not_contain(relative, &source, forbidden);
        }
    }

    let workbench_hit_source = source_file(&[
        "src",
        "ui",
        "retained_host",
        "host_contract",
        "surface_hit_test",
        "template_node.rs",
    ]);
    assert_does_not_contain(
        "workbench template hit testing",
        &workbench_hit_source,
        "template_nodes_surface_frame",
    );
    assert_does_not_contain(
        "workbench template hit testing",
        &workbench_hit_source,
        "hit_test_workbench_window_template_node(",
    );

    let indexed_hit_source = source_file(&[
        "src",
        "ui",
        "retained_host",
        "host_contract",
        "surface_hit_test",
        "template_node",
        "hit.rs",
    ]);
    for forbidden in [
        "fn hit_test_workbench_template_nodes(",
        "nodes.iter().rev()",
    ] {
        assert_does_not_contain(
            "indexed workbench hit testing",
            &indexed_hit_source,
            forbidden,
        );
    }

    let native_routing_exports = source_file(&[
        "src",
        "ui",
        "retained_host",
        "host_contract",
        "native_pointer",
        "routing.rs",
    ]);
    assert_does_not_contain(
        "native pointer routing exports",
        &native_routing_exports,
        "route_pointer_to_workbench_window",
    );
}

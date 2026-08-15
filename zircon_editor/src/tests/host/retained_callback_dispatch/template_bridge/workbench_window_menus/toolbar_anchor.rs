use super::support::*;

#[test]
fn workbench_toolbar_window_menus_anchor_to_toolbar_controls_across_widths() {
    for width in [900.0, 1260.0, 1672.0] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, 620.0))
                .unwrap_or_else(|error| {
                    panic!("workbench {width}px bridge should build: {error:?}")
                });
        assert_toolbar_menu_anchor(
            &mut bridge,
            "WorkbenchToolbarMenu",
            "WorkbenchToolbarMainMenu",
            ToolbarMenuAlign::Start,
        );
    }

    let mut compact = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("compact workbench bridge should build");
    assert_toolbar_menu_anchor(
        &mut compact,
        "WorkbenchModuleMore",
        "WorkbenchModuleOverflowMenu",
        ToolbarMenuAlign::Start,
    );

    for (trigger_id, menu_id) in [
        ("WorkbenchRunMode", "WorkbenchRunModeMenu"),
        ("WorkbenchLayoutGrid", "WorkbenchLayoutMenu"),
    ] {
        let mut wide = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
            .expect("wide workbench bridge should build");
        assert_toolbar_menu_anchor(&mut wide, trigger_id, menu_id, ToolbarMenuAlign::End);
    }
}

#[test]
fn workbench_toolbar_window_menu_anchor_metrics_stay_shared() {
    let source = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs",
    ))
    .expect("window menu state source should be readable");

    assert!(source.contains("popup_anchor_metrics"));
    assert!(
        !source.contains("const TOOLBAR_POPUP_EDGE_MARGIN"),
        "toolbar popup edge margin must stay in popup_anchor_metrics"
    );
    assert!(
        !source.contains("const TOOLBAR_POPUP_RENDER_GAP"),
        "toolbar popup render gap must stay in popup_anchor_metrics"
    );
}

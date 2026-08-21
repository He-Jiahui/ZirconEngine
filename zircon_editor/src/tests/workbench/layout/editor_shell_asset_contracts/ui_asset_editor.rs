use zircon_runtime::ui::v2::UiV2AssetLoader;

use super::{
    node_children_source, node_definition_source, stylesheet_rule_sources, UI_ASSET_EDITOR_ASSET,
    WORKBENCH_UI_ASSET_ACTION_BAR_ASSET,
};

#[test]
fn ui_asset_editor_keeps_content_rows_unframed_inside_tokenized_panels() {
    let panel_rules = stylesheet_rule_sources(UI_ASSET_EDITOR_ASSET, ".ui-asset-panel");
    assert_eq!(
        panel_rules.len(),
        1,
        "the outer editor panel surface must have one canonical stylesheet rule"
    );
    let panel_rule = panel_rules[0];
    assert!(
        panel_rule.contains("background_color = \"$panel\"")
            && panel_rule.contains("border_color = \"$outline\"")
            && panel_rule.contains("radius = \"$editor.control.radius.panel\""),
        "only the editor's outer panel layer may own the framed surface"
    );

    for selector in [".ui-asset-section", ".ui-asset-row"] {
        let rules = stylesheet_rule_sources(UI_ASSET_EDITOR_ASSET, selector);
        assert!(
            !rules.is_empty(),
            "UI Asset Editor must define the {selector} content rule"
        );
        for rule in rules {
            for framed_property in [
                "background_color =",
                "border_color =",
                "surface_variant =",
                "radius =",
                "corner_radius =",
                "border_width =",
            ] {
                assert!(
                    !rule.contains(framed_property),
                    "{selector} must remain an unframed content layer, not a nested card"
                );
            }
        }
    }

    let inset_rules = stylesheet_rule_sources(UI_ASSET_EDITOR_ASSET, ".ui-asset-inset");
    assert_eq!(
        inset_rules.len(),
        1,
        "the recessed canvas/source well must have one canonical stylesheet rule"
    );
    let inset_rule = inset_rules[0];
    assert!(
        inset_rule.contains("background_color = \"$panel_inset\"")
            && inset_rule.contains("border_color = \"$outline\""),
        "interactive canvas and source wells must retain their recessed visual boundary"
    );

    for (panel_node, child_count) in [
        ("header_panel", 3),
        ("left_column", 1),
        ("center_column", 1),
        ("right_column", 1),
    ] {
        let children = node_children_source(UI_ASSET_EDITOR_ASSET, panel_node);
        let direct_children = children
            .lines()
            .filter(|line| line.trim_start().starts_with("{ node ="))
            .collect::<Vec<_>>();
        assert_eq!(
            direct_children.len(),
            child_count,
            "{panel_node} must retain its expected direct child count"
        );
        for child in direct_children {
            assert!(
                child.contains("slot = { layout = { padding = { left = \"$editor.density.panel_padding\", right = \"$editor.density.panel_padding\" } } }"),
                "every direct child of {panel_node} must derive its horizontal inset from the shared token"
            );
        }
        for prohibited_layout_override in ["position", "width =", "height ="] {
            assert!(
                !children.contains(prohibited_layout_override),
                "{panel_node} child slots must remain relative and avoid {prohibited_layout_override} overrides"
            );
        }
    }

    let header_panel = node_definition_source(UI_ASSET_EDITOR_ASSET, "header_panel");
    assert!(
        !header_panel.contains("height ="),
        "header height must derive from its tokenized child rows instead of a fixed aggregate pixel value"
    );

    for scroll_region in [
        "left_scroll_region",
        "center_scroll_region",
        "right_scroll_region",
    ] {
        let definition = node_definition_source(UI_ASSET_EDITOR_ASSET, scroll_region);
        assert!(
            definition.contains("component = \"ScrollableBox\"")
                && definition.contains("kind = \"ScrollableBox\"")
                && definition.contains("axis = \"Vertical\"")
                && definition.contains("scrollbar_visibility = \"Auto\""),
            "{scroll_region} must use the V2 vertical ScrollableBox contract"
        );
        assert!(
            definition.contains("input_hoverable = true")
                && definition.contains("input_policy = \"Receive\""),
            "{scroll_region} must participate in pointer routing so wheel input reaches it"
        );
    }

    let designer_tools = node_children_source(UI_ASSET_EDITOR_ASSET, "designer_tool_mode_row");
    for button in [
        "designer_select_button",
        "designer_resize_slot_button",
        "designer_preview_interact_button",
    ] {
        assert!(
            designer_tools.contains(button),
            "designer mode strip must expose its {button} control as a real button node"
        );
    }
    for (tool_node, action_id) in [
        ("header_save_button", "save"),
        ("header_undo_button", "undo"),
        ("header_redo_button", "redo"),
        ("designer_select_button", "designer.tool.select"),
        ("designer_resize_slot_button", "designer.tool.resize_slot"),
        (
            "designer_preview_interact_button",
            "designer.tool.preview_interact",
        ),
    ] {
        let definition = node_definition_source(UI_ASSET_EDITOR_ASSET, tool_node);
        assert!(
            definition.contains(&format!("action_id = \"{action_id}\""))
                && definition.contains(&format!("event = \"Click\", route = \"{action_id}\"")),
            "{tool_node} must expose the retained-host action `{action_id}` through its V2 click binding"
        );
    }
    for (button_node, icon) in [
        ("header_save_button", "zircon_editor_shell/toolbar/save.svg"),
        ("header_undo_button", "zircon_editor_shell/toolbar/undo.svg"),
        ("header_redo_button", "zircon_editor_shell/toolbar/redo.svg"),
    ] {
        let definition = node_definition_source(UI_ASSET_EDITOR_ASSET, button_node);
        assert!(
            definition.contains("component = \"WorkbenchIconButton\"")
                && definition.contains(&format!("icon = \"{icon}\"")),
            "{button_node} must use the canonical icon-button primitive and its registered icon"
        );
    }
    let header_actions = node_definition_source(UI_ASSET_EDITOR_ASSET, "header_action_row");
    assert!(
        header_actions.contains("min = \"$editor.control.height.default\"")
            && header_actions.contains("preferred = \"$editor.control.height.default\"")
            && header_actions.contains("max = \"$editor.control.height.default\""),
        "the icon-button command row must honor the primitive's default control-height token"
    );
    assert!(
        !designer_tools.contains("designer_move_button")
            && !designer_tools.contains("designer_anchor_button")
            && !designer_tools.contains("designer_fit_button"),
        "designer mode strip must not expose visual-only tools without a retained-host action"
    );

    let action_bar_mount = node_definition_source(UI_ASSET_EDITOR_ASSET, "action_bar_panel");
    assert!(
        action_bar_mount.contains("component = \"WorkbenchUiAssetActionBar\"")
            && !action_bar_mount.contains("height ="),
        "the action bar must mount the intrinsic-height Workbench composite"
    );
    assert!(
        UI_ASSET_EDITOR_ASSET
            .contains("workbench_ui_asset_action_bar.zui#WorkbenchUiAssetActionBar"),
        "the UI Asset Editor must import its standard action-bar composite"
    );
    for (button_node, action_id) in [
        ("insert_child_button", "palette.insert.child"),
        ("insert_after_button", "palette.insert.after"),
        ("reparent_previous_button", "canvas.reparent.into_previous"),
        ("reparent_next_button", "canvas.reparent.into_next"),
        ("reparent_outdent_button", "canvas.reparent.outdent"),
        ("structure_up_button", "canvas.move.up"),
        ("structure_down_button", "canvas.move.down"),
        ("structure_wrap_button", "canvas.wrap.vertical_box"),
    ] {
        let definition = node_definition_source(WORKBENCH_UI_ASSET_ACTION_BAR_ASSET, button_node);
        assert!(
            definition.contains("component = \"WorkbenchButton\"")
                && definition.contains(&format!("action_id = \"{action_id}\""))
                && definition.contains(&format!("event = \"Click\", route = \"{action_id}\"")),
            "action-bar button {button_node} must route to the retained-host action `{action_id}`"
        );
    }

    let palette_grid = node_definition_source(UI_ASSET_EDITOR_ASSET, "palette_component_grid");
    assert!(
        palette_grid.contains("component = \"GridGroup\"")
            && palette_grid.contains("kind = \"GridBox\"")
            && palette_grid.contains("columns = 2"),
        "palette must use a responsive two-column layout rather than fixed child positions"
    );
    for slot in [
        "palette_button\", slot = { layout = { column = 0, row = 0",
        "palette_label\", slot = { layout = { column = 1, row = 0",
        "palette_image\", slot = { layout = { column = 0, row = 1",
        "palette_container\", slot = { layout = { column = 1, row = 1",
    ] {
        assert!(
            palette_grid.contains(slot),
            "palette grid must declare a distinct relative cell slot: {slot}"
        );
    }
    for search in ["palette_search", "hierarchy_search"] {
        assert!(
            node_definition_source(UI_ASSET_EDITOR_ASSET, search)
                .contains("component = \"WorkbenchSearchInput\""),
            "{search} must use the canonical accessible search primitive"
        );
    }
    for tree_row in [
        "hierarchy_root_row",
        "hierarchy_safe_area_row",
        "hierarchy_content_row",
    ] {
        assert!(
            node_definition_source(UI_ASSET_EDITOR_ASSET, tree_row)
                .contains("component = \"WorkbenchTreeRow\""),
            "{tree_row} must use the canonical hierarchy row primitive"
        );
    }
    let designer_canvas = node_definition_source(UI_ASSET_EDITOR_ASSET, "designer_canvas_panel");
    assert!(
        designer_canvas.contains("component = \"CanvasBox\"")
            && designer_canvas.contains("kind = \"Free\"")
            && designer_canvas.contains("clip = true"),
        "designer canvas must remain a clipped host surface for the real preview projection"
    );
    let document = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_ASSET)
        .expect("UI Asset Editor asset must remain a valid V2 document");
    assert!(
        !document
            .nodes
            .values()
            .any(|node| node.component == "WorkbenchSampleGrid"),
        "designer canvas must not regress to a static Blend Space grid under any node name"
    );
    assert!(
        !document
            .imports
            .widgets
            .iter()
            .any(|widget| widget.contains("workbench_sample_grid.zui")),
        "designer canvas must not import the static Blend Space grid primitive"
    );
}

use crate::ui::asset_editor::{
    apply_ui_asset_editor_designer_tool_mode, ui_asset_editor_node_projection,
    ui_asset_editor_surface_for_test,
};
use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::UiSize,
    surface::{UiPointerButton, UiPointerEventKind},
};

#[test]
fn ui_asset_editor_bootstrap_template_projection_exposes_pane_shell_regions() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(1280.0, 720.0)).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };

    for control_id in [
        "HeaderPanel",
        "HeaderAssetRow",
        "HeaderStatusRow",
        "HeaderActionRow",
        "HeaderSaveButton",
        "HeaderUndoButton",
        "HeaderRedoButton",
        "LeftColumn",
        "CenterColumn",
        "RightColumn",
        "PalettePanel",
        "PaletteSearchInput",
        "PaletteComponentGrid",
        "PaletteButton",
        "PaletteLabel",
        "PaletteImage",
        "PaletteContainer",
        "HierarchyPanel",
        "HierarchySearchInput",
        "HierarchyRootRow",
        "HierarchySafeAreaRow",
        "HierarchyContentRow",
        "DesignerPanel",
        "DesignerToolModeRow",
        "DesignerSelectButton",
        "DesignerResizeSlotButton",
        "DesignerPreviewInteractButton",
        "DesignerCanvasPanel",
        "DesignerDiagnosticOverlayPanel",
        "EmergencyShellPanel",
        "RenderStackPanel",
        "ActionBarPanel",
        "ActionInsertRow",
        "ActionReparentRow",
        "ActionStructureRow",
        "SourcePanel",
        "SourceInfoPanel",
        "SourceOutlinePanel",
        "MockWorkspacePanel",
        "MockSubjectsPanel",
        "MockEditorPanel",
        "MockStateGraphPanel",
        "SourceTextPanel",
        "InspectorPanel",
        "InspectorContentPanel",
        "InspectorWidgetSection",
        "InspectorPromoteSection",
        "InspectorSlotSection",
        "InspectorLayoutSection",
        "InspectorBindingSection",
        "StylesheetPanel",
        "StylesheetActionRow",
        "StylesheetStatePrimaryRow",
        "StylesheetStateSecondaryRow",
        "StylesheetContentPanel",
        "StylesheetThemeSection",
        "StylesheetAuthoringSection",
        "StylesheetMatchedRuleSection",
    ] {
        let node = node(control_id);
        assert!(
            node.frame.width > 0.0 && node.frame.height > 0.0,
            "expected `{control_id}` node to be laid out by the bootstrap asset, got {:?}",
            node.frame
        );
    }

    assert!(node("HeaderPanel").frame.y <= node("PalettePanel").frame.y);
    assert!(node("HeaderAssetRow").frame.y >= node("HeaderPanel").frame.y);
    assert!(node("HeaderStatusRow").frame.y >= node("HeaderAssetRow").frame.y);
    assert!(node("HeaderActionRow").frame.y >= node("HeaderStatusRow").frame.y);
    assert!(node("LeftColumn").frame.x < node("CenterColumn").frame.x);
    assert!(node("CenterColumn").frame.x < node("RightColumn").frame.x);
    let left_column = node("LeftColumn").frame.clone();
    let center_column = node("CenterColumn").frame.clone();
    let right_column = node("RightColumn").frame.clone();
    assert!(
        center_column.width >= left_column.width * 1.4
            && center_column.width >= right_column.width * 1.4,
        "the regular workbench width must preserve a visibly primary center pane, got \
         {left_column:?}, {center_column:?}, and {right_column:?}"
    );
    assert!(node("PalettePanel").frame.x < node("DesignerPanel").frame.x);
    assert!(node("DesignerPanel").frame.x < node("InspectorPanel").frame.x);
    assert!(node("DesignerToolModeRow").frame.y >= node("DesignerPanel").frame.y);
    assert!(node("DesignerCanvasPanel").frame.y >= node("DesignerToolModeRow").frame.y);
    assert!(
        node("DesignerCanvasPanel").frame.height >= 160.0,
        "the regular workbench must reserve a usable designer preview canvas instead of a blend-space-sized strip, got {:?}",
        node("DesignerCanvasPanel").frame
    );
    assert!(node("DesignerDiagnosticOverlayPanel").frame.y >= node("DesignerCanvasPanel").frame.y);
    assert!(node("EmergencyShellPanel").frame.y >= node("DesignerDiagnosticOverlayPanel").frame.y);
    assert!(node("RenderStackPanel").frame.y >= node("EmergencyShellPanel").frame.y);
    assert!(node("ActionBarPanel").frame.y >= node("DesignerPanel").frame.y);
    assert!(node("ActionInsertRow").frame.y >= node("ActionBarPanel").frame.y);
    assert!(node("ActionReparentRow").frame.y >= node("ActionInsertRow").frame.y);
    assert!(node("ActionStructureRow").frame.y >= node("ActionReparentRow").frame.y);
    assert!(node("SourcePanel").frame.y >= node("ActionBarPanel").frame.y);
    assert!(node("SourceInfoPanel").frame.y >= node("SourcePanel").frame.y);
    assert!(node("SourceOutlinePanel").frame.y >= node("SourceInfoPanel").frame.y);
    assert!(node("MockWorkspacePanel").frame.y >= node("SourceOutlinePanel").frame.y);
    assert!(node("MockSubjectsPanel").frame.y >= node("MockWorkspacePanel").frame.y);
    assert!(node("MockEditorPanel").frame.y >= node("MockSubjectsPanel").frame.y);
    assert!(node("MockStateGraphPanel").frame.y >= node("MockEditorPanel").frame.y);
    assert!(node("SourceTextPanel").frame.y >= node("MockStateGraphPanel").frame.y);
    assert!(node("InspectorContentPanel").frame.y >= node("InspectorPanel").frame.y);
    assert!(node("InspectorWidgetSection").frame.y >= node("InspectorContentPanel").frame.y);
    assert!(node("InspectorPromoteSection").frame.y >= node("InspectorWidgetSection").frame.y);
    assert!(node("InspectorSlotSection").frame.y >= node("InspectorPromoteSection").frame.y);
    assert!(node("InspectorLayoutSection").frame.y >= node("InspectorSlotSection").frame.y);
    assert!(node("InspectorBindingSection").frame.y >= node("InspectorLayoutSection").frame.y);
    assert!(node("StylesheetPanel").frame.y >= node("InspectorPanel").frame.y);
    assert!(node("StylesheetActionRow").frame.y >= node("StylesheetPanel").frame.y);
    assert!(node("StylesheetStatePrimaryRow").frame.y >= node("StylesheetActionRow").frame.y);
    assert!(
        node("StylesheetStateSecondaryRow").frame.y >= node("StylesheetStatePrimaryRow").frame.y
    );
    assert!(node("StylesheetContentPanel").frame.y >= node("StylesheetStateSecondaryRow").frame.y);
    assert!(node("StylesheetThemeSection").frame.y >= node("StylesheetContentPanel").frame.y);
    assert!(node("StylesheetAuthoringSection").frame.y >= node("StylesheetThemeSection").frame.y);
    assert!(
        node("StylesheetMatchedRuleSection").frame.y >= node("StylesheetAuthoringSection").frame.y
    );

    let palette_button = node("PaletteButton").frame.clone();
    let palette_label = node("PaletteLabel").frame.clone();
    let palette_image = node("PaletteImage").frame.clone();
    let palette_container = node("PaletteContainer").frame.clone();
    assert!(
        palette_button.x + palette_button.width <= palette_label.x
            && palette_button.y + palette_button.height <= palette_image.y
            && palette_image.x + palette_image.width <= palette_container.x
            && palette_label.y + palette_label.height <= palette_container.y,
        "palette components must occupy distinct two-column grid cells, got \
         {palette_button:?}, {palette_label:?}, {palette_image:?}, and {palette_container:?}"
    );
}

#[test]
fn ui_asset_editor_projection_disables_unbound_palette_samples() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(640.0, 720.0)).nodes;

    for control_id in [
        "PaletteButton",
        "PaletteLabel",
        "PaletteImage",
        "PaletteContainer",
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected palette sample `{control_id}`"));
        assert!(
            node.disabled,
            "{control_id} must not become interactive before the dynamic palette projection binds its insert route"
        );
        assert!(
            node.action_id.is_empty(),
            "{control_id} must not project an unhandled action id"
        );
    }
}

#[test]
fn ui_asset_editor_projection_keeps_three_panes_usable_at_minimum_workbench_width() {
    let size = UiSize::new(640.0, 720.0);
    let nodes = ui_asset_editor_node_projection(size).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };

    for (control_id, minimum_width) in [
        ("LeftColumn", 128.0),
        ("CenterColumn", 256.0),
        ("RightColumn", 128.0),
    ] {
        let frame = node(control_id).frame.clone();
        assert!(
            frame.x >= 0.0 && frame.width >= minimum_width && frame.x + frame.width <= size.width,
            "{control_id} must remain visible and usable at the minimum workbench width, got {frame:?}"
        );
    }

    let left = node("LeftColumn").frame.clone();
    let center = node("CenterColumn").frame.clone();
    let right = node("RightColumn").frame.clone();
    assert!(
        left.x + left.width <= center.x && center.x + center.width <= right.x,
        "responsive workbench panes must not overlap, got {left:?}, {center:?}, and {right:?}"
    );

    let canvas = node("DesignerCanvasPanel").frame.clone();
    assert!(
        canvas.x >= center.x
            && canvas.x + canvas.width <= center.x + center.width
            && canvas.width > 0.0,
        "the designer canvas must remain within the responsive center pane, got {canvas:?} in {center:?}"
    );

    let palette_grid = node("PaletteComponentGrid").frame.clone();
    let palette_button = node("PaletteButton").frame.clone();
    let palette_label = node("PaletteLabel").frame.clone();
    let palette_image = node("PaletteImage").frame.clone();
    let palette_container = node("PaletteContainer").frame.clone();
    for (control_id, frame) in [
        ("PaletteButton", &palette_button),
        ("PaletteLabel", &palette_label),
        ("PaletteImage", &palette_image),
        ("PaletteContainer", &palette_container),
    ] {
        assert!(
            frame.width > 0.0
                && frame.height > 0.0
                && frame.x >= left.x
                && frame.x + frame.width <= left.x + left.width
                && frame.x >= palette_grid.x
                && frame.x + frame.width <= palette_grid.x + palette_grid.width
                && frame.y >= palette_grid.y
                && frame.y + frame.height <= palette_grid.y + palette_grid.height,
            "{control_id} must remain inside the palette grid and responsive left pane, got {frame:?}"
        );
    }
    assert!(
        palette_button.x + palette_button.width <= palette_label.x
            && palette_button.y + palette_button.height <= palette_image.y
            && palette_image.x + palette_image.width <= palette_container.x
            && palette_label.y + palette_label.height <= palette_container.y,
        "palette cells must remain distinct at the minimum workbench width, got \
         {palette_button:?}, {palette_label:?}, {palette_image:?}, and {palette_container:?}"
    );
}

#[test]
fn ui_asset_editor_projection_keeps_scroll_viewports_usable_at_minimum_workbench_height() {
    let size = UiSize::new(640.0, 420.0);
    let nodes = ui_asset_editor_node_projection(size).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };

    for (region_id, column_id) in [
        ("LeftScrollRegion", "LeftColumn"),
        ("CenterScrollRegion", "CenterColumn"),
        ("RightScrollRegion", "RightColumn"),
    ] {
        let region = node(region_id).frame.clone();
        let column = node(column_id).frame.clone();
        assert!(
            region.width > 0.0
                && region.height > 0.0
                && region.x >= column.x
                && region.x + region.width <= column.x + column.width
                && region.y >= column.y
                && region.y + region.height <= column.y + column.height,
            "{region_id} must remain an in-bounds, non-empty viewport at the minimum workbench height, got {region:?} in {column:?}"
        );
    }
}

#[test]
fn ui_asset_editor_projection_resolves_compact_action_widths_from_runtime_tokens() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(640.0, 720.0)).nodes;

    for control_id in [
        "ActionInsertChildButton",
        "ActionInsertAfterButton",
        "ActionReparentPreviousButton",
        "ActionReparentNextButton",
        "ActionReparentOutdentButton",
        "ActionStructureUpButton",
        "ActionStructureDownButton",
        "ActionStructureWrapButton",
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected action `{control_id}`"));
        assert_eq!(
            node.frame.width, 52.0,
            "{control_id} must resolve the shared compact action preferred width from the runtime token registry"
        );
    }
}

#[test]
fn ui_asset_editor_designer_canvas_clips_real_preview_projection() {
    let surface = ui_asset_editor_surface_for_test(UiSize::new(640.0, 420.0));
    let canvas_id = surface
        .tree
        .nodes
        .iter()
        .find_map(|(node_id, node)| {
            (node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("DesignerCanvasPanel"))
            .then_some(*node_id)
        })
        .expect("designer canvas should remain in the V2 surface");
    let canvas = surface
        .tree
        .node(canvas_id)
        .expect("designer canvas should remain addressable after layout");

    assert!(
        canvas.layout_cache.frame.width > 0.0 && canvas.layout_cache.frame.height > 0.0,
        "the real preview host must retain a non-empty layout frame"
    );
    assert_eq!(
        canvas.layout_cache.clip_frame,
        Some(canvas.layout_cache.frame),
        "the real preview host must clip dynamic preview content to its canvas frame"
    );
}

#[test]
fn ui_asset_editor_projection_renders_composed_content_labels() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(640.0, 720.0)).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };

    for (parent_id, label_id, expected_text) in [
        (
            "DesignerDiagnosticOverlayPanel",
            "DesignerDiagnosticCaption",
            "Diagnostics",
        ),
        (
            "EmergencyShellPanel",
            "EmergencyShellCaption",
            "Emergency Shell",
        ),
        ("RenderStackPanel", "RenderStackLabel", "Render Stack"),
        ("ActionInsertRow", "ActionInsertCaption", "Insert"),
        ("ActionReparentRow", "ActionReparentCaption", "Reparent"),
        ("ActionStructureRow", "ActionStructureCaption", "Structure"),
        ("SourceInfoPanel", "SourceInfoCaption", "Source info"),
        (
            "SourceOutlinePanel",
            "SourceOutlineCaption",
            "Source outline",
        ),
        ("MockWorkspacePanel", "MockWorkspaceCaption", "Workspace"),
        ("MockSubjectsPanel", "MockSubjectsCaption", "Subjects"),
        ("MockEditorPanel", "MockEditorCaption", "Editor mock"),
        (
            "MockStateGraphPanel",
            "MockStateGraphCaption",
            "State graph",
        ),
        ("SourceTextPanel", "SourceTextCaption", "TOML source"),
        (
            "InspectorContentPanel",
            "InspectorContentLabel",
            "Inspector content",
        ),
        ("InspectorWidgetSection", "InspectorWidgetCaption", "Widget"),
        (
            "InspectorPromoteSection",
            "InspectorPromoteCaption",
            "Promote",
        ),
        ("InspectorSlotSection", "InspectorSlotCaption", "Slot"),
        ("InspectorLayoutSection", "InspectorLayoutCaption", "Layout"),
        (
            "InspectorBindingSection",
            "InspectorBindingCaption",
            "Bindings",
        ),
        (
            "StylesheetActionRow",
            "StylesheetActionCaption",
            "Stylesheet actions",
        ),
        (
            "StylesheetStatePrimaryRow",
            "StylesheetStatePrimaryCaption",
            "Primary state",
        ),
        (
            "StylesheetStateSecondaryRow",
            "StylesheetStateSecondaryCaption",
            "Secondary state",
        ),
        (
            "StylesheetContentPanel",
            "StylesheetContentLabel",
            "Style content",
        ),
        ("StylesheetThemeSection", "StylesheetThemeCaption", "Theme"),
        (
            "StylesheetAuthoringSection",
            "StylesheetAuthoringCaption",
            "Authoring",
        ),
        (
            "StylesheetMatchedRuleSection",
            "StylesheetMatchedRuleCaption",
            "Matched rule",
        ),
    ] {
        let parent = node(parent_id).frame.clone();
        let label = node(label_id);
        let label_frame = label.frame.clone();
        assert_eq!(label.text.as_str(), expected_text);
        assert!(
            label_frame.width > 0.0
                && label_frame.height > 0.0
                && label_frame.x >= parent.x
                && label_frame.y >= parent.y
                && label_frame.x + label_frame.width <= parent.x + parent.width
                && label_frame.y + label_frame.height <= parent.y + parent.height,
            "{label_id} must render within its parent {parent_id}, got {label_frame:?} in {parent:?}"
        );
    }
}

#[test]
fn ui_asset_editor_header_actions_fit_the_minimum_workbench_width() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(640.0, 420.0)).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };
    let header = node("HeaderPanel").frame.clone();
    let action_row = node("HeaderActionRow").frame.clone();
    assert!(
        action_row.x >= header.x
            && action_row.x + action_row.width <= header.x + header.width
            && action_row.y >= header.y
            && action_row.y + action_row.height <= header.y + header.height,
        "the header action row must remain inside the compact header, got {action_row:?} in {header:?}"
    );

    let mut previous_right = action_row.x;
    for control_id in ["HeaderSaveButton", "HeaderUndoButton", "HeaderRedoButton"] {
        let frame = node(control_id).frame.clone();
        assert!(
            frame.width > 0.0
                && frame.height > 0.0
                && frame.x >= previous_right
                && frame.x + frame.width <= action_row.x + action_row.width
                && frame.y >= action_row.y
                && frame.y + frame.height <= action_row.y + action_row.height,
            "{control_id} must remain an in-bounds, non-overlapping header command, got {frame:?} in {action_row:?}"
        );
        previous_right = frame.x + frame.width;
    }
}

#[test]
fn ui_asset_editor_action_bar_controls_fit_the_minimum_workbench_width() {
    let nodes = ui_asset_editor_node_projection(UiSize::new(640.0, 420.0)).nodes;
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected node `{control_id}`"))
    };

    for (row_control_id, control_ids) in [
        (
            "ActionInsertRow",
            [
                "ActionInsertCaption",
                "ActionInsertChildButton",
                "ActionInsertAfterButton",
                "",
            ],
        ),
        (
            "ActionReparentRow",
            [
                "ActionReparentCaption",
                "ActionReparentPreviousButton",
                "ActionReparentNextButton",
                "ActionReparentOutdentButton",
            ],
        ),
        (
            "ActionStructureRow",
            [
                "ActionStructureCaption",
                "ActionStructureUpButton",
                "ActionStructureDownButton",
                "ActionStructureWrapButton",
            ],
        ),
    ] {
        let row = node(row_control_id).frame.clone();
        assert!(
            row.width > 0.0 && row.height > 0.0,
            "{row_control_id} must retain a positive frame at the minimum workbench width"
        );

        let mut previous_right = row.x;
        for control_id in control_ids
            .into_iter()
            .filter(|control_id| !control_id.is_empty())
        {
            let frame = node(control_id).frame.clone();
            assert!(
                frame.width > 0.0
                    && frame.height > 0.0
                    && frame.x >= previous_right
                    && frame.x + frame.width <= row.x + row.width
                    && frame.y >= row.y
                    && frame.y + frame.height <= row.y + row.height,
                "{control_id} must remain an in-bounds, non-overlapping {row_control_id} control, got {frame:?} in {row:?}"
            );
            previous_right = frame.x + frame.width;
        }
    }
}

#[test]
fn ui_asset_editor_scroll_regions_route_wheel_input_and_clip_overflow() {
    let size = UiSize::new(640.0, 420.0);
    let mut surface = ui_asset_editor_surface_for_test(size);
    let dispatcher = UiPointerDispatcher::default();

    for (region_control_id, overflow_control_id) in [
        ("LeftScrollRegion", "PalettePanel"),
        ("CenterScrollRegion", "DesignerPanel"),
        ("RightScrollRegion", "InspectorPanel"),
    ] {
        let region_id = surface
            .tree
            .nodes
            .iter()
            .find_map(|(node_id, node)| {
                (node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(region_control_id))
                .then_some(*node_id)
            })
            .unwrap_or_else(|| panic!("missing scroll region `{region_control_id}`"));
        let overflow_id = surface
            .tree
            .nodes
            .iter()
            .find_map(|(node_id, node)| {
                (node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(overflow_control_id))
                .then_some(*node_id)
            })
            .unwrap_or_else(|| panic!("missing scroll content `{overflow_control_id}`"));
        let region = surface
            .tree
            .node(region_id)
            .expect("scroll region should remain in the test surface");
        let viewport = region.layout_cache.frame;
        let overflow_frame_before = surface
            .tree
            .node(overflow_id)
            .expect("overflow content should remain in the test surface")
            .layout_cache
            .frame;
        let initial_offset = region
            .scroll_state
            .expect("scroll region should own scroll state after V2 projection");
        assert!(
            initial_offset.content_extent > initial_offset.viewport_extent,
            "{region_control_id} must overflow at the minimum workbench height, got {initial_offset:?}"
        );

        let dispatch = surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Scroll, viewport.center())
                    .with_scroll_delta(48.0),
            )
            .expect("wheel dispatch should complete for the editor surface");
        assert_eq!(dispatch.handled_by, Some(region_id));
        assert!(dispatch.diagnostics.scroll_defaulted);

        surface
            .compute_layout(size)
            .expect("scrolling the editor surface should recompute its layout");
        let region = surface
            .tree
            .node(region_id)
            .expect("scroll region should survive its layout rebuild");
        let updated_offset = region
            .scroll_state
            .expect("scroll region should retain its scroll state");
        assert!(
            updated_offset.offset > initial_offset.offset,
            "{region_control_id} must advance after wheel input, got {initial_offset:?} then {updated_offset:?}"
        );
        let overflow = surface
            .tree
            .node(overflow_id)
            .expect("overflow content should survive its layout recomputation");
        assert!(
            overflow.layout_cache.frame.y < overflow_frame_before.y,
            "{overflow_control_id} must translate after {region_control_id} scrolls, got \
             {overflow_frame_before:?} then {:?}",
            overflow.layout_cache.frame
        );
        assert_eq!(
            overflow.layout_cache.clip_frame,
            Some(region.layout_cache.frame),
            "{overflow_control_id} must remain clipped to {region_control_id} after scrolling"
        );
    }
}

#[test]
fn ui_asset_editor_action_controls_emit_existing_host_action_routes() {
    let size = UiSize::new(1280.0, 720.0);
    let mut surface = ui_asset_editor_surface_for_test(size);
    let dispatcher = UiPointerDispatcher::default();

    for (control_id, expected_route) in [
        ("HeaderSaveButton", "save"),
        ("HeaderUndoButton", "undo"),
        ("HeaderRedoButton", "redo"),
        ("DesignerSelectButton", "designer.tool.select"),
        ("DesignerResizeSlotButton", "designer.tool.resize_slot"),
        (
            "DesignerPreviewInteractButton",
            "designer.tool.preview_interact",
        ),
        ("ActionInsertChildButton", "palette.insert.child"),
        ("ActionInsertAfterButton", "palette.insert.after"),
        (
            "ActionReparentPreviousButton",
            "canvas.reparent.into_previous",
        ),
        ("ActionReparentNextButton", "canvas.reparent.into_next"),
        ("ActionReparentOutdentButton", "canvas.reparent.outdent"),
        ("ActionStructureUpButton", "canvas.move.up"),
        ("ActionStructureDownButton", "canvas.move.down"),
        ("ActionStructureWrapButton", "canvas.wrap.vertical_box"),
    ] {
        let node_id = surface
            .tree
            .nodes
            .iter()
            .find_map(|(node_id, node)| {
                (node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(control_id))
                .then_some(*node_id)
            })
            .unwrap_or_else(|| panic!("missing designer tool `{control_id}`"));
        let point = surface
            .tree
            .node(node_id)
            .expect("designer tool should remain in the V2 surface")
            .layout_cache
            .frame
            .center();

        surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Down, point)
                    .with_button(UiPointerButton::Primary),
            )
            .expect("designer tool press should dispatch");
        surface
            .rebuild_dirty(size)
            .expect("designer tool press should refresh visual state");
        let release = surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Up, point)
                    .with_button(UiPointerButton::Primary),
            )
            .expect("designer tool release should dispatch");
        let action = release
            .component_events
            .iter()
            .find_map(|event| event.template_action.as_ref())
            .unwrap_or_else(|| panic!("{control_id} must emit a template action"));
        assert!(!action.is_action());
        assert_eq!(
            action.target_id(),
            expected_route,
            "{control_id} must route to its existing retained-host action"
        );
    }
}

#[test]
fn ui_asset_editor_designer_tool_projection_tracks_presented_mode() {
    let mut nodes = ui_asset_editor_node_projection(UiSize::new(1280.0, 720.0)).nodes;

    apply_ui_asset_editor_designer_tool_mode(&mut nodes, "Preview Interact");

    for (control_id, expected_selected) in [
        ("DesignerSelectButton", false),
        ("DesignerResizeSlotButton", false),
        ("DesignerPreviewInteractButton", true),
    ] {
        let selected = nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing designer tool `{control_id}`"))
            .selected;
        assert_eq!(
            selected, expected_selected,
            "{control_id} must reflect the session's active designer mode"
        );
    }
}

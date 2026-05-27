use crate::ui::surface::UiSurface;
use crate::ui::template::{UiTemplateInstance, UiTemplateLoader, UiTemplateSurfaceBuilder};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{
        UiAlignment, UiAlignment2D, UiContainerKind, UiGridBoxConfig, UiGridSlotPlacement,
        UiLinearBoxConfig, UiMasonryBoxConfig, UiSlotKind, UiWrapBoxConfig,
    },
    tree::UiVisibility,
};

const GRID_FLOW_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "GridBox"
control_id = "GridRoot"
attributes = { layout = { container = { kind = "GridBox", columns = 2, rows = 2, gap = 4.0 } } }
children = [
    { component = "IconButton", control_id = "GridChild", slot_attributes = { layout = { column = 1, row = 1, column_span = 1, row_span = 1, padding = { left = 4.0, top = 2.0, right = 6.0, bottom = 8.0 }, alignment = { horizontal = "End", vertical = "Center" } } }, attributes = { layout = { width = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" }, height = { min = 12.0, preferred = 12.0, max = 12.0, stretch = "Fixed" } } } },
    { component = "FlowBox", control_id = "FlowChild", attributes = { layout = { container = { kind = "FlowBox", gap = 3.0, item_min_width = 24.0 } } }, children = [
        { component = "IconButton", control_id = "FlowItem", slot_attributes = { layout = { order = 2, padding = { left = 1.0, top = 2.0, right = 3.0, bottom = 4.0 } } } }
    ] },
    { component = "Grid", control_id = "MuiGridContainer", attributes = { container = true, columns = { xs = 12, md = 16 }, spacing = { xs = 2, md = 4 }, rowSpacing = { xs = 1, md = 3 }, columnSpacing = { xs = 2, md = 5 } }, children = [
        { component = "Grid", control_id = "MuiGridItem", attributes = { size = { xs = 6, md = 4 }, offset = { xs = 2, md = 3 } } }
    ] },
    { component = "Stack", control_id = "MuiStackRow", attributes = { direction = { xs = "row", md = "column" }, spacing = { xs = 6, md = 10 } } },
    { component = "Stack", control_id = "MuiStackColumn", attributes = { spacing = 4 } },
    { component = "Label", control_id = "VisibilityProbe", attributes = { visibility = { xs = "hidden", md = "visible" } } },
    { component = "Label", control_id = "LegacyVisibleProbe", attributes = { visible = { xs = false, md = true } } },
    { component = "Label", control_id = "DisplayProbe", attributes = { display = { xs = "none", md = "block" } } },
    { component = "UseMediaQuery", control_id = "MediaMinProbe", attributes = { query = "(min-width: 900px)", defaultMatches = false } },
    { component = "UseMediaQuery", control_id = "MediaMaxProbe", attributes = { query = "(max-width: 899px)", defaultMatches = false } },
    { component = "UseMediaQuery", control_id = "MediaFallbackProbe", attributes = { query = "unsupported-query", defaultMatches = true } },
    { component = "UseMediaQuery", control_id = "MediaRangeProbe", attributes = { query = "(min-width: 600px) and (max-width: 959px)", defaultMatches = false } },
    { component = "UseMediaQuery", control_id = "MediaUpProbe", attributes = { up = "md", defaultMatches = false } },
    { component = "UseMediaQuery", control_id = "MediaDownProbe", attributes = { down = "md", defaultMatches = false } },
    { component = "UseMediaQuery", control_id = "MediaBetweenProbe", attributes = { between = ["sm", "md"], defaultMatches = false } },
    { component = "Masonry", control_id = "MasonryChild", attributes = { layout = { container = { kind = "Masonry", columns = 3, gap = 6.0, sequential = true } } }, children = [
        { component = "IconButton", control_id = "MasonryItem", slot_attributes = { layout = { order = 3, padding = { left = 2.0, top = 3.0, right = 4.0, bottom = 5.0 } } } }
    ] }
]
"#;

#[test]
fn template_builder_maps_grid_and_flow_slots_into_shared_runtime_layout_contract() {
    let document = UiTemplateLoader::load_toml_str(GRID_FLOW_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();
    let surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("runtime.ui.template.grid"),
        &instance,
    )
    .unwrap();

    let root = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("GridRoot")
        })
        .expect("grid root should be built");
    let grid_child = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("GridChild")
        })
        .expect("grid child should be built");
    let flow_child = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("FlowChild")
        })
        .expect("flow child should be built");
    let flow_item = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("FlowItem")
        })
        .expect("flow item should be built");
    let mui_grid_container = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MuiGridContainer")
        })
        .expect("mui grid container should be built");
    let mui_stack_row = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MuiStackRow")
        })
        .expect("mui stack row should be built");
    let mui_grid_item = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MuiGridItem")
        })
        .expect("mui grid item should be built");
    let mui_stack_column = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MuiStackColumn")
        })
        .expect("mui stack column should be built");
    let masonry_child = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MasonryChild")
        })
        .expect("masonry child should be built");
    let masonry_item = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("MasonryItem")
        })
        .expect("masonry item should be built");

    assert_eq!(
        root.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 2,
            column_gap: 4.0,
            row_gap: 4.0,
        })
    );
    assert_eq!(
        flow_child.container,
        UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: 3.0,
            vertical_gap: 3.0,
            item_min_width: 24.0,
        })
    );
    assert_eq!(
        mui_grid_container.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 12,
            rows: 1,
            column_gap: 16.0,
            row_gap: 8.0,
        })
    );
    assert_eq!(
        mui_stack_row.container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 48.0 })
    );
    assert_eq!(
        mui_stack_column.container,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 32.0 })
    );
    assert_eq!(
        masonry_child.container,
        UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: 3,
            gap: 6.0,
            sequential: true,
        })
    );

    let grid_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == grid_child.node_id)
        .expect("grid child should carry a parent slot");
    let grid_flow_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == flow_child.node_id)
        .expect("flow child should carry a parent slot");
    let flow_item_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == flow_item.node_id)
        .expect("flow item should carry a parent slot");
    let mui_grid_item_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == mui_grid_item.node_id)
        .expect("mui grid item should carry a parent slot");
    let grid_masonry_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == masonry_child.node_id)
        .expect("masonry child should carry a parent slot");
    let masonry_item_slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == masonry_item.node_id)
        .expect("masonry item should carry a parent slot");

    assert_eq!(grid_slot.kind, UiSlotKind::Grid);
    assert_eq!(
        grid_slot.grid_placement,
        Some(UiGridSlotPlacement::new(1, 1).with_span(1, 1))
    );
    assert_eq!(grid_slot.padding.left, 4.0);
    assert_eq!(
        grid_slot.alignment,
        UiAlignment2D::new(UiAlignment::End, UiAlignment::Center)
    );
    assert_eq!(grid_flow_slot.kind, UiSlotKind::Grid);
    assert_eq!(mui_grid_item_slot.kind, UiSlotKind::Grid);
    assert_eq!(
        mui_grid_item_slot.grid_placement,
        Some(UiGridSlotPlacement::new(2, 0).with_span(6, 1))
    );
    assert_eq!(flow_item_slot.kind, UiSlotKind::Flow);
    assert_eq!(flow_item_slot.order, 2);
    assert_eq!(flow_item_slot.padding.bottom, 4.0);
    assert_eq!(grid_masonry_slot.kind, UiSlotKind::Grid);
    assert_eq!(masonry_item_slot.kind, UiSlotKind::Flow);
    assert_eq!(masonry_item_slot.order, 3);
    assert_eq!(masonry_item_slot.padding.left, 2.0);
}

#[test]
fn template_mui_responsive_layout_recomputes_from_viewport_breakpoints() {
    let document = UiTemplateLoader::load_toml_str(GRID_FLOW_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();
    let mut surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("runtime.ui.template.grid.responsive"),
        &instance,
    )
    .unwrap();

    surface
        .compute_layout(zircon_runtime_interface::ui::layout::UiSize::new(
            720.0, 360.0,
        ))
        .unwrap();
    let grid = node_by_control_id(&surface, "MuiGridContainer");
    let stack = node_by_control_id(&surface, "MuiStackRow");
    let item = node_by_control_id(&surface, "MuiGridItem");
    let visibility_probe = node_by_control_id(&surface, "VisibilityProbe");
    let legacy_visible_probe = node_by_control_id(&surface, "LegacyVisibleProbe");
    let display_probe = node_by_control_id(&surface, "DisplayProbe");
    let media_min = node_by_control_id(&surface, "MediaMinProbe");
    let media_max = node_by_control_id(&surface, "MediaMaxProbe");
    let media_fallback = node_by_control_id(&surface, "MediaFallbackProbe");
    let media_range = node_by_control_id(&surface, "MediaRangeProbe");
    let media_up = node_by_control_id(&surface, "MediaUpProbe");
    let media_down = node_by_control_id(&surface, "MediaDownProbe");
    let media_between = node_by_control_id(&surface, "MediaBetweenProbe");
    assert_eq!(
        grid.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 12,
            rows: 1,
            column_gap: 16.0,
            row_gap: 8.0,
        })
    );
    assert_eq!(
        stack.container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 48.0 })
    );
    assert_eq!(
        grid_item_slot(&surface, item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(2, 0).with_span(6, 1))
    );
    assert_eq!(visibility_probe.visibility, UiVisibility::Hidden);
    assert!(!legacy_visible_probe.state_flags.visible);
    assert_eq!(display_probe.visibility, UiVisibility::Collapsed);
    assert_eq!(bool_metadata_attr(media_min, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_max, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_fallback, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_range, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_up, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_down, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_between, "matches"), Some(true));

    surface
        .compute_layout(zircon_runtime_interface::ui::layout::UiSize::new(
            960.0, 360.0,
        ))
        .unwrap();
    let grid = node_by_control_id(&surface, "MuiGridContainer");
    let stack = node_by_control_id(&surface, "MuiStackRow");
    let item = node_by_control_id(&surface, "MuiGridItem");
    let visibility_probe = node_by_control_id(&surface, "VisibilityProbe");
    let legacy_visible_probe = node_by_control_id(&surface, "LegacyVisibleProbe");
    let display_probe = node_by_control_id(&surface, "DisplayProbe");
    let media_min = node_by_control_id(&surface, "MediaMinProbe");
    let media_max = node_by_control_id(&surface, "MediaMaxProbe");
    let media_fallback = node_by_control_id(&surface, "MediaFallbackProbe");
    let media_range = node_by_control_id(&surface, "MediaRangeProbe");
    let media_up = node_by_control_id(&surface, "MediaUpProbe");
    let media_down = node_by_control_id(&surface, "MediaDownProbe");
    let media_between = node_by_control_id(&surface, "MediaBetweenProbe");
    assert_eq!(
        grid.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 16,
            rows: 1,
            column_gap: 40.0,
            row_gap: 24.0,
        })
    );
    assert_eq!(
        stack.container,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 80.0 })
    );
    assert_eq!(
        grid_item_slot(&surface, item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(3, 0).with_span(4, 1))
    );
    assert_eq!(visibility_probe.visibility, UiVisibility::Visible);
    assert!(legacy_visible_probe.state_flags.visible);
    assert_eq!(display_probe.visibility, UiVisibility::Visible);
    assert_eq!(bool_metadata_attr(media_min, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_max, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_fallback, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_range, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_up, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_down, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_between, "matches"), Some(false));

    surface
        .compute_layout(zircon_runtime_interface::ui::layout::UiSize::new(
            720.0, 360.0,
        ))
        .unwrap();
    let grid = node_by_control_id(&surface, "MuiGridContainer");
    let stack = node_by_control_id(&surface, "MuiStackRow");
    let item = node_by_control_id(&surface, "MuiGridItem");
    let visibility_probe = node_by_control_id(&surface, "VisibilityProbe");
    let legacy_visible_probe = node_by_control_id(&surface, "LegacyVisibleProbe");
    let display_probe = node_by_control_id(&surface, "DisplayProbe");
    let media_min = node_by_control_id(&surface, "MediaMinProbe");
    let media_max = node_by_control_id(&surface, "MediaMaxProbe");
    let media_fallback = node_by_control_id(&surface, "MediaFallbackProbe");
    let media_range = node_by_control_id(&surface, "MediaRangeProbe");
    let media_up = node_by_control_id(&surface, "MediaUpProbe");
    let media_down = node_by_control_id(&surface, "MediaDownProbe");
    let media_between = node_by_control_id(&surface, "MediaBetweenProbe");
    assert_eq!(
        grid.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 12,
            rows: 1,
            column_gap: 16.0,
            row_gap: 8.0,
        })
    );
    assert_eq!(
        stack.container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 48.0 })
    );
    assert_eq!(
        grid_item_slot(&surface, item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(2, 0).with_span(6, 1))
    );
    assert_eq!(visibility_probe.visibility, UiVisibility::Hidden);
    assert!(!legacy_visible_probe.state_flags.visible);
    assert_eq!(display_probe.visibility, UiVisibility::Collapsed);
    assert_eq!(bool_metadata_attr(media_min, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_max, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_fallback, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_range, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_up, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_down, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_between, "matches"), Some(true));
}

fn node_by_control_id<'a>(
    surface: &'a UiSurface,
    control_id: &str,
) -> &'a zircon_runtime_interface::ui::tree::UiTreeNode {
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
        .unwrap_or_else(|| panic!("{control_id} should be built"))
}

fn grid_item_slot(
    surface: &UiSurface,
    child_id: UiNodeId,
) -> &zircon_runtime_interface::ui::layout::UiSlot {
    surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == child_id)
        .expect("mui grid item should have a slot")
}

fn bool_metadata_attr(
    node: &zircon_runtime_interface::ui::tree::UiTreeNode,
    name: &str,
) -> Option<bool> {
    node.template_metadata
        .as_ref()
        .and_then(|metadata| metadata.attributes.get(name))
        .and_then(toml::Value::as_bool)
}

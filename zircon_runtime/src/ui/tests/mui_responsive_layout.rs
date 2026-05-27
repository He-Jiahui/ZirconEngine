use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{
        UiContainerKind, UiGridBoxConfig, UiGridSlotPlacement, UiLinearBoxConfig,
        UiMasonryBoxConfig, UiSize,
    },
    tree::{UiTreeNode, UiVisibility},
};

use crate::ui::{
    surface::UiSurface,
    v2::{UiV2AssetLoader, UiV2DocumentCompiler, UiV2SurfaceBuilder},
};

#[test]
fn ui_v2_mui_responsive_layout_recomputes_from_viewport_breakpoints() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/mui_responsive_breakpoints.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"
children = [{ node = "grid" }, { node = "stack" }, { node = "masonry" }, { node = "explicit_grid" }, { node = "visibility_probe" }, { node = "legacy_visible_probe" }, { node = "display_probe" }, { node = "media_min" }, { node = "media_max" }, { node = "media_fallback" }, { node = "media_range" }, { node = "media_up" }, { node = "media_down" }, { node = "media_between" }]

[nodes.grid]
component = "Grid"
control_id = "MuiV2ResponsiveGrid"
props = { container = true, columns = { xs = 4, sm = 8, md = 16 }, rowSpacing = { xs = 1, md = 3 }, columnSpacing = { xs = 1, md = 5 } }
children = [{ node = "item" }]

[nodes.item]
component = "Grid"
control_id = "MuiV2ResponsiveGridItem"
props = { size = { xs = 4, md = 8 }, offset = { xs = 0, md = 2 } }

[nodes.stack]
component = "Stack"
control_id = "MuiV2ResponsiveStack"
props = { direction = { xs = "column", md = "row" }, spacing = { xs = 1, md = 3 } }

[nodes.masonry]
component = "Masonry"
control_id = "MuiV2ResponsiveMasonry"
props = { columns = { xs = 1, md = 4 }, spacing = [1, 2, 3], sequential = true }

[nodes.explicit_grid]
component = "Grid"
control_id = "ExplicitGridBox"
props = { container = true, columns = { xs = 4, md = 12 }, layout = { container = { kind = "GridBox", columns = 2, rows = 1, gap = 7.0 } } }
children = [{ node = "explicit_item" }]

[nodes.explicit_item]
component = "Grid"
control_id = "ExplicitGridItem"
props = { size = { xs = 4, md = 12 }, offset = { xs = 0, md = 5 } }

[nodes.visibility_probe]
component = "Box"
control_id = "VisibilityProbe"
props = { visibility = { xs = "hidden", md = "visible" } }

[nodes.legacy_visible_probe]
component = "Box"
control_id = "LegacyVisibleProbe"
props = { visible = { xs = false, md = true } }

[nodes.display_probe]
component = "Box"
control_id = "DisplayProbe"
props = { display = { xs = "none", md = "flex" } }

[nodes.media_min]
component = "UseMediaQuery"
control_id = "MediaMinProbe"
props = { query = "(min-width: 900px)", defaultMatches = false }

[nodes.media_max]
component = "UseMediaQuery"
control_id = "MediaMaxProbe"
props = { query = "(max-width: 899px)", defaultMatches = false }

[nodes.media_fallback]
component = "UseMediaQuery"
control_id = "MediaFallbackProbe"
props = { query = "unsupported-query", defaultMatches = true }

[nodes.media_range]
component = "UseMediaQuery"
control_id = "MediaRangeProbe"
props = { query = "(min-width: 600px) and (max-width: 959px)", defaultMatches = false }

[nodes.media_up]
component = "UseMediaQuery"
control_id = "MediaUpProbe"
props = { up = "md", defaultMatches = false }

[nodes.media_down]
component = "UseMediaQuery"
control_id = "MediaDownProbe"
props = { down = "md", defaultMatches = false }

[nodes.media_between]
component = "UseMediaQuery"
control_id = "MediaBetweenProbe"
props = { between = ["sm", "md"], defaultMatches = false }
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.mui_responsive_breakpoints"),
        &document,
        &compiled,
    )
    .unwrap();

    surface.compute_layout(UiSize::new(599.0, 360.0)).unwrap();
    let grid = node_by_control_id(&surface, "MuiV2ResponsiveGrid");
    let stack = node_by_control_id(&surface, "MuiV2ResponsiveStack");
    let masonry = node_by_control_id(&surface, "MuiV2ResponsiveMasonry");
    let explicit_grid = node_by_control_id(&surface, "ExplicitGridBox");
    let explicit_item = node_by_control_id(&surface, "ExplicitGridItem");
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
    let item = node_by_control_id(&surface, "MuiV2ResponsiveGridItem");
    assert_eq!(
        grid.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 4,
            rows: 1,
            column_gap: 8.0,
            row_gap: 8.0,
        })
    );
    assert_eq!(
        stack.container,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 8.0 })
    );
    assert_eq!(
        masonry.container,
        UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: 1,
            gap: 8.0,
            sequential: true,
        })
    );
    assert_eq!(
        grid_item_slot(&surface, item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(0, 0).with_span(4, 1))
    );
    assert_eq!(
        explicit_grid.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 1,
            column_gap: 7.0,
            row_gap: 7.0,
        })
    );
    assert_eq!(
        grid_item_slot(&surface, explicit_item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(0, 0).with_span(4, 1))
    );
    assert_eq!(visibility_probe.visibility, UiVisibility::Hidden);
    assert!(legacy_visible_probe.visibility == UiVisibility::Visible);
    assert!(!legacy_visible_probe.state_flags.visible);
    assert_eq!(display_probe.visibility, UiVisibility::Collapsed);
    assert_eq!(bool_metadata_attr(media_min, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_max, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_fallback, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_range, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_up, "matches"), Some(false));
    assert_eq!(bool_metadata_attr(media_down, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_between, "matches"), Some(false));

    surface.compute_layout(UiSize::new(960.0, 360.0)).unwrap();
    let grid = node_by_control_id(&surface, "MuiV2ResponsiveGrid");
    let stack = node_by_control_id(&surface, "MuiV2ResponsiveStack");
    let masonry = node_by_control_id(&surface, "MuiV2ResponsiveMasonry");
    let explicit_grid = node_by_control_id(&surface, "ExplicitGridBox");
    let explicit_item = node_by_control_id(&surface, "ExplicitGridItem");
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
    let item = node_by_control_id(&surface, "MuiV2ResponsiveGridItem");
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
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 24.0 })
    );
    assert_eq!(
        masonry.container,
        UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: 4,
            gap: 24.0,
            sequential: true,
        })
    );
    assert_eq!(
        grid_item_slot(&surface, item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(2, 0).with_span(8, 1))
    );
    assert_eq!(
        explicit_grid.container,
        UiContainerKind::GridBox(UiGridBoxConfig {
            columns: 2,
            rows: 1,
            column_gap: 7.0,
            row_gap: 7.0,
        })
    );
    assert_eq!(
        grid_item_slot(&surface, explicit_item.node_id).grid_placement,
        Some(UiGridSlotPlacement::new(0, 0).with_span(4, 1))
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

    surface.compute_layout(UiSize::new(800.0, 360.0)).unwrap();
    let media_range = node_by_control_id(&surface, "MediaRangeProbe");
    let media_between = node_by_control_id(&surface, "MediaBetweenProbe");
    assert_eq!(bool_metadata_attr(media_range, "matches"), Some(true));
    assert_eq!(bool_metadata_attr(media_between, "matches"), Some(true));
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
        .unwrap_or_else(|| panic!("{control_id} should be projected"))
}

fn grid_item_slot(
    surface: &UiSurface,
    child_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
) -> &zircon_runtime_interface::ui::layout::UiSlot {
    surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == child_id)
        .expect("mui grid item should have a slot")
}

fn bool_metadata_attr(node: &UiTreeNode, name: &str) -> Option<bool> {
    node.template_metadata
        .as_ref()
        .and_then(|metadata| metadata.attributes.get(name))
        .and_then(toml::Value::as_bool)
}

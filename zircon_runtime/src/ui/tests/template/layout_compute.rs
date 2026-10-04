use super::*;

#[test]
fn template_surface_builder_computes_layout_from_template_contract_attributes() {
    let instance = compiled_instance_from_toml(LAYOUT_CONTRACT_TEMPLATE_TOML);

    let mut surface =
        UiTemplateSurfaceBuilder::build_surface(UiTreeId::new("layout.surface"), &instance)
            .unwrap();
    surface.compute_layout(UiSize::new(800.0, 600.0)).unwrap();

    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    assert_eq!(
        root.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 800.0, 600.0)
    );

    let toolbar = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Toolbar")
        })
        .unwrap();
    assert_eq!(
        toolbar.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 800.0, 48.0)
    );

    let viewport_host = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ViewportHost")
        })
        .unwrap();
    assert_eq!(
        viewport_host.layout_cache.frame,
        UiFrame::new(0.0, 60.0, 800.0, 408.0)
    );

    let overlay_badge = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OverlayBadge")
        })
        .unwrap();
    assert_eq!(
        overlay_badge.layout_cache.frame,
        UiFrame::new(724.0, 72.0, 60.0, 24.0)
    );

    let asset_list = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("AssetList")
        })
        .unwrap();
    assert_eq!(
        asset_list.layout_cache.frame,
        UiFrame::new(0.0, 480.0, 800.0, 120.0)
    );
    assert_eq!(
        asset_list.scroll_state,
        Some(UiScrollState {
            offset: 0.0,
            viewport_extent: 120.0,
            content_extent: 164.0,
        })
    );
    assert_eq!(
        asset_list.layout_cache.virtual_window,
        Some(UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 5,
        })
    );
}

#[test]
fn template_surface_builder_applies_node_layout_padding_to_children() {
    let instance = compiled_instance_from_toml(
        r#"
version = 1

[root]
component = "VerticalBox"
attributes = { layout = { container = { kind = "VerticalBox", gap = 12.0 }, padding = { left = 18.0, top = 18.0, right = 18.0, bottom = 18.0 } } }
children = [
    { component = "Label", attributes = { text = "first", layout = { height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" } } } },
    { component = "Label", attributes = { text = "second", layout = { height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" } } } },
]
"#,
    );

    let mut surface =
        UiTemplateSurfaceBuilder::build_surface(UiTreeId::new("layout.padding"), &instance)
            .unwrap();
    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();

    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    assert_eq!(
        root.layout_padding,
        zircon_runtime_interface::ui::layout::UiMargin::new(18.0, 18.0, 18.0, 18.0)
    );
    let children = root.children.clone();
    assert_eq!(children.len(), 2);
    assert_eq!(
        surface.tree.node(children[0]).unwrap().layout_cache.frame,
        UiFrame::new(18.0, 18.0, 164.0, 20.0)
    );
    assert_eq!(
        surface.tree.node(children[1]).unwrap().layout_cache.frame,
        UiFrame::new(18.0, 50.0, 164.0, 20.0)
    );
}

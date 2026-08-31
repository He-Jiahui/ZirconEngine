use super::*;

#[test]
fn template_surface_builder_maps_known_container_components_into_shared_runtime_nodes() {
    let instance = compiled_instance_from_toml(SHARED_CONTAINER_TEMPLATE_TOML);

    let surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("shared.container.template"),
        &instance,
    )
    .unwrap();

    assert_eq!(surface.tree.nodes.len(), 4);
    assert_eq!(
        surface.render_extract.tree_id.0,
        "shared.container.template"
    );

    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    assert_eq!(
        root.template_metadata
            .as_ref()
            .unwrap()
            .control_id
            .as_deref(),
        Some("ScrollRoot")
    );
    assert_eq!(
        root.container,
        UiContainerKind::ScrollableBox(Default::default())
    );
    assert_eq!(root.scroll_state, Some(UiScrollState::default()));
    assert!(root.clip_to_bounds);

    let row = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Row")
        })
        .unwrap();
    assert_eq!(
        row.container,
        UiContainerKind::HorizontalBox(Default::default())
    );

    let gap = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Gap")
        })
        .unwrap();
    assert_eq!(gap.container, UiContainerKind::Space);

    let interactive_leaf = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("InteractiveLeaf")
        })
        .unwrap();
    assert_eq!(interactive_leaf.input_policy, UiInputPolicy::Receive);
    assert!(interactive_leaf.state_flags.clickable);
    assert!(interactive_leaf.state_flags.hoverable);
    assert!(interactive_leaf.state_flags.focusable);
}

#[test]
fn template_surface_builder_leaves_projection_lazy_until_layout_or_rebuild() {
    let instance = compiled_instance_from_toml(SHARED_CONTAINER_TEMPLATE_TOML);

    let mut surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("shared.container.template.lazy"),
        &instance,
    )
    .unwrap();

    assert_eq!(surface.tree.nodes.len(), 4);
    assert!(surface.arranged_tree.nodes.is_empty());
    assert!(surface.render_extract.list.commands.is_empty());
    assert_eq!(surface.last_rebuild_report.render_command_count, 0);

    surface.compute_layout(UiSize::new(320.0, 180.0)).unwrap();

    assert_eq!(surface.arranged_tree.nodes.len(), surface.tree.nodes.len());
    assert_eq!(
        surface.render_extract.list.commands.len(),
        surface.tree.nodes.len()
    );
    assert_eq!(surface.last_rebuild_report.render_command_count, 4);
}

#[test]
fn template_tree_builder_maps_layout_contract_attributes_into_shared_runtime_nodes() {
    let instance = compiled_instance_from_toml(LAYOUT_CONTRACT_TEMPLATE_TOML);

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("layout.contract"), &instance).unwrap();

    let root = tree.node(tree.roots[0]).unwrap();
    assert_eq!(
        root.container,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 12.0 })
    );
    assert!(root.clip_to_bounds);

    let toolbar = tree
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
        toolbar.container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 8.0 })
    );
    assert_eq!(
        toolbar.constraints.height,
        AxisConstraint {
            min: 48.0,
            max: 48.0,
            preferred: 48.0,
            priority: 0,
            weight: 1.0,
            stretch_mode: StretchMode::Fixed,
        }
    );

    let overlay_badge = tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OverlayBadge")
        })
        .unwrap();
    assert_eq!(overlay_badge.anchor.x, 1.0);
    assert_eq!(overlay_badge.anchor.y, 0.0);
    assert_eq!(overlay_badge.pivot.x, 1.0);
    assert_eq!(overlay_badge.pivot.y, 0.0);
    assert_eq!(overlay_badge.position.x, -16.0);
    assert_eq!(overlay_badge.position.y, 12.0);
    assert_eq!(overlay_badge.z_index, 4);

    let asset_list = tree
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
        asset_list.container,
        UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: UiAxis::Vertical,
            gap: 6.0,
            scrollbar_visibility: UiScrollbarVisibility::Always,
            virtualization: Some(UiVirtualListConfig {
                item_extent: 28.0,
                overscan: 2,
            }),
        })
    );
    assert!(asset_list.clip_to_bounds);
}

#[test]
fn template_tree_builder_parses_size_box_container_contract() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "SizeBox", control_id = "PreviewFit", attributes = { layout = { container = { kind = "SizeBox", aspect_ratio = 2.0 } } }, children = [{ component = "Image", control_id = "PreviewImage" }] }"#,
    ));
    let root = only_root_node(&tree);

    assert_eq!(
        root.container,
        UiContainerKind::SizeBox(UiSizeBoxConfig { aspect_ratio: 2.0 })
    );
}

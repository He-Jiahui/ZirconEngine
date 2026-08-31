use super::*;

#[test]
fn rich_text_name_uses_compiled_visible_text_instead_of_source_markup() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/rich-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_template_metadata(metadata(
                    "Text",
                    "text = '<b>Visible</b> label'\nrich_text_format = 'html_subset_v1'",
                )),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(2)).unwrap().name.as_deref(),
        Some("Visible label")
    );
}

#[test]
fn labelled_by_rich_text_uses_the_target_compiled_visible_text() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/rich-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_template_metadata(metadata(
                    "Text",
                    "text = '[b]Rich[/b] relation'\nrich_text_format = 'bbcode_v1'",
                )),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 32.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Rich relation")
    );
}

#[test]
fn stale_rich_text_artifact_does_not_fall_back_to_source_markup() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/rich-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_template_metadata(metadata(
                    "Text",
                    "text = '<b>Before</b>'\nrich_text_format = 'html_subset_v1'",
                )),
        )
        .unwrap();
    surface.rebuild();
    surface
        .tree
        .nodes
        .get_mut(&id(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("rich label metadata")
        .attributes
        .insert(
            "text".to_string(),
            toml::Value::String("<b>After</b>".to_string()),
        );

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(snapshot.node(id(2)).unwrap().name, None);
}

#[test]
fn hidden_rich_relation_target_uses_surface_text_owner_without_render_command() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-rich-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 160.0, 24.0))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata(
                    "Text",
                    "text = '<b>Hidden</b> relation'\nrich_text_format = 'html_subset_v1'",
                )),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 32.0, 160.0, 24.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    assert!(surface.current_render_commands_for_node(id(2)).is_none());
    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Hidden relation")
    );
    assert!(snapshot.node(id(2)).unwrap().state.hidden);
}

#[test]
fn name_priority_uses_explicit_labelled_by_text_alt_then_tooltip() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/label"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 16.0))
                .with_template_metadata(metadata("Label", "text = 'Label source'")),
        )
        .unwrap();
    for (node_id, path, a11y, attributes) in [
        (
            3,
            "root/explicit",
            UiAccessibilityContract {
                name: Some("Explicit".to_string()),
                ..UiAccessibilityContract::default()
            },
            "text = 'Ignored'",
        ),
        (
            4,
            "root/labelled",
            UiAccessibilityContract {
                labelled_by: Some("2".to_string()),
                ..UiAccessibilityContract::default()
            },
            "text = 'Ignored own text'",
        ),
        (
            5,
            "root/text",
            UiAccessibilityContract::default(),
            "text = 'Own text'",
        ),
        (
            6,
            "root/alt",
            UiAccessibilityContract::default(),
            "alt_text = 'Alt text'",
        ),
        (
            7,
            "root/tooltip",
            UiAccessibilityContract::default(),
            "tooltip = 'Tooltip text'",
        ),
    ] {
        surface
            .tree
            .insert_child(
                id(1),
                UiTreeNode::new(id(node_id), UiNodePath::new(path))
                    .with_frame(UiFrame::new(4.0, node_id as f32 * 18.0, 80.0, 16.0))
                    .with_template_metadata(UiTemplateNodeMetadata {
                        component: "Button".to_string(),
                        attributes: toml::from_str(attributes).unwrap(),
                        a11y,
                        ..UiTemplateNodeMetadata::default()
                    }),
            )
            .unwrap();
    }
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Explicit")
    );
    assert_eq!(
        snapshot.node(id(4)).unwrap().name.as_deref(),
        Some("Label source")
    );
    assert_eq!(
        snapshot.node(id(5)).unwrap().name.as_deref(),
        Some("Own text")
    );
    assert_eq!(
        snapshot.node(id(6)).unwrap().name.as_deref(),
        Some("Alt text")
    );
    assert_eq!(
        snapshot.node(id(7)).unwrap().name.as_deref(),
        Some("Tooltip text")
    );
}

#[test]
fn labelled_by_uses_higher_id_tooltip_target_without_order_dependency() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("3".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/tooltip-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 16.0))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    a11y: UiAccessibilityContract {
                        tooltip: Some("Tooltip label".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(2)).unwrap().name.as_deref(),
        Some("Tooltip label")
    );
}

#[test]
fn hidden_label_references_are_retained_without_visible_children() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-label"))
                .with_frame(UiFrame::new(4.0, 4.0, 80.0, 16.0))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(metadata("Label", "text = 'Hidden label'")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(4.0, 28.0, 80.0, 20.0))
                .with_state_flags(state(false, true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Hidden label")
    );
    assert!(snapshot.node(id(2)).unwrap().state.hidden);
    assert!(snapshot.node(id(2)).unwrap().children.is_empty());
    assert!(!snapshot.node(id(1)).unwrap().children.contains(&id(2)));
    assert!(snapshot.diagnostics.iter().all(|diagnostic| {
        diagnostic.code != UiAccessibilityDiagnosticCode::MissingBounds
            || diagnostic.node_id != Some(id(2))
    }));
}

#[test]
fn excluded_hidden_relation_owners_do_not_retain_targets() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/unused-label-target")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden-owner"))
                .with_visibility(UiVisibility::Hidden)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    a11y: UiAccessibilityContract {
                        labelled_by: Some("2".to_string()),
                        ..UiAccessibilityContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(3)).is_none());
    assert!(snapshot.node(id(2)).is_none());
    assert!(snapshot.node(id(1)).unwrap().children.is_empty());
}

#[test]
fn included_descendants_are_promoted_through_excluded_containers() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(id(1), UiTreeNode::new(id(2), UiNodePath::new("root/slot")))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/slot/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Nested'")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert_eq!(snapshot.node(id(1)).unwrap().children, vec![id(3)]);
    assert_eq!(
        snapshot.node(id(3)).unwrap().name.as_deref(),
        Some("Nested")
    );
}

#[test]
fn hidden_excluded_containers_block_descendant_promotion() {
    let mut surface = root_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden-slot"))
                .with_visibility(UiVisibility::Hidden),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden-slot/button"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 24.0))
                .with_state_flags(state(true, true))
                .with_template_metadata(metadata("Button", "text = 'Hidden descendant'")),
        )
        .unwrap();
    surface.rebuild();

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(2)).is_none());
    assert!(snapshot.node(id(3)).is_none());
    assert!(!snapshot.node(id(1)).unwrap().children.contains(&id(3)));
}

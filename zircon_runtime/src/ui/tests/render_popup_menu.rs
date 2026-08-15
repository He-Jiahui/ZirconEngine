use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiDirtyFlags, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
    widget::{UiPopupAnchor, UiWidgetContract},
};

#[test]
fn popup_menu_rendering_uses_set_lookup_and_moves_owned_labels() {
    let source = include_str!("../surface/render/popup_menu.rs");
    let rows = include_str!("../surface/render/popup_rows.rs");

    assert!(
        !source.contains(".iter().any(|value| self.matches_id(value))"),
        "popup menu state lookup should use the BTreeSet index"
    );
    assert!(
        !source.contains("item.label.clone()"),
        "popup menu rows should move their already-owned labels into commands"
    );
    assert!(rows.contains("EditorDesignTokens"));
    assert!(rows.contains("EditorTypographyTokens"));
    assert!(!rows.contains("const POPUP_BACKGROUND"));
    assert!(!rows.contains("const POPUP_TEXT"));
}

#[test]
fn render_extract_expands_open_context_action_menu_items() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_menu"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 140.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 96.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "Play In Editor"
popup_open = true
menu_items = ["Play In Editor|checked,icon=play", "Simulate", "---", "Network Preview|submenu"]
background_color = "#151b1f"
border_color = "#303840"
border_width = 1.0
corner_radius = 5.0
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let owner = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Play In Editor")
        })
        .expect("open menu owner should keep its visible value command");
    assert_eq!(owner.kind, UiRenderCommandKind::Quad);

    let expanded_item_texts = commands
        .iter()
        .filter(|command| command.node_id == UiNodeId::new(2))
        .filter(|command| matches!(command.kind, UiRenderCommandKind::Text))
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(expanded_item_texts.contains(&"Simulate"));
    assert!(expanded_item_texts.contains(&"Network Preview"));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(8.0, 8.0, 160.0, 96.0)
    }));
}

#[test]
fn render_extract_positions_context_menu_from_explicit_anchor_and_flip() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.popup_menu.anchor_position",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 140.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(20.0, 20.0, 120.0, 72.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
popup_open = true
placement = "right-start"
popup_anchor_x = 184.0
popup_anchor_y = 24.0
popup_offset_x = -2.0
menu_items = ["Rename", "Delete|danger"]
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("menu node should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 200.0, 140.0));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let owner = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Rename")
        })
        .expect("open menu should render the first item label");
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index < owner.z_index
            && command.frame == UiFrame::new(58.0, 24.0, 120.0, 72.0)
            && command.clip_frame == Some(UiFrame::new(0.0, 0.0, 200.0, 140.0))
    }));
}

#[test]
fn popup_trigger_identity_resolves_open_popup_from_live_control_anchor_frame() {
    let mut surface = control_anchored_popup_surface(&["window-menu-trigger"]);

    assert_eq!(
        popup_background_frame(&surface),
        UiFrame::new(20.0, 20.0, 120.0, 72.0)
    );
    assert_eq!(
        surface.input.popup_owner("root/popup"),
        Some(UiNodeId::new(2))
    );

    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("trigger node should exist")
        .layout_cache
        .frame = UiFrame::new(72.0, 8.0, 40.0, 20.0);
    surface.rebuild();

    assert_eq!(
        popup_background_frame(&surface),
        UiFrame::new(72.0, 32.0, 120.0, 72.0)
    );
}

#[test]
fn render_extract_rejects_missing_duplicate_and_disabled_control_anchors() {
    let missing = control_anchored_popup_surface(&[]);
    assert_no_control_anchored_popup(&missing);

    let duplicate = control_anchored_popup_surface(&["window-menu-trigger", "window-menu-trigger"]);
    assert_no_control_anchored_popup(&duplicate);

    let mut disabled = control_anchored_popup_surface(&["window-menu-trigger"]);
    disabled
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("trigger node should exist")
        .state_flags
        .enabled = false;
    disabled.rebuild();
    assert_no_control_anchored_popup(&disabled);

    let mut collapsed = control_anchored_popup_surface(&["window-menu-trigger"]);
    collapsed
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("trigger node should exist")
        .visibility = UiVisibility::Collapsed;
    collapsed.rebuild();
    assert_no_control_anchored_popup(&collapsed);

    let mut invisible = control_anchored_popup_surface(&["window-menu-trigger"]);
    invisible
        .tree
        .node_mut(UiNodeId::new(2))
        .expect("trigger node should exist")
        .state_flags
        .visible = false;
    invisible.rebuild();
    assert_no_control_anchored_popup(&invisible);
}

#[test]
fn opened_control_anchored_popup_forces_full_render_extract_after_trigger_dirty() {
    let mut surface = control_anchored_popup_surface(&["window-menu-trigger"]);
    let root_size = UiSize::new(200.0, 140.0);
    surface.rebuild_dirty(root_size).unwrap();
    surface
        .mark_node_dirty(
            UiNodeId::new(2),
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();

    let report = surface.rebuild_dirty(root_size).unwrap();

    assert_eq!(
        report.render_outer_node_visit_count,
        surface.arranged_tree.draw_order.len(),
        "a trigger change must not leave a popup command patched from a partial arranged tree"
    );
}

#[test]
fn opened_control_anchored_popup_forces_full_render_extract_after_popup_ancestor_dirty() {
    let mut surface = control_anchored_popup_surface(&["window-menu-trigger"]);
    let root_size = UiSize::new(200.0, 140.0);
    surface.rebuild_dirty(root_size).unwrap();
    surface
        .mark_node_dirty(
            UiNodeId::new(1),
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();

    let report = surface.rebuild_dirty(root_size).unwrap();

    assert_eq!(
        report.render_outer_node_visit_count,
        surface.arranged_tree.draw_order.len(),
        "a popup ancestor change must not leave cached popup geometry or visibility stale"
    );
}

#[test]
fn opened_control_anchored_popup_keeps_unrelated_render_dirty_nodes_local() {
    let mut surface = control_anchored_popup_surface(&["window-menu-trigger"]);
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(11), UiNodePath::new("root/status"))
                .with_frame(UiFrame::new(8.0, 8.0, 80.0, 20.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Label".to_string(),
                    attributes: toml::from_str("text = \"Ready\"").unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();

    let root_size = UiSize::new(200.0, 140.0);
    surface.rebuild_dirty(root_size).unwrap();
    surface
        .mark_node_dirty(
            UiNodeId::new(11),
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
        .unwrap();

    let report = surface.rebuild_dirty(root_size).unwrap();

    assert_eq!(
        report.render_outer_node_visit_count, 1,
        "an unrelated render change must not rebuild an open popup whose trigger did not change"
    );
}

#[test]
fn opened_control_anchored_popup_rejects_a_runtime_duplicate_trigger() {
    let mut surface = control_anchored_popup_surface(&["window-menu-trigger"]);
    let root_size = UiSize::new(200.0, 140.0);
    surface.rebuild_dirty(root_size).unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(11), UiNodePath::new("root/duplicate-trigger"))
                .with_frame(UiFrame::new(120.0, 96.0, 40.0, 20.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("window-menu-trigger".to_string()),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    let report = surface.rebuild_dirty(root_size).unwrap();

    assert_eq!(
        report.render_outer_node_visit_count,
        surface.arranged_tree.draw_order.len(),
        "a new competing control id must invalidate the popup's trigger placement"
    );
    assert_no_control_anchored_popup(&surface);
}

#[test]
fn render_extract_loading_context_menu_item_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_menu.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 140.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 72.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
popup_open = true
menu_items = ["Delete|checked,hovered,pressed,danger,loading", "Simulate"]
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Delete")
            && command.frame == UiFrame::new(16.0, 18.0, 144.0, 16.0)
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(8.0, 8.0, 160.0, 36.0)
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(8.0, 12.0, 3.0, 28.0)
    }));
}

#[test]
fn render_extract_focused_context_menu_option_keeps_neutral_surface_until_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.popup_menu.focused_neutral",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 140.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(12.0, 12.0, 160.0, 60.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
open = true
options = ["rename|label=Rename", "delete|label=Delete"]
focused_index = 0
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let focused_surface = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Quad
                && command.style.painter_family == UiPainterFamily::PopupRow
                && command.frame == UiFrame::new(12.0, 12.0, 160.0, 30.0)
        })
        .expect("focused-only popup row should keep a neutral focus surface");
    assert_eq!(
        focused_surface.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_surface.style.background_color.as_deref(),
        Some("#141618")
    );
    assert_eq!(
        focused_surface.style.border_color.as_deref(),
        Some("#323a41")
    );
    assert_eq!(focused_surface.style.border_width, 1.0);
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 12.0, 160.0, 30.0)
            && command.style.background_color.as_deref() == Some("#2a3036")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Rename")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
    }));
}

#[test]
fn render_extract_context_menu_options_use_popup_row_state_matrix() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.popup_menu.context_menu_states",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 180.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/menu"))
                .with_frame(UiFrame::new(12.0, 12.0, 160.0, 120.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
open = true
options = ["open|label=Open,checked", "rename|label=Rename", "delete|label=Delete,danger", "archive|label=Archive"]
disabled_options = ["delete"]
loading_options = ["archive"]
focused_index = 1
hovered_option_id = "rename"
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.frame == UiFrame::new(12.0, 12.0, 160.0, 30.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.frame == UiFrame::new(12.0, 16.0, 3.0, 22.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Rename")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
            && command.style.foreground_color.as_deref() == Some("#3cc7d6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Delete")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Disabled
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Archive")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 72.0, 160.0, 30.0)
    }));
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn control_anchored_popup_surface(trigger_control_ids: &[&str]) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_menu.control_anchor"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 140.0))
            .with_state_flags(visible_state()),
    );
    for (index, control_id) in trigger_control_ids.iter().enumerate() {
        let node_id = UiNodeId::new(2 + index as u64);
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(format!("root/trigger/{index}")))
                    .with_frame(UiFrame::new(20.0 + index as f32 * 48.0, 96.0, 40.0, 20.0))
                    .with_state_flags(visible_state())
                    .with_template_metadata(UiTemplateNodeMetadata {
                        component: "Button".to_string(),
                        control_id: Some((*control_id).to_string()),
                        ..UiTemplateNodeMetadata::default()
                    }),
            )
            .unwrap();
    }
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(10), UiNodePath::new("root/popup"))
                .with_frame(UiFrame::new(0.0, 0.0, 120.0, 72.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r##"
popup_open = true
menu_items = ["Rename", "Delete|danger"]
"##,
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        popup_anchor: UiPopupAnchor::Control {
                            control_id: "window-menu-trigger".to_string(),
                        },
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .node_mut(UiNodeId::new(10))
        .expect("popup node should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 200.0, 140.0));
    surface.rebuild();
    surface
}

fn popup_background_frame(surface: &UiSurface) -> UiFrame {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(10)
                && command.kind == UiRenderCommandKind::Quad
                && command.z_index > 0
        })
        .expect("open popup should emit a positioned background")
        .frame
}

fn assert_no_control_anchored_popup(surface: &UiSurface) {
    assert!(
        surface
            .render_extract
            .list
            .commands
            .iter()
            .all(|command| command.node_id != UiNodeId::new(10)),
        "a rejected control anchor must not fall back to popup-owned geometry"
    );
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(10))
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get("popup_open"))
            .and_then(toml::Value::as_bool),
        Some(false),
        "an invalid trigger must close the runtime popup state"
    );
}

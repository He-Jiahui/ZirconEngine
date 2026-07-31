use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn popup_option_rendering_reuses_position_and_avoids_parser_allocations() {
    let options = include_str!("../surface/render/popup_options.rs");
    let position = include_str!("../surface/render/popup_position.rs");

    assert!(
        !options.contains("option_row_frame_within"),
        "popup row frames should derive from the already-resolved popup frame"
    );
    assert!(
        !position.contains("to_ascii_lowercase") && !position.contains(".replace("),
        "popup placement parsing should compare normalized separators without allocating"
    );
}

#[test]
fn render_extract_expands_open_dropdown_options() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_options"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 180.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(12.0, 16.0, 148.0, 28.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "surface"
value_text = "Surface"
popup_open = true
options = ["surface|label=Surface", "post_process|label=Post Process", "volume|label=Volume"]
hovered_options = ["post_process"]
disabled_options = ["volume"]
background_color = "#10161a"
border_color = "#323f47"
border_width = 1.0
corner_radius = 4.0
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
        .expect("dropdown node should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(12.0, 16.0, 148.0, 28.0));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let owner = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("surface")
        })
        .expect("open dropdown owner should keep its visible value command");
    assert_eq!(owner.kind, UiRenderCommandKind::Quad);

    let option_texts = commands
        .iter()
        .filter(|command| command.node_id == UiNodeId::new(2))
        .filter(|command| matches!(command.kind, UiRenderCommandKind::Text))
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert!(option_texts.contains(&"Surface"));
    assert!(option_texts.contains(&"Post Process"));
    assert!(option_texts.contains(&"Volume"));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(12.0, 48.0, 148.0, 84.0)
            && command.clip_frame.is_none()
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(12.0, 48.0, 148.0, 28.0)
    }));
}

#[test]
fn render_extract_positions_dropdown_popup_from_explicit_anchor_and_bounds() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.popup_options.anchor_position",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 120.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(12.0, 16.0, 148.0, 28.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "surface"
popup_open = true
placement = "bottom-start"
popup_anchor_x = 160.0
popup_anchor_y = 92.0
popup_anchor_width = 56.0
popup_anchor_height = 20.0
popup_offset_y = 2.0
options = ["surface|label=Surface", "post_process|label=Post Process", "volume|label=Volume"]
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
        .expect("dropdown node should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 220.0, 120.0));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let owner = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("surface")
        })
        .expect("open dropdown owner should keep its visible value command");
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.z_index > owner.z_index
            && command.frame == UiFrame::new(72.0, 6.0, 148.0, 84.0)
            && command.clip_frame == Some(UiFrame::new(0.0, 0.0, 220.0, 120.0))
    }));
}

#[test]
fn render_extract_loading_dropdown_option_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.popup_options.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 180.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(12.0, 16.0, 148.0, 28.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Dropdown".to_string(),
                    attributes: toml::from_str(
                        r##"
value = "surface"
value_text = "Surface"
popup_open = true
options = ["surface|label=Surface,loading,hovered,pressed,special", "post_process|label=Post Process"]
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
            && command.text.as_deref() == Some("Surface")
            && command.frame == UiFrame::new(21.0, 53.0, 130.0, 18.0)
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 48.0, 148.0, 28.0)
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 52.0, 3.0, 20.0)
    }));
}

#[test]
fn render_extract_dropdown_popup_options_use_popup_row_state_matrix() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.popup_options.dropdown_popup_states",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 180.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown_popup"))
                .with_frame(UiFrame::new(12.0, 16.0, 148.0, 112.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "DropdownPopup".to_string(),
                    attributes: toml::from_str(
                        r##"
open = true
options = ["scene|label=Scene", "assets|label=Assets", "console|label=Console", "render|label=Render"]
selected_options = ["assets"]
disabled_options = ["render"]
focused_index = 2
hovered_option_id = "console"
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
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::Open
            && command.frame == UiFrame::new(12.0, 16.0, 148.0, 112.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.frame == UiFrame::new(12.0, 44.0, 148.0, 28.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.frame == UiFrame::new(12.0, 48.0, 3.0, 20.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Console")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Focused
            && command.style.foreground_color.as_deref() == Some("#35c7d0")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Render")
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.style.painter_state == UiPainterResolvedState::Disabled
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::PopupRow
            && command.frame == UiFrame::new(12.0, 100.0, 148.0, 28.0)
    }));
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

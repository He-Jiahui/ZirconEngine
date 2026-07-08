use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_dropdown_trigger_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.dropdowns"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/dropdown"))
                .with_frame(UiFrame::new(12.0, 16.0, 160.0, 30.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Select".to_string(),
                    attributes: toml::from_str(
                        r##"
label = "Blend"
value = "post_process"
popup_open = true
options = ["surface|label=Surface", "post_process|label=Post Process", "volume|label=Volume"]
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

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 16.0, 160.0, 30.0)
            && command.style.background_color.as_deref() == Some("#16282d")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::Open
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Blend")
            && command.frame == UiFrame::new(20.0, 20.0, 124.0, 12.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Post Process")
            && command.frame == UiFrame::new(20.0, 31.0, 124.0, 13.200001)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-up".to_string()))
            && command.frame == UiFrame::new(148.0, 25.0, 12.0, 12.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(168.0, 21.0, 2.0, 20.0)
            && command.style.background_color.as_deref() == Some("#35c7d0")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Blend")
            })
            .count(),
        1
    );
    assert!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.text.as_deref() == Some("Post Process")
            })
            .count()
            >= 2,
        "open Select should render both trigger value text and popup option text"
    );
}

#[test]
fn render_extract_dropdown_uses_shared_metadata_painter_state_priority() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.dropdowns.state"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 280.0, 112.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Dropdown",
        UiFrame::new(12.0, 12.0, 160.0, 30.0),
        r##"
value_text = "Compiling"
popup_open = true
pressed = true
focused = true
hovered = true
loading = true
"##,
        UiStateFlags {
            pressed: true,
            ..visible_state()
        },
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Select",
        UiFrame::new(12.0, 56.0, 160.0, 30.0),
        r##"
value_text = "Drop Target"
active_drag_target = true
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 12.0, 160.0, 30.0)
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::Loading
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 56.0, 160.0, 30.0)
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::DropHovered
    }));
}

#[test]
fn render_extract_dropdown_trigger_keeps_focused_background_neutral_until_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.dropdowns.focused_neutral"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 280.0, 128.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Dropdown",
        UiFrame::new(12.0, 12.0, 160.0, 30.0),
        r##"
value_text = "Surface"
focused = true
background_color = "#10161a"
hover_background_color = "#1a2429"
border_color = "#323f47"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Select",
        UiFrame::new(12.0, 56.0, 160.0, 30.0),
        r##"
value_text = "Post Process"
focused = true
hovered = true
background_color = "#10161a"
hover_background_color = "#1a2429"
border_color = "#323f47"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let focused_surface = dropdown_surface(
        commands,
        UiNodeId::new(2),
        UiFrame::new(12.0, 12.0, 160.0, 30.0),
    );
    assert_eq!(
        focused_surface.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_surface.style.background_color.as_deref(),
        Some("#10161a")
    );
    assert_eq!(
        focused_surface.style.border_color.as_deref(),
        Some("#35c7d0")
    );
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 12.0, 160.0, 30.0)
            && command.style.background_color.as_deref() == Some("#1a2429")
    }));

    let focused_hovered_surface = dropdown_surface(
        commands,
        UiNodeId::new(3),
        UiFrame::new(12.0, 56.0, 160.0, 30.0),
    );
    assert_eq!(
        focused_hovered_surface.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_hovered_surface.style.background_color.as_deref(),
        Some("#1a2429")
    );
    assert_eq!(
        focused_hovered_surface.style.border_color.as_deref(),
        Some("#35c7d0")
    );
}

#[test]
fn render_extract_loading_dropdown_trigger_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.dropdowns.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Dropdown",
        UiFrame::new(12.0, 16.0, 160.0, 30.0),
        r##"
label = "Mode"
value_text = "Compiling"
popup_open = true
pressed = true
focused = true
hovered = true
loading = true
background_color = "#153940"
open_background_color = "#17454c"
pressed_background_color = "#1f5660"
hover_background_color = "#22636d"
border_color = "#2a909d"
focus_border_color = "#35c7d0"
label_color = "#a7dde4"
foreground_color = "#d9fbff"
icon_color = "#ef7066"
"##,
        UiStateFlags {
            pressed: true,
            ..visible_state()
        },
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 16.0, 160.0, 30.0)
            && command.style.painter_family == UiPainterFamily::Dropdown
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Mode")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Compiling")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-down".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(168.0, 21.0, 2.0, 20.0)
    }));
}

fn insert_control(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new(format!("root/{component}")))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn dropdown_surface(
    commands: &[UiRenderCommand],
    node_id: UiNodeId,
    frame: UiFrame,
) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.frame == frame
                && command.style.painter_family == UiPainterFamily::Dropdown
        })
        .expect("dropdown trigger surface should be rendered")
}

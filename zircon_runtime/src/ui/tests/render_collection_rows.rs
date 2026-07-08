use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_collection_row_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.collection_rows"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 360.0, 180.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "ListRow",
        UiFrame::new(12.0, 16.0, 180.0, 28.0),
        r##"
label = "Health Regen"
selected = true
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "TreeRow",
        UiFrame::new(12.0, 52.0, 240.0, 24.0),
        r##"
label = "LevelRoot"
selected = true
expanded = true
tree_depth = 1.0
icon = "box"
background_color = "#12383d"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "TableRow",
        UiFrame::new(12.0, 88.0, 240.0, 28.0),
        r##"
label = "Asset_01"
selected = true
background_color = "#0d4149"
"##,
        UiStateFlags {
            enabled: true,
            visible: true,
            pressed: true,
            ..UiStateFlags::default()
        },
    );
    insert_control(
        &mut surface,
        UiNodeId::new(5),
        "TreeRow",
        UiFrame::new(12.0, 124.0, 240.0, 24.0),
        r##"
label = "RuntimeFolder"
expanded = false
icon = "folder"
"##,
        visible_state(),
    );
    assert!(surface
        .component_states
        .set_expanded(UiNodeId::new(5), true));
    surface
        .mark_component_state_render_dirty(UiNodeId::new(5))
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::ListRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Health Regen")
            && command.style.painter_family == UiPainterFamily::ListRow
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.text.as_deref() == Some("Health Regen")
            })
            .count(),
        1
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TreeRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.style.background_color.as_deref() == Some("#12383d")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-down".to_string()))
            && command.style.painter_family == UiPainterFamily::TreeRow
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(3)
                    && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("box".to_string()))
            })
            .count(),
        1
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("LevelRoot")
            && command.style.painter_family == UiPainterFamily::TreeRow
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TableRow
            && command.style.painter_state == UiPainterResolvedState::Pressed
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Asset_01")
            && command.style.painter_family == UiPainterFamily::TableRow
            && command.style.painter_state == UiPainterResolvedState::Pressed
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(5)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-down".to_string()))
            && command.style.painter_family == UiPainterFamily::TreeRow
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(5)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-right".to_string()))
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(4) && command.text.as_deref() == Some("Asset_01")
            })
            .count(),
        1
    );
}

#[test]
fn render_extract_collection_rows_keep_selected_identity_when_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.collection_rows.selected_hover",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(20),
        "ListRow",
        UiFrame::new(12.0, 16.0, 200.0, 28.0),
        r##"
label = "Hovered Selected"
selected = true
background_color = "#0d4149"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(21),
        "TableRow",
        UiFrame::new(12.0, 52.0, 240.0, 28.0),
        r##"
label = "Hovered Checked"
checked = true
background_color = "#0d4149"
"##,
        visible_state(),
    );
    assert!(surface
        .component_states
        .set_hovered(UiNodeId::new(20), true));
    assert!(surface
        .component_states
        .set_drop_hovered(UiNodeId::new(20), true));
    assert!(surface
        .component_states
        .set_hovered(UiNodeId::new(21), true));
    assert!(surface
        .component_states
        .set_drop_hovered(UiNodeId::new(21), true));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(20)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::ListRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(21)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TableRow
            && command.style.painter_state == UiPainterResolvedState::Checked
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
}

#[test]
fn render_extract_collection_rows_keep_focused_background_neutral_until_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.collection_rows.focus_hover",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 360.0, 148.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(30),
        "ListRow",
        UiFrame::new(12.0, 12.0, 200.0, 28.0),
        r##"
label = "Focused List"
background_color = "#101820"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(31),
        "TreeRow",
        UiFrame::new(12.0, 48.0, 240.0, 24.0),
        r##"
label = "Focused Tree"
background_color = "#101820"
icon = "folder"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(32),
        "TableRow",
        UiFrame::new(12.0, 80.0, 280.0, 28.0),
        r##"
cells = ["Focused Asset", "Mesh", "12 KB", "Now"]
background_color = "#0d1114"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(33),
        "ListRow",
        UiFrame::new(12.0, 116.0, 200.0, 28.0),
        r##"
label = "Focused Hovered"
background_color = "#101820"
"##,
        visible_state(),
    );
    for node_id in [
        UiNodeId::new(30),
        UiNodeId::new(31),
        UiNodeId::new(32),
        UiNodeId::new(33),
    ] {
        assert!(surface.component_states.set_focused(node_id, true));
        surface.mark_component_state_render_dirty(node_id).unwrap();
    }
    assert!(surface
        .component_states
        .set_hovered(UiNodeId::new(33), true));
    surface
        .mark_component_state_render_dirty(UiNodeId::new(33))
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    for (node_id, family, background) in [
        (UiNodeId::new(30), UiPainterFamily::ListRow, "#101820"),
        (UiNodeId::new(31), UiPainterFamily::TreeRow, "#101820"),
        (UiNodeId::new(32), UiPainterFamily::TableRow, "#0d1114"),
    ] {
        let surface = row_surface(commands, node_id, family);
        assert_eq!(surface.style.painter_state, UiPainterResolvedState::Focused);
        assert_eq!(surface.style.background_color.as_deref(), Some(background));
        assert_eq!(surface.style.border_color.as_deref(), Some("#35c7d0"));
    }

    let hovered = row_surface(commands, UiNodeId::new(33), UiPainterFamily::ListRow);
    assert_eq!(hovered.style.painter_state, UiPainterResolvedState::Focused);
    assert_eq!(hovered.style.background_color.as_deref(), Some("#1a2429"));
}

#[test]
fn render_extract_loading_collection_rows_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.collection_rows.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 420.0, 160.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(10),
        "ListRow",
        UiFrame::new(12.0, 16.0, 200.0, 28.0),
        r##"
label = "Loading List"
selected = true
checked = true
hovered = true
focused = true
pressed = true
loading = true
background_color = "#0d4149"
foreground_color = "#35c7d0"
icon_color = "#7ae6f0"
focus_border_color = "#ff00ff"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(11),
        "TreeRow",
        UiFrame::new(12.0, 54.0, 260.0, 24.0),
        r##"
label = "Loading Tree"
selected = true
checked = true
expanded = true
hovered = true
focused = true
pressed = true
loading = true
icon = "folder"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(12),
        "TableRow",
        UiFrame::new(12.0, 88.0, 300.0, 28.0),
        r##"
cells = ["Loading Asset", "Mesh", "12 KB", "Now"]
selected = true
hovered = true
focused = true
pressed = true
loading = true
background_color = "#0d4149"
foreground_color = "#35c7d0"
value_color = "#aab5ba"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(10) && command.kind == UiRenderCommandKind::Quad
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(10)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Loading List")
            && command.style.painter_family == UiPainterFamily::ListRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(10)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("diamond".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(10)
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("check".to_string()))
    }));

    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(11) && command.kind == UiRenderCommandKind::Quad
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(11)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Loading Tree")
            && command.style.painter_family == UiPainterFamily::TreeRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(11)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-down".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(12)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TableRow
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.is_none()
            && command.style.border_width == 0.0
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(12)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(12)
                    && command.kind == UiRenderCommandKind::Text
                    && command.style.painter_state == UiPainterResolvedState::Loading
                    && command.style.foreground_color.as_deref() == Some("#59656c")
            })
            .count(),
        4
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(12)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref()
                == Some(&UiVisualAssetRef::Icon("more-horizontal".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
}

fn row_surface(
    commands: &[UiRenderCommand],
    node_id: UiNodeId,
    family: UiPainterFamily,
) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.style.painter_family == family
                && command.frame.width > 1.0
                && command.frame.height > 1.0
        })
        .expect("row surface command")
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

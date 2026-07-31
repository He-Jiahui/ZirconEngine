use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn divider_renderer_classifies_before_visual_resolution_and_uses_shared_tokens() {
    let source = include_str!("../surface/render/divider.rs");
    let classification = source
        .find("if !is_divider(metadata)")
        .expect("divider renderer should classify the component");
    let visual = source
        .find("let visual = DividerVisual::resolve")
        .expect("divider renderer should resolve the visual model");

    assert!(
        classification < visual,
        "non-divider nodes should exit before visual resolution"
    );
    for required_hook in [
        "EditorDesignTokens",
        "UiRenderPainterStateSource",
        "style_overrides",
        "horizontal_divider_frame",
        "vertical_divider_frame",
    ] {
        assert!(
            source.contains(required_hook),
            "divider renderer should retain {required_hook}"
        );
    }
}

#[test]
fn render_extract_expands_middle_horizontal_divider_with_relative_insets() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.divider"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_divider(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 200.0, 20.0),
        "orientation = \"horizontal\"\nvariant = \"middle\"",
        "",
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let line = divider_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(16.0, 20.0, 184.0, 1.0),
    );
    assert_eq!(line.style.background_color.as_deref(), Some("#262d33"));
    assert_eq!(line.style.painter_family, UiPainterFamily::Generic);
    assert_eq!(line.style.painter_state, UiPainterResolvedState::Normal);
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Quad
            })
            .count(),
        1,
        "the custom divider must suppress its generic owner surface"
    );
}

#[test]
fn divider_supports_vertical_inset_and_disabled_roles() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.divider.vertical"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 128.0))
            .with_state_flags(visible_state()),
    );
    insert_divider(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 42.0, 40.0, 40.0),
        "orientation = \"vertical\"\nvariant = \"inset\"",
        "",
        disabled_state(),
    );

    surface.rebuild();

    let line = divider_quad(
        &surface.render_extract.list.commands,
        UiNodeId::new(2),
        UiFrame::new(28.0, 50.0, 1.0, 32.0),
    );
    assert_eq!(line.style.background_color.as_deref(), Some("#2c3237"));
    assert_eq!(line.style.painter_state, UiPainterResolvedState::Disabled);
}

#[test]
fn divider_accepts_valid_overrides_and_rejects_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.divider.overrides"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 112.0))
            .with_state_flags(visible_state()),
    );
    insert_divider(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 200.0, 20.0),
        "variant = \"middle\"",
        r##"
separator_color = "#4c9dab"
thickness = 2.0
inset = 12.0
"##,
        visible_state(),
    );
    insert_divider(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 42.0, 200.0, 20.0),
        "variant = \"middle\"",
        r##"
separator_color = "invalid"
thickness = 0.0
inset = -1.0
"##,
        visible_state(),
    );

    surface.rebuild();

    assert_eq!(
        divider_quad(
            &surface.render_extract.list.commands,
            UiNodeId::new(2),
            UiFrame::new(20.0, 19.0, 176.0, 2.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#4c9dab")
    );
    assert_eq!(
        divider_quad(
            &surface.render_extract.list.commands,
            UiNodeId::new(3),
            UiFrame::new(16.0, 52.0, 184.0, 1.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#262d33")
    );
}

fn insert_divider(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    attributes: &str,
    style_overrides: &str,
    state_flags: UiStateFlags,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new("root/divider"))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Divider".to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    style_overrides: toml::from_str(style_overrides).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn divider_quad(
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
        })
        .expect("expected divider quad")
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn disabled_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: false,
        ..UiStateFlags::default()
    }
}

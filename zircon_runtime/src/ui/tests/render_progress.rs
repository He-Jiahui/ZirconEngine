use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPixelSnappingPolicy},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn progress_renderer_classifies_before_visual_resolution_and_uses_shared_tokens() {
    let source = include_str!("../surface/render/progress.rs");
    let classification = source
        .find("if !is_linear_progress(metadata)")
        .expect("progress renderer should classify the component");
    let visual = source
        .find("let visual = ProgressVisual::resolve")
        .expect("progress renderer should resolve the visual model");

    assert!(
        classification < visual,
        "non-linear progress nodes should exit before visual resolution"
    );
    for required_hook in [
        "EditorDesignTokens",
        "EditorTypographyTokens",
        "UiRenderPainterStateSource",
        "style_overrides",
        "text_command",
    ] {
        assert!(
            source.contains(required_hook),
            "progress renderer should retain {required_hook}"
        );
    }
}

#[test]
fn render_extract_expands_linear_progress_with_tokenized_label_and_relative_fill() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.progress"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_progress(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 200.0, 32.0),
        r##"
variant = "linear"
value = 50.0
min = 0.0
max = 100.0
show_label = true
label_text = "Importing"
"##,
        "",
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let track = progress_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(16.0, 34.0, 184.0, 4.0),
    );
    assert_eq!(track.style.background_color.as_deref(), Some("#0f1316"));
    assert_eq!(track.style.border_color.as_deref(), Some("#262d33"));
    assert_eq!(track.style.painter_family, UiPainterFamily::Generic);
    assert_eq!(track.style.painter_state, UiPainterResolvedState::Normal);
    let fill = progress_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(16.0, 34.0, 92.0, 4.0),
    );
    assert_eq!(fill.style.background_color.as_deref(), Some("#3cc7d6"));
    let label = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Text
                && command.text.as_deref() == Some("Importing")
        })
        .expect("progress should render its explicit label through the shared text pipeline");
    assert_eq!(label.frame, UiFrame::new(16.0, 14.0, 184.0, 16.0));
    assert_eq!(label.style.foreground_color.as_deref(), Some("#e8ecee"));
    assert!(label.text_layout.is_some());
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Importing")
            })
            .count(),
        1,
        "owner text must be suppressed once the progress renderer owns the label"
    );
}

#[test]
fn progress_preserves_fractional_fill_geometry_when_snapping_is_disabled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.progress.fractional"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/progress"))
                .with_frame(UiFrame::new(8.25, 10.5, 200.5, 20.25))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Progress".to_string(),
                    pixel_snapping: UiPixelSnappingPolicy::Disabled,
                    attributes: toml::from_str("value_percent = 0.375").unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let fill = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.style.background_color.as_deref() == Some("#3cc7d6")
        })
        .expect("fractional progress should emit its fill command");
    assert!(frame_approx(fill.frame, 16.25, 18.625, 69.1875, 4.0));
    assert_eq!(fill.style.pixel_snapping, UiPixelSnappingPolicy::Disabled);
}

#[test]
fn progress_uses_data_range_for_fill_and_disabled_token_roles() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.progress.disabled"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 112.0))
            .with_state_flags(visible_state()),
    );
    insert_progress(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 12.0, 200.0, 20.0),
        "value = 25.0\nmin = 0.0\nmax = 100.0",
        "",
        visible_state(),
    );
    insert_progress(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 52.0, 200.0, 20.0),
        "value_percent = 0.25",
        "",
        disabled_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert_eq!(
        progress_quad(
            commands,
            UiNodeId::new(2),
            UiFrame::new(16.0, 20.0, 46.0, 4.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#3cc7d6")
    );
    let disabled_track = progress_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(16.0, 60.0, 184.0, 4.0),
    );
    assert_eq!(
        disabled_track.style.background_color.as_deref(),
        Some("#22272b")
    );
    assert_eq!(
        disabled_track.style.painter_state,
        UiPainterResolvedState::Disabled
    );
    assert_eq!(
        progress_quad(
            commands,
            UiNodeId::new(3),
            UiFrame::new(16.0, 60.0, 46.0, 4.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#656f76")
    );
}

#[test]
fn progress_accepts_valid_visual_overrides_and_rejects_invalid_metrics_or_colors() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.progress.overrides"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 112.0))
            .with_state_flags(visible_state()),
    );
    insert_progress(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 200.0, 20.0),
        "value_percent = 0.25",
        r##"
track_color = "#254c5a"
fill_color = "#4c9dab"
track_height = 6.0
"##,
        visible_state(),
    );
    insert_progress(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 42.0, 200.0, 20.0),
        "value_percent = 0.25",
        r##"
track_color = "invalid"
fill_color = "#12"
track_height = 0.0
horizontal_inset = -1.0
font_size = 0.0
line_height_ratio = 0.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert_eq!(
        progress_quad(
            commands,
            UiNodeId::new(2),
            UiFrame::new(16.0, 17.0, 184.0, 6.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#254c5a")
    );
    assert_eq!(
        progress_quad(
            commands,
            UiNodeId::new(2),
            UiFrame::new(16.0, 17.0, 46.0, 6.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#4c9dab")
    );
    assert_eq!(
        progress_quad(
            commands,
            UiNodeId::new(3),
            UiFrame::new(16.0, 50.0, 184.0, 4.0),
        )
        .style
        .background_color
        .as_deref(),
        Some("#0f1316")
    );
}

fn insert_progress(
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
            UiTreeNode::new(node_id, UiNodePath::new("root/progress"))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Progress".to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    style_overrides: toml::from_str(style_overrides).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn progress_quad(
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
        .expect("expected progress quad")
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

fn frame_approx(actual: UiFrame, x: f32, y: f32, width: f32, height: f32) -> bool {
    (actual.x - x).abs() < 0.000_1
        && (actual.y - y).abs() < 0.000_1
        && (actual.width - width).abs() < 0.000_1
        && (actual.height - height).abs() < 0.000_1
}

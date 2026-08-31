use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPixelSnappingPolicy},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn skeleton_renderer_classifies_before_visual_resolution_and_uses_shared_tokens() {
    let source = include_str!("../surface/render/skeleton.rs");
    let classification = source
        .find("if !is_skeleton(metadata)")
        .expect("skeleton renderer should classify the component");
    let visual = source
        .find("let visual = SkeletonVisual::resolve")
        .expect("skeleton renderer should resolve the visual model");

    assert!(
        classification < visual,
        "non-skeleton nodes should exit before visual resolution"
    );
    for required_hook in [
        "EditorDesignTokens",
        "UiRenderPainterStateSource",
        "style_overrides",
        "skeleton_corner_radius",
        "SkeletonVariant",
    ] {
        assert!(
            source.contains(required_hook),
            "skeleton renderer should retain {required_hook}"
        );
    }
}

#[test]
fn render_extract_expands_loading_skeleton_from_the_arranged_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.skeleton"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_skeleton(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 200.0, 20.0),
        "loading = true\nvariant = \"rounded\"",
        "",
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let skeleton = skeleton_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 200.0, 20.0),
    );
    assert_eq!(skeleton.style.background_color.as_deref(), Some("#252b31"));
    assert_eq!(skeleton.style.border_color.as_deref(), Some("#323a41"));
    assert_eq!(skeleton.style.corner_radius, 4.0);
    assert_eq!(skeleton.style.painter_family, UiPainterFamily::Generic);
    assert_eq!(
        skeleton.style.painter_state,
        UiPainterResolvedState::Loading
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Quad
            })
            .count(),
        1,
        "the custom skeleton must suppress its generic owner surface"
    );
}

#[test]
fn skeleton_preserves_fractional_animation_geometry_when_snapping_is_disabled() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.skeleton.fractional"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/skeleton"))
                .with_frame(UiFrame::new(8.25, 10.5, 120.5, 18.25))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Skeleton".to_string(),
                    pixel_snapping: UiPixelSnappingPolicy::Disabled,
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let skeleton = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .expect("fractional skeleton should emit one surface command");
    assert_eq!(skeleton.frame, UiFrame::new(8.25, 10.5, 120.5, 18.25));
    assert_eq!(
        skeleton.style.pixel_snapping,
        UiPixelSnappingPolicy::Disabled
    );
}

#[test]
fn skeleton_uses_disabled_roles_and_rectangular_variant_has_no_radius() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.skeleton.disabled"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_skeleton(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 120.0, 18.0),
        "variant = \"rectangular\"",
        "",
        disabled_state(),
    );

    surface.rebuild();

    let skeleton = skeleton_quad(
        &surface.render_extract.list.commands,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 120.0, 18.0),
    );
    assert_eq!(skeleton.style.background_color.as_deref(), Some("#22272b"));
    assert_eq!(skeleton.style.border_color.as_deref(), Some("#2c3237"));
    assert_eq!(skeleton.style.corner_radius, 0.0);
    assert_eq!(
        skeleton.style.painter_state,
        UiPainterResolvedState::Disabled
    );
}

#[test]
fn skeleton_accepts_valid_visual_overrides_and_rejects_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.skeleton.overrides"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 112.0))
            .with_state_flags(visible_state()),
    );
    insert_skeleton(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 120.0, 18.0),
        "variant = \"rounded\"",
        r##"
background_color = "#254c5a"
highlight_color = "#4c9dab"
corner_radius = 6.0
border_width = 2.0
"##,
        visible_state(),
    );
    insert_skeleton(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 42.0, 120.0, 18.0),
        "variant = \"rounded\"",
        r##"
background_color = "invalid"
highlight_color = "#12"
corner_radius = -1.0
border_width = 0.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let overridden = skeleton_quad(
        &surface.render_extract.list.commands,
        UiNodeId::new(2),
        UiFrame::new(8.0, 10.0, 120.0, 18.0),
    );
    assert_eq!(
        overridden.style.background_color.as_deref(),
        Some("#254c5a")
    );
    assert_eq!(overridden.style.border_color.as_deref(), Some("#4c9dab"));
    assert_eq!(overridden.style.corner_radius, 6.0);
    assert_eq!(overridden.style.border_width, 2.0);

    let fallback = skeleton_quad(
        &surface.render_extract.list.commands,
        UiNodeId::new(3),
        UiFrame::new(8.0, 42.0, 120.0, 18.0),
    );
    assert_eq!(fallback.style.background_color.as_deref(), Some("#252b31"));
    assert_eq!(fallback.style.border_color.as_deref(), Some("#323a41"));
    assert_eq!(fallback.style.corner_radius, 4.0);
    assert_eq!(fallback.style.border_width, 1.0);
}

fn insert_skeleton(
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
            UiTreeNode::new(node_id, UiNodePath::new("root/skeleton"))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Skeleton".to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    style_overrides: toml::from_str(style_overrides).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn skeleton_quad(
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
        .expect("expected skeleton quad")
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

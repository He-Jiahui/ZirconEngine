use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiPoint, UiSize},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

fn rich_widget_metadata(markup: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(&format!(
            r#"
text = "{markup}"
font_size = 16.0
line_height = 20.0
wrap = "None"
rich_text_format = "bbcode_v1"
"#
        ))
        .expect("rich widget metadata"),
        ..UiTemplateNodeMetadata::default()
    }
}

fn interactive_child(node_id: u64) -> UiTreeNode {
    UiTreeNode::new(
        UiNodeId::new(node_id),
        UiNodePath::new(format!("root/widget-{node_id}")),
    )
    .with_state_flags(UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        ..UiStateFlags::default()
    })
    .with_template_metadata(UiTemplateNodeMetadata {
        component: "Panel".to_string(),
        style_overrides: toml::from_str(
            r##"
background_color = "#d94f4f"
"##,
        )
        .expect("widget child style"),
        ..UiTemplateNodeMetadata::default()
    })
}

fn widget_surface(markup: &str) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.inline-widget"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_template_metadata(rich_widget_metadata(markup)),
    );
    surface
        .tree
        .insert_child(UiNodeId::new(1), interactive_child(2))
        .expect("direct inline widget child");
    surface
}

#[test]
fn rich_inline_widget_arranges_and_hits_the_real_direct_child() {
    let mut surface = widget_surface("before[widget=2|24x16]after");

    surface
        .compute_layout(UiSize::new(240.0, 60.0))
        .expect("inline widget layout");

    let frame = surface
        .tree
        .node(UiNodeId::new(2))
        .expect("inline child")
        .layout_cache
        .frame;
    assert!((frame.width - 24.0).abs() < 0.01);
    assert!((frame.height - 16.0).abs() < 0.01);
    assert!(frame.x > 0.0, "the prefix advance must offset the child");
    assert!(frame.y >= 0.0 && frame.bottom() <= 60.0);

    let child_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .expect("the real child reaches normal render extraction");
    assert_eq!(child_command.kind, UiRenderCommandKind::Quad);
    assert_eq!(child_command.frame, frame);
    assert_eq!(
        surface
            .hit_test(UiPoint::new(
                frame.x + frame.width * 0.5,
                frame.y + frame.height * 0.5,
            ))
            .top_hit,
        Some(UiNodeId::new(2)),
    );
}

#[test]
fn duplicate_inline_widget_binding_hides_the_child_geometry() {
    let mut surface = widget_surface("[widget=2|24x16][widget=2|24x16]");

    surface
        .compute_layout(UiSize::new(240.0, 60.0))
        .expect("duplicate inline widget layout fails closed");

    let frame = surface
        .tree
        .node(UiNodeId::new(2))
        .expect("duplicate inline child")
        .layout_cache
        .frame;
    assert_eq!(frame, Default::default());
    assert_eq!(surface.hit_test(UiPoint::new(1.0, 1.0)).top_hit, None);
}

#[test]
fn missing_inline_widget_binding_does_not_leave_an_unbound_child_visible() {
    let mut surface = widget_surface("[widget=99|24x16]");

    surface
        .compute_layout(UiSize::new(240.0, 60.0))
        .expect("missing inline widget binding fails closed");

    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .expect("unbound direct child")
            .layout_cache
            .frame,
        Default::default(),
    );
    assert_eq!(surface.hit_test(UiPoint::new(1.0, 1.0)).top_hit, None);
}

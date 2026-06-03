use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommandKind, UiVisualAssetRef},
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

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_uses_component_state_store_for_shared_painter_priority() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.painter_state"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 420.0, 260.0))
            .with_state_flags(visible_state()),
    );

    let button_id = UiNodeId::new(2);
    let text_field_id = UiNodeId::new(3);
    let dropdown_id = UiNodeId::new(4);
    let checkbox_id = UiNodeId::new(5);
    let slider_id = UiNodeId::new(6);
    let loading_button_id = UiNodeId::new(7);
    insert_control(
        &mut surface,
        button_id,
        "Button",
        UiFrame::new(12.0, 12.0, 120.0, 30.0),
        r##"
text = "Run"
button_color = "primary"
"##,
    );
    insert_control(
        &mut surface,
        text_field_id,
        "InputField",
        UiFrame::new(12.0, 54.0, 164.0, 28.0),
        r##"
value = "Scene"
"##,
    );
    insert_control(
        &mut surface,
        dropdown_id,
        "Dropdown",
        UiFrame::new(12.0, 96.0, 180.0, 34.0),
        r##"
value_text = "Surface"
"##,
    );
    insert_control(
        &mut surface,
        checkbox_id,
        "Checkbox",
        UiFrame::new(220.0, 12.0, 144.0, 28.0),
        r##"
text = "Snap"
"##,
    );
    insert_control(
        &mut surface,
        slider_id,
        "RangeField",
        UiFrame::new(220.0, 54.0, 170.0, 30.0),
        r##"
value_percent = 0.5
"##,
    );
    insert_control(
        &mut surface,
        loading_button_id,
        "Button",
        UiFrame::new(220.0, 96.0, 120.0, 30.0),
        r##"
text = "Saving"
button_color = "secondary"
"##,
    );

    assert!(surface.component_states.set_hovered(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert!(surface.component_states.set_focused(text_field_id, true));
    surface
        .mark_component_state_render_dirty(text_field_id)
        .unwrap();
    assert!(surface.component_states.set_popup_open(dropdown_id, true));
    surface
        .mark_component_state_render_dirty(dropdown_id)
        .unwrap();
    assert!(surface.component_states.set_selected(checkbox_id, true));
    surface
        .mark_component_state_render_dirty(checkbox_id)
        .unwrap();
    assert!(surface
        .component_states
        .set_active_drag_target(slider_id, true));
    surface
        .mark_component_state_render_dirty(slider_id)
        .unwrap();
    assert!(surface
        .component_states
        .set_loading(loading_button_id, true));
    surface
        .mark_component_state_render_dirty(loading_button_id)
        .unwrap();

    surface.rebuild();

    let button = component_surface(
        &surface.render_extract.list.commands,
        button_id,
        UiPainterFamily::Button,
    );
    assert_eq!(button.style.painter_state, UiPainterResolvedState::Hovered);
    assert_eq!(button.style.background_color.as_deref(), Some("#43ccd8"));

    let text_field = component_surface(
        &surface.render_extract.list.commands,
        text_field_id,
        UiPainterFamily::TextField,
    );
    assert_eq!(
        text_field.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(text_field.style.border_color.as_deref(), Some("#35c7d0"));

    let dropdown = component_surface(
        &surface.render_extract.list.commands,
        dropdown_id,
        UiPainterFamily::Dropdown,
    );
    assert_eq!(dropdown.style.painter_state, UiPainterResolvedState::Open);
    assert_eq!(dropdown.style.background_color.as_deref(), Some("#16282d"));

    let checkbox = component_surface(
        &surface.render_extract.list.commands,
        checkbox_id,
        UiPainterFamily::Checkbox,
    );
    assert_eq!(
        checkbox.style.painter_state,
        UiPainterResolvedState::Selected
    );
    assert_eq!(checkbox.style.background_color.as_deref(), Some("#209fa8"));

    let slider = component_surface(
        &surface.render_extract.list.commands,
        slider_id,
        UiPainterFamily::Slider,
    );
    assert_eq!(
        slider.style.painter_state,
        UiPainterResolvedState::DropHovered
    );
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.node_id == slider_id
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::DropHovered
            && command.style.background_color.as_deref() == Some("#35c7d03a")
    }));

    let loading_button = component_surface(
        &surface.render_extract.list.commands,
        loading_button_id,
        UiPainterFamily::Button,
    );
    assert_eq!(
        loading_button.style.painter_state,
        UiPainterResolvedState::Loading
    );
}

fn insert_control(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    frame: UiFrame,
    attributes: &str,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new(format!("root/{component}")))
                .with_frame(frame)
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn component_surface(
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
        })
        .unwrap()
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

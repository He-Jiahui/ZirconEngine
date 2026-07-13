use super::super::super::template_nodes::push_template_node_commands;
use super::support::positioned_button_node;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

#[test]
fn workbench_toolbar_icon_and_label_keep_starship_gap_without_overlap() {
    let mut node = positioned_button_node(
        "WorkbenchModuleCompile",
        "Compile",
        "filled",
        8.0,
        6.0,
        104.0,
        30.0,
    );
    node.icon_name = "zircon_editor_shell/toolbar/compile.svg".into();
    node.action_id = "workbench.module.compile".into();
    node.layout_content_offset_x = 4.0;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 124.0,
        height: 44.0,
    };
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &origin, None, 0);

    let icon = commands
        .iter()
        .find(|command| command.image_pixels.is_some())
        .expect("toolbar command should paint a leading icon inside the button");
    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Compile"))
        .expect("toolbar command should paint its full authored label");
    let runtime_width = measure_runtime_text_width("Compile", text.font_size);

    let icon_label_gap = text.frame.x - (icon.frame.x + icon.frame.width);
    assert!(
        (icon_label_gap - node.layout_content_offset_x).abs() <= 0.01,
        "native Button painting must honor the authored Starship icon-label gap: authored={}, actual={icon_label_gap}, icon={:?}, text={:?}",
        node.layout_content_offset_x,
        icon.frame,
        text.frame,
    );
    assert!(
        text.frame.width >= runtime_width,
        "Compile should retain its full Runtime Text width: frame={}, runtime={runtime_width}",
        text.frame.width,
    );
    assert!(
        text.frame.x + runtime_width <= node.frame.x + node.frame.width - 8.0 + 0.5,
        "label ink must remain inside the authored Starship right padding"
    );
}

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
};

use super::render_commands_with_refs;

#[test]
fn command_refs_count_within_each_runtime_owner() {
    let commands = [command(7), command(7), command(9)];

    let refs = render_commands_with_refs(&commands)
        .map(|(_, command_ref)| {
            command_ref.map(|command_ref| (command_ref.node_id, command_ref.node_command_index))
        })
        .collect::<Vec<_>>();

    assert_eq!(
        refs,
        vec![
            Some((UiNodeId::new(7), 0)),
            Some((UiNodeId::new(7), 1)),
            Some((UiNodeId::new(9), 0)),
        ]
    );
}

fn command(node_id: u64) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::default(),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

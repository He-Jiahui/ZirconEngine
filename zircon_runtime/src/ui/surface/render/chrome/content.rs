use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{icon_command, separator_command, surface_command, text_command},
    metadata::{ChromeKind, chrome_icon, chrome_label},
    metrics::{ChromeMetrics, icon_frame, separator_edge, separator_frame, text_frame},
    state::ChromeRenderState,
    style::{icon_color, text_color},
};

pub(super) fn chrome_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    kind: ChromeKind,
    state: &ChromeRenderState,
    metrics: ChromeMetrics,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, metadata, kind, state, opacity,
    )];

    if let Some(edge) = separator_edge(metadata, kind) {
        commands.push(separator_command(
            node_id,
            separator_frame(frame, edge, metrics.separator_thickness),
            clip_frame,
            z_index.saturating_add(1),
            metadata,
            state,
            opacity,
        ));
    }

    let label = chrome_label(metadata);
    let icon = chrome_icon(metadata);
    let has_icon = icon.is_some();
    if let Some(icon) = icon {
        commands.push(icon_command(
            node_id,
            icon_frame(frame, label.is_some(), metrics),
            clip_frame,
            z_index.saturating_add(2),
            icon,
            icon_color(metadata, state),
            state,
            opacity,
        ));
    }
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            text_frame(frame, has_icon, metrics),
            clip_frame,
            z_index.saturating_add(2),
            label,
            text_color(metadata, state),
            state,
            metrics,
            opacity,
        ));
    }

    commands
}

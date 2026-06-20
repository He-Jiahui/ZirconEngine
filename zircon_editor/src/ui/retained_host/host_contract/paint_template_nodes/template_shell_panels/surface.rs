use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_chrome_style;
use super::identity::ShellPanelKind;
use super::separators::push_shell_panel_separators;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shell_panel_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    kind: ShellPanelKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = select_workbench_chrome_style(node, kind);
    if let Some(fill) = style.fill {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(fill),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    push_shell_panel_separators(commands, kind, &style, rect, clip, order + 1, opacity);
}

//! 把一个已经投影的候选组合为行绘制命令；selected 与 recent/special 只影响呈现。
//! label 是主标题，description 可包含快捷键或说明；执行操作由 options 的 id 对应交互链完成。

use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::{row_label_order, row_match_indicator_order};
use super::detail::push_command_row_detail;
use super::indicator::push_command_row_match_indicator;
use super::label::push_command_row_label;
use super::style::command_row_style;
use super::surface::push_command_row_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = command_row_style(option);
    push_command_row_surface(commands, row_rect, clip, order, style, opacity);
    push_command_row_match_indicator(
        commands,
        option,
        row_rect,
        clip,
        row_match_indicator_order(order),
        opacity,
    );
    push_command_row_label(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.label.as_str(),
        style.text,
        opacity,
    );
    push_command_row_detail(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.description.as_str(),
        style.shortcut,
        opacity,
    );
}

#[cfg(test)]
#[path = "tests/entry_optimization_batch_gy_editor580_tests.rs"]
mod optimization_batch_gy_editor580_tests;

//! 把一个已经投影的候选组合为行绘制命令；selected 与 recent/special 只影响呈现。
//! label 是主标题，description 可包含快捷键或说明；执行操作由 options 的 id 对应交互链完成。

use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::{row_label_order, row_match_indicator_order};
use super::detail::push_command_row_detail;
use super::indicator::push_command_row_match_indicator;
use super::label::push_command_row_label_with_detail;
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
    push_command_row_label_with_detail(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.label.as_str(),
        !option.description.trim().is_empty(),
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
#[path = "../../rows/tests/entry_perf.rs"]
mod optimization_batch_gy_editor580_tests;

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    #[test]
    fn command_title_and_detail_keep_disjoint_clips_for_long_content() {
        let mut option = TemplatePaneOptionData::default();
        option.label = "A long command title that must stay in its own column".into();
        option.description = "Ctrl+Shift+P".into();
        let row = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 400.0,
            height: 40.0,
        };
        let mut commands = Vec::new();
        push_command_row_commands(&mut commands, &option, &row, &row, 0, 1.0);
        let title = commands
            .iter()
            .find(|c| c.text.as_deref() == Some(option.label.as_str()))
            .expect("title command");
        let detail = commands
            .iter()
            .find(|c| c.text.as_deref() == Some(option.description.as_str()))
            .expect("detail command");
        assert!(title.frame.x + title.frame.width <= detail.frame.x);
        let title_clip = title.clip_frame.as_ref().expect("title has a column clip");
        let detail_clip = detail
            .clip_frame
            .as_ref()
            .expect("detail has a column clip");
        assert!(title_clip.x + title_clip.width <= detail_clip.x);
        option.description = "".into();
        let mut without_detail = Vec::new();
        push_command_row_commands(&mut without_detail, &option, &row, &row, 0, 1.0);
        let full_title = without_detail
            .iter()
            .find(|c| c.text.as_deref() == Some(option.label.as_str()))
            .unwrap();
        assert!(full_title.frame.width > title.frame.width);
    }
}

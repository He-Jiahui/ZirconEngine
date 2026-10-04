//! 文本设置在匹配 setting_key 的编辑焦点存在时显示未提交草稿，其他行继续显示快照中的持久值。
//! 焦点生命周期与提交/取消由窗口输入链管理，绘制器不回写草稿。

use super::super::super::data::{FrameRect, HostTextInputFocusData};
use super::super::super::paint_theme::{HostControlMetrics, HostMaterialPalette};
use super::super::render_commands::HostPaintCommand;
use super::field_control::push_settings_field_control;

#[allow(clippy::too_many_arguments)]
pub(super) fn push_string_control(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    setting_key: &str,
    value_text: &str,
    text_input_focus: Option<&HostTextInputFocusData>,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: HostMaterialPalette,
    metrics: HostControlMetrics,
) {
    let focused = text_input_focus.filter(|focus| focus.control_id.as_str() == setting_key);
    let value_text = focused.map_or_else(
        || value_text.to_owned(),
        |focus| focus.value_text.to_string(),
    );
    push_settings_field_control(
        commands,
        rect,
        value_text,
        focused.is_some(),
        clip,
        order,
        opacity,
        palette,
        metrics,
    );
}

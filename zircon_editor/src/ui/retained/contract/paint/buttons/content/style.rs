//! 把统一按钮选择器的交互结果投影到内容；文字与图标共享按压位移，表面外框保持稳定。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::style::button_style;
use super::metrics::content_offset_y;

/// 一次内容编排共享的颜色与位移快照；保持图标和文字的状态反馈一致。
pub(super) struct ButtonContentStyle {
    pub glyph: [u8; 4],
    pub text: [u8; 4],
    pub y_offset: f32,
}

pub(super) fn button_content_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
) -> ButtonContentStyle {
    let style = button_style(node, kind);
    ButtonContentStyle {
        glyph: style.glyph,
        text: style.text,
        y_offset: content_offset_y(style.interaction),
    }
}

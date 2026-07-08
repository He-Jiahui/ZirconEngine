use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::style::button_style;
use super::metrics::content_offset_y;

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

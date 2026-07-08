use super::super::super::super::data::FrameRect;
use super::super::super::style_selector::WorkbenchIconButtonStyle;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_icon_buttons) struct IconButtonContentStyle
{
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_icon_buttons) glyph:
        [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_icon_buttons) y_offset:
        f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_icon_buttons) fn icon_button_content_style(
    style: WorkbenchIconButtonStyle,
) -> IconButtonContentStyle {
    IconButtonContentStyle {
        glyph: style.glyph,
        y_offset: icon_button_content_offset_y(style.state),
    }
}

impl IconButtonContentStyle {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_icon_buttons) fn offset_glyph_rect(
        &self,
        mut rect: FrameRect,
    ) -> FrameRect {
        rect.y += self.y_offset;
        rect
    }
}

fn icon_button_content_offset_y(state: UiPainterResolvedState) -> f32 {
    if state == UiPainterResolvedState::Pressed {
        current_host_metrics().button_pressed_offset_y
    } else {
        0.0
    }
}

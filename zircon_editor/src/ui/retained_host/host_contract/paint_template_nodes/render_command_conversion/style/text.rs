use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextAlign, UiTextDirection};

use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn aligned_text_x(
    frame: &FrameRect,
    text: &str,
    style: &UiResolvedStyle,
) -> f32 {
    let estimated_width = text.chars().count() as f32 * (style.font_size.max(1.0) * 0.5);
    match resolved_text_align(style.text_align, style.text_direction) {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - estimated_width).max(0.0) * 0.5,
        UiTextAlign::Right => frame.x + (frame.width - estimated_width).max(0.0),
        UiTextAlign::Start | UiTextAlign::End => unreachable!("resolved align is physical"),
    }
}

fn resolved_text_align(align: UiTextAlign, direction: UiTextDirection) -> UiTextAlign {
    match align {
        UiTextAlign::Start => match direction {
            UiTextDirection::RightToLeft => UiTextAlign::Right,
            UiTextDirection::Auto | UiTextDirection::LeftToRight | UiTextDirection::Mixed => {
                UiTextAlign::Left
            }
        },
        UiTextAlign::End => match direction {
            UiTextDirection::RightToLeft => UiTextAlign::Left,
            UiTextDirection::Auto | UiTextDirection::LeftToRight | UiTextDirection::Mixed => {
                UiTextAlign::Right
            }
        },
        UiTextAlign::Left | UiTextAlign::Center | UiTextAlign::Right => align,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> FrameRect {
        FrameRect {
            x: 10.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        }
    }

    fn style(text_align: UiTextAlign, text_direction: UiTextDirection) -> UiResolvedStyle {
        UiResolvedStyle {
            text_align,
            text_direction,
            font_size: 10.0,
            ..UiResolvedStyle::default()
        }
    }

    #[test]
    fn aligned_text_x_resolves_logical_start_end_against_text_direction() {
        let frame = frame();
        let left = frame.x;
        let right = frame.x + frame.width - 15.0;

        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::Start, UiTextDirection::LeftToRight)
            ),
            left
        );
        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::End, UiTextDirection::LeftToRight)
            ),
            right
        );
        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::Start, UiTextDirection::RightToLeft)
            ),
            right
        );
        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::End, UiTextDirection::RightToLeft)
            ),
            left
        );
    }
}

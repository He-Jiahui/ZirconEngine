//! 滑块内容层向共享渲染协议投影轨道、拇指和值标签，所有装饰继续归同一个宿主节点。
//! 数值归一化、拖动捕获与键盘步进由其他所有者负责；这里只消费已有几何、角色色和状态。
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiRgbaColor,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
};

use super::{SliderRenderState, SliderVisual};

#[allow(clippy::too_many_arguments)]
pub(super) fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
    border_width: f32,
    corner_radius: f32,
    state: &SliderRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    visual: &SliderVisual,
    state: &SliderRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(foreground)),
            font_size: visual.font_size,
            line_height: visual.line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    let mut value = String::with_capacity(if alpha == u8::MAX { 7 } else { 9 });
    value.push('#');
    push_lower_hex_byte(&mut value, red);
    push_lower_hex_byte(&mut value, green);
    push_lower_hex_byte(&mut value, blue);
    if alpha != u8::MAX {
        push_lower_hex_byte(&mut value, alpha);
    }
    value
}

fn push_lower_hex_byte(output: &mut String, value: u8) {
    const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";
    output.push(char::from(LOWER_HEX[usize::from(value >> 4)]));
    output.push(char::from(LOWER_HEX[usize::from(value & 0x0f)]));
}

#[cfg(test)]
#[path = "tests/commands_optimization_batch_et_tests.rs"]
mod optimization_batch_et_tests;

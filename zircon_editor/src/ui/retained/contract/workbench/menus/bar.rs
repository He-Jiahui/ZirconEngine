use super::super::super::data::{paint_menu_state, FrameRect, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::draw_rect_clipped;
use super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics,
};
use super::geometry::scrolled_menu_frame;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

// 顶栏标签依据菜单状态的滚动量计算绘制框；打开状态仅改变视觉角色，输入路径独立解释同一菜单状态。
pub(in crate::ui::retained_host::host_contract) fn draw_menu_bar_labels(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    let menu_state = paint_menu_state(presentation);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: scene
            .layout
            .status_bar_frame
            .width
            .max(scene.layout.center_band_frame.width),
        height: scene.menu_chrome.top_bar_height_px.max(0.0),
    };
    for row in 0..scene.menu_chrome.menu_frames.row_count() {
        let Some(menu_frame) = scene.menu_chrome.menu_frames.get(row) else {
            continue;
        };
        let Some(menu) = scene.menu_chrome.menus.get(row) else {
            continue;
        };
        let visual = menu_bar_control_visual(menu_state.open_menu_index == row as i32, palette);
        let frame_rect = scrolled_menu_frame(&menu_frame.frame, menu_state.menu_bar_scroll_px);
        if let Some(background) = visual.background {
            draw_rect_clipped(frame, frame_rect.clone(), Some(&clip), background);
        }
        draw_menu_bar_label(
            frame,
            menu.label.as_str(),
            &frame_rect,
            Some(&clip),
            visual.text_color,
            metrics,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MenuBarControlVisual {
    background: Option<[u8; 4]>,
    text_color: [u8; 4],
}

fn menu_bar_control_visual(
    open: bool,
    palette: super::super::super::paint_theme::HostMaterialPalette,
) -> MenuBarControlVisual {
    if open {
        MenuBarControlVisual {
            background: Some(palette.surface_hover),
            text_color: palette.text,
        }
    } else {
        MenuBarControlVisual {
            background: None,
            text_color: palette.text_muted,
        }
    }
}

fn draw_menu_bar_label(
    frame: &mut HostRgbaFrame,
    text: &str,
    menu_frame: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    metrics: HostControlMetrics,
) {
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    let horizontal_inset = (metrics.gap_m - metrics.border_width * 2.0).max(0.0);
    let text_frame = menu_bar_label_frame(menu_frame, horizontal_inset, line_height);
    draw_text_with_size_and_style(
        frame,
        text_frame,
        text,
        clip,
        color,
        metrics.font_body,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

fn menu_bar_label_frame(
    menu_frame: &FrameRect,
    horizontal_inset: f32,
    line_height: f32,
) -> FrameRect {
    let line_height = line_height.min(menu_frame.height.max(1.0));
    FrameRect {
        x: menu_frame.x + horizontal_inset,
        y: menu_frame.y + (menu_frame.height - line_height).max(0.0) * 0.5,
        width: (menu_frame.width - horizontal_inset * 2.0).max(1.0),
        height: line_height,
    }
}

#[cfg(test)]
#[path = "tests/bar.rs"]
mod tests;

//! 选中复选框的勾号走统一 SVG 资产入口；宽高不足时保持已选表面但省略图标。

// BUG: [CR-EDITOR-PAINT-FORMS-0004] 使用固定 PALETTE.accent；主题切换后标记和文字采用当前 palette，勾号颜色仍保持默认主题。
use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;

const CHECKBOX_TICK_ICON: &str = "checkmark";

/// 选中复选框标记足够大时加载 checkmark 资产；图标加载失败会留下已选表面。
pub(super) fn push_checkbox_tick(
    commands: &mut Vec<HostPaintCommand>,
    mark: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if mark.width < 12.0 || mark.height < 12.0 {
        return;
    }
    let color = PALETTE.accent;
    push_icon_asset_pixels(
        commands,
        CHECKBOX_TICK_ICON,
        mark,
        clip,
        order,
        Some(color),
        opacity,
    );
}

#[cfg(test)]
#[path = "tests/tick.rs"]
mod tests;

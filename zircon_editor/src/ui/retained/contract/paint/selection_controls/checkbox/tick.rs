//! 选中复选框的勾号走统一 SVG 资产入口；宽高不足时保持已选表面但省略图标。

// BUG: [CR-EDITOR-PAINT-FORMS-0004] 使用固定 PALETTE.accent；主题切换后标记和文字采用当前 palette，勾号颜色仍保持默认主题。
use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::current_host_palette;
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
    let color = current_host_palette().accent;
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
#[path = "../../checkbox/tests/tick.rs"]
mod tests;

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_from_tokens_for_test,
    };
    use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, style::UiRgbaColor};
    #[test]
    fn checkbox_tick_asset_tint_changes_with_the_scoped_accent() {
        let key = |tokens: &EditorDesignTokens| {
            let _scope = enter_host_paint_theme_scope(
                host_paint_theme_snapshot_from_tokens_for_test(tokens),
            );
            let mark = FrameRect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 16.0,
            };
            let mut commands = Vec::new();
            push_checkbox_tick(&mut commands, &mark, &mark, 0, 1.0);
            commands
                .iter()
                .find_map(|c| c.image_pixels.as_ref())
                .expect("actual checkmark pixels")
                .resource_key
                .to_string()
        };
        let first = EditorDesignTokens::workbench_dark();
        let mut second = EditorDesignTokens::workbench_dark();
        second.palette.accent = UiRgbaColor::from_u8(13, 193, 113, 255);
        assert_ne!(key(&first), key(&second));
    }
}

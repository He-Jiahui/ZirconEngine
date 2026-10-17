use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::current_host_palette;
use super::super::super::super::template_style_color::{
    resolved_style_color, typed_button_variant_background,
};
use super::super::super::state::is_button_disabled;
use super::interaction::interaction_surface_color;
use super::severity::severity_surface_color;
use super::variants::variant_surface_color;

// BUG: [CR-EDITOR-PAINT-CONTROLSTYLE-0001] 设置更新宿主主题后，禁用、严重级别、交互及变体回退仍读固定 PALETTE；
// 同帧文字与 typed 颜色读取当前快照，通用表面/边框会混用新旧主题。证据：settings_window_actions 的主题同步与 surface 命令消费。
/// 供通用表面按校验、交互、显式样式与变体优先级选择填充；透明声明在活跃交互时仍可显示反馈。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if is_button_disabled(node) {
        return current_host_palette().surface_disabled;
    }
    if let Some(color) = severity_surface_color(node) {
        return color;
    }
    if let Some(color) = interaction_surface_color(node) {
        return color;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.background_color.as_ref()) {
        return color;
    }
    if let Some(color) = typed_button_variant_background(node) {
        return color;
    }
    variant_surface_color(node)
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_from_tokens_for_test,
    };
    use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, style::UiRgbaColor};
    #[test]
    fn surface_fallbacks_follow_the_active_theme_snapshot() {
        let mut tokens = EditorDesignTokens::workbench_dark();
        tokens.palette.accent = UiRgbaColor::from_u8(17, 37, 67, 255);
        tokens.palette.error_container = UiRgbaColor::from_u8(93, 33, 73, 255);
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_from_tokens_for_test(&tokens));
        let mut node = TemplatePaneNodeData::default();
        node.surface_variant = "primary".into();
        assert_eq!(surface_color(&node), [17, 37, 67, 255]);
        node.validation_level = "error".into();
        assert_eq!(surface_color(&node), [93, 33, 73, 255]);
    }
}

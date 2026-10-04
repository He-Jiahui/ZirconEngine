//! 主题角色在绘制时读取当前帧快照，显式 RGBA 原样保留；继承或未知角色让调用方继续执行回退策略。

use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use zircon_runtime_interface::ui::style::UiStyleColor;

/// 把已解析的 runtime 样式转换为宿主颜色；Some(透明) 是显式声明，不能当作 None 的继承语义。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolved_style_color(
    color: Option<&UiStyleColor>,
) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Inherit => None,
        UiStyleColor::Role(role) => material_role_color(role),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn material_role_color(
    role: &str,
) -> Option<[u8; 4]> {
    material_role_color_from_host(role, current_host_palette())
}

pub(super) fn material_role_color_from_host(
    role: &str,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    match role {
        "primary" | "accent" | "material.primary" | "material_color_primary" => {
            Some(palette.accent)
        }
        "on_primary" | "material.on_primary" | "material_color_on_primary" => {
            Some(palette.shell_background)
        }
        "surface" | "material.surface" => Some(palette.surface),
        "surface_inset" | "material.surface_inset" => Some(palette.surface_inset),
        "surface_hover" | "material.surface_hover" => Some(palette.surface_hover),
        "surface_pressed" | "material.surface_pressed" => Some(palette.surface_pressed),
        "surface_selected" | "material.surface_selected" => Some(palette.surface_selected),
        "disabled" | "material.disabled" => Some(palette.surface_disabled),
        "border" | "outline" | "material.outline" => Some(palette.border),
        "focus" | "focus_ring" | "material.focus_ring" => Some(palette.focus_ring),
        "text" | "on_surface" | "material.text" | "material.on_surface" => Some(palette.text),
        "text_muted" | "muted" | "material.text_muted" => Some(palette.text_muted),
        "text_disabled" | "material.text_disabled" => Some(palette.text_disabled),
        "warning" | "material.warning" => Some(palette.warning),
        "error" | "danger" | "material.error" => Some(palette.error),
        "success" | "material.success" => Some(palette.success),
        "info" | "material.info" => Some(palette.info),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/roles.rs"]
mod tests;

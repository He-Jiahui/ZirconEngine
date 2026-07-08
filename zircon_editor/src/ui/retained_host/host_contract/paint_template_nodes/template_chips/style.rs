use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchChipPalette {
    pub surface: [u8; 4],
    pub hover_surface: [u8; 4],
    pub pressed_surface: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
    pub border_disabled: [u8; 4],
    pub focus_ring: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_chip_palette(
) -> WorkbenchChipPalette {
    workbench_chip_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_chip_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchChipPalette {
    WorkbenchChipPalette {
        surface: palette.surface,
        hover_surface: palette.surface_hover,
        pressed_surface: palette.surface_pressed,
        surface_disabled: palette.surface_disabled,
        border: palette.border,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
        border_disabled: palette.border_disabled,
        focus_ring: palette.focus_ring,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_surface(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.surface_disabled
    } else if node.pressed || node.popup_open {
        palette.pressed_surface
    } else if node.hovered {
        palette.hover_surface
    } else {
        palette.surface
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.border_disabled
    } else if node.focused || node.pressed || node.popup_open {
        palette.focus_ring
    } else {
        palette.border
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.text_disabled
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        palette.text_muted
    } else {
        palette.text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_glyph_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = workbench_chip_palette();
    if node.disabled {
        palette.text_disabled
    } else if node.pressed || node.popup_open {
        palette.focus_ring
    } else {
        palette.text_muted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::project_host_palette;
    use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
    use zircon_runtime_interface::ui::style::UiRgbaColor;

    #[test]
    fn workbench_chip_palette_projects_from_host_palette() {
        let mut tokens = EditorDesignTokens::workbench_dark();
        tokens.palette.surface_hover = UiRgbaColor::from_u8(42, 53, 60, 255);
        tokens.palette.focus_ring = UiRgbaColor::from_u8(12, 140, 180, 255);
        tokens.palette.text_primary = UiRgbaColor::from_u8(220, 226, 230, 255);

        let palette = workbench_chip_palette_from_host(project_host_palette(&tokens));

        assert_eq!(palette.hover_surface, [42, 53, 60, 255]);
        assert_eq!(palette.focus_ring, [12, 140, 180, 255]);
        assert_eq!(palette.text, [220, 226, 230, 255]);
    }

    #[test]
    fn focused_chip_keeps_normal_surface_and_glyph_with_focus_border() {
        let mut node = TemplatePaneNodeData::default();
        node.focused = true;

        let palette = workbench_chip_palette();

        assert_eq!(chip_surface(&node), palette.surface);
        assert_eq!(chip_border(&node), palette.focus_ring);
        assert_eq!(chip_glyph_color(&node), palette.text_muted);
    }

    #[test]
    fn hovered_chip_still_uses_hover_surface() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;

        assert_eq!(chip_surface(&node), workbench_chip_palette().hover_surface);
    }

    #[test]
    fn pressed_chip_still_uses_pressed_surface_and_focus_glyph() {
        let mut node = TemplatePaneNodeData::default();
        node.pressed = true;

        let palette = workbench_chip_palette();

        assert_eq!(chip_surface(&node), palette.pressed_surface);
        assert_eq!(chip_glyph_color(&node), palette.focus_ring);
    }
}

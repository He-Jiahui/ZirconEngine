use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// 停靠区只取宿主当前主题中的表面角色，保证 shell、页眉和选中标记在同一帧使用同一配色。
pub(super) struct DockChromePalette {
    pub(super) shell: [u8; 4],
    pub(super) document: [u8; 4],
    pub(super) header: [u8; 4],
    pub(super) separator: [u8; 4],
    pub(super) accent: [u8; 4],
}

pub(super) fn current_dock_chrome_palette() -> DockChromePalette {
    dock_chrome_palette(current_host_palette())
}

fn dock_chrome_palette(palette: HostMaterialPalette) -> DockChromePalette {
    DockChromePalette {
        shell: palette.surface_inset,
        document: palette.surface_inset,
        header: palette.popup,
        separator: palette.border,
        accent: palette.accent,
    }
}

#[cfg(test)]
#[path = "tests/palette.rs"]
mod tests;

use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn dock_chrome_palette_projects_all_surface_roles_from_the_current_theme() {
        let mut palette = PALETTE;
        palette.surface_inset = [1, 2, 3, 255];
        palette.popup = [4, 5, 6, 255];
        palette.border = [7, 8, 9, 255];
        palette.accent = [10, 11, 12, 255];

        assert_eq!(
            dock_chrome_palette(palette),
            DockChromePalette {
                shell: [1, 2, 3, 255],
                document: [1, 2, 3, 255],
                header: [4, 5, 6, 255],
                separator: [7, 8, 9, 255],
                accent: [10, 11, 12, 255],
            }
        );
    }
}

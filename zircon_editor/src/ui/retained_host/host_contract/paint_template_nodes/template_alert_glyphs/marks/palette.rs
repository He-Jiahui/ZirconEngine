use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

pub(super) fn alert_glyph_dark() -> [u8; 4] {
    alert_glyph_dark_from_host(current_host_palette())
}

fn alert_glyph_dark_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.shell_background
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_theme::PALETTE;
    use super::*;

    #[test]
    fn alert_glyph_dark_projects_from_the_host_shell_role() {
        let mut host = PALETTE;
        host.shell_background = [8, 18, 18, 255];

        assert_eq!(alert_glyph_dark_from_host(host), [8, 18, 18, 255]);
    }
}

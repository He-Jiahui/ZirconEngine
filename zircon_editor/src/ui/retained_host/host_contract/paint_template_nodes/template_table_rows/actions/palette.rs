use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchTableActionPalette {
    pub data_row_slot_surface: [u8; 4],
    pub header_slot_surface: [u8; 4],
    pub slot_border: [u8; 4],
}

pub(super) fn table_action_palette() -> WorkbenchTableActionPalette {
    table_action_palette_from_host(current_host_palette())
}

fn table_action_palette_from_host(palette: HostMaterialPalette) -> WorkbenchTableActionPalette {
    WorkbenchTableActionPalette {
        data_row_slot_surface: palette.surface_hover,
        header_slot_surface: palette.surface_pressed,
        slot_border: palette.border,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_theme::PALETTE;
    use super::*;

    #[test]
    fn table_action_palette_projects_slot_roles_from_host_palette() {
        let mut host = PALETTE;
        host.surface_hover = [1, 2, 3, 4];
        host.surface_pressed = [5, 6, 7, 8];
        host.border = [9, 10, 11, 12];

        let palette = table_action_palette_from_host(host);

        assert_eq!(palette.data_row_slot_surface, [1, 2, 3, 4]);
        assert_eq!(palette.header_slot_surface, [5, 6, 7, 8]);
        assert_eq!(palette.slot_border, [9, 10, 11, 12]);
    }
}

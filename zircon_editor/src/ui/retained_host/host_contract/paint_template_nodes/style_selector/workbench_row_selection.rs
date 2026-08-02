use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchRowSelectionPalette {
    pub selected_outline: [u8; 4],
}

pub(super) fn workbench_row_selection_palette() -> WorkbenchRowSelectionPalette {
    workbench_row_selection_palette_from_host(current_host_palette())
}

pub(super) fn workbench_row_selection_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchRowSelectionPalette {
    WorkbenchRowSelectionPalette {
        selected_outline: palette.border,
    }
}

pub(super) fn selected_row_outline_color() -> [u8; 4] {
    selected_row_outline_color_from_palette(workbench_row_selection_palette())
}

pub(super) fn selected_row_outline_color_from_palette(
    palette: WorkbenchRowSelectionPalette,
) -> [u8; 4] {
    palette.selected_outline
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn row_selection_palette_projects_selected_outline_from_host_palette() {
        let mut palette = PALETTE;
        palette.border = [10, 11, 12, 255];

        let selection_palette = workbench_row_selection_palette_from_host(palette);

        assert_eq!(
            selected_row_outline_color_from_palette(selection_palette),
            [10, 11, 12, 255]
        );
    }
}

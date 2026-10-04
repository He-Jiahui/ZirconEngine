//! 列表、表格、树和弹出行共同使用当前宿主主题的强调色作为持久选中轮廓；各控件仍独立决定焦点轮廓的优先级。

use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
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
        selected_outline: palette.accent,
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
#[path = "tests/workbench_row_selection.rs"]
mod tests;

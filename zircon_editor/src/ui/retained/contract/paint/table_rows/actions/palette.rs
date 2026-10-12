//! 表头与数据行的操作槽分别投影当前主题表面色，按钮轮廓共享宿主边框色。

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
#[path = "tests/palette.rs"]
mod tests;

//! chip 箭头测量借用 chip 本体的宿主密度投影，避免图标实际大小与标签保留空间分叉。

use super::super::template_chips::{chip_chevron_reserve, chip_chevron_right, chip_chevron_size};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_glyph_chevron_size(
) -> f32 {
    chip_chevron_size()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_glyph_chevron_right(
) -> f32 {
    chip_chevron_right()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_glyph_chevron_reserve(
) -> f32 {
    chip_chevron_reserve()
}

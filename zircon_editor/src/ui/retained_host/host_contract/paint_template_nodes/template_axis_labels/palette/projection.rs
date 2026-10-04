//! 把当前宿主主题文本色投影为变换面板既有的几种弱化层级，同时保留原 alpha；固定比例仅维持基线色差。

use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::model::AxisLabelPalette;
use super::rgb::scaled_rgb;

const AXIS_LABEL_MUTED_SCALE: [f32; 3] = [0.7865854, 0.7816092, 0.7777778];
const AXIS_LABEL_SCALE_MUTED_SCALE: [f32; 3] = [0.7682927, 0.7586207, 0.7555556];
const AXIS_LABEL_LINK_MUTED_SCALE: [f32; 3] = [0.88414633, 0.90229887, 0.9111111];
const AXIS_LABEL_DISABLED_SCALE: [f32; 3] = [0.8118812, 0.8378378, 0.84745765];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_palette(
) -> AxisLabelPalette {
    axis_label_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_palette_from_host(
    palette: HostMaterialPalette,
) -> AxisLabelPalette {
    AxisLabelPalette {
        axis: scaled_rgb(palette.text_muted, AXIS_LABEL_MUTED_SCALE),
        scale_axis: scaled_rgb(palette.text_muted, AXIS_LABEL_SCALE_MUTED_SCALE),
        disabled_axis: scaled_rgb(palette.text_disabled, AXIS_LABEL_DISABLED_SCALE),
        scale_link: scaled_rgb(palette.text_muted, AXIS_LABEL_LINK_MUTED_SCALE),
        disabled_scale_link: scaled_rgb(palette.text_disabled, AXIS_LABEL_DISABLED_SCALE),
    }
}

#[cfg(test)]
#[path = "tests/projection.rs"]
mod tests;

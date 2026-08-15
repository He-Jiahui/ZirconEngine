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
mod tests {
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    use super::*;

    #[test]
    fn axis_label_palette_keeps_audited_default_tones_from_host_palette() {
        let palette = axis_label_palette_from_host(PALETTE);

        assert_eq!(palette.axis, [129, 136, 140, 255]);
        assert_eq!(palette.scale_axis, [126, 132, 136, 255]);
        assert_eq!(palette.scale_link, [145, 157, 164, 255]);
        assert_eq!(palette.disabled_axis, [82, 93, 100, 255]);
        assert_eq!(palette.disabled_scale_link, [82, 93, 100, 255]);
    }

    #[test]
    fn axis_label_palette_projects_from_host_material_palette() {
        let mut host = PALETTE;
        host.text_muted = [200, 150, 100, 180];
        host.text_disabled = [80, 90, 100, 170];

        let palette = axis_label_palette_from_host(host);

        assert_eq!(palette.axis, [157, 117, 78, 180]);
        assert_eq!(palette.scale_axis, [154, 114, 76, 180]);
        assert_eq!(palette.scale_link, [177, 135, 91, 180]);
        assert_eq!(palette.disabled_axis, [65, 75, 85, 170]);
        assert_eq!(palette.disabled_scale_link, [65, 75, 85, 170]);
    }
}

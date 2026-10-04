use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn paper_background_and_outlined_border_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.popup = [10, 11, 12, 255];
    palette.border = [20, 21, 22, 255];

    let node = TemplatePaneNodeData::default();
    assert_eq!(
        paper_background_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );
    assert_eq!(paper_border_color_from_host(&node, false, palette), None);
    assert_eq!(
        paper_border_color_from_host(&node, true, palette),
        Some([20, 21, 22, 255])
    );
}

#[test]
fn paper_declared_colors_override_palette_when_available() {
    let mut palette = PALETTE;
    palette.popup = [10, 11, 12, 255];
    palette.border = [20, 21, 22, 255];
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(40, 41, 42, 255)));

    assert_eq!(
        paper_background_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );
    assert_eq!(
        paper_border_color_from_host(&node, true, palette),
        Some([40, 41, 42, 255])
    );
}

#[test]
fn paper_outlined_border_width_projects_from_shared_host_metrics() {
    let metrics = HostControlMetrics {
        border_width: 1.5,
        ..METRICS
    };

    assert_eq!(
        paper_border_width_from_host(&TemplatePaneNodeData::default(), true, metrics),
        1.5
    );
    assert_eq!(
        paper_border_width_from_host(&TemplatePaneNodeData::default(), false, metrics),
        0.0
    );
}

#[test]
fn paper_declared_border_width_remains_authoritative() {
    let metrics = HostControlMetrics {
        border_width: 1.5,
        ..METRICS
    };
    let node = TemplatePaneNodeData {
        border_width: 2.5,
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(paper_border_width_from_host(&node, true, metrics), 2.5);
    assert_eq!(paper_border_width_from_host(&node, false, metrics), 2.5);
}

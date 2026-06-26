use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{pixel_at, positioned_progress_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_progress_defaults_to_low_emphasis_bar_fill() {
    let bytes = paint_template_nodes_for_test(
        220,
        48,
        model_rc(vec![positioned_progress_node(
            "WorkbenchFeedbackProgress",
            0.64,
            8.0,
            16.0,
            184.0,
            12.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 220, 48, 22), PALETTE.separator_strong);
    assert_eq!(pixel_at(&bytes, 220, 150, 22), PALETTE.surface_inset);
    assert_ne!(pixel_at(&bytes, 220, 48, 22), PALETTE.accent);
}

#[test]
fn generic_material_progress_keeps_accent_fallback() {
    let bytes = paint_template_nodes_for_test(
        220,
        48,
        model_rc(vec![positioned_progress_node(
            "MaterialProgress",
            0.64,
            8.0,
            16.0,
            184.0,
            12.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 220, 48, 22), PALETTE.accent);
}

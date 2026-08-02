use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::autolayout::{WorkbenchChromeMetrics, minimum_document_width_fraction};

pub(super) fn assert_assets_drawer_adaptive_layout(ui: &UiHostWindow, width: u32) {
    let presentation = ui.get_host_presentation();
    let layout = &presentation.host_layout;
    let metrics = WorkbenchChromeMetrics::default();

    assert!(
        layout.left_region_frame.width > metrics.rail_width
            && layout.right_region_frame.width > metrics.rail_width,
        "regular assets drawer capture should retain both expanded side regions"
    );
    assert!(
        layout.document_region_frame.width >= width as f32 * minimum_document_width_fraction(),
        "regular assets drawer capture should preserve the adaptive document reserve: {:?}",
        layout.document_region_frame
    );
}

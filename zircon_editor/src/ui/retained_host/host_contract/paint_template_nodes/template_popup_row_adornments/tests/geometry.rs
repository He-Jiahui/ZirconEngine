use super::popup_row_adornment_rect;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_popup_rows::metrics::workbench_popup_row_metrics;

#[test]
fn popup_row_adornment_requires_a_paintable_row_and_clip_intersection() {
    let metrics = workbench_popup_row_metrics();
    let minimum_width = metrics.adornment_right + metrics.adornment_size;
    let full_row = FrameRect {
        x: 10.0,
        y: 20.0,
        width: minimum_width + 1.0,
        height: metrics.adornment_size,
    };

    let adornment = popup_row_adornment_rect(&full_row, &full_row)
        .expect("a row with the full trailing slot should produce an adornment rect");
    assert!(popup_row_adornment_rect(
        &FrameRect {
            width: minimum_width - 0.1,
            ..full_row.clone()
        },
        &full_row,
    )
    .is_none());
    assert!(popup_row_adornment_rect(
        &FrameRect {
            height: metrics.adornment_size - 0.1,
            ..full_row.clone()
        },
        &full_row,
    )
    .is_none());
    assert!(popup_row_adornment_rect(
        &full_row,
        &FrameRect {
            x: adornment.right() - 0.75,
            y: adornment.y,
            width: 0.75,
            height: adornment.height,
        },
    )
    .is_some());
    assert!(popup_row_adornment_rect(
        &full_row,
        &FrameRect {
            x: adornment.right(),
            y: adornment.y,
            width: 1.0,
            height: adornment.height,
        },
    )
    .is_none());
    assert!(popup_row_adornment_rect(
        &FrameRect {
            x: f32::NAN,
            ..full_row.clone()
        },
        &full_row,
    )
    .is_none());
}

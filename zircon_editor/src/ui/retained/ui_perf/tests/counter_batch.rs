use super::*;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

#[test]
fn batch_maps_every_counter_to_the_active_scenario() {
    let named = named_counter_batch(
        UiPerfScenario::ViewportImage,
        vec![
            (UiPerfCounter::GpuDrawCalls, 3.0),
            (UiPerfCounter::GpuUploadBytes, 512.0),
        ],
    );

    assert_eq!(
        named,
        vec![
            ("ui.viewport_image.gpu_draw_calls", 3.0),
            ("ui.viewport_image.gpu_upload_bytes", 512.0),
        ]
    );
}

use zircon_runtime::rhi::{UiSurfacePresentStats, UiSurfacePresenter};

use super::GpuChromePresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) fn record_present_stats<P: UiSurfacePresenter>(
    presenter: &mut GpuChromePresenter<P>,
    stats: &UiSurfacePresentStats,
    region_present: bool,
) {
    presenter.last_upload_bytes = stats.image_upload_bytes;
    presenter.last_draw_calls = stats.draw_calls;
    record_current_ui_perf_counter(
        UiPerfCounter::GpuUploadBytes,
        stats.image_upload_bytes as f64,
    );
    record_current_ui_perf_counter(UiPerfCounter::GpuDrawCalls, stats.draw_calls as f64);
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVisibleCommands,
        stats.visible_command_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVisibleDrawItems,
        stats.visible_draw_item_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchLayers,
        stats.batch_layer_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchDependencies,
        stats.batch_dependency_count as f64,
    );
    if region_present {
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandPatchCount, 1.0);
    } else {
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0);
    }
}

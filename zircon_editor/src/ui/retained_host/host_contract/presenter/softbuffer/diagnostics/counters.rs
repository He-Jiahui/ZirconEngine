use super::super::super::super::chrome_command_stream::ChromeCommandStream;

#[cfg(feature = "profiling")]
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(in crate::ui::retained_host::host_contract) fn record_chrome_command_stream_counters(
    stream: &ChromeCommandStream,
) {
    #[cfg(feature = "profiling")]
    {
        let counter = if stream.is_full_rebuild() {
            UiPerfCounter::ChromeCommandFullRebuildCount
        } else {
            UiPerfCounter::ChromeCommandPatchCount
        };
        record_current_ui_perf_counter(counter, 1.0);
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = stream;
    }
}

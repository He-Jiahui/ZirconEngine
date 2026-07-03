use super::super::metrics::{workbench_status_metrics, WorkbenchStatusMetrics};

pub(super) fn status_signal_metrics() -> WorkbenchStatusMetrics {
    workbench_status_metrics()
}

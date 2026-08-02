use super::super::metrics::{WorkbenchStatusMetrics, workbench_status_metrics};

pub(super) fn status_signal_metrics() -> WorkbenchStatusMetrics {
    workbench_status_metrics()
}

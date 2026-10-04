//! 诊断指标以代际为单位累积，供渲染后端以快照差值统计帧区间。
use zr_rhi::{
    DeviceGeneration, DeviceId, DiagnosticReadbackAdmission, DiagnosticReadbackReceipt,
    DiagnosticReadbackTerminal,
};

/// Monotonic counters and bounded gauges owned by one WGPU diagnostic generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WgpuDiagnosticReadbackMetricsSnapshot {
    device_id: Option<DeviceId>,
    generation: Option<DeviceGeneration>,
    admitted_request_count: u64,
    admitted_bytes: u64,
    rejected_request_count: u64,
    rejected_bytes: u64,
    submitted_batch_count: u64,
    map_started_batch_count: u64,
    map_completed_batch_count: u64,
    terminal_request_count: u64,
    succeeded_request_count: u64,
    succeeded_bytes: u64,
    drained_delivery_count: u64,
    drained_delivery_bytes: u64,
    dropped_delivery_count: u64,
    active_request_count: usize,
    active_bytes: u64,
    in_flight_batch_count: usize,
    in_flight_request_count: usize,
    in_flight_bytes: u64,
    retained_delivery_count: usize,
    retained_delivery_bytes: u64,
    peak_in_flight_batch_count: usize,
    peak_in_flight_request_count: usize,
    peak_in_flight_bytes: u64,
    peak_retained_delivery_count: usize,
    peak_retained_delivery_bytes: u64,
}

/// One interval derived from snapshots belonging to the same device generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WgpuDiagnosticReadbackMetricsDelta {
    admitted_request_count: u64,
    admitted_bytes: u64,
    rejected_request_count: u64,
    rejected_bytes: u64,
    submitted_batch_count: u64,
    map_started_batch_count: u64,
    map_completed_batch_count: u64,
    terminal_request_count: u64,
    succeeded_request_count: u64,
    succeeded_bytes: u64,
    drained_delivery_count: u64,
    drained_delivery_bytes: u64,
    dropped_delivery_count: u64,
    active_request_count: usize,
    active_bytes: u64,
    in_flight_batch_count: usize,
    in_flight_request_count: usize,
    in_flight_bytes: u64,
    retained_delivery_count: usize,
    retained_delivery_bytes: u64,
    lifetime_peak_in_flight_batch_count: usize,
    lifetime_peak_in_flight_request_count: usize,
    lifetime_peak_in_flight_bytes: u64,
    lifetime_peak_retained_delivery_count: usize,
    lifetime_peak_retained_delivery_bytes: u64,
}

macro_rules! metric_getters {
    ($owner:ty; $($name:ident: $type:ty),+ $(,)?) => {
        impl $owner {
            $(pub const fn $name(self) -> $type { self.$name })+
        }
    };
}

metric_getters!(WgpuDiagnosticReadbackMetricsSnapshot;
    device_id: Option<DeviceId>,
    generation: Option<DeviceGeneration>,
    admitted_request_count: u64,
    admitted_bytes: u64,
    rejected_request_count: u64,
    rejected_bytes: u64,
    submitted_batch_count: u64,
    map_started_batch_count: u64,
    map_completed_batch_count: u64,
    terminal_request_count: u64,
    succeeded_request_count: u64,
    succeeded_bytes: u64,
    drained_delivery_count: u64,
    drained_delivery_bytes: u64,
    dropped_delivery_count: u64,
    active_request_count: usize,
    active_bytes: u64,
    in_flight_batch_count: usize,
    in_flight_request_count: usize,
    in_flight_bytes: u64,
    retained_delivery_count: usize,
    retained_delivery_bytes: u64,
    peak_in_flight_batch_count: usize,
    peak_in_flight_request_count: usize,
    peak_in_flight_bytes: u64,
    peak_retained_delivery_count: usize,
    peak_retained_delivery_bytes: u64,
);

metric_getters!(WgpuDiagnosticReadbackMetricsDelta;
    admitted_request_count: u64,
    admitted_bytes: u64,
    rejected_request_count: u64,
    rejected_bytes: u64,
    submitted_batch_count: u64,
    map_started_batch_count: u64,
    map_completed_batch_count: u64,
    terminal_request_count: u64,
    succeeded_request_count: u64,
    succeeded_bytes: u64,
    drained_delivery_count: u64,
    drained_delivery_bytes: u64,
    dropped_delivery_count: u64,
    active_request_count: usize,
    active_bytes: u64,
    in_flight_batch_count: usize,
    in_flight_request_count: usize,
    in_flight_bytes: u64,
    retained_delivery_count: usize,
    retained_delivery_bytes: u64,
    lifetime_peak_in_flight_batch_count: usize,
    lifetime_peak_in_flight_request_count: usize,
    lifetime_peak_in_flight_bytes: u64,
    lifetime_peak_retained_delivery_count: usize,
    lifetime_peak_retained_delivery_bytes: u64,
);

impl WgpuDiagnosticReadbackMetricsSnapshot {
    pub fn delta_since(self, baseline: Self) -> Option<WgpuDiagnosticReadbackMetricsDelta> {
        if !self.has_same_owner_as(baseline) || self.counters_regressed_from(baseline) {
            return None;
        }
        Some(WgpuDiagnosticReadbackMetricsDelta {
            admitted_request_count: self
                .admitted_request_count
                .saturating_sub(baseline.admitted_request_count),
            admitted_bytes: self.admitted_bytes.saturating_sub(baseline.admitted_bytes),
            rejected_request_count: self
                .rejected_request_count
                .saturating_sub(baseline.rejected_request_count),
            rejected_bytes: self.rejected_bytes.saturating_sub(baseline.rejected_bytes),
            submitted_batch_count: self
                .submitted_batch_count
                .saturating_sub(baseline.submitted_batch_count),
            map_started_batch_count: self
                .map_started_batch_count
                .saturating_sub(baseline.map_started_batch_count),
            map_completed_batch_count: self
                .map_completed_batch_count
                .saturating_sub(baseline.map_completed_batch_count),
            terminal_request_count: self
                .terminal_request_count
                .saturating_sub(baseline.terminal_request_count),
            succeeded_request_count: self
                .succeeded_request_count
                .saturating_sub(baseline.succeeded_request_count),
            succeeded_bytes: self
                .succeeded_bytes
                .saturating_sub(baseline.succeeded_bytes),
            drained_delivery_count: self
                .drained_delivery_count
                .saturating_sub(baseline.drained_delivery_count),
            drained_delivery_bytes: self
                .drained_delivery_bytes
                .saturating_sub(baseline.drained_delivery_bytes),
            dropped_delivery_count: self
                .dropped_delivery_count
                .saturating_sub(baseline.dropped_delivery_count),
            active_request_count: self.active_request_count,
            active_bytes: self.active_bytes,
            in_flight_batch_count: self.in_flight_batch_count,
            in_flight_request_count: self.in_flight_request_count,
            in_flight_bytes: self.in_flight_bytes,
            retained_delivery_count: self.retained_delivery_count,
            retained_delivery_bytes: self.retained_delivery_bytes,
            lifetime_peak_in_flight_batch_count: self.peak_in_flight_batch_count,
            lifetime_peak_in_flight_request_count: self.peak_in_flight_request_count,
            lifetime_peak_in_flight_bytes: self.peak_in_flight_bytes,
            lifetime_peak_retained_delivery_count: self.peak_retained_delivery_count,
            lifetime_peak_retained_delivery_bytes: self.peak_retained_delivery_bytes,
        })
    }

    fn has_same_owner_as(self, baseline: Self) -> bool {
        self.device_id.is_some()
            && self.device_id == baseline.device_id
            && self.generation == baseline.generation
    }

    fn counters_regressed_from(self, baseline: Self) -> bool {
        self.admitted_request_count < baseline.admitted_request_count
            || self.admitted_bytes < baseline.admitted_bytes
            || self.rejected_request_count < baseline.rejected_request_count
            || self.rejected_bytes < baseline.rejected_bytes
            || self.submitted_batch_count < baseline.submitted_batch_count
            || self.map_started_batch_count < baseline.map_started_batch_count
            || self.map_completed_batch_count < baseline.map_completed_batch_count
            || self.terminal_request_count < baseline.terminal_request_count
            || self.succeeded_request_count < baseline.succeeded_request_count
            || self.succeeded_bytes < baseline.succeeded_bytes
            || self.drained_delivery_count < baseline.drained_delivery_count
            || self.drained_delivery_bytes < baseline.drained_delivery_bytes
            || self.dropped_delivery_count < baseline.dropped_delivery_count
            || self.peak_in_flight_batch_count < baseline.peak_in_flight_batch_count
            || self.peak_in_flight_request_count < baseline.peak_in_flight_request_count
            || self.peak_in_flight_bytes < baseline.peak_in_flight_bytes
            || self.peak_retained_delivery_count < baseline.peak_retained_delivery_count
            || self.peak_retained_delivery_bytes < baseline.peak_retained_delivery_bytes
    }
}

pub(super) struct WgpuDiagnosticReadbackMetrics {
    snapshot: WgpuDiagnosticReadbackMetricsSnapshot,
}

impl WgpuDiagnosticReadbackMetrics {
    pub(super) fn new(device_id: DeviceId, generation: DeviceGeneration) -> Self {
        Self {
            snapshot: WgpuDiagnosticReadbackMetricsSnapshot {
                device_id: Some(device_id),
                generation: Some(generation),
                ..Default::default()
            },
        }
    }

    pub(super) fn snapshot(
        &self,
        retained_delivery_count: usize,
        retained_delivery_bytes: u64,
        dropped_delivery_count: u64,
    ) -> WgpuDiagnosticReadbackMetricsSnapshot {
        WgpuDiagnosticReadbackMetricsSnapshot {
            retained_delivery_count,
            retained_delivery_bytes,
            dropped_delivery_count,
            ..self.snapshot
        }
    }

    pub(super) fn begin_frame(&mut self) {
        self.snapshot.active_request_count = 0;
        self.snapshot.active_bytes = 0;
    }

    pub(super) fn record_admission(
        &mut self,
        admission: DiagnosticReadbackAdmission,
        accounted_bytes: u64,
    ) {
        match admission {
            DiagnosticReadbackAdmission::Admitted(_) => {
                self.snapshot.admitted_request_count =
                    self.snapshot.admitted_request_count.saturating_add(1);
                self.snapshot.admitted_bytes =
                    self.snapshot.admitted_bytes.saturating_add(accounted_bytes);
                self.snapshot.active_request_count =
                    self.snapshot.active_request_count.saturating_add(1);
                self.snapshot.active_bytes =
                    self.snapshot.active_bytes.saturating_add(accounted_bytes);
            }
            DiagnosticReadbackAdmission::Rejected(receipt) => {
                self.snapshot.rejected_request_count =
                    self.snapshot.rejected_request_count.saturating_add(1);
                self.snapshot.rejected_bytes = self
                    .snapshot
                    .rejected_bytes
                    .saturating_add(receipt.byte_len());
            }
        }
    }

    pub(super) fn seal_active_frame(&mut self) {
        self.snapshot.active_request_count = 0;
        self.snapshot.active_bytes = 0;
    }

    pub(super) fn record_submitted_batch(&mut self, request_count: usize, bytes: u64) {
        self.snapshot.submitted_batch_count = self.snapshot.submitted_batch_count.saturating_add(1);
        self.snapshot.in_flight_batch_count = self.snapshot.in_flight_batch_count.saturating_add(1);
        self.snapshot.in_flight_request_count = self
            .snapshot
            .in_flight_request_count
            .saturating_add(request_count);
        self.snapshot.in_flight_bytes = self.snapshot.in_flight_bytes.saturating_add(bytes);
        self.snapshot.peak_in_flight_batch_count = self
            .snapshot
            .peak_in_flight_batch_count
            .max(self.snapshot.in_flight_batch_count);
        self.snapshot.peak_in_flight_request_count = self
            .snapshot
            .peak_in_flight_request_count
            .max(self.snapshot.in_flight_request_count);
        self.snapshot.peak_in_flight_bytes = self
            .snapshot
            .peak_in_flight_bytes
            .max(self.snapshot.in_flight_bytes);
        self.seal_active_frame();
    }

    pub(super) fn release_in_flight_batch(&mut self, request_count: usize, bytes: u64) {
        self.snapshot.in_flight_batch_count = self.snapshot.in_flight_batch_count.saturating_sub(1);
        self.snapshot.in_flight_request_count = self
            .snapshot
            .in_flight_request_count
            .saturating_sub(request_count);
        self.snapshot.in_flight_bytes = self.snapshot.in_flight_bytes.saturating_sub(bytes);
    }

    pub(super) fn clear_in_flight(&mut self) {
        self.snapshot.in_flight_batch_count = 0;
        self.snapshot.in_flight_request_count = 0;
        self.snapshot.in_flight_bytes = 0;
    }

    pub(super) fn record_map_started(&mut self) {
        self.snapshot.map_started_batch_count =
            self.snapshot.map_started_batch_count.saturating_add(1);
    }

    pub(super) fn record_map_completed(&mut self) {
        self.snapshot.map_completed_batch_count =
            self.snapshot.map_completed_batch_count.saturating_add(1);
    }

    pub(super) fn record_terminal(&mut self, receipt: DiagnosticReadbackReceipt) {
        self.snapshot.terminal_request_count =
            self.snapshot.terminal_request_count.saturating_add(1);
        if receipt.terminal() == DiagnosticReadbackTerminal::Succeeded {
            self.snapshot.succeeded_request_count =
                self.snapshot.succeeded_request_count.saturating_add(1);
            self.snapshot.succeeded_bytes = self
                .snapshot
                .succeeded_bytes
                .saturating_add(receipt.byte_len());
        }
    }

    pub(super) fn record_delivery_drained(&mut self, bytes: u64) {
        self.snapshot.drained_delivery_count =
            self.snapshot.drained_delivery_count.saturating_add(1);
        self.snapshot.drained_delivery_bytes =
            self.snapshot.drained_delivery_bytes.saturating_add(bytes);
    }

    pub(super) fn observe_retained_deliveries(&mut self, count: usize, bytes: u64) {
        self.snapshot.peak_retained_delivery_count =
            self.snapshot.peak_retained_delivery_count.max(count);
        self.snapshot.peak_retained_delivery_bytes =
            self.snapshot.peak_retained_delivery_bytes.max(bytes);
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;

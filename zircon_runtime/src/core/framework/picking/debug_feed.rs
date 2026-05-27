use super::{HitTarget, PickingPipelineReport, PointerId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickingDebugFeed {
    pub metrics: Vec<PickingDebugMetric>,
    pub pointers: Vec<PickingDebugPointerRow>,
}

impl PickingDebugFeed {
    pub fn from_report(report: &PickingPipelineReport) -> Self {
        Self {
            metrics: vec![
                PickingDebugMetric::new(PickingDebugMetricKind::Pointers, report.pointer_count),
                PickingDebugMetric::new(PickingDebugMetricKind::Rays, report.ray_count),
                PickingDebugMetric::new(
                    PickingDebugMetricKind::BackendOutputs,
                    report.backend_output_count,
                ),
                PickingDebugMetric::new(PickingDebugMetricKind::RawHits, report.raw_hit_count),
                PickingDebugMetric::new(
                    PickingDebugMetricKind::HoveredHits,
                    report.hovered_hit_count,
                ),
                PickingDebugMetric::new(
                    PickingDebugMetricKind::BlockedPointers,
                    report.blocked_pointer_count,
                ),
            ],
            pointers: report
                .pointers
                .iter()
                .map(PickingDebugPointerRow::from_report)
                .collect(),
        }
    }

    pub fn metric(&self, kind: PickingDebugMetricKind) -> Option<usize> {
        self.metrics
            .iter()
            .find(|metric| metric.kind == kind)
            .map(|metric| metric.value)
    }

    pub fn pointer(&self, pointer: PointerId) -> Option<&PickingDebugPointerRow> {
        self.pointers.iter().find(|row| row.pointer == pointer)
    }

    pub fn blocked_pointers(&self) -> impl Iterator<Item = &PickingDebugPointerRow> {
        self.pointers.iter().filter(|row| row.blocked)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickingDebugMetric {
    pub kind: PickingDebugMetricKind,
    pub value: usize,
}

impl PickingDebugMetric {
    pub const fn new(kind: PickingDebugMetricKind, value: usize) -> Self {
        Self { kind, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickingDebugMetricKind {
    Pointers,
    Rays,
    BackendOutputs,
    RawHits,
    HoveredHits,
    BlockedPointers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickingDebugPointerRow {
    pub pointer: PointerId,
    pub ray_count: usize,
    pub backend_output_count: usize,
    pub raw_hit_count: usize,
    pub sorted_hit_count: usize,
    pub hovered_hit_count: usize,
    pub non_hoverable_hit_count: usize,
    pub blocked: bool,
    pub top_target: Option<HitTarget>,
    pub blocking_target: Option<HitTarget>,
}

impl PickingDebugPointerRow {
    fn from_report(report: &super::PickingPointerPipelineReport) -> Self {
        Self {
            pointer: report.pointer,
            ray_count: report.ray_count,
            backend_output_count: report.backend_output_count,
            raw_hit_count: report.raw_hit_count,
            sorted_hit_count: report.sorted_hit_count,
            hovered_hit_count: report.hovered_hit_count,
            non_hoverable_hit_count: report.non_hoverable_hit_count,
            blocked: report.blocking_target.is_some(),
            top_target: report.top_target,
            blocking_target: report.blocking_target,
        }
    }
}

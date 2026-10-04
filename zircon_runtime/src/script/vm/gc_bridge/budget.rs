//! GC 预算是宿主每帧调度与后端单次切片的共同协议；耗时报告用于诊断，真正的截止时间由宿主墙钟控制。
use std::collections::VecDeque;

use super::super::PluginSlotId;

pub const DEFAULT_VM_GC_MAX_MICROS_PER_FRAME: u64 = 1_000;
pub const VM_GC_DIAGNOSTICS_HISTORY_CAPACITY: usize = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmGcBudget {
    pub max_micros_per_frame: u64,
}

impl Default for VmGcBudget {
    fn default() -> Self {
        Self {
            max_micros_per_frame: DEFAULT_VM_GC_MAX_MICROS_PER_FRAME,
        }
    }
}

impl crate::core::framework::scene::SceneResource for VmGcBudget {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VmGcStepOutcome {
    pub pause_micros: u64,
    pub root_count: u64,
    pub cross_boundary_reference_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmGcSlotStepReport {
    pub slot: PluginSlotId,
    pub budget_micros: u64,
    pub host_elapsed_micros: u64,
    pub outcome: VmGcStepOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmGcStepReport {
    pub frame_index: u64,
    pub budget_micros: u64,
    pub pause_micros: u64,
    pub overrun_micros: u64,
    pub host_elapsed_micros: u64,
    pub host_overrun_micros: u64,
    pub root_count: u64,
    pub cross_boundary_reference_count: u64,
    pub slots: Vec<VmGcSlotStepReport>,
}

impl VmGcStepReport {
    pub(crate) fn from_slots(
        frame_index: u64,
        budget: VmGcBudget,
        host_elapsed_micros: u64,
        slots: Vec<VmGcSlotStepReport>,
    ) -> Self {
        let (pause_micros, root_count, cross_boundary_reference_count) = slots.iter().fold(
            (0_u64, 0_u64, 0_u64),
            |(pause_micros, root_count, cross_boundary_reference_count), slot| {
                (
                    pause_micros.saturating_add(slot.outcome.pause_micros),
                    root_count.saturating_add(slot.outcome.root_count),
                    cross_boundary_reference_count
                        .saturating_add(slot.outcome.cross_boundary_reference_count),
                )
            },
        );
        Self {
            frame_index,
            budget_micros: budget.max_micros_per_frame,
            pause_micros,
            overrun_micros: pause_micros.saturating_sub(budget.max_micros_per_frame),
            host_elapsed_micros,
            host_overrun_micros: host_elapsed_micros.saturating_sub(budget.max_micros_per_frame),
            root_count,
            cross_boundary_reference_count,
            slots,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmGcDiagnostics {
    capacity: usize,
    history: VecDeque<VmGcStepReport>,
}

impl VmGcDiagnostics {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            history: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, report: VmGcStepReport) {
        if self.capacity == 0 {
            return;
        }
        if self.history.len() == self.capacity {
            self.history.pop_front();
        }
        self.history.push_back(report);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    pub fn latest(&self) -> Option<&VmGcStepReport> {
        self.history.back()
    }

    pub fn history(&self) -> &VecDeque<VmGcStepReport> {
        &self.history
    }
}

impl Default for VmGcDiagnostics {
    fn default() -> Self {
        Self::with_capacity(VM_GC_DIAGNOSTICS_HISTORY_CAPACITY)
    }
}

impl crate::core::framework::scene::SceneResource for VmGcDiagnostics {}

#[cfg(test)]
#[path = "tests/budget.rs"]
mod tests;

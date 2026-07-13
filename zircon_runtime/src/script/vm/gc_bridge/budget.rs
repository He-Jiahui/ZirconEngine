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
    pub outcome: VmGcStepOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmGcStepReport {
    pub frame_index: u64,
    pub budget_micros: u64,
    pub pause_micros: u64,
    pub overrun_micros: u64,
    pub root_count: u64,
    pub cross_boundary_reference_count: u64,
    pub slots: Vec<VmGcSlotStepReport>,
}

impl VmGcStepReport {
    pub(crate) fn from_slots(
        frame_index: u64,
        budget: VmGcBudget,
        slots: Vec<VmGcSlotStepReport>,
    ) -> Self {
        let pause_micros = slots.iter().fold(0_u64, |total, slot| {
            total.saturating_add(slot.outcome.pause_micros)
        });
        let root_count = slots.iter().fold(0_u64, |total, slot| {
            total.saturating_add(slot.outcome.root_count)
        });
        let cross_boundary_reference_count = slots.iter().fold(0_u64, |total, slot| {
            total.saturating_add(slot.outcome.cross_boundary_reference_count)
        });
        Self {
            frame_index,
            budget_micros: budget.max_micros_per_frame,
            pause_micros,
            overrun_micros: pause_micros.saturating_sub(budget.max_micros_per_frame),
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
mod tests {
    use super::*;

    fn report(frame_index: u64) -> VmGcStepReport {
        VmGcStepReport::from_slots(
            frame_index,
            VmGcBudget {
                max_micros_per_frame: 10,
            },
            vec![VmGcSlotStepReport {
                slot: PluginSlotId::new(frame_index),
                budget_micros: 10,
                outcome: VmGcStepOutcome {
                    pause_micros: frame_index,
                    root_count: frame_index + 1,
                    cross_boundary_reference_count: frame_index + 2,
                },
            }],
        )
    }

    #[test]
    fn default_budget_and_history_capacity_use_shared_contract_constants() {
        assert_eq!(
            VmGcBudget::default().max_micros_per_frame,
            DEFAULT_VM_GC_MAX_MICROS_PER_FRAME
        );
        assert_eq!(
            VmGcDiagnostics::default().capacity(),
            VM_GC_DIAGNOSTICS_HISTORY_CAPACITY
        );
    }

    #[test]
    fn rolling_diagnostics_evict_oldest_report_at_capacity() {
        let mut diagnostics = VmGcDiagnostics::with_capacity(2);
        diagnostics.push(report(1));
        diagnostics.push(report(2));
        diagnostics.push(report(3));

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics.history()[0].frame_index, 2);
        assert_eq!(diagnostics.latest().unwrap().frame_index, 3);
    }

    #[test]
    fn step_report_preserves_real_backend_overrun_and_counts() {
        let report = VmGcStepReport::from_slots(
            4,
            VmGcBudget {
                max_micros_per_frame: 5,
            },
            vec![VmGcSlotStepReport {
                slot: PluginSlotId::new(1),
                budget_micros: 5,
                outcome: VmGcStepOutcome {
                    pause_micros: 8,
                    root_count: 3,
                    cross_boundary_reference_count: 2,
                },
            }],
        );

        assert_eq!(report.pause_micros, 8);
        assert_eq!(report.overrun_micros, 3);
        assert_eq!(report.root_count, 3);
        assert_eq!(report.cross_boundary_reference_count, 2);
    }
}

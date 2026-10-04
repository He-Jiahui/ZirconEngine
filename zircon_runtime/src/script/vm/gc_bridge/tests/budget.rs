use super::*;

fn report(frame_index: u64) -> VmGcStepReport {
    VmGcStepReport::from_slots(
        frame_index,
        VmGcBudget {
            max_micros_per_frame: 10,
        },
        frame_index,
        vec![VmGcSlotStepReport {
            slot: PluginSlotId::new(frame_index),
            budget_micros: 10,
            host_elapsed_micros: frame_index,
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
        9,
        vec![VmGcSlotStepReport {
            slot: PluginSlotId::new(1),
            budget_micros: 5,
            host_elapsed_micros: 9,
            outcome: VmGcStepOutcome {
                pause_micros: 8,
                root_count: 3,
                cross_boundary_reference_count: 2,
            },
        }],
    );

    assert_eq!(report.pause_micros, 8);
    assert_eq!(report.overrun_micros, 3);
    assert_eq!(report.host_elapsed_micros, 9);
    assert_eq!(report.host_overrun_micros, 4);
    assert_eq!(report.root_count, 3);
    assert_eq!(report.cross_boundary_reference_count, 2);
}

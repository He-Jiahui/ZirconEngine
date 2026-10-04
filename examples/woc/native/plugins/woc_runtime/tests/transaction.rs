use std::cell::Cell;
use std::rc::Rc;

use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, EntityRef, FixedTickInput, FixedTickInputRef,
    MovementFrame, MovementInputFlags, OfflineSessionBootstrap, OfflineWeaponSkinAccount,
    WorldSnapshot, FNV1A_OFFSET, OFFLINE_SESSION_BOOTSTRAP_VERSION, STANDARD_OFFLINE_WORLD_SEED,
};
use woc_runtime::{
    BudgetKind, RuntimeRole, RuntimeStatus, TickBudgets, TickUsage, VmTickError, VmTickResult,
    WocProjectVm, WocTickFaultKind, WocTransactionalRuntime,
};

#[derive(Clone, Debug)]
enum Behavior {
    Success(Vec<u8>),
    Malformed,
    Trap,
    Usage(TickUsage),
    Reject,
}

#[derive(Clone, Copy)]
enum RetainedFailure {
    Output,
    Budget,
    Trap,
}

struct RetainedVm {
    state: u64,
    fail_next: Option<RetainedFailure>,
    fail_install_once: bool,
    fail_rollback: bool,
    fail_rollback_after: Option<usize>,
    rollback_attempts: usize,
    live_checkpoints: Rc<Cell<usize>>,
}

struct RetainedCheckpoint {
    state: u64,
    live: Rc<Cell<usize>>,
}

impl Drop for RetainedCheckpoint {
    fn drop(&mut self) {
        self.live.set(self.live.get() - 1);
    }
}

impl RetainedVm {
    fn new(fail_next: Option<RetainedFailure>) -> Self {
        Self {
            state: 0,
            fail_next,
            fail_install_once: false,
            fail_rollback: false,
            fail_rollback_after: None,
            rollback_attempts: 0,
            live_checkpoints: Rc::new(Cell::new(0)),
        }
    }

    fn decode_state(bytes: &[u8]) -> Result<u64, VmTickError> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let bytes: [u8; 8] = bytes
            .try_into()
            .map_err(|_| VmTickError::Transport("invalid retained state".to_string()))?;
        Ok(u64::from_le_bytes(bytes))
    }
}

impl WocProjectVm for RetainedVm {
    type Checkpoint = RetainedCheckpoint;

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, VmTickError> {
        self.live_checkpoints.set(self.live_checkpoints.get() + 1);
        Ok(RetainedCheckpoint {
            state: self.state,
            live: Rc::clone(&self.live_checkpoints),
        })
    }

    fn rollback(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), VmTickError> {
        self.rollback_attempts += 1;
        let delayed_failure = match self.fail_rollback_after {
            Some(successful_attempts) => self.rollback_attempts > successful_attempts,
            None => false,
        };
        if self.fail_rollback || delayed_failure {
            return Err(VmTickError::Trap("injected rollback failure".to_string()));
        }
        self.state = checkpoint.state;
        Ok(())
    }

    fn install_full_snapshot(
        &mut self,
        snapshot: &woc_runtime::CommittedSnapshot,
    ) -> Result<(), VmTickError> {
        self.state = Self::decode_state(&snapshot.state)?;
        if self.fail_install_once {
            self.fail_install_once = false;
            self.state = u64::MAX;
            return Err(VmTickError::Trap(
                "injected snapshot install failure".to_string(),
            ));
        }
        Ok(())
    }

    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        let input = FixedTickInput::decode_payload(input_payload).expect("fixed input");
        self.state = self.state.saturating_add(1);
        match self.fail_next.take() {
            Some(RetainedFailure::Trap) => {
                Err(VmTickError::Trap("injected retained trap".to_string()))
            }
            Some(RetainedFailure::Output) => Ok(VmTickResult {
                output_payload: vec![0xff],
                presentation_payload: b"presentation".to_vec(),
                usage: TickUsage::default(),
            }),
            failure => {
                let state = self.state.to_le_bytes();
                successful_result(
                    &input,
                    &state,
                    if matches!(failure, Some(RetainedFailure::Budget)) {
                        TickUsage {
                            host_calls: u64::MAX,
                            ..TickUsage::default()
                        }
                    } else {
                        TickUsage::default()
                    },
                )
            }
        }
    }
}

struct ScriptedVm {
    behavior: Behavior,
    observed_inputs: Vec<FixedTickInput>,
    retained_state: Vec<u8>,
    calls: usize,
}

impl ScriptedVm {
    fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            observed_inputs: Vec::new(),
            retained_state: Vec::new(),
            calls: 0,
        }
    }
}

impl WocProjectVm for ScriptedVm {
    type Checkpoint = Vec<u8>;

    fn checkpoint(&mut self) -> Result<Vec<u8>, VmTickError> {
        self.calls += 1;
        Ok(self.retained_state.clone())
    }

    fn rollback(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), VmTickError> {
        self.calls += 1;
        self.retained_state = checkpoint.to_vec();
        Ok(())
    }

    fn install_full_snapshot(
        &mut self,
        snapshot: &woc_runtime::CommittedSnapshot,
    ) -> Result<(), VmTickError> {
        self.calls += 1;
        self.retained_state = snapshot.state.clone();
        Ok(())
    }

    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        self.calls += 1;
        let input = FixedTickInput::decode_payload(input_payload)
            .expect("runtime must send a valid fixed tick payload");
        self.observed_inputs.push(input.clone());
        self.retained_state = input.tick.to_le_bytes().to_vec();
        match &self.behavior {
            Behavior::Trap => Err(VmTickError::Trap("injected trap".to_string())),
            Behavior::Reject => Err(VmTickError::RejectedCommand {
                index: 0,
                reason: "injected rejection".to_string(),
            }),
            Behavior::Malformed => Ok(VmTickResult {
                output_payload: vec![0xff],
                presentation_payload: b"malformed presentation".to_vec(),
                usage: TickUsage::default(),
            }),
            Behavior::Success(state) => successful_result(&input, state, TickUsage::default()),
            Behavior::Usage(usage) => successful_result(&input, b"budgeted", *usage),
        }
    }
}

#[test]
fn successful_tick_commits_one_candidate_and_passes_the_committed_base_to_vm() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"next".to_vec())),
        TickBudgets::default(),
    );

    let committed = runtime
        .tick(Vec::<Command>::new())
        .expect("tick must commit");
    assert_eq!(committed.tick, 1);
    assert_eq!(committed.state, b"next");
    assert_eq!(committed.state_digest, fnv1a_bytes(b"next"));
    assert_eq!(committed.presentation_payload, b"presentation");
    assert_eq!(committed.presentation_digest, fnv1a_bytes(b"presentation"));
    assert_eq!(runtime.status(), &RuntimeStatus::Running);

    let input = &runtime.vm().observed_inputs[0];
    assert_eq!(input.tick, 1);
    assert!(input.wall_time_forbidden);
    assert!(input.committed_state.is_empty());
    assert_eq!(input.committed_state_digest, fnv1a_bytes(&[]));
    assert_eq!(input.generation, 0);
    assert!(input.movement_frames.is_empty());

    runtime.tick(vec![]).expect("second tick must commit");
    assert_eq!(runtime.vm().observed_inputs[1].committed_state, b"next");
}

#[test]
fn tick_exhaustion_rejects_before_vm_entry_and_preserves_the_terminal_snapshot() {
    for role in [
        RuntimeRole::Offline,
        RuntimeRole::Server,
        RuntimeRole::Client,
    ] {
        let mut runtime = WocTransactionalRuntime::new(
            role,
            ScriptedVm::new(Behavior::Success(b"last-state".to_vec())),
            TickBudgets::default(),
        );
        let snapshot = woc_runtime::CommittedSnapshot {
            generation: 7,
            tick: u64::MAX - 1,
            state: b"base".to_vec(),
            state_digest: fnv1a_bytes(b"base"),
            event_digest: FNV1A_OFFSET,
            presentation_payload: b"old-presentation".to_vec(),
            presentation_digest: fnv1a_bytes(b"old-presentation"),
        };
        runtime
            .install_full_snapshot(snapshot)
            .expect("install final base");
        runtime
            .tick(vec![])
            .expect("the last representable tick must commit");
        assert_eq!(runtime.committed().tick, u64::MAX);
        assert_eq!(runtime.vm().observed_inputs.len(), 1);
        assert_eq!(runtime.vm().observed_inputs[0].tick, u64::MAX);
        let before = runtime.committed().clone();
        let retained_state = runtime.vm().retained_state.clone();
        let vm_calls = runtime.vm().calls;
        let fault = runtime
            .tick(vec![])
            .expect_err("a terminal tick has no successor");
        assert_eq!(fault.attempted_tick, u64::MAX);
        assert!(matches!(fault.kind, WocTickFaultKind::TickExhausted));
        assert!(fault.rollback_error.is_none());
        assert_eq!(runtime.committed(), &before);
        assert_eq!(runtime.vm().retained_state, retained_state);
        assert_eq!(
            runtime.vm().calls,
            vm_calls,
            "do not checkpoint or enter the VM"
        );
        assert_eq!(runtime.vm().observed_inputs.len(), 1);
        match (role, runtime.status()) {
            (RuntimeRole::Offline, RuntimeStatus::Paused(stored))
            | (RuntimeRole::Server, RuntimeStatus::Faulted(stored))
            | (RuntimeRole::Client, RuntimeStatus::Recovering(stored)) => {
                assert_eq!(stored, &fault);
            }
            _ => panic!("exhaustion must follow the role's terminal policy"),
        }
        let terminal = runtime.status().clone();
        let retry = runtime
            .tick(vec![])
            .expect_err("a repeated tick stays stopped");
        assert!(matches!(retry.kind, WocTickFaultKind::SessionNotRunning));
        assert_eq!(runtime.status(), &terminal);
        assert_eq!(runtime.committed(), &before);
        assert_eq!(runtime.vm().calls, vm_calls);
    }
}

#[test]
fn exhausted_projected_tick_never_invokes_the_projection_consumer() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Client,
        ScriptedVm::new(Behavior::Success(b"next".to_vec())),
        TickBudgets::default(),
    );
    let snapshot = woc_runtime::CommittedSnapshot {
        tick: u64::MAX,
        ..woc_runtime::CommittedSnapshot::default()
    };
    runtime
        .install_full_snapshot(snapshot)
        .expect("install terminal snapshot");
    let before = runtime.committed().clone();
    let calls = runtime.vm().calls;
    let decoded = Cell::new(false);
    let fault = runtime
        .tick_with_projection_and_movement(vec![], vec![], |_| {
            decoded.set(true);
            Ok(())
        })
        .expect_err("projection must not receive another terminal-tick candidate");
    assert!(matches!(fault.kind, WocTickFaultKind::TickExhausted));
    assert!(!decoded.get());
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().calls, calls);
}

#[test]
fn direct_invalid_input_rejection_preserves_server_transaction() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Server,
        ScriptedVm::new(Behavior::Success(b"next".to_vec())),
        TickBudgets::default(),
    );
    let before = runtime.committed().clone();
    let fault = runtime
        .tick(vec![Command {
            command_id: u16::MAX,
            actor: EntityRef {
                id: 1,
                generation: 0,
            },
            sequence: 0,
            payload: Vec::new(),
        }])
        .expect_err("an unknown direct command must be rejected before VM entry");

    assert_eq!(fault.attempted_tick, 1);
    assert!(matches!(fault.kind, WocTickFaultKind::EncodeInput(_)));
    assert!(fault.rollback_error.is_none());
    assert_eq!(runtime.status(), &RuntimeStatus::Running);
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().calls, 0, "rejected input must not call the VM");
    assert!(runtime.vm().observed_inputs.is_empty());

    runtime.tick(vec![]).expect("server remains able to commit");
    assert_eq!(runtime.committed().tick, 1);
}

#[test]
fn movement_frames_enter_the_same_atomic_tick_input_as_commands() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"next".to_vec())),
        TickBudgets::default(),
    );
    let frames = vec![
        MovementFrame {
            actor: EntityRef {
                id: 8,
                generation: 1,
            },
            sequence: 4,
            flags: MovementInputFlags {
                forward: true,
                ..MovementInputFlags::default()
            },
            facing: Some(0.5),
        },
        MovementFrame {
            actor: EntityRef {
                id: 3,
                generation: 2,
            },
            sequence: 9,
            flags: MovementInputFlags {
                strafe_right: true,
                ..MovementInputFlags::default()
            },
            facing: None,
        },
    ];

    runtime
        .tick_with_movement(vec![], frames)
        .expect("movement input must commit through the regular transaction");

    let observed = &runtime.vm().observed_inputs[0].movement_frames;
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0].actor.id, 3);
    assert_eq!(observed[0].sequence, 9);
    assert!(observed[0].flags.strafe_right);
    assert_eq!(observed[1].actor.id, 8);
    assert_eq!(observed[1].sequence, 4);
    assert!(observed[1].flags.forward);
    assert_eq!(observed[1].facing, Some(0.5));
}

#[test]
fn offline_bootstrap_reaches_only_the_first_successful_tick() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"first".to_vec())),
        TickBudgets::default(),
    );
    let bootstrap = OfflineSessionBootstrap {
        launch_version: OFFLINE_SESSION_BOOTSTRAP_VERSION,
        world_seed: STANDARD_OFFLINE_WORLD_SEED,
        player_class: 1,
        player_name: "Vale".to_string(),
        skin_variant: 2,
        weapon_skin_account: OfflineWeaponSkinAccount::default(),
    };
    runtime
        .install_offline_bootstrap(bootstrap.clone())
        .expect("fresh offline runtime accepts bootstrap");

    runtime.tick(vec![]).expect("first tick commits");
    assert_eq!(
        runtime.vm().observed_inputs[0].offline_bootstrap,
        Some(bootstrap)
    );
    assert!(runtime.offline_bootstrap().is_none());

    runtime.tick(vec![]).expect("second tick commits");
    assert!(runtime.vm().observed_inputs[1].offline_bootstrap.is_none());
}

#[test]
fn projected_tick_validates_bulk_presentation_before_committing_authority() {
    let mut invalid = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"next".to_vec())),
        TickBudgets::default(),
    );
    let before = invalid.committed().clone();
    let fault = invalid
        .tick_with_projection(vec![], |_| -> Result<(), String> {
            Err("injected projection rejection".to_string())
        })
        .expect_err("invalid presentation must prevent commit");
    assert!(matches!(
        fault.kind,
        WocTickFaultKind::DecodePresentation(ref reason)
            if reason == "injected projection rejection"
    ));
    assert_eq!(invalid.committed(), &before);
    assert!(matches!(invalid.status(), RuntimeStatus::Paused(_)));

    let mut valid = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"next".to_vec())),
        TickBudgets::default(),
    );
    let (committed, projection) = valid
        .tick_with_projection(vec![], |bytes| Ok::<_, String>(bytes.to_vec()))
        .expect("valid presentation must commit with state");
    assert_eq!(committed.tick, 1);
    assert_eq!(projection, b"presentation");
}

#[test]
fn malformed_output_rolls_back_and_pauses_offline_session() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Malformed),
        TickBudgets::default(),
    );
    let before = runtime.committed().clone();
    let fault = runtime
        .tick(vec![])
        .expect_err("malformed output must fail");
    assert!(matches!(fault.kind, WocTickFaultKind::DecodeOutput(_)));
    assert_eq!(runtime.committed(), &before);
    assert!(matches!(runtime.status(), RuntimeStatus::Paused(_)));
}

#[test]
fn trap_faults_server_and_enters_client_recovery_without_changing_committed_bytes() {
    for (role, expected) in [
        (RuntimeRole::Server, "faulted"),
        (RuntimeRole::Client, "recovering"),
    ] {
        let mut runtime = WocTransactionalRuntime::new(
            role,
            ScriptedVm::new(Behavior::Trap),
            TickBudgets::default(),
        );
        let before = runtime.committed().clone();
        let fault = runtime.tick(vec![]).expect_err("trap must fail");
        assert!(matches!(
            fault.kind,
            WocTickFaultKind::Vm(VmTickError::Trap(_))
        ));
        assert_eq!(runtime.committed(), &before);
        match (expected, runtime.status()) {
            ("faulted", RuntimeStatus::Faulted(_)) => {}
            ("recovering", RuntimeStatus::Recovering(_)) => {}
            _ => panic!("unexpected role failure status"),
        }
    }
}

#[test]
fn every_post_execution_budget_is_checked_before_commit() {
    let budgets = TickBudgets {
        max_execution_micros: 100,
        max_memory_bytes: 200,
        max_host_calls: 3,
        max_gc_micros: 10,
    };
    let cases = [
        (
            TickUsage {
                execution_micros: 101,
                ..TickUsage::default()
            },
            BudgetKind::Execution,
        ),
        (
            TickUsage {
                memory_bytes: 201,
                ..TickUsage::default()
            },
            BudgetKind::Memory,
        ),
        (
            TickUsage {
                host_calls: 4,
                ..TickUsage::default()
            },
            BudgetKind::HostCalls,
        ),
        (
            TickUsage {
                gc_micros: 11,
                ..TickUsage::default()
            },
            BudgetKind::GarbageCollection,
        ),
    ];

    for (usage, expected) in cases {
        let mut runtime = WocTransactionalRuntime::new(
            RuntimeRole::Offline,
            ScriptedVm::new(Behavior::Usage(usage)),
            budgets,
        );
        let before = runtime.committed().clone();
        let fault = runtime.tick(vec![]).expect_err("budget excess must fail");
        assert!(matches!(
            fault.kind,
            WocTickFaultKind::Budget { budget, .. } if budget == expected
        ));
        assert_eq!(runtime.committed(), &before);
    }
}

#[test]
fn opaque_checkpoints_drop_after_commit_abort_install_and_outer_rollback() {
    for failure in [
        None,
        Some(RetainedFailure::Trap),
        Some(RetainedFailure::Output),
    ] {
        let mut runtime = WocTransactionalRuntime::new(
            RuntimeRole::Offline,
            RetainedVm::new(failure),
            TickBudgets::default(),
        );
        let live = Rc::clone(&runtime.vm().live_checkpoints);
        let outer = runtime.checkpoint().expect("outer checkpoint");
        assert_eq!(live.get(), 1);
        let _ = runtime.tick(vec![]);
        assert_eq!(
            live.get(),
            1,
            "candidate checkpoint must drop on every outcome"
        );
        runtime.rollback_checkpoint(outer).expect("outer rollback");
        assert_eq!(live.get(), 0);
        assert_eq!(runtime.vm().state, 0);
        runtime
            .install_full_snapshot(runtime.committed().clone())
            .expect("install");
        assert_eq!(live.get(), 0);
    }
}

#[test]
fn rejected_tick_paths_restore_retained_vm_state_and_replay_the_same_next_tick() {
    for failure in [
        RetainedFailure::Output,
        RetainedFailure::Budget,
        RetainedFailure::Trap,
    ] {
        let mut runtime = WocTransactionalRuntime::new(
            RuntimeRole::Offline,
            RetainedVm::new(Some(failure)),
            TickBudgets::default(),
        );
        let before = runtime.committed().clone();
        runtime.tick(vec![]).expect_err("injected tick must fail");
        assert_eq!(runtime.vm().state, 0);
        runtime
            .install_full_snapshot(before)
            .expect("the committed snapshot recovers the paused transaction");
        runtime.tick(vec![]).expect("replayed tick must commit");

        let mut clean = WocTransactionalRuntime::new(
            RuntimeRole::Offline,
            RetainedVm::new(None),
            TickBudgets::default(),
        );
        clean.tick(vec![]).expect("clean tick must commit");
        assert_eq!(runtime.committed(), clean.committed());
        assert_eq!(runtime.vm().state, clean.vm().state);
    }

    let mut projected = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        RetainedVm::new(None),
        TickBudgets::default(),
    );
    let before = projected.committed().clone();
    projected
        .tick_with_projection(vec![], |_| -> Result<(), String> {
            Err("injected projection failure".to_string())
        })
        .expect_err("projection must reject the candidate");
    assert_eq!(projected.vm().state, 0);
    projected
        .install_full_snapshot(before)
        .expect("snapshot recovers projection failure");
    projected.tick(vec![]).expect("next tick must commit");
    assert_eq!(projected.committed().tick, 1);
    assert_eq!(projected.vm().state, 1);
}

#[test]
fn failed_full_snapshot_install_restores_vm_and_preserves_the_original_fault() {
    let mut vm = RetainedVm::new(None);
    vm.fail_install_once = true;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, vm, TickBudgets::default());
    runtime.tick(vec![]).expect("baseline tick");
    let before = runtime.committed().clone();
    let target_state = 7_u64.to_le_bytes().to_vec();
    let fault = runtime
        .install_full_snapshot(woc_runtime::CommittedSnapshot {
            generation: 3,
            tick: 9,
            state_digest: fnv1a_bytes(&target_state),
            state: target_state,
            event_digest: FNV1A_OFFSET,
            presentation_digest: FNV1A_OFFSET,
            presentation_payload: Vec::new(),
        })
        .expect_err("snapshot adapter failure must reject the install");
    assert!(matches!(
        fault.kind,
        WocTickFaultKind::Vm(VmTickError::Trap(_))
    ));
    assert!(fault.rollback_error.is_none());
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().state, 1);
    assert!(matches!(runtime.status(), RuntimeStatus::Paused(_)));

    runtime
        .install_full_snapshot(before.clone())
        .expect("restoring the committed snapshot must recover the runtime");
    runtime.tick(vec![]).expect("next tick after recovery");
    assert_eq!(runtime.vm().state, 2);
}

#[test]
fn rollback_failure_keeps_the_original_tick_fault_and_never_reports_running() {
    let mut vm = RetainedVm::new(Some(RetainedFailure::Trap));
    vm.fail_rollback = true;
    let mut runtime = WocTransactionalRuntime::new(RuntimeRole::Client, vm, TickBudgets::default());

    let fault = runtime
        .tick(vec![])
        .expect_err("tick and rollback must fail");
    assert!(
        matches!(fault.kind, WocTickFaultKind::Vm(VmTickError::Trap(ref reason)) if reason == "injected retained trap")
    );
    assert!(
        matches!(fault.rollback_error, Some(VmTickError::Trap(ref reason)) if reason == "injected rollback failure")
    );
    assert!(matches!(runtime.status(), RuntimeStatus::Recovering(_)));
}

#[test]
fn failed_external_checkpoint_restore_transitions_client_out_of_running() {
    let mut vm = RetainedVm::new(None);
    vm.fail_rollback = true;
    let mut runtime = WocTransactionalRuntime::new(RuntimeRole::Client, vm, TickBudgets::default());
    let checkpoint = runtime.checkpoint().expect("capture client checkpoint");
    runtime
        .tick(vec![])
        .expect("advance the VM through a real transaction");
    let before = runtime.committed().clone();

    assert!(matches!(
        runtime.rollback_checkpoint(checkpoint),
        Err(VmTickError::Trap(ref reason)) if reason == "injected rollback failure"
    ));
    assert!(matches!(runtime.status(), RuntimeStatus::Recovering(_)));
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().state, 1);
}

#[test]
fn command_rejection_is_structured_and_deterministic_runs_match() {
    let mut rejected = WocTransactionalRuntime::new(
        RuntimeRole::Server,
        ScriptedVm::new(Behavior::Reject),
        TickBudgets::default(),
    );
    let fault = rejected.tick(vec![]).expect_err("rejection must fail");
    assert!(matches!(
        fault.kind,
        WocTickFaultKind::Vm(VmTickError::RejectedCommand { index: 0, .. })
    ));

    let mut left = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"same".to_vec())),
        TickBudgets::default(),
    );
    let mut right = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ScriptedVm::new(Behavior::Success(b"same".to_vec())),
        TickBudgets::default(),
    );
    left.tick(vec![]).expect("left tick must commit");
    right.tick(vec![]).expect("right tick must commit");
    assert_eq!(left.committed(), right.committed());
}

#[test]
#[ignore = "release performance gate; run through the validation coordinator"]
fn borrowed_tick_state_release_performance_gate() {
    const STATE_BYTES: usize = 16 * 1024 * 1024;
    const ENCODINGS_PER_SAMPLE: usize = 8;
    const SAMPLE_PAIRS: usize = 21;
    const THRESHOLD_PERCENT: f64 = 35.0;

    let state = vec![0x5a; STATE_BYTES];
    let owned_template = FixedTickInput {
        tick: 7,
        commands: vec![],
        wall_time_forbidden: true,
        committed_state: vec![],
        committed_state_digest: fnv1a_bytes(&state),
        generation: 3,
        movement_frames: vec![],
        offline_bootstrap: None,
    };
    let borrowed = FixedTickInputRef {
        tick: owned_template.tick,
        commands: &owned_template.commands,
        wall_time_forbidden: owned_template.wall_time_forbidden,
        committed_state: &state,
        committed_state_digest: owned_template.committed_state_digest,
        generation: owned_template.generation,
        movement_frames: &owned_template.movement_frames,
        offline_bootstrap: None,
    };

    let _ = measure_owned_tick_encoding(&owned_template, &state, 1);
    let _ = measure_borrowed_tick_encoding(borrowed, 1);

    let mut owned_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let (owned_ns, borrowed_ns) = if pair % 2 == 0 {
            (
                measure_owned_tick_encoding(&owned_template, &state, ENCODINGS_PER_SAMPLE),
                measure_borrowed_tick_encoding(borrowed, ENCODINGS_PER_SAMPLE),
            )
        } else {
            let borrowed_ns = measure_borrowed_tick_encoding(borrowed, ENCODINGS_PER_SAMPLE);
            let owned_ns =
                measure_owned_tick_encoding(&owned_template, &state, ENCODINGS_PER_SAMPLE);
            (owned_ns, borrowed_ns)
        };
        owned_samples.push(owned_ns);
        borrowed_samples.push(borrowed_ns);
    }

    let owned_p50 = nearest_rank(&owned_samples, 50);
    let owned_p95 = nearest_rank(&owned_samples, 95);
    let borrowed_p50 = nearest_rank(&borrowed_samples, 50);
    let borrowed_p95 = nearest_rank(&borrowed_samples, 95);
    let p50_improvement = improvement_percent(owned_p50, borrowed_p50);
    let p95_improvement = improvement_percent(owned_p95, borrowed_p95);
    let owned_ns = nanosecond_csv(&owned_samples);
    let borrowed_ns = nanosecond_csv(&borrowed_samples);
    eprintln!(
        "WOC_APP03_BORROWED_TICK_STATE_PERF state_bytes={STATE_BYTES} \
         encodings_per_sample={ENCODINGS_PER_SAMPLE} sample_pairs=21 \
         sample_order=alternating_owned_first_even percentile_method=nearest_rank \
         threshold_percent=35 owned_ns={owned_ns} borrowed_ns={borrowed_ns}"
    );
    assert!(
        p50_improvement >= THRESHOLD_PERCENT && p95_improvement >= THRESHOLD_PERCENT,
        "borrowed encoding improved P50 by {p50_improvement:.3}% and P95 by \
         {p95_improvement:.3}%, below the {THRESHOLD_PERCENT:.0}% gate"
    );
}

fn measure_owned_tick_encoding(template: &FixedTickInput, state: &[u8], encodings: usize) -> u128 {
    let started = std::time::Instant::now();
    for _ in 0..encodings {
        let input = FixedTickInput {
            tick: template.tick,
            commands: template.commands.clone(),
            wall_time_forbidden: template.wall_time_forbidden,
            committed_state: std::hint::black_box(state).to_vec(),
            committed_state_digest: template.committed_state_digest,
            generation: template.generation,
            movement_frames: template.movement_frames.clone(),
            offline_bootstrap: template.offline_bootstrap.clone(),
        };
        std::hint::black_box(input.encode_payload().expect("owned input encodes"));
    }
    started.elapsed().as_nanos()
}

fn measure_borrowed_tick_encoding(input: FixedTickInputRef<'_>, encodings: usize) -> u128 {
    let started = std::time::Instant::now();
    for _ in 0..encodings {
        std::hint::black_box(input.encode_payload().expect("borrowed input encodes"));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = percentile.saturating_mul(ordered.len()).saturating_add(99) / 100;
    ordered[rank.saturating_sub(1)]
}

fn nanosecond_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn improvement_percent(baseline: u128, optimized: u128) -> f64 {
    100.0 * (baseline as f64 - optimized as f64) / baseline as f64
}

fn successful_result(
    input: &FixedTickInput,
    state: &[u8],
    usage: TickUsage,
) -> Result<VmTickResult, VmTickError> {
    let snapshot = WorldSnapshot {
        tick: input.tick,
        state_digest: fnv1a_bytes(state),
        event_digest: event_stream_digest(&[]),
        state: state.to_vec(),
        events: vec![],
    };
    Ok(VmTickResult {
        output_payload: snapshot
            .encode_payload()
            .expect("scripted snapshot must encode"),
        presentation_payload: b"presentation".to_vec(),
        usage,
    })
}

#[test]
fn outer_checkpoint_rollback_preserves_role_terminal_fault_until_explicit_recovery() {
    for role in [
        RuntimeRole::Offline,
        RuntimeRole::Server,
        RuntimeRole::Client,
    ] {
        let mut runtime = WocTransactionalRuntime::new(
            role,
            RetainedVm::new(Some(RetainedFailure::Trap)),
            TickBudgets::default(),
        );
        let before = runtime.committed().clone();
        let outer = runtime.checkpoint().expect("outer owner checkpoint");
        let fault = runtime.tick(vec![]).expect_err("inner retained trap");
        let terminal = runtime.status().clone();
        assert_ne!(terminal, RuntimeStatus::Running);

        runtime
            .rollback_checkpoint(outer)
            .expect("outer VM restore");
        assert_eq!(runtime.committed(), &before);
        assert_eq!(runtime.vm().state, 0);
        assert_eq!(runtime.vm().live_checkpoints.get(), 0);
        assert_eq!(runtime.status(), &terminal);
        match (role, runtime.status()) {
            (RuntimeRole::Offline, RuntimeStatus::Paused(retained))
            | (RuntimeRole::Server, RuntimeStatus::Faulted(retained))
            | (RuntimeRole::Client, RuntimeStatus::Recovering(retained)) => {
                assert_eq!(retained, &fault);
            }
            _ => panic!("outer restore lost the original role terminal fault"),
        }
        let rejected = runtime.tick(vec![]).expect_err("terminal admission");
        assert!(matches!(rejected.kind, WocTickFaultKind::SessionNotRunning));
        assert_eq!(runtime.status(), &terminal);
        assert_eq!(runtime.vm().state, 0);

        runtime
            .install_full_snapshot(before)
            .expect("explicit recovery");
        assert_eq!(runtime.status(), &RuntimeStatus::Running);
        runtime
            .tick(vec![])
            .expect("replayed next tick after recovery");
        assert_eq!(runtime.committed().tick, 1);
        assert_eq!(runtime.vm().state, 1);
    }
}

#[test]
fn outer_checkpoint_restore_failure_keeps_original_fault_and_attaches_restore_error() {
    for role in [
        RuntimeRole::Offline,
        RuntimeRole::Server,
        RuntimeRole::Client,
    ] {
        let mut vm = RetainedVm::new(Some(RetainedFailure::Trap));
        vm.fail_rollback = true;
        let mut runtime = WocTransactionalRuntime::new(role, vm, TickBudgets::default());
        let before = runtime.committed().clone();
        let outer = runtime.checkpoint().expect("outer owner checkpoint");
        let fault = runtime
            .tick(vec![])
            .expect_err("inner trap and restore failure");
        let terminal = runtime.status().clone();
        assert!(
            matches!(fault.kind, WocTickFaultKind::Vm(VmTickError::Trap(ref reason))
            if reason == "injected retained trap")
        );
        assert!(
            matches!(fault.rollback_error, Some(VmTickError::Trap(ref reason))
            if reason == "injected rollback failure")
        );

        let restore_error = runtime
            .rollback_checkpoint(outer)
            .expect_err("outer restore failure");
        assert!(matches!(restore_error, VmTickError::Trap(ref reason)
            if reason == "injected rollback failure"));
        assert_eq!(runtime.status(), &terminal);
        assert_eq!(runtime.committed(), &before);
        assert_eq!(runtime.vm().state, 1);
        assert_eq!(runtime.vm().live_checkpoints.get(), 0);
        let rejected = runtime
            .tick(vec![])
            .expect_err("fault retains terminal admission");
        assert!(matches!(rejected.kind, WocTickFaultKind::SessionNotRunning));
        assert_eq!(runtime.status(), &terminal);
    }
}

#[test]
fn outer_checkpoint_restore_after_tick_exhaustion_keeps_the_original_terminal_fault() {
    for role in [
        RuntimeRole::Offline,
        RuntimeRole::Server,
        RuntimeRole::Client,
    ] {
        let mut runtime = WocTransactionalRuntime::new(
            role,
            ScriptedVm::new(Behavior::Success(b"unused".to_vec())),
            TickBudgets::default(),
        );
        let state = b"terminal".to_vec();
        let snapshot = woc_runtime::CommittedSnapshot {
            generation: 7,
            tick: u64::MAX,
            state_digest: fnv1a_bytes(&state),
            event_digest: event_stream_digest(&[]),
            presentation_digest: fnv1a_bytes(&[]),
            state,
            presentation_payload: vec![],
        };
        runtime
            .install_full_snapshot(snapshot.clone())
            .expect("terminal snapshot");
        let outer = runtime.checkpoint().expect("outer owner checkpoint");
        let calls_before_exhaustion = runtime.vm().calls;
        let fault = runtime.tick(vec![]).expect_err("no successor");
        assert_eq!(runtime.vm().calls, calls_before_exhaustion);
        assert!(matches!(fault.kind, WocTickFaultKind::TickExhausted));
        let terminal = runtime.status().clone();
        runtime
            .rollback_checkpoint(outer)
            .expect("outer VM restore");
        assert_eq!(runtime.status(), &terminal);
        assert_eq!(runtime.committed(), &snapshot);
        assert_eq!(runtime.vm().calls, calls_before_exhaustion + 1);
        let calls_before_retry = runtime.vm().calls;
        let repeated = runtime.tick(vec![]).expect_err("terminal admission");
        assert!(matches!(repeated.kind, WocTickFaultKind::SessionNotRunning));
        assert_eq!(runtime.status(), &terminal);
        assert_eq!(runtime.vm().calls, calls_before_retry);
    }
}

#[test]
fn successful_outer_checkpoint_restore_rewinds_a_running_transaction() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Client,
        RetainedVm::new(None),
        TickBudgets::default(),
    );
    let before = runtime.committed().clone();
    let outer = runtime.checkpoint().expect("running owner checkpoint");
    runtime.tick(vec![]).expect("committed tick");
    assert_eq!(runtime.vm().state, 1);
    runtime
        .rollback_checkpoint(outer)
        .expect("running owner restore");
    assert_eq!(runtime.status(), &RuntimeStatus::Running);
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().state, 0);
    runtime.tick(vec![]).expect("same tick replay");
    assert_eq!(runtime.committed().tick, 1);
}

#[test]
fn failed_outer_restore_records_its_error_after_a_successful_inner_rollback() {
    for role in [
        RuntimeRole::Offline,
        RuntimeRole::Server,
        RuntimeRole::Client,
    ] {
        let mut vm = RetainedVm::new(Some(RetainedFailure::Trap));
        vm.fail_rollback_after = Some(1);
        let mut runtime = WocTransactionalRuntime::new(role, vm, TickBudgets::default());
        let before = runtime.committed().clone();
        let outer = runtime.checkpoint().expect("outer owner checkpoint");
        let fault = runtime.tick(vec![]).expect_err("inner trap");
        assert!(
            fault.rollback_error.is_none(),
            "the inner restore succeeded"
        );
        assert_eq!(runtime.vm().state, 0);
        assert_eq!(runtime.vm().rollback_attempts, 1);

        let restore_error = runtime
            .rollback_checkpoint(outer)
            .expect_err("outer restore fails");
        assert!(matches!(restore_error, VmTickError::Trap(ref reason)
            if reason == "injected rollback failure"));
        let retained = match (role, runtime.status()) {
            (RuntimeRole::Offline, RuntimeStatus::Paused(retained))
            | (RuntimeRole::Server, RuntimeStatus::Faulted(retained))
            | (RuntimeRole::Client, RuntimeStatus::Recovering(retained)) => retained,
            _ => panic!("failed restore must retain role terminal admission"),
        };
        assert_eq!(retained.kind, fault.kind);
        assert_eq!(retained.attempted_tick, fault.attempted_tick);
        assert!(
            matches!(retained.rollback_error, Some(VmTickError::Trap(ref reason))
            if reason == "injected rollback failure")
        );
        assert_eq!(runtime.committed(), &before);
        assert_eq!(runtime.vm().state, 0);
        assert_eq!(runtime.vm().rollback_attempts, 2);
        assert_eq!(runtime.vm().live_checkpoints.get(), 0);
    }
}

use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, EntityRef, FixedTickInput, MovementFrame,
    MovementInputFlags, OfflineSessionBootstrap, OfflineWeaponSkinAccount, WorldSnapshot,
    OFFLINE_SESSION_BOOTSTRAP_VERSION, STANDARD_OFFLINE_WORLD_SEED,
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

struct ScriptedVm {
    behavior: Behavior,
    observed_inputs: Vec<FixedTickInput>,
}

impl ScriptedVm {
    fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            observed_inputs: Vec::new(),
        }
    }
}

impl WocProjectVm for ScriptedVm {
    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        let input = FixedTickInput::decode_payload(input_payload)
            .expect("runtime must send a valid fixed tick payload");
        self.observed_inputs.push(input.clone());
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

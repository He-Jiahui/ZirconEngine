use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, EntityRef, FixedTickInput, MovementFrame,
    MovementInputFlags, WorldSnapshot,
};
use woc_runtime::{RuntimeStatus, TickBudgets, TickUsage, VmTickError, VmTickResult, WocProjectVm};
use woc_server::{
    FixedServerTickDriver, ServerTickDriverInitError, ServerTickInputError, SERVER_TICK_NS,
};

#[derive(Default)]
struct RecordingVm {
    inputs: Vec<FixedTickInput>,
    fail_next: bool,
}

impl WocProjectVm for RecordingVm {
    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        let input = FixedTickInput::decode_payload(input_payload)
            .expect("server scheduler must pass one valid canonical batch");
        self.inputs.push(input.clone());
        if self.fail_next {
            return Err(VmTickError::Trap("injected server fault".to_string()));
        }
        let snapshot = WorldSnapshot {
            tick: input.tick,
            state_digest: fnv1a_bytes(b"server-state"),
            event_digest: event_stream_digest(&[]),
            state: b"server-state".to_vec(),
            events: Vec::new(),
        };
        Ok(VmTickResult {
            output_payload: snapshot
                .encode_payload()
                .expect("scripted server snapshot must encode"),
            presentation_payload: Vec::new(),
            usage: TickUsage::default(),
        })
    }
}

fn command(sequence: u32) -> Command {
    Command {
        command_id: 5,
        actor: EntityRef {
            id: 7,
            generation: 2,
        },
        sequence,
        payload: Vec::new(),
    }
}

fn movement(actor_id: u64, sequence: u32) -> MovementFrame {
    MovementFrame {
        actor: EntityRef {
            id: actor_id,
            generation: 1,
        },
        sequence,
        flags: MovementInputFlags {
            forward: true,
            ..MovementInputFlags::default()
        },
        facing: Some(0.5),
    }
}

#[test]
fn driver_requires_positive_catch_up_and_queue_budgets() {
    let budgets = TickBudgets::default();
    assert!(matches!(
        FixedServerTickDriver::new(RecordingVm::default(), budgets, 0, 1, 1),
        Err(ServerTickDriverInitError::ZeroCatchUpBudget)
    ));
    assert!(matches!(
        FixedServerTickDriver::new(RecordingVm::default(), budgets, 1, 0, 1),
        Err(ServerTickDriverInitError::ZeroCommandQueueBudget)
    ));
    assert!(matches!(
        FixedServerTickDriver::new(RecordingVm::default(), budgets, 1, 1, 0),
        Err(ServerTickDriverInitError::ZeroMovementQueueBudget)
    ));
}

#[test]
fn driver_delivers_one_canonical_batch_at_a_twenty_hz_boundary() {
    let mut driver =
        FixedServerTickDriver::new(RecordingVm::default(), TickBudgets::default(), 2, 4, 4)
            .expect("valid scheduler configuration");
    driver
        .enqueue_commands(vec![command(2), command(1)])
        .unwrap();
    driver
        .enqueue_movement(vec![movement(9, 1), movement(3, 2)])
        .unwrap();

    assert_eq!(
        driver.advance(SERVER_TICK_NS - 1).unwrap().committed_ticks,
        0
    );
    let advance = driver.advance(1).expect("exact fixed boundary must commit");
    assert_eq!(advance.committed_ticks, 1);
    assert_eq!(advance.backlog_ticks, 0);

    let input = &driver.runtime().vm().inputs[0];
    assert_eq!(input.tick, 1);
    assert!(input.wall_time_forbidden);
    assert_eq!(input.commands, vec![command(1), command(2)]);
    assert_eq!(input.movement_frames.len(), 2);
    assert_eq!(input.movement_frames[0].actor.id, 3);
    assert_eq!(input.movement_frames[1].actor.id, 9);
    assert_eq!(driver.pending_command_count(), 0);
    assert_eq!(driver.pending_movement_count(), 0);
}

#[test]
fn driver_bounds_input_atomically_and_limits_catch_up_without_dropping_backlog() {
    let mut driver =
        FixedServerTickDriver::new(RecordingVm::default(), TickBudgets::default(), 2, 2, 2)
            .expect("valid scheduler configuration");
    driver
        .enqueue_commands(vec![command(1), command(2)])
        .unwrap();
    assert!(matches!(
        driver.enqueue_commands(vec![command(3)]),
        Err(ServerTickInputError::CommandQueueFull { maximum: 2 })
    ));
    assert_eq!(driver.pending_command_count(), 2);

    let advance = driver.advance(SERVER_TICK_NS * 3).unwrap();
    assert_eq!(advance.committed_ticks, 2);
    assert_eq!(advance.backlog_ticks, 1);
    assert_eq!(driver.runtime().vm().inputs[0].commands.len(), 2);
    assert!(driver.runtime().vm().inputs[1].commands.is_empty());
}

#[test]
fn driver_rejects_duplicate_movement_actor_before_it_can_reach_the_vm() {
    let mut driver =
        FixedServerTickDriver::new(RecordingVm::default(), TickBudgets::default(), 1, 4, 4)
            .expect("valid scheduler configuration");
    driver.enqueue_movement(vec![movement(3, 1)]).unwrap();
    assert!(matches!(
        driver.enqueue_movement(vec![movement(3, 2)]),
        Err(ServerTickInputError::Movement(_))
    ));
    assert_eq!(driver.pending_movement_count(), 1);
}

#[test]
fn driver_canonicalizes_command_arrival_and_rejects_duplicate_actor_sequences() {
    let mut driver =
        FixedServerTickDriver::new(RecordingVm::default(), TickBudgets::default(), 1, 4, 4)
            .expect("valid scheduler configuration");
    driver
        .enqueue_commands(vec![command(2), command(1)])
        .unwrap();
    driver.advance(SERVER_TICK_NS).unwrap();
    assert_eq!(
        driver.runtime().vm().inputs[0].commands,
        vec![command(1), command(2)]
    );

    assert!(matches!(
        driver.enqueue_commands(vec![command(3), command(3)]),
        Err(ServerTickInputError::DuplicateCommandSequence {
            actor_id: 7,
            generation: 2,
            sequence: 3,
        })
    ));
    assert_eq!(driver.pending_command_count(), 0);
}

#[test]
fn driver_faults_the_server_and_retains_the_failed_canonical_batch_for_diagnostics() {
    let mut vm = RecordingVm::default();
    vm.fail_next = true;
    let mut driver = FixedServerTickDriver::new(vm, TickBudgets::default(), 1, 4, 4)
        .expect("valid scheduler configuration");
    driver.enqueue_commands(vec![command(1)]).unwrap();
    driver.enqueue_movement(vec![movement(3, 1)]).unwrap();

    let fault = driver
        .advance(SERVER_TICK_NS)
        .expect_err("VM trap must fault the server");
    assert!(matches!(fault, woc_server::ServerTickDriverError::Tick(_)));
    assert!(matches!(
        driver.runtime().status(),
        RuntimeStatus::Faulted(_)
    ));
    let failed = driver
        .last_failed_input()
        .expect("fault diagnostics must retain the canonical input once");
    assert_eq!(failed.commands, vec![command(1)]);
    assert_eq!(failed.movement_frames, vec![movement(3, 1)]);
}

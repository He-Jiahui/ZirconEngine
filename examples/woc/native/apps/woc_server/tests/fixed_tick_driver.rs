use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, EntityRef, FixedTickInput, MovementFrame,
    MovementInputFlags, WorldSnapshot, MAX_MOVEMENT_FRAMES_PER_TICK,
};
use woc_runtime::{RuntimeStatus, TickBudgets, TickUsage, VmTickError, VmTickResult, WocProjectVm};
use woc_server::{
    FixedServerTickDriver, ServerTickDriverInitError, ServerTickInputError, SERVER_TICK_NS,
};

#[test]
fn server_tick_duration_tracks_the_protocol_simulation_step() {
    assert_eq!(SERVER_TICK_NS, woc_protocol::SIMULATION_STEP_NS);
}

#[derive(Default)]
struct RecordingVm {
    inputs: Vec<FixedTickInput>,
    fail_next: bool,
}

impl WocProjectVm for RecordingVm {
    type Checkpoint = Vec<u8>;

    fn checkpoint(&mut self) -> Result<Vec<u8>, VmTickError> {
        Ok((self.inputs.len() as u64).to_le_bytes().to_vec())
    }

    fn rollback(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), VmTickError> {
        let count = checkpoint
            .get(..8)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or_else(|| VmTickError::Transport("invalid checkpoint".to_string()))?;
        self.inputs.truncate(count as usize);
        Ok(())
    }

    fn install_full_snapshot(
        &mut self,
        _snapshot: &woc_runtime::CommittedSnapshot,
    ) -> Result<(), VmTickError> {
        Ok(())
    }

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
fn rejected_driver_configuration_returns_the_vm_to_its_host() {
    let vm = RecordingVm::default();
    let result = FixedServerTickDriver::new_with_vm_recovery(
        vm,
        TickBudgets::default(),
        0,
        1,
        1,
    );
    let Err((error, vm)) = result else {
        panic!("zero catch-up budget must return the activated VM to the host");
    };
    assert_eq!(error, ServerTickDriverInitError::ZeroCatchUpBudget);
    assert!(vm.inputs.is_empty());
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
fn driver_rejects_invalid_commands_without_mutating_queue_or_reserving_sequences() {
    let mut malformed = command(3);
    malformed.payload.push(1);
    let mut unknown = command(3);
    unknown.command_id = u16::MAX;

    for invalid in [malformed, unknown] {
        assert!(invalid.validate().is_err());
        let mut driver =
            FixedServerTickDriver::new(RecordingVm::default(), TickBudgets::default(), 1, 4, 4)
                .unwrap();
        driver.enqueue_commands(vec![command(1)]).unwrap();

        assert!(matches!(
            driver.enqueue_commands(vec![command(2), invalid]),
            Err(ServerTickInputError::Command(_))
        ));
        assert_eq!(driver.pending_command_count(), 1);
        assert!(driver.runtime().vm().inputs.is_empty());

        driver
            .enqueue_commands(vec![command(2), command(3)])
            .unwrap();
        driver.advance(SERVER_TICK_NS).unwrap();
        assert_eq!(
            driver.runtime().vm().inputs[0].commands,
            vec![command(1), command(2), command(3)]
        );
    }
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
fn oversized_protocol_batch_is_rejected_without_losing_pending_frames() {
    let frame_count = MAX_MOVEMENT_FRAMES_PER_TICK + 1;
    let mut driver = FixedServerTickDriver::new(
        RecordingVm::default(),
        TickBudgets::default(),
        1,
        1,
        frame_count,
    )
    .expect("the host queue may expose a larger budget than one protocol tick");
    let frames = (1..=frame_count)
        .map(|actor_id| movement(actor_id as u64, 1))
        .collect();
    driver.enqueue_movement(frames).unwrap();

    assert!(matches!(
        driver.advance(SERVER_TICK_NS),
        Err(woc_server::ServerTickDriverError::Movement(
            woc_protocol::MovementInputError::TooManyFrames {
                actual,
                maximum: MAX_MOVEMENT_FRAMES_PER_TICK,
            }
        )) if actual == frame_count
    ));
    assert_eq!(driver.pending_movement_count(), frame_count);
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
    assert_eq!(driver.pending_command_count(), 1);
    assert_eq!(driver.pending_movement_count(), 1);

    let retry_fault = driver
        .advance(0)
        .expect_err("a faulted server must refuse work until recovery");
    assert!(matches!(
        retry_fault,
        woc_server::ServerTickDriverError::Tick(_)
    ));
    let failed_after_retry = driver
        .last_failed_input()
        .expect("the original failed batch must remain the diagnostic");
    assert_eq!(failed_after_retry.commands, vec![command(1)]);
    assert_eq!(failed_after_retry.movement_frames, vec![movement(3, 1)]);
}

#[test]
fn recovered_server_retries_the_same_failed_batch_without_new_time() {
    let mut vm = RecordingVm::default();
    vm.fail_next = true;
    let mut driver = FixedServerTickDriver::new(vm, TickBudgets::default(), 1, 4, 4)
        .expect("valid scheduler configuration");
    driver.enqueue_commands(vec![command(1)]).unwrap();
    driver.enqueue_movement(vec![movement(3, 1)]).unwrap();
    driver
        .advance(SERVER_TICK_NS)
        .expect_err("first tick fails");

    let snapshot = driver.runtime().committed().clone();
    driver
        .runtime_mut()
        .install_full_snapshot(snapshot)
        .expect("snapshot recovery resumes the runtime");
    let advance = driver.advance(0).expect("recovery retries pending input");
    assert_eq!(advance.committed_ticks, 1);
    assert_eq!(driver.pending_command_count(), 0);
    assert_eq!(driver.pending_movement_count(), 0);
    assert_eq!(driver.runtime().vm().inputs.len(), 1);
    assert_eq!(driver.runtime().vm().inputs[0].commands, vec![command(1)]);
}

use woc_client::{
    ClientAuthority, ClientCommandQueueError, ClientFrameDriver, ClientFrameDriverError,
    ClientFrameDriverInitError, ClientMovementInputError, ClientTickInput,
    TransactionalClientAuthority, MAX_PENDING_COMMANDS,
};
use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, EntityRef, FixedTickInput, MovementFrame,
    MovementInputError, MovementInputFlags, WorldSnapshot,
};
use woc_runtime::{
    ActorAnimationInput, ActorAppearance, ActorPresentation, ActorTransform,
    BulkPresentationProjection, ClientPresentationProjection, ClientWindowProjection, HudMeter,
    HudProjection, HudUnit, InventoryWindowProjection, PresentationBlendMode, PresentationCadence,
    PresentationSnapshot, PresentationTimelinePush, PresentationVec3, QuestLogWindowProjection,
    RuntimeStatus, TickBudgets, TickUsage, VmTickError, VmTickResult, WocProjectVm,
    WocTickFaultKind, CLIENT_PRESENTATION_SCHEMA_VERSION,
};

const ATTACK_COMMAND_ID: u16 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FakeAuthorityError {
    Planned,
}

#[derive(Default)]
struct FakeAuthority {
    tick: u64,
    attempts: Vec<(u64, Vec<u32>)>,
    movement_attempts: Vec<MovementFrame>,
    fail_next: bool,
}

#[derive(Default)]
struct ProjectingVm {
    invalid_projection: bool,
}

impl WocProjectVm for ProjectingVm {
    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        let input = FixedTickInput::decode_payload(input_payload).expect("fixed input");
        let state = input.tick.to_le_bytes().to_vec();
        let output = WorldSnapshot {
            tick: input.tick,
            state_digest: fnv1a_bytes(&state),
            event_digest: event_stream_digest(&[]),
            state,
            events: vec![],
        }
        .encode_payload()
        .expect("world snapshot");
        let presentation_payload = if self.invalid_projection {
            br#"{}"#.to_vec()
        } else {
            client_projection(input.tick)
                .encode_json()
                .expect("presentation projection")
        };
        Ok(VmTickResult {
            output_payload: output,
            presentation_payload,
            usage: TickUsage::default(),
        })
    }
}

fn client_projection(tick: u64) -> ClientPresentationProjection {
    ClientPresentationProjection {
        schema_version: CLIENT_PRESENTATION_SCHEMA_VERSION,
        world: BulkPresentationProjection {
            viewer: EntityRef {
                id: 1,
                generation: 1,
            },
            actors: vec![ActorPresentation {
                entity: EntityRef {
                    id: 1,
                    generation: 1,
                },
                template_id: "warrior".to_string(),
                transform: ActorTransform {
                    translation: PresentationVec3 {
                        x: tick as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    facing_radians: 0.0,
                },
                animation: ActorAnimationInput::default(),
                appearance: ActorAppearance::default(),
            }],
        },
        hud: HudProjection {
            player: HudUnit {
                entity: EntityRef {
                    id: 1,
                    generation: 1,
                },
                display_name: format!("Hero {tick}"),
                title_id: None,
                level: 1,
                health: HudMeter {
                    current: 100.0,
                    maximum: 100.0,
                },
                resource: None,
                absorb: 0.0,
                dead: false,
                hostile: false,
                elite: false,
                boss: false,
                cast: None,
            },
            target: None,
            target_of_target: None,
            combo_points: 0,
            actions: vec![],
            tracked_quests: vec![],
        },
        windows: ClientWindowProjection {
            inventory: InventoryWindowProjection {
                backpack_slots: 16,
                capacity: 16,
                copper: 0,
                bags: vec![None, None, None, None],
                items: Vec::new(),
            },
            quest_log: QuestLogWindowProjection {
                completed_count: 0,
                quests: Vec::new(),
            },
        },
    }
}

impl ClientAuthority<u64> for FakeAuthority {
    type Error = FakeAuthorityError;

    fn fixed_step(
        &mut self,
        input: ClientTickInput<'_>,
        received_at_ns: u64,
    ) -> Result<PresentationSnapshot<u64>, Self::Error> {
        self.attempts.push((
            received_at_ns,
            input.commands().iter().map(|c| c.sequence).collect(),
        ));
        self.movement_attempts.push(input.movement());
        if self.fail_next {
            self.fail_next = false;
            return Err(FakeAuthorityError::Planned);
        }
        self.tick += 1;
        Ok(PresentationSnapshot::new(
            1,
            self.tick,
            self.tick as u32,
            (self.tick as u32).rotate_left(7),
            (self.tick as u32).rotate_left(13),
            received_at_ns,
            self.tick,
        ))
    }
}

fn command(sequence: u32) -> Command {
    Command {
        command_id: ATTACK_COMMAND_ID,
        actor: EntityRef {
            id: 1,
            generation: 1,
        },
        sequence,
        payload: Vec::new(),
    }
}

fn bootstrap() -> PresentationSnapshot<u64> {
    PresentationSnapshot::new(1, 0, 0, 0, 0, 0, 0)
}

fn driver(max_catch_up_ticks: u32) -> ClientFrameDriver<FakeAuthority, u64> {
    let mut driver = ClientFrameDriver::new(
        FakeAuthority::default(),
        PresentationCadence::woc_default(),
        max_catch_up_ticks,
        EntityRef {
            id: 1,
            generation: 1,
        },
        1,
    )
    .expect("valid catch-up budget");
    assert_eq!(
        driver.install_initial(bootstrap()).expect("bootstrap"),
        PresentationTimelinePush::Reset
    );
    driver
}

fn movement_flags() -> MovementInputFlags {
    MovementInputFlags {
        forward: true,
        strafe_right: true,
        jump: true,
        ..MovementInputFlags::default()
    }
}

#[test]
fn three_sixty_hz_frames_commit_one_twenty_hz_authority_step() {
    let mut driver = driver(4);

    let first = driver.advance_frame(16_666_667).expect("first frame");
    let second = driver.advance_frame(16_666_667).expect("second frame");
    assert_eq!(first.committed_ticks, 0);
    assert_eq!(second.committed_ticks, 0);
    assert_eq!(driver.authority().attempts.len(), 0);

    let third = driver.advance_frame(16_666_667).expect("third frame");
    assert_eq!(third.committed_ticks, 1);
    assert_eq!(third.backlog_ticks, 0);
    assert_eq!(driver.authority().attempts, vec![(50_000_000, Vec::new())]);
    let sample = driver.sample().expect("interpolated sample");
    assert_eq!(sample.mode, PresentationBlendMode::Interpolate);
    assert_eq!((*sample.from, *sample.to), (0, 1));
    assert!(sample.alpha < 0.000_001);

    driver.advance_frame(16_666_667).expect("fourth frame");
    let sample = driver.sample().expect("one-third sample");
    assert!((sample.alpha - (1.0 / 3.0)).abs() < 0.000_001);
}

#[test]
fn queued_commands_are_delivered_once_to_the_next_successful_commit() {
    let mut driver = driver(4);
    driver.queue_command(command(7)).expect("queue command 7");
    driver.queue_command(command(8)).expect("queue command 8");

    driver.advance_frame(50_000_000).expect("first tick");
    driver.advance_frame(50_000_000).expect("second tick");

    assert_eq!(
        driver.authority().attempts,
        vec![(50_000_000, vec![7, 8]), (100_000_000, Vec::new())]
    );
    assert_eq!(driver.pending_command_count(), 0);
}

#[test]
fn authority_failure_retains_time_and_commands_for_a_retry() {
    let mut driver = driver(4);
    driver.authority_mut().fail_next = true;
    driver.queue_command(command(11)).expect("queue command 11");

    assert_eq!(
        driver
            .advance_frame(50_000_000)
            .expect_err("planned failure"),
        ClientFrameDriverError::Authority(FakeAuthorityError::Planned)
    );
    assert_eq!(driver.pending_command_count(), 1);

    let retry = driver
        .advance_frame(0)
        .expect("retry without advancing presentation time");
    assert_eq!(retry.committed_ticks, 1);
    assert_eq!(
        driver.authority().attempts,
        vec![(50_000_000, vec![11]), (50_000_000, vec![11])]
    );
    assert_eq!(driver.pending_command_count(), 0);
}

#[test]
fn movement_stream_retries_the_same_twenty_hz_frame_until_the_authority_commits() {
    let mut driver = driver(4);
    driver
        .set_movement_input(movement_flags(), Some(0.75))
        .expect("valid held movement state");
    driver.authority_mut().fail_next = true;

    assert_eq!(
        driver
            .advance_frame(50_000_000)
            .expect_err("the first authority attempt must fail"),
        ClientFrameDriverError::Authority(FakeAuthorityError::Planned)
    );
    assert_eq!(driver.authority().movement_attempts.len(), 1);
    assert_eq!(
        driver.authority().movement_attempts[0],
        MovementFrame {
            actor: EntityRef {
                id: 1,
                generation: 1,
            },
            sequence: 1,
            flags: movement_flags(),
            facing: Some(0.75),
        }
    );

    driver
        .advance_frame(0)
        .expect("the retried fixed boundary must commit");
    driver
        .advance_frame(50_000_000)
        .expect("the following boundary must commit");

    assert_eq!(
        driver.authority().movement_attempts,
        vec![
            MovementFrame {
                actor: EntityRef {
                    id: 1,
                    generation: 1,
                },
                sequence: 1,
                flags: movement_flags(),
                facing: Some(0.75),
            },
            MovementFrame {
                actor: EntityRef {
                    id: 1,
                    generation: 1,
                },
                sequence: 1,
                flags: movement_flags(),
                facing: Some(0.75),
            },
            MovementFrame {
                actor: EntityRef {
                    id: 1,
                    generation: 1,
                },
                sequence: 2,
                flags: movement_flags(),
                facing: Some(0.75),
            },
        ]
    );
}

#[test]
fn movement_stream_rejects_an_invalid_facing_without_replacing_the_last_valid_state() {
    let mut driver = driver(4);
    driver
        .set_movement_input(movement_flags(), Some(0.5))
        .expect("initial movement state");

    assert!(matches!(
        driver
            .set_movement_input(MovementInputFlags::default(), Some(f64::NAN))
            .expect_err("a non-finite facing must be refused before mutation"),
        ClientMovementInputError::InvalidInput(MovementInputError::InvalidFacing {
            value,
            maximum,
        }) if value.is_nan() && maximum == woc_protocol::MAX_MOVEMENT_FACING_MAGNITUDE
    ));

    driver
        .advance_frame(50_000_000)
        .expect("the last valid state must still reach the fixed boundary");
    assert_eq!(
        driver.authority().movement_attempts,
        vec![MovementFrame {
            actor: EntityRef {
                id: 1,
                generation: 1,
            },
            sequence: 1,
            flags: movement_flags(),
            facing: Some(0.5),
        }]
    );
}

#[test]
fn movement_sequence_never_wraps_after_the_u32_maximum_commit() {
    let mut driver = ClientFrameDriver::new(
        FakeAuthority::default(),
        PresentationCadence::woc_default(),
        4,
        EntityRef {
            id: 1,
            generation: 1,
        },
        u32::MAX,
    )
    .expect("the final positive sequence is valid");

    driver
        .advance_frame(50_000_000)
        .expect("the maximum sequence may commit once");
    assert_eq!(driver.authority().movement_attempts[0].sequence, u32::MAX);
    assert_eq!(
        driver
            .advance_frame(50_000_000)
            .expect_err("the next movement frame must not wrap to zero"),
        ClientFrameDriverError::Movement(ClientMovementInputError::SequenceExhausted)
    );
}

#[test]
fn catch_up_budget_defers_but_never_discards_authoritative_ticks() {
    assert!(matches!(
        ClientFrameDriver::<FakeAuthority, u64>::new(
            FakeAuthority::default(),
            PresentationCadence::woc_default(),
            0,
            EntityRef {
                id: 1,
                generation: 1,
            },
            1,
        ),
        Err(ClientFrameDriverInitError::ZeroCatchUpBudget)
    ));

    let mut driver = driver(2);
    let first = driver
        .advance_frame(250_000_000)
        .expect("first catch-up slice");
    assert_eq!((first.committed_ticks, first.backlog_ticks), (2, 3));
    let second = driver.advance_frame(0).expect("second catch-up slice");
    assert_eq!((second.committed_ticks, second.backlog_ticks), (2, 1));
    let third = driver.advance_frame(0).expect("last catch-up slice");
    assert_eq!((third.committed_ticks, third.backlog_ticks), (1, 0));
    assert_eq!(driver.authority().tick, 5);
    assert_eq!(
        driver
            .authority()
            .attempts
            .iter()
            .map(|attempt| attempt.0)
            .collect::<Vec<_>>(),
        vec![
            50_000_000,
            100_000_000,
            150_000_000,
            200_000_000,
            250_000_000
        ]
    );
}

#[test]
fn pending_commands_are_bounded_by_the_protocol_per_tick_limit() {
    let mut driver = driver(4);
    for sequence in 0..MAX_PENDING_COMMANDS {
        driver
            .queue_command(command(sequence as u32))
            .expect("command within protocol limit");
    }
    assert_eq!(driver.pending_command_count(), MAX_PENDING_COMMANDS);
    assert_eq!(
        driver
            .queue_command(command(MAX_PENDING_COMMANDS as u32))
            .expect_err("one command beyond the protocol limit"),
        ClientCommandQueueError::Full {
            maximum: MAX_PENDING_COMMANDS,
        }
    );
}

#[test]
fn transactional_authority_commits_state_and_bulk_projection_atomically() {
    let authority =
        TransactionalClientAuthority::new(ProjectingVm::default(), TickBudgets::default());
    let mut driver = ClientFrameDriver::new(
        authority,
        PresentationCadence::woc_default(),
        4,
        EntityRef {
            id: 1,
            generation: 1,
        },
        1,
    )
    .expect("client driver");

    let advance = driver
        .advance_frame(50_000_000)
        .expect("atomic projected tick");
    assert_eq!(advance.committed_ticks, 1);
    assert_eq!(driver.authority().runtime().committed().tick, 1);
    let sample = driver.sample().expect("committed presentation");
    assert_eq!(sample.mode, PresentationBlendMode::HoldCurrent);
    assert_eq!(sample.to.hud.player.display_name, "Hero 1");
    assert_eq!(sample.to.world.actors.len(), 1);
}

#[test]
fn presented_frame_interpolates_actor_transforms_but_uses_current_hud() {
    let authority =
        TransactionalClientAuthority::new(ProjectingVm::default(), TickBudgets::default());
    let mut driver = ClientFrameDriver::new(
        authority,
        PresentationCadence::woc_default(),
        4,
        EntityRef {
            id: 1,
            generation: 1,
        },
        1,
    )
    .expect("client driver");
    driver
        .advance_frame(100_000_000)
        .expect("two projected ticks");

    let mut presented_x = Vec::new();
    let frame = driver
        .visit_presented_actors(|_actor, transform| presented_x.push(transform.translation.x))
        .expect("valid actor projection")
        .expect("presented frame");
    assert_eq!(presented_x, vec![1.0]);
    assert_eq!(frame.hud.player.display_name, "Hero 2");
    assert_eq!(frame.blend_mode, PresentationBlendMode::Interpolate);
    assert_eq!(frame.alpha, 0.0);
}

#[test]
fn invalid_bulk_projection_recovers_without_committing_or_consuming_commands() {
    let authority = TransactionalClientAuthority::new(
        ProjectingVm {
            invalid_projection: true,
        },
        TickBudgets::default(),
    );
    let mut driver = ClientFrameDriver::new(
        authority,
        PresentationCadence::woc_default(),
        4,
        EntityRef {
            id: 1,
            generation: 1,
        },
        1,
    )
    .expect("client driver");
    driver.queue_command(command(21)).expect("queue command");

    let error = driver
        .advance_frame(50_000_000)
        .expect_err("invalid projection must fail before commit");
    assert!(matches!(
        error,
        ClientFrameDriverError::Authority(ref fault)
            if matches!(&fault.kind, WocTickFaultKind::DecodePresentation(_))
    ));
    assert_eq!(driver.pending_command_count(), 1);
    assert_eq!(driver.authority().runtime().committed().tick, 0);
    assert!(matches!(
        driver.authority().runtime().status(),
        RuntimeStatus::Recovering(_)
    ));
}

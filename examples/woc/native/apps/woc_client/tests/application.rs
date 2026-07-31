use std::convert::Infallible;

use woc_client::{
    ClientAuthority, ClientCommandQueueError, ClientGameplayIntent, ClientInputDevice,
    ClientInputEvent, ClientSessionHudRouteError, ClientSessionInputError, ClientTickInput,
    HudHostEffect, HudRouteError, ShellHostEffect, ShellRouteEffect, WocClientSession,
    WocShellController, MAX_PENDING_COMMANDS,
};
use woc_client::{GamepadMoveFlags, MovementInputSources};
use woc_protocol::{Command, EntityRef, MovementFrame, MovementInputFlags};
use woc_runtime::{PresentationCadence, PresentationSnapshot};

#[derive(Default)]
struct FakeAuthority {
    tick: u64,
    delivered_sequences: Vec<Vec<u32>>,
    delivered_movement: Vec<MovementFrame>,
}

impl ClientAuthority<u64> for FakeAuthority {
    type Error = Infallible;

    fn fixed_step(
        &mut self,
        input: ClientTickInput<'_>,
        scheduled_at_ns: u64,
    ) -> Result<PresentationSnapshot<u64>, Self::Error> {
        self.tick += 1;
        self.delivered_sequences.push(
            input
                .commands()
                .iter()
                .map(|command| command.sequence)
                .collect(),
        );
        self.delivered_movement.push(input.movement());
        Ok(PresentationSnapshot::new(
            1,
            self.tick,
            self.tick as u32,
            0,
            0,
            scheduled_at_ns,
            self.tick,
        ))
    }
}

fn input(attacking: bool) -> ClientInputEvent {
    ClientInputEvent {
        device: ClientInputDevice::KeyboardMouse,
        intent: ClientGameplayIntent::SetAttacking { attacking },
    }
}

fn queued_command(sequence: u32) -> Command {
    Command {
        command_id: 9,
        actor: EntityRef {
            id: 1,
            generation: 1,
        },
        sequence,
        payload: Vec::new(),
    }
}

fn session() -> WocClientSession<FakeAuthority, u64> {
    let mut session = WocClientSession::new(
        WocShellController::new(true, woc_client::CharacterSortMode::Level),
        EntityRef {
            id: 1,
            generation: 1,
        },
        1,
        FakeAuthority::default(),
        PresentationCadence::woc_default(),
        4,
    )
    .expect("session construction");
    session
        .install_initial(PresentationSnapshot::new(1, 0, 0, 0, 0, 0, 0))
        .expect("initial snapshot");
    session
}

#[test]
fn maps_input_once_and_delivers_it_to_the_next_twenty_hz_commit() {
    let mut session = session();
    let mapped = session.queue_input(input(true)).expect("map input");
    assert_eq!((mapped.command_id, mapped.sequence), (9, 1));

    let advance = session.advance_frame(50_000_000).expect("fixed tick");
    assert_eq!(advance.committed_ticks, 1);
    assert_eq!(
        session.frame_driver().authority().delivered_sequences,
        vec![vec![1]]
    );
    assert_eq!(session.command_mapper().next_sequence(), Some(2));
}

#[test]
fn full_command_queue_rejects_input_without_consuming_the_mapper_sequence() {
    let mut session = session();
    for sequence in 20..20 + MAX_PENDING_COMMANDS as u32 {
        session
            .frame_driver_mut()
            .queue_command(queued_command(sequence))
            .expect("fill command queue");
    }

    assert_eq!(
        session
            .queue_input(input(false))
            .expect_err("queue is full"),
        ClientSessionInputError::Queue(ClientCommandQueueError::Full {
            maximum: MAX_PENDING_COMMANDS,
        })
    );
    assert_eq!(session.command_mapper().next_sequence(), Some(1));
}

#[test]
fn forwards_shell_routes_without_claiming_a_platform_capability() {
    let mut session = session();
    assert_eq!(
        session
            .dispatch_shell_route("woc.shell.mode.copy_contract", None)
            .expect("host effect"),
        Some(ShellRouteEffect::Host(ShellHostEffect::CopyContractAddress))
    );
    assert_eq!(
        session.shell().mode().selected_mode(),
        woc_client::ServerMode::Online
    );
}

#[test]
fn queues_hud_authority_input_through_the_next_twenty_hz_commit() {
    let mut session = session();

    assert_eq!(
        session
            .dispatch_hud_route("woc.hud.touch.activate.0", true)
            .expect("HUD action input"),
        None
    );
    assert_eq!(session.command_mapper().next_sequence(), Some(2));

    session.advance_frame(50_000_000).expect("fixed tick");
    assert_eq!(
        session.frame_driver().authority().delivered_sequences,
        vec![vec![1]]
    );
}

#[test]
fn queues_touch_interact_through_the_next_twenty_hz_commit() {
    let mut session = session();

    assert_eq!(
        session
            .dispatch_hud_route("woc.hud.touch.interact", true)
            .expect("HUD interact input"),
        None
    );
    assert_eq!(session.command_mapper().next_sequence(), Some(2));

    session.advance_frame(50_000_000).expect("fixed tick");
    assert_eq!(
        session.frame_driver().authority().delivered_sequences,
        vec![vec![1]]
    );
}

#[test]
fn movement_state_reaches_the_next_fixed_tick_without_consuming_a_command_sequence() {
    let mut session = session();
    let flags = MovementInputFlags {
        forward: true,
        strafe_left: true,
        ..MovementInputFlags::default()
    };

    session
        .set_movement_input(flags, Some(-0.25))
        .expect("valid movement state");
    assert_eq!(session.command_mapper().next_sequence(), Some(1));

    session.advance_frame(50_000_000).expect("fixed tick");
    assert_eq!(
        session.frame_driver().authority().delivered_movement,
        vec![MovementFrame {
            actor: EntityRef {
                id: 1,
                generation: 1,
            },
            sequence: 1,
            flags,
            facing: Some(-0.25),
        }]
    );
    assert_eq!(
        session.frame_driver().authority().delivered_sequences,
        vec![Vec::<u32>::new()]
    );
}

#[test]
fn movement_sources_resolve_once_before_the_next_fixed_tick() {
    let mut session = session();
    session
        .set_movement_sources(
            MovementInputSources {
                keyboard: MovementInputFlags {
                    turn_left: true,
                    ..MovementInputFlags::default()
                },
                mouse_camera: true,
                gamepad: GamepadMoveFlags {
                    forward: true,
                    ..GamepadMoveFlags::default()
                },
                ..MovementInputSources::default()
            },
            Some(0.125),
        )
        .expect("host movement sources are valid");

    session.advance_frame(50_000_000).expect("fixed tick");
    assert_eq!(
        session.frame_driver().authority().delivered_movement,
        vec![MovementFrame {
            actor: EntityRef {
                id: 1,
                generation: 1,
            },
            sequence: 1,
            flags: MovementInputFlags {
                forward: true,
                strafe_left: true,
                ..MovementInputFlags::default()
            },
            facing: Some(0.125),
        }]
    );
}

#[test]
fn forwards_hud_host_effect_without_consuming_an_authority_sequence() {
    let mut session = session();

    assert_eq!(
        session
            .dispatch_hud_route("woc.hud.touch.open_chat", true)
            .expect("host effect"),
        Some(HudHostEffect::OpenChat)
    );
    assert_eq!(session.command_mapper().next_sequence(), Some(1));
}

#[test]
fn full_queue_rejects_hud_authority_input_before_mapping_it() {
    let mut session = session();
    for sequence in 20..20 + MAX_PENDING_COMMANDS as u32 {
        session
            .frame_driver_mut()
            .queue_command(queued_command(sequence))
            .expect("fill command queue");
    }

    assert_eq!(
        session
            .dispatch_hud_route("woc.hud.touch.activate.0", true)
            .expect_err("queue is full"),
        ClientSessionHudRouteError::Input(ClientSessionInputError::Queue(
            ClientCommandQueueError::Full {
                maximum: MAX_PENDING_COMMANDS,
            },
        ))
    );
    assert_eq!(session.command_mapper().next_sequence(), Some(1));
}

#[test]
fn forwards_hud_route_errors_without_activating_the_host() {
    let mut session = session();

    assert_eq!(
        session
            .dispatch_hud_route("woc.hud.pause.open.graphics", true)
            .expect_err("closed pause menu"),
        ClientSessionHudRouteError::Route(HudRouteError::PauseClosed)
    );
}

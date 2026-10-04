use woc_protocol::{Command, EntityRef, MovementInputFlags};
use woc_runtime::{
    ClientPresentationProjection, PresentationCadence, PresentationSnapshot,
    PresentationTimelineError, PresentationTimelinePush, WocOfflineBootstrapError, WocProjectVm,
};

use crate::{
    resolve_movement_input, CharacterSortMode, ClientAuthority, ClientCommandMapper,
    ClientCommandQueueError, ClientFrameAdvance, ClientFrameDriver, ClientFrameDriverError,
    ClientFrameDriverInitError, ClientInputEvent, ClientInputMappingError,
    ClientMovementInputError, ClientStateProjection, HudHostEffect, HudRouteController,
    HudRouteEffect, HudRouteError, MovementInputSources, OfflineSessionLaunch,
    ShellRouteDispatchError, ShellRouteEffect, StateOnlyClientAuthority,
    TransactionalClientAuthority, WocShellController, MAX_PENDING_COMMANDS,
};

#[cfg(feature = "engine-host")]
use serde_json::json;
#[cfg(feature = "engine-host")]
use std::path::Path;
#[cfg(feature = "engine-host")]
use woc_protocol::MovementFrame;
#[cfg(feature = "engine-host")]
use woc_runtime::{
    load_engine_project_vm, TickBudgets, WocHostRole, WocProjectIdentity, WocReloadableVm,
    ZrVmProjectOptions, ZrVmProjectVm,
};

#[derive(Debug, PartialEq)]
pub enum ClientSessionInitError {
    Input(ClientInputMappingError),
    Frame(ClientFrameDriverInitError),
}

impl From<ClientInputMappingError> for ClientSessionInitError {
    fn from(error: ClientInputMappingError) -> Self {
        Self::Input(error)
    }
}

impl From<ClientFrameDriverInitError> for ClientSessionInitError {
    fn from(error: ClientFrameDriverInitError) -> Self {
        Self::Frame(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientSessionInputError {
    Mapping(ClientInputMappingError),
    Queue(ClientCommandQueueError),
}

impl From<ClientInputMappingError> for ClientSessionInputError {
    fn from(error: ClientInputMappingError) -> Self {
        Self::Mapping(error)
    }
}

impl From<ClientCommandQueueError> for ClientSessionInputError {
    fn from(error: ClientCommandQueueError) -> Self {
        Self::Queue(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientSessionHudRouteError {
    Route(HudRouteError),
    Input(ClientSessionInputError),
}

impl From<HudRouteError> for ClientSessionHudRouteError {
    fn from(error: HudRouteError) -> Self {
        Self::Route(error)
    }
}

impl From<ClientSessionInputError> for ClientSessionHudRouteError {
    fn from(error: ClientSessionInputError) -> Self {
        Self::Input(error)
    }
}

/// Host-neutral client composition. The host owns UI painting, VM construction and effects.
pub struct WocClientSession<A, P> {
    shell: WocShellController,
    hud: HudRouteController,
    command_mapper: ClientCommandMapper,
    frame_driver: ClientFrameDriver<A, P>,
}

impl<A, P> WocClientSession<A, P>
where
    A: ClientAuthority<P>,
{
    pub fn new(
        shell: WocShellController,
        actor: EntityRef,
        next_sequence: u32,
        authority: A,
        cadence: PresentationCadence,
        max_catch_up_ticks: u32,
    ) -> Result<Self, ClientSessionInitError> {
        Self::new_with_authority_recovery(
            shell,
            actor,
            next_sequence,
            authority,
            cadence,
            max_catch_up_ticks,
        )
        .map_err(|(error, _)| error)
    }

    /// Variant for a product host that must explicitly tear down authority if
    /// route or frame-driver construction fails.
    pub fn new_with_authority_recovery(
        shell: WocShellController,
        actor: EntityRef,
        next_sequence: u32,
        authority: A,
        cadence: PresentationCadence,
        max_catch_up_ticks: u32,
    ) -> Result<Self, (ClientSessionInitError, A)> {
        let command_mapper = match ClientCommandMapper::new(actor, next_sequence) {
            Ok(command_mapper) => command_mapper,
            Err(error) => return Err((ClientSessionInitError::Input(error), authority)),
        };
        let frame_driver = match ClientFrameDriver::new_with_authority_recovery(
            authority,
            cadence,
            max_catch_up_ticks,
            actor,
            next_sequence,
        ) {
            Ok(frame_driver) => frame_driver,
            Err((error, authority)) => {
                return Err((ClientSessionInitError::Frame(error), authority))
            }
        };
        Ok(Self {
            shell,
            hud: HudRouteController::default(),
            command_mapper,
            frame_driver,
        })
    }

    pub fn install_initial(
        &mut self,
        snapshot: PresentationSnapshot<P>,
    ) -> Result<PresentationTimelinePush, PresentationTimelineError> {
        self.frame_driver.install_initial(snapshot)
    }

    pub fn dispatch_shell_route(
        &mut self,
        route: &str,
        text_value: Option<&str>,
    ) -> Result<Option<ShellRouteEffect>, ShellRouteDispatchError> {
        self.shell.dispatch_shell_route(route, text_value)
    }

    /// Queues only existing authority inputs and returns host-owned HUD effects unchanged.
    pub fn dispatch_hud_route(
        &mut self,
        route: &str,
        online: bool,
    ) -> Result<Option<HudHostEffect>, ClientSessionHudRouteError> {
        match self.hud.dispatch_route(route, online)? {
            HudRouteEffect::Input(input) => {
                self.queue_input(input)?;
                Ok(None)
            }
            HudRouteEffect::Host(effect) => Ok(Some(effect)),
        }
    }

    /// Refuses a full batch before mapping so an undeliverable input never consumes a sequence.
    pub fn queue_input(
        &mut self,
        input: ClientInputEvent,
    ) -> Result<Command, ClientSessionInputError> {
        if self.frame_driver.pending_command_count() >= MAX_PENDING_COMMANDS {
            return Err(ClientCommandQueueError::Full {
                maximum: MAX_PENDING_COMMANDS,
            }
            .into());
        }
        let command = self.command_mapper.map(input)?;
        self.frame_driver.queue_command(command.clone())?;
        Ok(command)
    }

    /// Updates the held movement stream for the locally controlled actor. It
    /// remains outside the command mapper and is sampled only by fixed ticks.
    pub fn set_movement_input(
        &mut self,
        flags: MovementInputFlags,
        facing: Option<f64>,
    ) -> Result<(), ClientMovementInputError> {
        self.frame_driver.set_movement_input(flags, facing)
    }

    /// Resolves the target-compatible host-held sources once, then delegates to
    /// the sole 20 Hz movement stream without creating a second input channel.
    pub fn set_movement_sources(
        &mut self,
        sources: MovementInputSources,
        facing: Option<f64>,
    ) -> Result<(), ClientMovementInputError> {
        self.set_movement_input(resolve_movement_input(sources), facing)
    }

    pub fn advance_frame(
        &mut self,
        elapsed_ns: u64,
    ) -> Result<ClientFrameAdvance, ClientFrameDriverError<A::Error>> {
        self.frame_driver.advance_frame(elapsed_ns)
    }

    pub fn shell(&self) -> &WocShellController {
        &self.shell
    }

    pub fn shell_mut(&mut self) -> &mut WocShellController {
        &mut self.shell
    }

    pub fn hud(&self) -> &HudRouteController {
        &self.hud
    }

    pub fn hud_mut(&mut self) -> &mut HudRouteController {
        &mut self.hud
    }

    pub fn command_mapper(&self) -> &ClientCommandMapper {
        &self.command_mapper
    }

    pub fn frame_driver(&self) -> &ClientFrameDriver<A, P> {
        &self.frame_driver
    }

    pub fn frame_driver_mut(&mut self) -> &mut ClientFrameDriver<A, P> {
        &mut self.frame_driver
    }

    /// Transfers frame authority to the host after the session loop stops.
    /// The shell and route state are intentionally discarded at this boundary.
    pub fn into_authority(self) -> A {
        self.frame_driver.into_authority()
    }
}

impl<V> WocClientSession<TransactionalClientAuthority<V>, ClientPresentationProjection>
where
    V: WocProjectVm,
{
    /// Hosts call this after receiving `PrepareOfflineSession`; only the
    /// transactional authority may carry the resulting constructor into Tick 1.
    pub fn prepare_offline_session(
        &mut self,
        launch: &OfflineSessionLaunch,
    ) -> Result<(), WocOfflineBootstrapError> {
        self.frame_driver
            .authority_mut()
            .prepare_offline_session(launch)
    }
}

impl<V> WocClientSession<StateOnlyClientAuthority<V>, ClientStateProjection>
where
    V: WocProjectVm,
{
    /// Hosts call this after receiving `PrepareOfflineSession`; this path is
    /// explicitly state-only until the project publishes a presentation payload.
    pub fn prepare_offline_session(
        &mut self,
        launch: &OfflineSessionLaunch,
    ) -> Result<(), WocOfflineBootstrapError> {
        self.frame_driver
            .authority_mut()
            .prepare_offline_session(launch)
    }
}

#[cfg(feature = "engine-host")]
const CLIENT_TICK_NS: u64 = woc_protocol::SIMULATION_STEP_NS;
#[cfg(feature = "engine-host")]
const OFFLINE_PLAYER_ENTITY_ID: u64 = 408;
#[cfg(feature = "engine-host")]
const OFFLINE_PLAYER_GENERATION: u32 = 1;
#[cfg(feature = "engine-host")]
const DEFAULT_MAX_CATCH_UP_TICKS: u32 = 4;

/// Engine-owned lifecycle for the transactional client presentation role.
/// Native window and rendering remain unavailable to this host.
#[cfg(feature = "engine-host")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientProductPhase {
    Constructing,
    Ready,
    Running,
    Stopping,
    Stopped,
}

#[cfg(feature = "engine-host")]
impl ClientProductPhase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Constructing => "constructing",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
        }
    }
}

#[cfg(feature = "engine-host")]
type ClientProductSession =
    WocClientSession<TransactionalClientAuthority<ZrVmProjectVm>, ClientPresentationProjection>;

#[cfg(feature = "engine-host")]
pub struct ClientProductHost {
    identity: WocProjectIdentity,
    phase: ClientProductPhase,
    transitions: Vec<ClientProductPhase>,
    session: Option<ClientProductSession>,
}

#[cfg(feature = "engine-host")]
#[derive(Clone, Copy)]
struct ClientProductMetrics {
    committed_tick: u64,
    state_bytes: usize,
    state_digest: u32,
    event_digest: u32,
    presentation_digest: u32,
    sampled_presentation_frames: u64,
    replay_verified: bool,
}

#[cfg(feature = "engine-host")]
impl ClientProductHost {
    pub fn start(project: impl AsRef<Path>, launch: &OfflineSessionLaunch) -> Result<Self, String> {
        let mut transitions = vec![ClientProductPhase::Constructing];
        let (identity, mut vm) =
            load_engine_project_vm(project, WocHostRole::Client, ZrVmProjectOptions::default())
                .map_err(|error| format!("load client project VM: {error}"))?;
        if let Err(error) = vm.activate() {
            let cleanup = vm.deactivate().err();
            return Err(match cleanup {
                Some(cleanup) => {
                    format!("activate client project VM: {error:?}; cleanup: {cleanup:?}")
                }
                None => format!("activate client project VM: {error:?}"),
            });
        }

        // These are fixed protocol facts for the first offline bootstrap. The
        // project allocates the first player at entity 408, generation 1.
        let authority = TransactionalClientAuthority::new(vm, TickBudgets::default());
        let mut session = match WocClientSession::new_with_authority_recovery(
            WocShellController::new(true, CharacterSortMode::Level),
            EntityRef {
                id: OFFLINE_PLAYER_ENTITY_ID,
                generation: OFFLINE_PLAYER_GENERATION,
            },
            1,
            authority,
            PresentationCadence::woc_default(),
            DEFAULT_MAX_CATCH_UP_TICKS,
        ) {
            Ok(session) => session,
            Err((error, authority)) => {
                let cleanup = deactivate_client_authority(authority);
                return Err(match cleanup {
                    Some(cleanup) => {
                        format!("construct client session: {error:?}; cleanup: {cleanup}")
                    }
                    None => format!("construct client session: {error:?}"),
                });
            }
        };
        if let Err(error) = session.prepare_offline_session(launch) {
            let cleanup = deactivate_client_authority(session.into_authority());
            return Err(match cleanup {
                Some(cleanup) => {
                    format!("prepare offline session: {error:?}; cleanup: {cleanup}")
                }
                None => format!("prepare offline session: {error:?}"),
            });
        }
        transitions.push(ClientProductPhase::Ready);
        Ok(Self {
            identity,
            phase: ClientProductPhase::Ready,
            transitions,
            session: Some(session),
        })
    }

    pub fn run(mut self, ticks: u64, verify_replay: bool) -> Result<String, String> {
        self.transition(ClientProductPhase::Running);
        let run_result = self.run_frames(ticks, verify_replay);
        self.transition(ClientProductPhase::Stopping);
        let cleanup_result = self.shutdown();
        let metrics = match (run_result, cleanup_result) {
            (Ok(metrics), Ok(())) => metrics,
            (Err(error), Ok(())) => return Err(error),
            (Ok(_), Err(error)) => return Err(error),
            (Err(error), Err(cleanup)) => return Err(format!("{error}; shutdown: {cleanup}")),
        };

        serde_json::to_string_pretty(&json!({
            "identity": self.identity,
            "role": "client",
            "lifecycle": self.transitions.iter().map(|phase| phase.as_str()).collect::<Vec<_>>(),
            "phase": self.phase.as_str(),
            "realVm": true,
            "stateOnly": false,
            "presentationReady": metrics.sampled_presentation_frames > 0,
            "nativeWindowReady": false,
            "renderedFrames": 0,
            "sampledPresentationFrames": metrics.sampled_presentation_frames,
            "committedTick": metrics.committed_tick,
            "stateBytes": metrics.state_bytes,
            "stateDigest": metrics.state_digest,
            "eventDigest": metrics.event_digest,
            "presentationDigest": metrics.presentation_digest,
            "replayVerified": metrics.replay_verified,
        }))
        .map_err(|error| format!("serialize client report: {error}"))
    }

    fn run_frames(
        &mut self,
        ticks: u64,
        verify_replay: bool,
    ) -> Result<ClientProductMetrics, String> {
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| "client session is not available".to_owned())?;
        let mut sampled_presentation_frames = 0;
        for _ in 0..ticks {
            session
                .advance_frame(CLIENT_TICK_NS)
                .map_err(|error| format!("client frame: {error:?}"))?;
            if session.frame_driver().sample().is_some() {
                sampled_presentation_frames += 1;
            }
        }
        if verify_replay {
            verify_client_runtime_replay(session)?;
        }
        let committed = session.frame_driver().authority().runtime().committed();
        Ok(ClientProductMetrics {
            committed_tick: committed.tick,
            state_bytes: committed.state.len(),
            state_digest: committed.state_digest,
            event_digest: committed.event_digest,
            presentation_digest: committed.presentation_digest,
            sampled_presentation_frames,
            replay_verified: verify_replay,
        })
    }

    fn shutdown(&mut self) -> Result<(), String> {
        let session = self
            .session
            .take()
            .ok_or_else(|| "client session was already stopped".to_owned())?;
        let authority = session.into_authority();
        let mut vm = authority.into_vm();
        vm.deactivate()
            .map_err(|error| format!("deactivate client project VM: {error:?}"))?;
        self.transition(ClientProductPhase::Stopped);
        Ok(())
    }

    fn transition(&mut self, phase: ClientProductPhase) {
        self.phase = phase;
        self.transitions.push(phase);
    }
}

#[cfg(feature = "engine-host")]
fn deactivate_client_authority(
    authority: TransactionalClientAuthority<ZrVmProjectVm>,
) -> Option<String> {
    let mut vm = authority.into_vm();
    vm.deactivate()
        .err()
        .map(|error| format!("deactivate client project VM: {error:?}"))
}

#[cfg(feature = "engine-host")]
fn verify_client_runtime_replay(session: &mut ClientProductSession) -> Result<(), String> {
    let runtime = session.frame_driver_mut().authority_mut().runtime_mut();
    let movement = MovementFrame {
        actor: EntityRef {
            id: OFFLINE_PLAYER_ENTITY_ID,
            generation: OFFLINE_PLAYER_GENERATION,
        },
        sequence: 1,
        flags: MovementInputFlags::default(),
        facing: None,
    };
    let checkpoint = runtime
        .checkpoint()
        .map_err(|error| format!("replay checkpoint: {error:?}"))?;
    let expected = {
        let (snapshot, _) = runtime
            .tick_with_projection_and_movement(Vec::new(), vec![movement], |bytes| {
                ClientPresentationProjection::decode_json(bytes)
                    .map_err(|error| format!("{error:?}"))
            })
            .map_err(|error| format!("replay first tick: {error:?}"))?;
        snapshot.clone()
    };
    runtime
        .rollback_checkpoint(checkpoint)
        .map_err(|error| format!("replay rollback: {error:?}"))?;
    let checkpoint = runtime
        .checkpoint()
        .map_err(|error| format!("replay second checkpoint: {error:?}"))?;
    let actual = {
        let (snapshot, _) = runtime
            .tick_with_projection_and_movement(Vec::new(), vec![movement], |bytes| {
                ClientPresentationProjection::decode_json(bytes)
                    .map_err(|error| format!("{error:?}"))
            })
            .map_err(|error| format!("replay second tick: {error:?}"))?;
        snapshot.clone()
    };
    runtime
        .rollback_checkpoint(checkpoint)
        .map_err(|error| format!("replay final rollback: {error:?}"))?;
    if actual != expected {
        return Err("client checkpoint replay changed the committed snapshot".into());
    }
    Ok(())
}

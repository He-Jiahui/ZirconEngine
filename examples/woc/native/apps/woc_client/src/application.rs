use woc_protocol::{Command, EntityRef, MovementInputFlags};
use woc_runtime::{
    ClientPresentationProjection, PresentationCadence, PresentationSnapshot,
    PresentationTimelineError, PresentationTimelinePush, WocOfflineBootstrapError, WocProjectVm,
};

use crate::{
    resolve_movement_input, ClientAuthority, ClientCommandMapper, ClientCommandQueueError,
    ClientFrameAdvance, ClientFrameDriver, ClientFrameDriverError, ClientFrameDriverInitError,
    ClientInputEvent, ClientInputMappingError, ClientMovementInputError, HudHostEffect,
    HudRouteController, HudRouteEffect, HudRouteError, MovementInputSources, OfflineSessionLaunch,
    ShellRouteDispatchError, ShellRouteEffect, TransactionalClientAuthority, WocShellController,
    MAX_PENDING_COMMANDS,
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
        Ok(Self {
            shell,
            hud: HudRouteController::default(),
            command_mapper: ClientCommandMapper::new(actor, next_sequence)?,
            frame_driver: ClientFrameDriver::new(
                authority,
                cadence,
                max_catch_up_ticks,
                actor,
                next_sequence,
            )?,
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

use woc_protocol::{Command, EntityRef, MovementInputFlags};
use woc_runtime::{
    ActorPresentation, ActorTransform, ClientPresentationProjection, PresentationCadence,
    PresentationProjectionError, PresentationSample, PresentationSnapshot, PresentationTimeline,
    PresentationTimelineError, PresentationTimelinePush, WocProjectVm,
};

use super::{
    ClientAuthority, ClientCommandQueueError, ClientFrameAdvance, ClientFrameDriverError,
    ClientFrameDriverInitError, ClientMovementInputError, ClientMovementStream,
    ClientPresentedFrame, ClientTickInput, TransactionalClientAuthority, MAX_PENDING_COMMANDS,
};

pub struct ClientFrameDriver<A, P> {
    authority: A,
    timeline: PresentationTimeline<P>,
    accumulator_ns: u64,
    presentation_time_ns: u64,
    pending_commands: Vec<Command>,
    movement: ClientMovementStream,
    max_catch_up_ticks: u32,
}

impl<A, P> ClientFrameDriver<A, P>
where
    A: ClientAuthority<P>,
{
    pub fn new(
        authority: A,
        cadence: PresentationCadence,
        max_catch_up_ticks: u32,
        movement_actor: EntityRef,
        next_movement_sequence: u32,
    ) -> Result<Self, ClientFrameDriverInitError> {
        if max_catch_up_ticks == 0 {
            return Err(ClientFrameDriverInitError::ZeroCatchUpBudget);
        }
        Ok(Self {
            authority,
            timeline: PresentationTimeline::new(cadence),
            accumulator_ns: 0,
            presentation_time_ns: 0,
            pending_commands: Vec::with_capacity(MAX_PENDING_COMMANDS),
            movement: ClientMovementStream::new(movement_actor, next_movement_sequence)
                .map_err(ClientFrameDriverInitError::Movement)?,
            max_catch_up_ticks,
        })
    }

    pub fn install_initial(
        &mut self,
        snapshot: PresentationSnapshot<P>,
    ) -> Result<PresentationTimelinePush, PresentationTimelineError> {
        let received_at_ns = snapshot.received_at_ns;
        let result = self.timeline.push(snapshot)?;
        self.presentation_time_ns = self.presentation_time_ns.max(received_at_ns);
        Ok(result)
    }

    pub fn queue_command(&mut self, command: Command) -> Result<(), ClientCommandQueueError> {
        if self.pending_commands.len() >= MAX_PENDING_COMMANDS {
            return Err(ClientCommandQueueError::Full {
                maximum: MAX_PENDING_COMMANDS,
            });
        }
        self.pending_commands.push(command);
        Ok(())
    }

    /// Replaces host-derived held movement without creating a command or
    /// consuming a movement sequence. Sampling occurs at the next 20 Hz step.
    pub fn set_movement_input(
        &mut self,
        flags: MovementInputFlags,
        facing: Option<f64>,
    ) -> Result<(), ClientMovementInputError> {
        self.movement.set_input(flags, facing)
    }

    pub fn advance_frame(
        &mut self,
        elapsed_ns: u64,
    ) -> Result<ClientFrameAdvance, ClientFrameDriverError<A::Error>> {
        self.presentation_time_ns = self.presentation_time_ns.saturating_add(elapsed_ns);
        self.accumulator_ns = self.accumulator_ns.saturating_add(elapsed_ns);
        let step_ns = self.timeline.cadence().simulation_step_ns();
        let mut committed_ticks = 0;

        while self.accumulator_ns >= step_ns && committed_ticks < self.max_catch_up_ticks {
            let scheduled_at_ns = self
                .presentation_time_ns
                .saturating_sub(self.accumulator_ns)
                .saturating_add(step_ns);
            let movement = self
                .movement
                .frame()
                .map_err(ClientFrameDriverError::Movement)?;
            let input = ClientTickInput::new(&self.pending_commands, movement);
            let mut snapshot = self
                .authority
                .fixed_step(input, scheduled_at_ns)
                .map_err(ClientFrameDriverError::Authority)?;
            snapshot.received_at_ns = scheduled_at_ns;

            let push_result = self.timeline.push(snapshot);
            self.accumulator_ns -= step_ns;
            self.pending_commands.clear();
            self.movement.commit();
            committed_ticks += 1;
            push_result.map_err(ClientFrameDriverError::Timeline)?;
        }

        Ok(ClientFrameAdvance {
            committed_ticks,
            backlog_ticks: self.accumulator_ns / step_ns,
        })
    }

    pub fn sample(&self) -> Option<PresentationSample<'_, P>> {
        self.timeline.sample(self.presentation_time_ns)
    }

    pub fn authority(&self) -> &A {
        &self.authority
    }

    pub fn authority_mut(&mut self) -> &mut A {
        &mut self.authority
    }

    pub fn pending_command_count(&self) -> usize {
        self.pending_commands.len()
    }

    pub fn presentation_time_ns(&self) -> u64 {
        self.presentation_time_ns
    }

    pub fn accumulator_ns(&self) -> u64 {
        self.accumulator_ns
    }
}

impl<V> ClientFrameDriver<TransactionalClientAuthority<V>, ClientPresentationProjection>
where
    V: WocProjectVm,
{
    pub fn visit_presented_actors(
        &self,
        visitor: impl FnMut(&ActorPresentation, ActorTransform),
    ) -> Result<Option<ClientPresentedFrame<'_>>, PresentationProjectionError> {
        let Some(sample) = self.sample() else {
            return Ok(None);
        };
        sample
            .to
            .world
            .visit_interpolated_from(&sample.from.world, sample.alpha, visitor)?;
        Ok(Some(ClientPresentedFrame {
            hud: &sample.to.hud,
            blend_mode: sample.mode,
            alpha: sample.alpha,
        }))
    }
}

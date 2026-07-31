//! Host-time scheduling for one authoritative ZrVM transaction per 20 Hz boundary.

use std::collections::BTreeSet;

use woc_protocol::{Command, MovementFrame, MovementFrameBatch, MovementInputError};
use woc_runtime::{RuntimeRole, TickBudgets, WocProjectVm, WocTickFault, WocTransactionalRuntime};

pub const SERVER_TICK_NS: u64 = 50_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerTickDriverInitError {
    ZeroCatchUpBudget,
    ZeroCommandQueueBudget,
    ZeroMovementQueueBudget,
}

#[derive(Debug)]
pub enum ServerTickInputError {
    CommandQueueFull {
        maximum: usize,
    },
    DuplicateCommandSequence {
        actor_id: u64,
        generation: u32,
        sequence: u32,
    },
    MovementQueueFull {
        maximum: usize,
    },
    Movement(MovementInputError),
}

impl From<MovementInputError> for ServerTickInputError {
    fn from(error: MovementInputError) -> Self {
        Self::Movement(error)
    }
}

#[derive(Debug)]
pub enum ServerTickDriverError {
    Tick(WocTickFault),
    Movement(MovementInputError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerTickInputBatch {
    pub commands: Vec<Command>,
    pub movement_frames: Vec<MovementFrame>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerTickAdvance {
    pub committed_ticks: u32,
    pub backlog_ticks: u64,
}

/// Owns host clock accumulation and ingress backpressure, never gameplay state.
pub struct FixedServerTickDriver<V> {
    runtime: WocTransactionalRuntime<V>,
    accumulator_ns: u64,
    pending_commands: Vec<Command>,
    pending_command_sequences: BTreeSet<(u64, u32, u32)>,
    pending_movement: Vec<MovementFrame>,
    pending_movement_actors: BTreeSet<(u64, u32)>,
    max_catch_up_ticks: u32,
    max_pending_commands: usize,
    max_pending_movement: usize,
    last_failed_input: Option<ServerTickInputBatch>,
}

impl<V: WocProjectVm> FixedServerTickDriver<V> {
    pub fn new(
        vm: V,
        budgets: TickBudgets,
        max_catch_up_ticks: u32,
        max_pending_commands: usize,
        max_pending_movement: usize,
    ) -> Result<Self, ServerTickDriverInitError> {
        if max_catch_up_ticks == 0 {
            return Err(ServerTickDriverInitError::ZeroCatchUpBudget);
        }
        if max_pending_commands == 0 {
            return Err(ServerTickDriverInitError::ZeroCommandQueueBudget);
        }
        if max_pending_movement == 0 {
            return Err(ServerTickDriverInitError::ZeroMovementQueueBudget);
        }
        Ok(Self {
            runtime: WocTransactionalRuntime::new(RuntimeRole::Server, vm, budgets),
            accumulator_ns: 0,
            pending_commands: Vec::with_capacity(max_pending_commands),
            pending_command_sequences: BTreeSet::new(),
            pending_movement: Vec::with_capacity(max_pending_movement),
            pending_movement_actors: BTreeSet::new(),
            max_catch_up_ticks,
            max_pending_commands,
            max_pending_movement,
            last_failed_input: None,
        })
    }

    /// Adds a whole ingress batch or rejects it before mutating the pending queue.
    pub fn enqueue_commands(&mut self, commands: Vec<Command>) -> Result<(), ServerTickInputError> {
        if commands.len() > self.max_pending_commands - self.pending_commands.len() {
            return Err(ServerTickInputError::CommandQueueFull {
                maximum: self.max_pending_commands,
            });
        }

        let mut incoming_sequences = BTreeSet::new();
        for command in &commands {
            let key = (command.actor.id, command.actor.generation, command.sequence);
            if self.pending_command_sequences.contains(&key) || !incoming_sequences.insert(key) {
                return Err(ServerTickInputError::DuplicateCommandSequence {
                    actor_id: command.actor.id,
                    generation: command.actor.generation,
                    sequence: command.sequence,
                });
            }
        }

        self.pending_command_sequences.extend(incoming_sequences);
        self.pending_commands.extend(commands);
        Ok(())
    }

    /// Validates every frame and rejects a duplicate actor before it can enter a VM tick.
    pub fn enqueue_movement(
        &mut self,
        frames: Vec<MovementFrame>,
    ) -> Result<(), ServerTickInputError> {
        if frames.len() > self.max_pending_movement - self.pending_movement.len() {
            return Err(ServerTickInputError::MovementQueueFull {
                maximum: self.max_pending_movement,
            });
        }

        let mut incoming_actors = BTreeSet::new();
        for frame in &frames {
            frame.validate()?;
            let actor = (frame.actor.id, frame.actor.generation);
            if self.pending_movement_actors.contains(&actor) || !incoming_actors.insert(actor) {
                return Err(ServerTickInputError::Movement(
                    MovementInputError::DuplicateActor {
                        actor_id: frame.actor.id,
                        generation: frame.actor.generation,
                    },
                ));
            }
        }

        self.pending_movement_actors.extend(incoming_actors);
        self.pending_movement.extend(frames);
        Ok(())
    }

    /// Advances host scheduling only. Wall-clock values never enter the authoritative input bytes.
    pub fn advance(&mut self, elapsed_ns: u64) -> Result<ServerTickAdvance, ServerTickDriverError> {
        self.accumulator_ns = self.accumulator_ns.saturating_add(elapsed_ns);
        let mut committed_ticks = 0;

        while self.accumulator_ns >= SERVER_TICK_NS && committed_ticks < self.max_catch_up_ticks {
            let mut commands = std::mem::take(&mut self.pending_commands);
            self.pending_command_sequences.clear();
            canonicalize_commands(&mut commands);
            let movement_frames = std::mem::take(&mut self.pending_movement);
            self.pending_movement_actors.clear();
            let movement_batch = match MovementFrameBatch::new(movement_frames.clone()) {
                Ok(batch) => batch,
                Err(error) => {
                    self.pending_commands = commands;
                    self.pending_command_sequences = self
                        .pending_commands
                        .iter()
                        .map(command_sequence_key)
                        .collect();
                    self.pending_movement = movement_frames;
                    self.pending_movement_actors = self
                        .pending_movement
                        .iter()
                        .map(|frame| (frame.actor.id, frame.actor.generation))
                        .collect();
                    return Err(ServerTickDriverError::Movement(error));
                }
            };
            let diagnostic = ServerTickInputBatch {
                commands: commands.clone(),
                movement_frames: movement_batch.frames().to_vec(),
            };

            match self
                .runtime
                .tick_with_movement(commands, diagnostic.movement_frames.clone())
            {
                Ok(_) => {
                    self.accumulator_ns -= SERVER_TICK_NS;
                    committed_ticks += 1;
                    self.last_failed_input = None;
                }
                Err(fault) => {
                    self.last_failed_input = Some(diagnostic);
                    return Err(ServerTickDriverError::Tick(fault));
                }
            }
        }

        Ok(ServerTickAdvance {
            committed_ticks,
            backlog_ticks: self.accumulator_ns / SERVER_TICK_NS,
        })
    }

    pub fn runtime(&self) -> &WocTransactionalRuntime<V> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut WocTransactionalRuntime<V> {
        &mut self.runtime
    }

    pub fn pending_command_count(&self) -> usize {
        self.pending_commands.len()
    }

    pub fn pending_movement_count(&self) -> usize {
        self.pending_movement.len()
    }

    pub fn last_failed_input(&self) -> Option<&ServerTickInputBatch> {
        self.last_failed_input.as_ref()
    }

    pub fn accumulator_ns(&self) -> u64 {
        self.accumulator_ns
    }
}

fn command_sequence_key(command: &Command) -> (u64, u32, u32) {
    (command.actor.id, command.actor.generation, command.sequence)
}

fn canonicalize_commands(commands: &mut [Command]) {
    commands.sort_by(|left, right| {
        (
            left.actor.id,
            left.actor.generation,
            left.sequence,
            left.command_id,
            &left.payload,
        )
            .cmp(&(
                right.actor.id,
                right.actor.generation,
                right.sequence,
                right.command_id,
                &right.payload,
            ))
    });
}

//! Host-time scheduling for one authoritative ZrVM transaction per 20 Hz boundary.

use std::collections::BTreeSet;

use woc_protocol::{Command, MovementFrame, MovementInputError, MAX_MOVEMENT_FRAMES_PER_TICK};
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
            let mut movement_frames = std::mem::take(&mut self.pending_movement);
            self.pending_movement_actors.clear();
            if let Err(error) = canonicalize_pending_movement(&mut movement_frames) {
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
            let diagnostic = ServerTickInputBatch {
                commands: commands.clone(),
                movement_frames: movement_frames.clone(),
            };

            match self.runtime.tick_with_movement(commands, movement_frames) {
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

fn canonicalize_pending_movement(
    movement_frames: &mut [MovementFrame],
) -> Result<(), MovementInputError> {
    if movement_frames.len() > MAX_MOVEMENT_FRAMES_PER_TICK {
        return Err(MovementInputError::TooManyFrames {
            actual: movement_frames.len(),
            maximum: MAX_MOVEMENT_FRAMES_PER_TICK,
        });
    }
    movement_frames.sort_by_key(|frame| (frame.actor.id, frame.actor.generation));
    Ok(())
}

#[cfg(test)]
mod performance_tests {
    use std::{hint::black_box, time::Instant};

    use woc_protocol::{EntityRef, MovementFrameBatch, MovementInputFlags};

    use super::*;

    const FRAMES_PER_BATCH: usize = 32_768;
    const ITERATIONS: usize = 4;
    const SAMPLE_PAIRS: usize = 21;
    const THRESHOLD_PERCENT: u64 = 35;

    fn fixture() -> Vec<MovementFrame> {
        (0..FRAMES_PER_BATCH)
            .rev()
            .map(|index| MovementFrame {
                actor: EntityRef {
                    id: index as u64 + 1,
                    generation: (index % 7) as u32,
                },
                sequence: index as u32 + 1,
                flags: MovementInputFlags {
                    forward: true,
                    turn_right: true,
                    jump: index % 2 == 0,
                    ..MovementInputFlags::default()
                },
                facing: Some(index as f64 / FRAMES_PER_BATCH as f64),
            })
            .collect()
    }

    fn prepared_inputs(fixture: &[MovementFrame]) -> Vec<Vec<MovementFrame>> {
        (0..ITERATIONS).map(|_| fixture.to_vec()).collect()
    }

    fn consume(frames: &[MovementFrame]) -> u64 {
        frames
            .iter()
            .fold(0x517c_c1b7_2722_0a95, |checksum, frame| {
                checksum.rotate_left(7)
                    ^ frame.actor.id
                    ^ u64::from(frame.actor.generation).rotate_left(11)
                    ^ u64::from(frame.sequence).rotate_left(23)
                    ^ u64::from(frame.flags.forward)
            })
    }

    fn measure_legacy(fixture: &[MovementFrame]) -> u64 {
        let inputs = prepared_inputs(fixture);
        let started = Instant::now();
        let mut checksum = 0;
        for movement_frames in inputs {
            let movement_batch = MovementFrameBatch::new(movement_frames.clone())
                .expect("fixture must produce a valid movement batch");
            let diagnostic = movement_batch.frames().to_vec();
            let runtime = diagnostic.clone();
            checksum = checksum
                .wrapping_add(consume(&diagnostic))
                .wrapping_add(consume(&runtime).rotate_left(17));
        }
        black_box(checksum);
        started.elapsed().as_nanos() as u64
    }

    fn measure_transferred(fixture: &[MovementFrame]) -> u64 {
        let inputs = prepared_inputs(fixture);
        let started = Instant::now();
        let mut checksum = 0;
        for mut movement_frames in inputs {
            canonicalize_pending_movement(&mut movement_frames)
                .expect("fixture must fit the protocol bound");
            let diagnostic = movement_frames.clone();
            let runtime = movement_frames;
            checksum = checksum
                .wrapping_add(consume(&diagnostic))
                .wrapping_add(consume(&runtime).rotate_left(17));
        }
        black_box(checksum);
        started.elapsed().as_nanos() as u64
    }

    fn sample_csv(samples: &[u64]) -> String {
        samples
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn reduction_percent(legacy: u64, transferred: u64) -> u64 {
        legacy.saturating_sub(transferred).saturating_mul(100) / legacy.max(1)
    }

    #[test]
    #[ignore = "release performance evidence; run through the coordinator"]
    fn woc_app05_movement_transfer_release_benchmark_evidence() {
        let fixture = fixture();
        let legacy = MovementFrameBatch::new(fixture.clone())
            .expect("fixture must produce a valid movement batch");
        let mut transferred = fixture.clone();
        canonicalize_pending_movement(&mut transferred)
            .expect("fixture must fit the protocol bound");
        assert_eq!(legacy.frames(), transferred.as_slice());

        for _ in 0..4 {
            black_box(measure_legacy(&fixture));
            black_box(measure_transferred(&fixture));
        }

        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut transferred_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_ns.push(measure_legacy(&fixture));
                transferred_ns.push(measure_transferred(&fixture));
            } else {
                transferred_ns.push(measure_transferred(&fixture));
                legacy_ns.push(measure_legacy(&fixture));
            }
        }

        let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let transferred_p50_ns = nearest_rank(&transferred_ns, 50);
        let transferred_p95_ns = nearest_rank(&transferred_ns, 95);
        let p50_reduction_percent = reduction_percent(legacy_p50_ns, transferred_p50_ns);
        let p95_reduction_percent = reduction_percent(legacy_p95_ns, transferred_p95_ns);

        println!(
            "WOC_APP05_MOVEMENT_TRANSFER_PERF frames_per_batch=32768 iterations=4 \
             sample_pairs=21 sample_order=alternating_legacy_first_even \
             percentile_method=nearest_rank threshold_percent=35 \
             legacy_full_vector_copies=3 transferred_full_vector_copies=1 \
             copy_reduction_percent=66 legacy_p50_ns={legacy_p50_ns} \
             transferred_p50_ns={transferred_p50_ns} \
             p50_reduction_percent={p50_reduction_percent} \
             legacy_p95_ns={legacy_p95_ns} transferred_p95_ns={transferred_p95_ns} \
             p95_reduction_percent={p95_reduction_percent} \
             legacy_ns={} transferred_ns={}",
            sample_csv(&legacy_ns),
            sample_csv(&transferred_ns)
        );

        assert!(
            p50_reduction_percent >= THRESHOLD_PERCENT,
            "ownership transfer must improve P50 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p50_ns}ns transferred={transferred_p50_ns}ns"
        );
        assert!(
            p95_reduction_percent >= THRESHOLD_PERCENT,
            "ownership transfer must improve P95 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p95_ns}ns transferred={transferred_p95_ns}ns"
        );
    }
}

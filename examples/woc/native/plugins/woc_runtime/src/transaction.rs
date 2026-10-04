use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, FixedTickInputRef, MovementFrame,
    OfflineSessionBootstrap, ProtocolError, WorldSnapshot, FNV1A_OFFSET,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRole {
    Offline,
    Server,
    Client,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickBudgets {
    pub max_execution_micros: u64,
    pub max_memory_bytes: u64,
    pub max_host_calls: u64,
    pub max_gc_micros: u64,
}

impl Default for TickBudgets {
    fn default() -> Self {
        Self {
            max_execution_micros: 40_000,
            max_memory_bytes: 128 * 1024 * 1024,
            max_host_calls: 4_096,
            max_gc_micros: 10_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickUsage {
    pub execution_micros: u64,
    pub memory_bytes: u64,
    pub host_calls: u64,
    pub gc_micros: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetKind {
    Execution,
    Memory,
    HostCalls,
    GarbageCollection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmExecutionTermination {
    InstructionLimit,
    Deadline,
    Cancelled,
    HeapLimit,
    NativeCallLimit,
    GcTimeLimit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmTickError {
    Trap(String),
    Limited(String),
    BudgetExceeded(BudgetKind),
    RejectedCommand {
        index: usize,
        reason: String,
    },
    Transport(String),
    Terminated {
        reason: VmExecutionTermination,
        executed_instructions: u64,
        elapsed_micros: u64,
        usage: TickUsage,
    },
    Rollback {
        source: Box<VmTickError>,
        rollback: Box<VmTickError>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmTickResult {
    pub output_payload: Vec<u8>,
    pub presentation_payload: Vec<u8>,
    pub usage: TickUsage,
}

pub trait WocProjectVm {
    /// Owns an opaque checkpoint until its transaction is committed or aborted.
    type Checkpoint;

    /// Captures every VM-owned mutable value that a fixed tick can change.
    ///
    /// A runtime transaction never treats an adapter as rollback-capable by
    /// default: adapters must provide a real checkpoint and restore path.
    fn checkpoint(&mut self) -> Result<Self::Checkpoint, VmTickError>;

    /// Restores a checkpoint produced by [`Self::checkpoint`].
    fn rollback(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), VmTickError>;

    /// Replaces the VM-owned state with a validated authoritative snapshot.
    fn install_full_snapshot(&mut self, snapshot: &CommittedSnapshot) -> Result<(), VmTickError>;

    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError>;
}

pub trait WocReloadableVm: WocProjectVm {
    fn state_schema(&self) -> Result<String, VmTickError>;

    /// Returns canonical world-state bytes compatible with
    /// [`CommittedSnapshot::state`]. These bytes are passed through schema
    /// migration and restored into a replacement VM.
    fn save_state(&mut self) -> Result<Vec<u8>, VmTickError>;
    fn deactivate(&mut self) -> Result<(), VmTickError>;
    fn activate(&mut self) -> Result<(), VmTickError>;
    /// Restores canonical world-state bytes from [`Self::save_state`]. This
    /// does not replace the opaque all-state checkpoint contract on
    /// [`WocProjectVm`].
    fn restore_state(&mut self, state: &[u8]) -> Result<(), VmTickError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmReloadStage {
    Generation,
    Save,
    Deactivate,
    Load,
    Migrate,
    Activate,
    Restore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WocReloadError {
    pub stage: VmReloadStage,
    pub source: VmTickError,
    pub rollback_error: Option<VmTickError>,
    pub checkpoint_rollback_error: Option<VmTickError>,
    pub replacement_cleanup_error: Option<VmTickError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedSnapshot {
    pub generation: u64,
    pub tick: u64,
    pub state: Vec<u8>,
    pub state_digest: u32,
    pub event_digest: u32,
    pub presentation_digest: u32,
    pub presentation_payload: Vec<u8>,
}

impl Default for CommittedSnapshot {
    fn default() -> Self {
        Self {
            generation: 0,
            tick: 0,
            state: Vec::new(),
            state_digest: FNV1A_OFFSET,
            event_digest: FNV1A_OFFSET,
            presentation_digest: FNV1A_OFFSET,
            presentation_payload: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WocTickFaultKind {
    SessionNotRunning,
    /// The committed tick has no representable successor; no VM call occurred.
    TickExhausted,
    EncodeInput(ProtocolError),
    Vm(VmTickError),
    Budget {
        budget: BudgetKind,
        actual: u64,
        maximum: u64,
    },
    DecodeOutput(ProtocolError),
    DecodePresentation(String),
    TickMismatch {
        actual: u64,
        expected: u64,
    },
    StateDigestMismatch {
        actual: u32,
        expected: u32,
    },
    EventDigestMismatch {
        actual: u32,
        expected: u32,
    },
    PresentationDigestMismatch {
        actual: u32,
        expected: u32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct WocTickFault {
    /// Requested successor, or the last committed tick when it is exhausted.
    pub attempted_tick: u64,
    pub kind: WocTickFaultKind,
    /// The original fault is retained in `kind`; this records a failed attempt
    /// to restore VM-owned mutable state after it.
    pub rollback_error: Option<VmTickError>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeStatus {
    Running,
    Paused(WocTickFault),
    Faulted(WocTickFault),
    Recovering(WocTickFault),
}

#[derive(Clone, Debug, PartialEq)]
pub enum WocOfflineBootstrapError {
    ServerRole,
    SessionAlreadyStarted { tick: u64 },
    Invalid(ProtocolError),
}

pub struct WocTransactionalRuntime<V> {
    role: RuntimeRole,
    vm: V,
    budgets: TickBudgets,
    committed: CommittedSnapshot,
    status: RuntimeStatus,
    offline_bootstrap: Option<OfflineSessionBootstrap>,
}

pub struct WocRuntimeCheckpoint<C> {
    vm: C,
    committed: CommittedSnapshot,
    status: RuntimeStatus,
    offline_bootstrap: Option<OfflineSessionBootstrap>,
}

struct TickCandidate<C> {
    snapshot: CommittedSnapshot,
    checkpoint: C,
}

impl<V: WocProjectVm> WocTransactionalRuntime<V> {
    pub fn new(role: RuntimeRole, vm: V, budgets: TickBudgets) -> Self {
        Self {
            role,
            vm,
            budgets,
            committed: CommittedSnapshot::default(),
            status: RuntimeStatus::Running,
            offline_bootstrap: None,
        }
    }

    /// Schedules a source-derived offline session constructor for the first
    /// authoritative tick. It is retained across faults and consumed only by a
    /// successful transaction.
    pub fn install_offline_bootstrap(
        &mut self,
        bootstrap: OfflineSessionBootstrap,
    ) -> Result<(), WocOfflineBootstrapError> {
        if self.role == RuntimeRole::Server {
            return Err(WocOfflineBootstrapError::ServerRole);
        }
        if self.committed.tick != 0 || !self.committed.state.is_empty() {
            return Err(WocOfflineBootstrapError::SessionAlreadyStarted {
                tick: self.committed.tick,
            });
        }
        bootstrap
            .validate()
            .map_err(WocOfflineBootstrapError::Invalid)?;
        self.offline_bootstrap = Some(bootstrap);
        Ok(())
    }

    pub fn offline_bootstrap(&self) -> Option<&OfflineSessionBootstrap> {
        self.offline_bootstrap.as_ref()
    }

    pub fn tick(&mut self, commands: Vec<Command>) -> Result<&CommittedSnapshot, WocTickFault> {
        self.tick_with_movement(commands, Vec::new())
    }

    pub fn tick_with_movement(
        &mut self,
        commands: Vec<Command>,
        movement_frames: Vec<MovementFrame>,
    ) -> Result<&CommittedSnapshot, WocTickFault> {
        let candidate = self.prepare_tick(commands, movement_frames)?;
        self.commit_candidate(candidate);
        Ok(&self.committed)
    }

    pub fn tick_with_projection<P>(
        &mut self,
        commands: Vec<Command>,
        decode: impl FnOnce(&[u8]) -> Result<P, String>,
    ) -> Result<(&CommittedSnapshot, P), WocTickFault> {
        self.tick_with_projection_and_movement(commands, Vec::new(), decode)
    }

    pub fn tick_with_projection_and_movement<P>(
        &mut self,
        commands: Vec<Command>,
        movement_frames: Vec<MovementFrame>,
        decode: impl FnOnce(&[u8]) -> Result<P, String>,
    ) -> Result<(&CommittedSnapshot, P), WocTickFault> {
        let candidate = self.prepare_tick(commands, movement_frames)?;
        let projection = match decode(&candidate.snapshot.presentation_payload) {
            Ok(projection) => projection,
            Err(reason) => {
                return Err(
                    self.fail_candidate(candidate, WocTickFaultKind::DecodePresentation(reason))
                );
            }
        };
        self.commit_candidate(candidate);
        Ok((&self.committed, projection))
    }

    fn prepare_tick(
        &mut self,
        commands: Vec<Command>,
        movement_frames: Vec<MovementFrame>,
    ) -> Result<TickCandidate<V::Checkpoint>, WocTickFault> {
        let next_tick = self.committed.tick.checked_add(1);
        if self.status != RuntimeStatus::Running {
            return Err(WocTickFault {
                attempted_tick: next_tick.unwrap_or(self.committed.tick),
                kind: WocTickFaultKind::SessionNotRunning,
                rollback_error: None,
            });
        }
        let attempted_tick = match next_tick {
            Some(tick) => tick,
            None => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick: self.committed.tick,
                    kind: WocTickFaultKind::TickExhausted,
                    rollback_error: None,
                }));
            }
        };

        let input = FixedTickInputRef {
            tick: attempted_tick,
            commands: &commands,
            wall_time_forbidden: true,
            committed_state: &self.committed.state,
            committed_state_digest: self.committed.state_digest,
            generation: self.committed.generation,
            movement_frames: &movement_frames,
            offline_bootstrap: self.bootstrap_for_next_tick(),
        };
        let input_payload = match input.encode_payload() {
            Ok(payload) => payload,
            Err(error) => {
                return Err(WocTickFault {
                    attempted_tick,
                    kind: WocTickFaultKind::EncodeInput(error),
                    rollback_error: None,
                });
            }
        };
        let checkpoint = match self.vm.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick,
                    kind: WocTickFaultKind::Vm(error),
                    rollback_error: None,
                }));
            }
        };
        let result = match self.vm.fixed_tick(&input_payload, self.budgets) {
            Ok(result) => result,
            Err(error) => {
                return Err(self.fail_after_checkpoint(
                    attempted_tick,
                    checkpoint,
                    WocTickFaultKind::Vm(error),
                ));
            }
        };
        if let Some((budget, actual, maximum)) = result.usage.exceeded(self.budgets) {
            return Err(self.fail_after_checkpoint(
                attempted_tick,
                checkpoint,
                WocTickFaultKind::Budget {
                    budget,
                    actual,
                    maximum,
                },
            ));
        }
        let output = match WorldSnapshot::decode_payload(&result.output_payload) {
            Ok(output) => output,
            Err(error) => {
                return Err(self.fail_after_checkpoint(
                    attempted_tick,
                    checkpoint,
                    WocTickFaultKind::DecodeOutput(error),
                ));
            }
        };
        if output.tick != attempted_tick {
            return Err(self.fail_after_checkpoint(
                attempted_tick,
                checkpoint,
                WocTickFaultKind::TickMismatch {
                    actual: output.tick,
                    expected: attempted_tick,
                },
            ));
        }
        let state_digest = fnv1a_bytes(&output.state);
        if output.state_digest != state_digest {
            return Err(self.fail_after_checkpoint(
                attempted_tick,
                checkpoint,
                WocTickFaultKind::StateDigestMismatch {
                    actual: output.state_digest,
                    expected: state_digest,
                },
            ));
        }
        let event_digest = event_stream_digest(&output.events);
        if output.event_digest != event_digest {
            return Err(self.fail_after_checkpoint(
                attempted_tick,
                checkpoint,
                WocTickFaultKind::EventDigestMismatch {
                    actual: output.event_digest,
                    expected: event_digest,
                },
            ));
        }
        let presentation_digest = fnv1a_bytes(&result.presentation_payload);

        Ok(TickCandidate {
            snapshot: CommittedSnapshot {
                generation: self.committed.generation,
                tick: output.tick,
                state: output.state,
                state_digest,
                event_digest,
                presentation_digest,
                presentation_payload: result.presentation_payload,
            },
            checkpoint,
        })
    }

    pub fn install_full_snapshot(
        &mut self,
        snapshot: CommittedSnapshot,
    ) -> Result<(), WocTickFault> {
        let expected_state = fnv1a_bytes(&snapshot.state);
        if snapshot.state_digest != expected_state {
            return Err(WocTickFault {
                attempted_tick: snapshot.tick,
                kind: WocTickFaultKind::StateDigestMismatch {
                    actual: snapshot.state_digest,
                    expected: expected_state,
                },
                rollback_error: None,
            });
        }
        let expected_presentation = fnv1a_bytes(&snapshot.presentation_payload);
        if snapshot.presentation_digest != expected_presentation {
            return Err(WocTickFault {
                attempted_tick: snapshot.tick,
                kind: WocTickFaultKind::PresentationDigestMismatch {
                    actual: snapshot.presentation_digest,
                    expected: expected_presentation,
                },
                rollback_error: None,
            });
        }
        let checkpoint = match self.vm.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick: snapshot.tick,
                    kind: WocTickFaultKind::Vm(error),
                    rollback_error: None,
                }));
            }
        };
        if let Err(error) = self.vm.install_full_snapshot(&snapshot) {
            return Err(self.fail_after_checkpoint(
                snapshot.tick,
                checkpoint,
                WocTickFaultKind::Vm(error),
            ));
        }
        if snapshot.tick != 0 || !snapshot.state.is_empty() {
            self.offline_bootstrap = None;
        }
        self.committed = snapshot;
        self.status = RuntimeStatus::Running;
        Ok(())
    }

    pub fn committed(&self) -> &CommittedSnapshot {
        &self.committed
    }

    pub fn checkpoint(&mut self) -> Result<WocRuntimeCheckpoint<V::Checkpoint>, VmTickError> {
        Ok(WocRuntimeCheckpoint {
            vm: self.vm.checkpoint()?,
            committed: self.committed.clone(),
            status: self.status.clone(),
            offline_bootstrap: self.offline_bootstrap.clone(),
        })
    }

    /// Restores owned state while retaining any terminal fault raised since capture.
    /// A verified full snapshot is required to resume a failed transaction.
    pub fn rollback_checkpoint(
        &mut self,
        checkpoint: WocRuntimeCheckpoint<V::Checkpoint>,
    ) -> Result<(), VmTickError> {
        if let Err(error) = self.vm.rollback(&checkpoint.vm) {
            match &mut self.status {
                RuntimeStatus::Running => {
                    self.transition_failure(WocTickFault {
                        attempted_tick: self.committed.tick,
                        kind: WocTickFaultKind::Vm(error.clone()),
                        rollback_error: None,
                    });
                }
                RuntimeStatus::Paused(fault)
                | RuntimeStatus::Faulted(fault)
                | RuntimeStatus::Recovering(fault) => {
                    if fault.rollback_error.is_none() {
                        fault.rollback_error = Some(error.clone());
                    }
                }
            }
            return Err(error);
        }
        self.committed = checkpoint.committed;
        if self.status == RuntimeStatus::Running {
            self.status = checkpoint.status;
        }
        self.offline_bootstrap = checkpoint.offline_bootstrap;
        Ok(())
    }

    pub fn status(&self) -> &RuntimeStatus {
        &self.status
    }

    fn bootstrap_for_next_tick(&self) -> Option<&OfflineSessionBootstrap> {
        (self.committed.tick == 0 && self.committed.state.is_empty())
            .then_some(())
            .and(self.offline_bootstrap.as_ref())
    }

    fn commit_candidate(&mut self, candidate: TickCandidate<V::Checkpoint>) {
        let consumed_bootstrap = self.bootstrap_for_next_tick().is_some();
        self.committed = candidate.snapshot;
        if consumed_bootstrap {
            self.offline_bootstrap = None;
        }
    }

    pub fn vm(&self) -> &V {
        &self.vm
    }

    /// Transfers the VM back to its host for explicit lifecycle teardown.
    pub fn into_vm(self) -> V {
        self.vm
    }

    fn fail_candidate(
        &mut self,
        candidate: TickCandidate<V::Checkpoint>,
        kind: WocTickFaultKind,
    ) -> WocTickFault {
        self.fail_after_checkpoint(candidate.snapshot.tick, candidate.checkpoint, kind)
    }

    fn fail_after_checkpoint(
        &mut self,
        attempted_tick: u64,
        checkpoint: V::Checkpoint,
        kind: WocTickFaultKind,
    ) -> WocTickFault {
        let rollback_error = self.vm.rollback(&checkpoint).err();
        self.transition_failure(WocTickFault {
            attempted_tick,
            kind,
            rollback_error,
        })
    }

    fn transition_failure(&mut self, fault: WocTickFault) -> WocTickFault {
        self.status = match self.role {
            RuntimeRole::Offline => RuntimeStatus::Paused(fault.clone()),
            RuntimeRole::Server => RuntimeStatus::Faulted(fault.clone()),
            RuntimeRole::Client => RuntimeStatus::Recovering(fault.clone()),
        };
        fault
    }
}

impl TickUsage {
    fn exceeded(self, budgets: TickBudgets) -> Option<(BudgetKind, u64, u64)> {
        [
            (
                BudgetKind::Execution,
                self.execution_micros,
                budgets.max_execution_micros,
            ),
            (
                BudgetKind::Memory,
                self.memory_bytes,
                budgets.max_memory_bytes,
            ),
            (
                BudgetKind::HostCalls,
                self.host_calls,
                budgets.max_host_calls,
            ),
            (
                BudgetKind::GarbageCollection,
                self.gc_micros,
                budgets.max_gc_micros,
            ),
        ]
        .into_iter()
        .find(|(_, actual, maximum)| actual > maximum)
    }
}

impl<V: WocReloadableVm> WocTransactionalRuntime<V> {
    /// Replaces the VM at a fixed-tick boundary. The migration result is the
    /// canonical state restored into the replacement and supplied as the
    /// committed base state to the next tick.
    pub fn hot_reload(
        &mut self,
        mut replacement: V,
        migrate: impl FnOnce(&str, &str, &[u8]) -> Result<Vec<u8>, VmTickError>,
    ) -> Result<u64, WocReloadError> {
        if self.status != RuntimeStatus::Running {
            return Err(WocReloadError {
                stage: VmReloadStage::Save,
                source: VmTickError::Limited("session is not running".to_string()),
                rollback_error: None,
                checkpoint_rollback_error: None,
                replacement_cleanup_error: None,
            });
        }
        let next_generation =
            self.committed
                .generation
                .checked_add(1)
                .ok_or_else(|| WocReloadError {
                    stage: VmReloadStage::Generation,
                    source: VmTickError::Limited("generation exhausted".to_string()),
                    rollback_error: None,
                    checkpoint_rollback_error: None,
                    replacement_cleanup_error: None,
                })?;
        let checkpoint = match self.vm.checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(source) => {
                return Err(WocReloadError {
                    stage: VmReloadStage::Save,
                    source,
                    rollback_error: None,
                    checkpoint_rollback_error: None,
                    replacement_cleanup_error: None,
                });
            }
        };
        let old_schema = match self.vm.state_schema() {
            Ok(schema) => schema,
            Err(source) => {
                let mut error = WocReloadError {
                    stage: VmReloadStage::Save,
                    source,
                    rollback_error: None,
                    checkpoint_rollback_error: None,
                    replacement_cleanup_error: None,
                };
                self.rollback_reload(None, &checkpoint, &mut error);
                return Err(error);
            }
        };
        let saved_state = match self.vm.save_state() {
            Ok(state) => state,
            Err(source) => {
                let mut error = WocReloadError {
                    stage: VmReloadStage::Save,
                    source,
                    rollback_error: None,
                    checkpoint_rollback_error: None,
                    replacement_cleanup_error: None,
                };
                self.rollback_reload(None, &checkpoint, &mut error);
                return Err(error);
            }
        };
        if let Err(source) = self.vm.deactivate() {
            let mut error = WocReloadError {
                stage: VmReloadStage::Deactivate,
                source,
                rollback_error: None,
                checkpoint_rollback_error: None,
                replacement_cleanup_error: None,
            };
            self.rollback_reload(Some(&saved_state), &checkpoint, &mut error);
            return Err(error);
        }
        let new_schema = match replacement.state_schema() {
            Ok(schema) => schema,
            Err(source) => {
                let mut error = WocReloadError {
                    stage: VmReloadStage::Load,
                    source,
                    rollback_error: None,
                    checkpoint_rollback_error: None,
                    replacement_cleanup_error: None,
                };
                self.rollback_reload(Some(&saved_state), &checkpoint, &mut error);
                return Err(error);
            }
        };
        let migrated_state = match migrate(&old_schema, &new_schema, &saved_state) {
            Ok(state) => state,
            Err(source) => {
                let mut error = WocReloadError {
                    stage: VmReloadStage::Migrate,
                    source,
                    rollback_error: None,
                    checkpoint_rollback_error: None,
                    replacement_cleanup_error: None,
                };
                self.rollback_reload(Some(&saved_state), &checkpoint, &mut error);
                return Err(error);
            }
        };
        if let Err(source) = replacement.activate() {
            let replacement_cleanup_error = replacement.deactivate().err();
            let mut error = WocReloadError {
                stage: VmReloadStage::Activate,
                source,
                rollback_error: None,
                checkpoint_rollback_error: None,
                replacement_cleanup_error,
            };
            self.rollback_reload(Some(&saved_state), &checkpoint, &mut error);
            return Err(error);
        }
        if let Err(source) = replacement.restore_state(&migrated_state) {
            let replacement_cleanup_error = replacement.deactivate().err();
            let mut error = WocReloadError {
                stage: VmReloadStage::Restore,
                source,
                rollback_error: None,
                checkpoint_rollback_error: None,
                replacement_cleanup_error,
            };
            self.rollback_reload(Some(&saved_state), &checkpoint, &mut error);
            return Err(error);
        }

        self.vm = replacement;
        self.committed.generation = next_generation;
        self.committed.state_digest = fnv1a_bytes(&migrated_state);
        self.committed.state = migrated_state;
        self.committed.presentation_payload.clear();
        self.committed.presentation_digest = FNV1A_OFFSET;
        Ok(self.committed.generation)
    }

    fn rollback_reload(
        &mut self,
        saved_state: Option<&[u8]>,
        checkpoint: &V::Checkpoint,
        error: &mut WocReloadError,
    ) {
        if let Some(saved_state) = saved_state {
            error.rollback_error = self
                .vm
                .activate()
                .and_then(|()| self.vm.restore_state(saved_state))
                .err();
        }
        error.checkpoint_rollback_error = self.vm.rollback(checkpoint).err();
        if let Some(checkpoint_error) = error.checkpoint_rollback_error.clone() {
            self.transition_failure(WocTickFault {
                attempted_tick: self.committed.tick,
                kind: WocTickFaultKind::Vm(error.source.clone()),
                rollback_error: Some(checkpoint_error),
            });
        } else if let Some(lifecycle_error) = error.rollback_error.clone() {
            self.transition_failure(WocTickFault {
                attempted_tick: self.committed.tick,
                kind: WocTickFaultKind::Vm(error.source.clone()),
                rollback_error: Some(lifecycle_error),
            });
        } else if let Some(cleanup_error) = error.replacement_cleanup_error.clone() {
            self.transition_failure(WocTickFault {
                attempted_tick: self.committed.tick,
                kind: WocTickFaultKind::Vm(error.source.clone()),
                rollback_error: Some(cleanup_error),
            });
        }
    }
}

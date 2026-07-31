use woc_protocol::{
    event_stream_digest, fnv1a_bytes, Command, FixedTickInput, MovementFrame,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmTickError {
    Trap(String),
    Limited(String),
    BudgetExceeded(BudgetKind),
    RejectedCommand { index: usize, reason: String },
    Transport(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmTickResult {
    pub output_payload: Vec<u8>,
    pub presentation_payload: Vec<u8>,
    pub usage: TickUsage,
}

pub trait WocProjectVm {
    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError>;
}

pub trait WocReloadableVm: WocProjectVm {
    fn state_schema(&self) -> Result<String, VmTickError>;
    fn save_state(&mut self) -> Result<Vec<u8>, VmTickError>;
    fn deactivate(&mut self) -> Result<(), VmTickError>;
    fn activate(&mut self) -> Result<(), VmTickError>;
    fn restore_state(&mut self, state: &[u8]) -> Result<(), VmTickError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmReloadStage {
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
    pub attempted_tick: u64,
    pub kind: WocTickFaultKind,
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
        let projection = match decode(&candidate.presentation_payload) {
            Ok(projection) => projection,
            Err(reason) => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick: candidate.tick,
                    kind: WocTickFaultKind::DecodePresentation(reason),
                }));
            }
        };
        self.commit_candidate(candidate);
        Ok((&self.committed, projection))
    }

    fn prepare_tick(
        &mut self,
        commands: Vec<Command>,
        movement_frames: Vec<MovementFrame>,
    ) -> Result<CommittedSnapshot, WocTickFault> {
        let attempted_tick = self.committed.tick.saturating_add(1);
        if self.status != RuntimeStatus::Running {
            return Err(WocTickFault {
                attempted_tick,
                kind: WocTickFaultKind::SessionNotRunning,
            });
        }

        let input = FixedTickInput {
            tick: attempted_tick,
            commands,
            wall_time_forbidden: true,
            committed_state: self.committed.state.clone(),
            committed_state_digest: self.committed.state_digest,
            generation: self.committed.generation,
            movement_frames,
            offline_bootstrap: self.bootstrap_for_next_tick().cloned(),
        };
        let input_payload = match input.encode_payload() {
            Ok(payload) => payload,
            Err(error) => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick,
                    kind: WocTickFaultKind::EncodeInput(error),
                }));
            }
        };
        let result = match self.vm.fixed_tick(&input_payload, self.budgets) {
            Ok(result) => result,
            Err(error) => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick,
                    kind: WocTickFaultKind::Vm(error),
                }));
            }
        };
        if let Some((budget, actual, maximum)) = result.usage.exceeded(self.budgets) {
            return Err(self.transition_failure(WocTickFault {
                attempted_tick,
                kind: WocTickFaultKind::Budget {
                    budget,
                    actual,
                    maximum,
                },
            }));
        }
        let output = match WorldSnapshot::decode_payload(&result.output_payload) {
            Ok(output) => output,
            Err(error) => {
                return Err(self.transition_failure(WocTickFault {
                    attempted_tick,
                    kind: WocTickFaultKind::DecodeOutput(error),
                }));
            }
        };
        if output.tick != attempted_tick {
            return Err(self.transition_failure(WocTickFault {
                attempted_tick,
                kind: WocTickFaultKind::TickMismatch {
                    actual: output.tick,
                    expected: attempted_tick,
                },
            }));
        }
        let state_digest = fnv1a_bytes(&output.state);
        if output.state_digest != state_digest {
            return Err(self.transition_failure(WocTickFault {
                attempted_tick,
                kind: WocTickFaultKind::StateDigestMismatch {
                    actual: output.state_digest,
                    expected: state_digest,
                },
            }));
        }
        let event_digest = event_stream_digest(&output.events);
        if output.event_digest != event_digest {
            return Err(self.transition_failure(WocTickFault {
                attempted_tick,
                kind: WocTickFaultKind::EventDigestMismatch {
                    actual: output.event_digest,
                    expected: event_digest,
                },
            }));
        }
        let presentation_digest = fnv1a_bytes(&result.presentation_payload);

        Ok(CommittedSnapshot {
            generation: self.committed.generation,
            tick: output.tick,
            state: output.state,
            state_digest,
            event_digest,
            presentation_digest,
            presentation_payload: result.presentation_payload,
        })
    }

    pub fn install_full_snapshot(
        &mut self,
        snapshot: CommittedSnapshot,
    ) -> Result<(), WocTickFaultKind> {
        let expected_state = fnv1a_bytes(&snapshot.state);
        if snapshot.state_digest != expected_state {
            return Err(WocTickFaultKind::StateDigestMismatch {
                actual: snapshot.state_digest,
                expected: expected_state,
            });
        }
        let expected_presentation = fnv1a_bytes(&snapshot.presentation_payload);
        if snapshot.presentation_digest != expected_presentation {
            return Err(WocTickFaultKind::PresentationDigestMismatch {
                actual: snapshot.presentation_digest,
                expected: expected_presentation,
            });
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

    pub fn status(&self) -> &RuntimeStatus {
        &self.status
    }

    fn bootstrap_for_next_tick(&self) -> Option<&OfflineSessionBootstrap> {
        (self.committed.tick == 0 && self.committed.state.is_empty())
            .then_some(())
            .and(self.offline_bootstrap.as_ref())
    }

    fn commit_candidate(&mut self, candidate: CommittedSnapshot) {
        let consumed_bootstrap = self.bootstrap_for_next_tick().is_some();
        self.committed = candidate;
        if consumed_bootstrap {
            self.offline_bootstrap = None;
        }
    }

    pub fn vm(&self) -> &V {
        &self.vm
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
                replacement_cleanup_error: None,
            });
        }
        let old_schema = self.vm.state_schema().map_err(|source| WocReloadError {
            stage: VmReloadStage::Save,
            source,
            rollback_error: None,
            replacement_cleanup_error: None,
        })?;
        let saved_state = self.vm.save_state().map_err(|source| WocReloadError {
            stage: VmReloadStage::Save,
            source,
            rollback_error: None,
            replacement_cleanup_error: None,
        })?;
        if let Err(source) = self.vm.deactivate() {
            let mut error = WocReloadError {
                stage: VmReloadStage::Deactivate,
                source,
                rollback_error: None,
                replacement_cleanup_error: None,
            };
            self.rollback_reload(&saved_state, &mut error);
            return Err(error);
        }
        let new_schema = match replacement.state_schema() {
            Ok(schema) => schema,
            Err(source) => {
                let mut error = WocReloadError {
                    stage: VmReloadStage::Load,
                    source,
                    rollback_error: None,
                    replacement_cleanup_error: None,
                };
                self.rollback_reload(&saved_state, &mut error);
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
                    replacement_cleanup_error: None,
                };
                self.rollback_reload(&saved_state, &mut error);
                return Err(error);
            }
        };
        if let Err(source) = replacement.activate() {
            let mut error = WocReloadError {
                stage: VmReloadStage::Activate,
                source,
                rollback_error: None,
                replacement_cleanup_error: None,
            };
            self.rollback_reload(&saved_state, &mut error);
            return Err(error);
        }
        if let Err(source) = replacement.restore_state(&migrated_state) {
            let replacement_cleanup_error = replacement.deactivate().err();
            let mut error = WocReloadError {
                stage: VmReloadStage::Restore,
                source,
                rollback_error: None,
                replacement_cleanup_error,
            };
            self.rollback_reload(&saved_state, &mut error);
            return Err(error);
        }

        self.vm = replacement;
        self.committed.generation = self.committed.generation.saturating_add(1);
        self.committed.presentation_payload.clear();
        self.committed.presentation_digest = FNV1A_OFFSET;
        Ok(self.committed.generation)
    }

    fn rollback_reload(&mut self, saved_state: &[u8], error: &mut WocReloadError) {
        let rollback_error = self
            .vm
            .activate()
            .and_then(|()| self.vm.restore_state(saved_state))
            .err();
        if let Some(rollback_error) = rollback_error {
            error.rollback_error = Some(rollback_error.clone());
            self.transition_failure(WocTickFault {
                attempted_tick: self.committed.tick,
                kind: WocTickFaultKind::Vm(rollback_error),
            });
        }
    }
}

use woc_runtime::{
    ClientPresentationProjection, PresentationSnapshot, RuntimeRole, TickBudgets,
    WocOfflineBootstrapError, WocProjectVm, WocTickFault, WocTransactionalRuntime,
};

use crate::OfflineSessionLaunch;

use super::ClientTickInput;

pub trait ClientAuthority<P> {
    type Error;
    type Checkpoint;

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, Self::Error>;
    fn rollback(&mut self, checkpoint: Self::Checkpoint) -> Result<(), Self::Error>;

    /// An error must mean that no authoritative commit occurred.
    fn fixed_step(
        &mut self,
        input: ClientTickInput<'_>,
        scheduled_at_ns: u64,
    ) -> Result<PresentationSnapshot<P>, Self::Error>;
}

pub struct TransactionalClientAuthority<V> {
    runtime: WocTransactionalRuntime<V>,
}

/// State returned by the real project VM while the native presentation
/// producer is unavailable. Keeping this distinct from the visual projection
/// prevents a smoke runner from fabricating HUD or actor data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientStateProjection {
    pub state: Vec<u8>,
    pub state_digest: u32,
    pub event_digest: u32,
}

/// Runs the same transactional VM and frame-driver path as the visual client,
/// but deliberately exposes state only. Hosts must not treat this as a native
/// window or rendered-frame capability.
pub struct StateOnlyClientAuthority<V> {
    runtime: WocTransactionalRuntime<V>,
}

impl<V: WocProjectVm> TransactionalClientAuthority<V> {
    pub fn new(vm: V, budgets: TickBudgets) -> Self {
        Self {
            runtime: WocTransactionalRuntime::new(RuntimeRole::Client, vm, budgets),
        }
    }

    pub fn runtime(&self) -> &WocTransactionalRuntime<V> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut WocTransactionalRuntime<V> {
        &mut self.runtime
    }

    pub fn into_vm(self) -> V {
        self.runtime.into_vm()
    }

    /// Converts the shell's fresh offline selection into the sole first-tick
    /// constructor understood by the authoritative ZrVM package.
    pub fn prepare_offline_session(
        &mut self,
        launch: &OfflineSessionLaunch,
    ) -> Result<(), WocOfflineBootstrapError> {
        self.runtime.install_offline_bootstrap(launch.bootstrap())
    }
}

impl<V: WocProjectVm> StateOnlyClientAuthority<V> {
    pub fn new(vm: V, budgets: TickBudgets) -> Self {
        Self {
            runtime: WocTransactionalRuntime::new(RuntimeRole::Client, vm, budgets),
        }
    }

    pub fn runtime(&self) -> &WocTransactionalRuntime<V> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut WocTransactionalRuntime<V> {
        &mut self.runtime
    }

    pub fn prepare_offline_session(
        &mut self,
        launch: &OfflineSessionLaunch,
    ) -> Result<(), WocOfflineBootstrapError> {
        self.runtime.install_offline_bootstrap(launch.bootstrap())
    }

    pub fn into_vm(self) -> V {
        self.runtime.into_vm()
    }
}

impl<V> ClientAuthority<ClientPresentationProjection> for TransactionalClientAuthority<V>
where
    V: WocProjectVm,
{
    type Error = WocTickFault;
    type Checkpoint = woc_runtime::WocRuntimeCheckpoint<V::Checkpoint>;

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, Self::Error> {
        self.runtime.checkpoint().map_err(|error| WocTickFault {
            attempted_tick: self.runtime.committed().tick.saturating_add(1),
            kind: woc_runtime::WocTickFaultKind::Vm(error),
            rollback_error: None,
        })
    }

    fn rollback(&mut self, checkpoint: Self::Checkpoint) -> Result<(), Self::Error> {
        self.runtime
            .rollback_checkpoint(checkpoint)
            .map_err(|error| WocTickFault {
                attempted_tick: self.runtime.committed().tick,
                kind: woc_runtime::WocTickFaultKind::Vm(error),
                rollback_error: None,
            })
    }

    fn fixed_step(
        &mut self,
        input: ClientTickInput<'_>,
        scheduled_at_ns: u64,
    ) -> Result<PresentationSnapshot<ClientPresentationProjection>, Self::Error> {
        let (committed, projection) = self.runtime.tick_with_projection_and_movement(
            input.commands().to_vec(),
            vec![input.movement()],
            |bytes| {
                ClientPresentationProjection::decode_json(bytes)
                    .map_err(|error| format!("{error:?}"))
            },
        )?;
        Ok(PresentationSnapshot::new(
            committed.generation,
            committed.tick,
            committed.state_digest,
            committed.event_digest,
            committed.presentation_digest,
            scheduled_at_ns,
            projection,
        ))
    }
}

impl<V> ClientAuthority<ClientStateProjection> for StateOnlyClientAuthority<V>
where
    V: WocProjectVm,
{
    type Error = WocTickFault;
    type Checkpoint = woc_runtime::WocRuntimeCheckpoint<V::Checkpoint>;

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, Self::Error> {
        self.runtime.checkpoint().map_err(|error| WocTickFault {
            attempted_tick: self.runtime.committed().tick.saturating_add(1),
            kind: woc_runtime::WocTickFaultKind::Vm(error),
            rollback_error: None,
        })
    }

    fn rollback(&mut self, checkpoint: Self::Checkpoint) -> Result<(), Self::Error> {
        self.runtime
            .rollback_checkpoint(checkpoint)
            .map_err(|error| WocTickFault {
                attempted_tick: self.runtime.committed().tick,
                kind: woc_runtime::WocTickFaultKind::Vm(error),
                rollback_error: None,
            })
    }

    fn fixed_step(
        &mut self,
        input: ClientTickInput<'_>,
        scheduled_at_ns: u64,
    ) -> Result<PresentationSnapshot<ClientStateProjection>, Self::Error> {
        let committed = self
            .runtime
            .tick_with_movement(input.commands().to_vec(), vec![input.movement()])?;
        let projection = ClientStateProjection {
            state: committed.state.clone(),
            state_digest: committed.state_digest,
            event_digest: committed.event_digest,
        };
        Ok(PresentationSnapshot::new(
            committed.generation,
            committed.tick,
            committed.state_digest,
            committed.event_digest,
            committed.presentation_digest,
            scheduled_at_ns,
            projection,
        ))
    }
}

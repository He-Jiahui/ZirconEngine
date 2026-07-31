use woc_runtime::{
    ClientPresentationProjection, PresentationSnapshot, RuntimeRole, TickBudgets,
    WocOfflineBootstrapError, WocProjectVm, WocTickFault, WocTransactionalRuntime,
};

use crate::OfflineSessionLaunch;

use super::ClientTickInput;

pub trait ClientAuthority<P> {
    type Error;

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

    /// Converts the shell's fresh offline selection into the sole first-tick
    /// constructor understood by the authoritative ZrVM package.
    pub fn prepare_offline_session(
        &mut self,
        launch: &OfflineSessionLaunch,
    ) -> Result<(), WocOfflineBootstrapError> {
        self.runtime.install_offline_bootstrap(launch.bootstrap())
    }
}

impl<V> ClientAuthority<ClientPresentationProjection> for TransactionalClientAuthority<V>
where
    V: WocProjectVm,
{
    type Error = WocTickFault;

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

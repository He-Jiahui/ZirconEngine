use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex};

use zircon_runtime::plugin::PluginModuleId;

#[cfg(test)]
#[path = "execution_gate/canonical_owner_tests.rs"]
mod canonical_owner_tests;

#[derive(Clone, Debug, Default)]
pub(super) struct BehaviorNodeExecutionGate {
    inner: Arc<ExecutionGateInner>,
}

#[derive(Debug, Default)]
struct ExecutionGateInner {
    state: Mutex<ExecutionGateState>,
    idle: Condvar,
}

#[derive(Debug, Default)]
struct ExecutionGateState {
    revoked: HashSet<PluginModuleId>,
    revoking: HashSet<PluginModuleId>,
    in_flight: HashMap<PluginModuleId, usize>,
}

impl BehaviorNodeExecutionGate {
    pub(super) fn acquire(
        &self,
        mut owners: Vec<PluginModuleId>,
    ) -> Option<BehaviorNodeExecutionLease> {
        canonicalize_owners(&mut owners);
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if owners.iter().any(|owner| state.revoked.contains(owner)) {
            return None;
        }
        for owner in &owners {
            *state.in_flight.entry(*owner).or_default() += 1;
        }
        drop(state);
        Some(BehaviorNodeExecutionLease {
            gate: self.clone(),
            owners,
        })
    }

    pub(super) fn acquire_registration(&self, owner: PluginModuleId) -> BehaviorNodeExecutionLease {
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.revoking.contains(&owner) {
            state = self
                .inner
                .idle
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        state.revoked.remove(&owner);
        *state.in_flight.entry(owner).or_default() += 1;
        drop(state);
        BehaviorNodeExecutionLease {
            gate: self.clone(),
            owners: vec![owner],
        }
    }

    pub(super) fn revoke_and_wait(&self, owner: PluginModuleId) -> BehaviorNodeRevocationGuard {
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.revoking.contains(&owner) {
            state = self
                .inner
                .idle
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        state.revoking.insert(owner);
        state.revoked.insert(owner);
        while state.in_flight.get(&owner).copied().unwrap_or_default() != 0 {
            state = self
                .inner
                .idle
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        drop(state);
        BehaviorNodeRevocationGuard {
            gate: self.clone(),
            owner,
        }
    }

    fn release(&self, owners: &[PluginModuleId]) {
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if release_owners(&mut state, owners) {
            self.inner.idle.notify_all();
        }
    }
}

fn release_owners(state: &mut ExecutionGateState, owners: &[PluginModuleId]) -> bool {
    let mut revoking_owner_became_idle = false;
    for owner in owners {
        let became_idle = if let Some(count) = state.in_flight.get_mut(owner) {
            *count -= 1;
            *count == 0
        } else {
            false
        };
        if became_idle {
            state.in_flight.remove(owner);
            revoking_owner_became_idle |= state.revoking.contains(owner);
        }
    }
    revoking_owner_became_idle
}

fn canonicalize_owners(owners: &mut Vec<PluginModuleId>) {
    if owners.len() < 2 {
        return;
    }
    let first = owners[0];
    if owners[1..].iter().all(|owner| *owner == first) {
        owners.truncate(1);
        return;
    }
    if owners.windows(2).all(|pair| pair[0].raw() < pair[1].raw()) {
        return;
    }
    owners.sort_unstable_by_key(|owner| owner.raw());
    owners.dedup();
}

#[derive(Debug)]
pub(super) struct BehaviorNodeExecutionLease {
    gate: BehaviorNodeExecutionGate,
    owners: Vec<PluginModuleId>,
}

impl Drop for BehaviorNodeExecutionLease {
    fn drop(&mut self) {
        self.gate.release(&self.owners);
    }
}

#[derive(Debug)]
pub(super) struct BehaviorNodeRevocationGuard {
    gate: BehaviorNodeExecutionGate,
    owner: PluginModuleId,
}

impl Drop for BehaviorNodeRevocationGuard {
    fn drop(&mut self) {
        let mut state = self
            .gate
            .inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.revoking.remove(&self.owner);
        self.gate.inner.idle.notify_all();
    }
}

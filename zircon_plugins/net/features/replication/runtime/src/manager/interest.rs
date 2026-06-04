use zircon_runtime::core::framework::net::{
    NetSessionId, SyncInterestDescriptor, SyncObjectSnapshot,
};

use super::NetReplicationRuntimeManager;

impl NetReplicationRuntimeManager {
    pub(in crate::manager) fn set_interest_impl(&self, interest: SyncInterestDescriptor) {
        self.state
            .lock()
            .expect("net replication state mutex poisoned")
            .interests
            .insert(interest.session, interest);
    }

    pub(in crate::manager) fn visible_snapshots_impl(
        &self,
        session: NetSessionId,
    ) -> Vec<SyncObjectSnapshot> {
        let state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let interest = state.interests.get(&session);
        state
            .snapshots
            .values()
            .filter(|snapshot| {
                interest.map_or(true, |interest| {
                    interest.allows_group(snapshot.interest_group.as_deref())
                })
            })
            .cloned()
            .collect()
    }
}

impl super::state::NetReplicationRuntimeState {
    pub(in crate::manager) fn allows_interest(
        &self,
        session: NetSessionId,
        snapshot: &SyncObjectSnapshot,
    ) -> bool {
        self.interests.get(&session).map_or(true, |interest| {
            interest.allows_group(snapshot.interest_group.as_deref())
        })
    }
}

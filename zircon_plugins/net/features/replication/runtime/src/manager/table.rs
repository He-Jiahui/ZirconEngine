use zircon_runtime::core::framework::net::{
    SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor, SyncReplicationStrategy,
};

use super::NetReplicationRuntimeManager;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetReplicationTable {
    entries: Vec<NetReplicationTableEntry>,
}

impl NetReplicationTable {
    pub fn new(entries: Vec<NetReplicationTableEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[NetReplicationTableEntry] {
        &self.entries
    }

    pub fn entry_for_component(&self, component_type: &str) -> Option<&NetReplicationTableEntry> {
        self.entries
            .iter()
            .find(|entry| entry.component_type == component_type)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetReplicationTableEntry {
    pub dense_index: usize,
    pub component_type: String,
    pub authority: SyncAuthority,
    pub replication_strategy: SyncReplicationStrategy,
    pub fields: Vec<SyncFieldDescriptor>,
    pub update_hz: u16,
    pub replication_priority: u16,
    pub interest_group: Option<String>,
}

impl NetReplicationTableEntry {
    fn from_descriptor(dense_index: usize, descriptor: SyncComponentDescriptor) -> Self {
        Self {
            dense_index,
            component_type: descriptor.component_type,
            authority: descriptor.authority,
            replication_strategy: descriptor.replication_strategy,
            fields: descriptor.fields,
            update_hz: descriptor.update_hz,
            replication_priority: descriptor.replication_priority,
            interest_group: descriptor.interest_group,
        }
    }
}

impl NetReplicationRuntimeManager {
    pub fn compile_replication_table(&self) -> NetReplicationTable {
        let mut descriptors = self
            .state
            .lock()
            .expect("net replication state mutex poisoned")
            .descriptors
            .values()
            .cloned()
            .collect::<Vec<_>>();
        descriptors.sort_by(|left, right| left.component_type.cmp(&right.component_type));

        let entries = descriptors
            .into_iter()
            .enumerate()
            .map(|(index, descriptor)| NetReplicationTableEntry::from_descriptor(index, descriptor))
            .collect();
        NetReplicationTable::new(entries)
    }
}

use serde::{Deserialize, Serialize};

use super::{NetObjectId, NetSessionId};

pub const SYNC_DEFAULT_COMPONENT_UPDATE_HZ: u16 = 20;
pub const SYNC_DEFAULT_REPLICATION_PRIORITY: u16 = 0;
pub const SYNC_REPLICATION_UNBOUNDED_BUDGET: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncAuthority {
    Server,
    ClientOwned,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncFieldDescriptor {
    pub name: String,
    pub value_type: String,
    pub delta_compressed: bool,
}

impl SyncFieldDescriptor {
    pub fn new(name: impl Into<String>, value_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            delta_compressed: true,
        }
    }

    pub fn delta_compressed(mut self, enabled: bool) -> Self {
        self.delta_compressed = enabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncComponentDescriptor {
    pub component_type: String,
    pub authority: SyncAuthority,
    pub fields: Vec<SyncFieldDescriptor>,
    pub update_hz: u16,
    pub replication_priority: u16,
    pub interest_group: Option<String>,
}

impl SyncComponentDescriptor {
    pub fn new(component_type: impl Into<String>, authority: SyncAuthority) -> Self {
        Self {
            component_type: component_type.into(),
            authority,
            fields: Vec::new(),
            update_hz: SYNC_DEFAULT_COMPONENT_UPDATE_HZ,
            replication_priority: SYNC_DEFAULT_REPLICATION_PRIORITY,
            interest_group: None,
        }
    }

    pub fn with_field(mut self, field: SyncFieldDescriptor) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_update_hz(mut self, update_hz: u16) -> Self {
        self.update_hz = update_hz;
        self
    }

    pub fn with_replication_priority(mut self, priority: u16) -> Self {
        self.replication_priority = priority;
        self
    }

    pub fn with_interest_group(mut self, group: impl Into<String>) -> Self {
        self.interest_group = Some(group.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReplicationBudget {
    /// `SYNC_REPLICATION_UNBOUNDED_BUDGET` keeps the snapshot count unlimited for local tests and tooling.
    pub max_snapshots: usize,
    /// `SYNC_REPLICATION_UNBOUNDED_BUDGET` keeps the byte budget unlimited.
    pub max_bytes: usize,
}

impl SyncReplicationBudget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_snapshots(mut self, max_snapshots: usize) -> Self {
        self.max_snapshots = max_snapshots;
        self
    }

    pub fn without_snapshot_limit(mut self) -> Self {
        self.max_snapshots = SYNC_REPLICATION_UNBOUNDED_BUDGET;
        self
    }

    pub fn with_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    pub fn without_byte_limit(mut self) -> Self {
        self.max_bytes = SYNC_REPLICATION_UNBOUNDED_BUDGET;
        self
    }

    pub fn allows_snapshot_count(self, count: usize) -> bool {
        self.max_snapshots == SYNC_REPLICATION_UNBOUNDED_BUDGET || count < self.max_snapshots
    }

    pub fn allows_byte_count(self, current_bytes: usize, snapshot_bytes: usize) -> bool {
        self.max_bytes == SYNC_REPLICATION_UNBOUNDED_BUDGET
            || current_bytes.saturating_add(snapshot_bytes) <= self.max_bytes
    }
}

impl Default for SyncReplicationBudget {
    fn default() -> Self {
        Self {
            max_snapshots: SYNC_REPLICATION_UNBOUNDED_BUDGET,
            max_bytes: SYNC_REPLICATION_UNBOUNDED_BUDGET,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReplicationScheduleReport {
    pub session: NetSessionId,
    pub tick_time_ms: u64,
    pub budget: SyncReplicationBudget,
    pub sent_snapshots: Vec<SyncObjectSnapshot>,
    pub used_bytes: usize,
    pub skipped_not_due: usize,
    pub skipped_by_interest: usize,
    pub deferred_snapshots: usize,
}

impl SyncReplicationScheduleReport {
    pub fn new(session: NetSessionId, tick_time_ms: u64, budget: SyncReplicationBudget) -> Self {
        Self {
            session,
            tick_time_ms,
            budget,
            sent_snapshots: Vec::new(),
            used_bytes: 0,
            skipped_not_due: 0,
            skipped_by_interest: 0,
            deferred_snapshots: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncFieldValue {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl SyncFieldValue {
    pub fn new(name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            bytes: bytes.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncObjectSnapshot {
    pub object: NetObjectId,
    pub component_type: String,
    pub authority: SyncAuthority,
    pub interest_group: Option<String>,
    pub fields: Vec<SyncFieldValue>,
}

impl SyncObjectSnapshot {
    pub fn new(
        object: NetObjectId,
        descriptor: &SyncComponentDescriptor,
        fields: impl IntoIterator<Item = SyncFieldValue>,
    ) -> Self {
        Self {
            object,
            component_type: descriptor.component_type.clone(),
            authority: descriptor.authority,
            interest_group: descriptor.interest_group.clone(),
            fields: fields.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncDelta {
    pub object: NetObjectId,
    pub component_type: String,
    pub sequence: u64,
    pub changed_fields: Vec<SyncFieldValue>,
}

impl SyncDelta {
    pub fn new(
        object: NetObjectId,
        component_type: impl Into<String>,
        sequence: u64,
        changed_fields: impl IntoIterator<Item = SyncFieldValue>,
    ) -> Self {
        Self {
            object,
            component_type: component_type.into(),
            sequence,
            changed_fields: changed_fields.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncInterestDescriptor {
    pub session: NetSessionId,
    pub groups: Vec<String>,
}

impl SyncInterestDescriptor {
    pub fn new(session: NetSessionId) -> Self {
        Self {
            session,
            groups: Vec::new(),
        }
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.groups.push(group.into());
        self
    }

    pub fn allows_group(&self, group: Option<&str>) -> bool {
        match group {
            Some(group) => self.groups.iter().any(|allowed| allowed == group),
            None => true,
        }
    }
}

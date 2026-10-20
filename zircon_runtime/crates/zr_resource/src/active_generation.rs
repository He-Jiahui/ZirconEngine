//! Immutable active-resident metadata. These views never own payloads or residency leases.
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::{ResourceId, ResourceLocator, ResourceRecord};

const SHARDS: usize = 64;

/// Process-local identity of one installed payload, including remove/recreate ABA.
#[derive(Clone, Debug)]
pub struct ResourceActiveVersionKey {
    id: ResourceId,
    identity: Arc<()>,
}

impl ResourceActiveVersionKey {
    pub(crate) fn new(id: ResourceId) -> Self {
        Self {
            id,
            identity: Arc::new(()),
        }
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }
}

impl PartialEq for ResourceActiveVersionKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.identity, &other.identity)
    }
}
impl Eq for ResourceActiveVersionKey {}
impl Hash for ResourceActiveVersionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        state.write_usize(Arc::as_ptr(&self.identity) as usize);
    }
}

#[derive(Clone, Debug)]
pub struct ResourceActiveRow {
    pub record: Arc<ResourceRecord>,
    pub key: ResourceActiveVersionKey,
}

type Rows = HashMap<ResourceId, Arc<ResourceActiveRow>>;
type Locators = HashMap<ResourceLocator, ResourceId>;
type Sources = HashMap<ResourceId, Arc<[ResourceId]>>;
type Owners = HashMap<ResourceId, ResourceId>;
type GenerationChanges = std::collections::VecDeque<(Arc<()>, Arc<[ResourceId]>)>;

#[derive(Clone, Debug)]
pub struct ResourceActiveGeneration {
    rows: Arc<[Arc<Rows>]>,
    locators: Arc<[Arc<Locators>]>,
    sources: Arc<[Arc<Sources>]>,
    owners: Arc<[Arc<Owners>]>,
    identity: Arc<()>,
    changes: Arc<GenerationChanges>,
}

fn empty_shards<K, V>() -> Arc<[Arc<HashMap<K, V>>]> {
    (0..SHARDS)
        .map(|_| Arc::new(HashMap::new()))
        .collect::<Vec<_>>()
        .into()
}

fn shard(value: &impl Hash) -> usize {
    let mut hasher = std::hash::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as usize % SHARDS
}

impl Default for ResourceActiveGeneration {
    fn default() -> Self {
        Self {
            rows: empty_shards(),
            locators: empty_shards(),
            sources: empty_shards(),
            owners: empty_shards(),
            identity: Arc::new(()),
            changes: Arc::new(std::collections::VecDeque::new()),
        }
    }
}

impl ResourceActiveGeneration {
    pub fn row(&self, id: ResourceId) -> Option<&Arc<ResourceActiveRow>> {
        self.rows[shard(&id)].get(&id)
    }

    pub fn key(&self, id: ResourceId) -> Option<ResourceActiveVersionKey> {
        self.row(id).map(|row| row.key.clone())
    }

    pub fn id_for_locator(&self, locator: &ResourceLocator) -> Option<ResourceId> {
        self.locators[shard(locator)].get(locator).copied()
    }

    pub fn source_members(&self, source: ResourceId) -> &[ResourceId] {
        self.sources[shard(&source)]
            .get(&source)
            .map(AsRef::as_ref)
            .unwrap_or(&[])
    }

    pub fn sources(&self) -> impl Iterator<Item = (ResourceId, &[ResourceId])> {
        self.sources
            .iter()
            .flat_map(|shard| shard.iter().map(|(id, members)| (*id, members.as_ref())))
    }

    pub fn source_owner(&self, id: ResourceId) -> Option<ResourceId> {
        self.owners[shard(&id)].get(&id).copied()
    }

    /// A bounded metadata journal; callers fall back to their bounded resync on a gap.
    pub fn changed_since(&self, previous: &Self) -> Option<Vec<ResourceId>> {
        if Arc::ptr_eq(&self.identity, &previous.identity) {
            return Some(Vec::new());
        }
        let mut ids = Vec::new();
        for (before, changed) in self.changes.iter().rev() {
            ids.extend(changed.iter().copied());
            if Arc::ptr_eq(before, &previous.identity) {
                ids.sort_unstable();
                ids.dedup();
                return Some(ids);
            }
        }
        None
    }

    /// Only the Resource authority may publish a new metadata generation.
    /// Clone the fixed shard roots and only copy maps touched by this change.
    pub(crate) fn updated(
        current: &Arc<Self>,
        updates: Vec<(ResourceId, Option<Arc<ResourceActiveRow>>)>,
        sources: &[(ResourceId, Vec<ResourceId>)],
    ) -> Arc<Self> {
        if updates.is_empty()
            && sources
                .iter()
                .all(|(owner, members)| current.source_members(*owner) == members.as_slice())
        {
            return Arc::clone(current);
        }
        let changed_ids: Arc<[ResourceId]> =
            updates.iter().map(|(id, _)| *id).collect::<Vec<_>>().into();
        let mut changes = current.changes.as_ref().clone();
        if changes.len() == SHARDS {
            changes.pop_front();
        }
        changes.push_back((Arc::clone(&current.identity), changed_ids));
        let mut rows = current.rows.to_vec();
        let mut locators = current.locators.to_vec();
        let mut owners = current.owners.to_vec();
        let mut source_shards = current.sources.to_vec();
        // Remove old locators before adding replacements, including a locator swap.
        for (id, _) in &updates {
            if let Some(previous) = current.row(*id) {
                Arc::make_mut(&mut locators[shard(&previous.record.primary_locator)])
                    .remove(&previous.record.primary_locator);
            }
        }
        for (id, row) in updates {
            let map = Arc::make_mut(&mut rows[shard(&id)]);
            if let Some(row) = row {
                Arc::make_mut(&mut locators[shard(&row.record.primary_locator)])
                    .insert(row.record.primary_locator.clone(), id);
                map.insert(id, row);
            } else {
                map.remove(&id);
            }
        }
        for (source, members) in sources {
            for id in current.source_members(*source) {
                if owners[shard(id)].get(id) == Some(source) {
                    Arc::make_mut(&mut owners[shard(id)]).remove(id);
                }
            }
            let map = Arc::make_mut(&mut source_shards[shard(source)]);
            if members.is_empty() {
                map.remove(source);
            } else {
                map.insert(*source, members.clone().into());
                for id in members {
                    Arc::make_mut(&mut owners[shard(id)]).insert(*id, *source);
                }
            }
        }
        Arc::new(Self {
            rows: rows.into(),
            locators: locators.into(),
            sources: source_shards.into(),
            owners: owners.into(),
            identity: Arc::new(()),
            changes: Arc::new(changes),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceActiveSnapshotError {
    Stale,
    KindConflict,
}

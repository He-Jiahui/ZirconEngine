use std::collections::HashMap;
use std::hash::Hash;

use super::owner::PluginModuleId;

#[cfg(test)]
#[path = "typed_extension_point/tests.rs"]
mod tests;

pub trait ExtensionKey: Clone + Eq + Hash + Ord {}

impl<T> ExtensionKey for T where T: Clone + Eq + Hash + Ord {}

/// Stable logical id for one registry contribution.
///
/// Slots are never reassigned while a registry lives. Owner revocation retires
/// its slots while surviving rows keep their ids even when dense storage moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExtensionSlot(u32);

impl ExtensionSlot {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u32 {
        self.0
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug)]
pub struct TypedExtensionPoint<K, V>
where
    K: ExtensionKey,
{
    state: TypedExtensionState<K, V>,
}

#[derive(Clone, Debug)]
enum TypedExtensionState<K, V>
where
    K: ExtensionKey,
{
    Staging(StagingExtensionTable<K, V>),
    Frozen(FrozenExtensionTable<K, V>),
}

#[derive(Clone, Debug)]
struct StagingExtensionTable<K, V>
where
    K: ExtensionKey,
{
    keys: Vec<K>,
    owners: Vec<PluginModuleId>,
    values: Vec<V>,
    slots: Vec<ExtensionSlot>,
    index: HashMap<K, u32>,
    slot_indices: Vec<Option<u32>>,
}

impl<K, V> Default for StagingExtensionTable<K, V>
where
    K: ExtensionKey,
{
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            owners: Vec::new(),
            values: Vec::new(),
            slots: Vec::new(),
            index: HashMap::new(),
            slot_indices: Vec::new(),
        }
    }
}

impl<K, V> Default for TypedExtensionPoint<K, V>
where
    K: ExtensionKey,
{
    fn default() -> Self {
        Self {
            state: TypedExtensionState::Staging(StagingExtensionTable::default()),
        }
    }
}

impl<K, V> TypedExtensionPoint<K, V>
where
    K: ExtensionKey,
{
    pub fn register(
        &mut self,
        owner: PluginModuleId,
        key: K,
        value: V,
    ) -> Result<ExtensionSlot, ExtensionSlot> {
        if let Some(slot) = self.resolve(&key) {
            return Err(slot);
        }

        let table = self.staging_mut();
        let slot = ExtensionSlot(table.slot_indices.len() as u32);
        let dense_index = table.values.len() as u32;
        table.index.insert(key.clone(), slot.raw());
        table.keys.push(key);
        table.owners.push(owner);
        table.values.push(value);
        table.slots.push(slot);
        table.slot_indices.push(Some(dense_index));
        Ok(slot)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.resolve(key).is_some()
    }

    pub fn resolve(&self, key: &K) -> Option<ExtensionSlot> {
        match &self.state {
            TypedExtensionState::Staging(table) => table.index.get(key).copied().map(ExtensionSlot),
            TypedExtensionState::Frozen(table) => table.resolve(key),
        }
    }

    pub fn values(&self) -> &[V] {
        match &self.state {
            TypedExtensionState::Staging(table) => &table.values,
            TypedExtensionState::Frozen(table) => table.values(),
        }
    }

    pub fn values_mut(&mut self) -> &mut [V] {
        &mut self.staging_mut().values
    }

    pub fn iter(&self) -> impl Iterator<Item = (PluginModuleId, &K, &V)> {
        self.owners_slice()
            .iter()
            .copied()
            .zip(self.keys_slice().iter())
            .zip(self.values().iter())
            .map(|((owner, key), value)| (owner, key, value))
    }

    pub fn get(&self, slot: ExtensionSlot) -> Option<&V> {
        match &self.state {
            TypedExtensionState::Staging(table) => table
                .dense_index_for_slot(slot)
                .and_then(|index| table.values.get(index)),
            TypedExtensionState::Frozen(table) => table.get(slot),
        }
    }

    pub fn key_for_slot(&self, slot: ExtensionSlot) -> Option<&K> {
        match &self.state {
            TypedExtensionState::Staging(table) => table
                .dense_index_for_slot(slot)
                .and_then(|index| table.keys.get(index)),
            TypedExtensionState::Frozen(table) => table.key_for_slot(slot),
        }
    }

    pub fn owner_for_slot(&self, slot: ExtensionSlot) -> Option<PluginModuleId> {
        match &self.state {
            TypedExtensionState::Staging(table) => table
                .dense_index_for_slot(slot)
                .and_then(|index| table.owners.get(index))
                .copied(),
            TypedExtensionState::Frozen(table) => table.owner_for_slot(slot),
        }
    }

    pub fn entries_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> impl Iterator<Item = ExtensionSlot> + '_ {
        self.owners_slice()
            .iter()
            .enumerate()
            .filter_map(move |(dense_index, candidate)| {
                (*candidate == owner)
                    .then(|| self.slots_slice().get(dense_index).copied())
                    .flatten()
            })
    }

    pub fn remove_owned_by(&mut self, owner: PluginModuleId) -> Vec<ExtensionSlot> {
        let table = self.staging_mut();
        let mut removed = Vec::new();
        let keys = std::mem::take(&mut table.keys);
        let owners = std::mem::take(&mut table.owners);
        let values = std::mem::take(&mut table.values);
        let slots = std::mem::take(&mut table.slots);
        table.index.clear();

        for (((key, candidate), value), slot) in keys.into_iter().zip(owners).zip(values).zip(slots)
        {
            if candidate == owner {
                if let Some(dense_index) = table.slot_indices.get_mut(slot.index()) {
                    *dense_index = None;
                }
                removed.push(slot);
                continue;
            }

            let dense_index = table.values.len() as u32;
            table.index.insert(key.clone(), slot.raw());
            if let Some(slot_index) = table.slot_indices.get_mut(slot.index()) {
                *slot_index = Some(dense_index);
            }
            table.keys.push(key);
            table.owners.push(candidate);
            table.values.push(value);
            table.slots.push(slot);
        }

        removed
    }

    /// Rebinds contributions to a new lifecycle owner without changing their
    /// stable slots. This is used when bootstrap-owned declarations become
    /// attached to an interned runtime plugin owner during registration.
    pub fn reassign_owned_by(
        &mut self,
        current_owner: PluginModuleId,
        new_owner: PluginModuleId,
    ) -> Vec<ExtensionSlot> {
        let table = self.staging_mut();
        let mut reassigned = Vec::new();
        for (owner, slot) in table.owners.iter_mut().zip(table.slots.iter().copied()) {
            if *owner == current_owner {
                *owner = new_owner;
                reassigned.push(slot);
            }
        }
        reassigned
    }

    pub fn sort_by_values<F>(&mut self, mut compare: F)
    where
        F: FnMut(&V, &V) -> std::cmp::Ordering,
    {
        let table = self.staging_mut();
        let keys = std::mem::take(&mut table.keys);
        let owners = std::mem::take(&mut table.owners);
        let values = std::mem::take(&mut table.values);
        let slots = std::mem::take(&mut table.slots);
        let mut entries = keys
            .into_iter()
            .zip(owners)
            .zip(values)
            .zip(slots)
            .map(|(((key, owner), value), slot)| (key, owner, value, slot))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| compare(&left.2, &right.2));

        table.index.clear();
        for (key, owner, value, slot) in entries {
            let dense_index = table.values.len() as u32;
            table.index.insert(key.clone(), slot.raw());
            if let Some(slot_index) = table.slot_indices.get_mut(slot.index()) {
                *slot_index = Some(dense_index);
            }
            table.keys.push(key);
            table.owners.push(owner);
            table.values.push(value);
            table.slots.push(slot);
        }
    }

    /// Moves registration storage into the hash-free runtime representation.
    pub fn freeze(&mut self) {
        if self.is_frozen() {
            return;
        }
        let state = std::mem::replace(
            &mut self.state,
            TypedExtensionState::Staging(StagingExtensionTable::default()),
        );
        let TypedExtensionState::Staging(table) = state else {
            unreachable!("frozen state was checked above");
        };
        self.state = TypedExtensionState::Frozen(FrozenExtensionTable::from_staging(table));
    }

    pub fn is_frozen(&self) -> bool {
        matches!(&self.state, TypedExtensionState::Frozen(_))
    }

    pub fn finalize(mut self) -> FrozenExtensionTable<K, V> {
        self.freeze();
        let TypedExtensionState::Frozen(table) = self.state else {
            unreachable!("freeze always produces a frozen table");
        };
        table
    }

    fn staging_mut(&mut self) -> &mut StagingExtensionTable<K, V> {
        if self.is_frozen() {
            let state = std::mem::replace(
                &mut self.state,
                TypedExtensionState::Staging(StagingExtensionTable::default()),
            );
            let TypedExtensionState::Frozen(table) = state else {
                unreachable!("frozen state was checked above");
            };
            self.state = TypedExtensionState::Staging(table.into_staging());
        }
        let TypedExtensionState::Staging(table) = &mut self.state else {
            unreachable!("frozen state was thawed above");
        };
        table
    }

    fn keys_slice(&self) -> &[K] {
        match &self.state {
            TypedExtensionState::Staging(table) => &table.keys,
            TypedExtensionState::Frozen(table) => &table.keys,
        }
    }

    fn owners_slice(&self) -> &[PluginModuleId] {
        match &self.state {
            TypedExtensionState::Staging(table) => &table.owners,
            TypedExtensionState::Frozen(table) => &table.owners,
        }
    }

    fn slots_slice(&self) -> &[ExtensionSlot] {
        match &self.state {
            TypedExtensionState::Staging(table) => &table.slots,
            TypedExtensionState::Frozen(table) => &table.slots,
        }
    }
}

impl<K, V> StagingExtensionTable<K, V>
where
    K: ExtensionKey,
{
    fn dense_index_for_slot(&self, slot: ExtensionSlot) -> Option<usize> {
        self.slot_indices
            .get(slot.index())
            .copied()
            .flatten()
            .map(|index| index as usize)
    }
}

#[derive(Clone, Debug)]
pub struct FrozenExtensionTable<K, V>
where
    K: ExtensionKey,
{
    keys: Box<[K]>,
    owners: Box<[PluginModuleId]>,
    values: Box<[V]>,
    slots: Box<[ExtensionSlot]>,
    slot_indices: Box<[Option<u32>]>,
    sorted_key_indices: Box<[u32]>,
}

impl<K, V> FrozenExtensionTable<K, V>
where
    K: ExtensionKey,
{
    fn from_staging(table: StagingExtensionTable<K, V>) -> Self {
        let mut sorted_key_indices = (0..table.keys.len() as u32).collect::<Vec<_>>();
        sorted_key_indices
            .sort_by(|left, right| table.keys[*left as usize].cmp(&table.keys[*right as usize]));
        Self {
            keys: table.keys.into_boxed_slice(),
            owners: table.owners.into_boxed_slice(),
            values: table.values.into_boxed_slice(),
            slots: table.slots.into_boxed_slice(),
            slot_indices: table.slot_indices.into_boxed_slice(),
            sorted_key_indices: sorted_key_indices.into_boxed_slice(),
        }
    }

    fn into_staging(self) -> StagingExtensionTable<K, V> {
        let keys = self.keys.into_vec();
        let owners = self.owners.into_vec();
        let values = self.values.into_vec();
        let slots = self.slots.into_vec();
        let slot_indices = self.slot_indices.into_vec();
        let index = keys
            .iter()
            .cloned()
            .zip(slots.iter().copied())
            .map(|(key, slot)| (key, slot.raw()))
            .collect();
        StagingExtensionTable {
            keys,
            owners,
            values,
            slots,
            index,
            slot_indices,
        }
    }

    pub fn get(&self, slot: ExtensionSlot) -> Option<&V> {
        self.dense_index_for_slot(slot)
            .and_then(|index| self.values.get(index))
    }

    pub fn resolve(&self, key: &K) -> Option<ExtensionSlot> {
        self.sorted_key_indices
            .binary_search_by(|dense_index| self.keys[*dense_index as usize].cmp(key))
            .ok()
            .and_then(|sorted_index| self.sorted_key_indices.get(sorted_index))
            .and_then(|dense_index| self.slots.get(*dense_index as usize))
            .copied()
    }

    pub fn values(&self) -> &[V] {
        &self.values
    }

    pub fn key_for_slot(&self, slot: ExtensionSlot) -> Option<&K> {
        self.dense_index_for_slot(slot)
            .and_then(|index| self.keys.get(index))
    }

    pub fn owner_for_slot(&self, slot: ExtensionSlot) -> Option<PluginModuleId> {
        self.dense_index_for_slot(slot)
            .and_then(|index| self.owners.get(index))
            .copied()
    }

    pub fn entries_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> impl Iterator<Item = ExtensionSlot> + '_ {
        self.owners
            .iter()
            .enumerate()
            .filter_map(move |(dense_index, candidate)| {
                (*candidate == owner)
                    .then(|| self.slots.get(dense_index).copied())
                    .flatten()
            })
    }

    fn dense_index_for_slot(&self, slot: ExtensionSlot) -> Option<usize> {
        self.slot_indices
            .get(slot.index())
            .copied()
            .flatten()
            .map(|index| index as usize)
    }
}

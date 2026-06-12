use std::collections::HashMap;
use std::hash::Hash;

use super::owner::PluginModuleId;

pub trait ExtensionKey: Clone + Eq + Hash {}

impl<T> ExtensionKey for T where T: Clone + Eq + Hash {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExtensionSlot(u32);

impl ExtensionSlot {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u32 {
        self.0
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug)]
pub struct TypedExtensionPoint<K, V>
where
    K: ExtensionKey,
{
    keys: Vec<K>,
    owners: Vec<PluginModuleId>,
    values: Vec<V>,
    index: HashMap<K, u32>,
}

impl<K, V> Default for TypedExtensionPoint<K, V>
where
    K: ExtensionKey,
{
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            owners: Vec::new(),
            values: Vec::new(),
            index: HashMap::new(),
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

        let slot = ExtensionSlot(self.values.len() as u32);
        self.index.insert(key.clone(), slot.raw());
        self.keys.push(key);
        self.owners.push(owner);
        self.values.push(value);
        Ok(slot)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    pub fn resolve(&self, key: &K) -> Option<ExtensionSlot> {
        self.index.get(key).copied().map(ExtensionSlot)
    }

    pub fn values(&self) -> &[V] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut [V] {
        &mut self.values
    }

    pub fn iter(&self) -> impl Iterator<Item = (PluginModuleId, &K, &V)> {
        self.owners
            .iter()
            .copied()
            .zip(self.keys.iter())
            .zip(self.values.iter())
            .map(|((owner, key), value)| (owner, key, value))
    }

    pub fn get(&self, slot: ExtensionSlot) -> Option<&V> {
        self.values.get(slot.index())
    }

    pub fn key_for_slot(&self, slot: ExtensionSlot) -> Option<&K> {
        self.keys.get(slot.index())
    }

    pub fn owner_for_slot(&self, slot: ExtensionSlot) -> Option<PluginModuleId> {
        self.owners.get(slot.index()).copied()
    }

    pub fn entries_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> impl Iterator<Item = ExtensionSlot> + '_ {
        self.owners
            .iter()
            .enumerate()
            .filter_map(move |(index, candidate)| {
                (*candidate == owner).then(|| ExtensionSlot(index as u32))
            })
    }

    pub fn remove_owned_by(&mut self, owner: PluginModuleId) -> Vec<ExtensionSlot> {
        let mut removed = Vec::new();
        let keys = std::mem::take(&mut self.keys);
        let owners = std::mem::take(&mut self.owners);
        let values = std::mem::take(&mut self.values);
        self.index.clear();

        for (index, ((key, candidate), value)) in
            keys.into_iter().zip(owners).zip(values).enumerate()
        {
            if candidate == owner {
                removed.push(ExtensionSlot::from_raw(index as u32));
                continue;
            }

            let slot = self.values.len() as u32;
            self.index.insert(key.clone(), slot);
            self.keys.push(key);
            self.owners.push(candidate);
            self.values.push(value);
        }

        removed
    }

    pub fn sort_by_values<F>(&mut self, mut compare: F)
    where
        F: FnMut(&V, &V) -> std::cmp::Ordering,
    {
        let keys = std::mem::take(&mut self.keys);
        let owners = std::mem::take(&mut self.owners);
        let values = std::mem::take(&mut self.values);
        let mut entries = keys
            .into_iter()
            .zip(owners)
            .zip(values)
            .map(|((key, owner), value)| (key, owner, value))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| compare(&left.2, &right.2));

        self.index.clear();
        for (key, owner, value) in entries {
            let slot = self.values.len() as u32;
            self.index.insert(key.clone(), slot);
            self.keys.push(key);
            self.owners.push(owner);
            self.values.push(value);
        }
    }

    pub fn finalize(self) -> FrozenExtensionTable<K, V> {
        FrozenExtensionTable {
            keys: self.keys.into_boxed_slice(),
            owners: self.owners.into_boxed_slice(),
            values: self.values.into_boxed_slice(),
            index: self.index,
        }
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
    index: HashMap<K, u32>,
}

impl<K, V> FrozenExtensionTable<K, V>
where
    K: ExtensionKey,
{
    pub fn get(&self, slot: ExtensionSlot) -> Option<&V> {
        self.values.get(slot.index())
    }

    pub fn resolve(&self, key: &K) -> Option<ExtensionSlot> {
        self.index.get(key).copied().map(ExtensionSlot)
    }

    pub fn values(&self) -> &[V] {
        &self.values
    }

    pub fn key_for_slot(&self, slot: ExtensionSlot) -> Option<&K> {
        self.keys.get(slot.index())
    }

    pub fn owner_for_slot(&self, slot: ExtensionSlot) -> Option<PluginModuleId> {
        self.owners.get(slot.index()).copied()
    }

    pub fn entries_owned_by(
        &self,
        owner: PluginModuleId,
    ) -> impl Iterator<Item = ExtensionSlot> + '_ {
        self.owners
            .iter()
            .enumerate()
            .filter_map(move |(index, candidate)| {
                (*candidate == owner).then(|| ExtensionSlot(index as u32))
            })
    }
}

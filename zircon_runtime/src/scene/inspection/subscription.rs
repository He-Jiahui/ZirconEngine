use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::resource::ResourceId;
use zircon_runtime_interface::world_sync::{
    AssetReloadFrameApplyReportDto, InvalidationBatch, WatchKey, WatchRegistration, WatchToken,
    WorldFact,
};

use crate::scene::{EntityId, World};

#[cfg(test)]
mod tests;

const DEFAULT_MAX_PENDING_FACTS: usize = 4_096;
const DEFAULT_MAX_PENDING_ESTIMATED_BYTES: usize = 512 * 1_024;
const DEFAULT_MAX_PENDING_AGE_GENERATIONS: u64 = 8;

/// Hard limits for one session's unflushed world facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionTableLimits {
    max_pending_facts: usize,
    max_pending_estimated_bytes: usize,
    max_pending_age_generations: u64,
}

impl SubscriptionTableLimits {
    pub const fn new(
        max_pending_facts: usize,
        max_pending_estimated_bytes: usize,
        max_pending_age_generations: u64,
    ) -> Self {
        Self {
            max_pending_facts,
            max_pending_estimated_bytes,
            max_pending_age_generations,
        }
    }

    pub const fn max_pending_facts(self) -> usize {
        self.max_pending_facts
    }

    pub const fn max_pending_estimated_bytes(self) -> usize {
        self.max_pending_estimated_bytes
    }

    pub const fn max_pending_age_generations(self) -> u64 {
        self.max_pending_age_generations
    }
}

impl Default for SubscriptionTableLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_PENDING_FACTS,
            DEFAULT_MAX_PENDING_ESTIMATED_BYTES,
            DEFAULT_MAX_PENDING_AGE_GENERATIONS,
        )
    }
}

/// Cumulative routing and backpressure evidence for one subscription table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubscriptionTableDiagnostics {
    ancestor_walks: u64,
    ancestor_nodes: u64,
    ancestor_visited_allocations: u64,
    direct_key_probes: u64,
    matched_tokens: u64,
    coalesced_facts: u64,
    overflowed_facts: u64,
    age_budget_exceeded: u64,
    pending_peak_count: usize,
    pending_peak_estimated_bytes: usize,
    oldest_pending_age_generations: u64,
    overflowed: bool,
}

impl SubscriptionTableDiagnostics {
    pub const fn ancestor_walks(self) -> u64 {
        self.ancestor_walks
    }

    pub const fn ancestor_nodes(self) -> u64 {
        self.ancestor_nodes
    }

    pub const fn ancestor_visited_allocations(self) -> u64 {
        self.ancestor_visited_allocations
    }

    pub const fn direct_key_probes(self) -> u64 {
        self.direct_key_probes
    }

    pub const fn matched_tokens(self) -> u64 {
        self.matched_tokens
    }

    pub const fn coalesced_facts(self) -> u64 {
        self.coalesced_facts
    }

    pub const fn overflowed_facts(self) -> u64 {
        self.overflowed_facts
    }

    pub const fn age_budget_exceeded(self) -> u64 {
        self.age_budget_exceeded
    }

    pub const fn pending_peak_count(self) -> usize {
        self.pending_peak_count
    }

    pub const fn pending_peak_estimated_bytes(self) -> usize {
        self.pending_peak_estimated_bytes
    }

    pub const fn oldest_pending_age_generations(self) -> u64 {
        self.oldest_pending_age_generations
    }

    pub const fn overflowed(self) -> bool {
        self.overflowed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PendingFactKey {
    Entity(EntityId),
    Scene(ResourceId),
    AssetReload,
}

impl PendingFactKey {
    fn for_fact(fact: &WorldFact) -> Self {
        match fact {
            WorldFact::Spawned(entity) | WorldFact::Despawned(entity) => Self::Entity(*entity),
            WorldFact::Reparented { entity, .. } => Self::Entity(*entity),
            WorldFact::SceneLoaded { scene } | WorldFact::SceneUnloaded { scene } => {
                Self::Scene(*scene)
            }
            WorldFact::AssetReloadApplied(_) => Self::AssetReload,
        }
    }
}

/// Session-owned watch state for runtime world invalidation.
#[derive(Clone, Debug)]
pub struct SubscriptionTable {
    next_token: u64,
    by_token: BTreeMap<WatchToken, WatchKey>,
    world_tokens: BTreeSet<WatchToken>,
    subtree_tokens: BTreeMap<EntityId, BTreeSet<WatchToken>>,
    component_tokens: BTreeMap<String, BTreeSet<WatchToken>>,
    asset_tokens: BTreeMap<ResourceId, BTreeSet<WatchToken>>,
    pending_facts: Vec<WorldFact>,
    pending_fact_index: BTreeMap<PendingFactKey, usize>,
    pending_estimated_bytes: usize,
    pending_oldest_generation: Option<u64>,
    pending_age_overflowed: bool,
    pending_dirty: BTreeSet<WatchToken>,
    limits: SubscriptionTableLimits,
    diagnostics: SubscriptionTableDiagnostics,
}

impl Default for SubscriptionTable {
    fn default() -> Self {
        Self::with_limits(SubscriptionTableLimits::default())
    }
}

impl SubscriptionTable {
    pub fn with_limits(limits: SubscriptionTableLimits) -> Self {
        Self {
            next_token: 0,
            by_token: BTreeMap::new(),
            world_tokens: BTreeSet::new(),
            subtree_tokens: BTreeMap::new(),
            component_tokens: BTreeMap::new(),
            asset_tokens: BTreeMap::new(),
            pending_facts: Vec::new(),
            pending_fact_index: BTreeMap::new(),
            pending_estimated_bytes: 0,
            pending_oldest_generation: None,
            pending_age_overflowed: false,
            pending_dirty: BTreeSet::new(),
            limits,
            diagnostics: SubscriptionTableDiagnostics::default(),
        }
    }

    pub fn watch(&mut self, registration: WatchRegistration) -> WatchToken {
        let token = self.allocate_token();
        self.insert_typed_index(&registration.key, token);
        self.by_token.insert(token, registration.key);
        token
    }

    pub fn unwatch(&mut self, token: WatchToken) -> bool {
        self.pending_dirty.remove(&token);
        let Some(key) = self.by_token.remove(&token) else {
            return false;
        };
        self.remove_typed_index(&key, token);
        true
    }

    pub fn record_fact(&mut self, world: &World, fact: WorldFact) {
        match &fact {
            WorldFact::Spawned(entity) | WorldFact::Despawned(entity) => {
                self.invalidate_world_structure();
                self.invalidate_subtree(world, *entity);
            }
            WorldFact::Reparented { entity, .. } => {
                self.invalidate_world_structure();
                self.invalidate_subtree(world, *entity);
            }
            WorldFact::SceneLoaded { scene } | WorldFact::SceneUnloaded { scene } => {
                self.invalidate_world_structure();
                self.invalidate_asset(*scene);
            }
            WorldFact::AssetReloadApplied(_) => self.invalidate_all_assets(),
        }
        self.enqueue_fact(world.world_generation(), fact);
    }

    /// Marks subscribed roots in one bounded walk of the entity's current ancestry.
    /// Reparent callers invoke this once before and once after mutation to cover both chains.
    pub fn invalidate_subtree(&mut self, world: &World, entity: EntityId) {
        let ancestor_chain = ancestor_chain(entity, |current| world.parent_of(current));
        self.diagnostics.ancestor_walks = self.diagnostics.ancestor_walks.saturating_add(1);
        self.diagnostics.ancestor_visited_allocations = self
            .diagnostics
            .ancestor_visited_allocations
            .saturating_add(1);
        self.diagnostics.ancestor_nodes = self
            .diagnostics
            .ancestor_nodes
            .saturating_add(ancestor_chain.len() as u64);
        for ancestor in ancestor_chain {
            self.diagnostics.direct_key_probes =
                self.diagnostics.direct_key_probes.saturating_add(1);
            if let Some(tokens) = self.subtree_tokens.get(&ancestor) {
                self.diagnostics.matched_tokens = self
                    .diagnostics
                    .matched_tokens
                    .saturating_add(tokens.len() as u64);
                self.pending_dirty.extend(tokens.iter().copied());
            }
        }
    }

    /// BTreeMap's borrowed lookup avoids allocating a String on the mutation throat.
    pub fn invalidate_component_type(&mut self, type_name: &str) {
        self.diagnostics.direct_key_probes = self.diagnostics.direct_key_probes.saturating_add(1);
        if let Some(tokens) = self.component_tokens.get(type_name) {
            self.diagnostics.matched_tokens = self
                .diagnostics
                .matched_tokens
                .saturating_add(tokens.len() as u64);
            self.pending_dirty.extend(tokens.iter().copied());
        }
    }

    pub fn invalidate_asset(&mut self, resource_id: ResourceId) {
        self.diagnostics.direct_key_probes = self.diagnostics.direct_key_probes.saturating_add(1);
        if let Some(tokens) = self.asset_tokens.get(&resource_id) {
            self.diagnostics.matched_tokens = self
                .diagnostics
                .matched_tokens
                .saturating_add(tokens.len() as u64);
            self.pending_dirty.extend(tokens.iter().copied());
        }
    }

    pub fn flush(&mut self, generation: u64) -> Option<InvalidationBatch> {
        self.update_pending_age(generation);
        if self.pending_dirty.is_empty() && self.pending_facts.is_empty() {
            return None;
        }
        let batch = InvalidationBatch {
            generation,
            dirty: std::mem::take(&mut self.pending_dirty)
                .into_iter()
                .collect(),
            facts: std::mem::take(&mut self.pending_facts),
        };
        self.pending_fact_index.clear();
        self.pending_estimated_bytes = 0;
        self.pending_oldest_generation = None;
        self.pending_age_overflowed = false;
        Some(batch)
    }

    pub fn diagnostics(&self) -> SubscriptionTableDiagnostics {
        self.diagnostics
    }

    pub fn limits(&self) -> SubscriptionTableLimits {
        self.limits
    }

    pub fn pending_fact_count(&self) -> usize {
        self.pending_facts.len()
    }

    pub fn pending_estimated_bytes(&self) -> usize {
        self.pending_estimated_bytes
    }

    pub fn len(&self) -> usize {
        self.by_token.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_token.is_empty()
    }

    fn allocate_token(&mut self) -> WatchToken {
        loop {
            self.next_token = self.next_token.wrapping_add(1);
            if self.next_token == 0 {
                continue;
            }
            let token = WatchToken::new(self.next_token);
            if !self.by_token.contains_key(&token) {
                return token;
            }
        }
    }

    fn insert_typed_index(&mut self, key: &WatchKey, token: WatchToken) {
        match key {
            WatchKey::WorldStructure => {
                self.world_tokens.insert(token);
            }
            WatchKey::Subtree { root } => {
                self.subtree_tokens.entry(*root).or_default().insert(token);
            }
            WatchKey::ComponentType { type_name } => {
                self.component_tokens
                    .entry(type_name.clone())
                    .or_default()
                    .insert(token);
            }
            WatchKey::Asset { resource_id } => {
                self.asset_tokens
                    .entry(*resource_id)
                    .or_default()
                    .insert(token);
            }
        }
    }

    fn remove_typed_index(&mut self, key: &WatchKey, token: WatchToken) {
        match key {
            WatchKey::WorldStructure => {
                self.world_tokens.remove(&token);
            }
            WatchKey::Subtree { root } => {
                remove_indexed_token(&mut self.subtree_tokens, root, token);
            }
            WatchKey::ComponentType { type_name } => {
                remove_indexed_token(&mut self.component_tokens, type_name, token);
            }
            WatchKey::Asset { resource_id } => {
                remove_indexed_token(&mut self.asset_tokens, resource_id, token);
            }
        }
    }

    fn invalidate_world_structure(&mut self) {
        self.diagnostics.direct_key_probes = self.diagnostics.direct_key_probes.saturating_add(1);
        self.diagnostics.matched_tokens = self
            .diagnostics
            .matched_tokens
            .saturating_add(self.world_tokens.len() as u64);
        self.pending_dirty.extend(self.world_tokens.iter().copied());
    }

    fn invalidate_all_assets(&mut self) {
        for tokens in self.asset_tokens.values() {
            self.diagnostics.direct_key_probes =
                self.diagnostics.direct_key_probes.saturating_add(1);
            self.diagnostics.matched_tokens = self
                .diagnostics
                .matched_tokens
                .saturating_add(tokens.len() as u64);
            self.pending_dirty.extend(tokens.iter().copied());
        }
    }

    fn enqueue_fact(&mut self, generation: u64, fact: WorldFact) {
        self.update_pending_age(generation);
        let key = PendingFactKey::for_fact(&fact);
        if let Some(index) = self.pending_fact_index.get(&key).copied() {
            merge_fact(&mut self.pending_facts[index], fact);
            self.diagnostics.coalesced_facts = self.diagnostics.coalesced_facts.saturating_add(1);
            return;
        }

        let fact_bytes = estimated_fact_bytes();
        let next_bytes = self.pending_estimated_bytes.saturating_add(fact_bytes);
        if self.pending_facts.len() >= self.limits.max_pending_facts
            || next_bytes > self.limits.max_pending_estimated_bytes
        {
            self.diagnostics.overflowed_facts = self.diagnostics.overflowed_facts.saturating_add(1);
            self.diagnostics.overflowed = true;
            return;
        }

        let index = self.pending_facts.len();
        self.pending_fact_index.insert(key, index);
        self.pending_facts.push(fact);
        self.pending_estimated_bytes = next_bytes;
        self.pending_oldest_generation.get_or_insert(generation);
        self.diagnostics.pending_peak_count = self
            .diagnostics
            .pending_peak_count
            .max(self.pending_facts.len());
        self.diagnostics.pending_peak_estimated_bytes = self
            .diagnostics
            .pending_peak_estimated_bytes
            .max(self.pending_estimated_bytes);
    }

    fn update_pending_age(&mut self, generation: u64) {
        let Some(oldest) = self.pending_oldest_generation else {
            return;
        };
        let age = generation.saturating_sub(oldest);
        self.diagnostics.oldest_pending_age_generations =
            self.diagnostics.oldest_pending_age_generations.max(age);
        if age > self.limits.max_pending_age_generations && !self.pending_age_overflowed {
            self.pending_age_overflowed = true;
            self.diagnostics.age_budget_exceeded =
                self.diagnostics.age_budget_exceeded.saturating_add(1);
            self.diagnostics.overflowed = true;
            self.invalidate_world_structure();
        }
    }
}

fn remove_indexed_token<K: Ord>(
    index: &mut BTreeMap<K, BTreeSet<WatchToken>>,
    key: &K,
    token: WatchToken,
) {
    let remove_key = index.get_mut(key).is_some_and(|tokens| {
        tokens.remove(&token);
        tokens.is_empty()
    });
    if remove_key {
        index.remove(key);
    }
}

fn ancestor_chain(
    entity: EntityId,
    mut parent_of: impl FnMut(EntityId) -> Option<EntityId>,
) -> Vec<EntityId> {
    let mut chain = Vec::new();
    let mut visited = BTreeSet::new();
    let mut current = Some(entity);
    while let Some(entity) = current {
        if !visited.insert(entity) {
            break;
        }
        chain.push(entity);
        current = parent_of(entity);
    }
    chain
}

fn ancestor_chain_contains(
    entity: EntityId,
    root: EntityId,
    parent_of: impl FnMut(EntityId) -> Option<EntityId>,
) -> bool {
    ancestor_chain(entity, parent_of).contains(&root)
}

fn merge_fact(existing: &mut WorldFact, incoming: WorldFact) {
    match (existing, incoming) {
        (WorldFact::AssetReloadApplied(existing), WorldFact::AssetReloadApplied(incoming)) => {
            merge_asset_reload_report(existing, incoming)
        }
        (slot, incoming) => *slot = incoming,
    }
}

fn merge_asset_reload_report(
    existing: &mut AssetReloadFrameApplyReportDto,
    incoming: AssetReloadFrameApplyReportDto,
) {
    existing.applied = existing.applied.saturating_add(incoming.applied);
    existing.failed = existing.failed.saturating_add(incoming.failed);
    existing.stale = existing.stale.saturating_add(incoming.stale);
    existing.pending_count = incoming.pending_count;
}

const fn estimated_fact_bytes() -> usize {
    std::mem::size_of::<WorldFact>()
        + std::mem::size_of::<PendingFactKey>()
        + std::mem::size_of::<usize>()
}

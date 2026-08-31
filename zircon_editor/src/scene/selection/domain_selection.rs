use indexmap::IndexSet;
use zircon_runtime::core::framework::scene::EntityId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct DomainSelection {
    items: IndexSet<EntityId>,
    primary: Option<EntityId>,
    generation: u64,
}

impl DomainSelection {
    pub(super) fn items(&self) -> &IndexSet<EntityId> {
        &self.items
    }

    pub(super) fn primary(&self) -> Option<EntityId> {
        self.primary
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn replace<I>(&mut self, items: I, primary: Option<EntityId>) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        let items = items.into_iter().collect::<IndexSet<_>>();
        let primary = primary
            .filter(|entity| items.contains(entity))
            .or_else(|| items.last().copied());
        self.apply(items, primary)
    }

    pub(super) fn select_only(&mut self, entity: EntityId) -> bool {
        self.replace([entity], Some(entity))
    }

    pub(super) fn extend<I>(&mut self, items: I) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        let mut items = items.into_iter();
        let (lower_bound, _) = items.size_hint();
        self.items.reserve(lower_bound);
        let mut changed = false;
        for entity in items {
            if self.items.insert(entity) {
                self.primary = Some(entity);
                changed = true;
            }
        }
        if changed {
            self.bump_generation();
        }
        changed
    }

    pub(super) fn toggle(&mut self, entity: EntityId) -> bool {
        if self.items.shift_remove(&entity) {
            if self.primary == Some(entity) {
                self.primary = self.items.last().copied();
            }
        } else {
            self.items.insert(entity);
            self.primary = Some(entity);
        }
        self.bump_generation();
        true
    }

    pub(super) fn clear(&mut self) -> bool {
        if self.items.is_empty() && self.primary.is_none() {
            return false;
        }
        self.items.clear();
        self.primary = None;
        self.bump_generation();
        true
    }

    fn apply(&mut self, items: IndexSet<EntityId>, primary: Option<EntityId>) -> bool {
        if self.items.iter().eq(items.iter()) && self.primary == primary {
            return false;
        }
        self.items = items;
        self.primary = primary;
        self.bump_generation();
        true
    }

    fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
#[path = "domain_selection/optimization_tests.rs"]
mod optimization_tests;

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
        let mut next = self.items.clone();
        let mut primary = self.primary;
        for entity in items {
            if next.insert(entity) {
                primary = Some(entity);
            }
        }
        self.apply(next, primary)
    }

    pub(super) fn toggle(&mut self, entity: EntityId) -> bool {
        let mut next = self.items.clone();
        let primary = if next.shift_remove(&entity) {
            if self.primary == Some(entity) {
                next.last().copied()
            } else {
                self.primary
            }
        } else {
            next.insert(entity);
            Some(entity)
        };
        self.apply(next, primary)
    }

    pub(super) fn clear(&mut self) -> bool {
        self.apply(IndexSet::new(), None)
    }

    fn apply(&mut self, items: IndexSet<EntityId>, primary: Option<EntityId>) -> bool {
        if self.items.iter().eq(items.iter()) && self.primary == primary {
            return false;
        }
        self.items = items;
        self.primary = primary;
        self.generation = self.generation.wrapping_add(1);
        true
    }
}

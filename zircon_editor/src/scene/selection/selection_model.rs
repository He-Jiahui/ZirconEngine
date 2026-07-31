use indexmap::IndexSet;
use zircon_runtime::core::framework::scene::EntityId;

use super::{SelectionMutation, WorldDomain, domain_selection::DomainSelection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionModel {
    edit: DomainSelection,
    play: DomainSelection,
    active_domain: WorldDomain,
    revision: u64,
}

impl SelectionModel {
    pub fn active_domain(&self) -> WorldDomain {
        self.active_domain
    }

    pub fn set_active_domain(&mut self, domain: WorldDomain) -> bool {
        if self.active_domain == domain {
            return false;
        }
        self.active_domain = domain;
        self.bump_revision();
        true
    }

    pub fn items(&self, domain: WorldDomain) -> &IndexSet<EntityId> {
        self.domain(domain).items()
    }

    pub fn active_items(&self) -> &IndexSet<EntityId> {
        self.items(self.active_domain)
    }

    pub fn primary(&self, domain: WorldDomain) -> Option<EntityId> {
        self.domain(domain).primary()
    }

    pub fn active_primary(&self) -> Option<EntityId> {
        self.primary(self.active_domain)
    }

    pub fn generation(&self, domain: WorldDomain) -> u64 {
        self.domain(domain).generation()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn replace<I>(&mut self, domain: WorldDomain, items: I, primary: Option<EntityId>) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.mutate(domain, |selection| selection.replace(items, primary))
    }

    pub fn replace_active<I>(&mut self, items: I, primary: Option<EntityId>) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.replace(self.active_domain, items, primary)
    }

    pub fn select_only(&mut self, domain: WorldDomain, entity: EntityId) -> bool {
        self.mutate(domain, |selection| selection.select_only(entity))
    }

    pub fn select_only_active(&mut self, entity: EntityId) -> bool {
        self.select_only(self.active_domain, entity)
    }

    pub fn extend<I>(&mut self, domain: WorldDomain, items: I) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.mutate(domain, |selection| selection.extend(items))
    }

    pub fn extend_active<I>(&mut self, items: I) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.extend(self.active_domain, items)
    }

    pub fn toggle(&mut self, domain: WorldDomain, entity: EntityId) -> bool {
        self.mutate(domain, |selection| selection.toggle(entity))
    }

    pub fn toggle_active(&mut self, entity: EntityId) -> bool {
        self.toggle(self.active_domain, entity)
    }

    pub fn apply_active<I>(&mut self, items: I, mutation: SelectionMutation) -> bool
    where
        I: IntoIterator<Item = EntityId>,
    {
        let items = items.into_iter().collect::<Vec<_>>();
        match mutation {
            SelectionMutation::Replace => {
                let primary = items.last().copied();
                self.replace_active(items, primary)
            }
            SelectionMutation::Extend => self.extend_active(items),
            SelectionMutation::Toggle => items.into_iter().fold(false, |changed, entity| {
                self.toggle_active(entity) || changed
            }),
        }
    }

    pub fn clear(&mut self, domain: WorldDomain) -> bool {
        self.mutate(domain, DomainSelection::clear)
    }

    pub fn clear_active(&mut self) -> bool {
        self.clear(self.active_domain)
    }

    fn domain(&self, domain: WorldDomain) -> &DomainSelection {
        match domain {
            WorldDomain::Edit => &self.edit,
            WorldDomain::Play => &self.play,
        }
    }

    fn domain_mut(&mut self, domain: WorldDomain) -> &mut DomainSelection {
        match domain {
            WorldDomain::Edit => &mut self.edit,
            WorldDomain::Play => &mut self.play,
        }
    }

    fn mutate(
        &mut self,
        domain: WorldDomain,
        mutation: impl FnOnce(&mut DomainSelection) -> bool,
    ) -> bool {
        let changed = mutation(self.domain_mut(domain));
        if changed {
            self.bump_revision();
        }
        changed
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

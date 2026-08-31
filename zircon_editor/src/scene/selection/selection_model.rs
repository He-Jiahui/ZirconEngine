use std::collections::BTreeMap;

use indexmap::IndexSet;
use zircon_runtime::core::framework::scene::EntityId;

use crate::core::play::{PlayInstanceId, WorldDomain};

use super::{domain_selection::DomainSelection, SelectionMutation};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionModel {
    edit: DomainSelection,
    play: BTreeMap<PlayInstanceId, DomainSelection>,
    empty: DomainSelection,
    active_domain: WorldDomain,
    revision: u64,
}

impl SelectionModel {
    pub fn active_domain(&self) -> WorldDomain {
        self.active_domain
    }

    pub fn set_active_domain(&mut self, domain: WorldDomain) -> bool {
        if self.active_domain == domain || self.domain(domain).is_none() {
            return false;
        }
        self.active_domain = domain;
        self.bump_revision();
        true
    }

    pub fn activate_play_domain(&mut self, instance: PlayInstanceId) -> bool {
        let inserted = if self.play.contains_key(&instance) {
            false
        } else {
            self.play.insert(instance, self.edit.clone());
            true
        };
        let domain = WorldDomain::Play(instance);
        let activated = self.active_domain != domain;
        if activated {
            self.active_domain = domain;
        }
        if inserted || activated {
            self.bump_revision();
        }
        inserted || activated
    }

    pub fn retire_play_domain(&mut self, instance: PlayInstanceId) -> bool {
        let removed = self.play.remove(&instance).is_some();
        let activated_edit = self.active_domain == WorldDomain::Play(instance);
        if activated_edit {
            self.active_domain = WorldDomain::Edit;
        }
        if removed || activated_edit {
            self.bump_revision();
        }
        removed || activated_edit
    }

    pub fn items(&self, domain: WorldDomain) -> &IndexSet<EntityId> {
        self.domain(domain).unwrap_or(&self.empty).items()
    }

    pub fn active_items(&self) -> &IndexSet<EntityId> {
        self.items(self.active_domain)
    }

    pub fn primary(&self, domain: WorldDomain) -> Option<EntityId> {
        self.domain(domain).and_then(DomainSelection::primary)
    }

    pub fn active_primary(&self) -> Option<EntityId> {
        self.primary(self.active_domain)
    }

    pub fn generation(&self, domain: WorldDomain) -> u64 {
        self.domain(domain)
            .map(DomainSelection::generation)
            .unwrap_or(0)
    }

    pub fn total_item_count(&self) -> usize {
        self.play
            .values()
            .fold(self.edit.items().len(), |count, selection| {
                count.saturating_add(selection.items().len())
            })
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
        let items = items.into_iter().collect::<IndexSet<_>>();
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

    fn domain(&self, domain: WorldDomain) -> Option<&DomainSelection> {
        match domain {
            WorldDomain::Edit => Some(&self.edit),
            WorldDomain::Play(instance) => self.play.get(&instance),
        }
    }

    fn domain_mut(&mut self, domain: WorldDomain) -> Option<&mut DomainSelection> {
        match domain {
            WorldDomain::Edit => Some(&mut self.edit),
            WorldDomain::Play(instance) => self.play.get_mut(&instance),
        }
    }

    fn mutate(
        &mut self,
        domain: WorldDomain,
        mutation: impl FnOnce(&mut DomainSelection) -> bool,
    ) -> bool {
        let Some(selection) = self.domain_mut(domain) else {
            return false;
        };
        let changed = mutation(selection);
        if changed {
            self.bump_revision();
        }
        changed
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

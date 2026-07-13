use zircon_runtime::asset::AssetId;
use zircon_runtime::scene::EntityId;

const MAX_MACHINE_INSTANCE_DEPTH: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MachineInstanceKey {
    entity: EntityId,
    lineage: Box<[AssetId]>,
    owners: Box<[String]>,
}

impl MachineInstanceKey {
    pub(super) fn root(entity: EntityId, machine: AssetId) -> Self {
        Self {
            entity,
            lineage: Box::new([machine]),
            owners: Box::new([]),
        }
    }

    pub(super) fn nested(&self, owner_state: &str, machine: AssetId) -> Option<Self> {
        if self.lineage.len() >= MAX_MACHINE_INSTANCE_DEPTH || self.lineage.contains(&machine) {
            return None;
        }
        let mut lineage = Vec::with_capacity(self.lineage.len() + 1);
        lineage.extend_from_slice(&self.lineage);
        lineage.push(machine);
        let mut owners = Vec::with_capacity(self.owners.len() + 1);
        owners.extend_from_slice(&self.owners);
        owners.push(owner_state.to_string());
        Some(Self {
            entity: self.entity,
            lineage: lineage.into_boxed_slice(),
            owners: owners.into_boxed_slice(),
        })
    }

    pub(super) fn entity(&self) -> EntityId {
        self.entity
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::resource::ResourceId;

    use super::*;

    #[test]
    fn machine_instance_key_separates_lineages_and_rejects_cycles() {
        let root_id = ResourceId::new();
        let child_id = ResourceId::new();
        let root = MachineInstanceKey::root(7, root_id);
        let child = root.nested("Locomotion", child_id).unwrap();
        let sibling = root.nested("Combat", child_id).unwrap();

        assert_ne!(root, child);
        assert_ne!(child, sibling);
        assert_eq!(child.entity(), 7);
        assert!(child.nested("Cycle", root_id).is_none());
    }
}

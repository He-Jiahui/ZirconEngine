use crate::scene::ecs::ComponentId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeSignature {
    table_components: Vec<ComponentId>,
    sparse_set_components: Vec<ComponentId>,
}

impl ArchetypeSignature {
    pub fn new(
        table_components: impl Into<Vec<ComponentId>>,
        sparse_set_components: impl Into<Vec<ComponentId>>,
    ) -> Self {
        Self {
            table_components: normalize_components(table_components.into()),
            sparse_set_components: normalize_components(sparse_set_components.into()),
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    pub fn contains(&self, component_id: ComponentId) -> bool {
        self.table_components.binary_search(&component_id).is_ok()
            || self
                .sparse_set_components
                .binary_search(&component_id)
                .is_ok()
    }

    pub fn table_components(&self) -> &[ComponentId] {
        &self.table_components
    }

    pub fn sparse_set_components(&self) -> &[ComponentId] {
        &self.sparse_set_components
    }
}

fn normalize_components(mut components: Vec<ComponentId>) -> Vec<ComponentId> {
    components.sort_unstable();
    components.dedup();
    components
}

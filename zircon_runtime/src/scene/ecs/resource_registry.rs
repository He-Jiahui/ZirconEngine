use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::{Resource, ResourceId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub id: ResourceId,
    pub type_name: String,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ResourceRegistry {
    descriptors: Vec<ResourceDescriptor>,
    ids_by_type: HashMap<TypeId, ResourceId>,
}

impl ResourceRegistry {
    pub fn resource_id<T>(&mut self) -> ResourceId
    where
        T: Resource,
    {
        if let Some(id) = self.ids_by_type.get(&TypeId::of::<T>()).copied() {
            return id;
        }
        let id = ResourceId::new(self.descriptors.len());
        self.descriptors.push(ResourceDescriptor {
            id,
            type_name: type_name::<T>().to_string(),
        });
        self.ids_by_type.insert(TypeId::of::<T>(), id);
        id
    }

    pub fn registered_resource_id<T>(&self) -> Option<ResourceId>
    where
        T: Resource,
    {
        self.ids_by_type.get(&TypeId::of::<T>()).copied()
    }

    pub fn descriptor(&self, id: ResourceId) -> Option<&ResourceDescriptor> {
        self.descriptors.get(id.index())
    }

    pub fn descriptors(&self) -> &[ResourceDescriptor] {
        &self.descriptors
    }
}

impl fmt::Debug for ResourceRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceRegistry")
            .field("descriptors", &self.descriptors)
            .finish()
    }
}

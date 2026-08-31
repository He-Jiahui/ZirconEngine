use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::fmt;

use super::Resource;
use super::id::ResourceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub id: ResourceId,
    pub type_name: String,
    pub source: ResourceDescriptorSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceDescriptorSource {
    RustType { type_id: TypeId },
    ExternalNative { stable_id: String },
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ResourceRegistry {
    descriptors: Vec<ResourceDescriptor>,
    ids_by_type: HashMap<TypeId, ResourceId>,
    external_ids_by_stable_id: HashMap<String, ResourceId>,
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
            source: ResourceDescriptorSource::RustType {
                type_id: TypeId::of::<T>(),
            },
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

    /// Allocates one schedule-conflict identity for host state exposed by a native plugin.
    /// It does not insert a Rust value into the resource store.
    pub fn external_resource_id(&mut self, stable_id: &str) -> ResourceId {
        if let Some(id) = self.external_ids_by_stable_id.get(stable_id).copied() {
            return id;
        }
        let id = ResourceId::new(self.descriptors.len());
        self.descriptors.push(ResourceDescriptor {
            id,
            type_name: stable_id.to_string(),
            source: ResourceDescriptorSource::ExternalNative {
                stable_id: stable_id.to_string(),
            },
        });
        self.external_ids_by_stable_id
            .insert(stable_id.to_string(), id);
        id
    }

    pub fn registered_external_resource_id(&self, stable_id: &str) -> Option<ResourceId> {
        self.external_ids_by_stable_id.get(stable_id).copied()
    }

    pub fn descriptor(&self, id: ResourceId) -> Option<&ResourceDescriptor> {
        self.descriptors.get(id.index())
    }

    pub fn descriptors(&self) -> &[ResourceDescriptor] {
        &self.descriptors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TypedResource;
    impl Resource for TypedResource {}

    #[test]
    fn external_resource_ids_are_stable_and_do_not_alias_rust_resources() {
        let mut registry = ResourceRegistry::default();
        let typed = registry.resource_id::<TypedResource>();
        let external = registry.external_resource_id("physics.solver");

        assert_ne!(typed, external);
        assert_eq!(registry.external_resource_id("physics.solver"), external);
        assert_eq!(
            registry.registered_external_resource_id("physics.solver"),
            Some(external)
        );
        assert!(matches!(
            &registry.descriptor(external).unwrap().source,
            ResourceDescriptorSource::ExternalNative { stable_id }
                if stable_id == "physics.solver"
        ));
    }
}

impl fmt::Debug for ResourceRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceRegistry")
            .field("descriptors", &self.descriptors)
            .finish()
    }
}

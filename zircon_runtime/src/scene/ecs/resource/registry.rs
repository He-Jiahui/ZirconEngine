use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt;

use super::id::ResourceId;
use super::Resource;

/// Rust 资源或外部 native 资源在调度冲突图中的稳定描述。
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
    // Rust 类型和外部稳定 id 共用递增 ResourceId，但分别维护反向索引避免相互别名。
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
#[path = "tests/registry.rs"]
mod tests;

impl fmt::Debug for ResourceRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceRegistry")
            .field("descriptors", &self.descriptors)
            .finish()
    }
}

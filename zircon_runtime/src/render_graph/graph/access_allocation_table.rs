use std::collections::HashMap;

use super::super::access::{RenderGraphResourceAccessId, RenderGraphVersionedAccessKey};
use super::super::types::{RenderGraphResource, RenderGraphResourceLifetime};
use super::{CompiledRenderGraphTransientAllocationPlan, RenderGraphPhysicalAllocationId};

/// One compiler-order access row paired with its proven transient backing.
///
/// `physical_allocation` is absent for external and persistent resources until
/// their typed lease contracts exist. This table is backend-neutral; product
/// execution later pairs each row with a device-local view or buffer slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphAccessAllocationBinding {
    pub key: RenderGraphVersionedAccessKey,
    pub physical_allocation: Option<RenderGraphPhysicalAllocationId>,
}

/// Dense compiler-order access rows plus exact access-ID lookup.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphAccessAllocationTable {
    bindings: Vec<CompiledRenderGraphAccessAllocationBinding>,
    binding_indices: HashMap<RenderGraphResourceAccessId, usize>,
}

impl CompiledRenderGraphAccessAllocationTable {
    pub(super) fn new(
        keys: &[RenderGraphVersionedAccessKey],
        physical_allocations: &HashMap<RenderGraphResource, RenderGraphPhysicalAllocationId>,
    ) -> Self {
        let mut bindings = Vec::with_capacity(keys.len());
        let mut binding_indices = HashMap::with_capacity(keys.len());

        for key in keys {
            let binding_index = bindings.len();
            binding_indices.insert(key.access_id, binding_index);
            bindings.push(CompiledRenderGraphAccessAllocationBinding {
                key: *key,
                physical_allocation: physical_allocations.get(&key.resource).copied(),
            });
        }

        Self {
            bindings,
            binding_indices,
        }
    }

    pub fn bindings(&self) -> &[CompiledRenderGraphAccessAllocationBinding] {
        &self.bindings
    }

    pub fn binding(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<&CompiledRenderGraphAccessAllocationBinding> {
        self.binding_indices
            .get(&access)
            .and_then(|index| self.bindings.get(*index))
    }
}

/// Precomputes each logical resource's compiler-proven transient backing.
///
/// Access binding construction is a steady compiler phase, so it must not
/// scan every resource lifetime for every live access. Alias projection stays
/// here, before the exact access table is built.
pub(super) fn physical_allocation_ids_by_resource(
    resource_lifetimes: &[RenderGraphResourceLifetime],
    transient_allocation_plan: &CompiledRenderGraphTransientAllocationPlan,
) -> HashMap<RenderGraphResource, RenderGraphPhysicalAllocationId> {
    let allocation_ids_by_resource = transient_allocation_plan
        .allocations
        .iter()
        .map(|allocation| {
            (
                allocation.resource,
                RenderGraphPhysicalAllocationId::from_transient_allocation(allocation),
            )
        })
        .collect::<HashMap<_, _>>();

    resource_lifetimes
        .iter()
        .filter_map(|lifetime| {
            let physical_resource = lifetime
                .texture_view_alias
                .map(|alias| RenderGraphResource::TransientTexture(alias.parent))
                .unwrap_or(lifetime.resource);
            allocation_ids_by_resource
                .get(&physical_resource)
                .copied()
                .map(|allocation_id| (lifetime.resource, allocation_id))
        })
        .collect()
}

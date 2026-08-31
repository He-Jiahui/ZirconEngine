use std::collections::HashMap;

mod access_allocation_table;
mod access_index;
mod compute_binding_access_packet;
mod compute_dispatch_access_packet;
mod external_access_packet;
mod transient_allocation;

use super::access::{
    RenderGraphResourceAccessId, RenderGraphResourceAccessMetadata, RenderGraphVersionedAccessKey,
};
use super::error::RenderGraphError;
use super::types::{
    PassFlags, QueueLane, RenderGraphComputePassMetadata, RenderGraphComputeWorkload,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderGraphResourceVersion, RenderPassId,
};
use super::RenderGraphDump;
use access_allocation_table::physical_allocation_ids_by_resource;
pub use access_allocation_table::{
    CompiledRenderGraphAccessAllocationBinding, CompiledRenderGraphAccessAllocationTable,
};
use access_index::CompiledRenderGraphAccessIndex;
use compute_binding_access_packet::build_compute_binding_access_packets;
pub use compute_binding_access_packet::{
    CompiledRenderGraphComputeBindingAccess, CompiledRenderGraphComputeBindingAccessPacket,
};
use compute_dispatch_access_packet::build_compute_dispatch_access_packets;
pub use compute_dispatch_access_packet::{
    CompiledRenderGraphComputeDispatchAccess, CompiledRenderGraphComputeDispatchAccessPacket,
};
use external_access_packet::build_external_access_packet;
pub use external_access_packet::{
    CompiledRenderGraphExternalAccess, CompiledRenderGraphExternalAccessPacket,
};
use transient_allocation::{
    build_transient_allocation_plan, validate_resource_lifetime_storage_sizes,
};
pub use transient_allocation::{
    CompiledRenderGraphTransientAllocation, CompiledRenderGraphTransientAllocationId,
    CompiledRenderGraphTransientAllocationPlan, CompiledRenderGraphTransientSlotReservation,
    RenderGraphPhysicalAllocationId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPass {
    pub id: RenderPassId,
    pub name: String,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub dependencies: Vec<RenderPassId>,
    pub culled: bool,
    pub executor_id: Option<String>,
    pub compute_workload: Option<RenderGraphComputeWorkload>,
    pub compute_pass_metadata: Option<RenderGraphComputePassMetadata>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphStats {
    pub total_pass_count: usize,
    pub executable_pass_count: usize,
    pub culled_pass_count: usize,
    pub graphics_pass_count: usize,
    pub async_compute_pass_count: usize,
    pub async_copy_pass_count: usize,
    pub queue_fallback_pass_count: usize,
    pub resource_lifetime_count: usize,
    pub total_resource_access_count: usize,
    pub read_resource_access_count: usize,
    pub write_resource_access_count: usize,
    pub total_dependency_count: usize,
    pub external_output_count: usize,
    pub sparse_texture_lifetime_count: usize,
    // Compile-miss algorithmic work counters, not CPU or GPU wall-clock timings.
    pub compile_resource_access_visit_count: usize,
    pub compile_execution_dependency_count: usize,
    pub compile_provenance_dependency_count: usize,
    pub compile_cull_root_count: usize,
    pub compile_cull_dependency_visit_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CompiledRenderGraphCompileWork {
    pub resource_access_visit_count: usize,
    pub execution_dependency_count: usize,
    pub provenance_dependency_count: usize,
    pub cull_root_count: usize,
    pub cull_dependency_visit_count: usize,
}

impl CompiledRenderGraphStats {
    pub fn queue_lane_count(&self, queue: QueueLane) -> usize {
        match queue {
            QueueLane::Graphics => self.graphics_pass_count,
            QueueLane::AsyncCompute => self.async_compute_pass_count,
            QueueLane::AsyncCopy => self.async_copy_pass_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraph {
    name: String,
    passes: Vec<CompiledRenderPass>,
    pass_indices: HashMap<RenderPassId, usize>,
    access_index: CompiledRenderGraphAccessIndex,
    compute_binding_access_packets:
        HashMap<RenderPassId, CompiledRenderGraphComputeBindingAccessPacket>,
    compute_dispatch_access_packets:
        HashMap<RenderPassId, CompiledRenderGraphComputeDispatchAccessPacket>,
    external_access_packet: CompiledRenderGraphExternalAccessPacket,
    resource_declarations: Vec<RenderGraphResourceDeclaration>,
    resource_declaration_indices: HashMap<RenderGraphResource, usize>,
    resource_declaration_indices_by_name: HashMap<String, usize>,
    resource_lifetimes: Vec<RenderGraphResourceLifetime>,
    resource_lifetime_indices: HashMap<RenderGraphResource, usize>,
    transient_allocation_plan: CompiledRenderGraphTransientAllocationPlan,
    physical_allocation_ids: HashMap<RenderGraphResource, RenderGraphPhysicalAllocationId>,
    access_allocation_table: CompiledRenderGraphAccessAllocationTable,
    // Frame statistics are compiled with the graph so steady-frame diagnostics
    // do not rescan pass, access, and lifetime metadata.
    stats: CompiledRenderGraphStats,
}

impl CompiledRenderGraph {
    pub(crate) fn new(
        name: String,
        passes: Vec<CompiledRenderPass>,
        resource_declarations: Vec<RenderGraphResourceDeclaration>,
        resource_lifetimes: Vec<RenderGraphResourceLifetime>,
        pass_resource_versions: Vec<Vec<RenderGraphResourceVersion>>,
        pass_resource_input_versions: Vec<Vec<Option<RenderGraphResourceVersion>>>,
        pass_resource_access_metadata: Vec<Vec<RenderGraphResourceAccessMetadata>>,
        compile_work: CompiledRenderGraphCompileWork,
    ) -> Result<Self, RenderGraphError> {
        let pass_indices = passes
            .iter()
            .enumerate()
            .map(|(index, pass)| (pass.id, index))
            .collect();
        let resource_declaration_indices = resource_declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.resource, index))
            .collect();
        let resource_declaration_indices_by_name = resource_declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.name.clone(), index))
            .collect::<HashMap<_, _>>();
        let access_index = CompiledRenderGraphAccessIndex::new(
            &passes,
            &resource_declarations,
            &resource_declaration_indices_by_name,
            &pass_resource_versions,
            &pass_resource_input_versions,
            &pass_resource_access_metadata,
        )?;
        let compute_binding_access_packets =
            build_compute_binding_access_packets(&passes, &access_index, &resource_declarations)?;
        let compute_dispatch_access_packets =
            build_compute_dispatch_access_packets(&passes, &access_index, &resource_declarations)?;
        let external_access_packet =
            build_external_access_packet(&passes, &access_index, &resource_lifetimes)
                .map_err(|message| RenderGraphError::ExternalAccessPacketBuild { message })?;
        let resource_lifetime_indices = resource_lifetimes
            .iter()
            .enumerate()
            .map(|(index, lifetime)| (lifetime.resource, index))
            .collect();
        validate_resource_lifetime_storage_sizes(&resource_lifetimes)?;
        let transient_allocation_plan = build_transient_allocation_plan(&resource_lifetimes)?;
        transient_allocation_plan.validate_transient_allocation_intervals()?;
        let physical_allocation_ids =
            physical_allocation_ids_by_resource(&resource_lifetimes, &transient_allocation_plan);
        let access_allocation_table = CompiledRenderGraphAccessAllocationTable::new(
            access_index.versioned_access_keys(),
            &physical_allocation_ids,
        );
        let stats = CompiledRenderGraphStats::from_compiled_graph(
            &passes,
            &resource_lifetimes,
            compile_work,
        );
        Ok(Self {
            name,
            passes,
            pass_indices,
            access_index,
            compute_binding_access_packets,
            compute_dispatch_access_packets,
            external_access_packet,
            resource_declarations,
            resource_declaration_indices,
            resource_declaration_indices_by_name,
            resource_lifetimes,
            resource_lifetime_indices,
            transient_allocation_plan,
            physical_allocation_ids,
            access_allocation_table,
            stats,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[CompiledRenderPass] {
        &self.passes
    }

    pub(crate) fn pass(&self, pass: RenderPassId) -> Option<&CompiledRenderPass> {
        self.indexed_pass(pass).map(|(_, pass)| pass)
    }

    pub(crate) fn indexed_pass(&self, pass: RenderPassId) -> Option<(usize, &CompiledRenderPass)> {
        let index = *self.pass_indices.get(&pass)?;
        self.passes.get(index).map(|pass| (index, pass))
    }

    pub(crate) fn pass_resource_access(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<&RenderGraphPassResourceAccess> {
        self.access_index
            .pass_resource_access(&self.passes, pass, resource, access)
    }

    /// Returns the stable compiled identity at an authoring access ordinal.
    ///
    /// The pass handle remains stable when compiler dependency inference
    /// topologically reorders the compiled pass list.
    pub fn access_id_at(
        &self,
        pass: RenderPassId,
        access_index: usize,
    ) -> Option<RenderGraphResourceAccessId> {
        self.access_index.access_id_at(pass, access_index)
    }

    /// Returns the stable compiled identity for an unambiguous pass resource access.
    ///
    /// Returns `None` if future range-aware authoring records more than one
    /// matching access. Callers that require an exact binding must use
    /// [`Self::access_id_at`] instead.
    pub fn access_id_for(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceAccessId> {
        self.access_index.access_id_for(pass, resource, access)
    }

    /// Returns the immutable exact-ID packet for one live generic-compute pass.
    ///
    /// External rows may carry a logical access key but cannot resolve a WGPU
    /// binding until the separate typed lease packet exists.
    pub fn compute_binding_access_packet(
        &self,
        pass: RenderPassId,
    ) -> Option<&CompiledRenderGraphComputeBindingAccessPacket> {
        self.compute_binding_access_packets.get(&pass)
    }

    /// Returns the exact compiled resource target for a dynamic compute dispatch.
    pub fn compute_dispatch_access_packet(
        &self,
        pass: RenderPassId,
    ) -> Option<&CompiledRenderGraphComputeDispatchAccessPacket> {
        self.compute_dispatch_access_packets.get(&pass)
    }

    /// Returns the immutable access-ID packet for live imported resources.
    pub fn external_access_packet(&self) -> &CompiledRenderGraphExternalAccessPacket {
        &self.external_access_packet
    }

    /// Returns the range and use intent frozen for one compiled access.
    pub fn access_metadata(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResourceAccessMetadata> {
        self.access_index.metadata(access)
    }

    pub fn access_metadata_for(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceAccessMetadata> {
        self.access_id_for(pass, resource, access)
            .and_then(|access| self.access_metadata(access))
    }

    pub fn resource_version_for_access(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceVersion> {
        self.access_id_for(pass, resource, access)
            .and_then(|access| self.resource_version_for_id(access))
    }

    /// Returns the version produced by one exact compiled access.
    pub fn resource_version_for_id(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResourceVersion> {
        self.access_index.produced_version(access)
    }

    /// Returns the explicit producer value consumed by this access, when authoring
    /// selected one instead of relying on the legacy latest-resource path.
    pub fn input_version_for_access(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceVersion> {
        self.access_id_for(pass, resource, access)
            .and_then(|access| self.input_version_for_id(access))
    }

    /// Returns the explicit producer value selected by one exact access.
    pub fn input_version_for_id(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResourceVersion> {
        self.access_index.input_version(access)
    }

    /// Returns the backend-neutral exact binding key for one compiled access.
    ///
    /// Physical WGPU view and buffer-slice resolution is intentionally owned by
    /// the frame-scoped execution binding table, never by this compiled graph.
    pub fn versioned_access_key(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphVersionedAccessKey> {
        self.access_index.versioned_access_key(access)
    }

    /// Returns the compiler-proven transient backing identity for a logical resource.
    ///
    /// Texture view aliases resolve to their parent transient backing. External
    /// and persistent resources deliberately return `None` until their typed
    /// lease contracts exist; callers must not substitute string names.
    pub fn physical_allocation_id_for_resource(
        &self,
        resource: RenderGraphResource,
    ) -> Option<RenderGraphPhysicalAllocationId> {
        self.physical_allocation_ids.get(&resource).copied()
    }

    /// Returns the transient backing identity selected by one exact compiled access.
    ///
    /// This only exposes a backend-neutral identity. The frame-scoped product
    /// binding table owns WGPU views and buffer slices.
    pub fn physical_allocation_id_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphPhysicalAllocationId> {
        self.access_allocation_table
            .binding(access)
            .and_then(|binding| binding.physical_allocation)
    }

    /// Returns one dense compiler-order row for every live resource access.
    ///
    /// The rows retain logical version/scope facts and may expose only a
    /// compiler-proven transient allocation. Device-local bindings remain in
    /// the product execution layer.
    pub fn access_allocation_bindings(&self) -> &[CompiledRenderGraphAccessAllocationBinding] {
        self.access_allocation_table.bindings()
    }

    pub fn access_allocation_binding(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<&CompiledRenderGraphAccessAllocationBinding> {
        self.access_allocation_table.binding(access)
    }

    pub fn resource_declarations(&self) -> &[RenderGraphResourceDeclaration] {
        &self.resource_declarations
    }

    pub fn resource_declaration(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&RenderGraphResourceDeclaration> {
        self.resource_declaration_indices
            .get(&resource)
            .and_then(|index| self.resource_declarations.get(*index))
    }

    pub fn resource_declaration_by_name(
        &self,
        name: &str,
    ) -> Option<&RenderGraphResourceDeclaration> {
        self.resource_declaration_indices_by_name
            .get(name)
            .and_then(|index| self.resource_declarations.get(*index))
    }

    pub fn resource_lifetimes(&self) -> &[RenderGraphResourceLifetime] {
        &self.resource_lifetimes
    }

    pub fn resource_lifetime(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&RenderGraphResourceLifetime> {
        self.resource_lifetime_indices
            .get(&resource)
            .and_then(|index| self.resource_lifetimes.get(*index))
    }

    pub fn resource_lifetime_by_name(&self, name: &str) -> Option<&RenderGraphResourceLifetime> {
        self.resource_declaration_by_name(name)
            .and_then(|declaration| self.resource_lifetime(declaration.resource))
    }

    /// Resolves a graph-owned texture to the persistent physical owner that
    /// supplies its frame lease.
    ///
    /// Texture-view aliases keep their own logical access identity, while the
    /// parent owns lifetime and storage. Access ranges are already projected
    /// into the parent subresource space by the compiler.
    pub fn persistent_texture_backing_resource(
        &self,
        resource: RenderGraphResource,
    ) -> Option<RenderGraphResource> {
        let declaration = self.resource_declaration(resource)?;
        if declaration.kind != RenderGraphResourceKind::TransientTexture {
            return None;
        }
        let backing_resource = declaration
            .texture_view_alias
            .map(|alias| RenderGraphResource::TransientTexture(alias.parent))
            .unwrap_or(resource);
        self.resource_lifetime(backing_resource)
            .is_some_and(|lifetime| lifetime.usage.persistent)
            .then_some(backing_resource)
    }

    pub fn dump(&self) -> RenderGraphDump {
        RenderGraphDump::from_graph(self)
    }

    pub fn transient_allocation_plan(&self) -> &CompiledRenderGraphTransientAllocationPlan {
        &self.transient_allocation_plan
    }

    pub fn stats(&self) -> CompiledRenderGraphStats {
        self.stats
    }

    pub fn queue_lane_count(&self, queue: QueueLane) -> usize {
        self.stats.queue_lane_count(queue)
    }
}

impl CompiledRenderGraphStats {
    fn from_compiled_graph(
        passes: &[CompiledRenderPass],
        resource_lifetimes: &[RenderGraphResourceLifetime],
        compile_work: CompiledRenderGraphCompileWork,
    ) -> Self {
        let mut stats = Self {
            total_pass_count: passes.len(),
            resource_lifetime_count: resource_lifetimes.len(),
            compile_resource_access_visit_count: compile_work.resource_access_visit_count,
            compile_execution_dependency_count: compile_work.execution_dependency_count,
            compile_provenance_dependency_count: compile_work.provenance_dependency_count,
            compile_cull_root_count: compile_work.cull_root_count,
            compile_cull_dependency_visit_count: compile_work.cull_dependency_visit_count,
            sparse_texture_lifetime_count: resource_lifetimes
                .iter()
                .filter(|lifetime| lifetime.is_sparse_reserved_texture())
                .count(),
            ..Self::default()
        };

        for pass in passes {
            stats.total_dependency_count += pass.dependencies.len();
            for resource in &pass.resources {
                stats.total_resource_access_count += 1;
                match resource.access {
                    RenderGraphResourceAccessKind::Read => stats.read_resource_access_count += 1,
                    RenderGraphResourceAccessKind::Write => {
                        stats.write_resource_access_count += 1;
                        if resource.kind == RenderGraphResourceKind::External {
                            stats.external_output_count += 1;
                        }
                    }
                }
            }

            if pass.culled {
                stats.culled_pass_count += 1;
                continue;
            }

            stats.executable_pass_count += 1;
            if pass.declared_queue != pass.queue {
                stats.queue_fallback_pass_count += 1;
            }
            match pass.queue {
                QueueLane::Graphics => stats.graphics_pass_count += 1,
                QueueLane::AsyncCompute => stats.async_compute_pass_count += 1,
                QueueLane::AsyncCopy => stats.async_copy_pass_count += 1,
            }
        }

        stats
    }
}

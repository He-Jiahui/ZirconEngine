use std::collections::HashMap;
use std::ops::Range;

use crate::graphics::pipeline::RenderGraphExecutionPass;
use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderGraphAccessAllocationBinding, RenderGraphBufferRange,
    RenderGraphResourceAccessId, RenderGraphResourceAccessRange, RenderGraphResourceKind,
    RenderGraphVersionedAccessKey,
};

use super::super::texture_views::texture_range_covers_full_view;
use super::super::RenderGraphExecutionResources;

impl RenderGraphExecutionResources {
    /// Validates one graph pass against this frame's generation-qualified
    /// device resources before a pass executor may record native work.
    pub(in crate::graphics::scene::scene_renderer) fn validate_device_execution_pass(
        &self,
        graph: &CompiledRenderGraph,
        execution_pass: &RenderGraphExecutionPass,
        access_ids: &[RenderGraphResourceAccessId],
    ) -> Result<(), String> {
        if self.device_epoch.is_none() {
            return Err(format!(
                "render graph pass {} has no materialized device epoch",
                execution_pass.graph_pass_index
            ));
        }

        let graph_pass_index = execution_pass.graph_pass_index;
        let pass = graph.passes().get(graph_pass_index).ok_or_else(|| {
            format!("device execution plan references missing graph pass index {graph_pass_index}")
        })?;
        if pass.culled {
            return Err(format!(
                "device execution plan attempted to admit culled graph pass `{}`",
                pass.name
            ));
        }
        if pass.resources.len() != access_ids.len()
            || pass.resources.len() != execution_pass.device_access_bindings.len()
        {
            return Err(format!(
                "device execution plan for pass `{}` has access cardinality drift: graph={}, IDs={}, plan={}",
                pass.name,
                pass.resources.len(),
                access_ids.len(),
                execution_pass.device_access_bindings.len()
            ));
        }

        let mut seen = HashMap::with_capacity(access_ids.len());
        for (ordinal, ((access_id, planned_binding), pass_access)) in access_ids
            .iter()
            .copied()
            .zip(execution_pass.device_access_bindings.iter())
            .zip(pass.resources.iter())
            .enumerate()
        {
            let expected_id = graph.access_id_at(pass.id, ordinal).ok_or_else(|| {
                format!(
                    "device execution plan for pass `{}` is missing compiler access identity at ordinal {ordinal}",
                    pass.name
                )
            })?;
            if access_id != expected_id || seen.insert(access_id, ()).is_some() {
                return Err(format!(
                    "device execution plan for pass `{}` has an invalid or duplicate access identity at ordinal {ordinal}",
                    pass.name
                ));
            }

            let compiled_binding = graph.access_allocation_binding(access_id).ok_or_else(|| {
                format!(
                    "device execution plan for pass `{}` is missing compiled access binding {:?}",
                    pass.name, access_id
                )
            })?;
            let key = graph.versioned_access_key(access_id).ok_or_else(|| {
                format!(
                    "device execution plan for pass `{}` is missing versioned access key {:?}",
                    pass.name, access_id
                )
            })?;
            let metadata = graph.access_metadata(access_id).ok_or_else(|| {
                format!(
                    "device execution plan for pass `{}` is missing access metadata {:?}",
                    pass.name, access_id
                )
            })?;
            let declaration = graph.resource_declaration(key.resource).ok_or_else(|| {
                format!(
                    "device execution plan access {:?} references an undeclared graph resource",
                    access_id
                )
            })?;

            if *planned_binding != *compiled_binding
                || planned_binding.key != key
                || key.access_id != access_id
                || key.access != pass_access.access
                || key.range != metadata.range
                || key.intent != metadata.intent
                || declaration.name != pass_access.name
                || declaration.kind != pass_access.kind
            {
                return Err(format!(
                    "device execution plan access {:?} disagrees with pass `{}` compiler metadata",
                    access_id, pass.name
                ));
            }
            if self.compiled_access_key(access_id) != Some(key) {
                return Err(format!(
                    "device execution access {:?} was not materialized from this compiled graph key",
                    access_id
                ));
            }

            self.validate_materialized_access_lease(graph, *compiled_binding)?;
        }

        // The immutable execution packet partitions and validates the complete
        // state plan once. Stage admission validates only this pass's rows so
        // frame cost stays proportional to local accesses and transitions.
        for transition in execution_pass.device_transitions_before.iter() {
            let source_key = self
                .compiled_access_key(transition.from_access)
                .ok_or_else(|| {
                    format!(
                        "device execution transition source {:?} has no frame graph key",
                        transition.from_access
                    )
                })?;
            let destination_key =
                self.compiled_access_key(transition.to_access)
                    .ok_or_else(|| {
                        format!(
                            "device execution transition destination {:?} has no frame graph key",
                            transition.to_access
                        )
                    })?;
            let source_graph_key = graph
                .versioned_access_key(transition.from_access)
                .ok_or_else(|| {
                    format!(
                        "device execution transition source {:?} is not in this graph",
                        transition.from_access
                    )
                })?;
            let destination_graph_key = graph
                .versioned_access_key(transition.to_access)
                .ok_or_else(|| {
                    format!(
                        "device execution transition destination {:?} is not in this graph",
                        transition.to_access
                    )
                })?;
            let (source_pass_index, source_pass) = graph
                .indexed_pass(transition.from_access.pass())
                .ok_or_else(|| {
                    format!(
                        "device execution transition source {:?} references a missing pass",
                        transition.from_access
                    )
                })?;
            if transition.resource != source_key.resource
                || transition.resource != destination_key.resource
                || source_key != source_graph_key
                || destination_key != destination_graph_key
                || transition.from_state
                    != crate::render_graph::RenderGraphResourceState::from(source_key.intent)
                || transition.to_state
                    != crate::render_graph::RenderGraphResourceState::from(destination_key.intent)
                || source_pass_index >= graph_pass_index
                || transition.from_queue != source_pass.queue
                || transition.to_queue != pass.queue
                || transition.to_access.pass() != pass.id
                || !seen.contains_key(&transition.to_access)
            {
                return Err(format!(
                    "device execution transition {:?}->{:?} disagrees with its frame access keys",
                    transition.from_access, transition.to_access
                ));
            }
        }

        Ok(())
    }

    fn validate_materialized_access_lease(
        &self,
        graph: &CompiledRenderGraph,
        compiled_binding: CompiledRenderGraphAccessAllocationBinding,
    ) -> Result<(), String> {
        let key = compiled_binding.key;
        let access_id = key.access_id;
        let transient_key = self.transient_access_key(access_id);
        let persistent_key = self.persistent_texture_access_key(access_id);
        let external_key = self.external_access_key(access_id);
        let materialized_kind_count = [transient_key, persistent_key, external_key]
            .into_iter()
            .filter(Option::is_some)
            .count();
        if materialized_kind_count > 1 {
            return Err(format!(
                "device execution access {:?} has multiple physical lease owners",
                access_id
            ));
        }

        if let Some(expected_allocation) = compiled_binding.physical_allocation {
            if transient_key != Some(key)
                || persistent_key.is_some()
                || external_key.is_some()
                || self.transient_physical_allocation_for_access(access_id)
                    != Some(expected_allocation)
            {
                return Err(format!(
                    "transient device access {:?} does not own its compiler-selected physical allocation {:?}",
                    access_id, expected_allocation
                ));
            }
            return self.validate_transient_access_scope(key);
        }

        if graph
            .persistent_texture_backing_resource(key.resource)
            .is_some()
        {
            if persistent_key != Some(key) || transient_key.is_some() || external_key.is_some() {
                return Err(format!(
                    "persistent texture access {:?} has no exact persistent physical lease",
                    access_id
                ));
            }
            if !matches!(key.range, RenderGraphResourceAccessRange::Texture(_)) {
                return Err(format!(
                    "persistent texture access {:?} has a non-texture subresource range",
                    access_id
                ));
            }
            self.persistent_texture_for_access(access_id)?;
            self.persistent_texture_view_for_access(access_id)?;
            return Ok(());
        }

        let declaration = graph.resource_declaration(key.resource).ok_or_else(|| {
            format!(
                "device execution access {:?} references an undeclared resource",
                access_id
            )
        })?;
        if declaration.kind == RenderGraphResourceKind::External {
            if transient_key.is_some() || persistent_key.is_some() {
                return Err(format!(
                    "external device access {:?} has a graph-owned physical lease",
                    access_id
                ));
            }
            if let Some((packet_key, external_binding)) =
                self.external_access_contract_for_access(access_id)
            {
                if packet_key != key {
                    return Err(format!(
                        "external device access {:?} differs from its typed lease packet key",
                        access_id
                    ));
                }
                if external_binding.is_required() && external_key != Some(key) {
                    return Err(format!(
                        "required external device access {:?} has no physical lease",
                        access_id
                    ));
                }
            } else {
                return Err(format!(
                    "external device access {:?} is missing from its typed lease packet",
                    access_id
                ));
            }
            if external_key == Some(key) {
                return self.validate_external_access_scope(key);
            }
            // Optional and report-only external accesses may legitimately have
            // no WGPU object. They remain logical graph rows, not device leases.
            return Ok(());
        }

        if transient_key.is_some() || persistent_key.is_some() || external_key.is_some() {
            return Err(format!(
                "imported device access {:?} is registered in the wrong physical lease table",
                access_id
            ));
        }
        let lifetime = graph.resource_lifetime(key.resource).ok_or_else(|| {
            format!(
                "device execution access {:?} has no graph resource lifetime",
                access_id
            )
        })?;
        if !lifetime.imported || compiled_binding.physical_allocation.is_some() {
            return Err(format!(
                "device execution access {:?} has no exact physical lease class",
                access_id
            ));
        }
        self.validate_imported_access_scope(&declaration.name, key)
    }

    fn validate_transient_access_scope(
        &self,
        key: RenderGraphVersionedAccessKey,
    ) -> Result<(), String> {
        match key.range {
            RenderGraphResourceAccessRange::Texture(_) => {
                self.transient_texture_view_for_access(key.access_id)?;
                // A view-only imported backing pins the native texture too.
                // Materialization accepts it only for the descriptor's full
                // view; copy/readback paths that need the Texture itself still
                // request that backing explicitly at their own boundary.
                Ok(())
            }
            RenderGraphResourceAccessRange::Buffer(_) => {
                let (buffer, actual_range) =
                    self.transient_buffer_binding_for_access(key.access_id)?;
                let expected_range = buffer_access_range(key.range, buffer.size(), key.access_id)?;
                if actual_range != expected_range {
                    return Err(format!(
                        "transient buffer access {:?} has physical range {:?}, expected {:?}",
                        key.access_id, actual_range, expected_range
                    ));
                }
                Ok(())
            }
            RenderGraphResourceAccessRange::UnresolvedExternal => Err(format!(
                "transient device access {:?} has unresolved physical scope",
                key.access_id
            )),
        }
    }

    fn validate_external_access_scope(
        &self,
        key: RenderGraphVersionedAccessKey,
    ) -> Result<(), String> {
        match key.range {
            RenderGraphResourceAccessRange::Texture(_)
            | RenderGraphResourceAccessRange::UnresolvedExternal => {
                self.external_texture_view_for_access(key.access_id)?;
                Ok(())
            }
            range @ RenderGraphResourceAccessRange::Buffer(_) => {
                let (buffer, actual_range) =
                    self.external_buffer_binding_for_access(key.access_id)?;
                let expected_range = buffer_access_range(range, buffer.size(), key.access_id)?;
                if actual_range != expected_range {
                    return Err(format!(
                        "external buffer access {:?} has physical range {:?}, expected {:?}",
                        key.access_id, actual_range, expected_range
                    ));
                }
                Ok(())
            }
        }
    }

    fn validate_imported_access_scope(
        &self,
        resource_name: &str,
        key: RenderGraphVersionedAccessKey,
    ) -> Result<(), String> {
        match key.range {
            RenderGraphResourceAccessRange::Texture(range) => {
                if self.physical_texture(resource_name).is_some() {
                    self.physical_texture_subresource_view(resource_name, range)?;
                    return Ok(());
                }
                self.texture_view(resource_name).ok_or_else(|| {
                    format!(
                        "imported texture access {:?} has no physical view `{resource_name}`",
                        key.access_id
                    )
                })?;
                let desc = self.physical_texture_desc(resource_name).ok_or_else(|| {
                    format!(
                        "imported texture access {:?} has a view-only lease without a physical descriptor",
                        key.access_id
                    )
                })?;
                if !texture_range_covers_full_view(range, desc) {
                    return Err(format!(
                        "imported texture access {:?} requires subresource scope {:?}, but its physical lease is view-only",
                        key.access_id, range
                    ));
                }
                Ok(())
            }
            RenderGraphResourceAccessRange::Buffer(_) => {
                let buffer = self.buffer(resource_name).ok_or_else(|| {
                    format!(
                        "imported buffer access {:?} has no physical buffer `{resource_name}`",
                        key.access_id
                    )
                })?;
                buffer_access_range(key.range, buffer.size(), key.access_id)?;
                Ok(())
            }
            RenderGraphResourceAccessRange::UnresolvedExternal => {
                if self.texture_view(resource_name).is_some()
                    || self.physical_texture(resource_name).is_some()
                    || self.buffer(resource_name).is_some()
                {
                    Ok(())
                } else {
                    Err(format!(
                        "imported access {:?} has no physical lease for `{resource_name}`",
                        key.access_id
                    ))
                }
            }
        }
    }
}

fn buffer_access_range(
    range: RenderGraphResourceAccessRange,
    buffer_size: wgpu::BufferAddress,
    access_id: RenderGraphResourceAccessId,
) -> Result<Range<wgpu::BufferAddress>, String> {
    let resolved = match range {
        RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange { offset, size }) => {
            let end = size
                .map(|size| offset.checked_add(size))
                .unwrap_or(Some(buffer_size))
                .ok_or_else(|| format!("buffer access {:?} range overflows", access_id))?;
            offset..end
        }
        RenderGraphResourceAccessRange::UnresolvedExternal => 0..buffer_size,
        RenderGraphResourceAccessRange::Texture(_) => {
            return Err(format!(
                "buffer access {:?} has a texture subresource range",
                access_id
            ));
        }
    };
    if resolved.start >= resolved.end || resolved.end > buffer_size {
        return Err(format!(
            "buffer access {:?} physical range {:?} exceeds buffer size {}",
            access_id, resolved, buffer_size
        ));
    }
    Ok(resolved)
}

use std::collections::{HashMap, HashSet, VecDeque};

use crate::rhi::{BufferDesc, BufferUsage, TextureUsage};

use super::super::error::RenderGraphError;
use super::super::graph::{
    CompiledRenderGraph, CompiledRenderGraphCompileWork, CompiledRenderPass,
};
use super::super::types::{
    ComputeBindingKind, RenderGraphComputeDispatchExtent, RenderGraphComputeShaderSource,
    RenderGraphExternalResourceType, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDeclaration, RenderGraphResourceDesc,
    RenderGraphResourceLifetime, RenderPassId,
};
use super::{access_validation, RenderGraphBuilder, ResourceAccessKind, ResourceNode};

impl RenderGraphBuilder {
    pub fn compile(self) -> Result<CompiledRenderGraph, RenderGraphError> {
        self.validate_unique_pass_names()?;
        self.validate_unique_resource_names()?;
        self.validate_resource_admission()?;
        self.validate_compute_dispatch_resources()?;
        self.validate_compute_pass_metadata()?;
        let resource_names = self.resource_names();
        access_validation::validate_resource_access_ranges(&self, &resource_names)?;
        let mut manual_dependencies = self
            .passes
            .iter()
            .map(|pass| pass.dependencies.clone())
            .collect::<Vec<_>>();
        self.add_explicit_version_dependencies(&mut manual_dependencies)?;
        let manual_order = self.topological_order(&manual_dependencies)?;
        let inferred_dependencies =
            self.infer_resource_dependencies(&resource_names, &manual_order, manual_dependencies)?;
        let ordered = self.topological_order(&inferred_dependencies.execution)?;
        let culling = self.cull_passes(
            &inferred_dependencies.culling,
            &inferred_dependencies.cull_roots,
        )?;
        let compile_work = CompiledRenderGraphCompileWork {
            resource_access_visit_count: inferred_dependencies.resource_access_visit_count,
            execution_dependency_count: inferred_dependencies.execution_dependency_count,
            provenance_dependency_count: inferred_dependencies.provenance_dependency_count,
            cull_root_count: culling.root_count,
            cull_dependency_visit_count: culling.dependency_visit_count,
        };
        let compiled_passes = ordered
            .iter()
            .map(|id| {
                let pass = &self.passes[id.0];
                let resources = pass
                    .resources
                    .iter()
                    .map(|access| RenderGraphPassResourceAccess {
                        name: resource_names
                            .get(&access.resource)
                            .copied()
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("{:?}", access.resource)),
                        kind: access.resource.kind(),
                        access: render_graph_resource_access_kind(access.kind),
                        attachment_ops: access.attachment_ops,
                    })
                    .collect();
                CompiledRenderPass {
                    id: *id,
                    name: pass.name.clone(),
                    declared_queue: pass.declared_queue,
                    queue: pass.queue,
                    flags: pass.flags,
                    dependencies: inferred_dependencies.execution[id.0]
                        .iter()
                        .copied()
                        .filter(|dependency| !culling.culled.contains(dependency))
                        .collect(),
                    culled: culling.culled.contains(id),
                    executor_id: pass.executor_id.clone(),
                    compute_workload: pass.compute_workload.clone(),
                    compute_pass_metadata: pass.compute_pass_metadata.clone(),
                    resources,
                }
            })
            .collect::<Vec<_>>();
        let resource_declarations = self.resource_declarations();
        let lifetimes =
            self.resource_lifetimes(&ordered, &culling.culled, &resource_declarations)?;
        let pass_resource_versions = ordered
            .iter()
            .map(|id| inferred_dependencies.resource_access_versions[id.0].clone())
            .collect();
        let pass_resource_input_versions = ordered
            .iter()
            .map(|id| {
                self.passes[id.0]
                    .resources
                    .iter()
                    .map(|access| {
                        access
                            .input_version
                            .map(|token| {
                                self.resolve_compiled_input_version(token, &inferred_dependencies)
                            })
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, RenderGraphError>>()
            })
            .collect::<Result<Vec<_>, RenderGraphError>>()?;
        let pass_resource_access_metadata = ordered
            .iter()
            .map(|id| inferred_dependencies.resource_access_metadata[id.0].clone())
            .collect();

        CompiledRenderGraph::new(
            self.name,
            compiled_passes,
            resource_declarations,
            lifetimes,
            pass_resource_versions,
            pass_resource_input_versions,
            pass_resource_access_metadata,
            compile_work,
        )
    }

    fn validate_unique_pass_names(&self) -> Result<(), RenderGraphError> {
        let mut seen = HashSet::new();
        for pass in &self.passes {
            if !seen.insert(pass.name.as_str()) {
                return Err(RenderGraphError::DuplicatePassName {
                    pass: pass.name.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_unique_resource_names(&self) -> Result<(), RenderGraphError> {
        let mut seen = HashSet::new();
        for resource in &self.resources {
            if !seen.insert(resource.name.as_str()) {
                return Err(RenderGraphError::DuplicateResourceName {
                    resource: resource.name.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_resource_admission(&self) -> Result<(), RenderGraphError> {
        for resource in &self.resources {
            if resource.external_texture_desc.is_some()
                && resource.external_binding.resource_type
                    != RenderGraphExternalResourceType::Texture
            {
                return Err(RenderGraphError::ExternalTextureBindingTypeMismatch {
                    resource: resource.name.clone(),
                });
            }
            if resource.external_buffer_desc.is_some()
                && resource.external_binding.resource_type
                    != RenderGraphExternalResourceType::Buffer
            {
                return Err(RenderGraphError::ExternalBufferBindingTypeMismatch {
                    resource: resource.name.clone(),
                });
            }
            if let Some(desc) = resource.external_buffer_desc.as_ref() {
                if desc.size_bytes == 0
                    || desc.usage == BufferUsage::NONE
                    || desc.usage.has_unknown_bits()
                {
                    return Err(RenderGraphError::ExternalBufferDescriptorInvalid {
                        resource: resource.name.clone(),
                    });
                }
            }
            let texture_desc = match &resource.desc {
                RenderGraphResourceDesc::Texture(desc) => Some(desc),
                RenderGraphResourceDesc::External => resource.external_texture_desc.as_ref(),
                RenderGraphResourceDesc::Buffer(_) => None,
            };
            let Some(desc) = texture_desc else {
                continue;
            };
            if desc.is_sparse_reserved() {
                return Err(RenderGraphError::SparseTextureUnsupported {
                    resource: resource.name.clone(),
                });
            }
            if desc.usage.contains(TextureUsage::STORAGE)
                && !desc.format.supports_write_only_storage()
            {
                return Err(RenderGraphError::TextureStorageUsageUnsupported {
                    resource: resource.name.clone(),
                    format: desc.format,
                });
            }
        }
        Ok(())
    }

    fn validate_compute_dispatch_resources(&self) -> Result<(), RenderGraphError> {
        let resources_by_name = self
            .resources
            .iter()
            .map(|resource| (resource.name.as_str(), resource))
            .collect::<HashMap<_, _>>();
        for pass in &self.passes {
            let Some(workload) = &pass.compute_workload else {
                continue;
            };
            match &workload.dispatch_extent {
                RenderGraphComputeDispatchExtent::FromBuffer { buffer, .. } => {
                    let is_declared_read_buffer = resources_by_name
                        .get(buffer.as_str())
                        .is_some_and(|resource| {
                            let is_buffer =
                                matches!(&resource.desc, RenderGraphResourceDesc::Buffer(_))
                                    || resource.external_binding.resource_type
                                        == RenderGraphExternalResourceType::Buffer;
                            is_buffer
                                && pass.resources.iter().any(|access| {
                                    access.resource == resource.resource
                                        && access.kind == ResourceAccessKind::Read
                                })
                        });
                    if !is_declared_read_buffer {
                        return Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
                            pass: pass.name.clone(),
                            resource: buffer.clone(),
                            required_access: "read buffer",
                        });
                    }
                }
                RenderGraphComputeDispatchExtent::PerPixel { target, .. } => {
                    let is_declared_texture =
                        resources_by_name
                            .get(target.as_str())
                            .is_some_and(|resource| {
                                let is_texture =
                                    matches!(&resource.desc, RenderGraphResourceDesc::Texture(_))
                                        || resource.external_binding.resource_type
                                            == RenderGraphExternalResourceType::Texture;
                                is_texture
                                    && pass
                                        .resources
                                        .iter()
                                        .any(|access| access.resource == resource.resource)
                            });
                    if !is_declared_texture {
                        return Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
                            pass: pass.name.clone(),
                            resource: target.clone(),
                            required_access: "read or write texture",
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_compute_pass_metadata(&self) -> Result<(), RenderGraphError> {
        const INDIRECT_DISPATCH_OFFSET_ALIGNMENT_BYTES: u64 = 4;
        let resources_by_name = self
            .resources
            .iter()
            .map(|resource| (resource.name.as_str(), resource))
            .collect::<HashMap<_, _>>();

        for pass in &self.passes {
            let Some(metadata) = &pass.compute_pass_metadata else {
                continue;
            };
            let Some(workload) = &pass.compute_workload else {
                return Err(RenderGraphError::ComputePassMetadataMissingWorkload {
                    pass: pass.name.clone(),
                });
            };
            if metadata.entry_point.trim().is_empty() {
                return Err(RenderGraphError::ComputePassEntryPointEmpty {
                    pass: pass.name.clone(),
                });
            }
            if matches!(
                &metadata.shader,
                RenderGraphComputeShaderSource::Wgsl { source, .. } if source.trim().is_empty()
            ) {
                return Err(RenderGraphError::ComputePassShaderSourceEmpty {
                    pass: pass.name.clone(),
                });
            }
            if workload.workgroup_size.contains(&0) {
                return Err(RenderGraphError::InvalidComputeWorkgroupSize {
                    pass: pass.name.clone(),
                });
            }
            let mut declared_bindings = HashSet::with_capacity(metadata.bindings.len());
            for binding in &metadata.bindings {
                if !declared_bindings.insert(binding.binding) {
                    return Err(RenderGraphError::DuplicateComputeBinding {
                        pass: pass.name.clone(),
                        binding: binding.binding,
                    });
                }
                if let Some(mip_level) = binding.texture_mip_level {
                    if !matches!(
                        binding.kind,
                        ComputeBindingKind::SampledTexture
                            | ComputeBindingKind::StorageTextureWrite
                    ) {
                        return Err(RenderGraphError::ComputeTextureMipBindingNotTexture {
                            pass: pass.name.clone(),
                            binding: binding.binding,
                            mip_level,
                        });
                    }
                    if let Some(resource) = resources_by_name.get(binding.resource.as_str()) {
                        match &resource.desc {
                            RenderGraphResourceDesc::Texture(desc)
                                if mip_level >= desc.mip_levels =>
                            {
                                return Err(RenderGraphError::ComputeTextureMipOutOfRange {
                                    pass: pass.name.clone(),
                                    binding: binding.binding,
                                    resource: binding.resource.clone(),
                                    mip_level,
                                    mip_levels: desc.mip_levels,
                                });
                            }
                            RenderGraphResourceDesc::External => {
                                return Err(
                                    RenderGraphError::ComputeTextureMipRequiresTransientTexture {
                                        pass: pass.name.clone(),
                                        binding: binding.binding,
                                        resource: binding.resource.clone(),
                                        mip_level,
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                }
                if let Some(range) = binding.buffer_range {
                    if !matches!(
                        binding.kind,
                        ComputeBindingKind::UniformBuffer
                            | ComputeBindingKind::StorageBufferRead
                            | ComputeBindingKind::StorageBufferReadWrite
                    ) {
                        return Err(RenderGraphError::ComputeBufferRangeBindingNotBuffer {
                            pass: pass.name.clone(),
                            binding: binding.binding,
                            offset: range.offset,
                            size: range.size,
                        });
                    }
                    if matches!(range.size, Some(0)) {
                        return Err(RenderGraphError::ComputeBufferBindingRangeEmpty {
                            pass: pass.name.clone(),
                            binding: binding.binding,
                            resource: binding.resource.clone(),
                        });
                    }
                    if let Some(desc) = resources_by_name
                        .get(binding.resource.as_str())
                        .and_then(|resource| resource_buffer_desc(resource))
                    {
                        if buffer_range_exceeds_buffer(range.offset, range.size, desc.size_bytes) {
                            return Err(RenderGraphError::ComputeBufferBindingRangeOutOfBounds {
                                pass: pass.name.clone(),
                                binding: binding.binding,
                                resource: binding.resource.clone(),
                                offset: range.offset,
                                size: range.size,
                                buffer_size: desc.size_bytes,
                            });
                        }
                    }
                }
                let (requires_buffer, requires_read, requires_write, required_access) =
                    match binding.kind {
                        ComputeBindingKind::UniformBuffer
                        | ComputeBindingKind::StorageBufferRead => {
                            (true, true, false, "read buffer")
                        }
                        ComputeBindingKind::StorageBufferReadWrite => {
                            (true, true, true, "read/write buffer")
                        }
                        ComputeBindingKind::SampledTexture => (false, true, false, "read texture"),
                        ComputeBindingKind::StorageTextureWrite => {
                            (false, false, true, "write texture")
                        }
                    };
                let required_buffer_usage = match binding.kind {
                    ComputeBindingKind::UniformBuffer => Some(BufferUsage::UNIFORM),
                    ComputeBindingKind::StorageBufferRead
                    | ComputeBindingKind::StorageBufferReadWrite => Some(BufferUsage::STORAGE),
                    ComputeBindingKind::SampledTexture
                    | ComputeBindingKind::StorageTextureWrite => None,
                };
                let is_declared = resources_by_name
                    .get(binding.resource.as_str())
                    .is_some_and(|resource| {
                        let resource_type_matches = if requires_buffer {
                            matches!(&resource.desc, RenderGraphResourceDesc::Buffer(_))
                                || resource.external_binding.resource_type
                                    == RenderGraphExternalResourceType::Buffer
                        } else {
                            matches!(&resource.desc, RenderGraphResourceDesc::Texture(_))
                                || resource.external_binding.resource_type
                                    == RenderGraphExternalResourceType::Texture
                        };
                        resource_type_matches
                            && (!requires_read
                                || pass.resources.iter().any(|access| {
                                    access.resource == resource.resource
                                        && access.kind == ResourceAccessKind::Read
                                }))
                            && (!requires_write
                                || pass.resources.iter().any(|access| {
                                    access.resource == resource.resource
                                        && access.kind == ResourceAccessKind::Write
                                }))
                    });
                if !is_declared {
                    return Err(RenderGraphError::ComputeBindingResourceNotDeclared {
                        pass: pass.name.clone(),
                        binding: binding.binding,
                        resource: binding.resource.clone(),
                        required_access,
                    });
                }
                if let Some(required_usage) = required_buffer_usage {
                    if let Some(desc) = resources_by_name
                        .get(binding.resource.as_str())
                        .and_then(|resource| resource_buffer_desc(resource))
                    {
                        if !desc.usage.contains(required_usage) {
                            return Err(RenderGraphError::ComputeBufferBindingUsageMissing {
                                pass: pass.name.clone(),
                                binding: binding.binding,
                                resource: binding.resource.clone(),
                                required: required_usage,
                                actual: desc.usage,
                            });
                        }
                    }
                }
            }
            match &workload.dispatch_extent {
                RenderGraphComputeDispatchExtent::FromBuffer { offset, .. }
                    if offset % INDIRECT_DISPATCH_OFFSET_ALIGNMENT_BYTES != 0 =>
                {
                    return Err(RenderGraphError::ComputeIndirectDispatchOffsetUnaligned {
                        pass: pass.name.clone(),
                        offset: *offset,
                        alignment: INDIRECT_DISPATCH_OFFSET_ALIGNMENT_BYTES,
                    });
                }
                RenderGraphComputeDispatchExtent::PerPixel { local_size, .. }
                    if *local_size != [workload.workgroup_size[0], workload.workgroup_size[1]] =>
                {
                    return Err(RenderGraphError::PerPixelComputeWorkgroupMismatch {
                        pass: pass.name.clone(),
                        local_size: *local_size,
                        workgroup_size: workload.workgroup_size,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn resource_names(&self) -> HashMap<RenderGraphResource, &str> {
        self.resources
            .iter()
            .map(|resource| (resource.resource, resource.name.as_str()))
            .collect()
    }

    fn resource_declarations(&self) -> Vec<RenderGraphResourceDeclaration> {
        self.resources
            .iter()
            .map(|resource| RenderGraphResourceDeclaration {
                resource: resource.resource,
                name: resource.name.clone(),
                kind: resource.resource.kind(),
                desc: resource.desc.clone(),
                external_binding: resource.external_binding,
                external_texture_desc: resource.external_texture_desc.clone(),
                external_buffer_desc: resource.external_buffer_desc.clone(),
                texture_view_alias: resource.texture_view_alias,
                imported: matches!(&resource.desc, RenderGraphResourceDesc::External),
                usage: resource.usage,
            })
            .collect()
    }

    fn topological_order(
        &self,
        dependencies: &[Vec<RenderPassId>],
    ) -> Result<Vec<RenderPassId>, RenderGraphError> {
        let mut indegree = vec![0_usize; self.passes.len()];
        let mut dependents = vec![Vec::new(); self.passes.len()];

        for pass in &self.passes {
            indegree[pass.id.0] = dependencies[pass.id.0].len();
            for dependency in &dependencies[pass.id.0] {
                dependents[dependency.0].push(pass.id);
            }
        }

        let mut ready = VecDeque::new();
        for pass in &self.passes {
            if indegree[pass.id.0] == 0 {
                ready.push_back(pass.id);
            }
        }

        let mut ordered = Vec::with_capacity(self.passes.len());
        while let Some(id) = ready.pop_front() {
            ordered.push(id);

            for dependent in &dependents[id.0] {
                indegree[dependent.0] -= 1;
                if indegree[dependent.0] == 0 {
                    ready.push_back(*dependent);
                }
            }
        }

        if ordered.len() != self.passes.len() {
            return Err(RenderGraphError::CycleDetected {
                graph_name: self.name.clone(),
            });
        }

        Ok(ordered)
    }

    fn cull_passes(
        &self,
        dependencies: &[Vec<RenderPassId>],
        cull_roots: &[RenderPassId],
    ) -> Result<CullingResult, RenderGraphError> {
        let mut live_passes = HashSet::<RenderPassId>::new();
        let mut pending = cull_roots.to_vec();
        pending.extend(self.passes.iter().filter_map(|pass| {
            (!pass.flags.allow_culling || pass.flags.has_side_effects).then_some(pass.id)
        }));
        let root_count = pending.len();

        if !self.passes.is_empty() && pending.is_empty() {
            return Err(RenderGraphError::MissingCullRoot {
                graph_name: self.name.clone(),
            });
        }

        // Walk only semantic provenance edges. Execution still carries WAW/WAR edges to
        // serialize reused backing resources, but those hazards do not make overwritten data live.
        let mut dependency_visit_count = 0;
        while let Some(pass) = pending.pop() {
            if live_passes.insert(pass) {
                dependency_visit_count += dependencies[pass.0].len();
                pending.extend(dependencies[pass.0].iter().copied());
            }
        }

        let culled = self
            .passes
            .iter()
            .map(|pass| pass.id)
            .filter(|id| !live_passes.contains(id))
            .collect();
        Ok(CullingResult {
            culled,
            root_count,
            dependency_visit_count,
        })
    }

    fn resource_lifetimes(
        &self,
        ordered: &[RenderPassId],
        culled: &HashSet<RenderPassId>,
        resource_declarations: &[RenderGraphResourceDeclaration],
    ) -> Result<Vec<RenderGraphResourceLifetime>, RenderGraphError> {
        let resource_declaration_indices = resource_declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.resource, index))
            .collect::<HashMap<_, _>>();
        let mut spans = HashMap::<RenderGraphResource, (usize, usize)>::new();

        for (pass_order, pass_id) in ordered.iter().enumerate() {
            if culled.contains(pass_id) {
                continue;
            }
            let pass = &self.passes[pass_id.0];
            for access in &pass.resources {
                extend_resource_lifetime_span(&mut spans, access.resource, pass_order);
                if let Some(parent) = self
                    .resources
                    .iter()
                    .find(|resource| resource.resource == access.resource)
                    .and_then(|resource| resource.texture_view_alias)
                    .map(|alias| RenderGraphResource::TransientTexture(alias.parent))
                {
                    // A texture view owns no physical slot, but each live view
                    // access extends its parent backing lifetime.
                    extend_resource_lifetime_span(&mut spans, parent, pass_order);
                }
            }
        }

        let mut lifetimes = Vec::with_capacity(spans.len());
        for (resource, (first_pass, last_pass)) in spans {
            let declaration = resource_declaration_indices
                .get(&resource)
                .and_then(|index| resource_declarations.get(*index))
                .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                })?;
            let usage = declaration.usage;
            // Readback consumers run after graph recording, so their backing must
            // remain unique and live through the terminal compiled pass.
            let last_pass = if usage.readback {
                ordered.len().saturating_sub(1)
            } else {
                last_pass
            };
            lifetimes.push(RenderGraphResourceLifetime {
                resource,
                name: declaration.name.clone(),
                kind: declaration.kind,
                desc: declaration.desc.clone(),
                external_binding: declaration.external_binding,
                external_texture_desc: declaration.external_texture_desc.clone(),
                external_buffer_desc: declaration.external_buffer_desc.clone(),
                texture_view_alias: declaration.texture_view_alias,
                first_pass,
                last_pass,
                imported: declaration.imported,
                usage,
            });
        }
        lifetimes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(lifetimes)
    }
}

fn extend_resource_lifetime_span(
    spans: &mut HashMap<RenderGraphResource, (usize, usize)>,
    resource: RenderGraphResource,
    pass_order: usize,
) {
    spans
        .entry(resource)
        .and_modify(|span| {
            span.0 = span.0.min(pass_order);
            span.1 = span.1.max(pass_order);
        })
        .or_insert((pass_order, pass_order));
}

fn buffer_range_exceeds_buffer(offset: u64, size: Option<u64>, buffer_size: u64) -> bool {
    match size {
        Some(size) => match offset.checked_add(size) {
            Some(end) => end > buffer_size,
            None => true,
        },
        None => offset >= buffer_size,
    }
}

fn resource_buffer_desc(resource: &ResourceNode) -> Option<&BufferDesc> {
    match &resource.desc {
        RenderGraphResourceDesc::Buffer(desc) => Some(desc),
        RenderGraphResourceDesc::External => resource.external_buffer_desc.as_ref(),
        RenderGraphResourceDesc::Texture(_) => None,
    }
}

struct CullingResult {
    culled: HashSet<RenderPassId>,
    root_count: usize,
    dependency_visit_count: usize,
}

fn render_graph_resource_access_kind(kind: ResourceAccessKind) -> RenderGraphResourceAccessKind {
    match kind {
        ResourceAccessKind::Read => RenderGraphResourceAccessKind::Read,
        ResourceAccessKind::Write => RenderGraphResourceAccessKind::Write,
    }
}

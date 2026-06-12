use std::sync::Arc;

use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::resources::{GpuMeshResource, PipelineKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshPipelineVariantId(u32);

impl MeshPipelineVariantId {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshPassPipelineKind {
    DepthPrepass,
    Base,
    ShadowDepth,
    ShadowDepthAlphaMask,
    MotionVector,
}

#[derive(Clone)]
pub(crate) struct MeshBindHandle {
    id: u64,
    bind_group: Option<wgpu::BindGroup>,
}

impl MeshBindHandle {
    pub(crate) fn new(id: u64, bind_group: wgpu::BindGroup) -> Self {
        Self {
            id,
            bind_group: Some(bind_group),
        }
    }

    #[cfg(test)]
    pub(crate) const fn test(id: u64) -> Self {
        Self {
            id,
            bind_group: None,
        }
    }

    pub(crate) const fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        self.bind_group
            .as_ref()
            .expect("mesh draw command bind handle must carry a WGPU bind group")
    }
}

#[derive(Clone)]
pub(crate) struct MeshGeometryHandle {
    id: u64,
    mesh: Option<Arc<GpuMeshResource>>,
}

impl MeshGeometryHandle {
    pub(crate) fn new(id: u64, mesh: Arc<GpuMeshResource>) -> Self {
        Self {
            id,
            mesh: Some(mesh),
        }
    }

    #[cfg(test)]
    pub(crate) const fn test(id: u64) -> Self {
        Self { id, mesh: None }
    }

    pub(crate) const fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn mesh(&self) -> &GpuMeshResource {
        self.mesh
            .as_deref()
            .expect("mesh draw command geometry handle must carry a GPU mesh resource")
    }
}

#[derive(Clone)]
pub(crate) enum MeshDrawArgs {
    DirectIndexed {
        first_index: u32,
        index_count: u32,
        first_instance: u32,
        instance_count: u32,
    },
    IndexedIndirect {
        buffer_id: u64,
        buffer: Option<Arc<wgpu::Buffer>>,
        offset: u64,
    },
}

impl MeshDrawArgs {
    pub(crate) const fn direct_indexed(first_index: u32, index_count: u32) -> Self {
        Self::DirectIndexed {
            first_index,
            index_count,
            first_instance: 0,
            instance_count: 1,
        }
    }

    pub(crate) fn indexed_indirect(buffer_id: u64, buffer: Arc<wgpu::Buffer>, offset: u64) -> Self {
        Self::IndexedIndirect {
            buffer_id,
            buffer: Some(buffer),
            offset,
        }
    }

    #[cfg(test)]
    pub(crate) const fn test_indexed_indirect(buffer_id: u64, offset: u64) -> Self {
        Self::IndexedIndirect {
            buffer_id,
            buffer: None,
            offset,
        }
    }

    pub(crate) fn with_instance_span(self, first_instance: u32, instance_count: u32) -> Self {
        debug_assert!(instance_count > 0);
        match self {
            Self::DirectIndexed {
                first_index,
                index_count,
                ..
            } => Self::DirectIndexed {
                first_index,
                index_count,
                first_instance,
                instance_count,
            },
            indirect @ Self::IndexedIndirect { .. } => indirect,
        }
    }

    pub(crate) const fn is_indirect(&self) -> bool {
        matches!(self, Self::IndexedIndirect { .. })
    }

    pub(crate) fn record_indexed_draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        match self {
            Self::DirectIndexed {
                first_index,
                index_count,
                first_instance,
                instance_count,
            } => {
                let index_end = (*first_index)
                    .checked_add(*index_count)
                    .expect("direct indexed mesh command index range overflowed");
                let instance_end = (*first_instance)
                    .checked_add(*instance_count)
                    .expect("direct indexed mesh command instance range overflowed");
                pass.draw_indexed(*first_index..index_end, 0, *first_instance..instance_end);
            }
            Self::IndexedIndirect { buffer, offset, .. } => {
                pass.draw_indexed_indirect(
                    buffer
                        .as_deref()
                        .expect("indexed indirect mesh command must carry an indirect args buffer"),
                    *offset,
                );
            }
        }
    }
}

#[derive(Clone)]
pub(crate) enum DrawInstanceSource {
    GpuSceneInstance {
        first_instance_index: u32,
        instance_count: u32,
    },
}

impl DrawInstanceSource {
    pub(crate) const fn uses_gpu_scene(&self) -> bool {
        true
    }
}

#[derive(Clone)]
pub(crate) struct MeshDrawCommand {
    pub(crate) source_draw_index: usize,
    pub(crate) phase: RenderPhase,
    pub(crate) pipeline_kind: MeshPassPipelineKind,
    pipeline_key: PipelineKey,
    pub(crate) pipeline_variant_id: MeshPipelineVariantId,
    pub(crate) sort_key: u64,
    pub(crate) instance_source: DrawInstanceSource,
    pub(crate) material_textures: Option<MeshBindHandle>,
    pub(crate) base_color_texture: Option<MeshBindHandle>,
    pub(crate) material: Option<MeshBindHandle>,
    pub(crate) standard_material: Option<MeshBindHandle>,
    pub(crate) gpu_scene_bind_group: Option<MeshBindHandle>,
    pub(crate) geometry: MeshGeometryHandle,
    pub(crate) draw_args: MeshDrawArgs,
}

impl MeshDrawCommand {
    pub(crate) fn new(
        phase: RenderPhase,
        pipeline_kind: MeshPassPipelineKind,
        pipeline_key: PipelineKey,
        pipeline_variant_id: MeshPipelineVariantId,
        sort_key: u64,
        instance_source: DrawInstanceSource,
        geometry: MeshGeometryHandle,
        draw_args: MeshDrawArgs,
    ) -> Self {
        Self {
            source_draw_index: 0,
            phase,
            pipeline_kind,
            pipeline_key,
            pipeline_variant_id,
            sort_key,
            instance_source,
            material_textures: None,
            base_color_texture: None,
            material: None,
            standard_material: None,
            gpu_scene_bind_group: None,
            geometry,
            draw_args,
        }
    }

    pub(crate) fn pipeline_key(&self) -> &PipelineKey {
        &self.pipeline_key
    }

    pub(crate) fn with_source_draw_index(mut self, source_draw_index: usize) -> Self {
        self.source_draw_index = source_draw_index;
        self
    }

    pub(crate) fn with_material_textures(mut self, handle: MeshBindHandle) -> Self {
        self.material_textures = Some(handle);
        self
    }

    pub(crate) fn with_base_color_texture(mut self, handle: MeshBindHandle) -> Self {
        self.base_color_texture = Some(handle);
        self
    }

    pub(crate) fn with_material(mut self, handle: MeshBindHandle) -> Self {
        self.material = Some(handle);
        self
    }

    pub(crate) fn with_standard_material(mut self, handle: MeshBindHandle) -> Self {
        self.standard_material = Some(handle);
        self
    }

    pub(crate) fn with_gpu_scene_bind_group(mut self, handle: MeshBindHandle) -> Self {
        self.gpu_scene_bind_group = Some(handle);
        self
    }

    pub(crate) fn bind_geometry_buffers<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        let mesh = self.geometry.mesh();
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    }

    pub(crate) fn record_indexed_draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        self.draw_args.record_indexed_draw(pass);
    }
}

use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::framework::render::{RenderPhase, RenderPhaseSortComponents, packed_sort_key_u64};
use crate::core::framework::scene::EntityId;
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
    GBuffer,
    Base,
    ShadowDepth,
    ShadowDepthAlphaMask,
    Velocity,
    TaaReactiveMask,
    TaaReactiveMaterialMask,
    HitProxy,
}

#[derive(Clone)]
pub(crate) struct MeshBindHandle {
    id: u64,
    bind_group: Option<wgpu::BindGroup>,
}

static NEXT_MESH_BIND_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

fn next_mesh_bind_handle_id() -> u64 {
    NEXT_MESH_BIND_HANDLE_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .expect("mesh bind handle ID space is exhausted")
}

impl MeshBindHandle {
    pub(crate) fn new(bind_group: wgpu::BindGroup) -> Self {
        Self {
            id: next_mesh_bind_handle_id(),
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

#[derive(Clone, Copy)]
struct MeshDirectIndexedTopology {
    first_index: u32,
    index_count: u32,
}

impl MeshDirectIndexedTopology {
    fn from_draw_args(draw_args: &MeshDrawArgs) -> Option<Self> {
        match draw_args {
            MeshDrawArgs::DirectIndexed {
                first_index,
                index_count,
                ..
            } => Some(Self {
                first_index: *first_index,
                index_count: *index_count,
            }),
            MeshDrawArgs::IndexedIndirect { .. } => None,
        }
    }

    fn with_instance_span(self, first_instance: u32, instance_count: u32) -> MeshDrawArgs {
        MeshDrawArgs::direct_indexed(self.first_index, self.index_count)
            .with_instance_span(first_instance, instance_count)
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

/// Immutable draw-submission state shared by cache entries and current-frame commands.
#[derive(Clone)]
pub(crate) struct MeshDrawCommandPayload {
    pub(crate) phase: RenderPhase,
    pub(crate) pipeline_kind: MeshPassPipelineKind,
    pipeline_key: PipelineKey,
    pub(crate) pipeline_variant_id: MeshPipelineVariantId,
    half_resolution_transparency: bool,
    pub(crate) material_textures: Option<MeshBindHandle>,
    pub(crate) base_color_texture: Option<MeshBindHandle>,
    pub(crate) material: Option<MeshBindHandle>,
    pub(crate) standard_material: Option<MeshBindHandle>,
    pub(crate) geometry: MeshGeometryHandle,
    pub(crate) previous_velocity_geometry: Option<MeshGeometryHandle>,
    direct_indexed_topology: Option<MeshDirectIndexedTopology>,
}

impl MeshDrawCommandPayload {
    pub(crate) fn is_direct_indexed(&self) -> bool {
        self.direct_indexed_topology.is_some()
    }

    fn material_discriminant(&self) -> u16 {
        self.material
            .as_ref()
            .or(self.standard_material.as_ref())
            .map(|handle| handle.id() as u16)
            .unwrap_or_default()
    }
}

#[derive(Clone)]
enum MeshDrawCommandPayloadStorage {
    Inline(MeshDrawCommandPayload),
    Shared(Arc<MeshDrawCommandPayload>),
}

impl MeshDrawCommandPayloadStorage {
    fn as_ref(&self) -> &MeshDrawCommandPayload {
        match self {
            Self::Inline(payload) => payload,
            Self::Shared(payload) => payload,
        }
    }

    fn make_mut(&mut self) -> &mut MeshDrawCommandPayload {
        match self {
            Self::Inline(payload) => payload,
            Self::Shared(payload) => Arc::make_mut(payload),
        }
    }

    fn into_shared(self) -> Arc<MeshDrawCommandPayload> {
        match self {
            Self::Inline(payload) => Arc::new(payload),
            Self::Shared(payload) => payload,
        }
    }
}

/// Current-frame visibility and GPUScene projection over an immutable submission payload.
#[derive(Clone)]
pub(crate) struct MeshDrawCommand {
    pub(crate) source_entity: EntityId,
    pub(crate) source_draw_index: usize,
    pub(crate) sort_key: u64,
    pub(crate) instance_source: DrawInstanceSource,
    pub(crate) gpu_scene_bind_group: Option<MeshBindHandle>,
    pub(crate) draw_args: MeshDrawArgs,
    payload: MeshDrawCommandPayloadStorage,
}

impl Deref for MeshDrawCommand {
    type Target = MeshDrawCommandPayload;

    fn deref(&self) -> &Self::Target {
        // Replay keeps one command surface while cache hits share immutable WGPU resources.
        self.payload.as_ref()
    }
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
        let direct_indexed_topology = MeshDirectIndexedTopology::from_draw_args(&draw_args);
        Self {
            source_entity: 0,
            source_draw_index: 0,
            sort_key,
            instance_source,
            gpu_scene_bind_group: None,
            draw_args,
            payload: MeshDrawCommandPayloadStorage::Inline(MeshDrawCommandPayload {
                phase,
                pipeline_kind,
                pipeline_key,
                pipeline_variant_id,
                half_resolution_transparency: false,
                material_textures: None,
                base_color_texture: None,
                material: None,
                standard_material: None,
                geometry,
                previous_velocity_geometry: None,
                direct_indexed_topology,
            }),
        }
    }

    pub(crate) fn from_cached_payload(
        payload: Arc<MeshDrawCommandPayload>,
        source_entity: EntityId,
        source_draw_index: usize,
        sort_components: RenderPhaseSortComponents,
        gpu_scene_instance_span: (u32, u32),
        gpu_scene_bind_group: Option<MeshBindHandle>,
    ) -> Self {
        let (first_instance_index, instance_count) = gpu_scene_instance_span;
        debug_assert!(instance_count > 0);
        let sort_key = packed_sort_key_u64(
            payload.phase,
            sort_components,
            payload.pipeline_variant_id.value(),
            payload.material_discriminant(),
        );
        let draw_args = payload
            .direct_indexed_topology
            .expect("cached mesh draw payload must carry direct indexed topology")
            .with_instance_span(first_instance_index, instance_count);

        Self {
            source_entity,
            source_draw_index,
            sort_key,
            instance_source: DrawInstanceSource::GpuSceneInstance {
                first_instance_index,
                instance_count,
            },
            gpu_scene_bind_group,
            draw_args,
            payload: MeshDrawCommandPayloadStorage::Shared(payload),
        }
    }

    #[cfg(test)]
    pub(crate) fn static_payload(&self) -> Arc<MeshDrawCommandPayload> {
        match &self.payload {
            MeshDrawCommandPayloadStorage::Inline(payload) => Arc::new(payload.clone()),
            MeshDrawCommandPayloadStorage::Shared(payload) => payload.clone(),
        }
    }

    pub(crate) fn into_shared_payload(self) -> (Self, Arc<MeshDrawCommandPayload>) {
        let Self {
            source_entity,
            source_draw_index,
            sort_key,
            instance_source,
            gpu_scene_bind_group,
            draw_args,
            payload,
        } = self;
        let payload = payload.into_shared();
        let command = Self {
            source_entity,
            source_draw_index,
            sort_key,
            instance_source,
            gpu_scene_bind_group,
            draw_args,
            payload: MeshDrawCommandPayloadStorage::Shared(payload.clone()),
        };
        (command, payload)
    }

    #[cfg(test)]
    fn payload_is_shared(&self) -> bool {
        matches!(&self.payload, MeshDrawCommandPayloadStorage::Shared(_))
    }

    pub(crate) fn pipeline_key(&self) -> &PipelineKey {
        &self.payload.as_ref().pipeline_key
    }

    pub(crate) fn with_source_draw_index(mut self, source_draw_index: usize) -> Self {
        self.source_draw_index = source_draw_index;
        self
    }

    pub(crate) fn with_source_entity(mut self, source_entity: EntityId) -> Self {
        self.source_entity = source_entity;
        self
    }

    pub(crate) fn with_half_resolution_transparency(mut self, enabled: bool) -> Self {
        self.payload.make_mut().half_resolution_transparency = enabled;
        self
    }

    pub(crate) fn uses_half_resolution_transparency(&self) -> bool {
        self.payload.as_ref().half_resolution_transparency
    }

    pub(crate) fn with_material_textures(mut self, handle: MeshBindHandle) -> Self {
        self.payload.make_mut().material_textures = Some(handle);
        self
    }

    pub(crate) fn with_base_color_texture(mut self, handle: MeshBindHandle) -> Self {
        self.payload.make_mut().base_color_texture = Some(handle);
        self
    }

    pub(crate) fn with_material(mut self, handle: MeshBindHandle) -> Self {
        self.payload.make_mut().material = Some(handle);
        self
    }

    pub(crate) fn with_standard_material(mut self, handle: MeshBindHandle) -> Self {
        self.payload.make_mut().standard_material = Some(handle);
        self
    }

    pub(crate) fn with_gpu_scene_bind_group(mut self, handle: MeshBindHandle) -> Self {
        self.gpu_scene_bind_group = Some(handle);
        self
    }

    pub(crate) fn with_previous_velocity_geometry(mut self, handle: MeshGeometryHandle) -> Self {
        self.payload.make_mut().previous_velocity_geometry = Some(handle);
        self
    }

    pub(crate) fn bind_geometry_buffers<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        let payload = self.payload.as_ref();
        let mesh = payload.geometry.mesh();
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        if payload.pipeline_kind == MeshPassPipelineKind::Velocity {
            let previous_mesh = payload
                .previous_velocity_geometry
                .as_ref()
                .unwrap_or(&payload.geometry)
                .mesh();
            pass.set_vertex_buffer(1, previous_mesh.vertex_buffer.slice(..));
        }
        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    }

    pub(crate) fn geometry_bind_key(&self) -> (u64, u64) {
        let payload = self.payload.as_ref();
        let previous_velocity_geometry_id =
            if payload.pipeline_kind == MeshPassPipelineKind::Velocity {
                payload
                    .previous_velocity_geometry
                    .as_ref()
                    .map(MeshGeometryHandle::id)
                    .unwrap_or_else(|| payload.geometry.id())
            } else {
                0
            };
        (payload.geometry.id(), previous_velocity_geometry_id)
    }

    pub(crate) fn record_indexed_draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        self.draw_args.record_indexed_draw(pass);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderPhase, RenderPhaseSortComponents, packed_sort_key_u64,
    };
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind,
        MeshPipelineVariantId,
    };

    use super::{MeshBindHandle, MeshDrawCommand, next_mesh_bind_handle_id};

    #[test]
    fn mesh_bind_handle_ids_are_nonzero_unique_and_stable_across_clone() {
        let first = next_mesh_bind_handle_id();
        let second = next_mesh_bind_handle_id();
        let handle = MeshBindHandle::test(first);

        assert_ne!(first, 0);
        assert_ne!(second, 0);
        assert_ne!(first, second);
        assert_eq!(handle.id(), handle.clone().id());
    }

    #[test]
    fn velocity_geometry_bind_key_includes_previous_geometry_slot() {
        let velocity = command(MeshPassPipelineKind::Velocity)
            .with_previous_velocity_geometry(MeshGeometryHandle::test(20));
        let base = command(MeshPassPipelineKind::Base)
            .with_previous_velocity_geometry(MeshGeometryHandle::test(20));

        assert_eq!(velocity.geometry_bind_key(), (10, 20));
        assert_eq!(base.geometry_bind_key(), (10, 0));
    }

    #[test]
    fn uncached_indirect_command_keeps_submission_payload_inline() {
        let command = MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            0,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 0,
                instance_count: 1,
            },
            MeshGeometryHandle::test(10),
            MeshDrawArgs::test_indexed_indirect(91, 0),
        );

        assert!(!command.payload_is_shared());
    }

    #[test]
    fn cached_visible_projection_refreshes_sort_and_gpu_scene_span() {
        let sort_components = RenderPhaseSortComponents::new(0.75, 33);
        let cached = command(MeshPassPipelineKind::Base)
            .with_source_entity(1)
            .with_source_draw_index(2)
            .with_material(MeshBindHandle::test(0x1_0003));
        let projected = MeshDrawCommand::from_cached_payload(
            cached.static_payload(),
            9,
            11,
            sort_components,
            (27, 4),
            None,
        );

        assert_eq!(projected.source_entity, 9);
        assert_eq!(projected.source_draw_index, 11);
        assert_eq!(
            projected.sort_key,
            packed_sort_key_u64(
                RenderPhase::PostProcess,
                sort_components,
                MeshPipelineVariantId::new(1).value(),
                3,
            )
        );
        match projected.instance_source {
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index,
                instance_count,
            } => {
                assert_eq!(first_instance_index, 27);
                assert_eq!(instance_count, 4);
            }
        }
        match projected.draw_args {
            MeshDrawArgs::DirectIndexed {
                first_instance,
                instance_count,
                ..
            } => {
                assert_eq!(first_instance, 27);
                assert_eq!(instance_count, 4);
            }
            MeshDrawArgs::IndexedIndirect { .. } => panic!("test command must be direct"),
        }
    }

    #[test]
    fn cached_payload_reprojects_only_current_frame_visible_state() {
        let cached = command(MeshPassPipelineKind::Base)
            .with_material(MeshBindHandle::test(0x1_0003))
            .with_gpu_scene_bind_group(MeshBindHandle::test(41));
        let payload = cached.static_payload();
        let sort_components = RenderPhaseSortComponents::new(0.75, 33);

        let projected = MeshDrawCommand::from_cached_payload(
            payload.clone(),
            9,
            11,
            sort_components,
            (27, 4),
            Some(MeshBindHandle::test(99)),
        );

        assert!(Arc::ptr_eq(&payload, &projected.static_payload()));
        assert_eq!(projected.source_entity, 9);
        assert_eq!(projected.source_draw_index, 11);
        assert_eq!(
            projected.sort_key,
            packed_sort_key_u64(
                RenderPhase::PostProcess,
                sort_components,
                MeshPipelineVariantId::new(1).value(),
                3,
            )
        );
        assert_eq!(
            projected
                .gpu_scene_bind_group
                .as_ref()
                .map(MeshBindHandle::id),
            Some(99)
        );
        assert_eq!(projected.material.as_ref().map(MeshBindHandle::id), Some(3));
        match projected.instance_source {
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index,
                instance_count,
            } => {
                assert_eq!(first_instance_index, 27);
                assert_eq!(instance_count, 4);
            }
        }
        match projected.draw_args {
            MeshDrawArgs::DirectIndexed {
                first_instance,
                instance_count,
                ..
            } => {
                assert_eq!(first_instance, 27);
                assert_eq!(instance_count, 4);
            }
            MeshDrawArgs::IndexedIndirect { .. } => panic!("test command must be direct"),
        }
    }

    fn command(kind: MeshPassPipelineKind) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::PostProcess,
            kind,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            0,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 0,
                instance_count: 1,
            },
            MeshGeometryHandle::test(10),
            MeshDrawArgs::direct_indexed(0, 3),
        )
    }
}

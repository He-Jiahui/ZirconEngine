use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;

use crate::core::framework::render::CubemapFace;
use crate::graphics::scene::scene_renderer::lighting::light_buffer::{
    pack_lighting_extract_with_cookies, PackedGpuLightBuffer,
};
use crate::graphics::scene::scene_renderer::lighting::light_grid_builder::{
    build_light_grid, LightGridCpuOutput, LightGridParams, LightGridViewInfo,
};
use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::EnvironmentCaptureSceneBatch;

const CUBEMAP_FACE_COUNT: usize = 6;

#[derive(Clone, Debug)]
struct EnvironmentCaptureLightGridPayloadRanges {
    params: Range<usize>,
    zbins: Range<usize>,
    tile_masks: Range<usize>,
}

/// One packed direct-light list with six immutable, view-specific light grids.
///
/// The GPU light buffer is scene-owned and written once. Only the culling grid differs by
/// cubemap face; all 18 uploads share one CPU payload so capture does not clone the masks.
pub(in crate::graphics) struct EnvironmentCaptureLightGridPlan {
    packed_lights: PackedGpuLightBuffer,
    grids: Option<[LightGridCpuOutput; CUBEMAP_FACE_COUNT]>,
    payload: Arc<[u8]>,
    ranges: Option<[EnvironmentCaptureLightGridPayloadRanges; CUBEMAP_FACE_COUNT]>,
}

impl EnvironmentCaptureLightGridPlan {
    pub(in crate::graphics) fn from_scene_batch(batch: &mut EnvironmentCaptureSceneBatch) -> Self {
        crate::profile_scope!("render", "environment_capture", "light_grid_plan");
        let lighting = &batch.frame().extract.lighting;
        let packed_lights =
            pack_lighting_extract_with_cookies(lighting, &lighting.advanced_lighting.cookies, true);
        if packed_lights.lights.is_empty() {
            return Self {
                packed_lights,
                grids: None,
                payload: Arc::<[u8]>::from([]),
                ranges: None,
            };
        }
        let grids = CubemapFace::ALL.map(|face| {
            let view = batch.select_face(face);
            let grid_view = LightGridViewInfo::from_camera(
                &view.frame().effective_camera(),
                view.frame().viewport_size,
            );
            build_light_grid(&packed_lights.lights, &grid_view)
        });
        let payload_capacity = grids.iter().fold(0_usize, |bytes, grid| {
            bytes
                .saturating_add(size_of::<LightGridParams>())
                .saturating_add(grid.zbins.len().saturating_mul(size_of::<u32>()))
                .saturating_add(grid.tile_masks.len().saturating_mul(size_of::<u32>()))
        });
        let mut bytes = Vec::with_capacity(payload_capacity);
        let ranges = std::array::from_fn(|face_index| {
            let grid = &grids[face_index];
            let params = append_payload_bytes(&mut bytes, bytemuck::bytes_of(&grid.params));
            let zbins = append_payload_bytes(&mut bytes, bytemuck::cast_slice(&grid.zbins));
            let tile_masks =
                append_payload_bytes(&mut bytes, bytemuck::cast_slice(&grid.tile_masks));
            EnvironmentCaptureLightGridPayloadRanges {
                params,
                zbins,
                tile_masks,
            }
        });

        Self {
            packed_lights,
            grids: Some(grids),
            payload: bytes.into(),
            ranges: Some(ranges),
        }
    }

    pub(in crate::graphics) fn lights(&self) -> &[crate::core::framework::render::GpuLightData] {
        &self.packed_lights.lights
    }

    pub(in crate::graphics) fn has_lights(&self) -> bool {
        !self.packed_lights.lights.is_empty()
    }

    pub(in crate::graphics) fn light_count(&self) -> u32 {
        self.packed_lights.light_count()
    }

    #[cfg(test)]
    fn face_count(&self) -> usize {
        self.grids.as_ref().map_or(0, |grids| grids.len())
    }

    pub(in crate::graphics) fn upload_count(&self) -> usize {
        self.ranges.as_ref().map_or(0, |ranges| ranges.len() * 3)
    }

    pub(in crate::graphics) fn payload_bytes(&self) -> usize {
        self.payload.len()
    }

    #[cfg(test)]
    fn grid(&self, face: CubemapFace) -> &LightGridCpuOutput {
        &self
            .grids
            .as_ref()
            .expect("lit environment capture must own six light grids")[face.index()]
    }

    pub(in crate::graphics) fn prepare_uploads(
        &self,
        workspace: &EnvironmentCaptureLightGridWorkspace,
    ) -> Result<WgpuBufferUploadBatch, GraphicsError> {
        let mut uploads = WgpuBufferUploadBatch::new();
        let ranges = self
            .ranges
            .as_ref()
            .expect("light-grid uploads require a lit capture plan");
        for face in CubemapFace::ALL {
            let ranges = &ranges[face.index()];
            for (binding, range) in [
                (workspace.params_binding(face), ranges.params.clone()),
                (workspace.zbins_binding(face), ranges.zbins.clone()),
                (
                    workspace.tile_masks_binding(face),
                    ranges.tile_masks.clone(),
                ),
            ] {
                uploads.push(
                    WgpuBufferUpload::new(
                        binding.buffer.clone(),
                        binding.offset,
                        Arc::clone(&self.payload),
                        range,
                    )
                    .ok_or(GraphicsError::InvalidBufferUploadRange {
                        label: "environment-capture-light-grid",
                    })?,
                );
            }
        }
        Ok(uploads)
    }
}

fn append_payload_bytes(bytes: &mut Vec<u8>, source: &[u8]) -> Range<usize> {
    let start = bytes.len();
    bytes.extend_from_slice(source);
    start..bytes.len()
}

struct EnvironmentCaptureLightGridSlot {
    params_buffer: wgpu::Buffer,
    zbins_buffer: wgpu::Buffer,
    tile_masks_buffer: wgpu::Buffer,
}

/// Six face-owned GPU light grids for one capture submission.
pub(in crate::graphics) struct EnvironmentCaptureLightGridWorkspace {
    slots: [EnvironmentCaptureLightGridSlot; CUBEMAP_FACE_COUNT],
    allocated_bytes: u64,
}

impl EnvironmentCaptureLightGridWorkspace {
    pub(in crate::graphics) fn new(
        device: &wgpu::Device,
        plan: &EnvironmentCaptureLightGridPlan,
    ) -> Self {
        let mut allocated_bytes = 0_u64;
        let plan_ranges = plan
            .ranges
            .as_ref()
            .expect("light-grid workspace requires a lit capture plan");
        let slots = CubemapFace::ALL.map(|face| {
            let ranges = &plan_ranges[face.index()];
            let params_buffer = create_capture_grid_buffer(
                device,
                face,
                "params",
                ranges.params.len(),
                wgpu::BufferUsages::UNIFORM,
            );
            let zbins_buffer = create_capture_grid_buffer(
                device,
                face,
                "zbins",
                ranges.zbins.len(),
                wgpu::BufferUsages::STORAGE,
            );
            let tile_masks_buffer = create_capture_grid_buffer(
                device,
                face,
                "tile-masks",
                ranges.tile_masks.len(),
                wgpu::BufferUsages::STORAGE,
            );
            allocated_bytes = allocated_bytes
                .saturating_add(ranges.params.len() as u64)
                .saturating_add(ranges.zbins.len() as u64)
                .saturating_add(ranges.tile_masks.len() as u64);
            EnvironmentCaptureLightGridSlot {
                params_buffer,
                zbins_buffer,
                tile_masks_buffer,
            }
        });
        Self {
            slots,
            allocated_bytes,
        }
    }

    pub(in crate::graphics) fn params_binding(&self, face: CubemapFace) -> wgpu::BufferBinding<'_> {
        capture_buffer_binding(&self.slots[face.index()].params_buffer)
    }

    pub(in crate::graphics) fn zbins_binding(&self, face: CubemapFace) -> wgpu::BufferBinding<'_> {
        capture_buffer_binding(&self.slots[face.index()].zbins_buffer)
    }

    pub(in crate::graphics) fn tile_masks_binding(
        &self,
        face: CubemapFace,
    ) -> wgpu::BufferBinding<'_> {
        capture_buffer_binding(&self.slots[face.index()].tile_masks_buffer)
    }

    pub(in crate::graphics) fn allocated_bytes(&self) -> u64 {
        self.allocated_bytes
    }
}

fn create_capture_grid_buffer(
    device: &wgpu::Device,
    face: CubemapFace,
    kind: &str,
    size: usize,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let label = format!(
        "zircon-environment-capture-light-grid-face-{}-{kind}",
        face.index()
    );
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&label),
        size: size as u64,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn capture_buffer_binding(buffer: &wgpu::Buffer) -> wgpu::BufferBinding<'_> {
    wgpu::BufferBinding {
        buffer,
        offset: 0,
        size: None,
    }
}

/// Six immutable per-face scene constants packed into one CPU allocation.
///
/// The source ranges are uploaded to separate GPU buffers. This is required
/// when all faces share one encoder: overwriting a single uniform buffer six
/// times before submission would make every pass observe the final face.
pub(in crate::graphics) struct EnvironmentCaptureSceneUniformPlan {
    uniforms: [SceneUniform; CUBEMAP_FACE_COUNT],
    payload: Arc<[u8]>,
}

impl EnvironmentCaptureSceneUniformPlan {
    pub(in crate::graphics) fn from_scene_batch(
        batch: &mut EnvironmentCaptureSceneBatch,
        global_material_mip_bias: f32,
    ) -> Self {
        Self::from_scene_batch_with_environment(batch, global_material_mip_bias, None)
    }

    pub(in crate::graphics) fn from_scene_batch_with_realtime_ibl(
        batch: &mut EnvironmentCaptureSceneBatch,
        global_material_mip_bias: f32,
        source_face_size: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
    ) -> Self {
        Self::from_scene_batch_with_environment(
            batch,
            global_material_mip_bias,
            Some([source_face_size, pmrem_face_size, pmrem_mip_count]),
        )
    }

    fn from_scene_batch_with_environment(
        batch: &mut EnvironmentCaptureSceneBatch,
        global_material_mip_bias: f32,
        realtime_ibl_dimensions: Option<[u32; 3]>,
    ) -> Self {
        let uniforms = CubemapFace::ALL.map(|face| {
            let view = batch.select_face(face);
            let mut uniform = SceneUniform::from_frame(view.frame());
            uniform.set_global_material_mip_bias(global_material_mip_bias);
            uniform.use_environment_capture_surface_policy();
            if let Some([source_face_size, pmrem_face_size, pmrem_mip_count]) =
                realtime_ibl_dimensions
            {
                uniform.use_realtime_ibl(source_face_size, pmrem_face_size, pmrem_mip_count);
            }
            uniform
        });
        let payload: Arc<[u8]> = Arc::from(bytemuck::cast_slice(&uniforms));

        Self { uniforms, payload }
    }

    pub(in crate::graphics) fn uniform(&self, face: CubemapFace) -> &SceneUniform {
        &self.uniforms[face.index()]
    }

    pub(in crate::graphics) fn payload_bytes(&self) -> usize {
        self.payload.len()
    }

    pub(in crate::graphics) fn prepare_uploads(
        &self,
        workspace: &EnvironmentCaptureSceneUniformWorkspace,
    ) -> Result<WgpuBufferUploadBatch, GraphicsError> {
        let mut uploads = WgpuBufferUploadBatch::new();
        for face in CubemapFace::ALL {
            uploads.push(
                WgpuBufferUpload::new(
                    workspace.uniform_buffer(face).clone(),
                    0,
                    Arc::clone(&self.payload),
                    face_payload_range(face),
                )
                .ok_or(GraphicsError::InvalidBufferUploadRange {
                    label: "environment-capture-scene-uniform",
                })?,
            );
        }
        Ok(uploads)
    }
}

struct EnvironmentCaptureSceneUniformSlot {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

/// Persistent six-face bindings for one active environment capture job.
///
/// `create_bind_group` projects the renderer's current environment resources
/// into the unchanged scene bind-group layout. Only binding zero differs per
/// face, so this owner does not require dynamic offsets or new permutations.
pub(in crate::graphics) struct EnvironmentCaptureSceneUniformWorkspace {
    slots: [EnvironmentCaptureSceneUniformSlot; CUBEMAP_FACE_COUNT],
}

impl EnvironmentCaptureSceneUniformWorkspace {
    pub(in crate::graphics) fn new(
        device: &wgpu::Device,
        mut create_bind_group: impl FnMut(&wgpu::Buffer) -> wgpu::BindGroup,
    ) -> Self {
        let slots = CubemapFace::ALL.map(|face| {
            let label = format!(
                "zircon-environment-capture-scene-uniform-face-{}",
                face.index()
            );
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&label),
                size: size_of::<SceneUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind_group = create_bind_group(&uniform_buffer);
            EnvironmentCaptureSceneUniformSlot {
                uniform_buffer,
                bind_group,
            }
        });
        Self { slots }
    }

    pub(in crate::graphics) fn uniform_buffer(&self, face: CubemapFace) -> &wgpu::Buffer {
        &self.slots[face.index()].uniform_buffer
    }

    pub(in crate::graphics) fn bind_group(&self, face: CubemapFace) -> &wgpu::BindGroup {
        &self.slots[face.index()].bind_group
    }

    pub(in crate::graphics) fn uniform_buffer_bytes(&self) -> u64 {
        (size_of::<SceneUniform>() * CUBEMAP_FACE_COUNT) as u64
    }
}

fn face_payload_range(face: CubemapFace) -> Range<usize> {
    let start = face.index() * size_of::<SceneUniform>();
    start..start + size_of::<SceneUniform>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        EnvironmentExtract, PreviewEnvironmentExtract, RenderEnvironmentCaptureRequest,
        RenderOverlayExtract, RenderSceneGeometryExtract, SceneViewportRenderPacket,
        ViewportCameraSnapshot,
    };
    use crate::core::math::Vec4;

    #[test]
    fn one_packed_payload_retains_six_distinct_face_uniforms() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [4.0, 5.0, 6.0], 7)
            .unwrap()
            .with_face_size(256)
            .unwrap();
        let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

        let plan = EnvironmentCaptureSceneUniformPlan::from_scene_batch(&mut batch, 0.75);

        assert_eq!(
            plan.payload_bytes(),
            CUBEMAP_FACE_COUNT * size_of::<SceneUniform>()
        );
        assert_eq!(size_of::<SceneUniform>(), 496);
        assert_eq!(plan.payload_bytes(), 2_976);
        assert_eq!(batch.selected_face(), Some(CubemapFace::NegativeZ));
        for face in CubemapFace::ALL {
            assert_eq!(
                plan.uniform(face).camera_world_position,
                [4.0, 5.0, 6.0, 0.75]
            );
            assert_eq!(plan.uniform(face).sky_sun_params[3], 1.0);
            assert_eq!(face_payload_range(face).len(), size_of::<SceneUniform>());
        }
        assert_ne!(
            plan.uniform(CubemapFace::PositiveX).view_proj_unjittered,
            plan.uniform(CubemapFace::NegativeX).view_proj_unjittered
        );
    }

    #[test]
    fn capture_surface_policy_is_shared_by_forward_templates_and_fallback() {
        const SURFACE_TYPES: &str = include_str!("../../../shader/wgsl/zr_surface_types.wgsl");
        const FORWARD: &str = include_str!("../../../shader/wgsl/zr_template_forward.wgsl");
        const ENVIRONMENT_ONLY: &str =
            include_str!("../../../shader/wgsl/zr_template_forward_environment_only_pbr.wgsl");
        const FALLBACK: &str = include_str!("../mesh/shaders/fallback_mesh.wgsl");

        assert!(SURFACE_TYPES.contains("fn zr_surface_apply_environment_capture_policy("));
        assert!(SURFACE_TYPES.contains("resolved.clearcoat_roughness = 1.0;"));
        for source in [FORWARD, ENVIRONMENT_ONLY] {
            let alpha_clip = source.find("zr_apply_alpha_clip(surface);").unwrap();
            let capture_policy = source
                .find("surface = zr_surface_apply_environment_capture_policy(surface);")
                .unwrap();
            assert!(alpha_clip < capture_policy);
        }
        assert!(FALLBACK.contains(
            "material.roughness = zr_environment_capture_roughness(material.roughness);"
        ));
    }

    #[test]
    fn packed_face_ranges_are_contiguous_and_non_overlapping() {
        let mut previous_end = 0;
        for face in CubemapFace::ALL {
            let range = face_payload_range(face);
            assert_eq!(range.start, previous_end);
            previous_end = range.end;
        }
        assert_eq!(previous_end, CUBEMAP_FACE_COUNT * size_of::<SceneUniform>());
    }

    #[test]
    fn realtime_ibl_override_is_packed_into_every_face_without_a_second_payload() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1).unwrap();
        let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

        let plan = EnvironmentCaptureSceneUniformPlan::from_scene_batch_with_realtime_ibl(
            &mut batch, 0.0, 128, 64, 7,
        );

        assert_eq!(plan.payload_bytes(), 2_976);
        for face in CubemapFace::ALL {
            assert_eq!(
                plan.uniform(face).environment_sample_params,
                [4.0, 128.0, 64.0, 7.0]
            );
        }
    }

    #[test]
    fn capture_light_grid_packs_lights_once_and_owns_six_face_views() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_face_size(128)
            .unwrap();
        let mut scene = test_scene();
        scene.scene.directional_lights.push(
            crate::core::framework::render::RenderDirectionalLightSnapshot {
                node_id: 7,
                light_id: 7,
                layer_mask: crate::core::framework::render::RenderLayerSet::default(),
                direction: crate::core::math::Vec3::NEG_Y,
                color: crate::core::math::Vec3::ONE,
                intensity: 2.0,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: None,
            },
        );
        let mut batch = EnvironmentCaptureSceneBatch::new(scene, request);

        let plan = EnvironmentCaptureLightGridPlan::from_scene_batch(&mut batch);

        assert_eq!(plan.light_count(), 1);
        assert_eq!(plan.face_count(), CUBEMAP_FACE_COUNT);
        assert_eq!(plan.upload_count(), CUBEMAP_FACE_COUNT * 3);
        assert_eq!(plan.payload_bytes(), 105_192);
        assert_ne!(
            plan.grid(CubemapFace::PositiveX).params.world_to_view,
            plan.grid(CubemapFace::NegativeX).params.world_to_view
        );
    }

    #[test]
    fn capture_light_grid_empty_path_builds_no_face_payload() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1).unwrap();
        let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

        let plan = EnvironmentCaptureLightGridPlan::from_scene_batch(&mut batch);

        assert_eq!(plan.light_count(), 0);
        assert_eq!(plan.face_count(), 0);
        assert_eq!(plan.upload_count(), 0);
        assert_eq!(plan.payload_bytes(), 0);
        assert_eq!(batch.selected_face(), None);
    }

    fn test_scene() -> SceneViewportRenderPacket {
        let environment = EnvironmentExtract::default();
        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract::from_environment(&environment, false, Vec4::ZERO),
            environment,
            virtual_geometry_debug: None,
        }
    }
}

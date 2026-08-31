use crate::core::framework::render::{
    source_cubemap_mip_count, CubemapFace, RenderEnvironmentCaptureRequest,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};

use super::super::core::{DEPTH_FORMAT, SCENE_COLOR_HDR_FORMAT};

const CUBEMAP_FACE_COUNT: u32 = 6;
const RGBA16_FLOAT_TEXEL_BYTES: u64 = 8;
const DEPTH32_FLOAT_TEXEL_BYTES: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureGpuTargetPlan {
    face_size: u32,
    source_mip_count: u32,
    color_texture_bytes: u64,
    pmrem_texture_bytes: u64,
    sh9_buffer_bytes: u64,
    depth_texture_bytes: u64,
}

impl EnvironmentCaptureGpuTargetPlan {
    pub(in crate::graphics) fn from_request(request: &RenderEnvironmentCaptureRequest) -> Self {
        let face_size = request.face_size();
        let source_mip_count = source_cubemap_mip_count(face_size);
        let color_texture_bytes = cubemap_texel_count(face_size, source_mip_count)
            .saturating_mul(RGBA16_FLOAT_TEXEL_BYTES);
        let pmrem_texture_bytes = cubemap_texel_count(
            SOURCE_CUBEMAP_PMREM_FACE_SIZE,
            SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        )
        .saturating_mul(RGBA16_FLOAT_TEXEL_BYTES);
        let sh9_buffer_bytes = IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64;
        let depth_texture_bytes = u64::from(face_size)
            .saturating_mul(u64::from(face_size))
            .saturating_mul(DEPTH32_FLOAT_TEXEL_BYTES);
        Self {
            face_size,
            source_mip_count,
            color_texture_bytes,
            pmrem_texture_bytes,
            sh9_buffer_bytes,
            depth_texture_bytes,
        }
    }

    pub(in crate::graphics) fn face_size(self) -> u32 {
        self.face_size
    }

    pub(in crate::graphics) fn source_mip_count(self) -> u32 {
        self.source_mip_count
    }

    pub(in crate::graphics) fn color_texture_bytes(self) -> u64 {
        self.color_texture_bytes
    }

    pub(in crate::graphics) fn depth_texture_bytes(self) -> u64 {
        self.depth_texture_bytes
    }

    pub(in crate::graphics) fn pmrem_texture_bytes(self) -> u64 {
        self.pmrem_texture_bytes
    }

    pub(in crate::graphics) fn sh9_buffer_bytes(self) -> u64 {
        self.sh9_buffer_bytes
    }

    pub(in crate::graphics) fn total_texture_bytes(self) -> u64 {
        self.color_texture_bytes
            .saturating_add(self.pmrem_texture_bytes)
            .saturating_add(self.depth_texture_bytes)
    }

    pub(in crate::graphics) fn total_gpu_bytes(self) -> u64 {
        self.total_texture_bytes()
            .saturating_add(self.sh9_buffer_bytes)
    }
}

pub(in crate::graphics) struct EnvironmentCaptureGpuTarget {
    plan: EnvironmentCaptureGpuTargetPlan,
    _color_texture: wgpu::Texture,
    sampled_cube: wgpu::TextureView,
    sampled_mips: Vec<wgpu::TextureView>,
    color_faces: [wgpu::TextureView; CUBEMAP_FACE_COUNT as usize],
    storage_mips: Vec<wgpu::TextureView>,
    _pmrem_texture: wgpu::Texture,
    pmrem_sampled_cube: wgpu::TextureView,
    pmrem_storage_mips: Vec<wgpu::TextureView>,
    sh9_buffer: wgpu::Buffer,
    _depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

/// Filtered capture resources that remain useful after source/depth scratch retires.
pub(in crate::graphics) struct EnvironmentCaptureGpuOutput {
    plan: EnvironmentCaptureGpuTargetPlan,
    _pmrem_texture: wgpu::Texture,
    pmrem_sampled_cube: wgpu::TextureView,
    sh9_buffer: wgpu::Buffer,
}

impl EnvironmentCaptureGpuTarget {
    pub(in crate::graphics) fn new(
        device: &wgpu::Device,
        request: &RenderEnvironmentCaptureRequest,
    ) -> Self {
        let plan = EnvironmentCaptureGpuTargetPlan::from_request(request);
        let color_label = format!("zircon-environment-capture-{}-source", request.capture_id());
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&color_label),
            size: wgpu::Extent3d {
                width: plan.face_size,
                height: plan.face_size,
                depth_or_array_layers: CUBEMAP_FACE_COUNT,
            },
            mip_level_count: plan.source_mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SCENE_COLOR_HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let sampled_cube =
            color_texture.create_view(&sampled_cube_view_descriptor(plan.source_mip_count));
        let sampled_mips = (0..plan.source_mip_count)
            .map(|mip_level| color_texture.create_view(&sampled_mip_view_descriptor(mip_level)))
            .collect();
        let color_faces = std::array::from_fn(|face_index| {
            color_texture.create_view(&color_face_view_descriptor(face_index as u32))
        });
        let storage_mips = (0..plan.source_mip_count)
            .map(|mip_level| color_texture.create_view(&storage_mip_view_descriptor(mip_level)))
            .collect();

        let pmrem_label = format!("zircon-environment-capture-{}-pmrem", request.capture_id());
        let pmrem_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&pmrem_label),
            size: wgpu::Extent3d {
                width: SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                height: SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                depth_or_array_layers: CUBEMAP_FACE_COUNT,
            },
            mip_level_count: SOURCE_CUBEMAP_PMREM_MIP_COUNT,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SCENE_COLOR_HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let pmrem_sampled_cube = pmrem_texture.create_view(&sampled_cube_view_descriptor(
            SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        ));
        let pmrem_storage_mips = (0..SOURCE_CUBEMAP_PMREM_MIP_COUNT)
            .map(|mip_level| pmrem_texture.create_view(&storage_mip_view_descriptor(mip_level)))
            .collect();
        let sh9_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-environment-capture-sh9"),
            size: plan.sh9_buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // All six faces record sequentially, so one depth attachment is sufficient.
        let depth_label = format!("zircon-environment-capture-{}-depth", request.capture_id());
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&depth_label),
            size: wgpu::Extent3d {
                width: plan.face_size,
                height: plan.face_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-environment-capture-depth-view"),
            format: Some(DEPTH_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        Self {
            plan,
            _color_texture: color_texture,
            sampled_cube,
            sampled_mips,
            color_faces,
            storage_mips,
            _pmrem_texture: pmrem_texture,
            pmrem_sampled_cube,
            pmrem_storage_mips,
            sh9_buffer,
            _depth_texture: depth_texture,
            depth_view,
        }
    }

    pub(in crate::graphics) fn plan(&self) -> EnvironmentCaptureGpuTargetPlan {
        self.plan
    }

    pub(in crate::graphics) fn sampled_cube(&self) -> &wgpu::TextureView {
        &self.sampled_cube
    }

    pub(in crate::graphics) fn source_texture(&self) -> &wgpu::Texture {
        &self._color_texture
    }

    pub(in crate::graphics) fn sampled_mips(&self) -> &[wgpu::TextureView] {
        &self.sampled_mips
    }

    pub(in crate::graphics) fn storage_mips(&self) -> &[wgpu::TextureView] {
        &self.storage_mips
    }

    pub(in crate::graphics) fn color_face(&self, face: CubemapFace) -> &wgpu::TextureView {
        &self.color_faces[face.index()]
    }

    pub(in crate::graphics) fn storage_mip(&self, mip_level: u32) -> Option<&wgpu::TextureView> {
        self.storage_mips.get(mip_level as usize)
    }

    pub(in crate::graphics) fn pmrem_sampled_cube(&self) -> &wgpu::TextureView {
        &self.pmrem_sampled_cube
    }

    pub(in crate::graphics) fn pmrem_texture(&self) -> &wgpu::Texture {
        &self._pmrem_texture
    }

    pub(in crate::graphics) fn pmrem_storage_mip(
        &self,
        mip_level: u32,
    ) -> Option<&wgpu::TextureView> {
        self.pmrem_storage_mips.get(mip_level as usize)
    }

    pub(in crate::graphics) fn sh9_buffer(&self) -> &wgpu::Buffer {
        &self.sh9_buffer
    }

    pub(in crate::graphics) fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    pub(in crate::graphics) fn into_filtered_output(self) -> EnvironmentCaptureGpuOutput {
        let Self {
            plan,
            _pmrem_texture,
            pmrem_sampled_cube,
            sh9_buffer,
            ..
        } = self;
        EnvironmentCaptureGpuOutput {
            plan,
            _pmrem_texture,
            pmrem_sampled_cube,
            sh9_buffer,
        }
    }
}

impl EnvironmentCaptureGpuOutput {
    pub(in crate::graphics) fn gpu_bytes(&self) -> u64 {
        self.plan
            .pmrem_texture_bytes()
            .saturating_add(self.plan.sh9_buffer_bytes())
    }

    pub(in crate::graphics) fn pmrem_sampled_cube(&self) -> &wgpu::TextureView {
        &self.pmrem_sampled_cube
    }

    pub(in crate::graphics) fn sh9_buffer(&self) -> &wgpu::Buffer {
        &self.sh9_buffer
    }
}

fn cubemap_texel_count(face_size: u32, mip_count: u32) -> u64 {
    let mut texels_per_face = 0_u64;
    for mip_level in 0..mip_count {
        let mip_size = (face_size >> mip_level).max(1);
        texels_per_face =
            texels_per_face.saturating_add(u64::from(mip_size).saturating_mul(u64::from(mip_size)));
    }
    texels_per_face.saturating_mul(u64::from(CUBEMAP_FACE_COUNT))
}

fn sampled_cube_view_descriptor(mip_count: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-environment-capture-sampled-cube"),
        format: Some(SCENE_COLOR_HDR_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(mip_count),
        base_array_layer: 0,
        array_layer_count: Some(CUBEMAP_FACE_COUNT),
    }
}

fn sampled_mip_view_descriptor(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-environment-capture-sampled-mip"),
        format: Some(SCENE_COLOR_HDR_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(CUBEMAP_FACE_COUNT),
    }
}

fn color_face_view_descriptor(face_index: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-environment-capture-color-face"),
        format: Some(SCENE_COLOR_HDR_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::D2),
        usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: face_index,
        array_layer_count: Some(1),
    }
}

fn storage_mip_view_descriptor(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-environment-capture-storage-mip"),
        format: Some(SCENE_COLOR_HDR_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        usage: Some(wgpu::TextureUsages::STORAGE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(CUBEMAP_FACE_COUNT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_plan_counts_full_rgba16f_cube_chain_and_one_depth_face() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_face_size(128)
            .unwrap();
        let plan = EnvironmentCaptureGpuTargetPlan::from_request(&request);

        assert_eq!(plan.face_size(), 128);
        assert_eq!(plan.source_mip_count(), 8);
        assert_eq!(plan.color_texture_bytes(), 1_048_560);
        assert_eq!(plan.depth_texture_bytes(), 65_536);
        assert_eq!(plan.pmrem_texture_bytes(), 1_048_560);
        assert_eq!(
            plan.sh9_buffer_bytes(),
            IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64
        );
        assert_eq!(plan.sh9_buffer_bytes(), 144);
        assert_eq!(plan.total_texture_bytes(), 2_162_656);
        assert_eq!(plan.total_gpu_bytes(), 2_162_800);
    }

    #[test]
    fn maximum_request_exposes_bounded_admission_cost_before_allocation() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_face_size(1024)
            .unwrap();
        let plan = EnvironmentCaptureGpuTargetPlan::from_request(&request);

        assert_eq!(plan.source_mip_count(), 11);
        assert_eq!(plan.color_texture_bytes(), 67_108_848);
        assert_eq!(plan.depth_texture_bytes(), 4_194_304);
        assert_eq!(plan.pmrem_texture_bytes(), 1_048_560);
        assert_eq!(
            plan.sh9_buffer_bytes(),
            IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64
        );
        assert_eq!(plan.sh9_buffer_bytes(), 144);
        assert_eq!(plan.total_texture_bytes(), 72_351_712);
        assert_eq!(plan.total_gpu_bytes(), 72_351_856);
    }

    #[test]
    fn resident_output_budget_excludes_source_and_depth_scratch() {
        let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
            .unwrap()
            .with_face_size(1024)
            .unwrap();
        let plan = EnvironmentCaptureGpuTargetPlan::from_request(&request);

        assert_eq!(
            plan.pmrem_texture_bytes() + plan.sh9_buffer_bytes(),
            1_048_704
        );
        assert!(plan.total_gpu_bytes() > 72_000_000);
    }
}

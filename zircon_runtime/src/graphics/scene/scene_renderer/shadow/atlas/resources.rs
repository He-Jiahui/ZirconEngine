use bytemuck::{bytes_of, cast_slice};
use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::core::DEPTH_FORMAT;
use crate::graphics::scene::scene_renderer::shadow::slot::{
    GpuShadowGlobals, GpuShadowSlot, GPU_SHADOW_SLOT_STRIDE,
};

use super::ShadowAtlasConfig;

pub(crate) const SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY: u32 = 256;
pub(crate) const SHADOW_ATLAS_FALLBACK_SIZE: u32 = 2048;
pub(crate) const SHADOW_ATLAS_COMPARE_FUNCTION: wgpu::CompareFunction =
    wgpu::CompareFunction::LessEqual;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShadowAtlasResourceConfig {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) slot_capacity: u32,
}

impl ShadowAtlasResourceConfig {
    pub(crate) const fn new(width: u32, height: u32, slot_capacity: u32) -> Self {
        Self {
            width,
            height,
            slot_capacity,
        }
    }

    pub(crate) fn from_atlas_config(atlas_config: ShadowAtlasConfig, slot_capacity: u32) -> Self {
        Self::new(
            atlas_config.width,
            atlas_config.height,
            slot_capacity.max(1),
        )
    }

    pub(crate) fn normalized(self) -> Self {
        Self {
            width: self.width.max(1),
            height: self.height.max(1),
            slot_capacity: self.slot_capacity.max(1),
        }
    }

    pub(crate) fn with_max_texture_dimension(self, max_texture_dimension_2d: u32) -> Self {
        let mut config = self.normalized();
        let max_texture_dimension_2d = max_texture_dimension_2d.max(1);
        if config.width > max_texture_dimension_2d || config.height > max_texture_dimension_2d {
            let fallback_size = SHADOW_ATLAS_FALLBACK_SIZE
                .min(max_texture_dimension_2d)
                .max(1);
            config.width = config.width.min(fallback_size);
            config.height = config.height.min(fallback_size);
        }
        config
    }

    pub(crate) fn slot_buffer_size_bytes(self) -> u64 {
        self.normalized().slot_capacity as u64 * GPU_SHADOW_SLOT_STRIDE as u64
    }

    fn extent(self) -> wgpu::Extent3d {
        let config = self.normalized();
        wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        }
    }
}

impl Default for ShadowAtlasResourceConfig {
    fn default() -> Self {
        Self::from_atlas_config(
            ShadowAtlasConfig::default(),
            SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShadowAtlasUploadReport {
    pub(crate) uploaded_slot_count: u32,
    pub(crate) cleared_stale_slot_count: u32,
    pub(crate) slot_capacity: u32,
}

pub(crate) struct ShadowAtlasResources {
    config: ShadowAtlasResourceConfig,
    atlas_texture: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    compare_sampler: wgpu::Sampler,
    slot_buffer: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,
    last_uploaded_slot_count: u32,
}

impl ShadowAtlasResources {
    pub(crate) fn new(device: &wgpu::Device, config: ShadowAtlasResourceConfig) -> Self {
        let config = config.with_max_texture_dimension(device.limits().max_texture_dimension_2d);
        let atlas_texture = create_atlas_texture(device, config);
        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let compare_sampler = create_compare_sampler(device);
        let slot_buffer = create_slot_buffer(device, config);
        let globals_buffer = create_globals_buffer(device, config);

        Self {
            config,
            atlas_texture,
            atlas_view,
            compare_sampler,
            slot_buffer,
            globals_buffer,
            last_uploaded_slot_count: 0,
        }
    }

    pub(crate) fn config(&self) -> ShadowAtlasResourceConfig {
        self.config
    }

    pub(crate) fn atlas_texture(&self) -> &wgpu::Texture {
        &self.atlas_texture
    }

    pub(crate) fn atlas_view(&self) -> &wgpu::TextureView {
        &self.atlas_view
    }

    pub(crate) fn compare_sampler(&self) -> &wgpu::Sampler {
        &self.compare_sampler
    }

    pub(crate) fn slot_buffer(&self) -> &wgpu::Buffer {
        &self.slot_buffer
    }

    pub(crate) fn globals_buffer(&self) -> &wgpu::Buffer {
        &self.globals_buffer
    }

    pub(crate) fn upload_frame(
        &mut self,
        queue: &wgpu::Queue,
        slots: &[GpuShadowSlot],
        globals: GpuShadowGlobals,
    ) -> Result<ShadowAtlasUploadReport, String> {
        if slots.len() > self.config.slot_capacity as usize {
            return Err(format!(
                "shadow atlas slot upload requested {} slots but capacity is {}",
                slots.len(),
                self.config.slot_capacity
            ));
        }

        if !slots.is_empty() {
            queue.write_buffer(&self.slot_buffer, 0, cast_slice(slots));
        }
        queue.write_buffer(&self.globals_buffer, 0, bytes_of(&globals));

        let uploaded_slot_count = slots.len() as u32;
        let stale_slot_count = self
            .last_uploaded_slot_count
            .saturating_sub(uploaded_slot_count);
        if stale_slot_count > 0 {
            let disabled_slots = vec![GpuShadowSlot::disabled(); stale_slot_count as usize];
            queue.write_buffer(
                &self.slot_buffer,
                uploaded_slot_count as u64 * GPU_SHADOW_SLOT_STRIDE as u64,
                cast_slice(&disabled_slots),
            );
        }
        self.last_uploaded_slot_count = uploaded_slot_count;

        Ok(ShadowAtlasUploadReport {
            uploaded_slot_count,
            cleared_stale_slot_count: stale_slot_count,
            slot_capacity: self.config.slot_capacity,
        })
    }

    pub(crate) fn upload_disabled(&mut self, queue: &wgpu::Queue) -> ShadowAtlasUploadReport {
        self.upload_frame(
            queue,
            &[],
            GpuShadowGlobals::disabled(self.config.width, self.config.height),
        )
        .expect("empty shadow atlas upload must fit the configured slot capacity")
    }
}

fn create_atlas_texture(device: &wgpu::Device, config: ShadowAtlasResourceConfig) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-shadow-atlas"),
        size: config.extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_compare_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-shadow-atlas-compare-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        compare: Some(SHADOW_ATLAS_COMPARE_FUNCTION),
        ..Default::default()
    })
}

fn create_slot_buffer(device: &wgpu::Device, config: ShadowAtlasResourceConfig) -> wgpu::Buffer {
    let disabled_slots = vec![GpuShadowSlot::disabled(); config.slot_capacity as usize];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-shadow-atlas-slots"),
        contents: cast_slice(&disabled_slots),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

fn create_globals_buffer(device: &wgpu::Device, config: ShadowAtlasResourceConfig) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-shadow-atlas-globals"),
        contents: bytes_of(&GpuShadowGlobals::disabled(config.width, config.height)),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_shadow_atlas_resource_config_normalizes_zero_values() {
        let config = ShadowAtlasResourceConfig::new(0, 0, 0).normalized();

        assert_eq!(config.width, 1);
        assert_eq!(config.height, 1);
        assert_eq!(config.slot_capacity, 1);
        assert_eq!(
            config.slot_buffer_size_bytes(),
            GPU_SHADOW_SLOT_STRIDE as u64
        );
    }

    #[test]
    fn render_shadow_atlas_resource_config_uses_plan_05_defaults() {
        let config = ShadowAtlasResourceConfig::default();

        assert_eq!(config.width, 4096);
        assert_eq!(config.height, 4096);
        assert_eq!(config.slot_capacity, SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY);
        assert_eq!(
            config.slot_buffer_size_bytes(),
            SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY as u64 * GPU_SHADOW_SLOT_STRIDE as u64
        );
    }

    #[test]
    fn render_shadow_atlas_compare_function_matches_forward_depth_contract() {
        assert_eq!(
            SHADOW_ATLAS_COMPARE_FUNCTION,
            wgpu::CompareFunction::LessEqual
        );
    }

    #[test]
    fn render_shadow_atlas_resource_config_downgrades_to_capability_limit() {
        let config =
            ShadowAtlasResourceConfig::new(4096, 4096, 16).with_max_texture_dimension(3072);

        assert_eq!(config.width, SHADOW_ATLAS_FALLBACK_SIZE);
        assert_eq!(config.height, SHADOW_ATLAS_FALLBACK_SIZE);
        assert_eq!(config.slot_capacity, 16);
    }

    #[test]
    fn render_shadow_atlas_upload_report_describes_cleared_tail() {
        let report = ShadowAtlasUploadReport {
            uploaded_slot_count: 2,
            cleared_stale_slot_count: 3,
            slot_capacity: 8,
        };

        assert_eq!(report.uploaded_slot_count, 2);
        assert_eq!(report.cleared_stale_slot_count, 3);
        assert_eq!(report.slot_capacity, 8);
    }
}

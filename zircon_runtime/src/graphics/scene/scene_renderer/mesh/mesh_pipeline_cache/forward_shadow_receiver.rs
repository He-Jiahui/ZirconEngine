use bytemuck::bytes_of;
use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_apply_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::irradiance_volume_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::light_cookie_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::advanced_lighting::transmission::transmission_scene_color_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::environment::lightmap_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::environment::reflection_probe_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::lighting::light_grid_builder::{
    LIGHT_GRID_EMPTY_ZBIN_HEADER, LightGridParams,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::{
    SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING, SHADOW_ATLAS_SLOT_BUFFER_BINDING,
    SHADOW_GLOBALS_BINDING, ShadowAtlasResources, shadow_atlas_bind_group_layout_entries,
};
use crate::graphics::scene::scene_renderer::shadow::slot::{GpuShadowGlobals, GpuShadowSlot};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};

use super::MeshPipelineCache;

const FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES: wgpu::ShaderStages =
    wgpu::ShaderStages::FRAGMENT;
const LIGHT_GRID_PARAMS_BINDING: u32 = 20;
const LIGHT_ZBINS_BINDING: u32 = 21;
const LIGHT_TILE_MASKS_BINDING: u32 = 22;

#[derive(Clone, Copy)]
enum ForwardReflectionProviderPolicy {
    Scene,
    EnvironmentCaptureDisabled,
}

impl MeshPipelineCache {
    pub(crate) fn begin_forward_receiver_binding_profile_frame(&mut self) {
        self.standard_forward_receiver_bind_group_create_count = 0;
        self.full_forward_receiver_bind_group_create_count = 0;
    }

    pub(crate) fn emit_forward_receiver_binding_profile_frame(&self) {
        crate::profile_counter!(
            "render",
            "forward_receiver_standard_bind_group_create_count",
            self.standard_forward_receiver_bind_group_create_count,
        );
        crate::profile_counter!(
            "render",
            "forward_receiver_full_bind_group_create_count",
            self.full_forward_receiver_bind_group_create_count,
        );
    }

    pub(in crate::graphics::scene::scene_renderer) fn create_forward_shadow_receiver_bind_group(
        &mut self,
        device: &wgpu::Device,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
    ) -> wgpu::BindGroup {
        crate::profile_scope!("render", "forward_receiver", "standard_bind_group_create");
        let bind_group = self.create_forward_receiver_bind_group_with_volumetric(
            device,
            shadow_atlas_resources,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            &self.forward_volumetric_disabled_params_buffer,
            None,
            None,
            ForwardReflectionProviderPolicy::Scene,
        );
        self.record_standard_forward_receiver_bind_group_creation();
        bind_group
    }

    /// Creates the lightweight receiver used while baking a reflection probe.
    ///
    /// Capture has no viewport volumetrics or transmission input and must not sample existing
    /// local reflection providers. Keeping this as a receiver policy avoids both feedback and a
    /// transient volumetric-parameter allocation for every cubemap face.
    pub(in crate::graphics::scene::scene_renderer) fn create_environment_capture_forward_receiver_bind_group(
        &mut self,
        device: &wgpu::Device,
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
    ) -> wgpu::BindGroup {
        crate::profile_scope!(
            "render",
            "forward_receiver",
            "environment_capture_bind_group_create"
        );
        let bind_group = self.create_forward_receiver_bind_group_with_volumetric(
            device,
            None,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            &self.forward_volumetric_disabled_params_buffer,
            None,
            None,
            ForwardReflectionProviderPolicy::EnvironmentCaptureDisabled,
        );
        self.record_standard_forward_receiver_bind_group_creation();
        bind_group
    }

    fn record_standard_forward_receiver_bind_group_creation(&mut self) {
        self.standard_forward_receiver_bind_group_create_count = self
            .standard_forward_receiver_bind_group_create_count
            .saturating_add(1);
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn create_forward_shading_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        transmission_scene_color_view: Option<&wgpu::TextureView>,
    ) -> wgpu::BindGroup {
        crate::profile_scope!("render", "forward_receiver", "full_binding_prepare");
        let params_buffer = self.forward_volumetric_apply.create_params_buffer(
            device,
            frame,
            render_region,
            integrated_volumetric_view.is_some(),
            "zircon-forward-volumetric-params",
        );
        let bind_group = self.create_forward_receiver_bind_group_with_volumetric(
            device,
            shadow_atlas_resources,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            &params_buffer,
            integrated_volumetric_view,
            transmission_scene_color_view,
            ForwardReflectionProviderPolicy::Scene,
        );
        self.full_forward_receiver_bind_group_create_count = self
            .full_forward_receiver_bind_group_create_count
            .saturating_add(1);
        bind_group
    }

    #[allow(clippy::too_many_arguments)]
    fn create_forward_receiver_bind_group_with_volumetric(
        &self,
        device: &wgpu::Device,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
        volumetric_params_buffer: &wgpu::Buffer,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        transmission_scene_color_view: Option<&wgpu::TextureView>,
        reflection_provider_policy: ForwardReflectionProviderPolicy,
    ) -> wgpu::BindGroup {
        let shadow_atlas_view = shadow_atlas_resources
            .map(ShadowAtlasResources::atlas_view)
            .unwrap_or(&self.fallback_shadow_atlas_view);
        let shadow_atlas_sampler = shadow_atlas_resources
            .map(ShadowAtlasResources::compare_sampler)
            .unwrap_or(&self.forward_shadow_compare_sampler);
        let shadow_atlas_slot_buffer = shadow_atlas_resources
            .map(ShadowAtlasResources::slot_buffer)
            .unwrap_or(&self.forward_shadow_atlas_fallback_slot_buffer);
        let shadow_atlas_globals_buffer = shadow_atlas_resources
            .map(ShadowAtlasResources::globals_buffer)
            .unwrap_or(&self.forward_shadow_atlas_fallback_globals_buffer);
        let reflection_probe_bindings = match reflection_provider_policy {
            ForwardReflectionProviderPolicy::Scene => self.reflection_probes.bindings(),
            ForwardReflectionProviderPolicy::EnvironmentCaptureDisabled => {
                self.reflection_probes.environment_capture_bindings()
            }
        };
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: SHADOW_ATLAS_BINDING,
                resource: wgpu::BindingResource::TextureView(shadow_atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: SHADOW_ATLAS_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(shadow_atlas_sampler),
            },
            wgpu::BindGroupEntry {
                binding: SHADOW_ATLAS_SLOT_BUFFER_BINDING,
                resource: shadow_atlas_slot_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: SHADOW_GLOBALS_BINDING,
                resource: shadow_atlas_globals_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: LIGHT_GRID_PARAMS_BINDING,
                resource: wgpu::BindingResource::Buffer(light_grid_params_buffer.unwrap_or(
                    wgpu::BufferBinding {
                        buffer: &self.forward_light_grid_params_buffer,
                        offset: 0,
                        size: None,
                    },
                )),
            },
            wgpu::BindGroupEntry {
                binding: LIGHT_ZBINS_BINDING,
                resource: wgpu::BindingResource::Buffer(light_zbins_buffer.unwrap_or(
                    wgpu::BufferBinding {
                        buffer: &self.forward_light_grid_empty_zbins_buffer,
                        offset: 0,
                        size: None,
                    },
                )),
            },
            wgpu::BindGroupEntry {
                binding: LIGHT_TILE_MASKS_BINDING,
                resource: wgpu::BindingResource::Buffer(light_tile_masks_buffer.unwrap_or(
                    wgpu::BufferBinding {
                        buffer: &self.forward_light_grid_empty_tile_masks_buffer,
                        offset: 0,
                        size: None,
                    },
                )),
            },
        ];
        entries.extend(reflection_probe_bindings.bind_group_entries());
        let lightmap_bindings = self.lightmaps.bindings();
        entries.extend(lightmap_bindings.bind_group_entries());
        entries.extend(
            self.forward_volumetric_apply
                .bind_group_entries(volumetric_params_buffer, integrated_volumetric_view),
        );
        entries.extend(
            self.transmission_scene_color
                .bind_group_entries(transmission_scene_color_view),
        );
        entries.extend(self.light_cookies.bind_group_entries());
        entries.extend(self.irradiance_volume.bind_group_entries());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-forward-shadow-receiver-bind-group"),
            layout: &self.forward_shadow_receiver_layout,
            entries: &entries,
        })
    }
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_receiver_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    let entries = forward_shadow_receiver_layout_entries();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-forward-shadow-receiver-layout"),
        entries: &entries,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn forward_shadow_receiver_layout_entries()
-> Vec<wgpu::BindGroupLayoutEntry> {
    let mut entries = Vec::new();
    entries.extend(shadow_atlas_bind_group_layout_entries(
        FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
    ));
    entries.extend(reflection_probe_bind_group_layout_entries());
    entries.extend(lightmap_bind_group_layout_entries());
    entries.extend(volumetric_apply_bind_group_layout_entries(
        FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
    ));
    entries.extend(transmission_scene_color_bind_group_layout_entries());
    entries.extend(light_cookie_bind_group_layout_entries());
    entries.extend(irradiance_volume_bind_group_layout_entries());
    entries.extend([
        wgpu::BindGroupLayoutEntry {
            binding: LIGHT_GRID_PARAMS_BINDING,
            visibility: FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: LIGHT_ZBINS_BINDING,
            visibility: FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: LIGHT_TILE_MASKS_BINDING,
            visibility: FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]);
    entries
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_light_grid_params_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-forward-light-grid-params-fallback"),
        contents: bytes_of(&LightGridParams::disabled()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_light_grid_empty_zbins_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    let words = [LIGHT_GRID_EMPTY_ZBIN_HEADER, 0, 0];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-forward-light-grid-zbins-fallback"),
        contents: bytemuck::cast_slice(&words),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_light_grid_empty_tile_masks_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    let words = [0_u32];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-forward-light-grid-tile-masks-fallback"),
        contents: bytemuck::cast_slice(&words),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_atlas_fallback_slot_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-forward-shadow-atlas-slots-fallback"),
        contents: bytemuck::bytes_of(&GpuShadowSlot::disabled()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_atlas_fallback_globals_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-forward-shadow-atlas-globals-fallback"),
        contents: bytemuck::bytes_of(&GpuShadowGlobals::disabled(1, 1)),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_compare_sampler(
    device: &wgpu::Device,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-forward-shadow-compare-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_fallback_shadow_atlas_view(
    device: &wgpu::Device,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-forward-shadow-fallback-texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: super::super::super::core::DEPTH_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

#[cfg(test)]
mod profile_contract_tests {
    #[test]
    fn forward_receiver_creation_profiles_standard_and_full_shapes_per_frame() {
        let source = include_str!("forward_shadow_receiver.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("forward receiver production source");

        assert!(source.contains("fn begin_forward_receiver_binding_profile_frame(&mut self)"));
        assert!(source.contains("standard_forward_receiver_bind_group_create_count = 0;"));
        assert!(source.contains("full_forward_receiver_bind_group_create_count = 0;"));
        assert!(source.contains("\"standard_bind_group_create\""));
        assert!(source.contains("\"full_binding_prepare\""));
        assert!(source.contains("forward_receiver_standard_bind_group_create_count"));
        assert!(source.contains("forward_receiver_full_bind_group_create_count"));
        assert_eq!(source.matches(".saturating_add(1);").count(), 2);
    }
}

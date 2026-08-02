use std::num::NonZeroU64;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};

use crate::core::math::UVec2;
use crate::text::sdf::{SdfBakeParams, SdfMode};

use super::super::render::ScreenSpaceUiTextBatch;

pub(super) const SDF_TEXT_EFFECT_OUTLINE: u32 = 1 << 0;
pub(super) const SDF_TEXT_EFFECT_SHADOW: u32 = 1 << 1;
pub(super) const SDF_TEXT_EFFECT_GLOW: u32 = 1 << 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub(super) enum SdfScreenPxRangeMode {
    #[default]
    CpuScreenSpace = 0,
    FragmentDerived = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SdfTextMaterial {
    pub(super) fill_color: [f32; 4],
    pub(super) outline_color: [f32; 4],
    pub(super) shadow_color: [f32; 4],
    pub(super) glow_color: [f32; 4],
    pub(super) outline_width_px: f32,
    pub(super) shadow_offset_px: [f32; 2],
    pub(super) glow_radius_px: f32,
    pub(super) effect_flags: u32,
    pub(super) projection_mode: SdfScreenPxRangeMode,
    pub(super) distance_field_mode: SdfMode,
    pub(super) atlas_dimensions: [f32; 2],
}

impl Default for SdfTextMaterial {
    fn default() -> Self {
        Self {
            fill_color: [0.0; 4],
            outline_color: [0.0; 4],
            shadow_color: [0.0; 4],
            glow_color: [0.0; 4],
            outline_width_px: 0.0,
            shadow_offset_px: [0.0; 2],
            glow_radius_px: 0.0,
            effect_flags: 0,
            projection_mode: SdfScreenPxRangeMode::CpuScreenSpace,
            distance_field_mode: SdfMode::Sdf,
            atlas_dimensions: [1.0, 1.0],
        }
    }
}

impl SdfTextMaterial {
    pub(super) fn from_text(text: &ScreenSpaceUiTextBatch, atlas_size: UVec2) -> Self {
        let effect_limit =
            (SdfBakeParams::default().screen_px_range(text.font_size) * 0.5).max(1.0);
        let mut material = Self {
            fill_color: text.color,
            distance_field_mode: text.distance_field_mode,
            projection_mode: if text.clip_transform.is_some() {
                SdfScreenPxRangeMode::FragmentDerived
            } else {
                SdfScreenPxRangeMode::CpuScreenSpace
            },
            atlas_dimensions: [atlas_size.x.max(1) as f32, atlas_size.y.max(1) as f32],
            ..Self::default()
        };
        if let Some(outline) = text.text_effects.outline {
            material.outline_width_px = outline.width_px.clamp(0.0, effect_limit);
            material.outline_color = outline.color;
            material.effect_flags |= SDF_TEXT_EFFECT_OUTLINE;
        }
        if let Some(shadow) = text.text_effects.shadow {
            material.shadow_offset_px = [
                shadow.offset_px[0].clamp(-effect_limit, effect_limit),
                shadow.offset_px[1].clamp(-effect_limit, effect_limit),
            ];
            material.shadow_color = shadow.color;
            material.effect_flags |= SDF_TEXT_EFFECT_SHADOW;
        }
        if let Some(glow) = text.text_effects.glow {
            material.glow_radius_px = glow.radius_px.clamp(0.0, effect_limit);
            material.glow_color = glow.color;
            material.effect_flags |= SDF_TEXT_EFFECT_GLOW;
        }
        material
    }

    pub(super) fn uniform(self) -> SdfTextMaterialUniform {
        SdfTextMaterialUniform {
            fill_color: self.fill_color,
            outline_color: self.outline_color,
            shadow_color: self.shadow_color,
            glow_color: self.glow_color,
            effect_params: [
                self.outline_width_px,
                self.shadow_offset_px[0],
                self.shadow_offset_px[1],
                self.glow_radius_px,
            ],
            flags: [
                self.effect_flags,
                self.projection_mode as u32,
                self.distance_field_mode.shader_discriminant(),
                0,
            ],
            projection_params: [self.atlas_dimensions[0], self.atlas_dimensions[1], 0.0, 0.0],
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
pub(super) struct SdfTextMaterialUniform {
    pub(super) fill_color: [f32; 4],
    pub(super) outline_color: [f32; 4],
    pub(super) shadow_color: [f32; 4],
    pub(super) glow_color: [f32; 4],
    pub(super) effect_params: [f32; 4],
    pub(super) flags: [u32; 4],
    pub(super) projection_params: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SdfTextMaterialDraw {
    pub(super) vertices: Range<u32>,
    pub(super) material_index: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SdfTextMaterialDrawPlan {
    pub(super) materials: Vec<SdfTextMaterial>,
    pub(super) draws: Vec<SdfTextMaterialDraw>,
}

impl SdfTextMaterialDrawPlan {
    #[cfg(test)]
    pub(super) fn from_ranges(
        texts: &[ScreenSpaceUiTextBatch],
        atlas_size: UVec2,
        decoration_vertex_count: u32,
        text_ranges: &[Range<u32>],
    ) -> Self {
        let mut plan = Self::default();
        plan.rebuild(texts, atlas_size, decoration_vertex_count, text_ranges);
        plan
    }

    pub(super) fn rebuild(
        &mut self,
        texts: &[ScreenSpaceUiTextBatch],
        atlas_size: UVec2,
        decoration_vertex_count: u32,
        text_ranges: &[Range<u32>],
    ) {
        self.materials.clear();
        self.draws.clear();
        if decoration_vertex_count > 0 {
            self.materials.push(SdfTextMaterial::default());
            self.draws.push(SdfTextMaterialDraw {
                vertices: 0..decoration_vertex_count,
                material_index: 0,
            });
        }
        for (text, range) in texts.iter().zip(text_ranges) {
            if range.is_empty() {
                continue;
            }
            let range = range.start.saturating_add(decoration_vertex_count)
                ..range.end.saturating_add(decoration_vertex_count);
            self.push_text_draw(range, SdfTextMaterial::from_text(text, atlas_size));
        }
        if self.materials.is_empty() {
            self.materials.push(SdfTextMaterial::default());
        }
    }

    fn push_text_draw(&mut self, vertices: Range<u32>, material: SdfTextMaterial) {
        if let Some(last_draw) = self.draws.last_mut() {
            let last_material = self.materials.get(last_draw.material_index as usize);
            if last_draw.vertices.end == vertices.start && last_material == Some(&material) {
                last_draw.vertices.end = vertices.end;
                return;
            }
        }
        let material_index = self.materials.len() as u32;
        self.materials.push(material);
        self.draws.push(SdfTextMaterialDraw {
            vertices,
            material_index,
        });
    }
}

pub(super) struct SdfTextMaterialResources {
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
    pub(super) bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    uniform_stride: u32,
    capacity: usize,
    uploaded_materials: Vec<SdfTextMaterial>,
    upload_bytes: Vec<u8>,
    upload_initialized: bool,
}

impl SdfTextMaterialResources {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<SdfTextMaterialUniform>() as u64;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-screen-space-ui-sdf-material-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: NonZeroU64::new(uniform_size),
                },
                count: None,
            }],
        });
        let uniform_stride = aligned_uniform_stride(
            uniform_size as u32,
            device.limits().min_uniform_buffer_offset_alignment,
        );
        let buffer = create_material_buffer(device, u64::from(uniform_stride));
        let bind_group =
            create_material_bind_group(device, &bind_group_layout, &buffer, uniform_size);
        Self {
            bind_group_layout,
            bind_group,
            buffer,
            uniform_stride,
            capacity: 1,
            uploaded_materials: Vec::new(),
            upload_bytes: Vec::new(),
            upload_initialized: false,
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        materials: &[SdfTextMaterial],
    ) {
        let material_count = materials.len().max(1);
        let byte_len = usize::try_from(self.uniform_stride)
            .unwrap_or(usize::MAX)
            .saturating_mul(material_count);
        let mut buffer_recreated = false;
        if material_count > self.capacity {
            self.buffer = create_material_buffer(device, byte_len as u64);
            self.bind_group = create_material_bind_group(
                device,
                &self.bind_group_layout,
                &self.buffer,
                std::mem::size_of::<SdfTextMaterialUniform>() as u64,
            );
            self.capacity = material_count;
            buffer_recreated = true;
        }
        if self.upload_initialized && !buffer_recreated && self.uploaded_materials == materials {
            return;
        }

        self.upload_bytes.clear();
        self.upload_bytes.resize(byte_len, 0);
        for (index, material) in materials.iter().enumerate() {
            let offset = index * self.uniform_stride as usize;
            let uniform = material.uniform();
            let source = bytemuck::bytes_of(&uniform);
            self.upload_bytes[offset..offset + source.len()].copy_from_slice(source);
        }
        queue.write_buffer(&self.buffer, 0, &self.upload_bytes);
        self.uploaded_materials.clear();
        self.uploaded_materials.extend_from_slice(materials);
        self.upload_initialized = true;
    }

    pub(super) fn dynamic_offset(&self, material_index: u32) -> u32 {
        material_index.saturating_mul(self.uniform_stride)
    }

    #[cfg(test)]
    pub(super) fn uniform_stride(&self) -> u32 {
        self.uniform_stride
    }
}

pub(super) fn aligned_uniform_stride(uniform_size: u32, alignment: u32) -> u32 {
    let alignment = alignment.max(1);
    uniform_size
        .saturating_add(alignment - 1)
        .saturating_div(alignment)
        .saturating_mul(alignment)
}

pub(super) fn fragment_screen_px_range(
    atlas_px_range: f32,
    atlas_dimensions: [f32; 2],
    uv_dx: [f32; 2],
    uv_dy: [f32; 2],
) -> f32 {
    let atlas_dimensions = [atlas_dimensions[0].max(1.0), atlas_dimensions[1].max(1.0)];
    let uv_fwidth = [
        uv_dx[0].abs() + uv_dy[0].abs(),
        uv_dx[1].abs() + uv_dy[1].abs(),
    ];
    let screen_texture_size = [
        1.0 / uv_fwidth[0].max(f32::EPSILON),
        1.0 / uv_fwidth[1].max(f32::EPSILON),
    ];
    let atlas_unit_range = [
        atlas_px_range.max(1.0) / atlas_dimensions[0],
        atlas_px_range.max(1.0) / atlas_dimensions[1],
    ];
    (0.5 * (atlas_unit_range[0] * screen_texture_size[0]
        + atlas_unit_range[1] * screen_texture_size[1]))
        .max(1.0)
}

pub(super) fn sdf_effect_coverage(distance: f32, screen_px_range: f32, expand_px: f32) -> f32 {
    let signed_distance = (distance - 0.5) * screen_px_range.max(1.0) + expand_px;
    (signed_distance + 0.5).clamp(0.0, 1.0)
}

pub(super) fn mtsdf_glow_coverage(true_distance: f32, screen_px_range: f32, radius_px: f32) -> f32 {
    let radius_px = radius_px.max(f32::EPSILON);
    let signed_distance = (true_distance - 0.5) * screen_px_range.max(1.0);
    (1.0 - (-signed_distance).max(0.0) / radius_px).clamp(0.0, 1.0)
        * (1.0 - sdf_effect_coverage(true_distance, screen_px_range, 0.0))
}

pub(super) fn shadow_sample_uv(
    uv: [f32; 2],
    uv_dx: [f32; 2],
    uv_dy: [f32; 2],
    offset_px: [f32; 2],
) -> [f32; 2] {
    [
        uv[0] - uv_dx[0] * offset_px[0] - uv_dy[0] * offset_px[1],
        uv[1] - uv_dx[1] * offset_px[0] - uv_dy[1] * offset_px[1],
    ]
}

pub(super) fn straight_alpha_over(under: [f32; 4], over: [f32; 4]) -> [f32; 4] {
    let alpha = over[3] + under[3] * (1.0 - over[3]);
    if alpha <= f32::EPSILON {
        return [0.0; 4];
    }
    let under_factor = under[3] * (1.0 - over[3]);
    [
        (over[0] * over[3] + under[0] * under_factor) / alpha,
        (over[1] * over[3] + under[1] * under_factor) / alpha,
        (over[2] * over[3] + under[2] * under_factor) / alpha,
        alpha,
    ]
}

fn create_material_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-screen-space-ui-sdf-material-buffer"),
        size: size.max(std::mem::size_of::<SdfTextMaterialUniform>() as u64),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    uniform_size: u64,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-screen-space-ui-sdf-material-bind-group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: NonZeroU64::new(uniform_size),
            }),
        }],
    })
}

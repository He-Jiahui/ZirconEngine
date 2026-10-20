use crate::asset::{TextureAsset, TexturePayload};
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderColorGradingSettings, RenderColorLookupSettings,
    RenderColorLookupTextureLayout, RenderColorLutReadbackReport, RenderExposureReadbackReport,
    RenderImageDescriptor, RenderPostProcessEffectStackSettings, RenderSceneVelocityReadbackReport,
    RenderTonemapOperator, RenderTonemapSettings,
};
use crate::graphics::backend::{read_buffer_f32x4, read_texture_rgba, read_texture_rgba16float_3d};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use crate::graphics::types::ViewportRenderFrame;
use crate::rhi::TextureFormat;

#[cfg(test)]
pub(super) fn attach_scene_velocity_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    graph_resources: &RenderGraphExecutionResources,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let resource_name = PostProcessGraphResourceNames::SCENE_VELOCITY;
    let Some(texture) = graph_resources.physical_texture(resource_name) else {
        return;
    };
    let Some(desc) = graph_resources.physical_texture_desc(resource_name) else {
        return;
    };
    if desc.format != TextureFormat::Rg16Float
        || desc.sample_count != 1
        || desc.depth != 1
        || desc.array_layers != 1
    {
        return;
    }
    let size = crate::core::math::UVec2::new(desc.width, desc.height);
    if size.x == 0 || size.y == 0 {
        return;
    }
    let Ok(bytes) = read_texture_rgba(device, queue, texture, size) else {
        return;
    };
    graph_execution_record.set_scene_velocity_readback_report(
        RenderSceneVelocityReadbackReport::from_raw_rg16_float_bytes(size, &bytes),
    );
    graph_execution_record.set_scene_velocity_readback_rg16_float_bytes(bytes);
}

#[cfg(test)]
pub(super) fn attach_exposure_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    graph_resources: &RenderGraphExecutionResources,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) -> RenderExposureReadbackReport {
    let Some(buffer) = graph_resources.buffer(PostProcessGraphResourceNames::EXPOSURE_CURRENT)
    else {
        return RenderExposureReadbackReport::default();
    };
    let Ok(words) = read_buffer_f32x4(device, queue, buffer) else {
        return RenderExposureReadbackReport::default();
    };
    let report = RenderExposureReadbackReport::from_words(words);
    graph_execution_record.set_exposure_readback_report(report);
    report
}

#[cfg(test)]
pub(super) fn attach_color_lut_readback_stats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    graph_resources: &RenderGraphExecutionResources,
    exposure_readback_report: RenderExposureReadbackReport,
    graph_execution_record: &mut RenderGraphExecutionRecord,
) {
    let resource_name = PostProcessGraphResourceNames::COLOR_LUT;
    let Some(texture) = graph_resources.owned_texture(resource_name) else {
        return;
    };
    let Some(desc) = graph_resources.owned_texture_desc(resource_name) else {
        return;
    };
    if desc.format != TextureFormat::Rgba16Float || desc.sample_count != 1 {
        return;
    }
    let size = [desc.width, desc.height, desc.depth];
    if size.iter().any(|extent| *extent == 0) {
        return;
    }
    let Ok(bytes) = read_texture_rgba16float_3d(device, queue, texture, size) else {
        return;
    };
    graph_execution_record.set_color_lut_readback_report(color_lut_readback_report_for_frame(
        streamer,
        frame,
        size,
        &bytes,
        exposure_readback_report,
    ));
}

#[cfg(test)]
fn color_lut_readback_report_for_frame(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    size: [u32; 3],
    bytes: &[u8],
    exposure_readback_report: RenderExposureReadbackReport,
) -> RenderColorLutReadbackReport {
    if let Some(reference) = UserColorLutReadbackReference::from_frame(streamer, frame, size) {
        return RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
            size,
            bytes,
            |source_color| reference.expected_rgb(source_color),
        );
    }
    if let Some(reference) =
        ColorTransformLutReadbackReference::from_frame(frame, exposure_readback_report)
    {
        return RenderColorLutReadbackReport::from_raw_rgba16_float_color_transform_bytes(
            size,
            bytes,
            |source_color| reference.expected_rgb(source_color),
        );
    }

    RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes(size, bytes)
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct ColorTransformLutReadbackReference {
    tonemap: RenderTonemapSettings,
    grading: RenderColorGradingSettings,
    exposure_multiplier: f32,
}

#[cfg(test)]
impl ColorTransformLutReadbackReference {
    fn from_frame(
        frame: &ViewportRenderFrame,
        exposure_readback_report: RenderExposureReadbackReport,
    ) -> Option<Self> {
        let effect_stack = frame.post_process().effect_stack;
        if effect_stack.color_lookup.is_enabled() || !exposure_readback_report.history_valid() {
            return None;
        }
        let grading = frame.post_process().color_grading;
        let exposure_multiplier = exposure_readback_report.multiplier();
        color_transform_requires_reference(grading, effect_stack, exposure_multiplier).then_some(
            Self {
                tonemap: effect_stack.tonemap,
                grading,
                exposure_multiplier,
            },
        )
    }

    fn expected_rgb(self, source_color: [f32; 3]) -> [f32; 3] {
        self.apply_color_grading(self.apply_tonemap(source_color))
    }

    fn apply_tonemap(self, color: [f32; 3]) -> [f32; 3] {
        let exposure = 2.0_f32.powf(self.tonemap.render_exposure_bias() as f32)
            * self.exposure_multiplier.max(0.0);
        let white_point = (self.tonemap.render_white_point() as f32).max(0.001);
        let mut mapped = map_color(color, |channel| (channel * exposure).max(0.0));
        mapped = match self.tonemap.operator {
            RenderTonemapOperator::None => mapped,
            RenderTonemapOperator::Reinhard => {
                map_color(mapped, |channel| channel / (1.0 + channel / white_point))
            }
            RenderTonemapOperator::Aces => map_color(mapped, |channel| {
                let a = 2.51;
                let b = 0.03;
                let c = 2.43;
                let d = 0.59;
                let e = 0.14;
                ((channel * (a * channel + b)) / (channel * (c * channel + d) + e)).clamp(0.0, 1.0)
            }),
            RenderTonemapOperator::Filmic => map_color(mapped, |channel| {
                let mapped = (channel - 0.004).max(0.0);
                (mapped * (6.2 * mapped + 0.5)) / (mapped * (6.2 * mapped + 1.7) + 0.06)
            }),
        };
        mapped
    }

    fn apply_color_grading(self, color: [f32; 3]) -> [f32; 3] {
        let exposure = (self.grading.exposure as f32).max(0.0);
        let contrast = (self.grading.contrast as f32).max(0.0);
        let saturation = (self.grading.saturation as f32).max(0.0);
        let gamma = (self.grading.gamma as f32).max(0.001);
        let tint = [
            (self.grading.tint.x as f32).max(0.0),
            (self.grading.tint.y as f32).max(0.0),
            (self.grading.tint.z as f32).max(0.0),
        ];
        let mut graded = map_color(color, |channel| channel * exposure);
        let luma = graded[0] * 0.2126 + graded[1] * 0.7152 + graded[2] * 0.0722;
        graded = [
            mix_channel(luma, graded[0], saturation),
            mix_channel(luma, graded[1], saturation),
            mix_channel(luma, graded[2], saturation),
        ];
        graded = map_color(graded, |channel| ((channel - 0.5) * contrast) + 0.5);
        graded = map_color(graded, |channel| channel.max(0.0));
        graded = map_color(graded, |channel| channel.powf(1.0 / gamma));
        [
            graded[0] * tint[0],
            graded[1] * tint[1],
            graded[2] * tint[2],
        ]
    }
}

#[cfg(test)]
fn color_transform_requires_reference(
    grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
    exposure_multiplier: f32,
) -> bool {
    grading != RenderColorGradingSettings::default()
        || effect_stack.tonemap.is_enabled()
        || (exposure_multiplier - 1.0).abs() > 0.0001
}

#[cfg(test)]
fn map_color(color: [f32; 3], mut map: impl FnMut(f32) -> f32) -> [f32; 3] {
    [map(color[0]), map(color[1]), map(color[2])]
}

#[cfg(test)]
struct UserColorLutReadbackReference {
    texture: std::sync::Arc<TextureAsset>,
    width: u32,
    height: u32,
    mode: UserColorLutReadbackMode,
    intensity: f32,
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum UserColorLutReadbackMode {
    Texture2d,
    Texture2dStrip { size: u32 },
    Texture3d { size: u32 },
}

#[cfg(test)]
impl UserColorLutReadbackMode {
    fn rgba_byte_len(self, width: u32, height: u32) -> Option<usize> {
        match self {
            Self::Texture2d | Self::Texture2dStrip { .. } => rgba8_len(width, height),
            Self::Texture3d { size } => (size as usize)
                .checked_mul(size as usize)?
                .checked_mul(size as usize)?
                .checked_mul(4),
        }
    }
}

#[cfg(test)]
impl UserColorLutReadbackReference {
    fn from_frame(
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        readback_size: [u32; 3],
    ) -> Option<Self> {
        let effect_stack = frame.post_process().effect_stack;
        if !user_lut_readback_supports_frame(frame.post_process().color_grading, effect_stack) {
            return None;
        }
        Self::from_settings(streamer, effect_stack.color_lookup, readback_size)
    }

    fn from_settings(
        streamer: &ResourceStreamer,
        settings: RenderColorLookupSettings,
        readback_size: [u32; 3],
    ) -> Option<Self> {
        let texture_id = settings
            .is_enabled()
            .then(|| settings.texture.map(|texture| texture.id()))
            .flatten()?;
        let texture = streamer
            .asset_manager()
            .ok()?
            .load_texture_asset_snapshot(texture_id)
            .ok()?;
        let descriptor = texture.render_image_descriptor();
        let mode = user_lut_readback_mode(
            streamer,
            texture_id,
            settings.texture_layout,
            &descriptor,
            readback_size,
        )?;
        let width = texture.width;
        let height = texture.height;
        if texture.payload != TexturePayload::Rgba8
            || texture.rgba.len() < mode.rgba_byte_len(width, height)?
        {
            return None;
        }

        Some(Self {
            texture: std::sync::Arc::clone(texture.resource()),
            width,
            height,
            mode,
            intensity: (settings.render_intensity() as f32).clamp(0.0, 1.0),
        })
    }

    fn expected_rgb(&self, source_color: [f32; 3]) -> [f32; 3] {
        let user_color = self.sample(source_color);
        [
            mix_channel(source_color[0], user_color[0], self.intensity),
            mix_channel(source_color[1], user_color[1], self.intensity),
            mix_channel(source_color[2], user_color[2], self.intensity),
        ]
    }

    fn sample(&self, color: [f32; 3]) -> [f32; 3] {
        match self.mode {
            UserColorLutReadbackMode::Texture2d => [
                self.sample_1d_channel(color[0]),
                self.sample_1d_channel(color[1]),
                self.sample_1d_channel(color[2]),
            ],
            UserColorLutReadbackMode::Texture2dStrip { size } => {
                let red = lut_axis_index(color[0], size);
                let green = lut_axis_index(color[1], size);
                let blue = lut_axis_index(color[2], size);
                let x = blue.saturating_mul(size).saturating_add(red);
                self.texel_rgb(x.min(self.width.saturating_sub(1)), green.min(size - 1))
            }
            UserColorLutReadbackMode::Texture3d { size } => {
                let red = lut_axis_index(color[0], size);
                let green = lut_axis_index(color[1], size);
                let blue = lut_axis_index(color[2], size);
                let x = red.min(self.width.saturating_sub(1));
                let y = green.min(self.height.saturating_sub(1));
                let z_offset = blue.saturating_mul(self.width.saturating_mul(self.height));
                self.texel_rgb_by_flat_index(z_offset.saturating_add(y * self.width + x))
            }
        }
    }

    fn sample_1d_channel(&self, value: f32) -> f32 {
        let x = lut_axis_index(value, self.width);
        self.texel_rgb(x.min(self.width.saturating_sub(1)), 0)[0]
    }

    fn texel_rgb(&self, x: u32, y: u32) -> [f32; 3] {
        self.texel_rgb_by_flat_index(y.saturating_mul(self.width).saturating_add(x))
    }

    fn texel_rgb_by_flat_index(&self, flat_index: u32) -> [f32; 3] {
        let offset = flat_index as usize * 4;
        if offset + 2 >= self.texture.rgba.len() {
            return [0.0; 3];
        }
        [
            self.texture.rgba[offset] as f32 / 255.0,
            self.texture.rgba[offset + 1] as f32 / 255.0,
            self.texture.rgba[offset + 2] as f32 / 255.0,
        ]
    }
}

#[cfg(test)]
fn user_lut_readback_supports_frame(
    color_grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> bool {
    color_grading == RenderColorGradingSettings::default()
        && effect_stack.tonemap == RenderTonemapSettings::default()
}

#[cfg(test)]
fn user_lut_readback_mode(
    streamer: &ResourceStreamer,
    texture_id: crate::core::resource::ResourceId,
    layout: RenderColorLookupTextureLayout,
    descriptor: &RenderImageDescriptor,
    readback_size: [u32; 3],
) -> Option<UserColorLutReadbackMode> {
    let lut_size = readback_size[0];
    if readback_size != [lut_size; 3] || lut_size == 0 {
        return None;
    }
    if streamer
        .prepared_post_process_lut_3d_view(texture_id, layout)
        .is_some()
        && layout.matches_texture_3d(descriptor)
        && descriptor.width == lut_size
        && descriptor.height == lut_size
        && descriptor.depth_or_array_layers == lut_size
    {
        return Some(UserColorLutReadbackMode::Texture3d { size: lut_size });
    }
    if let Some((_, is_strip)) = streamer.prepared_post_process_lut_2d_view(texture_id, layout) {
        if is_strip
            && layout.matches_texture_2d_strip(descriptor)
            && descriptor.width == lut_size.saturating_mul(lut_size)
            && descriptor.height == lut_size
        {
            return Some(UserColorLutReadbackMode::Texture2dStrip { size: lut_size });
        }
        if !is_strip && descriptor.width == lut_size && descriptor.height > 0 {
            return Some(UserColorLutReadbackMode::Texture2d);
        }
    }
    None
}

#[cfg(test)]
fn lut_axis_index(value: f32, size: u32) -> u32 {
    let max_index = size.max(1) - 1;
    (value.clamp(0.0, 1.0) * max_index as f32)
        .round()
        .min(max_index as f32) as u32
}

#[cfg(test)]
fn mix_channel(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
fn rgba8_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)
}

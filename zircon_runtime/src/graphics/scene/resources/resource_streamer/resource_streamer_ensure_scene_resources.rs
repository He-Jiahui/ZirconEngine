use crate::core::framework::render::{RenderColorLookupTextureLayout, RenderImageDescriptor};
use crate::core::resource::ResourceId;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_scene_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &ViewportRenderFrame,
    ) -> Result<(), GraphicsError> {
        self.last_material_count = 0;
        self.last_material_ready_count = 0;
        self.last_material_fallback_count = 0;
        self.last_material_validation_error_count = 0;
        self.last_material_diagnostic_count = 0;
        self.last_sprite_count = 0;
        self.last_sprite_ready_count = 0;
        self.last_sprite_texture_fallback_count = 0;
        self.last_post_process_lut_request_count = 0;
        self.last_post_process_lut_ready_count = 0;
        self.last_post_process_lut_fallback_count = 0;
        self.last_post_process_lut_2d_strip_ready_count = 0;
        self.last_post_process_lut_3d_request_count = 0;
        self.last_post_process_lut_unsupported_shape_count = 0;
        for mesh in frame.meshes() {
            let direct_mesh_ready = mesh
                .mesh
                .map(|mesh| self.ensure_mesh(device, mesh).is_ok())
                .unwrap_or(false);
            if !direct_mesh_ready {
                self.ensure_model(device, mesh.model)?;
            }
            self.ensure_material(device, queue, texture_layout, mesh.material)?;
            self.record_material_summary(mesh.material.id());
        }
        for sprite in frame.sprites() {
            self.last_sprite_count += 1;
            if self
                .ensure_sprite_texture(device, queue, texture_layout, sprite.image.id())
                .is_ok()
            {
                self.last_sprite_ready_count += 1;
            } else {
                self.last_sprite_texture_fallback_count += 1;
            }
        }
        if let Some(request) = effect_stack_lut_texture_request(frame) {
            self.last_post_process_lut_request_count += 1;
            if matches!(
                request.texture_layout,
                RenderColorLookupTextureLayout::Texture3d { .. }
            ) {
                self.last_post_process_lut_3d_request_count += 1;
            }
            self.record_effect_stack_lut_texture_readiness(device, queue, texture_layout, request);
        }
        Ok(())
    }

    fn record_material_summary(&mut self, material_id: crate::core::resource::ResourceId) {
        self.last_material_count += 1;
        if let Some(summary) = self.material_readiness_summary(&material_id) {
            if summary.is_ready {
                self.last_material_ready_count += 1;
            }
            if summary.uses_fallback {
                self.last_material_fallback_count += 1;
            }
            self.last_material_validation_error_count += summary.validation_error_count;
            self.last_material_diagnostic_count += summary.diagnostic_count;
        }
    }

    fn record_effect_stack_lut_texture_readiness(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        request: EffectStackLutTextureRequest,
    ) {
        let Some(texture_id) = request.texture_id else {
            self.last_post_process_lut_fallback_count += 1;
            return;
        };

        let Ok(texture) = self.asset_manager.load_texture_asset(texture_id) else {
            self.last_post_process_lut_fallback_count += 1;
            return;
        };
        let status = effect_stack_lut_texture_status(
            request.texture_layout,
            &texture.render_image_descriptor(),
        );

        match status {
            EffectStackLutTextureStatus::Ready2d | EffectStackLutTextureStatus::Ready2dStrip => {
                if self
                    .ensure_texture(device, queue, texture_layout, texture_id)
                    .is_ok()
                {
                    self.last_post_process_lut_ready_count += 1;
                    if status == EffectStackLutTextureStatus::Ready2dStrip {
                        self.last_post_process_lut_2d_strip_ready_count += 1;
                    }
                } else {
                    self.last_post_process_lut_fallback_count += 1;
                }
            }
            EffectStackLutTextureStatus::Ready3d => {
                if !matches!(
                    request.texture_layout,
                    RenderColorLookupTextureLayout::Texture3d { .. }
                ) {
                    self.last_post_process_lut_3d_request_count += 1;
                }
                if self
                    .ensure_post_process_lut_texture(device, queue, texture_id)
                    .is_ok()
                {
                    self.last_post_process_lut_ready_count += 1;
                } else {
                    self.last_post_process_lut_fallback_count += 1;
                }
            }
            EffectStackLutTextureStatus::UnsupportedShape => {
                self.last_post_process_lut_unsupported_shape_count += 1;
                self.last_post_process_lut_fallback_count += 1;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectStackLutTextureRequest {
    texture_id: Option<ResourceId>,
    texture_layout: RenderColorLookupTextureLayout,
}

fn effect_stack_lut_texture_request(
    frame: &ViewportRenderFrame,
) -> Option<EffectStackLutTextureRequest> {
    let settings = frame.extract.post_process.effect_stack.color_lookup;
    settings.is_enabled().then(|| EffectStackLutTextureRequest {
        texture_id: settings.texture.map(|texture| texture.id()),
        texture_layout: settings.texture_layout,
    })
}

#[cfg(test)]
fn effect_stack_lut_texture_id(frame: &ViewportRenderFrame) -> Option<ResourceId> {
    effect_stack_lut_texture_request(frame).and_then(|request| request.texture_id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectStackLutTextureStatus {
    Ready2d,
    Ready2dStrip,
    Ready3d,
    UnsupportedShape,
}

fn effect_stack_lut_texture_status(
    layout: RenderColorLookupTextureLayout,
    descriptor: &RenderImageDescriptor,
) -> EffectStackLutTextureStatus {
    if layout.matches_texture_3d(descriptor) {
        return EffectStackLutTextureStatus::Ready3d;
    }
    if layout.matches_texture_2d_strip(descriptor) {
        return EffectStackLutTextureStatus::Ready2dStrip;
    }
    if layout.accepts_current_post_process_binding(descriptor) {
        return EffectStackLutTextureStatus::Ready2d;
    }
    EffectStackLutTextureStatus::UnsupportedShape
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderColorLookupSettings, RenderColorLookupTextureLayout, RenderFrameExtract,
        RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension,
        RenderImageFallbackKind, RenderImageUsage, RenderPostProcessEffectStackSettings,
        RenderSamplerDescriptor, RenderWorldSnapshotHandle,
    };
    use crate::core::math::UVec2;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::types::ViewportRenderFrame;
    use crate::scene::World;

    use super::{
        effect_stack_lut_texture_id, effect_stack_lut_texture_request,
        effect_stack_lut_texture_status, EffectStackLutTextureStatus,
    };

    #[test]
    fn effect_stack_lut_texture_id_uses_enabled_lookup_handle() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "postprocess/lut/filmic",
        ));
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );
        extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: Some(texture),
                intensity: 0.75,
                ..Default::default()
            },
            ..Default::default()
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        assert_eq!(effect_stack_lut_texture_id(&frame), Some(texture.id()));
    }

    #[test]
    fn effect_stack_lut_texture_id_ignores_disabled_lookup_handle() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "postprocess/lut/disabled",
        ));
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );
        extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: Some(texture),
                intensity: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        assert_eq!(effect_stack_lut_texture_id(&frame), None);
    }

    #[test]
    fn effect_stack_lut_texture_request_tracks_enabled_lut_without_handle() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );
        extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
            color_lookup: RenderColorLookupSettings {
                texture: None,
                intensity: 0.5,
                ..Default::default()
            },
            ..Default::default()
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        assert!(effect_stack_lut_texture_request(&frame).is_some());
        assert_eq!(effect_stack_lut_texture_id(&frame), None);
    }

    #[test]
    fn effect_stack_lut_texture_status_accepts_2d_strip_for_current_binding() {
        let descriptor = texture_descriptor(33 * 33, 33, 1, RenderImageDimension::D2);

        assert_eq!(
            effect_stack_lut_texture_status(
                RenderColorLookupTextureLayout::Texture2dStrip { size: 33 },
                &descriptor,
            ),
            EffectStackLutTextureStatus::Ready2dStrip
        );
    }

    #[test]
    fn effect_stack_lut_texture_status_accepts_3d_lut_for_texture_3d_binding() {
        let descriptor = texture_descriptor(33, 33, 33, RenderImageDimension::D3);

        assert_eq!(
            effect_stack_lut_texture_status(
                RenderColorLookupTextureLayout::Texture3d { size: 33 },
                &descriptor,
            ),
            EffectStackLutTextureStatus::Ready3d
        );
    }

    #[test]
    fn effect_stack_lut_texture_status_rejects_non_2d_binding_shapes() {
        let array_descriptor = texture_descriptor(64, 64, 4, RenderImageDimension::D2);
        let wrong_strip = texture_descriptor(64, 64, 1, RenderImageDimension::D2);

        assert_eq!(
            effect_stack_lut_texture_status(
                RenderColorLookupTextureLayout::Auto,
                &array_descriptor
            ),
            EffectStackLutTextureStatus::UnsupportedShape
        );
        assert_eq!(
            effect_stack_lut_texture_status(
                RenderColorLookupTextureLayout::Texture2dStrip { size: 33 },
                &wrong_strip,
            ),
            EffectStackLutTextureStatus::UnsupportedShape
        );
    }

    fn texture_descriptor(
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        dimension: RenderImageDimension,
    ) -> RenderImageDescriptor {
        RenderImageDescriptor {
            width,
            height,
            depth_or_array_layers,
            dimension,
            format: "rgba8unorm".to_string(),
            color_space: RenderImageColorSpace::Linear,
            sampler: RenderSamplerDescriptor::default(),
            usage: vec![RenderImageUsage::Sampled],
            asset_usage: Vec::new(),
            mip_count: 1,
            array_layer_count: 1,
            fallback: RenderImageFallbackKind::MissingImage,
        }
    }
}

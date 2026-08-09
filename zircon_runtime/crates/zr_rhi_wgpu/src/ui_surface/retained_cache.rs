const UI_SURFACE_COPY_BYTES_PER_PIXEL: u64 = 4;

pub(super) struct WgpuRetainedSurfaceCache {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    initialized: bool,
}

impl WgpuRetainedSurfaceCache {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let size = (size.0.max(1), size.1.max(1));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-retained-cache"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            size,
            format,
            initialized: false,
        }
    }

    pub(super) fn resize(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) {
        *self = Self::new(device, format, size);
    }

    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(super) fn initialized(&self) -> bool {
        self.initialized
    }

    pub(super) fn invalidate(&mut self) {
        self.initialized = false;
    }

    pub(super) fn matches(&self, format: wgpu::TextureFormat, size: (u32, u32)) -> bool {
        self.format == format && self.size == (size.0.max(1), size.1.max(1))
    }

    pub(super) fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    pub(super) fn record_copy_to_surface(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture: &wgpu::Texture,
        surface_size: (u32, u32),
    ) -> u64 {
        encoder.copy_texture_to_texture(
            self.texture.as_image_copy(),
            surface_texture.as_image_copy(),
            wgpu::Extent3d {
                width: surface_size.0.max(1),
                height: surface_size.1.max(1),
                depth_or_array_layers: 1,
            },
        );
        retained_copy_byte_count(self.format, surface_size)
    }
}

fn retained_copy_byte_count(format: wgpu::TextureFormat, surface_size: (u32, u32)) -> u64 {
    let bytes_per_pixel = match format {
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm => {
            UI_SURFACE_COPY_BYTES_PER_PIXEL
        }
        _ => 0,
    };
    u64::from(surface_size.0.max(1))
        .saturating_mul(u64::from(surface_size.1.max(1)))
        .saturating_mul(bytes_per_pixel)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::mpsc;

    use zr_rhi::{UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceRect};

    use super::super::batching::{batch_draw_plan, CompiledUiBatchPlanCache};
    use super::super::pipeline::{
        create_image_bind_group_layout, create_image_pipeline, create_solid_instance_pipeline,
        create_solid_pipeline,
    };
    use super::super::render_pass::{record_draw_ops_to_view, TargetLoad, WgpuUiDrawBufferCache};
    use super::super::text::WgpuUiTextRenderer;
    use super::super::WgpuUiImageResource;
    use super::{retained_copy_byte_count, WgpuRetainedSurfaceCache};

    const TEST_SIZE: (u32, u32) = (4, 4);
    const TEST_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    #[test]
    fn retained_copy_bytes_match_the_supported_byte_surface_formats() {
        let surface_size = (5, 3);

        assert_eq!(
            retained_copy_byte_count(wgpu::TextureFormat::Rgba8Unorm, surface_size),
            60
        );
        assert_eq!(
            retained_copy_byte_count(wgpu::TextureFormat::Bgra8Unorm, surface_size),
            60
        );
    }

    #[test]
    fn retained_cache_damage_patch_copy_preserves_old_and_new_pixels() {
        let Some((device, queue)) = offscreen_test_device() else {
            eprintln!("skipping retained cache copy pixel test: no WGPU adapter is available");
            return;
        };
        let image_layout = create_image_bind_group_layout(&device);
        let cache = WgpuRetainedSurfaceCache::new(&device, TEST_FORMAT, TEST_SIZE);
        let copied_surface = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-retained-cache-copy-test-surface"),
            size: wgpu::Extent3d {
                width: TEST_SIZE.0,
                height: TEST_SIZE.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEST_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let seed_draw_list = UiSurfaceDrawList::new(
            TEST_SIZE,
            None,
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, TEST_SIZE.0 as f32, TEST_SIZE.1 as f32),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [255, 0, 0, 255],
                    corner_radius: 0.0,
                },
            }],
        );
        let seed_draw_plan = batch_draw_plan(&seed_draw_list);
        let mut draw_buffers = WgpuUiDrawBufferCache::default();
        let seed_buffers = draw_buffers
            .resolve(&device, &queue, &seed_draw_list, &seed_draw_plan)
            .buffers;
        let solid_pipeline = create_solid_pipeline(&device, TEST_FORMAT);
        let solid_instance_pipeline = create_solid_instance_pipeline(&device, TEST_FORMAT);
        let image_pipeline = create_image_pipeline(&device, TEST_FORMAT, &image_layout);
        let image_cache = HashMap::<String, WgpuUiImageResource>::new();
        let mut text = WgpuUiTextRenderer::new(&device, &queue, TEST_FORMAT);
        let mut seed_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-ui-retained-cache-copy-test-seed-encoder"),
        });
        let encoded_seed = record_draw_ops_to_view(
            &mut seed_encoder,
            cache.view(),
            TargetLoad::ClearBlack,
            TEST_SIZE,
            TEST_SIZE,
            None,
            &seed_draw_plan.ops,
            &seed_buffers,
            &solid_pipeline,
            &solid_instance_pipeline,
            &image_pipeline,
            &image_cache,
            &mut text,
        );
        // The production presenter submits each frame before the persistent vertex buffer is
        // reused. Keep that boundary here so the seed pass observes its own uploaded vertices.
        queue.submit([seed_encoder.finish()]);

        let damage = UiSurfaceRect::new(0.0, 0.0, 2.0, TEST_SIZE.1 as f32);
        let patch_draw_list = UiSurfaceDrawList::new(
            TEST_SIZE,
            Some(damage),
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: damage,
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [0, 0, 255, 255],
                    corner_radius: 0.0,
                },
            }],
        );
        let patch_draw_plan = batch_draw_plan(&patch_draw_list);
        let patch_buffers = draw_buffers
            .resolve(&device, &queue, &patch_draw_list, &patch_draw_plan)
            .buffers;
        let mut patch_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-ui-retained-cache-copy-test-patch-encoder"),
        });
        let encoded_patch = record_draw_ops_to_view(
            &mut patch_encoder,
            cache.view(),
            TargetLoad::Load,
            TEST_SIZE,
            TEST_SIZE,
            Some(damage),
            &patch_draw_plan.ops,
            &patch_buffers,
            &solid_pipeline,
            &solid_instance_pipeline,
            &image_pipeline,
            &image_cache,
            &mut text,
        );
        let copied_bytes =
            cache.record_copy_to_surface(&mut patch_encoder, &copied_surface, TEST_SIZE);
        queue.submit([patch_encoder.finish()]);

        assert_eq!(encoded_seed.draw_calls, 1);
        assert_eq!(encoded_seed.render_pass_count, 1);
        assert_eq!(encoded_patch.draw_calls, 1);
        assert_eq!(encoded_patch.render_pass_count, 1);
        assert_eq!(copied_bytes, 4 * 4 * 4);
        let pixels = read_texture_rgba(&device, &queue, &copied_surface, TEST_SIZE)
            .expect("retained cache copy texture must support readback");
        assert_damage_patch_pixels(&pixels);
    }

    #[test]
    fn unversioned_damage_bootstrap_full_redraw_keeps_pixels_outside_the_damage_rect() {
        let Some((device, queue)) = offscreen_test_device() else {
            eprintln!("skipping UI full-redraw pixel test: no WGPU adapter is available");
            return;
        };
        let image_layout = create_image_bind_group_layout(&device);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-full-redraw-bootstrap-test-target"),
            size: wgpu::Extent3d {
                width: TEST_SIZE.0,
                height: TEST_SIZE.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEST_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let damage = UiSurfaceRect::new(0.0, 0.0, 2.0, TEST_SIZE.1 as f32);
        let draw_list = UiSurfaceDrawList::new(
            TEST_SIZE,
            Some(damage),
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: damage,
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [255, 0, 0, 255],
                        corner_radius: 0.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(2.0, 0.0, 2.0, TEST_SIZE.1 as f32),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [0, 255, 0, 255],
                        corner_radius: 0.0,
                    },
                },
            ],
        );
        let mut batch_cache = CompiledUiBatchPlanCache::default();
        let full_redraw = batch_cache.resolve(&draw_list, true);
        let mut draw_buffers = WgpuUiDrawBufferCache::default();
        let buffers = draw_buffers
            .resolve(&device, &queue, &draw_list, &full_redraw.plan)
            .buffers;
        let solid_pipeline = create_solid_pipeline(&device, TEST_FORMAT);
        let solid_instance_pipeline = create_solid_instance_pipeline(&device, TEST_FORMAT);
        let image_pipeline = create_image_pipeline(&device, TEST_FORMAT, &image_layout);
        let image_cache = HashMap::<String, WgpuUiImageResource>::new();
        let mut text = WgpuUiTextRenderer::new(&device, &queue, TEST_FORMAT);
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-ui-full-redraw-bootstrap-test-encoder"),
        });
        let encoded = record_draw_ops_to_view(
            &mut encoder,
            &target_view,
            TargetLoad::ClearBlack,
            TEST_SIZE,
            TEST_SIZE,
            None,
            &full_redraw.plan.ops,
            &buffers,
            &solid_pipeline,
            &solid_instance_pipeline,
            &image_pipeline,
            &image_cache,
            &mut text,
        );
        queue.submit([encoder.finish()]);

        assert_eq!(full_redraw.plan.stats.visible_draw_item_count, 2);
        assert_eq!(encoded.draw_calls, 2);
        let pixels = read_texture_rgba(&device, &queue, &target, TEST_SIZE)
            .expect("full-redraw target must support readback");
        for y in 0..TEST_SIZE.1 as usize {
            for x in 0..TEST_SIZE.0 as usize {
                let offset = (y * TEST_SIZE.0 as usize + x) * 4;
                let expected = if x < 2 {
                    [255, 0, 0, 255]
                } else {
                    [0, 255, 0, 255]
                };
                assert_eq!(
                    &pixels[offset..offset + 4],
                    expected.as_slice(),
                    "full redraw must preserve pixel ({x}, {y}) outside a stale damage rect"
                );
            }
        }
    }

    fn assert_damage_patch_pixels(pixels: &[u8]) {
        for y in 0..TEST_SIZE.1 as usize {
            for x in 0..TEST_SIZE.0 as usize {
                let offset = (y * TEST_SIZE.0 as usize + x) * 4;
                let pixel = [
                    pixels[offset],
                    pixels[offset + 1],
                    pixels[offset + 2],
                    pixels[offset + 3],
                ];
                let expected = if x < 2 {
                    [0, 0, 255, 255]
                } else {
                    [255, 0, 0, 255]
                };
                assert_eq!(
                    pixel, expected,
                    "pixel ({x}, {y}) must preserve patch order"
                );
            }
        }
    }

    fn read_texture_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        size: (u32, u32),
    ) -> Result<Vec<u8>, String> {
        let unpadded_bytes_per_row = size.0.saturating_mul(4);
        let padded_bytes_per_row = unpadded_bytes_per_row
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            .saturating_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-ui-retained-cache-readback"),
            size: u64::from(padded_bytes_per_row).saturating_mul(u64::from(size.1)),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-ui-retained-cache-readback-encoder"),
        });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(size.1),
                },
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);

        let slice = buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| error.to_string())?;
        receiver
            .recv()
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;

        let mapped = slice.get_mapped_range();
        let mut rgba = vec![0; size.0.saturating_mul(size.1).saturating_mul(4) as usize];
        for row in 0..size.1 as usize {
            let source_offset = row * padded_bytes_per_row as usize;
            let target_offset = row * unpadded_bytes_per_row as usize;
            rgba[target_offset..target_offset + unpadded_bytes_per_row as usize].copy_from_slice(
                &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
            );
        }
        drop(mapped);
        buffer.unmap();
        Ok(rgba)
    }

    fn offscreen_test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("zircon-ui-retained-cache-copy-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .ok()
    }
}

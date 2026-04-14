use crate::scene::{create_depth_texture, ResourceStreamer, SceneRendererCore, OFFSCREEN_FORMAT};
use crate::types::{EditorOrRuntimeFrame, GraphicsError};
use std::sync::{mpsc, Arc};
use winit::window::Window;
use zircon_asset::ProjectAssetManager;
use zircon_math::{UVec2, Vec2};

pub(crate) struct RenderBackend {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl RenderBackend {
    pub(crate) fn new_offscreen() -> Result<Self, GraphicsError> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|_| GraphicsError::NoAdapter)?;
        let (device, queue) = request_device(&adapter)?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub(crate) fn new_with_surface(
        window: Arc<Window>,
    ) -> Result<(Self, SurfaceState), GraphicsError> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|_| GraphicsError::NoAdapter)?;
        let (device, queue) = request_device(&adapter)?;
        let size = UVec2::new(
            window.inner_size().width.max(1),
            window.inner_size().height.max(1),
        );
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.x,
            height: size.y,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
            },
            SurfaceState {
                surface,
                config,
                size,
                window,
            },
        ))
    }
}

fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), GraphicsError> {
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(GraphicsError::from)
}

pub(crate) struct SurfaceState {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: UVec2,
    #[allow(dead_code)]
    window: Arc<Window>,
}

impl SurfaceState {
    pub(crate) fn resize(&mut self, device: &wgpu::Device, size: UVec2) {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        self.size = size;
        self.config.width = size.x;
        self.config.height = size.y;
        self.surface.configure(device, &self.config);
    }
}

pub(crate) struct OffscreenTarget {
    pub(crate) size: UVec2,
    pub(crate) color: wgpu::Texture,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
}

impl OffscreenTarget {
    pub(crate) fn new(device: &wgpu::Device, size: UVec2) -> Self {
        let color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-offscreen-color"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
        let depth = create_depth_texture(device, size);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            size,
            color,
            color_view,
            depth_view,
        }
    }
}

pub(crate) fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: UVec2,
) -> Result<Vec<u8>, GraphicsError> {
    let bytes_per_pixel = 4_u32;
    let unpadded_bytes_per_row = size.x * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer_size = padded_bytes_per_row as u64 * size.y as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-encoder"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(size.y),
            },
        },
        wgpu::Extent3d {
            width: size.x,
            height: size.y,
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
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;

    let mapped = slice.get_mapped_range();
    let mut rgba = vec![0_u8; (size.x * size.y * 4) as usize];
    for row in 0..size.y as usize {
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

pub struct RuntimePreviewRenderer {
    backend: RenderBackend,
    surface_state: SurfaceState,
    scene_renderer: SceneRendererCore,
    streamer: ResourceStreamer,
}

impl RuntimePreviewRenderer {
    pub fn new(
        window: Arc<Window>,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<Self, GraphicsError> {
        let (backend, surface_state) = RenderBackend::new_with_surface(window)?;
        let scene_renderer = SceneRendererCore::new(&backend.device, surface_state.config.format);
        let streamer = ResourceStreamer::new(
            asset_manager,
            &backend.device,
            &backend.queue,
            &scene_renderer.texture_bind_group_layout,
        );

        Ok(Self {
            backend,
            surface_state,
            scene_renderer,
            streamer,
        })
    }

    pub fn resize(&mut self, size: UVec2) {
        self.surface_state.resize(&self.backend.device, size);
    }

    pub fn render(&mut self, frame: &EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        self.streamer
            .ensure_scene_resources(
                &self.backend.device,
                &self.backend.queue,
                &self.scene_renderer.texture_bind_group_layout,
                frame,
            )?;

        let output = self.surface_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth = create_depth_texture(&self.backend.device, self.surface_state.size);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        self.scene_renderer.render_scene(
            &self.backend.device,
            &self.backend.queue,
            &self.streamer,
            frame,
            &view,
            &depth_view,
        );
        output.present();
        Ok(())
    }

    pub fn viewport_center(&self) -> Vec2 {
        Vec2::new(
            self.surface_state.size.x as f32 * 0.5,
            self.surface_state.size.y as f32 * 0.5,
        )
    }
}

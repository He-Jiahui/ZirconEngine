use crate::backend::{read_texture_rgba, OffscreenTarget, RenderBackend};
use crate::scene::{ResourceStreamer, SceneRendererCore, OFFSCREEN_FORMAT};
use crate::types::{
    EditorOrRuntimeFrame, GraphicsError, ViewportFrame, ViewportFrameTextureHandle,
};
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use zircon_asset::ProjectAssetManager;
use zircon_core::{spawn_named_thread, ZirconError};

impl From<ZirconError> for GraphicsError {
    fn from(value: ZirconError) -> Self {
        Self::ThreadBootstrap(value.to_string())
    }
}

pub struct RenderService {
    command_tx: Sender<RenderThreadCommand>,
    frame_rx: Receiver<ViewportFrame>,
    join: Option<JoinHandle<()>>,
}

impl RenderService {
    pub fn spawn(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        let (command_tx, command_rx) = unbounded();
        let (frame_tx, frame_rx) = unbounded();
        let join = spawn_named_thread("zircon-render-thread", move || {
            render_thread_main(command_rx, frame_tx, asset_manager)
        })?;

        Ok(Self {
            command_tx,
            frame_rx,
            join: Some(join),
        })
    }

    pub fn submit_frame(&self, frame: EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        self.command_tx
            .send(RenderThreadCommand::Frame(frame))
            .map_err(|_| GraphicsError::Channel("render command receiver dropped".to_string()))
    }

    pub fn try_recv_latest_frame(&self) -> Option<ViewportFrame> {
        let mut latest = None;
        while let Ok(frame) = self.frame_rx.try_recv() {
            latest = Some(frame);
        }
        latest
    }
}

pub struct SharedTextureRenderService {
    command_tx: Sender<RenderThreadCommand>,
    frame_rx: Receiver<ViewportFrameTextureHandle>,
    join: Option<JoinHandle<()>>,
}

impl SharedTextureRenderService {
    pub fn spawn_with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<Self, GraphicsError> {
        let (command_tx, command_rx) = unbounded();
        let (frame_tx, frame_rx) = unbounded();
        let join = spawn_named_thread("zircon-shared-render-thread", move || {
            shared_texture_render_thread_main(command_rx, frame_tx, asset_manager, device, queue)
        })?;

        Ok(Self {
            command_tx,
            frame_rx,
            join: Some(join),
        })
    }

    pub fn submit_frame(&self, frame: EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        self.command_tx
            .send(RenderThreadCommand::Frame(frame))
            .map_err(|_| GraphicsError::Channel("render command receiver dropped".to_string()))
    }

    pub fn try_recv_latest_frame(&self) -> Option<ViewportFrameTextureHandle> {
        let mut latest = None;
        while let Ok(frame) = self.frame_rx.try_recv() {
            latest = Some(frame);
        }
        latest
    }
}

fn render_thread_main(
    command_rx: Receiver<RenderThreadCommand>,
    frame_tx: Sender<ViewportFrame>,
    asset_manager: Arc<ProjectAssetManager>,
) {
    let mut backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let mut scene_renderer = SceneRendererCore::new(&backend.device, OFFSCREEN_FORMAT);
    let mut streamer = ResourceStreamer::new(
        asset_manager,
        &backend.device,
        &backend.queue,
        &scene_renderer.texture_bind_group_layout,
    );
    let mut offscreen = OffscreenRenderer::default();
    let mut latest_frame: Option<EditorOrRuntimeFrame> = None;
    let mut dirty = false;
    let mut generation = 0_u64;

    loop {
        select! {
            recv(command_rx) -> message => {
                match message {
                    Ok(RenderThreadCommand::Frame(frame)) => {
                        latest_frame = Some(frame);
                        dirty = true;
                    }
                    Ok(RenderThreadCommand::Shutdown) | Err(_) => break,
                }
            }
            default(Duration::from_millis(16)) => {}
        }

        if let Some(frame) = latest_frame.as_ref() {
            if dirty {
                streamer
                    .ensure_scene_resources(
                        &backend.device,
                        &backend.queue,
                        &scene_renderer.texture_bind_group_layout,
                        frame,
                    )
                    .expect("scene resources");
                generation += 1;
                let rendered = offscreen
                    .render(
                        &mut backend,
                        &mut scene_renderer,
                        &streamer,
                        frame,
                        generation,
                    )
                    .expect("offscreen render");
                let _ = frame_tx.send(rendered);
                dirty = false;
            }
        }
    }
}

fn shared_texture_render_thread_main(
    command_rx: Receiver<RenderThreadCommand>,
    frame_tx: Sender<ViewportFrameTextureHandle>,
    asset_manager: Arc<ProjectAssetManager>,
    device: wgpu::Device,
    queue: wgpu::Queue,
) {
    let mut scene_renderer = SceneRendererCore::new(&device, OFFSCREEN_FORMAT);
    let mut streamer = ResourceStreamer::new(
        asset_manager,
        &device,
        &queue,
        &scene_renderer.texture_bind_group_layout,
    );
    let mut offscreen = SharedTextureOffscreenRenderer::default();
    let mut latest_frame: Option<EditorOrRuntimeFrame> = None;
    let mut dirty = false;
    let mut generation = 0_u64;

    loop {
        select! {
            recv(command_rx) -> message => {
                match message {
                    Ok(RenderThreadCommand::Frame(frame)) => {
                        latest_frame = Some(frame);
                        dirty = true;
                    }
                    Ok(RenderThreadCommand::Shutdown) | Err(_) => break,
                }
            }
            default(Duration::from_millis(16)) => {}
        }

        if let Some(frame) = latest_frame.as_ref() {
            if dirty {
                streamer
                    .ensure_scene_resources(
                        &device,
                        &queue,
                        &scene_renderer.texture_bind_group_layout,
                        frame,
                    )
                    .expect("scene resources");
                generation += 1;
                let rendered = offscreen.render(
                    &device,
                    &queue,
                    &mut scene_renderer,
                    &streamer,
                    frame,
                    generation,
                );
                let _ = frame_tx.send(rendered);
                dirty = false;
            }
        }
    }
}

enum RenderThreadCommand {
    Frame(EditorOrRuntimeFrame),
    Shutdown,
}

impl Drop for RenderService {
    fn drop(&mut self) {
        let _ = self.command_tx.send(RenderThreadCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for SharedTextureRenderService {
    fn drop(&mut self) {
        let _ = self.command_tx.send(RenderThreadCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[derive(Default)]
struct OffscreenRenderer {
    target: Option<OffscreenTarget>,
}

impl OffscreenRenderer {
    fn render(
        &mut self,
        backend: &mut RenderBackend,
        scene_renderer: &mut SceneRendererCore,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        generation: u64,
    ) -> Result<ViewportFrame, GraphicsError> {
        let size =
            zircon_math::UVec2::new(frame.viewport.size.x.max(1), frame.viewport.size.y.max(1));
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.size != size)
        {
            self.target = Some(OffscreenTarget::new(&backend.device, size));
        }
        let target = self.target.as_ref().unwrap();

        scene_renderer.render_scene(
            &backend.device,
            &backend.queue,
            streamer,
            frame,
            &target.color_view,
            &target.depth_view,
        );
        let rgba = read_texture_rgba(&backend.device, &backend.queue, &target.color, target.size)?;

        Ok(ViewportFrame {
            width: target.size.x,
            height: target.size.y,
            rgba,
            generation,
        })
    }
}

#[derive(Default)]
struct SharedTextureOffscreenRenderer {
    target: Option<OffscreenTarget>,
}

impl SharedTextureOffscreenRenderer {
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_renderer: &mut SceneRendererCore,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        generation: u64,
    ) -> ViewportFrameTextureHandle {
        let size =
            zircon_math::UVec2::new(frame.viewport.size.x.max(1), frame.viewport.size.y.max(1));
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.size != size)
        {
            self.target = Some(OffscreenTarget::new(device, size));
        }
        let target = self.target.as_ref().unwrap();

        scene_renderer.render_scene(
            device,
            queue,
            streamer,
            frame,
            &target.color_view,
            &target.depth_view,
        );

        ViewportFrameTextureHandle {
            width: target.size.x,
            height: target.size.y,
            texture: target.color.clone(),
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            generation,
        }
    }
}

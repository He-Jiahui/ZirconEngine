use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender};
use zircon_asset::ProjectAssetManager;

use crate::scene::{ResourceStreamer, SceneRendererCore, ViewportIconSource, OFFSCREEN_FORMAT};
use crate::types::{EditorOrRuntimeFrame, ViewportFrameTextureHandle};

use super::{
    render_thread_command::RenderThreadCommand,
    shared_texture_offscreen_renderer::SharedTextureOffscreenRenderer,
};

pub(in crate::service) fn shared_texture_render_thread_main(
    command_rx: Receiver<RenderThreadCommand>,
    frame_tx: Sender<ViewportFrameTextureHandle>,
    asset_manager: Arc<ProjectAssetManager>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    icon_source: Option<Arc<dyn ViewportIconSource>>,
) {
    let mut scene_renderer = if let Some(icon_source) = icon_source {
        SceneRendererCore::new_with_icon_source(&device, &queue, OFFSCREEN_FORMAT, icon_source)
    } else {
        SceneRendererCore::new(&device, &queue, OFFSCREEN_FORMAT)
    };
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
                let rendered = offscreen
                    .render(
                        &device,
                        &queue,
                        &mut scene_renderer,
                        &streamer,
                        frame,
                        generation,
                    )
                    .expect("shared texture render");
                let _ = frame_tx.send(rendered);
                dirty = false;
            }
        }
    }
}

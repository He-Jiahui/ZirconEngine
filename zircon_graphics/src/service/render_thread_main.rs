use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender};
use zircon_asset::ProjectAssetManager;

use crate::backend::RenderBackend;
use crate::scene::{ResourceStreamer, SceneRendererCore, ViewportIconSource, OFFSCREEN_FORMAT};
use crate::types::{EditorOrRuntimeFrame, ViewportFrame};

use super::{offscreen_renderer::OffscreenRenderer, render_thread_command::RenderThreadCommand};

pub(in crate::service) fn render_thread_main(
    command_rx: Receiver<RenderThreadCommand>,
    frame_tx: Sender<ViewportFrame>,
    asset_manager: Arc<ProjectAssetManager>,
    icon_source: Option<Arc<dyn ViewportIconSource>>,
) {
    let mut backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let mut scene_renderer = if let Some(icon_source) = icon_source {
        SceneRendererCore::new_with_icon_source(
            &backend.device,
            &backend.queue,
            OFFSCREEN_FORMAT,
            icon_source,
        )
    } else {
        SceneRendererCore::new(&backend.device, &backend.queue, OFFSCREEN_FORMAT)
    };
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

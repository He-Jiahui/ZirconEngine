use std::sync::Arc;

use crossbeam_channel::unbounded;
use zircon_asset::ProjectAssetManager;
use zircon_core::spawn_named_thread;

use crate::scene::ViewportIconSource;
use crate::types::GraphicsError;

use super::{
    shared_texture_render_service::SharedTextureRenderService,
    shared_texture_render_thread_main::shared_texture_render_thread_main,
};

impl SharedTextureRenderService {
    pub fn spawn_with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<Self, GraphicsError> {
        Self::spawn_with_device_and_icon_source(device, queue, asset_manager, None)
    }

    pub fn spawn_with_device_and_icon_source(
        device: wgpu::Device,
        queue: wgpu::Queue,
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Option<Arc<dyn ViewportIconSource>>,
    ) -> Result<Self, GraphicsError> {
        let (command_tx, command_rx) = unbounded();
        let (frame_tx, frame_rx) = unbounded();
        let join = spawn_named_thread("zircon-shared-render-thread", move || {
            shared_texture_render_thread_main(
                command_rx,
                frame_tx,
                asset_manager,
                device,
                queue,
                icon_source,
            )
        })?;

        Ok(Self {
            command_tx,
            frame_rx,
            join: Some(join),
        })
    }
}

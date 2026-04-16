use std::sync::Arc;

use crossbeam_channel::unbounded;
use zircon_asset::ProjectAssetManager;
use zircon_core::spawn_named_thread;

use crate::scene::ViewportIconSource;
use crate::types::GraphicsError;

use super::{render_service::RenderService, render_thread_main::render_thread_main};

impl RenderService {
    pub fn spawn(asset_manager: Arc<ProjectAssetManager>) -> Result<Self, GraphicsError> {
        Self::spawn_with_icon_source(asset_manager, None)
    }

    pub fn spawn_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Option<Arc<dyn ViewportIconSource>>,
    ) -> Result<Self, GraphicsError> {
        let (command_tx, command_rx) = unbounded();
        let (frame_tx, frame_rx) = unbounded();
        let join = spawn_named_thread("zircon-render-thread", move || {
            render_thread_main(command_rx, frame_tx, asset_manager, icon_source)
        })?;

        Ok(Self {
            command_tx,
            frame_rx,
            join: Some(join),
        })
    }
}

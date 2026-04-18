use std::sync::{Arc, Mutex, RwLock};

use zircon_core::ChannelSender;

use super::super::EditorAssetChangeRecord;

mod asset_details;
mod broadcast;
mod catalog_snapshot;
mod editor_asset_error;
mod editor_asset_state;
mod open_project;
mod parse_uuid;
mod preview_trait_bridge;
mod record_access;
mod record_to_facade;
mod reference_to_facade;
mod subscribe_editor_asset_changes;

pub(super) use editor_asset_state::EditorAssetState;

#[derive(Clone, Debug, Default)]
pub struct DefaultEditorAssetManager {
    pub(super) state: Arc<RwLock<EditorAssetState>>,
    change_subscribers: Arc<Mutex<Vec<ChannelSender<EditorAssetChangeRecord>>>>,
}

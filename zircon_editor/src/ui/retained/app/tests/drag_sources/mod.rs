use super::*;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetDetailsGeneration, EditorAssetDetailsRecord, EditorAssetFolderRecord,
    EditorAssetReferenceRecord,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetReferenceSnapshot, AssetTypeProjectionSnapshot, AssetWorkspaceSnapshot,
};
use std::sync::Arc;
use zircon_runtime::asset::project::{AssetSourceUnit, PreviewState};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

#[path = "asset_browser.rs"]
mod asset_browser;
#[path = "../../../../app/tests/drag_sources/asset_metadata_and_fields.rs"]
mod asset_metadata_and_fields;
#[path = "hierarchy_gesture.rs"]
mod hierarchy_gesture;
#[path = "hierarchy_identity.rs"]
mod hierarchy_identity;
#[path = "scene_and_object.rs"]
mod scene_and_object;
#[path = "support.rs"]
mod support;

use support::*;

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

mod asset_browser;
mod asset_metadata_and_fields;
mod scene_and_object;
mod support;

use support::*;

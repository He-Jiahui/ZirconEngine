use crossbeam_channel::RecvTimeoutError;
use std::fs;
use std::time::{Duration, Instant};

use crate::core::framework::asset::ResourceManager;
use crate::core::resource::{ResourceEventKind, ResourceKind, ResourceState, RuntimeResourceState};

use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    write_checker_png, write_default_material, write_default_scene, write_triangle_obj,
    write_valid_wgsl,
};
use crate::asset::watch::AssetChangeKind;
use crate::asset::{
    AssetImporterCapabilityStatus, AssetManager, AssetUri, MaterialAsset, ProjectAssetManager,
};

mod model_import;
mod project_open;
mod resource_records;
mod resource_revisions;
mod runtime_leases;
mod service_capabilities;
mod watcher;

fn project_asset_manager_with_first_wave_plugin_fixtures() -> ProjectAssetManager {
    let manager = ProjectAssetManager::default();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
}

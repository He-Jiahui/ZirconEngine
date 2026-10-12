//! 这些集成测试从项目目录进入 ProjectAssetManager，再观察资源记录、运行时驻留和变更发布。

use crossbeam_channel::RecvTimeoutError;
use std::fs;
use std::time::{Duration, Instant};

use crate::core::framework::asset::ResourceManager;
use crate::core::resource::{ResourceEventKind, ResourceKind, ResourceState, RuntimeResourceState};

use crate::asset::project::{AssetMetaDocument, ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    read_project_material, write_checker_png, write_default_material, write_default_scene,
    write_project_material, write_triangle_obj, write_valid_wgsl,
};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetImporterCapabilityStatus, AssetManager, AssetUri, ProjectAssetManager};

#[path = "manager/deletion.rs"]
mod deletion;
#[path = "manager/lazy_residency.rs"]
mod lazy_residency;
#[path = "manager/model_import.rs"]
mod model_import;
#[path = "manager/project_open.rs"]
mod project_open;
#[path = "manager/resource_records.rs"]
mod resource_records;
#[path = "manager/resource_revisions.rs"]
mod resource_revisions;
#[path = "manager/runtime_leases.rs"]
mod runtime_leases;
#[path = "manager/service_capabilities.rs"]
mod service_capabilities;
#[path = "manager/watcher.rs"]
mod watcher;

fn project_asset_manager_with_first_wave_plugin_fixtures() -> ProjectAssetManager {
    let manager = ProjectAssetManager::default();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
}

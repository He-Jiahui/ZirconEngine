use std::sync::Arc;

use crate::core::framework::asset::{
    ResourceCacheIdentity, ResourceManager as ResourceManagerContract,
};
use crate::core::resource::{ResourceEventReceiver, ResourceManagementGeneration, ResourceRecord};

use super::super::project_asset_manager::ProjectAssetManager;
use crate::asset::AssetUri;

impl ResourceManagerContract for ProjectAssetManager {
    fn resolve_resource_id(&self, locator: &str) -> Option<String> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.id().to_string())
    }

    fn resource_status(&self, locator: &str) -> Option<ResourceRecord> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .cloned()
    }

    fn resource_management_generation(&self) -> Arc<ResourceManagementGeneration> {
        self.resource_manager().management_generation()
    }

    fn resource_revision(&self, locator: &str) -> Option<u64> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.revision)
    }

    fn resource_cache_identity(&self, locator: &str) -> Option<ResourceCacheIdentity> {
        let locator = AssetUri::parse(locator).ok()?;
        let resources = self.resource_manager();
        let identity =
            resources
                .registry()
                .get_by_locator(&locator)
                .map(|record| ResourceCacheIdentity {
                    revision: record.revision,
                    state: record.state,
                });
        identity
    }

    fn subscribe_resource_changes(&self) -> ResourceEventReceiver {
        self.resource_manager().subscribe()
    }
}

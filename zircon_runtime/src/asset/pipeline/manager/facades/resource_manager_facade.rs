use crate::core::framework::asset::ResourceManager as ResourceManagerFacade;
use crate::core::resource::{ResourceEvent, ResourceRecord};
use crate::core::ChannelReceiver;

use super::super::project_asset_manager::ProjectAssetManager;
use crate::asset::AssetUri;

impl ResourceManagerFacade for ProjectAssetManager {
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

    fn list_resources(&self) -> Vec<ResourceRecord> {
        let mut resources = self
            .resource_manager()
            .registry()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        resources.sort_by_key(|record| record.primary_locator.to_string());
        resources
    }

    fn resource_revision(&self, locator: &str) -> Option<u64> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.revision)
    }

    fn subscribe_resource_changes(&self) -> ChannelReceiver<ResourceEvent> {
        self.resource_manager().subscribe()
    }
}

use crossbeam_channel::unbounded;
use zircon_core::ChannelReceiver;
use zircon_manager::{
    ResourceChangeRecord, ResourceManager as ResourceManagerFacade, ResourceStatusRecord,
};

use super::records::{resource_change_record, resource_status_record};
use super::ProjectAssetManager;
use crate::AssetUri;

impl ResourceManagerFacade for ProjectAssetManager {
    fn resolve_resource_id(&self, locator: &str) -> Option<String> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.id().to_string())
    }

    fn resource_status(&self, locator: &str) -> Option<ResourceStatusRecord> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(resource_status_record)
    }

    fn list_resources(&self) -> Vec<ResourceStatusRecord> {
        let mut resources = self
            .resource_manager()
            .registry()
            .values()
            .map(resource_status_record)
            .collect::<Vec<_>>();
        resources.sort_by(|left, right| left.locator.cmp(&right.locator));
        resources
    }

    fn resource_revision(&self, locator: &str) -> Option<u64> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.revision)
    }

    fn subscribe_resource_changes(&self) -> ChannelReceiver<ResourceChangeRecord> {
        let source = self.resource_manager().subscribe();
        let (sender, receiver) = unbounded();
        std::thread::Builder::new()
            .name("zircon-resource-event-bridge".to_string())
            .spawn(move || {
                while let Ok(event) = source.recv() {
                    if sender.send(resource_change_record(event)).is_err() {
                        break;
                    }
                }
            })
            .expect("resource event bridge thread");
        receiver
    }
}

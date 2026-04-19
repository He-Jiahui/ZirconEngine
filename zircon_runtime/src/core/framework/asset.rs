use crate::core::ChannelReceiver;
use crate::core::resource::{ResourceEvent, ResourceRecord};

pub trait ResourceManager: Send + Sync {
    fn resolve_resource_id(&self, locator: &str) -> Option<String>;
    fn resource_status(&self, locator: &str) -> Option<ResourceRecord>;
    fn list_resources(&self) -> Vec<ResourceRecord>;
    fn resource_revision(&self, locator: &str) -> Option<u64>;
    fn subscribe_resource_changes(&self) -> ChannelReceiver<ResourceEvent>;
}

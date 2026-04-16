use super::host_registry::HostRegistry;

#[derive(Debug, Default)]
pub struct PluginHostDriver {
    registry: HostRegistry,
}

impl PluginHostDriver {
    pub fn registry(&self) -> HostRegistry {
        self.registry.clone()
    }
}

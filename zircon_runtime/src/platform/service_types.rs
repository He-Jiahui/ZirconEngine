#[derive(Clone, Debug, Default)]
pub struct PlatformDriver;

#[derive(Clone, Debug, Default)]
pub struct PlatformManager;

impl PlatformManager {
    pub fn capability_report(
        &self,
        config: &super::PlatformConfig,
    ) -> super::PlatformCapabilityReport {
        config.capability_report()
    }
}

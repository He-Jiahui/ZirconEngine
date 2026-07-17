use zircon_runtime::core::framework::sound::{SoundBackendCapability, SoundOutputDeviceInfo};

use crate::kira_bridge::{available_backends, available_devices};
use crate::output::SoundOutputDeviceRuntimeState;
use crate::SoundConfig;

pub(super) struct KiraCatalogFixture {
    pub(super) config: SoundConfig,
    pub(super) backends: Vec<SoundBackendCapability>,
    pub(super) devices: Vec<SoundOutputDeviceInfo>,
    pub(super) output: SoundOutputDeviceRuntimeState,
}

pub(super) fn kira_catalog_fixture() -> KiraCatalogFixture {
    let config = SoundConfig::default();
    KiraCatalogFixture {
        backends: available_backends(),
        devices: available_devices(&config),
        output: SoundOutputDeviceRuntimeState::new(&config),
        config,
    }
}

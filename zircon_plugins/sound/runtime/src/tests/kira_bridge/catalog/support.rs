// 汇集能力声明、CPAL 设备枚举和输出状态夹具，供目录用例读取；不创建 Kira 播放后端。
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

//! 经理对象通过模块注册表提供中立 SoundManager 接口；弱引用只用于按需取资产与运行时钟，避免与 Core 的服务注册表形成引用环。
use std::sync::{Arc, Mutex};

use zircon_runtime::core::{CoreHandle, CoreWeak};

use crate::engine::SoundEngineState;
use crate::poison_recovery::lock_recover;
use crate::SoundConfig;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

/// 声音服务的共享实例；克隆只共享配置与运行状态，装载项目素材还要求关联的 Core 仍存活。
#[derive(Clone, Debug)]
pub struct DefaultSoundManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    pub(super) core: Option<CoreWeak>,
    pub(super) config: Arc<Mutex<SoundConfig>>,
    pub(super) state: Arc<Mutex<SoundEngineState>>,
}

impl Default for DefaultSoundManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultSoundManager {
    pub fn new(core: Option<&CoreHandle>) -> Self {
        Self::with_config(core, SoundConfig::default())
    }

    /// 构造停机服务并生成默认图及设备描述；需要播放时由调用者启动输出，构造不会自动套用预设或激活后端。
    pub fn with_config(core: Option<&CoreHandle>, config: SoundConfig) -> Self {
        Self::with_weak_core(core.map(CoreHandle::downgrade), config)
    }

    pub(crate) fn from_weak_core(core: &CoreWeak) -> Self {
        Self::with_weak_core(Some(core.clone()), SoundConfig::default())
    }

    fn with_weak_core(core: Option<CoreWeak>, config: SoundConfig) -> Self {
        Self {
            core,
            config: Arc::new(Mutex::new(config.clone())),
            state: Arc::new(Mutex::new(SoundEngineState::new(&config))),
        }
    }

    pub(super) fn config(&self) -> SoundConfig {
        lock_recover(&self.config).clone()
    }

    #[cfg(test)]
    pub(crate) fn poison_state_for_test(&self) {
        let state = Arc::clone(&self.state);
        let _ = std::panic::catch_unwind(move || {
            let _guard = lock_recover(&state);
            panic!("poison sound state for recovery coverage");
        });
    }
}

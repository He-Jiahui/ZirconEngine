use std::sync::{Arc, Mutex};

use zircon_runtime::core::CoreHandle;

use crate::engine::SoundEngineState;
use crate::SoundConfig;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

#[derive(Clone, Debug)]
pub struct DefaultSoundManager {
    pub(super) core: Option<CoreHandle>,
    pub(super) config: Arc<Mutex<SoundConfig>>,
    pub(super) state: Arc<Mutex<SoundEngineState>>,
}

impl Default for DefaultSoundManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultSoundManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        Self::with_config(core, SoundConfig::default())
    }

    pub fn with_config(core: Option<CoreHandle>, config: SoundConfig) -> Self {
        Self {
            core,
            config: Arc::new(Mutex::new(config.clone())),
            state: Arc::new(Mutex::new(SoundEngineState::new(&config))),
        }
    }

    pub(super) fn config(&self) -> SoundConfig {
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .clone()
    }
}

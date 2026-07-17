use std::sync::{Arc, Mutex};

use zircon_runtime::core::{CoreHandle, CoreWeak};

use crate::engine::SoundEngineState;
use crate::poison_recovery::lock_recover;
use crate::SoundConfig;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

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

    pub fn with_config(core: Option<&CoreHandle>, config: SoundConfig) -> Self {
        Self {
            core: core.map(CoreHandle::downgrade),
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

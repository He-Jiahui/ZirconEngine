mod acoustics;
mod automation_timeline;
mod dynamic_events;
mod external_sources;
mod manager_trait;
mod mixer_graph;
mod output_device;
mod playback;
mod playback_status;
mod playback_validation;
mod runtime_settings;
mod source_status;
mod sources;

use std::sync::{Arc, Mutex};

use zircon_runtime::core::CoreHandle;

use super::engine::SoundEngineState;
use super::SoundConfig;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

#[derive(Clone, Debug)]
pub struct DefaultSoundManager {
    core: Option<CoreHandle>,
    config: Arc<Mutex<SoundConfig>>,
    state: Arc<Mutex<SoundEngineState>>,
}

impl Default for DefaultSoundManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultSoundManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        let config = SoundConfig::default();
        Self {
            core,
            config: Arc::new(Mutex::new(config.clone())),
            state: Arc::new(Mutex::new(SoundEngineState::new(&config))),
        }
    }

    fn config(&self) -> SoundConfig {
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .clone()
    }
}

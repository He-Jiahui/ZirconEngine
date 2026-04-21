mod config;
mod module;
mod service_types;

pub use config::SoundConfig;
pub use module::{
    module_descriptor, SoundModule, SOUND_DRIVER_NAME, SOUND_MANAGER_NAME, SOUND_MODULE_NAME,
};
pub use service_types::{DefaultSoundManager, SoundDriver};

#[cfg(test)]
mod tests;

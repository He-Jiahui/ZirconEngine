mod acoustics;
mod automation_timeline;
mod backend;
mod dynamic_events;
mod mixer_graph;
mod output_device;
mod playback;
mod render;
mod runtime_settings;
mod source;

use super::DefaultSoundManager;

impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager {}

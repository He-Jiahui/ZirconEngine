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

pub use acoustics::SoundAcousticsManager;
pub use automation_timeline::SoundAutomationTimelineManager;
pub use backend::SoundBackendManager;
pub use dynamic_events::SoundDynamicEventManager;
pub use mixer_graph::SoundMixerGraphManager;
pub use output_device::SoundOutputDeviceManager;
pub use playback::SoundPlaybackManager;
pub use render::SoundMixRenderManager;
pub use runtime_settings::SoundRuntimeSettingsManager;
pub use source::SoundSourceManager;

pub trait SoundManager:
    SoundBackendManager
    + SoundOutputDeviceManager
    + SoundRuntimeSettingsManager
    + SoundPlaybackManager
    + SoundMixerGraphManager
    + SoundSourceManager
    + SoundAutomationTimelineManager
    + SoundDynamicEventManager
    + SoundAcousticsManager
    + SoundMixRenderManager
    + Send
    + Sync
{
}

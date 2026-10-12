#[path = "manager/acoustics.rs"]
mod acoustics;
#[path = "manager/automation_timeline.rs"]
mod automation_timeline;
#[path = "manager/backend.rs"]
mod backend;
#[path = "manager/dynamic_events.rs"]
mod dynamic_events;
#[path = "manager/mixer_graph.rs"]
mod mixer_graph;
#[path = "manager/output_device.rs"]
mod output_device;
#[path = "manager/playback.rs"]
mod playback;
#[path = "manager/render.rs"]
mod render;
#[path = "manager/runtime_settings.rs"]
mod runtime_settings;
#[path = "manager/source.rs"]
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

/// 运行时声音服务的聚合契约；插件实现各子接口，宿主通过统一管理器句柄解析能力。
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

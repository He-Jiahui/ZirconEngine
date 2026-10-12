//! 这里组合框架的各声音子接口；外部经注册的 SoundManager 调用，实际状态与后端操作由同目录服务实现承担。
#[path = "manager_trait/acoustics.rs"]
mod acoustics;
#[path = "../manager_trait_automation_timeline.rs"]
mod automation_timeline;
#[path = "manager_trait/backend.rs"]
mod backend;
#[path = "manager_trait/dynamic_events.rs"]
mod dynamic_events;
#[path = "manager_trait/mixer_graph.rs"]
mod mixer_graph;
#[path = "manager_trait/output_device.rs"]
mod output_device;
#[path = "manager_trait/playback.rs"]
mod playback;
#[path = "manager_trait/render.rs"]
mod render;
#[path = "manager_trait/runtime_settings.rs"]
mod runtime_settings;
#[path = "manager_trait/source.rs"]
mod source;

use super::DefaultSoundManager;

impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager {}

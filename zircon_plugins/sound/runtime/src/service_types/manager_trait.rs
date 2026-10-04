//! 这里组合框架的各声音子接口；外部经注册的 SoundManager 调用，实际状态与后端操作由同目录服务实现承担。
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

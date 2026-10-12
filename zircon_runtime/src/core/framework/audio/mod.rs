//! 提供后端无关的音频格式元数据，供资产系统和可选声音服务共享声道契约。
//!
//! 本模块导出布局与位置类型；设备打开、混音调度和播放资源的生命周期由声音服务承担。
//! 使用方在接收外部布局时检查完整契约，并在具体后端边界另行检查支持能力。
//! 参见 [`AudioChannelLayout`] 与 [`AudioSpeakerChannel`]。
//! Backend-neutral audio format data shared by assets and optional sound services.

#[path = "channel_layout.rs"]
mod channel_layout;

pub use channel_layout::{AudioChannelLayout, AudioSpeakerChannel};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

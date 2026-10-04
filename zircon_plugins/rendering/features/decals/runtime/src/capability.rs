//! 贴花的编辑器与运行时能力键；这些键用于清单选择，不代表 GPU 已支持或已执行该效果。
pub const EDITOR_CAPABILITY: &str = "editor.feature.rendering.decals";
pub const RUNTIME_CAPABILITY: &str = "runtime.feature.rendering.decals";

pub const RUNTIME_CAPABILITIES: &[&str] = &[RUNTIME_CAPABILITY];

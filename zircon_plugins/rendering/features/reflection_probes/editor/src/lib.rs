//! 反射探针的编辑器公共入口；宿主可查询能力与清单，具体编辑器扩展仍由注册契约决定。
#[path = "capability.rs"]
mod capability;
#[path = "capture/mod.rs"]
mod capture;
#[path = "plugin.rs"]
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use capture::{
    ReflectionProbeCaptureEditorCommand, ReflectionProbeCaptureEditorCommandError,
    ReflectionProbeCaptureEditorExecutionError, ReflectionProbeCaptureEditorResult,
    ReflectionProbeCaptureEditorTrigger, ReflectionProbeCaptureProjectPublicationError,
    publish_reflection_probe_capture_source,
};
pub use plugin::{
    RenderingReflectionProbesEditorFeature, editor_capabilities, editor_feature, feature_manifest,
};

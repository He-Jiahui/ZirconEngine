//! 反射探针的运行时公共契约；特性提供者将此处元数据提交到目录与图编译。
//! 反射探针运行时公共面；默认启用只提供提取声明，捕获由显式框架请求和资源消费接口完成。
use zircon_runtime::graphics::RenderFeatureDescriptor;

mod capability;
mod capture;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use capture::{
    CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError,
    CapturedReflectionProbeInfluence, CapturedReflectionProbePlacement,
    EncodedReflectionProbeCaptureSource, PersistedReflectionProbeCapture,
    REFLECTION_PROBE_CAPTURE_FACE_VIEWS, REFLECTION_PROBE_CAPTURE_REQUEST_SCHEMA_VERSION,
    ReflectionProbeCaptureError, ReflectionProbeCaptureFace, ReflectionProbeCaptureFaceView,
    ReflectionProbeCaptureQuality, ReflectionProbeCaptureRequest,
    ReflectionProbeCaptureRequestError, ReflectionProbeCaptureStorageTransform,
    cancel_reflection_probe_capture, encode_reflection_probe_capture_source,
    poll_reflection_probe_capture, register_captured_reflection_probe,
    register_captured_reflection_probe_from_runtime_cache, request_reflection_probe_capture,
    request_reflection_probe_capture_with_placement, take_reflection_probe_capture_source,
};
pub use plugin::{
    RenderingReflectionProbesRuntimeFeature, feature_manifest, plugin_feature_registration,
    runtime_plugin_feature,
};

pub const FEATURE_ID: &str = "rendering.reflection_probes";
pub const FEATURE_NAME: &str = "reflection_probes";
/// 声明探针需要的提取域，避免默认启用时自动发起捕获；捕获由框架句柄请求独立驱动。
/// 仅登记探针相关提取域；捕获不会随普通帧或特性注册自动发生。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        Vec::new(),
    )
}

// 此测试边界覆盖声明与注册约束；GPU 效果证据需由对应产品测试另行提供。
#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;

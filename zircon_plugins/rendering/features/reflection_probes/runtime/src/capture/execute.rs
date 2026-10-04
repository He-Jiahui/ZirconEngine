//! 把反射探针捕获请求转为 RenderFramework 的异步句柄；本模块不持有场景渲染器或资源缓存。
use thiserror::Error;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureSourcePayload,
    RenderEnvironmentCaptureStatus, RenderFramework, RenderFrameworkError, RenderSceneSnapshot,
};
use zircon_runtime::core::resource::ResourceId;

use super::{
    CapturedReflectionProbeConsumeError, CapturedReflectionProbePlacement,
    ReflectionProbeCaptureRequest, ReflectionProbeCaptureRequestError,
};

/// 提交没有 placement 的独立捕获，调用方用返回句柄向同一框架轮询、取消或领取结果。
pub fn request_reflection_probe_capture(
    framework: &dyn RenderFramework,
    scene: &RenderSceneSnapshot,
    request: &ReflectionProbeCaptureRequest,
) -> Result<RenderEnvironmentCaptureHandle, ReflectionProbeCaptureError> {
    let render_request = request.render_request()?;
    framework
        .request_environment_capture(scene.clone(), render_request)
        .map_err(ReflectionProbeCaptureError::Framework)
}

/// 为指定探针目标附加已验证 placement 和纹理身份；目标 URI 必须可由资源系统解析。
pub fn request_reflection_probe_capture_with_placement(
    framework: &dyn RenderFramework,
    scene: &RenderSceneSnapshot,
    request: &ReflectionProbeCaptureRequest,
    placement: &CapturedReflectionProbePlacement,
) -> Result<RenderEnvironmentCaptureHandle, ReflectionProbeCaptureError> {
    request.validate()?;
    placement.validate()?;
    let pmrem_uri = AssetUri::parse(&placement.pmrem_uri)
        .map_err(|error| ReflectionProbeCaptureError::TargetResourceUri(error.to_string()))?;
    let cubemap = ResourceId::from_locator(&pmrem_uri);
    let render_request = request
        .render_request()?
        .with_reflection_probe_target(placement.probe_id, cubemap);
    framework
        .request_environment_capture(scene.clone(), render_request)
        .map_err(ReflectionProbeCaptureError::Framework)
}

/// 查询该句柄的非阻塞状态；句柄属于创建它的框架，不能跨独立框架复用。
pub fn poll_reflection_probe_capture(
    framework: &dyn RenderFramework,
    handle: RenderEnvironmentCaptureHandle,
) -> Result<RenderEnvironmentCaptureStatus, ReflectionProbeCaptureError> {
    framework
        .poll_environment_capture(handle)
        .map_err(ReflectionProbeCaptureError::Framework)
}

/// 向原框架请求取消未完成捕获；已发生的异步工作是否完成由框架状态决定。
pub fn cancel_reflection_probe_capture(
    framework: &dyn RenderFramework,
    handle: RenderEnvironmentCaptureHandle,
) -> Result<(), ReflectionProbeCaptureError> {
    framework
        .cancel_environment_capture(handle)
        .map_err(ReflectionProbeCaptureError::Framework)
}

/// 从原框架取走已完成源 payload；后续编码及项目持久化由消费方完成。
pub fn take_reflection_probe_capture_source(
    framework: &dyn RenderFramework,
    handle: RenderEnvironmentCaptureHandle,
) -> Result<Option<RenderEnvironmentCaptureSourcePayload>, ReflectionProbeCaptureError> {
    framework
        .take_environment_capture_source_payload(handle)
        .map_err(ReflectionProbeCaptureError::Framework)
}

/// 请求验证、目标 URI 和框架执行错误在此边界统一向编辑器命令传递。
#[derive(Debug, Error)]
pub enum ReflectionProbeCaptureError {
    #[error(transparent)]
    InvalidRequest(#[from] ReflectionProbeCaptureRequestError),
    #[error(transparent)]
    Framework(#[from] RenderFrameworkError),
    #[error(transparent)]
    Placement(#[from] CapturedReflectionProbeConsumeError),
    #[error("invalid captured reflection-probe target resource URI: {0}")]
    TargetResourceUri(String),
}

#[cfg(test)]
#[path = "tests/execute.rs"]
mod tests;

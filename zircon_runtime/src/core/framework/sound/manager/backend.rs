use super::super::SoundBackendStatus;

/// 向宿主报告请求后端与实际后端的可用状态，供启动诊断和能力选择使用。
pub trait SoundBackendManager {
    fn backend_name(&self) -> String;
    fn backend_status(&self) -> SoundBackendStatus;
}

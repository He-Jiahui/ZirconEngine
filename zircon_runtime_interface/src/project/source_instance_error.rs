use thiserror::Error;

/// Rejects the reserved nil launch-process identity.
/// 启动实例的 nil UUID 在构造和反序列化入口都会被拒绝。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProjectLaunchInstanceIdError {
    #[error("project launch instance id must not be nil")]
    Nil,
}

use thiserror::Error;

/// Rejects the reserved zero operation sequence.
/// 反序列化的零序号映射为此错误；`new(0)` 则以 `None` 拒绝保留值。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProjectActivationOperationSequenceError {
    #[error("project activation operation sequence must not be zero")]
    Zero,
}

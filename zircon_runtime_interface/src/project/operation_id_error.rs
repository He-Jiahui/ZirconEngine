use thiserror::Error;

/// Rejects an activation operation identity whose nonce cannot distinguish it from an invalid wire value.
/// 操作身份的 nil nonce 在构造和反序列化入口都会被拒绝。
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProjectActivationOperationIdError {
    #[error("project activation operation nonce must not be nil")]
    NilNonce,
}

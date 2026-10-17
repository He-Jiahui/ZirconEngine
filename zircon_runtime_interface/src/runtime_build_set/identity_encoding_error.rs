use thiserror::Error;

/// Reports a serialization failure while deriving a runtime release identity.
/// 报告派生 Runtime 发布身份时的接口或身份序列化失败。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeIdentityEncodingError {
    #[error("embedded Runtime InterfaceSpec V1 cannot be decoded: {message}")]
    InterfaceSpecDecode { message: String },
    #[error("Runtime InterfaceSpec V1 cannot be encoded for its digest: {message}")]
    InterfaceSpecEncode { message: String },
    #[error("Runtime BuildSet identity cannot be encoded: {message}")]
    BuildSetEncode { message: String },
}

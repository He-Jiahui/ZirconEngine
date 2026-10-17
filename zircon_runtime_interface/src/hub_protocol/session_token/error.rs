use thiserror::Error;

/// Failure while parsing a Hub launch token supplied across the process boundary.
#[derive(Debug, Error)]
/// 令牌文本不符合当前规范表示时返回的解析错误。
pub enum HubSessionTokenParseError {
    #[error("Hub session token is not a UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Hub session token must use canonical lowercase hyphenated UUID syntax")]
    NonCanonical,
    #[error("Hub session token must be a UUID v4")]
    UnsupportedVersion,
}

//! Hub 启动会话在命令行和邮箱间传递的 UUID v4 不透明令牌。
#[path = "construct.rs"]
mod construct;
#[path = "error.rs"]
mod error;
#[path = "format.rs"]
mod format;
#[path = "parse.rs"]
mod parse;
#[path = "serde.rs"]
mod serde;

use uuid::Uuid;

pub use error::HubSessionTokenParseError;

/// Opaque canonical UUID v4 shared by Hub and Editor for one launch handshake.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// 仅通过规范构造和解析入口创建的会话标识，内部 UUID 不向调用方暴露。
pub struct HubSessionToken(pub(super) Uuid);

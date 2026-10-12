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
pub struct HubSessionToken(pub(super) Uuid);

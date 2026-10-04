//! Current-SDK-image authority for the unchanged V3 byte carrier.
mod error;
mod registry;
mod release;

pub use error::{NativePluginOwnedBytesError, NativePluginOwnedBytesErrorKind};
pub(super) use registry::register;
pub(super) use release::release;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

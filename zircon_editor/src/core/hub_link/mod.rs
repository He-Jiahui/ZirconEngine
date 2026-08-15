//! Hub-facing editor protocol adapters.
//!
//! This module owns mailbox semantics only. Native-window attention remains at the retained host
//! boundary, and project liveness remains exclusively owned by `recovery::SessionGuard`.

mod error;
mod focus_signal;
mod focus_watch;
mod handshake;
mod recent_writeback;

pub use error::HubFocusSignalError;
pub use focus_signal::{consume_focus_signal, focus_signal_path, publish_focus_signal};
pub(crate) use focus_watch::{HubFocusSignalWatch, HubFocusSignalWatchError};
pub(crate) use handshake::{HubEditorHandshake, HubHandshakeError};
pub(crate) use recent_writeback::{
    forget_recent_project, load_recent_projects, record_recent_project, HubRecentProjectsStoreError,
};

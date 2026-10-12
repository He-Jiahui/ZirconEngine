//! Hub-facing editor protocol adapters.
//!
//! This module owns mailbox semantics only. Native-window attention remains at the retained host
//! boundary, and project liveness remains exclusively owned by `recovery::SessionGuard`.

#[path = "error.rs"]
mod error;
#[path = "focus_ack_bridge.rs"]
mod focus_ack_bridge;
#[path = "focus_binding.rs"]
mod focus_binding;
#[path = "focus_signal.rs"]
mod focus_signal;
#[path = "focus_watch.rs"]
mod focus_watch;
#[path = "handshake.rs"]
mod handshake;
#[path = "recent_writeback.rs"]
mod recent_writeback;

pub use error::HubFocusSignalError;
pub(crate) use focus_ack_bridge::HubFocusAcknowledgementBridge;
pub(crate) use focus_binding::{HubFocusBinding, HubFocusBindingError, HubFocusBindingTarget};
pub use focus_signal::{
    consume_focus_signals, focus_signal_path, publish_focus_ack, publish_focus_signal,
};
pub(crate) use focus_watch::{HubFocusSignalWatch, HubFocusSignalWatchError};
pub(crate) use handshake::{HubEditorHandshake, HubHandshakeError};
pub(crate) use recent_writeback::{
    forget_recent_project, load_recent_projects, record_recent_project, HubRecentProjectsStoreError,
};

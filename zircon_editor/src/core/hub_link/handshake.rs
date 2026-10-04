use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime_interface::hub_protocol::{
    HubEditorMailboxV1, HubEditorReadyReceiptV1, HubEditorStartupFailureCodeV1, HubSessionToken,
};

const ZIRCON_DIRECTORY: &str = ".zircon";
const HUB_DIRECTORY: &str = "hub";

/// Immutable launch context for one Hub-initiated project editor session.
///
/// This owns the file-mailbox address only. `SessionGuard` remains the separate and exclusive
/// authority for project liveness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HubEditorHandshake {
    project_root: PathBuf,
    session: HubSessionToken,
}

impl HubEditorHandshake {
    pub(crate) fn new(project_root: impl Into<PathBuf>, session: HubSessionToken) -> Self {
        Self {
            project_root: project_root.into(),
            session,
        }
    }

    pub(crate) fn session(&self) -> HubSessionToken {
        self.session
    }

    pub(crate) fn mailbox_path(&self) -> PathBuf {
        handshake_mailbox_path(&self.project_root, self.session)
    }

    pub(crate) fn publish_ready(
        &self,
        receipt: HubEditorReadyReceiptV1,
    ) -> Result<(), HubHandshakeError> {
        self.publish(HubEditorMailboxV1::ready(self.session, receipt))
    }

    pub(crate) fn publish_failed(
        &self,
        code: HubEditorStartupFailureCodeV1,
    ) -> Result<(), HubHandshakeError> {
        self.publish(HubEditorMailboxV1::failed(self.session, code))
    }

    fn publish(&self, mailbox: HubEditorMailboxV1) -> Result<(), HubHandshakeError> {
        let path = self.mailbox_path();
        let bytes = serde_json::to_vec(&mailbox).map_err(|source| HubHandshakeError::Encode {
            path: path.clone(),
            source,
        })?;
        atomic_write(&path, &bytes).map_err(|source| HubHandshakeError::Io {
            operation: "publish",
            path,
            source,
        })
    }
}

pub(crate) fn handshake_mailbox_path(
    project_root: impl AsRef<Path>,
    session: HubSessionToken,
) -> PathBuf {
    project_root
        .as_ref()
        .join(ZIRCON_DIRECTORY)
        .join(HUB_DIRECTORY)
        .join(format!("{session}.json"))
}

#[derive(Debug, Error)]
pub(crate) enum HubHandshakeError {
    #[error("failed to encode Hub handshake mailbox `{path}`: {source}")]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to {operation} Hub handshake mailbox `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[cfg(test)]
#[path = "tests/handshake.rs"]
mod tests;

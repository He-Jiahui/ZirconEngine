use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime_interface::hub_protocol::{HubEditorMailboxV1, HubSessionToken};

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

    pub(crate) fn publish_ready(&self, process_id: u32) -> Result<(), HubHandshakeError> {
        self.publish(HubEditorMailboxV1::ready(process_id, &self.project_root))
    }

    pub(crate) fn publish_failed(
        &self,
        reason: impl Into<String>,
    ) -> Result<(), HubHandshakeError> {
        self.publish(HubEditorMailboxV1::failed(reason))
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
mod tests {
    use std::str::FromStr;

    use zircon_runtime_interface::hub_protocol::{HubEditorMailboxV1, HubSessionToken};

    use super::{handshake_mailbox_path, HubEditorHandshake};

    #[test]
    fn handshake_mailbox_is_scoped_to_the_project_and_typed_session() {
        let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
            .expect("valid test session");

        assert_eq!(
            handshake_mailbox_path("E:/Projects/My Game", session),
            std::path::PathBuf::from(
                "E:/Projects/My Game/.zircon/hub/0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52.json"
            )
        );
    }

    #[test]
    fn handshake_publishes_a_ready_mailbox_with_the_editor_process_and_project() {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR")
            .expect("Hub handshake filesystem tests require coordinator-managed CARGO_TARGET_DIR");
        let directory = std::path::PathBuf::from(target_directory).join(format!(
            "zircon-editor-hub-handshake-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
            .expect("valid test session");
        let handshake = HubEditorHandshake::new(&directory, session);

        handshake.publish_ready(913).expect("publish ready mailbox");

        let bytes = std::fs::read(handshake.mailbox_path()).expect("read ready mailbox");
        assert_eq!(
            serde_json::from_slice::<HubEditorMailboxV1>(&bytes).expect("decode ready mailbox"),
            HubEditorMailboxV1::ready(913, &directory)
        );
        std::fs::remove_dir_all(&directory).expect("remove temporary project root");
    }
}

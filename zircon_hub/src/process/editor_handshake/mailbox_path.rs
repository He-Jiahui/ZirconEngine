use std::path::{Path, PathBuf};

use zircon_runtime_interface::hub_protocol::HubSessionToken;

/// Returns the per-launch mailbox path below the project-owned Zircon directory.
pub(super) fn editor_handshake_mailbox_path(
    project_root: impl AsRef<Path>,
    session: HubSessionToken,
) -> PathBuf {
    project_root
        .as_ref()
        .join(".zircon")
        .join("hub")
        .join(format!("{session}.json"))
}

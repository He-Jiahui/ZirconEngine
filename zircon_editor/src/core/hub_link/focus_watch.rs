use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;

use super::{consume_focus_signal, focus_signal_path, HubFocusSignalError};

/// Keeps one OS watcher alive for the active editor session's focus inbox.
///
/// `notify` provides event ingress; this type never polls the filesystem from the UI frame loop.
/// The callback consumes the exact target mailbox before asking the retained host to focus.
pub(crate) struct HubFocusSignalWatch {
    _watcher: RecommendedWatcher,
}

impl HubFocusSignalWatch {
    pub(crate) fn start(
        project_root: impl AsRef<Path>,
        local_instance_id: impl Into<String>,
        request_window_attention: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, HubFocusSignalWatchError> {
        let project_root = project_root.as_ref().to_path_buf();
        let local_instance_id = local_instance_id.into();
        let mailbox_path = focus_signal_path(&project_root, &local_instance_id)?;
        let focus_directory = mailbox_path
            .parent()
            .expect("focus mailbox always has its project-owned parent directory")
            .to_path_buf();
        fs::create_dir_all(&focus_directory).map_err(|source| HubFocusSignalWatchError::Io {
            operation: "create",
            path: focus_directory.clone(),
            source,
        })?;

        let callback_project_root = project_root.clone();
        let callback_instance_id = local_instance_id.clone();
        let callback_mailbox_path = mailbox_path.clone();
        let request_window_attention = Arc::new(request_window_attention);
        let callback_attention = Arc::clone(&request_window_attention);
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let Ok(event) = event else {
                    return;
                };
                if !event
                    .paths
                    .iter()
                    .any(|path| path == &callback_mailbox_path)
                {
                    return;
                }
                if let Err(error) = consume_pending_focus_signal(
                    &callback_project_root,
                    &callback_instance_id,
                    || callback_attention(),
                ) {
                    eprintln!("[zircon_editor] Hub focus mailbox was rejected: {error}")
                }
            })
            .map_err(|source| HubFocusSignalWatchError::CreateWatcher { source })?;
        watcher
            .watch(&focus_directory, RecursiveMode::NonRecursive)
            .map_err(|source| HubFocusSignalWatchError::Watch {
                path: focus_directory,
                source,
            })?;
        consume_pending_focus_signal(&project_root, &local_instance_id, || {
            request_window_attention()
        })?;

        Ok(Self { _watcher: watcher })
    }
}

/// Consumes the target mailbox once after watch registration and from every notify callback.
///
/// The rename claim in `consume_focus_signal` makes concurrent calls safe: only one path can
/// observe a published request, so startup recovery cannot duplicate window attention.
fn consume_pending_focus_signal(
    project_root: &Path,
    local_instance_id: &str,
    request_window_attention: impl Fn(),
) -> Result<(), HubFocusSignalError> {
    if consume_focus_signal(project_root, local_instance_id)?.is_some() {
        request_window_attention();
    }
    Ok(())
}

#[derive(Debug, Error)]
pub(crate) enum HubFocusSignalWatchError {
    #[error(transparent)]
    Signal(#[from] HubFocusSignalError),
    #[error("failed to {operation} Hub focus directory `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to create Hub focus watcher: {source}")]
    CreateWatcher {
        #[source]
        source: notify::Error,
    },
    #[error("failed to watch Hub focus directory `{path}`: {source}")]
    Watch {
        path: PathBuf,
        #[source]
        source: notify::Error,
    },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use zircon_runtime_interface::hub_protocol::HubSessionToken;

    use crate::core::hub_link::{consume_focus_signal, publish_focus_signal};
    use crate::core::recovery::SessionLockRecord;

    use super::consume_pending_focus_signal;

    #[test]
    fn focus_mailbox_path_names_the_target_instance_only() {
        let target = super::super::focus_signal_path("E:/Projects/My Game", "4132-17")
            .expect("valid instance id");

        assert_eq!(
            target,
            PathBuf::from("E:/Projects/My Game/.zircon/hub/focus/4132-17.json")
        );
    }

    #[test]
    fn watcher_startup_consumes_a_focus_signal_published_before_registration() {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR")
            .expect("focus watcher filesystem tests require coordinator-managed CARGO_TARGET_DIR");
        let project_root = std::path::PathBuf::from(target_directory).join(format!(
            "zircon-editor-focus-watch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        let instance_id = "913-42";
        let lock_record = SessionLockRecord::new(913, instance_id, 42)
            .expect("test session lock record should be valid");
        let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
            .expect("test Hub session token should be valid");
        std::fs::create_dir_all(project_root.join(".zircon/hub/focus"))
            .expect("create focus mailbox directory");
        publish_focus_signal(&project_root, &lock_record, session)
            .expect("publish focus signal before watcher registration");

        let attention_count = AtomicUsize::new(0);
        consume_pending_focus_signal(&project_root, instance_id, || {
            attention_count.fetch_add(1, Ordering::Relaxed);
        })
        .expect("startup consume should accept the targeted signal");

        assert_eq!(attention_count.load(Ordering::Relaxed), 1);
        assert_eq!(
            consume_focus_signal(&project_root, instance_id)
                .expect("focus mailbox should remain readable after startup consume"),
            None
        );
        std::fs::remove_dir_all(project_root).expect("remove temporary project root");
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;

use super::{consume_focus_signals, HubFocusSignalError};
use zircon_runtime_interface::hub_protocol::{
    hub_editor_focus_request_directory, HubEditorFocusSignalV1,
};

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
        local_session_generation: u64,
        on_focus_request: impl Fn(HubEditorFocusSignalV1) + Send + Sync + 'static,
    ) -> Result<Self, HubFocusSignalWatchError> {
        let project_root = project_root.as_ref().to_path_buf();
        let local_instance_id = local_instance_id.into();
        let focus_directory = hub_editor_focus_request_directory(&project_root, &local_instance_id)
            .map_err(|error| HubFocusSignalWatchError::Signal(HubFocusSignalError::Path(error)))?;
        fs::create_dir_all(&focus_directory).map_err(|source| HubFocusSignalWatchError::Io {
            operation: "create",
            path: focus_directory.clone(),
            source,
        })?;

        let callback_project_root = project_root.clone();
        let callback_instance_id = local_instance_id.clone();
        let callback_focus_directory = focus_directory.clone();
        let on_focus_request = Arc::new(on_focus_request);
        let callback_focus_request = Arc::clone(&on_focus_request);
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let Ok(event) = event else {
                    return;
                };
                if !event
                    .paths
                    .iter()
                    .any(|path| path.starts_with(&callback_focus_directory))
                {
                    return;
                }
                if let Err(error) = consume_pending_focus_signals(
                    &callback_project_root,
                    &callback_instance_id,
                    local_session_generation,
                    |request| callback_focus_request(request),
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
        consume_pending_focus_signals(
            &project_root,
            &local_instance_id,
            local_session_generation,
            |request| on_focus_request(request),
        )?;

        Ok(Self { _watcher: watcher })
    }
}

/// Consumes the target mailbox once after watch registration and from every notify callback.
///
/// The rename claim in `consume_focus_signal` makes concurrent calls safe: only one path can
/// observe a published request, so startup recovery cannot duplicate window attention.
fn consume_pending_focus_signals(
    project_root: &Path,
    local_instance_id: &str,
    local_session_generation: u64,
    on_focus_request: impl Fn(HubEditorFocusSignalV1),
) -> Result<(), HubFocusSignalError> {
    for request in consume_focus_signals(project_root, local_instance_id, local_session_generation)?
    {
        on_focus_request(request);
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
#[path = "tests/focus_watch.rs"]
mod tests;

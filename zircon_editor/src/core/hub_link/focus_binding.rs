use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;

use super::{
    HubFocusAcknowledgementBridge, HubFocusSignalError, HubFocusSignalWatch,
    HubFocusSignalWatchError,
};

/// Immutable identity for the focus inbox bound to one committed project session generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HubFocusBindingTarget {
    project_root: PathBuf,
    instance_id: String,
    session_generation: u64,
}

impl HubFocusBindingTarget {
    pub(crate) fn new(project_root: PathBuf, instance_id: String, session_generation: u64) -> Self {
        Self {
            project_root,
            instance_id,
            session_generation,
        }
    }
}

/// Owns at most one generation-qualified filesystem watcher and its native-focus acknowledgement
/// bridge. Rebinding never scans or creates a watcher when the target identity is unchanged.
#[derive(Default)]
pub(crate) struct HubFocusBinding {
    active: Option<ActiveHubFocusBinding>,
}

struct ActiveHubFocusBinding {
    target: HubFocusBindingTarget,
    acknowledgement: HubFocusAcknowledgementBridge,
    _watch: HubFocusSignalWatch,
}

impl HubFocusBinding {
    pub(crate) fn sync(
        &mut self,
        target: Option<HubFocusBindingTarget>,
        request_window_attention: &Arc<dyn Fn() + Send + Sync>,
    ) -> Result<(), HubFocusBindingError> {
        if self
            .active
            .as_ref()
            .is_some_and(|active| Some(&active.target) == target.as_ref())
        {
            return Ok(());
        }

        // Retire first: an in-flight old watcher can now only publish `RejectedStale`.
        if let Some(active) = self.active.as_ref() {
            if let Err(error) = active.acknowledgement.retire() {
                self.active = None;
                return Err(error.into());
            }
        }
        let next = target
            .map(|target| ActiveHubFocusBinding::start(target, request_window_attention))
            .transpose();
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                self.active = None;
                return Err(error);
            }
        };
        self.active = next;
        Ok(())
    }

    pub(crate) fn acknowledge_native_window_focus(&self) -> Result<(), HubFocusSignalError> {
        self.active
            .as_ref()
            .map(|active| active.acknowledgement.acknowledge_window_focused())
            .transpose()
            .map(|_| ())
    }

    pub(crate) fn is_bound(&self) -> bool {
        self.active.is_some()
    }
}

impl ActiveHubFocusBinding {
    fn start(
        target: HubFocusBindingTarget,
        request_window_attention: &Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self, HubFocusBindingError> {
        let acknowledgement = HubFocusAcknowledgementBridge::new(&target.project_root);
        let callback_acknowledgement = acknowledgement.clone();
        let request_window_attention = Arc::clone(request_window_attention);
        let watch = HubFocusSignalWatch::start(
            &target.project_root,
            target.instance_id.clone(),
            target.session_generation,
            move |request| match callback_acknowledgement.enqueue(request) {
                Ok(true) => request_window_attention(),
                Ok(false) => {}
                Err(error) => eprintln!(
                    "[zircon_editor] failed to retain Hub focus request for native acknowledgement: {error}"
                ),
            },
        )?;
        Ok(Self {
            target,
            acknowledgement,
            _watch: watch,
        })
    }
}

#[derive(Debug, Error)]
pub(crate) enum HubFocusBindingError {
    #[error(transparent)]
    Signal(#[from] HubFocusSignalError),
    #[error(transparent)]
    Watch(#[from] HubFocusSignalWatchError),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::HubFocusBindingTarget;

    #[test]
    fn focus_binding_identity_requires_project_instance_and_generation_to_match() {
        let base =
            HubFocusBindingTarget::new(PathBuf::from("E:/Projects/Game"), "913-42".into(), 7);

        assert_eq!(
            base,
            HubFocusBindingTarget::new(PathBuf::from("E:/Projects/Game"), "913-42".into(), 7)
        );
        assert_ne!(
            base,
            HubFocusBindingTarget::new(PathBuf::from("E:/Projects/Game"), "913-42".into(), 8)
        );
    }
}

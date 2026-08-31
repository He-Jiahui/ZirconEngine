use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime_interface::hub_protocol::{
    HubEditorFocusAckDispositionV1, HubEditorFocusSignalV1,
};

use super::{HubFocusSignalError, publish_focus_ack};

const MAX_PENDING_ACKNOWLEDGEMENTS: usize = 32;

/// Transfers validated focus requests from the filesystem watcher to the native window owner.
///
/// Acknowledgements are deliberately published from the `Focused(true)` event, never from the
/// watcher thread or the call that merely requested window attention.
#[derive(Clone)]
pub(crate) struct HubFocusAcknowledgementBridge {
    project_root: PathBuf,
    pending: Arc<Mutex<Vec<HubEditorFocusSignalV1>>>,
    accepting: Arc<AtomicBool>,
}

impl HubFocusAcknowledgementBridge {
    pub(crate) fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            pending: Arc::new(Mutex::new(Vec::new())),
            accepting: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Returns whether the native host should request window attention.
    ///
    /// A full queue is terminally acknowledged here, so the Hub receives a typed rejection
    /// rather than waiting for a request that cannot be retained by this editor owner.
    pub(crate) fn enqueue(
        &self,
        request: HubEditorFocusSignalV1,
    ) -> Result<bool, HubFocusSignalError> {
        if !self.accepting.load(Ordering::Acquire) {
            self.publish_rejection(request, HubEditorFocusAckDispositionV1::RejectedStale)?;
            return Ok(false);
        }
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !self.accepting.load(Ordering::Acquire) {
            drop(pending);
            self.publish_rejection(request, HubEditorFocusAckDispositionV1::RejectedStale)?;
            return Ok(false);
        }
        if pending.len() >= MAX_PENDING_ACKNOWLEDGEMENTS {
            drop(pending);
            self.publish_rejection(request, HubEditorFocusAckDispositionV1::RejectedInboxFull)?;
            return Ok(false);
        }
        pending.push(request);
        Ok(true)
    }

    pub(crate) fn acknowledge_window_focused(&self) -> Result<(), HubFocusSignalError> {
        let now_unix_millis = unix_millis_now()?;
        self.publish_pending(|request| {
            if !self.accepting.load(Ordering::Acquire) {
                HubEditorFocusAckDispositionV1::RejectedStale
            } else if request.is_expired_at(now_unix_millis) {
                HubEditorFocusAckDispositionV1::RejectedExpired
            } else {
                HubEditorFocusAckDispositionV1::Focused
            }
        })
    }

    /// Stops accepting this generation before its watcher is dropped.
    ///
    /// A watcher callback already in flight observes the atomic flag and emits `RejectedStale`.
    /// This prevents a retired project generation from later acknowledging `Focused`.
    pub(crate) fn retire(&self) -> Result<(), HubFocusSignalError> {
        self.accepting.store(false, Ordering::Release);
        self.publish_pending(|_| HubEditorFocusAckDispositionV1::RejectedStale)
    }

    fn publish_rejection(
        &self,
        request: HubEditorFocusSignalV1,
        disposition: HubEditorFocusAckDispositionV1,
    ) -> Result<(), HubFocusSignalError> {
        publish_focus_ack(&self.project_root, &request, disposition)
    }

    fn publish_pending(
        &self,
        disposition: impl Fn(&HubEditorFocusSignalV1) -> HubEditorFocusAckDispositionV1,
    ) -> Result<(), HubFocusSignalError> {
        let pending = std::mem::take(
            &mut *self
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        let mut requests = pending.into_iter();
        while let Some(request) = requests.next() {
            if let Err(error) =
                publish_focus_ack(&self.project_root, &request, disposition(&request))
            {
                let mut pending = self
                    .pending
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                pending.push(request);
                pending.extend(requests);
                return Err(error);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn accepting_for_test(&self) -> bool {
        self.accepting.load(Ordering::Acquire)
    }
}

fn unix_millis_now() -> Result<u64, HubFocusSignalError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .map_err(|source| HubFocusSignalError::Clock { source })
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::hub_protocol::{HubEditorFocusSignalV1, HubSessionToken};

    use super::HubFocusAcknowledgementBridge;

    #[test]
    fn bridge_preserves_multiple_requests_until_the_native_focus_owner_acknowledges_them() {
        let bridge = HubFocusAcknowledgementBridge::new("E:/Projects/My Game");
        let first = HubEditorFocusSignalV1::new(HubSessionToken::new(), "913-42", 1, 1, u64::MAX)
            .expect("first request");
        let second = HubEditorFocusSignalV1::new(HubSessionToken::new(), "913-42", 1, 2, u64::MAX)
            .expect("second request");

        assert!(bridge.enqueue(first).expect("queue first request"));
        assert!(bridge.enqueue(second).expect("queue second request"));

        let pending = bridge
            .pending
            .lock()
            .expect("test queue lock must be available");
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].sequence, 1);
        assert_eq!(pending[1].sequence, 2);
    }

    #[test]
    fn retired_bridge_refuses_to_acknowledge_the_previous_session_generation() {
        let bridge = HubFocusAcknowledgementBridge::new("E:/Projects/My Game");

        bridge.retire().expect("an empty bridge can retire");

        assert!(!bridge.accepting_for_test());
    }
}

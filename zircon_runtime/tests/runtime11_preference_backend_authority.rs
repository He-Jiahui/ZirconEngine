use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime::core::framework::platform::{
    PreferenceKey, PreferenceMutationTerminal, PreferenceStorageBackendKind,
    PreferenceStorageError, PreferenceTicketWaitResult, PreferenceWorkDeadline,
};
use zircon_runtime::platform::preferences::{
    PreferenceBackendWorkAuthority, PreferencePersistenceAdapter, PreferencePersistenceLimits,
    PreferenceStorageBackend,
};

#[derive(Default)]
struct ExternalBackend {
    calls: AtomicUsize,
}

impl PreferenceStorageBackend for ExternalBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::HostProvided
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
    ) -> Result<Option<Box<dyn Read + Send>>, PreferenceStorageError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(Some(Box::new(Cursor::new(Vec::<u8>::new()))))
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
        _value: &[u8],
    ) -> Result<(), PreferenceStorageError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn external_backend_can_implement_host_spi() {
    let backend: Arc<dyn PreferenceStorageBackend> = Arc::new(ExternalBackend::default());
    assert_eq!(
        backend.backend_kind(),
        PreferenceStorageBackendKind::HostProvided
    );
}

#[test]
fn authority_is_issued_only_by_persistence_worker() {
    let backend = Arc::new(ExternalBackend::default());
    let adapter = PreferencePersistenceAdapter::new(
        backend.clone(),
        PreferencePersistenceLimits {
            max_value_bytes: 1024,
            max_overlay_entries: 4,
            max_overlay_retained_bytes: 16 * 1024,
            max_lane_entries: 4,
            max_lane_retained_bytes: 16 * 1024,
        },
    );
    let submission = adapter
        .submit_write(
            PreferenceKey::new("external.host", "value").unwrap(),
            Arc::from(&b"value"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();

    assert!(matches!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert_eq!(backend.calls.load(Ordering::Relaxed), 1);
}

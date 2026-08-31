use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::runtime::TaskCancellationToken;

use super::super::{RenderArtifactStore, RenderArtifactStoreError, RenderArtifactStoreLimits};
use super::contract::{
    RenderArtifactManifestCancelReason, RenderArtifactManifestFailure,
    RenderArtifactManifestFailureCode,
};
use super::state::RenderArtifactManifestEntry;

#[derive(Default)]
pub(super) struct RenderArtifactManifestLoaderMetrics {
    pub(super) submitted_io_tasks: AtomicU64,
    pub(super) merged_requests: AtomicU64,
    pub(super) ready_entries: AtomicU64,
    pub(super) failed_entries: AtomicU64,
    pub(super) cancelled_entries: AtomicU64,
    pub(super) expired_tickets: AtomicU64,
    pub(super) io_worker_wall_ns: AtomicU64,
}

pub(super) fn run_manifest_io_task(
    entry: Arc<RenderArtifactManifestEntry>,
    store: RenderArtifactStore,
    limits: RenderArtifactStoreLimits,
    metrics: Arc<RenderArtifactManifestLoaderMetrics>,
    cancellation: TaskCancellationToken,
) {
    if cancellation.is_cancellation_requested() {
        cancellation.acknowledge_cancellation();
        record_cancelled(&entry, &metrics);
        return;
    }
    if !entry.begin_io() {
        return;
    }
    let started = Instant::now();
    let result = store.read_manifest(
        entry.key().resource(),
        entry.key().asset_revision(),
        entry.key().target_platform(),
        limits,
    );
    atomic_add_duration(&metrics.io_worker_wall_ns, started.elapsed());
    if cancellation.is_cancellation_requested() {
        cancellation.acknowledge_cancellation();
        record_cancelled(&entry, &metrics);
        return;
    }
    match result {
        Ok(manifest) => {
            if entry.complete(Arc::new(manifest)) {
                metrics.ready_entries.fetch_add(1, Ordering::Relaxed);
            }
        }
        Err(error) => {
            if entry.fail(store_failure(error)) {
                metrics.failed_entries.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

pub(super) fn atomic_add(counter: &AtomicU64, value: u64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(value))
    });
}

fn record_cancelled(
    entry: &RenderArtifactManifestEntry,
    metrics: &RenderArtifactManifestLoaderMetrics,
) {
    if entry.cancel(RenderArtifactManifestCancelReason::OwnerClosed) {
        metrics.cancelled_entries.fetch_add(1, Ordering::Relaxed);
    }
}

fn store_failure(error: RenderArtifactStoreError) -> RenderArtifactManifestFailure {
    let code = match &error {
        RenderArtifactStoreError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
            RenderArtifactManifestFailureCode::NotFound
        }
        RenderArtifactStoreError::ZeroByteLimit { .. }
        | RenderArtifactStoreError::ByteLimitExceeded { .. }
        | RenderArtifactStoreError::AddressSpaceOverflow { .. } => {
            RenderArtifactManifestFailureCode::StoreLimitExceeded
        }
        RenderArtifactStoreError::ManifestMagicMismatch
        | RenderArtifactStoreError::ManifestIdentityMismatch
        | RenderArtifactStoreError::ManifestDeserialize(_)
        | RenderArtifactStoreError::ManifestValidation(_) => {
            RenderArtifactManifestFailureCode::InvalidManifest
        }
        _ => RenderArtifactManifestFailureCode::StoreUnavailable,
    };
    RenderArtifactManifestFailure::new(code, error.to_string())
}

fn atomic_add_duration(counter: &AtomicU64, duration: Duration) {
    atomic_add(
        counter,
        duration.as_nanos().min(u128::from(u64::MAX)) as u64,
    );
}

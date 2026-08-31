use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use crate::core::runtime::TaskCancellationToken;

use super::super::{
    RenderArtifactBlockCodec, RenderArtifactStore, RenderArtifactStoreError,
    RenderArtifactStoreLimits,
};
use super::contract::{
    RenderArtifactBlockCancelReason, RenderArtifactBlockFailure, RenderArtifactBlockFailureCode,
};
use super::decode::decode_zstd_block;
use super::entry::RenderArtifactBlockEntry;
use super::loader::RenderArtifactBlockLoaderInner;

#[derive(Default)]
pub(super) struct RenderArtifactBlockLoaderMetrics {
    pub(super) submitted_io_tasks: AtomicU64,
    pub(super) submitted_decode_tasks: AtomicU64,
    pub(super) merged_requests: AtomicU64,
    pub(super) ready_entries: AtomicU64,
    pub(super) failed_entries: AtomicU64,
    pub(super) cancelled_entries: AtomicU64,
    pub(super) expired_tickets: AtomicU64,
    pub(super) encoded_bytes_read: AtomicU64,
    pub(super) decoded_bytes: AtomicU64,
    pub(super) io_worker_wall_ns: AtomicU64,
    pub(super) decode_worker_wall_ns: AtomicU64,
}

pub(super) fn run_io_task(
    loader: Weak<RenderArtifactBlockLoaderInner>,
    entry: Arc<RenderArtifactBlockEntry>,
    store: RenderArtifactStore,
    limits: RenderArtifactStoreLimits,
    metrics: Arc<RenderArtifactBlockLoaderMetrics>,
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
    let result = store.read_block(entry.descriptor(), limits);
    atomic_add_duration(&metrics.io_worker_wall_ns, started.elapsed());
    if cancellation.is_cancellation_requested() {
        cancellation.acknowledge_cancellation();
        record_cancelled(&entry, &metrics);
        return;
    }
    let encoded = match result {
        Ok(encoded) => encoded,
        Err(error) => {
            if entry.fail(store_failure(error)) {
                metrics.failed_entries.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
    };
    atomic_add(&metrics.encoded_bytes_read, encoded.len() as u64);
    match entry.descriptor().codec() {
        RenderArtifactBlockCodec::Raw => {
            let decoded_bytes = entry.descriptor().decoded_bytes();
            if entry.complete(encoded) {
                metrics.ready_entries.fetch_add(1, Ordering::Relaxed);
                atomic_add(&metrics.decoded_bytes, decoded_bytes);
            }
        }
        RenderArtifactBlockCodec::Zstd => {
            if !entry.queue_decode() {
                return;
            }
            let Some(loader) = loader.upgrade() else {
                record_cancelled(&entry, &metrics);
                return;
            };
            loader.schedule_decode(entry, encoded);
        }
    }
}

pub(super) fn run_decode_task(
    entry: Arc<RenderArtifactBlockEntry>,
    encoded: Arc<[u8]>,
    metrics: Arc<RenderArtifactBlockLoaderMetrics>,
    cancellation: TaskCancellationToken,
) {
    if cancellation.is_cancellation_requested() {
        cancellation.acknowledge_cancellation();
        record_cancelled(&entry, &metrics);
        return;
    }
    if !entry.begin_decode() {
        return;
    }
    let started = Instant::now();
    let result = decode_zstd_block(&encoded, entry.descriptor().decoded_bytes());
    atomic_add_duration(&metrics.decode_worker_wall_ns, started.elapsed());
    if cancellation.is_cancellation_requested() {
        cancellation.acknowledge_cancellation();
        record_cancelled(&entry, &metrics);
        return;
    }
    match result {
        Ok(decoded) => {
            let decoded_bytes = entry.descriptor().decoded_bytes();
            if entry.complete(decoded) {
                metrics.ready_entries.fetch_add(1, Ordering::Relaxed);
                atomic_add(&metrics.decoded_bytes, decoded_bytes);
            }
        }
        Err(failure) => {
            if entry.fail(failure) {
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

fn record_cancelled(entry: &RenderArtifactBlockEntry, metrics: &RenderArtifactBlockLoaderMetrics) {
    if entry.cancel(RenderArtifactBlockCancelReason::OwnerClosed) {
        metrics.cancelled_entries.fetch_add(1, Ordering::Relaxed);
    }
}

fn store_failure(error: RenderArtifactStoreError) -> RenderArtifactBlockFailure {
    let code = match &error {
        RenderArtifactStoreError::BlockSizeMismatch { .. } => {
            RenderArtifactBlockFailureCode::BlockSizeMismatch
        }
        RenderArtifactStoreError::BlockContentHashMismatch { .. } => {
            RenderArtifactBlockFailureCode::BlockHashMismatch
        }
        RenderArtifactStoreError::ZeroByteLimit { .. }
        | RenderArtifactStoreError::ByteLimitExceeded { .. }
        | RenderArtifactStoreError::AddressSpaceOverflow { .. } => {
            RenderArtifactBlockFailureCode::StoreLimitExceeded
        }
        _ => RenderArtifactBlockFailureCode::StoreUnavailable,
    };
    RenderArtifactBlockFailure::new(code, error.to_string())
}

fn atomic_add_duration(counter: &AtomicU64, duration: Duration) {
    atomic_add(
        counter,
        duration.as_nanos().min(u128::from(u64::MAX)) as u64,
    );
}

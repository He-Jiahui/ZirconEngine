use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{after, bounded, never, select, Receiver, Sender, TrySendError};
use notify::Event;

use super::asset_watch_batch::{AssetWatchBatch, AssetWatchBatchDiagnostics};
use super::asset_watch_event::AssetWatchEvent;
use super::asset_watcher::AssetWatcherOptions;
use super::fold_events::{finish_folded_events, fold_event, FoldedAssetChangeMap};
use super::map_notify_event::map_notify_event;
use super::{AssetChange, AssetWatchError};

struct WatchIngressMessage {
    received_at: Instant,
    approximate_bytes: usize,
    result: notify::Result<Event>,
}

#[derive(Clone)]
pub(crate) struct WatchIngressSender {
    sender: Sender<WatchIngressMessage>,
    queued_bytes: Arc<AtomicUsize>,
    overflow_count: Arc<AtomicUsize>,
    overflow_signal: Sender<()>,
    byte_capacity: usize,
}

pub(crate) struct WatchIngressReceiver {
    receiver: Receiver<WatchIngressMessage>,
    queued_bytes: Arc<AtomicUsize>,
    overflow_count: Arc<AtomicUsize>,
    overflow_signal: Receiver<()>,
}

pub(crate) fn watch_ingress(
    options: AssetWatcherOptions,
) -> (WatchIngressSender, WatchIngressReceiver) {
    let entry_capacity = options.ingress_entry_capacity.max(1);
    let queued_bytes = Arc::new(AtomicUsize::new(0));
    let overflow_count = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = bounded(entry_capacity);
    let (overflow_tx, overflow_rx) = bounded(1);
    (
        WatchIngressSender {
            sender,
            queued_bytes: queued_bytes.clone(),
            overflow_count: overflow_count.clone(),
            overflow_signal: overflow_tx,
            byte_capacity: options.ingress_byte_capacity.max(1),
        },
        WatchIngressReceiver {
            receiver,
            queued_bytes,
            overflow_count,
            overflow_signal: overflow_rx,
        },
    )
}

impl WatchIngressSender {
    pub(crate) fn try_send(&self, result: notify::Result<Event>) {
        let approximate_bytes = approximate_notify_result_bytes(&result);
        if self
            .queued_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |queued| {
                queued
                    .checked_add(approximate_bytes)
                    .filter(|next| *next <= self.byte_capacity)
            })
            .is_err()
        {
            self.record_overflow();
            return;
        }
        let message = WatchIngressMessage {
            received_at: Instant::now(),
            approximate_bytes,
            result,
        };
        if let Err(error) = self.sender.try_send(message) {
            let message = match error {
                TrySendError::Full(message) | TrySendError::Disconnected(message) => message,
            };
            self.queued_bytes
                .fetch_sub(message.approximate_bytes, Ordering::AcqRel);
            self.record_overflow();
        }
    }

    fn record_overflow(&self) {
        self.overflow_count.fetch_add(1, Ordering::AcqRel);
        let _ = self.overflow_signal.try_send(());
    }
}

impl WatchIngressReceiver {
    fn finish_receive(&self, message: &WatchIngressMessage) {
        self.queued_bytes
            .fetch_sub(message.approximate_bytes, Ordering::AcqRel);
    }

    fn take_overflow_count(&self) -> usize {
        self.overflow_count.swap(0, Ordering::AcqRel)
    }
}

pub(super) fn watch_loop(
    assets_root: PathBuf,
    options: AssetWatcherOptions,
    stop_rx: Receiver<()>,
    ingress: WatchIngressReceiver,
    on_changes: Arc<dyn Fn(AssetWatchBatch) + Send + Sync>,
    on_error: Arc<dyn Fn(AssetWatchError) + Send + Sync>,
) {
    watch_loop_inner(assets_root, options, stop_rx, ingress, on_changes, on_error);
}

#[cfg(test)]
pub(crate) fn watch_loop_for_test(
    assets_root: PathBuf,
    options: AssetWatcherOptions,
    stop_rx: Receiver<()>,
    ingress: WatchIngressReceiver,
    on_changes: Arc<dyn Fn(AssetWatchBatch) + Send + Sync>,
    on_error: Arc<dyn Fn(AssetWatchError) + Send + Sync>,
) {
    watch_loop_inner(assets_root, options, stop_rx, ingress, on_changes, on_error);
}

fn watch_loop_inner(
    assets_root: PathBuf,
    options: AssetWatcherOptions,
    stop_rx: Receiver<()>,
    ingress: WatchIngressReceiver,
    on_changes: Arc<dyn Fn(AssetWatchBatch) + Send + Sync>,
    on_error: Arc<dyn Fn(AssetWatchError) + Send + Sync>,
) {
    let mut pending = FoldedAssetChangeMap::new();
    let mut pending_bytes = 0usize;
    let mut started_at = None;
    let mut last_event_at = None;
    let mut raw_event_count = 0usize;
    let mut pending_overflow_count = 0usize;
    let mut ingress_overflow_count = 0usize;
    let mut requires_reconciliation = false;

    loop {
        let ingress_overflow = ingress.take_overflow_count();
        if ingress_overflow > 0 {
            ingress_overflow_count = ingress_overflow_count.saturating_add(ingress_overflow);
            requires_reconciliation = true;
            started_at.get_or_insert_with(Instant::now);
            last_event_at = Some(Instant::now());
        }
        let now = Instant::now();
        if should_flush(now, started_at, last_event_at, options) {
            flush_pending(
                &on_changes,
                &mut pending,
                &mut pending_bytes,
                &mut started_at,
                &mut last_event_at,
                &mut raw_event_count,
                &mut ingress_overflow_count,
                &mut pending_overflow_count,
                &mut requires_reconciliation,
            );
            continue;
        }

        let timer = if started_at.is_some() {
            after(next_wakeup(now, started_at, last_event_at, options))
        } else {
            never()
        };
        select! {
            recv(stop_rx) -> _ => {
                flush_pending(
                    &on_changes,
                    &mut pending,
                    &mut pending_bytes,
                    &mut started_at,
                    &mut last_event_at,
                    &mut raw_event_count,
                    &mut ingress_overflow_count,
                    &mut pending_overflow_count,
                    &mut requires_reconciliation,
                );
                return;
            },
            recv(ingress.receiver) -> message => match message {
                Ok(message) => {
                    ingress.finish_receive(&message);
                    match message.result {
                        Ok(event) => {
                            for event in map_notify_event(&assets_root, event) {
                                raw_event_count = raw_event_count.saturating_add(1);
                                started_at.get_or_insert(message.received_at);
                                last_event_at = Some(Instant::now());
                                if !try_fold_bounded(
                                    &mut pending,
                                    &mut pending_bytes,
                                    event,
                                    options.pending_entry_capacity.max(1),
                                    options.pending_byte_capacity.max(1),
                                ) {
                                    pending_overflow_count = pending_overflow_count.saturating_add(1);
                                    requires_reconciliation = true;
                                }
                            }
                        }
                        Err(error) => {
                            requires_reconciliation = true;
                            started_at.get_or_insert(message.received_at);
                            last_event_at = Some(Instant::now());
                            on_error(AssetWatchError::from_notify_error(
                                assets_root.clone(),
                                error,
                            ));
                        }
                    }
                }
                Err(_) => {
                    flush_pending(
                        &on_changes,
                        &mut pending,
                        &mut pending_bytes,
                        &mut started_at,
                        &mut last_event_at,
                        &mut raw_event_count,
                        &mut ingress_overflow_count,
                        &mut pending_overflow_count,
                        &mut requires_reconciliation,
                    );
                    return;
                }
            },
            recv(ingress.overflow_signal) -> _ => {},
            recv(timer) -> _ => {}
        }
    }
}

fn should_flush(
    now: Instant,
    started_at: Option<Instant>,
    last_event_at: Option<Instant>,
    options: AssetWatcherOptions,
) -> bool {
    started_at.is_some_and(|started| {
        now.saturating_duration_since(started) >= options.max_batch_latency
            || last_event_at
                .is_some_and(|last| now.saturating_duration_since(last) >= options.debounce)
    })
}

fn next_wakeup(
    now: Instant,
    started_at: Option<Instant>,
    last_event_at: Option<Instant>,
    options: AssetWatcherOptions,
) -> Duration {
    let Some(started) = started_at else {
        return options.max_batch_latency;
    };
    let max_remaining = options
        .max_batch_latency
        .saturating_sub(now.saturating_duration_since(started));
    let quiet_remaining = last_event_at
        .map(|last| {
            options
                .debounce
                .saturating_sub(now.saturating_duration_since(last))
        })
        .unwrap_or(max_remaining);
    max_remaining.min(quiet_remaining)
}

#[allow(clippy::too_many_arguments)]
fn flush_pending(
    on_changes: &Arc<dyn Fn(AssetWatchBatch) + Send + Sync>,
    pending: &mut FoldedAssetChangeMap,
    pending_bytes: &mut usize,
    started_at: &mut Option<Instant>,
    last_event_at: &mut Option<Instant>,
    raw_event_count: &mut usize,
    ingress_overflow_count: &mut usize,
    pending_overflow_count: &mut usize,
    requires_reconciliation: &mut bool,
) {
    if pending.is_empty() && !*requires_reconciliation {
        return;
    }
    let oldest_age = started_at
        .map(|started| Instant::now().saturating_duration_since(started))
        .unwrap_or_default();
    let changes = finish_folded_events(std::mem::take(pending));
    let batch = AssetWatchBatch {
        diagnostics: AssetWatchBatchDiagnostics {
            raw_event_count: *raw_event_count,
            coalesced_event_count: (*raw_event_count).saturating_sub(changes.len()),
            ingress_overflow_count: *ingress_overflow_count,
            pending_overflow_count: *pending_overflow_count,
            approximate_bytes: *pending_bytes,
            oldest_age,
        },
        changes,
        requires_reconciliation: *requires_reconciliation,
    };
    *pending_bytes = 0;
    *started_at = None;
    *last_event_at = None;
    *raw_event_count = 0;
    *ingress_overflow_count = 0;
    *pending_overflow_count = 0;
    *requires_reconciliation = false;
    on_changes(batch);
}

fn try_fold_bounded(
    folded: &mut FoldedAssetChangeMap,
    approximate_bytes: &mut usize,
    event: AssetWatchEvent,
    entry_capacity: usize,
    byte_capacity: usize,
) -> bool {
    let touched = touched_uris(&event);
    let previous = touched
        .iter()
        .map(|uri| (uri.clone(), folded.get(uri).cloned()))
        .collect::<Vec<_>>();
    let previous_bytes = previous
        .iter()
        .filter_map(|(uri, value)| value.as_ref().map(|value| folded_entry_bytes(uri, value)))
        .sum::<usize>();
    fold_event(folded, event);
    let next_bytes = touched
        .iter()
        .filter_map(|uri| folded.get(uri).map(|value| folded_entry_bytes(uri, value)))
        .sum::<usize>();
    let candidate_bytes = approximate_bytes
        .saturating_sub(previous_bytes)
        .saturating_add(next_bytes);
    if folded.len() <= entry_capacity && candidate_bytes <= byte_capacity {
        *approximate_bytes = candidate_bytes;
        return true;
    }
    for uri in touched {
        folded.remove(&uri);
    }
    for (uri, value) in previous {
        if let Some(value) = value {
            folded.insert(uri, value);
        }
    }
    false
}

fn touched_uris(event: &AssetWatchEvent) -> Vec<crate::asset::AssetUri> {
    match event {
        AssetWatchEvent::Added(uri)
        | AssetWatchEvent::Modified(uri)
        | AssetWatchEvent::Removed(uri) => vec![uri.clone()],
        AssetWatchEvent::Renamed { from, to } => vec![from.clone(), to.clone()],
    }
}

fn folded_entry_bytes(
    uri: &crate::asset::AssetUri,
    value: &(super::AssetChangeKind, Option<crate::asset::AssetUri>),
) -> usize {
    locator_bytes(uri)
        + value.1.as_ref().map(locator_bytes).unwrap_or_default()
        + std::mem::size_of_val(value)
}

fn locator_bytes(uri: &crate::asset::AssetUri) -> usize {
    uri.path().len() + uri.label().map(str::len).unwrap_or_default() + 12
}

fn approximate_notify_result_bytes(result: &notify::Result<Event>) -> usize {
    match result {
        Ok(event) => {
            std::mem::size_of::<Event>()
                + event
                    .paths
                    .iter()
                    .map(|path| path.as_os_str().len())
                    .sum::<usize>()
        }
        Err(error) => {
            std::mem::size_of::<notify::Error>()
                + error
                    .paths
                    .iter()
                    .map(|path| path.as_os_str().len())
                    .sum::<usize>()
        }
    }
}

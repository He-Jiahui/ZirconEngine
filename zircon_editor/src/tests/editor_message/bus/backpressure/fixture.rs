//! Shared message constructors and allocation instrumentation for backpressure tests.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorMessageRequest,
    EditorMessageResponse, EditorRequestHandler, FocusMessage, SelectionDomain,
};

pub(super) const MIXED_LOSSLESS_BACKLOG: u64 = 4_096;
pub(super) const MAX_PUBLISH_P95_NS: u64 = 50_000_000;

pub(super) fn selection_changed(domain: SelectionDomain, revision: u64) -> EditorMessage {
    EditorMessage::new(EditorMessagePayload::Focus(
        FocusMessage::SelectionChanged { domain, revision },
    ))
}

pub(super) fn document_opened(doc: DocumentId) -> EditorMessage {
    EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Opened {
        doc,
    }))
}

#[derive(Default)]
pub(super) struct CountingHandler {
    pub(super) calls: usize,
}

impl EditorRequestHandler for CountingHandler {
    fn handle_editor_request(&mut self, _request: &EditorMessageRequest) -> EditorMessageResponse {
        self.calls += 1;
        EditorMessageResponse::handled(document_opened(DocumentId::new(99)))
    }
}

#[derive(Default)]
pub(super) struct PayloadSharingHandler {
    pub(super) request: Option<EditorMessageRequest>,
}

impl EditorRequestHandler for PayloadSharingHandler {
    fn handle_editor_request(&mut self, request: &EditorMessageRequest) -> EditorMessageResponse {
        self.request = Some(request.clone());
        EditorMessageResponse::handled(document_opened(DocumentId::new(100)))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AllocationSample {
    pub(super) operations: u64,
    pub(super) bytes: u64,
}

impl AllocationSample {
    pub(super) fn accumulate(&mut self, sample: Self) {
        self.operations = self.operations.saturating_add(sample.operations);
        self.bytes = self.bytes.saturating_add(sample.bytes);
    }
}

pub(super) fn measure_allocations<T>(
    operation: impl FnOnce() -> T,
) -> (T, Duration, AllocationSample) {
    let tracking = AllocationTrackingGuard::start();
    let started = Instant::now();
    let output = operation();
    let elapsed = started.elapsed();
    drop(tracking);
    let sample = AllocationSample {
        operations: TRACKED_ALLOCATION_OPERATIONS.load(Ordering::Relaxed),
        bytes: TRACKED_ALLOCATION_BYTES.load(Ordering::Relaxed),
    };
    (output, elapsed, sample)
}

struct AllocationTrackingGuard;

impl AllocationTrackingGuard {
    fn start() -> Self {
        TRACKED_ALLOCATION_OPERATIONS.store(0, Ordering::Relaxed);
        TRACKED_ALLOCATION_BYTES.store(0, Ordering::Relaxed);
        TRACK_ALLOCATIONS.store(true, Ordering::Release);
        Self
    }
}

impl Drop for AllocationTrackingGuard {
    fn drop(&mut self) {
        TRACK_ALLOCATIONS.store(false, Ordering::Release);
    }
}

fn record_allocation(size: usize, pointer_is_valid: bool) {
    if pointer_is_valid && TRACK_ALLOCATIONS.load(Ordering::Acquire) {
        TRACKED_ALLOCATION_OPERATIONS.fetch_add(1, Ordering::Relaxed);
        TRACKED_ALLOCATION_BYTES
            .fetch_add(u64::try_from(size).unwrap_or(u64::MAX), Ordering::Relaxed);
    }
}

struct TrackingAllocator;

// SAFETY: every operation delegates to `System` with the original pointer/layout contract;
// the additional relaxed atomics do not retain or modify allocation addresses.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: delegated with the caller-provided valid allocation layout.
        let pointer = unsafe { System.alloc(layout) };
        record_allocation(layout.size(), !pointer.is_null());
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: delegated with the caller-provided valid allocation layout.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation(layout.size(), !pointer.is_null());
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: delegated with the caller-provided pointer, original layout, and new size.
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        record_allocation(new_size, !new_pointer.is_null());
        new_pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: delegated with the exact pointer/layout pair supplied by the caller.
        unsafe { System.dealloc(pointer, layout) };
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;
static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static TRACKED_ALLOCATION_OPERATIONS: AtomicU64 = AtomicU64::new(0);
static TRACKED_ALLOCATION_BYTES: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
pub(super) fn working_set_bytes() -> Option<u64> {
    let command = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8(output.stdout).ok()?.trim().parse().ok()
}

#[cfg(not(windows))]
pub(super) fn working_set_bytes() -> Option<u64> {
    None
}

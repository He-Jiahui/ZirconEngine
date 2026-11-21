//! Run each probe by exact name in a fresh native test process: the production
//! wrapper's process-wide counters are intentionally not resettable.
#![cfg(all(feature = "profiling-memory", not(target_family = "wasm")))]

use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use zircon_runtime::core::diagnostics::profiling::{
    memory::CountingAllocator, snapshot, start_capture, ProfileCaptureConfig, ProfileFrameScope,
};

const STREAM: &str = "realloc-regression";
const HELD_BYTES: usize = 1024;
const REJECTED_BYTES: usize = 16 * 1024;
const LIVE_FRAMES: usize = 8192;
const PEAK_PULSES: usize = 8192;

// Real System-backed blocks. The only policies supplied by this fixture are a
// successful in-place same-size resize, a forced move for other sizes, and a
// predictable failure without consuming the original block.
struct Backing;

// SAFETY: successful alloc/dealloc use the original System layout. Realloc
// either retains the old block, or allocates the new layout, copies its valid
// prefix and releases the old block; a null result leaves the old block valid.
unsafe impl GlobalAlloc for Backing {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, old: Layout, new_size: usize) -> *mut u8 {
        if new_size == REJECTED_BYTES {
            return ptr::null_mut();
        }
        if new_size == old.size() {
            return pointer;
        }
        let Ok(new) = Layout::from_size_align(new_size, old.align()) else {
            return ptr::null_mut();
        };
        let replacement = unsafe { System.alloc(new) };
        if !replacement.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(pointer, replacement, old.size().min(new_size));
                System.dealloc(pointer, old);
            }
        }
        replacement
    }
}

struct Block<'a> {
    allocator: &'a CountingAllocator<Backing>,
    pointer: NonNull<u8>,
    layout: Layout,
}

impl<'a> Block<'a> {
    fn allocate(allocator: &'a CountingAllocator<Backing>, size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let pointer = NonNull::new(unsafe { allocator.alloc(layout) })?;
        Some(Self {
            allocator,
            pointer,
            layout,
        })
    }

    fn try_resize(&mut self, size: usize) -> bool {
        let new = Layout::from_size_align(size, self.layout.align()).unwrap();
        let replacement = unsafe {
            self.allocator
                .realloc(self.pointer.as_ptr(), self.layout, size)
        };
        if let Some(pointer) = NonNull::new(replacement) {
            self.pointer = pointer;
            self.layout = new;
            true
        } else {
            false
        }
    }

    fn address(&self) -> usize {
        self.pointer.as_ptr() as usize
    }
}

impl Drop for Block<'_> {
    fn drop(&mut self) {
        unsafe { self.allocator.dealloc(self.pointer.as_ptr(), self.layout) };
    }
}

fn begin_capture(max_counters: usize) {
    assert!(
        start_capture(ProfileCaptureConfig {
            max_frames: 1,
            max_counters,
            ..ProfileCaptureConfig::default()
        })
        .active
    );
}

fn record_heap_frame() {
    drop(ProfileFrameScope::enter(STREAM, "heap"));
}

fn sampled_heap() -> [f64; 3] {
    record_heap_frame();
    let report = snapshot();
    let frame_index = Some(report.frames.last().expect("heap frame").frame_index);
    [
        "memory.live_bytes",
        "memory.peak_bytes",
        "memory.allocations",
    ]
    .map(|name| {
        report
            .counters
            .iter()
            .rev()
            .find(|row| row.stream == STREAM && row.name == name && row.frame_index == frame_index)
            .expect("heap counter on sampled frame")
            .value
    })
}

fn fresh_counters() {
    begin_capture(3);
    assert_eq!(
        sampled_heap(),
        [0.0; 3],
        "run one exact test in a fresh process"
    );
}

#[test]
fn realloc_accounting_controls() {
    fresh_counters();
    // Not the test executable's global allocator: recorder/libtest allocations
    // never use this instance or enter its production counters.
    let allocator = CountingAllocator(Backing);
    let mut a = Block::allocate(&allocator, 1024).expect("initial block");
    unsafe { a.pointer.as_ptr().write(0x5a) };
    assert_eq!(sampled_heap(), [1024.0, 1024.0, 1.0]);

    let b = Block::allocate(&allocator, 512).expect("overlapping block");
    assert_eq!(sampled_heap(), [1536.0, 1536.0, 2.0]);
    let old = a.address();
    assert!(a.try_resize(1536));
    assert_ne!(a.address(), old, "fixture must move on growth");
    assert_eq!(unsafe { a.pointer.as_ptr().read() }, 0x5a);
    assert_eq!(sampled_heap(), [2048.0, 2048.0, 3.0]);
    drop(b);
    assert_eq!(sampled_heap(), [1536.0, 2048.0, 3.0]);

    let old = a.address();
    assert!(a.try_resize(768));
    assert_ne!(a.address(), old, "fixture must move on shrink");
    assert_eq!(unsafe { a.pointer.as_ptr().read() }, 0x5a);
    assert_eq!(sampled_heap(), [768.0, 2048.0, 4.0]);
    let current = a.address();
    assert!(a.try_resize(768));
    assert_eq!(a.address(), current);
    assert_eq!(sampled_heap(), [768.0, 2048.0, 5.0]);
    assert!(!a.try_resize(REJECTED_BYTES));
    assert_eq!(a.address(), current);
    assert_eq!(a.layout.size(), 768);
    assert_eq!(unsafe { a.pointer.as_ptr().read() }, 0x5a);
    assert_eq!(sampled_heap(), [768.0, 2048.0, 5.0]);
    drop(a);
    assert_eq!(sampled_heap(), [0.0, 2048.0, 5.0]);
}

struct Stop<'a>(&'a AtomicBool);

impl Drop for Stop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

fn while_blocks_resize<T>(
    observe: impl FnOnce(&CountingAllocator<Backing>, usize) -> T,
) -> (T, usize) {
    let allocator = CountingAllocator(Backing);
    let stopped = AtomicBool::new(false);
    let completed = [AtomicUsize::new(0), AtomicUsize::new(0)];
    thread::scope(|scope| {
        let _stop = Stop(&stopped);
        for progress in &completed {
            let allocator = &allocator;
            let stopped = &stopped;
            scope.spawn(move || {
                let mut block = Block::allocate(allocator, HELD_BYTES).expect("held block");
                // Both real blocks stay owned throughout observation. Progress
                // is published only after whole calls, never within accounting.
                while !stopped.load(Ordering::Acquire) {
                    for _ in 0..64 {
                        assert!(block.try_resize(HELD_BYTES));
                    }
                    progress.fetch_add(64, Ordering::Release);
                }
            });
        }
        // Seeing progress from both workers establishes that both real blocks
        // are already allocated. It does not force a particular interleaving.
        let counts = || completed.each_ref().map(|n| n.load(Ordering::Acquire));
        let deadline = Instant::now() + Duration::from_secs(5);
        let before = loop {
            let counts = counts();
            if counts.into_iter().all(|n| n > 0) {
                break counts;
            }
            assert!(
                Instant::now() < deadline,
                "inconclusive: resize workers did not start"
            );
            thread::yield_now();
        };
        let result = observe(&allocator, completed.len() * HELD_BYTES);
        let resizes = counts()
            .into_iter()
            .zip(before)
            .map(|(after, before)| after - before)
            .sum();
        (result, resizes)
    })
}

#[test]
#[ignore = "observe natural concurrent realloc; run this test alone"]
fn realloc_live_bytes_never_lose_a_held_block() {
    fresh_counters();
    begin_capture(3 * LIVE_FRAMES);
    let ((floor, minimum, frames), resizes) = while_blocks_resize(|_, held_bytes| {
        for _ in 0..LIVE_FRAMES {
            record_heap_frame();
        }
        let report = snapshot();
        let live = report
            .counters
            .iter()
            .filter(|row| row.stream == STREAM && row.name == "memory.live_bytes");
        let frames = live.clone().count();
        let minimum = live.map(|row| row.value).fold(f64::INFINITY, f64::min);
        (held_bytes, minimum, frames)
    });
    assert_eq!(frames, LIVE_FRAMES);
    println!("realloc live: frames={frames}, completed_resizes={resizes}, minimum={minimum}, held={floor}");
    assert!(
        minimum >= floor as f64,
        "held bytes disappeared from a real profiler row"
    );
    assert!(
        resizes > 0,
        "inconclusive: no resize batch completed during sampling"
    );
    assert_eq!(sampled_heap()[0], 0.0, "all held blocks have been freed");
}

#[test]
#[ignore = "observe natural concurrent realloc; run this test alone"]
fn realloc_peak_bytes_remember_a_short_overlap() {
    fresh_counters();
    let ((pulses, expected, sampled), resizes) = while_blocks_resize(|allocator, held_bytes| {
        for pulse in 1..=PEAK_PULSES {
            // Strictly increase the requested bytes on each pulse so the
            // previous lifetime peak cannot hide a miss on this pulse.
            let size = 64 + pulse;
            let block = Block::allocate(allocator, size).expect("pulse block");
            drop(block);
            let [_, peak, _] = sampled_heap();
            let expected = held_bytes + size;
            if peak != expected as f64 || pulse == PEAK_PULSES {
                return (pulse, expected, peak);
            }
        }
        unreachable!()
    });
    // A worker may have computed the correct peak during the last pulse but
    // not published it when our first read occurred. Join all reallocs and
    // sample again before counting it as lost history, with no later pulse.
    let [live, retained, _] = sampled_heap();
    println!("realloc peak: pulses={pulses}, completed_resizes={resizes}, sampled={sampled}, retained={retained}, expected={expected}");
    assert_eq!(live, 0.0, "all pulse and held blocks have been freed");
    assert_eq!(
        retained, expected as f64,
        "wrong retained peak after all real allocation overlaps completed"
    );
    assert!(
        pulses == PEAK_PULSES && resizes > 0,
        "inconclusive: stopped early without a retained mismatch or concurrent progress"
    );
}

//! Compare the admitted bounds with allocation requests from the actual linked std.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use zircon_runtime_interface::retained_memory::*;

struct TrackingAllocator;
thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static LIVE_BYTES: Cell<isize> = const { Cell::new(0) };
}
fn charge(bytes: isize) {
    if TRACKING.try_with(Cell::get).unwrap_or(false) {
        let _ = LIVE_BYTES.try_with(|live| live.set(live.get() + bytes));
    }
}
// SAFETY: Every request is delegated unchanged to System; accounting never accesses payloads.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: GlobalAlloc callers provide the valid nonzero allocation layout.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            charge(layout.size() as isize);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        charge(-(layout.size() as isize));
        // SAFETY: The pointer and original layout are forwarded to their owning System allocator.
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: GlobalAlloc callers provide the live System allocation and valid new size.
        let pointer = unsafe { System.realloc(pointer, layout, size) };
        if !pointer.is_null() {
            charge(size as isize - layout.size() as isize);
        }
        pointer
    }
}
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn allocated<T>(create: impl FnOnce() -> T) -> (T, u64) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TRACKING.with(|tracking| tracking.set(false));
        }
    }
    LIVE_BYTES.with(|live| live.set(0));
    TRACKING.with(|tracking| tracking.set(true));
    let reset = Reset;
    let value = create();
    drop(reset);
    let bytes = LIVE_BYTES.with(Cell::get);
    assert!(bytes >= 0, "measurement must own all allocations it frees");
    (value, bytes as u64)
}

#[repr(align(64))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Aligned(u64);
zircon_runtime_interface::impl_retained_owner_v1!(Aligned { 0 });
// SAFETY: The only field is an allocation-free integer with no interior mutation.
unsafe impl ImmutableRetainedOwnerV1 for Aligned {}

#[test]
fn admitted_model_bounds_real_tree_arc_and_buffer_allocations() {
    let model = verified_std_model_v1().expect("run with an audited ordinary-number toolchain");
    fn check<T: RetainedOwnerV1>(model: RetainedAccountingModelV1, create: impl FnOnce() -> T) {
        let (value, requested) = allocated(create);
        let bound = retained_bound_v1(&value, model).unwrap().max_bytes.get();
        assert!(
            bound >= requested,
            "bound {bound} underestimated {requested} requested bytes"
        );
    }
    for len in [0, 1, 11, 12, 64, 512] {
        check(model, || {
            (0..len)
                .map(|i| (Aligned(i), [0u8; 17]))
                .collect::<BTreeMap<_, _>>()
        });
        check(model, || {
            let mut tree = (0..len).map(|i| (i, i)).collect::<BTreeMap<_, _>>();
            tree.retain(|key, _| *key == 0);
            tree
        });
        check(model, || VecDeque::<u64>::with_capacity(len as usize));
        check(model, || Vec::<u8>::with_capacity(len as usize));
        check(model, || Arc::<str>::from("x".repeat(len as usize)));
    }
    check(model, || Arc::new(Aligned(42)));
    check(model, || {
        let mut text = std::ffi::OsString::with_capacity(4096);
        text.push("/project/assets");
        text
    });
}

#[test]
fn fresh_hash_certificate_bounds_actual_small_large_and_aligned_tables() {
    verified_std_model_v1().expect("run with an audited ordinary-number toolchain");
    fn check<T>(
        capacity: usize,
        create: impl FnOnce() -> T,
        certificate: FreshHashStorageV1,
        bytes: u64,
    ) {
        let (value, requested) = allocated(create);
        assert!(
            bytes >= requested,
            "capacity {capacity}, {certificate:?}: {bytes} < {requested}"
        );
        std::hint::black_box(value);
    }
    macro_rules! table {
        ($element:ty, $create:expr) => {
            for capacity in [0, 1, 2, 3, 4, 7, 8, 14, 15, 28, 29, 64, 1024] {
                let certificate = FreshHashStorageV1::for_request::<$element>(capacity).unwrap();
                check(
                    capacity,
                    || $create(capacity),
                    certificate,
                    certificate.allocation::<$element>().unwrap(),
                );
            }
        };
    }
    table!((), HashSet::<()>::with_capacity);
    table!(u8, HashSet::<u8>::with_capacity);
    table!(u16, HashSet::<u16>::with_capacity);
    table!((u64, u64), HashMap::<u64, u64>::with_capacity);
    // The certificate uses the actual pair layout, including its 64-byte alignment.
    table!((Aligned, u8), HashMap::<Aligned, u8>::with_capacity);
}

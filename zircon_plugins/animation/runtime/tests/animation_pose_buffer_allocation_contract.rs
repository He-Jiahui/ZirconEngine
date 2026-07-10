use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use zircon_plugin_animation_runtime::{PoseBuffer, PosePool};
use zircon_runtime::core::math::{Quat, Transform, Vec3};

struct CountingAllocator;

static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        System.alloc_zeroed(layout)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout)
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        System.realloc(pointer, layout, size)
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn pooled_pose_buffer_blend_performs_zero_allocations() {
    const JOINTS: usize = 128;
    let mut pool = PosePool::with_buffers(2, JOINTS);

    run_pooled_evaluation(&mut pool, JOINTS);
    let allocations_before = ALLOCATION_COUNT.load(Ordering::SeqCst);
    for _ in 0..256 {
        run_pooled_evaluation(&mut pool, JOINTS);
    }
    let allocations_after = ALLOCATION_COUNT.load(Ordering::SeqCst);

    assert_eq!(allocations_after, allocations_before);
    assert_eq!(pool.miss_count(), 0);
    assert_eq!(pool.available_count(), 2);
}

fn run_pooled_evaluation(pool: &mut PosePool, joints: usize) {
    let mut base = pool.acquire(joints);
    let mut source = pool.acquire(joints);
    fill_source(&mut source, joints);
    base.blend_override(&source, 0.5).unwrap();
    base.accumulate_additive(&source, 0.25).unwrap();
    pool.release(source);
    pool.release(base);
}

fn fill_source(source: &mut PoseBuffer, joints: usize) {
    for index in 0..joints {
        source
            .set_transform(
                index,
                Transform {
                    translation: Vec3::splat(index as f32 * 0.01),
                    rotation: Quat::from_rotation_y(index as f32 * 0.001),
                    scale: Vec3::splat(1.0 + index as f32 * 0.0001),
                },
            )
            .unwrap();
    }
}

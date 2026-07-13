use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use zircon_plugin_animation_runtime::{PoseBuffer, PosePool};
use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Quat, Transform, Vec3};

struct CountingAllocator;

thread_local! {
    static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        System.alloc_zeroed(layout)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout)
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation();
        System.realloc(pointer, layout, size)
    }
}

fn record_allocation() {
    let _ = COUNT_ALLOCATIONS.try_with(|enabled| {
        if enabled.get() {
            let _ = ALLOCATION_COUNT.try_with(|count| count.set(count.get() + 1));
        }
    });
}

fn count_allocations(operation: impl FnOnce()) -> usize {
    ALLOCATION_COUNT.set(0);
    COUNT_ALLOCATIONS.set(true);
    operation();
    COUNT_ALLOCATIONS.set(false);
    ALLOCATION_COUNT.get()
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn pooled_pose_buffer_blend_performs_zero_allocations() {
    const JOINTS: usize = 128;
    let mut pool = PosePool::with_buffers(2, JOINTS);

    run_pooled_evaluation(&mut pool, JOINTS);
    let allocation_count = count_allocations(|| {
        for _ in 0..256 {
            run_pooled_evaluation(&mut pool, JOINTS);
        }
    });

    assert_eq!(allocation_count, 0);
    assert_eq!(pool.miss_count(), 0);
    assert_eq!(pool.available_count(), 2);
}

#[test]
fn final_pose_output_clone_reuses_bone_and_name_storage() {
    let mut target = final_pose(1.0);
    let source = final_pose(2.0);
    let bones_pointer = target.bones.as_ptr();
    let name_pointers = target
        .bones
        .iter()
        .map(|bone| bone.name.as_ptr())
        .collect::<Vec<_>>();
    let allocation_count = count_allocations(|| target.clone_from_reusing_storage(&source));

    assert_eq!(allocation_count, 0);
    assert_eq!(target.bones.as_ptr(), bones_pointer);
    assert_eq!(
        target
            .bones
            .iter()
            .map(|bone| bone.name.as_ptr())
            .collect::<Vec<_>>(),
        name_pointers
    );
    assert_eq!(target.bones[1].local_transform.translation.x, 2.0);
}

fn final_pose(hand_x: f32) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::StateMachine,
        active_state: Some("Move".to_string()),
        bones: vec![
            AnimationPoseBone {
                name: "Root".to_string(),
                local_transform: Transform::default(),
            },
            AnimationPoseBone {
                name: "Root/Hand".to_string(),
                local_transform: Transform {
                    translation: Vec3::new(hand_x, 0.0, 0.0),
                    ..Transform::default()
                },
            },
        ],
    }
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

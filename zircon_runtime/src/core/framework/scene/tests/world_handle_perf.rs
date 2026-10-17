use super::WorldHandle;

/// 验证句柄按数值键排序，此测试没有耗时测量。
#[test]
fn optimization_batch_ic_runtime612_world_handles_have_total_order() {
    let mut handles = [
        WorldHandle::new(9),
        WorldHandle::new(2),
        WorldHandle::new(5),
    ];

    handles.sort_unstable();

    assert_eq!(
        handles,
        [
            WorldHandle::new(2),
            WorldHandle::new(5),
            WorldHandle::new(9),
        ]
    );
}

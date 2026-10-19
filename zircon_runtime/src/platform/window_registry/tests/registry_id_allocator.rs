//! 身份分配器的单计数器耗尽边界夹具。
//! 只断言顺序返回值；不覆盖并发竞争、全局计数器或 driver 生命周期。

use std::sync::atomic::AtomicU64;

use super::allocate_from;

/// 验证最大非零值只发出一次，后续调用持续返回无身份。
/// 使用独立局部原子，不消耗进程全局身份。
#[test]
fn allocator_issues_the_final_nonzero_identity_once_then_reports_exhaustion() {
    let next_window_registry_id = AtomicU64::new(u64::MAX);

    // 第一次返回旧的最大值；接着两次空结果同时约束耗尽保持状态。
    assert_eq!(
        allocate_from(&next_window_registry_id).map(|identity| identity.raw()),
        Some(u64::MAX)
    );
    assert_eq!(allocate_from(&next_window_registry_id), None);
    assert_eq!(allocate_from(&next_window_registry_id), None);
}

//! 窗口注册表的进程内身份分配，供平台 driver 创建独立注册表。
//! 身份来自共享原子计数器；driver 重建不会回收旧值，数值耗尽后持续拒绝。
//! 这里只分配身份，不创建窗口，也不承载窗口状态的同步。

use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::framework::window::WindowRegistryId;

// 0 标记身份永久耗尽；最后一个非零值发出后不再分配，避免新 driver 复用旧 registry 身份。
static NEXT_WINDOW_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

/// 为一次 driver 构建取得新的注册表身份。
///
/// 返回本进程计数器尚未发出的非零身份；耗尽时返回 `None`。
/// 结果按值独立于计数器，调用期间的共享原子访问不发布窗口状态。
/// 调用方见 `PlatformDriver::from_preference_storage`；偏好适配器不提供此身份。
/// Allocates one process-unique platform-host identity without reusing a
/// value after a driver has been torn down. Driver construction is cold, so
/// relaxed atomic ordering is sufficient: uniqueness is the only shared fact.
pub(in crate::platform) fn allocate_window_registry_id() -> Option<WindowRegistryId> {
    allocate_from(&NEXT_WINDOW_REGISTRY_ID)
}

/// 从借入的计数器取出一个身份；全局入口与边界夹具共用此操作。
///
/// `next_window_registry_id` 是下一待发值，0 表示已耗尽；调用者须保持同一身份域不重置计数。
/// 返回旧的非零计数值，成功后推进计数器；最后一个值发出时将计数器置 0。
/// 借用只持续到返回，不保留原子引用；不同计数器之间没有去重保证。
fn allocate_from(next_window_registry_id: &AtomicU64) -> Option<WindowRegistryId> {
    let raw = next_window_registry_id
        // 更新闭包可能因竞争重试，只计算后继值，不对窗口或身份域执行副作用。
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            // 已耗尽时拒绝本次更新；下方的 `.ok()?` 将该失败转为无身份。
            if current == 0 {
                return None;
            }
            // 发出的仍是旧值；后继溢出时写入 0，避免回绕到已发出的身份。
            Some(match current.checked_add(1) {
                Some(next) => next,
                None => 0,
            })
        })
        .ok()?;
    // 这里只有成功更新取得的非零旧值；包装器自身只检查非零。
    WindowRegistryId::new(raw)
}

#[cfg(test)]
#[path = "tests/registry_id_allocator.rs"]
mod tests;

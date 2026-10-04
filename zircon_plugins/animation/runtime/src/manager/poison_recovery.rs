//! 动画管理器从已中毒互斥锁继续读取状态；调用端需自行判断此前失败是否留下可用数据。
use std::sync::{Mutex, MutexGuard};

pub(super) fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

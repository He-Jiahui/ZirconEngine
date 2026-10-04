use std::sync::{Mutex, MutexGuard};

// TODO: [CR-SOUND-AUDIT-0007] 核实持锁操作中途 panic 后状态不变量是否仍成立；into_inner 仅取出当时内容，无快照回退，声音状态测试未覆盖部分更新后崩溃。
/// 获取中毒互斥锁内的当前值；持锁 panic 后不保证数据仍满足原有不变量。
pub(crate) fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

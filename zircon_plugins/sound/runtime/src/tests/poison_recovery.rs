// 用 Mutex<u32> 验证 panic 后取得锁内当时内容；不证明声音状态不变量或公共管理器完整恢复。
use std::sync::{Arc, Mutex};

use crate::poison_recovery::lock_recover;

#[test]
fn poisoned_sound_mutex_recovers_last_value() {
    let value = Arc::new(Mutex::new(7_u32));
    let poison_target = Arc::clone(&value);
    let _ = std::panic::catch_unwind(move || {
        let mut guard = poison_target.lock().unwrap();
        *guard = 11;
        panic!("poison sound mutex for recovery coverage");
    });

    assert_eq!(*lock_recover(&value), 11);
}

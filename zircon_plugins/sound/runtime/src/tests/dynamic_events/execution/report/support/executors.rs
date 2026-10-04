// 对两个处理器分别模拟成功与业务失败，共享调用日志供报告用例核对顺序；第三个处理器故意不注册执行器。
use super::super::super::super::*;

use std::sync::{Arc, Mutex};

pub(super) fn register_executors(sound: &DefaultSoundManager, calls: &Arc<Mutex<Vec<String>>>) {
    let analytics_calls = calls.clone();
    sound
        .register_dynamic_event_executor("analytics", "combat-counter", move |delivery| {
            analytics_calls
                .lock()
                .unwrap()
                .push(delivery.handler.plugin_id.clone());
            Ok(())
        })
        .unwrap();
    let gameplay_calls = calls.clone();
    sound
        .register_dynamic_event_executor("gameplay_audio", "weapon-foley", move |delivery| {
            gameplay_calls
                .lock()
                .unwrap()
                .push(delivery.handler.plugin_id.clone());
            Err("foley unavailable".to_string())
        })
        .unwrap();
}

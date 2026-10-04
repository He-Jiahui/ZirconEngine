// 组合已注册事件、处理器、执行器和待投递调用，供报告顺序与排空用例共用同一输入。
use super::super::super::super::*;

use std::sync::{Arc, Mutex};

use super::executors::register_executors;
use super::registration::{register_event, register_handlers};
use super::submission::submit_event;

pub(crate) struct ReportFixture {
    pub(crate) sound: DefaultSoundManager,
    pub(crate) calls: Arc<Mutex<Vec<String>>>,
}

pub(crate) fn report_fixture() -> ReportFixture {
    let sound = DefaultSoundManager::default();
    register_event(&sound);
    register_handlers(&sound);
    let calls = Arc::new(Mutex::new(Vec::new()));
    register_executors(&sound, &calls);
    submit_event(&sound);
    ReportFixture { sound, calls }
}

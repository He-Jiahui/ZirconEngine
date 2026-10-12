//! 汇集调度、消息泵、进度、配额和线程所有权契约，记录任务夹具用于验证可控执行顺序。
#[path = "admission_scaling_contract.rs"]
mod admission_scaling_contract;
#[path = "background_storm_contract.rs"]
mod background_storm_contract;
#[path = "progress_contract.rs"]
mod progress_contract;
#[path = "pump_contract.rs"]
mod pump_contract;
#[path = "quota_settings_contract.rs"]
mod quota_settings_contract;
#[path = "scheduling_contract.rs"]
mod scheduling_contract;
#[path = "thread_ownership_contract.rs"]
mod thread_ownership_contract;

use std::sync::{Arc, Mutex};

use super::{EditorJob, JobContext, JobError};

struct RecordingJob {
    label: &'static str,
    order: Arc<Mutex<Vec<&'static str>>>,
}

impl RecordingJob {
    fn new(label: &'static str, order: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self { label, order }
    }
}

impl EditorJob for RecordingJob {
    type Output = &'static str;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.order.lock().unwrap().push(self.label);
        Ok(self.label)
    }
}

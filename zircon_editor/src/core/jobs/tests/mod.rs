mod admission_scaling_contract;
mod background_storm_contract;
mod progress_contract;
mod pump_contract;
mod quota_settings_contract;
mod scheduling_contract;
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

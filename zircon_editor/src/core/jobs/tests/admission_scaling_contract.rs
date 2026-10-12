pub(super) use std::sync::mpsc::{self, Receiver, Sender};
pub(super) use std::time::{Duration, Instant};

pub(super) use super::super::{
    test_job_system_with_limits, CancellationToken, EditorJob, EditorJobAdmission,
    EditorJobAdmissionKey, EditorJobAdmissionLimits, EditorJobAdmissionRequest, EditorJobLimits,
    EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, JobSubmitError,
};

#[path = "admission_scaling_contract/indexed.rs"]
mod indexed;
#[path = "admission_scaling_contract/keyed.rs"]
mod keyed;
#[path = "admission_scaling_contract/reservation.rs"]
mod reservation;
#[path = "admission_scaling_contract/support.rs"]
mod support;

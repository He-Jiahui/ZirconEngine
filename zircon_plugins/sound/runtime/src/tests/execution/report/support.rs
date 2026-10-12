#[path = "support/executors.rs"]
mod executors;
#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/ids.rs"]
mod ids;
#[path = "support/registration.rs"]
mod registration;
#[path = "support/submission.rs"]
mod submission;

pub(super) use fixture::report_fixture;

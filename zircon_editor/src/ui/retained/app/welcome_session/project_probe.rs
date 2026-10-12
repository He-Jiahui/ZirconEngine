#[path = "project_probe/host.rs"]
mod host;
#[path = "project_probe/job.rs"]
mod job;
#[path = "project_probe/projection.rs"]
mod projection;
#[path = "project_probe/state.rs"]
mod state;

pub(in crate::ui::retained_host::app) use state::WelcomeProjectProbeState;

#[cfg(test)]
#[path = "project_probe/tests/cases.rs"]
mod tests;

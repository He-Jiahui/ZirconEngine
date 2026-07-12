use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::coordinator_client::{ActionRecord, CoordinatorClient};
use crate::process_identity::{inspect_process, terminate_after_reverification};
use crate::runtime_descriptor::RuntimeDescriptor;
use crate::tray_state::SupervisionState;
use crate::TrayError;

const ACTION_POLL_INTERVAL: Duration = Duration::from_millis(100);
const FORCE_STOP_ACK_GRACE: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleAction {
    Drain,
    Resume,
    Stop,
    Restart,
    ForceStop,
}

impl LifecycleAction {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Drain => "service.drain",
            Self::Resume => "service.resume",
            Self::Stop => "service.stop",
            Self::Restart => "service.restart",
            Self::ForceStop => "service.force_stop",
        }
    }
}

pub fn request(
    client: &CoordinatorClient<'_>,
    action: LifecycleAction,
    timeout_seconds: u32,
    reason: &str,
) -> Result<ActionRecord, TrayError> {
    let preview = client.preview_lifecycle(action.kind(), timeout_seconds)?;
    let phrase = preview
        .confirmation_phrase
        .as_deref()
        .ok_or(TrayError::InvalidDescriptor(
            "lifecycle preview omitted confirmation phrase",
        ))?;
    client.confirm_action(&preview.action_id, phrase, reason)
}

pub fn wait_for_terminal(
    client: &CoordinatorClient<'_>,
    action_id: &str,
    timeout: Duration,
) -> Result<ActionRecord, TrayError> {
    let deadline = Instant::now() + timeout;
    wait_for_terminal_from(
        client.action(action_id)?,
        || client.action(action_id),
        deadline,
        ACTION_POLL_INTERVAL,
    )
}

pub fn is_terminal_status(status: &str) -> bool {
    matches!(
        status,
        "succeeded" | "failed" | "cancelled" | "expired" | "state_changed" | "denied"
    )
}

fn wait_for_terminal_from(
    mut record: ActionRecord,
    mut poll: impl FnMut() -> Result<ActionRecord, TrayError>,
    deadline: Instant,
    poll_interval: Duration,
) -> Result<ActionRecord, TrayError> {
    loop {
        if is_terminal_status(&record.status) {
            return Ok(record);
        }
        if Instant::now() >= deadline {
            return Err(TrayError::Http("lifecycle action polling timed out".into()));
        }
        thread::sleep(poll_interval.min(deadline.saturating_duration_since(Instant::now())));
        record = poll()?;
    }
}

pub fn force_stop(
    client: &CoordinatorClient<'_>,
    descriptor: &RuntimeDescriptor,
    timeout_seconds: u32,
) -> Result<(), TrayError> {
    inspect_process(descriptor.pid)?.verify(descriptor)?;
    client.verify_health()?;
    let accepted = request(
        client,
        LifecycleAction::ForceStop,
        timeout_seconds,
        "tray advanced recovery",
    )?;
    force_stop_confirmed(
        client,
        descriptor,
        accepted,
        Duration::from_secs(u64::from(timeout_seconds) + 5),
    )
}

/// Completes an already-confirmed force-stop without treating `executing` as kill authority.
///
/// The server must first persist the action and lifecycle intent as `succeeded`. A native
/// termination is considered only while authenticated health reports `offline` and the exact
/// process identity still matches the runtime descriptor.
pub fn force_stop_confirmed(
    client: &CoordinatorClient<'_>,
    descriptor: &RuntimeDescriptor,
    accepted: ActionRecord,
    timeout: Duration,
) -> Result<(), TrayError> {
    if accepted.kind != LifecycleAction::ForceStop.kind() || accepted.action_id.is_empty() {
        return Err(TrayError::InvalidDescriptor(
            "force-stop confirmation returned the wrong action identity",
        ));
    }
    let action_id = accepted.action_id.clone();
    let acknowledgement_id = action_id.clone();
    finish_confirmed_force_stop(
        accepted,
        || client.action(&action_id),
        || observe_force_stop_shutdown(client, descriptor),
        || client.acknowledge_force_stop(&acknowledgement_id),
        || terminate_after_reverification(descriptor),
        timeout,
        ACTION_POLL_INTERVAL,
        FORCE_STOP_ACK_GRACE,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ForceStopObservation {
    ProcessExitedOrIdentityChanged,
    Pending,
    VerifiedOffline,
}

fn observe_force_stop_shutdown(
    client: &CoordinatorClient<'_>,
    descriptor: &RuntimeDescriptor,
) -> Result<ForceStopObservation, TrayError> {
    match client.verify_health() {
        Ok(health) => {
            inspect_process(descriptor.pid)?.verify(descriptor)?;
            if health.supervision.state == SupervisionState::Offline {
                Ok(ForceStopObservation::VerifiedOffline)
            } else {
                Ok(ForceStopObservation::Pending)
            }
        }
        Err(_health_error) => match inspect_process(descriptor.pid) {
            Ok(identity) => {
                identity.verify(descriptor)?;
                Ok(ForceStopObservation::Pending)
            }
            Err(_) => Ok(ForceStopObservation::ProcessExitedOrIdentityChanged),
        },
    }
}

fn finish_confirmed_force_stop(
    accepted: ActionRecord,
    mut poll_action: impl FnMut() -> Result<ActionRecord, TrayError>,
    mut observe_shutdown: impl FnMut() -> Result<ForceStopObservation, TrayError>,
    mut acknowledge: impl FnMut() -> Result<(), TrayError>,
    mut terminate: impl FnMut() -> Result<(), TrayError>,
    timeout: Duration,
    poll_interval: Duration,
    acknowledgement_grace: Duration,
) -> Result<(), TrayError> {
    let deadline = Instant::now() + timeout;
    let terminal = wait_for_terminal_from(accepted, &mut poll_action, deadline, poll_interval)?;
    if terminal.status != "succeeded" {
        return Err(TrayError::Coordinator {
            code: terminal
                .error_code
                .unwrap_or_else(|| format!("lifecycle_action_{}", terminal.status)),
            message: "Force-stop action did not succeed".into(),
        });
    }
    loop {
        match observe_shutdown()? {
            ForceStopObservation::ProcessExitedOrIdentityChanged => return Ok(()),
            ForceStopObservation::VerifiedOffline => break,
            ForceStopObservation::Pending => {}
        }
        if Instant::now() >= deadline {
            return Err(TrayError::Http(
                "force-stop reached succeeded but the verified service did not go offline".into(),
            ));
        }
        thread::sleep(poll_interval.min(deadline.saturating_duration_since(Instant::now())));
    }
    acknowledge()?;
    let acknowledgement_deadline = deadline.min(Instant::now() + acknowledgement_grace);
    loop {
        if matches!(
            observe_shutdown()?,
            ForceStopObservation::ProcessExitedOrIdentityChanged
        ) {
            return Ok(());
        }
        if Instant::now() >= acknowledgement_deadline {
            return terminate();
        }
        thread::sleep(
            poll_interval.min(acknowledgement_deadline.saturating_duration_since(Instant::now())),
        );
    }
}

pub fn start_hidden(repo_root: &Path) -> Result<(), TrayError> {
    let script = repo_root.join("tools").join("zircon-session.ps1");
    if !script.is_file() {
        return Err(TrayError::InvalidDescriptor("start script is missing"));
    }
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ]);
    command.arg(script);
    command.arg("start");
    command.arg("-RepoRoot");
    command.arg(repo_root);
    command.arg("-Json");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde_json::Value;

    use super::*;

    fn action(status: &str, error_code: Option<&str>) -> ActionRecord {
        ActionRecord {
            action_id: "force-stop-action".into(),
            kind: "service.force_stop".into(),
            status: status.into(),
            confirmation_phrase: None,
            impact: None,
            warnings: Vec::new(),
            expires_at: None,
            result: None::<Value>,
            error_code: error_code.map(str::to_owned),
        }
    }

    #[test]
    fn executing_force_stop_waits_for_succeeded_before_termination() {
        let mut polls = VecDeque::from([action("executing", None), action("succeeded", None)]);
        let terminations = AtomicUsize::new(0);
        let acknowledgements = AtomicUsize::new(0);

        let result = finish_confirmed_force_stop(
            action("executing", None),
            || Ok(polls.pop_front().expect("poll record")),
            || Ok(ForceStopObservation::VerifiedOffline),
            || {
                acknowledgements.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            || {
                terminations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            Duration::from_millis(25),
            Duration::ZERO,
            Duration::ZERO,
        );

        assert!(result.is_ok());
        assert_eq!(1, acknowledgements.load(Ordering::SeqCst));
        assert_eq!(1, terminations.load(Ordering::SeqCst));
    }

    #[test]
    fn failed_force_stop_never_terminates_process() {
        let terminations = AtomicUsize::new(0);

        let result = finish_confirmed_force_stop(
            action("failed", Some("lifecycle_drain_timeout")),
            || panic!("terminal action must not be polled"),
            || panic!("failed action must not inspect shutdown state"),
            || panic!("failed action must not acknowledge shutdown"),
            || {
                terminations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            Duration::from_millis(25),
            Duration::ZERO,
            Duration::ZERO,
        );

        assert!(matches!(
            result,
            Err(TrayError::Coordinator { ref code, .. }) if code == "lifecycle_drain_timeout"
        ));
        assert_eq!(0, terminations.load(Ordering::SeqCst));
    }

    #[test]
    fn force_stop_poll_timeout_never_terminates_process() {
        let terminations = AtomicUsize::new(0);

        let result = finish_confirmed_force_stop(
            action("executing", None),
            || Ok(action("executing", None)),
            || panic!("non-terminal action must not inspect shutdown state"),
            || panic!("non-terminal action must not acknowledge shutdown"),
            || {
                terminations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        );

        assert!(
            matches!(result, Err(TrayError::Http(ref message)) if message.contains("timed out"))
        );
        assert_eq!(0, terminations.load(Ordering::SeqCst));
    }

    #[test]
    fn succeeded_force_stop_without_verified_offline_never_terminates_process() {
        let terminations = AtomicUsize::new(0);

        let result = finish_confirmed_force_stop(
            action("succeeded", None),
            || panic!("terminal action must not be polled"),
            || Ok(ForceStopObservation::Pending),
            || panic!("unverified offline state must not acknowledge shutdown"),
            || {
                terminations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        );

        assert!(
            matches!(result, Err(TrayError::Http(ref message)) if message.contains("did not go offline"))
        );
        assert_eq!(0, terminations.load(Ordering::SeqCst));
    }

    #[test]
    fn force_stop_acknowledgement_failure_never_terminates_process() {
        let terminations = AtomicUsize::new(0);

        let result = finish_confirmed_force_stop(
            action("succeeded", None),
            || panic!("terminal action must not be polled"),
            || Ok(ForceStopObservation::VerifiedOffline),
            || Err(TrayError::Http("ack transport failed".into())),
            || {
                terminations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            Duration::from_millis(25),
            Duration::ZERO,
            Duration::ZERO,
        );

        assert!(matches!(result, Err(TrayError::Http(ref message)) if message.contains("ack")));
        assert_eq!(0, terminations.load(Ordering::SeqCst));
    }
}

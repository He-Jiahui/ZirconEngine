use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::coordinator_client::{ActionRecord, CoordinatorClient};
use crate::process_identity::{inspect_process, terminate_after_reverification};
use crate::runtime_descriptor::RuntimeDescriptor;
use crate::TrayError;

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
    loop {
        let record = client.action(action_id)?;
        if matches!(
            record.status.as_str(),
            "succeeded" | "failed" | "cancelled" | "expired" | "state_changed" | "denied"
        ) {
            return Ok(record);
        }
        if Instant::now() >= deadline {
            return Err(TrayError::Http("lifecycle action polling timed out".into()));
        }
        thread::sleep(Duration::from_millis(100));
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
    if accepted.status == "succeeded" {
        return Ok(());
    }
    client.verify_health()?;
    inspect_process(descriptor.pid)?.verify(descriptor)?;
    terminate_after_reverification(descriptor)
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

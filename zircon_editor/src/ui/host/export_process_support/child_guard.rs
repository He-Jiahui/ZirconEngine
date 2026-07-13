use std::process::Child;

use super::process_tree::terminate_process_tree;

/// Ensures an unwinding export job cannot detach a still-running child process.
pub(in crate::ui::host) struct ExportProcessChildGuard {
    child: Child,
    label: String,
    armed: bool,
}

impl ExportProcessChildGuard {
    pub(in crate::ui::host) fn new(child: Child, label: impl Into<String>) -> Self {
        Self {
            child,
            label: label.into(),
            armed: true,
        }
    }

    pub(in crate::ui::host) fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    pub(in crate::ui::host) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ExportProcessChildGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.child.try_wait().is_ok_and(|status| status.is_some()) {
            return;
        }
        let termination = terminate_process_tree(&mut self.child, &self.label);
        if termination.succeeded {
            let _ = self.child.wait();
        }
    }
}

#[cfg(all(test, any(windows, unix)))]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::super::process_tree::configure_process_tree_cancellation;
    use super::ExportProcessChildGuard;

    #[test]
    fn dropping_guard_terminates_and_reaps_the_process_tree() {
        let sentinel = unique_sentinel_path();
        let ready = sentinel.with_extension("ready");
        let _ = fs::remove_file(&sentinel);
        let _ = fs::remove_file(&ready);
        let mut command = process_tree_command(&ready, &sentinel);
        command.stdout(Stdio::null()).stderr(Stdio::null());
        configure_process_tree_cancellation(&mut command);
        let child = command.spawn().expect("process-tree fixture should start");
        let parent_id = child.id();

        {
            let _guard = ExportProcessChildGuard::new(child, "child-guard drop contract");
            wait_for_path(&ready, Duration::from_secs(10));
        }

        std::thread::sleep(Duration::from_secs(3));
        assert!(
            !process_is_running(parent_id),
            "guard drop must reap the parent process"
        );
        assert!(
            !sentinel.exists(),
            "guard drop must terminate descendants before they can outlive the parent"
        );
        let _ = fs::remove_file(sentinel);
        let _ = fs::remove_file(ready);
    }

    fn unique_sentinel_path() -> PathBuf {
        let sequence = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zircon-export-child-guard-{}-{sequence}.txt",
            std::process::id()
        ))
    }

    #[cfg(windows)]
    fn process_tree_command(ready: &Path, sentinel: &Path) -> Command {
        let ready = ready.to_string_lossy().replace(char::from(39), "''");
        let path = sentinel.to_string_lossy().replace(char::from(39), "''");
        let descendant = format!(
            "Set-Content -LiteralPath '{ready}' -Value ready; Start-Sleep -Seconds 2; Set-Content -LiteralPath '{path}' -Value survived"
        );
        let descendant = descendant.replace(char::from(39), "''");
        let script = format!(
            "$child = Start-Process -FilePath powershell -ArgumentList @('-NoProfile','-Command','{descendant}') -PassThru; Start-Sleep -Seconds 30"
        );
        let mut command = Command::new("powershell");
        command.args(["-NoProfile", "-Command", &script]);
        command
    }

    #[cfg(unix)]
    fn process_tree_command(ready: &Path, sentinel: &Path) -> Command {
        let ready = ready.to_string_lossy().replace(char::from(39), "'\\''");
        let path = sentinel.to_string_lossy().replace(char::from(39), "'\\''");
        let script =
            format!("(printf ready > '{ready}'; sleep 2; printf survived > '{path}') & sleep 30");
        let mut command = Command::new("sh");
        command.args(["-c", &script]);
        command
    }

    #[cfg(windows)]
    fn process_is_running(process_id: u32) -> bool {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {process_id}"), "/FO", "CSV", "/NH"])
            .output()
            .is_ok_and(|output| {
                String::from_utf8_lossy(&output.stdout).contains(&process_id.to_string())
            })
    }

    #[cfg(unix)]
    fn process_is_running(process_id: u32) -> bool {
        Command::new("kill")
            .args(["-0", &process_id.to_string()])
            .status()
            .is_ok_and(|status| status.success())
    }

    fn wait_for_path(path: &Path, timeout: Duration) {
        let deadline = std::time::Instant::now() + timeout;
        while !path.exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "process-tree descendant did not report ready: {}",
                path.display()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}

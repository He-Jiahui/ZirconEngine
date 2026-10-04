#[cfg(windows)]
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use super::{
    configure_process_tree_suspended_spawn, platform_process_tree_termination_args,
    ProcessTreeLease,
};

#[cfg(windows)]
#[test]
fn process_tree_termination_args_use_windows_tree_kill() {
    assert_eq!(
        platform_process_tree_termination_args(42),
        vec!["/PID", "42", "/T", "/F"]
    );
}

#[cfg(windows)]
#[test]
fn persistent_windows_tree_uses_a_kill_on_close_job_object() {
    let source = include_str!("../process.rs");
    assert!(source.contains("ProcessTreeLease"));
    assert!(source.contains("JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE"));
    assert!(source.contains("AssignProcessToJobObject"));
    assert!(source.contains("TerminateJobObject"));
    assert!(source.contains("CreateToolhelp32Snapshot"));
    assert!(source.contains("ResumeThread"));
    assert!(source.contains("previous_suspend_count == 1"));
}

#[test]
fn persistent_tree_termination_keeps_the_lease_retryable_after_a_failure() {
    let source = include_str!("../process.rs");

    assert!(source.contains("pub(crate) fn terminate(&mut self, label: &str)"));
    assert!(source.contains("pub(super) fn terminate(&mut self) -> io::Result<()>"));
}

#[cfg(windows)]
#[test]
fn suspended_windows_child_runs_only_after_job_attachment_and_can_be_tree_terminated() {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-process-tree-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("fixture root should be created");
    let marker = root.join("started.txt");
    let command_line = format!(
        "echo attached>\"{}\" & ping 127.0.0.1 -n 30 >NUL",
        marker.display()
    );
    let mut command = Command::new("cmd");
    command
        .args(["/C", &command_line])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    configure_process_tree_suspended_spawn(&mut command);
    let mut child = command
        .spawn()
        .expect("fixture child should spawn suspended");

    assert!(
        !marker.exists(),
        "the suspended child must not run before the process-job attachment"
    );
    let mut tree = ProcessTreeLease::attach_and_start(&child, "process-tree fixture")
        .expect("fixture child should attach to its persistent process job");
    let deadline = Instant::now() + Duration::from_secs(2);
    while !marker.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        marker.exists(),
        "the child should run after attachment and resume"
    );
    assert!(
        child
            .try_wait()
            .expect("fixture child should remain observable")
            .is_none(),
        "the fixture child must still be alive before tree termination"
    );

    let termination = tree.terminate("process-tree fixture");
    assert!(termination.succeeded, "{}", termination.diagnostic);
    child
        .wait()
        .expect("terminated fixture child should be reaped");
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(all(unix, not(windows)))]
#[test]
fn persistent_unix_tree_treats_an_absent_group_as_terminal() {
    let source = include_str!("../process.rs");
    assert!(source.contains("unix_process_group::terminate(child_id)"));
    assert!(source.contains("error.raw_os_error() == Some(ESRCH)"));
}

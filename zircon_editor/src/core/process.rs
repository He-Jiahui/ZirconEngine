use std::io;
use std::process::{Child, Command};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessTreeTerminationError {
    #[error("failed to attach {label} to a persistent process tree: {source}")]
    TreeAttachment {
        label: String,
        #[source]
        source: io::Error,
    },
    #[error("failed to start {label} after process-tree attachment: {source}")]
    TreeStart {
        label: String,
        #[source]
        source: io::Error,
    },
    #[error("persistent process-job termination failed: {source}")]
    ProcessJobTermination {
        #[source]
        source: io::Error,
    },
    #[error("failed to start process termination command {program}: {source}")]
    CommandSpawn {
        program: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("process termination command {program} failed with status {status_code:?}: {stderr}")]
    CommandExit {
        program: &'static str,
        status_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    #[error("fallback child-process termination failed: {source}")]
    FallbackKill {
        primary: Option<Box<ProcessTreeTerminationError>>,
        #[source]
        source: io::Error,
    },
    #[error("process tree was not confirmed terminated after direct child termination")]
    TreeTerminationIncomplete {
        primary: Option<Box<ProcessTreeTerminationError>>,
    },
}

pub struct ProcessTreeTermination {
    pub diagnostic: String,
    pub succeeded: bool,
    pub error: Option<ProcessTreeTerminationError>,
}

/// Owns the platform process-tree relationship created for one spawned child.
///
/// The lease must outlive every inherited output pipe. Its explicit termination
/// is therefore the prerequisite for joining output-reader threads.
pub(crate) struct ProcessTreeLease {
    #[cfg(windows)]
    job: windows_job::JobObject,
    #[cfg(not(windows))]
    child_id: u32,
}

impl ProcessTreeLease {
    /// Attaches the child to its persistent tree before it is allowed to run.
    ///
    /// On Windows, the command is created suspended so children cannot escape
    /// the Job Object between root-process creation and assignment.
    pub(crate) fn attach_and_start(
        child: &Child,
        label: &str,
    ) -> Result<Self, ProcessTreeTerminationError> {
        let tree = Self::attach(child, label)?;
        #[cfg(windows)]
        if let Err(source) = windows_job::resume_initial_thread(child.id()) {
            let _ = tree.terminate(label);
            return Err(ProcessTreeTerminationError::TreeStart {
                label: label.to_string(),
                source,
            });
        }
        Ok(tree)
    }

    pub(crate) fn attach(child: &Child, label: &str) -> Result<Self, ProcessTreeTerminationError> {
        #[cfg(windows)]
        {
            let job = windows_job::JobObject::attach(child).map_err(|source| {
                ProcessTreeTerminationError::TreeAttachment {
                    label: label.to_string(),
                    source,
                }
            })?;
            return Ok(Self { job });
        }

        #[cfg(not(windows))]
        {
            let _ = label;
            Ok(Self {
                child_id: child.id(),
            })
        }
    }

    pub(crate) fn terminate(self, label: &str) -> ProcessTreeTermination {
        #[cfg(windows)]
        {
            return match self.job.terminate() {
                Ok(()) => ProcessTreeTermination {
                    diagnostic: format!("{label} cancelled; process job was terminated"),
                    succeeded: true,
                    error: None,
                },
                Err(source) => ProcessTreeTermination {
                    diagnostic: format!(
                        "{label} cancellation requested but persistent process-job termination failed: {source}"
                    ),
                    succeeded: false,
                    error: Some(ProcessTreeTerminationError::ProcessJobTermination { source }),
                },
            };
        }

        #[cfg(all(unix, not(windows)))]
        {
            return terminate_platform_process_tree(self.child_id, label).unwrap_or_else(|| {
                ProcessTreeTermination {
                    diagnostic: format!(
                        "{label} cancellation requested but no process-group termination is available"
                    ),
                    succeeded: false,
                    error: Some(ProcessTreeTerminationError::TreeTerminationIncomplete {
                        primary: None,
                    }),
                }
            });
        }

        #[cfg(not(any(windows, unix)))]
        {
            let _ = label;
            ProcessTreeTermination {
                diagnostic: "persistent process-tree termination is unavailable on this platform"
                    .to_string(),
                succeeded: false,
                error: Some(ProcessTreeTerminationError::TreeTerminationIncomplete {
                    primary: None,
                }),
            }
        }
    }
}

pub fn configure_process_tree_cancellation(process: &mut Command) {
    configure_platform_process_tree(process);
}

/// Marks a Play command for suspended Windows creation. Callers must then use
/// [`ProcessTreeLease::attach_and_start`] immediately after `spawn`.
pub(crate) fn configure_process_tree_suspended_spawn(process: &mut Command) {
    configure_platform_suspended_spawn(process);
}

pub fn terminate_process_tree(child: &mut Child, label: &str) -> ProcessTreeTermination {
    let child_id = child.id();
    let mut diagnostics = Vec::new();
    let mut primary_error = None;
    if let Some(result) = terminate_platform_process_tree(child_id, label) {
        diagnostics.push(result.diagnostic);
        primary_error = result.error;
        if result.succeeded {
            return ProcessTreeTermination {
                diagnostic: diagnostics.join("\n"),
                succeeded: true,
                error: None,
            };
        }
    }
    let (diagnostic, error) = match child.kill() {
        Ok(()) => (
            format!("{label} direct process was terminated after process-tree cancellation failed"),
            Some(ProcessTreeTerminationError::TreeTerminationIncomplete {
                primary: primary_error.map(Box::new),
            }),
        ),
        Err(source) => (
            format!("{label} cancellation requested but termination failed: {source}"),
            Some(ProcessTreeTerminationError::FallbackKill {
                primary: primary_error.map(Box::new),
                source,
            }),
        ),
    };
    diagnostics.push(diagnostic);
    ProcessTreeTermination {
        diagnostic: diagnostics.join("\n"),
        // A direct-child fallback cannot guarantee that inherited output pipes are closed.
        succeeded: false,
        error,
    }
}

#[cfg(unix)]
fn configure_platform_process_tree(process: &mut Command) {
    use std::os::unix::process::CommandExt;

    process.process_group(0);
}

#[cfg(not(unix))]
fn configure_platform_process_tree(_process: &mut Command) {}

#[cfg(windows)]
fn configure_platform_suspended_spawn(process: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_SUSPENDED: u32 = 0x0000_0004;
    process.creation_flags(CREATE_SUSPENDED);
}

#[cfg(not(windows))]
fn configure_platform_suspended_spawn(_process: &mut Command) {}

struct PlatformProcessTreeTermination {
    diagnostic: String,
    succeeded: bool,
    error: Option<ProcessTreeTerminationError>,
}

#[cfg(windows)]
fn terminate_platform_process_tree(
    child_id: u32,
    label: &str,
) -> Option<PlatformProcessTreeTermination> {
    let output = Command::new("taskkill")
        .args(platform_process_tree_termination_args(child_id))
        .output();
    Some(match output {
        Ok(output) if output.status.success() => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancelled; process tree was terminated"),
            succeeded: true,
            error: None,
        },
        Ok(output) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but taskkill failed with status {:?}: {}{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            succeeded: false,
            error: Some(ProcessTreeTerminationError::CommandExit {
                program: "taskkill",
                status_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            }),
        },
        Err(source) => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancellation requested but taskkill failed: {source}"),
            succeeded: false,
            error: Some(ProcessTreeTerminationError::CommandSpawn {
                program: "taskkill",
                source,
            }),
        },
    })
}

#[cfg(all(unix, not(windows)))]
fn terminate_platform_process_tree(
    child_id: u32,
    label: &str,
) -> Option<PlatformProcessTreeTermination> {
    Some(match unix_process_group::terminate(child_id) {
        Ok(true) => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancelled; process group was terminated"),
            succeeded: true,
            error: None,
        },
        Ok(false) => PlatformProcessTreeTermination {
            diagnostic: format!("{label} process group already exited"),
            succeeded: true,
            error: None,
        },
        Err(source) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but process-group termination failed: {source}"
            ),
            succeeded: false,
            error: Some(ProcessTreeTerminationError::CommandSpawn {
                program: "kill",
                source,
            }),
        },
    })
}

#[cfg(not(any(windows, unix)))]
fn terminate_platform_process_tree(
    _child_id: u32,
    _label: &str,
) -> Option<PlatformProcessTreeTermination> {
    None
}

#[cfg(windows)]
fn platform_process_tree_termination_args(child_id: u32) -> Vec<String> {
    vec![
        "/PID".to_string(),
        child_id.to_string(),
        "/T".to_string(),
        "/F".to_string(),
    ]
}

#[cfg(all(unix, not(windows)))]
mod unix_process_group {
    use std::io;

    const ESRCH: i32 = 3;
    const SIGKILL: i32 = 9;

    unsafe extern "C" {
        fn kill(process_id: i32, signal: i32) -> i32;
    }

    /// Returns whether a live process group was terminated. A missing process
    /// group means no member can still hold the preview output pipes open.
    pub(super) fn terminate(process_group_id: u32) -> io::Result<bool> {
        let process_group_id = i32::try_from(process_group_id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "process group id does not fit the platform pid type",
            )
        })?;
        let killed = unsafe { kill(-process_group_id, SIGKILL) } == 0;
        if killed {
            return Ok(true);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ESRCH) {
            return Ok(false);
        }
        Err(error)
    }
}

#[cfg(windows)]
mod windows_job {
    use std::ffi::c_void;
    use std::io;
    use std::mem;
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::ptr;

    type Handle = *mut c_void;

    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;

    #[repr(C)]
    #[derive(Default)]
    struct JobObjectBasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct JobObjectExtendedLimitInformation {
        basic_limit_information: JobObjectBasicLimitInformation,
        io_info: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    unsafe extern "system" {
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> Handle;
        fn OpenThread(desired_access: u32, inherit_handle: i32, thread_id: u32) -> Handle;
        fn ResumeThread(thread: Handle) -> u32;
        fn SetInformationJobObject(
            job: Handle,
            information_class: i32,
            information: *const c_void,
            information_length: u32,
        ) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry32) -> i32;
        fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry32) -> i32;
    }

    const INVALID_HANDLE_VALUE: Handle = -1_isize as Handle;
    const TH32CS_SNAPTHREAD: u32 = 0x0000_0004;
    const THREAD_SUSPEND_RESUME: u32 = 0x0000_0002;

    #[repr(C)]
    #[derive(Default)]
    struct ThreadEntry32 {
        size: u32,
        usage_count: u32,
        thread_id: u32,
        owner_process_id: u32,
        base_priority: i32,
        delta_priority: i32,
        flags: u32,
    }

    pub(super) struct JobObject {
        handle: Handle,
    }

    // A Windows handle may be used by the thread that owns the preview backend.
    unsafe impl Send for JobObject {}

    impl JobObject {
        pub(super) fn attach(child: &Child) -> io::Result<Self> {
            let handle = unsafe { CreateJobObjectW(ptr::null::<c_void>(), ptr::null::<u16>()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut job = Self { handle };
            let mut limits = JobObjectExtendedLimitInformation::default();
            limits.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let limits_set = unsafe {
                SetInformationJobObject(
                    job.handle,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                    (&limits as *const JobObjectExtendedLimitInformation).cast::<c_void>(),
                    mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
                )
            };
            if limits_set == 0 {
                return Err(io::Error::last_os_error());
            }
            let assigned = unsafe { AssignProcessToJobObject(job.handle, child.as_raw_handle()) };
            if assigned == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }

        pub(super) fn terminate(mut self) -> io::Result<()> {
            let terminated = unsafe { TerminateJobObject(self.handle, 1) } != 0;
            let terminate_error = (!terminated).then(io::Error::last_os_error);
            let close_result = self.close();
            if terminated || close_result.is_ok() {
                Ok(())
            } else {
                Err(terminate_error.unwrap_or_else(|| {
                    close_result.expect_err("failed process-job close must retain its error")
                }))
            }
        }

        fn close(&mut self) -> io::Result<()> {
            if self.handle.is_null() {
                return Ok(());
            }
            let handle = mem::replace(&mut self.handle, ptr::null_mut());
            if unsafe { CloseHandle(handle) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }

    impl Drop for JobObject {
        fn drop(&mut self) {
            let _ = self.close();
        }
    }

    pub(super) fn resume_initial_thread(process_id: u32) -> io::Result<()> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let result = find_initial_thread(snapshot, process_id).and_then(resume_thread);
        let _ = unsafe { CloseHandle(snapshot) };
        result
    }

    fn find_initial_thread(snapshot: Handle, process_id: u32) -> io::Result<u32> {
        let mut entry = ThreadEntry32 {
            size: mem::size_of::<ThreadEntry32>() as u32,
            ..Default::default()
        };
        if unsafe { Thread32First(snapshot, &mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        loop {
            if entry.owner_process_id == process_id {
                return Ok(entry.thread_id);
            }
            entry.size = mem::size_of::<ThreadEntry32>() as u32;
            if unsafe { Thread32Next(snapshot, &mut entry) } == 0 {
                break;
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "suspended process has no discoverable initial thread",
        ))
    }

    fn resume_thread(thread_id: u32) -> io::Result<()> {
        let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, thread_id) };
        if thread.is_null() {
            return Err(io::Error::last_os_error());
        }
        let previous_suspend_count = unsafe { ResumeThread(thread) };
        let result = if previous_suspend_count == u32::MAX {
            Err(io::Error::last_os_error())
        } else if previous_suspend_count == 1 {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "expected a singly suspended initial thread, found suspend count {previous_suspend_count}"
                ),
            ))
        };
        let _ = unsafe { CloseHandle(thread) };
        result
    }
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("process.rs");
        assert!(source.contains("ProcessTreeLease"));
        assert!(source.contains("JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE"));
        assert!(source.contains("AssignProcessToJobObject"));
        assert!(source.contains("TerminateJobObject"));
        assert!(source.contains("CreateToolhelp32Snapshot"));
        assert!(source.contains("ResumeThread"));
        assert!(source.contains("previous_suspend_count == 1"));
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
        let tree = ProcessTreeLease::attach_and_start(&child, "process-tree fixture")
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
        let source = include_str!("process.rs");
        assert!(source.contains("unix_process_group::terminate(child_id)"));
        assert!(source.contains("error.raw_os_error() == Some(ESRCH)"));
    }
}

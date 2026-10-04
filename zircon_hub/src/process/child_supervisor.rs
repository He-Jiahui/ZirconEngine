use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

use crate::error::HubError;

const PROCESS_TREE_TERMINATION_WAIT: Duration = Duration::from_secs(2);
const PROCESS_TREE_TERMINATION_POLL: Duration = Duration::from_millis(10);

pub(crate) struct SupervisedChild {
    child: Child,
    process_tree: ProcessTreeLease,
    terminated: bool,
    cleanup_deadline: Option<Instant>,
}

impl SupervisedChild {
    pub(crate) fn spawn(command: &mut Command, label: &str) -> Result<Self, HubError> {
        configure_process_tree(command);
        let mut child = command.spawn()?;
        let process_tree = match ProcessTreeLease::attach_and_start(&child, label) {
            Ok(process_tree) => process_tree,
            Err(error) => {
                let cleanup_error = terminate_direct_child_and_reap(&mut child).err();
                return Err(combine_process_errors(Some(error), cleanup_error)
                    .expect_err("process-tree attachment failure must remain an error"));
            }
        };
        Ok(Self {
            child,
            process_tree,
            terminated: false,
            cleanup_deadline: None,
        })
    }

    pub(crate) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn try_wait(&mut self) -> Result<Option<ExitStatus>, HubError> {
        self.child.try_wait().map_err(Into::into)
    }

    pub(crate) fn terminate_tree_and_reap(&mut self) -> Result<(), HubError> {
        self.terminate_tree_and_reap_until(Instant::now() + PROCESS_TREE_TERMINATION_WAIT)
    }

    pub(crate) fn terminate_tree_and_reap_until(
        &mut self,
        deadline: Instant,
    ) -> Result<(), HubError> {
        self.cleanup_deadline = Some(deadline);
        let tree_error = match self.process_tree.is_empty() {
            Ok(true) => None,
            Ok(false) => self.process_tree.terminate().err(),
            Err(query_error) => {
                let termination_error = self.process_tree.terminate().err();
                Some(
                    combine_process_errors(Some(query_error), termination_error)
                        .expect_err("process-tree query/termination failure must remain an error"),
                )
            }
        };
        let reap_error = terminate_direct_child_and_reap_until(&mut self.child, deadline).err();
        let drain_error = if tree_error.is_none() {
            self.process_tree.wait_until_empty_until(deadline).err()
        } else {
            None
        };
        let close_error = self.process_tree.close().err();
        let result =
            combine_process_error_options([tree_error, reap_error, drain_error, close_error]);
        if result.is_ok() {
            self.terminated = true;
        }
        result
    }
}

impl Drop for SupervisedChild {
    fn drop(&mut self) {
        if !self.terminated {
            let deadline = self
                .cleanup_deadline
                .unwrap_or_else(|| Instant::now() + PROCESS_TREE_TERMINATION_WAIT);
            if let Err(error) = self.terminate_tree_and_reap_until(deadline) {
                eprintln!(
                    "zircon_hub: failed to terminate supervised child {} during drop: {error}",
                    self.child.id()
                );
            }
        }
    }
}

trait DirectChildControl {
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>>;
    fn kill(&mut self) -> std::io::Result<()>;
}

impl DirectChildControl for Child {
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        Child::try_wait(self)
    }

    fn kill(&mut self) -> std::io::Result<()> {
        Child::kill(self)
    }
}

fn terminate_direct_child_and_reap(child: &mut Child) -> Result<(), HubError> {
    terminate_direct_child_and_reap_until(child, Instant::now() + PROCESS_TREE_TERMINATION_WAIT)
}

fn terminate_direct_child_and_reap_until(
    child: &mut impl DirectChildControl,
    deadline: Instant,
) -> Result<(), HubError> {
    let initial_error = match child.try_wait() {
        Ok(Some(_)) => return Ok(()),
        Ok(None) => None,
        Err(error) => Some(HubError::from(error)),
    };
    let kill_error = child.kill().err().map(HubError::from);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return combine_process_error_options([initial_error]),
            Ok(None) => {}
            Err(error) => {
                return combine_process_error_options([
                    initial_error,
                    kill_error,
                    Some(HubError::from(error)),
                ]);
            }
        }
        if Instant::now() >= deadline {
            return combine_process_error_options([
                initial_error,
                kill_error,
                Some(HubError::message(
                    "direct child did not exit before its reap deadline",
                )),
            ]);
        }
        std::thread::sleep(
            PROCESS_TREE_TERMINATION_POLL.min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

fn combine_process_errors(
    first: Option<HubError>,
    second: Option<HubError>,
) -> Result<(), HubError> {
    combine_process_error_options([first, second])
}

fn combine_process_error_options(
    errors: impl IntoIterator<Item = Option<HubError>>,
) -> Result<(), HubError> {
    let messages = errors
        .into_iter()
        .flatten()
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    if messages.is_empty() {
        Ok(())
    } else {
        Err(HubError::message(messages.join("; ")))
    }
}

struct ProcessTreeLease {
    #[cfg(windows)]
    job: windows_job::JobObject,
    #[cfg(all(unix, not(windows)))]
    process_group_id: u32,
}

impl ProcessTreeLease {
    fn attach_and_start(child: &Child, label: &str) -> Result<Self, HubError> {
        #[cfg(windows)]
        {
            let job = windows_job::JobObject::attach(child).map_err(|error| {
                HubError::message(format!(
                    "failed to attach {label} to a persistent process job: {error}"
                ))
            })?;
            let mut process_tree = Self { job };
            if let Err(error) = windows_job::resume_initial_thread(child.id()) {
                let termination_error = process_tree.terminate().err();
                return Err(combine_process_errors(
                    Some(HubError::message(format!(
                        "failed to start {label} after process-job attachment: {error}"
                    ))),
                    termination_error,
                )
                .expect_err("process-tree start failure must remain an error"));
            }
            return Ok(process_tree);
        }

        #[cfg(all(unix, not(windows)))]
        {
            let _ = label;
            return Ok(Self {
                process_group_id: child.id(),
            });
        }

        #[cfg(not(any(windows, unix)))]
        {
            let _ = (child, label);
            Ok(Self {})
        }
    }

    fn terminate(&mut self) -> Result<(), HubError> {
        #[cfg(windows)]
        {
            return self.job.terminate().map_err(|error| {
                HubError::message(format!(
                    "persistent process-job termination failed: {error}"
                ))
            });
        }

        #[cfg(all(unix, not(windows)))]
        {
            return unix_process_group::terminate(self.process_group_id).map_err(|error| {
                HubError::message(format!("process-group termination failed: {error}"))
            });
        }

        #[cfg(not(any(windows, unix)))]
        Ok(())
    }

    fn wait_until_empty_until(&self, deadline: Instant) -> Result<(), HubError> {
        loop {
            if self.is_empty()? {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(HubError::message(
                    "supervised process descendants did not terminate before the cleanup deadline",
                ));
            }
            std::thread::sleep(
                PROCESS_TREE_TERMINATION_POLL
                    .min(deadline.saturating_duration_since(Instant::now())),
            );
        }
    }

    fn is_empty(&self) -> Result<bool, HubError> {
        #[cfg(windows)]
        {
            return self
                .job
                .is_empty()
                .map_err(|error| HubError::message(format!("process-job query failed: {error}")));
        }

        #[cfg(all(unix, not(windows)))]
        {
            return unix_process_group::is_empty(self.process_group_id).map_err(|error| {
                HubError::message(format!("process-group query failed: {error}"))
            });
        }

        #[cfg(not(any(windows, unix)))]
        {
            Ok(true)
        }
    }

    fn close(&mut self) -> Result<(), HubError> {
        #[cfg(windows)]
        {
            return self
                .job
                .close()
                .map_err(|error| HubError::message(format!("process-job close failed: {error}")));
        }

        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}

#[cfg(unix)]
fn configure_process_tree(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
}

#[cfg(windows)]
fn configure_process_tree(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_SUSPENDED: u32 = 0x0000_0004;
    command.creation_flags(CREATE_SUSPENDED);
}

#[cfg(not(any(windows, unix)))]
fn configure_process_tree(_command: &mut Command) {}

#[cfg(all(unix, not(windows)))]
mod unix_process_group {
    use std::io;

    const EPERM: i32 = 1;
    const ESRCH: i32 = 3;
    const SIGKILL: i32 = 9;

    unsafe extern "C" {
        fn kill(process_id: i32, signal: i32) -> i32;
    }

    pub(super) fn terminate(process_group_id: u32) -> io::Result<()> {
        let process_group_id = i32::try_from(process_group_id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "process group id does not fit the platform pid type",
            )
        })?;
        if unsafe { kill(-process_group_id, SIGKILL) } == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ESRCH) {
            return Ok(());
        }
        Err(error)
    }

    pub(super) fn is_empty(process_group_id: u32) -> io::Result<bool> {
        let process_group_id = i32::try_from(process_group_id).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "process group id does not fit the platform pid type",
            )
        })?;
        if unsafe { kill(-process_group_id, 0) } == 0 {
            return Ok(false);
        }
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(ESRCH) => Ok(true),
            Some(EPERM) => Ok(false),
            _ => Err(error),
        }
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
    const JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION: i32 = 1;
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;
    const INVALID_HANDLE_VALUE: Handle = -1_isize as Handle;
    const TH32CS_SNAPTHREAD: u32 = 0x0000_0004;
    const THREAD_SUSPEND_RESUME: u32 = 0x0000_0002;

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

    #[repr(C)]
    #[derive(Default)]
    struct JobObjectBasicAccountingInformation {
        total_user_time: i64,
        total_kernel_time: i64,
        this_period_total_user_time: i64,
        this_period_total_kernel_time: i64,
        total_page_fault_count: u32,
        total_processes: u32,
        active_processes: u32,
        total_terminated_processes: u32,
    }

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

    #[link(name = "kernel32")]
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
        fn QueryInformationJobObject(
            job: Handle,
            information_class: i32,
            information: *mut c_void,
            information_length: u32,
            return_length: *mut u32,
        ) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry32) -> i32;
        fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry32) -> i32;
    }

    pub(super) struct JobObject {
        handle: Handle,
    }

    unsafe impl Send for JobObject {}

    impl JobObject {
        pub(super) fn attach(child: &Child) -> io::Result<Self> {
            let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut job = Self { handle };
            let mut limits = JobObjectExtendedLimitInformation::default();
            limits.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if unsafe {
                SetInformationJobObject(
                    job.handle,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                    (&limits as *const JobObjectExtendedLimitInformation).cast(),
                    mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            if unsafe { AssignProcessToJobObject(job.handle, child.as_raw_handle()) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }

        pub(super) fn terminate(&mut self) -> io::Result<()> {
            if self.handle.is_null() {
                return Ok(());
            }
            let termination_error = if unsafe { TerminateJobObject(self.handle, 1) } == 0 {
                Some(io::Error::last_os_error())
            } else {
                None
            };
            termination_error.map_or(Ok(()), Err)
        }

        pub(super) fn is_empty(&self) -> io::Result<bool> {
            if self.handle.is_null() {
                return Ok(true);
            }
            let mut accounting = JobObjectBasicAccountingInformation::default();
            let mut return_length = 0;
            if unsafe {
                QueryInformationJobObject(
                    self.handle,
                    JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION,
                    (&mut accounting as *mut JobObjectBasicAccountingInformation).cast(),
                    mem::size_of::<JobObjectBasicAccountingInformation>() as u32,
                    &mut return_length,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(accounting.active_processes == 0)
        }

        pub(super) fn close(&mut self) -> io::Result<()> {
            if self.handle.is_null() {
                return Ok(());
            }
            if unsafe { CloseHandle(self.handle) } == 0 {
                return Err(io::Error::last_os_error());
            }
            self.handle = ptr::null_mut();
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
        let close_error = if unsafe { CloseHandle(snapshot) } == 0 {
            Some(io::Error::last_os_error())
        } else {
            None
        };
        match (result, close_error) {
            (Ok(()), None) => Ok(()),
            (Err(error), None) | (Ok(()), Some(error)) => Err(error),
            (Err(resume_error), Some(close_error)) => Err(io::Error::other(format!(
                "{resume_error}; failed to close thread snapshot handle: {close_error}"
            ))),
        }
    }

    fn find_initial_thread(snapshot: Handle, process_id: u32) -> io::Result<Handle> {
        let mut entry = ThreadEntry32 {
            size: mem::size_of::<ThreadEntry32>() as u32,
            ..ThreadEntry32::default()
        };
        if unsafe { Thread32First(snapshot, &mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        loop {
            if entry.owner_process_id == process_id {
                let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.thread_id) };
                if thread.is_null() {
                    return Err(io::Error::last_os_error());
                }
                return Ok(thread);
            }
            if unsafe { Thread32Next(snapshot, &mut entry) } == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("initial thread for process {process_id} was not found"),
                ));
            }
        }
    }

    fn resume_thread(thread: Handle) -> io::Result<()> {
        let previous_suspend_count = unsafe { ResumeThread(thread) };
        let resume_error = if previous_suspend_count == u32::MAX {
            Some(io::Error::last_os_error())
        } else {
            None
        };
        let close_error = if unsafe { CloseHandle(thread) } == 0 {
            Some(io::Error::last_os_error())
        } else {
            None
        };
        match (resume_error, close_error) {
            (None, None) => Ok(()),
            (Some(error), None) | (None, Some(error)) => Err(error),
            (Some(resume_error), Some(close_error)) => Err(io::Error::other(format!(
                "{resume_error}; failed to close initial thread handle: {close_error}"
            ))),
        }
    }
}

#[cfg(all(test, any(windows, unix)))]
#[path = "tests/child_supervisor.rs"]
mod tests;

#[cfg(test)]
#[path = "child_supervisor/tests/cases.rs"]
mod direct_child_tests;

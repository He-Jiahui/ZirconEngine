use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use zircon_runtime_interface::project::session_lock::windows_project_session_mutex_name;
use zircon_runtime_interface::project::session_lock::{
    decode_project_session_admission_record, project_session_lock_path,
    ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordV1,
};

use crate::error::HubError;
use crate::projects::normalize_project_root;

/// Result of Hub's read-only view of the platform lease held by an editor instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectEditorSessionProbe {
    Inactive,
    /// The editor holds the exclusive lease but cannot yet consume focus requests.
    Pending(ProjectSessionAdmissionRecordV1),
    /// The editor committed an addressable project generation and may receive a focus request.
    Ready(ProjectSessionAdmissionRecordV1),
    /// The exclusive lease remains intentionally held until recovery is resolved.
    RecoveryRequired(ProjectSessionAdmissionRecordV1),
}

/// Detects an active Editor17 session without creating, replacing, or acquiring its lease.
///
/// A lock record becomes focusable only after the same OS lease namespace is held and its
/// lifecycle has committed `Ready`. Pending and recovery leases prevent a second writer but
/// never authorize Hub focus.
pub(crate) fn probe_project_editor_session(
    project_root: impl AsRef<Path>,
) -> Result<ProjectEditorSessionProbe, HubError> {
    let root = resolve_project_session_root(project_root.as_ref())?;
    if !platform_lease_is_active(&root)? {
        return Ok(ProjectEditorSessionProbe::Inactive);
    }

    let lock_path = project_session_lock_path(&root);
    let source = fs::read_to_string(&lock_path).map_err(|source| {
        HubError::message(format!(
            "active editor session is missing its lock record `{}`: {source}",
            lock_path.display()
        ))
    })?;
    let record = decode_project_session_admission_record(&source).map_err(|source| {
        HubError::message(format!(
            "active editor session has an invalid lock record `{}`: {source}",
            lock_path.display()
        ))
    })?;
    Ok(classify_live_session_record(record))
}

fn classify_live_session_record(
    record: ProjectSessionAdmissionRecordV1,
) -> ProjectEditorSessionProbe {
    match record.lifecycle() {
        ProjectSessionAdmissionLifecycleV1::Ready => ProjectEditorSessionProbe::Ready(record),
        ProjectSessionAdmissionLifecycleV1::RecoveryRequired => {
            ProjectEditorSessionProbe::RecoveryRequired(record)
        }
        ProjectSessionAdmissionLifecycleV1::Claimed
        | ProjectSessionAdmissionLifecycleV1::PreflightApproved
        | ProjectSessionAdmissionLifecycleV1::Activating
        | ProjectSessionAdmissionLifecycleV1::Closing => ProjectEditorSessionProbe::Pending(record),
    }
}

fn resolve_project_session_root(project_root: &Path) -> Result<PathBuf, HubError> {
    let canonical = project_root.canonicalize().map_err(|source| {
        HubError::message(format!(
            "resolve project session identity `{}`: {source}",
            project_root.display()
        ))
    })?;
    Ok(normalize_project_root(canonical))
}

#[cfg(windows)]
fn platform_lease_is_active(project_root: &Path) -> Result<bool, HubError> {
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const ERROR_FILE_NOT_FOUND: i32 = 2;

    let name = windows_project_session_mutex_name(project_root);
    let name = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // SAFETY: the name is NUL-terminated and this only opens a pre-existing named mutex.
    let handle = unsafe { OpenMutexW(SYNCHRONIZE, 0, name.as_ptr()) };
    if handle == 0 {
        let source = io::Error::last_os_error();
        if source.raw_os_error() == Some(ERROR_FILE_NOT_FOUND) {
            return Ok(false);
        }
        return Err(HubError::message(format!(
            "open project editor session lease: {source}"
        )));
    }
    // SAFETY: a nonzero OpenMutexW result is an owned handle closed exactly once here.
    unsafe {
        CloseHandle(handle);
    }
    Ok(true)
}

#[cfg(unix)]
fn platform_lease_is_active(project_root: &Path) -> Result<bool, HubError> {
    use std::os::unix::io::AsRawFd;

    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    const LOCK_UN: i32 = 8;

    let directory = project_root.join(".zircon");
    let directory = match fs::File::open(&directory) {
        Ok(directory) => directory,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => return Err(source.into()),
    };
    // SAFETY: the file descriptor stays alive through the paired unlock below.
    let locked = unsafe { flock(directory.as_raw_fd(), LOCK_EX | LOCK_NB) };
    if locked == 0 {
        // SAFETY: the descriptor was exclusively locked by this call and remains valid.
        unsafe {
            flock(directory.as_raw_fd(), LOCK_UN);
        }
        return Ok(false);
    }
    let source = io::Error::last_os_error();
    if matches!(
        source.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::AlreadyExists
    ) {
        Ok(true)
    } else {
        Err(source.into())
    }
}

#[cfg(not(any(windows, unix)))]
fn platform_lease_is_active(_project_root: &Path) -> Result<bool, HubError> {
    Ok(false)
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenMutexW(desired_access: u32, inherit_handle: i32, name: *const u16) -> isize;
    fn CloseHandle(handle: isize) -> i32;
}

#[cfg(unix)]
unsafe extern "C" {
    fn flock(file_descriptor: i32, operation: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::project::session_lock::{
        ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordV1,
        ProjectSessionGenerationV1, ProjectSessionPrincipalV1,
    };
    use zircon_runtime_interface::project::{
        ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    };
    use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

    use super::{classify_live_session_record, ProjectEditorSessionProbe};

    #[test]
    fn live_activating_lease_is_pending_not_focusable() {
        let activating = fixture_claimed_record()
            .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
            .expect("fixture preflight approval")
            .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
            .expect("fixture activation");

        assert!(matches!(
            classify_live_session_record(activating),
            ProjectEditorSessionProbe::Pending(_)
        ));
    }

    #[test]
    fn live_ready_lease_is_the_only_focusable_session() {
        let ready = fixture_claimed_record()
            .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
            .expect("fixture preflight approval")
            .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
            .expect("fixture activation")
            .commit_ready(ProjectSessionGenerationV1::new(1).expect("fixture generation"))
            .expect("fixture ready commit");

        assert!(matches!(
            classify_live_session_record(ready),
            ProjectEditorSessionProbe::Ready(_)
        ));
    }

    fn fixture_claimed_record() -> ProjectSessionAdmissionRecordV1 {
        let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
            .allocate()
            .expect("fixture operation");
        ProjectSessionAdmissionRecordV1::claim(
            913,
            "913-1723718523000-1",
            ProjectSessionPrincipalV1::Hub,
            ZrRuntimeBuildSetId::parse(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .expect("fixture BuildSet"),
            operation,
            1_723_718_523_000,
        )
        .expect("fixture admission record")
    }
}

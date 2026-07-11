use std::path::PathBuf;
use std::process::Command;

use crate::runtime_descriptor::RuntimeDescriptor;
use crate::TrayError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedProcessIdentity {
    pub pid: u32,
    pub creation_time: String,
    pub executable: PathBuf,
    pub command_line: String,
}

impl ObservedProcessIdentity {
    pub fn verify(&self, descriptor: &RuntimeDescriptor) -> Result<(), TrayError> {
        if self.pid != descriptor.pid || self.creation_time != descriptor.process_creation_time {
            return Err(TrayError::IdentityMismatch("PID creation identity differs"));
        }
        if !self
            .executable
            .to_string_lossy()
            .eq_ignore_ascii_case(&descriptor.executable.to_string_lossy())
        {
            return Err(TrayError::IdentityMismatch("process executable differs"));
        }
        let command = self.command_line.to_lowercase();
        let repository = descriptor.repo_root.to_string_lossy().to_lowercase();
        if !command.contains(&repository)
            || !(command.contains("tools.session_coordinator")
                || command.contains("session_coordinator"))
        {
            return Err(TrayError::IdentityMismatch(
                "process command line is not bound to this repository coordinator",
            ));
        }
        Ok(())
    }
}

#[cfg(windows)]
struct ProcessHandle(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl ProcessHandle {
    fn open(pid: u32, terminate: bool) -> Result<Self, TrayError> {
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
        };
        let mut rights = PROCESS_QUERY_LIMITED_INFORMATION;
        if terminate {
            rights |= PROCESS_TERMINATE;
        }
        Ok(Self(unsafe { OpenProcess(rights, false, pid)? }))
    }
}

#[cfg(windows)]
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.0) };
    }
}

#[cfg(windows)]
pub fn inspect_process(pid: u32) -> Result<ObservedProcessIdentity, TrayError> {
    let handle = ProcessHandle::open(pid, false)?;
    inspect_handle(pid, &handle)
}

#[cfg(windows)]
fn inspect_handle(pid: u32, handle: &ProcessHandle) -> Result<ObservedProcessIdentity, TrayError> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Threading::{
        GetProcessTimes, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    };

    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(handle.0, &mut created, &mut exited, &mut kernel, &mut user)?;
    }
    let creation_time =
        ((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime)).to_string();
    let mut image = vec![0u16; 32_768];
    let mut image_len = image.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            handle.0,
            PROCESS_NAME_WIN32,
            PWSTR(image.as_mut_ptr()),
            &mut image_len,
        )?;
    }
    image.truncate(image_len as usize);
    let executable = PathBuf::from(String::from_utf16_lossy(&image));
    let command_line = read_command_line(pid)?;
    Ok(ObservedProcessIdentity {
        pid,
        creation_time,
        executable,
        command_line,
    })
}

#[cfg(windows)]
fn read_command_line(pid: u32) -> Result<String, TrayError> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = format!("(Get-CimInstance Win32_Process -Filter 'ProcessId={pid}').CommandLine");
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    if !output.status.success() {
        return Err(TrayError::IdentityMismatch(
            "process command line is unavailable",
        ));
    }
    let line = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if line.is_empty() {
        return Err(TrayError::IdentityMismatch("process command line is empty"));
    }
    Ok(line)
}

#[cfg(windows)]
pub fn terminate_after_reverification(descriptor: &RuntimeDescriptor) -> Result<(), TrayError> {
    use windows::Win32::System::Threading::TerminateProcess;
    let handle = ProcessHandle::open(descriptor.pid, true)?;
    inspect_handle(descriptor.pid, &handle)?.verify(descriptor)?;
    unsafe { TerminateProcess(handle.0, 0x5A17)? };
    Ok(())
}

#[cfg(not(windows))]
pub fn inspect_process(_pid: u32) -> Result<ObservedProcessIdentity, TrayError> {
    Err(TrayError::IdentityMismatch(
        "process identity verification is Windows-only",
    ))
}

#[cfg(not(windows))]
pub fn terminate_after_reverification(_descriptor: &RuntimeDescriptor) -> Result<(), TrayError> {
    Err(TrayError::IdentityMismatch(
        "process termination is Windows-only",
    ))
}

#[cfg(all(test, windows))]
mod tests {
    use std::process::{Child, Command};

    use crate::repository_identity::RepositoryIdentity;
    use crate::runtime_descriptor::SecretString;

    use super::*;

    struct ChildGuard(Child);

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn fixture_descriptor(
        observed: &ObservedProcessIdentity,
        repository: &RepositoryIdentity,
    ) -> RuntimeDescriptor {
        RuntimeDescriptor {
            descriptor_version: 2,
            host: "127.0.0.1".into(),
            port: 1,
            token: serde_json::from_str::<SecretString>("\"fixture\"").unwrap(),
            pid: observed.pid,
            process_creation_time: observed.creation_time.clone(),
            executable: observed.executable.clone(),
            command_line: vec!["tools.session_coordinator".into()],
            repo_root: repository.canonical_path.clone(),
            repository_identity_version: 1,
            repository_key: repository.key.clone(),
            instance_id: "fixture-instance".into(),
            started_at: "fixture".into(),
            schema_version: 21,
            control_api_versions: vec![1],
            supervision_api_versions: vec![1],
        }
    }

    #[test]
    fn stale_creation_identity_is_rejected_without_terminating_child() {
        let repository = RepositoryIdentity::for_path(std::env::current_dir().unwrap()).unwrap();
        let fixture_command = format!(
            "$null='tools.session_coordinator {}'; Start-Sleep -Seconds 30",
            repository.canonical_path.display()
        );
        let child = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &fixture_command,
            ])
            .spawn()
            .unwrap();
        let mut child = ChildGuard(child);
        let observed = inspect_process(child.0.id()).unwrap();
        let mut descriptor = fixture_descriptor(&observed, &repository);
        descriptor.process_creation_time.push('0');

        assert!(matches!(
            observed.verify(&descriptor),
            Err(TrayError::IdentityMismatch(_))
        ));
        assert!(child.0.try_wait().unwrap().is_none());
    }

    #[test]
    fn unrelated_process_is_never_authorized_for_termination() {
        let repository = RepositoryIdentity::for_path(std::env::current_dir().unwrap()).unwrap();
        let child = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 30",
            ])
            .spawn()
            .unwrap();
        let mut child = ChildGuard(child);
        let observed = inspect_process(child.0.id()).unwrap();
        let descriptor = fixture_descriptor(&observed, &repository);

        assert!(matches!(
            observed.verify(&descriptor),
            Err(TrayError::IdentityMismatch(_))
        ));
        assert!(child.0.try_wait().unwrap().is_none());
    }
}

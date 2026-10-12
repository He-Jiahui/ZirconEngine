use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::sync::{Arc, Mutex, OnceLock, Weak};

use super::native_artifact_trust::{digest_file, open_library_for_admission};
use super::{
    NativePluginArtifactAdmissionError, NativePluginArtifactDependency, NativePluginArtifactDigest,
};

#[cfg(all(test, windows))]
#[path = "native_artifact_staging/tests/cases.rs"]
pub(super) mod tests;

#[cfg(windows)]
static ADMITTED_STAGINGS: OnceLock<Mutex<Vec<Weak<NativePluginArtifactStaging>>>> = OnceLock::new();

#[derive(Debug)]
pub(super) struct NativePluginArtifactStaging {
    root: PathBuf,
    guards: Vec<File>,
    main_path: PathBuf,
    loaded_dependencies: Vec<libloading::Library>,
    directory_guard: Option<File>,
    #[cfg(windows)]
    admitted_dependencies: Vec<NativePluginArtifactDependency>,
    #[cfg(windows)]
    retained_stagings: Vec<Arc<NativePluginArtifactStaging>>,
}

impl NativePluginArtifactStaging {
    pub(super) fn prepare(
        plugin_id: &str,
        main_source: &Path,
        main_digest: &NativePluginArtifactDigest,
        dependencies: &[NativePluginArtifactDependency],
    ) -> Result<Self, NativePluginArtifactAdmissionError> {
        let main_name = main_source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid("native library needs a UTF-8 filename"))?;
        validate_file_name(main_name)?;
        let mut names = HashSet::with_capacity(dependencies.len() + 1);
        names.insert(main_name.to_ascii_lowercase());
        for dependency in dependencies {
            validate_file_name(&dependency.file_name)?;
            if !names.insert(dependency.file_name.to_ascii_lowercase()) {
                return Err(invalid("native dependency closure has duplicate DLL names"));
            }
        }
        let root = std::env::temp_dir().join(format!("zircon-native-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).map_err(|error| invalid(error.to_string()))?;
        let mut staged = Self {
            main_path: root.join(main_name),
            root,
            guards: Vec::with_capacity(dependencies.len() + 1),
            loaded_dependencies: Vec::new(),
            directory_guard: None,
            #[cfg(windows)]
            admitted_dependencies: dependencies.to_vec(),
            #[cfg(windows)]
            retained_stagings: Vec::new(),
        };
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
            const FILE_SHARE_READ_WRITE: u32 = 3;
            staged.directory_guard = Some(
                OpenOptions::new()
                    .read(true)
                    .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
                    .share_mode(FILE_SHARE_READ_WRITE)
                    .open(&staged.root)
                    .map_err(|error| invalid(error.to_string()))?,
            );
        }
        staged.copy_verified(plugin_id, main_source, main_name, main_digest)?;
        for dependency in dependencies {
            let source = main_source
                .parent()
                .expect("canonical library has parent")
                .join(&dependency.file_name);
            staged.copy_verified(
                plugin_id,
                &source,
                &dependency.file_name,
                &dependency.digest,
            )?;
        }
        #[cfg(windows)]
        staged.validate_import_closure(&names)?;
        #[cfg(windows)]
        for dependency in dependencies {
            if let Some((library, owner)) = verify_already_loaded_dependency(dependency)? {
                staged.loaded_dependencies.push(library);
                staged.retained_stagings.push(owner);
            }
        }
        Ok(staged)
    }

    fn copy_verified(
        &mut self,
        plugin_id: &str,
        source: &Path,
        file_name: &str,
        expected: &NativePluginArtifactDigest,
    ) -> Result<(), NativePluginArtifactAdmissionError> {
        let mut input = open_library_for_admission(source)?;
        verify_handle(plugin_id, source, &input, expected)?;
        input
            .seek(SeekFrom::Start(0))
            .map_err(|error| invalid(error.to_string()))?;
        let destination = self.root.join(file_name);
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| invalid(error.to_string()))?;
        std::io::copy(&mut input, &mut output).map_err(|error| invalid(error.to_string()))?;
        output.flush().map_err(|error| invalid(error.to_string()))?;
        drop(output);
        let guard = open_library_for_admission(&destination)?;
        verify_handle(plugin_id, &destination, &guard, expected)?;
        self.guards.push(guard);
        Ok(())
    }

    pub(super) fn library_path(&self) -> &Path {
        &self.main_path
    }

    #[cfg(windows)]
    pub(super) fn register_admitted(
        staging: &Arc<Self>,
    ) -> Result<(), NativePluginArtifactAdmissionError> {
        let mut admitted = ADMITTED_STAGINGS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .map_err(|_| invalid("native staging admission registry is unavailable"))?;
        admitted.retain(|owner| owner.strong_count() != 0);
        admitted.push(Arc::downgrade(staging));
        Ok(())
    }

    #[cfg(windows)]
    fn validate_import_closure(
        &mut self,
        admitted_names: &HashSet<String>,
    ) -> Result<(), NativePluginArtifactAdmissionError> {
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;
        const WINDOWS_PATH_UNITS: usize = 32_768;
        const MAX_NATIVE_IMAGE_BYTES: u64 = 512 * 1024 * 1024;
        let mut system = vec![0_u16; WINDOWS_PATH_UNITS];
        let count =
            unsafe { GetSystemDirectoryW(system.as_mut_ptr(), system.len() as u32) } as usize;
        if count == 0 || count >= system.len() {
            return Err(invalid(
                "could not resolve trusted Windows system directory",
            ));
        }
        let system = PathBuf::from(std::ffi::OsString::from_wide(&system[..count]));
        let mut imported = HashSet::new();
        for name in admitted_names {
            let path = self.root.join(name);
            if std::fs::metadata(&path)
                .map_err(|error| invalid(error.to_string()))?
                .len()
                > MAX_NATIVE_IMAGE_BYTES
            {
                return Err(invalid("native image exceeds PE admission budget"));
            }
            let bytes = std::fs::read(&path).map_err(|error| invalid(error.to_string()))?;
            let pe = goblin::pe::PE::parse(&bytes)
                .map_err(|error| invalid(format!("native PE parse failed: {error}")))?;
            let expected_machine = if cfg!(target_arch = "x86_64") {
                goblin::pe::header::COFF_MACHINE_X86_64
            } else if cfg!(target_arch = "aarch64") {
                goblin::pe::header::COFF_MACHINE_ARM64
            } else if cfg!(target_arch = "x86") {
                goblin::pe::header::COFF_MACHINE_X86
            } else {
                return Err(invalid(
                    "native PE admission does not support this host architecture",
                ));
            };
            if !pe.is_lib || pe.header.coff_header.machine != expected_machine {
                return Err(invalid(
                    "native image does not match the host DLL architecture",
                ));
            }
            if pe
                .header
                .optional_header
                .as_ref()
                .and_then(|header| header.data_directories.get_delay_import_descriptor())
                .is_some()
            {
                return Err(invalid(
                    "delay-import native images require an explicit delay-load policy",
                ));
            }
            imported.extend(pe.libraries.iter().map(|name| name.to_ascii_lowercase()));
            if pe.exports.iter().any(|export| export.reexport.is_some()) {
                return Err(invalid("forwarded native package exports require an explicit authenticated forwarding policy"));
            }
        }
        let mut pending = imported.into_iter().collect::<Vec<_>>();
        let mut checked = HashSet::new();
        let mut system_paths = Vec::new();
        while let Some(name) = pending.pop() {
            if !checked.insert(name.clone()) {
                continue;
            }
            if checked.len() > 512 {
                return Err(invalid(
                    "Windows native dependency closure exceeds admission budget",
                ));
            }
            validate_file_name(&name)?;
            if admitted_names.contains(&name) {
                continue;
            }
            if let Ok(library) = libloading::os::windows::Library::open_already_loaded(&name) {
                let (library, loaded_path) = loaded_module_path(library)?;
                if !loaded_path.parent().is_some_and(|parent| {
                    parent
                        .as_os_str()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&system.as_os_str().to_string_lossy())
                }) {
                    return Err(invalid(format!(
                        "native system import {name} is already bound outside System32"
                    )));
                }
                self.loaded_dependencies.push(library.into());
            }
            // API set contracts are resolved by the Windows loader, never by package search paths.
            if name.starts_with("api-ms-win-") || name.starts_with("ext-ms-win-") {
                let library =
                    unsafe { libloading::os::windows::Library::load_with_flags(&name, 0x800) }
                        .map_err(|error| {
                            invalid(format!("Windows API set resolution failed: {error}"))
                        })?;
                let (library, actual_path) = loaded_module_path(library)?;
                if !actual_path.parent().is_some_and(|parent| {
                    parent
                        .as_os_str()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&system.as_os_str().to_string_lossy())
                }) {
                    return Err(invalid(format!(
                        "Windows API set {name} resolved outside System32"
                    )));
                }
                if let Some(actual_name) = actual_path.file_name().and_then(|name| name.to_str()) {
                    pending.push(actual_name.to_ascii_lowercase());
                }
                self.loaded_dependencies.push(library.into());
                continue;
            }
            let path = system.join(&name);
            let guard = open_library_for_admission(&path).map_err(|_| invalid(format!("native import {name} is outside authenticated dependency closure and Windows System32")))?;
            if guard
                .metadata()
                .map_err(|error| invalid(error.to_string()))?
                .len()
                > MAX_NATIVE_IMAGE_BYTES
            {
                return Err(invalid("Windows system image exceeds PE admission budget"));
            }
            let bytes = std::fs::read(&path).map_err(|error| invalid(error.to_string()))?;
            let pe = goblin::pe::PE::parse(&bytes)
                .map_err(|error| invalid(format!("Windows system PE parse failed: {error}")))?;
            pending.extend(pe.libraries.iter().map(|name| name.to_ascii_lowercase()));
            for export in &pe.exports {
                if let Some(reexport) = &export.reexport {
                    let library = match reexport {
                        goblin::pe::export::Reexport::DLLName { lib, .. }
                        | goblin::pe::export::Reexport::DLLOrdinal { lib, .. } => lib,
                    };
                    let mut name = library.to_ascii_lowercase();
                    if !name.ends_with(".dll") {
                        name.push_str(".dll");
                    }
                    validate_file_name(&name)?;
                    pending.push(name);
                }
            }
            self.guards.push(guard);
            system_paths.push(path);
        }
        // Pin trusted system imports so newly planted files in a DLL search directory cannot win.
        for path in system_paths {
            let library =
                unsafe { libloading::os::windows::Library::load_with_flags(&path, 0x800) }
                    .map_err(|error| {
                        invalid(format!("Windows system dependency pin failed: {error}"))
                    })?;
            let (library, actual_path) = loaded_module_path(library)?;
            if !actual_path.parent().is_some_and(|parent| {
                parent
                    .as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&system.as_os_str().to_string_lossy())
            }) {
                return Err(invalid(
                    "Windows system dependency resolved outside System32",
                ));
            }
            self.loaded_dependencies.push(library.into());
        }
        Ok(())
    }
}

impl Drop for NativePluginArtifactStaging {
    fn drop(&mut self) {
        self.guards.clear();
        self.loaded_dependencies.clear();
        self.directory_guard.take();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[cfg(windows)]
fn verify_already_loaded_dependency(
    dependency: &NativePluginArtifactDependency,
) -> Result<
    Option<(libloading::Library, Arc<NativePluginArtifactStaging>)>,
    NativePluginArtifactAdmissionError,
> {
    use libloading::os::windows::Library;

    let Ok(library) = Library::open_already_loaded(&dependency.file_name) else {
        return Ok(None);
    };
    let (library, path) = loaded_module_path(library)?;
    let admitted = ADMITTED_STAGINGS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .map_err(|_| invalid("native staging admission registry is unavailable"))?;
    let owner = admitted
        .iter()
        .filter_map(Weak::upgrade)
        .find(|owner| {
            owner.root.join(&dependency.file_name) == path
                && owner.admitted_dependencies.iter().any(|admitted| {
                    admitted.file_name.eq_ignore_ascii_case(&dependency.file_name)
                        && admitted.digest == dependency.digest
                })
        })
        .ok_or_else(|| {
            invalid(format!(
                "already loaded native dependency {} is not owned by a previously admitted staging generation",
                dependency.file_name
            ))
        })?;
    Ok(Some((library.into(), owner)))
}

#[cfg(windows)]
fn loaded_module_path(
    library: libloading::os::windows::Library,
) -> Result<(libloading::os::windows::Library, PathBuf), NativePluginArtifactAdmissionError> {
    use libloading::os::windows::Library;
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
    let handle = library.into_raw();
    let library = unsafe { Library::from_raw(handle) };
    const MAX_MODULE_PATH_UNITS: usize = 32_768;
    let mut buffer = vec![0_u16; MAX_MODULE_PATH_UNITS];
    let length =
        unsafe { GetModuleFileNameW(handle as _, buffer.as_mut_ptr(), buffer.len() as u32) }
            as usize;
    if length == 0 || length >= buffer.len() {
        return Err(invalid(
            "could not identify an already loaded native dependency",
        ));
    }
    let path = PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length]));
    Ok((library, path))
}

fn verify_handle(
    plugin_id: &str,
    path: &Path,
    file: &File,
    expected: &NativePluginArtifactDigest,
) -> Result<(), NativePluginArtifactAdmissionError> {
    let length = file
        .metadata()
        .map_err(|error| invalid(error.to_string()))?
        .len();
    if length != expected.byte_length {
        return Err(invalid(format!(
            "native artifact {} length mismatch",
            path.display()
        )));
    }
    let actual = digest_file(file, path)?;
    if actual != *expected {
        return Err(NativePluginArtifactAdmissionError::DigestMismatch {
            plugin_id: plugin_id.to_string(),
            artifact: "native closure",
            path: path.to_path_buf(),
            expected: expected.clone(),
            actual,
        });
    }
    Ok(())
}

fn validate_file_name(name: &str) -> Result<(), NativePluginArtifactAdmissionError> {
    if name.is_empty()
        || !name.is_ascii()
        || name.contains(['/', '\\', ':'])
        || name.trim() != name
        || !name.to_ascii_lowercase().ends_with(".dll")
        || name.bytes().any(|byte| byte < 32)
    {
        return Err(invalid("native dependency must be one plain DLL filename"));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> NativePluginArtifactAdmissionError {
    NativePluginArtifactAdmissionError::InvalidAuthority {
        reason: reason.into(),
    }
}

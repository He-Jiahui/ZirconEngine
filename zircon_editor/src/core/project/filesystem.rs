use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath, PROJECT_MANIFEST_FILE};

use super::ProjectAuthorityError;

pub(super) fn canonical_project_root(path: &Path) -> Result<PathBuf, ProjectAuthorityError> {
    resolve_project_root_identity(path).map(ResolvedProjectPath::into_operation_path)
}

/// Resolves a project directory or manifest input once and retains its operation/display views.
///
/// Project opens must pass this identity through to `ProjectManager::open_resolved` instead of
/// converting it to a raw path and asking a lower layer to resolve an alias again.
pub(super) fn canonical_resolved_project_root(
    path: &Path,
) -> Result<ResolvedProjectPath, ProjectAuthorityError> {
    let resolved = resolve_project_root_identity(path)?;
    validate_canonical_existing_project_root(resolved.operation_path())?;
    Ok(resolved)
}

/// Resolves a project directory or manifest input without requiring that it already exists.
///
/// Recent-project and creation-target callers use this to classify absent paths themselves;
/// open callers use [`canonical_resolved_project_root`] to require a valid project directory.
pub(super) fn resolve_project_root_identity(
    path: &Path,
) -> Result<ResolvedProjectPath, ProjectAuthorityError> {
    reject_blank_project_path(path)?;
    let root = if ProjectPaths::is_project_manifest_file(path) {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    resolve_project_path_with_identity(root)
}

/// Resolves an existing project path, or its deepest existing ancestor when publishing a new one.
///
/// This turns an OS path alias such as a Windows junction, SUBST drive, or symlink into one
/// physical project identity before project ownership code reads or writes beneath it.
pub(super) fn resolve_project_path(path: &Path) -> Result<PathBuf, ProjectAuthorityError> {
    resolve_project_path_with_identity(path).map(ResolvedProjectPath::into_operation_path)
}

pub(super) fn resolve_project_path_with_identity(
    path: &Path,
) -> Result<ResolvedProjectPath, ProjectAuthorityError> {
    reject_blank_project_path(path)?;
    ProjectPaths::resolve_path(path)
        .map_err(|source| ProjectAuthorityError::io("canonicalize project path", path, source))
}

pub(super) fn validate_creation_target(root: &Path) -> Result<(), ProjectAuthorityError> {
    reject_blank_project_path(root)?;
    reject_linked_components(root)?;
    if root.is_file() {
        return Err(ProjectAuthorityError::TargetIsFile {
            path: root.to_path_buf(),
        });
    }
    if root.is_dir() {
        let mut entries = fs::read_dir(root)
            .map_err(|source| ProjectAuthorityError::io("read target directory", root, source))?;
        if entries
            .next()
            .transpose()
            .map_err(|source| {
                ProjectAuthorityError::io("read target directory entry", root, source)
            })?
            .is_some()
        {
            return Err(ProjectAuthorityError::TargetNotEmpty {
                path: root.to_path_buf(),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_canonical_existing_project_root(
    root: &Path,
) -> Result<(), ProjectAuthorityError> {
    if !root.is_dir() {
        return Err(ProjectAuthorityError::ProjectMissing {
            path: root.to_path_buf(),
        });
    }
    let manifest = root.join(PROJECT_MANIFEST_FILE);
    if !manifest.is_file() {
        return Err(ProjectAuthorityError::ManifestMissing { path: manifest });
    }
    Ok(())
}

fn reject_blank_project_path(path: &Path) -> Result<(), ProjectAuthorityError> {
    if path.as_os_str().is_empty() || path.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(ProjectAuthorityError::EmptyProjectPath);
    }
    Ok(())
}

pub(super) fn reject_linked_components(path: &Path) -> Result<(), ProjectAuthorityError> {
    let mut existing = path;
    while !existing.exists() {
        let Some(parent) = existing.parent() else {
            break;
        };
        existing = parent;
    }
    for ancestor in existing.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|source| {
            ProjectAuthorityError::io("inspect project path", ancestor, source)
        })?;
        if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
            return Err(ProjectAuthorityError::LinkedPath {
                path: ancestor.to_path_buf(),
            });
        }
    }
    Ok(())
}

/// Keeps the project-owned directory chain stable while a scene source is opened or published.
///
/// On Windows, metadata checks alone are not sufficient because an attacker can replace a checked
/// directory with a reparse point before the runtime opens the scene path. The guarded handles are
/// opened without following reparse points and deny delete sharing, so the existing runtime path
/// APIs cannot be redirected while this guard is alive.
pub(super) fn protect_scene_path(
    project_root: &Path,
    path: &Path,
    include_file: bool,
) -> Result<ScenePathGuard, ProjectAuthorityError> {
    reject_linked_components(path)?;
    if path.strip_prefix(project_root).is_err() {
        return Err(ProjectAuthorityError::SceneTarget {
            uri: path.display().to_string(),
            reason: "resolved scene source is outside the active project root",
        });
    }
    ScenePathGuard::acquire(project_root, path, include_file)
}

#[cfg(not(windows))]
pub(super) struct ScenePathGuard;

#[cfg(not(windows))]
impl ScenePathGuard {
    fn acquire(
        _project_root: &Path,
        path: &Path,
        _include_file: bool,
    ) -> Result<Self, ProjectAuthorityError> {
        // POSIX directory descriptors do not prevent a concurrent rename. The runtime loader
        // currently accepts only paths, not an openat-derived descriptor, so allowing this route
        // would reintroduce a path-replacement window after the guard returns.
        Err(ProjectAuthorityError::SceneTarget {
            uri: path.display().to_string(),
            reason: "scene document paths require Windows no-follow lease support",
        })
    }
}

#[cfg(windows)]
pub(super) struct ScenePathGuard {
    _handles: Vec<ReparseSafeHandle>,
}

#[cfg(windows)]
impl ScenePathGuard {
    fn acquire(
        project_root: &Path,
        path: &Path,
        include_file: bool,
    ) -> Result<Self, ProjectAuthorityError> {
        let target_parent = path
            .parent()
            .ok_or_else(|| ProjectAuthorityError::SceneTarget {
                uri: path.display().to_string(),
                reason: "scene source path has no parent directory",
            })?;
        let relative_parent = target_parent.strip_prefix(project_root).map_err(|_| {
            ProjectAuthorityError::SceneTarget {
                uri: path.display().to_string(),
                reason: "resolved scene source is outside the active project root",
            }
        })?;
        let mut guarded_paths = project_root
            .ancestors()
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        guarded_paths.reverse();
        let mut current = project_root.to_path_buf();
        for component in relative_parent.components() {
            current.push(component.as_os_str());
            guarded_paths.push(current.clone());
        }
        if include_file {
            guarded_paths.push(path.to_path_buf());
        }

        let mut handles = Vec::with_capacity(guarded_paths.len());
        let file_index = include_file.then_some(guarded_paths.len().saturating_sub(1));
        for (index, guarded_path) in guarded_paths.into_iter().enumerate() {
            handles.push(ReparseSafeHandle::open(
                &guarded_path,
                Some(index) != file_index,
            )?);
        }
        Ok(Self { _handles: handles })
    }
}

#[cfg(windows)]
struct ReparseSafeHandle(isize);

#[cfg(windows)]
impl ReparseSafeHandle {
    fn open(path: &Path, allow_shared_writes: bool) -> Result<Self, ProjectAuthorityError> {
        use std::os::windows::ffi::OsStrExt;

        const FILE_READ_ATTRIBUTES: u32 = 0x0080;
        const FILE_SHARE_READ: u32 = 0x0001;
        const FILE_SHARE_WRITE: u32 = 0x0002;
        const OPEN_EXISTING: u32 = 3;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        const INVALID_HANDLE_VALUE: isize = -1;

        let mut wide_path = path.as_os_str().encode_wide().collect::<Vec<_>>();
        wide_path.push(0);
        let share_mode = if allow_shared_writes {
            FILE_SHARE_READ | FILE_SHARE_WRITE
        } else {
            FILE_SHARE_READ
        };
        let handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                FILE_READ_ATTRIBUTES,
                share_mode,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                0,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(ProjectAuthorityError::io(
                "open protected scene path",
                path,
                std::io::Error::last_os_error(),
            ));
        }

        let mut information = ByHandleFileInformation::default();
        let inspected = unsafe { GetFileInformationByHandle(handle, &mut information) } != 0;
        if !inspected {
            let error = std::io::Error::last_os_error();
            unsafe {
                CloseHandle(handle);
            }
            return Err(ProjectAuthorityError::io(
                "inspect protected scene path",
                path,
                error,
            ));
        }
        if information.file_attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            unsafe {
                CloseHandle(handle);
            }
            return Err(ProjectAuthorityError::LinkedPath {
                path: path.to_path_buf(),
            });
        }
        Ok(Self(handle))
    }
}

#[cfg(windows)]
impl Drop for ReparseSafeHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct ByHandleFileInformation {
    file_attributes: u32,
    creation_time_low: u32,
    creation_time_high: u32,
    last_access_time_low: u32,
    last_access_time_high: u32,
    last_write_time_low: u32,
    last_write_time_high: u32,
    volume_serial_number: u32,
    file_size_high: u32,
    file_size_low: u32,
    number_of_links: u32,
    file_index_high: u32,
    file_index_low: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CreateFileW(
        file_name: *const u16,
        desired_access: u32,
        share_mode: u32,
        security_attributes: *const std::ffi::c_void,
        creation_disposition: u32,
        flags_and_attributes: u32,
        template_file: isize,
    ) -> isize;
    fn GetFileInformationByHandle(handle: isize, information: *mut ByHandleFileInformation) -> i32;
    fn CloseHandle(handle: isize) -> i32;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::ProjectAuthorityError;
    use super::{canonical_project_root, protect_scene_path, validate_creation_target};

    #[test]
    fn project_root_validation_rejects_empty_and_blank_paths_before_filesystem_access() {
        for path in [Path::new(""), Path::new(" "), Path::new("\u{2003}")] {
            assert!(matches!(
                canonical_project_root(path),
                Err(ProjectAuthorityError::EmptyProjectPath)
            ));
            assert!(matches!(
                validate_creation_target(path),
                Err(ProjectAuthorityError::EmptyProjectPath)
            ));
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn scene_path_guard_rejects_unsupported_platforms_before_loading_or_publishing() {
        let root =
            std::env::temp_dir().join(format!("zircon-editor-scene-guard-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();

        let error = protect_scene_path(&root, &root.join("assets/scenes/main.scene.toml"), false)
            .unwrap_err();
        assert!(matches!(
            error,
            ProjectAuthorityError::SceneTarget { reason, .. }
                if reason.contains("Windows no-follow lease support")
        ));

        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

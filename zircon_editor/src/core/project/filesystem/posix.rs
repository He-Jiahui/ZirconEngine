//! Linux scene I/O stays relative to retained no-follow directory descriptors.
use std::ffi::{CStr, CString, OsStr};
use std::fs::{File, Metadata};
use std::io::{self, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use zircon_runtime::core::resource::io::SourceSnapshot;
use zircon_runtime::scene::world::SceneSourceRevision;

use super::ProjectAuthorityError;

pub(in crate::core::project) struct ScenePathGuard {
    path: PathBuf,
    directories: Vec<(CString, File)>,
    source: Option<File>,
    staging: Option<StagedScene>,
}

impl ScenePathGuard {
    pub(super) fn acquire(
        project_root: &Path,
        path: &Path,
        include_file: bool,
    ) -> Result<Self, ProjectAuthorityError> {
        if !project_root.is_absolute()
            || !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(invalid(
                path,
                "scene lease requires an absolute path without traversal",
            ));
        }
        let parent = path
            .parent()
            .ok_or_else(|| invalid(path, "scene source has no parent"))?;
        if parent.strip_prefix(project_root).is_err() {
            return Err(invalid(
                path,
                "scene source parent is outside the active project root",
            ));
        }
        let root_name = CString::new("/").expect("root has no NUL");
        let root = open_at(
            libc::AT_FDCWD,
            &root_name,
            libc::O_RDONLY | libc::O_DIRECTORY,
            0,
        )
        .map_err(|error| io_error(path, error))?;
        let mut directories = vec![(root_name, root)];
        for part in parent.components() {
            match part {
                Component::RootDir => continue,
                Component::Normal(name) => {
                    let name = component_name(name).map_err(|error| io_error(path, error))?;
                    let directory = open_at(
                        directories.last().expect("root descriptor").1.as_raw_fd(),
                        &name,
                        libc::O_RDONLY | libc::O_DIRECTORY,
                        0,
                    )
                    .map_err(|error| io_error(path, error))?;
                    directories.push((name, directory));
                }
                _ => {
                    return Err(invalid(
                        path,
                        "scene lease contains an invalid path component",
                    ))
                }
            }
        }
        let mut guard = Self {
            path: path.to_path_buf(),
            directories,
            source: None,
            staging: None,
        };
        if include_file {
            guard.source = Some(guard.open_file(path)?);
        }
        guard.verify()?;
        Ok(guard)
    }

    pub(in crate::core::project) fn verify(&self) -> Result<(), ProjectAuthorityError> {
        // Reopen each name through its retained parent and compare fstat identities. A rename
        // can detach our directory, but cannot redirect any of the descriptor-relative I/O.
        for pair in self.directories.windows(2) {
            let reopened = open_at(
                pair[0].1.as_raw_fd(),
                &pair[1].0,
                libc::O_RDONLY | libc::O_DIRECTORY,
                0,
            )
            .map_err(|error| io_error(&self.path, error))?;
            if !same_object(&reopened, &pair[1].1).map_err(|error| io_error(&self.path, error))? {
                return Err(invalid(
                    &self.path,
                    "scene directory identity changed during its lease",
                ));
            }
        }
        if let Some(source) = &self.source {
            let reopened = self.open_file(&self.path)?;
            if !same_object(source, &reopened).map_err(|error| io_error(&self.path, error))? {
                return Err(invalid(
                    &self.path,
                    "scene source identity changed during its lease",
                ));
            }
        } else {
            let name = self.sibling_name(&self.path)?;
            if let Some(stat) =
                stat_at(self.parent_fd(), &name).map_err(|error| io_error(&self.path, error))?
            {
                if stat.st_mode & libc::S_IFMT != libc::S_IFREG {
                    return Err(invalid(
                        &self.path,
                        "scene destination is not a regular file",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(in crate::core::project) fn read_snapshot(
        &self,
    ) -> Result<SourceSnapshot, ProjectAuthorityError> {
        self.verify()?;
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| invalid(&self.path, "scene lease has no source"))?;
        // Opening independently avoids sharing seek offsets between snapshot readers.
        let file = self.open_file(&self.path)?;
        if !same_object(source, &file).map_err(|error| io_error(&self.path, error))? {
            return Err(invalid(
                &self.path,
                "scene source identity changed before reading",
            ));
        }
        let before = file
            .metadata()
            .map_err(|error| io_error(&self.path, error))?;
        let pinned = file
            .try_clone()
            .map_err(|error| io_error(&self.path, error))?;
        let snapshot =
            SourceSnapshot::from_file(file).map_err(|error| io_error(&self.path, error))?;
        let after = pinned
            .metadata()
            .map_err(|error| io_error(&self.path, error))?;
        if content_stamp(&before) != content_stamp(&after) {
            return Err(invalid(&self.path, "scene source changed while reading"));
        }
        self.verify()?;
        Ok(snapshot)
    }

    pub(in crate::core::project) fn stage_scene(
        &mut self,
        path: &Path,
        bytes: &[u8],
    ) -> Result<SceneSourceRevision, ProjectAuthorityError> {
        self.verify()?;
        if self.staging.is_some() || path == self.path {
            return Err(invalid(
                path,
                "scene lease already owns staging or staging equals destination",
            ));
        }
        let name = self.sibling_name(path)?;
        let parent = self
            .parent()
            .try_clone()
            .map_err(|error| io_error(path, error))?;
        let file = open_at(
            self.parent_fd(),
            &name,
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )
        .map_err(|error| io_error(path, error))?;
        let mut staging = StagedScene {
            parent,
            file,
            name,
            revision: None,
        };
        staging
            .file
            .write_all(bytes)
            .map_err(|error| io_error(path, error))?;
        staging
            .file
            .sync_all()
            .map_err(|error| io_error(path, error))?;
        let snapshot = SourceSnapshot::from_file(
            staging
                .file
                .try_clone()
                .map_err(|error| io_error(path, error))?,
        )
        .map_err(|error| io_error(path, error))?;
        if snapshot.bytes() != bytes {
            return Err(invalid(path, "scene staging content changed while writing"));
        }
        let revision = snapshot.revision().clone();
        staging.revision = Some(revision.clone());
        self.verify()?;
        staging
            .verify(&staging.name)
            .map_err(|error| io_error(path, error))?;
        self.staging = Some(staging);
        Ok(revision)
    }

    pub(in crate::core::project) fn publish_scene(
        &self,
        staging_path: &Path,
    ) -> Result<(), ProjectAuthorityError> {
        self.verify()?;
        let staging = self
            .staging
            .as_ref()
            .ok_or_else(|| invalid(staging_path, "scene lease has no staging source"))?;
        if self.sibling_name(staging_path)? != staging.name {
            return Err(invalid(
                staging_path,
                "scene staging name does not match its lease",
            ));
        }
        staging
            .verify(&staging.name)
            .map_err(|error| io_error(staging_path, error))?;
        let target = self.sibling_name(&self.path)?;
        // Link the pinned inode, never a replaceable staging name. AT_EMPTY_PATH is Linux's
        // descriptor form of linkat; the destination remains create-only and never follows links.
        // SAFETY: both descriptors and the NUL-terminated destination name are live; the empty
        // source name requests the owned file descriptor rather than a path lookup.
        let result = unsafe {
            libc::linkat(
                staging.file.as_raw_fd(),
                c"".as_ptr(),
                self.parent_fd(),
                target.as_ptr(),
                libc::AT_EMPTY_PATH,
            )
        };
        if result != 0 {
            let error = io::Error::last_os_error();
            return if error.kind() == io::ErrorKind::AlreadyExists {
                Err(ProjectAuthorityError::SceneAlreadyExists {
                    path: self.path.clone(),
                })
            } else {
                Err(io_error(&self.path, error))
            };
        }
        let verified = self
            .verify()
            .and_then(|()| {
                staging
                    .verify(&target)
                    .map_err(|error| io_error(&self.path, error))
            })
            .and_then(|()| {
                self.parent()
                    .sync_all()
                    .map_err(|error| io_error(&self.path, error))
            });
        if let Err(failure) = verified {
            return match self.remove_owned_file(
                &self.path,
                staging.revision.as_ref().expect("staged revision"),
            ) {
                Ok(()) => Err(failure),
                Err(rollback) => Err(ProjectAuthorityError::SceneLeaseRollback {
                    failure: Box::new(failure),
                    rollback: Box::new(rollback),
                }),
            };
        }
        Ok(())
    }

    pub(in crate::core::project) fn remove_owned_file(
        &self,
        path: &Path,
        expected: &SceneSourceRevision,
    ) -> Result<(), ProjectAuthorityError> {
        let staging = self
            .staging
            .as_ref()
            .ok_or_else(|| invalid(path, "scene lease has no owned source"))?;
        let name = self.sibling_name(path)?;
        if (path != self.path && name != staging.name)
            || staging.revision.as_ref() != Some(expected)
        {
            return Err(invalid(
                path,
                "scene cleanup does not match its owned publication",
            ));
        }
        // Deliberately do not require the path chain to remain attached: rollback must clean the
        // leased original directory even if its former name now points into another tree.
        staging.remove(&name).map_err(|error| io_error(path, error))
    }

    fn parent(&self) -> &File {
        &self.directories.last().expect("leased root").1
    }

    fn parent_fd(&self) -> RawFd {
        self.parent().as_raw_fd()
    }

    fn sibling_name(&self, path: &Path) -> Result<CString, ProjectAuthorityError> {
        if path.parent() != self.path.parent() {
            return Err(invalid(
                path,
                "scene operation must stay in its leased parent directory",
            ));
        }
        let name = path
            .file_name()
            .ok_or_else(|| invalid(path, "scene source has no file name"))?;
        component_name(name).map_err(|error| io_error(path, error))
    }

    fn open_file(&self, path: &Path) -> Result<File, ProjectAuthorityError> {
        let name = self.sibling_name(path)?;
        let file = open_at(
            self.parent_fd(),
            &name,
            libc::O_RDONLY | libc::O_NONBLOCK,
            0,
        )
        .map_err(|error| io_error(path, error))?;
        ensure_regular(&file).map_err(|error| io_error(path, error))?;
        Ok(file)
    }
}

struct StagedScene {
    parent: File,
    file: File,
    name: CString,
    revision: Option<SceneSourceRevision>,
}

impl StagedScene {
    fn verify(&self, name: &CStr) -> io::Result<()> {
        let current = open_at(
            self.parent.as_raw_fd(),
            name,
            libc::O_RDONLY | libc::O_NONBLOCK,
            0,
        )?;
        ensure_regular(&current)?;
        if !same_object(&self.file, &current)? {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "owned scene file identity changed",
            ));
        }
        if let Some(expected) = &self.revision {
            if SourceSnapshot::from_file(current)?.revision() != expected {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "owned scene file content changed",
                ));
            }
        }
        Ok(())
    }

    fn remove(&self, name: &CStr) -> io::Result<()> {
        if stat_at(self.parent.as_raw_fd(), name)?.is_none() {
            return Ok(());
        }
        self.verify(name)?;
        // unlinkat removes the directory entry itself; it never follows a replacement symlink.
        // SAFETY: the retained parent descriptor and NUL-terminated leaf remain live for the call.
        let result = unsafe { libc::unlinkat(self.parent.as_raw_fd(), name.as_ptr(), 0) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        self.parent.sync_all()
    }
}

impl Drop for StagedScene {
    fn drop(&mut self) {
        let _ = self.remove(&self.name);
    }
}

fn open_at(parent: RawFd, name: &CStr, flags: i32, mode: libc::mode_t) -> io::Result<File> {
    // SAFETY: the directory descriptor and NUL-terminated name are live for this call. The new
    // descriptor is transferred to File exactly once, with no-follow and close-on-exec enforced.
    let fd = unsafe {
        libc::openat(
            parent,
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            mode,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: openat returned a new owned descriptor; no other File has taken ownership of it.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn stat_at(parent: RawFd, name: &CStr) -> io::Result<Option<libc::stat>> {
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: fstatat initializes stat only on success; the name and descriptor remain live.
    let result = unsafe {
        libc::fstatat(
            parent,
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result == 0 {
        // SAFETY: successful fstatat initialized every field of the stat buffer.
        return Ok(Some(unsafe { stat.assume_init() }));
    }
    let error = io::Error::last_os_error();
    if error.kind() == io::ErrorKind::NotFound {
        Ok(None)
    } else {
        Err(error)
    }
}

fn component_name(name: &OsStr) -> io::Result<CString> {
    CString::new(name.as_bytes())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

fn ensure_regular(file: &File) -> io::Result<()> {
    if file.metadata()?.is_file() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "scene source is not a regular file",
        ))
    }
}

fn same_object(a: &File, b: &File) -> io::Result<bool> {
    let a = a.metadata()?;
    let b = b.metadata()?;
    Ok(a.dev() == b.dev() && a.ino() == b.ino())
}

fn content_stamp(metadata: &Metadata) -> (u64, i64, i64, i64, i64) {
    (
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

fn invalid(path: &Path, reason: &'static str) -> ProjectAuthorityError {
    ProjectAuthorityError::SceneTarget {
        uri: path.display().to_string(),
        reason,
    }
}

fn io_error(path: &Path, source: io::Error) -> ProjectAuthorityError {
    ProjectAuthorityError::io("use protected scene descriptor", path, source)
}

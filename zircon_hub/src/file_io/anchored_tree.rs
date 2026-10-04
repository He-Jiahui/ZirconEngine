mod entries;
pub(crate) use entries::AnchoredEntry;

use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

const MAX_TREE_DEPTH: usize = 256;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TreeRemovalLimits {
    pub(crate) entries: usize,
    pub(crate) directories: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TreeRemovalUsage {
    pub(crate) entries: usize,
    pub(crate) directories: usize,
}

/// A directory handle whose operations stay relative to the opened directory.
///
/// On Windows the root path and each ancestor remain open without delete sharing,
/// preventing an ancestor rename while path-based directory enumeration is used.
/// On Unix the root handle pins the opened inode and all child operations use
/// `*at` calls relative to it.
pub(crate) struct AnchoredDirectory {
    handle: File,
    path: PathBuf,
    #[cfg(windows)]
    _ancestors: Vec<File>,
}

impl AnchoredDirectory {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        Self::open_with_access(path, false, false)
    }

    pub(crate) fn open_for_file_writes(path: &Path) -> io::Result<Self> {
        Self::open_with_access(path, true, false)
    }

    pub(crate) fn open_for_directory_writes(path: &Path) -> io::Result<Self> {
        Self::open_with_access(path, false, true)
    }

    fn open_with_access(
        path: &Path,
        allow_file_creation: bool,
        allow_directory_creation: bool,
    ) -> io::Result<Self> {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;

            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            let path = std::path::absolute(path)?;
            validate_absolute_components(&path)?;
            let mut opened = Vec::new();
            for ancestor in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
                let is_final_directory = ancestor == path;
                let directory = unsafe {
                    super::windows::open_no_reparse(
                        ancestor,
                        0x80 | if is_final_directory && allow_file_creation {
                            0x2
                        } else {
                            0
                        } | if is_final_directory && allow_directory_creation {
                            0x4
                        } else {
                            0
                        },
                        // FILE_READ_ATTRIBUTES | requested creation right.
                        3, // FILE_SHARE_READ | FILE_SHARE_WRITE; deny delete sharing.
                        super::windows::FILE_OPEN,
                        true,
                        std::ptr::null_mut(),
                    )?
                };
                let metadata = directory.metadata()?;
                if !metadata.is_dir()
                    || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
                {
                    return Err(invalid_tree());
                }
                opened.push(directory);
            }
            let handle = opened.pop().ok_or_else(invalid_tree)?;
            Ok(Self {
                handle,
                path,
                _ancestors: opened,
            })
        }

        #[cfg(unix)]
        {
            let _ = (allow_file_creation, allow_directory_creation);
            use std::{
                ffi::CString,
                os::{
                    fd::{AsRawFd, FromRawFd},
                    unix::ffi::OsStrExt,
                },
            };

            let path = std::path::absolute(path)?;
            let mut components = path.components();
            if components.next() != Some(Component::RootDir) {
                return Err(invalid_tree());
            }
            let mut handle = File::open("/")?;
            for component in components {
                let Component::Normal(name) = component else {
                    return Err(invalid_tree());
                };
                let name = CString::new(name.as_bytes()).map_err(|_| invalid_tree())?;
                let descriptor = unsafe {
                    libc::openat(
                        handle.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY
                            | libc::O_DIRECTORY
                            | libc::O_NOFOLLOW
                            | libc::O_CLOEXEC
                            | libc::O_NONBLOCK,
                    )
                };
                if descriptor < 0 {
                    return Err(io::Error::last_os_error());
                }
                handle = unsafe { File::from_raw_fd(descriptor) };
            }
            if !handle.metadata()?.is_dir() {
                return Err(invalid_tree());
            }
            Ok(Self { handle, path })
        }

        #[cfg(not(any(windows, unix)))]
        {
            let _ = (path, allow_file_creation, allow_directory_creation);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "anchored directory operations are unavailable",
            ))
        }
    }

    pub(crate) fn remove_tree(
        &self,
        name: &OsStr,
        limits: TreeRemovalLimits,
    ) -> io::Result<TreeRemovalUsage> {
        self.remove_tree_inner(name, None, limits)
    }

    /// Removes the contents first, then the identity marker, then the operation directory.
    /// A missing marker is accepted with expected bytes only after an empty-tree preflight.
    pub(crate) fn remove_tree_preserving_marker(
        &self,
        name: &OsStr,
        marker: &OsStr,
        expected_marker: Option<&[u8]>,
        limits: TreeRemovalLimits,
    ) -> io::Result<TreeRemovalUsage> {
        validate_component(marker)?;
        self.remove_tree_inner(name, Some((marker, expected_marker)), limits)
    }

    pub(crate) fn remove_file(&self, name: &OsStr) -> io::Result<()> {
        validate_component(name)?;
        let file = self.open_child(name, ChildKind::File)?;
        remove_open_child(self, name, &file, ChildKind::File)
    }

    pub(crate) fn create_new_file(&self, name: &OsStr) -> io::Result<File> {
        self.create_new_file_with_mode(name, 0o600, true)
    }

    pub(crate) fn create_new_project_file(&self, name: &OsStr) -> io::Result<File> {
        self.create_new_file_with_mode(name, 0o666, false)
    }

    fn create_new_file_with_mode(
        &self,
        name: &OsStr,
        unix_mode: u32,
        private: bool,
    ) -> io::Result<File> {
        validate_component(name)?;
        #[cfg(windows)]
        {
            let _ = unix_mode;
            let file = if private {
                super::windows::create_relative_private_file(&self.handle, name)?
            } else {
                super::windows::create_relative_file(&self.handle, name)?
            };
            verify_child(&file, ChildKind::File)?;
            Ok(file)
        }
        #[cfg(unix)]
        {
            let _ = private;
            use std::{
                ffi::CString,
                os::{
                    fd::{AsRawFd, FromRawFd},
                    unix::ffi::OsStrExt,
                },
            };
            let name = CString::new(name.as_bytes()).map_err(|_| invalid_tree())?;
            let descriptor = unsafe {
                libc::openat(
                    self.handle.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC
                        | libc::O_NONBLOCK,
                    unix_mode,
                )
            };
            if descriptor < 0 {
                return Err(io::Error::last_os_error());
            }
            let file = unsafe { File::from_raw_fd(descriptor) };
            verify_child(&file, ChildKind::File)?;
            Ok(file)
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = (name, unix_mode, private);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "anchored file creation is unavailable",
            ))
        }
    }

    pub(crate) fn hard_link(&self, source: &OsStr, target: &OsStr) -> io::Result<()> {
        validate_component(source)?;
        validate_component(target)?;
        #[cfg(windows)]
        {
            let source = self.open_child(source, ChildKind::File)?;
            super::windows::hard_link_relative(&source, target)
        }
        #[cfg(unix)]
        {
            use std::{
                ffi::CString,
                os::{fd::AsRawFd, unix::ffi::OsStrExt},
            };
            let source = CString::new(source.as_bytes()).map_err(|_| invalid_tree())?;
            let target = CString::new(target.as_bytes()).map_err(|_| invalid_tree())?;
            let result = unsafe {
                libc::linkat(
                    self.handle.as_raw_fd(),
                    source.as_ptr(),
                    self.handle.as_raw_fd(),
                    target.as_ptr(),
                    0,
                )
            };
            if result == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = (source, target);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "anchored hard links are unavailable",
            ))
        }
    }

    pub(crate) fn ensure_child_directory(&self, name: &OsStr) -> io::Result<Self> {
        self.ensure_child_directory_with_mode(name, 0o700, true)
    }

    pub(crate) fn ensure_project_child_directory(&self, name: &OsStr) -> io::Result<Self> {
        self.ensure_child_directory_with_mode(name, 0o777, false)
    }

    pub(crate) fn open_existing_private_child(&self, name: &OsStr) -> io::Result<Self> {
        validate_component(name)?;
        #[cfg(windows)]
        let handle = {
            const FILE_READ_ATTRIBUTES: u32 = 0x80;
            const WRITE_DAC: u32 = 0x0004_0000;
            let file = super::windows::open_relative(
                &self.handle,
                name,
                FILE_READ_ATTRIBUTES | WRITE_DAC,
                Some(true),
                3,
            )?;
            verify_child(&file, ChildKind::Directory)?;
            super::windows::protect_private_object(&file)?;
            file
        };
        #[cfg(unix)]
        let handle = {
            let directory = self.open_child(name, ChildKind::Directory)?;
            use std::os::unix::fs::PermissionsExt;
            directory.set_permissions(std::fs::Permissions::from_mode(0o700))?;
            directory
        };
        #[cfg(not(any(windows, unix)))]
        let handle = {
            let _ = name;
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "private directory protection is unavailable",
            ));
        };

        let mut child = Self {
            handle,
            path: self.path.join(name),
            #[cfg(windows)]
            _ancestors: Vec::new(),
        };
        #[cfg(windows)]
        {
            child._ancestors = self
                ._ancestors
                .iter()
                .map(File::try_clone)
                .collect::<io::Result<Vec<_>>>()?;
            child._ancestors.push(self.handle.try_clone()?);
        }
        Ok(child)
    }

    /// Opens and hardens an existing private artifact while retaining the same no-follow handle
    /// that callers use to read it.
    pub(crate) fn open_existing_private_file(&self, name: &OsStr) -> io::Result<File> {
        validate_component(name)?;
        #[cfg(windows)]
        {
            const FILE_READ_DATA: u32 = 0x1;
            const FILE_READ_ATTRIBUTES: u32 = 0x80;
            const WRITE_DAC: u32 = 0x0004_0000;
            let file = super::windows::open_relative(
                &self.handle,
                name,
                FILE_READ_DATA | FILE_READ_ATTRIBUTES | WRITE_DAC,
                Some(false),
                3,
            )?;
            verify_child(&file, ChildKind::File)?;
            super::windows::protect_private_object(&file)?;
            Ok(file)
        }
        #[cfg(unix)]
        {
            let file = self.open_child(name, ChildKind::File)?;
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
            Ok(file)
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = name;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "private file protection is unavailable",
            ))
        }
    }

    fn ensure_child_directory_with_mode(
        &self,
        name: &OsStr,
        unix_mode: u32,
        private: bool,
    ) -> io::Result<Self> {
        validate_component(name)?;
        #[cfg(windows)]
        {
            let _ = unix_mode;
            drop(if private {
                super::windows::create_relative_private_directory(&self.handle, name)?
            } else {
                super::windows::create_relative_directory(&self.handle, name)?
            });
        }
        #[cfg(unix)]
        {
            let _ = private;
            use std::{ffi::CString, os::fd::AsRawFd, os::unix::ffi::OsStrExt};
            let name_c = CString::new(name.as_bytes()).map_err(|_| invalid_tree())?;
            let result =
                unsafe { libc::mkdirat(self.handle.as_raw_fd(), name_c.as_ptr(), unix_mode) };
            if result != 0 {
                let error = io::Error::last_os_error();
                if error.kind() != io::ErrorKind::AlreadyExists {
                    return Err(error);
                }
            }
        }
        self.open_child_directory_for_creation(name, private)
    }

    fn open_child_directory_for_creation(&self, name: &OsStr, private: bool) -> io::Result<Self> {
        validate_component(name)?;
        #[cfg(windows)]
        let handle = {
            const FILE_READ_ATTRIBUTES: u32 = 0x80;
            const FILE_ADD_SUBDIRECTORY: u32 = 0x4;
            const WRITE_DAC: u32 = 0x0004_0000;
            let file = super::windows::open_relative(
                &self.handle,
                name,
                FILE_READ_ATTRIBUTES | FILE_ADD_SUBDIRECTORY | if private { WRITE_DAC } else { 0 },
                Some(true),
                3,
            )?;
            verify_child(&file, ChildKind::Directory)?;
            if private {
                super::windows::protect_private_object(&file)?;
            }
            file
        };
        #[cfg(unix)]
        let handle = {
            let directory = self.open_child(name, ChildKind::Directory)?;
            if private {
                use std::os::unix::fs::PermissionsExt;
                directory.set_permissions(std::fs::Permissions::from_mode(0o700))?;
            }
            directory
        };
        let mut child = Self {
            handle,
            path: self.path.join(name),
            #[cfg(windows)]
            _ancestors: Vec::new(),
        };
        #[cfg(windows)]
        {
            child._ancestors = self
                ._ancestors
                .iter()
                .map(File::try_clone)
                .collect::<io::Result<Vec<_>>>()?;
            child._ancestors.push(self.handle.try_clone()?);
        }
        Ok(child)
    }

    pub(crate) fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            handle: self.handle.try_clone()?,
            path: self.path.clone(),
            #[cfg(windows)]
            _ancestors: self
                ._ancestors
                .iter()
                .map(File::try_clone)
                .collect::<io::Result<Vec<_>>>()?,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Reopens the public path without following links and verifies it still names this handle.
    pub(crate) fn path_still_matches(&self) -> io::Result<bool> {
        let current = Self::open(&self.path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let opened = self.handle.metadata()?;
            let current = current.handle.metadata()?;
            Ok(opened.dev() == current.dev() && opened.ino() == current.ino())
        }
        #[cfg(windows)]
        {
            super::windows::same_file(&self.handle, &current.handle)
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = current;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "anchored identity comparison is unavailable",
            ))
        }
    }

    pub(crate) fn remove_empty_directory(&self, name: &OsStr) -> io::Result<()> {
        validate_component(name)?;
        let directory = self.open_child_directory_for_removal(name)?;
        if directory.has_any_entry()? {
            return Err(io::Error::new(
                io::ErrorKind::DirectoryNotEmpty,
                "directory is not empty",
            ));
        }
        remove_open_child(self, name, &directory.handle, ChildKind::Directory)
    }

    pub(crate) fn remove_empty_directory_if_empty(&self, name: &OsStr) -> io::Result<bool> {
        validate_component(name)?;
        let directory = self.open_child_directory_for_removal(name)?;
        if directory.has_any_entry()? {
            return Ok(false);
        }
        remove_open_child(self, name, &directory.handle, ChildKind::Directory)?;
        Ok(true)
    }

    pub(crate) fn remove_file_path(path: &Path) -> io::Result<()> {
        let (parent, name) = split_path(path)?;
        Self::open(parent)?.remove_file(name)
    }

    pub(crate) fn remove_empty_directory_path(path: &Path) -> io::Result<()> {
        let (parent, name) = split_path(path)?;
        Self::open(parent)?.remove_empty_directory(name)
    }

    pub(crate) fn remove_empty_directory_path_if_empty(path: &Path) -> io::Result<bool> {
        let (parent, name) = split_path(path)?;
        Self::open(parent)?.remove_empty_directory_if_empty(name)
    }

    fn remove_tree_inner(
        &self,
        name: &OsStr,
        marker: Option<(&OsStr, Option<&[u8]>)>,
        limits: TreeRemovalLimits,
    ) -> io::Result<TreeRemovalUsage> {
        self.remove_tree_inner_with_hook(name, marker, limits, || {})
    }

    fn remove_tree_inner_with_hook<F>(
        &self,
        name: &OsStr,
        marker: Option<(&OsStr, Option<&[u8]>)>,
        limits: TreeRemovalLimits,
        after_marker_check: F,
    ) -> io::Result<TreeRemovalUsage>
    where
        F: FnOnce(),
    {
        validate_component(name)?;
        let root = match self.open_child_directory_for_removal(name) {
            Ok(root) => root,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(TreeRemovalUsage::default());
            }
            Err(error) => return Err(error),
        };
        let mut usage = TreeRemovalUsage::default();
        visit_directory(&root, 0, limits, &mut usage)?;
        let marker_file = if let Some((marker_name, expected_bytes)) = marker {
            match root.open_child_with_share(marker_name, ChildKind::Any, 1, true) {
                Ok(file) => {
                    if child_kind(&file)? != ChildKind::File {
                        return Err(invalid_tree());
                    }
                    if let Some(expected_bytes) = expected_bytes {
                        let mut reader = file.try_clone()?;
                        if reader.metadata()?.len() != expected_bytes.len() as u64 {
                            return Err(invalid_tree());
                        }
                        let bound = (expected_bytes.len() as u64)
                            .checked_add(1)
                            .ok_or_else(tree_limit)?;
                        let mut bytes = Vec::with_capacity(expected_bytes.len());
                        reader.take(bound).read_to_end(&mut bytes)?;
                        if bytes != expected_bytes {
                            return Err(invalid_tree());
                        }
                    }
                    Some(file)
                }
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        && (expected_bytes.is_none() || usage.entries == 0) =>
                {
                    None
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        };
        let retrying_markerless_empty_tree = marker
            .is_some_and(|(_, expected_bytes)| expected_bytes.is_some())
            && marker_file.is_none();
        after_marker_check();

        if retrying_markerless_empty_tree {
            // The marker was already removed by an interrupted prior cleanup. The empty
            // preflight is the only remaining identity proof, so never recurse after it.
            remove_open_child(self, name, &root.handle, ChildKind::Directory)?;
            return Ok(usage);
        }

        let mut removal_usage = TreeRemovalUsage {
            directories: 1,
            ..TreeRemovalUsage::default()
        };
        remove_directory_contents(
            &root,
            marker.map(|(name, _)| name),
            0,
            limits,
            &mut removal_usage,
        )?;
        if let (Some((marker, _)), Some(marker_file)) = (marker, marker_file) {
            remove_open_child(&root, marker, &marker_file, ChildKind::File)?;
        }
        remove_open_child(self, name, &root.handle, ChildKind::Directory)?;
        Ok(usage)
    }

    fn open_child(&self, name: &OsStr, expected: ChildKind) -> io::Result<File> {
        self.open_child_with_share(name, expected, 3, false)
    }

    fn open_child_with_share(
        &self,
        name: &OsStr,
        expected: ChildKind,
        share: u32,
        read_contents: bool,
    ) -> io::Result<File> {
        validate_component(name)?;
        #[cfg(windows)]
        {
            const FILE_READ_DATA: u32 = 0x1;
            const FILE_READ_ATTRIBUTES: u32 = 0x80;
            const DELETE: u32 = 0x0001_0000;
            let directory = match expected {
                ChildKind::Directory => Some(true),
                ChildKind::File => Some(false),
                ChildKind::Any => None,
            };
            let access =
                FILE_READ_ATTRIBUTES | DELETE | if read_contents { FILE_READ_DATA } else { 0 };
            let file = super::windows::open_relative(&self.handle, name, access, directory, share)?;
            verify_child(&file, expected)?;
            Ok(file)
        }

        #[cfg(unix)]
        {
            let _ = (share, read_contents);
            use std::{
                ffi::CString,
                os::{
                    fd::{AsRawFd, FromRawFd},
                    unix::ffi::OsStrExt,
                },
            };
            let name = CString::new(name.as_bytes()).map_err(|_| invalid_tree())?;
            let flags = libc::O_RDONLY
                | libc::O_CLOEXEC
                | libc::O_NOFOLLOW
                | libc::O_NONBLOCK
                | if expected == ChildKind::Directory {
                    libc::O_DIRECTORY
                } else {
                    0
                };
            let descriptor = unsafe { libc::openat(self.handle.as_raw_fd(), name.as_ptr(), flags) };
            if descriptor < 0 {
                return Err(io::Error::last_os_error());
            }
            let file = unsafe { File::from_raw_fd(descriptor) };
            verify_child(&file, expected)?;
            Ok(file)
        }

        #[cfg(not(any(windows, unix)))]
        {
            let _ = (name, expected);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "anchored child opens are unavailable",
            ))
        }
    }

    fn open_child_directory_for_removal(&self, name: &OsStr) -> io::Result<Self> {
        let handle = self.open_child(name, ChildKind::Directory)?;
        let path = self.path.join(name);
        Ok(Self {
            handle,
            path,
            #[cfg(windows)]
            _ancestors: Vec::new(),
        })
    }

    fn has_any_entry(&self) -> io::Result<bool> {
        match self.read_names(1) {
            Ok(names) => Ok(!names.is_empty()),
            // A strict one-entry scan reports tree_limit only after it has observed
            // a second entry, which is enough to prove this directory is nonempty.
            Err(error)
                if error.kind() == io::ErrorKind::InvalidData
                    && error.to_string() == "directory tree exceeds removal limits" =>
            {
                Ok(true)
            }
            Err(error) => Err(error),
        }
    }

    /// Enumerates the opened directory with a strict entry bound.
    pub(crate) fn read_names(&self, maximum: usize) -> io::Result<Vec<OsString>> {
        #[cfg(windows)]
        {
            let mut names = Vec::new();
            for entry in std::fs::read_dir(&self.path)? {
                let name = entry?.file_name();
                if name == OsStr::new(".") || name == OsStr::new("..") {
                    continue;
                }
                if names.len() == maximum {
                    return Err(tree_limit());
                }
                names.push(name);
            }
            Ok(names)
        }

        #[cfg(unix)]
        {
            use std::{ffi::CStr, os::fd::AsRawFd};
            let descriptor = unsafe {
                libc::openat(
                    self.handle.as_raw_fd(),
                    b".\0".as_ptr().cast(),
                    libc::O_RDONLY
                        | libc::O_DIRECTORY
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC
                        | libc::O_NONBLOCK,
                )
            };
            if descriptor < 0 {
                return Err(io::Error::last_os_error());
            }
            let directory = unsafe { libc::fdopendir(descriptor) };
            if directory.is_null() {
                let error = io::Error::last_os_error();
                unsafe { libc::close(descriptor) };
                return Err(error);
            }
            let mut names = Vec::new();
            let mut failure = None;
            loop {
                if let Err(error) = clear_posix_errno() {
                    failure = Some(error);
                    break;
                }
                let entry = unsafe { libc::readdir(directory) };
                if entry.is_null() {
                    let errno = io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    match classify_readdir_null(errno, libc::EINTR) {
                        Ok(true) => break,
                        Ok(false) => continue,
                        Err(error) => {
                            failure = Some(error);
                            break;
                        }
                    }
                }
                let bytes = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
                if bytes == b"." || bytes == b".." {
                    continue;
                }
                if names.len() == maximum {
                    failure = Some(tree_limit());
                    break;
                }
                use std::os::unix::ffi::OsStringExt;
                names.push(OsString::from_vec(bytes.to_vec()));
            }
            let close_status = unsafe { libc::closedir(directory) };
            if close_status != 0 && failure.is_none() {
                failure = Some(io::Error::last_os_error());
            }
            if let Some(error) = failure {
                return Err(error);
            }
            Ok(names)
        }

        #[cfg(not(any(windows, unix)))]
        {
            let _ = maximum;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "anchored directory enumeration is unavailable",
            ))
        }
    }
}

fn classify_readdir_null(errno: i32, interrupted: i32) -> io::Result<bool> {
    match errno {
        0 => Ok(true),
        code if code == interrupted => Ok(false),
        code => Err(io::Error::from_raw_os_error(code)),
    }
}

#[cfg(all(unix, any(target_os = "linux", target_os = "android")))]
fn clear_posix_errno() -> io::Result<()> {
    unsafe { *libc::__errno_location() = 0 };
    Ok(())
}

#[cfg(all(
    unix,
    any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )
))]
fn clear_posix_errno() -> io::Result<()> {
    unsafe { *libc::__error() = 0 };
    Ok(())
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
fn clear_posix_errno() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "readdir error checks are unavailable on this Unix target",
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChildKind {
    Any,
    File,
    Directory,
}

fn visit_directory(
    directory: &AnchoredDirectory,
    depth: usize,
    limits: TreeRemovalLimits,
    usage: &mut TreeRemovalUsage,
) -> io::Result<()> {
    if depth > MAX_TREE_DEPTH {
        return Err(tree_limit());
    }
    usage.directories = usage.directories.checked_add(1).ok_or_else(tree_limit)?;
    if usage.directories > limits.directories {
        return Err(tree_limit());
    }
    let remaining = limits.entries.saturating_sub(usage.entries);
    for name in directory.read_names(remaining)? {
        usage.entries = usage.entries.checked_add(1).ok_or_else(tree_limit)?;
        if usage.entries > limits.entries {
            return Err(tree_limit());
        }
        let child = directory.open_child(&name, ChildKind::Any)?;
        match child_kind(&child)? {
            ChildKind::Directory => {
                let child = directory.open_child_directory_for_removal(&name)?;
                visit_directory(&child, depth + 1, limits, usage)?;
            }
            ChildKind::File => {}
            ChildKind::Any => unreachable!(),
        }
    }
    Ok(())
}

fn remove_directory_contents(
    directory: &AnchoredDirectory,
    skip: Option<&OsStr>,
    depth: usize,
    limits: TreeRemovalLimits,
    usage: &mut TreeRemovalUsage,
) -> io::Result<()> {
    if depth > MAX_TREE_DEPTH {
        return Err(tree_limit());
    }
    let remaining = limits.entries.saturating_sub(usage.entries);
    for name in directory.read_names(remaining)? {
        if skip.is_some_and(|skip| skip == name) {
            continue;
        }
        usage.entries = usage.entries.checked_add(1).ok_or_else(tree_limit)?;
        if usage.entries > limits.entries {
            return Err(tree_limit());
        }
        let child = directory.open_child(&name, ChildKind::Any)?;
        match child_kind(&child)? {
            ChildKind::Directory => {
                let child = directory.open_child_directory_for_removal(&name)?;
                usage.directories = usage.directories.checked_add(1).ok_or_else(tree_limit)?;
                if usage.directories > limits.directories {
                    return Err(tree_limit());
                }
                remove_directory_contents(&child, None, depth + 1, limits, usage)?;
                remove_open_child(directory, &name, &child.handle, ChildKind::Directory)?;
            }
            ChildKind::File => remove_open_child(directory, &name, &child, ChildKind::File)?,
            ChildKind::Any => unreachable!(),
        }
    }
    Ok(())
}

fn remove_open_child(
    parent: &AnchoredDirectory,
    name: &OsStr,
    child: &File,
    kind: ChildKind,
) -> io::Result<()> {
    verify_child(child, kind)?;
    #[cfg(windows)]
    {
        super::windows::mark_delete(child)
    }
    #[cfg(unix)]
    {
        use std::{
            ffi::CString,
            os::{
                fd::AsRawFd,
                unix::{ffi::OsStrExt, fs::MetadataExt},
            },
        };
        let name_c = CString::new(name.as_bytes()).map_err(|_| invalid_tree())?;
        let opened = child.metadata()?;
        let mut current: libc::stat = unsafe { std::mem::zeroed() };
        let status = unsafe {
            libc::fstatat(
                parent.handle.as_raw_fd(),
                name_c.as_ptr(),
                &mut current,
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if status != 0 {
            return Err(io::Error::last_os_error());
        }
        let current_dev = current.st_dev as u64;
        let current_ino = current.st_ino as u64;
        if opened.dev() != current_dev || opened.ino() != current_ino {
            return Err(invalid_tree());
        }
        let flags = if kind == ChildKind::Directory {
            libc::AT_REMOVEDIR
        } else {
            0
        };
        let result = unsafe { libc::unlinkat(parent.handle.as_raw_fd(), name_c.as_ptr(), flags) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (parent, name, child, kind);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "anchored deletion is unavailable",
        ))
    }
}

fn verify_child(file: &File, expected: ChildKind) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        let metadata = file.metadata()?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(invalid_tree());
        }
        let actual = if metadata.is_dir() {
            ChildKind::Directory
        } else if metadata.is_file() {
            ChildKind::File
        } else {
            return Err(invalid_tree());
        };
        if expected != ChildKind::Any && expected != actual {
            return Err(invalid_tree());
        }
        Ok(())
    }
    #[cfg(unix)]
    {
        let metadata = file.metadata()?;
        let actual = if metadata.is_dir() {
            ChildKind::Directory
        } else if metadata.is_file() {
            ChildKind::File
        } else {
            return Err(invalid_tree());
        };
        if expected != ChildKind::Any && expected != actual {
            return Err(invalid_tree());
        }
        Ok(())
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (file, expected);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "anchored child validation is unavailable",
        ))
    }
}

fn child_kind(file: &File) -> io::Result<ChildKind> {
    verify_child(file, ChildKind::Any)?;
    let metadata = file.metadata()?;
    Ok(if metadata.is_dir() {
        ChildKind::Directory
    } else {
        ChildKind::File
    })
}

fn validate_component(name: &OsStr) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(invalid_tree());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        if name.encode_wide().any(|unit| {
            unit == 0 || unit == b':' as u16 || unit == b'/' as u16 || unit == b'\\' as u16
        }) {
            return Err(invalid_tree());
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        if name.as_bytes().contains(&0) || name.as_bytes().contains(&b'/') {
            return Err(invalid_tree());
        }
    }
    Ok(())
}

fn split_path(path: &Path) -> io::Result<(&Path, &OsStr)> {
    let parent = path.parent().ok_or_else(invalid_tree)?;
    let name = path.file_name().ok_or_else(invalid_tree)?;
    validate_component(name)?;
    Ok((parent, name))
}

#[cfg(windows)]
fn validate_absolute_components(path: &Path) -> io::Result<()> {
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Prefix(_)))
        || components.next() != Some(Component::RootDir)
    {
        return Err(invalid_tree());
    }
    for component in components {
        let Component::Normal(name) = component else {
            return Err(invalid_tree());
        };
        validate_component(name)?;
    }
    Ok(())
}

fn invalid_tree() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "expected a private regular directory tree",
    )
}

fn tree_limit() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "directory tree exceeds removal limits",
    )
}

#[cfg(test)]
#[path = "tests/anchored_tree.rs"]
mod tests;

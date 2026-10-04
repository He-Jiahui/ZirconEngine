use super::{child_kind, remove_open_child, validate_component, AnchoredDirectory, ChildKind};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io,
};

/// An entry remains bound to its opened parent until inspection and removal finish.
pub(crate) enum AnchoredEntry<'a> {
    Directory(AnchoredChildDirectory<'a>),
    File(AnchoredFile<'a>),
}

pub(crate) struct AnchoredChildDirectory<'a> {
    parent: &'a AnchoredDirectory,
    name: OsString,
    directory: AnchoredDirectory,
}

impl AnchoredChildDirectory<'_> {
    pub(crate) fn as_directory(&self) -> &AnchoredDirectory {
        &self.directory
    }

    /// Removes the inspected directory handle, after checking that handle for emptiness.
    pub(crate) fn remove_if_empty(self) -> io::Result<bool> {
        if self.directory.has_any_entry()? {
            return Ok(false);
        }
        remove_open_child(
            self.parent,
            &self.name,
            &self.directory.handle,
            ChildKind::Directory,
        )?;
        Ok(true)
    }
}

pub(crate) struct AnchoredFile<'a> {
    parent: &'a AnchoredDirectory,
    name: OsString,
    file: File,
}

impl AnchoredFile<'_> {
    /// The original handle remains open while an existing bounded reader consumes its clone.
    pub(crate) fn try_clone_reader(&self) -> io::Result<File> {
        self.file.try_clone()
    }

    pub(crate) fn remove(self) -> io::Result<()> {
        remove_open_child(self.parent, &self.name, &self.file, ChildKind::File)
    }
}

impl AnchoredDirectory {
    /// Opens an ordinary project directory without changing its permissions.
    pub(crate) fn open_existing_child_directory(&self, name: &OsStr) -> io::Result<Self> {
        validate_component(name)?;
        #[cfg(windows)]
        let handle = {
            const FILE_READ_ATTRIBUTES: u32 = 0x80;
            let file = super::super::windows::open_relative(
                &self.handle,
                name,
                FILE_READ_ATTRIBUTES,
                Some(true),
                3,
            )?;
            super::verify_child(&file, ChildKind::Directory)?;
            file
        };
        #[cfg(not(windows))]
        let handle = self.open_child(name, ChildKind::Directory)?;
        self.anchor_child(name, handle)
    }

    pub(crate) fn open_entry(&self, name: &OsStr) -> io::Result<AnchoredEntry<'_>> {
        self.open_entry_with_protection(name, false)
    }

    pub(crate) fn open_private_entry(&self, name: &OsStr) -> io::Result<AnchoredEntry<'_>> {
        self.open_entry_with_protection(name, true)
    }

    fn open_entry_with_protection(
        &self,
        name: &OsStr,
        private: bool,
    ) -> io::Result<AnchoredEntry<'_>> {
        validate_component(name)?;
        #[cfg(windows)]
        let file = {
            const FILE_READ_DATA: u32 = 0x1;
            const FILE_READ_ATTRIBUTES: u32 = 0x80;
            const DELETE: u32 = 0x0001_0000;
            const WRITE_DAC: u32 = 0x0004_0000;
            let file = super::super::windows::open_relative(
                &self.handle,
                name,
                FILE_READ_DATA
                    | FILE_READ_ATTRIBUTES
                    | DELETE
                    | if private { WRITE_DAC } else { 0 },
                None,
                3,
            )?;
            super::verify_child(&file, ChildKind::Any)?;
            if private {
                super::super::windows::protect_private_object(&file)?;
            }
            file
        };
        #[cfg(not(windows))]
        let file = self.open_child(name, ChildKind::Any)?;
        let kind = child_kind(&file)?;
        #[cfg(unix)]
        if private {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(
                if kind == ChildKind::Directory {
                    0o700
                } else {
                    0o600
                },
            ))?;
        }
        #[cfg(not(any(windows, unix)))]
        let _ = private;
        match kind {
            ChildKind::Directory => Ok(AnchoredEntry::Directory(AnchoredChildDirectory {
                parent: self,
                name: name.to_os_string(),
                directory: self.anchor_child(name, file)?,
            })),
            ChildKind::File => Ok(AnchoredEntry::File(AnchoredFile {
                parent: self,
                name: name.to_os_string(),
                file,
            })),
            ChildKind::Any => unreachable!(),
        }
    }

    fn anchor_child(&self, name: &OsStr, handle: File) -> io::Result<Self> {
        Ok(Self {
            handle,
            path: self.path.join(name),
            #[cfg(windows)]
            _ancestors: self
                ._ancestors
                .iter()
                .map(File::try_clone)
                .chain(std::iter::once(self.handle.try_clone()))
                .collect::<io::Result<Vec<_>>>()?,
        })
    }
}

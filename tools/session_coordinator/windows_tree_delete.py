from __future__ import annotations

import ctypes
import os
from dataclasses import dataclass
from pathlib import Path


_DELETE = 0x00010000
_FILE_READ_ATTRIBUTES = 0x00000080
_FILE_SHARE_READ = 0x00000001
_FILE_SHARE_WRITE = 0x00000002
_OPEN_EXISTING = 3
_FILE_FLAG_BACKUP_SEMANTICS = 0x02000000
_FILE_FLAG_OPEN_REPARSE_POINT = 0x00200000
_FILE_ATTRIBUTE_DIRECTORY = 0x00000010
_FILE_ATTRIBUTE_READONLY = 0x00000001
_FILE_ATTRIBUTE_REPARSE_POINT = 0x00000400
_FILE_ATTRIBUTE_TAG_INFO = 9
_FILE_ID_INFO = 18
_FILE_STANDARD_INFO = 1
_FILE_DISPOSITION_INFO_EX = 21
_FILE_DISPOSITION_FLAG_DELETE = 0x00000001
_FILE_DISPOSITION_FLAG_POSIX_SEMANTICS = 0x00000002
_FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE = 0x00000010


class _FileAttributeTagInfo(ctypes.Structure):
    _fields_ = (
        ("file_attributes", ctypes.c_uint32),
        ("reparse_tag", ctypes.c_uint32),
    )


class _FileIdInfo(ctypes.Structure):
    _fields_ = (
        ("volume_serial_number", ctypes.c_uint64),
        ("file_id", ctypes.c_ubyte * 16),
    )


class _FileStandardInfo(ctypes.Structure):
    _fields_ = (
        ("allocation_size", ctypes.c_int64),
        ("end_of_file", ctypes.c_int64),
        ("number_of_links", ctypes.c_uint32),
        ("delete_pending", ctypes.c_ubyte),
        ("directory", ctypes.c_ubyte),
    )


class _FileDispositionInfoEx(ctypes.Structure):
    _fields_ = (("flags", ctypes.c_uint32),)


@dataclass(slots=True)
class _EntryHandle:
    path: Path
    value: int

    def close(self) -> None:
        if not self.value:
            return
        kernel32 = _kernel32()
        if not kernel32.CloseHandle(self.value):
            error = ctypes.WinError(ctypes.get_last_error())
            self.value = 0
            raise error
        self.value = 0

    def __enter__(self) -> "_EntryHandle":
        return self

    def __exit__(self, _type, _value, _traceback) -> None:
        self.close()


def filesystem_identity(path: Path) -> str:
    if os.name != "nt":
        metadata = path.stat(follow_symlinks=False)
        return f"{metadata.st_dev:x}:{metadata.st_ino:x}"
    with _open_entry(path) as entry:
        _require_plain_entry(entry)
        return _identity(entry)


def remove_tree(path: Path, *, expected_identity: str | None = None) -> None:
    if os.name != "nt":
        raise OSError("handle-bound artifact deletion requires Windows")
    root = Path(os.path.abspath(path))
    with _open_entry(root) as entry:
        attributes = _require_plain_entry(entry)
        if not attributes & _FILE_ATTRIBUTE_DIRECTORY:
            raise NotADirectoryError(str(root))
        actual_identity = _identity(entry)
        if expected_identity is not None and actual_identity != expected_identity:
            raise OSError("cleanup candidate filesystem identity changed")
        _remove_open_directory(entry)
    try:
        root.lstat()
    except FileNotFoundError:
        return
    raise OSError("cleanup candidate was recreated before deletion completed")


def _remove_open_directory(entry: _EntryHandle) -> None:
    try:
        children = tuple(os.scandir(entry.path))
    except OSError:
        raise
    for child in children:
        child_path = entry.path / child.name
        with _open_entry(child_path) as child_entry:
            attributes = _require_plain_entry(child_entry)
            if attributes & _FILE_ATTRIBUTE_DIRECTORY:
                _remove_open_directory(child_entry)
            else:
                _mark_delete(child_entry, attributes)
    _mark_delete(entry, _attributes(entry))


def _open_entry(path: Path) -> _EntryHandle:
    if os.name != "nt":
        raise OSError("Windows entry handles are unavailable on this platform")
    kernel32 = _kernel32()
    handle = int(
        kernel32.CreateFileW(
            str(path),
            _DELETE | _FILE_READ_ATTRIBUTES,
            _FILE_SHARE_READ | _FILE_SHARE_WRITE,
            None,
            _OPEN_EXISTING,
            _FILE_FLAG_BACKUP_SEMANTICS | _FILE_FLAG_OPEN_REPARSE_POINT,
            None,
        )
    )
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    return _EntryHandle(Path(path), handle)


def _require_plain_entry(entry: _EntryHandle) -> int:
    attributes = _attributes(entry)
    if attributes & _FILE_ATTRIBUTE_REPARSE_POINT:
        raise OSError(f"cleanup refuses filesystem reparse point: {entry.path}")
    return attributes


def _attributes(entry: _EntryHandle) -> int:
    value = _FileAttributeTagInfo()
    _query(entry, _FILE_ATTRIBUTE_TAG_INFO, value)
    return int(value.file_attributes)


def _identity(entry: _EntryHandle) -> str:
    value = _FileIdInfo()
    _query(entry, _FILE_ID_INFO, value)
    return f"{int(value.volume_serial_number):016x}:{bytes(value.file_id).hex()}"


def _link_count(entry: _EntryHandle) -> int:
    value = _FileStandardInfo()
    _query(entry, _FILE_STANDARD_INFO, value)
    return int(value.number_of_links)


def _query(entry: _EntryHandle, information_class: int, value: ctypes.Structure) -> None:
    kernel32 = _kernel32()
    if not kernel32.GetFileInformationByHandleEx(
        entry.value,
        information_class,
        ctypes.byref(value),
        ctypes.sizeof(value),
    ):
        raise ctypes.WinError(ctypes.get_last_error())


def _mark_delete(entry: _EntryHandle, attributes: int) -> None:
    if attributes & _FILE_ATTRIBUTE_READONLY and _link_count(entry) != 1:
        raise PermissionError(
            f"cleanup refuses readonly multiply-linked file: {entry.path}"
        )
    disposition = _FileDispositionInfoEx(
        _FILE_DISPOSITION_FLAG_DELETE
        | _FILE_DISPOSITION_FLAG_POSIX_SEMANTICS
        | _FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE
    )
    kernel32 = _kernel32()
    if not kernel32.SetFileInformationByHandle(
        entry.value,
        _FILE_DISPOSITION_INFO_EX,
        ctypes.byref(disposition),
        ctypes.sizeof(disposition),
    ):
        raise ctypes.WinError(ctypes.get_last_error())


def _kernel32():
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.CreateFileW.argtypes = (
        ctypes.c_wchar_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.c_void_p,
    )
    kernel32.CreateFileW.restype = ctypes.c_void_p
    kernel32.GetFileInformationByHandleEx.argtypes = (
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_uint32,
    )
    kernel32.GetFileInformationByHandleEx.restype = ctypes.c_int
    kernel32.SetFileInformationByHandle.argtypes = (
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_uint32,
    )
    kernel32.SetFileInformationByHandle.restype = ctypes.c_int
    kernel32.CloseHandle.argtypes = (ctypes.c_void_p,)
    kernel32.CloseHandle.restype = ctypes.c_int
    return kernel32

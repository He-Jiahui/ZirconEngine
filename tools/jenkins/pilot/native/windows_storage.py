"""Handle-relative storage operations for Windows worker roots.

Every path below the configured root is opened one component at a time from a
pinned directory handle.  The directory handles deliberately omit
``FILE_SHARE_DELETE`` so a concurrent process cannot rename a checked parent
and replace it with a junction while an operation is in progress.  Leaf opens
use ``FILE_OPEN_REPARSE_POINT`` and reject reparse points before exposing data.

This module is Windows-only.  The portable ``WorkerStorage`` implementation
remains responsible for non-Windows workers.
"""

from __future__ import annotations

import ctypes
import hashlib
import ntpath
import os
import re
import secrets
import threading
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import PureWindowsPath
from typing import BinaryIO, Iterable, Iterator

from .portable_paths import normalize_portable_relative_path


_FILE_OPEN = 0x00000001
_FILE_CREATE = 0x00000002
_FILE_OPEN_IF = 0x00000003
_FILE_DIRECTORY_FILE = 0x00000001
_FILE_NON_DIRECTORY_FILE = 0x00000040
_FILE_SYNCHRONOUS_IO_NONALERT = 0x00000020
_FILE_OPEN_REPARSE_POINT = 0x00200000

_FILE_READ_DATA = 0x00000001
_FILE_WRITE_DATA = 0x00000002
_FILE_LIST_DIRECTORY = 0x00000001
_FILE_ADD_FILE = 0x00000002
_FILE_ADD_SUBDIRECTORY = 0x00000004
_FILE_TRAVERSE = 0x00000020
_FILE_DELETE_CHILD = 0x00000040
_FILE_READ_ATTRIBUTES = 0x00000080
_FILE_WRITE_ATTRIBUTES = 0x00000100
_DELETE = 0x00010000
_SYNCHRONIZE = 0x00100000

_FILE_SHARE_READ = 0x00000001
_FILE_SHARE_WRITE = 0x00000002

_DIRECTORY_READ_ACCESS = (
    _FILE_LIST_DIRECTORY | _FILE_READ_ATTRIBUTES | _FILE_TRAVERSE | _SYNCHRONIZE
)
# Leaf unlink and replacement operations request DELETE on the verified file or
# directory handle. Parent directory handles only need to create entries; asking
# for DELETE_CHILD prevents use under worker roots that grant per-entry DELETE.
_DIRECTORY_MUTATE_ACCESS = (
    _DIRECTORY_READ_ACCESS | _FILE_ADD_FILE | _FILE_ADD_SUBDIRECTORY
)

_FILE_ATTRIBUTE_DIRECTORY = 0x00000010
_FILE_ATTRIBUTE_REPARSE_POINT = 0x00000400
_FILE_ATTRIBUTE_NORMAL = 0x00000080
_FILE_ATTRIBUTE_TAG_INFO = 9
_FILE_STANDARD_INFO = 1
_FILE_BASIC_INFO = 0
_FILE_ID_INFO = 18
_VOLUME_NAME_NT = 0x00000002
_FILE_RENAME_INFO = 3
_FILE_DISPOSITION_INFO = 4
_FILE_DIRECTORY_INFORMATION = 1
_OBJ_CASE_INSENSITIVE = 0x00000040

_STATUS_SUCCESS = 0x00000000
_STATUS_NO_MORE_FILES = 0x80000006
_STATUS_OBJECT_NAME_NOT_FOUND = 0xC0000034
_STATUS_OBJECT_PATH_NOT_FOUND = 0xC000003A
_STATUS_OBJECT_NAME_COLLISION = 0xC0000035
_NTSTATUS_NOT_FOUND = {_STATUS_OBJECT_NAME_NOT_FOUND, _STATUS_OBJECT_PATH_NOT_FOUND}
_WINDOWS_EPOCH_TICKS = 116444736000000000
_SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
_NT_DEVICE_PATH_RE = re.compile(r"^\\Device\\HarddiskVolume[0-9]+(?:\\[^\\]+)*$", re.IGNORECASE)


class WindowsStorageError(OSError):
    """A handle-relative Windows storage operation failed."""


class WindowsPathError(ValueError):
    """A path is not a safe relative path below the configured root."""


class WindowsIdentityMismatch(WindowsStorageError):
    """A pinned file identity changed or did not match the expected identity."""


@dataclass(frozen=True, slots=True)
class WindowsFileIdentity:
    """Stable Windows volume and 128-bit file identity plus file metadata."""

    device: int
    file_id: bytes
    size: int
    mtime_ns: int
    link_count: int
    creation_time: int = 0

    @property
    def inode(self) -> int:
        """Expose a 64-bit compatibility value for existing ``FileIdentity``."""
        return int.from_bytes(self.file_id[-8:], "little", signed=False)

    def same_object(self, other: object) -> bool:
        return (
            isinstance(other, WindowsFileIdentity)
            and self.device == other.device
            and self.file_id == other.file_id
        )

    def same_file_state(self, other: object) -> bool:
        return (
            self.same_object(other)
            and isinstance(other, WindowsFileIdentity)
            and self.size == other.size
            and self.mtime_ns == other.mtime_ns
            and self.link_count == other.link_count
        )


def is_available() -> bool:
    """Return whether the native Windows handle APIs are available."""
    return os.name == "nt"


class _UnicodeString(ctypes.Structure):
    _fields_ = [
        ("Length", ctypes.c_ushort),
        ("MaximumLength", ctypes.c_ushort),
        ("Buffer", ctypes.c_void_p),
    ]


class _ObjectAttributes(ctypes.Structure):
    _fields_ = [
        ("Length", ctypes.c_ulong),
        ("RootDirectory", ctypes.c_void_p),
        ("ObjectName", ctypes.POINTER(_UnicodeString)),
        ("Attributes", ctypes.c_ulong),
        ("SecurityDescriptor", ctypes.c_void_p),
        ("SecurityQualityOfService", ctypes.c_void_p),
    ]


class _IoStatusBlock(ctypes.Structure):
    _fields_ = [("Status", ctypes.c_long), ("Information", ctypes.c_size_t)]


class _FileAttributeTagInfo(ctypes.Structure):
    _fields_ = [("FileAttributes", ctypes.c_ulong), ("ReparseTag", ctypes.c_ulong)]


class _FileStandardInfo(ctypes.Structure):
    _fields_ = [
        ("AllocationSize", ctypes.c_longlong),
        ("EndOfFile", ctypes.c_longlong),
        ("NumberOfLinks", ctypes.c_ulong),
        ("DeletePending", ctypes.c_ubyte),
        ("Directory", ctypes.c_ubyte),
    ]


class _FileBasicInfo(ctypes.Structure):
    _fields_ = [
        ("CreationTime", ctypes.c_longlong),
        ("LastAccessTime", ctypes.c_longlong),
        ("LastWriteTime", ctypes.c_longlong),
        ("ChangeTime", ctypes.c_longlong),
        ("FileAttributes", ctypes.c_ulong),
    ]


class _FileId128(ctypes.Structure):
    _fields_ = [("Identifier", ctypes.c_ubyte * 16)]


class _FileIdInfo(ctypes.Structure):
    _fields_ = [("VolumeSerialNumber", ctypes.c_ulonglong), ("FileId", _FileId128)]


class _FileDispositionInfo(ctypes.Structure):
    _fields_ = [("DeleteFile", ctypes.c_ubyte)]


class _FileRenameInfo(ctypes.Structure):
    _fields_ = [
        ("ReplaceIfExists", ctypes.c_ubyte),
        ("RootDirectory", ctypes.c_void_p),
        ("FileNameLength", ctypes.c_ulong),
        ("FileName", ctypes.c_wchar * 1),
    ]


class _FileDirectoryInformation(ctypes.Structure):
    _fields_ = [
        ("NextEntryOffset", ctypes.c_ulong),
        ("FileIndex", ctypes.c_ulong),
        ("CreationTime", ctypes.c_longlong),
        ("LastAccessTime", ctypes.c_longlong),
        ("LastWriteTime", ctypes.c_longlong),
        ("ChangeTime", ctypes.c_longlong),
        ("EndOfFile", ctypes.c_longlong),
        ("AllocationSize", ctypes.c_longlong),
        ("FileAttributes", ctypes.c_ulong),
        ("FileNameLength", ctypes.c_ulong),
        ("FileName", ctypes.c_wchar * 1),
    ]


_api_cache: tuple[object, object, object] | None = None
_api_lock = threading.Lock()


def _apis():
    global _api_cache
    if not is_available():
        raise NotImplementedError("WindowsStorage is available only on Windows")
    if _api_cache is not None:
        return _api_cache
    with _api_lock:
        if _api_cache is not None:
            return _api_cache
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        ntdll = ctypes.WinDLL("ntdll")
        kernel32.CreateFileW.argtypes = [
            ctypes.c_wchar_p,
            ctypes.c_ulong,
            ctypes.c_ulong,
            ctypes.c_void_p,
            ctypes.c_ulong,
            ctypes.c_ulong,
            ctypes.c_void_p,
        ]
        kernel32.CreateFileW.restype = ctypes.c_void_p
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle.restype = ctypes.c_int
        kernel32.GetFileInformationByHandleEx.argtypes = [
            ctypes.c_void_p,
            ctypes.c_int,
            ctypes.c_void_p,
            ctypes.c_ulong,
        ]
        kernel32.GetFileInformationByHandleEx.restype = ctypes.c_int
        kernel32.SetFileInformationByHandle.argtypes = [
            ctypes.c_void_p,
            ctypes.c_int,
            ctypes.c_void_p,
            ctypes.c_ulong,
        ]
        kernel32.SetFileInformationByHandle.restype = ctypes.c_int
        kernel32.GetFinalPathNameByHandleW.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_wchar),
            ctypes.c_ulong,
            ctypes.c_ulong,
        ]
        kernel32.GetFinalPathNameByHandleW.restype = ctypes.c_ulong
        kernel32.FlushFileBuffers.argtypes = [ctypes.c_void_p]
        kernel32.FlushFileBuffers.restype = ctypes.c_int
        kernel32.ReadFile.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_ulong,
            ctypes.POINTER(ctypes.c_ulong),
            ctypes.c_void_p,
        ]
        kernel32.ReadFile.restype = ctypes.c_int
        kernel32.WriteFile.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_ulong,
            ctypes.POINTER(ctypes.c_ulong),
            ctypes.c_void_p,
        ]
        kernel32.WriteFile.restype = ctypes.c_int
        ntdll.NtCreateFile.argtypes = [
            ctypes.POINTER(ctypes.c_void_p),
            ctypes.c_ulong,
            ctypes.POINTER(_ObjectAttributes),
            ctypes.POINTER(_IoStatusBlock),
            ctypes.c_void_p,
            ctypes.c_ulong,
            ctypes.c_ulong,
            ctypes.c_ulong,
            ctypes.c_ulong,
            ctypes.c_void_p,
            ctypes.c_ulong,
        ]
        ntdll.NtCreateFile.restype = ctypes.c_long
        ntdll.NtSetInformationFile.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(_IoStatusBlock),
            ctypes.c_void_p,
            ctypes.c_ulong,
            ctypes.c_int,
        ]
        ntdll.NtSetInformationFile.restype = ctypes.c_long
        ntdll.NtQueryDirectoryFile.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.POINTER(_IoStatusBlock),
            ctypes.c_void_p,
            ctypes.c_ulong,
            ctypes.c_int,
            ctypes.c_ubyte,
            ctypes.POINTER(_UnicodeString),
            ctypes.c_ubyte,
        ]
        ntdll.NtQueryDirectoryFile.restype = ctypes.c_long
        ntdll.RtlNtStatusToDosError.argtypes = [ctypes.c_long]
        ntdll.RtlNtStatusToDosError.restype = ctypes.c_ulong
        _api_cache = (kernel32, ntdll, ctypes.WinDLL("msvcrt"))
        return _api_cache


def _handle_value(handle: int | ctypes.c_void_p) -> int:
    return int(handle.value if isinstance(handle, ctypes.c_void_p) else handle)


def _close_handle(handle: int | ctypes.c_void_p | None) -> None:
    if handle is None:
        return
    value = _handle_value(handle)
    if value and value != ctypes.c_void_p(-1).value:
        kernel32, _ntdll, _msvcrt = _apis()
        if not kernel32.CloseHandle(ctypes.c_void_p(value)):
            _raise_last_error("CloseHandle")


def _raise_last_error(operation: str) -> None:
    code = ctypes.get_last_error()
    raise WindowsStorageError(code, f"{operation} failed: {ctypes.FormatError(code).strip()}")


def _windows_error_code(error: OSError) -> int | None:
    return getattr(error, "winerror", None) or error.errno


def _raise_nt_status(status: int, operation: str) -> None:
    unsigned = status & 0xFFFFFFFF
    if unsigned == _STATUS_SUCCESS:
        return
    _kernel32, ntdll, _msvcrt = _apis()
    code = int(ntdll.RtlNtStatusToDosError(ctypes.c_long(status)))
    message = ctypes.FormatError(code).strip() if code else f"NTSTATUS 0x{unsigned:08x}"
    if unsigned in _NTSTATUS_NOT_FOUND:
        raise WindowsStorageError(code or 2, f"{operation} failed: {message}")
    raise WindowsStorageError(code, f"{operation} failed: {message}")


def _validate_component(value: str) -> str:
    if (
        not value
        or value in {".", ".."}
        or "/" in value
        or "\\" in value
        or ":" in value
        or "\x00" in value
        or value.endswith((".", " "))
    ):
        raise WindowsPathError("worker path component is invalid")
    return value


def _relative_parts(relative: os.PathLike[str] | str) -> tuple[str, ...]:
    raw = os.fspath(relative)
    try:
        normalized = normalize_portable_relative_path(
            raw,
            code="worker_path_invalid",
            message="Worker paths must be safe portable relative paths",
        )
    except Exception as error:
        raise WindowsPathError("worker path must be a safe portable relative path") from error
    parts = tuple(_validate_component(part) for part in normalized.split("/"))
    if not parts:
        raise WindowsPathError("worker path must not be empty")
    return parts


def _open_relative_handle(
    parent: int,
    name: str,
    *,
    access: int,
    share: int,
    disposition: int = _FILE_OPEN,
    options: int = _FILE_NON_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT | _FILE_SYNCHRONOUS_IO_NONALERT,
    attributes: int = _FILE_ATTRIBUTE_NORMAL,
) -> int:
    _kernel32, ntdll, _msvcrt = _apis()
    _validate_component(name)
    encoded = name.encode("utf-16-le")
    if len(encoded) > 0xFFFC:
        raise WindowsPathError("worker path component is too long")
    buffer = ctypes.create_unicode_buffer(name)
    unicode_name = _UnicodeString(
        len(encoded),
        len(encoded) + ctypes.sizeof(ctypes.c_wchar),
        ctypes.cast(buffer, ctypes.c_void_p),
    )
    object_attributes = _ObjectAttributes(
        ctypes.sizeof(_ObjectAttributes),
        ctypes.c_void_p(parent),
        ctypes.pointer(unicode_name),
        _OBJ_CASE_INSENSITIVE,
        None,
        None,
    )
    io_status = _IoStatusBlock()
    file_handle = ctypes.c_void_p()
    status = ntdll.NtCreateFile(
        ctypes.byref(file_handle),
        access,
        ctypes.byref(object_attributes),
        ctypes.byref(io_status),
        None,
        attributes,
        share,
        disposition,
        options,
        None,
        0,
    )
    _raise_nt_status(status, f"open worker path component {name!r}")
    if not file_handle.value:
        raise WindowsStorageError(1, "NtCreateFile returned an empty file handle")
    return int(file_handle.value)


def _open_volume_root(drive: str) -> int:
    kernel32, _ntdll, _msvcrt = _apis()
    root = f"{drive}\\"
    # The drive root is only the trusted starting handle.  Asking it for
    # mutation or listing rights makes otherwise writable worker roots
    # unusable when the drive ACL intentionally restricts its root directory.
    access = _FILE_READ_ATTRIBUTES | _FILE_TRAVERSE | _SYNCHRONIZE
    handle = kernel32.CreateFileW(
        root,
        access,
        _FILE_SHARE_READ | _FILE_SHARE_WRITE,
        None,
        3,  # OPEN_EXISTING
        0x02000000 | _FILE_OPEN_REPARSE_POINT,  # BACKUP_SEMANTICS | OPEN_REPARSE_POINT
        None,
    )
    if not handle or handle == ctypes.c_void_p(-1).value:
        _raise_last_error("open worker volume root")
    return int(handle)


def _file_info(handle: int, information_class: int, structure):
    kernel32, _ntdll, _msvcrt = _apis()
    value = structure()
    if not kernel32.GetFileInformationByHandleEx(
        ctypes.c_void_p(handle), information_class, ctypes.byref(value), ctypes.sizeof(value)
    ):
        _raise_last_error("GetFileInformationByHandleEx")
    return value


def _attributes(handle: int) -> _FileAttributeTagInfo:
    return _file_info(handle, _FILE_ATTRIBUTE_TAG_INFO, _FileAttributeTagInfo)


def _final_nt_path(handle: int) -> str:
    """Return the open handle's canonical NT device path, never a DOS alias."""
    kernel32, _ntdll, _msvcrt = _apis()
    query = kernel32.GetFinalPathNameByHandleW
    buffer = ctypes.create_unicode_buffer(32768)
    length = query(
        ctypes.c_void_p(_handle_value(handle)),
        buffer,
        len(buffer),
        _VOLUME_NAME_NT,
    )
    if not length:
        _raise_last_error("GetFinalPathNameByHandleW")
    if length >= len(buffer):
        buffer = ctypes.create_unicode_buffer(length + 1)
        length = query(
            ctypes.c_void_p(_handle_value(handle)),
            buffer,
            len(buffer),
            _VOLUME_NAME_NT,
        )
        if not length or length >= len(buffer):
            _raise_last_error("GetFinalPathNameByHandleW")
    result = ntpath.normpath(buffer.value.replace("/", "\\"))
    if not _NT_DEVICE_PATH_RE.fullmatch(result):
        raise WindowsPathError("pinned worker root has no local physical NT device path")
    return result


def identity_from_handle(handle: int | ctypes.c_void_p) -> WindowsFileIdentity:
    """Read stable volume/file identity directly from an already-open handle."""
    value = _handle_value(handle)
    attributes = _attributes(value)
    if attributes.FileAttributes & _FILE_ATTRIBUTE_REPARSE_POINT:
        raise WindowsPathError("worker path is a reparse point")
    standard = _file_info(value, _FILE_STANDARD_INFO, _FileStandardInfo)
    basic = _file_info(value, _FILE_BASIC_INFO, _FileBasicInfo)
    file_id = _file_info(value, _FILE_ID_INFO, _FileIdInfo)
    raw_id = bytes(file_id.FileId.Identifier)
    write_ticks = int(basic.LastWriteTime)
    mtime_ns = (write_ticks - _WINDOWS_EPOCH_TICKS) * 100
    return WindowsFileIdentity(
        int(file_id.VolumeSerialNumber),
        raw_id,
        max(0, int(standard.EndOfFile)),
        mtime_ns,
        int(standard.NumberOfLinks),
        int(basic.CreationTime),
    )


def identity_from_stream(stream: BinaryIO) -> WindowsFileIdentity:
    """Return the native identity behind an ``open_relative`` file object."""
    if not is_available():
        raise NotImplementedError("Windows file identities are available only on Windows")
    import msvcrt

    return identity_from_handle(msvcrt.get_osfhandle(stream.fileno()))


def _ensure_directory_handle(handle: int) -> WindowsFileIdentity:
    attributes = _attributes(handle)
    if attributes.FileAttributes & _FILE_ATTRIBUTE_REPARSE_POINT:
        raise WindowsPathError("worker directory is a reparse point")
    if not attributes.FileAttributes & _FILE_ATTRIBUTE_DIRECTORY:
        raise WindowsPathError("worker path component is not a directory")
    return identity_from_handle(handle)


def _ensure_regular_file_handle(handle: int) -> WindowsFileIdentity:
    attributes = _attributes(handle)
    if attributes.FileAttributes & _FILE_ATTRIBUTE_REPARSE_POINT:
        raise WindowsPathError("worker file is a reparse point")
    if attributes.FileAttributes & _FILE_ATTRIBUTE_DIRECTORY:
        raise WindowsPathError("worker path is not a regular file")
    identity = identity_from_handle(handle)
    if identity.link_count > 1:
        raise WindowsPathError("worker file has a hard-link alias")
    return identity




def _rename_handle(handle: int, parent: int, new_name: str, *, replace: bool) -> None:
    _validate_component(new_name)
    name_bytes = new_name.encode("utf-16-le")
    name_offset = _FileRenameInfo.FileName.offset
    # Win32 callers allocate sizeof(FILE_RENAME_INFO) plus the filename bytes;
    # the structure includes its one-WCHAR placeholder in addition to the
    # offset at which the variable-length name begins.
    raw = ctypes.create_string_buffer(ctypes.sizeof(_FileRenameInfo) + len(name_bytes))
    info = ctypes.cast(raw, ctypes.POINTER(_FileRenameInfo)).contents
    info.ReplaceIfExists = int(replace)
    info.RootDirectory = ctypes.c_void_p(parent)
    info.FileNameLength = len(name_bytes)
    ctypes.memmove(ctypes.addressof(raw) + name_offset, name_bytes, len(name_bytes))
    kernel32, _ntdll, _msvcrt = _apis()
    if not kernel32.SetFileInformationByHandle(
        ctypes.c_void_p(handle), _FILE_RENAME_INFO, ctypes.cast(raw, ctypes.c_void_p), ctypes.sizeof(raw)
    ):
        win32_code = ctypes.get_last_error()
        _kernel32, ntdll, _msvcrt = _apis()
        io_status = _IoStatusBlock()
        status = ntdll.NtSetInformationFile(
            ctypes.c_void_p(handle),
            ctypes.byref(io_status),
            ctypes.cast(raw, ctypes.c_void_p),
            ctypes.sizeof(raw),
            10,  # FileRenameInformation; preserves RootDirectory-relative semantics.
        )
        if status & 0xFFFFFFFF:
            nt_code = int(ntdll.RtlNtStatusToDosError(ctypes.c_long(status)))
            raise WindowsStorageError(
                nt_code,
                "handle-relative rename failed via Win32 and native APIs: "
                f"WinError {win32_code}, NTSTATUS 0x{status & 0xFFFFFFFF:08x}; "
                f"stored_parent={int(info.RootDirectory or 0):#x}, "
                f"name_bytes={len(name_bytes)}, buffer_bytes={ctypes.sizeof(raw)}",
            )


def _mark_delete(handle: int) -> None:
    info = _FileDispositionInfo(1)
    kernel32, _ntdll, _msvcrt = _apis()
    if not kernel32.SetFileInformationByHandle(
        ctypes.c_void_p(handle), _FILE_DISPOSITION_INFO, ctypes.byref(info), ctypes.sizeof(info)
    ):
        _raise_last_error("SetFileInformationByHandle(FileDispositionInfo)")


def _read_handle(handle: int, maximum: int) -> bytes:
    kernel32, _ntdll, _msvcrt = _apis()
    result = bytearray()
    buffer = ctypes.create_string_buffer(min(maximum, 1024 * 1024) or 1)
    while len(result) < maximum:
        wanted = min(len(buffer), maximum - len(result))
        count = ctypes.c_ulong()
        if not kernel32.ReadFile(
            ctypes.c_void_p(handle), buffer, wanted, ctypes.byref(count), None
        ):
            _raise_last_error("ReadFile")
        if count.value == 0:
            break
        result.extend(buffer.raw[: count.value])
    return bytes(result)


def _write_handle(handle: int, data: memoryview) -> None:
    kernel32, _ntdll, _msvcrt = _apis()
    cursor = 0
    while cursor < len(data):
        length = min(len(data) - cursor, 1024 * 1024)
        payload = (ctypes.c_ubyte * length).from_buffer_copy(data[cursor : cursor + length])
        written = ctypes.c_ulong()
        if not kernel32.WriteFile(
            ctypes.c_void_p(handle), payload, length, ctypes.byref(written), None
        ):
            _raise_last_error("WriteFile")
        if written.value <= 0:
            raise WindowsStorageError(29, "WriteFile made no progress")
        cursor += int(written.value)


def _directory_entries(handle: int) -> list[tuple[str, int]]:
    _kernel32, ntdll, _msvcrt = _apis()
    records: list[tuple[str, int]] = []
    buffer_size = 64 * 1024
    buffer = ctypes.create_string_buffer(buffer_size)
    restart = True
    while True:
        io_status = _IoStatusBlock()
        status = ntdll.NtQueryDirectoryFile(
            ctypes.c_void_p(handle),
            None,
            None,
            None,
            ctypes.byref(io_status),
            buffer,
            buffer_size,
            _FILE_DIRECTORY_INFORMATION,
            0,
            None,
            int(restart),
        )
        restart = False
        unsigned = status & 0xFFFFFFFF
        if unsigned == _STATUS_NO_MORE_FILES:
            break
        if unsigned != _STATUS_SUCCESS:
            _raise_nt_status(status, "NtQueryDirectoryFile")
        offset = 0
        while True:
            entry = ctypes.cast(
                ctypes.addressof(buffer) + offset,
                ctypes.POINTER(_FileDirectoryInformation),
            ).contents
            byte_length = int(entry.FileNameLength)
            if byte_length % 2:
                raise WindowsStorageError(13, "directory entry has an invalid UTF-16 name")
            name_offset = offset + _FileDirectoryInformation.FileName.offset
            name_bytes = ctypes.string_at(ctypes.addressof(buffer) + name_offset, byte_length)
            try:
                name = name_bytes.decode("utf-16-le", errors="strict")
            except UnicodeDecodeError as error:
                raise WindowsStorageError(13, "directory entry name is invalid UTF-16") from error
            if name not in {".", ".."}:
                _validate_component(name)
                records.append((name, int(entry.FileAttributes)))
            next_offset = int(entry.NextEntryOffset)
            if not next_offset:
                break
            if next_offset < _FileDirectoryInformation.FileName.offset or offset + next_offset >= buffer_size:
                raise WindowsStorageError(13, "directory enumeration returned an invalid record offset")
            offset += next_offset
    return records


class WindowsStorage:
    """A pinned Windows directory hierarchy with handle-relative file I/O."""

    def __init__(self, root: os.PathLike[str] | str, *, expected_device: str,
                 create: bool = False) -> None:
        if not is_available():
            raise NotImplementedError("WindowsStorage is available only on Windows")
        raw = os.fspath(root)
        upper = raw.replace("/", "\\").upper()
        if raw.startswith(("\\\\", "//")) or upper.startswith(("\\\\?\\", "\\\\.\\")):
            raise WindowsPathError("UNC and device worker roots are not allowed")
        drive, tail = ntpath.splitdrive(raw)
        if drive.upper() not in {"D:", "E:", "F:"} or not tail.startswith(("\\", "/")):
            raise WindowsPathError("worker root must use an approved absolute D/E/F drive")
        if any(part == ".." for part in PureWindowsPath(raw).parts):
            raise WindowsPathError("worker root cannot contain parent traversal")
        components = [part for part in tail.replace("/", "\\").split("\\") if part]
        if not components or components[0].casefold() != "cargo-targets":
            raise WindowsPathError("worker root must be below a drive-root cargo-targets directory")
        for component in components:
            _validate_component(component)

        self.root_path = ntpath.normpath(drive + "\\" + "\\".join(components))
        self._lock = threading.RLock()
        self._closed = False
        self._ancestor_handles: list[tuple[str, int, WindowsFileIdentity]] = []
        volume_handle: int | None = None
        parent = volume_handle
        current_path = drive + "\\"
        try:
            volume_handle = _open_volume_root(drive)
            volume_identity = _ensure_directory_handle(volume_handle)
            self._ancestor_handles.append((drive + "\\", volume_handle, volume_identity))
            # Bind the drive handle before creating any component. A remapping
            # between preflight and opening the drive must not write elsewhere.
            observed = ntpath.normcase(ntpath.normpath(_final_nt_path(volume_handle)))
            expected = ntpath.normcase(ntpath.normpath(expected_device + "\\"))
            if observed != expected:
                raise WindowsIdentityMismatch("pinned volume differs from its approved physical mapping")
            parent = volume_handle
            for index, component in enumerate(components):
                access = (_DIRECTORY_MUTATE_ACCESS if create or index == len(components) - 1
                          else _DIRECTORY_READ_ACCESS)
                options = (_FILE_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT
                           | _FILE_SYNCHRONOUS_IO_NONALERT)
                try:
                    handle = _open_relative_handle(
                        parent, component, access=access,
                        share=_FILE_SHARE_READ | _FILE_SHARE_WRITE, options=options,
                    )
                except WindowsStorageError as error:
                    if not create or _windows_error_code(error) not in {2, 3}:
                        raise
                    try:
                        handle = _open_relative_handle(
                            parent, component, access=access,
                            share=_FILE_SHARE_READ | _FILE_SHARE_WRITE, options=options,
                            disposition=_FILE_CREATE, attributes=_FILE_ATTRIBUTE_DIRECTORY,
                        )
                    except WindowsStorageError as create_error:
                        if _windows_error_code(create_error) not in {80, 183}:
                            raise
                        handle = _open_relative_handle(
                            parent, component, access=access,
                            share=_FILE_SHARE_READ | _FILE_SHARE_WRITE, options=options,
                        )
                current_path = ntpath.join(current_path, component)
                try:
                    identity = _ensure_directory_handle(handle)
                except BaseException:
                    _close_handle(handle)
                    raise
                self._ancestor_handles.append((current_path, handle, identity))
                parent = handle
        except BaseException:
            if volume_handle is not None and not self._ancestor_handles:
                _close_handle(volume_handle)
            self.close()
            raise
        self._root_handle = parent
        self._root_identity = self._ancestor_handles[-1][2]

    @property
    def root_identity(self) -> WindowsFileIdentity:
        return self._root_identity

    @property
    def root_device_path(self) -> str:
        """Return the pinned root's full physical NT path from its handle."""
        with self._lock:
            self._assert_open()
            return _final_nt_path(self._root_handle)

    def verify_physical_path(self, expected_nt_path: str) -> bool:
        """Verify that the pinned root resolves to the expected NT device path.

        ``expected_nt_path`` should combine a previously checked
        ``QueryDosDeviceW`` target with the original root components beneath
        that drive. Comparing the complete path also rejects SUBST aliases to
        another directory on the same physical volume.
        """
        if not isinstance(expected_nt_path, str) or "\x00" in expected_nt_path:
            raise WindowsPathError("expected physical worker path is invalid")
        expected = ntpath.normpath(expected_nt_path.replace("/", "\\"))
        if not _NT_DEVICE_PATH_RE.fullmatch(expected):
            raise WindowsPathError("expected worker path must use a local physical NT device path")
        observed = self.root_device_path
        if ntpath.normcase(observed) != ntpath.normcase(expected):
            raise WindowsIdentityMismatch("pinned worker root differs from its approved physical path")
        return True

    def verify_root(self) -> bool:
        with self._lock:
            self._assert_open()
            for _path, handle, expected in self._ancestor_handles:
                observed = _ensure_directory_handle(handle)
                if not observed.same_object(expected):
                    raise WindowsIdentityMismatch("a pinned worker root directory identity changed")
            return True

    def close(self) -> None:
        with self._lock:
            if self._closed:
                return
            self._closed = True
            handles, self._ancestor_handles = self._ancestor_handles, []
        first_error: BaseException | None = None
        for _path, handle, _identity in reversed(handles):
            try:
                _close_handle(handle)
            except BaseException as error:
                first_error = first_error or error
        if first_error is not None:
            raise first_error

    def _assert_open(self) -> None:
        if self._closed:
            raise WindowsStorageError(6, "WindowsStorage has been closed")

    @contextmanager
    def _pinned_parent(self, parts: tuple[str, ...]) -> Iterator[int]:
        self._assert_open()
        parent = self._root_handle
        owned: list[int] = []
        try:
            for component in parts[:-1]:
                child = _open_relative_handle(
                    parent,
                    component,
                    access=_DIRECTORY_READ_ACCESS,
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    options=(
                        _FILE_DIRECTORY_FILE
                        | _FILE_OPEN_REPARSE_POINT
                        | _FILE_SYNCHRONOUS_IO_NONALERT
                    ),
                )
                try:
                    _ensure_directory_handle(child)
                except BaseException:
                    _close_handle(child)
                    raise
                owned.append(child)
                parent = child
            yield parent
        finally:
            for handle in reversed(owned):
                _close_handle(handle)

    @contextmanager
    def _mutable_parent(self, parts: tuple[str, ...]) -> Iterator[int]:
        """Open a mutable destination parent through the pinned root handle."""
        self._assert_open()
        parent = self._root_handle
        owned: list[int] = []
        parent_parts = parts[:-1]
        try:
            for index, component in enumerate(parent_parts):
                try:
                    handle = _open_relative_handle(
                        parent,
                        component,
                        access=_DIRECTORY_MUTATE_ACCESS,
                        share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                        options=(
                            _FILE_DIRECTORY_FILE
                            | _FILE_OPEN_REPARSE_POINT
                            | _FILE_SYNCHRONOUS_IO_NONALERT
                        ),
                    )
                except WindowsStorageError as error:
                    if _windows_error_code(error) not in {2, 3}:
                        raise
                    try:
                        handle = _open_relative_handle(
                            parent,
                            component,
                            access=_DIRECTORY_MUTATE_ACCESS,
                            share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                            disposition=_FILE_CREATE,
                            options=(
                                _FILE_DIRECTORY_FILE
                                | _FILE_OPEN_REPARSE_POINT
                                | _FILE_SYNCHRONOUS_IO_NONALERT
                            ),
                            attributes=_FILE_ATTRIBUTE_DIRECTORY,
                        )
                    except WindowsStorageError as create_error:
                        if _windows_error_code(create_error) not in {80, 183}:
                            raise
                        handle = _open_relative_handle(
                            parent,
                            component,
                            access=_DIRECTORY_MUTATE_ACCESS,
                            share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                            options=(
                                _FILE_DIRECTORY_FILE
                                | _FILE_OPEN_REPARSE_POINT
                                | _FILE_SYNCHRONOUS_IO_NONALERT
                            ),
                        )
                try:
                    _ensure_directory_handle(handle)
                except BaseException:
                    _close_handle(handle)
                    raise
                owned.append(handle)
                parent = handle
            yield parent
        finally:
            for handle in reversed(owned):
                _close_handle(handle)

    def _open_leaf(self, parent: int, name: str, mode: str = "rb", *, create: bool = False) -> int:
        if create:
            access = _FILE_READ_DATA | _FILE_WRITE_DATA | _FILE_READ_ATTRIBUTES | _DELETE | _SYNCHRONIZE
            share = _FILE_SHARE_READ
            disposition = _FILE_CREATE
        elif mode == "rb":
            access = _FILE_READ_DATA | _FILE_READ_ATTRIBUTES | _SYNCHRONIZE
            share = _FILE_SHARE_READ
            disposition = _FILE_OPEN
        elif mode == "r+b":
            access = _FILE_READ_DATA | _FILE_WRITE_DATA | _FILE_READ_ATTRIBUTES | _FILE_WRITE_ATTRIBUTES | _SYNCHRONIZE
            share = _FILE_SHARE_READ
            disposition = _FILE_OPEN
        elif mode == "delete":
            access = _DELETE | _FILE_READ_ATTRIBUTES | _SYNCHRONIZE
            share = _FILE_SHARE_READ
            disposition = _FILE_OPEN
        else:
            raise ValueError("open_relative supports only 'rb' and 'r+b' modes")
        return _open_relative_handle(
            parent,
            name,
            access=access,
            share=share,
            disposition=disposition,
            options=_FILE_NON_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT | _FILE_SYNCHRONOUS_IO_NONALERT,
            attributes=_FILE_ATTRIBUTE_NORMAL,
        )

    @contextmanager
    def _file_stream(self, handle: int, mode: str) -> Iterator[BinaryIO]:
        import msvcrt

        flags = os.O_BINARY | (os.O_RDWR if "w" in mode or "+" in mode else os.O_RDONLY)
        try:
            fd = msvcrt.open_osfhandle(handle, flags)
        except BaseException:
            _close_handle(handle)
            raise
        try:
            stream = os.fdopen(fd, mode, buffering=0)
        except BaseException:
            os.close(fd)
            raise
        try:
            yield stream
        finally:
            stream.close()

    @contextmanager
    def open_relative(self, relative: os.PathLike[str] | str, mode: str = "rb") -> Iterator[BinaryIO]:
        """Open one regular file from pinned parent handles and yield its FD stream."""
        parts = _relative_parts(relative)
        if mode not in {"rb", "r+b"}:
            raise ValueError("open_relative supports only 'rb' and 'r+b' modes")
        with self._lock:
            self.verify_root()
            with self._pinned_parent(parts) as parent:
                handle = self._open_leaf(parent, parts[-1], mode)
                try:
                    _ensure_regular_file_handle(handle)
                except BaseException:
                    _close_handle(handle)
                    raise
                with self._file_stream(handle, mode) as stream:
                    yield stream

    @contextmanager
    def open_lock_file(self, relative: os.PathLike[str] | str) -> Iterator[BinaryIO]:
        """Open/create a lock file without truncating through retained parent pins.

        Share read and write so each contender can attempt a CRT byte lock on
        the same file. Omitting share-delete prevents renaming/replacing the
        lock entry until all lock streams close. The yielded unbuffered FD
        owns the exact checked handle and can be used with msvcrt.locking.
        """
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            with self._mutable_parent(parts) as parent:
                handle = _open_relative_handle(
                    parent, parts[-1],
                    access=(_FILE_READ_DATA | _FILE_WRITE_DATA | _FILE_READ_ATTRIBUTES | _SYNCHRONIZE),
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    disposition=_FILE_OPEN_IF,
                    options=(_FILE_NON_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT
                             | _FILE_SYNCHRONOUS_IO_NONALERT),
                    attributes=_FILE_ATTRIBUTE_NORMAL,
                )
                try:
                    _ensure_regular_file_handle(handle)
                except BaseException:
                    _close_handle(handle)
                    raise
                with self._file_stream(handle, "r+b") as stream:
                    yield stream

    @contextmanager
    def pin_readonly_directory(self, relative: os.PathLike[str] | str) -> Iterator[WindowsFileIdentity]:
        """Retain directory pins that deny mutable native opens and renames.

        Windows directory sharing does not prevent direct absolute-path child
        creation by the same user. Namespace ownership remains a private-root
        contract; these pins protect the existing checked tree identities.
        """
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            handles = []
            parent = self._root_handle
            try:
                for index, name in enumerate(parts):
                    handle = _open_relative_handle(
                        parent, name, access=_DIRECTORY_READ_ACCESS,
                        share=(_FILE_SHARE_READ if index == len(parts)-1 else _FILE_SHARE_READ | _FILE_SHARE_WRITE),
                        options=(_FILE_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT | _FILE_SYNCHRONOUS_IO_NONALERT),
                    )
                    try:
                        identity = _ensure_directory_handle(handle)
                    except BaseException:
                        _close_handle(handle)
                        raise
                    handles.append(handle)
                    parent = handle
                yield identity
            finally:
                for handle in reversed(handles):
                    _close_handle(handle)

    def file_identity(self, relative: os.PathLike[str] | str) -> WindowsFileIdentity:
        with self.open_relative(relative, "rb") as stream:
            return identity_from_stream(stream)

    def exists(self, relative: os.PathLike[str] | str) -> bool:
        parts = _relative_parts(relative)
        try:
            with self.open_relative("/".join(parts), "rb"):
                return True
        except WindowsStorageError as error:
            if _windows_error_code(error) in {2, 3}:
                return False
            raise

    def read_bytes(
        self,
        relative: os.PathLike[str] | str,
        *,
        expected_sha256: str | None = None,
        expected_identity: WindowsFileIdentity | None = None,
        max_bytes: int | None = None,
    ) -> bytes:
        if max_bytes is not None and (
            isinstance(max_bytes, bool) or not isinstance(max_bytes, int) or max_bytes < 0
        ):
            raise ValueError("max_bytes must be a non-negative integer")
        with self.open_relative(relative, "rb") as stream:
            before = identity_from_stream(stream)
            if expected_identity is not None and not before.same_file_state(expected_identity):
                raise WindowsIdentityMismatch("worker file identity does not match the expected identity")
            if max_bytes is not None and before.size > max_bytes:
                raise WindowsStorageError(223, "worker file exceeds the configured read limit")
            # Read at most one byte past the bound.  This preserves a bounded
            # allocation even when the file grows after the preflight size
            # check; the post-read identity check still rejects that race.
            data = stream.read(max_bytes + 1 if max_bytes is not None else -1)
            after = identity_from_stream(stream)
            if not before.same_file_state(after) or len(data) != before.size:
                raise WindowsIdentityMismatch("worker file changed while being read")
            if max_bytes is not None and len(data) > max_bytes:
                raise WindowsStorageError(223, "worker file exceeds the configured read limit")
        if expected_sha256 is not None and hashlib.sha256(data).hexdigest() != expected_sha256:
            raise WindowsStorageError(23, "worker file digest does not match its object id")
        return data

    def read_range(
        self,
        relative: os.PathLike[str] | str,
        offset: int,
        length: int,
        *,
        expected_sha256: str | None = None,
    ) -> bytes:
        if isinstance(offset, bool) or not isinstance(offset, int) or offset < 0:
            raise ValueError("offset must be a non-negative integer")
        if isinstance(length, bool) or not isinstance(length, int) or length < 0:
            raise ValueError("length must be a non-negative integer")
        with self.open_relative(relative, "rb") as stream:
            before = identity_from_stream(stream)
            if offset + length > before.size:
                raise ValueError("requested range is outside the worker file")
            if expected_sha256 is not None:
                stream.seek(0)
                digest = hashlib.sha256()
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
                if digest.hexdigest() != expected_sha256:
                    raise WindowsStorageError(23, "worker file digest does not match its object id")
            stream.seek(offset)
            result = stream.read(length)
            after = identity_from_stream(stream)
            if not before.same_file_state(after) or len(result) != length:
                raise WindowsIdentityMismatch("worker file changed during range read")
            return result

    def sha256(
        self,
        relative: os.PathLike[str] | str,
        *,
        chunk_bytes: int = 1024 * 1024,
    ) -> tuple[str, WindowsFileIdentity]:
        if isinstance(chunk_bytes, bool) or not isinstance(chunk_bytes, int) or chunk_bytes <= 0:
            raise ValueError("chunk_bytes must be a positive integer")
        with self.open_relative(relative, "rb") as stream:
            before = identity_from_stream(stream)
            digest = hashlib.sha256()
            while chunk := stream.read(chunk_bytes):
                digest.update(chunk)
            after = identity_from_stream(stream)
            if not before.same_file_state(after):
                raise WindowsIdentityMismatch("worker file changed while hashing")
            return digest.hexdigest(), after

    def iter_file(
        self,
        relative: os.PathLike[str] | str,
        *,
        chunk_bytes: int = 1024 * 1024,
    ) -> Iterator[bytes]:
        if isinstance(chunk_bytes, bool) or not isinstance(chunk_bytes, int) or chunk_bytes <= 0:
            raise ValueError("chunk_bytes must be a positive integer")
        with self.open_relative(relative, "rb") as stream:
            before = identity_from_stream(stream)
            while chunk := stream.read(chunk_bytes):
                yield chunk
            after = identity_from_stream(stream)
            if not before.same_file_state(after):
                raise WindowsIdentityMismatch("worker file changed while streaming")

    def write_bytes(
        self,
        relative: os.PathLike[str] | str,
        data: bytes,
        *,
        expected_sha256: str | None = None,
        expected_identity: WindowsFileIdentity | None = None,
        mode: int = 0o644,
    ) -> WindowsFileIdentity:
        del mode  # Windows ACL inheritance, rather than POSIX mode bits, applies.
        if not isinstance(data, (bytes, bytearray, memoryview)):
            raise TypeError("worker writes require bytes")
        payload = bytes(data)
        digest = hashlib.sha256(payload).hexdigest()
        if expected_sha256 is not None and digest != expected_sha256:
            raise WindowsStorageError(23, "worker write digest does not match expected object id")
        return self.atomic_write_stream(
            relative,
            (payload,),
            expected_sha256=digest,
            expected_size=len(payload),
            expected_identity=expected_identity,
        )

    def atomic_write_stream(
        self,
        relative: os.PathLike[str] | str,
        chunks: Iterable[bytes],
        *,
        expected_sha256: str,
        expected_size: int,
        mode: int = 0o644,
        expected_identity: WindowsFileIdentity | None = None,
    ) -> WindowsFileIdentity:
        del mode
        if isinstance(expected_size, bool) or not isinstance(expected_size, int) or expected_size < 0:
            raise ValueError("expected_size must be a non-negative integer")
        if not isinstance(expected_sha256, str) or not _SHA256_RE.fullmatch(expected_sha256):
            raise ValueError("expected_sha256 must be a lowercase SHA-256 digest")
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            with self._mutable_parent(parts) as parent:
                destination = parts[-1]
                old_identity: WindowsFileIdentity | None = None
                try:
                    old_handle = self._open_leaf(parent, destination, "rb")
                except WindowsStorageError as error:
                    if _windows_error_code(error) not in {2, 3}:
                        raise
                else:
                    try:
                        old_identity = _ensure_regular_file_handle(old_handle)
                    finally:
                        _close_handle(old_handle)
                if expected_identity is not None:
                    if old_identity is None or not old_identity.same_file_state(expected_identity):
                        raise WindowsIdentityMismatch("worker destination changed before replacement")

                temporary_name = f".worker-{secrets.token_hex(16)}.tmp"
                temporary_handle = self._open_leaf(parent, temporary_name, "w+b", create=True)
                renamed = False
                try:
                    identity = _ensure_regular_file_handle(temporary_handle)
                    observed_size = 0
                    digest = hashlib.sha256()
                    for chunk in chunks:
                        if not isinstance(chunk, (bytes, bytearray, memoryview)):
                            raise TypeError("stream chunks must be bytes")
                        payload = memoryview(chunk)
                        observed_size += len(payload)
                        if observed_size > expected_size:
                            raise WindowsStorageError(223, "worker stream exceeds its declared size")
                        digest.update(payload)
                        _write_handle(temporary_handle, payload)
                    if observed_size != expected_size or digest.hexdigest() != expected_sha256:
                        raise WindowsStorageError(23, "worker stream digest or size does not match declaration")
                    kernel32, _ntdll, _msvcrt = _apis()
                    if not kernel32.FlushFileBuffers(ctypes.c_void_p(temporary_handle)):
                        _raise_last_error("FlushFileBuffers")
                    after_write = identity_from_handle(temporary_handle)
                    if not identity.same_object(after_write) or after_write.size != expected_size:
                        raise WindowsIdentityMismatch("worker temporary file identity changed while writing")
                    _rename_handle(temporary_handle, parent, destination, replace=True)
                    renamed = True
                    published = identity_from_handle(temporary_handle)
                    if published.size != expected_size:
                        raise WindowsIdentityMismatch("published worker file has an unexpected size")
                    # The renamed handle remains open without FILE_SHARE_DELETE,
                    # so another process cannot swap the destination entry
                    # before this operation returns.  Its post-rename identity
                    # is therefore also the destination identity.
                    return published
                finally:
                    if not renamed:
                        try:
                            _mark_delete(temporary_handle)
                        except WindowsStorageError:
                            pass
                    _close_handle(temporary_handle)

    def delete(
        self,
        relative: os.PathLike[str] | str,
        *,
        expected_identity: WindowsFileIdentity | None = None,
    ) -> None:
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            with self._mutable_parent(parts) as parent:
                handle = self._open_leaf(parent, parts[-1], "delete")
                try:
                    identity = _ensure_regular_file_handle(handle)
                    if expected_identity is not None and not identity.same_file_state(expected_identity):
                        raise WindowsIdentityMismatch("worker file changed before deletion")
                    _mark_delete(handle)
                finally:
                    _close_handle(handle)

    def ensure_directory(self, relative: os.PathLike[str] | str) -> None:
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            parent = self._root_handle
            owned: list[int] = []
            try:
                for component in parts:
                    try:
                        handle = _open_relative_handle(
                            parent,
                            component,
                            access=_DIRECTORY_MUTATE_ACCESS,
                            share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                            options=(
                                _FILE_DIRECTORY_FILE
                                | _FILE_OPEN_REPARSE_POINT
                                | _FILE_SYNCHRONOUS_IO_NONALERT
                            ),
                        )
                    except WindowsStorageError as error:
                        if _windows_error_code(error) not in {2, 3}:
                            raise
                        try:
                            handle = _open_relative_handle(
                                parent,
                                component,
                                access=_DIRECTORY_MUTATE_ACCESS,
                                share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                                disposition=_FILE_CREATE,
                                options=(
                                    _FILE_DIRECTORY_FILE
                                    | _FILE_OPEN_REPARSE_POINT
                                    | _FILE_SYNCHRONOUS_IO_NONALERT
                                ),
                                attributes=_FILE_ATTRIBUTE_DIRECTORY,
                            )
                        except WindowsStorageError as create_error:
                            if _windows_error_code(create_error) not in {80, 183}:
                                raise
                            handle = _open_relative_handle(
                                parent,
                                component,
                                access=_DIRECTORY_MUTATE_ACCESS,
                                share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                                options=(
                                    _FILE_DIRECTORY_FILE
                                    | _FILE_OPEN_REPARSE_POINT
                                    | _FILE_SYNCHRONOUS_IO_NONALERT
                                ),
                            )
                    try:
                        _ensure_directory_handle(handle)
                    except BaseException:
                        _close_handle(handle)
                        raise
                    owned.append(handle)
                    parent = handle
            finally:
                for handle in reversed(owned):
                    _close_handle(handle)

    def create_directory(self, relative: os.PathLike[str] | str) -> WindowsFileIdentity:
        """Exclusively create one directory and return its handle-bound identity.

        Missing parent components are created and verified through pinned
        directory handles.  The final component uses ``FILE_CREATE`` without
        an existing-path fallback, so an existing directory can never be
        mistaken for one created by this call.  If identity collection fails
        after creation, the new directory is intentionally left in place; this
        method never deletes a path it cannot prove it owns.
        """
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            with self._mutable_parent(parts) as parent:
                try:
                    handle = _open_relative_handle(
                        parent,
                        parts[-1],
                        access=_DIRECTORY_MUTATE_ACCESS,
                        share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                        disposition=_FILE_CREATE,
                        options=(
                            _FILE_DIRECTORY_FILE
                            | _FILE_OPEN_REPARSE_POINT
                            | _FILE_SYNCHRONOUS_IO_NONALERT
                        ),
                        attributes=_FILE_ATTRIBUTE_DIRECTORY,
                    )
                except WindowsStorageError as error:
                    if _windows_error_code(error) in {80, 183}:
                        raise FileExistsError(
                            _windows_error_code(error),
                            "worker directory already exists",
                            os.fspath(relative),
                        ) from error
                    raise
                try:
                    # Read identity from the exact FILE_CREATE handle while
                    # its no-share-delete pin still prevents name replacement.
                    return _ensure_directory_handle(handle)
                finally:
                    _close_handle(handle)

    def delete_empty_directory(
        self,
        relative: os.PathLike[str] | str,
        *,
        expected_identity: WindowsFileIdentity | None = None,
    ) -> None:
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            with self._mutable_parent(parts) as parent:
                handle = _open_relative_handle(
                    parent,
                    parts[-1],
                    access=_DELETE | _FILE_LIST_DIRECTORY | _FILE_READ_ATTRIBUTES | _SYNCHRONIZE,
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    options=(
                        _FILE_DIRECTORY_FILE
                        | _FILE_OPEN_REPARSE_POINT
                        | _FILE_SYNCHRONOUS_IO_NONALERT
                    ),
                )
                try:
                    identity = _ensure_directory_handle(handle)
                    if expected_identity is not None and not identity.same_object(expected_identity):
                        raise WindowsIdentityMismatch("worker directory changed before removal")
                    if _directory_entries(handle):
                        raise WindowsStorageError(145, "worker directory is not empty")
                    _mark_delete(handle)
                finally:
                    _close_handle(handle)

    def _directory_relative(self, relative: os.PathLike[str] | str):
        parts = _relative_parts(relative)
        parent_parts = parts[:-1]
        leaf = parts[-1]
        stack: list[int] = []
        parent = self._root_handle
        try:
            for component in parent_parts:
                handle = _open_relative_handle(
                    parent,
                    component,
                    access=_DIRECTORY_READ_ACCESS,
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    options=(
                        _FILE_DIRECTORY_FILE
                        | _FILE_OPEN_REPARSE_POINT
                        | _FILE_SYNCHRONOUS_IO_NONALERT
                    ),
                )
                try:
                    _ensure_directory_handle(handle)
                except BaseException:
                    _close_handle(handle)
                    raise
                stack.append(handle)
                parent = handle
            handle = _open_relative_handle(
                parent,
                leaf,
                access=_DIRECTORY_READ_ACCESS,
                share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                options=(
                    _FILE_DIRECTORY_FILE
                    | _FILE_OPEN_REPARSE_POINT
                    | _FILE_SYNCHRONOUS_IO_NONALERT
                ),
            )
            try:
                _ensure_directory_handle(handle)
            except BaseException:
                _close_handle(handle)
                raise
            stack.append(handle)
            return parts, stack
        except BaseException:
            for handle in reversed(stack):
                _close_handle(handle)
            raise

    def _walk_directory(
        self,
        handle: int,
        prefix: str,
        files: list[str],
        directories: list[str],
        compiled_check=None,
    ) -> None:
        for name, attributes in _directory_entries(handle):
            relative = f"{prefix}/{name}" if prefix else name
            if attributes & _FILE_ATTRIBUTE_REPARSE_POINT:
                raise WindowsPathError("worker storage contains a reparse point")
            if attributes & _FILE_ATTRIBUTE_DIRECTORY:
                child = _open_relative_handle(
                    handle,
                    name,
                    access=_DIRECTORY_READ_ACCESS,
                    share=_FILE_SHARE_READ | _FILE_SHARE_WRITE,
                    options=(
                        _FILE_DIRECTORY_FILE
                        | _FILE_OPEN_REPARSE_POINT
                        | _FILE_SYNCHRONOUS_IO_NONALERT
                    ),
                )
                try:
                    _ensure_directory_handle(child)
                    directories.append(relative)
                    self._walk_directory(child, relative, files, directories, compiled_check)
                finally:
                    _close_handle(child)
            else:
                file_handle = self._open_leaf(handle, name, "rb")
                try:
                    if compiled_check is None:
                        _ensure_regular_file_handle(file_handle)
                    else:
                        compiled_check(relative, file_handle)
                    files.append(relative)
                finally:
                    _close_handle(file_handle)

    def list_files(self, relative_directory: os.PathLike[str] | str) -> list[str]:
        with self._lock:
            self.verify_root()
            parts, handles = self._directory_relative(relative_directory)
            files: list[str] = []
            directories: list[str] = []
            try:
                self._walk_directory(handles[-1], "/".join(parts), files, directories)
            finally:
                for handle in reversed(handles):
                    _close_handle(handle)
            return sorted(files)

    def list_directories(self, relative_directory: os.PathLike[str] | str) -> list[str]:
        with self._lock:
            self.verify_root()
            parts, handles = self._directory_relative(relative_directory)
            files: list[str] = []
            directories: list[str] = []
            try:
                self._walk_directory(handles[-1], "/".join(parts), files, directories)
            finally:
                for handle in reversed(handles):
                    _close_handle(handle)
            return sorted(directories, key=lambda item: (item.count("/"), item), reverse=True)

    def compiled_inventory(self, relative_directory: str, *, allowed_directories: tuple[str, ...]) -> tuple[list[str], list[str]]:
        """Use the dedicated exact-generation compiled hardlink ownership audit."""
        from .compiled_products import compiled_inventory
        return compiled_inventory(self, relative_directory, allowed_directories=allowed_directories)

    def set_mtime(
        self,
        relative: os.PathLike[str] | str,
        mtime_ns: int,
        *,
        expected_identity: WindowsFileIdentity,
    ) -> WindowsFileIdentity:
        if isinstance(mtime_ns, bool) or not isinstance(mtime_ns, int) or mtime_ns < 0:
            raise ValueError("mtime_ns must be a non-negative integer")
        parts = _relative_parts(relative)
        with self._lock:
            self.verify_root()
            with self._pinned_parent(parts) as parent:
                handle = _open_relative_handle(
                    parent,
                    parts[-1],
                    access=_FILE_READ_ATTRIBUTES | _FILE_WRITE_ATTRIBUTES | _SYNCHRONIZE,
                    share=_FILE_SHARE_READ,
                    options=_FILE_NON_DIRECTORY_FILE | _FILE_OPEN_REPARSE_POINT | _FILE_SYNCHRONOUS_IO_NONALERT,
                )
                try:
                    before = _ensure_regular_file_handle(handle)
                    if not before.same_file_state(expected_identity):
                        raise WindowsIdentityMismatch("worker file changed before timestamp update")
                    basic = _FileBasicInfo()
                    basic.LastWriteTime = (mtime_ns // 100) + _WINDOWS_EPOCH_TICKS
                    kernel32, _ntdll, _msvcrt = _apis()
                    if not kernel32.SetFileInformationByHandle(
                        ctypes.c_void_p(handle), _FILE_BASIC_INFO, ctypes.byref(basic), ctypes.sizeof(basic)
                    ):
                        _raise_last_error("SetFileInformationByHandle(FileBasicInfo)")
                    after = identity_from_handle(handle)
                    if not before.same_object(after) or after.size != before.size:
                        raise WindowsIdentityMismatch("worker file changed during timestamp update")
                    return after
                finally:
                    _close_handle(handle)

    def __enter__(self) -> "WindowsStorage":
        self.verify_root()
        return self

    def __exit__(self, _exc_type, _exc_value, _traceback) -> None:
        self.close()


__all__ = [
    "WindowsFileIdentity",
    "WindowsIdentityMismatch",
    "WindowsPathError",
    "WindowsStorage",
    "WindowsStorageError",
    "identity_from_handle",
    "identity_from_stream",
    "is_available",
]

"""Independent Windows handle-pinned storage under physical Cargo roots."""
from __future__ import annotations
import os
import ntpath
import re
import hashlib
from contextlib import contextmanager
from pathlib import Path
from dataclasses import dataclass, field
from typing import BinaryIO, Iterable, Iterator
from ..contracts import PilotError
from .portable_paths import normalize_portable_relative_path
from . import windows_storage as _windows_storage


WORKER_SUBDIRECTORIES = (
    "source",
    "target",
    "cargohome",
    "sccache",
    "logs",
    "tmp",
    "artifacts",
)

class PathBoundaryError(PilotError):
    """A path is not a safe worker-relative path or storage root."""

class StorageSecurityError(PathBoundaryError):
    """A link, replacement race, or identity mismatch was detected."""

class StorageConfigurationError(PathBoundaryError):
    """The worker storage configuration violates the physical path policy."""

class FileIdentityMismatch(StorageSecurityError):
    """A file changed between the preflight and the operation."""

@dataclass(frozen=True)
class FileIdentity:
    """A native volume/file identity and metadata for replacement checks."""

    device: int
    inode: int
    size: int
    mtime_ns: int
    links: int = 1
    native_identity: object | None = field(default=None, compare=False, repr=False)

    def same_file_state(self, other: object) -> bool:
        if not isinstance(other, FileIdentity):
            return False
        if self.native_identity is not None and other.native_identity is not None:
            compare = getattr(self.native_identity, "same_file_state", None)
            if compare is not None:
                return bool(compare(other.native_identity))
        return (
            self.device == other.device
            and self.inode == other.inode
            and self.size == other.size
            and self.mtime_ns == other.mtime_ns
            and self.links == other.links
        )

    @classmethod
    def from_stat(cls, value: os.stat_result) -> "FileIdentity":
        return cls(
            int(getattr(value, "st_dev", 0)),
            int(getattr(value, "st_ino", 0)),
            int(value.st_size),
            int(value.st_mtime_ns),
            int(getattr(value, "st_nlink", 1)),
        )

def _file_identity_from_windows(value: object) -> FileIdentity:
    try:
        return FileIdentity(
            device=int(getattr(value, "device")),
            inode=int(getattr(value, "inode")),
            size=int(getattr(value, "size")),
            mtime_ns=int(getattr(value, "mtime_ns")),
            links=int(getattr(value, "link_count")),
            native_identity=value,
        )
    except (AttributeError, TypeError, ValueError) as error:
        raise StorageSecurityError("Windows storage returned an invalid file identity") from error

def _native_windows_identity(value: FileIdentity | None) -> object | None:
    if value is None:
        return None
    native = value.native_identity
    if native is None:
        raise FileIdentityMismatch("file identity did not originate from the active storage backend")
    return native

def _validate_local_drive_target(drive: str, target: str) -> None:
    """Reject SUBST, network, device and other non-volume D/E/F aliases."""
    normalized_drive = drive.upper()
    if normalized_drive not in {"D:", "E:", "F:"}:
        raise StorageConfigurationError("local compiled/cache storage must use D:, E:, or F:")
    if not re.fullmatch(r"\\Device\\HarddiskVolume[0-9]+", target, flags=re.IGNORECASE):
        raise StorageConfigurationError(
            f"local compiled/cache drive {normalized_drive} is aliased instead of a physical volume"
        )

def _query_dos_device(drive: str, *, required: bool = True) -> str | None:
    if os.name != "nt":
        raise StorageConfigurationError("physical local drive verification is only available on Windows")
    import ctypes
    from ctypes import wintypes

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    query = kernel32.QueryDosDeviceW
    query.argtypes = [wintypes.LPCWSTR, wintypes.LPWSTR, wintypes.DWORD]
    query.restype = wintypes.DWORD
    buffer = ctypes.create_unicode_buffer(32768)
    length = query(drive, buffer, len(buffer))
    if not length:
        error = ctypes.get_last_error()
        if not required and error in {2, 3}:
            return None
        raise StorageConfigurationError(f"cannot resolve local drive mapping for {drive}")
    if length >= len(buffer):
        raise StorageConfigurationError(f"local drive mapping for {drive} is too long")
    return buffer.value

def _verify_local_drive_mapping(drive: str) -> str:
    target = _query_dos_device(drive)
    if target is None:
        raise StorageConfigurationError(f"local drive {drive} has no physical volume mapping")
    _validate_local_drive_target(drive, target)
    # A second DOS drive letter for the same volume is a physical alias,
    # including an approved letter mapped to the C: system volume.
    for other in (f"{letter}:" for letter in "ABCDEFGHIJKLMNOPQRSTUVWXYZ"):
        if other == drive.upper():
            continue
        other_target = _query_dos_device(other, required=False)
        if other_target is not None and other_target.casefold() == target.casefold():
            raise StorageConfigurationError(
                f"local compiled/cache drive {drive} aliases the same volume as {other}"
            )
    return target

def _verify_pinned_local_storage_mapping(
    storage: object, root: os.PathLike[str] | str, expected_device: str
) -> None:
    """Bind a local root's pinned handle to the preflighted physical drive.

    QueryDosDevice is checked before directory creation.  Rechecking it after
    WindowsStorage pins the root closes a drive-remapping window, while the
    handle's final NT path proves that the pinned directory resolved through
    that same mapping.
    """
    raw_root = os.fspath(root).replace("/", "\\")
    drive, tail = ntpath.splitdrive(raw_root)
    drive = drive.upper()
    if drive not in {"D:", "E:", "F:"}:
        raise StorageConfigurationError("local storage root must use D:, E:, or F:")
    _validate_local_drive_target(drive, expected_device)
    current_device = _verify_local_drive_mapping(drive)
    if current_device.casefold() != expected_device.casefold():
        raise StorageConfigurationError(
            f"local drive {drive} mapping changed while its worker root was being pinned"
        )

    relative_components = [part for part in tail.split("\\") if part]
    if any(part in {".", ".."} for part in relative_components):
        raise StorageConfigurationError("local worker root contains unsafe path components")
    expected_nt_path = expected_device.rstrip("\\")
    if relative_components:
        expected_nt_path += "\\" + "\\".join(relative_components)
    try:
        verified = storage.verify_physical_path(expected_nt_path)
    except _windows_storage.WindowsStorageError as error:
        raise StorageConfigurationError(
            "pinned local worker root does not match its approved physical drive mapping"
        ) from error
    if verified is not True:
        raise StorageConfigurationError(
            "pinned local worker root does not match its approved physical drive mapping"
        )
    final_device = _verify_local_drive_mapping(drive)
    if final_device.casefold() != expected_device.casefold():
        raise StorageConfigurationError(
            f"local drive {drive} mapping changed while its worker root was being pinned"
        )

def _normalise_relative(value: object) -> str:
    try:
        return normalize_portable_relative_path(
            value,
            code="worker_path_invalid",
            message="Worker paths must be safe portable relative paths",
        )
    except Exception as error:
        if isinstance(error, PathBoundaryError):
            raise
        raise PathBoundaryError(str(error)) from error

class WorkerStorage:
    """A local Windows storage root with physical volume and pinned handle checks.

    Every root component is opened or created relative to its pinned parent.
    Remote policy and test bypasses are deliberately unavailable.
    """
    def __init__(self, storage_root: os.PathLike[str] | str, *,
                 storage_class: str = "local", create: bool = True) -> None:
        if storage_class != "local" or not _windows_storage.is_available():
            raise StorageConfigurationError("native physical local Windows storage is required")
        raw = os.fspath(storage_root).replace("/", "\\")
        drive, tail = ntpath.splitdrive(raw)
        components = tail.split("\\")[1:]
        if (drive.upper() not in {"D:", "E:", "F:"} or not tail.startswith("\\")
                or not components or components[0].casefold() != "cargo-targets"):
            raise StorageConfigurationError("storage root must be under drive-root D/E/F cargo-targets")
        for component in components:
            _normalise_relative(component)
        self.root = Path(raw)
        self.storage_class = "local"
        self._windows = None
        device = _verify_local_drive_mapping(drive.upper())
        try:
            self._windows = self._windows_action(lambda: _windows_storage.WindowsStorage(
                self.root, create=create, expected_device=device))
            _verify_pinned_local_storage_mapping(self._windows, self.root, device)
            self._root_identity = _file_identity_from_windows(self._windows.root_identity)
            self._subdirs = {name: self.root / name for name in WORKER_SUBDIRECTORIES}
            if create:
                for name in WORKER_SUBDIRECTORIES:
                    self.ensure_directory(name)
        except BaseException:
            if self._windows is not None:
                self._windows.close()
            raise

    @property
    def root_identity(self) -> FileIdentity:
        self._windows_action(self._windows.verify_root)
        return self._root_identity

    def subdir(self, name: str) -> Path:
        if name not in self._subdirs:
            raise PathBoundaryError(f"Unknown storage directory: {name}")
        self._windows_action(self._windows.verify_root)
        return self._subdirs[name]

    def close(self) -> None:
        if self._windows is not None:
            self._windows.close()

    def __enter__(self) -> "WorkerStorage":
        self._windows_action(self._windows.verify_root)
        return self

    def __exit__(self, _exc_type, _exc_value, _traceback) -> None:
        self.close()


    def ensure_directory(self, relative: object) -> Path:
        """Create a worker-relative directory one checked component at a time."""
        value = _normalise_relative(relative)
        self._windows_action(lambda: self._windows.ensure_directory(value))
        return self.root.joinpath(*value.split('/'))

    def create_directory(self, relative: object) -> FileIdentity:
        """Exclusively create a directory relative to its pinned parent."""
        value = _normalise_relative(relative)
        identity = self._windows_action(lambda: self._windows.create_directory(value))
        return _file_identity_from_windows(identity)

    @contextmanager
    def open_lock_file(self, relative: object) -> Iterator[BinaryIO]:
        """Retain a checked lock-file handle and its parents for the caller."""
        value = _normalise_relative(relative)
        try:
            with self._windows.open_lock_file(value) as stream:
                yield stream
        except _windows_storage.WindowsIdentityMismatch as error:
            raise FileIdentityMismatch(str(error)) from error
        except (_windows_storage.WindowsPathError, _windows_storage.WindowsStorageError) as error:
            raise StorageSecurityError(str(error)) from error

    def file_identity(self, relative: object) -> FileIdentity:
        value = _normalise_relative(relative)
        identity = self._windows_action(lambda: self._windows.file_identity(value))
        return _file_identity_from_windows(identity)

    def exists(self, relative: object) -> bool:
        value = _normalise_relative(relative)
        return bool(self._windows_action(lambda: self._windows.exists(value)))

    def read_bytes(self, relative: object, *, expected_sha256: str | None=None, expected_identity: FileIdentity | None=None, max_bytes: int | None=None) -> bytes:
        """Read one regular file with an optional identity and byte bound.

            Bounded reads request at most ``max_bytes + 1`` bytes so an oversized
            file cannot force an unbounded allocation between the size preflight
            and the actual read.
            """
        if max_bytes is not None and (isinstance(max_bytes, bool) or not isinstance(max_bytes, int) or max_bytes < 0):
            raise ValueError('max_bytes must be a non-negative integer')
        if expected_identity is not None and (not isinstance(expected_identity, FileIdentity)):
            raise TypeError('expected_identity must be a FileIdentity')
        value = _normalise_relative(relative)
        native_expected = None
        if expected_identity is not None:
            native_expected = expected_identity.native_identity
            if native_expected is None:
                raise FileIdentityMismatch('expected_identity was not captured from this native worker storage')
        return self._windows_action(lambda: self._windows.read_bytes(value, expected_sha256=expected_sha256, expected_identity=native_expected, max_bytes=max_bytes))

    def read_range(self, relative: object, offset: int, length: int, *, expected_sha256: str | None=None) -> bytes:
        if not isinstance(offset, int) or not isinstance(length, int) or offset < 0 or (length < 0):
            raise ValueError('offset and length must be non-negative integers')
        value = _normalise_relative(relative)
        return self._windows_action(lambda: self._windows.read_range(value, offset, length, expected_sha256=expected_sha256))

    def sha256(self, relative: object, *, chunk_bytes: int=1024 * 1024) -> tuple[str, FileIdentity]:
        if not isinstance(chunk_bytes, int) or chunk_bytes <= 0:
            raise ValueError('chunk_bytes must be a positive integer')
        value = _normalise_relative(relative)
        digest, identity = self._windows_action(lambda: self._windows.sha256(value, chunk_bytes=chunk_bytes))
        return (digest, _file_identity_from_windows(identity))

    def iter_file(self, relative: object, *, chunk_bytes: int=1024 * 1024) -> Iterator[bytes]:
        """Read a file through one verified handle in bounded chunks."""
        if not isinstance(chunk_bytes, int) or chunk_bytes <= 0:
            raise ValueError('chunk_bytes must be a positive integer')
        value = _normalise_relative(relative)
        yield from self._windows_iter_file(value, chunk_bytes)
        return

    def set_mtime(self, relative: object, mtime_ns: int, *, expected_identity: FileIdentity) -> FileIdentity:
        """Set a worker file's timestamp through its verified open handle."""
        if not isinstance(mtime_ns, int) or isinstance(mtime_ns, bool) or mtime_ns < 0:
            raise ValueError('mtime_ns must be a non-negative integer')
        value = _normalise_relative(relative)
        identity = self._windows_action(lambda: self._windows.set_mtime(value, mtime_ns, expected_identity=_native_windows_identity(expected_identity)))
        return _file_identity_from_windows(identity)

    def write_bytes(self, relative: object, data: bytes, *, expected_sha256: str | None=None, expected_identity: FileIdentity | None=None, mode: int=420) -> FileIdentity:
        if not isinstance(data, (bytes, bytearray, memoryview)):
            raise TypeError('worker writes require bytes')
        payload = bytes(data)
        if expected_sha256 is not None and hashlib.sha256(payload).hexdigest() != expected_sha256:
            raise StorageSecurityError('Worker write digest does not match expected object id')
        value = _normalise_relative(relative)
        identity = self._windows_action(lambda: self._windows.write_bytes(value, payload, expected_sha256=expected_sha256, expected_identity=_native_windows_identity(expected_identity), mode=mode))
        return _file_identity_from_windows(identity)

    def atomic_write_stream(self, relative: object, chunks: Iterable[bytes], *, expected_sha256: str, expected_size: int, mode: int=420, expected_identity: FileIdentity | None=None) -> FileIdentity:
        """Write a stream to a same-directory temporary file and replace atomically."""
        if not isinstance(expected_size, int) or expected_size < 0:
            raise ValueError('expected_size must be a non-negative integer')
        if not isinstance(expected_sha256, str) or len(expected_sha256) != 64:
            raise ValueError('expected_sha256 must be a SHA-256 hex digest')
        value = _normalise_relative(relative)
        identity = self._windows_action(lambda: self._windows.atomic_write_stream(value, chunks, expected_sha256=expected_sha256, expected_size=expected_size, mode=mode, expected_identity=_native_windows_identity(expected_identity)))
        return _file_identity_from_windows(identity)

    def delete(self, relative: object, *, expected_identity: FileIdentity | None=None) -> None:
        value = _normalise_relative(relative)
        self._windows_action(lambda: self._windows.delete(value, expected_identity=_native_windows_identity(expected_identity)))
        return

    def list_files(self, relative_directory: object) -> list[str]:
        value = _normalise_relative(relative_directory)
        return list(self._windows_action(lambda: self._windows.list_files(value)))

    def list_directories(self, relative_directory: object) -> list[str]:
        value = _normalise_relative(relative_directory)
        return list(self._windows_action(lambda: self._windows.list_directories(value)))

    def list_compiled_products(self, relative_directory: str = "build", *,
                               allowed_directories: tuple[str, ...] = ("build",)) -> list[str]:
        return self._compiled_inventory(relative_directory, allowed_directories)[0]

    def list_compiled_directories(self, relative_directory: str = "build", *,
                                  allowed_directories: tuple[str, ...] = ("build",)) -> list[str]:
        return self._compiled_inventory(relative_directory, allowed_directories)[1]

    def _compiled_inventory(self, relative_directory: str, allowed_directories: tuple[str, ...]):
        device = _verify_local_drive_mapping(self.root.drive)
        _verify_pinned_local_storage_mapping(self._windows, self.root, device)
        inventory = self._windows_action(lambda: self._windows.compiled_inventory(
            _normalise_relative(relative_directory), allowed_directories=allowed_directories))
        _verify_pinned_local_storage_mapping(self._windows, self.root, device)
        return inventory

    def delete_empty_directory(self, relative: object, *, expected_identity: FileIdentity | None=None) -> None:
        value = _normalise_relative(relative)
        self._windows_action(lambda: self._windows.delete_empty_directory(value, expected_identity=_native_windows_identity(expected_identity)))
        return

    @staticmethod
    def _windows_action(operation):
        try:
            return operation()
        except _windows_storage.WindowsIdentityMismatch as error:
            raise FileIdentityMismatch(str(error)) from error
        except _windows_storage.WindowsPathError as error:
            raise StorageSecurityError(str(error)) from error
        except _windows_storage.WindowsStorageError as error:
            code = getattr(error, 'winerror', None) or getattr(error, 'errno', None)
            if code in {2, 3}:
                raise FileNotFoundError(code, str(error)) from error
            raise StorageSecurityError(str(error)) from error

    def _windows_iter_file(self, relative: str, chunk_bytes: int) -> Iterator[bytes]:
        try:
            yield from self._windows.iter_file(relative, chunk_bytes=chunk_bytes)
        except _windows_storage.WindowsIdentityMismatch as error:
            raise FileIdentityMismatch(str(error)) from error
        except _windows_storage.WindowsPathError as error:
            raise StorageSecurityError(str(error)) from error
        except _windows_storage.WindowsStorageError as error:
            raise StorageSecurityError(str(error)) from error

"""Managed storage primitives for the Jenkins pilot.

The pilot deliberately has a smaller storage surface than the Session
coordinator.  It still has to obey the repository's drive-root Cargo storage
contract, so all state is confined below a D/E/F ``cargo-targets`` root.
"""

from __future__ import annotations

import os
import stat
import re
import secrets
from dataclasses import dataclass
from pathlib import Path, PureWindowsPath

from .native.paths import (
    WorkerStorage,
    _verify_local_drive_mapping,
)
from .native.portable_paths import normalize_portable_relative_path


DEFAULT_PILOT_ROOT = Path(r"D:\cargo-targets\jenkins-pilot-01a0f6e1")
_ALLOWED_DRIVES = frozenset({"D:", "E:", "F:"})


class StorageError(ValueError):
    """Base class for managed-storage admission failures."""


class StoragePathError(StorageError):
    """A path is outside the approved managed storage boundary."""


def _windows_parts(value: str | os.PathLike[str]) -> tuple[str, tuple[str, ...]]:
    text = os.fspath(value)
    parsed = PureWindowsPath(text)
    drive = parsed.drive.upper()
    if not parsed.is_absolute() or drive not in _ALLOWED_DRIVES:
        raise StoragePathError(
            "managed storage must be an absolute D:, E:, or F: path"
        )
    parts = tuple(part for part in parsed.parts[1:] if part not in {"\\", "/"})
    raw_parts = text.replace("/", "\\").split("\\")[1:]
    if any(part in {".", ".."} for part in raw_parts):
        raise StoragePathError("managed storage cannot contain traversal or path aliases")
    for part in parts:
        try:
            normalize_portable_relative_path(part, code="pilot_path", message="unsafe storage component")
        except ValueError as error:
            raise StoragePathError("managed storage contains an unsafe component") from error
    if not parts or parts[0].casefold() != "cargo-targets":
        raise StoragePathError("managed storage must be below a drive-root cargo-targets directory")
    return drive, parts


def require_managed_root(path: str | os.PathLike[str], *, allow_root: bool = True) -> Path:
    """Validate an absolute path below ``D/E/F:\\cargo-targets``.

    The lexical Windows check runs even when a test fixture is created on a
    non-Windows host. Existing components are then checked for symlinks or
    Windows reparse points before a caller is allowed to use the path.
    """

    drive, parts = _windows_parts(path)
    if not allow_root and len(parts) == 1:
        raise StoragePathError("a child of cargo-targets is required")
    if os.name != "nt":
        raise StoragePathError("this pilot requires native Windows physical storage")
    try:
        _verify_local_drive_mapping(drive)
    except (ValueError, RuntimeError) as error:
        raise StoragePathError("managed storage drive is not an approved physical volume") from error
    actual = Path(os.fspath(path))
    current = Path(f"{drive}\\")
    components = parts
    for component in components:
        current = current / component
        try:
            info = current.lstat()
        except FileNotFoundError:
            break
        except OSError as error:
            raise StoragePathError(f"cannot inspect managed storage path: {current}") from error
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise StoragePathError(f"managed storage cannot cross a symlink or reparse point: {current}")
        if stat.S_ISREG(info.st_mode) and info.st_nlink != 1:
            raise StoragePathError("managed storage cannot use hard-linked files")
    return actual


def _confined(root: Path, child: Path) -> Path:
    root_text = os.path.normcase(os.path.abspath(os.fspath(root)))
    child_text = os.path.normcase(os.path.abspath(os.fspath(child)))
    try:
        common = os.path.commonpath((root_text, child_text))
    except ValueError as error:
        raise StoragePathError("managed storage path has a different drive") from error
    if common != root_text:
        raise StoragePathError(f"path escapes managed storage: {child}")
    return child


def physical_identity(path: str | os.PathLike[str]) -> dict[str, object]:
    """Read stable directory identity from a physically verified pinned handle.

    This opens existing directories only. Timestamps and directory sizes that
    change when children are written are deliberately absent from the record.
    """
    root = require_managed_root(path)
    with WorkerStorage(root, create=False) as backend:
        identity = backend.root_identity.native_identity
        if identity is None:
            raise StoragePathError("physical identity requires the native pinned backend")
        return {"device": identity.device, "inode": identity.inode,
                "fileId": identity.file_id.hex(), "creationTime": identity.creation_time}


@dataclass(frozen=True)
class ManagedStorage:
    """A private filesystem layout under the approved target root."""

    root: Path

    def __init__(self, root: str | os.PathLike[str] = DEFAULT_PILOT_ROOT) -> None:
        value = require_managed_root(root, allow_root=False)
        object.__setattr__(self, "root", value)

    @property
    def snapshots_dir(self) -> Path:
        return _confined(self.root, self.root / "snapshots")

    @property
    def receipts_dir(self) -> Path:
        return _confined(self.root, self.root / "receipts")

    @property
    def tmp_dir(self) -> Path:
        return _confined(self.root, self.root / "tmp")

    def ensure_layout(self) -> None:
        with self.backend() as backend:
            for name in ("snapshots", "receipts", "tmp"):
                backend.ensure_directory(name)

    def backend(self) -> WorkerStorage:
        """Open the independently owned native handle-pinned implementation."""
        require_managed_root(self.root, allow_root=False)
        return WorkerStorage(self.root, storage_class="local")

    def path(self, relative: str | os.PathLike[str]) -> Path:
        """Return a confined child and reject absolute or traversal paths."""

        raw = os.fspath(relative).replace("\\", "/")
        if any(part in {"", ".", ".."} for part in raw.split("/")):
            raise StoragePathError("managed storage child must be a canonical relative path")
        try:
            name = normalize_portable_relative_path(raw, code="pilot_path", message="unsafe storage child")
        except ValueError as error:
            raise StoragePathError("managed storage child is unsafe") from error
        return require_managed_root(_confined(self.root, self.root.joinpath(*name.split("/"))))

    def temporary_directory(self, *, prefix: str = "job-") -> Path:
        if not re.fullmatch(r"[A-Za-z0-9_-]{1,64}", prefix):
            raise StoragePathError("temporary prefix must be a bounded identifier")
        self.ensure_layout()
        with self.backend() as backend:
            # Exclusive creation relative to a pinned parent also prevents a
            # checked temporary parent from being swapped before creation.
            for _ in range(100):
                relative = f"tmp/{prefix}{secrets.token_hex(16)}"
                try:
                    backend.create_directory(relative)
                    return self.root.joinpath(*relative.split("/"))
                except FileExistsError:
                    continue
        raise StorageError("could not allocate a unique temporary directory")

    def atomic_write(self, relative: str, content: bytes) -> Path:
        """Write bytes through a same-directory temporary file and replace."""

        target = self.path(relative)
        with self.backend() as backend:
            backend.write_bytes(target.relative_to(self.root).as_posix(), content)
        return target

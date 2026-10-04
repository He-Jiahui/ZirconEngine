"""Physical admission for the repository build root and historical stores."""
from __future__ import annotations

import ctypes
import ctypes.wintypes as wt
import ntpath
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from ..contracts import JenkinsError

APPROVED_NAMES = {"D:\\cargo-targets", "E:\\cargo-targets", "F:\\cargo-targets"}
_REPARSE = 0x0400
_FILE_FLAG_BACKUP_SEMANTICS = 0x02000000
_GENERIC_READ = 0x80000000
_FILE_SHARE_ALL = 0x00000001 | 0x00000002 | 0x00000004
_OPEN_EXISTING = 3
_INVALID_HANDLE = ctypes.c_void_p(-1).value


def _fail(code: str, message: str) -> JenkinsError:
    return JenkinsError(code, message)


def _win32() -> ctypes.WinDLL:
    if os.name != "nt":
        raise _fail("physical_identity_unavailable", "Windows physical identity APIs are required")
    k = ctypes.WinDLL("kernel32", use_last_error=True)
    k.QueryDosDeviceW.argtypes = [wt.LPCWSTR, wt.LPWSTR, wt.DWORD]
    k.QueryDosDeviceW.restype = wt.DWORD
    k.GetFileAttributesW.argtypes = [wt.LPCWSTR]
    k.GetFileAttributesW.restype = wt.DWORD
    k.CreateFileW.argtypes = [wt.LPCWSTR, wt.DWORD, wt.DWORD, wt.LPVOID, wt.DWORD, wt.DWORD, wt.HANDLE]
    k.CreateFileW.restype = wt.HANDLE
    k.GetFinalPathNameByHandleW.argtypes = [wt.HANDLE, wt.LPWSTR, wt.DWORD, wt.DWORD]
    k.GetFinalPathNameByHandleW.restype = wt.DWORD
    k.CloseHandle.argtypes = [wt.HANDLE]
    k.CloseHandle.restype = wt.BOOL
    return k


def _drive_identity(drive: str) -> str:
    """Return the DOS-device target, rejecting SUBST and mapped drives."""
    k = _win32()
    buf = ctypes.create_unicode_buffer(32768)
    n = k.QueryDosDeviceW(f"{drive.upper()}:", buf, len(buf))
    if not n:
        raise _fail("physical_identity_unavailable", f"QueryDosDevice failed for {drive}:")
    target = buf.value
    # SUBST and mapped/alias drives resolve through a DOS path, rather than a
    # local volume device.  Accept only local device namespace identities.
    low = target.casefold()
    if low.startswith(("\\??\\", "\\dosdevices\\")) or "\\mup\\" in low or "\\device\\lanmanredirector" in low:
        raise _fail("build_root_alias", f"Drive {drive}: is a mapped or SUBST alias")
    if not low.startswith("\\device\\"):
        raise _fail("physical_identity_unavailable", f"Unrecognised drive identity: {target}")
    return target


def _attrs(path: str) -> int:
    k = _win32()
    value = k.GetFileAttributesW(path)
    if value == 0xFFFFFFFF:
        err = ctypes.get_last_error()
        if err in (2, 3):
            return -1
        raise _fail("physical_identity_unavailable", f"GetFileAttributes failed for {path} ({err})")
    return int(value)


def _final_path(path: str) -> str:
    k = _win32()
    h = k.CreateFileW(path, _GENERIC_READ, _FILE_SHARE_ALL, None, _OPEN_EXISTING, _FILE_FLAG_BACKUP_SEMANTICS, None)
    if ctypes.cast(h, ctypes.c_void_p).value == _INVALID_HANDLE:
        raise _fail("physical_identity_unavailable", f"Cannot open path for physical identity: {path}")
    try:
        buf = ctypes.create_unicode_buffer(32768)
        n = k.GetFinalPathNameByHandleW(h, buf, len(buf), 0)
        if not n or n >= len(buf):
            raise _fail("physical_identity_unavailable", f"GetFinalPathNameByHandle failed for {path}")
        return buf.value
    finally:
        k.CloseHandle(h)


def _strip_extended(path: str) -> str:
    value = path.replace("/", "\\")
    if value.startswith("\\\\?\\UNC\\"):
        return "\\\\" + value[8:]
    if value.startswith("\\\\?\\"):
        return value[4:]
    return value


def _same_path(a: str, b: str) -> bool:
    return ntpath.normcase(ntpath.normpath(_strip_extended(a))).casefold() == ntpath.normcase(ntpath.normpath(_strip_extended(b))).casefold()


def _existing_components(path: str) -> Iterable[str]:
    drive, tail = ntpath.splitdrive(path)
    current = drive + "\\"
    yield current
    for part in [p for p in tail.split("\\") if p]:
        current = ntpath.join(current, part)
        yield current


def _check_components(path: str, *, allow_missing_leaf: bool) -> None:
    components = list(_existing_components(path))
    for index, component in enumerate(components):
        attributes = _attrs(component)
        if attributes == -1:
            if allow_missing_leaf and all(_attrs(c) == -1 for c in components[index:]):
                return
            raise _fail("physical_identity_unavailable", f"Path component does not exist: {component}")
        if attributes & _REPARSE:
            raise _fail("build_root_reparse_point", f"Path component is a reparse point: {component}")


@dataclass(frozen=True)
class ApprovedBuildRoot:
    path: Path
    volume_identity: str = ""

    def namespace(self) -> Path:
        return self.path / "zircon-jenkins"


def repository_build_root(repository_root: str | os.PathLike[str] | None = None) -> Path:
    source = Path(__file__).absolute()
    repo = repository_root or next((part.parent for part in source.parents if part.name.casefold() == '.jenkins'), source.parents[3])
    return Path(repo).absolute() / '.jenkins' / 'builds'


def canonical_build_root(path: str | os.PathLike[str] | None, *, default: str | None = None,
                         repository_root: str | os.PathLike[str] | None = None) -> ApprovedBuildRoot:
    """Validate physical identity; formal write admission also checks the deployment policy.

    Historical drive roots remain identifiable for old receipts and recovery.
    New work defaults to the exact repository-local root, never to a drive fallback.
    """
    raw = (default or str(repository_build_root(repository_root))) if path is None else os.fspath(path)
    raw = str(raw).replace("/", "\\")
    drive, tail = ntpath.splitdrive(raw)
    if len(drive) != 2 or drive[1] != ":" or not tail.startswith("\\") or tail.endswith("\\") or ntpath.normpath(raw) != raw:
        raise _fail("build_root_not_approved", f"Build root is not canonical: {raw}")
    normalized = drive.upper() + tail
    approved_names = APPROVED_NAMES | {str(repository_build_root(repository_root))}
    if normalized.casefold() not in {x.casefold() for x in approved_names}:
        raise _fail("build_root_not_approved", f"Build root is not approved: {raw}")
    identity = _drive_identity(drive[0])
    _check_components(normalized, allow_missing_leaf=False)
    final = _final_path(normalized)
    if not _same_path(final, normalized):
        raise _fail("build_root_alias", f"Build root resolves to a different physical path: {final}")
    return ApprovedBuildRoot(Path(normalized), identity)


def build_root_containing(path: str | os.PathLike[str]) -> ApprovedBuildRoot:
    """Resolve an existing or planned child through an exact approved root."""
    raw = str(os.fspath(path)).replace('/', '\\')
    for name in sorted(APPROVED_NAMES | {str(repository_build_root())}, key=len, reverse=True):
        if raw.casefold() == name.casefold() or raw.casefold().startswith(name.casefold() + '\\'):
            root = canonical_build_root(name)
            physical_path_under(root, raw)
            return root
    raise _fail('build_root_not_approved', 'Path has no approved physical build root')


def physical_path_under(root: ApprovedBuildRoot, path: str | os.PathLike[str], *, allow_missing: bool = True) -> Path:
    """Validate a path and all ancestors under a previously verified build root."""
    raw = str(os.fspath(path)).replace("/", "\\")
    if ntpath.splitdrive(raw)[0] == "" or ntpath.normpath(raw) != raw or any(part in ("", ".", "..") for part in raw.split("\\")):
        raise _fail("namespace_escape", f"Path is not canonical: {raw}")
    if any(":" in part for part in raw.split("\\")[1:]):
        raise _fail("namespace_escape", f"Alternate data streams are not admitted: {raw}")
    base = str(root.path)
    candidate = ntpath.normpath(raw)
    try:
        common = ntpath.commonpath([base, candidate])
    except ValueError:
        common = ""
    if ntpath.normcase(common) != ntpath.normcase(base):
        raise _fail("namespace_escape", f"Path escapes approved build root: {raw}")
    _check_components(candidate, allow_missing_leaf=allow_missing)
    # Existing candidates must resolve to themselves, preventing junctions and
    # symlink aliases even when the lexical path is inside the root.
    if _attrs(candidate) != -1:
        final = _final_path(candidate)
        if not _same_path(final, candidate):
            raise _fail("build_root_alias", f"Path resolves through an alias: {candidate}")
    return Path(candidate)


def build_namespace(root: ApprovedBuildRoot, *parts: str) -> Path:
    if any(not p or p in (".", "..") or ntpath.isabs(p) or "/" in p or "\\" in p for p in parts):
        raise _fail("namespace_escape", "Namespace components must be relative names")
    return physical_path_under(root, root.namespace().joinpath(*parts), allow_missing=True)

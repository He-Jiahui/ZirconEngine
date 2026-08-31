"""NativeDynamic materialization IO and path helpers."""

from __future__ import annotations

import hashlib
import os
import re
import shutil
import stat
import threading
import time
import uuid
from contextlib import contextmanager
from contextvars import ContextVar
from pathlib import Path
from typing import Callable, Iterator


_NATIVE_DYNAMIC_CAS_ENV = "ZIRCON_NATIVE_DYNAMIC_CAS_ROOT"
_NATIVE_DYNAMIC_CAS_MAX_BYTES_ENV = "ZIRCON_NATIVE_DYNAMIC_CAS_MAX_BYTES"
_DEFAULT_NATIVE_DYNAMIC_CAS_MAX_BYTES = 4 * 1024 * 1024 * 1024
_CAS_CHUNK_SIZE = 1024 * 1024
_CAS_LOCK_TIMEOUT_SECONDS = 30.0
_CAS_LOCK_RETRY_SECONDS = 0.01
_CAS_TEMPORARY_NAME = re.compile(
    r"^\.(?P<suffix>[0-9a-f]{62})\.(?P<pid>[1-9][0-9]*)\."
    r"(?:(?P<thread>[1-9][0-9]*)\.)?(?P<nonce>[0-9a-f]{32})\.tmp$"
)
_CAS_THREAD_LOCKS = tuple(threading.Lock() for _ in range(64))
_CAS_PRUNE_THREAD_LOCK = threading.Lock()
_CAS_HARDLINKS_ALLOWED: ContextVar[bool] = ContextVar(
    "native_dynamic_cas_hardlinks_allowed",
    default=True,
)
_CopyFunction = Callable[[Path, Path], object]
_FileIdentity = tuple[int, int, int, int, int]


@contextmanager
def native_dynamic_cas_scope(*, allow_hardlinks: bool) -> Iterator[None]:
    """Apply one run-local hardlink policy without mutating process globals."""
    token = _CAS_HARDLINKS_ALLOWED.set(bool(allow_hardlinks))
    try:
        yield
    finally:
        try:
            if allow_hardlinks:
                cas_root = resolve_native_dynamic_cas_root()
                if cas_root is not None:
                    prune_native_dynamic_cas(cas_root)
        except OSError:
            # Cache maintenance must not invalidate an otherwise complete export.
            pass
        finally:
            _CAS_HARDLINKS_ALLOWED.reset(token)


def resolve_native_dynamic_cas_root(
    cas_root: str | Path | None = None,
) -> Path | None:
    """Resolve the optional shared content-addressed artifact store."""
    configured = cas_root
    if configured is None:
        configured = os.environ.get(_NATIVE_DYNAMIC_CAS_ENV)
    if configured is None or not str(configured).strip():
        return None
    if not _CAS_HARDLINKS_ALLOWED.get():
        return None
    candidate = Path(configured).expanduser().absolute()
    _assert_native_dynamic_no_reparse_components(candidate, "native dynamic CAS root")
    return candidate


def native_dynamic_cas_max_bytes() -> int:
    configured = os.environ.get(_NATIVE_DYNAMIC_CAS_MAX_BYTES_ENV)
    if configured is None or not configured.strip():
        return _DEFAULT_NATIVE_DYNAMIC_CAS_MAX_BYTES
    try:
        maximum = int(configured)
    except ValueError as error:
        raise OSError("native dynamic CAS byte limit must be an integer") from error
    if maximum < 0:
        raise OSError("native dynamic CAS byte limit cannot be negative")
    return maximum


def prune_native_dynamic_cas(
    cas_root: str | Path,
    *,
    max_bytes: int | None = None,
) -> dict[str, int]:
    """Evict oldest unreferenced blobs until the content store fits its limit."""
    root = Path(cas_root).expanduser().absolute()
    _assert_native_dynamic_no_reparse_components(root, "native dynamic CAS root")
    limit = native_dynamic_cas_max_bytes() if max_bytes is None else int(max_bytes)
    if limit < 0:
        raise OSError("native dynamic CAS byte limit cannot be negative")
    with _native_dynamic_prune_lock(root):
        temporary_before, temporary_after, removed_temporaries = (
            _remove_stale_native_dynamic_temporaries(root)
        )
        blobs = _native_dynamic_cas_blobs(root)
        blob_bytes = sum(
            size for _path, _digest, size, _modified in blobs
        )
        total_bytes = temporary_after + blob_bytes
        before_bytes = temporary_before + blob_bytes
        removed = 0
        for blob, digest, known_size, _modified in sorted(
            blobs, key=lambda item: (item[3], str(item[0]).casefold())
        ):
            if total_bytes <= limit:
                break
            with _native_dynamic_blob_lock(root, digest):
                try:
                    current = blob.stat()
                except FileNotFoundError:
                    total_bytes -= known_size
                    continue
                if current.st_nlink > 1:
                    continue
                _make_native_dynamic_writable(blob)
                blob.unlink()
                total_bytes -= current.st_size
                removed += 1
        return {
            "beforeBytes": before_bytes,
            "afterBytes": max(total_bytes, 0),
            "removedBlobs": removed,
            "removedTemporaryFiles": removed_temporaries,
        }


def reset_native_dynamic_plugins_dir(
    stage_dir: Path,
    diagnostics: list[str],
) -> bool:
    plugins_dir = stage_dir / "plugins"
    if plugins_dir.exists():
        if not remove_native_dynamic_dir(
            "NativeDynamic plugins directory",
            plugins_dir,
            diagnostics,
        ):
            return False
    try:
        plugins_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic plugins directory {plugins_dir} could not be created: {error}"
        )
        return False
    return True


def remove_native_dynamic_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> bool:
    try:
        shutil.rmtree(directory, onerror=_remove_native_dynamic_readonly)
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be removed: {error}")
        return False
    return True


def list_native_dynamic_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> list[Path] | None:
    try:
        return list(directory.iterdir())
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be listed: {error}")
        return None


def copy_native_dynamic_file(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
    *,
    cas_root: str | Path | None = None,
    copy_function: _CopyFunction | None = None,
) -> bool:
    try:
        _validate_native_dynamic_source_file(source)
        destination.parent.mkdir(parents=True, exist_ok=True)
        copy_file = copy_function or shutil.copy2
        resolved_cas_root = resolve_native_dynamic_cas_root(cas_root)
        if resolved_cas_root is None:
            # Keep the legacy path intact when CAS is not configured. Several
            # callers deliberately inject copy2 failures for diagnostics.
            copy_file(source, destination)
        else:
            _materialize_native_dynamic_source(
                source,
                destination,
                resolved_cas_root,
                copy_function=copy_file,
            )
    except OSError as error:
        diagnostics.append(f"{label} {source} could not be copied to {destination}: {error}")
        return False
    return True


def copy_native_dynamic_tree(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
    *,
    cas_root: str | Path | None = None,
) -> bool:
    try:
        _validate_native_dynamic_source_tree(source)
        resolved_cas_root = resolve_native_dynamic_cas_root(cas_root)
        if resolved_cas_root is None:
            shutil.copytree(source, destination)
        else:
            _materialize_native_dynamic_tree(
                source,
                destination,
                resolved_cas_root,
            )
    except OSError as error:
        diagnostics.append(f"{label} {source} could not be copied to {destination}: {error}")
        return False
    return True


def _ensure_native_dynamic_blob(
    source: Path,
    cas_root: Path,
) -> tuple[Path, str, int, _FileIdentity, int]:
    """Publish one source file atomically under its SHA-256 identity."""
    source_stat = _validate_native_dynamic_source_file(source)
    source_identity = _native_dynamic_stat_identity(source_stat)
    digest = _sha256_file(source)
    _require_native_dynamic_source_identity(source, source_identity)
    blob_directory = cas_root / "sha256" / digest[:2]
    blob = blob_directory / digest[2:]
    with _native_dynamic_blob_lock(cas_root, digest):
        identity = _validated_native_dynamic_blob_identity(
            blob, digest, source_stat.st_size
        )
        if identity is not None:
            _require_native_dynamic_source_identity(source, source_identity)
            _make_native_dynamic_readonly(blob)
            refreshed = _native_dynamic_blob_identity(blob, source_stat.st_size)
            if refreshed is None:
                raise OSError("native dynamic CAS blob disappeared while sealing")
            return (
                blob,
                digest,
                source_stat.st_size,
                refreshed,
                stat.S_IMODE(source_stat.st_mode),
            )
        blob_directory.mkdir(parents=True, exist_ok=True)
        _assert_native_dynamic_no_reparse_components(
            blob_directory, "native dynamic CAS blob directory"
        )
        temporary = blob_directory / (
            f".{blob.name}.{os.getpid()}.{threading.get_ident()}."
            f"{uuid.uuid4().hex}.tmp"
        )
        try:
            copied_digest = _copy_and_hash_file(source, temporary)
            if copied_digest != digest:
                raise OSError("source changed while publishing the CAS blob")
            _require_native_dynamic_source_identity(source, source_identity)
            try:
                shutil.copystat(source, temporary)
            except OSError:
                # Metadata is useful but content identity is the cache contract.
                pass
            try:
                existing = blob.stat()
            except FileNotFoundError:
                existing = None
            if existing is not None:
                if existing.st_nlink > 1:
                    # Upgrade recovery: detach only the corrupt CAS pathname.
                    # Legacy stage links retain their old inode and cannot poison
                    # the verified blob published below.
                    _unlink_native_dynamic_stage_path(blob)
                else:
                    _make_native_dynamic_writable(blob)
            os.replace(temporary, blob)
            _make_native_dynamic_readonly(blob)
            identity = _native_dynamic_blob_identity(blob, source_stat.st_size)
            if identity is None:
                raise OSError("native dynamic CAS blob disappeared after publication")
            return (
                blob,
                digest,
                source_stat.st_size,
                identity,
                stat.S_IMODE(source_stat.st_mode),
            )
        finally:
            try:
                temporary.unlink()
            except FileNotFoundError:
                pass
            except OSError:
                pass


def _validated_native_dynamic_blob_identity(
    blob: Path,
    digest: str,
    size: int,
) -> _FileIdentity | None:
    before = _native_dynamic_blob_identity(blob, size)
    if before is None or _sha256_file(blob) != digest:
        return None
    after = _native_dynamic_blob_identity(blob, size)
    return after if after == before else None


def _native_dynamic_blob_identity(
    blob: Path,
    size: int,
) -> _FileIdentity | None:
    try:
        current = os.lstat(blob)
        if _native_dynamic_stat_is_reparse(current) or not stat.S_ISREG(current.st_mode):
            return None
        if current.st_size != size:
            return None
        return (
            int(current.st_dev),
            int(current.st_ino),
            int(current.st_size),
            int(current.st_mtime_ns),
            int(current.st_ctime_ns),
        )
    except OSError:
        return None


def _materialize_native_dynamic_source(
    source: Path,
    destination: Path,
    cas_root: Path,
    *,
    copy_function: _CopyFunction = shutil.copy2,
) -> None:
    for _attempt in range(2):
        blob, digest, size, identity, source_mode = _ensure_native_dynamic_blob(
            source, cas_root
        )
        with _native_dynamic_blob_lock(cas_root, digest):
            if _native_dynamic_blob_identity(blob, size) != identity:
                continue
            _materialize_native_dynamic_blob(
                blob,
                destination,
                source_mode=source_mode,
                copy_function=copy_function,
            )
            return
    raise OSError("native dynamic CAS blob changed before it could be materialized")


@contextmanager
def _native_dynamic_blob_lock(cas_root: Path, digest: str) -> Iterator[None]:
    """Serialize one digest publication across threads and processes."""
    thread_lock = _CAS_THREAD_LOCKS[int(digest[:8], 16) % len(_CAS_THREAD_LOCKS)]
    lock_directory = cas_root / "locks" / digest[:2]
    lock_directory.mkdir(parents=True, exist_ok=True)
    _assert_native_dynamic_no_reparse_components(
        lock_directory, "native dynamic CAS lock directory"
    )
    lock_path = lock_directory / f"{digest[2:]}.lock"
    with thread_lock, lock_path.open("a+b") as stream:
        stream.seek(0, os.SEEK_END)
        if stream.tell() == 0:
            stream.write(b"\0")
            stream.flush()
        _lock_native_dynamic_stream(stream)
        try:
            yield
        finally:
            _unlock_native_dynamic_stream(stream)


@contextmanager
def _native_dynamic_prune_lock(cas_root: Path) -> Iterator[None]:
    lock_directory = cas_root / "locks"
    lock_directory.mkdir(parents=True, exist_ok=True)
    _assert_native_dynamic_no_reparse_components(
        lock_directory, "native dynamic CAS lock directory"
    )
    lock_path = lock_directory / "prune.lock"
    with _CAS_PRUNE_THREAD_LOCK, lock_path.open("a+b") as stream:
        stream.seek(0, os.SEEK_END)
        if stream.tell() == 0:
            stream.write(b"\0")
            stream.flush()
        _lock_native_dynamic_stream(stream)
        try:
            yield
        finally:
            _unlock_native_dynamic_stream(stream)


def _native_dynamic_cas_blobs(
    cas_root: Path,
) -> list[tuple[Path, str, int, int]]:
    blob_root = cas_root / "sha256"
    if not blob_root.is_dir():
        return []
    _assert_native_dynamic_no_reparse_components(
        blob_root, "native dynamic CAS blob root"
    )
    result: list[tuple[Path, str, int, int]] = []
    for prefix in blob_root.iterdir():
        if _native_dynamic_path_is_reparse(prefix):
            raise OSError(f"native dynamic CAS prefix cannot be a reparse point: {prefix}")
        if (
            not prefix.is_dir()
            or len(prefix.name) != 2
            or any(character not in "0123456789abcdef" for character in prefix.name)
        ):
            continue
        for blob in prefix.iterdir():
            digest = prefix.name + blob.name
            if (
                _native_dynamic_path_is_reparse(blob)
                or not blob.is_file()
                or len(digest) != 64
                or any(character not in "0123456789abcdef" for character in digest)
            ):
                continue
            current = os.lstat(blob)
            result.append((blob, digest, current.st_size, current.st_mtime_ns))
    return result


def _remove_stale_native_dynamic_temporaries(
    cas_root: Path,
) -> tuple[int, int, int]:
    blob_root = cas_root / "sha256"
    if not blob_root.is_dir():
        return 0, 0, 0
    before_bytes = 0
    after_bytes = 0
    removed = 0
    live_threads = {
        int(thread.ident)
        for thread in threading.enumerate()
        if thread.ident is not None
    }
    for prefix in blob_root.iterdir():
        if _native_dynamic_path_is_reparse(prefix):
            raise OSError(f"native dynamic CAS prefix cannot be a reparse point: {prefix}")
        if (
            not prefix.is_dir()
            or len(prefix.name) != 2
            or any(character not in "0123456789abcdef" for character in prefix.name)
        ):
            continue
        for candidate in prefix.iterdir():
            match = _CAS_TEMPORARY_NAME.fullmatch(candidate.name)
            if match is None or _native_dynamic_path_is_reparse(candidate):
                continue
            try:
                current = os.lstat(candidate)
            except FileNotFoundError:
                continue
            if not stat.S_ISREG(current.st_mode):
                continue
            before_bytes += current.st_size
            owner_pid = int(match.group("pid"))
            owner_thread_text = match.group("thread")
            owner_active = (
                owner_pid == os.getpid()
                and (
                    owner_thread_text is None
                    or int(owner_thread_text) in live_threads
                )
            ) or (owner_pid != os.getpid() and _native_dynamic_process_is_alive(owner_pid))
            if owner_active:
                after_bytes += current.st_size
                continue
            digest = prefix.name + match.group("suffix")
            expected = _native_dynamic_stat_identity(current)
            with _native_dynamic_blob_lock(cas_root, digest):
                try:
                    refreshed = os.lstat(candidate)
                except FileNotFoundError:
                    continue
                if (
                    not stat.S_ISREG(refreshed.st_mode)
                    or _native_dynamic_stat_is_reparse(refreshed)
                    or _native_dynamic_stat_identity(refreshed) != expected
                ):
                    after_bytes += refreshed.st_size
                    continue
                _make_native_dynamic_writable(candidate)
                candidate.unlink()
                removed += 1
    return before_bytes, after_bytes, removed


def _native_dynamic_process_is_alive(pid: int) -> bool:
    if pid <= 0:
        return False
    if os.name == "nt":
        import ctypes

        process_query_limited_information = 0x1000
        still_active = 259
        kernel32 = ctypes.windll.kernel32
        kernel32.OpenProcess.argtypes = [
            ctypes.c_uint32,
            ctypes.c_bool,
            ctypes.c_uint32,
        ]
        kernel32.OpenProcess.restype = ctypes.c_void_p
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle.restype = ctypes.c_bool
        kernel32.GetExitCodeProcess.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_uint32),
        ]
        kernel32.GetExitCodeProcess.restype = ctypes.c_bool
        handle = kernel32.OpenProcess(
            process_query_limited_information, False, pid
        )
        if not handle:
            return False
        try:
            exit_code = ctypes.c_uint32()
            if not kernel32.GetExitCodeProcess(handle, ctypes.byref(exit_code)):
                return False
            return int(exit_code.value) == still_active
        finally:
            kernel32.CloseHandle(handle)
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


def _lock_native_dynamic_stream(stream) -> None:
    deadline = time.monotonic() + _CAS_LOCK_TIMEOUT_SECONDS
    while True:
        try:
            stream.seek(0)
            if os.name == "nt":
                import msvcrt

                msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl

                fcntl.flock(stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            return
        except OSError as error:
            if time.monotonic() >= deadline:
                raise OSError("timed out waiting for native dynamic CAS lock") from error
            time.sleep(_CAS_LOCK_RETRY_SECONDS)


def _unlock_native_dynamic_stream(stream) -> None:
    stream.seek(0)
    if os.name == "nt":
        import msvcrt

        msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)
    else:
        import fcntl

        fcntl.flock(stream.fileno(), fcntl.LOCK_UN)


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while True:
            chunk = stream.read(_CAS_CHUNK_SIZE)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def _copy_and_hash_file(source: Path, destination: Path) -> str:
    digest = hashlib.sha256()
    with source.open("rb") as source_stream, destination.open(
        "wb"
    ) as destination_stream:
        while True:
            chunk = source_stream.read(_CAS_CHUNK_SIZE)
            if not chunk:
                break
            digest.update(chunk)
            destination_stream.write(chunk)
        destination_stream.flush()
        os.fsync(destination_stream.fileno())
    return digest.hexdigest()


def _materialize_native_dynamic_blob(
    blob: Path,
    destination: Path,
    *,
    source_mode: int,
    copy_function: _CopyFunction = shutil.copy2,
) -> None:
    """Atomically copy one verified blob into a task-owned stage path.

    A writable hardlink would expose the shared CAS inode to validation code:
    the task could make its link writable and mutate every other stage.  The
    ordinary copy deliberately trades transient stage bytes for strict
    cross-task isolation on NTFS, where block cloning is unavailable.
    """
    temporary = destination.with_name(
        f".{destination.name}.{os.getpid()}.{uuid.uuid4().hex}.tmp"
    )
    try:
        copy_function(blob, temporary)
        temporary.chmod(source_mode)
        if os.path.lexists(destination):
            _unlink_native_dynamic_stage_path(destination)
        os.replace(temporary, destination)
    finally:
        try:
            if os.path.lexists(temporary):
                _unlink_native_dynamic_stage_path(temporary)
        except FileNotFoundError:
            pass
        except OSError:
            pass


def _make_native_dynamic_readonly(path: Path) -> None:
    path.chmod(stat.S_IREAD | stat.S_IRGRP | stat.S_IROTH)


def _make_native_dynamic_writable(path: Path) -> None:
    path.chmod(stat.S_IREAD | stat.S_IWRITE)


def _remove_native_dynamic_readonly(function, path: str, _error) -> None:
    candidate = Path(path)
    if not candidate.is_dir():
        _unlink_native_dynamic_stage_path(candidate)
        return
    _make_native_dynamic_writable(candidate)
    function(path)


def _materialize_native_dynamic_tree(
    source: Path,
    destination: Path,
    cas_root: Path,
) -> None:
    if destination.exists():
        raise FileExistsError(destination)
    destination.mkdir(parents=True, exist_ok=False)
    for child in source.iterdir():
        target = destination / child.name
        if child.is_symlink():
            raise OSError(f"symbolic links are not supported in CAS trees: {child}")
        if child.is_dir():
            _materialize_native_dynamic_tree(child, target, cas_root)
        elif child.is_file():
            _materialize_native_dynamic_source(child, target, cas_root)
        else:
            raise OSError(f"unsupported native dynamic tree entry: {child}")
    try:
        shutil.copystat(source, destination)
    except OSError:
        pass


def _native_dynamic_stat_is_reparse(current: os.stat_result) -> bool:
    reparse_flag = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
    return stat.S_ISLNK(current.st_mode) or bool(
        getattr(current, "st_file_attributes", 0) & reparse_flag
    )


def _native_dynamic_path_is_reparse(path: Path) -> bool:
    try:
        return _native_dynamic_stat_is_reparse(os.lstat(path))
    except FileNotFoundError:
        return False


def _unlink_native_dynamic_stage_path(path: Path) -> None:
    try:
        path.unlink()
        return
    except PermissionError:
        current = os.lstat(path)
        if current.st_nlink > 1:
            if os.name != "nt":
                raise
            _unlink_windows_readonly_hardlink(path)
            return
        _make_native_dynamic_writable(path)
        path.unlink()


def _unlink_windows_readonly_hardlink(path: Path) -> None:
    """Delete one readonly NTFS link without changing the shared inode mode."""
    import ctypes
    from ctypes import wintypes

    class FileDispositionInfoEx(ctypes.Structure):
        _fields_ = [("flags", wintypes.DWORD)]

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    create_file = kernel32.CreateFileW
    create_file.argtypes = (
        wintypes.LPCWSTR,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.LPVOID,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.HANDLE,
    )
    create_file.restype = wintypes.HANDLE
    set_information = kernel32.SetFileInformationByHandle
    set_information.argtypes = (
        wintypes.HANDLE,
        ctypes.c_int,
        wintypes.LPVOID,
        wintypes.DWORD,
    )
    set_information.restype = wintypes.BOOL
    close_handle = kernel32.CloseHandle
    close_handle.argtypes = (wintypes.HANDLE,)
    close_handle.restype = wintypes.BOOL

    delete_access = 0x00010000
    synchronize = 0x00100000
    file_read_attributes = 0x00000080
    share_all = 0x00000001 | 0x00000002 | 0x00000004
    open_existing = 3
    open_reparse = 0x00200000
    disposition_info_ex = 21
    delete_flags = 0x00000001 | 0x00000002 | 0x00000010
    handle = create_file(
        str(path),
        delete_access | synchronize | file_read_attributes,
        share_all,
        None,
        open_existing,
        open_reparse,
        None,
    )
    invalid_handle = ctypes.c_void_p(-1).value
    if handle == invalid_handle:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        disposition = FileDispositionInfoEx(delete_flags)
        if not set_information(
            handle,
            disposition_info_ex,
            ctypes.byref(disposition),
            ctypes.sizeof(disposition),
        ):
            raise ctypes.WinError(ctypes.get_last_error())
    finally:
        close_handle(handle)


def _assert_native_dynamic_no_reparse_components(path: Path, label: str) -> None:
    candidate = path.expanduser().absolute()
    existing: list[Path] = []
    current = candidate
    while True:
        if os.path.lexists(current):
            existing.append(current)
        if current == current.parent:
            break
        current = current.parent
    for component in reversed(existing):
        if _native_dynamic_path_is_reparse(component):
            raise OSError(f"{label} cannot traverse a reparse point: {component}")


def _validate_native_dynamic_source_file(source: Path) -> os.stat_result:
    _assert_native_dynamic_no_reparse_components(source, "native dynamic source file")
    try:
        current = os.lstat(source)
    except FileNotFoundError as error:
        raise OSError(f"native dynamic source file does not exist: {source}") from error
    if _native_dynamic_stat_is_reparse(current) or not stat.S_ISREG(current.st_mode):
        raise OSError(f"native dynamic source must be a regular file: {source}")
    return current


def _native_dynamic_stat_identity(current: os.stat_result) -> _FileIdentity:
    return (
        int(current.st_dev),
        int(current.st_ino),
        int(current.st_size),
        int(current.st_mtime_ns),
        int(current.st_ctime_ns),
    )


def _require_native_dynamic_source_identity(
    source: Path, expected: _FileIdentity
) -> None:
    current = _validate_native_dynamic_source_file(source)
    if _native_dynamic_stat_identity(current) != expected:
        raise OSError("native dynamic source changed while its CAS blob was selected")


def _validate_native_dynamic_source_tree(source: Path) -> None:
    _assert_native_dynamic_no_reparse_components(source, "native dynamic source tree")
    try:
        current = os.lstat(source)
    except FileNotFoundError as error:
        raise OSError(f"native dynamic source tree does not exist: {source}") from error
    if _native_dynamic_stat_is_reparse(current) or not stat.S_ISDIR(current.st_mode):
        raise OSError(f"native dynamic source tree must be a regular directory: {source}")
    for child in source.iterdir():
        child_stat = os.lstat(child)
        if _native_dynamic_stat_is_reparse(child_stat):
            raise OSError(f"native dynamic source tree cannot contain reparse points: {child}")
        if stat.S_ISDIR(child_stat.st_mode):
            _validate_native_dynamic_source_tree(child)
        elif not stat.S_ISREG(child_stat.st_mode):
            raise OSError(f"unsupported native dynamic source entry: {child}")


def resolve_stage_child(
    stage_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"native dynamic package directory {relative_path} must be relative")
        return None
    try:
        resolved_root = stage_root.resolve()
        resolved = (resolved_root / child_path).resolve()
    except OSError as error:
        diagnostics.append(
            f"native dynamic package directory {relative_path} could not be resolved: {error}"
        )
        return None
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        diagnostics.append(
            f"native dynamic package directory {relative_path} escapes the NativeDynamic stage"
        )
        return None
    return resolved

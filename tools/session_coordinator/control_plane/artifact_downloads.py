from __future__ import annotations

import ctypes
import os
import re
import stat
from contextlib import contextmanager
from pathlib import Path
from typing import BinaryIO, Iterator
from urllib.parse import quote

from ..database import Database
from ..models import CoordinatorError
from .assets import BinaryResponse


_OPAQUE_ID = re.compile(r"^[A-Za-z0-9_-]{1,128}$")
_RANGE = re.compile(r"^bytes=(\d*)-(\d*)$")
_MAX_RANGE_BYTES = 8 * 1024 * 1024
_MAX_DIRECT_BYTES = 16 * 1024 * 1024

_GENERIC_READ = 0x80000000
_FILE_READ_ATTRIBUTES = 0x00000080
_FILE_SHARE_READ = 0x00000001
_OPEN_EXISTING = 3
_FILE_FLAG_OPEN_REPARSE_POINT = 0x00200000
_FILE_FLAG_SEQUENTIAL_SCAN = 0x08000000
_FILE_ATTRIBUTE_DIRECTORY = 0x00000010
_FILE_ATTRIBUTE_REPARSE_POINT = 0x00000400
_FILE_ATTRIBUTE_TAG_INFO = 9
_FILE_STANDARD_INFO = 1


class _FileAttributeTagInfo(ctypes.Structure):
    _fields_ = (
        ("file_attributes", ctypes.c_uint32),
        ("reparse_tag", ctypes.c_uint32),
    )


class _FileStandardInfo(ctypes.Structure):
    _fields_ = (
        ("allocation_size", ctypes.c_int64),
        ("end_of_file", ctypes.c_int64),
        ("number_of_links", ctypes.c_uint32),
        ("delete_pending", ctypes.c_ubyte),
        ("directory", ctypes.c_ubyte),
    )


class ArtifactDownloadService:
    def __init__(self, database: Database, artifact_root: Path):
        self.database = database
        self.artifact_root = artifact_root.resolve()

    def download(self, artifact_id: str, range_header: str | None) -> BinaryResponse:
        if not _OPAQUE_ID.fullmatch(artifact_id):
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT display_name, storage_path, byte_count
                   FROM workflow_artifacts WHERE artifact_id = ?""",
                (artifact_id,),
            ).fetchone()
        if row is None or not row["storage_path"]:
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        stored = Path(row["storage_path"])
        path = stored if stored.is_absolute() else self.artifact_root / stored
        try:
            expected_size = int(row["byte_count"])
        except (TypeError, ValueError) as error:
            raise CoordinatorError("artifact_not_found", "Artifact was not found") from error
        if expected_size < 0:
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        try:
            with _open_verified_artifact(path, self.artifact_root, expected_size) as (
                stream,
                size,
            ):
                try:
                    start, end, partial = self._range(size, range_header)
                except CoordinatorError as error:
                    if error.code != "invalid_range":
                        raise
                    return self._invalid_range(size)
                count = max(0, end - start + 1)
                if (partial and count > _MAX_RANGE_BYTES) or (
                    not partial and count > _MAX_DIRECT_BYTES
                ):
                    return self._invalid_range(size)
                stream.seek(start)
                body = stream.read(count)
        except CoordinatorError:
            raise
        except (OSError, ValueError) as error:
            raise CoordinatorError("artifact_not_found", "Artifact was not found") from error
        if len(body) != count:
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        safe_name = quote(str(row["display_name"]), safe="")
        headers = {
            "Content-Type": "application/octet-stream",
            "Content-Disposition": f"attachment; filename*=UTF-8''{safe_name}",
            "Accept-Ranges": "bytes",
            "Cache-Control": "no-store",
            "X-Content-Type-Options": "nosniff",
        }
        if partial:
            headers["Content-Range"] = f"bytes {start}-{end}/{size}"
        return BinaryResponse(206 if partial else 200, body, headers)

    @staticmethod
    def _invalid_range(size: int) -> BinaryResponse:
        return BinaryResponse(
            416,
            b"",
            {
                "Content-Range": f"bytes */{size}",
                "Accept-Ranges": "bytes",
                "Cache-Control": "no-store",
                "X-Content-Type-Options": "nosniff",
            },
        )

    @staticmethod
    def _range(size: int, header: str | None) -> tuple[int, int, bool]:
        if not header:
            return 0, size - 1, False
        match = _RANGE.fullmatch(header.strip())
        if not match or size == 0:
            raise CoordinatorError("invalid_range", "Artifact byte range is invalid")
        start_text, end_text = match.groups()
        if not start_text and not end_text:
            raise CoordinatorError("invalid_range", "Artifact byte range is invalid")
        if not start_text:
            suffix = int(end_text)
            if suffix <= 0:
                raise CoordinatorError("invalid_range", "Artifact byte range is invalid")
            start, end = max(0, size - suffix), size - 1
        else:
            start = int(start_text)
            end = min(int(end_text), size - 1) if end_text else size - 1
        if start >= size or start > end:
            raise CoordinatorError("invalid_range", "Artifact byte range is unsatisfiable")
        return start, end, True


@contextmanager
def _open_verified_artifact(
    path: Path, artifact_root: Path, expected_size: int
) -> Iterator[tuple[BinaryIO, int]]:
    if os.name == "nt":
        with _open_windows_artifact(path, artifact_root, expected_size) as result:
            yield result
        return
    descriptor = -1
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
        metadata = os.fstat(descriptor)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
            or metadata.st_size != expected_size
        ):
            raise OSError("artifact handle identity is invalid")
        final_path = _descriptor_path(descriptor, path)
        _require_contained(final_path, artifact_root)
        with os.fdopen(descriptor, "rb", closefd=True) as stream:
            descriptor = -1
            yield stream, int(metadata.st_size)
    finally:
        if descriptor >= 0:
            os.close(descriptor)


@contextmanager
def _open_windows_artifact(
    path: Path, artifact_root: Path, expected_size: int
) -> Iterator[tuple[BinaryIO, int]]:
    import msvcrt

    kernel32 = _kernel32()
    handle = int(
        kernel32.CreateFileW(
            str(path),
            _GENERIC_READ | _FILE_READ_ATTRIBUTES,
            _FILE_SHARE_READ,
            None,
            _OPEN_EXISTING,
            _FILE_FLAG_OPEN_REPARSE_POINT | _FILE_FLAG_SEQUENTIAL_SCAN,
            None,
        )
    )
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    descriptor = -1
    try:
        attributes = _FileAttributeTagInfo()
        _query_handle(kernel32, handle, _FILE_ATTRIBUTE_TAG_INFO, attributes)
        if attributes.file_attributes & (
            _FILE_ATTRIBUTE_DIRECTORY | _FILE_ATTRIBUTE_REPARSE_POINT
        ):
            raise OSError("artifact handle is not a plain file")
        standard = _FileStandardInfo()
        _query_handle(kernel32, handle, _FILE_STANDARD_INFO, standard)
        size = int(standard.end_of_file)
        if standard.directory or standard.number_of_links != 1 or size != expected_size:
            raise OSError("artifact handle identity is invalid")
        _require_contained(_final_windows_path(kernel32, handle), artifact_root)
        descriptor = msvcrt.open_osfhandle(handle, os.O_RDONLY)
        handle = 0
        with os.fdopen(descriptor, "rb", closefd=True) as stream:
            descriptor = -1
            yield stream, size
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        elif handle:
            kernel32.CloseHandle(handle)


def _query_handle(kernel32, handle: int, information_class: int, value: ctypes.Structure) -> None:
    if not kernel32.GetFileInformationByHandleEx(
        handle,
        information_class,
        ctypes.byref(value),
        ctypes.sizeof(value),
    ):
        raise ctypes.WinError(ctypes.get_last_error())


def _final_windows_path(kernel32, handle: int) -> Path:
    required = kernel32.GetFinalPathNameByHandleW(handle, None, 0, 0)
    if not required:
        raise ctypes.WinError(ctypes.get_last_error())
    buffer = ctypes.create_unicode_buffer(required + 1)
    written = kernel32.GetFinalPathNameByHandleW(handle, buffer, len(buffer), 0)
    if not written or written >= len(buffer):
        raise ctypes.WinError(ctypes.get_last_error())
    value = buffer.value
    if value.startswith("\\\\?\\UNC\\"):
        value = "\\\\" + value[8:]
    elif value.startswith("\\\\?\\"):
        value = value[4:]
    return Path(value)


def _descriptor_path(descriptor: int, fallback: Path) -> Path:
    proc_path = Path(f"/proc/self/fd/{descriptor}")
    return proc_path.resolve() if proc_path.exists() else fallback.resolve()


def _require_contained(path: Path, artifact_root: Path) -> None:
    normalized_path = os.path.normcase(os.path.abspath(path))
    normalized_root = os.path.normcase(os.path.abspath(artifact_root))
    try:
        contained = os.path.commonpath((normalized_path, normalized_root)) == normalized_root
    except ValueError:
        contained = False
    if not contained:
        raise OSError("artifact handle escaped its managed root")


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
    kernel32.GetFinalPathNameByHandleW.argtypes = (
        ctypes.c_void_p,
        ctypes.c_wchar_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
    )
    kernel32.GetFinalPathNameByHandleW.restype = ctypes.c_uint32
    kernel32.CloseHandle.argtypes = (ctypes.c_void_p,)
    kernel32.CloseHandle.restype = ctypes.c_int
    return kernel32

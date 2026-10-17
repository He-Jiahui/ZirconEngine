"""Jenkins-only scoped source transitions with locked parents and file handles."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import hashlib
import os
from pathlib import Path

from .guarded_delete import locked_parents, remove_verified
from .paths import _opened_identity, file_fact, physical


def write_verified(path: Path, expected: dict | None, content: bytes | None) -> dict:
    physical(path, exists=False)
    if content is None:
        if expected:
            return remove_verified(expected)
        return {"path": str(path), "status": "absent"}
    if expected is None and path.exists():
        raise ValueError("source_path_appeared")
    if expected is not None and file_fact(path) != expected:
        raise ValueError("source_preimage_changed")
    path.parent.mkdir(parents=True, exist_ok=True)
    physical(path.parent)
    with locked_parents(path) as api:
        import msvcrt
        disposition = 3 if expected else 1  # OPEN_EXISTING or CREATE_NEW, never truncate by pathname.
        handle = api.CreateFileW("\\\\?\\" + str(path), 0xC0000000, 1, None, disposition, 0x00200000, None)
        if handle == wintypes.HANDLE(-1).value:
            raise ctypes.WinError(ctypes.get_last_error())
        descriptor = None
        try:
            attributes = (wintypes.DWORD * 2)()
            if not api.GetFileInformationByHandleEx(handle, 9, attributes, ctypes.sizeof(attributes)):
                raise ctypes.WinError(ctypes.get_last_error())
            if attributes[0] & (0x400 | 0x10):
                raise ValueError("source_alias_or_type_changed")
            descriptor = msvcrt.open_osfhandle(handle, os.O_RDWR | os.O_BINARY)
            opened = os.fstat(descriptor)
            if opened.st_nlink != 1:
                raise ValueError("hardlinked_source_write_requires_review")
            if expected:
                if _opened_identity(opened) != [expected["identity"][i] for i in (0, 1, 2, 3, 5)]:
                    raise ValueError("source_physical_identity_changed")
                checksum = hashlib.sha256()
                while block := os.read(descriptor, 1024 * 1024):
                    checksum.update(block)
                if checksum.hexdigest() != expected["sha256"]:
                    raise ValueError("source_preimage_hash_changed")
            os.lseek(descriptor, 0, os.SEEK_SET)
            remaining = memoryview(content)
            while remaining:
                count = os.write(descriptor, remaining)
                if count <= 0:
                    raise OSError("source_write_made_no_progress")
                remaining = remaining[count:]
            os.ftruncate(descriptor, len(content))
            os.fsync(descriptor)
        finally:
            if descriptor is not None:
                os.close(descriptor)
            else:
                api.CloseHandle(handle)
    after = file_fact(path)
    if after["sha256"] != hashlib.sha256(content).hexdigest():
        raise ValueError("source_changed_after_managed_write")
    return {"status": "written", **after}

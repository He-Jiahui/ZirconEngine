"""Delete the exact verified Windows file handle while ancestors stay locked."""
from __future__ import annotations

from contextlib import contextmanager
import ctypes
from ctypes import wintypes
import hashlib
import os
from pathlib import Path

from .paths import _opened_identity, physical


@contextmanager
def locked_parents(path: Path, *, expected_directory_identities=None):
    if os.name != "nt":
        raise ValueError("guarded_cleanup_requires_windows")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p,
                               wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
    api.CreateFileW.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    api.GetFileInformationByHandleEx.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    api.GetFileInformationByHandleEx.restype = wintypes.BOOL
    handles = []
    try:
        for parent in reversed(path.parents):
            # FILE_LIST_DIRECTORY participates in Windows share-access checks;
            # FILE_READ_ATTRIBUTES alone does not reserve sharing rights.
            handle = api.CreateFileW("\\\\?\\" + str(parent), 0x81, 1, None, 3, 0x02200000, None)
            if handle == wintypes.HANDLE(-1).value:
                raise ctypes.WinError(ctypes.get_last_error())
            handles.append(handle)
            # FileAttributeTagInfo: FILE_ATTRIBUTE_REPARSE_POINT is rejected on the locked handle.
            info = (wintypes.DWORD * 2)()
            if not api.GetFileInformationByHandleEx(handle, 9, info, ctypes.sizeof(info)):
                raise ctypes.WinError(ctypes.get_last_error())
            if info[0] & 0x400 or not info[0] & 0x10:
                raise ValueError("cleanup_ancestor_alias_or_type_changed")
            if expected_directory_identities is not None and str(parent) in expected_directory_identities:
                class FileIdInfo(ctypes.Structure):
                    _fields_ = [("volume", ctypes.c_ulonglong), ("file_id", ctypes.c_ubyte * 16)]
                identity = FileIdInfo()
                if not api.GetFileInformationByHandleEx(handle, 18, ctypes.byref(identity), ctypes.sizeof(identity)):
                    raise ctypes.WinError(ctypes.get_last_error())
                if [identity.volume, int.from_bytes(bytes(identity.file_id), "little")] != expected_directory_identities[str(parent)]:
                    raise ValueError("cleanup_ancestor_physical_identity_changed")
        yield api
    finally:
        for handle in reversed(handles):
            api.CloseHandle(handle)


def remove_verified(fact: dict, *, remove=True, owned_readonly_fixture=False, expected_directory_identities=None) -> dict:
    path = physical(fact["path"])
    if fact["identity"][-1] != 1:
        raise ValueError("hardlinked_cleanup_file_requires_review")
    with locked_parents(path, expected_directory_identities=expected_directory_identities) as api:
        import msvcrt
        access = 0x80000000 | (0x10000 if remove else 0)
        handle = api.CreateFileW("\\\\?\\" + str(path), access, 1, None, 3, 0x02200000, None)
        if handle == wintypes.HANDLE(-1).value:
            raise ctypes.WinError(ctypes.get_last_error())
        descriptor = None
        try:
            attributes = (wintypes.DWORD * 2)()
            if not api.GetFileInformationByHandleEx(handle, 9, attributes, ctypes.sizeof(attributes)):
                raise ctypes.WinError(ctypes.get_last_error())
            if attributes[0] & (0x400 | 0x10):
                raise ValueError("cleanup_file_alias_or_type_changed")
            descriptor = msvcrt.open_osfhandle(handle, os.O_RDONLY | os.O_BINARY)
            expected = [fact["identity"][i] for i in (0, 1, 2, 3, 5)]
            if _opened_identity(os.fstat(descriptor)) != expected:
                raise ValueError("cleanup_physical_identity_changed")
            class StandardInfo(ctypes.Structure):
                _fields_ = [("allocation", ctypes.c_longlong), ("end", ctypes.c_longlong),
                            ("links", wintypes.DWORD), ("pending", ctypes.c_ubyte), ("directory", ctypes.c_ubyte)]
            standard = StandardInfo()
            if not api.GetFileInformationByHandleEx(handle, 1, ctypes.byref(standard), ctypes.sizeof(standard)):
                raise ctypes.WinError(ctypes.get_last_error())
            checksum = hashlib.sha256()
            while data := os.read(descriptor, 1024 * 1024):
                checksum.update(data)
            if checksum.hexdigest() != fact["sha256"] or _opened_identity(os.fstat(descriptor)) != expected:
                raise ValueError("cleanup_content_changed")
            if remove:
                api.SetFileInformationByHandle.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
                api.SetFileInformationByHandle.restype = wintypes.BOOL
                # Only exact native-owned generated fixtures may ignore Git's readonly attribute.
                disposition = wintypes.DWORD(0x11) if owned_readonly_fixture else ctypes.c_ubyte(1)
                info_class = 21 if owned_readonly_fixture else 4
                if not api.SetFileInformationByHandle(handle, info_class, ctypes.byref(disposition), ctypes.sizeof(disposition)):
                    raise ctypes.WinError(ctypes.get_last_error())
            return {"path": str(path), "status": "deleted" if remove else "verified", "logicalBytes": fact["bytes"],
                    "allocationBytes": standard.allocation, "sha256": fact["sha256"], "identity": fact["identity"]}
        finally:
            if descriptor is not None:
                os.close(descriptor)
            else:
                api.CloseHandle(handle)


def remove_empty(path: Path, *, expected_directory_identity=None) -> bool:
    path = physical(path)
    with locked_parents(path) as api:
        handle = api.CreateFileW("\\\\?\\" + str(path), 0x10080, 3, None, 3, 0x02200000, None)
        if handle == wintypes.HANDLE(-1).value:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            attributes = (wintypes.DWORD * 2)()
            if not api.GetFileInformationByHandleEx(handle, 9, attributes, ctypes.sizeof(attributes)):
                raise ctypes.WinError(ctypes.get_last_error())
            if attributes[0] & 0x400 or not attributes[0] & 0x10:
                raise ValueError("cleanup_directory_alias_or_type_changed")
            if expected_directory_identity is not None:
                class FileIdInfo(ctypes.Structure):
                    _fields_ = [("volume", ctypes.c_ulonglong), ("file_id", ctypes.c_ubyte * 16)]
                identity = FileIdInfo()
                if not api.GetFileInformationByHandleEx(handle, 18, ctypes.byref(identity), ctypes.sizeof(identity)):
                    raise ctypes.WinError(ctypes.get_last_error())
                if [identity.volume, int.from_bytes(bytes(identity.file_id), "little")] != expected_directory_identity:
                    raise ValueError("cleanup_directory_physical_identity_changed")
            disposition = ctypes.c_ubyte(1)
            api.SetFileInformationByHandle.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
            api.SetFileInformationByHandle.restype = wintypes.BOOL
            if api.SetFileInformationByHandle(handle, 4, ctypes.byref(disposition), 1):
                return True
            if ctypes.get_last_error() == 145:
                return False
            raise ctypes.WinError(ctypes.get_last_error())
        finally:
            api.CloseHandle(handle)

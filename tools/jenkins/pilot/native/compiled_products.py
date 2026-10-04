"""Native complete-link ownership proofs for Cargo generation build products.

This dedicated inventory policy never relaxes ordinary source/state/receipt
storage: every OS-enumerated physical alias must remain inside this exact
generation's build subtree and match the full pinned volume/file identity.
"""
from __future__ import annotations
import ctypes
import ntpath
from contextlib import ExitStack
from .windows_storage import (
    WindowsPathError, WindowsIdentityMismatch, _raise_last_error, _relative_parts,
    _attributes, identity_from_handle, _close_handle,
    _FILE_ATTRIBUTE_DIRECTORY, _FILE_ATTRIBUTE_REPARSE_POINT,
)

def _hardlink_names(path: str) -> list[str]:
    """Enumerate every NTFS volume-relative link name; never infer from a walk."""
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    first, next_name, close = kernel32.FindFirstFileNameW, kernel32.FindNextFileNameW, kernel32.FindClose
    first.argtypes = [ctypes.c_wchar_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_ulong), ctypes.c_wchar_p]
    first.restype = ctypes.c_void_p
    next_name.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ulong), ctypes.c_wchar_p]
    next_name.restype = ctypes.c_int
    close.argtypes = [ctypes.c_void_p]
    close.restype = ctypes.c_int
    size = ctypes.c_ulong(32768)
    buffer = ctypes.create_unicode_buffer(size.value)
    handle = first(path, 0, ctypes.byref(size), buffer)
    if handle in {None, ctypes.c_void_p(-1).value}:
        _raise_last_error("FindFirstFileNameW")
    names = [buffer.value]
    try:
        while True:
            size.value = len(buffer)
            if not next_name(handle, ctypes.byref(size), buffer):
                if ctypes.get_last_error() == 38:  # ERROR_HANDLE_EOF
                    break
                _raise_last_error("FindNextFileNameW")
            names.append(buffer.value)
            if len(names) > 1024:
                raise WindowsPathError("compiled product has too many link names to prove ownership")
    finally:
        if not close(handle):
            _raise_last_error("FindClose")
    return names


def compiled_inventory(self, relative_directory: str, *, allowed_directories: tuple[str, ...]) -> tuple[list[str], list[str]]:
    """Audit Cargo build products, proving all native hardlink names stay owned.

    This policy applies solely to the generation's build subtree. Every
    hardlink name comes from the OS enumeration, is opened relative to the
    pinned generation root, and must expose the same full volume/file ID.
    Retained leaf and parent handles protect all aliases through the audit.
    """
    if allowed_directories != ("build",) or relative_directory != "build":
        raise WindowsPathError("compiled hardlink policy is restricted to generation build products")
    parts = _relative_parts(relative_directory)
    root_tail = ntpath.splitdrive(self.root_path)[1].rstrip("\\")
    with self._lock, ExitStack() as pins:
        self.verify_root()
        # Close drive-mapping aliases before resolving volume-relative names.
        physical = self.root_device_path
        observed = []
        def check(relative, handle):
            attributes = _attributes(handle).FileAttributes
            if attributes & (_FILE_ATTRIBUTE_DIRECTORY | _FILE_ATTRIBUTE_REPARSE_POINT):
                raise WindowsPathError("compiled product is not a plain regular file")
            identity = identity_from_handle(handle)
            if identity.link_count <= 1:
                return
            path = ntpath.join(self.root_path, relative.replace("/", "\\"))
            names = _hardlink_names(path)
            if len(names) != identity.link_count or len({name.casefold() for name in names}) != len(names):
                raise WindowsIdentityMismatch("compiled hardlink enumeration is incomplete or changed")
            for name in names:
                prefix = root_tail + "\\"
                if not name.casefold().startswith(prefix.casefold()):
                    raise WindowsPathError("compiled product has a hardlink outside its exact generation")
                alias = name[len(prefix):].replace("\\", "/")
                alias_parts = _relative_parts(alias)
                if alias_parts[0].casefold() != "build":
                    raise WindowsPathError("compiled product aliases noncompiled generation state")
                parent = pins.enter_context(self._pinned_parent(alias_parts))
                alias_handle = self._open_leaf(parent, alias_parts[-1], "rb")
                pins.callback(_close_handle, alias_handle)
                alias_attributes = _attributes(alias_handle).FileAttributes
                alias_identity = identity_from_handle(alias_handle)
                if alias_attributes & (_FILE_ATTRIBUTE_DIRECTORY | _FILE_ATTRIBUTE_REPARSE_POINT) or not identity.same_file_state(alias_identity):
                    raise WindowsIdentityMismatch("compiled hardlink name does not identify the exact pinned file")
                observed.append((alias_handle, alias_identity, path, tuple(sorted(names))))
            if not identity.same_file_state(identity_from_handle(handle)) or sorted(_hardlink_names(path)) != sorted(names):
                raise WindowsIdentityMismatch("compiled hardlink identity or names changed during audit")
        walk_parts, handles = self._directory_relative("/".join(parts))
        for handle in handles:
            pins.callback(_close_handle, handle)
        files, directories = [], []
        self._walk_directory(handles[-1], "/".join(walk_parts), files, directories, check)
        for handle, identity, path, names in observed:
            if not identity.same_file_state(identity_from_handle(handle)) or tuple(sorted(_hardlink_names(path))) != names:
                raise WindowsIdentityMismatch("compiled hardlinks changed before inventory publication")
        if self.root_device_path != physical:
            raise WindowsIdentityMismatch("compiled storage physical identity changed")
        return sorted(files), sorted(directories, key=lambda item: (item.count("/"), item), reverse=True)

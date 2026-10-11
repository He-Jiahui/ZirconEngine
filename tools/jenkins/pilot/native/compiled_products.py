"""Native complete-link ownership proofs for Cargo generation build products.

This dedicated inventory policy never relaxes ordinary source/state/receipt
storage: every OS-enumerated physical alias must remain inside this exact
generation's declared compiler output subtrees and match the pinned file identity.
"""
from __future__ import annotations
import ctypes
import hashlib
import ntpath
from contextlib import ExitStack
from .windows_storage import (
    WindowsPathError, WindowsIdentityMismatch, _raise_last_error, _relative_parts,
    _attributes, identity_from_handle, identity_from_stream, _close_handle, _final_nt_path,
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


def compiled_file_fact(
    self, relative: str, *, allowed_directories: tuple[str, ...]
) -> tuple[str, int]:
    """Hash one Cargo executable while proving every native alias is owned.

    Cargo may hard-link an executable from the caller's target into ``build``.
    The caller declares the exact target leaf and its distinct ``build`` sibling;
    this path admits only those two direct generation roots.  Every
    OS-enumerated name is opened through pinned parents and retained until the
    digest, identity, and link-name checks complete.  The returned tuple is
    deliberately limited to the existing artifact fact fields: SHA-256 and
    byte length.
    """
    if (not isinstance(allowed_directories, tuple) or len(allowed_directories) != 2
            or not isinstance(allowed_directories[0], str) or allowed_directories[1] != "build"):
        raise WindowsPathError("compiled artifact requires an exact target leaf and build sibling")
    target_parts = _relative_parts(allowed_directories[0])
    if len(target_parts) != 1 or target_parts[0].casefold() == "build":
        raise WindowsPathError("compiled artifact target must be a distinct direct generation directory")
    allowed_roots = {target_parts[0].casefold(), "build"}
    parts = _relative_parts(relative)
    if len(parts) < 2 or parts[0].casefold() not in allowed_roots:
        raise WindowsPathError("compiled artifact must be below the exact compiler roots")
    root_tail = ntpath.splitdrive(self.root_path)[1].rstrip("\\")
    with self._lock, ExitStack() as pins:
        self.verify_root()
        physical = self.root_device_path

        def regular_identity(handle):
            attributes = _attributes(handle).FileAttributes
            if attributes & (_FILE_ATTRIBUTE_DIRECTORY | _FILE_ATTRIBUTE_REPARSE_POINT):
                raise WindowsPathError("compiled artifact is not a plain regular file")
            return identity_from_handle(handle)

        parent = pins.enter_context(self._pinned_parent(parts))
        handle = self._open_leaf(parent, parts[-1], "rb")
        pins.callback(_close_handle, handle)
        identity = regular_identity(handle)
        file_nt_path = _final_nt_path(handle)
        expected_nt_path = ntpath.join(physical, *parts)
        if ntpath.normcase(file_nt_path) != ntpath.normcase(expected_nt_path):
            raise WindowsIdentityMismatch("compiled artifact path differs from its pinned physical path")
        # This path comes only from the checked leaf handle. DOS remapping cannot
        # redirect native link enumeration to another volume or namespace.
        path = "\\\\?\\GLOBALROOT" + file_nt_path
        aliases = []
        names = []
        if identity.link_count > 1:
            names = _hardlink_names(path)
            if len(names) != identity.link_count or len({name.casefold() for name in names}) != len(names):
                raise WindowsIdentityMismatch("compiled artifact hardlink enumeration is incomplete or changed")
            prefix = root_tail + "\\"
            for name in names:
                if not name.casefold().startswith(prefix.casefold()):
                    raise WindowsPathError("compiled artifact has a hardlink outside its exact generation")
                alias = name[len(prefix):].replace("\\", "/")
                alias_parts = _relative_parts(alias)
                if len(alias_parts) < 2 or alias_parts[0].casefold() not in allowed_roots:
                    raise WindowsPathError("compiled artifact has a hardlink outside the exact compiler roots")
                alias_parent = pins.enter_context(self._pinned_parent(alias_parts))
                alias_handle = self._open_leaf(alias_parent, alias_parts[-1], "rb")
                pins.callback(_close_handle, alias_handle)
                alias_identity = regular_identity(alias_handle)
                if not identity.same_file_state(alias_identity):
                    raise WindowsIdentityMismatch("compiled artifact hardlink does not identify the exact file")
                aliases.append((alias_handle, alias_identity))

        hash_parent = pins.enter_context(self._pinned_parent(parts))
        hash_handle = self._open_leaf(hash_parent, parts[-1], "rb")
        stream = pins.enter_context(self._file_stream(hash_handle, "rb"))
        hash_before = identity_from_stream(stream)
        if not identity.same_file_state(hash_before):
            raise WindowsIdentityMismatch("compiled artifact changed before hashing")
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
        hash_after = identity_from_stream(stream)
        if not hash_before.same_file_state(hash_after):
            raise WindowsIdentityMismatch("compiled artifact changed while hashing")

        if not identity.same_file_state(identity_from_handle(handle)):
            raise WindowsIdentityMismatch("compiled artifact identity changed during audit")
        for alias_handle, alias_identity in aliases:
            if not alias_identity.same_file_state(identity_from_handle(alias_handle)):
                raise WindowsIdentityMismatch("compiled artifact hardlink changed during audit")
        if names and sorted(_hardlink_names(path)) != sorted(names):
            raise WindowsIdentityMismatch("compiled artifact hardlink names changed during audit")
        if self.root_device_path != physical:
            raise WindowsIdentityMismatch("compiled artifact storage physical identity changed")
        return digest, hash_after.size


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

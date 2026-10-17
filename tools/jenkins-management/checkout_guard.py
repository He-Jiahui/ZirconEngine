"""Native read guards for Jenkins validation of an existing checkout.

No source, state, ACL, Git or build writes occur here. The caller owns durable
admission and must keep this guard through its native descendant/EOF proof.
File pins deny data writes and deletion; directory notifications also reject
transient namespace changes. A final hash comparison alone is not a guard.
"""
from __future__ import annotations

from collections.abc import Mapping
import copy
import ctypes
from ctypes import wintypes
import hashlib
import json
import ntpath
import os
from pathlib import Path
import re
import struct
import threading
import time


class CheckoutGuardBusy(ValueError):
    code = "checkout_source_busy"
    retryable = True

    def __init__(self, message, *, details=None):
        super().__init__(message)
        self.details = dict(details or {})


class CheckoutGuardChanged(ValueError):
    code = "checkout_input_changed"
    retryable = False

    def __init__(self, message, *, details=None):
        super().__init__(message)
        self.details = dict(details or {})


def _encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def _digest(value):
    return hashlib.sha256(_encoded(value)).hexdigest()


def _absolute(value, *, allow_drive_root=False):
    raw = os.fspath(value).replace("/", "\\")
    drive, tail = ntpath.splitdrive(raw)
    if (not re.fullmatch(r"[A-Za-z]:", drive) or not tail.startswith("\\")
            or raw.startswith("\\\\") or ntpath.normpath(raw) != raw
            or (not (allow_drive_root and tail == "\\")
                and any(part in {"", ".", ".."} or ":" in part
                        for part in tail[1:].split("\\")))):
        raise ValueError("Checkout inputs require literal absolute local paths")
    return drive.upper() + tail


def _key(path):
    return ntpath.normcase(path)


def _under(path, root):
    return _key(path) == _key(root) or _key(path).startswith(_key(root).rstrip("\\") + "\\")


def _relative(value):
    if (not isinstance(value, str) or not value or "\\" in value or ":" in value
            or value.startswith("/") or any(p in {"", ".", ".."} for p in value.split("/"))):
        raise ValueError("Ignored activity requires an explicit relative subtree")
    return value


def parse_notifications(raw):
    """Parse one completed FILE_NOTIFY_INFORMATION buffer; lost events fail."""
    if not isinstance(raw, (bytes, bytearray, memoryview)) or not raw:
        raise CheckoutGuardChanged("Directory monitoring lost activity records",
                                   details={"reason": "watch_overflow"})
    result, offset = [], 0
    while True:
        if offset + 12 > len(raw):
            raise CheckoutGuardChanged("Invalid directory notification record",
                                       details={"reason": "watch_invalid_notification"})
        next_offset, action, length = struct.unpack_from("<III", raw, offset)
        end = offset + 12 + length
        if action not in {1, 2, 3, 4, 5} or not length or length % 2 or end > len(raw):
            raise CheckoutGuardChanged("Invalid directory notification bounds",
                                       details={"reason": "watch_invalid_notification"})
        try:
            name = bytes(raw[offset + 12:end]).decode("utf-16-le", "strict").replace("\\", "/")
            _relative(name)
        except (UnicodeError, ValueError) as error:
            raise CheckoutGuardChanged("Invalid directory notification name",
                                       details={"reason": "watch_invalid_notification"}) from error
        result.append({"path": name, "action": action})
        if not next_offset:
            break
        if next_offset % 4 or next_offset < 12 + length or offset + next_offset >= len(raw):
            raise CheckoutGuardChanged("Invalid directory notification offset",
                                       details={"reason": "watch_invalid_notification"})
        offset += next_offset
    return result


class _Overlapped(ctypes.Structure):
    _fields_ = [("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
                ("Offset", wintypes.DWORD), ("OffsetHigh", wintypes.DWORD),
                ("hEvent", wintypes.HANDLE)]


def _notification_api():
    if os.name != "nt":
        raise CheckoutGuardChanged("Native checkout read guards require Windows",
                                   details={"reason": "native_api_unavailable"})
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateEventW.argtypes = [ctypes.c_void_p, wintypes.BOOL, wintypes.BOOL, wintypes.LPCWSTR]
    api.CreateEventW.restype = wintypes.HANDLE
    api.ResetEvent.argtypes = [wintypes.HANDLE]
    api.ResetEvent.restype = wintypes.BOOL
    api.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    api.WaitForSingleObject.restype = wintypes.DWORD
    api.ReadDirectoryChangesW.argtypes = [wintypes.HANDLE, ctypes.c_void_p, wintypes.DWORD,
        wintypes.BOOL, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD),
        ctypes.POINTER(_Overlapped), ctypes.c_void_p]
    api.ReadDirectoryChangesW.restype = wintypes.BOOL
    api.GetOverlappedResult.argtypes = [wintypes.HANDLE, ctypes.POINTER(_Overlapped),
                                      ctypes.POINTER(wintypes.DWORD), wintypes.BOOL]
    api.GetOverlappedResult.restype = wintypes.BOOL
    api.CancelIoEx.argtypes = [wintypes.HANDLE, ctypes.POINTER(_Overlapped)]
    api.CancelIoEx.restype = wintypes.BOOL
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    return api


def _native_error(error, path):
    number = getattr(error, "winerror", None) or getattr(error, "errno", None)
    details = {"path": path, "nativeError": number}
    if number in {5, 32, 33}:
        return CheckoutGuardBusy("An input cannot acquire a read pin", details=details)
    return CheckoutGuardChanged("Cannot pin the exact physical input",
                                details={**details, "reason": "input_pin_failed"})


def _identity(identity, *, path, checksum=None, attributes=None):
    return {"path": path, "device": identity.device, "fileId": identity.file_id.hex(),
            "size": identity.size, "mtimeNs": identity.mtime_ns,
            "creationTime": identity.creation_time, "linkCount": identity.link_count,
            "sha256": checksum, "attributes": attributes}


def _regular_input_identity(native, handle, *, trusted_git=False):
    if not trusted_git:
        return native._ensure_regular_file_handle(handle)
    attributes = int(native._attributes(handle).FileAttributes)
    if attributes & (native._FILE_ATTRIBUTE_REPARSE_POINT | native._FILE_ATTRIBUTE_DIRECTORY):
        raise CheckoutGuardChanged("Trusted Git input is not a plain physical file",
                                   details={"reason": "input_alias"})
    return native.identity_from_handle(handle)


_checksum_read_buffers = threading.local()


def _read_input_checksum(native, handle):
    api, _ntdll, _msvcrt = native._apis()
    buffer = getattr(_checksum_read_buffers, "buffer", None)
    if buffer is None:
        buffer = _checksum_read_buffers.buffer = ctypes.create_string_buffer(1024 * 1024)
    checksum = hashlib.sha256()
    while True:
        count = wintypes.DWORD()
        if not api.ReadFile(handle, buffer, len(buffer), ctypes.byref(count), None):
            raise ctypes.WinError(ctypes.get_last_error())
        if not count.value:
            return checksum.hexdigest()
        checksum.update(memoryview(buffer)[:count.value])


def _verify_trusted_git_identity(native, handle, identity, expected, path):
    actual = {"kind": "installed-git", **_identity(identity, path=path,
        checksum=expected["sha256"], attributes=int(native._attributes(handle).FileAttributes))}
    if actual != expected:
        raise CheckoutGuardChanged("The exact trusted Git input identity changed",
            details={"reason": "input_identity", "path": path,
                     "expected": expected, "actual": actual})


class _PinnedHierarchy:
    """Reuse verified relative native opens, without writable-root authority."""
    def __init__(self):
        from tools.jenkins.pilot.native import windows_storage
        self.native = windows_storage
        self.directories, self.files, self.drives = {}, {}, {}
        self.order = []

    def directory(self, path):
        key = _key(path)
        if key in self.directories:
            return self.directories[key][0]
        drive, tail = ntpath.splitdrive(path)
        if tail == "\\":
            from tools.jenkins.pilot.native.paths import _verify_local_drive_mapping
            expected = _verify_local_drive_mapping(drive)
            handle = self.native._open_volume_root(drive)
            try:
                identity = self.native._ensure_directory_handle(handle)
                actual = ntpath.normcase(ntpath.normpath(self.native._final_nt_path(handle)))
                if actual != ntpath.normcase(ntpath.normpath(expected + "\\")):
                    raise CheckoutGuardChanged("Checkout volume identity differs",
                                               details={"reason": "input_alias", "path": path})
            except BaseException:
                self.native._close_handle(handle)
                raise
            self.drives[drive] = expected
        else:
            parent = self.directory(ntpath.dirname(path))
            handle = self.native._open_relative_handle(parent, ntpath.basename(path),
                access=self.native._DIRECTORY_READ_ACCESS,
                share=self.native._FILE_SHARE_READ | self.native._FILE_SHARE_WRITE,
                options=(self.native._FILE_DIRECTORY_FILE | self.native._FILE_OPEN_REPARSE_POINT
                         | self.native._FILE_SYNCHRONOUS_IO_NONALERT))
            try:
                identity = self.native._ensure_directory_handle(handle)
            except BaseException:
                self.native._close_handle(handle)
                raise
        self.directories[key] = (handle, identity, path)
        self.order.append(handle)
        return handle

    def file(self, path, expected, *, trusted_native_input=None):
        try:
            parent = self.directory(ntpath.dirname(path))
            handle = self.native._open_relative_handle(parent, ntpath.basename(path),
                access=(self.native._FILE_READ_DATA | self.native._FILE_READ_ATTRIBUTES
                        | self.native._SYNCHRONIZE), share=self.native._FILE_SHARE_READ)
        except OSError as error:
            number = getattr(error, "winerror", None) or getattr(error, "errno", None)
            if expected is None and number in {2, 3}:
                return None
            raise _native_error(error, path) from error
        self.order.append(handle)
        before = _regular_input_identity(self.native, handle,
                                         trusted_git=trusted_native_input is not None)
        if expected is None:
            raise CheckoutGuardChanged("A declared absent input exists",
                                       details={"reason": "input_presence", "path": path})
        if trusted_native_input is not None:
            _verify_trusted_git_identity(self.native, handle, before, trusted_native_input, path)
        checksum = _read_input_checksum(self.native, handle)
        after = _regular_input_identity(self.native, handle,
                                        trusted_git=trusted_native_input is not None)
        if not before.same_file_state(after) or checksum != expected:
            raise CheckoutGuardChanged("Pinned input differs from its declared bytes",
                details={"reason": "input_hash", "path": path, "expected": expected,
                         "actual": checksum})
        if trusted_native_input is not None:
            _verify_trusted_git_identity(self.native, handle, after, trusted_native_input, path)
        attributes = int(self.native._attributes(handle).FileAttributes)
        self.files[_key(path)] = (handle, after, attributes, path, expected, trusted_native_input)
        result = _identity(after, path=path, checksum=expected, attributes=attributes)
        return {**result, "kind": "installed-git"} if trusted_native_input is not None else result

    def verify(self):
        from tools.jenkins.pilot.native.paths import _verify_local_drive_mapping
        for drive, expected in self.drives.items():
            if _verify_local_drive_mapping(drive) != expected:
                raise CheckoutGuardChanged("An input drive mapping changed",
                                           details={"reason": "input_alias", "drive": drive})
        for handle, before, path in self.directories.values():
            if not before.same_object(self.native._ensure_directory_handle(handle)):
                raise CheckoutGuardChanged("A pinned directory identity changed",
                                           details={"reason": "input_identity", "path": path})
        for handle, before, attributes, path, _expected, trusted in self.files.values():
            after = _regular_input_identity(self.native, handle, trusted_git=trusted is not None)
            if trusted is not None:
                _verify_trusted_git_identity(self.native, handle, after, trusted, path)
            if (not before.same_file_state(after) or before.creation_time != after.creation_time
                    or attributes != int(self.native._attributes(handle).FileAttributes)):
                raise CheckoutGuardChanged("A held input identity or attributes changed",
                                           details={"reason": "input_identity", "path": path})

    def close(self):
        while self.order:
            # Drop each released handle immediately so a failed later close
            # cannot make a retry close a reused or already released handle.
            self.native._close_handle(self.order[-1])
            self.order.pop()
        self.files.clear()
        self.directories.clear()


def capture_trusted_git_input():
    """Capture the internally bound installed Git; never trust a request field."""
    from tools.jenkins.pilot.trusted_tools import git_identity
    binding = git_identity()
    path = _absolute(binding["path"])
    expected = binding["sha256"]
    if (ntpath.basename(path).casefold() != "git.exe" or not isinstance(expected, str)
            or not re.fullmatch(r"[0-9a-f]{64}", expected)):
        raise ValueError("Installed Git requires an exact executable path and digest")
    hierarchy = _PinnedHierarchy()
    try:
        parent = hierarchy.directory(ntpath.dirname(path))
        native = hierarchy.native
        handle = native._open_relative_handle(parent, ntpath.basename(path),
            access=(native._FILE_READ_DATA | native._FILE_READ_ATTRIBUTES | native._SYNCHRONIZE),
            share=native._FILE_SHARE_READ)
        hierarchy.order.append(handle)
        before = _regular_input_identity(native, handle, trusted_git=True)
        attributes = int(native._attributes(handle).FileAttributes)
        checksum = _read_input_checksum(native, handle)
        after = _regular_input_identity(native, handle, trusted_git=True)
        if (not before.same_file_state(after) or before.creation_time != after.creation_time
                or attributes != int(native._attributes(handle).FileAttributes)
                or checksum != expected or after.link_count < 1):
            raise CheckoutGuardChanged("Installed Git changed during its native identity capture",
                                       details={"reason": "input_identity", "path": path})
        return {"kind": "installed-git",
                **_identity(after, path=path, checksum=checksum, attributes=attributes)}
    except OSError as error:
        raise _native_error(error, path) from error
    finally:
        hierarchy.close()


class _DirectoryWatcher:
    """An owned asynchronous notification request and its joined observer."""
    def __init__(self, hierarchy, root, ignored, changed, *, interests=None, channel="all"):
        self.root, self.ignored, self.changed = root, ignored, changed
        filters = {"all": 0x01 | 0x02 | 0x04 | 0x08 | 0x10 | 0x40 | 0x100,
                   "data": 0x01 | 0x02 | 0x08 | 0x10 | 0x40,
                   "metadata": 0x04 | 0x100}
        if channel not in filters:
            raise ValueError("Unknown native notification channel")
        self.channel, self.notify_filter = channel, filters[channel]
        self.interests = interests
        first_segments = sorted({value.split("/", 1)[0] for value in interests or ()})
        self._shallow_candidates = [ntpath.join(root, value) for value in first_segments]
        self.recursive = interests is None or any(Path(value).is_dir() for value in self._shallow_candidates)
        self.armed_at = None
        self.api, self.native = _notification_api(), hierarchy.native
        self._lock, self._stop = threading.RLock(), threading.Event()
        self._buffer = (wintypes.DWORD * (64 * 1024 // 4))()
        self._overlapped = _Overlapped()
        self._pending, self._closed = False, False
        self._events, self._errors = [], []
        self._count, self._ignored_count = 0, 0
        self._event_digest = hashlib.sha256()
        self.started_at, self.finished_at = None, None
        self._handle, self._event, self._thread = None, None, None
        # An independent async handle permits notification I/O. Existing
        # anchored directory pins separately deny ancestor rename/deletion.
        if ntpath.splitdrive(root)[1] == "\\":
            api = self.native._apis()[0]
            self._handle = api.CreateFileW(root, self.native._DIRECTORY_READ_ACCESS, 7, None, 3,
                0x02000000 | 0x00200000 | 0x40000000, None)  # BACKUP, REPARSE, OVERLAPPED
            if not self._handle or self._handle == ctypes.c_void_p(-1).value:
                raise ctypes.WinError(ctypes.get_last_error())
        else:
            parent = hierarchy.directory(ntpath.dirname(root))
            self._handle = self.native._open_relative_handle(parent, ntpath.basename(root),
                access=self.native._DIRECTORY_READ_ACCESS, share=7,
                options=self.native._FILE_DIRECTORY_FILE | self.native._FILE_OPEN_REPARSE_POINT)
        try:
            observed = self.native._ensure_directory_handle(self._handle)
            before = hierarchy.directories[_key(root)][1]
            if not before.same_object(observed):
                raise CheckoutGuardChanged("Notification root identity differs from its pin",
                                           details={"reason": "input_alias", "path": root})
            self._event = self.api.CreateEventW(None, True, False, None)
            if not self._event:
                raise ctypes.WinError(ctypes.get_last_error())
            self._overlapped.hEvent = self._event
        except BaseException:
            self.native._close_handle(self._handle)
            self._handle = None
            raise

    def _error(self, reason, number=None, message=None):
        with self._lock:
            record = {"root": self.root, "channel": self.channel,
                      "notifyFilter": self.notify_filter, "reason": reason}
            if number is not None:
                record["nativeError"] = number
            if message is not None:
                record["message"] = str(message)
            if len(self._errors) < 32:
                self._errors.append(record)
            self.changed.set()

    def _submit(self):
        with self._lock:
            if self._stop.is_set():
                return False
            if not self.api.ResetEvent(self._event):
                raise ctypes.WinError(ctypes.get_last_error())
            self._overlapped.Internal = self._overlapped.InternalHigh = 0
            self._overlapped.Offset = self._overlapped.OffsetHigh = 0
            # LAST_ACCESS is deliberately excluded: reading an input is allowed.
            if not self.api.ReadDirectoryChangesW(self._handle, self._buffer,
                    ctypes.sizeof(self._buffer), self.recursive, self.notify_filter, None,
                    ctypes.byref(self._overlapped), None):
                raise ctypes.WinError(ctypes.get_last_error())
            self._pending = True
            return True

    def start(self):
        try:
            self._submit()
            # A shallow absence monitor must not miss a directory created
            # between selecting its mode and arming the native request.
            # Later first-segment creation is already queued by this request.
            if not self.recursive and any(Path(value).is_dir() for value in self._shallow_candidates):
                raise CheckoutGuardChanged("An interest became a directory before its monitor was armed",
                                           details={"reason": "watch_arming_changed", "path": self.root})
            self.armed_at = time.time()
            self.started_at = self.armed_at
            self._thread = threading.Thread(target=self._run,
                                            name="checkout-input-watch", daemon=False)
            self._thread.start()
        except BaseException:
            self._stop.set()
            self._cancel()
            self._drain()
            self.close()
            raise
        return self

    def _record(self, raw):
        for item in parse_notifications(raw):
            event = {"root": self.root, "channel": self.channel,
                     "notifyFilter": self.notify_filter, **item}
            folded = item["path"].casefold()
            # Namespace changes to an ancestor affect the exact input path.
            # A directory's data MODIFIED event can merely reflect an unrelated
            # sibling write. Attributes/security use a separate native filter,
            # whose ancestor MODIFIED records must still invalidate the read.
            relevant = self.interests is None or any(
                folded == value.casefold() or folded.startswith(value.casefold() + "/")
                or (value.casefold().startswith(folded + "/")
                    and (self.channel != "data" or item["action"] != 3))
                for value in self.interests)
            ignored = not relevant or any(
                folded == value.casefold() or folded.startswith(value.casefold() + "/")
                for value in self.ignored)
            with self._lock:
                self._count += 1
                self._event_digest.update(_encoded({**event, "ignored": ignored}) + b"\n")
                self._ignored_count += int(ignored)
                if not ignored:
                    if len(self._events) < 32:
                        self._events.append(event)
                    self.changed.set()

    def _cancel(self):
        with self._lock:
            if self._pending and not self.api.CancelIoEx(self._handle, ctypes.byref(self._overlapped)):
                number = ctypes.get_last_error()
                # The original request can already have completed normally.
                if number != 1168:
                    self._error("watch_cancel_failed", number)

    def _complete(self, *, wait=False):
        count = wintypes.DWORD()
        if self.api.GetOverlappedResult(self._handle, ctypes.byref(self._overlapped),
                                       ctypes.byref(count), wait):
            with self._lock:
                self._pending = False
            if count.value > ctypes.sizeof(self._buffer):
                raise CheckoutGuardChanged("Directory notification exceeded its owned buffer",
                                           details={"reason": "watch_invalid_notification"})
            self._record(ctypes.string_at(ctypes.addressof(self._buffer), count.value))
            return
        number = ctypes.get_last_error()
        if number == 996:  # ERROR_IO_INCOMPLETE: buffer is still owned by the OS.
            raise OSError(number, "Notification I/O has not completed")
        with self._lock:
            self._pending = False
        if number == 995 and self._stop.is_set():
            return
        self._error("watch_overflow" if number == 1022 else "watch_completion_failed", number)

    def _drain(self):
        if self._pending:
            try:
                # CancelIoEx is only a request. Retain the buffer and handle
                # until the original IRP has actually acknowledged completion.
                self._complete(wait=True)
            except BaseException as error:
                self._error("watch_drain_failed", message=error)

    def _run(self):
        try:
            while self._pending:
                outcome = self.api.WaitForSingleObject(self._event, 100)
                if outcome == 258:
                    continue
                if outcome != 0:
                    raise ctypes.WinError(ctypes.get_last_error())
                self._complete()
                if self._stop.is_set() or self._errors:
                    break
                if not self._submit():
                    break
        except BaseException as error:
            reason = error.details.get("reason") if isinstance(error, CheckoutGuardChanged) else "watch_observer_failed"
            self._error(reason, message=error)
        finally:
            self._stop.set()
            self._cancel()
            self._drain()
            self.finished_at = time.time()

    def finish(self, timeout=10.0):
        self._stop.set()
        self._cancel()
        if self._thread is not None and self._thread.ident is not None:
            self._thread.join(timeout)
        alive = self._thread is not None and self._thread.is_alive()
        joined = self._thread is not None and self._thread.ident is not None and not alive
        with self._lock:
            return {"root": self.root, "channel": self.channel, "notifyFilter": self.notify_filter,
                    "recursive": self.recursive, "armedAt": self.armed_at,
                    "startedAt": self.started_at,
                    "finishedAt": self.finished_at, "waiterJoined": joined,
                    "cancellationComplete": not alive and not self._pending,
                    "eventCount": self._count, "ignoredEventCount": self._ignored_count,
                    "eventDigest": self._event_digest.hexdigest(),
                    "events": copy.deepcopy(self._events), "errors": copy.deepcopy(self._errors)}

    def snapshot(self):
        with self._lock:
            return {"events": copy.deepcopy(self._events), "errors": copy.deepcopy(self._errors),
                    "observerAlive": self._thread is not None and self._thread.is_alive()}

    def close(self):
        if self._pending or (self._thread is not None and self._thread.is_alive()):
            raise CheckoutGuardChanged("Notification I/O or its observer remains active",
                                       details={"reason": "watch_unjoined", "path": self.root})
        if self._closed:
            return
        if self._event is not None:
            if not self.api.CloseHandle(self._event):
                raise ctypes.WinError(ctypes.get_last_error())
            self._event = None
        if self._handle is not None:
            self.native._close_handle(self._handle)
            self._handle = None
        self._closed = True


class CheckoutReadGuard:
    """Hold exact physical inputs until the caller has proven native death.

    ``files`` maps absolute paths to SHA256 or explicit absence. Every input
    must be covered by a watch root; ignored activity must never cover an input.
    ``finish`` ends monitoring but retains file pins. ``close`` releases them.
    No method creates a source view, changes author claims or issues acceptance.
    """
    def __init__(self, source_root, *, files, watch_roots, ignored_activity=None, watch_interests=None,
                 trusted_native_inputs=None):
        self.source_root = _absolute(source_root)
        if not isinstance(files, Mapping) or not files:
            raise ValueError("Checkout guard requires an exact nonempty input map")
        self.files, seen = {}, set()
        for path, checksum in files.items():
            path = _absolute(path)
            if _key(path) in seen or (checksum is not None
                    and (not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum))):
                raise ValueError("Checkout inputs contain an ambiguous path or digest")
            seen.add(_key(path))
            self.files[path] = checksum
        self.trusted_native_inputs = {}
        if trusted_native_inputs is not None and not isinstance(trusted_native_inputs, Mapping):
            raise ValueError("Trusted native inputs require internally captured exact Git facts")
        required = {"kind", "path", "sha256", "device", "fileId", "size", "mtimeNs",
                    "creationTime", "linkCount", "attributes"}
        for path, fact in (trusted_native_inputs or {}).items():
            path = _absolute(path)
            matched = next((name for name in self.files if _key(name) == _key(path)), None)
            if (matched is None or _under(path, self.source_root) or not isinstance(fact, Mapping)
                    or set(fact) != required or fact["kind"] != "installed-git"
                    or _key(_absolute(fact["path"])) != _key(path)
                    or ntpath.basename(path).casefold() != "git.exe"
                    or fact["sha256"] != self.files[matched] or fact["sha256"] is None
                    or not isinstance(fact["fileId"], str)
                    or not re.fullmatch(r"[0-9a-f]{32}", fact["fileId"])
                    or any(type(fact[name]) is not int for name in (
                        "device", "size", "mtimeNs", "creationTime", "linkCount", "attributes"))
                    or any(fact[name] < 0 for name in ("device", "size", "creationTime", "attributes"))
                    or fact["linkCount"] < 1 or fact["attributes"] & 0x410
                    or matched in self.trusted_native_inputs):
                raise ValueError("Trusted Git facts must bind a declared physical tool outside the source")
            self.trusted_native_inputs[matched] = {**copy.deepcopy(dict(fact)), "path": matched}
        if isinstance(watch_roots, (str, bytes, Path)):
            raise ValueError("Checkout guard requires an explicit watch-root list")
        self.watch_roots = list(dict.fromkeys(
            _absolute(path, allow_drive_root=True) for path in watch_roots))
        if not self.watch_roots or not any(_key(root) == _key(self.source_root) for root in self.watch_roots):
            raise ValueError("The actual source root must have its own directory monitor")
        if any(not any(_under(path, root) for root in self.watch_roots) for path in self.files):
            raise ValueError("Every input must be covered by directory monitoring")
        self.interests = {}
        if watch_interests is not None and not isinstance(watch_interests, Mapping):
            raise ValueError("Watch interests must name exact existing monitor roots")
        for root, relative_paths in (watch_interests or {}).items():
            root = _absolute(root, allow_drive_root=True)
            matched = next((value for value in self.watch_roots if _key(root) == _key(value)), None)
            if matched is None or not isinstance(relative_paths, (list, tuple)) or not relative_paths:
                raise ValueError("Watch interests require an existing root and nonempty relative list")
            self.interests[matched] = list(dict.fromkeys(_relative(value) for value in relative_paths))
        if any(not any(
                _under(path, root) and (root not in self.interests or any(
                    _under(path, ntpath.join(root, relative.replace("/", "\\")))
                    for relative in self.interests[root]))
                for root in self.watch_roots) for path in self.files):
            raise ValueError("Every declared input must have effective directory monitoring")
        self.ignored = {root: [] for root in self.watch_roots}
        if ignored_activity is not None and not isinstance(ignored_activity, Mapping):
            raise ValueError("Ignored activity must name its exact watch roots")
        for root, subtrees in (ignored_activity or {}).items():
            root = _absolute(root, allow_drive_root=True)
            matched = next((value for value in self.watch_roots if _key(root) == _key(value)), None)
            if matched is None or not isinstance(subtrees, (list, tuple)):
                raise ValueError("Ignored activity requires an existing watch root and explicit list")
            for relative in subtrees:
                relative = _relative(relative)
                ignored = ntpath.join(root, relative.replace("/", "\\"))
                if any(_under(path, ignored) for path in self.files):
                    raise ValueError("Ignored activity overlaps a declared input")
                self.ignored[matched].append(relative)
        expected = dict(sorted(self.files.items()))
        self.expected_inputs_digest = _digest(
            {"files": expected, "trustedNativeInputs": self.trusted_native_inputs}
            if self.trusted_native_inputs else expected)
        self._changed = threading.Event()
        self._hierarchy, self._watchers, self._identities = None, [], []
        self._errors, self._finished = [], None
        self._acquired, self._closed = False, False
        self.started_at = None

    def acquire(self):
        if self._acquired or self._closed or self._hierarchy is not None:
            raise ValueError("A checkout guard cannot be reacquired or reused")
        if os.name != "nt":
            raise CheckoutGuardChanged("Native checkout read guards require Windows",
                                       details={"reason": "native_api_unavailable"})
        self.started_at = time.time()
        self._hierarchy = _PinnedHierarchy()
        try:
            # Queue monitors before hashing or pinning any file. Kernel buffers
            # retain activity between successive notification requests.
            for root in self.watch_roots:
                self._hierarchy.directory(root)
                channels = ("data", "metadata") if root in self.interests else ("all",)
                for channel in channels:
                    watcher = _DirectoryWatcher(self._hierarchy, root, self.ignored[root], self._changed,
                                                interests=self.interests.get(root), channel=channel)
                    self._watchers.append(watcher)
                    watcher.start()
            for path, expected in sorted(self.files.items()):
                identity = self._hierarchy.file(path, expected,
                    trusted_native_input=self.trusted_native_inputs.get(path))
                if identity is not None:
                    self._identities.append(identity)
            self._acquired = True
            self.assert_stable()
        except BaseException as error:
            try:
                self.close()
            except BaseException as close_error:
                raise CheckoutGuardChanged("Partial input pins could not be completely released",
                    details={"reason": "guard_cleanup_incomplete", "cause": str(error),
                             "cleanup": str(close_error)}) from error
            if isinstance(error, (CheckoutGuardBusy, CheckoutGuardChanged)) or not isinstance(error, Exception):
                raise
            raise _native_error(error, self.source_root) from error
        return self

    def _observations(self):
        snapshots = [watcher.snapshot() for watcher in self._watchers]
        return ([event for item in snapshots for event in item["events"]],
                [*self._errors, *[error for item in snapshots for error in item["errors"]]])

    def facts(self):
        if self._finished is not None:
            return copy.deepcopy(self._finished)
        events, errors = self._observations()
        return {"schemaVersion": 1, "sourceRoot": self.source_root,
                "active": self._acquired and not self._closed,
                "stable": self._acquired and not events and not errors and not self._changed.is_set(),
                "complete": False, "expectedInputsDigest": self.expected_inputs_digest,
                "trustedNativeInputs": copy.deepcopy(self.trusted_native_inputs),
                "trustedNativeInputsDigest": _digest(self.trusted_native_inputs),
                "fileIdentityDigest": _digest(self._identities),
                "handleIdentities": copy.deepcopy(self._identities),
                "heldFileCount": len(self._identities),
                "missingFileCount": sum(value is None for value in self.files.values()),
                "watchRoots": list(self.watch_roots), "watchInterests": copy.deepcopy(self.interests),
                "watchModes": {
                    root: {"recursive": matching[0].recursive, "armedAt": matching[0].armed_at,
                           "channels": [{"channel": watcher.channel, "notifyFilter": watcher.notify_filter,
                                         "recursive": watcher.recursive, "armedAt": watcher.armed_at}
                                        for watcher in matching]}
                    for root in self.watch_roots
                    if (matching := [watcher for watcher in self._watchers if watcher.root == root])},
                "startedAt": self.started_at,
                "events": events[:32], "errors": errors[:32], "inputCopiesCreated": 0}

    def assert_stable(self):
        if not self._acquired or self._closed:
            raise CheckoutGuardChanged("Checkout input guard is not held",
                                       details={"reason": "guard_not_active"})
        if self._finished is not None:
            if self._finished["stable"] is True and self._finished["complete"] is True:
                return self.facts()
            raise CheckoutGuardChanged("Checkout monitoring did not prove stable input",
                                       details={"reason": "guard_finish_failed", "evidence": self.facts()})
        self._hierarchy.verify()
        events, errors = self._observations()
        if events or errors or self._changed.is_set():
            raise CheckoutGuardChanged("Checkout input activity occurred while guarded",
                details={"reason": "watch_activity" if events else "watch_failed",
                         "events": events[:32], "errors": errors[:32]})
        if any(not watcher.snapshot()["observerAlive"] for watcher in self._watchers):
            raise CheckoutGuardChanged("A checkout observer stopped before native termination",
                                       details={"reason": "watch_stopped"})
        return self.facts()

    def wait_for_change(self, timeout):
        """Wait for recorded activity without inventing a filesystem barrier."""
        if type(timeout) not in (int, float) or not 0 <= timeout <= 60:
            raise ValueError("Activity waits must be bounded to at most 60 seconds")
        return self._changed.wait(timeout)

    def finish(self):
        """Acknowledge cancellation and join monitors; retain all input pins."""
        if self._finished is not None:
            return copy.deepcopy(self._finished)
        if self._acquired and any(not watcher.snapshot()["observerAlive"] for watcher in self._watchers):
            self._errors.append({"reason": "watch_stopped", "message": "Observer ended before guard finish"})
        def verify_inputs():
            if self._hierarchy is None:
                return
            try:
                self._hierarchy.verify()
            except BaseException as error:
                typed = isinstance(error, CheckoutGuardChanged)
                details = copy.deepcopy(error.details) if typed else {}
                known_change = typed and details.get("reason") in {"input_identity", "input_alias"}
                self._errors.append({"reason": "input_identity", "message": str(error),
                    "type": type(error).__name__, "details": details,
                    "knownInputChange": known_change})

        verify_inputs()
        watchers = [watcher.finish() for watcher in self._watchers]
        verify_inputs()
        value = self.facts()
        events = [event for watcher in watchers for event in watcher["events"]]
        watcher_errors = [error for watcher in watchers for error in watcher["errors"]]
        errors = [*self._errors, *watcher_errors]
        input_changes = [error for error in self._errors if error.get("knownInputChange") is True]
        completion_errors = [error for error in self._errors if error.get("knownInputChange") is not True]
        completion_errors.extend(watcher_errors)
        joined = bool(watchers) and all(watcher["waiterJoined"] is True for watcher in watchers)
        cancelled = bool(watchers) and all(watcher["cancellationComplete"] is True for watcher in watchers)
        # A detected input change invalidates stability, not completed native I/O.
        complete = self._acquired and joined and cancelled and not completion_errors
        stable = complete and not errors and not events and not self._changed.is_set()
        self._finished = {**value, "active": False, "complete": complete,
            "observedComplete": complete, "stable": stable, "sourceNoWrites": stable,
            "membershipStable": stable, "cancellationComplete": cancelled,
            "waitersJoined": joined, "finishedAt": time.time(), "watchers": watchers,
            "events": events[:32], "errors": errors[:32],
            "inputChanges": input_changes[:32], "completionErrors": completion_errors[:32]}
        return copy.deepcopy(self._finished)

    def close(self):
        """Release pins only after native notification I/O really completed."""
        if self._closed:
            return
        self.finish()
        for watcher in self._watchers:
            if not watcher.finish()["cancellationComplete"]:
                raise CheckoutGuardChanged("Checkout observer cancellation remains incomplete",
                                           details={"reason": "watch_unjoined", "path": watcher.root})
        for watcher in self._watchers:
            watcher.close()
        if self._hierarchy is not None:
            self._hierarchy.close()
        self._closed = True

    def __enter__(self):
        return self.acquire()

    def __exit__(self, _kind, _value, _traceback):
        self.close()

"""Read-only Windows observations for an abandoned controller startup."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import os
import ntpath
import struct
import sys
from pathlib import Path

from ..contracts import JenkinsError
from ..processes.identity import current_identity, process_alive
from .driver import _plain


def observe_departed_identity(expected: dict) -> dict:
    """An inaccessible PID is inconclusive, never evidence of its departure."""
    pid = expected["pid"]
    alive = process_alive(pid)
    if alive is False:
        return {"pid": pid, "expectedCreationTime": expected["creationTime"], "status": "absent"}
    if alive is None:
        raise JenkinsError("deployment_identity_inconclusive", "Cannot establish the old owner identity", retryable=True)
    try:
        actual = current_identity(pid).to_dict()
    except (JenkinsError, OSError) as error:
        if process_alive(pid) is False:
            return {"pid": pid, "expectedCreationTime": expected["creationTime"], "status": "absent"}
        raise JenkinsError("deployment_identity_inconclusive", "Cannot query the old owner identity", retryable=True) from error
    if str(actual["creationTime"]) == str(expected["creationTime"]):
        raise JenkinsError("deployment_owner_live", "The old lifecycle component is still alive", retryable=True)
    return {"pid": pid, "expectedCreationTime": expected["creationTime"],
            "status": "pid-reused", "observedIdentity": actual}


_SPI_HEADER_SIZE = 256
_SPI_THREAD_SIZE = 80
_SPI_MAX_BYTES = 64 * 1024 * 1024
_SPI_RETRY_STATUSES = {0xC0000004, 0xC0000023}


def _inventory_inconclusive(message):
    raise JenkinsError("deployment_inventory_inconclusive", message, retryable=True)


def _parse_process_snapshot(raw: bytes, base_address: int) -> dict[int, dict]:
    """Parse the fixed NT 10 x64 SystemProcessInformation ABI fail-closed."""
    if not isinstance(raw, bytes) or not raw or type(base_address) is not int or base_address <= 0:
        _inventory_inconclusive("Native process snapshot input is invalid")
    if len(raw) > _SPI_MAX_BYTES:
        _inventory_inconclusive("Native process snapshot exceeds its fixed bound")
    processes = {}
    offset = 0
    while True:
        if offset % 8 or offset + _SPI_HEADER_SIZE > len(raw):
            _inventory_inconclusive("Native process record header is truncated or misaligned")
        next_offset, thread_count, _, image_length, image_maximum, image_pointer, _, pid, parent, handles, _ = \
            struct.unpack_from("<II48sHH4xQi4xQQII", raw, offset)
        peak_threads = struct.unpack_from("<I", raw, offset + 20)[0]
        creation, user_time, kernel_time = struct.unpack_from("<qqq", raw, offset + 32)
        record_end = len(raw) if next_offset == 0 else offset + next_offset
        minimum = _SPI_HEADER_SIZE + thread_count * _SPI_THREAD_SIZE
        if minimum > record_end - offset or (next_offset and
                (next_offset % 8 or next_offset < minimum or record_end >= len(raw))):
            _inventory_inconclusive("Native process record bounds or thread extent are invalid")
        if thread_count > 65536 or (pid != 0 and peak_threads < thread_count) or min(creation, user_time, kernel_time) < 0:
            _inventory_inconclusive("Native process counters are inconsistent")
        if image_length > image_maximum or image_length % 2 or image_maximum % 2 or image_length > 65534:
            _inventory_inconclusive("Native process image descriptor is invalid")
        image_name = ""
        if image_length:
            image_offset = image_pointer - base_address
            if image_offset < offset + minimum or image_offset + image_maximum > record_end:
                _inventory_inconclusive("Native process image string escaped its record")
            try:
                image_name = raw[image_offset:image_offset + image_length].decode("utf-16-le", errors="strict")
            except UnicodeDecodeError as error:
                raise JenkinsError("deployment_inventory_inconclusive",
                                   "Native process image string is malformed", retryable=True) from error
            if not image_name or "\x00" in image_name:
                _inventory_inconclusive("Native process image string is not canonical")
        if pid in processes:
            _inventory_inconclusive("Native process snapshot contains a duplicate PID")
        processes[pid] = {"pid": pid, "parentPid": parent, "creationTime": creation,
            "imageName": image_name, "numberOfThreads": thread_count, "handleCount": handles,
            "peakThreads": peak_threads, "userTime": user_time, "kernelTime": kernel_time}
        if next_offset == 0:
            used_end = max(offset + minimum, image_pointer - base_address + image_maximum if image_length else 0)
            trailing = raw[used_end:]
            if len(trailing) > 7 or any(trailing):
                _inventory_inconclusive("Native process snapshot has an invalid final record extent")
            break
        offset = record_end
    if not processes:
        _inventory_inconclusive("Native process snapshot is empty")
    return processes


def _native_process_snapshot() -> dict[int, dict]:
    if os.name != "nt" or ctypes.sizeof(ctypes.c_void_p) != 8:
        _inventory_inconclusive("The fixed process ABI requires 64-bit Windows")
    try:
        if sys.getwindowsversion().major != 10:
            _inventory_inconclusive("The fixed process ABI is limited to NT 10")
        query = ctypes.WinDLL("ntdll.dll").NtQuerySystemInformation
    except (AttributeError, OSError) as error:
        raise JenkinsError("deployment_inventory_unavailable",
                           "Native process inventory is unavailable", retryable=True) from error
    query.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_ulong)]
    query.restype = ctypes.c_long
    capacity = 1024 * 1024
    while capacity <= _SPI_MAX_BYTES:
        buffer = ctypes.create_string_buffer(capacity)
        returned = ctypes.c_ulong()
        status = int(query(5, buffer, capacity, ctypes.byref(returned))) & 0xFFFFFFFF
        if status == 0:
            length = int(returned.value)
            if not 0 < length <= capacity:
                _inventory_inconclusive("Native process inventory returned an invalid length")
            return _parse_process_snapshot(ctypes.string_at(buffer, length), ctypes.addressof(buffer))
        if status not in _SPI_RETRY_STATUSES:
            _inventory_inconclusive("Native process inventory query failed")
        capacity = max(capacity * 2, int(returned.value))
    _inventory_inconclusive("Native process inventory exceeded its allocation bound")


def _system_process_image(pid: int) -> str:
    class UnicodeString(ctypes.Structure):
        _fields_=[("Length",ctypes.c_ushort),("MaximumLength",ctypes.c_ushort),("Buffer",ctypes.c_void_p)]
    class ProcessIdInformation(ctypes.Structure):
        _fields_=[("ProcessId",ctypes.c_void_p),("ImageName",UnicodeString)]
    try:
        query=ctypes.WinDLL("ntdll.dll").NtQuerySystemInformation
    except (AttributeError,OSError) as error:
        raise JenkinsError("deployment_inventory_inconclusive","Native process image query is unavailable",retryable=True) from error
    query.argtypes=[ctypes.c_int,ctypes.c_void_p,ctypes.c_ulong,ctypes.POINTER(ctypes.c_ulong)]
    query.restype=ctypes.c_long
    storage=ctypes.create_unicode_buffer(32768); info=ProcessIdInformation()
    info.ProcessId=ctypes.c_void_p(pid); info.ImageName.Length=0
    info.ImageName.MaximumLength=ctypes.sizeof(storage)-ctypes.sizeof(ctypes.c_wchar)
    info.ImageName.Buffer=ctypes.addressof(storage); returned=ctypes.c_ulong()
    status=int(query(88,ctypes.byref(info),ctypes.sizeof(info),ctypes.byref(returned)))&0xFFFFFFFF
    if status!=0 or info.ImageName.Buffer!=ctypes.addressof(storage) or info.ImageName.Length==0 \
            or info.ImageName.Length%2 or info.ImageName.Length>info.ImageName.MaximumLength:
        _inventory_inconclusive("Native process image identity could not be verified")
    if info.ImageName.MaximumLength > ctypes.sizeof(storage)-ctypes.sizeof(ctypes.c_wchar):
        _inventory_inconclusive("Native process image descriptor exceeded its allocation")
    value=ctypes.wstring_at(storage,info.ImageName.Length//2)
    # NT paths have one leading separator; isabs changed in Python 3.13.
    parts=value.split("\\")
    if (not value.startswith("\\Device\\") or "\x00" in value
            or len(parts)<4 or any(part in {"",".",".."} for part in parts[1:])
            or "/" in value or ntpath.normpath(value)!=value):
        _inventory_inconclusive("Native process image identity is not a canonical NT device path")
    return value


def _physical_dos_path(value: str) -> str:
    if value.startswith("\\Device\\"):
        return ntpath.normcase(ntpath.normpath(value))
    drive,tail=ntpath.splitdrive(value)
    if len(drive)!=2 or drive[1]!=":" or not tail.startswith("\\"):
        _inventory_inconclusive("Process image path has no verifiable DOS device")
    function=ctypes.WinDLL("kernel32",use_last_error=True).QueryDosDeviceW
    function.argtypes=[wintypes.LPCWSTR,wintypes.LPWSTR,wintypes.DWORD]
    function.restype=wintypes.DWORD
    target=ctypes.create_unicode_buffer(32768)
    if not function(drive,target,len(target)) or not target.value.startswith("\\Device\\"):
        _inventory_inconclusive("Runtime DOS device mapping is unavailable")
    return ntpath.normcase(ntpath.normpath(target.value+tail))


def _under_runtime(image_path: str, repository: Path) -> bool:
    runtime=(repository/".jenkins/runtime").absolute()
    _plain(runtime)
    root=_physical_dos_path(str(runtime))
    image=_physical_dos_path(image_path)
    try:
        if ntpath.normcase(ntpath.commonpath((root,image)))!=root or image==root:
            return False
    except ValueError:
        return False
    _plain(runtime/ntpath.relpath(image,root))
    return True


def _classify_nonexecuting_object(first: dict, second: dict, pid: int, image_path: str) -> dict:
    left,right=first.get(pid),second.get(pid)
    if not isinstance(left,dict) or not isinstance(right,dict):
        _inventory_inconclusive("Retained process identity changed between complete snapshots")
    fields=("pid","creationTime","parentPid","imageName")
    if any(left.get(field)!=right.get(field) for field in fields) or type(pid) is not int or pid<=0 \
            or right["creationTime"]<=0 or not image_path \
            or ntpath.basename(image_path).casefold()!=right["imageName"].casefold():
        _inventory_inconclusive("Retained process birth, parent or image changed")
    for row in (left,right):
        if row["numberOfThreads"]!=0 or row["handleCount"]!=0 \
                or row["peakThreads"]<row["numberOfThreads"] or min(row["userTime"],row["kernelTime"])<0:
            _inventory_inconclusive("Retained process still has execution activity or owned handles")
    if right["peakThreads"]<left["peakThreads"] or right["userTime"]<left["userTime"] \
            or right["kernelTime"]<left["kernelTime"]:
        _inventory_inconclusive("Retained process history counters regressed")
    if right["userTime"]+right["kernelTime"]==0 and (
            right["parentPid"] in first or right["parentPid"] in second):
        _inventory_inconclusive("A never-run retained object still has a live or uncertain creator")
    return {"processId":pid,"creationTime":str(right["creationTime"]),"parentPid":right["parentPid"],
        "imagePath":image_path,"numberOfThreads":0,"handleCount":0,"peakThreads":right["peakThreads"],
        "userTime":right["userTime"],"kernelTime":right["kernelTime"],
        "status":"non-executing-retained-process-object","observationOnly":True,
        "terminalProof":False,"nativeProofIssued":False,"stdoutEof":False,"stderrEof":False,
        "executionHoldsReleased":False}


class _RuntimeOwnerInventory(list):
    def __init__(self,active_owners,non_executing_objects):
        super().__init__(active_owners); self.non_executing_objects=list(non_executing_objects)


def _observe_nonexecuting_object(pid: int, repository: Path) -> dict:
    # Independently bracket both full image identities with complete native inventories.
    first=_native_process_snapshot()
    first_image=_system_process_image(pid)
    second=_native_process_snapshot()
    second_image=_system_process_image(pid)
    if (not _under_runtime(first_image,repository) or not _under_runtime(second_image,repository)
            or _physical_dos_path(first_image)!=_physical_dos_path(second_image)):
        _inventory_inconclusive("Retained process full physical image changed or escaped the dedicated runtime")
    return _classify_nonexecuting_object(first,second,pid,second_image)


def runtime_owner_inventory(repository: Path) -> list[dict]:
    """Enumerate all processes without WMI or exposing process arguments."""
    if os.name != "nt":
        raise JenkinsError("deployment_inventory_unavailable", "This recovery observation requires Windows")
    class ProcessEntry(ctypes.Structure):
        _fields_ = [("size", wintypes.DWORD), ("usage", wintypes.DWORD),
                    ("pid", wintypes.DWORD), ("heap", ctypes.c_size_t),
                    ("module", wintypes.DWORD), ("threads", wintypes.DWORD),
                    ("parentPid", wintypes.DWORD), ("priority", wintypes.LONG),
                    ("flags", wintypes.DWORD), ("executableName", wintypes.WCHAR * 260)]
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    api.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    for name in ("Process32FirstW", "Process32NextW"):
        function = getattr(api, name)
        function.argtypes = [wintypes.HANDLE, ctypes.POINTER(ProcessEntry)]
        function.restype = wintypes.BOOL
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    snapshot = api.CreateToolhelp32Snapshot(2, 0)
    if not snapshot or snapshot == ctypes.c_void_p(-1).value:
        raise JenkinsError("deployment_inventory_unavailable", "Windows process inventory is unavailable", retryable=True)
    owners = []
    non_executing = []
    runtime = (repository / ".jenkins/runtime").absolute()
    try:
        entry = ProcessEntry()
        entry.size = ctypes.sizeof(entry)
        available = api.Process32FirstW(snapshot, ctypes.byref(entry))
        while available:
            if entry.pid != os.getpid() and entry.executableName.casefold() in {"java.exe", "javaw.exe", "python.exe", "pythonw.exe"}:
                try:
                    actual = current_identity(int(entry.pid)).to_dict()
                except (JenkinsError, OSError) as error:
                    if process_alive(int(entry.pid)) is not False:
                        non_executing.append(_observe_nonexecuting_object(int(entry.pid), repository))
                else:
                    executable = Path(actual["executable"]).absolute()
                    if executable.is_relative_to(runtime):
                        owners.append(actual)
            available = api.Process32NextW(snapshot, ctypes.byref(entry))
        if ctypes.get_last_error() != 18:
            raise JenkinsError("deployment_inventory_inconclusive", "Process enumeration did not finish", retryable=True)
    finally:
        api.CloseHandle(snapshot)
    return _RuntimeOwnerInventory(owners, non_executing)


def observe_runtime_owner_inventory(repository: Path) -> dict:
    from types import SimpleNamespace
    from .runtime_roles import observe_runtime_process_boundary

    repository = repository.absolute()
    inventory = runtime_owner_inventory(repository)
    manager = SimpleNamespace(paths=SimpleNamespace(repo=repository, state=repository / ".jenkins/state"))
    observation = observe_runtime_process_boundary(manager, inventory=inventory)
    return {**observation, "activeOwners": list(inventory),
            "nonExecutingObjects": list(getattr(inventory, "non_executing_objects", []))}

def observe_home_available(home: Path) -> dict:
    """Probe existing controller files without deleting stale owner markers."""
    _plain(home)
    if os.name != "nt" or not (home / "config.xml").is_file():
        raise JenkinsError("deployment_home_unproven", "The existing Jenkins Home is unavailable")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                              ctypes.c_void_p, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    api.CreateFileW.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    observations = []
    for name in ("config.xml", "queue.xml", ".owner"):
        path = home / name
        _plain(path)
        if not path.exists():
            continue
        handle = api.CreateFileW(str(path), 0x80000000, 0, None, 3, 0x80, None)
        if not handle or handle == ctypes.c_void_p(-1).value:
            raise JenkinsError("deployment_home_busy", "A Jenkins Home file is occupied or inaccessible", retryable=True,
                               details={"file": name, "osError": ctypes.get_last_error()})
        try:
            stat = path.stat()
            observations.append({"file": name, "bytes": stat.st_size, "modifiedNs": stat.st_mtime_ns})
        finally:
            api.CloseHandle(handle)
    return {"exclusiveReadProbes": observations, "staleMarkersDeleted": False}

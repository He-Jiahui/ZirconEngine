"""Query only the retained, runner-owned Job; never grant termination rights."""
from __future__ import annotations

import ctypes
import os
from pathlib import PureWindowsPath
from ctypes import wintypes


def error_record(stage, error):
    return {"stage": stage, "type": type(error).__name__, "message": str(error)}


class ObservedPipe:
    """Delegate a pipe, recording only an actual empty read as EOF."""
    def __init__(self, stream, observation):
        self.stream, self.observation = stream, observation
        self.buffer = self
        self.raw = getattr(stream, "buffer", stream)

    def read1(self, size):
        try:
            data = getattr(self.raw, "read1", self.raw.read)(size)
        except (OSError, ValueError) as error:
            self.observation["error"] = error_record("pipe_read", error)
            raise
        if data == b"":
            self.observation["eof"] = True
        return data

    def read(self, size=-1):
        return self.read1(size)

    def close(self):
        self.observation["closed"] = True
        return self.stream.close()

    def __getattr__(self, name):
        return getattr(self.stream, name)


def observed_atomic_factory(observations):
    def create(*args, **kwargs):
        from tools.jenkins.pilot.native.windows_job_process import create_atomic_kill_on_close_process
        process, job = create_atomic_kill_on_close_process(*args, **kwargs)
        for channel in ("stdout", "stderr"):
            observation = observations[channel] = {"eof": False, "closed": False}
            setattr(process, channel, ObservedPipe(getattr(process, channel), observation))
        return process, job
    return create


class JobEvidence:
    def __init__(self, process):
        self.handle = None
        self.records = {}
        self.errors = []
        try:
            if os.name != "nt" or not process.job_handle:
                raise OSError("owned Windows Job handle unavailable")
            self.api = ctypes.WinDLL("kernel32", use_last_error=True)
            self.api.GetCurrentProcess.restype = wintypes.HANDLE
            self.api.DuplicateHandle.argtypes = [wintypes.HANDLE, wintypes.HANDLE, wintypes.HANDLE,
                ctypes.POINTER(wintypes.HANDLE), wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            self.api.DuplicateHandle.restype = wintypes.BOOL
            self.api.CloseHandle.argtypes = [wintypes.HANDLE]
            self.api.QueryInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int,
                ctypes.c_void_p, wintypes.DWORD, ctypes.c_void_p]
            self.api.QueryInformationJobObject.restype = wintypes.BOOL
            local = self.api.GetCurrentProcess()
            duplicate = wintypes.HANDLE()
            if not self.api.DuplicateHandle(local, process.job_handle, local,
                    ctypes.byref(duplicate), 4, False, 0):  # JOB_OBJECT_QUERY only
                raise ctypes.WinError(ctypes.get_last_error())
            self.handle = duplicate.value
        except Exception as error:
            self.errors.append(error_record("duplicate_query_handle", error))

    def _accounting(self):
        from tools.jenkins.pilot.native.windows_job_process import _JobObjectBasicAccountingInformation
        value = _JobObjectBasicAccountingInformation()
        if not self.api.QueryInformationJobObject(self.handle, 1, ctypes.byref(value), ctypes.sizeof(value), None):
            raise ctypes.WinError(ctypes.get_last_error())
        return int(value.active_processes)

    def _members(self):
        # JobObjectBasicProcessIdList: two DWORDs followed by ULONG_PTR values.
        capacity = 64
        while capacity <= 65536:
            raw = ctypes.create_string_buffer(8 + capacity * ctypes.sizeof(ctypes.c_size_t))
            if self.api.QueryInformationJobObject(self.handle, 3, raw, len(raw), None):
                count = wintypes.DWORD.from_buffer(raw, 4).value
                ids = (ctypes.c_size_t * count).from_buffer(raw, 8)
                return [self._member(int(pid)) for pid in ids]
            code = ctypes.get_last_error()
            if code != 234:  # ERROR_MORE_DATA
                raise ctypes.WinError(code)
            capacity *= 2
        raise OSError("owned Job PID inventory exceeded bound")

    def _member(self, pid):
        result = {"pid": pid, "creationTime": None, "exeBasename": None, "member": None}
        api = self.api
        api.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        api.OpenProcess.restype = wintypes.HANDLE
        api.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        api.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
        api.IsProcessInJob.argtypes = [wintypes.HANDLE, wintypes.HANDLE, ctypes.POINTER(wintypes.BOOL)]
        handle = api.OpenProcess(0x1000, False, pid)  # PROCESS_QUERY_LIMITED_INFORMATION
        try:
            if not handle:
                raise ctypes.WinError(ctypes.get_last_error())
            times = [wintypes.FILETIME() for _ in range(4)]
            if not api.GetProcessTimes(handle, *(ctypes.byref(value) for value in times)):
                raise ctypes.WinError(ctypes.get_last_error())
            result["creationTime"] = str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)
            size = wintypes.DWORD(32768)
            name = ctypes.create_unicode_buffer(size.value)
            if not api.QueryFullProcessImageNameW(handle, 0, name, ctypes.byref(size)):
                raise ctypes.WinError(ctypes.get_last_error())
            result["exeBasename"] = PureWindowsPath(name.value).name
            member = wintypes.BOOL()
            if not api.IsProcessInJob(handle, self.handle, ctypes.byref(member)):
                raise ctypes.WinError(ctypes.get_last_error())
            result["member"] = bool(member.value)
            if not member.value:
                raise OSError("enumerated PID no longer belongs to the owned Job")
        except Exception as error:
            result["error"] = error_record("job_member", error)
            self.errors.append(result["error"])
        finally:
            if handle:
                api.CloseHandle(handle)
        return result

    def capture(self, stage, *, members=False):
        record = {"activeProcesses": None}
        try:
            if self.handle is None:
                raise OSError("query duplicate unavailable")
            record["activeProcesses"] = self._accounting()
            if members:
                record["members"] = self._members()
        except Exception as error:
            self.errors.append(error_record(stage, error))
        self.records[stage] = record
        return record

    def close(self):
        if self.handle is not None:
            handle, self.handle = self.handle, None
            if not self.api.CloseHandle(handle):
                self.errors.append(error_record("close_query_duplicate", ctypes.WinError(ctypes.get_last_error())))

    def to_dict(self):
        return {"access": "JOB_OBJECT_QUERY", "records": self.records, "errors": self.errors}

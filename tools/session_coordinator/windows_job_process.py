from __future__ import annotations

import ctypes
import io
import os
import subprocess
import time
from pathlib import Path


class _IoCounters(ctypes.Structure):
    _fields_ = (
        ("read_operation_count", ctypes.c_uint64),
        ("write_operation_count", ctypes.c_uint64),
        ("other_operation_count", ctypes.c_uint64),
        ("read_transfer_count", ctypes.c_uint64),
        ("write_transfer_count", ctypes.c_uint64),
        ("other_transfer_count", ctypes.c_uint64),
    )


class _JobObjectBasicLimitInformation(ctypes.Structure):
    _fields_ = (
        ("per_process_user_time_limit", ctypes.c_int64),
        ("per_job_user_time_limit", ctypes.c_int64),
        ("limit_flags", ctypes.c_uint32),
        ("minimum_working_set_size", ctypes.c_size_t),
        ("maximum_working_set_size", ctypes.c_size_t),
        ("active_process_limit", ctypes.c_uint32),
        ("affinity", ctypes.c_size_t),
        ("priority_class", ctypes.c_uint32),
        ("scheduling_class", ctypes.c_uint32),
    )


class _JobObjectExtendedLimitInformation(ctypes.Structure):
    _fields_ = (
        ("basic_limit_information", _JobObjectBasicLimitInformation),
        ("io_info", _IoCounters),
        ("process_memory_limit", ctypes.c_size_t),
        ("job_memory_limit", ctypes.c_size_t),
        ("peak_process_memory_used", ctypes.c_size_t),
        ("peak_job_memory_used", ctypes.c_size_t),
    )


class _SecurityAttributes(ctypes.Structure):
    _fields_ = (
        ("length", ctypes.c_uint32),
        ("security_descriptor", ctypes.c_void_p),
        ("inherit_handle", ctypes.c_int32),
    )


class _StartupInfoW(ctypes.Structure):
    _fields_ = (
        ("cb", ctypes.c_uint32),
        ("reserved", ctypes.c_wchar_p),
        ("desktop", ctypes.c_wchar_p),
        ("title", ctypes.c_wchar_p),
        ("x", ctypes.c_uint32),
        ("y", ctypes.c_uint32),
        ("x_size", ctypes.c_uint32),
        ("y_size", ctypes.c_uint32),
        ("x_count_chars", ctypes.c_uint32),
        ("y_count_chars", ctypes.c_uint32),
        ("fill_attribute", ctypes.c_uint32),
        ("flags", ctypes.c_uint32),
        ("show_window", ctypes.c_uint16),
        ("reserved_2_size", ctypes.c_uint16),
        ("reserved_2", ctypes.POINTER(ctypes.c_ubyte)),
        ("stdin", ctypes.c_void_p),
        ("stdout", ctypes.c_void_p),
        ("stderr", ctypes.c_void_p),
    )


class _StartupInfoExW(ctypes.Structure):
    _fields_ = (("startup_info", _StartupInfoW), ("attribute_list", ctypes.c_void_p))


class _ProcessInformation(ctypes.Structure):
    _fields_ = (
        ("process", ctypes.c_void_p),
        ("thread", ctypes.c_void_p),
        ("process_id", ctypes.c_uint32),
        ("thread_id", ctypes.c_uint32),
    )


class _JobObjectBasicAccountingInformation(ctypes.Structure):
    _fields_ = (
        ("total_user_time", ctypes.c_int64),
        ("total_kernel_time", ctypes.c_int64),
        ("this_period_total_user_time", ctypes.c_int64),
        ("this_period_total_kernel_time", ctypes.c_int64),
        ("total_page_fault_count", ctypes.c_uint32),
        ("total_processes", ctypes.c_uint32),
        ("active_processes", ctypes.c_uint32),
        ("total_terminated_processes", ctypes.c_uint32),
    )


class AtomicJobProcess:
    """Popen-compatible root created atomically inside a Windows Job Object."""

    def __init__(
        self,
        *,
        args: tuple[str, ...],
        process_handle: int,
        pid: int,
        stdout: io.TextIOWrapper,
        stderr: io.TextIOWrapper,
    ) -> None:
        self.args = args
        self._handle = process_handle
        self.pid = pid
        self.stdout = stdout
        self.stderr = stderr
        self.returncode: int | None = None
        self._closed = False

    def poll(self) -> int | None:
        if self.returncode is not None:
            return self.returncode
        kernel32 = ctypes.windll.kernel32
        kernel32.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
        kernel32.WaitForSingleObject.restype = ctypes.c_uint32
        result = int(kernel32.WaitForSingleObject(self._handle, 0))
        if result == 258:
            return None
        if result != 0:
            raise OSError(ctypes.get_last_error(), "WaitForSingleObject failed")
        self.returncode = _process_exit_code(self._handle)
        return self.returncode

    def wait(self, timeout: float | None = None) -> int:
        if self.returncode is not None:
            return self.returncode
        kernel32 = ctypes.windll.kernel32
        kernel32.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
        kernel32.WaitForSingleObject.restype = ctypes.c_uint32
        milliseconds = 0xFFFFFFFF if timeout is None else max(0, int(timeout * 1000))
        result = int(kernel32.WaitForSingleObject(self._handle, milliseconds))
        if result == 258:
            raise subprocess.TimeoutExpired(self.args, timeout)
        if result != 0:
            raise OSError(ctypes.get_last_error(), "WaitForSingleObject failed")
        self.returncode = _process_exit_code(self._handle)
        return self.returncode

    def kill(self) -> None:
        _terminate_process_handle(self._handle)

    def terminate(self) -> None:
        self.kill()

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        for stream in (self.stdout, self.stderr):
            try:
                stream.close()
            except OSError:
                pass
        if self._handle:
            _close_handle(self._handle)
            self._handle = 0

    def __del__(self) -> None:
        self.close()


def create_atomic_kill_on_close_process(
    args: tuple[str, ...],
    *,
    cwd: Path,
    env: dict[str, str],
) -> tuple[AtomicJobProcess, int]:
    """Create a suspended root whose Job membership exists at first instruction."""
    if os.name != "nt":
        raise OSError("Atomic Windows Job launch is unavailable on this platform")
    import msvcrt

    kernel32 = ctypes.windll.kernel32
    _declare_create_process_functions(kernel32)
    inheritable = _SecurityAttributes(ctypes.sizeof(_SecurityAttributes), None, True)
    handles: list[int] = []
    attribute_buffer: ctypes.Array[ctypes.c_char] | None = None
    attribute_list_initialized = False
    stdout_stream: io.TextIOWrapper | None = None
    stderr_stream: io.TextIOWrapper | None = None
    process_handle = 0
    thread_handle = 0
    job_handle = _create_kill_on_close_job()
    try:
        stdout_read, stdout_write = _create_pipe(kernel32, inheritable, handles)
        stderr_read, stderr_write = _create_pipe(kernel32, inheritable, handles)
        stdin_handle = _open_inheritable_null(kernel32, inheritable)
        handles.append(stdin_handle)

        attribute_size = ctypes.c_size_t()
        kernel32.InitializeProcThreadAttributeList(None, 2, 0, ctypes.byref(attribute_size))
        attribute_buffer = ctypes.create_string_buffer(attribute_size.value)
        if not kernel32.InitializeProcThreadAttributeList(
            attribute_buffer, 2, 0, ctypes.byref(attribute_size)
        ):
            raise OSError(
                ctypes.get_last_error(), "InitializeProcThreadAttributeList failed"
            )
        attribute_list_initialized = True
        job_value = ctypes.c_void_p(job_handle)
        if not kernel32.UpdateProcThreadAttribute(
            attribute_buffer,
            0,
            0x0002000D,
            ctypes.byref(job_value),
            ctypes.sizeof(job_value),
            None,
            None,
        ):
            raise OSError(ctypes.get_last_error(), "Job-list attribute failed")
        inherited_handles = (ctypes.c_void_p * 3)(stdin_handle, stdout_write, stderr_write)
        if not kernel32.UpdateProcThreadAttribute(
            attribute_buffer,
            0,
            0x00020002,
            inherited_handles,
            ctypes.sizeof(inherited_handles),
            None,
            None,
        ):
            raise OSError(ctypes.get_last_error(), "Handle-list attribute failed")

        startup = _StartupInfoExW()
        startup.startup_info.cb = ctypes.sizeof(_StartupInfoExW)
        startup.startup_info.flags = 0x00000100
        startup.startup_info.stdin = stdin_handle
        startup.startup_info.stdout = stdout_write
        startup.startup_info.stderr = stderr_write
        startup.attribute_list = ctypes.cast(attribute_buffer, ctypes.c_void_p)
        process_information = _ProcessInformation()
        command_line = ctypes.create_unicode_buffer(subprocess.list2cmdline(args))
        environment_block = ctypes.create_unicode_buffer(
            "\0".join(f"{key}={value}" for key, value in sorted(env.items())) + "\0\0"
        )
        if not kernel32.CreateProcessW(
            None,
            command_line,
            None,
            None,
            True,
            0x00000004 | 0x00000400 | 0x00080000,
            environment_block,
            str(cwd),
            ctypes.byref(startup),
            ctypes.byref(process_information),
        ):
            raise OSError(ctypes.get_last_error(), "CreateProcessW failed")
        process_handle = int(process_information.process or 0)
        thread_handle = int(process_information.thread or 0)
        if not _process_is_in_job(process_handle, job_handle):
            raise OSError("CreateProcessW returned without atomic Job membership")
        _close_handle(thread_handle)
        thread_handle = 0
        for handle in (stdout_write, stderr_write, stdin_handle):
            _close_handle(handle)
            handles.remove(handle)
        handles.remove(stdout_read)
        stdout_stream = _text_stream_from_handle(msvcrt, stdout_read)
        handles.remove(stderr_read)
        stderr_stream = _text_stream_from_handle(msvcrt, stderr_read)
        return (
            AtomicJobProcess(
                args=args,
                process_handle=process_handle,
                pid=int(process_information.process_id),
                stdout=stdout_stream,
                stderr=stderr_stream,
            ),
            job_handle,
        )
    except BaseException:
        cleanup_actions = (
            (lambda: _terminate_process_handle(process_handle))
            if process_handle
            else None,
            (lambda: _close_handle(process_handle)) if process_handle else None,
            (lambda: _close_handle(thread_handle)) if thread_handle else None,
            stdout_stream.close if stdout_stream is not None else None,
            stderr_stream.close if stderr_stream is not None else None,
            lambda: close_process_job(job_handle),
        )
        for cleanup in cleanup_actions:
            if cleanup is None:
                continue
            try:
                cleanup()
            except BaseException:
                continue
        raise
    finally:
        if attribute_list_initialized and attribute_buffer is not None:
            kernel32.DeleteProcThreadAttributeList(attribute_buffer)
        for handle in handles:
            if handle:
                _close_handle(handle)


def resume_popen_process(process: subprocess.Popen[object] | AtomicJobProcess) -> None:
    if os.name != "nt":
        return
    process_handle = int(getattr(process, "_handle", 0) or 0)
    if not process_handle:
        raise OSError("Process did not retain its suspended Windows handle")
    ntdll = ctypes.windll.ntdll
    ntdll.NtResumeProcess.argtypes = [ctypes.c_void_p]
    ntdll.NtResumeProcess.restype = ctypes.c_long
    status = int(ntdll.NtResumeProcess(process_handle))
    if status != 0:
        raise OSError(status, "NtResumeProcess failed")


def close_process_job(handle: int | None) -> None:
    if handle:
        _close_handle(handle)


def terminate_and_close_process_job(
    handle: int | None, *, timeout_seconds: float = 10.0
) -> None:
    if not handle:
        return
    if os.name != "nt":
        close_process_job(handle)
        return
    kernel32 = ctypes.windll.kernel32
    kernel32.TerminateJobObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel32.TerminateJobObject.restype = ctypes.c_bool
    kernel32.QueryInformationJobObject.argtypes = [
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_void_p,
    ]
    kernel32.QueryInformationJobObject.restype = ctypes.c_bool
    try:
        if not kernel32.TerminateJobObject(handle, 1):
            raise OSError(ctypes.get_last_error(), "TerminateJobObject failed")
        deadline = time.monotonic() + timeout_seconds
        while True:
            accounting = _JobObjectBasicAccountingInformation()
            if not kernel32.QueryInformationJobObject(
                handle, 1, ctypes.byref(accounting), ctypes.sizeof(accounting), None
            ):
                raise OSError(
                    ctypes.get_last_error(), "QueryInformationJobObject failed"
                )
            if accounting.active_processes == 0:
                return
            if time.monotonic() >= deadline:
                raise TimeoutError("Benchmark Job Object did not become terminal")
            time.sleep(0.05)
    finally:
        close_process_job(handle)


def wait_for_process_job_terminal(
    handle: int | None, *, timeout_seconds: float = 120.0
) -> None:
    """Wait for every process in a retained Job Object without terminating it."""

    if not handle or os.name != "nt":
        return
    kernel32 = ctypes.windll.kernel32
    kernel32.QueryInformationJobObject.argtypes = [
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_void_p,
    ]
    kernel32.QueryInformationJobObject.restype = ctypes.c_bool
    deadline = time.monotonic() + timeout_seconds
    while True:
        accounting = _JobObjectBasicAccountingInformation()
        if not kernel32.QueryInformationJobObject(
            handle, 1, ctypes.byref(accounting), ctypes.sizeof(accounting), None
        ):
            raise OSError(
                ctypes.get_last_error(), "QueryInformationJobObject failed"
            )
        if accounting.active_processes == 0:
            return
        if time.monotonic() >= deadline:
            raise TimeoutError("Cargo Job Object did not become terminal")
        time.sleep(0.05)


def _create_kill_on_close_job() -> int:
    kernel32 = ctypes.windll.kernel32
    kernel32.CreateJobObjectW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p]
    kernel32.CreateJobObjectW.restype = ctypes.c_void_p
    kernel32.SetInformationJobObject.argtypes = [
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_uint32,
    ]
    kernel32.SetInformationJobObject.restype = ctypes.c_bool
    kernel32.SetHandleInformation.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
    ]
    kernel32.SetHandleInformation.restype = ctypes.c_bool
    handle = int(kernel32.CreateJobObjectW(None, None) or 0)
    if not handle:
        raise OSError(ctypes.get_last_error(), "CreateJobObjectW failed")
    try:
        information = _JobObjectExtendedLimitInformation()
        information.basic_limit_information.limit_flags = 0x00002000
        if not kernel32.SetInformationJobObject(
            handle, 9, ctypes.byref(information), ctypes.sizeof(information)
        ):
            raise OSError(ctypes.get_last_error(), "SetInformationJobObject failed")
        if not kernel32.SetHandleInformation(handle, 0x00000001, 0):
            raise OSError(ctypes.get_last_error(), "SetHandleInformation failed")
        return handle
    except BaseException:
        _close_handle(handle)
        raise


def _create_pipe(kernel32, inheritable: _SecurityAttributes, handles: list[int]) -> tuple[int, int]:
    read_handle = ctypes.c_void_p()
    write_handle = ctypes.c_void_p()
    if not kernel32.CreatePipe(
        ctypes.byref(read_handle), ctypes.byref(write_handle), ctypes.byref(inheritable), 0
    ):
        raise OSError(ctypes.get_last_error(), "CreatePipe failed")
    read_value = int(read_handle.value or 0)
    write_value = int(write_handle.value or 0)
    handles.extend((read_value, write_value))
    if not kernel32.SetHandleInformation(read_value, 0x00000001, 0):
        raise OSError(ctypes.get_last_error(), "SetHandleInformation failed")
    return read_value, write_value


def _open_inheritable_null(kernel32, inheritable: _SecurityAttributes) -> int:
    handle = int(
        kernel32.CreateFileW(
            "NUL",
            0x80000000,
            0x00000001 | 0x00000002,
            ctypes.byref(inheritable),
            3,
            0x00000080,
            None,
        )
        or 0
    )
    if not handle or handle == ctypes.c_void_p(-1).value:
        raise OSError(ctypes.get_last_error(), "CreateFileW(NUL) failed")
    return handle


def _text_stream_from_handle(msvcrt, handle: int) -> io.TextIOWrapper:
    try:
        descriptor = msvcrt.open_osfhandle(handle, os.O_RDONLY)
    except BaseException:
        _close_handle(handle)
        raise
    try:
        return io.open(
            descriptor, "r", encoding="utf-8", errors="replace", newline=None
        )
    except BaseException:
        os.close(descriptor)
        raise


def _process_is_in_job(process_handle: int, job_handle: int) -> bool:
    kernel32 = ctypes.windll.kernel32
    kernel32.IsProcessInJob.argtypes = [
        ctypes.c_void_p,
        ctypes.c_void_p,
        ctypes.POINTER(ctypes.c_int32),
    ]
    kernel32.IsProcessInJob.restype = ctypes.c_bool
    in_job = ctypes.c_int32()
    if not kernel32.IsProcessInJob(
        process_handle, job_handle, ctypes.byref(in_job)
    ):
        raise OSError(ctypes.get_last_error(), "IsProcessInJob failed")
    return bool(in_job.value)


def _process_exit_code(handle: int) -> int:
    kernel32 = ctypes.windll.kernel32
    kernel32.GetExitCodeProcess.argtypes = [
        ctypes.c_void_p,
        ctypes.POINTER(ctypes.c_uint32),
    ]
    kernel32.GetExitCodeProcess.restype = ctypes.c_bool
    exit_code = ctypes.c_uint32()
    if not kernel32.GetExitCodeProcess(handle, ctypes.byref(exit_code)):
        raise OSError(ctypes.get_last_error(), "GetExitCodeProcess failed")
    value = int(exit_code.value)
    return value if value < 0x80000000 else value - 0x100000000


def _terminate_process_handle(handle: int) -> None:
    kernel32 = ctypes.windll.kernel32
    kernel32.TerminateProcess.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel32.TerminateProcess.restype = ctypes.c_bool
    if not kernel32.TerminateProcess(handle, 1):
        exit_code = _process_exit_code(handle)
        if exit_code == 259:
            raise OSError(ctypes.get_last_error(), "TerminateProcess failed")


def _close_handle(handle: int) -> None:
    kernel32 = ctypes.windll.kernel32
    kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
    kernel32.CloseHandle.restype = ctypes.c_bool
    kernel32.CloseHandle(handle)


def _declare_create_process_functions(kernel32) -> None:
    kernel32.CreatePipe.argtypes = [
        ctypes.POINTER(ctypes.c_void_p),
        ctypes.POINTER(ctypes.c_void_p),
        ctypes.POINTER(_SecurityAttributes),
        ctypes.c_uint32,
    ]
    kernel32.CreatePipe.restype = ctypes.c_bool
    kernel32.SetHandleInformation.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
    ]
    kernel32.SetHandleInformation.restype = ctypes.c_bool
    kernel32.CreateFileW.argtypes = [
        ctypes.c_wchar_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.POINTER(_SecurityAttributes),
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.c_void_p,
    ]
    kernel32.CreateFileW.restype = ctypes.c_void_p
    kernel32.InitializeProcThreadAttributeList.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.POINTER(ctypes.c_size_t),
    ]
    kernel32.InitializeProcThreadAttributeList.restype = ctypes.c_bool
    kernel32.UpdateProcThreadAttribute.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_size_t,
        ctypes.c_void_p,
        ctypes.c_size_t,
        ctypes.c_void_p,
        ctypes.c_void_p,
    ]
    kernel32.UpdateProcThreadAttribute.restype = ctypes.c_bool
    kernel32.DeleteProcThreadAttributeList.argtypes = [ctypes.c_void_p]
    kernel32.CreateProcessW.argtypes = [
        ctypes.c_wchar_p,
        ctypes.c_wchar_p,
        ctypes.c_void_p,
        ctypes.c_void_p,
        ctypes.c_bool,
        ctypes.c_uint32,
        ctypes.c_void_p,
        ctypes.c_wchar_p,
        ctypes.POINTER(_StartupInfoExW),
        ctypes.POINTER(_ProcessInformation),
    ]
    kernel32.CreateProcessW.restype = ctypes.c_bool

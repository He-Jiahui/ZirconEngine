"""Bounded cancellation of this task's read-only scanner; keep its original owner."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import json
from pathlib import Path

from .paths import digest, file_fact, physical


def cancel_scan(authority, payload, report):
    original = authority.state.get_operation(payload["originalOperationId"])
    if not original or original["kind"] != "host_governance" or original["status"] != "running":
        raise ValueError("original_running_scan_required")
    request = original["payload"]["request"]
    if request["action"] != "scan" or request["taskId"] != authority.runtime["taskId"]:
        raise ValueError("only_this_task_readonly_scan_can_be_cancelled")
    evidence = physical(Path("E:/cargo-targets/zircon-local/reports") / request["taskId"] / request["operationId"] / "native-scan/started.json")
    facts = file_fact(evidence)
    if facts["sha256"] != payload["startedSha256"]:
        raise ValueError("original_native_scan_evidence_changed")
    started = json.loads(evidence.read_text(encoding="utf-8"))
    command = started["command"]
    if "tools.jenkins.governance.scan_child" not in command or str(evidence.parent.parent / "inventory.json") not in command:
        raise ValueError("original_native_scan_command_mismatch")
    from tools.jenkins.processes.identity import current_identity
    # The original waiter stays alive to settle the retained Job, descendants and pipe EOF.
    owner = original["result"]["processIdentity"]
    if current_identity(owner["pid"]).to_dict()["creationTime"] != owner["creationTime"]:
        raise ValueError("original_native_scan_owner_unavailable")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    api.OpenProcess.restype = wintypes.HANDLE
    api.GetProcessTimes.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.FILETIME), ctypes.POINTER(wintypes.FILETIME),
                                   ctypes.POINTER(wintypes.FILETIME), ctypes.POINTER(wintypes.FILETIME)]
    api.GetProcessTimes.restype = wintypes.BOOL
    api.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
    api.TerminateProcess.restype = wintypes.BOOL
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = api.OpenProcess(0x100001 | 0x1000, False, started["pid"])
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        times = [wintypes.FILETIME() for _ in range(4)]
        if not api.GetProcessTimes(handle, *[ctypes.byref(item) for item in times]):
            raise ctypes.WinError(ctypes.get_last_error())
        birth = str((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime)
        if birth != started["creationTime"]:
            raise ValueError("original_native_scan_pid_reused")
        value = {"status": "cancellation-requested", "originalOperationId": request["operationId"],
                 "originalRequestId": request["requestId"], "originalAttemptId": request["attemptId"],
                 "started": facts, "reason": payload["reason"], "terminalProof": "pending-original-waiter", **authority.runtime}
        authority.put("cancellation", request["operationId"], value, expected_version=0)
        if not api.TerminateProcess(handle, 1223):
            raise ctypes.WinError(ctypes.get_last_error())
        return value
    finally:
        api.CloseHandle(handle)

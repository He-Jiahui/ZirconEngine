"""Failure recovery for historical anonymous jobs across a real OS restart."""
from __future__ import annotations

import ctypes
import json
import os
from pathlib import Path
import subprocess
import time

from ..contracts import JenkinsError, digest


class _TimeOfDay(ctypes.Structure):
    _fields_ = [("BootTime", ctypes.c_int64), ("CurrentTime", ctypes.c_int64), ("TimeZoneBias", ctypes.c_int64),
                ("TimeZoneId", ctypes.c_uint32), ("Reserved", ctypes.c_uint32),
                ("BootTimeBias", ctypes.c_uint64), ("SleepTimeBias", ctypes.c_uint64)]


def _native_boot_require(value, message):
    if not value:
        raise JenkinsError("boot_evidence_unavailable", message)


def _query_time_of_day() -> dict:
    _native_boot_require(os.name == "nt", "Native Windows boot evidence requires Windows")
    _native_boot_require(ctypes.sizeof(_TimeOfDay) == 48 and tuple(getattr(_TimeOfDay, name).offset for name, _ in _TimeOfDay._fields_)
          == (0, 8, 16, 24, 28, 32, 40), "Native Windows time-of-day layout is unknown")
    try:
        # ntdll is a Windows KnownDLL already loaded in this native process.
        query = ctypes.WinDLL("ntdll.dll").NtQuerySystemInformation
        query.argtypes = [ctypes.c_int32, ctypes.c_void_p, ctypes.c_uint32, ctypes.POINTER(ctypes.c_uint32)]
        query.restype = ctypes.c_int32
        value, returned = _TimeOfDay(), ctypes.c_uint32()
        status = query(3, ctypes.byref(value), ctypes.sizeof(value), ctypes.byref(returned))
    except (OSError, AttributeError, ctypes.ArgumentError) as error:
        raise JenkinsError("boot_evidence_unavailable", "Native Windows boot query could not be completed") from error
    return {"status": status, "returnedLength": returned.value, "bootFileTime": value.BootTime, "currentFileTime": value.CurrentTime}


def windows_native_boot_observation() -> dict:
    samples = [_query_time_of_day(), _query_time_of_day()]
    fields = {"status", "returnedLength", "bootFileTime", "currentFileTime"}
    _native_boot_require(all(isinstance(s, dict) and set(s) == fields and all(type(s[k]) is int for k in fields)
              and s["status"] == 0 and s["returnedLength"] == 48 and 0 < s["bootFileTime"] <= s["currentFileTime"] < 2 ** 63
              for s in samples), "Native Windows boot status, byte length or FILETIME is invalid")
    _native_boot_require(samples[0]["bootFileTime"] == samples[1]["bootFileTime"], "Native Windows boot changed during observation")
    return {"schemaVersion": 1, "kind": "windows-native-boot-observation", "provider": "ntdll.NtQuerySystemInformation",
        "informationClass": 3, "queryCount": 2, "bufferBytes": 48, "returnedBytes": 48, "ntStatus": 0,
        "bootFileTime": str(samples[0]["bootFileTime"]), "observedAt": time.time()}


def windows_boot_filetime() -> int:
    """Real repeated native observation; no WMI or inferred boot fallback."""
    return int(windows_native_boot_observation()["bootFileTime"])


def recover_previous_boot(state, execution_id: str):
    execution = state.get("execution", execution_id)
    host = state.get("execution_host", execution_id)
    launch = state.get("execution_launch", execution_id)
    jobs = [r for r in state.list("native_job") if r["payload"].get("executionId") == execution_id]
    if not execution or not host or not launch or not jobs:
        raise JenkinsError("boot_recovery_identity_missing", "Recovery requires durable host, launch and native identities")
    boot = windows_boot_filetime()
    births = [host["payload"].get("hostBirthToken"), launch["payload"].get("launcherBirthToken")]
    births.extend(r["payload"].get("birthToken") for r in jobs)
    if any(not str(b).isdigit() or int(b) <= 0 or int(b) >= boot for b in births):
        raise JenkinsError("previous_boot_unproven", "Every owned process must predate the current Windows boot", retryable=True)
    evidence = {"kind": "windows-boot-boundary", "executionId": execution_id,
        "bootFileTime": str(boot), "birthTokens": [str(b) for b in births],
        "nativeJobIds": [r["key"] for r in jobs], "observedAt": time.time(),
        "outcome": "failed", "artifactPublicationAllowed": False}
    with state.transaction() as connection:
        current = state.get("execution", execution_id, connection=connection)
        if current["version"] != execution["version"]:
            raise JenkinsError("boot_recovery_raced", "Execution changed during boot recovery", retryable=True)
        state.put("boot_termination", execution_id, evidence, connection=connection)
        state.event("execution_previous_boot_terminated", {"executionId": execution_id,
                    "evidenceDigest": digest(evidence)}, connection=connection)
    return evidence


def boot_abort_evidence(state, execution_id: str):
    row = state.get("boot_termination", execution_id)
    if not row:
        return None
    evidence = row["payload"]
    if int(evidence["bootFileTime"]) > windows_boot_filetime():
        raise JenkinsError("boot_evidence_stale", "A boot recovery cannot refer to a future boot")
    jobs = [r for r in state.list("native_job") if r["payload"].get("executionId") == execution_id]
    if {r["key"] for r in jobs} != set(evidence["nativeJobIds"]):
        raise JenkinsError("boot_evidence_changed", "The native inventory changed after boot recovery")
    if any(int(r["payload"]["birthToken"]) >= int(evidence["bootFileTime"]) for r in jobs):
        raise JenkinsError("previous_boot_unproven", "A native process belongs to the current boot")
    return evidence

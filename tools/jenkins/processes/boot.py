"""Failure recovery for historical anonymous jobs across a real OS restart."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import time

from ..contracts import JenkinsError, digest


def windows_boot_filetime() -> int:
    if os.name != "nt":
        raise JenkinsError("boot_evidence_unavailable", "Windows boot evidence requires Windows")
    system = Path(os.environ["SystemRoot"])
    executable = system / "System32/WindowsPowerShell/v1.0/powershell.exe"
    script = "(Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime.ToUniversalTime().ToFileTimeUtc()"
    try:
        result = subprocess.run([str(executable), "-NoProfile", "-NonInteractive", "-Command", script],
            env=dict(os.environ, PSModulePath=str(system / "System32/WindowsPowerShell/v1.0/Modules")),
            capture_output=True, timeout=30, creationflags=subprocess.CREATE_NO_WINDOW)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise JenkinsError("boot_evidence_unavailable", "Windows boot evidence could not be queried") from error
    # Windows PowerShell diagnostics use the active console code page.  Only
    # ASCII decimal stdout is evidence; error text must never be UTF-8 decoded
    # by a subprocess reader thread or interpreted as a boot identity.
    output = result.stdout.strip()
    if result.returncode or not output.isdigit():
        raise JenkinsError("boot_evidence_unavailable", "Windows did not report a verifiable boot time")
    value = int(output)
    if value <= 0:
        raise JenkinsError("boot_evidence_unavailable", "Windows boot time is invalid")
    return value


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

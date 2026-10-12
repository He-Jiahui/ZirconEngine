"""Cancel one positively identified recovery failure before runtime creation.

The archived Stop remains unknown.  This capability retires only the empty
failed keeper; it neither terminates an old runtime nor issues historical EOF.
The reviewed failure implementation is deliberately a closed catalog.
"""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import time
import uuid

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..processes.boot import windows_boot_filetime
from ..state.atomic import replace_file
from ..state.locks import process_lock
from .driver import _plain
from .execution_activation import protected_holds
from .live_observations import observe_live_identity, _windows_command_line
from .owned_upgrade import _historical_release
from .recovery_keeper import (_archive, _archive_bytes, _raw, _read, _reference,
                              _restriction, _full_identity, _keeper_argv)
from .startup_observations import runtime_owner_inventory

KIND = "formal-pre-spawn-recovery-cancellation"
_ISSUED = object()
_FALSE = ("nativeProofIssued", "nativeTerminationProven", "historicalEofClaimed",
          "nativeProofSynthesized", "bootBoundaryClaimed", "runtimeAcceptance", "cargoAcceptance",
          "historicalHoldsReleased", "executionHoldsReleaseAllowed", "validationAcceptanceAllowed",
          "queueReplayRequested", "automaticRetry")
_POINTERS = {"originalCommand": "recovery-lifecycle/command.json",
             "originalDescriptor": "formal-lifecycle/recovery-keeper.json",
             "previousOperation": "deployment-operation.json", "previousHost": "deployment/host.json",
             "previousSelector": "deployment/driver.json"}
# Reviewed source proves this reason/message is thrown by plan.revalidate in
# consume / the manager's old-owner branch, before RecoveryStartTransaction.
# No missing PID, transaction, or createdSupervisor field supplies that proof.
_REVIEWED_FAILURE_CODE = {
    "035b26d30c6d8b57ae6dcd91ec4d5dd123db4199dba8be3302e3fd06401b20ba": {
        "tools/jenkins/deployment/recovery_keeper.py": "096847c6d600897b443fa5514359ad10d9e1cdf217740b841e940602536ad19c",
        "tools/jenkins/deployment/lost_recovery_runtime.py": "113839854cfcfc02e0ea0c26b94cff259b7d8bd7366dec6e6585aa3940765766",
        "tools/jenkins/deployment/manager.py": "c7559d298f6a2cc88874d3681e2cdf9530efec24e0fa4027f67396350fa93dca",
        "tools/jenkins/deployment/start_transaction.py": "771a5ee932888bde5b950e3102401a56fd483c8ffa8e369793f92a77202aaf4d",
        "tools/dev/jenkins_recovery_keeper.py": "edb2e852e15f094cc78ffe8f51eb178b011a2b7ea06891341d2e0ec3012be392",
    },
}


def _require(value, message, code="pre_spawn_cancel_unproven"):
    if not value:
        raise JenkinsError(code, message, retryable=True)


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _ref(path):
    return {"path": str(path), "sha256": _sha(_raw(path))}


def _local_reference(reference, parent):
    _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}, "Exact evidence reference required")
    path = Path(reference["path"]).absolute()
    _require(str(path) == reference["path"] and path.parent == parent and ".." not in path.parts
             and path.suffix == ".json" and _sha(_raw(path)) == require_digest(reference["sha256"]),
             "Failed holder evidence changed or escaped its namespace")
    return _read(path)


def _reviewed_implementation(repo, release, target):
    sources = _REVIEWED_FAILURE_CODE.get(target["driverDigest"])
    _require(sources is not None and release.get("actorCodeRoot") == target["root"],
             "Only an explicitly audited pre-creation failure implementation may be cancelled")
    for name, checksum in sources.items():
        path = (Path(target["root"]) if name.startswith("tools/jenkins/") else repo) / name
        _require(release.get("operatorFiles", {}).get(str(repo / name)) == checksum
                 and _sha(_raw(path)) == checksum, "The positively audited failure implementation changed")
    return sources


def _ledger(state):
    """Read logical rows through a fresh read-only snapshot, never State()."""
    path = state / "coordination.sqlite3"
    _plain(path)
    with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, isolation_level=None) as connection:
        connection.execute("PRAGMA query_only=ON")
        connection.execute("BEGIN")
        tables = connection.execute("SELECT name,sql FROM sqlite_master WHERE type='table' ORDER BY name").fetchall()
        allowed = {"metadata", "records", "requests", "operations", "authorizations", "events", "sqlite_sequence"}
        _require({row[0] for row in tables} <= allowed and "records" in {row[0] for row in tables},
                 "The coordination ledger schema is unreviewed")
        contents = {name: {"schema": schema, "rows": sorted(connection.execute('SELECT * FROM "' + name + '"').fetchall(), key=repr)}
                    for name, schema in tables}
        return digest(contents)


def _capture(manager, command_sha, holder_ref):
    """Positive failure evidence and all protected ownership pins."""
    repo, state = manager.paths.repo.absolute(), manager.paths.state.absolute()
    _require(state == repo / ".jenkins/state" and manager.paths.home.absolute() == repo / ".jenkins/jenkins_home",
             "Cancellation requires the existing formal deployment")
    path = state / _POINTERS["originalCommand"]
    command_raw, command = _raw(path), _read(path)
    _require(_sha(command_raw) == command_sha, "Failed command changed", "pre_spawn_cancel_changed")
    _require(command.get("schemaVersion") == 1 and command.get("kind") == "formal-lost-owner-recovery-command"
             and command.get("status") == "unknown" and command.get("reasonCode") == "lost_recovery_runtime_changed"
             and command.get("previousRuntimeReceipt") is None and "result" not in command
             and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False,
             "The exact initial pre-creation failed unknown command is required")
    holder = _local_reference(holder_ref, state / "recovery-lifecycle/holders")
    release, target = _historical_release(manager, command.get("release"))
    implementation = _reviewed_implementation(repo, release, target)
    _require(holder.get("schemaVersion") == 1 and holder.get("kind") == "formal-recovery-desktop-holder"
             and holder.get("status") == "recovery-blocked" and holder.get("reasonCode") == command["reasonCode"]
             and holder.get("message") == "Original interruption evidence changed"
             and holder.get("reviewedRelease") == command["release"] and holder.get("actorCodeRoot") == target["root"]
             and holder.get("recoveryDigest") == command.get("recoveryDigest")
             and holder.get("ownedUpgradeDigest") is None
             and all(holder.get(name) is False for name in ("runtimeAcceptance", "cargoAcceptance", "automaticRestart",
                                                           "requestsResubmitted", "serviceLifecycleExecuted")),
             "A matching positive sealed pre-creation fault publication is required")
    request = _reference(state, command.get("request"))
    _require(request.get("schemaVersion") == 1 and request.get("kind") == "formal-lost-owner-recovery-request"
             and request.get("action") == "recover" and request.get("requestId") == release.get("requestId")
             and request.get("keeper") == command.get("keeper") and request.get("release") == command["release"]
             and request.get("recoveryDigest") == command.get("recoveryDigest") and request.get("runtimeAcceptance") is False,
             "Only the original initial recover request may be cancelled")
    from .lost_recovery_runtime import reviewed_marker
    planned = reviewed_marker(state, release)
    _require(planned is not None and command.get("recoveryDigest") == digest(planned), "The complete reviewed plan changed")
    interruption = _reference(state, command.get("lostRecoveryRuntimeInterruption"))
    _require(interruption.get("schemaVersion") == 1 and interruption.get("kind") == "formal-lost-recovery-runtime-interruption"
             and interruption.get("outcome") == "interrupted" and interruption.get("originalCommandStatus") == "unknown"
             and interruption.get("keeper") == command.get("keeper") and interruption.get("release") == command["release"]
             and interruption.get("request") == command["request"] and interruption.get("recoveryDigest") == digest(planned)
             and interruption.get("reviewedPlan") == release["lostRecoveryRuntime"]["plan"]
             and interruption.get("operationId") == planned.get("operationId")
             and interruption.get("originalRequestId") == planned.get("originalRequestId")
             and str(interruption.get("bootFileTime")) == str(planned.get("bootFileTime")) == str(windows_boot_filetime())
             and all(interruption.get(name) is False for name in _FALSE), "The exact seven-archive interruption changed")
    originals = interruption.get("originalArtifacts", {})
    _require(set(originals) == set(_POINTERS) | {"originalRequest", "previousRuntimeReceipt"},
             "Every original ownership archive is required")
    archived, references = {}, {}
    for name, binding in originals.items():
        _require(isinstance(binding, dict) and set(binding) == {"original", "archive"}
                 and binding["original"] == planned.get(name)
                 and binding["archive"].get("sha256") == binding["original"]["sha256"], "Original archive binding changed")
        archived[name] = _reference(state, binding["archive"])
        references[name] = binding["archive"]
        if name in _POINTERS:
            _require(binding["original"]["path"] == str(state / _POINTERS[name]), "Original ownership path changed")
        if name != "originalCommand":
            _require(_raw(binding["original"]["path"]) == _raw(binding["archive"]["path"]),
                     "Original deployment ownership changed", "pre_spawn_cancel_changed")
    old = archived["originalCommand"]
    _require(command.get("replacesOriginalCommand") == references["originalCommand"]
             and command.get("replacesLostRecoveryDescriptor") == references["originalDescriptor"]
             and old.get("kind") == "formal-lost-owner-recovery-command" and old.get("status") == "unknown" and "result" not in old
             and old.get("request") == planned["originalRequest"] and old.get("previousRuntimeReceipt") == planned["previousRuntimeReceipt"]
             and archived["originalRequest"].get("action") == "stop"
             and archived["originalRequest"].get("requestId") == planned["originalRequestId"]
             and archived["originalDescriptor"].get("runtimeReceipt") == planned["previousRuntimeReceipt"],
             "The original unknown Stop must be restored without rewriting its history")
    _require(_read(state / "recovery-lifecycle/requests" / (planned["originalRequestId"] + ".json")) == planned["originalRequest"],
             "The original pending Stop request changed")
    for request_id in (planned["originalRequestId"], release["requestId"]):
        _require(not (state / "recovery-lifecycle/results" / (request_id + ".json")).exists(), "A result contradicts this pre-creation failure")
    for transaction in (state / "recovery-lifecycle/start-transactions").glob("*.json"):
        value = _read(transaction)
        _require(value.get("keeper") != command["keeper"] and value.get("request") != command["request"],
                 "A recorded start contradicts the positively audited pre-creation fault")
    fence = release.get("expectedLifecycleFence", {})
    _require(fence == planned.get("lifecycleFence") and fence, "The complete previous lifecycle fence is required")
    for name, checksum in fence.items():
        _require((_sha(_raw(state / name)) if (state / name).exists() else None) == checksum,
                 "Previous lifecycle ownership changed", "pre_spawn_cancel_changed")
    holds = protected_holds(state)
    _require(len(holds) == 4 and holds == release.get("protectedHolds") == planned.get("executionQuarantine", {}).get("protectedHolds"),
             "All four original historical holds must remain exact")
    _restriction(holder.get("restrictionPreflight", {}), command["keeper"], release["config"])
    sandbox = holder.get("officialSandboxHost", {})
    host_record = _local_reference(sandbox.get("record"), state / "recovery-lifecycle/sandbox-hosts")
    pinned_host = release.get("sandboxHost", {})
    _require(host_record.get("kind") == "official-strict-recovery-sandbox-host"
             and host_record.get("release") == command["release"] and host_record.get("host") == sandbox.get("host")
             and host_record.get("profile") == pinned_host.get("profile") == "zircon-strict"
             and host_record.get("includeManagedConfig") is pinned_host.get("includeManagedConfig") is True
             and host_record.get("automaticRestart") is False and host_record.get("runtimeAcceptance") is False
             and sandbox.get("officialHostSha256") == pinned_host.get("sha256")
             and _sha(_raw(pinned_host["path"])) == pinned_host["sha256"], "Official strict sandbox evidence changed")
    dispatcher = host_record.get("dispatcher", {})
    _require(dispatcher.get("path") == str(repo / "tools/dev/start-jenkins-coordinator.ps1")
             and release.get("operatorFiles", {}).get(dispatcher["path"]) == dispatcher.get("sha256")
             and _sha(_raw(dispatcher["path"])) == dispatcher["sha256"], "The original dispatcher implementation changed")
    files = {str(item): _sha(_raw(item)) for directory in ("requests", "results", "start-transactions")
             for item in sorted((state / "recovery-lifecycle" / directory).glob("*.json"))}
    pins = {"command": command_sha, "holder": holder_ref, "release": command["release"], "plan": release["lostRecoveryRuntime"]["plan"],
            "interruption": command["lostRecoveryRuntimeInterruption"], "originalArchives": references,
            "lifecycleFence": fence, "protectedHolds": holds, "ledgerDigest": _ledger(state), "requestFiles": files,
            "implementation": implementation, "bootFileTime": str(planned["bootFileTime"])}
    return pins, command_raw, command, holder, host_record, release


def _kernel():
    _require(os.name == "nt", "Retained cancellation requires Windows native handles")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    signatures = {
        "OpenProcess": ([wintypes.DWORD, wintypes.BOOL, wintypes.DWORD], wintypes.HANDLE),
        "CloseHandle": ([wintypes.HANDLE], wintypes.BOOL),
        "GetProcessId": ([wintypes.HANDLE], wintypes.DWORD),
        "GetProcessTimes": ([wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4, wintypes.BOOL),
        "WaitForSingleObject": ([wintypes.HANDLE, wintypes.DWORD], wintypes.DWORD),
        "GetExitCodeProcess": ([wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)], wintypes.BOOL),
        "TerminateProcess": ([wintypes.HANDLE, wintypes.UINT], wintypes.BOOL),
        "IsProcessInJob": ([wintypes.HANDLE, wintypes.HANDLE, ctypes.POINTER(wintypes.BOOL)], wintypes.BOOL),
    }
    for name, (args, result) in signatures.items():
        function = getattr(api, name)
        function.argtypes, function.restype = args, result
    return api


def _times(api, handle):
    values = [wintypes.FILETIME() for _ in range(4)]
    _require(api.GetProcessTimes(handle, *(ctypes.byref(value) for value in values)), "Cannot query the retained process times")
    return tuple(str((value.dwHighDateTime << 32) | value.dwLowDateTime) for value in values)


def _parents():
    from tools.dev.jenkins_recovery_keeper import parent_pid
    return parent_pid


def _descendants(root_pid):
    """Current complete process tree; it does not substitute for a Job query."""
    class Entry(ctypes.Structure):
        _fields_ = [("size", wintypes.DWORD), ("usage", wintypes.DWORD), ("pid", wintypes.DWORD),
                    ("heap", ctypes.c_size_t), ("module", wintypes.DWORD), ("threads", wintypes.DWORD),
                    ("parent", wintypes.DWORD), ("priority", wintypes.LONG), ("flags", wintypes.DWORD),
                    ("name", wintypes.WCHAR * 260)]
    api = _kernel()
    api.CreateToolhelp32Snapshot.argtypes, api.CreateToolhelp32Snapshot.restype = [wintypes.DWORD, wintypes.DWORD], wintypes.HANDLE
    for name in ("Process32FirstW", "Process32NextW"):
        function = getattr(api, name)
        function.argtypes, function.restype = [wintypes.HANDLE, ctypes.POINTER(Entry)], wintypes.BOOL
    handle = api.CreateToolhelp32Snapshot(2, 0)
    _require(handle and handle != ctypes.c_void_p(-1).value, "Cannot inspect the failed attempt process tree")
    try:
        entry, parents = Entry(), {}
        entry.size = ctypes.sizeof(entry)
        available = api.Process32FirstW(handle, ctypes.byref(entry))
        while available:
            parents[int(entry.pid)] = int(entry.parent)
            available = api.Process32NextW(handle, ctypes.byref(entry))
        _require(ctypes.get_last_error() == 18 and root_pid in parents, "The failed attempt process tree is incomplete")
        result = {root_pid}
        while True:
            fresh = result | {pid for pid, parent in parents.items() if parent in result}
            if fresh == result:
                return sorted(result)
            result = fresh
    finally:
        api.CloseHandle(handle)


class _RetainedAttempt:
    """Only keeper has TERMINATE access. Every exit is a real handle wait."""
    def __init__(self, manager, command, holder, host_record, release, dispatcher_identity, *, runtime_review=None):
        self.api, self.manager, self.handles, self.identities = _kernel(), manager, {}, {}
        self.command, self.holder, self.host_record, self.release = command, holder, host_record, release
        if runtime_review is not None:
            from .runtime_roles import ReviewedRuntimeRoles, _ISSUED as runtime_issued
            _require(type(runtime_review) is ReviewedRuntimeRoles and runtime_review._token is runtime_issued,
                     "The retained attempt needs an exact issued foreign runtime review")
            runtime_review.observe(manager)
        self.runtime_review = runtime_review
        try:
            keeper, host = command["keeper"], host_record["host"]
            observe_live_identity(keeper)
            _keeper_argv(manager.paths.repo, keeper, command["release"], release["config"]["sha256"])
            args = list(_windows_command_line(host["pid"]))
            expected = [host["executable"], "sandbox", "-P", "zircon-strict", "--include-managed-config", "-C",
                        str(manager.paths.repo), "--", *_windows_command_line(keeper["pid"])]
            _require(host["executable"] == release["sandboxHost"]["path"] and args == expected == host_record.get("arguments"),
                     "The official host argv differs from the exact initial recovery")
            parent = _parents()
            runner_pid = parent(keeper["pid"])
            _require(holder["officialSandboxHost"].get("actualAncestry") == [runner_pid, host["pid"]]
                     and parent(runner_pid) == host["pid"] and parent(host["pid"]) == dispatcher_identity["pid"],
                     "The original official sandbox ancestry changed")
            runner = _full_identity(runner_pid)
            runner_args = list(_windows_command_line(runner_pid))
            _require(re.fullmatch(r"codex-command-runner-[A-Za-z0-9_.-]+\.exe", Path(runner["executable"]).name)
                     and len(runner_args) == 3
                     and Path(os.path.realpath(runner_args[0])) == Path(runner["executable"])
                     and re.fullmatch(r"--pipe-in=\\\\\.\\pipe\\codex-runner-[0-9a-f]{32}-in", runner_args[1])
                     and runner_args[2] == runner_args[1].replace("--pipe-in=", "--pipe-out=").removesuffix("-in") + "-out",
                     "The official sandbox runner role is unproven")
            job_pids = holder["restrictionPreflight"]["associatedJob"]["memberPids"]
            _require(len(job_pids) == 2 and len(set(job_pids)) == 2 and keeper["pid"] in job_pids,
                     "Only the keeper and its original console may occupy this empty attempt")
            console = _full_identity(next(pid for pid in job_pids if pid != keeper["pid"]))
            console_args = list(_windows_command_line(console["pid"]))
            console_path = Path(os.environ["SystemRoot"]) / "System32/conhost.exe"
            _require(Path(console["executable"]) == console_path and parent(console["pid"]) == keeper["pid"]
                     and len(console_args) == 2 and Path(console_args[0].removeprefix("\\??\\")) == console_path
                     and console_args[1] == "0x4", "The second strict Job member is not the keeper's bounded console")
            host_children = set(_descendants(host["pid"])) - {host["pid"], runner["pid"], keeper["pid"], console["pid"]}
            _require(len(host_children) == 1, "Only the official host's original console may join this bounded process tree")
            host_console = _full_identity(host_children.pop())
            host_console_args = list(_windows_command_line(host_console["pid"]))
            _require(Path(host_console["executable"]) == console_path and parent(host_console["pid"]) == host["pid"]
                     and len(host_console_args) == 2 and Path(host_console_args[0].removeprefix("\\??\\")) == console_path
                     and host_console_args[1] == "0x4"
                     and int(host["creationTime"]) <= int(host_console["creationTime"]) < int(runner["creationTime"]),
                     "The extra host descendant is not its original bounded console")
            dispatcher_args = list(_windows_command_line(dispatcher_identity["pid"]))
            _require(Path(dispatcher_identity["executable"]).name.casefold() in {"pwsh.exe", "powershell.exe"}
                     and dispatcher_args == [dispatcher_identity["executable"], "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                                              host_record["dispatcher"]["path"], "-RecoveryReleaseFile", command["release"]["path"]],
                     "The retained dispatcher argv differs from this exact failed attempt")
            actors = {"keeper": keeper, "console": console, "runner": runner, "officialHost": host,
                      "hostConsole": host_console, "dispatcher": dispatcher_identity}
            _require(len({value["pid"] for value in actors.values()}) == 6, "Failed attempt roles conflict")
            for role, identity in actors.items():
                access = 0x1000 | 0x00100000 | (1 if role == "keeper" else 0)
                handle = self.api.OpenProcess(access, False, identity["pid"])
                _require(handle, "An original failed-attempt process cannot be retained")
                self.handles[role], self.identities[role] = handle, dict(identity)
                self._verify(role)
            _require(int(dispatcher_identity["creationTime"]) < int(host["creationTime"]) < int(runner["creationTime"])
                     < int(keeper["creationTime"]) <= int(console["creationTime"]), "Failed process births do not match their roles")
            self.image_pins = {value["executable"]: _sha(_raw(value["executable"])) for value in actors.values()}
            stamp = Path(holder["officialSandboxHost"]["record"]["path"]).stem
            self.logs = [manager.paths.repo / ".jenkins/logs" / ("recovery-sandbox-" + stamp + "." + stream + ".log")
                         for stream in ("stdout", "stderr")]
            self.verify_scope()
        except BaseException:
            self.close()
            raise

    def _verify(self, role):
        handle, identity = self.handles[role], self.identities[role]
        _require(self.api.GetProcessId(handle) == identity["pid"] and _times(self.api, handle)[0] == str(identity["creationTime"])
                 and self.api.WaitForSingleObject(handle, 0) == 258, "The retained actor birth or liveness changed")
        observe_live_identity(identity)

    def verify_scope(self):
        for role in self.handles:
            self._verify(role)
        for name, checksum in self.image_pins.items():
            _require(_sha(_raw(name)) == checksum, "A retained failed-attempt executable changed")
        expected = sorted(self.holder["restrictionPreflight"]["associatedJob"]["memberPids"])
        membership = []
        for role in ("keeper", "console"):
            member = wintypes.BOOL()
            _require(self.api.IsProcessInJob(self.handles[role], None, ctypes.byref(member)) and member.value,
                     "An original strict actor escaped native Job containment")
            membership.append(self.identities[role]["pid"])
        descendants = _descendants(self.identities["officialHost"]["pid"])
        _require(descendants == sorted(value["pid"] for role, value in self.identities.items() if role != "dispatcher"),
                 "An extra process contradicts this empty failed attempt")
        boundary = None if getattr(self, "runtime_review", None) is None else self.runtime_review.observe(self.manager)
        inventory = (runtime_owner_inventory(self.manager.paths.repo) if boundary is None else boundary["blockingOwners"])
        _require(len(inventory) == 1 and all(str(inventory[0].get(key)) == str(self.identities["keeper"][key])
                                          for key in ("pid", "creationTime", "executable")), "Another runtime owner contradicts an empty failed attempt")
        result = {"processes": self.identities, "publishedAssociatedJobMemberPids": expected,
                "currentContainedActorPids": sorted(membership), "currentOfficialHostDescendants": descendants,
                "completeLiveJobInventoryClaimed": False, "executablePins": self.image_pins,
                "terminationScope": ["keeper"], "otherActorsExitNaturally": True}
        if boundary is not None:
            result["runtimeBoundary"] = boundary
        return result

    def terminate_keeper(self):
        self._verify("keeper")
        _require(self.api.TerminateProcess(self.handles["keeper"], 2), "Windows refused exact failed keeper retirement")

    def collect(self, timeout):
        deadline, exits = time.monotonic() + timeout, []
        for role in ("keeper", "console", "runner", "officialHost", "hostConsole", "dispatcher"):
            handle = self.handles[role]
            wait = self.api.WaitForSingleObject(handle, max(0, int((deadline - time.monotonic()) * 1000)))
            _require(wait == 0, "A retained attempt process has not actually exited", "pre_spawn_cancel_exit_unproven")
            code = wintypes.DWORD()
            _require(self.api.GetExitCodeProcess(handle, ctypes.byref(code)), "Actual retained exit code is unavailable")
            born, ended, _, _ = _times(self.api, handle)
            _require(born == str(self.identities[role]["creationTime"]) and int(ended) > int(born), "Actual retained exit time is unavailable")
            exits.append({"role": role, "identity": self.identities[role], "exitFileTime": ended,
                          "exitCode": int(code.value), "processHandleSignaled": True, "terminatedByCancellation": role == "keeper"})
        boundary = None if getattr(self, "runtime_review", None) is None else self.runtime_review.observe(self.manager)
        owners = runtime_owner_inventory(self.manager.paths.repo) if boundary is None else boundary["blockingOwners"]
        _require(owners == [], "A new runtime owner appeared during cancellation")
        return {"exits": exits, "logs": [_released_log(path) for path in self.logs]}

    def close(self):
        for handle in self.handles.values():
            self.api.CloseHandle(handle)
        self.handles.clear()


def _released_log(path):
    """An exclusive existing-file read proves these new logs were released."""
    import msvcrt
    _plain(path)
    api = _kernel()
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p,
                               wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    api.CreateFileW.restype = wintypes.HANDLE
    handle = api.CreateFileW(str(path), 0x80000000, 0, None, 3, 0x80, None)
    _require(handle and handle != ctypes.c_void_p(-1).value, "A new sandbox log is still held", "pre_spawn_cancel_logs_held")
    try:
        descriptor = msvcrt.open_osfhandle(int(handle), os.O_RDONLY | os.O_BINARY)
    except BaseException:
        api.CloseHandle(handle)
        raise
    checksum, size = hashlib.sha256(), 0
    with os.fdopen(descriptor, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(block)
            size += len(block)
    return {"path": str(path), "sha256": checksum.hexdigest(), "size": size,
            "exclusiveExistingRead": True, "probeHandleClosed": True, "historicalEofClaimed": False}


class PreSpawnRecoveryCancellation:
    """Issued capability retains its native handles across incomplete cleanup."""
    def __init__(self, manager, command_sha, holder_ref, snapshot, command_raw, restore_raw, native, scope, *, token=None):
        _require(token is _ISSUED, "Only the read-only cancellation factory may issue this capability")
        self._manager, self._command_sha, self._holder_ref = manager, command_sha, json.loads(canonical_json(holder_ref))
        self._snapshot, self._command_raw, self._restore_raw = snapshot, command_raw, restore_raw
        self._native, self._scope = native, json.loads(canonical_json(scope))
        self._token, self._terminated, self._complete, self._cleanup = token, False, False, None

    @property
    def document(self):
        return {"schemaVersion": 1, "kind": KIND, "phase": "pre-runtime-create", "pins": json.loads(self._snapshot),
                "nativeScope": json.loads(canonical_json(self._scope)), **{name: False for name in _FALSE}}

    def _unchanged(self):
        pins, *_ = _capture(self._manager, self._command_sha, self._holder_ref)
        _require(canonical_json(pins) == self._snapshot, "Protected cancellation evidence changed", "pre_spawn_cancel_changed")

    def cancel(self, *, timeout=30.0):
        _require(type(self) is PreSpawnRecoveryCancellation and self._token is _ISSUED and not self._complete,
                 "Cancellation authority cannot be constructed or replayed")
        _require(type(timeout) in {int, float} and 0 < timeout <= 60, "Cancellation needs a bounded positive native wait")
        state = self._manager.paths.state.absolute()
        with process_lock(state / "deployment-start.lock", reason="deployment_operation_busy"):
            self._unchanged()
            failed = _archive_bytes(state, "pre-spawn-recovery-failed-command", self._command_raw)
            intent = _archive(state, "pre-spawn-recovery-cancel-intent", {**self.document, "failedCommand": failed,
                             "restoresOriginalCommandSha256": _sha(self._restore_raw), "createdAt": time.time()})
            try:
                if not self._terminated:
                    _require(self._native.verify_scope() == self._scope, "The retained native scope changed")
                    self._unchanged()
                    self._native.terminate_keeper()
                    self._terminated = True
                self._cleanup = self._native.collect(timeout)
                self._unchanged()
                proof = _archive(state, "pre-spawn-recovery-cancel-cleanup", {**self.document, "intent": intent,
                                 "failedCommand": failed, "ownedEmptyAttemptCleanup": self._cleanup,
                                 "outcome": "ready-to-compensate-command", "observedAt": time.time()})
                command_path = state / _POINTERS["originalCommand"]
                pending = command_path.with_name(command_path.name + ".pending-" + uuid.uuid4().hex)
                _plain(pending)
                with pending.open("xb") as stream:
                    stream.write(self._restore_raw)
                    stream.flush()
                    os.fsync(stream.fileno())
                replace_file(pending, command_path, before_replace=self._unchanged)
                _require(_raw(command_path) == self._restore_raw, "Command compensation is unknown")
                self._complete = True
                self._native.close()
                return {"schemaVersion": 1, "kind": KIND, "outcome": "original-unknown-command-restored",
                        "failedCommand": failed, "cleanupProof": proof, "restoredCommand": _ref(command_path),
                        "originalCommandStatus": "unknown", **{name: False for name in _FALSE}}
            except BaseException as error:
                # A post-termination timeout can be retried with this same live
                # capability; its original handles remain retained. A new
                # factory never treats absent actors as cleanup evidence.
                _archive(state, "pre-spawn-recovery-cancel-incomplete", {**self.document, "intent": intent,
                         "failedCommand": failed, "keeperTerminationRequested": self._terminated,
                         "ownedEmptyAttemptCleanup": self._cleanup, "outcome": "compensation-not-confirmed",
                         "reasonCode": getattr(error, "code", type(error).__name__), "observedAt": time.time()})
                raise

    def close(self):
        """Release query handles without changing processes or State."""
        self._native.close()
        self._token = None


def prepare_pre_spawn_recovery_cancel(manager, *, expected_command_sha256, holder_reference, dispatcher_identity):
    """Read-only factory; the returned cancel() method is the sole mutator.

    The dispatcher pin must come from a live birth/executable/argv observation.
    Factory errors never terminate processes, archive evidence, or replace State.
    """
    checksum = require_digest(expected_command_sha256, "failedCommandSha256")
    pins, command_raw, command, holder, host_record, release = _capture(manager, checksum, holder_reference)
    native = _RetainedAttempt(manager, command, holder, host_record, release, dispatcher_identity)
    try:
        scope = native.verify_scope()
        fresh, *_ = _capture(manager, checksum, holder_reference)
        _require(fresh == pins, "Cancellation evidence changed while retaining native ownership", "pre_spawn_cancel_changed")
        archive = pins["originalArchives"]["originalCommand"]
        return PreSpawnRecoveryCancellation(manager, checksum, holder_reference, canonical_json(pins), command_raw,
                                            _raw(archive["path"]), native, scope, token=_ISSUED)
    except BaseException:
        native.close()
        raise


__all__ = ["PreSpawnRecoveryCancellation", "prepare_pre_spawn_recovery_cancel"]

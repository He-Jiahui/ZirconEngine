"""Windows-owned service recovery, independent of desktop clients and sandboxes.

Only lifecycle pointers are replaced. Interrupted execution records, reservations,
queues and historical native proofs are retained under their original identities.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from uuid import uuid4

from ..contracts import JenkinsError, digest
from ..processes.identity import current_identity, process_alive
from ..state.atomic import atomic_json
from ..state.locks import process_lock
from .driver import _plain, verify_driver
from .drain_observations import execution_inventory, classify_executions, owner_observation
from .lifecycle import check_port_available
from .paths import resolve_paths
from .spec import load_spec
from .startup_observations import (observe_departed_identity, observe_home_available,
                                   observe_runtime_owner_inventory, _native_process_snapshot,
                                   _system_process_image, _physical_dos_path)

_ISSUER = object()
_FIELDS = ("pid", "creationTime", "executable", "commandDigest")


def _require(ok, message, code="independent_service_unproven"):
    if not ok:
        raise JenkinsError(code, message, retryable=True)


def _raw(path):
    path = Path(path)
    _plain(path)
    _require(path.is_absolute() and ".." not in path.parts and path.resolve() == path,
             "Service inputs must use their physical absolute paths")
    before = path.stat()
    _require(path.is_file() and before.st_nlink == 1, "Service inputs must be ordinary unaliased files")
    raw = path.read_bytes()
    after = path.stat()
    _require((before.st_ino, before.st_mtime_ns, before.st_size) ==
             (after.st_ino, after.st_mtime_ns, after.st_size), "Service input changed while reading")
    return raw


def _read(path):
    value = json.loads(_raw(path))
    _require(isinstance(value, dict), "Service evidence must be an object")
    return value


def _sha(path):
    return hashlib.sha256(_raw(path)).hexdigest()


def _identity(value):
    _require(isinstance(value, dict) and type(value.get("pid")) is int and value["pid"] > 0
             and str(value.get("creationTime", "")).isdecimal()
             and bool(value.get("executable")) and len(value.get("commandDigest", "")) == 64,
             "Service ownership needs a complete native identity")
    return {key: value[key] for key in _FIELDS}


def _snapshot(manager):
    state = manager.paths.state
    operation = _read(manager.journal)
    host = _read(state / "deployment/host.json")
    selected = _read(state / "deployment/driver.json")
    runtime = operation.get("operationId")
    failed_before_agent = False
    stopped_with_proofs = (host.get("status") == "stopped"
                           and host.get("nativeTerminationProof", {}).get("complete") is True
                           and host.get("agentTerminationProof", {}).get("complete") is True)
    current = state / "independent-service/current.json"
    if (host.get("status") == "failed" and operation.get("state") in {"starting", "failed"}
            and host.get("agentLaunchAttempted") is False and not host.get("agentIdentity")
            and current.exists()):
        attempt = _read(current)
        failed_before_agent = (attempt.get("phase") in {"created", "failed"} and attempt.get("operationId") == runtime
                               and attempt.get("identity") == _identity(operation)
                               and (operation.get("state") == "failed" or
                                    (host.get("nativeTerminationProof", {}).get("complete") is True
                                     and host["nativeTerminationProof"].get("identity") == host.get("identity"))))
    _require(runtime and runtime == operation.get("generation") == host.get("operationId")
             == host.get("generation") == selected.get("runtimeOperationId")
             and operation.get("pid") == operation.get("hostPid") == host.get("hostPid")
             and operation.get("component") == "lifecycle-host"
             and operation.get("homePath") == str(manager.paths.home.absolute())
             and operation.get("controlPlaneOnly") is False
             and (host.get("controlPlaneOnly") is False or failed_before_agent or stopped_with_proofs),
             "The exact predecessor must be a bound full Jenkins runtime")
    _require(isinstance(host.get("agentIdentity"), dict) or failed_before_agent or stopped_with_proofs,
             "Missing agent identity needs an exact failed independent start")
    return {"operation": operation, "host": host, "selected": selected}


def _owner(snapshot):
    operation, host, selected = (snapshot[key] for key in ("operation", "host", "selected"))
    return {"operationId": operation["operationId"], "generation": operation["generation"],
            "driverDigest": selected["driverDigest"], "homePath": operation["homePath"],
            "supervisor": _identity(operation),
            "controller": _identity(host.get("controllerIdentity") or host.get("identity")),
            "agent": (_identity(host["agentIdentity"]) if host.get("agentIdentity") else
                      _identity(host["agentTerminationProof"]["identity"])
                      if host.get("agentTerminationProof", {}).get("identity") else None)}


def verify_policy(manager, path):
    path = Path(path).absolute()
    _require(path == manager.paths.state / "independent-service/policy.json", "Unexpected service policy path")
    policy = _read(path)
    _require(policy.get("schemaVersion") == 1 and policy.get("kind") == "windows-independent-jenkins-service"
             and policy.get("repository") == str(manager.paths.repo)
             and policy.get("homePath") == str(manager.paths.home)
             and policy.get("automaticRestart") is True and policy.get("clientLifetimeDependency") is False
             and policy.get("historicalHoldsReleaseAllowed") is False
             and policy.get("authorization", {}).get("independentWindowsHosting") is True,
             "Independent service authorization does not bind this deployment")
    # Installation hashes are audit evidence, not permanent startup gates.
    # Runtime paths, exclusive ownership and actual health determine startup;
    # live job/spec configuration and unrelated source edits remain flexible.
    return policy, _sha(path)


def _pending_without_launch(rows, execution_id):
    """Pending intake can remain open; an ambiguous native spawn cannot be adopted."""
    execution = next(row for row in rows if row["domain"] == "execution" and row["key"] == execution_id)
    _require(execution["payload"].get("status") in {"requested", "queued", "preparing", "prepared"},
             "Interrupted execution requires its native recovery owner", "independent_execution_pending")
    for row in rows:
        payload = row["payload"]
        if row["domain"] in {"execution_launch", "execution_host", "native_job"}:
            _require(not (row["key"] == execution_id or payload.get("executionId") == execution_id
                          or payload.get("owner") == execution_id),
                     "Pending execution has native launch evidence; keep its recovery open",
                     "independent_execution_pending")


def capture_replacement(manager, policy, policy_sha):
    snapshot = _snapshot(manager)
    selected = snapshot["selected"]
    root = manager.paths.runtime / "drivers" / selected["driverDigest"]
    _require(Path(selected["root"]) == root, "Selected driver escaped the service runtime")
    verify_driver(root, expected_digest=selected["driverDigest"])
    service_root = manager.paths.state / "independent-service"
    current = service_root / "current.json"
    if current.exists():
        attempt = _read(current)
        if attempt.get("phase") == "reconciled-departure":
            _require(attempt.get("policySha256") == policy_sha and attempt.get("snapshotDigest") == digest(snapshot)
                     and _sha(Path(attempt["originalAttempt"]["path"])) == attempt["originalAttempt"]["sha256"],
                     "Reconciled interrupted start evidence changed")
        else:
            _require(attempt.get("policySha256") in {policy_sha, *policy.get("previousPolicySha256", [])} and attempt.get("operationId")
                     == snapshot["operation"]["operationId"] and attempt.get("phase") in
                     {"created", "running", "failed", "unknown"} and isinstance(attempt.get("identity"), dict)
                     and _identity(attempt["identity"]) == _identity(snapshot["operation"]),
                     "An unfinished or foreign start must be reconciled under its original identity")
    else:
        _require(digest(snapshot) == policy["predecessorDigest"], "Migration predecessor changed")
    owner = _owner(snapshot)
    native = _native_process_snapshot()
    departed = {}
    for role in ("supervisor", "controller", "agent"):
        expected = owner[role]
        if expected is None:
            continue
        actual = native.get(expected["pid"])
        _require(actual is None or str(actual["creationTime"]) != str(expected["creationTime"]),
                 "The previous Jenkins " + role + " is still alive", "deployment_owner_live")
        departed[role] = {"pid": expected["pid"], "expectedCreationTime": expected["creationTime"],
                          "status": "absent" if actual is None else "pid-reused"}
    # Service availability does not accept, replay or release any business work.
    # Its gates are the old process departure, usable Home and available port.
    home = observe_home_available(manager.paths.home)
    check_port_available(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]))
    _require(_snapshot(manager) == snapshot, "Lifecycle changed during replacement observation")
    return {"kind": "independent-native-service-interruption", "policySha256": policy_sha,
            "snapshot": snapshot, "operationDigest": digest(snapshot["operation"]),
            "hostRecordDigest": digest(snapshot["host"]), "selectedDriverDigest": digest(selected),
            "departures": departed, "home": home,
            "retainedPendingExecutions": list(policy.get("retainedPendingExecutions", [])),
            "businessRecordsModified": False, "protectedHolds": [], "nativeProofIssued": False,
            "executionHoldsReleaseAllowed": False, "validationAcceptanceAllowed": False}



class IndependentServiceStart:
    """One consumed capability from a locked, fresh native replacement observation."""
    def __init__(self, issuer, manager, policy, policy_sha, evidence):
        _require(issuer is _ISSUER, "Service capabilities must be issued by the public lifecycle entry")
        self.manager, self.policy, self.policy_sha = manager, policy, policy_sha
        self.evidence, self.consumed = evidence, False
        self.root = manager.paths.state / "independent-service"
        self.attempt = {"schemaVersion": 1, "attemptId": uuid4().hex, "phase": "prepared",
                        "policySha256": policy_sha, "evidenceDigest": digest(evidence),
                        "createdAt": time.time(), "nativeProofIssued": False}

    def consume(self, manager):
        _require(manager is self.manager and not self.consumed, "Service start authority was already consumed")
        self.revalidate(manager)
        self.consumed = True

    def revalidate(self, manager):
        _require(manager is self.manager, "Service manager changed")
        current = capture_replacement(manager, self.policy, self.policy_sha)
        # capture_replacement independently rejects every blocking native owner
        # on each observation. Ordinary request clients may come and go while
        # registration polls health; their diagnostic inventory is not a lock.
        fenced = lambda value: {key: item for key, item in value.items() if key != "runtimeInventoryDigest"}
        _require(fenced(current) == fenced(self.evidence), "Native replacement evidence changed before startup")
        self.evidence = current
        self.attempt["evidenceDigest"] = digest(current)
        return current

    def _write(self):
        atomic_json(self.root / "attempts" / (self.attempt["attemptId"] + ".json"), self.attempt)
        atomic_json(self.root / "current.json", self.attempt)

    def archive_predecessor(self, operation, host, evidence):
        original = operation
        for _ in range(32):
            previous = operation.get("deploymentRecoveryEvidence", {})
            if operation.get("state") != "interrupted" or previous.get("kind") != "independent-native-service-interruption":
                _require(operation is original, "Original interruption archive is missing")
                self.manager._archive_deployment(original, host, evidence)
                return
            record = {"operation": previous["snapshot"]["operation"], "host": previous["snapshot"]["host"],
                      "deploymentRecoveryEvidence": previous}
            path = self.manager.paths.state / "deployment/history" / (digest(operation["operationId"]) + "-" + digest(record) + ".json")
            _require(host == record["host"] and record["operation"].get("operationId") == original.get("operationId"),
                     "Original interruption archive changed")
            if path.exists():
                _require(_read(path) == record, "Original interruption archive changed")
                return
            # Repeated failures before CreateProcess can add an interruption
            # observation without creating another runtime generation. Follow
            # only observations retained by the original start attempt.
            attempts = self.manager.paths.state / "independent-service/attempts"
            _require(any(_read(item) == previous for item in attempts.glob("*-predecessor.json")),
                     "Intermediate interruption has no original attempt")
            operation = record["operation"]
        raise JenkinsError("independent_service_unproven", "Interruption history exceeds its fixed bound")

    def creation_flags(self):
        return _independent_task_creation_flags()

    def before_spawn(self, operation_id, args, selected):
        _require(self.consumed and self.attempt["phase"] == "prepared", "Service spawn intent cannot be replayed")
        self.attempt.update(phase="spawn-intent", operationId=operation_id, selectedDriver=selected,
                            commandDigest=digest(args), creator=current_identity(os.getpid()).to_dict())
        self.root.mkdir(parents=True, exist_ok=True)
        atomic_json(self.root / "attempts" / (self.attempt["attemptId"] + "-predecessor.json"), self.evidence)
        self._write()

    def host_command(self, operation_id, selected):
        """Only this issued native capability may preserve an existing Home."""
        _require(self.consumed and selected.get("driverDigest"),
                 "Independent Home preservation needs the selected immutable driver")
        return [str(self.manager.paths.repo / ".jenkins/runtime/python/python.exe"),
                str(self.manager.paths.repo / "tools/dev/jenkins_service_host.py"),
                "--spec", str(self.manager.spec.path), "--java", str(self.manager.java),
                "--war", str(self.manager.war), "--operation-id", operation_id,
                "--policy", str(self.root / "policy.json"), "--policy-sha256", self.policy_sha]

    def created(self, pid, identity=None):
        self.attempt.update(phase="created", pid=pid)
        if identity is not None:
            self.attempt["identity"] = identity
        self._write()

    def unknown(self, error):
        no_child = self.attempt.get("phase") == "spawn-intent" and isinstance(error, OSError)
        self.attempt.update(phase="spawn-failed-before-create" if no_child else "unknown",
                            reasonCode=getattr(error, "code", type(error).__name__),
                            winerror=getattr(error, "winerror", None), errno=getattr(error, "errno", None),
                            childCreated=False if no_child else None)
        self._write()

    def finish(self, operation):
        self.attempt.update(phase=operation.get("state", "unknown"), operation=operation)
        self._write()


def start_service(manager, policy_path):
    with process_lock(manager.paths.state / "deployment-start.lock", reason="deployment_start_busy"):
        return _start_service_locked(manager, policy_path)


def _start_service_locked(manager, policy_path):

    policy, checksum = verify_policy(manager, policy_path)
    current = manager.paths.state / "independent-service/current.json"
    if current.exists() and _read(current).get("phase") == "spawn-failed-before-create":
        _reconcile_missing_spawn_locked(manager, policy, checksum)
    evidence = capture_replacement(manager, policy, checksum)
    capability = IndependentServiceStart(_ISSUER, manager, policy, checksum, evidence)
    result = manager._start_locked(manager.paths.state / "deployment-start.lock",
                                   independent_service_start=capability)
    capability.finish(result)
    return result


def _reconcile_missing_spawn_locked(manager, policy, policy_sha):
    """Reconcile one exact failed spawn without inventing native termination/EOF."""
    root = manager.paths.state / "independent-service"
    attempt = _read(root / "current.json")
    attempt_path = root / "attempts" / (attempt["attemptId"] + ".json")
    if attempt.get("phase") == "reconciled-departure":
        snapshot = _snapshot(manager)
        _require(attempt == _read(attempt_path) and attempt["policySha256"] in
                 {policy_sha, *policy.get("previousPolicySha256", [])}
                 and attempt["snapshotDigest"] == digest(snapshot)
                 and _sha(Path(attempt["originalAttempt"]["path"])) == attempt["originalAttempt"]["sha256"],
                 "Reconciled start changed before policy refresh")
        for key in ("supervisor", "controller", "agent"):
            if _owner(snapshot)[key] is not None:
                observe_departed_identity(_owner(snapshot)[key])
        inventory = observe_runtime_owner_inventory(manager.paths.repo)
        _require(not inventory["blockingOwners"], "A native service owner may still be live")
        observe_home_available(manager.paths.home)
        check_port_available(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]))
        _require(_snapshot(manager) == snapshot, "Lifecycle changed during policy refresh")
        receipt = {**attempt, "attemptId": uuid4().hex, "policySha256": policy_sha,
                   "previousReconciliation": {"path": str(attempt_path), "sha256": _sha(attempt_path)},
                   "observedAt": time.time()}
        atomic_json(root / "attempts" / (receipt["attemptId"] + ".json"), receipt)
        atomic_json(root / "current.json", receipt)
        return receipt
    _require(attempt == _read(attempt_path) and attempt.get("phase") in {"unknown", "spawn-failed-before-create"}
             and not attempt.get("pid") and not attempt.get("identity"), "A created spawn needs its original owner")
    evidence_path = root / "attempts" / (attempt["attemptId"] + "-predecessor.json")
    evidence = _read(evidence_path)
    _require(digest(evidence) == attempt["evidenceDigest"] and evidence["policySha256"] == attempt["policySha256"],
             "Original start observation changed")
    accepted_policies = {policy_sha, *policy.get("previousPolicySha256", [])}
    _require(attempt["policySha256"] in accepted_policies, "This policy cannot reconcile the original start")
    previous = evidence["snapshot"]
    operation = _read(manager.journal)
    selected = _read(manager.paths.state / "deployment/driver.json")
    _require(operation == {**previous["operation"], "state": "interrupted", "deploymentRecoveryEvidence": evidence}
             and _read(manager.paths.state / "deployment/host.json") == previous["host"]
             and selected == attempt["selectedDriver"] and selected["runtimeOperationId"] == attempt["operationId"]
             and selected["driverDigest"] == previous["selected"]["driverDigest"],
             "Failed spawn no longer owns the exact lifecycle pointers")
    if selected["driverDigest"] != policy["driverDigest"]:
        desired = _read(root / "desired.json")
        handoff = _verify_initial_handoff(manager, root, desired)
        target = _read(handoff["planRef"]["path"]).get("targetDriver")
        _require(target is not None and target["driverDigest"] == selected["driverDigest"],
                 "Failed target spawn lost its genuine selection evidence")
    creator = attempt["creator"]
    if creator["pid"] == os.getpid():
        _require(attempt.get("phase") == "spawn-failed-before-create" and attempt.get("childCreated") is False
                 and (attempt.get("winerror") or attempt.get("errno")), "Live creator still owns an unknown spawn")
    else:
        observe_departed_identity(creator)
    departures = {key: observe_departed_identity(value) for key, value in _owner(previous).items()
                  if key in {"supervisor", "controller", "agent"} and value is not None}
    inventory = observe_runtime_owner_inventory(manager.paths.repo)
    _require(not inventory["blockingOwners"], "A native service owner may still be live")
    home = observe_home_available(manager.paths.home)
    check_port_available(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]))
    _require(_read(manager.journal) == operation and _read(manager.paths.state / "deployment/driver.json") == selected,
             "Failed start changed during native reconciliation")
    original = {"path": str(attempt_path), "sha256": _sha(attempt_path)}
    # The failed attempt remains byte-identical. Only its unspawned selection
    # is restored; the original runtime stays explicitly interrupted.
    atomic_json(manager.paths.state / "deployment/driver.json", previous["selected"])
    receipt = {"schemaVersion": 1, "attemptId": uuid4().hex, "phase": "reconciled-departure",
               "policySha256": policy_sha, "snapshotDigest": digest(_snapshot(manager)),
               "originalAttempt": original, "departures": departures, "home": home,
               "runtimeInventoryDigest": digest(inventory), "nativeProofIssued": False,
               "validationAcceptanceAllowed": False, "historicalHoldsReleaseAllowed": False,
               "observedAt": time.time()}
    atomic_json(root / "attempts" / (receipt["attemptId"] + ".json"), receipt)
    atomic_json(root / "current.json", receipt)
    return receipt


def supervise(manager, policy_path, *, interval=15, max_ticks=None, controls=False):
    """Health errors are durable observations, never reasons to discard service Jobs."""
    if controls:
        return supervise_controlled(manager, policy_path, interval=interval, max_ticks=max_ticks)
    root = manager.paths.state / "independent-service"
    with process_lock(root / "resident.lock", reason="independent_service_already_running"):
        resident = current_identity(os.getpid()).to_dict()
        ticks, previous = 0, None
        while max_ticks is None or ticks < max_ticks:
            ticks += 1
            try:
                operation = manager._read() or {}
                pid = operation.get("pid")
                alive = process_alive(pid) if type(pid) is int and pid > 0 else None
                if alive is True:
                    # A foreign/reused PID or an HTTP response alone never permits adoption.
                    owner = owner_observation(manager, allow_starting=True)
                    current = _read(root / "current.json")
                    _require(current.get("operationId") == owner["operationId"]
                             and current.get("identity") == owner["supervisor"], "Live runtime is not this service's generation")
                    health = manager.health()
                    status = {"status": "healthy" if health.get("ready") else "degraded",
                              "operationId": owner["operationId"], "health": health}
                elif alive is False:
                    result = start_service(manager, policy_path)
                    status = {"status": "healthy" if result.get("state") == "running" else "recovery-pending",
                              "operationId": result.get("operationId"), "health": result.get("healthProof", {})}
                else:
                    raise JenkinsError("independent_owner_unknown", "Cannot establish lifecycle owner departure", retryable=True)
            except Exception as error:
                status = {"status": "recovery-pending", "reasonCode": getattr(error, "code", type(error).__name__),
                          "message": str(error)}
            value = {"schemaVersion": 1, **status, "observedAt": time.time(), "resident": resident,
                     "clientLifetimeDependency": False, "automaticRestart": True}
            try:
                atomic_json(root / "status.json", value)
                if status != previous:
                    with (root / "events.jsonl").open("a", encoding="utf-8") as stream:
                        stream.write(json.dumps(value, sort_keys=True) + "\n")
                    print(json.dumps(value, sort_keys=True), flush=True)
                    previous = status
            except Exception as error:
                print(json.dumps({"status": "evidence-write-pending", "errorType": type(error).__name__}), flush=True)
            if max_ticks is None or ticks < max_ticks:
                time.sleep(interval)


def _independent_task_creation_flags():
    # Task Scheduler already supplies an independent native lifetime Job.
    # Its Job forbids breakaway; inheriting it is correct only after proving
    # that the resident was started by the Windows scheduler, not a client.
    if os.name == "nt":
        processes = _native_process_snapshot()
        parent = processes.get(processes.get(os.getpid(), {}).get("parentPid"), {})
        _require(parent.get("imageName", "").casefold() in {"svchost.exe", "taskeng.exe", "taskhostw.exe"},
                 "Standalone resident must be a native Task Scheduler child")
        # Limited process handles can be denied for SYSTEM parents. The
        # existing NT inventory still supplies exact birth and image paths.
        expected = Path(os.environ["SystemRoot"]) / "System32" / parent["imageName"]
        _require(os.path.normcase(_system_process_image(parent["pid"])) == _physical_dos_path(str(expected)),
                 "Native scheduler image is outside Windows System32")
        fresh = _native_process_snapshot()
        _require(parent.get("creationTime", 0) > 0
                 and fresh.get(os.getpid(), {}).get("parentPid") == parent["pid"]
                 and all(fresh.get(parent["pid"], {}).get(key) == parent.get(key)
                         for key in ("pid", "creationTime", "imageName")),
                 "Native scheduler identity changed during observation")
    return getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0) | getattr(subprocess, "CREATE_NO_WINDOW", 0)



_CONTROL_ACTIONS = frozenset({"start", "stop", "restart"})
_CONTROL_PROTOCOL = "independent-service-desired-state-v1"
_LOADED_SERVICE_SHA256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def _write_control_once(path, value):
    path = Path(path)
    _plain(path)
    raw = (json.dumps(value, sort_keys=True, indent=2) + "\n").encode("utf-8")
    if path.exists():
        _require(_raw(path) == raw, "Original service request bytes changed", "service_control_busy")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())


def request_service_action(root, action, request_id, expected):
    """Transport only: no native owner observation, service spawn or State write."""
    root = Path(root)
    _require(action in _CONTROL_ACTIONS and len(request_id) == 32
             and all(c in "0123456789abcdef" for c in request_id),
             "Invalid finite service control request")
    _plain(root)
    with process_lock(root / "control.lock", reason="service_control_busy"):
        heartbeat = _read(root / "status.json")
        _require(heartbeat.get("controlProtocol") == _CONTROL_PROTOCOL
                 and heartbeat.get("resident") == expected.get("resident")
                 and heartbeat.get("controlImplementation") == expected.get("implementation")
                 and heartbeat.get("operationId") == expected.get("operationId")
                 and heartbeat.get("driverDigest") == expected.get("driverDigest")
                 and heartbeat.get("homePath") == expected.get("homePath")
                 and -5 <= time.time() - heartbeat.get("observedAt", 0) <= 60,
                 "Resident control binding is unavailable", "service_control_pending")
        active_path = root / "control-active.json"
        active = _read(active_path) if active_path.exists() else {}
        if active.get("requestId"):
            _require(active["requestId"] == request_id, "Query the original active control request", "service_control_busy")
            return {"status": "pending", "requestId": request_id, "resubmitted": False}
        report = root / "reports" / request_id
        if (report / "result.json").exists():
            return _read(report / "result.json")
        _require(not (report / "request.json").exists(),
                 "Original control request must be queried without resubmission", "service_control_pending")
        atomic_json(report / "artifacts.json", {"schemaVersion": 1, "taskId": request_id,
                    "owner": "independent-service-control", "root": str(report),
                    "outputs": ["request.json", "runtime-before.json", "result.json", "native-helper-evidence"],
                    "dependencies": expected, "retention": "Keep original requests and every failure/native proof"})
        request = {"schemaVersion": 1, "protocol": _CONTROL_PROTOCOL, "requestId": request_id,
                   "action": action, "expected": expected, "createdAt": time.time()}
        _write_control_once(report / "request.json", request)
        request_raw = _raw(report / "request.json")
        ref = {"path": str(report / "request.json"),
               "sha256": hashlib.sha256(request_raw).hexdigest(), "bytes": len(request_raw)}
        atomic_json(active_path, {"requestId": request_id, "requestRef": ref})
        return {"status": "submitted", "requestId": request_id, "requestRef": ref, "resubmitted": False}


def _control_implementation():
    from . import job_provisioning
    return {"serviceSha256": _LOADED_SERVICE_SHA256,
            "jobProvisioningSha256": job_provisioning._LOADED_SERVICE_REBIND_SHA256}


def verify_control_registration(manager, policy_path):
    policy, _ = verify_policy(manager, policy_path)
    registration = policy.get("runtimeRebind")
    _require(isinstance(registration, dict) and registration.get("schemaVersion") == 1
             and registration.get("controlImplementation") == _control_implementation(),
             "Resident controls are not registered to these implementation bytes")
    from .job_provisioning import _service_ref_read
    for key in ("managementInstaller", "codex", "activationRuntimeBefore"):
        _service_ref_read(registration[key])
    _require(registration["managementInstaller"].get("sha256") == registration.get("reviewedRuntimeRebindInstallerSha256"),
             "Cross-runtime retained-prefix installer has no reviewed exact pin")
    return registration


def _controlled_observation(manager, resident, desired, result=None):
    operation = _read(manager.journal)
    selected = _read(manager.paths.state / "deployment/driver.json")
    runtime = operation["operationId"]
    _require(operation.get("generation") == runtime == selected.get("runtimeOperationId")
             and operation.get("homePath") == str(manager.paths.home.absolute())
             and operation.get("component") == "lifecycle-host" and operation.get("controlPlaneOnly") is False,
             "Service lifecycle no longer binds this Home")
    stopped = desired.get("desiredState") == "stopped" and desired.get("phase") == "completed"
    if stopped:
        # The existing public Stop validates the real old native proofs and bound supervisor departure.
        checked = manager._stop_locked()
        _require(checked == operation and checked.get("state") == "stopped"
                 and checked.get("stopBinding") and checked.get("stopResult"),
                 "Stopped service needs its actual owned Stop receipt")
        health = {"ready": False}
        state = "stopped"
    else:
        owner = owner_observation(manager, allow_starting=True)
        attempt = _read(manager.paths.state / "independent-service/current.json")
        _require(owner["operationId"] == runtime and attempt.get("operationId") == runtime
                 and attempt.get("identity") == owner["supervisor"],
                 "Runtime owner changed during status")
        health = manager.health()
        bindings = None
        if desired.get("phase") == "completed":
            from .job_provisioning import _service_ref_read, _SERVICE_REQUIRED_JOBS
            bindings = json.loads(_service_ref_read(desired["runtimeRebindRef"]))
            _require(bindings.get("status") == "runtime-bindings-verified"
                     and bindings.get("operationId") == runtime
                     and bindings.get("driverDigest") == selected["driverDigest"]
                     and set(bindings.get("jobs", {})) == set(_SERVICE_REQUIRED_JOBS)
                     and set(bindings.get("requiredJobs", [])) == set(_SERVICE_REQUIRED_JOBS),
                     "Runtime has no exact eleven-job binding receipt", "service_rebind_pending")
        state = "healthy" if health.get("ready") and bindings is not None else "rebind-pending"
    return {"schemaVersion": 1, "status": state, "operationId": runtime, "nativeTaskContextVerified": True,
            "driverDigest": selected["driverDigest"], "homePath": str(manager.paths.home.absolute()),
            "health": health, "resident": resident, "observedAt": time.time(),
            "desiredState": desired.get("desiredState"), "controlPhase": desired.get("phase"),
            "activeRequestId": desired.get("requestId") if desired.get("phase") != "completed" else None,
            "controlProtocol": _CONTROL_PROTOCOL, "controlImplementation": _control_implementation(),
            "lastResult": result, "clientLifetimeDependency": False, "automaticRestart": True,
            "nativeProofSynthesized": False}


def _control_report(root, request_id, dependencies):
    report = root / "reports" / request_id
    atomic_json(report / "artifacts.json", {"schemaVersion": 1, "taskId": request_id,
                "owner": "independent-service-control", "root": str(report),
                "outputs": ["request.json", "runtime-before.json", "result.json", "native-helper-evidence"],
                "dependencies": dependencies, "retention": "Keep original request, failure and native proofs"})
    return report


def _finish_controlled_start_locked(manager, registration, root, desired, before, output):
    from .job_provisioning import (rebind_runtime_preserving_jobs, capture_runtime_rebind,
                                  _service_ref, verify_runtime_rebind_inputs)
    if desired["phase"] != "rebind-pending":
        _require(desired["phase"] == "ready-to-start", "Original Start outcome needs reconciliation",
                 "service_control_unknown")
        verify_runtime_rebind_inputs(manager, registration, before, offline=True)
        desired.update(desiredState="running", phase="start-intent")
        atomic_json(root / "desired.json", desired)
        current = _start_service_locked(manager, root / "policy.json")
        _require(current.get("state") == "running", "Independent Start has not completed")
        desired.update(phase="rebind-pending", startedOperationId=current["operationId"])
        atomic_json(root / "desired.json", desired)
    current = _read(manager.journal)
    _require(current["operationId"] == desired["startedOperationId"], "Replacement runtime changed")
    rebind = rebind_runtime_preserving_jobs(manager, before, current, registration, output,
                                           target_driver=desired.get("targetDriver"))
    _require(manager.health().get("ready") is True, "Bound replacement is not healthy", "service_rebind_pending")
    # The next automatic recovery reuses bindings observed while this runtime is available.
    fresh_output = _control_report(root, desired["requestId"] + "-current", {"runtimeRebindRef": rebind})
    capture_runtime_rebind(manager, registration, fresh_output, preflight=False)
    desired.update(phase="completed", runtimeRebindRef=rebind,
                   runtimeBeforeRef=_service_ref(fresh_output / "runtime-before.json"))
    atomic_json(root / "desired.json", desired)
    result = {"schemaVersion": 1, "status": "completed", "action": desired["action"],
              "requestId": desired["requestId"], "operationId": current["operationId"],
              "runtimeRebindRef": rebind, "nativeProofSynthesized": False}
    _write_control_once(output / "result.json", result)
    return result


def _run_service_request_locked(manager, registration, root, resident, active):
    from .job_provisioning import (capture_runtime_rebind, _service_ref, _service_ref_read,
                                  verify_runtime_rebind_inputs)
    request_path = root / "reports" / active["requestId"] / "request.json"
    _require(active["requestRef"]["path"] == str(request_path)
             and _service_ref(request_path) == active["requestRef"], "Service request reference differs")
    request = _read(request_path)
    _require(request.get("protocol") == _CONTROL_PROTOCOL and request.get("schemaVersion") == 1
             and request.get("requestId") == active["requestId"] and request.get("action") in _CONTROL_ACTIONS,
             "Service command is not a finite owned request")
    output = request_path.parent
    if (output / "result.json").exists():
        return _read(output / "result.json")
    desired_path = root / "desired.json"
    previous = _read(desired_path)
    _verify_initial_handoff(manager, root, previous)
    current = _read(manager.journal)
    selected = _read(manager.paths.state / "deployment/driver.json")
    expected = request["expected"]
    _require(expected.get("resident") == resident and expected.get("implementation") == _control_implementation()
             and expected.get("homePath") == str(manager.paths.home.absolute())
             and expected.get("driverDigest") == selected["driverDigest"],
             "Service request targets another resident or deployment")
    action = request["action"]
    same_request = previous.get("requestId") == request["requestId"]
    if same_request and previous.get("phase") == "completed":
        if action == "stop":
            checked = manager._stop_locked()
            _require(checked["operationId"] == current["operationId"] and checked.get("stopBinding")
                     and checked.get("stopResult"), "Known Stop lost its native receipts")
            result = {"schemaVersion": 1, "status": "completed", "action": action,
                      "requestId": request["requestId"], "operationId": current["operationId"],
                      "stopBinding": current["stopBinding"], "stopResult": current["stopResult"],
                      "nativeProofSynthesized": False}
            _write_control_once(output / "result.json", result)
            return result
        before = json.loads(_service_ref_read(previous["previousRuntimeBeforeRef"]))
        return _finish_controlled_start_locked(manager, registration, root,
                    {**previous, "phase": "rebind-pending"}, before, output)
    if same_request and previous.get("phase") == "rebind-pending":
        before = json.loads(_service_ref_read(previous["previousRuntimeBeforeRef"]))
        return _finish_controlled_start_locked(manager, registration, root, previous, before, output)
    _require((same_request and previous.get("phase") == "waiting-drain")
             or previous.get("phase") == "completed", "Original lifecycle mutation needs reconciliation",
             "service_control_unknown")
    _require(expected.get("operationId") == current["operationId"] == selected["runtimeOperationId"],
             "Service request targets another runtime")
    if action == "start":
        _require(current.get("state") == "stopped" and previous.get("desiredState") == "stopped",
                 "Start requires the genuinely stopped runtime")
        checked = manager._stop_locked()  # Idempotent verification of real Stop/native receipts.
        _require(checked.get("stopBinding") and checked.get("stopResult"), "Owned Stop receipt is missing")
        before_ref = previous["runtimeBeforeRef"]
        before = json.loads(_service_ref_read(before_ref))
        verify_runtime_rebind_inputs(manager, registration, before, offline=True)
        desired = {"schemaVersion": 1, "requestId": request["requestId"], "action": action,
                   "desiredState": "running", "phase": "ready-to-start", "requestRef": active["requestRef"],
                   "previousRuntimeBeforeRef": before_ref, "initialHandoffRef": previous["initialHandoffRef"]}
    else:
        owner_observation(manager)
        before = capture_runtime_rebind(manager, registration, output)
        before_ref = _service_ref(output / "runtime-before.json")
        desired = {"schemaVersion": 1, "requestId": request["requestId"], "action": action,
                   "desiredState": "stopped", "phase": "stop-intent", "requestRef": active["requestRef"],
                   "previousRuntimeBeforeRef": before_ref, "initialHandoffRef": previous["initialHandoffRef"]}
        atomic_json(desired_path, desired)  # Suppress automatic restart before Stop.
        try:
            current = manager._stop_locked()
            _require(current.get("state") == "stopped" and current.get("stopBinding")
                     and current.get("stopResult"), "Owned Stop has no actual native receipts")
        except JenkinsError as error:
            # Only these failures precede the existing stop-command write.
            if error.code in {"drain_timeout", "drain_failed", "deployment_operation_busy"}:
                desired.update(phase="waiting-drain", reasonCode=error.code)
                atomic_json(desired_path, desired)
                return {"status": "waiting", "requestId": request["requestId"], "reasonCode": error.code}
            raise
        if action == "stop":
            desired.update(phase="completed", runtimeBeforeRef=before_ref)
            atomic_json(desired_path, desired)
            result = {"schemaVersion": 1, "status": "completed", "action": action,
                      "requestId": request["requestId"], "operationId": current["operationId"],
                      "stopBinding": current["stopBinding"], "stopResult": current["stopResult"],
                      "nativeProofSynthesized": False}
            _write_control_once(output / "result.json", result)
            return result
        desired["phase"] = "ready-to-start"
    atomic_json(desired_path, desired)
    return _finish_controlled_start_locked(manager, registration, root, desired, before, output)


class _RetainedResident:
    """One pre-opened native process; this is deliberately not a NativeJob."""
    def __init__(self, manager, expected, task, *, controlled=False):
        import ctypes
        from ctypes import wintypes
        from ..processes.identity import _kernel32
        from .live_observations import _windows_command_line, _digest_args
        self.ctypes, self.wintypes, self.api = ctypes, wintypes, _kernel32()
        self.api.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
        self.api.WaitForSingleObject.restype = wintypes.DWORD
        self.api.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
        self.api.TerminateProcess.restype = wintypes.BOOL
        self.handle = self.api.OpenProcess(0x1000 | 0x100000 | 1, False, expected["pid"])
        if not self.handle:
            raise OSError(ctypes.get_last_error(), "OpenProcess(resident) failed")
        try:
            birth, exited = self._times()
            size = wintypes.DWORD(32768)
            image = ctypes.create_unicode_buffer(size.value)
            if not self.api.QueryFullProcessImageNameW(self.handle, 0, image, ctypes.byref(size)):
                raise OSError(ctypes.get_last_error(), "QueryFullProcessImageNameW(resident) failed")
            python = manager.paths.repo / ".jenkins/runtime/python/python.exe"
            _require(str(birth) == str(expected["creationTime"]) and exited == 0
                     and Path(image.value) == Path(expected["executable"]) == python
                     and self.api.WaitForSingleObject(self.handle, 0) == 258,
                     "The retained resident is not the expected live root", "service_handoff_pending")
            args = _windows_command_line(expected["pid"])
            entry = manager.paths.repo / "tools/dev/jenkins_service.py"
            _require(len(args) == (9 if controlled else 8)
                     and (not controlled or args[8] == "--service-controls")
                     and Path(args[0]) == python and args[1] == "-B"
                     and Path(args[2]) == entry and args[3:5] == ("run", "--spec")
                     and Path(args[5]) == manager.paths.repo / ".jenkins/deployment-spec.json"
                     and args[6] == "--policy"
                     and Path(args[7]) == manager.paths.state / "independent-service/policy.json",
                     "Resident argv is not the original independent service task", "service_handoff_pending")
            processes = _native_process_snapshot()
            cursor = expected["pid"]
            visited = set()
            for _ in range(8):
                if cursor == task["enginePid"]:
                    break
                _require(cursor not in visited and cursor in processes,
                         "Resident task-engine ancestry is unknown", "service_handoff_pending")
                visited.add(cursor)
                cursor = processes[cursor].get("parentPid")
            _require(cursor == task["enginePid"] and cursor in processes
                     and self._times() == (birth, 0),
                     "Resident is not from the captured task instance", "service_handoff_pending")
            self.identity = {"pid": expected["pid"], "creationTime": str(birth),
                             "executable": image.value, "commandDigest": _digest_args(args)}
            self.task = task
        except Exception:
            self.close()
            raise

    def _times(self):
        values = [self.wintypes.FILETIME() for _ in range(4)]
        if not self.api.GetProcessTimes(self.handle, *(self.ctypes.byref(v) for v in values)):
            raise OSError(self.ctypes.get_last_error(), "GetProcessTimes(resident) failed")
        return tuple((v.dwHighDateTime << 32) | v.dwLowDateTime for v in values[:2])

    def finish(self, stop_attempt):
        # Caller has already verified actual owned controller/agent/host Stop.
        _require(stop_attempt.get("instanceStopAttempted") is True
                 and stop_attempt.get("instanceGuid") == self.task["instanceGuid"]
                 and stop_attempt.get("enginePid") == self.task["enginePid"],
                 "The exact Task instance Stop was not actually attempted", "service_handoff_pending")
        wait = self.api.WaitForSingleObject(self.handle, 5000)
        if wait == 0xFFFFFFFF:
            raise OSError(self.ctypes.get_last_error(), "WaitForSingleObject(resident) failed")
        forced = False
        if wait == 258:
            _require(self._times() == (int(self.identity["creationTime"]), 0),
                     "Original resident handle changed before controlled termination", "service_handoff_pending")
            if not self.api.TerminateProcess(self.handle, 0xE0000001):
                raise OSError(self.ctypes.get_last_error(), "TerminateProcess(owned resident) failed")
            forced = True
            wait = self.api.WaitForSingleObject(self.handle, 30000)
            if wait == 0xFFFFFFFF:
                raise OSError(self.ctypes.get_last_error(), "WaitForSingleObject(resident) failed")
        _require(wait == 0, "Original resident has no actual signalled process exit", "service_handoff_pending")
        birth, exited = self._times()
        code = self.wintypes.DWORD()
        if not self.api.GetExitCodeProcess(self.handle, self.ctypes.byref(code)):
            raise OSError(self.ctypes.get_last_error(), "GetExitCodeProcess(resident) failed")
        _require(birth == int(self.identity["creationTime"]) and exited > 0,
                 "Original resident exit times differ", "service_handoff_pending")
        return {"schemaVersion": 1, "kind": "windows-native-resident-process-exit",
                "identity": self.identity, "taskInstance": self.task,
                "taskInstanceStopAttempted": True, "taskStopHresult": stop_attempt["hresult"],
                "terminateRetainedHandleUsed": forced,
                "waitResult": wait, "rootExitCode": code.value, "exitTime": str(exited),
                "rootProcessTerminal": True, "jobProofIssued": False, "eofProofIssued": False}

    def close(self):
        if self.handle:
            handle, self.handle = self.handle, None
            if not self.api.CloseHandle(handle):
                raise OSError(self.ctypes.get_last_error(), "CloseHandle(resident) failed")



def _handoff_archive(manager, verification, name):
    """Read one already-verified native archive, preserving its actual Home binding."""
    from .job_provisioning import _service_ref_read
    rows = verification.get("archives", [])
    wanted = {"artifacts.json", "stdout.bin", "stderr.bin", "started.json",
              "native-terminal.json", "source-hashes.json", "completion-receipt.json",
              "fresh-physical-binding.json", "cleanup-plan.json", "cleanup.json", "summary.json"}
    by_name = {row["name"]: row for row in rows}
    _require(verification.get("status") == "passed" and verification.get("rootTerminal") is True
             and verification.get("jobResult") == "SUCCESS" and verification.get("nativeComplete") is True
             and verification.get("localApiHomeBytesEqual") is True
             and verification.get("missingArchives") == verification.get("unequalArchives") == []
             and len(rows) == 11 and set(by_name) == wanted
             and all(row.get("apiStatus") == 200 and row.get("bytesEqual") is True for row in rows),
             "Target driver needs its actual complete native publication", "service_handoff_pending")
    row = by_name[name]
    home = manager.paths.home / "jobs/zircon-management-tests/builds" / str(verification["build"]) / "archive" / name
    ref = {"path": row["local"], "sha256": row["sha256"], "bytes": row["bytes"]}
    raw = _service_ref_read(ref)
    _require(Path(row["home"]) == home
             and raw == _service_ref_read({**ref, "path": str(home)}),
             "Target driver archive differs from its actual Jenkins Home", "service_handoff_pending")
    return json.loads(raw)


def _verify_handoff_target_driver(manager, plan, source_pins):
    """An optional target is a real sealed publication, never a selector bypass."""
    from .job_provisioning import _service_ref_read
    target = plan.get("targetDriver")
    if target is None:
        _require(not any(key in plan for key in ("targetDriverQualificationRef", "targetDriverSourceRefs")),
                 "Target driver evidence has no target", "service_handoff_pending")
        return None
    keys = {"driverDigest", "root", "launcher", "steps", "manifest"}
    _require(isinstance(target, dict) and set(target) == keys
             and Path(target["root"]) == manager.paths.runtime / "drivers" / target["driverDigest"]
             and target == verify_driver(target["root"], expected_digest=target["driverDigest"]),
             "Target is not the exact published canonical driver", "service_handoff_pending")
    publication = json.loads(_service_ref_read(plan["targetDriverQualificationRef"]))
    receipt = _handoff_archive(manager, publication, "completion-receipt.json")
    manifest = _read(target["manifest"])
    refs = plan["targetDriverSourceRefs"]
    _require(receipt.get("targetDriver") == target
             and receipt.get("sourceQualificationRef") == plan["preparationTerminalRef"]
             and receipt.get("sourceRefs") == refs and isinstance(refs, dict) and len(refs) == 10
             and receipt.get("driverFiles") == manifest["files"]
             and receipt.get("manualSourceTreeCopiesCreated") == 0,
             "Target publication is not this exact qualified managed source", "service_handoff_pending")
    roots = set()
    for name, ref in refs.items():
        relative = Path(name)
        path = Path(ref["path"])
        _require(not relative.is_absolute() and ".." not in relative.parts
                 and source_pins.get(path) == {"sha256": ref["sha256"], "bytes": ref["bytes"]},
                 "Target input was not in the real source qualification", "service_handoff_pending")
        root = path
        for _ in relative.parts:
            root = root.parent
        _require(root / relative == path and root.is_relative_to(manager.paths.repo / ".jenkins/worktrees")
                 and len(_service_ref_read(ref)) == ref["bytes"],
                 "Target source is not the same actual managed checkout", "service_handoff_pending")
        roots.add(root)
        if name.startswith("tools/jenkins/") and "/tests/" not in name:
            _require(manifest["files"].get(name) == {"sha256": ref["sha256"], "bytes": ref["bytes"]},
                     "Target runtime member differs from its qualified source", "service_handoff_pending")
    _require(len(roots) == 1, "Target publication mixed source checkouts", "service_handoff_pending")
    return target


def _select_handoff_target_driver(manager, plan, plan_ref, output, stopped, resident_ref, target):
    """Called once under both lifecycle and resident locks after genuine termination."""
    from .job_provisioning import _service_ref, _service_ref_read
    if target is None:
        return None
    _require(target == verify_driver(target["root"], expected_digest=target["driverDigest"]),
             "Target driver changed before selection", "service_handoff_pending")
    selector_path = manager.paths.state / "deployment/driver.json"
    before = _read(selector_path)
    process_proof = json.loads(_service_ref_read(resident_ref))
    _require(_read(manager.journal) == stopped and stopped.get("state") == "stopped"
             and stopped.get("stopBinding") and stopped.get("stopResult")
             and process_proof.get("rootProcessTerminal") is True
             and process_proof.get("waitResult") == 0
             and before["driverDigest"] == plan["driverDigest"]
             and before["runtimeOperationId"] == stopped["operationId"] == plan["runtimeOperationId"],
             "Target selection precedes genuine predecessor termination", "service_handoff_pending")
    _write_control_once(output / "driver-before.json", before)
    before_ref = _service_ref(output / "driver-before.json")
    selected = {**target, "runtimeOperationId": stopped["operationId"],
                "state": "selected-for-independent-handoff"}
    intent = {"schemaVersion": 1, "planRef": plan_ref, "beforeRef": before_ref,
              "after": selected, "stoppedRuntimeRef": _service_ref(output / "stopped-runtime.json"),
              "residentProcessProof": resident_ref,
              "targetDriverQualificationRef": plan["targetDriverQualificationRef"],
              "sourceQualificationRef": plan["preparationTerminalRef"]}
    _write_control_once(output / "driver-selection-intent.json", intent)
    _require(_read(selector_path) == before and _read(manager.journal) == stopped,
             "Predecessor changed before target selection", "service_handoff_pending")
    atomic_json(selector_path, selected)
    _require(_read(selector_path) == selected, "Target selector readback differs", "service_handoff_pending")
    _write_control_once(output / "driver-selection.json",
                        {**intent, "status": "target-driver-selected", "nativeProofSynthesized": False})
    return _service_ref(output / "driver-selection.json")


def _verify_initial_handoff(manager, root, desired, *, previous_driver=None):
    from .job_provisioning import _service_ref_read
    implementation = _control_implementation()
    if previous_driver is not None:
        _require(Path(previous_driver["root"]) == manager.paths.runtime / "drivers" / previous_driver["driverDigest"]
                 and previous_driver == verify_driver(previous_driver["root"],
                                                      expected_digest=previous_driver["driverDigest"]),
                 "Previous implementation has no verified canonical driver", "service_handoff_pending")
        files = _read(previous_driver["manifest"])["files"]
        implementation = {"serviceSha256": files["tools/jenkins/deployment/independent_service.py"]["sha256"],
                          "jobProvisioningSha256": files["tools/jenkins/deployment/job_provisioning.py"]["sha256"]}
    _require(isinstance(desired, dict) and desired.get("initialHandoffRef"),
             "Controlled activation requires its genuine predecessor handoff", "service_handoff_pending")
    receipt = json.loads(_service_ref_read(desired["initialHandoffRef"]))
    _require(receipt.get("kind") == "independent-service-resident-handoff"
             and receipt.get("status") == "predecessor-retired"
             and receipt.get("controlImplementation") == implementation,
             "Predecessor handoff implementation differs", "service_handoff_pending")
    plan = json.loads(_service_ref_read(receipt["planRef"]))
    _require(plan.get("kind") == "independent-service-resident-handoff-v1"
             and Path(plan.get("repository", "")) == manager.paths.repo
             and plan.get("taskName") == "ZirconEngine-Jenkins-Service"
             and plan.get("controlImplementation") == implementation,
             "Handoff plan is not this service task", "service_handoff_pending")
    proof = json.loads(_service_ref_read(receipt["residentProcessProof"]))
    expected = plan["resident"]
    _require(proof.get("kind") == "windows-native-resident-process-exit"
             and all(str(proof.get("identity", {}).get(k)) == str(expected[k])
                     for k in ("pid", "creationTime", "executable"))
             and len(proof["identity"].get("commandDigest", "")) == 64
             and proof.get("taskInstanceStopAttempted") is True
             and proof.get("rootProcessTerminal") is True and proof.get("waitResult") == 0
             and int(proof.get("exitTime", "0")) > int(expected["creationTime"])
             and proof.get("jobProofIssued") is False and proof.get("eofProofIssued") is False,
             "Predecessor resident has no actual retained-handle exit proof", "service_handoff_pending")
    stopped = json.loads(_service_ref_read(receipt["stoppedRuntimeRef"]))
    for ref in (stopped["stopBinding"], stopped["stopResult"]):
        _require(_sha(Path(ref["path"])) == ref["sha256"],
                 "Actual predecessor Stop evidence changed", "service_handoff_pending")
    binding = _read(stopped["stopBinding"]["path"])
    from .recovery import has_terminal_lifecycle_proofs
    _require(stopped.get("state") == "stopped"
             and stopped["operationId"] == plan["runtimeOperationId"]
             and binding.get("nativeProofSynthesized") is False
             and has_terminal_lifecycle_proofs(stopped, binding["terminalHost"],
                     str(manager.paths.home.absolute()), binding["hostDeparture"]),
             "Predecessor controller/agent/host Stop is incomplete", "service_handoff_pending")
    target = plan.get("targetDriver")
    if target is not None:
        binding = json.loads(_service_ref_read(receipt["targetDriverSelectionRef"]))
        prior = json.loads(_service_ref_read(binding["beforeRef"]))
        selected = previous_driver if previous_driver is not None else _read(manager.paths.state / "deployment/driver.json")
        _require(binding.get("status") == "target-driver-selected"
                 and binding.get("nativeProofSynthesized") is False
                 and binding["planRef"] == receipt["planRef"]
                 and binding["stoppedRuntimeRef"] == receipt["stoppedRuntimeRef"]
                 and binding["residentProcessProof"] == receipt["residentProcessProof"]
                 and binding["targetDriverQualificationRef"] == plan["targetDriverQualificationRef"]
                 and binding["sourceQualificationRef"] == plan["preparationTerminalRef"]
                 and prior["driverDigest"] == plan["driverDigest"]
                 and prior["runtimeOperationId"] == plan["runtimeOperationId"]
                 and {key: binding["after"][key] for key in target} == target
                 and {key: selected[key] for key in target} == target
                 and desired.get("targetDriver", target) == target,
                 "Initial target selection lost its exact native handoff", "service_handoff_pending")
        _service_ref_read(plan["targetDriverQualificationRef"])
    previous_fields = ("previousDesiredRef", "previousInitialHandoffRef", "previousPolicyRef")
    _require("reviewedManagementInstallerRef" not in plan or all(key in plan for key in previous_fields),
             "Reviewed installer override requires an explicit successor", "service_handoff_pending")
    if any(key in plan for key in previous_fields):
        _require(all(key in plan and receipt.get(key) == plan[key] for key in previous_fields),
                 "Successor lost its explicit predecessor references", "service_handoff_pending")
        previous = json.loads(_service_ref_read(plan["previousDesiredRef"]))
        old_driver = verify_driver(manager.paths.runtime / "drivers" / plan["driverDigest"],
                                   expected_digest=plan["driverDigest"])
        old_receipt = _verify_initial_handoff(manager, root, previous, previous_driver=old_driver)
        old_policy = json.loads(_service_ref_read(plan["previousPolicyRef"]))
        next_policy = json.loads(_service_ref_read(receipt["policyRef"]))
        previous_policy_shas = list(old_policy.get("previousPolicySha256", []))
        if plan["previousPolicyRef"]["sha256"] not in previous_policy_shas:
            previous_policy_shas.append(plan["previousPolicyRef"]["sha256"])
        expected_registration = {**old_policy["runtimeRebind"], "controlImplementation": implementation}
        if "reviewedManagementInstallerRef" in plan:
            reviewed_installer = plan["reviewedManagementInstallerRef"]
            _service_ref_read(reviewed_installer)
            expected_registration.update(managementInstaller=reviewed_installer,
                reviewedRuntimeRebindInstallerSha256=reviewed_installer["sha256"])
        _require(previous.get("phase") == "completed" and previous.get("desiredState") == "running"
                 and previous.get("startedOperationId") == plan["runtimeOperationId"]
                 and previous.get("initialHandoffRef") == plan["previousInitialHandoffRef"]
                 and old_policy.get("runtimeRebind", {}).get("controlImplementation") == old_receipt["controlImplementation"]
                 and next_policy == {**old_policy, "previousPolicySha256": previous_policy_shas,
                                     "runtimeRebind": expected_registration},
                 "Successor policy or previous completed binding differs", "service_handoff_pending")
    return receipt


def handoff_resident(manager, policy_path, plan_path, expected_sha):
    """Run only in a separate fixed Scheduler task after formal business drain."""
    from .job_provisioning import (_service_ref, _service_ref_read, _service_command,
                                  capture_runtime_rebind, verify_runtime_rebind_inputs)
    _independent_task_creation_flags()
    root = manager.paths.state / "independent-service"
    plan_path = Path(plan_path)
    plan_ref = _service_ref(plan_path)
    _require(plan_ref["sha256"] == expected_sha, "Handoff plan digest differs")
    plan = json.loads(_service_ref_read(plan_ref))
    request_id = plan.get("requestId", "")
    _require(len(request_id) == 32 and all(c in "0123456789abcdef" for c in request_id)
             and plan.get("schemaVersion") == 1
             and plan.get("kind") == "independent-service-resident-handoff-v1"
             and plan.get("taskName") == "ZirconEngine-Jenkins-Service"
             and plan.get("handoffTaskName") == "ZirconEngine-Jenkins-Service-Handoff"
             and Path(plan.get("repository", "")) == manager.paths.repo
             and plan_path == root / "handoffs" / request_id / "plan.json"
             and plan.get("controlImplementation") == _control_implementation(),
             "Handoff plan is outside the fixed deployment boundary")
    output = plan_path.parent
    _require(not (output / "handoff-intent.json").exists(),
             "Original handoff must be reconciled without a new Stop", "service_handoff_pending")
    artifacts = json.loads(_service_ref_read(plan["artifactsRef"]))
    _require(Path(plan["artifactsRef"]["path"]) == output / "artifacts.json"
             and Path(artifacts.get("root", "")) == output
             and artifacts.get("taskId") == request_id
             and artifacts.get("sourceCopies") == 0,
             "Handoff outputs were not preregistered in their exact owned root")
    for ref in plan["sourceRefs"].values():
        _service_ref_read(ref)
    _require(plan["sourceRefs"]["service"]["sha256"] == _LOADED_SERVICE_SHA256
             and plan["sourceRefs"]["jobProvisioning"]["sha256"] == _control_implementation()["jobProvisioningSha256"],
             "Handoff code was not qualified to these actual loaded bytes")
    # These are already-real archived preparation/native terminals, never a
    # predicted proof for this handoff or an inference from a missing PID.
    preparation = json.loads(_service_ref_read(plan["preparationTerminalRef"]))
    archives = preparation.get("archives", [])
    archive_names = {"artifacts.json", "stdout.bin", "stderr.bin", "started.json",
                     "native-terminal.json", "source-hashes.json", "completion-receipt.json",
                     "fresh-physical-binding.json", "cleanup-plan.json", "cleanup.json", "summary.json"}
    by_name = {item["name"]: item for item in archives}
    _require(preparation.get("status") == "passed" and preparation.get("rootTerminal") is True
             and preparation.get("job") == "zircon-management-tests"
             and type(preparation.get("build")) is int and preparation["build"] > 0
             and preparation.get("jobResult") == "SUCCESS" and preparation.get("nativeComplete") is True
             and preparation.get("localApiHomeBytesEqual") is True
             and preparation.get("missingArchives") == preparation.get("unequalArchives") == []
             and len(archives) == 11 and set(by_name) == archive_names
             and all(item.get("apiStatus") == 200 and item.get("bytesEqual") is True for item in archives),
             "Formal handoff preparation has not actually completed")
    source_archive = by_name["source-hashes.json"]
    home_source = (manager.paths.home / "jobs/zircon-management-tests/builds"
                   / str(preparation["build"]) / "archive/source-hashes.json")
    _require(Path(source_archive["home"]) == home_source, "Preparation source archive is from another build")
    source_ref = {"path": source_archive["local"], "sha256": source_archive["sha256"],
                  "bytes": source_archive["bytes"]}
    source_raw = _service_ref_read(source_ref)
    _require(source_raw == _service_ref_read({**source_ref, "path": str(home_source)}),
             "Preparation source archive no longer matches Jenkins Home")
    source_pins = json.loads(source_raw)["sourcePins"]
    physical_pins = {Path(path): pin for path, pin in source_pins.items()}
    _require(len(physical_pins) == len(source_pins)
             and all(physical_pins.get(Path(ref["path"])) == {"sha256": ref["sha256"], "bytes": ref["bytes"]}
                     for ref in plan["sourceRefs"].values()),
             "Formal preparation did not qualify these actual source bytes")
    target_driver = _verify_handoff_target_driver(manager, plan, physical_pins)
    previous_fields = ("previousDesiredRef", "previousInitialHandoffRef", "previousPolicyRef")
    successor = any(key in plan for key in previous_fields)
    _require("reviewedManagementInstallerRef" not in plan or all(key in plan for key in previous_fields),
             "Reviewed installer override requires an explicit successor", "service_handoff_pending")
    if successor:
        _require(all(key in plan for key in previous_fields)
                 and Path(plan["previousDesiredRef"]["path"]) == output / "previous-desired.json"
                 and Path(plan["previousPolicyRef"]["path"]) == output / "previous-policy.json",
                 "Successor needs its explicit immutable predecessor snapshots", "service_handoff_pending")
        previous_raw = _service_ref_read(plan["previousDesiredRef"])
        previous = json.loads(previous_raw)
        previous_policy_raw = _service_ref_read(plan["previousPolicyRef"])
        policy, _ = verify_policy(manager, policy_path)
        _require(_raw(policy_path) == previous_policy_raw
                 and policy == json.loads(previous_policy_raw)
                 and previous.get("phase") == "completed" and previous.get("desiredState") == "running"
                 and previous.get("startedOperationId") == plan["runtimeOperationId"]
                 and previous.get("initialHandoffRef") == plan["previousInitialHandoffRef"],
                 "Successor predecessor policy or completed desired state differs", "service_handoff_pending")
        old_driver = verify_driver(manager.paths.runtime / "drivers" / plan["driverDigest"],
                                   expected_digest=plan["driverDigest"])
        old_receipt = _verify_initial_handoff(manager, root, previous, previous_driver=old_driver)
        registration = policy.get("runtimeRebind")
        _require(isinstance(registration, dict) and registration.get("schemaVersion") == 1
                 and registration.get("controlImplementation") == old_receipt["controlImplementation"],
                 "Old policy is not registered to its verified previous implementation", "service_handoff_pending")
        for key in ("managementInstaller", "codex", "activationRuntimeBefore"):
            _service_ref_read(registration[key])
        _require(registration["managementInstaller"].get("sha256") == registration.get("reviewedRuntimeRebindInstallerSha256"),
                 "Previous retained-prefix installer pin differs", "service_handoff_pending")
        if "reviewedManagementInstallerRef" in plan:
            reviewed_installer = plan["reviewedManagementInstallerRef"]
            _service_ref_read(reviewed_installer)
            registration = {**registration, "managementInstaller": reviewed_installer,
                            "reviewedRuntimeRebindInstallerSha256": reviewed_installer["sha256"]}
        _require(target_driver is not None and target_driver["driverDigest"] != plan["driverDigest"],
                 "Successor requires its explicitly qualified different target", "service_handoff_pending")
        target_files = _read(target_driver["manifest"])["files"]
        _require(all(target_files[name]["sha256"] == _control_implementation()[key] for name, key in (
                     ("tools/jenkins/deployment/independent_service.py", "serviceSha256"),
                     ("tools/jenkins/deployment/job_provisioning.py", "jobProvisioningSha256"))),
                 "Successor target differs from its loaded control implementation", "service_handoff_pending")
    else:
        registration = verify_control_registration(manager, policy_path)
    installer = plan["sourceRefs"]["taskInstaller"]
    _require(Path(installer["path"]) == manager.paths.repo / "tools/dev/Install-JenkinsService.ps1",
             "Handoff COM executor is not the existing task installer")
    powershell = Path("C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
    def task_step(step, evidence=None):
        _service_ref_read(installer)
        # Fixed OS input only: WinSxS hard links are valid, never copied or
        # accepted through the ordinary writable-artifact file reader.
        expected = plan["windowsPowerShell"]
        _require(Path(expected["path"]) == powershell, "Handoff OS executable path differs")
        _plain(powershell)
        before_os = powershell.stat()
        with powershell.open("rb") as stream:
            opened = os.fstat(stream.fileno())
            raw_os = stream.read()
            after_os = os.fstat(stream.fileno())
        fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_nlink", "st_file_attributes")
        facts = {k: getattr(before_os, k, 0) for k in fields}
        _require(facts == expected["identity"]
                 and all(facts == {k: getattr(x, k, 0) for k in fields}
                         for x in (opened, after_os, powershell.stat()))
                 and len(raw_os) == expected["bytes"]
                 and hashlib.sha256(raw_os).hexdigest() == expected["sha256"],
                 "Fixed OS PowerShell identity/bytes changed")
        argv = [str(powershell), "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                "-File", installer["path"], "-HandoffStep", step, "-HandoffPlan", str(plan_path),
                "-HandoffSha", expected_sha]
        if evidence:
            argv += ["-HandoffEvidence", evidence["path"], "-HandoffEvidenceSha", evidence["sha256"]]
        _service_command(manager, output, "task-" + step, argv)
        return _read(output / ("task-" + step) / "stdout.log")
    with process_lock(root / "handoff.lock", reason="service_handoff_busy"):
        from contextlib import nullcontext
        with process_lock(manager.paths.state / "deployment-start.lock", reason="deployment_start_busy"), \
                (process_lock(root / "control.lock", reason="service_control_busy") if successor else nullcontext()):
            if successor:
                active_path = root / "control-active.json"
                _require(not (_read(active_path) if active_path.exists() else {}).get("requestId")
                         and _raw(root / "desired.json") == previous_raw
                         and _raw(policy_path) == previous_policy_raw,
                         "Previous control request or desired/policy preimage changed", "service_handoff_pending")
            operation = _read(manager.journal)
            selected = _read(manager.paths.state / "deployment/driver.json")
            heartbeat = _read(root / "status.json")
            if successor:
                _require(heartbeat.get("controlImplementation") == registration["controlImplementation"],
                         "Previous resident implementation changed", "service_handoff_pending")
            _require(operation.get("state") == "running"
                     and operation["operationId"] == plan["runtimeOperationId"] == selected["runtimeOperationId"]
                     and selected["driverDigest"] == plan["driverDigest"]
                     and all(str(heartbeat.get("resident", {}).get(k)) == str(plan["resident"][k])
                             for k in ("pid", "creationTime", "executable"))
                     and plan["resident"]["pid"] not in {os.getpid(), operation.get("pid")},
                     "Initial resident/runtime binding changed", "service_handoff_pending")
            observed = task_step("observe")
            from .live_observations import _windows_command_line
            own_args = _windows_command_line(os.getpid())
            _require(len(own_args) == 12 and own_args[3] == "handoff"
                     and Path(own_args[9]) == plan_path and own_args[11] == expected_sha
                     and observed.get("handoffTaskPath") == "\\ZirconEngine-Jenkins-Service-Handoff"
                     and observed.get("handoffInstanceGuid") != observed.get("instanceGuid"),
                     "Handoff is not the separate exact Scheduler command")
            snapshot = _native_process_snapshot()
            cursor, seen = os.getpid(), set()
            for _ in range(8):
                if cursor == observed["handoffEnginePid"]:
                    break
                _require(cursor not in seen and cursor in snapshot, "Handoff native ancestry is unknown")
                seen.add(cursor)
                cursor = snapshot[cursor].get("parentPid")
            _require(cursor == observed["handoffEnginePid"], "Handoff has no matching task-engine ancestry")
            _write_control_once(output / "task-observation.json", observed)
            observation_ref = _service_ref(output / "task-observation.json")
            resident = _RetainedResident(manager, plan["resident"], observed, controlled=successor)
            try:
                before = capture_runtime_rebind(manager, registration, output, target_driver=target_driver)
                _require(before["runtimeOperationId"] == plan["runtimeOperationId"],
                         "Handoff old job bindings changed")
                manager.drain()
                verify_runtime_rebind_inputs(manager, registration, before, offline=True)
                if successor:
                    _require(_raw(root / "desired.json") == previous_raw and _raw(policy_path) == previous_policy_raw,
                             "Predecessor changed before formal Stop", "service_handoff_pending")
                _write_control_once(output / "handoff-intent.json",
                                    {"planRef": plan_ref, "runtimeBeforeRef": _service_ref(output / "runtime-before.json"),
                                     "taskObservationRef": observation_ref, "phase": "formal-drain-completed"})
                paused = task_step("pause", observation_ref)
                _write_control_once(output / "task-paused.json", paused)
                stopped = manager._stop_locked()
                _require(stopped.get("state") == "stopped" and stopped.get("stopBinding") and stopped.get("stopResult"),
                         "Owned lifecycle Stop is not complete", "service_handoff_pending")
                _write_control_once(output / "stopped-runtime.json", stopped)
                # Re-run the existing Stop validator before touching the old root.
                _require(manager._stop_locked() == stopped, "Owned Stop changed before resident retirement")
                stop_attempt = task_step("stop-instance", observation_ref)
                proof = resident.finish(stop_attempt)
                proof.update(planRef=plan_ref, controlImplementation=_control_implementation())
                _write_control_once(output / "resident-process-exit.json", proof)
                resident.close()
                with process_lock(root / "resident.lock", reason="independent_service_already_running"):
                    receipt = {"schemaVersion": 1, "kind": "independent-service-resident-handoff",
                               "status": "predecessor-retired", "planRef": plan_ref,
                               "controlImplementation": _control_implementation(),
                               "residentProcessProof": _service_ref(output / "resident-process-exit.json"),
                               "stoppedRuntimeRef": _service_ref(output / "stopped-runtime.json"),
                               "runtimeBeforeRef": _service_ref(output / "runtime-before.json"),
                               "nativeJobSynthesized": False}
                    selection_ref = _select_handoff_target_driver(manager, plan, plan_ref, output, stopped,
                                        receipt["residentProcessProof"], target_driver)
                    if selection_ref is not None:
                        receipt["targetDriverSelectionRef"] = selection_ref
                    if successor:
                        for ref in plan["sourceRefs"].values():
                            _service_ref_read(ref)
                        _require(_raw(root / "desired.json") == previous_raw and _raw(policy_path) == previous_policy_raw,
                                 "Predecessor changed before policy/desired CAS", "service_handoff_pending")
                        previous_policy_shas = list(policy.get("previousPolicySha256", []))
                        if plan["previousPolicyRef"]["sha256"] not in previous_policy_shas:
                            previous_policy_shas.append(plan["previousPolicyRef"]["sha256"])
                        if "reviewedManagementInstallerRef" in plan:
                            _service_ref_read(registration["managementInstaller"])
                        next_policy = {**policy, "previousPolicySha256": previous_policy_shas, "runtimeRebind": {
                            **registration, "controlImplementation": _control_implementation()}}
                        _write_control_once(output / "policy-after.json", next_policy)
                        receipt.update({key: plan[key] for key in previous_fields})
                        receipt["policyRef"] = _service_ref(output / "policy-after.json")
                        atomic_json(Path(policy_path), next_policy)
                        verify_control_registration(manager, policy_path)
                        _require(_raw(policy_path) == _service_ref_read(receipt["policyRef"]),
                                 "Successor policy CAS readback differs", "service_handoff_pending")
                    _write_control_once(output / "handoff-receipt.json", receipt)
                    desired = {"schemaVersion": 1, "requestId": request_id, "action": "start",
                               "desiredState": "running", "phase": "ready-to-start",
                               "previousRuntimeBeforeRef": receipt["runtimeBeforeRef"],
                               "initialHandoffRef": _service_ref(output / "handoff-receipt.json")}
                    if target_driver is not None:
                        desired["targetDriver"] = target_driver
                    _verify_initial_handoff(manager, root, desired)
                    _control_report(root, request_id,
                                    {"initialHandoffRef": desired["initialHandoffRef"],
                                     "runtimeBeforeRef": receipt["runtimeBeforeRef"]})
                    _require(_raw(root / "desired.json") == previous_raw if successor
                             else not (root / "desired.json").exists(),
                             "Previous desired-state CAS differs")
                    atomic_json(root / "desired.json", desired)
                    _require(_read(root / "desired.json") == desired, "Desired-state CAS readback differs")
                # The replacement takes resident.lock non-blockingly; release it
                # before Task.Run. deployment-start.lock still serializes Start.
                activated = task_step("activate", receipt["residentProcessProof"])
                _write_control_once(output / "task-activation.json", activated)
                result = {"status": "handoff-completed-start-pending", "handoffRef": desired["initialHandoffRef"],
                          "taskActivationRef": _service_ref(output / "task-activation.json"),
                          "serviceHealthyClaimed": False, "actualElevenJobRebindRequired": True}
                _write_control_once(output / "result.json", result)
                return result
            finally:
                resident.close()


def _controlled_recovery_locked(manager, registration, root, desired):
    from .job_provisioning import _service_ref_read, verify_runtime_rebind_inputs
    if desired is None:
        # There is no reviewed executor/validator for the initial Task Scheduler
        # resident handoff yet. A policy ref, a PID absence or this loop's own
        # Stop cannot supply it. Fail before any lifecycle mutation.
        operation = _read(manager.journal)
        _require(operation.get("state") == "stopped"
                 and operation.get("stopBinding") and operation.get("stopResult"),
                 "First activation needs the pre-existing actual predecessor Stop",
                 "service_resident_handoff_unavailable")
        raise JenkinsError("service_resident_handoff_unavailable",
                           "Initial controlled-resident activation needs a separately "
                           "managed native resident handoff; this draft cannot validate it",
                           retryable=True)
    else:
        _verify_initial_handoff(manager, root, desired)
        if desired.get("phase") in {"ready-to-start", "rebind-pending"}:
            before = json.loads(_service_ref_read(desired["previousRuntimeBeforeRef"]))
            output = root / "reports" / desired["requestId"]
            _finish_controlled_start_locked(manager, registration, root, desired, before, output)
            return _read(root / "desired.json")
        if desired.get("desiredState") == "stopped" or desired.get("phase") != "completed":
            return desired
        operation = _read(manager.journal)
        alive = process_alive(operation.get("pid")) if type(operation.get("pid")) is int else None
        if alive is not False:
            _require(alive is True, "Lifecycle owner is unknown", "independent_owner_unknown")
            return desired
        before_ref = desired["runtimeBeforeRef"]
        before = json.loads(_service_ref_read(before_ref))
        # Existing interruption authority retains all business holds; departure
        # is never labelled as a Stop/native-termination proof.
        request_id = uuid4().hex
    verify_runtime_rebind_inputs(manager, registration, before, offline=True)
    output = _control_report(root, request_id, {"runtimeBeforeRef": before_ref})
    desired = {"schemaVersion": 1, "requestId": request_id, "action": "start",
               "desiredState": "running", "phase": "ready-to-start",
               "previousRuntimeBeforeRef": before_ref, "initialHandoffRef": desired["initialHandoffRef"]}
    atomic_json(root / "desired.json", desired)
    _finish_controlled_start_locked(manager, registration, root, desired, before, output)
    return _read(root / "desired.json")


def supervise_controlled(manager, policy_path, *, interval=15, max_ticks=None):
    """A stopped Jenkins runtime leaves its Windows-owned resident alive."""
    root = manager.paths.state / "independent-service"
    registration = verify_control_registration(manager, policy_path)
    _independent_task_creation_flags()
    with process_lock(root / "resident.lock", reason="independent_service_already_running"):
        resident = current_identity(os.getpid()).to_dict()
        ticks = 0
        initial = _read(root / "desired.json") if (root / "desired.json").exists() else None
        _verify_initial_handoff(manager, root, initial)
        while max_ticks is None or ticks < max_ticks:
            ticks += 1
            active = {}
            result = None
            try:
                active_path = root / "control-active.json"
                with process_lock(root / "control.lock", reason="service_control_busy"):
                    active = _read(active_path) if active_path.exists() else {}
                with process_lock(manager.paths.state / "deployment-start.lock", reason="deployment_start_busy"):
                    if active.get("requestId"):
                        result = _run_service_request_locked(manager, registration, root, resident, active)
                        if result.get("status") == "completed":
                            with process_lock(root / "control.lock", reason="service_control_busy"):
                                _require(_read(active_path) == active, "Active service request changed")
                                atomic_json(active_path, {})
                    desired_path = root / "desired.json"
                    desired = _read(desired_path) if desired_path.exists() else None
                    if not active.get("requestId"):
                        desired = _controlled_recovery_locked(manager, registration, root, desired)
                    value = _controlled_observation(manager, resident, desired, result)
            except Exception as error:
                value = {"schemaVersion": 1, "status": "recovery-pending", "resident": resident,
                         "observedAt": time.time(), "controlProtocol": _CONTROL_PROTOCOL,
                         "controlImplementation": _control_implementation(),
                         "activeRequestId": active.get("requestId"),
                         "reasonCode": getattr(error, "code", type(error).__name__), "message": str(error),
                         "clientLifetimeDependency": False, "nativeProofSynthesized": False}
            try:
                atomic_json(root / "status.json", value)
            except Exception as error:
                print(json.dumps({"status": "evidence-write-pending", "errorType": type(error).__name__}), flush=True)
            if max_ticks is None or ticks < max_ticks:
                time.sleep(interval)



def main():
    parser = argparse.ArgumentParser(description="Independent Windows Jenkins service lifecycle")
    parser.add_argument("action", choices=("check", "run", "status", "recover-start", "verify", "health", "handoff"))
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--policy", type=Path, required=True)
    parser.add_argument("--service-controls", action="store_true")
    parser.add_argument("--handoff-plan", type=Path)
    parser.add_argument("--handoff-sha")
    args = parser.parse_args()
    from .manager import DeploymentManager
    spec = load_spec(args.spec)
    manager = DeploymentManager(spec, resolve_paths(spec))
    if args.action == "handoff":
        _require(args.handoff_plan is not None and args.handoff_sha is not None,
                 "Handoff requires its exact reviewed immutable plan")
        print(json.dumps(handoff_resident(manager, args.policy, args.handoff_plan, args.handoff_sha), sort_keys=True))
    elif args.action == "status":
        print(json.dumps(_read(manager.paths.state / "independent-service/status.json"), sort_keys=True))
    elif args.action == "verify":
        _, checksum = verify_policy(manager, args.policy)
        if args.service_controls:
            verify_control_registration(manager, args.policy)
        print(json.dumps({"status": "inputs-verified", "policySha256": checksum}))
    elif args.action == "health":
        # A health client needs HTTP/agent availability, not privileged process
        # handles across the unattended task's Windows logon session.
        health = manager.health()
        _require(health.get("ready") is True, "Independent Jenkins runtime is not ready")
        print(json.dumps({"status": "healthy", "health": health,
                          "nativeProofIssued": False, "businessRequestAcceptance": False}, sort_keys=True))
    elif args.action == "recover-start":
        with process_lock(manager.paths.state / "deployment-start.lock", reason="deployment_start_busy"):
            policy, checksum = verify_policy(manager, args.policy)
            receipt = _reconcile_missing_spawn_locked(manager, policy, checksum)
            print(json.dumps(receipt, sort_keys=True))
    elif args.action == "check":
        policy, checksum = verify_policy(manager, args.policy)
        evidence = capture_replacement(manager, policy, checksum)
        print(json.dumps({"status": "ready-to-start", "policySha256": checksum,
                          "evidenceDigest": digest(evidence),
                          "retainedPendingExecutions": evidence["retainedPendingExecutions"]}, sort_keys=True))
    else:
        supervise(manager, args.policy, controls=args.service_controls)
    return 0

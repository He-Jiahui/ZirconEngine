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
             and selected["driverDigest"] == previous["selected"]["driverDigest"] == policy["driverDigest"],
             "Failed spawn no longer owns the exact lifecycle pointers")
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


def supervise(manager, policy_path, *, interval=15, max_ticks=None):
    """Health errors are durable observations, never reasons to discard service Jobs."""
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


def main():
    parser = argparse.ArgumentParser(description="Independent Windows Jenkins service lifecycle")
    parser.add_argument("action", choices=("check", "run", "status", "recover-start", "verify", "health"))
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--policy", type=Path, required=True)
    args = parser.parse_args()
    from .manager import DeploymentManager
    spec = load_spec(args.spec)
    manager = DeploymentManager(spec, resolve_paths(spec))
    if args.action == "status":
        print(json.dumps(_read(manager.paths.state / "independent-service/status.json"), sort_keys=True))
    elif args.action == "verify":
        _, checksum = verify_policy(manager, args.policy)
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
        supervise(manager, args.policy)
    return 0

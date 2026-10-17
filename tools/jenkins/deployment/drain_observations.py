"""Observe owned shutdown without reconciling historical execution or holds."""
from __future__ import annotations

import hashlib
import json
import sqlite3
import time
from contextlib import closing
from pathlib import Path

from ..contracts import JenkinsError, digest
from .driver import _plain, verify_driver
from .live_observations import observe_endpoint_owner, observe_live_identity
from .recovery import has_terminal_deployment_proofs, has_terminal_lifecycle_proofs, previous_boot_deployment_evidence
from .startup_observations import observe_departed_identity

TERMINAL_EXECUTIONS = {"passed", "failed", "cancelled"}
IDENTITY_FIELDS = ("pid", "creationTime", "executable", "commandDigest")
COMPUTER_QUERY = ("computer/api/json?tree=computer[displayName,numExecutors,"
                  "executors[idle,currentExecutable[url]],oneOffExecutors[idle,currentExecutable[url]]]")


def _require(condition, message, code="drain_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


def _json(path: Path, *, with_bytes=False):
    _plain(path)
    try:
        raw = path.read_bytes()
        value = json.loads(raw)
    except (OSError, ValueError) as error:
        raise JenkinsError("drain_state_unavailable", "Drain evidence could not be read", retryable=True) from error
    _require(isinstance(value, dict), "Drain evidence is not an object")
    return (value, raw) if with_bytes else value


def _identity(value, executable=None):
    _require(isinstance(value, dict) and type(value.get("pid")) is int and value["pid"] > 0
             and str(value.get("creationTime", "")).isascii()
             and str(value.get("creationTime", "")).isdigit() and int(value["creationTime"]) > 0
             and isinstance(value.get("executable"), str) and bool(value["executable"])
             and isinstance(value.get("commandDigest"), str) and len(value["commandDigest"]) == 64
             and all(c in "0123456789abcdef" for c in value["commandDigest"]),
             "A complete native identity is required")
    if executable is not None:
        _require(Path(value["executable"]).absolute() == Path(executable).absolute(),
                 "Native executable differs from the formal deployment")
    return {key: value[key] for key in IDENTITY_FIELDS}


def owner_observation(manager, *, expected=None, allow_starting=False) -> dict:
    operation = _json(manager.journal)
    host = _json(manager.paths.state / "deployment/host.json")
    allowed = {"running", "starting"} if allow_starting else {"running"}
    runtime = operation.get("operationId")
    _require(operation.get("state") in allowed and host.get("status") == "running"
             and operation.get("component") == "lifecycle-host" and isinstance(runtime, str) and runtime
             and operation.get("generation") == runtime == host.get("operationId") == host.get("generation")
             and operation.get("homePath") == str(manager.paths.home.absolute())
             and operation.get("pid") == operation.get("hostPid") == host.get("hostPid")
             and operation.get("controlPlaneOnly") is False and host.get("controlPlaneOnly") is False
             and host.get("agentLaunchAttempted") is True,
             "The executing lifecycle owner is not completely bound", "drain_owner_unproven")
    supervisor = _identity(operation, manager.paths.repo / ".jenkins/runtime/python/python.exe")
    controller = _identity(host.get("controllerIdentity") or host.get("identity"), manager.java)
    agent = _identity(host.get("agentIdentity"), manager.java)
    _require(host.get("identity", host.get("controllerIdentity")) == host.get("controllerIdentity"),
             "Controller identities conflict")
    _require(len({value["pid"] for value in (supervisor, controller, agent)}) == 3,
             "Lifecycle components do not have distinct identities")
    selected = _json(manager.paths.state / "deployment/driver.json")
    checksum = selected.get("driverDigest")
    _require(isinstance(checksum, str) and len(checksum) == 64
             and all(c in "0123456789abcdef" for c in checksum)
             and selected.get("runtimeOperationId") == runtime,
             "Selected driver does not bind the current lifecycle")
    root = manager.paths.repo / ".jenkins/runtime/drivers" / checksum
    _require(Path(selected.get("root", "")).absolute() == root.absolute(), "Selected driver root changed")
    verify_driver(root, expected_digest=checksum)
    owner = {"operationId": runtime, "generation": runtime, "homePath": operation["homePath"],
             "supervisor": supervisor, "controller": controller, "agent": agent, "driverDigest": checksum}
    if expected is not None:
        _require(owner == expected, "Lifecycle ownership changed during observation", "drain_owner_changed")
    for identity in (supervisor, controller, agent):
        observe_live_identity(identity)
    observe_endpoint_owner(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]), controller["pid"])
    return owner


def execution_inventory(state: Path) -> list[dict]:
    """Use a read-only transaction; do not initialize State or its schema."""
    path = state / "coordination.sqlite3"
    _plain(path)
    try:
        with closing(sqlite3.connect(path.absolute().as_uri() + "?mode=ro", uri=True)) as connection:
            connection.execute("PRAGMA query_only=ON")
            connection.execute("BEGIN")
            _require(connection.execute("SELECT value FROM metadata WHERE key='schemaVersion'").fetchone() == ("1",),
                     "Execution schema is unsupported")
            rows = [{"domain": domain, "key": key, "payload": json.loads(payload), "version": version}
                    for domain, key, payload, version in connection.execute(
                        "SELECT domain,key,payload,version FROM records WHERE domain IN "
                        "('execution','execution_launch','execution_host','native_job',"
                        "'resource_reservation','pool_writer_hold','execution_consumer',"
                        "'resource_queue','jenkins_submission') ORDER BY domain,key")]
            _require(all(isinstance(row["payload"], dict) for row in rows), "Execution payload is malformed")
            return rows
    except (OSError, ValueError, sqlite3.Error) as error:
        raise JenkinsError("drain_state_unavailable", "Execution ownership could not be read", retryable=True) from error


def _bundle(rows, key):
    return {"execution": next((r for r in rows if r["domain"] == "execution" and r["key"] == key), None),
            "launch": next((r for r in rows if r["domain"] == "execution_launch" and r["key"] == key), None),
            "host": next((r for r in rows if r["domain"] == "execution_host" and r["key"] == key), None),
            "nativeJobs": [r for r in rows if r["domain"] == "native_job" and r["payload"].get("executionId") == key]}


def _bound_execution(bundle):
    _require(all(isinstance(bundle.get(name), dict) for name in ("execution", "launch", "host")),
             "Execution launch and host provenance are incomplete")
    ep, lp, hp = (bundle[name]["payload"] for name in ("execution", "launch", "host"))
    runtime, generation, checksum, driver = ep.get("operationId"), ep.get("generation"), ep.get("driverDigest"), ep.get("driver")
    _require(isinstance(runtime, str) and runtime and isinstance(generation, str) and generation
             and isinstance(checksum, str) and len(checksum) == 64 and all(c in "0123456789abcdef" for c in checksum)
             and isinstance(driver, dict) and driver.get("runtimeOperationId") == runtime
             and driver.get("generation") == generation and driver.get("digest") == checksum
             and all(p.get("operationId") == runtime and p.get("generation") == generation for p in (lp, hp))
             and lp.get("executionId") == hp.get("executionId") == bundle["execution"]["key"]
             and lp.get("driverDigest") == checksum and lp.get("brokerGeneration") == runtime,
             "Execution and native owner bindings disagree")
    launcher = _identity(lp.get("launcherIdentity"))
    _require(hp.get("hostPid") == launcher["pid"] and str(hp.get("hostBirthToken")) == launcher["creationTime"],
             "Execution host identity differs from its launcher")
    return runtime, checksum, launcher


def native_terminal(row):
    payload = row["payload"]
    proof = payload.get("completeProof")
    identity = proof.get("identity", {}) if isinstance(proof, dict) else {}
    try:
        _identity(identity)
    except JenkinsError:
        return False
    return (isinstance(proof, dict) and payload.get("status") == "terminal" and proof.get("complete") is True
            and proof.get("childrenGone") is True and type(proof.get("activeProcesses")) is int and proof["activeProcesses"] == 0
            and proof.get("stdoutEof") is True and proof.get("stderrEof") is True and type(proof.get("processExitCode")) is int
            and identity.get("pid") == payload.get("pid") and str(identity.get("creationTime")) == str(payload.get("birthToken"))
            and identity.get("executable") == payload.get("executable"))


def _current_terminal(bundle, owner):
    runtime, checksum, launcher = _bound_execution(bundle)
    ep, lp, hp = (bundle[name]["payload"] for name in ("execution", "launch", "host"))
    guard = next((r for r in bundle["nativeJobs"] if r["key"] == lp.get("guardNativeJobId")), None)
    _require(runtime == owner["operationId"] and checksum == owner["driverDigest"]
             and lp.get("brokerPid") == owner["supervisor"]["pid"]
             and ep.get("status") in TERMINAL_EXECUTIONS and lp.get("status") == "exited"
             and hp.get("status") in {"terminal", "cancelled"} and guard is not None
             and guard["payload"].get("role") == "execution_host_guard" and guard["payload"].get("owner") == bundle["execution"]["key"]
             and guard["payload"].get("pid") == launcher["pid"]
             and str(guard["payload"].get("birthToken")) == launcher["creationTime"]
             and guard["payload"].get("executable") == launcher["executable"]
             and all(native_terminal(r) for r in bundle["nativeJobs"]),
             "Current execution lacks complete owned native termination")


def _cancelled_before_launch(manager, owner, bundle, rows):
    execution = bundle["execution"]
    payload = execution["payload"]
    runtime, generation = payload.get("operationId"), payload.get("generation")
    driver, key = payload.get("driver"), payload.get("executionKey")
    _require(payload.get("cancelledBeforeLaunch") is True and payload.get("status") == "cancelled" and payload.get("cancelRequested") is True
             and type(payload.get("refs")) is int and payload["refs"] == 0 and payload.get("consumers") == []
             and bundle["launch"] is None and bundle["host"] is None and bundle["nativeJobs"] == []
             and payload.get("phaseRecords") in (None, []) and payload.get("stageReceipts") in (None, {})
             and isinstance(runtime, str) and bool(runtime)
             and isinstance(generation, str) and generation.isascii() and generation.isdecimal() and int(generation) > 0
             and isinstance(driver, dict) and driver.get("runtimeOperationId") == runtime
             and driver.get("generation") == generation and driver.get("digest") == payload.get("driverDigest")
             and all(isinstance(payload.get(name), str) and len(payload[name]) == 64
                     and all(c in "0123456789abcdef" for c in payload[name])
                     for name in ("sourceDigest", "coverageDigest", "driverDigest", "recipeRef", "preparationKey"))
             and payload.get("sealedInputRef") == payload.get("sourceDigest")
             and key == {"source": payload["sourceDigest"], "preparation": payload["preparationKey"],
                         "recipe": payload["recipeRef"], "driver": payload["driverDigest"]}
             and payload.get("executionKeyDigest") == digest(key) and payload.get("recipeDigest") == payload["recipeRef"],
             "Exact released prelaunch cancellation authority is required")
    if runtime == owner["operationId"]:
        _require(payload["driverDigest"] == owner["driverDigest"], "Cancelled execution belongs to another current driver")
    else:
        verify_driver(manager.paths.repo / ".jenkins/runtime/drivers" / payload["driverDigest"],
                      expected_digest=payload["driverDigest"])
    related = []
    for record in rows:
        domain, value = record["domain"], record["payload"]
        execution_id = execution["key"]
        if domain in {"execution_launch", "execution_host", "native_job"} and (
                record["key"] == execution_id or value.get("executionId") == execution_id
                or (domain == "native_job" and value.get("owner") == execution_id)):
            _require(False, "Cancelled prelaunch execution has native launch ownership")
        if domain == "execution_consumer" and (value.get("executionId") == execution_id
                                               or record["key"].startswith(execution_id + ":")):
            _require(value.get("executionId") == execution_id and isinstance(value.get("consumerId"), str)
                     and value["consumerId"] and record["key"] == execution_id + ":" + value["consumerId"]
                     and value.get("status") == "released", "Cancelled execution has an unresolved consumer")
            related.append(record)
        if domain in {"resource_reservation", "pool_writer_hold", "resource_queue"} and (
                value.get("owner") == execution_id or record["key"] == payload.get("reservationKey")
                or (domain == "resource_queue" and record["key"].startswith(execution_id + ":"))):
            closed = {"cancelled"} if domain == "resource_queue" else {"released"}
            _require(value.get("owner") == execution_id and value.get("status") in closed,
                     "Cancelled execution retains unresolved resource ownership or queue work")
            related.append(record)
        if domain == "jenkins_submission":
            parameters = value.get("parameters")
            bindings = [value.get("executionId")]
            if isinstance(parameters, dict):
                bindings.extend([parameters.get("executionId"), parameters.get("EXECUTION_ID")])
            if execution_id not in bindings:
                continue
            _require(isinstance(parameters, dict) and all(v in (None, execution_id) for v in bindings)
                     and value.get("delivery") in {"submitted", "accepted", "completed"}
                     and value.get("sourceRef") == value.get("sourceDigest") == payload["sourceDigest"]
                     and value.get("coverageRef") == value.get("coverageDigest") == payload["coverageDigest"]
                     and parameters.get("recipeRef") == payload["recipeRef"] and parameters.get("generation") == generation,
                     "Cancelled execution has unknown or mismatched Jenkins delivery")
            for name, expected in (("runtimeOperationId", runtime), ("RUNTIME_OPERATION_ID", runtime),
                                   ("stageImplementationDigest", payload["driverDigest"]),
                                   ("STAGE_IMPLEMENTATION_DIGEST", payload["driverDigest"])):
                _require(all(mapping.get(name, expected) == expected for mapping in (value, parameters)),
                         "Cancelled execution delivery belongs to another runtime or driver")
            related.append(record)
    return {"executionId": execution["key"], "operationId": runtime, "driverDigest": payload["driverDigest"],
            "recordsDigest": digest({"execution": execution, "relatedRecords": related})}


def _stopped_owner_after_boot(manager, owner, boot):
    """A later observation may retain a genuinely completed predecessor Stop."""
    paths = (manager.journal, manager.paths.state / "deployment/host.json",
             manager.paths.state / "deployment/driver.json")
    for path in paths:
        _plain(path)
        if not path.is_file():
            return False
    operation, host, selected = (_json(path) for path in paths)
    if (operation.get("state") != "stopped" or host.get("status") != "stopped"
            or operation.get("component") != "lifecycle-host"
            or operation.get("operationId") != owner["operationId"]
            or operation.get("generation") != owner["generation"]
            or selected.get("runtimeOperationId") != owner["operationId"]
            or selected.get("driverDigest") != owner["driverDigest"]
            or not has_terminal_deployment_proofs(operation, host, owner["homePath"])):
        return False
    identities = (_identity(operation), _identity(host.get("controllerIdentity") or host.get("identity")),
                  _identity(host.get("agentIdentity")))
    if (identities != tuple(owner[key] for key in ("supervisor", "controller", "agent"))
            or any(int(value["creationTime"]) >= int(boot) for value in identities)
            or any(operation.get(key) != host.get(key) for key in ("nativeTerminationProof", "agentTerminationProof"))):
        return False
    observations = [observe_departed_identity(value) for value in identities]
    return has_terminal_lifecycle_proofs(operation, host, owner["homePath"], observations[0])


def _historical_lifecycle(manager, owner, bundle):
    runtime, checksum, launcher = _bound_execution(bundle)
    _require(runtime != owner["operationId"], "Current work cannot use historical provenance")
    root = manager.paths.repo / ".jenkins/runtime/drivers" / checksum
    verify_driver(root, expected_digest=checksum)
    paths = sorted((manager.paths.state / "deployment/history").glob("*.json"))
    legacy = manager.paths.state / "migration-baseline/guarded-runtime-before-stop.json"
    if legacy.exists():
        paths.append(legacy)
    matches = []
    for path in paths:
        if path.name.endswith(".pending.json"):
            continue
        record, raw = _json(path, with_bytes=True)
        operation = record.get("operation", record)
        if isinstance(operation, dict) and operation.get("operationId") == runtime:
            matches.append((path, record, hashlib.sha256(raw).hexdigest()))
    _require(len(matches) == 1, "Historical work requires one exact archived lifecycle")
    path, record, archive_sha256 = matches[0]
    operation = record.get("operation", record)
    host = record.get("host")
    if host is None:
        _require(path == legacy and operation.get("schemaVersion") == 2 and operation.get("state") == "stopped",
                 "Historical stopped-owner receipt is unverified")
        controller_proof = operation.get("nativeTerminationProof") or {}
        agent_proof = operation.get("agentTerminationProof") or {}
        host = {"status": "stopped", "operationId": runtime, "generation": runtime, "hostPid": operation.get("hostPid"),
                "controllerIdentity": controller_proof.get("identity"), "agentIdentity": agent_proof.get("identity"),
                "nativeTerminationProof": controller_proof, "agentTerminationProof": agent_proof, "agentLaunchAttempted": True}
    else:
        # Existing boot archives predate the current filename convention.
        # Bind their actual bytes and complete owner evidence, not their name.
        _require(path.parent == manager.paths.state / "deployment/history", "Historical lifecycle evidence is not archived")
    _require(operation.get("component") == "lifecycle-host" and operation.get("generation") == runtime
             and operation.get("homePath") == owner["homePath"] and host.get("operationId") == runtime
             and host.get("generation") == runtime and operation.get("pid") == operation.get("hostPid") == host.get("hostPid")
             and bundle["launch"]["payload"].get("brokerPid") == operation.get("pid"),
             "Historical lifecycle differs from the exact execution broker")
    identities = [_identity(operation, manager.paths.repo / ".jenkins/runtime/python/python.exe"),
                  _identity(host.get("controllerIdentity") or host.get("identity"), manager.java),
                  _identity(host.get("agentIdentity"), manager.java)]
    _require(len({v["pid"] for v in identities}) == 3
             and all(int(v["creationTime"]) >= int(identities[0]["creationTime"]) for v in identities[1:])
             and int(launcher["creationTime"]) >= int(identities[0]["creationTime"])
             and not ({(v["pid"], v["creationTime"]) for v in identities}
                      | {(launcher["pid"], launcher["creationTime"])})
                     & {(owner[k]["pid"], owner[k]["creationTime"]) for k in ("supervisor", "controller", "agent")
                        if k != "agent" or owner[k] is not None},
             "Historical and current native owner identities are not disjoint")
    boot = record.get("bootRecoveryEvidence") or record.get("deploymentRecoveryEvidence") or {}
    departures = [*identities, launcher]
    if boot.get("kind") == "windows-boot-boundary":
        _require(boot.get("operationId") == runtime and boot.get("homePath") == owner["homePath"]
                 and boot.get("hostRecordDigest") == digest(host) and boot.get("outcome") == "interrupted"
                 and boot.get("validationAcceptanceAllowed") is False and boot.get("identities") == identities
                 and isinstance(boot.get("bootFileTime"), str) and boot["bootFileTime"].isascii()
                 and boot["bootFileTime"].isdecimal() and int(boot["bootFileTime"]) > 0
                 and all(int(v["creationTime"]) < int(boot["bootFileTime"]) for v in identities)
                 and int(boot["bootFileTime"]) <= int(owner["supervisor"]["creationTime"]),
                 "Historical boot interruption evidence is incomplete")
        observed = previous_boot_deployment_evidence(operation, host, owner["homePath"])
        actual_boot = observed.get("bootFileTime") if isinstance(observed, dict) else None
        _require(isinstance(actual_boot, str) and actual_boot.isascii() and actual_boot.isdecimal()
                 and actual_boot == str(int(actual_boot))
                 and int(boot["bootFileTime"]) <= int(actual_boot)
                 and (int(actual_boot) < int(owner["supervisor"]["creationTime"])
                      or _stopped_owner_after_boot(manager, owner, actual_boot))
                 and {k: v for k, v in observed.items() if k != "observedAt"} == {
                     "kind": "windows-boot-boundary", "operationId": runtime, "homePath": owner["homePath"],
                     "bootFileTime": actual_boot, "identities": identities, "hostRecordDigest": digest(host),
                     "outcome": "interrupted", "validationAcceptanceAllowed": False},
                 "Actual Windows boot does not prove this exact historical interruption")
        # A later real OS boot proves only the archived lifecycle interruption.
        # The independent execution launcher still needs its exact departure.
        departures = [launcher]
    else:
        observation = observe_departed_identity(identities[0])
        _require(has_terminal_lifecycle_proofs(operation, host, owner["homePath"], observation),
                 "Historical native lifecycle termination is unproven")
    for value in departures:
        observe_departed_identity(value)
    return {"path": str(path), "sha256": archive_sha256,
            "executionId": bundle["execution"]["key"], "recordsDigest": digest(bundle),
            "historicalNativeAcceptance": False, "historicalHoldsReleaseAllowed": False}


def classify_executions(manager, owner, rows):
    current = [r["key"] for r in rows if r["domain"] == "execution" and r["payload"].get("operationId") == owner["operationId"]]
    blocked, historical, closed_prelaunch = [], [], []
    for row in rows:
        if row["domain"] != "execution":
            continue
        payload = row["payload"]
        bundle = _bundle(rows, row["key"])
        if "cancelledBeforeLaunch" in payload:
            closed_prelaunch.append(_cancelled_before_launch(manager, owner, bundle, rows))
        elif row["key"] in current:
            if payload.get("status") not in TERMINAL_EXECUTIONS:
                blocked.append(row["key"])
            elif payload.get("failedBeforeLaunch") is True and not bundle["launch"] and not bundle["host"] and not bundle["nativeJobs"]:
                if any(r["domain"] in {"resource_reservation", "pool_writer_hold"}
                       and r["payload"].get("owner") == row["key"] and r["payload"].get("status") == "active" for r in rows):
                    blocked.append(row["key"])
            else:
                _current_terminal(bundle, owner)
                if any(r["domain"] in {"resource_reservation", "pool_writer_hold"}
                       and r["payload"].get("owner") == row["key"] and r["payload"].get("status") == "active" for r in rows):
                    blocked.append(row["key"])
        elif payload.get("status") not in TERMINAL_EXECUTIONS or any(not native_terminal(r) for r in bundle["nativeJobs"]):
            historical.append(_historical_lifecycle(manager, owner, bundle))
    executions = {r["key"] for r in rows if r["domain"] == "execution"}
    for row in rows:
        if row["domain"] in {"execution_launch", "execution_host"} and row["key"] not in executions:
            _require(False, "An orphaned execution launch or host cannot be ignored")
        if row["domain"] == "native_job" and row["payload"].get("executionId") not in executions and not native_terminal(row):
            _require(False, "An unresolved orphaned native job cannot be ignored")
    protected = [r for r in rows if r["domain"] in {"resource_reservation", "pool_writer_hold"}
                 and r["payload"].get("status") == "active" and r["payload"].get("owner") not in current]
    _require(all(r["payload"].get("owner") in executions for r in protected),
             "An unresolved hold has no execution ownership provenance")
    return {"activeExecutions": blocked, "historicalExecutions": historical, "protectedHolds": protected,
            "closedBeforeLaunchExecutions": closed_prelaunch}


def executor_observation(computers):
    values = computers.get("computer")
    _require(isinstance(values, list) and bool(values), "Jenkins computer inventory was missing")
    observed = []
    for computer in values:
        _require(isinstance(computer, dict) and isinstance(computer.get("displayName"), str) and computer["displayName"]
                 and isinstance(computer.get("executors"), list) and isinstance(computer.get("oneOffExecutors"), list)
                 and ("numExecutors" not in computer or (type(computer["numExecutors"]) is int
                      and computer["numExecutors"] >= 0 and computer["numExecutors"] == len(computer["executors"]))),
                 "Jenkins executor inventory was missing or malformed")
        busy = 0
        for executor in computer["executors"] + computer["oneOffExecutors"]:
            _require(isinstance(executor, dict) and type(executor.get("idle")) is bool,
                     "Executor idleness was missing or malformed")
            executable = executor.get("currentExecutable")
            if executor["idle"]:
                _require(executable is None, "Idle executor has conflicting work")
            else:
                _require(isinstance(executable, dict) and isinstance(executable.get("url"), str) and executable["url"],
                         "Busy executor work was unobservable")
                busy += 1
        observed.append({"displayName": computer["displayName"], "busyExecutors": busy})
    return {"computer": observed, "busyExecutors": sum(c["busyExecutors"] for c in observed)}


def api_json(manager, endpoint):
    status, body = manager._request(endpoint)
    try:
        value = json.loads(body)
    except (ValueError, TypeError) as error:
        raise JenkinsError("drain_unproven", "Jenkins API observation was malformed", retryable=True) from error
    _require(status == 200 and isinstance(value, dict), "Jenkins API observation was unavailable")
    return value


def quiet_observation(manager):
    value = api_json(manager, "api/json?tree=quietingDown")
    _require(type(value.get("quietingDown")) is bool, "Jenkins quiet mode was unobservable")
    return value["quietingDown"]


def ensure_accepting_requests(manager, expected_operation_id, *, expected_owner=None, expected_supervisor=None):
    owner = owner_observation(manager, expected=expected_owner, allow_starting=True)
    _require(owner["operationId"] == expected_operation_id, "Startup owner changed", "drain_owner_changed")
    if expected_supervisor is not None:
        _require(owner["supervisor"] == _identity(expected_supervisor), "Startup supervisor changed", "drain_owner_changed")
    quiet = quiet_observation(manager)
    if quiet:
        owner_observation(manager, expected=owner, allow_starting=True)
        status, _ = manager._request("cancelQuietDown", method="POST")
        _require(status in {200, 201, 302}, "Jenkins did not acknowledge quiet mode restoration")
    _require(quiet_observation(manager) is False, "Jenkins is still refusing new tasks")
    owner_observation(manager, expected=owner, allow_starting=True)
    return {"quietingDown": False, "acceptingRequests": True, "operationId": expected_operation_id}


def drain_owned(manager, timeout=30.0):
    owner = owner_observation(manager)
    initial = classify_executions(manager, owner, execution_inventory(manager.paths.state))
    was_quiet = quiet_observation(manager)
    owner_observation(manager, expected=owner)
    status, _ = manager._request("quietDown", method="POST")
    _require(status in {200, 201, 302}, "Jenkins refused quietDown", "drain_failed")
    deadline = time.monotonic() + timeout
    try:
        while True:
            owner_observation(manager, expected=owner)
            classified = classify_executions(manager, owner, execution_inventory(manager.paths.state))
            _require(classified["historicalExecutions"] == initial["historicalExecutions"]
                     and classified["protectedHolds"] == initial["protectedHolds"]
                     and classified["closedBeforeLaunchExecutions"] == initial["closedBeforeLaunchExecutions"],
                     "Historical execution provenance or protected holds changed")
            jobs = api_json(manager, "api/json?tree=quietingDown,jobs[name,builds[building]]")
            computers = api_json(manager, COMPUTER_QUERY)
            queue = api_json(manager, "queue/api/json?tree=items[id]")
            _require(type(jobs.get("quietingDown")) is bool and isinstance(jobs.get("jobs"), list)
                     and isinstance(queue.get("items"), list)
                     and all(isinstance(j, dict) and isinstance(j.get("builds"), list)
                             and all(isinstance(b, dict) and type(b.get("building")) is bool for b in j["builds"]) for j in jobs["jobs"]),
                     "Jenkins queue or build activity was unobservable")
            executors = executor_observation(computers)
            building = [j.get("name") for j in jobs["jobs"] if any(b["building"] for b in j["builds"])]
            if jobs["quietingDown"] and not executors["busyExecutors"] and not building and not queue["items"] and not classified["activeExecutions"]:
                final = classify_executions(manager, owner, execution_inventory(manager.paths.state))
                _require(final == classified, "Execution ownership changed before shutdown")
                owner_observation(manager, expected=owner)
                return {"status": "drained", "proof": {**classified, "busyExecutors": 0, "buildingJobs": [], "queueItems": [],
                        "executorObservation": executors, "quietingDown": True,
                        "historicalNativeAcceptance": False, "historicalHoldsReleased": False}}
            if time.monotonic() >= deadline:
                raise JenkinsError("drain_timeout", "Current or unknown Jenkins work did not drain", retryable=True)
            time.sleep(.25)
    except Exception:
        if not was_quiet:
            # Restore only the same still-live owner; never unpause a replacement.
            ensure_accepting_requests(manager, owner["operationId"], expected_owner=owner)
        raise

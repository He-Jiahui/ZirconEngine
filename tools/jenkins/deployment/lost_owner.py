"""Read-only authority for replacing an exactly bound, departed formal owner.

The caller holds deployment-start.lock throughout preparation, revalidation,
and the eventual history/pointer transaction.  This module never starts a
process, initializes State, writes a receipt, or manufactures NativeJob EOF.
The resulting in-memory capability authorizes only the reviewed lost-owner
recovery path.  Normal Stop/Start and real Windows boot proofs remain separate.
"""
from __future__ import annotations

from contextlib import closing
from dataclasses import dataclass
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import sqlite3
import sys
import time
import xml.etree.ElementTree as ET

from ..contracts import JenkinsError, canonical_json, digest
from .driver import _LAUNCHER, _plain, verify_driver
from .execution_activation import protected_holds
from . import keeper_control
from .drain_observations import _identity, classify_executions, execution_inventory, native_terminal
from .lifecycle import check_port_available
from .startup_observations import observe_departed_identity, observe_home_available, runtime_owner_inventory
from ..processes.boot import windows_boot_filetime


_ISSUED = object()
_READERS = {}
_REVIEWED_SELECTION_READERS = {
    "c59798ef7c4451a2d95b5a4ecc9bff161ae7fd818fedf0c7a7a1a9d82f390413",
}
SCOPES = {"execution-runtime", "quiet-management"}
KIND = "formal-lost-owner-interruption"
_FAILED_START = "failed-own-start"
_COMMAND_KIND = "formal-lost-owner-recovery-command"
_REQUEST_KIND = "formal-lost-owner-recovery-request"

# Reviewed launch semantics, independent of the containing driver digest:
# NativeJob is atomic/suspended, JOB_LIST and HANDLE_LIST exclude the Job
# handle, KILL_ON_JOB_CLOSE is the sole Job limit, and breakaway is disabled.
# The host retains controller/agent Jobs and records execution guards before
# resuming them; a fresh broker cannot claim launches from another operation.
# A different containment implementation needs its own source review.
_REVIEWED_CONTAINMENT = {
    "tools/jenkins/processes/native.py": "4b348ec33b5927f44bbaca9074f64e43343a76b5a7f2d5308850c768f1246009",
    "tools/jenkins/processes/job.py": "9985eaf1a93585b83769cc916a86a2694c411cead71932b57e104fdc5f6948ca",
    "tools/jenkins/deployment/host.py": "bc7676a14a684a8b513b5137e99e8c72e75d0ca525b7525fce70f530e67a4eb7",
    "tools/jenkins/deployment/lifecycle.py": "0cc9dc5a2d2c103df655e21e5fd5c290ee098aec828039c65c167fa7edb43a2a",
    "tools/jenkins/processes/registry.py": "c8247c6645e35bb67df6363590d64ef2447759798641b2aa49964b40bcb5b8bf",
    "tools/jenkins/deployment/quarantine.py": "a684aef0ac0bd5f6c5f2ad59f12c500cc2f38fc4fb511d25eeb926476160d3be",
    "tools/jenkins/processes/execution_host.py": "0a82803c69a4080a5a8536937254f534fe9cee50f0b4915115b255926a747f44",
}


def _require(condition, message, code="formal_lost_owner_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


def _raw(path):
    path = Path(path).absolute()
    _plain(path)
    return path.read_bytes()


def _sha(path):
    return hashlib.sha256(_raw(path)).hexdigest()


def _document(raw):
    def unique(items):
        result = {}
        for key, value in items:
            _require(key not in result, "Duplicate recovery evidence key")
            result[key] = value
        return result
    value = json.loads(raw, object_pairs_hook=unique,
                       parse_constant=lambda _: (_ for _ in ()).throw(
                           JenkinsError("formal_lost_owner_unproven", "Nonfinite recovery evidence")))
    _require(isinstance(value, dict), "Recovery evidence must be a typed object")
    return value


def _read(path):
    return _document(_raw(path))


def _reference(context, reference):
    return keeper_control._artifact(context, reference)


def _load_verified_selection(manager, context, release):
    """Compile only the original release's exact already-reviewed reader bytes."""
    path = context.repo / ".jenkins/state/migration-baseline/select_execution_upgrade.py"
    raw = _raw(path)
    checksum = hashlib.sha256(raw).hexdigest()
    _require(checksum in _REVIEWED_SELECTION_READERS
             and release.get("operatorFiles", {}).get(str(path)) == checksum,
             "Original immutable selection reader changed")
    cached = _READERS.get(checksum)
    if cached is None:
        name = "jenkins_lost_owner_reader_" + checksum
        module = importlib.util.module_from_spec(importlib.util.spec_from_loader(name, loader=None))
        module.__file__ = str(path)
        sys.modules[name] = module
        exec(compile(raw, str(path), "exec"), module.__dict__)
        cached = (raw, module)
        _READERS[checksum] = cached
    _require(cached[0] == raw, "Selection reader cache does not match its immutable source")
    reader = cached[1].UpgradeSelection(manager, helper_path=path,
                                       operator_generation=release.get("operatorGeneration"))
    if release.get("operatorGeneration") is not None:
        reader.verify_operator_generation(release["operatorGeneration"])
    return reader


def _verified_issuance(manager, *, previous_snapshot=None):
    from types import SimpleNamespace
    repo, state, home = (manager.paths.repo.absolute(), manager.paths.state.absolute(),
                         manager.paths.home.absolute())
    for path in (repo, state, home):
        _plain(path)
    _require(state == repo / ".jenkins/state" and home == repo / ".jenkins/jenkins_home",
             "Lost-owner recovery requires the existing formal deployment")
    context = SimpleNamespace(repo=repo, state=state)
    descriptor_path = state / "formal-lifecycle/keeper.json"
    descriptor_raw = _raw(descriptor_path)
    descriptor = _document(descriptor_raw)
    descriptor_sha = hashlib.sha256(descriptor_raw).hexdigest()
    _require(type(descriptor.get("schemaVersion")) is int and descriptor["schemaVersion"] == 1
             and descriptor.get("kind") == "formal-tray-strict-keeper-descriptor"
             and descriptor.get("admission") == "idle", "Only an idle formal keeper may be replaced")
    value = descriptor.get("anchor")
    _require(isinstance(value, dict) and value.get("state") == "running",
             "Unknown or stopped formal ownership requires its existing recovery protocol")
    protocol = keeper_control._protocol(repo)
    receipt = _reference(context, descriptor["runtimeReceipt"])
    anchor = protocol.RuntimeAnchor.from_verified_issuance(
        binding=value["binding"], keeper=value["keeper"], runtime_owner=value["owner"],
        selected_driver=value["selected"], protected_holds=value["protectedHolds"],
        journals=value["journals"], runtime_receipt=receipt,
        restriction_preflight_digest=value["restrictionPreflightDigest"], state=value["state"])
    _require(anchor.digest == descriptor.get("anchorDigest"), "Formal keeper anchor changed")
    binding = value["binding"]
    _require(binding["repository"] == str(repo) and binding["homePath"] == str(home)
             and binding["endpoint"] == {"address": manager.spec.controller["listenAddress"],
                                          "port": int(manager.spec.controller["httpPort"])},
             "Formal keeper belongs to a different repository, Home, or endpoint")
    release = _reference(context, binding["originalRelease"]).document
    origin = _reference(context, descriptor["keeperRecord"]).document
    _require(origin.get("kind") == "desktop-execution-upgrade-keeper"
             and origin.get("reviewedRelease") == binding["originalRelease"]
             and origin.get("automaticRestart") is False
             and digest(origin.get("restrictionPreflight")) == value["restrictionPreflightDigest"]
             and all(origin.get("restrictionPreflight", {}).get("holderIdentity", {}).get(key)
                     == value["keeper"].get(key) for key in ("pid", "creationTime", "executable")),
             "Original reviewed keeper issuance is incomplete")
    reader = _load_verified_selection(manager, context, release)
    keeper_control.assert_no_unresolved_keeper_command(manager)
    journal, start = _read(reader.journal), _read(reader.start_journal)
    pending_journal, pending_start = keeper_control._pending_chain(reader, journal, start)
    current_snapshot, current_selected = reader._snapshot(), _read(reader.selector)
    intent, body, owner = reader._started_chain(
        pending_journal, pending_start, check_activation=previous_snapshot is None)
    operation, host = (current_snapshot if previous_snapshot is None else
                       (previous_snapshot["operation"], previous_snapshot["host"]))
    selected = current_selected if previous_snapshot is None else previous_snapshot["selected"]
    _require(owner == value["owner"] == reader._owner(operation, host)
             and selected == value["selected"]
             and operation.get("state") == "running" and host.get("status") == "running"
             and pending_journal.get("startupReceipt") == descriptor["runtimeReceipt"],
             "Keeper descriptor does not bind the exact verified current lineage")
    expected_binding = keeper_control._binding(
        reader, intent, binding["originalRelease"], binding["configuration"],
        binding["deploymentSpec"], binding["workflowSpec"])
    expected_binding.update(upgradeIntent=pending_journal["intent"],
                            selectionReceipt=pending_journal["selectionReceipt"])
    _require(expected_binding == binding, "Keeper stable issuance binding changed")
    for key, path in (("selection", reader.journal), ("start", reader.start_journal)):
        _require(value["journals"][key] == {"path": str(path), "sha256": _sha(path)},
                 "Acceptance journals differ from the advertised keeper")
    _require(_sha(manager.spec.path) == binding["deploymentSpec"]["sha256"]
             and _sha(repo / ".jenkins/workflow-spec.json") == binding["workflowSpec"]["sha256"]
             and _sha(repo / ".codex/config.toml") == binding["configuration"]["sha256"],
             "The current specification or restriction configuration changed")
    _require(_sha(descriptor_path) == descriptor_sha and reader._snapshot() == current_snapshot
             and _read(reader.selector) == current_selected and _read(reader.journal) == journal
             and _read(reader.start_journal) == start, "Formal issuance changed during immutable verification")
    return {"descriptor": descriptor, "descriptorSha256": descriptor_sha,
            "anchor": anchor, "owner": owner, "operation": operation, "host": host,
            "selected": selected, "reader": reader, "release": release,
            "startupBodyDigest": digest(body)}


def _verified_containment(issuance):
    selected = issuance["selected"]
    repo = issuance["reader"].repo
    root = repo / ".jenkins/runtime/drivers" / selected["driverDigest"]
    _require(Path(selected["root"]).absolute() == root, "Containment driver escaped the runtime")
    verify_driver(root, expected_digest=selected["driverDigest"])
    manifest = _read(root / "driver-manifest.json")
    for name, expected in _REVIEWED_CONTAINMENT.items():
        _require(manifest["files"].get(name, {}).get("sha256") == expected
                 and _sha(root / name) == expected, "This sealed containment implementation has no review")
    return {"driverDigest": selected["driverDigest"], "manifestSha256": _sha(root / "driver-manifest.json"),
            "sourceHashes": dict(_REVIEWED_CONTAINMENT), "sourceReviewDigest": digest(_REVIEWED_CONTAINMENT),
            "nativeTerminationClaimed": False}


def _departure(expected):
    value = observe_departed_identity(expected)
    _require(value.get("pid") == expected["pid"]
             and value.get("expectedCreationTime") == expected["creationTime"], "Departed identity binding changed")
    if value.get("status") == "absent":
        _require(set(value) == {"pid", "expectedCreationTime", "status"}, "Conflicting departed identity evidence")
    else:
        actual = value.get("observedIdentity", {})
        _require(value.get("status") == "pid-reused" and type(actual.get("pid")) is int
                 and actual["pid"] == expected["pid"] and isinstance(actual.get("creationTime"), str)
                 and actual["creationTime"].isascii() and actual["creationTime"].isdecimal()
                 and int(actual["creationTime"]) > int(expected["creationTime"])
                 and isinstance(actual.get("executable"), str) and bool(actual["executable"]),
                 "Live or inconclusive ownership cannot authorize replacement")
    return value


def _resource_inventory(state):
    path = state / "coordination.sqlite3"
    _plain(path)
    with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as connection:
        connection.execute("PRAGMA query_only=ON")
        connection.execute("BEGIN")
        rows = [{"domain": domain, "key": key, "version": version, "payload": _document(payload)}
                for domain, key, version, payload in connection.execute(
                    "SELECT domain,key,version,payload FROM records WHERE domain IN "
                    "('resource_policy','capacity_inventory','resource_queue') ORDER BY domain,key")]
    _require(any(row["domain"] == "resource_policy" and row["key"] == "default" for row in rows),
             "The existing admission budget is unavailable")
    return rows


def _execution_gate(manager, issuance):
    """Reuse existing native proofs and historical provenance, without mutation."""
    owner = issuance["owner"]
    runtime = owner["operation"]["operationId"]
    drain_owner = {"operationId": runtime, "generation": runtime,
                   "driverDigest": issuance["selected"]["driverDigest"],
                   "homePath": owner["operation"]["homePath"],
                   **{key: owner[key] for key in ("supervisor", "controller", "agent")}}
    rows = execution_inventory(manager.paths.state)
    classification = classify_executions(manager, drain_owner, rows)
    _require(classification["activeExecutions"] == [],
             "Current native work must retain its original recovery authority", "formal_lost_owner_execution_active")
    for row in rows:
        payload = row["payload"]
        if row["domain"] == "native_job" and native_terminal(row):
            proof = payload["completeProof"]
            _require(payload.get("terminalProof") == proof
                     and isinstance(proof["identity"].get("commandDigest"), str)
                     and re.fullmatch("[0-9a-f]{64}", proof["identity"]["commandDigest"])
                     and payload.get("commandDigest", proof["identity"]["commandDigest"])
                         == proof["identity"]["commandDigest"], "Existing native terminal proof is inconsistent")
        bindings = [payload, payload.get("driver"), payload.get("parameters")]
        current = any(isinstance(value, dict) and any(value.get(field) == runtime for field in
                      ("operationId", "runtimeOperationId", "brokerGeneration")) for value in bindings)
        if row["domain"] == "jenkins_submission" and current:
            _require(payload.get("delivery") in {"submitted", "completed", "accepted"},
                     "Current dispatch outcome remains unknown", "formal_lost_owner_execution_active")
    holds = protected_holds(manager.paths.state)
    _require(holds == issuance["anchor"].value["protectedHolds"] == classification["protectedHolds"],
             "Protected reservations must retain their exact reviewed ownership")
    return {"recordsDigest": digest(rows), "classification": classification,
            "protectedHolds": holds, "protectedHoldsDigest": digest(holds),
            "resourceInventoryDigest": digest(_resource_inventory(manager.paths.state)),
            "nativeProofIssued": False, "executionHoldsReleaseAllowed": False}


def _empty_persisted_queue(root):
    """Accept empty queue entries and the default strategy's saved ID only."""
    def counter(element):
        text = (element.text or "").strip()
        number = text.lstrip("0") or "0"
        return (len(element) == 0 and text.isascii() and text.isdecimal()
                and len(number) <= 19 and int(number) <= 2 ** 63 - 1)

    fields = [item.tag for item in root]
    if (root.tag not in {"queue", "hudson.model.Queue_-State"}
            or (root.text or "").strip() or len(fields) != len(set(fields))
            or not set(fields) <= {"items", "pendingItems", "counter", "properties"}
            or any(node.attrib or (node.tail or "").strip() for node in root.iter())):
        return False
    for item in root:
        if item.tag in {"items", "pendingItems"}:
            if len(item) or (item.text or "").strip():
                return False
        elif item.tag == "counter":
            if not counter(item):
                return False
        else:
            if (root.tag != "hudson.model.Queue_-State" or "items" not in fields
                    or (item.text or "").strip() or [child.tag for child in item] != ["entry"]):
                return False
            entry = item[0]
            if ((entry.text or "").strip() or [child.tag for child in entry] != ["string", "long"]
                    or len(entry[0]) or entry[0].text != "jenkins.model.queue.QueueIdStrategy$DefaultStrategy"
                    or not counter(entry[1])):
                return False
    return True


def _home_policy(home, scope):
    config = home / "config.xml"
    queue = home / "queue.xml"
    _plain(home)
    _plain(config)
    _plain(queue)
    raw = _raw(config)
    try:
        config_root = ET.fromstring(raw)
    except ET.ParseError as error:
        raise JenkinsError("formal_lost_owner_home_unproven", "Persisted Jenkins configuration is malformed XML",
                           retryable=True) from error
    _require(config_root.findtext("numExecutors") == "0",
             "The controller must retain zero build executors")
    queue_raw = _raw(queue) if queue.exists() else None
    if queue_raw is not None and scope == "execution-runtime":
        try:
            root = ET.fromstring(queue_raw)
        except ET.ParseError as error:
            raise JenkinsError("formal_lost_owner_queue_pending", "Persisted Jenkins queue is malformed XML",
                               retryable=True) from error
        _require(_empty_persisted_queue(root),
                 "Persisted Jenkins queue requires separate reconciliation", "formal_lost_owner_queue_pending")
    return {"configurationSha256": hashlib.sha256(raw).hexdigest(),
            "queueSha256": hashlib.sha256(queue_raw).hexdigest() if queue_raw is not None else None,
            "controllerExecutors": 0}


def _lifecycle_fence(manager):
    state = manager.paths.state
    names = ("deployment-operation.json", "deployment/host.json", "deployment/driver.json",
             "deployment/execution-activation.json", "deployment/execution-upgrade-selection.json",
             "deployment/execution-upgrade-start.json", "formal-lifecycle/keeper.json",
             "deployment/formal-lifecycle-command.json")
    result = {}
    for name in names:
        path = state / name
        _plain(path)
        result[name] = _sha(path) if path.exists() else None
    _require(all(result[name] is not None for name in names[:-1]), "Current lifecycle evidence is incomplete")
    return result


def _assert_request_unused(state, request_id):
    for area in ("requests", "results"):
        path = state / "formal-lifecycle" / area / (request_id + ".json")
        _plain(path)
        _require(not path.exists(), "A used formal request cannot become recovery authority", "formal_lost_owner_replay")
    history = state / "deployment/history"
    _plain(history)
    for path in sorted(history.glob("formal-lost-owner-interruption-*.json")):
        body = _read(path)
        _require(body.get("requestId") != request_id, "Lost-owner recovery request already has history",
                 "formal_lost_owner_replay")


def _capture(manager, request_id, scope):
    _require(isinstance(request_id, str) and re.fullmatch("[0-9a-f]{32}", request_id)
             and scope in SCOPES, "An explicit unique recovery request and bounded scope are required")
    _assert_request_unused(manager.paths.state, request_id)
    issuance = _verified_issuance(manager)
    containment = _verified_containment(issuance)
    fence = _lifecycle_fence(manager)
    value = issuance["anchor"].value
    boot = str(windows_boot_filetime())
    identities = {"keeper": value["keeper"],
                  **{key: value["owner"][key] for key in ("supervisor", "controller", "agent")}}
    for expected in identities.values():
        _departure(expected)
    inventory = runtime_owner_inventory(manager.paths.repo)
    _require(inventory == [], "A repository runtime still may own Home", "formal_lost_owner_runtime_live")
    execution = _execution_gate(manager, issuance)
    policy = _home_policy(manager.paths.home, scope)
    home = observe_home_available(manager.paths.home)
    endpoint = value["binding"]["endpoint"]
    check_port_available(endpoint["address"], endpoint["port"])
    departed = {key: _departure(expected) for key, expected in identities.items()}
    _require(str(windows_boot_filetime()) == boot, "Windows boot changed during lost-owner observation")
    _require(runtime_owner_inventory(manager.paths.repo) == inventory,
             "Runtime ownership changed during recovery observation")
    _require(_execution_gate(manager, issuance) == execution and _home_policy(manager.paths.home, scope) == policy,
             "Execution quarantine, budget, or Home changed during observation")
    _require(_lifecycle_fence(manager) == fence, "Current keeper lineage changed during observation")
    return {"schemaVersion": 1, "kind": KIND, "requestId": request_id, "scope": scope,
            "operationId": value["owner"]["operation"]["operationId"],
            "homePath": value["binding"]["homePath"], "binding": value["binding"],
            "owner": value["owner"], "departedKeeper": value["keeper"],
            "descriptorSha256": issuance["descriptorSha256"], "anchorDigest": issuance["anchor"].digest,
            "keeperRecord": issuance["descriptor"]["keeperRecord"],
            "previousRuntimeReceipt": issuance["descriptor"]["runtimeReceipt"],
            "selectedDriverDigest": digest(issuance["selected"]),
            "operationDigest": digest(issuance["operation"]), "hostRecordDigest": digest(issuance["host"]),
            "startupBodyDigest": issuance["startupBodyDigest"], "lifecycleFence": fence,
            "containmentReview": containment, "bootFileTime": boot, "departedIdentities": departed,
            "runtimeOwnerInventory": inventory, "executionQuarantine": execution,
            "homePolicy": policy, "homeObservation": home, "endpointAvailable": endpoint,
            "controlPlaneOnly": scope == "quiet-management", "outcome": "interrupted",
            "nativeProofIssued": False, "historicalEofClaimed": False, "bootBoundaryClaimed": False,
            "validationAcceptanceAllowed": False, "executionHoldsReleaseAllowed": False,
            "runtimeAcceptance": False, "historicalHoldsReleased": False}


def _failure_reference(state, reference, *, mutable=(), release=False):
    _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}
             and isinstance(reference["path"], str)
             and re.fullmatch("[0-9a-f]{64}", str(reference["sha256"])),
             "Failed-start evidence requires an exact file reference")
    path = Path(reference["path"]).absolute()
    allowed = {state / "migration-baseline", state / "deployment/history",
               state / "recovery-lifecycle/history"}
    if release:
        allowed.add(state / "deployment")
    _require(str(path) == reference["path"] and ".." not in path.parts
             and (path in mutable or path.parent in allowed), "Failed-start evidence escaped its namespace")
    raw = _raw(path)
    _require(hashlib.sha256(raw).hexdigest() == reference["sha256"], "Failed-start evidence bytes changed")
    return _document(raw), raw


def _file_reference(path):
    return {"path": str(Path(path).absolute()), "sha256": _sha(path)}


def _replacement_command(manager, transition, recovery_digest):
    state = manager.paths.state.absolute()
    command = _read(state / "recovery-lifecycle/command.json")
    _require(command.get("schemaVersion") == 1 and command.get("kind") == _COMMAND_KIND
             and command.get("status") in {"pending", "start-attempted"}
             and all(command.get(key) == transition[key] for key in ("keeper", "release", "request"))
             and command.get("recoveryDigest") == recovery_digest
             and command.get("replacesOriginalCommand") == transition["originalCommand"]
             and command.get("failedOwnStartInterruption") == transition["interruption"]
             and command.get("previousRuntimeReceipt") is None
             and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False
             and not {"createdSupervisor", "result", "nativeTerminationProof", "agentTerminationProof"} & command.keys(),
             "Only the privately bound replacement request may supersede this UNKNOWN command")
    for reference in (transition["release"], transition["request"], transition["interruption"],
                      transition["originalCommand"], transition["startTransaction"], transition["attemptedHost"]):
        _failure_reference(state, reference, release=reference == transition["release"])


def _failed_historical_regression(repo, state, release, target):
    """An earlier complete receipt covers its sealed inventory, not new files."""
    receipt, _ = _failure_reference(state, release["regression"])
    from .support_regression import SCOPED_SCOPE, verify_support_regression
    if receipt.get("scope") == SCOPED_SCOPE:
        receipt = verify_support_regression(repo, release["regression"], target, current=False, state_root=state,
            required_operators=(*_REVIEWED_CONTAINMENT, "tools/jenkins/deployment/manager.py",
                "tools/jenkins/deployment/recovery_keeper.py", "tools/jenkins/deployment/lost_owner.py",
                "tools/jenkins/deployment/start_transaction.py"))
        for field, name in (("deploymentSpec", ".jenkins/deployment-spec.json"), ("workflowSpec", ".jenkins/workflow-spec.json")):
            _require(receipt["sourceFiles"][name] == release[field]["sha256"], "Original tested deployment specifications changed")
        return receipt
    _require(Path(release["regression"]["path"]).parent == state / "migration-baseline"
             and receipt.get("scope") == "complete-current-Jenkins-support-pytest-discovery"
             and receipt.get("status") == "passed" and receipt.get("sourceIdentityStable") is True
             and type(receipt.get("testsRun")) is int and receipt["testsRun"] > 0
             and receipt.get("exitCode") == 0 and receipt.get("failures") == 0 and receipt.get("errors") == 0
             and receipt.get("collectionErrors") == [], "Original complete support regression is not passing")
    log = Path(receipt.get("log", "")).absolute()
    _require(str(log) == receipt.get("log") and ".." not in log.parts
             and log.parent == state / "migration-baseline"
             and re.fullmatch("[0-9a-f]{64}", str(receipt.get("logSha256")))
             and _sha(log) == receipt["logSha256"], "Original support regression output changed")
    sources = receipt.get("sourceFiles")
    _require(isinstance(sources, dict) and sources, "Original tested source inventory is missing")
    expected, tests = {}, []
    for name, checksum in sources.items():
        relative = PurePosixPath(name)
        _require(relative.as_posix() == name and not relative.is_absolute() and "\\" not in name and ":" not in name
                 and not any(part in {".", ".."} for part in relative.parts)
                 and re.fullmatch("[0-9a-f]{64}", str(checksum))
                 and ((name.startswith("tools/jenkins/") and name.endswith(".py"))
                      or (name.startswith(".jenkins/pipeline/") and name.endswith(".groovy"))
                      or name in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}),
                 "Original tested inventory contains an undeclared path")
        if name.startswith("tools/jenkins/") and "/tests/" in name:
            tests.append(name)
        elif name not in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}:
            expected[name] = checksum
    required = {*_REVIEWED_CONTAINMENT, "tools/jenkins/deployment/manager.py",
                "tools/jenkins/deployment/recovery_keeper.py", "tools/jenkins/deployment/lost_owner.py",
                "tools/jenkins/deployment/start_transaction.py", ".jenkins/deployment-spec.json",
                ".jenkins/workflow-spec.json"}
    _require(tests and required <= set(sources), "Original support receipt lacks its complete recovery containment")
    expected["launcher.py"] = hashlib.sha256(_LAUNCHER.encode("utf-8")).hexdigest()
    expected["tools/__init__.py"] = hashlib.sha256(
        b'"""This package belongs to one verified Jenkins snapshot."""\n').hexdigest()
    manifest = _read(target["manifest"])
    observed = {".jenkins/" + name if name.startswith("pipeline/") else name: entry["sha256"]
                for name, entry in manifest["files"].items()}
    _require(observed == expected, "Original passed inventory differs from its full sealed runtime")
    for field, name in (("deploymentSpec", ".jenkins/deployment-spec.json"), ("workflowSpec", ".jenkins/workflow-spec.json")):
        _require(sources[name] == release[field]["sha256"], "Original tested deployment specifications changed")
    return receipt


def _failed_recovery_release(manager, command, transaction, selected):
    """Verify historical sealed bytes, independent of subsequently edited source."""
    from .recovery_keeper import _restriction
    repo, state = manager.paths.repo.absolute(), manager.paths.state.absolute()
    release, _ = _failure_reference(state, command["release"], release=True)
    _require(release.get("schemaVersion") == 1 and release.get("kind") == "formal-lost-owner-recovery-release"
             and release.get("scope") == "execution-runtime" and release.get("automaticRestart") is False
             and release.get("queueReplayRequested") is False
             and release.get("runtimeAcceptance") is False and release.get("cargoAcceptance") is False
             and release.get("failedOwnStart") is None, "Only the original reviewed first recovery attempt is in scope")
    for key, path in (("config", repo / ".codex/config.toml"), ("deploymentSpec", manager.spec.path.absolute()),
                      ("workflowSpec", repo / ".jenkins/workflow-spec.json")):
        _require(release.get(key) == _file_reference(path), "Original recovery specification or configuration changed")
    target = release["targetDriver"]
    root = repo / ".jenkins/runtime/drivers" / target["driverDigest"]
    verified = verify_driver(root, expected_digest=target["driverDigest"])
    _require(target == verified and all(selected.get(key) == value for key, value in verified.items())
             and selected.get("recoveryRelease") == command["release"]
             and selected.get("supportRegression") == release["regression"]
             and selected.get("preparation") == release["preparation"]["path"],
             "Failed startup differs from the exact reviewed sealed target")
    historical = _failed_historical_regression(repo, state, release, verified)
    operators = release.get("operatorFiles")
    _require(isinstance(operators, dict) and operators
             and all(historical["sourceFiles"].get(Path(name).relative_to(repo).as_posix()) == checksum
                     for name, checksum in operators.items() if Path(name).is_relative_to(repo / "tools/jenkins")),
             "Original recovery operators differ from their passed sealed implementation")
    for name, checksum in operators.items():
        path = Path(name).absolute()
        _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo)
                 and re.fullmatch("[0-9a-f]{64}", str(checksum)), "Original operator inventory is malformed")
    preparation, _ = _failure_reference(state, release["preparation"])
    _require(preparation.get("schemaVersion") == 1 and preparation.get("state") == "prepared-not-selected"
             and preparation.get("driver") == target and preparation.get("protectedHolds") == release["protectedHolds"]
             and preparation.get("supportRegression", preparation.get("regression")) == release["regression"]
             and preparation.get("previousSelectedDriver") == transaction["previousSelector"]
             and preparation.get("previousSelectorSha256") == transaction["previousSelectorSha256"],
             "Original target preparation and predecessor evidence changed")
    spawn = command.get("createdSupervisor", {})
    _require(spawn.get("creator") == command["keeper"] and spawn.get("child") == transaction["createdIdentity"]
             and spawn.get("creationFlags") == 0x08000200 and spawn.get("requestedProfile") == "zircon-strict"
             and spawn.get("creatorInJob") is True and spawn.get("childInJob") is True,
             "The failed attempt lacks its exact inherited strict creation evidence")
    for key in ("actualRestrictionPreflight", "associatedJobAtSpawn"):
        _restriction(spawn.get(key, {}), command["keeper"], release["config"])
    members = spawn["associatedJobAtSpawn"]["associatedJob"]["memberPids"]
    _require(command["keeper"]["pid"] in members and transaction["createdIdentity"]["pid"] in members,
             "Original strict parent and supervisor membership is incomplete")
    args = [str(repo / ".jenkins/runtime/python/python.exe"), "-B", str(repo / "tools/dev/jenkins_recovery_keeper.py"),
            "--config-sha256", release["config"]["sha256"], "--release-file", command["release"]["path"],
            "--release-sha256", command["release"]["sha256"]]
    _require(command["keeper"]["commandDigest"] in {
        hashlib.sha256("\0".join(value).encode("utf-8")).hexdigest() for value in (args, [*args, "--no-tray"])},
        "Original keeper command is not bound to its reviewed release")
    return release


def _failed_issuance(manager, original_command_ref=None, transition=None, recovery_digest=None):
    repo, state, home = (manager.paths.repo.absolute(), manager.paths.state.absolute(), manager.paths.home.absolute())
    command_path = state / "recovery-lifecycle/command.json"
    if transition is None:
        reference = _file_reference(command_path)
        _require(original_command_ref in (None, reference), "The explicitly reviewed UNKNOWN command changed")
        command, command_raw = _failure_reference(state, reference, mutable=(command_path,))
    else:
        _replacement_command(manager, transition, recovery_digest)
        reference = original_command_ref
        command, command_raw = _failure_reference(state, transition["originalCommand"])
        _require(reference == {"path": str(command_path), "sha256": transition["originalCommand"]["sha256"]},
                 "Replacement does not preserve the original UNKNOWN pointer bytes")
    _require(command.get("schemaVersion") == 1 and command.get("kind") == _COMMAND_KIND
             and command.get("status") == "unknown" and command.get("runtimeAcceptance") is False
             and command.get("historicalHoldsReleased") is False and command.get("previousRuntimeReceipt") is None
             and "result" not in command, "Only an unresolved first recovery start can be reviewed")
    operation, host, attempted, selected = (_read(state / name) for name in
        ("deployment-operation.json", "deployment/host.json", "deployment/host.tmp", "deployment/driver.json"))
    runtime = operation.get("operationId")
    _require(isinstance(runtime, str) and re.fullmatch("start-[1-9][0-9]*", runtime)
             and operation.get("state") == "failed" and operation.get("generation") == runtime
             and operation.get("component") == "lifecycle-host" and operation.get("controlPlaneOnly") is False
             and operation.get("homePath") == str(home), "Exact failed execution startup is required")
    transaction_path = state / "recovery-lifecycle/start-transactions" / (runtime + ".json")
    transaction = _read(transaction_path)
    _require(transaction.get("schemaVersion") == 1 and transaction.get("kind") == "formal-recovery-start-transaction"
             and transaction.get("state") == "unknown" and transaction.get("operationId") == runtime
             and transaction.get("spawnAttempted") is True and transaction.get("homePath") == str(home)
             and all(transaction.get(key) == command[key] for key in ("keeper", "release", "request"))
             and all(transaction.get(key) is False for key in ("automaticRetry", "runtimeAcceptance", "cargoAcceptance",
                                                              "nativeTerminationProven", "historicalHoldsReleased"))
             and transaction.get("operation") == operation and transaction.get("selected") == selected
             and transaction.get("selectedSha256") == _sha(state / "deployment/driver.json"),
             "The original UNKNOWN spawn transaction is incomplete or changed")
    keeper = _identity(command["keeper"], repo / ".jenkins/runtime/python/python.exe")
    supervisor = _identity(operation, repo / ".jenkins/runtime/python/python.exe")
    controller = _identity(attempted.get("controllerIdentity"), manager.java)
    agent = _identity(attempted.get("agentIdentity"), manager.java)
    _require(transaction.get("createdIdentity") == supervisor and transaction.get("createdPid") == supervisor["pid"]
             and operation.get("hostPid") == supervisor["pid"] and host.get("hostPid") == supervisor["pid"]
             and attempted.get("hostPid") == supervisor["pid"]
             and host.get("status") == "starting" and host.get("agentLaunchAttempted") is False
             and "agentIdentity" not in host and host.get("identity") == controller
             and attempted.get("identity") == controller and attempted.get("status") == "running"
             and attempted.get("agentLaunchAttempted") is True and attempted.get("controlPlaneOnly") is False
             and attempted.get("healthProof") == {"controller": True, "agent": True, "ready": True}
             and all(value.get("operationId") == value.get("generation") == runtime for value in (host, attempted))
             and attempted.get("startedAt") == host.get("startedAt")
             and all(not {"nativeTerminationProof", "agentTerminationProof", "terminalProof"} & value.keys()
                     for value in (operation, host, attempted)),
             "Committed starting metadata and the uncommitted full agent attempt must both be preserved")
    identities = {"keeper": keeper, "supervisor": supervisor, "controller": controller, "agent": agent}
    _require(len({value["pid"] for value in identities.values()}) == 4
             and all(isinstance(value["creationTime"], str)
                     and value["creationTime"] == str(int(value["creationTime"])) for value in identities.values())
             and int(keeper["creationTime"]) <= int(supervisor["creationTime"]) <= int(controller["creationTime"])
             <= int(agent["creationTime"]), "Failed-attempt native identities are not distinct and ordered")
    args = [str(repo / ".jenkins/runtime/python/python.exe"), selected["launcher"], "__lifecycle-host", "--spec",
            str(manager.spec.path.absolute()), "--java", str(manager.java), "--war", str(manager.war),
            "--operation-id", runtime]
    command_digest = hashlib.sha256("\0".join(args).encode("utf-8")).hexdigest()
    _require(transaction.get("requestedCommand") == args and transaction.get("requestedCommandDigest") == command_digest
             and supervisor["commandDigest"] == command_digest and selected.get("runtimeOperationId") == runtime
             and selected.get("state") == "selected-for-runtime-start"
             and selected.get("runtimeAcceptance") is False and selected.get("cargoAcceptance") is False,
             "Failed supervisor command or selector differs from its recorded spawn attempt")
    release = _failed_recovery_release(manager, command, transaction, selected)
    request, _ = _failure_reference(state, command["request"])
    _require(request.get("schemaVersion") == 1 and request.get("kind") == _REQUEST_KIND
             and request.get("action") == "recover" and request.get("requestId") == release["requestId"]
             and all(request.get(key) == command[key] for key in ("keeper", "release", "recoveryDigest"))
             and request.get("runtimeAcceptance") is False, "The failed original request is not immutable and exact")
    interruption = transaction["interruption"]
    initial = {key: value for key, value in interruption.items() if key not in {"observedAt", "recoveryDigest"}}
    _require(interruption.get("recoveryDigest") == command["recoveryDigest"] == digest(initial)
             and initial.get("schemaVersion") == 1 and initial.get("kind") == KIND
             and initial.get("scope") == "execution-runtime" and initial.get("requestId") == request["requestId"]
             and initial.get("outcome") == "interrupted"
             and initial.get("owner") == release["expectedPreviousOwner"]
             and initial.get("lifecycleFence") == release["expectedLifecycleFence"]
             and operation.get("lostOwnerRecoveryDigest") == command["recoveryDigest"]
             and all(initial.get(key) is False for key in ("nativeProofIssued", "historicalEofClaimed",
                 "bootBoundaryClaimed", "runtimeAcceptance", "historicalHoldsReleased", "executionHoldsReleaseAllowed")),
             "The failed attempt is not derived from its exact original lost-owner review")
    initial_paths = [path for path in sorted((state / "recovery-lifecycle/history").glob("recovery-initial-plan-*.json"))
                     if canonical_json(_read(path)) == canonical_json(initial)]
    _require(len(initial_paths) == 1, "Original recovery plan requires one exact immutable issuance")
    previous = _verified_issuance(manager, previous_snapshot={"operation": transaction["previousOperation"],
        "host": transaction["previousHost"], "selected": transaction["previousSelector"]})
    _require(initial["owner"] == previous["owner"] and initial["descriptorSha256"] == previous["descriptorSha256"]
             and initial["anchorDigest"] == previous["anchor"].digest
             and initial["previousRuntimeReceipt"] == previous["descriptor"]["runtimeReceipt"]
             and initial["containmentReview"] == _verified_containment(previous)
             and initial["operationDigest"] == digest(transaction["previousOperation"])
             and initial["hostRecordDigest"] == digest(transaction["previousHost"])
             and initial["selectedDriverDigest"] == digest(transaction["previousSelector"])
             and transaction["previousSelectorSha256"] == initial["lifecycleFence"]["deployment/driver.json"]
             and release["protectedHolds"] == previous["anchor"].value["protectedHolds"],
             "Original formal lineage and retained historical reservations changed")
    for name in ("deployment/execution-activation.json", "deployment/execution-upgrade-selection.json",
                 "deployment/execution-upgrade-start.json", "formal-lifecycle/keeper.json",
                 "deployment/formal-lifecycle-command.json"):
        path = state / name
        _require((_sha(path) if path.exists() else None) == initial["lifecycleFence"][name],
                 "Original formal journals or native quarantine authority changed")
    owner = {"operation": {key: operation[key] for key in
             ("operationId", "generation", "hostPid", "homePath", "component", "controlPlaneOnly")},
             "supervisor": supervisor, "controller": controller, "agent": agent}
    artifacts = {"originalCommand": (reference, command_raw)}
    for key, path, expected in (("startTransaction", transaction_path, transaction),
                                ("attemptedHost", state / "deployment/host.tmp", attempted),
                                ("committedHost", state / "deployment/host.json", host)):
        raw = _raw(path)
        _require(_document(raw) == expected, "Failure evidence changed during lineage verification")
        artifacts[key] = ({"path": str(path), "sha256": hashlib.sha256(raw).hexdigest()}, raw)
    if transition is not None:
        for key in ("originalCommand", "startTransaction", "attemptedHost"):
            _, archived_raw = _failure_reference(state, transition[key])
            _require(archived_raw == artifacts[key][1], "Archived failure evidence does not preserve its original bytes")
    return {"owner": owner, "identities": identities, "operation": operation, "host": host,
            "selected": selected, "release": release, "previous": previous, "initial": initial,
            "initialReference": _file_reference(initial_paths[0]), "artifacts": artifacts,
            "command": command, "request": request}


def _capture_failed(manager, request_id, scope, original_command_ref=None, transition=None, recovery_digest=None):
    _require(isinstance(request_id, str) and re.fullmatch("[0-9a-f]{32}", request_id)
             and scope == "execution-runtime", "Failed own startup recovery requires a new explicit execution request")
    _assert_request_unused(manager.paths.state, request_id)
    for area in ("requests", "results"):
        _require(not (manager.paths.state / "recovery-lifecycle" / area / (request_id + ".json")).exists(),
                 "A recovery lifecycle request cannot be replayed", "formal_lost_owner_replay")
    issuance = _failed_issuance(manager, original_command_ref, transition, recovery_digest)
    _require(request_id != issuance["request"]["requestId"], "The failed original request cannot be resubmitted")
    fence = _lifecycle_fence(manager)
    boot = str(windows_boot_filetime())
    _require(boot == issuance["initial"]["bootFileTime"]
             and all(int(value["creationTime"]) >= int(boot) for value in issuance["identities"].values()),
             "The reviewed failed attempt must remain in its exact actual Windows boot")
    containment = _verified_containment({**issuance["previous"], "selected": issuance["selected"]})
    for expected in issuance["identities"].values():
        _departure(expected)
    inventory = runtime_owner_inventory(manager.paths.repo)
    _require(inventory == [], "A runtime or failed owner may still own Home", "formal_lost_owner_runtime_live")
    execution = _execution_gate(manager, {**issuance["previous"], "owner": issuance["owner"],
                                         "selected": issuance["selected"]})
    _require(execution == issuance["initial"]["executionQuarantine"],
             "Failed startup must preserve every native record, protected hold, and existing budget",
             "formal_lost_owner_execution_active")
    policy, home = _home_policy(manager.paths.home, scope), observe_home_available(manager.paths.home)
    binding = {**issuance["initial"]["binding"], "driverDigest": issuance["selected"]["driverDigest"],
               "driverRoot": issuance["selected"]["root"]}
    endpoint = binding["endpoint"]
    check_port_available(endpoint["address"], endpoint["port"])
    departed = {key: _departure(expected) for key, expected in issuance["identities"].items()}
    _require(str(windows_boot_filetime()) == boot and runtime_owner_inventory(manager.paths.repo) == inventory,
             "Actual boot or runtime ownership changed during failed-start review")
    _require(_execution_gate(manager, {**issuance["previous"], "owner": issuance["owner"],
                                      "selected": issuance["selected"]}) == execution
             and _home_policy(manager.paths.home, scope) == policy and _lifecycle_fence(manager) == fence
             and _read(manager.paths.state / "deployment-operation.json") == issuance["operation"]
             and _read(manager.paths.state / "deployment/driver.json") == issuance["selected"],
             "Failed startup lineage, native inventory, budget, or Home changed during review")
    _require(_verified_containment({**issuance["previous"], "selected": issuance["selected"]}) == containment,
             "Original sealed containment changed during failed-start review")
    for reference in (issuance["command"]["release"], issuance["command"]["request"], issuance["initialReference"]):
        _failure_reference(manager.paths.state.absolute(), reference, release=reference == issuance["command"]["release"])
    for field, path in (("config", manager.paths.repo / ".codex/config.toml"),
                        ("deploymentSpec", manager.spec.path), ("workflowSpec", manager.paths.repo / ".jenkins/workflow-spec.json")):
        _require(issuance["release"][field] == _file_reference(path), "Reviewed startup binding changed during observation")
    for key, (reference, raw) in issuance["artifacts"].items():
        if key == "originalCommand" and transition is not None:
            _replacement_command(manager, transition, recovery_digest)
        else:
            _require(_raw(reference["path"]) == raw, "Original failure evidence changed during review")
    previous = issuance["previous"]
    value = {"schemaVersion": 1, "kind": KIND, "recoveryVariant": _FAILED_START, "requestId": request_id,
        "scope": scope, "operationId": issuance["operation"]["operationId"], "homePath": str(manager.paths.home.absolute()),
        "owner": issuance["owner"], "binding": binding, "departedKeeper": issuance["identities"]["keeper"],
        "descriptorSha256": previous["descriptorSha256"], "anchorDigest": previous["anchor"].digest,
        "keeperRecord": issuance["command"]["request"], "previousRuntimeReceipt": previous["descriptor"]["runtimeReceipt"],
        "selectedDriverDigest": digest(issuance["selected"]), "operationDigest": digest(issuance["operation"]),
        "hostRecordDigest": digest(issuance["host"]), "startupBodyDigest": previous["startupBodyDigest"],
        "lifecycleFence": fence, **{key: ref for key, (ref, _) in issuance["artifacts"].items()},
        "originalRequest": issuance["command"]["request"], "previousRecoveryRelease": issuance["command"]["release"],
        "originalInterruption": issuance["initialReference"], "originalRecoveryDigest": issuance["command"]["recoveryDigest"],
        "containmentReview": containment, "bootFileTime": boot, "departedIdentities": departed,
        "runtimeOwnerInventory": inventory, "executionQuarantine": execution, "homePolicy": policy,
        "homeObservation": home, "endpointAvailable": endpoint, "controlPlaneOnly": False, "outcome": "interrupted",
        "nativeProofIssued": False, "nativeTerminationProven": False, "historicalEofClaimed": False,
        "bootBoundaryClaimed": False, "validationAcceptanceAllowed": False, "executionHoldsReleaseAllowed": False,
        "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False,
        "attemptedHostCommitted": False, "automaticRetry": False}
    return value, issuance["artifacts"]


@dataclass(frozen=True, init=False)
class LostFormalOwnerRecovery:
    """Unserializable issuance authority; JSON is evidence, never a capability."""
    snapshot: bytes
    _token: object
    _variant: str | None
    _artifacts: tuple
    _transition: bytes | None

    def __init__(self, snapshot, token=None, *, variant=None, artifacts=(), transition=None):
        _require(token is _ISSUED and type(snapshot) is bytes,
                 "Recovery authority must be issued by the reviewed read-only preparation")
        _require(variant in {None, _FAILED_START} and type(artifacts) is tuple
                 and (transition is None or variant == _FAILED_START and type(transition) is bytes),
                 "Recovery provenance must be issued in memory")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)
        object.__setattr__(self, "_variant", variant)
        object.__setattr__(self, "_artifacts", artifacts)
        object.__setattr__(self, "_transition", transition)

    @property
    def document(self):
        return _document(self.snapshot)

    @property
    def recovery_digest(self):
        return hashlib.sha256(self.snapshot).hexdigest()

    @property
    def original_artifacts(self):
        _require(self._token is _ISSUED and self._variant == _FAILED_START,
                 "Only an issued failed-start review exposes its original immutable bytes")
        return {key: {"reference": _document(reference), "raw": raw}
                for key, reference, raw in self._artifacts}

    def authorize_keeper_command(self, manager, *, keeper, release_ref, request, original_archive,
                                 transaction_archive, attempted_host_archive, interruption_ref):
        """Bind one replacement command after exact raw archives, before its CAS.

        This returns a new frozen capability without changing its evidence or
        digest.  Only private in-memory provenance permits the manager's later
        revalidation to see the bound replacement pending/start-attempted
        command.  A saved JSON plan cannot authorize that transition.
        """
        _require(type(self) is LostFormalOwnerRecovery and self._token is _ISSUED
                 and self._variant == _FAILED_START and self._transition is None,
                 "Only a fresh failed-start capability may bind a replacement command")
        self.revalidate(manager)
        previous, state = self.document, manager.paths.state.absolute()
        issued_keeper = _identity(keeper, manager.paths.repo / ".jenkins/runtime/python/python.exe")
        _require(issued_keeper == keeper and keeper != previous["departedKeeper"]
                 and int(keeper["creationTime"]) > int(previous["owner"]["agent"]["creationTime"]),
                 "A new exact retained keeper must own the replacement request")
        release, _ = _failure_reference(state, release_ref, release=True)
        marker = release.get("failedOwnStart", {})
        reviewed, _ = _failure_reference(state, marker.get("plan"))
        _require(release.get("kind") == "formal-lost-owner-recovery-release" and release.get("schemaVersion") == 1
                 and release.get("requestId") == previous["requestId"] and release.get("scope") == previous["scope"]
                 and release.get("expectedPreviousOwner") == previous["owner"]
                 and release.get("expectedLifecycleFence") == previous["lifecycleFence"]
                 and release.get("runtimeAcceptance") is False and release.get("cargoAcceptance") is False
                 and canonical_json(reviewed) == self.snapshot
                 and all(marker.get(key) == previous[key] for key in
                         ("originalCommand", "startTransaction", "attemptedHost")),
                 "Replacement release does not pin this exact failed-start preparation")
        request_body, _ = _failure_reference(state, request)
        _require(request_body.get("schemaVersion") == 1 and request_body.get("kind") == _REQUEST_KIND
                 and request_body.get("action") == "recover" and request_body.get("requestId") == previous["requestId"]
                 and request_body.get("release") == release_ref and request_body.get("keeper") == keeper
                 and request_body.get("recoveryDigest") == self.recovery_digest
                 and request_body.get("runtimeAcceptance") is False,
                 "Replacement keeper request differs from its reviewed interruption")
        archives = {"originalCommand": original_archive, "startTransaction": transaction_archive,
                    "attemptedHost": attempted_host_archive}
        originals = self.original_artifacts
        for key, reference in archives.items():
            _, raw = _failure_reference(state, reference)
            _require(Path(reference["path"]).parent == state / "recovery-lifecycle/history"
                     and reference["sha256"] == previous[key]["sha256"] and raw == originals[key]["raw"],
                     "Replacement must archive the exact original UNKNOWN evidence bytes")
        interruption, _ = _failure_reference(state, interruption_ref)
        _require(interruption.get("schemaVersion") == 1 and interruption.get("kind") == "failed-recovery-start-interruption"
                 and interruption.get("recoveryDigest") == self.recovery_digest
                 and all(interruption.get(key) == reference for key, reference in archives.items())
                 and interruption.get("originalRequest") == previous["originalRequest"]
                 and interruption.get("operationId") == previous["operationId"]
                 and interruption.get("outcome") == "interrupted"
                 and all(interruption.get(key) is False for key in ("nativeProofIssued", "nativeTerminationProven",
                     "historicalEofClaimed", "runtimeAcceptance", "historicalHoldsReleased"))
                 and all(interruption.get(key, False) is False for key in
                         ("cargoAcceptance", "nativeProofSynthesized", "executionHoldsReleaseAllowed",
                          "validationAcceptanceAllowed", "bootBoundaryClaimed")),
                 "Failed-start interruption must preserve the unknown native outcome")
        transition = {"keeper": keeper, "release": release_ref, "request": request,
                      **archives, "interruption": interruption_ref}
        return LostFormalOwnerRecovery(self.snapshot, _ISSUED, variant=self._variant,
                                       artifacts=self._artifacts, transition=canonical_json(transition))

    def revalidate(self, manager):
        _require(type(self) is LostFormalOwnerRecovery and self._token is _ISSUED,
                 "A typed issued recovery capability is required")
        previous = self.document
        try:
            if self._variant == _FAILED_START:
                current, _ = _capture_failed(manager, previous["requestId"], previous["scope"],
                    previous["originalCommand"], _document(self._transition) if self._transition is not None else None,
                    self.recovery_digest)
            else:
                current = _capture(manager, previous["requestId"], previous["scope"])
        except JenkinsError:
            raise
        except (OSError, ValueError, TypeError, KeyError, AttributeError, sqlite3.Error, ET.ParseError) as error:
            raise JenkinsError("formal_lost_owner_unproven", "Lost-owner recovery observations are unavailable",
                               retryable=True) from error
        _require(canonical_json(current) == self.snapshot,
                 "Lost-owner recovery evidence changed before the pointer transaction",
                 "formal_lost_owner_changed")
        return {**current, "recoveryDigest": self.recovery_digest, "observedAt": time.time()}


def prepare_lost_formal_owner_recovery(manager, *, request_id, scope="execution-runtime"):
    """Capture fresh evidence under the caller's retained deployment-start lock."""
    try:
        value = _capture(manager, request_id, scope)
    except JenkinsError:
        raise
    except (OSError, ValueError, TypeError, KeyError, AttributeError, sqlite3.Error, ET.ParseError) as error:
        raise JenkinsError("formal_lost_owner_unproven", "Exact departed formal ownership could not be verified",
                           retryable=True) from error
    return LostFormalOwnerRecovery(canonical_json(value), _ISSUED)


def prepare_failed_recovery_start(manager, *, request_id, scope="execution-runtime", original_command_ref=None):
    """Review one exact failed first recovery attempt; never retry or write it."""
    try:
        value, artifacts = _capture_failed(manager, request_id, scope, original_command_ref)
    except JenkinsError:
        raise
    except (OSError, ValueError, TypeError, KeyError, AttributeError, sqlite3.Error, ET.ParseError) as error:
        raise JenkinsError("formal_lost_owner_unproven", "Exact failed recovery startup could not be verified",
                           retryable=True) from error
    sealed = tuple((key, canonical_json(reference), raw) for key, (reference, raw) in sorted(artifacts.items()))
    return LostFormalOwnerRecovery(canonical_json(value), _ISSUED, variant=_FAILED_START, artifacts=sealed)


__all__ = ["LostFormalOwnerRecovery", "prepare_lost_formal_owner_recovery", "prepare_failed_recovery_start"]

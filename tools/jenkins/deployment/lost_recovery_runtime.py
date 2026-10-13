"""Read-only, boot-bound interruption of an exact departed recovery runtime.

An unknown public Stop still requires a real later Windows boot.  An idle
committed runtime may be replaced in its exact same boot after native departure.
Both paths preserve original outcomes and never issue termination or acceptance.
"""
from __future__ import annotations

from contextlib import closing
from dataclasses import dataclass
import hashlib
from pathlib import Path
import re
import sqlite3
import time

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..processes.boot import windows_boot_filetime
from .driver import _plain
from .drain_observations import (_bound_execution, _bundle, _identity,
                                 execution_inventory, native_terminal, TERMINAL_EXECUTIONS)
from .execution_activation import protected_holds
from .lifecycle import check_port_available
from .lost_owner import (_assert_request_unused, _departure, _document, _home_policy,
                         _lifecycle_fence, _raw, _resource_inventory, _REVIEWED_CONTAINMENT)
from .startup_observations import observe_home_available, runtime_owner_inventory

KIND = "formal-lost-recovery-runtime-interruption"
VARIANT = "lost-recovery-runtime"
SAME_BOOT_VARIANT = "departed-committed-recovery-runtime"
_ISSUED = object()
_RUNTIME_WAIT_SECONDS = 60.0
_FLAGS = ("nativeProofIssued", "nativeTerminationProven", "historicalEofClaimed",
          "nativeProofSynthesized", "bootBoundaryClaimed", "runtimeAcceptance", "cargoAcceptance",
          "historicalHoldsReleased", "executionHoldsReleaseAllowed", "validationAcceptanceAllowed",
          "queueReplayRequested", "automaticRetry")
_POINTERS = {"originalCommand": "recovery-lifecycle/command.json",
             "originalDescriptor": "formal-lifecycle/recovery-keeper.json",
             "previousOperation": "deployment-operation.json", "previousHost": "deployment/host.json",
             "previousSelector": "deployment/driver.json"}


def _require(condition, message, code="lost_recovery_runtime_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


class _RuntimeOwnerBusy(JenkinsError):
    """Only a positively observed native owner permits another read-only try."""

    def __init__(self, owners):
        super().__init__("lost_recovery_runtime_unproven", "A repository runtime still may own Jenkins Home",
                         retryable=True, details={"runtimeOwnerInventory": owners})


def _clear_runtime_inventory(repository):
    owners = runtime_owner_inventory(repository)
    if owners:
        raise _RuntimeOwnerBusy(owners)
    return owners


def _ref(path, raw=None):
    path = Path(path).absolute()
    return {"path": str(path), "sha256": hashlib.sha256(_raw(path) if raw is None else raw).hexdigest()}


def _artifact(state, reference, *, mutable=(), release=False):
    from .recovery_keeper import _reference
    if isinstance(reference, dict) and Path(reference.get("path", "")).absolute() in mutable:
        path = Path(reference["path"]).absolute()
        _require(set(reference) == {"path", "sha256"} and str(path) == reference["path"]
                 and ".." not in path.parts, "Original mutable evidence must have its exact canonical identity")
        require_digest(reference["sha256"], "originalRuntimeArtifactSha256")
        raw = _raw(path)
        _require(hashlib.sha256(raw).hexdigest() == reference["sha256"], "Original runtime evidence changed")
        return _document(raw), raw
    value = _reference(state, reference, release=release)
    return value, _raw(reference["path"])


def reviewed_marker(state, release):
    """A saved plan is reviewed evidence; only the factory issues authority."""
    marker = release.get("lostRecoveryRuntime")
    if marker is None:
        return None
    _require(release.get("kind") == "formal-lost-owner-recovery-release"
             and release.get("failedOwnStart") is None and release.get("ownedUpgrade") is None
             and isinstance(marker, dict) and set(marker) == {"plan", "originalCommand", "originalDescriptor"},
             "Lost recovery runtime needs its distinct exact reviewed marker")
    planned, _ = _artifact(state, marker["plan"])
    _require(Path(marker["plan"]["path"]).parent == state / "migration-baseline"
             and planned.get("kind") == KIND and planned.get("recoveryVariant") in {VARIANT, SAME_BOOT_VARIANT}
             and planned.get("originalCommandStatus") ==
                 ("committed" if planned.get("recoveryVariant") == SAME_BOOT_VARIANT else "unknown")
             and planned.get("requestId") == release.get("requestId")
             and planned.get("scope") == release.get("scope") == "execution-runtime"
             and planned.get("owner") == release.get("expectedPreviousOwner")
             and planned.get("lifecycleFence") == release.get("expectedLifecycleFence")
             and planned.get("executionQuarantine", {}).get("protectedHolds") == release.get("protectedHolds")
             and all(planned.get(name) is False for name in _FLAGS),
             "Lost runtime release differs from its complete reviewed interruption")
    for name in ("originalCommand", "originalDescriptor"):
        reference = marker[name]
        _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}
                 and reference == planned.get(name) and reference["path"] == str(state / _POINTERS[name]),
                 "Lost runtime marker changed its original pointer identity")
        require_digest(reference["sha256"], "lostRuntimePointerSha256")
    return planned


def validate_interruption_receipt(state, descriptor, release, *, request_ref=None):
    """Retain the predecessor's exact committed result or unknown Stop."""
    planned = reviewed_marker(state, release)
    _require(planned is not None, "An interruption requires its explicit reviewed marker")
    committed = planned["recoveryVariant"] == SAME_BOOT_VARIANT
    interruption, _ = _artifact(state, descriptor.get("lostRecoveryRuntimeInterruption"))
    _require(interruption.get("schemaVersion") == 1 and interruption.get("kind") == KIND
             and interruption.get("keeper") == descriptor.get("keeper")
             and interruption.get("release") == descriptor.get("release")
             and interruption.get("reviewedPlan") == release["lostRecoveryRuntime"]["plan"]
             and interruption.get("recoveryDigest") == descriptor.get("recoveryDigest") == digest(planned)
             and interruption.get("operationId") == planned["operationId"]
             and interruption.get("originalRequestId") == planned["originalRequestId"]
             and interruption.get("bootFileTime") == planned["bootFileTime"] == descriptor.get("bootFileTime")
             and interruption.get("originalCommandStatus") == planned["originalCommandStatus"]
             and interruption.get("outcome") == "interrupted"
             and all(interruption.get(name) is False for name in _FLAGS),
             "Runtime interruption changed its original outcome or acceptance boundary")
    originals = interruption.get("originalArtifacts", {})
    expected = set(_POINTERS) | {"originalRequest", "previousRuntimeReceipt"}
    if committed:
        expected.add("originalResult")
    _require(set(originals) == expected, "Lost runtime interruption must retain every original ownership byte")
    archived = {}
    for name, binding in originals.items():
        _require(isinstance(binding, dict) and set(binding) == {"original", "archive"}
                 and binding["original"] == planned[name]
                 and binding["archive"].get("sha256") == planned[name]["sha256"],
                 "Original lost runtime archive identity changed")
        archived[name], _ = _artifact(state, binding["archive"])
    old_command = archived["originalCommand"]
    if committed:
        from .owned_upgrade import _committed_command
        _require(old_command.get("status") == "committed" and old_command.get("result") == planned["originalResult"],
                 "The original committed start must retain its real immutable result")
        _committed_command(state, archived["originalDescriptor"], archived["previousRuntimeReceipt"], old_command)
    else:
        _require(old_command.get("status") == "unknown" and "result" not in old_command
                 and old_command.get("request") == planned["originalRequest"]
                 and old_command.get("previousRuntimeReceipt") == planned["previousRuntimeReceipt"]
                 and archived["originalRequest"].get("requestId") == planned["originalRequestId"]
                 and archived["originalDescriptor"].get("runtimeReceipt") == planned["previousRuntimeReceipt"]
                 and _document(_raw(state / "recovery-lifecycle/requests" / (planned["originalRequestId"] + ".json")))
                     == planned["originalRequest"]
                 and not (state / "recovery-lifecycle/results" / (planned["originalRequestId"] + ".json")).exists(),
                 "The original Stop must remain honestly unknown with its original pending request")
    if request_ref is not None:
        _require(not committed and request_ref == planned["originalRequest"],
                 "Only the exact original unknown request may be skipped")
    return interruption


def interrupted_request(state, reference, *, historical_configuration=None):
    """An exact owned handoff preserves its predecessor's interrupted request."""
    from .recovery_keeper import _owned_handoff_receipt, _reference
    if historical_configuration is not None:
        from .owned_upgrade import _HistoricalExternalConfiguration
        _require(type(historical_configuration) is _HistoricalExternalConfiguration,
                 "A typed historical external Stop context is required")
    state = Path(state).absolute()
    current_raw = _raw(state / _POINTERS["originalDescriptor"])
    current = descriptor = _document(current_raw)
    lineage, seen, owned_current = [], set(), None
    for _ in range(64):
        identity = digest(descriptor)
        _require(identity not in seen, "Interrupted request ownership lineage contains a cycle")
        seen.add(identity)
        if descriptor.get("lostRecoveryRuntimeInterruption") is not None:
            release = _reference(state, descriptor.get("release"), release=True)
            planned = reviewed_marker(state, release)
            if planned is not None and planned.get("recoveryVariant") == SAME_BOOT_VARIANT:
                interruption = validate_interruption_receipt(state, descriptor, release)
                if reference == planned["originalRequest"]:
                    return False
                descriptor = _reference(state, interruption["originalArtifacts"]["originalDescriptor"]["archive"])
                continue
            if planned is None or reference != planned["originalRequest"]:
                return False
            validate_interruption_receipt(state, descriptor, release, request_ref=reference)
            if lineage:
                release = lineage[0]
                context = (_lineage_context(state, release) if historical_configuration is None else
                    _lineage_context(state, release, historical_configuration=historical_configuration))
                # This existing validator checks the reviewed plan, every raw
                # predecessor archive, request, retirement and real owned Stop.
                if historical_configuration is None:
                    _owned_handoff_receipt(context, owned_current, release)
                else:
                    _owned_handoff_receipt(context, owned_current, release, historical_configuration=historical_configuration)
            _require(_raw(state / _POINTERS["originalDescriptor"]) == current_raw,
                     "Current interrupted request lineage changed during observation")
            return True
        if descriptor.get("ownedUpgradeHandoff") is None:
            return False
        release = _reference(state, descriptor.get("release"), release=True)
        _require(release.get("kind") == "formal-owned-recovery-upgrade-release",
                 "Only an exact owned upgrade may inherit interruption history")
        handoff = _reference(state, descriptor["ownedUpgradeHandoff"])
        marker = release.get("ownedUpgrade", {})
        _require(isinstance(marker, dict), "Interrupted request lineage lacks its exact owned transfer marker")
        variant = marker.get("variant")
        _require(variant in {None, "external-native-stop"} and handoff.get("variant") == variant,
                 "Interrupted request lineage has an unsupported owned transfer variant")
        predecessor = "originalBlockedDescriptor" if variant == "external-native-stop" else "stoppedDescriptor"
        if owned_current is None:
            owned_current = descriptor
        lineage.append(release)
        descriptor = _reference(state, handoff.get(predecessor))
    raise JenkinsError("lost_recovery_runtime_unproven", "Interrupted request ownership lineage exceeds its bound")


def _lineage_context(state, release, *, historical_configuration=None):
    """Read the pinned deployment inputs required by the existing Stop validator."""
    from types import SimpleNamespace
    repo = state.parent.parent
    spec_path = repo / ".jenkins/deployment-spec.json"
    _require(state == repo / ".jenkins/state" and release.get("deploymentSpec", {}).get("path") == str(spec_path),
             "Owned interruption history requires its exact formal deployment inputs")
    if historical_configuration is None:
        spec, _ = _artifact(state, release["deploymentSpec"], mutable=(spec_path,))
    else:
        from .owned_upgrade import _HistoricalExternalConfiguration, _reference
        _require(type(historical_configuration) is _HistoricalExternalConfiguration,
                 "A typed historical external Stop context is required")
        inputs = SimpleNamespace(paths=SimpleNamespace(repo=repo, state=state, home=repo / ".jenkins/jenkins_home"),
                                 spec=SimpleNamespace(path=spec_path))
        matches = [entry["release"] for entry in historical_configuration.document["releases"]
                   if _reference(state, entry["release"], release=True)[0] == release]
        _require(len(matches) == 1, "Historical interruption release is outside its exact Stop context")
        spec = historical_configuration.validate(inputs, matches[0])
    controller = spec.get("controller", {})
    _require(isinstance(controller, dict) and isinstance(controller.get("java"), dict)
             and isinstance(controller["java"].get("executable"), str) and isinstance(controller.get("warPath"), str),
             "Owned interruption history lacks its pinned runtime inputs")
    return SimpleNamespace(paths=SimpleNamespace(repo=repo, state=state, home=repo / ".jenkins/jenkins_home"),
                           spec=SimpleNamespace(path=spec_path, controller=controller),
                           java=Path(controller["java"]["executable"]), war=Path(controller["warPath"]))


def _historical_runtime(manager, descriptor, release, target):
    """Verify the original strict issuance without asking a dead owner to live."""
    from .recovery_keeper import CREATION_FLAGS, _owned_handoff_receipt, _restriction
    state, keeper, runtime = manager.paths.state.absolute(), descriptor["keeper"], descriptor["runtime"]
    body, _ = _artifact(state, descriptor["runtimeReceipt"])
    _require(body.get("schemaVersion") == 1 and body.get("kind") == "recovery-keeper-runtime"
             and body.get("keeper") == keeper and body.get("release") == descriptor["release"]
             and body.get("runtime") == runtime and body.get("bootFileTime") == descriptor.get("bootFileTime")
             and body.get("protectedHolds") == release.get("protectedHolds") == descriptor.get("protectedHolds")
             and all(body.get(name) is False for name in
                     ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased", "nativeProofSynthesized",
                      "queueReplayRequested"))
             and all(body.get(name) == descriptor.get(name) for name in
                     ("initialRecovery", "recoveryDigest", "failedOwnStartInterruption", "initialOwnedUpgrade",
                      "ownedUpgradeDigest", "ownedUpgradeHandoff", "previousStoppedRuntimeReceipt")),
             "Departed descriptor lacks its exact immutable original runtime issuance")
    spawn = body.get("strictSpawn", {})
    _require(spawn.get("creator") == keeper and spawn.get("child") == runtime["owner"]["supervisor"]
             and spawn.get("creationFlags") == CREATION_FLAGS and spawn.get("requestedProfile") == "zircon-strict"
             and spawn.get("creatorInJob") is True and spawn.get("childInJob") is True
             and all(runtime["selected"].get(key) == value for key, value in target.items()),
             "Departed runtime has no exact retained strict native creation proof")
    _restriction(spawn.get("actualRestrictionPreflight", {}), keeper, release["config"])
    observed = _restriction(spawn.get("associatedJobAtSpawn", {}), keeper, release["config"])
    _require(spawn["child"]["pid"] in observed["associatedJob"]["memberPids"],
             "Departed supervisor was not created in its original restricted outer Job")
    if release.get("kind") == "formal-owned-recovery-upgrade-release":
        _owned_handoff_receipt(manager, descriptor, release)
    else:
        _require(release.get("kind") == "formal-lost-owner-recovery-release"
                 and release.get("lostRecoveryRuntime") is not None,
                 "This interruption route requires an exact reviewed recovery successor")
        validate_interruption_receipt(state, descriptor, release)
    return body


def _execution_gate(manager, owner, selected, holds, boot, *, same_boot=False):
    """Require reconciled launch metadata and reuse every recorded native proof."""
    rows = execution_inventory(manager.paths.state)
    runtime = owner["operation"]["operationId"]
    current = [row for row in rows if row["domain"] == "execution"
               and row["payload"].get("operationId") == runtime]
    observations = []
    for row in current:
        key, ep = row["key"], row["payload"]
        bundle = _bundle(rows, key)
        actual_runtime, checksum, launcher = _bound_execution(bundle)
        lp, hp = (bundle[name]["payload"] for name in ("launch", "host"))
        native = {item["key"]: item for item in bundle["nativeJobs"]}
        guard_id = lp.get("guardNativeJobId")
        guard = native.get(guard_id)
        phases = ep.get("phaseRecords")
        _require(actual_runtime == runtime and checksum == selected["driverDigest"]
                 and lp.get("brokerPid") == owner["supervisor"]["pid"]
                 and ep.get("status") in TERMINAL_EXECUTIONS and ep.get("refs") == 0 and ep.get("consumers") == []
                 and lp.get("status") == "exited" and hp.get("status") == "terminal"
                 and isinstance(phases, list) and bool(phases) and phases == hp.get("phaseRecords")
                 and guard is not None and guard["payload"].get("role") == "execution_host_guard"
                 and guard["payload"].get("owner") == key
                 and guard["payload"].get("completeProof", {}).get("identity") == launcher
                 and all(native_terminal(item) and item["payload"].get("terminalProof") == item["payload"].get("completeProof")
                         for item in native.values()),
                 "Current execution must retain every complete original native proof and no consumers",
                 "lost_recovery_runtime_execution_active")
        phase_ids = [phase.get("nativeJobId") for phase in phases]
        _require(len(phase_ids) == len(set(phase_ids)) and set(native) == set(phase_ids) | {guard_id},
                 "Current execution native job inventory is incomplete")
        for phase in phases:
            job = native[phase["nativeJobId"]]["payload"]
            _require(job.get("role") == "phase" and job.get("owner") == key
                     and job.get("guardNativeJobId") == guard_id and phase.get("terminalProof") == job["completeProof"],
                     "Current execution phase proof differs from its native registry")
        _require(ep.get("terminalProof") == hp.get("terminalProof") == phases[-1]["terminalProof"],
                 "Execution and host final phase proofs conflict")
        identities = [launcher, *(item["payload"]["completeProof"]["identity"] for item in native.values())]
        if not same_boot:
            _require(all(int(identity["creationTime"]) < int(boot) for identity in identities),
                     "Current native proof identities must precede the actual new boot")
        departures = [_departure(identity) for identity in identities]
        for related in rows:
            payload = related["payload"]
            if payload.get("owner") == key and related["domain"] in {"resource_reservation", "pool_writer_hold"}:
                _require(payload.get("status") in {"released", "cancelled"}, "Current execution still owns a resource hold")
            if payload.get("owner") == key and related["domain"] == "resource_queue":
                reservation = next((item for item in rows if item["domain"] == "resource_reservation"
                                    and item["key"] == related["key"] and item["payload"].get("owner") == key), None)
                _require(payload.get("status") in {"cancelled", "released"}
                         or payload.get("status") == "admitted" and reservation is not None
                            and reservation["payload"].get("status") == "released",
                         "Current queue admission must retain its exact already-released reservation")
            if related["domain"] == "execution_consumer" and payload.get("executionId") == key:
                _require(payload.get("status") == "released", "Current execution has an unresolved consumer")
        observations.append({"executionId": key, "recordsDigest": digest(bundle), "launchStatus": lp["status"],
                             "launchMetadataReconciled": lp["status"] == "exited", "departures": departures,
                             "nativeProofsReused": True, "artifactAcceptance": False})
    execution_keys = {row["key"] for row in rows if row["domain"] == "execution"}
    current_keys = {row["key"] for row in current}
    for row in rows:
        payload = row["payload"]
        if row["domain"] in {"execution_launch", "execution_host"}:
            _require(row["key"] in execution_keys, "Orphaned execution ownership cannot authorize recovery")
        if row["domain"] == "native_job":
            _require(payload.get("executionId") in execution_keys or native_terminal(row),
                     "Unresolved orphaned native ownership cannot authorize recovery")
        if row["domain"] == "jenkins_submission":
            bindings = (payload, payload.get("driver"), payload.get("parameters"))
            if any(isinstance(value, dict) and any(value.get(name) == runtime for name in
                   ("operationId", "runtimeOperationId", "brokerGeneration")) for value in bindings):
                _require(payload.get("delivery") in {"submitted", "completed", "accepted"},
                         "Current dispatch outcome still requires separate reconciliation")
    _require(len(holds) == 4 and protected_holds(manager.paths.state) == holds
             and all(row["payload"].get("owner") in execution_keys - current_keys for row in holds),
             "All four historical holds must retain their original execution ownership")
    return {"recordsDigest": digest(rows), "records": rows, "currentExecutions": observations,
            "protectedHolds": holds, "protectedHoldsDigest": digest(holds),
            "resourceInventoryDigest": digest(_resource_inventory(manager.paths.state)),
            "nativeProofIssued": False, "executionHoldsReleaseAllowed": False, "artifactAcceptance": False}


def _same_boot_containment(repo, target):
    """Review the actual sealed kill-on-close owner; absence proves no EOF."""
    expected = {**_REVIEWED_CONTAINMENT,
        "tools/jenkins/deployment/host.py": "99c1a2e1f7ae5c707fd37b47b0fce85509799c9aea8b69ceaaff33a746fd5821",
        "tools/jenkins/processes/execution_host.py": "194d430231b04f2f207878b80aa7ce89885f0b7a97cb1719e2f627c4dd025833"}
    root = Path(target["root"])
    _require(root == repo / ".jenkins/runtime/drivers" / target["driverDigest"],
             "Same-boot containment must retain the exact verified sealed runtime")
    manifest = _document(_raw(target["manifest"]))
    for name, checksum in expected.items():
        _require(manifest.get("files", {}).get(name, {}).get("sha256") == checksum
                 and hashlib.sha256(_raw(root / name)).hexdigest() == checksum,
                 "Same-boot recovery requires the reviewed native containment implementation")
    return {"driverDigest": target["driverDigest"], "sourceHashes": expected,
            "sourceReviewDigest": digest(expected), "nativeTerminationClaimed": False}


def _management_rows(state):
    """Read management state without changing unknown outcomes or ownership."""
    path = state / "coordination.sqlite3"
    _plain(path)
    with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as connection:
        connection.execute("PRAGMA query_only=ON")
        connection.execute("BEGIN")
        return [{"domain": domain, "key": key, "version": version, "payload": _document(payload)}
                for domain, key, version, payload in connection.execute(
                    "SELECT domain,key,version,payload FROM records WHERE domain GLOB 'management_*' ORDER BY domain,key")]


def _management_quarantine(manager, owner, selected, native_rows):
    rows = _management_rows(manager.paths.state)
    runtime = owner["operation"]["operationId"]
    def current(payload):
        bindings = (payload, payload.get("payload"), payload.get("result"), payload.get("driver"), payload.get("parameters"))
        matching = [value for value in bindings if isinstance(value, dict)
                    and any(value.get(field) == runtime for field in ("operationId", "runtimeOperationId", "brokerGeneration"))]
        _require(all(value.get("driverDigest", selected["driverDigest"]) == selected["driverDigest"] for value in matching),
                 "Management state has a conflicting current runtime driver")
        return bool(matching)
    operations = [row for row in rows if row["domain"] == "management_operation" and current(row["payload"])]
    operation_ids = {row["payload"].get(field) for row in operations for field in ("requestId", "operationId")}
    operation_ids.discard(None)
    keys = {row["key"] for row in operations}
    related = [row for row in rows if current(row["payload"]) or row["key"] in keys
               or row["payload"].get("operationId") in operation_ids or row["payload"].get("requestId") in operation_ids]
    departures = []
    for row in native_rows:
        payload = row["payload"]
        if row["domain"] != "native_job" or not (current(payload)
                or payload.get("operationId") in operation_ids or payload.get("requestId") in operation_ids
                or payload.get("owner") in {"management-" + key for key in keys}):
            continue
        _require(native_terminal(row) and payload.get("terminalProof") == payload.get("completeProof"),
                 "Current management native work requires its existing original terminal proof")
        identity = _identity(payload["completeProof"]["identity"])
        departures.append({"nativeJobId": row["key"], "identity": identity, "departure": _departure(identity),
                           "nativeProofReused": True, "artifactAcceptance": False})
    return {"recordsDigest": digest(rows),
            "records": [{"domain": row["domain"], "key": row["key"], "version": row["version"],
                         "payloadDigest": digest(row["payload"])} for row in rows],
            "currentRecords": [{"domain": row["domain"], "key": row["key"]} for row in related],
            "nativeDepartures": departures, "nativeProofIssued": False, "artifactAcceptance": False,
            "executionHoldsReleaseAllowed": False}


def _management_comparison(value):
    comparison = _document(canonical_json(value))
    for binding in comparison["nativeDepartures"]:
        binding["departure"] = _departure_binding(binding["departure"])
    return comparison


def _departure_binding(observation):
    """Compare the old identity after its fresh native departure was verified."""
    from .owned_upgrade import _departure_record
    _require(isinstance(observation, dict) and type(observation.get("pid")) is int
             and observation["pid"] > 0 and isinstance(observation.get("expectedCreationTime"), str)
             and observation["expectedCreationTime"].isascii() and observation["expectedCreationTime"].isdecimal(),
             "Departure observation lacks its exact original identity")
    identity = {"pid": observation["pid"], "creationTime": observation["expectedCreationTime"]}
    _departure_record(observation, identity)
    _require(set(observation) == {"pid", "expectedCreationTime", "status"}
             | ({"observedIdentity"} if observation["status"] == "pid-reused" else set()),
             "Departure observation has conflicting evidence")
    return {"pid": identity["pid"], "expectedCreationTime": identity["creationTime"]}


def _execution_comparison(value):
    comparison = _document(canonical_json(value))
    for execution in comparison["currentExecutions"]:
        execution["departures"] = [_departure_binding(item) for item in execution["departures"]]
    return comparison


def _comparison(value):
    comparison = _document(canonical_json(value))
    comparison["departedIdentities"] = {key: _departure_binding(item)
                                       for key, item in comparison["departedIdentities"].items()}
    comparison["executionQuarantine"] = _execution_comparison(comparison["executionQuarantine"])
    if "managementQuarantine" in comparison:
        comparison["managementQuarantine"] = _management_comparison(comparison["managementQuarantine"])
    return comparison


def _departure_observations(value):
    observations = {"/departedIdentities/" + key: item for key, item in value["departedIdentities"].items()}
    for index, execution in enumerate(value["executionQuarantine"]["currentExecutions"]):
        for position, observation in enumerate(execution["departures"]):
            observations[f"/executionQuarantine/currentExecutions/{index}/departures/{position}"] = observation
    for index, binding in enumerate(value.get("managementQuarantine", {}).get("nativeDepartures", [])):
        observations[f"/managementQuarantine/nativeDepartures/{index}/departure"] = binding["departure"]
    return observations


def _changed_paths(before, after, path=""):
    if type(before) is not type(after):
        return [path or "/"]
    if isinstance(before, dict):
        changes = []
        for key in sorted(set(before) | set(after)):
            child = path + "/" + key.replace("~", "~0").replace("/", "~1")
            changes.extend([child] if key not in before or key not in after
                           else _changed_paths(before[key], after[key], child))
        return changes
    if isinstance(before, list):
        if len(before) != len(after):
            return [path or "/"]
        return [child for index, (left, right) in enumerate(zip(before, after))
                for child in _changed_paths(left, right, path + "/" + str(index))]
    return [] if before == after else [path or "/"]


def _replacement(state, transition, recovery_digest):
    command = _document(_raw(state / _POINTERS["originalCommand"]))
    _require(command.get("kind") == "formal-lost-owner-recovery-command"
             and command.get("status") in {"pending", "start-attempted"}
             and all(command.get(name) == transition[name] for name in ("keeper", "release", "request"))
             and command.get("recoveryDigest") == recovery_digest
             and command.get("replacesOriginalCommand") == transition["archives"]["originalCommand"]
             and command.get("replacesLostRecoveryDescriptor") == transition["archives"]["originalDescriptor"]
             and command.get("lostRecoveryRuntimeInterruption") == transition["interruption"]
             and command.get("previousRuntimeReceipt") is None and command.get("runtimeAcceptance") is False
             and command.get("historicalHoldsReleased") is False and "result" not in command,
             "Only this issued replacement command may supersede the exact original command")
    for reference in transition["archives"].values():
        _artifact(state, reference)
    _artifact(state, transition["interruption"])


def _capture_once(manager, request_id, scope, command_ref=None, descriptor_ref=None, transition=None, recovery_digest=None):
    from .owned_upgrade import _historical_release
    repo, state, home = (manager.paths.repo.absolute(), manager.paths.state.absolute(), manager.paths.home.absolute())
    _require(state == repo / ".jenkins/state" and home == repo / ".jenkins/jenkins_home"
             and re.fullmatch("[0-9a-f]{32}", str(request_id)) and scope == "execution-runtime",
             "A unique bounded request for the existing formal deployment is required")
    for path in (repo, state, home):
        _plain(path)
    _assert_request_unused(state, request_id)
    for area in ("requests", "results"):
        _require(not (state / "recovery-lifecycle" / area / (request_id + ".json")).exists(),
                 "A recovery request cannot be replayed")
    artifacts = {}
    for name, relative in _POINTERS.items():
        path = state / relative
        if name == "originalCommand" and transition is not None:
            _replacement(state, transition, recovery_digest)
            document, raw = _artifact(state, transition["archives"][name])
            reference = _ref(path, raw)
        else:
            raw = _raw(path)
            document, reference = _document(raw), _ref(path, raw)
        artifacts[name] = (reference, raw)
    descriptor, command, operation, host, selected = (_document(artifacts[name][1]) for name in
        ("originalDescriptor", "originalCommand", "previousOperation", "previousHost", "previousSelector"))
    _require(command_ref in (None, artifacts["originalCommand"][0])
             and descriptor_ref in (None, artifacts["originalDescriptor"][0]), "Explicit original recovery pointers changed")
    committed = command.get("status") == "committed"
    _require(descriptor.get("schemaVersion") == 1 and descriptor.get("kind") == "formal-lost-owner-recovery-keeper-descriptor"
             and descriptor.get("admission") == ("idle" if committed else "blocked") and descriptor.get("runtime", {}).get("state") == "running"
             and all(descriptor.get(name) is False for name in ("automaticRestart", "runtimeAcceptance", "cargoAcceptance",
                                                               "historicalHoldsReleased")),
             "Only the exact committed idle or unknown blocked running descriptor can be interrupted")
    runtime = descriptor["runtime"]
    runtime_id = operation.get("operationId")
    _require(runtime.get("operation") == operation and runtime.get("host") == host and runtime.get("selected") == selected
             and operation.get("state") == "running" and host.get("status") == "running"
             and re.fullmatch("start-[1-9][0-9]*", str(runtime_id))
             and operation.get("generation") == host.get("generation") == host.get("operationId") == runtime_id
             and selected.get("runtimeOperationId") == runtime_id and selected.get("recoveryRelease") == descriptor.get("release")
             and operation.get("component") == "lifecycle-host" and operation.get("homePath") == str(home)
             and operation.get("pid") == operation.get("hostPid") == host.get("hostPid")
             and operation.get("controlPlaneOnly") is False and host.get("controlPlaneOnly") is False
             and host.get("agentLaunchAttempted") is True
             and all(not {"nativeTerminationProof", "agentTerminationProof", "stopBinding", "stopResult"} & value.keys()
                     for value in (operation, host)), "Exact original running ownership metadata must remain unchanged")
    identities = {"keeper": _identity(descriptor["keeper"], repo / ".jenkins/runtime/python/python.exe"),
                  "supervisor": _identity(operation, repo / ".jenkins/runtime/python/python.exe"),
                  "controller": _identity(host.get("controllerIdentity"), manager.java),
                  "agent": _identity(host.get("agentIdentity"), manager.java)}
    _require(identities["keeper"] == descriptor["keeper"] and host.get("identity") == identities["controller"]
             and runtime.get("owner") == {key: identities[key] for key in ("supervisor", "controller", "agent")}
             and runtime.get("nativeIdentities") == [identities[key] for key in ("supervisor", "controller", "agent")]
             and len({value["pid"] for value in identities.values()}) == 4, "Original recovery native identities conflict")
    old_release, old_target = _historical_release(manager, descriptor["release"])
    keeper_args = [str(repo / ".jenkins/runtime/python/python.exe"), "-B",
                   str(repo / "tools/dev/jenkins_recovery_keeper.py"), "--config-sha256", old_release["config"]["sha256"],
                   "--release-file", descriptor["release"]["path"], "--release-sha256", descriptor["release"]["sha256"]]
    supervisor_args = [str(repo / ".jenkins/runtime/python/python.exe"), old_target["launcher"], "__lifecycle-host",
                       "--spec", str(manager.spec.path.absolute()), "--java", str(manager.java), "--war", str(manager.war),
                       "--operation-id", runtime_id]
    command_digest = lambda args: hashlib.sha256("\0".join(args).encode("utf-8")).hexdigest()
    _require(identities["keeper"]["commandDigest"] in {command_digest(keeper_args), command_digest([*keeper_args, "--no-tray"])}
             and identities["supervisor"]["commandDigest"] == command_digest(supervisor_args),
             "Original keeper and supervisor commands must bind their exact reviewed release and sealed target")
    components = old_release.get("runtimeFiles", {})
    required = {str(repo / ".jenkins/runtime/python" / name) for name in
                ("python.exe", "pythonw.exe", "python3.dll", "python314.dll", "vcruntime140.dll", "vcruntime140_1.dll")}
    required |= {str(manager.java.absolute()), str(manager.war.absolute())}
    required |= {str(repo / ".jenkins/runtime/jdk" / name) for name in ("bin/server/jvm.dll", "lib/modules", "release")}
    _require(isinstance(components, dict) and required <= set(components), "Original reviewed runtime closure is incomplete")
    for name, checksum in components.items():
        path = Path(name).absolute()
        _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo / ".jenkins/runtime")
                 and _ref(path)["sha256"] == checksum, "Original pinned runtime component changed")
    receipt = _historical_runtime(manager, descriptor, old_release, old_target)
    _require(all(selected.get(key) == value for key, value in old_target.items())
             and old_release.get("actorCodeRoot") == old_target["root"]
             and selected.get("supportRegression") == old_release.get("regression")
             and selected.get("preparation") == old_release.get("preparation", {}).get("path"),
             "Original selector differs from its sealed passing owned successor")
    if committed:
        from .owned_upgrade import _committed_command, _external_delivery_inventory
        _committed_command(state, descriptor, receipt, command)
        request, request_raw = _artifact(state, command["request"])
        stop_id = request["requestId"]
        _require(stop_id != request_id, "A committed request cannot be replayed as recovery")
        result, result_raw = _artifact(state, command["result"])
        artifacts["originalResult"] = (command["result"], result_raw)
        deliveries = _external_delivery_inventory(state)
    else:
        _require(command.get("schemaVersion") == 1 and command.get("kind") == "formal-lost-owner-recovery-command"
                 and command.get("status") == "unknown" and command.get("reasonCode") == "drain_unproven"
                 and command.get("keeper") == identities["keeper"] and command.get("release") == descriptor["release"]
                 and command.get("previousRuntimeReceipt") == descriptor["runtimeReceipt"]
                 and command.get("recoveryDigest") == descriptor.get("recoveryDigest")
                 and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False
                 and "result" not in command, "Only this unresolved public Stop before native drain may be reviewed")
        request, request_raw = _artifact(state, command.get("request"))
        stop_id = request.get("requestId")
        idle_descriptor = {**descriptor, "admission": "idle"}
        _require(request.get("schemaVersion") == 1 and request.get("kind") == "formal-lost-owner-recovery-request"
                 and request.get("action") == "stop" and re.fullmatch("[0-9a-f]{32}", str(stop_id)) and stop_id != request_id
                 and request.get("keeper") == identities["keeper"] and request.get("release") == descriptor["release"]
                 and request.get("runtimeReceipt") == descriptor["runtimeReceipt"]
                 and request.get("protectedHoldsDigest") == digest(descriptor["protectedHolds"])
                 and request.get("descriptorSha256") == hashlib.sha256(canonical_json(idle_descriptor) + b"\n").hexdigest()
                 and request.get("runtimeAcceptance") is False
                 and _document(_raw(state / "recovery-lifecycle/requests" / (stop_id + ".json"))) == command["request"]
                 and not (state / "recovery-lifecycle/results" / (stop_id + ".json")).exists(),
                 "Unknown Stop must retain its exact original request, preceding descriptor and pending result")
        for marker in sorted((state / "recovery-lifecycle/requests").glob("*.json")):
            if not (state / "recovery-lifecycle/results" / marker.name).exists():
                _require(marker.stem == stop_id and _document(_raw(marker)) == command["request"],
                         "Another unresolved request must retain its separate recovery authority")

    artifacts["originalRequest"] = (command["request"], request_raw)
    artifacts["previousRuntimeReceipt"] = (descriptor["runtimeReceipt"], _raw(descriptor["runtimeReceipt"]["path"]))
    commands = []
    for path in sorted((state / "deployment/commands").glob("stop-*.json")):
        value = _document(_raw(path))
        _require(value.get("operationId") != runtime_id, "A native Stop attempt requires its separate exact reconciliation")
        commands.append(_ref(path))
    fence = _lifecycle_fence(manager)
    for name in ("deployment/execution-activation.json", "deployment/execution-upgrade-selection.json",
                 "deployment/execution-upgrade-start.json", "formal-lifecycle/keeper.json",
                 "deployment/formal-lifecycle-command.json"):
        _require(fence[name] == old_release["expectedLifecycleFence"][name], "Original formal anchors changed")
    boot = str(windows_boot_filetime())
    if committed:
        _require(boot == descriptor["bootFileTime"]
                 and all(int(value["creationTime"]) >= int(boot) for value in identities.values()),
                 "A committed idle recovery owner must remain in its exact original Windows boot")
        containment = _same_boot_containment(repo, old_target)
    else:
        _require(boot.isascii() and boot.isdecimal() and int(boot) > int(descriptor["bootFileTime"])
                 and all(int(value["creationTime"]) < int(boot) for value in identities.values()),
                 "Only a genuine newer Windows boot beyond every original native birth permits interruption")
    departed = {key: _departure(value) for key, value in identities.items()}
    inventory = _clear_runtime_inventory(repo)
    owner = {"operation": {key: operation[key] for key in
             ("operationId", "generation", "hostPid", "homePath", "component", "controlPlaneOnly")},
             **{key: identities[key] for key in ("supervisor", "controller", "agent")}}
    execution = _execution_gate(manager, owner, selected, descriptor["protectedHolds"], boot, same_boot=committed)
    management = _management_quarantine(manager, owner, selected, execution["records"]) if committed else None
    policy, observation = _home_policy(home, scope), observe_home_available(home)
    endpoint = {"address": manager.spec.controller["listenAddress"], "port": int(manager.spec.controller["httpPort"])}
    check_port_available(endpoint["address"], endpoint["port"])
    _require(str(windows_boot_filetime()) == boot and _lifecycle_fence(manager) == fence,
             "Boot, execution quarantine, Home or lifecycle changed during review")
    _clear_runtime_inventory(repo)
    latest_execution = _execution_gate(manager, owner, selected, descriptor["protectedHolds"], boot, same_boot=committed)
    _require(canonical_json(_execution_comparison(latest_execution)) == canonical_json(_execution_comparison(execution))
             and _home_policy(home, scope) == policy, "Boot, execution quarantine, Home or lifecycle changed during review")
    execution = latest_execution
    if committed:
        latest_management = _management_quarantine(manager, owner, selected, execution["records"])
        _require(_management_comparison(latest_management) == _management_comparison(management)
                 and _external_delivery_inventory(state) == deliveries,
                 "Management quarantine or original recovery deliveries changed during review")
        management = latest_management
        departed = {key: _departure(value) for key, value in identities.items()}
        _require(str(windows_boot_filetime()) == boot, "Same-boot native ownership changed before interruption")
        _clear_runtime_inventory(repo)
    for name, (reference, raw) in artifacts.items():
        if name == "originalCommand" and transition is not None:
            _replacement(state, transition, recovery_digest)
        else:
            _require(_raw(reference["path"]) == raw, "Original recovery evidence changed during observation")
    return {"schemaVersion": 1, "kind": KIND,
            "recoveryVariant": SAME_BOOT_VARIANT if committed else VARIANT, "requestId": request_id,
            "scope": scope, "operationId": runtime_id, "homePath": str(home), "owner": owner,
            "departedKeeper": identities["keeper"], "originalRequestId": stop_id,
            "previousRecoveryRelease": descriptor["release"], "previousBootFileTime": descriptor["bootFileTime"],
            "bootFileTime": boot, "departedIdentities": departed, "runtimeOwnerInventory": inventory,
            "operationDigest": digest(operation), "hostRecordDigest": digest(host),
            "selectedDriverDigest": digest(selected), "lifecycleFence": fence,
            "originalCommandStatus": "committed" if committed else "unknown", "outcome": "interrupted", "controlPlaneOnly": False,
            "originalNativeStopIssued": False, "nativeStopInventory": commands,
            "executionQuarantine": execution, "homePolicy": policy, "homeObservation": observation,
            "endpointAvailable": endpoint, **{name: reference for name, (reference, _) in artifacts.items()},
            **({"containmentReview": containment, "managementQuarantine": management,
                "recoveryDeliveryInventory": deliveries} if committed else {}),
            **{name: False for name in _FLAGS}}, artifacts


def _capture(manager, request_id, scope, command_ref=None, descriptor_ref=None, transition=None, recovery_digest=None):
    """Wait for transient users without replaying a command or excluding owners.

    Every retry starts the complete original review again. Unknown/inaccessible
    identities, changed evidence and occupied Home files never enter this wait.
    """
    deadline = time.monotonic() + _RUNTIME_WAIT_SECONDS
    boot, fence = str(windows_boot_filetime()), _lifecycle_fence(manager)
    while True:
        _require(str(windows_boot_filetime()) == boot and _lifecycle_fence(manager) == fence,
                 "Boot or lifecycle changed while waiting for repository runtime departure")
        try:
            return _capture_once(manager, request_id, scope, command_ref, descriptor_ref, transition, recovery_digest)
        except _RuntimeOwnerBusy:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise
            time.sleep(min(.25, remaining))


@dataclass(frozen=True, init=False)
class LostRecoveryRuntimeInterruption:
    snapshot: bytes
    _token: object
    _artifacts: tuple
    _transition: bytes | None

    def __init__(self, snapshot, token=None, *, artifacts=(), transition=None):
        _require(token is _ISSUED and type(snapshot) is bytes and type(artifacts) is tuple
                 and (transition is None or type(transition) is bytes), "Only the read-only factory issues interruption authority")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)
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
        _require(self._token is _ISSUED, "Issued exact original evidence is required")
        return {name: {"reference": _document(reference), "raw": raw} for name, reference, raw in self._artifacts}

    def revalidate(self, manager):
        previous = self.document
        value, _ = _capture(manager, previous["requestId"], previous["scope"], previous["originalCommand"],
                            previous["originalDescriptor"], _document(self._transition) if self._transition else None,
                            self.recovery_digest)
        previous_comparison, comparison = _comparison(previous), _comparison(value)
        if canonical_json(comparison) != canonical_json(previous_comparison):
            changes = _changed_paths(previous_comparison, comparison)
            raise JenkinsError("lost_recovery_runtime_changed", "Original interruption evidence changed", retryable=True,
                               details={"reviewedPlanDigest": self.recovery_digest, "currentObservationDigest": digest(value),
                                        "changedPaths": changes[:64], "changedPathsTruncated": len(changes) > 64})
        before, after = _departure_observations(previous), _departure_observations(value)
        refreshes = [{"path": path, "before": before[path], "after": after[path]} for path in sorted(before)
                     if canonical_json(before[path]) != canonical_json(after[path])]
        return {**value, "recoveryDigest": self.recovery_digest, "observedAt": time.time(),
                "revalidation": {"reviewedPlanDigest": self.recovery_digest, "currentObservationDigest": digest(value),
                                 "departureRefreshes": refreshes}}

    def authorize_keeper_command(self, manager, *, keeper, release_ref, request, archives, interruption_ref):
        _require(type(self) is LostRecoveryRuntimeInterruption and self._token is _ISSUED and self._transition is None,
                 "Only one fresh issued capability may authorize a replacement command")
        self.revalidate(manager)
        state, previous = manager.paths.state.absolute(), self.document
        issued_keeper = _identity(keeper, manager.paths.repo / ".jenkins/runtime/python/python.exe")
        _require(issued_keeper == keeper and int(keeper["creationTime"]) >= int(previous["bootFileTime"]),
                 "The replacement must retain its exact current-boot strict keeper")
        release, _ = _artifact(state, release_ref, release=True)
        _require(reviewed_marker(state, release) == previous, "Replacement release must pin the exact reviewed interruption")
        request_body, _ = _artifact(state, request)
        _require(request_body.get("kind") == "formal-lost-owner-recovery-request" and request_body.get("action") == "recover"
                 and request_body.get("requestId") == previous["requestId"] and request_body.get("keeper") == keeper
                 and request_body.get("release") == release_ref and request_body.get("recoveryDigest") == self.recovery_digest
                 and request_body.get("runtimeAcceptance") is False, "Replacement request differs from this issued interruption")
        originals = self.original_artifacts
        _require(set(archives) == set(originals), "Every original recovery byte must be archived before replacement")
        for name, reference in archives.items():
            _, raw = _artifact(state, reference)
            _require(Path(reference["path"]).parent == state / "recovery-lifecycle/history"
                     and reference["sha256"] == originals[name]["reference"]["sha256"] and raw == originals[name]["raw"],
                     "Replacement archive must preserve every original byte")
        interruption, _ = _artifact(state, interruption_ref)
        _require(interruption.get("kind") == KIND and interruption.get("outcome") == "interrupted"
                 and interruption.get("keeper") == keeper and interruption.get("release") == release_ref
                 and interruption.get("request") == request and interruption.get("recoveryDigest") == self.recovery_digest
                 and interruption.get("operationId") == previous["operationId"]
                 and interruption.get("originalCommandStatus") == previous["originalCommandStatus"]
                 and interruption.get("originalArtifacts") == {name: {"original": originals[name]["reference"], "archive": ref}
                                                                for name, ref in archives.items()}
                 and all(interruption.get(name) is False for name in _FLAGS),
                 "Interruption receipt cannot claim native EOF, old Stop completion or acceptance")
        transition = {"keeper": keeper, "release": release_ref, "request": request,
                      "archives": archives, "interruption": interruption_ref}
        return LostRecoveryRuntimeInterruption(self.snapshot, _ISSUED, artifacts=self._artifacts,
                                               transition=canonical_json(transition))


def prepare_lost_recovery_runtime(manager, *, request_id, scope="execution-runtime",
                                  original_command_ref=None, original_descriptor_ref=None, reviewed_plan_ref=None):
    """Verify fresh ownership while retaining an authenticated reviewed snapshot."""
    try:
        document, artifacts = _capture(manager, request_id, scope, original_command_ref, original_descriptor_ref)
        if reviewed_plan_ref is not None:
            state = manager.paths.state.absolute()
            reviewed, reviewed_raw = _artifact(state, reviewed_plan_ref)
            _require(Path(reviewed_plan_ref["path"]).parent == state / "migration-baseline"
                     and hashlib.sha256(reviewed_raw).hexdigest() == reviewed_plan_ref["sha256"]
                     and _document(reviewed_raw) == reviewed,
                     "Reviewed interruption must retain its exact immutable plan")
            previous, current = _comparison(reviewed), _comparison(document)
            if canonical_json(previous) != canonical_json(current):
                changes = _changed_paths(previous, current)
                raise JenkinsError("lost_recovery_runtime_changed", "Reviewed interruption evidence changed", retryable=True,
                                   details={"reviewedPlanDigest": digest(reviewed), "currentObservationDigest": digest(document),
                                            "changedPaths": changes[:64], "changedPathsTruncated": len(changes) > 64})
            # Only verified departure metadata may refresh. Keep the reviewed
            # digest and issue authority with the freshly observed original bytes.
            document = reviewed
    except JenkinsError:
        raise
    except (OSError, ValueError, TypeError, KeyError, AttributeError, sqlite3.Error) as error:
        raise JenkinsError("lost_recovery_runtime_unproven", "Exact departed recovery runtime could not be verified",
                           retryable=True) from error
    sealed = tuple((name, canonical_json(reference), raw) for name, (reference, raw) in sorted(artifacts.items()))
    return LostRecoveryRuntimeInterruption(canonical_json(document), _ISSUED, artifacts=sealed)


__all__ = ["LostRecoveryRuntimeInterruption", "prepare_lost_recovery_runtime"]

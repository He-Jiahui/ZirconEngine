"""Reconcile one genuinely spawned, native-terminal owned upgrade before agent launch.

The failed UNKNOWN command and start transaction remain immutable evidence.
Only a factory-issued capability may restore the previous authentic stopped
bytes for one explicit successor request and sealed target. No process is
started or stopped here, and keeper departure never substitutes for native EOF.
"""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import os
from pathlib import Path
import re
from types import SimpleNamespace

from ..contracts import JenkinsError, canonical_json, digest
from ..state.locks import process_lock
from . import keeper_control, lost_owner, owned_upgrade as upgrade, recovery_keeper as recovery
from .pre_spawn_recovery_cancel import _ledger
from .recovery import has_terminal_lifecycle_proofs
from .startup_observations import runtime_owner_inventory
from ..processes.boot import windows_boot_filetime

KIND = "failed-after-spawn-owned-upgrade-reconciliation"
RETIREMENT_KIND = "failed-after-spawn-keeper-retirement"
GATE_KIND = "failed-after-spawn-owned-upgrade-restoration-gate"
_GATE = "recovery-lifecycle/failed-upgrade-gate.json"
_ISSUED = object()
_FALSE = ("nativeProofSynthesized", "nativeTerminationProofSynthesized",
          "historicalHoldsReleased", "runtimeAcceptance", "cargoAcceptance",
          "queueReplayRequested", "automaticRetry", "executionHoldsReleaseAllowed",
          "validationAcceptanceAllowed")
_RESTORE = {"stoppedOperation": "deployment-operation.json",
            "stoppedHost": "deployment/host.json", "previousSelector": "deployment/driver.json",
            "committedCommand": "recovery-lifecycle/command.json"}


def _require(condition, message, code="failed_upgrade_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _ref(path):
    return {"path": str(path), "sha256": _sha(upgrade._raw(path))}


def _paths(state, runtime):
    return {"command": state / "recovery-lifecycle/command.json",
            "operation": state / "deployment-operation.json", "host": state / "deployment/host.json",
            "selected": state / "deployment/driver.json",
            "transaction": state / "recovery-lifecycle/start-transactions" / (runtime + ".json")}


def _request_files(state):
    return {str(path): _sha(upgrade._raw(path))
            for area in ("requests", "results", "start-transactions")
            for path in sorted((state / "recovery-lifecycle" / area).glob("*.json"))}


def _failure(state, documents, references, release):
    """Validate positive recorded creation and real pre-agent terminal proofs."""
    command, operation, host, transaction, selected = (documents[key] for key in
        ("command", "operation", "host", "transaction", "selected"))
    repo, home = state.parent.parent, state.parent / "jenkins_home"
    runtime = operation.get("operationId")
    _require(isinstance(runtime, str) and re.fullmatch(r"start-[1-9][0-9]*", runtime)
             and set(references) == set(_paths(state, runtime))
             and all(references[key]["path"] == str(path) for key, path in _paths(state, runtime).items()),
             "The exact failed runtime pointers are required")
    _require(command.get("schemaVersion") == 1 and command.get("kind") == upgrade._COMMAND_KIND
             and command.get("status") == "unknown" and command.get("reasonCode") == "recovery_keeper_unproven"
             and command.get("variant") == upgrade.EXTERNAL_VARIANT and command.get("previousRuntimeReceipt") is None
             and command.get("recoveryDigest") is None and "result" not in command
             and all(command.get(key) is False for key in ("runtimeAcceptance", "historicalHoldsReleased")),
             "Only the original unresolved spawned upgrade is in scope")
    _require(operation.get("schemaVersion") == 2 and operation.get("component") == "lifecycle-host"
             and operation.get("state") == "failed" and operation.get("reasonCode") == "deployment_not_ready"
             and operation.get("controlPlaneOnly") is False and operation.get("homePath") == str(home)
             and host.get("status") == "failed" and host.get("reasonCode") == "agent_node_migration_unproven"
             and host.get("agentLaunchAttempted") is False and host.get("agentIdentity") is None
             and host.get("agentTerminationProof") is None
             and all(value.get("operationId") == value.get("generation") == runtime for value in (operation, host)),
             "This route requires a recorded bootstrap failure before any agent launch")
    keeper = upgrade._identity(command.get("keeper"), repo / ".jenkins/runtime/python/python.exe")
    supervisor = upgrade._identity(operation, repo / ".jenkins/runtime/python/python.exe")
    controller = upgrade._identity(host.get("controllerIdentity") or host.get("identity"),
                                   repo / ".jenkins/runtime/jdk/bin/java.exe")
    _require(len({item["pid"] for item in (keeper, supervisor, controller)}) == 3
             and int(keeper["creationTime"]) <= int(supervisor["creationTime"]) <= int(controller["creationTime"])
             and host.get("identity") == controller and host.get("hostPid") == supervisor["pid"],
             "The failed keeper, supervisor, and controller identities must be exact")
    _require(transaction.get("schemaVersion") == 1 and transaction.get("kind") == "formal-recovery-start-transaction"
             and transaction.get("state") == "unknown" and transaction.get("operationId") == runtime
             and transaction.get("spawnAttempted") is True and transaction.get("homePath") == str(home)
             and transaction.get("createdPid") == supervisor["pid"] and transaction.get("createdIdentity") == supervisor
             and transaction.get("operation") == operation and transaction.get("selected") == selected
             and transaction.get("selectedSha256") == references["selected"]["sha256"]
             and all(transaction.get(key) == command[key] for key in ("keeper", "release", "request"))
             and all(transaction.get(key) is False for key in ("automaticRetry", "runtimeAcceptance", "cargoAcceptance",
                                                              "nativeTerminationProven", "historicalHoldsReleased")),
             "The original unknown start transaction must preserve its exact creation")
    _require(release.get("kind") == recovery.OWNED_UPGRADE_RELEASE_KIND
             and release.get("ownedUpgrade", {}).get("variant") == upgrade.EXTERNAL_VARIANT
             and release.get("failedOwnStart") is None and release.get("failedStartReconciliation") is None
             and selected.get("state") == "selected-for-runtime-start" and selected.get("runtimeOperationId") == runtime
             and selected.get("recoveryRelease") == command["release"]
             and selected.get("supportRegression") == release["regression"]
             and selected.get("preparation") == release["preparation"]["path"]
             and all(selected.get(key) == value for key, value in release["targetDriver"].items())
             and selected.get("runtimeAcceptance") is False and selected.get("cargoAcceptance") is False
             and operation.get("ownedUpgradeDigest") == command.get("ownedUpgradeDigest")
             and operation.get("previousOwnedStop") == command.get("externalNativeStop"),
             "The failed selector must belong to its original sealed upgrade")
    args = [str(repo / ".jenkins/runtime/python/python.exe"), selected["launcher"], "__lifecycle-host",
            "--spec", str(repo / ".jenkins/deployment-spec.json"), "--java", controller["executable"],
            "--war", str(repo / ".jenkins/runtime/jenkins.war"), "--operation-id", runtime]
    _require(transaction.get("requestedCommand") == args
             and transaction.get("requestedCommandDigest") == supervisor["commandDigest"] == _sha("\0".join(args).encode()),
             "The actual supervisor creation must bind its exact selected launcher")
    spawn = command.get("createdSupervisor", {})
    _require(spawn.get("creator") == keeper and spawn.get("child") == supervisor
             and spawn.get("creationFlags") == 0x08000200 and spawn.get("requestedProfile") == "zircon-strict"
             and spawn.get("creatorInJob") is True and spawn.get("childInJob") is True,
             "Real inherited strict supervisor creation evidence is required")
    for key in ("actualRestrictionPreflight", "associatedJobAtSpawn"):
        recovery._restriction(spawn.get(key, {}), keeper, release["config"])
    members = spawn["associatedJobAtSpawn"]["associatedJob"]["memberPids"]
    _require(keeper["pid"] in members and supervisor["pid"] in members,
             "The recorded strict creation membership must include both identities")
    proof = host.get("nativeTerminationProof", {})
    _require(proof.get("requested") is True and type(proof.get("processExitCode")) is int
             and 0 <= proof["processExitCode"] < 2 ** 32 and proof["processExitCode"] != 259,
             "The existing controller proof must record its actual completed termination")
    return {"keeper": keeper, "supervisor": supervisor, "controller": controller}


def _retirement(state, reference, pins, documents, release):
    value, _ = upgrade._reference(state, reference)
    keeper = pins["identities"]["keeper"]
    expected = [str(state.parent.parent / ".jenkins/runtime/python/python.exe"), "-B",
                str(state.parent.parent / "tools/dev/jenkins_recovery_keeper.py"),
                "--config-sha256", release["config"]["sha256"], "--release-file", documents["command"]["release"]["path"],
                "--release-sha256", documents["command"]["release"]["sha256"]]
    arguments = value.get("beforeArguments")
    _require(value.get("schemaVersion") == 1 and value.get("kind") == RETIREMENT_KIND
             and value.get("keeper") == value.get("beforeIdentity") == keeper
             and value.get("runtimeOperationId") == pins["runtimeOperationId"]
             and value.get("originalRefs") == pins["originalRefs"]
             and value.get("protectedHolds") == pins["protectedHolds"]
             and value.get("serviceNativeTerminationProof") == pins["originalRefs"]["host"]
             and value.get("nativeHandleRetained") is True and value.get("handleIdentityVerified") is True
             and value.get("handleIdentity") == {key: keeper[key] for key in ("pid", "creationTime", "executable")}
             and value.get("method") == "forced-terminate-held-handle"
             and type(value.get("waitResult")) is int and value["waitResult"] == 0
             and type(value.get("processExitCode")) is int and value["processExitCode"] == 198
             and isinstance(value.get("endFileTime"), str) and value["endFileTime"].isascii()
             and value["endFileTime"].isdecimal() and int(value["endFileTime"]) >= int(keeper["creationTime"])
             and arguments in (expected, [*expected, "--no-tray"])
             and _sha("\0".join(arguments).encode()) == keeper["commandDigest"]
             and all(value.get(key) is False for key in ("historicalHoldsReleased", "runtimeAcceptance",
                                                        "cargoAcceptance", "nativeTerminationProofSynthesized")),
             "The failed keeper needs its real identity-bound retained-handle exit")
    _require(value.get("bootFileTime", pins["bootFileTime"]) == pins["bootFileTime"],
             "The recorded failed keeper exit belongs to another boot")
    forced = value.get("terminateProcess", {})
    _require(forced.get("api") == "TerminateProcess" and forced.get("succeeded") is True
             and type(forced.get("requestedExitCode")) is int and forced["requestedExitCode"] == value["processExitCode"],
             "The retirement must record the actual native API result")
    for name, field in (("keeper", "afterDeparture"), ("supervisor", "supervisorDeparture"),
                        ("controller", "controllerDeparture")):
        upgrade._departure_record(value.get(field), pins["identities"][name])
    _require(not {"nativeTerminationProof", "agentTerminationProof", "stdoutEof", "stderrEof"} & value.keys(),
             "Keeper exit cannot synthesize service NativeJob EOF")
    return value


def _chain(manager, command, release, handoff, planned, transaction):
    state = manager.paths.state.absolute()
    marker = recovery._owned_upgrade_marker(state, release)
    _require(handoff.get("schemaVersion") == 1 and handoff.get("kind") == "owned-recovery-keeper-handoff"
             and handoff.get("outcome") == "owned-transfer" and handoff.get("variant") == upgrade.EXTERNAL_VARIANT
             and all(handoff.get(key) == command[key] for key in ("keeper", "release", "request", "ownedUpgradeDigest"))
             and all(handoff.get(key) == marker[key] for key in ("externalNativeStop", "keeperRetirement"))
             and handoff.get("originalBlockedDescriptor") == command.get("replacesOriginalBlockedDescriptor")
             and handoff.get("committedCommand") == command.get("replacesCommittedCommand")
             and handoff.get("externalNativeStop") == command.get("externalNativeStop")
             and all(handoff.get(key) is False for key in ("nativeProofSynthesized", "historicalHoldsReleased",
                                                          "runtimeAcceptance", "cargoAcceptance", "queueReplayRequested", "automaticRetry")),
             "The failed upgrade must retain its complete genuine owned handoff")
    artifacts = handoff.get("originalArtifacts", {})
    _require(set(artifacts) == {*_RESTORE, "originalBlockedDescriptor"},
             "All five authentic predecessor byte archives are required")
    documents = {}
    for name, binding in artifacts.items():
        _require(set(binding) == {"original", "archive"}
                 and binding["original"]["sha256"] == binding["archive"]["sha256"],
                 "A predecessor archive must preserve the exact original bytes")
        documents[name], _ = upgrade._reference(state, binding["archive"])
        expected_path = state / (_RESTORE.get(name) or "formal-lifecycle/recovery-keeper.json")
        _require(binding["original"]["path"] == str(expected_path), "A predecessor pointer escaped its exact namespace")
    descriptor = documents["originalBlockedDescriptor"]
    _require(upgrade._raw(state / "formal-lifecycle/recovery-keeper.json")
             == upgrade._raw(Path(artifacts["originalBlockedDescriptor"]["archive"]["path"])),
             "The failed attempt must not have issued a replacement runtime descriptor")
    _require(transaction.get("previousOperation") == documents["stoppedOperation"]
             and transaction.get("previousHost") == documents["stoppedHost"]
             and transaction.get("previousSelector") == documents["previousSelector"]
             and transaction.get("previousSelectorSha256") == artifacts["previousSelector"]["archive"]["sha256"],
             "The spawn transaction must retain the authentic stopped predecessor projections")
    context = upgrade._external_historical_configuration(manager, descriptor, marker["externalNativeStop"])
    kwargs = {} if context is None else {"historical_configuration": context}
    old_release, old_target = upgrade._historical_release(manager, descriptor["release"], **kwargs)
    origin = recovery._runtime_receipt(manager, descriptor, old_release, old_target,
                                      external_native_stop_ref=marker["externalNativeStop"], **kwargs)
    upgrade._committed_command(state, descriptor, origin, documents["committedCommand"])
    stopped = upgrade._external_native_stop(manager, descriptor, marker["externalNativeStop"])
    _require(stopped["operation"] == documents["stoppedOperation"] and stopped["host"] == documents["stoppedHost"]
             and descriptor["runtime"]["selected"] == documents["previousSelector"]
             and descriptor["protectedHolds"] == release["protectedHolds"]
             and planned.get("schemaVersion") == 1 and planned.get("kind") == upgrade.KIND
             and planned.get("outcome") == "owned-transfer" and planned.get("variant") == upgrade.EXTERNAL_VARIANT
             and digest(planned) == command["ownedUpgradeDigest"]
             and planned.get("successorRelease") == command["release"]
             and planned.get("requestId") == release["requestId"]
             and planned.get("owner") == release["expectedPreviousOwner"]
             and planned.get("lifecycleFence") == release["expectedLifecycleFence"]
             and planned.get("previousKeeper") == descriptor["keeper"]
             and planned.get("previousRelease") == descriptor["release"]
             and planned.get("rawAnchors") == {name: value["original"] for name, value in artifacts.items()},
             "The original owned plan and real stopped native chain must remain exact")
    from .prelaunch_upgrade_recovery import validate_prelaunch_restoration_lineage
    prior = validate_prelaunch_restoration_lineage(state, release, command["release"])
    retirement_id = release["requestId"] if prior is None else prior["eligibility"]["originalRequestId"]
    upgrade._retirement(manager, marker["keeperRetirement"], request_id=retirement_id,
        descriptor={**descriptor, "configurationSha256": old_release["config"]["sha256"]},
        stopped_descriptor=marker["originalBlockedDescriptor"], owned_stop=marker["externalNativeStop"],
        boot=planned["bootFileTime"], external=True)
    return artifacts, retirement_id, upgrade._external_delivery_inventory(state, **kwargs)


def _runtime(manager, review_ref, target):
    if review_ref is None:
        inventory = runtime_owner_inventory(manager.paths.repo)
        _require(inventory == [], "A repository lifecycle owner or unknown runtime remains live")
        return []
    from .runtime_roles import prepare_runtime_role_review
    review = prepare_runtime_role_review(manager, review_reference=review_ref)
    _require(review.document["reviewedTarget"] == target, "The foreign client review belongs to another sealed target")
    observed = review.observe(manager)
    _require(observed["blockingOwners"] == [], "A reviewed client cannot exempt a lifecycle owner")
    return {"review": review_ref, "originalRuntimeInventory": review.document["originalRuntimeInventory"],
            "reviewedClients": review.document["clients"], "blockingOwners": []}


def _prepare(manager, command_sha, retirement_ref, request_id, target, review_ref=None):
    repo, state, home = upgrade._paths(manager)
    _require(isinstance(request_id, str) and re.fullmatch(r"[0-9a-f]{32}", request_id),
             "One explicit fresh replacement request is required")
    upgrade._request_unused(state, request_id)
    _require(not (state / _GATE).exists(), "An existing after-spawn restoration must be reconciled")
    command_path = state / "recovery-lifecycle/command.json"
    _require(_sha(upgrade._raw(command_path)) == command_sha, "The original UNKNOWN command changed")
    command = recovery._read(command_path)
    release, failed_target = upgrade._historical_release(manager, command["release"])
    target = upgrade._sealed_target(repo, target)
    _require(target["driverDigest"] != failed_target["driverDigest"] and request_id != release["requestId"],
             "A failed original request or unchanged target cannot be retried")
    runtime = recovery._read(state / "deployment-operation.json")["operationId"]
    paths = _paths(state, runtime)
    raw = {key: upgrade._raw(path) for key, path in paths.items()}
    references = {key: {"path": str(paths[key]), "sha256": _sha(value)} for key, value in raw.items()}
    documents = {key: upgrade._document(value) for key, value in raw.items()}
    identities = _failure(state, documents, references, release)
    request, _ = upgrade._reference(state, command["request"])
    _require(request.get("schemaVersion") == 1 and request.get("kind") == upgrade._REQUEST_KIND
             and request.get("action") == "upgrade" and request.get("requestId") == release["requestId"]
             and all(request.get(key) == command[key] for key in ("keeper", "release", "ownedUpgradeDigest"))
             and request.get("protectedHoldsDigest") == digest(release["protectedHolds"])
             and request.get("runtimeAcceptance") is False, "The original immutable upgrade request changed")
    handoff, _ = upgrade._reference(state, command["ownedUpgradeHandoff"])
    plans = [path for path in sorted((state / "recovery-lifecycle/history").glob("owned-upgrade-initial-plan-*.json"))
             if digest(recovery._read(path)) == command["ownedUpgradeDigest"]]
    _require(len(plans) == 1, "The original failed upgrade needs one exact immutable plan")
    plan_ref = _ref(plans[0])
    planned = recovery._read(plans[0])
    artifacts, retirement_id, delivery = _chain(manager, command, release, handoff, planned, documents["transaction"])
    boot = str(windows_boot_filetime())
    _require(boot == planned["bootFileTime"] and all(int(identity["creationTime"]) >= int(boot)
             for identity in identities.values()), "The failed creation must remain in its actual original boot")
    departures = {name: upgrade._departure(identity) for name, identity in identities.items()}
    _require(has_terminal_lifecycle_proofs(documents["operation"], documents["host"], str(home), departures["supervisor"]),
             "The real failed supervisor and NativeJob terminal proof must both be complete")
    holds = upgrade.protected_holds(state)
    _require(len(holds) == 4 and holds == release["protectedHolds"]
             == planned["executionQuarantine"]["protectedHolds"], "All four historical holds must remain byte exact")
    pins = {"originalRefs": references, "runtimeOperationId": runtime, "identities": identities,
            "bootFileTime": boot, "protectedHolds": holds}
    _retirement(state, retirement_ref, pins, documents, release)
    inventory = _runtime(manager, review_ref, target)
    policy = lost_owner._home_policy(home, "execution-runtime")
    home_observation = upgrade.observe_home_available(home)
    _require(home_observation.get("staleMarkersDeleted") is False, "Home ownership markers must remain preserved")
    upgrade.check_port_available(manager.spec.controller["listenAddress"], int(manager.spec.controller["httpPort"]))
    execution = lost_owner._execution_gate(manager, {"owner": {"operation": documents["operation"],
        "supervisor": identities["supervisor"], "controller": identities["controller"], "agent": None},
        "selected": documents["selected"], "anchor": SimpleNamespace(value={"protectedHolds": holds})})
    _require(execution["protectedHolds"] == holds and execution["classification"]["activeExecutions"] == [],
             "Current execution work requires its original separate authority")
    fence = lost_owner._lifecycle_fence(manager)
    for name in upgrade._LEGACY:
        _require(fence[name] == planned["lifecycleFence"][name], "A historical lifecycle anchor changed")
    _require(all(upgrade._raw(path) == raw[key] for key, path in paths.items())
             and str(windows_boot_filetime()) == boot
             and _runtime(manager, review_ref, target) == inventory, "Failed startup evidence changed during observation")
    eligibility = {**pins, "originalRequestId": release["requestId"],
        "predecessorRetirementRequestId": retirement_id, "release": command["release"], "request": command["request"],
        "ownedUpgradeHandoff": command["ownedUpgradeHandoff"], "initialUpgradePlan": plan_ref,
        "rawAnchors": {key: value["original"] for key, value in artifacts.items()},
        "predecessorArchives": {key: value["archive"] for key, value in artifacts.items()},
        "lifecycleFence": fence, "ledgerDigest": _ledger(state), "requestFiles": _request_files(state),
        "executionQuarantine": execution, "deliveryInventory": delivery, "runtimeOwnerInventory": inventory,
        "homePolicy": policy, "homeObservation": home_observation, "departedIdentities": departures}
    return {"schemaVersion": 1, "kind": KIND, "eligibility": eligibility, "retirement": retirement_ref,
            "replacementRequestId": request_id, "replacementTarget": target,
            "runtimeRoleReview": review_ref, **{key: False for key in _FALSE}}


@dataclass(frozen=True, init=False)
class FailedUpgradeRecovery:
    snapshot: bytes
    _token: object
    _used: list

    def __init__(self, snapshot, token=None):
        _require(token is _ISSUED and type(snapshot) is bytes, "Only verified preparation may issue reconciliation authority")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)
        object.__setattr__(self, "_used", [False])

    @property
    def document(self):
        return upgrade._document(self.snapshot)

    def revalidate(self, manager):
        _require(type(self) is FailedUpgradeRecovery and self._token is _ISSUED and not self._used[0],
                 "An unused exact typed failed-upgrade capability is required")
        value = self.document
        current = _prepare(manager, value["eligibility"]["originalRefs"]["command"]["sha256"], value["retirement"],
                           value["replacementRequestId"], value["replacementTarget"], value["runtimeRoleReview"])
        _require(canonical_json(current) == self.snapshot, "The failed startup reconciliation changed", "failed_upgrade_changed")
        return current

    def restore(self, manager):
        state = manager.paths.state.absolute()
        with process_lock(state / "deployment-start.lock", reason="deployment_start_busy"):
            with process_lock(state / "recovery-lifecycle/owner.lock", reason="recovery_keeper_already_live"):
                with process_lock(state / "recovery-lifecycle-submit.lock", reason="recovery_keeper_request_busy"):
                    value = self.revalidate(manager)
                    pins = value["eligibility"]
                    evidence = {**pins["originalRefs"], "release": pins["release"], "request": pins["request"],
                                "ownedUpgradeHandoff": pins["ownedUpgradeHandoff"],
                                "initialUpgradePlan": pins["initialUpgradePlan"], "retirement": value["retirement"]}
                    archives = {}
                    for name, reference in evidence.items():
                        raw = upgrade._raw(Path(reference["path"]))
                        _require(_sha(raw) == reference["sha256"], "Failed startup bytes changed before archive")
                        archives[name] = {"original": reference,
                            "archive": recovery._archive_bytes(state, "failed-after-spawn-" + name, raw)}
                    self.revalidate(manager)
                    self._used[0] = True
                    intent = recovery._archive(state, "failed-after-spawn-restore-intent", {**value,
                        "outcome": "authentic-stopped-predecessor-restoration-authorized", "archivedEvidence": archives,
                        "originalCommandStatus": "unknown"})
                    gate_path = state / _GATE
                    recovery._require(not gate_path.exists(), "After-spawn restoration already exists")
                    with gate_path.open("xb") as stream:
                        stream.write(canonical_json({"schemaVersion": 1, "kind": GATE_KIND,
                            "status": "restoring", "intent": intent, "claimedRelease": None}) + b"\n")
                        stream.flush()
                        os.fsync(stream.fileno())
                    source_names = {"stoppedOperation": "operation", "stoppedHost": "host",
                                    "previousSelector": "selected", "committedCommand": "command"}
                    restored = {}
                    for name, relative in _RESTORE.items():
                        path = state / relative
                        reference = pins["predecessorArchives"][name]
                        raw = upgrade._raw(Path(reference["path"]))
                        _require(_sha(raw) == reference["sha256"], "Authentic stopped predecessor bytes changed")
                        keeper_control._write_raw(path, raw, expected_sha=pins["originalRefs"][source_names[name]]["sha256"])
                        restored[name] = _ref(path)
                    _require(_ledger(state) == pins["ledgerDigest"] and _request_files(state) == pins["requestFiles"]
                             and upgrade.protected_holds(state) == pins["protectedHolds"]
                             and str(windows_boot_filetime()) == pins["bootFileTime"]
                             and _ref(state / "formal-lifecycle/recovery-keeper.json") == pins["rawAnchors"]["originalBlockedDescriptor"],
                             "Restoration must preserve every old transaction, hold, and runtime issuance")
                    receipt = recovery._archive(state, "failed-after-spawn-restored", {**value,
                        "outcome": "authentic-stopped-predecessor-restored", "intent": intent, "archivedEvidence": archives,
                        "restoredPointers": restored, "restoredCommand": restored["committedCommand"],
                        "originalCommandStatus": "unknown"})
                    recovery._write(gate_path, {"schemaVersion": 1, "kind": GATE_KIND, "status": "reserved",
                        "reconciliation": receipt, "claimedRelease": None}, expected_sha=_sha(upgrade._raw(gate_path)))
                    return {"reconciliation": receipt, "restoredCommand": restored["committedCommand"],
                            "replacementRequestId": value["replacementRequestId"], "replacementTarget": value["replacementTarget"],
                            "runtimeRoleReview": value["runtimeRoleReview"], **{key: False for key in _FALSE}}


def prepare_failed_upgrade_recovery(manager, *, expected_command_sha256, keeper_retirement_ref,
                                    replacement_request_id, replacement_target, runtime_role_review_ref=None):
    value = _prepare(manager, expected_command_sha256, keeper_retirement_ref,
                     replacement_request_id, replacement_target, runtime_role_review_ref)
    return FailedUpgradeRecovery(canonical_json(value), _ISSUED)


def _receipt(state, reference):
    value, _ = upgrade._reference(state, reference)
    _require(value.get("schemaVersion") == 1 and value.get("kind") == KIND
             and value.get("outcome") == "authentic-stopped-predecessor-restored"
             and value.get("originalCommandStatus") == "unknown" and all(value.get(key) is False for key in _FALSE),
             "The failed-start reconciliation must preserve its honest original unknown")
    pins, archives = value["eligibility"], value.get("archivedEvidence", {})
    required = {"command", "operation", "host", "transaction", "selected", "release", "request",
                "ownedUpgradeHandoff", "initialUpgradePlan", "retirement"}
    _require(set(archives) == required, "Every original after-spawn failure artifact must remain archived")
    documents = {}
    for name, binding in archives.items():
        _require(set(binding) == {"original", "archive"} and binding["original"]["sha256"] == binding["archive"]["sha256"]
                 and Path(binding["archive"]["path"]).parent == state / "recovery-lifecycle/history",
                 "An immutable failure archive must preserve its original bytes")
        documents[name], _ = upgrade._reference(state, binding["archive"])
        _require(binding["original"] == (pins["originalRefs"].get(name) or
                 (value["retirement"] if name == "retirement" else pins[name])), "The failure archive was detached")
    repo = state.parent.parent
    failed_release = documents["release"]
    target = upgrade._sealed_target(repo, failed_release["targetDriver"])
    upgrade._historical_support(repo, state, failed_release, target)
    originals = {key: documents[key] for key in pins["originalRefs"]}
    identities = _failure(state, originals, pins["originalRefs"], failed_release)
    _require(identities == pins["identities"] and pins["originalRequestId"] == failed_release["requestId"]
             == documents["request"]["requestId"] and documents["request"]["action"] == "upgrade"
             and digest(documents["initialUpgradePlan"]) == documents["command"]["ownedUpgradeDigest"]
             and documents["command"]["ownedUpgradeHandoff"] == pins["ownedUpgradeHandoff"],
             "The immutable failed creation and original request must retain their exact lineage")
    _retirement(state, archives["retirement"]["archive"], pins, originals, failed_release)
    _require(has_terminal_lifecycle_proofs(documents["operation"], documents["host"],
             str(state.parent / "jenkins_home"), pins["departedIdentities"]["supervisor"]),
             "The original archived native EOF proof must remain complete")
    handoff = documents["ownedUpgradeHandoff"]
    _require(pins["predecessorArchives"] == {key: binding["archive"] for key, binding in handoff["originalArtifacts"].items()}
             and pins["rawAnchors"] == {key: binding["original"] for key, binding in handoff["originalArtifacts"].items()},
             "The authentic predecessor byte archives changed")
    for name, relative in _RESTORE.items():
        _, raw = upgrade._reference(state, pins["predecessorArchives"][name])
        _require(value["restoredPointers"][name] == {"path": str(state / relative), "sha256": _sha(raw)},
                 "Restoration must use the authentic predecessor bytes")
    _require(value["restoredCommand"] == value["restoredPointers"]["committedCommand"]
             and pins["protectedHolds"] == failed_release["protectedHolds"]
             and value["replacementRequestId"] != pins["originalRequestId"]
             and value["replacementTarget"]["driverDigest"] != target["driverDigest"],
             "The failed original request or target cannot be replayed")
    return value


def validate_failed_upgrade_restoration_gate(manager, release_ref, release, *, request_id,
                                              descriptor_sha256, command_sha256):
    state = manager.paths.state.absolute()
    path, reference = state / _GATE, release.get("failedStartReconciliation")
    if not path.exists():
        _require(reference is None, "A declared failed-start restoration lacks its gate")
        return None
    gate = recovery._read(path)
    _require(gate.get("schemaVersion") == 1 and gate.get("kind") == GATE_KIND
             and gate.get("status") in {"reserved", "claimed"}, "An incomplete restoration must remain blocked")
    receipt = _receipt(state, gate["reconciliation"])
    if receipt["eligibility"]["rawAnchors"]["originalBlockedDescriptor"]["sha256"] != descriptor_sha256:
        _require(reference is None, "Failed-start restoration cannot authorize another predecessor")
        return None
    _require(reference == gate["reconciliation"] and request_id == receipt["replacementRequestId"]
             and release.get("targetDriver") == receipt["replacementTarget"]
             and release.get("runtimeRoleReview") == receipt["runtimeRoleReview"]
             and command_sha256 == receipt["restoredCommand"]["sha256"]
             and gate.get("claimedRelease") in (None, release_ref),
             "The failed startup authorizes only its one explicit replacement request and target")
    return receipt


def claim_failed_upgrade_restoration_gate(manager, plan, request, keeper):
    _require(type(plan) is upgrade.OwnedRecoveryUpgradePlan and plan._token is upgrade._ISSUED,
             "Only an issued owned upgrade may claim failed-start restoration")
    value = plan.document
    state = manager.paths.state.absolute()
    release, _ = upgrade._reference(state, value["successorRelease"], release=True)
    descriptor_key, command_key, _ = upgrade._transfer_keys(value)
    receipt = validate_failed_upgrade_restoration_gate(manager, value["successorRelease"], release,
        request_id=value["requestId"], descriptor_sha256=value["rawAnchors"][descriptor_key]["sha256"],
        command_sha256=value["rawAnchors"][command_key]["sha256"])
    if receipt is None:
        return
    body, _ = upgrade._reference(state, request)
    _require(keeper["pid"] == os.getpid() and recovery._full_identity(os.getpid()) == keeper
             and body.get("keeper") == keeper and body.get("release") == value["successorRelease"]
             and body.get("requestId") == value["requestId"] and body.get("action") == "upgrade"
             and body.get("ownedUpgradeDigest") == plan.upgrade_digest and body.get("runtimeAcceptance") is False,
             "Only the exact live successor actor may claim its immutable request")
    path = state / _GATE
    gate = recovery._read(path)
    _require(gate.get("status") == "reserved" and gate.get("claimedRelease") is None,
             "This after-spawn restoration has already been consumed")
    recovery._write(path, {**gate, "status": "claimed", "claimedRelease": value["successorRelease"],
                          "request": request, "keeper": keeper}, expected_sha=_sha(upgrade._raw(path)))


def validate_failed_upgrade_restoration_lineage(state, release, release_ref):
    reference = release.get("failedStartReconciliation")
    if reference is None:
        return None
    receipt = _receipt(state, reference)
    gate = recovery._read(state / _GATE)
    _require(gate.get("status") == "claimed" and gate.get("reconciliation") == reference
             and gate.get("claimedRelease") == release_ref and release.get("requestId") == receipt["replacementRequestId"]
             and release.get("targetDriver") == receipt["replacementTarget"]
             and release.get("runtimeRoleReview") == receipt["runtimeRoleReview"],
             "The actual owned handoff must retain its consumed after-spawn restoration")
    return receipt


__all__ = ["FailedUpgradeRecovery", "prepare_failed_upgrade_recovery"]

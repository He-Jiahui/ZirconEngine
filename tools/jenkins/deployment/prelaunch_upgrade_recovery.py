"""Compensate one positively reviewed owned-upgrade rejection before spawn.

The failed command stays unknown in its immutable archive.  A real retained
keeper exit and the audited sealed rejection path authorize restoring only the
previous committed bytes for one fresh request and sealed target.
"""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import os
from pathlib import Path
import time
import uuid

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..state.atomic import replace_file
from ..state.locks import process_lock
from . import owned_upgrade as upgrade, recovery_keeper as recovery
from .driver import verify_driver
from .live_observations import observe_live_identity, _windows_command_line, _digest_args
from .pre_spawn_recovery_cancel import _ledger, _local_reference
from .startup_observations import runtime_owner_inventory, observe_departed_identity

KIND = "prelaunch-owned-upgrade-reconciliation"
RETIREMENT_KIND = "prelaunch-owned-upgrade-keeper-retirement"
GATE_KIND = "prelaunch-owned-upgrade-restoration-gate"
_GATE = "recovery-lifecycle/prelaunch-upgrade-gate.json"
_ISSUED = object()
_FALSE = ("nativeProofSynthesized", "historicalHoldsReleased", "runtimeAcceptance",
          "cargoAcceptance", "queueReplayRequested", "automaticRetry",
          "executionHoldsReleaseAllowed", "validationAcceptanceAllowed")
# Every occurrence of this rejection in this exact implementation is in
# OwnedRecoveryUpgradePlan.revalidate, before the manager creates a transaction
# or calls Popen.  A missing transaction or absent PID does not provide this
# positive proof.  Other generations or messages require a separate audit.
_REVIEWED_FAILURE_CODE = {
    "52d056b4222e40969259919085cba05970ae1311027acf2ce6c8e6e4a5e7a361": {
        "tools/jenkins/deployment/owned_upgrade.py": "2357f3e8db20db7609cd0df76777e456888b10314bc1e83896ae261de01f2536",
        "tools/jenkins/deployment/recovery_keeper.py": "e3330667d1d92540a2156e5de33caba394865e98e8ea64c1c26de74c198d2f7d",
        "tools/jenkins/deployment/manager.py": "c7559d298f6a2cc88874d3681e2cdf9530efec24e0fa4027f67396350fa93dca",
        "tools/jenkins/deployment/start_transaction.py": "771a5ee932888bde5b950e3102401a56fd483c8ffa8e369793f92a77202aaf4d",
        "tools/jenkins/deployment/startup_observations.py": "b9cf027720c18f9cc8c2cd357740d80cb5eed71ad51d07997807fa78f6baeb5f",
        "tools/dev/jenkins_recovery_keeper.py": "62ad2f63058f6aa970e972695da2319f69f51e538d12c1a199bbfa0f7a6938d3",
    },
}


def _require(condition, message, code="prelaunch_upgrade_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _ref(path):
    return {"path": str(path), "sha256": _sha(upgrade._raw(path))}


def _implementation(repo, release, target):
    sources = _REVIEWED_FAILURE_CODE.get(target["driverDigest"])
    _require(sources is not None and release.get("actorCodeRoot") == target["root"],
             "The original sealed rejection implementation has no positive review")
    result = {}
    for name, checksum in sources.items():
        path = (Path(target["root"]) if name.startswith("tools/jenkins/") else repo) / name
        _require(release.get("operatorFiles", {}).get(str(repo / name)) == checksum
                 and _sha(upgrade._raw(path)) == checksum, "The reviewed pre-spawn control bytes changed")
        result[str(path)] = checksum
    return result


def _request_files(state):
    return {str(path): _sha(upgrade._raw(path))
            for name in ("requests", "results", "start-transactions")
            for path in sorted((state / "recovery-lifecycle" / name).glob("*.json"))}


def _historical_chain(manager, release, descriptor, restored, plan, handoff):
    state = manager.paths.state.absolute()
    marker = release["ownedUpgrade"]
    context = upgrade._external_historical_configuration(manager, descriptor, marker["externalNativeStop"])
    old_release, old_target = upgrade._historical_release(
        manager, descriptor["release"], **({} if context is None else {"historical_configuration": context}))
    kwargs = {} if context is None else {"historical_configuration": context}
    origin = recovery._runtime_receipt(manager, descriptor, old_release, old_target,
                                      external_native_stop_ref=marker["externalNativeStop"], **kwargs)
    upgrade._committed_command(state, descriptor, origin, restored)
    stopped = upgrade._external_native_stop(manager, descriptor, marker["externalNativeStop"])
    _require(stopped["operation"] == recovery._read(state / "deployment-operation.json")
             and stopped["host"] == recovery._read(state / "deployment/host.json")
             and descriptor["runtime"]["selected"] == recovery._read(state / "deployment/driver.json"),
             "The genuine stopped predecessor pointers changed")
    upgrade._retirement(manager, marker["keeperRetirement"], request_id=release["requestId"],
        descriptor={**descriptor, "configurationSha256": old_release["config"]["sha256"]},
        stopped_descriptor=marker["originalBlockedDescriptor"], owned_stop=marker["externalNativeStop"],
        boot=plan["bootFileTime"], external=True)
    _require(plan.get("previousRelease") == descriptor["release"]
             and plan.get("previousKeeper") == descriptor["keeper"]
             and plan.get("previousTarget") == old_target
             and plan.get("owner") == release.get("expectedPreviousOwner")
             and plan.get("lifecycleFence") == release.get("expectedLifecycleFence")
             and handoff.get("previousRelease") == descriptor["release"]
             and handoff.get("previousKeeper") == descriptor["keeper"],
             "The original owned upgrade chain changed")
    return upgrade._external_delivery_inventory(state, **kwargs)


def _capture(manager, command_sha, holder_reference, *, runtime_role_review_ref=None):
    repo, state, home = upgrade._paths(manager)
    command_path = state / "recovery-lifecycle/command.json"
    raw = upgrade._raw(command_path)
    command = upgrade._document(raw)
    _require(_sha(raw) == command_sha and command.get("schemaVersion") == 1
             and command.get("kind") == upgrade._COMMAND_KIND and command.get("status") == "unknown"
             and command.get("reasonCode") == "owned_recovery_upgrade_unproven"
             and command.get("variant") == upgrade.EXTERNAL_VARIANT
             and command.get("previousRuntimeReceipt") is None and command.get("recoveryDigest") is None
             and all(command.get(name) is False for name in ("runtimeAcceptance", "historicalHoldsReleased"))
             and all(name not in command for name in ("createdSupervisor", "result", "startTransaction", "operationId")),
             "Only the exact positively reviewed pre-spawn unknown upgrade may be restored")
    release, target = upgrade._historical_release(manager, command["release"])
    _require(release["kind"] == recovery.OWNED_UPGRADE_RELEASE_KIND
             and release.get("ownedUpgrade", {}).get("variant") == upgrade.EXTERNAL_VARIANT,
             "An exact sealed external NativeStop upgrade release is required")
    control = _implementation(repo, release, target)
    holder = _local_reference(holder_reference, state / "recovery-lifecycle/holders")
    _require(holder.get("schemaVersion") == 1 and holder.get("kind") == "formal-recovery-desktop-holder"
             and holder.get("status") == "recovery-blocked"
             and holder.get("reasonCode") == command["reasonCode"]
             and holder.get("message") == "Another repository runtime still owns the deployment"
             and holder.get("reviewedRelease") == command["release"]
             and holder.get("actorCodeRoot") == target["root"]
             and holder.get("ownedUpgradeDigest") == command.get("ownedUpgradeDigest")
             and holder.get("recoveryDigest") is None
             and all(holder.get(name) is False for name in ("automaticRestart", "runtimeAcceptance",
                      "cargoAcceptance", "requestsResubmitted", "serviceLifecycleExecuted")),
             "A genuine publication of the audited pre-spawn rejection is required")
    request, _ = upgrade._reference(state, command["request"])
    _require(request.get("kind") == upgrade._REQUEST_KIND and request.get("schemaVersion") == 1
             and request.get("action") == "upgrade" and request.get("requestId") == release["requestId"]
             and request.get("keeper") == command.get("keeper") and request.get("release") == command["release"]
             and request.get("ownedUpgradeDigest") == command.get("ownedUpgradeDigest")
             and request.get("protectedHoldsDigest") == digest(release["protectedHolds"])
             and request.get("runtimeAcceptance") is False, "The original immutable upgrade request changed")
    handoff, _ = upgrade._reference(state, command["ownedUpgradeHandoff"])
    _require(handoff.get("schemaVersion") == 1 and handoff.get("kind") == upgrade.HANDOFF_KIND
             and handoff.get("outcome") == "owned-transfer" and handoff.get("variant") == upgrade.EXTERNAL_VARIANT
             and handoff.get("keeper") == command["keeper"] and handoff.get("release") == command["release"]
             and handoff.get("request") == command["request"]
             and handoff.get("ownedUpgradeDigest") == command["ownedUpgradeDigest"]
             and all(handoff.get(name) is False for name in (*upgrade._FLAGS, "queueReplayRequested", "automaticRetry")),
             "The original genuine owned handoff changed")
    marker = release["ownedUpgrade"]
    _require(command.get("externalNativeStop") == handoff.get("externalNativeStop") == marker["externalNativeStop"]
             and handoff.get("keeperRetirement") == marker["keeperRetirement"]
             and command.get("replacesOriginalBlockedDescriptor") == handoff.get("originalBlockedDescriptor")
             and command.get("replacesCommittedCommand") == handoff.get("committedCommand"),
             "Unknown command, handoff and reviewed NativeStop differ")
    keys = {"originalBlockedDescriptor", "committedCommand", "stoppedOperation", "stoppedHost", "previousSelector"}
    artifacts = handoff.get("originalArtifacts", {})
    _require(set(artifacts) == keys, "Every original stopped ownership archive is required")
    originals = {}
    documents = {}
    paths = {**upgrade._ARTIFACTS, "originalBlockedDescriptor": upgrade._ARTIFACTS["stoppedDescriptor"],
             "committedCommand": upgrade._ARTIFACTS["terminalCommand"]}
    for name, binding in artifacts.items():
        _require(set(binding) == {"original", "archive"}
                 and binding["original"]["path"] == str(state / paths[name])
                 and binding["original"]["sha256"] == binding["archive"]["sha256"],
                 "The original stopped ownership archive binding changed")
        documents[name], archived_raw = upgrade._reference(state, binding["archive"])
        if name != "committedCommand":
            _require(upgrade._raw(state / paths[name]) == archived_raw, "The stopped predecessor bytes changed")
        originals[name] = binding["original"]
    _require(handoff["originalBlockedDescriptor"] == artifacts["originalBlockedDescriptor"]["archive"]
             and handoff["committedCommand"] == artifacts["committedCommand"]["archive"]
             and marker["originalBlockedDescriptor"]["sha256"] == originals["originalBlockedDescriptor"]["sha256"]
             and marker["committedCommand"]["sha256"] == originals["committedCommand"]["sha256"],
             "The archived restoration bytes differ from the reviewed release")
    plan_matches = []
    for path in sorted((state / "recovery-lifecycle/history").glob("owned-upgrade-initial-plan-*.json")):
        value = recovery._read(path)
        if digest(value) == command["ownedUpgradeDigest"]:
            plan_matches.append((_ref(path), value))
    _require(len(plan_matches) == 1, "The exact immutable initial upgrade plan is required")
    plan_ref, plan = plan_matches[0]
    _require(plan.get("kind") == upgrade.KIND and plan.get("schemaVersion") == 1
             and plan.get("outcome") == "owned-transfer" and plan.get("variant") == upgrade.EXTERNAL_VARIANT
             and plan.get("requestId") == release["requestId"] and plan.get("successorRelease") == command["release"]
             and plan.get("targetDriver") == target and plan.get("rawAnchors") == originals
             and all(plan.get(name) is False for name in (*upgrade._FLAGS, "automaticRetry")),
             "The original issued upgrade plan changed")
    boot = str(upgrade.windows_boot_filetime())
    _require(boot == plan.get("bootFileTime") and int(command["keeper"]["creationTime"]) >= int(boot),
             "This reconciliation cannot cross a Windows boot")
    holds = upgrade.protected_holds(state)
    _require(len(holds) == 4 and holds == release["protectedHolds"]
             == documents["originalBlockedDescriptor"]["protectedHolds"]
             == plan["executionQuarantine"]["protectedHolds"], "All four original historical holds must remain exact")
    _require(plan.get("binding", {}).get("homePath") == str(home)
             and plan["binding"].get("repository") == str(repo), "The original deployment layout changed")
    delivery = _historical_chain(manager, release, documents["originalBlockedDescriptor"],
                                 documents["committedCommand"], plan, handoff)
    recovery._restriction(holder.get("restrictionPreflight", {}), command["keeper"], release["config"])
    sandbox = holder["officialSandboxHost"]
    host = _local_reference(sandbox["record"], state / "recovery-lifecycle/sandbox-hosts")
    pinned = release["sandboxHost"]
    _require(host.get("kind") == "official-strict-recovery-sandbox-host"
             and host.get("release") == command["release"] and host.get("host") == sandbox.get("host")
             and host.get("profile") == pinned.get("profile") == "zircon-strict"
             and host.get("includeManagedConfig") is pinned.get("includeManagedConfig") is True
             and host.get("automaticRestart") is False and host.get("runtimeAcceptance") is False
             and sandbox.get("officialHostSha256") == pinned["sha256"]
             and _sha(upgrade._raw(Path(pinned["path"]))) == pinned["sha256"],
             "The original official strict sandbox host changed")
    files = _request_files(state)
    for path in (state / "recovery-lifecycle/start-transactions").glob("*.json"):
        transaction = recovery._read(path)
        _require(transaction.get("keeper") != command["keeper"] and transaction.get("request") != command["request"]
                 and transaction.get("release") != command["release"],
                 "A real start transaction contradicts the positive pre-spawn rejection")
    _require(not (state / "recovery-lifecycle/results" / (release["requestId"] + ".json")).exists(),
             "A result contradicts the failed unknown upgrade")
    fence = upgrade.lost_owner._lifecycle_fence(manager)
    _require(fence == plan["lifecycleFence"], "The exact stopped deployment fence changed")
    home_observation = upgrade.observe_home_available(home)
    _require(home_observation.get("staleMarkersDeleted") is False, "Home ownership evidence may not be erased")
    endpoint = plan["binding"]["endpoint"]
    upgrade.check_port_available(endpoint["address"], endpoint["port"])
    from types import SimpleNamespace
    execution = upgrade.lost_owner._execution_gate(manager, {"owner": plan["owner"],
        "selected": documents["previousSelector"], "anchor": SimpleNamespace(value={"protectedHolds": holds})})
    _require(execution.get("protectedHolds") == holds and execution["classification"]["activeExecutions"] == [],
             "A current execution contradicts this empty prelaunch upgrade")
    pins = {"schemaVersion": 1, "kind": "prelaunch-owned-upgrade-eligibility",
            "originalCommand": _ref(command_path), "holder": holder_reference, "release": command["release"],
            "keeper": command["keeper"], "request": command["request"], "originalRequestId": release["requestId"],
            "ownedUpgradeHandoff": command["ownedUpgradeHandoff"],
            "initialUpgradePlan": plan_ref, "bootFileTime": boot, "controlFlow": control,
            "rawAnchors": {**originals, "committedCommand": artifacts["committedCommand"]["archive"],
                           "originalCommand": _ref(command_path)},
            "originalArtifacts": artifacts, "restoredCommand": artifacts["committedCommand"]["archive"],
            "protectedHolds": holds, "ledgerDigest": _ledger(state), "requestFiles": files,
            "lifecycleFence": fence, "deliveryInventory": delivery, **{name: False for name in _FALSE}}
    if runtime_role_review_ref is not None:
        from .runtime_roles import prepare_runtime_role_review
        reviewed = prepare_runtime_role_review(manager, review_reference=runtime_role_review_ref)
        pins["runtimeRoleReview"] = runtime_role_review_ref
        pins["runtimeBoundary"] = {"review": runtime_role_review_ref,
            "originalRuntimeInventory": reviewed.document["originalRuntimeInventory"],
            "reviewedClients": reviewed.document["clients"]}
    return pins, raw, command, holder, host


def inspect_prelaunch_upgrade_recovery(manager, *, expected_command_sha256, holder_reference,
                                      runtime_role_review_ref=None):
    """Read-only live eligibility; native retirement remains the operator's act."""
    kwargs = {} if runtime_role_review_ref is None else {"runtime_role_review_ref": runtime_role_review_ref}
    pins, _, command, _, _ = _capture(manager, require_digest(expected_command_sha256), holder_reference, **kwargs)
    observe_live_identity(command["keeper"])
    recovery._keeper_argv(manager.paths.repo, command["keeper"], command["release"],
                         recovery._read(Path(command["release"]["path"]))["config"]["sha256"])
    inventory = runtime_owner_inventory(manager.paths.repo)
    if runtime_role_review_ref is not None:
        from .runtime_roles import prepare_runtime_role_review
        boundary = prepare_runtime_role_review(manager, review_reference=runtime_role_review_ref).observe(manager)
        return {**pins, "observedRuntimeOwnerInventory": boundary["runtimeInventory"], "runtimeBoundary": boundary}
    return {**pins, "observedRuntimeOwnerInventory": inventory}


def _retirement(state, reference, pins, command, holder, host, *, manager=None):
    record = _local_reference(reference, state / "recovery-lifecycle/history")
    keeper = command["keeper"]
    _require(record.get("schemaVersion") == 1 and record.get("kind") == RETIREMENT_KIND
             and record.get("originalCommand") == pins["originalCommand"] and record.get("holder") == pins["holder"]
             and record.get("ownedUpgradeHandoff") == pins["ownedUpgradeHandoff"]
             and record.get("release") == pins["release"] and record.get("keeper") == keeper
             and record.get("bootFileTime") == pins["bootFileTime"]
             and all(record.get(name) == pins[name] for name in
                     ("rawAnchors", "requestFiles", "controlFlow", "protectedHolds", "ledgerDigest"))
             and all(record.get(name) is False for name in
                     ("nativeProofSynthesized", "historicalHoldsReleased", "runtimeAcceptance", "cargoAcceptance",
                      "queueReplayRequested", "automaticRetry")),
             "The retained exit record differs from the exact positive rejection pins")
    _require(record.get("beforeIdentity") == keeper and record.get("handleIdentity") ==
             {name: keeper[name] for name in ("pid", "creationTime", "executable")}
             and record.get("handleIdentityVerified") is True and record.get("nativeHandleRetained") is True
             and type(record.get("waitResult")) is int and record["waitResult"] == 0
             and type(record.get("processExitCode")) is int and record["processExitCode"] == 196
             and record.get("method") == "forced-terminate-held-handle"
             and record.get("forced") is True and record.get("graceful") is False
             and record.get("terminateProcess") == {"api": "TerminateProcess", "succeeded": True, "requestedExitCode": 196},
             "Only the actual retained keeper handle exit may authorize restoration")
    expected = [keeper["executable"], "-B", str(state.parent.parent / "tools/dev/jenkins_recovery_keeper.py"),
                "--config-sha256", recovery._read(Path(command["release"]["path"]))["config"]["sha256"],
                "--release-file", command["release"]["path"], "--release-sha256", command["release"]["sha256"]]
    arguments = record.get("beforeArguments")
    _require(arguments in (expected, [*expected, "--no-tray"]) and _digest_args(tuple(arguments)) == keeper["commandDigest"],
             "The retired handle belongs to another keeper command")
    scope = record.get("scope", {})
    roles = {"keeper", "console", "runner", "officialHost", "hostConsole", "dispatcher"}
    actors = scope.get("processes", {})
    _require(set(actors) == roles and actors["keeper"] == keeper and actors["officialHost"] == host["host"]
             and len({value["pid"] for value in actors.values()}) == 6
             and scope.get("terminationScope") == ["keeper"] and scope.get("otherActorsExitNaturally") is True
             and scope.get("completeLiveJobInventoryClaimed") is False
             and scope.get("publishedAssociatedJobMemberPids") == sorted(holder["restrictionPreflight"]["associatedJob"]["memberPids"])
             and scope.get("currentContainedActorPids") == sorted([keeper["pid"], actors["console"]["pid"]])
             and scope.get("currentOfficialHostDescendants") == sorted(value["pid"] for name, value in actors.items()
                                                                      if name != "dispatcher"),
             "The positively observed empty official sandbox process tree changed")
    before = record.get("runtimeOwnerInventoryBefore", [])
    review_ref = pins.get("runtimeRoleReview")
    if review_ref is None:
        _require(record.get("runtimeRoleReview") is None and scope.get("runtimeBoundary") is None,
                 "A legacy native retirement cannot silently declare a foreign runtime review")
        blocking_before = before
    else:
        from .runtime_roles import prepare_runtime_role_review
        review = prepare_runtime_role_review(manager, review_reference=review_ref)
        boundary = scope.get("runtimeBoundary", {})
        _require(record.get("runtimeRoleReview") == review_ref and boundary.get("review") == review_ref
                 and boundary.get("kind") == "reviewed-runtime-owner-observation"
                 and boundary.get("runtimeInventory") == before
                 and all(boundary.get(name) is False for name in ("runtimeAcceptance", "cargoAcceptance",
                         "historicalHoldsReleased", "nativeProofIssued", "serviceLifecycleExecuted", "loadedModuleBytesProven")),
                 "The native retirement must preserve its complete explicit runtime boundary")
        enrolled = {item["identity"]["pid"]: item for item in review.document["clients"]}
        foreign = boundary.get("reviewedForeign", [])
        _require(len(foreign) == len(enrolled) and {item["identity"]["pid"] for item in foreign} == set(enrolled)
                 and all(all(item.get(key) == enrolled[item["identity"]["pid"]][key]
                             for key in ("identity", "arguments", "role", "sourceFiles", "physicalOutputs"))
                         and item.get("status") in {"live", "departed"} for item in foreign),
                 "The exact positively reviewed foreign clients changed in the native receipt")
        blocking_before = boundary.get("blockingOwners", [])
        live = [item["identity"] for item in foreign if item["status"] == "live"]
        _require(len(before) == len(blocking_before) + len(live)
                 and all(any(all(str(actual.get(key)) == str(expected[key])
                     for key in ("pid", "creationTime", "executable")) for actual in before)
                     for expected in [*blocking_before, *live]),
                 "The preserved full native inventory contains an unreviewed owner")
    _require(len(blocking_before) == 1 and all(str(blocking_before[0].get(name)) == str(keeper[name])
                                    for name in ("pid", "creationTime", "executable")),
             "The retained attempt was not the sole original native service owner")
    exits = record.get("exits", [])
    _require(len(exits) == 6 and {item.get("role") for item in exits} == roles, "Every original held process exit is required")
    for item in exits:
        _require(item.get("identity") == actors[item["role"]] and item.get("nativeHandleRetained") is True
                 and item.get("handleIdentityVerified") is True and type(item.get("waitResult")) is int
                 and item["waitResult"] == 0 and type(item.get("processExitCode")) is int
                 and 0 <= item["processExitCode"] < 2 ** 32 and item["processExitCode"] != 259
                 and (item["role"] != "keeper" or item["processExitCode"] == 196),
                 "PID disappearance cannot replace a retained process handle wait")
        upgrade._departure_record(observe_departed_identity(actors[item["role"]]), actors[item["role"]])
    upgrade._departure_record(record.get("afterDeparture"), keeper)
    _require(all(name not in record for name in ("nativeTerminationProof", "agentTerminationProof", "stdoutEof", "stderrEof")),
             "Prelaunch keeper exit may not manufacture service termination or EOF")
    current = runtime_owner_inventory(state.parent.parent) if review_ref is None else review.observe(manager)["blockingOwners"]
    _require(current == [], "A new repository runtime owner appeared")
    return record


@dataclass(frozen=True, init=False)
class PrelaunchUpgradeRecovery:
    snapshot: bytes
    _token: object
    _used: list

    def __init__(self, snapshot, token=None):
        _require(token is _ISSUED and type(snapshot) is bytes, "Only the verified preparation factory may issue restoration authority")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)
        object.__setattr__(self, "_used", [False])

    @property
    def document(self):
        return upgrade._document(self.snapshot)

    def revalidate(self, manager):
        _require(type(self) is PrelaunchUpgradeRecovery and self._token is _ISSUED and not self._used[0],
                 "An exact unused typed prelaunch restoration capability is required")
        expected = self.document
        current = _prepare(manager, expected["eligibility"]["originalCommand"]["sha256"],
                           expected["eligibility"]["holder"], expected["replacementRequestId"],
                           expected["replacementTarget"], expected["retirement"], expected.get("runtimeRoleReview"))
        _require(canonical_json(current) == self.snapshot, "Prelaunch restoration evidence changed", "prelaunch_upgrade_changed")
        return current

    def restore(self, manager):
        """Archive the honest unknown, then CAS only the original committed bytes."""
        _require(type(self) is PrelaunchUpgradeRecovery and self._token is _ISSUED and not self._used[0],
                 "A caller boolean or dictionary cannot authorize command restoration")
        state = manager.paths.state.absolute()
        with process_lock(state / "deployment-start.lock", reason="deployment_start_busy"):
            with process_lock(state / "recovery-lifecycle/owner.lock", reason="recovery_keeper_already_live"):
                with process_lock(state / "recovery-lifecycle-submit.lock", reason="recovery_keeper_request_busy"):
                    value = self.revalidate(manager)
                    self._used[0] = True
                    pins = value["eligibility"]
                    evidence = {"originalCommand": pins["originalCommand"], "holder": pins["holder"],
                        "release": pins["release"], "request": pins["request"], "ownedUpgradeHandoff": pins["ownedUpgradeHandoff"],
                        "initialUpgradePlan": pins["initialUpgradePlan"], "retirement": value["retirement"]}
                    evidence.update({"control-" + str(index): {"path": path, "sha256": checksum}
                                     for index, (path, checksum) in enumerate(pins["controlFlow"].items())})
                    archives = {}
                    for name, reference in evidence.items():
                        raw = upgrade._raw(Path(reference["path"]))
                        _require(_sha(raw) == reference["sha256"], "Protected prelaunch evidence changed before archive")
                        archive = recovery._archive_bytes(state, "prelaunch-upgrade-" + name, raw)
                        archives[name] = {"original": reference, "archive": archive}
                    intent = recovery._archive(state, "prelaunch-upgrade-restore-intent", {
                        **value, "kind": KIND, "outcome": "original-committed-restoration-authorized",
                        "originalCommandStatus": "unknown", "archivedEvidence": archives, **{name: False for name in _FALSE}})
                    gate_path = state / _GATE
                    recovery._write(gate_path, {"schemaVersion": 1, "kind": GATE_KIND, "status": "restoring",
                                               "intent": intent, "claimedRelease": None}, expected_sha=None)
                    restore_raw = upgrade._raw(Path(pins["restoredCommand"]["path"]))
                    command_path = state / "recovery-lifecycle/command.json"
                    _require(_sha(restore_raw) == pins["restoredCommand"]["sha256"]
                             and _sha(upgrade._raw(command_path)) == pins["originalCommand"]["sha256"],
                             "The exact command compensation CAS changed", "prelaunch_upgrade_changed")
                    temporary = command_path.with_name(command_path.name + ".prelaunch-" + uuid.uuid4().hex + ".tmp")
                    try:
                        with temporary.open("xb") as stream:
                            stream.write(restore_raw)
                            stream.flush()
                            os.fsync(stream.fileno())
                        _require(_sha(upgrade._raw(command_path)) == pins["originalCommand"]["sha256"],
                                 "The original unknown changed before compensation", "prelaunch_upgrade_changed")
                        replace_file(temporary, command_path)
                    finally:
                        if temporary.exists():
                            temporary.unlink()
                    _require(upgrade._raw(command_path) == restore_raw, "The original committed restoration is unresolved")
                    receipt = recovery._archive(state, "prelaunch-upgrade-restored", {
                        **value, "kind": KIND, "outcome": "original-committed-command-restored",
                        "intent": intent, "originalCommandStatus": "unknown", "archivedEvidence": archives,
                        "restoredCommand": _ref(command_path), **{name: False for name in _FALSE}})
                    gate_sha = _sha(upgrade._raw(gate_path))
                    recovery._write(gate_path, {"schemaVersion": 1, "kind": GATE_KIND, "status": "reserved",
                                               "reconciliation": receipt, "claimedRelease": None}, expected_sha=gate_sha)
                    return {"reconciliation": receipt, "restoredCommand": _ref(command_path),
                            "replacementRequestId": value["replacementRequestId"], "replacementTarget": value["replacementTarget"],
                            **({"runtimeRoleReview": value["runtimeRoleReview"]} if value.get("runtimeRoleReview") is not None else {}),
                            **{name: False for name in _FALSE}}


def _prepare(manager, command_sha, holder, request_id, target, retirement, runtime_role_review_ref=None):
    _require(isinstance(request_id, str) and len(request_id) == 32
             and all(char in "0123456789abcdef" for char in request_id), "A fresh exact replacement request ID is required")
    kwargs = {} if runtime_role_review_ref is None else {"runtime_role_review_ref": runtime_role_review_ref}
    pins, _, command, holder_doc, host = _capture(manager, command_sha, holder, **kwargs)
    _require(request_id != recovery._read(Path(command["release"]["path"]))["requestId"], "The failed request may not be replayed")
    upgrade._request_unused(manager.paths.state, request_id)
    _require(not (manager.paths.state / _GATE).exists(), "An existing restoration gate must be reconciled")
    _require(isinstance(target, dict) and target == verify_driver(Path(target.get("root", "")),
             expected_digest=target.get("driverDigest")), "The replacement target must be a complete verified sealed candidate")
    _require(Path(target["root"]) == manager.paths.repo / ".jenkins/runtime/drivers" / target["driverDigest"],
             "The replacement target escaped its canonical namespace")
    if runtime_role_review_ref is not None:
        from .runtime_roles import prepare_runtime_role_review
        review = prepare_runtime_role_review(manager, review_reference=runtime_role_review_ref)
        _require(review.document["reviewedTarget"] == target, "The runtime role review belongs to another replacement target")
    _retirement(manager.paths.state.absolute(), retirement, pins, command, holder_doc, host, manager=manager)
    return {"schemaVersion": 1, "kind": KIND, "eligibility": pins, "retirement": retirement,
            "replacementRequestId": request_id, "replacementTarget": target,
            **({"runtimeRoleReview": runtime_role_review_ref} if runtime_role_review_ref is not None else {}),
            **{name: False for name in _FALSE}}


def prepare_prelaunch_upgrade_recovery(manager, *, expected_command_sha256, holder_reference,
                                       replacement_request_id, replacement_target, retirement_reference,
                                       runtime_role_review_ref=None):
    """Read-only issuance. restore() is the sole compensating mutator."""
    value = _prepare(manager, require_digest(expected_command_sha256), holder_reference,
                     replacement_request_id, replacement_target, retirement_reference, runtime_role_review_ref)
    return PrelaunchUpgradeRecovery(canonical_json(value), _ISSUED)


def _receipt(state, reference):
    value, _ = upgrade._reference(state, reference)
    _require(value.get("schemaVersion") == 1 and value.get("kind") == KIND
             and value.get("outcome") == "original-committed-command-restored"
             and value.get("originalCommandStatus") == "unknown"
             and all(value.get(name) is False for name in _FALSE), "The restoration receipt is not an honest prelaunch compensation")
    pins = value["eligibility"]
    archives = value.get("archivedEvidence", {})
    required = {"originalCommand", "holder", "release", "request", "ownedUpgradeHandoff", "initialUpgradePlan", "retirement"}
    _require(required <= set(archives), "Every original prelaunch fault artifact must remain archived")
    for name, binding in archives.items():
        _require(set(binding) == {"original", "archive"} and binding["original"]["sha256"] == binding["archive"]["sha256"],
                 "Protected prelaunch bytes changed in their immutable archive")
        archive_path = Path(binding["archive"]["path"]).absolute()
        _require(str(archive_path) == binding["archive"]["path"] and archive_path.parent == state / "recovery-lifecycle/history"
                 and ".." not in archive_path.parts, "Protected prelaunch archive escaped its namespace")
        raw = upgrade._raw(archive_path)
        _require(_sha(raw) == binding["archive"]["sha256"] == binding["original"]["sha256"],
                 "Protected prelaunch byte archive changed")
    failed, _ = upgrade._reference(state, archives["originalCommand"]["archive"])
    _require(archives["originalCommand"]["original"] == pins["originalCommand"]
             and failed.get("status") == "unknown" and failed.get("reasonCode") == "owned_recovery_upgrade_unproven"
             and failed.get("request") == pins["request"] and failed.get("ownedUpgradeHandoff") == pins["ownedUpgradeHandoff"]
             and failed.get("keeper") == pins["keeper"] and failed.get("release") == pins["release"],
             "The original unknown upgrade was rewritten or detached")
    retired, _ = upgrade._reference(state, archives["retirement"]["archive"])
    _require(retired.get("kind") == RETIREMENT_KIND and retired.get("keeper") == pins["keeper"]
             and retired.get("originalCommand") == pins["originalCommand"]
             and retired.get("nativeHandleRetained") is True and retired.get("handleIdentityVerified") is True
             and type(retired.get("waitResult")) is int and retired["waitResult"] == 0
             and retired.get("processExitCode") == 196, "The original positive retained exit changed")
    control = {item["original"]["path"]: item["original"]["sha256"]
               for name, item in archives.items() if name.startswith("control-")}
    _require(control == pins["controlFlow"], "The complete reviewed pre-spawn control bytes are required")
    failed_release, _ = upgrade._reference(state, archives["release"]["archive"], release=True)
    original_request, _ = upgrade._reference(state, archives["request"]["archive"])
    _require(pins.get("originalRequestId") == failed_release.get("requestId") == original_request.get("requestId")
             and original_request.get("keeper") == pins["keeper"] and original_request.get("release") == pins["release"]
             and original_request.get("action") == "upgrade", "The original retirement request binding changed")
    sources = _REVIEWED_FAILURE_CODE.get(failed_release.get("targetDriver", {}).get("driverDigest"))
    _require(sources is not None and len(control) == len(sources), "The original fault implementation has no closed review")
    for name, checksum in sources.items():
        root = Path(failed_release["targetDriver"]["root"]) if name.startswith("tools/jenkins/") else state.parent.parent
        _require(control.get(str(root / name)) == checksum, "The archived fault control flow changed")
    return value


def validate_prelaunch_restoration_gate(manager, release_ref, release, *, request_id, descriptor_sha256, command_sha256):
    """Strict admission only for the one reserved request, target and release."""
    state = manager.paths.state.absolute()
    path = state / _GATE
    reference = release.get("prelaunchReconciliation")
    if not path.exists():
        _require(reference is None, "A declared restoration lacks its coordinator gate")
        return None
    gate = recovery._read(path)
    _require(gate.get("schemaVersion") == 1 and gate.get("kind") == GATE_KIND
             and gate.get("status") in {"reserved", "claimed"}, "An incomplete restoration must remain blocked")
    receipt = _receipt(state, gate["reconciliation"])
    pins = receipt["eligibility"]
    scoped = pins["rawAnchors"]["originalBlockedDescriptor"]["sha256"] == descriptor_sha256
    if not scoped:
        _require(reference is None, "A prelaunch restoration cannot authorize an unrelated stopped generation")
        return None
    _require(reference == gate["reconciliation"] and request_id == receipt["replacementRequestId"]
             and release.get("targetDriver") == receipt["replacementTarget"]
             and release.get("runtimeRoleReview") == receipt.get("runtimeRoleReview") == pins.get("runtimeRoleReview")
             and command_sha256 == receipt["restoredCommand"]["sha256"]
             and gate.get("claimedRelease") in (None, release_ref), "The original unknown may authorize only its one explicit successor")
    return receipt


def claim_prelaunch_restoration_gate(manager, plan, request, keeper):
    """Called under the deployment lock before the first real handoff CAS."""
    _require(type(plan) is upgrade.OwnedRecoveryUpgradePlan and plan._token is upgrade._ISSUED,
             "Only the actual issued owned upgrade may claim restoration")
    state = manager.paths.state.absolute()
    path = state / _GATE
    if not path.exists():
        return
    document = plan.document
    release, _ = upgrade._reference(state, document["successorRelease"], release=True)
    descriptor_key, command_key, _ = upgrade._transfer_keys(document)
    receipt = validate_prelaunch_restoration_gate(manager, document["successorRelease"], release,
        request_id=document["requestId"], descriptor_sha256=document["rawAnchors"][descriptor_key]["sha256"],
        command_sha256=document["rawAnchors"][command_key]["sha256"])
    if receipt is None:
        return
    body, _ = upgrade._reference(state, request)
    _require(keeper["pid"] == os.getpid() and recovery._full_identity(os.getpid()) == keeper
             and body.get("keeper") == keeper and body.get("release") == document["successorRelease"]
             and body.get("requestId") == document["requestId"] and body.get("action") == "upgrade"
             and body.get("ownedUpgradeDigest") == plan.upgrade_digest and body.get("runtimeAcceptance") is False,
             "Only the exact live successor keeper may claim its immutable request")
    gate = recovery._read(path)
    _require(gate.get("status") == "reserved" and gate.get("claimedRelease") is None,
             "This prelaunch restoration has already been consumed")
    checksum = _sha(upgrade._raw(path))
    recovery._write(path, {**gate, "status": "claimed", "claimedRelease": document["successorRelease"],
                          "request": request, "keeper": keeper}, expected_sha=checksum)


def validate_prelaunch_restoration_lineage(state, release, release_ref):
    reference = release.get("prelaunchReconciliation")
    if reference is None:
        return
    receipt = _receipt(state, reference)
    gate = recovery._read(state / _GATE)
    _require(gate.get("status") == "claimed" and gate.get("reconciliation") == reference
             and gate.get("claimedRelease") == release_ref
             and release.get("requestId") == receipt["replacementRequestId"]
             and release.get("targetDriver") == receipt["replacementTarget"]
             and release.get("runtimeRoleReview") == receipt.get("runtimeRoleReview"),
             "The running handoff lacks its exact consumed prelaunch restoration")
    return receipt


__all__ = ["PrelaunchUpgradeRecovery", "inspect_prelaunch_upgrade_recovery", "prepare_prelaunch_upgrade_recovery"]

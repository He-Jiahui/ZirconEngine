"""Read-only authority to transfer a genuinely stopped recovery deployment.

Preparation and revalidation run under the caller's deployment-start lock.
An owned Stop proves the service's NativeJobs; a separate keeper retirement
receipt proves only that keeper's exit.  Neither JSON nor PID disappearance
can issue this process-local capability.  This module performs no writes,
process management, admission changes, or native proof synthesis.
"""
from __future__ import annotations

import ast
import base64
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import sqlite3
import time
import xml.etree.ElementTree as ET

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..processes.boot import windows_boot_filetime
from .driver import _plain, verify_driver
from .execution_activation import protected_holds, verify_regression
from .drain_observations import _identity
from .lifecycle import check_port_available
from .recovery import has_terminal_lifecycle_proofs
from .startup_observations import observe_departed_identity, observe_home_available, runtime_owner_inventory
from . import lost_owner


KIND = "formal-owned-recovery-upgrade-plan"
RELEASE_KIND = "formal-owned-recovery-upgrade-release"
RETIREMENT_KIND = "owned-recovery-keeper-retirement"
HANDOFF_KIND = "owned-recovery-keeper-handoff"
EXTERNAL_VARIANT = "external-native-stop"
_ISSUED = object()
_FLAGS = ("nativeProofSynthesized", "historicalHoldsReleased", "runtimeAcceptance", "cargoAcceptance")
_OLD_RELEASE_KIND = "formal-lost-owner-recovery-release"
_DESCRIPTOR_KIND = "formal-lost-owner-recovery-keeper-descriptor"
_COMMAND_KIND = "formal-lost-owner-recovery-command"
_REQUEST_KIND = "formal-lost-owner-recovery-request"
_LEGACY = ("deployment/execution-activation.json", "deployment/execution-upgrade-selection.json",
           "deployment/execution-upgrade-start.json", "formal-lifecycle/keeper.json",
           "deployment/formal-lifecycle-command.json")
_ARTIFACTS = {"stoppedDescriptor": "formal-lifecycle/recovery-keeper.json",
              "terminalCommand": "recovery-lifecycle/command.json",
              "stoppedOperation": "deployment-operation.json",
              "stoppedHost": "deployment/host.json", "previousSelector": "deployment/driver.json"}


def _transfer_keys(value):
    variant = value.get("variant")
    _require(variant in (None, EXTERNAL_VARIANT), "Unknown owned transfer variant")
    return (("originalBlockedDescriptor", "committedCommand", "externalNativeStop")
            if variant == EXTERNAL_VARIANT else ("stoppedDescriptor", "terminalCommand", "ownedStop"))


def _require(condition, message, code="owned_recovery_upgrade_unproven"):
    if not condition:
        raise JenkinsError(code, message, retryable=True)


def _raw(path):
    path = Path(path).absolute()
    _plain(path)
    return path.read_bytes()


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _document(raw):
    def unique(pairs):
        value = {}
        for key, item in pairs:
            _require(key not in value, "Upgrade evidence contains duplicate keys")
            value[key] = item
        return value
    value = json.loads(raw, object_pairs_hook=unique, parse_constant=lambda _: (_ for _ in ()).throw(
        JenkinsError("owned_recovery_upgrade_unproven", "Upgrade evidence contains nonfinite values")))
    _require(isinstance(value, dict), "Upgrade evidence must be an object")
    return value


def _ref(path, raw):
    return {"path": str(path), "sha256": _sha(raw)}


def _reference(state, reference, *, release=False):
    _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"},
             "Upgrade evidence needs an exact immutable reference")
    checksum = require_digest(reference.get("sha256"), "upgradeArtifactSha256")
    path = Path(reference.get("path", "")).absolute()
    parents = {state / "migration-baseline", state / "deployment/history", state / "recovery-lifecycle/history"}
    if release:
        parents.add(state / "deployment")
    _require(str(path) == reference["path"] and ".." not in path.parts and path.parent in parents
             and path.suffix == ".json", "Upgrade evidence escaped its immutable namespace")
    raw = _raw(path)
    _require(_sha(raw) == checksum, "Immutable upgrade evidence changed", "owned_recovery_upgrade_changed")
    return _document(raw), raw


def _paths(manager):
    repo, state, home = (manager.paths.repo.absolute(), manager.paths.state.absolute(), manager.paths.home.absolute())
    for path in (repo, state, home):
        _plain(path)
    _require(state == repo / ".jenkins/state" and home == repo / ".jenkins/jenkins_home",
             "Owned upgrade requires the existing formal deployment")
    return repo, state, home


def _configuration(manager, release):
    repo = manager.paths.repo.absolute()
    for field, path in (("config", repo / ".codex/config.toml"), ("deploymentSpec", manager.spec.path.absolute()),
                        ("workflowSpec", repo / ".jenkins/workflow-spec.json")):
        _require(release.get(field) == _ref(path, _raw(path)), "Reviewed configuration or specification changed")


# Only scheduling leaves may differ while an earlier native owner is drained.
_DEPLOYMENT_SCHEDULING = {
    ("agent", "executors"),
    *(("storage", "resourcePolicy", name) for name in (
        "cpuBudget", "memoryBudget", "diskBudget", "maxHeavyWriters", "maxNewHeavyWriters",
        "configurationScope", "skipProvenRootDiskBlockedHeads", "postTaskCache")),
    ("storage", "postTaskCache"),
    *(("productionResourceDefaults", name) for name in (
        "cpuBudget", "maxHeavyWriters", "activateAfter", "historicalProtectedReservationsCountAgainstBudget")),
}
_WORKFLOW_SCHEDULING = {
    ("localResourcePolicy", "diskReservationAccounting"),
    ("localResourcePolicy", "singleHostAgent"),
    ("localResourcePolicy", "afterEveryTerminalTask"),
}
_CACHE_FIELDS = {"enabled", "everyTerminalTask", "minFreeBytes", "maxFiles", "maxBytes", "maxSeconds",
    "preserveWarmPreparation", "preserveAcceptedReferences", "unknownObjectsProtected",
    "automaticDeletionRequiresOriginalGcAuthorization"}
_CACHE_PROTECTIONS = {"enabled", "everyTerminalTask", "preserveWarmPreparation", "preserveAcceptedReferences",
    "unknownObjectsProtected", "automaticDeletionRequiresOriginalGcAuthorization"}
_TERMINAL_CACHE_FIELDS = {"assessFreeCapacityAndManagedCache", "cleanupOnlyNecessaryBoundedOwnedTerminalScratch",
    "preserveWarmPoolsAcceptedReferencesAndUnknownObjects", "recordActualReclaimedBytes",
    "cleanupFailureKeepsAcceptedSourceAndOriginalReceipt"}
_HISTORICAL_CONFIG_ISSUED = object()


def _scheduling_difference(before, after, allowed, path=()):
    if path in allowed:
        return
    _require(type(before) is type(after), "Historical operational configuration type changed")
    if isinstance(before, dict):
        keys = set(before) | set(after)
        for key in keys:
            child = (*path, key)
            if child in allowed:
                continue
            _require(key in before and key in after, "An undeclared operational configuration field changed")
            _scheduling_difference(before[key], after[key], allowed, child)
    else:
        _require(before == after, "Historical operational paths, ownership or authority changed")


def _positive(value, label, *, zero=False):
    _require(type(value) is int and value >= (0 if zero else 1), label + " must be an integer budget")
    return value


def _cache_policy(value):
    _require(isinstance(value, dict) and set(value) == _CACHE_FIELDS,
             "Historical upgrade cache policy has undeclared fields")
    _require(all(value[name] is True for name in _CACHE_PROTECTIONS),
             "Historical upgrade must retain cache ownership and reference protections")
    for name in _CACHE_FIELDS - _CACHE_PROTECTIONS:
        _positive(value[name], name)


def _bounded_configuration(old_spec, new_spec, old_workflow, new_workflow, holds):
    from .spec import _validate
    canonical = Path(new_spec["repositoryRoot"]) / ".jenkins/deployment-spec.json"
    _validate(old_spec, canonical)
    _validate(new_spec, canonical)
    _scheduling_difference(old_spec, new_spec, _DEPLOYMENT_SCHEDULING)
    _scheduling_difference(old_workflow, new_workflow, _WORKFLOW_SCHEDULING)
    executors = _positive(new_spec["agent"]["executors"], "agent executors")
    policy = new_spec["storage"]["resourcePolicy"]
    reservations = [row["payload"] for row in holds if row.get("domain") == "resource_reservation"]
    for budget, quantity in (("cpuBudget", "cpu"), ("memoryBudget", "memoryBytes"), ("diskBudget", "diskBytes")):
        minimum = sum(_positive(row.get(quantity), quantity, zero=True) for row in reservations)
        _require(_positive(policy[budget], budget) >= minimum, "Budget is below protected historical reservations")
    heavy = sum(row.get("heavyWriter") is True for row in reservations)
    writers = _positive(policy["maxHeavyWriters"], "heavy writers")
    new_writers = _positive(policy.get("maxNewHeavyWriters", max(1, writers - heavy)), "new heavy writers")
    _require(writers >= heavy + new_writers and executors >= new_writers,
             "Writer capacity must preserve historical reservations and fit the agent")
    if "skipProvenRootDiskBlockedHeads" in policy:
        _require(policy["skipProvenRootDiskBlockedHeads"] is True, "Only proven disk-blocked scheduling is permitted")
    for cache in (new_spec["storage"].get("postTaskCache"), policy.get("postTaskCache")):
        if cache is not None:
            _cache_policy(cache)
    if "postTaskCache" in policy:
        _require(policy["postTaskCache"] == new_spec["storage"].get("postTaskCache"),
                 "Deployment cache policies must agree")
    production = new_spec.get("productionResourceDefaults", {})
    for name in ("cpuBudget", "maxHeavyWriters"):
        if name in production:
            _positive(production[name], "production " + name)
    if "cpuBudget" in production:
        minimum_cpu = sum(row["cpu"] for row in reservations)
        _require(minimum_cpu <= production["cpuBudget"] <= policy["cpuBudget"],
                 "Production CPU budget must include protected reservations within the shared budget")
    if "maxHeavyWriters" in production:
        _require(production["maxHeavyWriters"] == new_writers,
                 "Production writer limit must match the bounded new writer capacity")
    if "historicalProtectedReservationsCountAgainstBudget" in production:
        _require(production["historicalProtectedReservationsCountAgainstBudget"] is True,
                 "Production must retain protected reservation accounting")
    local = new_workflow.get("localResourcePolicy", {})
    agent = local.get("singleHostAgent")
    if agent is not None:
        _require(isinstance(agent, dict) and set(agent) == {"name", "uniquePersistentNode", "executors",
            "newHeavyWriters", "protectedHistoricalReservationsCountAgainstAllBudgets"}
            and agent["name"] == new_spec["agent"]["name"] and agent["uniquePersistentNode"] is True
            and type(agent["executors"]) is int and agent["executors"] == executors
            and type(agent["newHeavyWriters"]) is int and agent["newHeavyWriters"] == new_writers
            and agent["protectedHistoricalReservationsCountAgainstAllBudgets"] is True,
            "Workflow scheduling changed the fixed agent or protected reservations")
    terminal = local.get("afterEveryTerminalTask")
    if terminal is not None:
        _require(isinstance(terminal, dict) and set(terminal) == _TERMINAL_CACHE_FIELDS
                 and all(value is True for value in terminal.values()),
                 "Workflow terminal cleanup must preserve ownership, accepted receipts and warm inputs")


@dataclass(frozen=True, init=False)
class _HistoricalExternalConfiguration:
    """Process-local, immutable configuration authority for one external Stop chain."""
    snapshot: bytes
    _token: object

    def __init__(self, snapshot, token=None):
        _require(token is _HISTORICAL_CONFIG_ISSUED and type(snapshot) is bytes,
                 "Historical configuration requires an issued external Stop context")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)

    @property
    def document(self):
        return _document(self.snapshot)

    def validate(self, manager, reference):
        _require(type(self) is _HistoricalExternalConfiguration and self._token is _HISTORICAL_CONFIG_ISSUED,
                 "A typed historical external Stop context is required")
        current = self.document
        _require(_configuration_references(manager) == current["currentConfiguration"],
                 "Configuration changed after historical Stop review", "owned_recovery_upgrade_changed")
        _require(protected_holds(manager.paths.state) == current["protectedHolds"],
                 "Protected historical holds changed")
        entries = [entry for entry in current["releases"] if entry["release"] == reference]
        _require(len(entries) == 1, "Historical context does not authorize this predecessor release")
        entry = entries[0]
        release, _ = _reference(manager.paths.state, reference, release=True)
        _require(release["config"] == current["currentConfiguration"]["config"],
                 "Historical restriction configuration changed")
        raw = {}
        for field in ("deploymentSpec", "workflowSpec"):
            archived, archived_raw = _reference(manager.paths.state, entry[field])
            _require(_sha(archived_raw) == release[field]["sha256"], "Archived historical specification changed")
            raw[field] = archived
        new_spec = _document(_raw(manager.paths.repo / ".jenkins/deployment-spec.json"))
        new_workflow = _document(_raw(manager.paths.repo / ".jenkins/workflow-spec.json"))
        _bounded_configuration(raw["deploymentSpec"], new_spec, raw["workflowSpec"], new_workflow,
                               current["protectedHolds"])
        return raw["deploymentSpec"]


def _configuration_references(manager):
    repo = manager.paths.repo.absolute()
    return {field: _ref(path, _raw(path)) for field, path in (
        ("config", repo / ".codex/config.toml"), ("deploymentSpec", repo / ".jenkins/deployment-spec.json"),
        ("workflowSpec", repo / ".jenkins/workflow-spec.json"))}


def _historical_configuration(manager, descriptor):
    """Issue a context from exact release ancestry; callers still prove the native Stop."""
    repo, state, _ = _paths(manager)
    current = _configuration_references(manager)
    holds = protected_holds(state)
    _require(len(holds) == 4 and descriptor.get("protectedHolds") == holds,
             "Historical Stop must retain the four original holds")
    releases, seen, reference = [], set(), descriptor.get("release")
    while reference is not None:
        release, raw = _reference(state, reference, release=True)
        _require(reference["sha256"] not in seen and len(seen) < 32, "Historical release ancestry is cyclic or unbounded")
        seen.add(reference["sha256"])
        _require(release.get("config") == current["config"] and release.get("protectedHolds") == holds,
                 "Historical restriction configuration or protected holds changed")
        entry = {"release": reference}
        for field, prefix, canonical in (("deploymentSpec", "spec", repo / ".jenkins/deployment-spec.json"),
                                         ("workflowSpec", "workflow", repo / ".jenkins/workflow-spec.json")):
            fact = release.get(field)
            _require(isinstance(fact, dict) and set(fact) == {"path", "sha256"} and fact["path"] == str(canonical),
                     "Historical specification is not its original canonical identity")
            checksum = require_digest(fact["sha256"], "historicalConfigurationSha256")
            archive = state / "deployment/history" / ("formal-lifecycle-" + prefix + "-" + checksum + ".json")
            archived_raw = _raw(archive)
            _require(_sha(archived_raw) == checksum, "Exact historical specification archive is required")
            entry[field] = _ref(archive, archived_raw)
        releases.append(entry)
        marker = release.get("ownedUpgrade")
        if marker is None:
            break
        _require(release.get("kind") == RELEASE_KIND and isinstance(marker, dict), "Historical release ancestry is malformed")
        key, _, _ = _transfer_keys(marker)
        previous, _ = _reference(state, marker[key])
        reference = previous.get("release")
    context = _HistoricalExternalConfiguration(canonical_json({"currentConfiguration": current,
        "protectedHolds": holds, "releases": releases}), _HISTORICAL_CONFIG_ISSUED)
    for entry in releases:
        context.validate(manager, entry["release"])
    return context


def _external_historical_configuration(manager, descriptor, native_stop_ref):
    # No dict, bool, PID disappearance or claim substitutes for this native binding.
    _external_native_stop(manager, descriptor, native_stop_ref)
    release, _ = _reference(manager.paths.state, descriptor["release"], release=True)
    if all(release.get(field) == fact for field, fact in _configuration_references(manager).items()):
        return None
    return _historical_configuration(manager, descriptor)


def _live_historical_operators(manager, release):
    # Only the current live owner's stable entry must still be its reviewed bytes.
    # Retired ancestors retain their immutable release and sealed implementation;
    # a completed handoff may legitimately have replaced their canonical launcher.
    repo = manager.paths.repo.absolute()
    for name, checksum in release["operatorFiles"].items():
        path = Path(name).absolute()
        if not path.is_relative_to(repo / "tools/jenkins"):
            _require(_sha(_raw(path)) == checksum, "Stable live historical operator bytes changed")


def _historical_closure(manager, release):
    repo = manager.paths.repo.absolute()
    runtime = release.get("runtimeFiles")
    _require(isinstance(runtime, dict) and str(manager.java.absolute()) in runtime and str(manager.war.absolute()) in runtime,
             "Historical runtime closure is incomplete")
    for name, checksum in runtime.items():
        path = Path(name).absolute()
        _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo / ".jenkins/runtime")
                 and _sha(_raw(path)) == require_digest(checksum, "historicalRuntimeSha256"),
                 "Historical installed runtime bytes changed")


@dataclass(frozen=True, init=False)
class HistoricalExternalStop:
    """Read-only preflight; authorizes one exact live owner, never its replacement."""
    snapshot: bytes
    _token: object
    _manager: object
    _configuration: object

    def __init__(self, snapshot, token=None, *, manager=None, configuration=None):
        _require(token is _HISTORICAL_CONFIG_ISSUED and type(snapshot) is bytes
                 and type(configuration) is _HistoricalExternalConfiguration,
                 "Only verified preparation may issue historical external Stop authority")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)
        object.__setattr__(self, "_manager", manager)
        object.__setattr__(self, "_configuration", configuration)

    @property
    def document(self):
        return _document(self.snapshot)

    def revalidate(self):
        _require(type(self) is HistoricalExternalStop and self._token is _HISTORICAL_CONFIG_ISSUED,
                 "An issued historical external Stop is required")
        previous = self.document
        current = prepare_historical_external_stop(self._manager,
            expected_driver_digest=previous["targetDriver"]["driverDigest"],
            expected_runtime_operation_id=previous["descriptor"]["runtime"]["operation"]["operationId"],
            expected_keeper_identity=previous["descriptor"]["keeper"])
        _require(current.snapshot == self.snapshot, "Historical Stop owner or configuration changed",
                 "owned_recovery_upgrade_changed")
        return current.document

    def native_manager(self):
        self.revalidate()
        from .manager import DeploymentManager
        from .paths import resolve_paths
        from .spec import DeploymentSpec
        document = self.document
        old_spec = self._configuration.validate(self._manager, document["release"])
        spec = DeploymentSpec(old_spec, self._manager.paths.repo / ".jenkins/deployment-spec.json")
        paths = resolve_paths(spec)
        _require(paths == self._manager.paths and spec.controller["java"]["executable"] == str(self._manager.java)
                 and spec.controller["warPath"] == str(self._manager.war), "Historical native layout changed")
        return DeploymentManager(spec, paths, self._manager.java, self._manager.war)


def prepare_historical_external_stop(manager, *, expected_driver_digest, expected_runtime_operation_id,
                                     expected_keeper_identity):
    """Observe current owned issuance using SHA-verified archival layout, without writes.

    The caller must retain deployment-start.lock from final revalidation through
    the native manager's existing _stop_locked path to fence a concurrent restart.
    """
    from . import recovery_keeper
    repo, state, home = _paths(manager)
    descriptor_path, command_path = (state / _ARTIFACTS[name] for name in ("stoppedDescriptor", "terminalCommand"))
    descriptor_raw, command_raw = _raw(descriptor_path), _raw(command_path)
    descriptor, command = _document(descriptor_raw), _document(command_raw)
    runtime = descriptor.get("runtime", {})
    _require(descriptor.get("kind") == _DESCRIPTOR_KIND and descriptor.get("admission") in {"idle", "blocked"}
             and runtime.get("state") == "running" and descriptor.get("keeper") == expected_keeper_identity
             and runtime.get("operation", {}).get("operationId") == expected_runtime_operation_id
             and runtime.get("selected", {}).get("driverDigest") == require_digest(expected_driver_digest, "expectedDriverDigest")
             and descriptor.get("automaticRestart") is False
             and all(descriptor.get(name) is False for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased"))
             and str(windows_boot_filetime()) == descriptor.get("bootFileTime"),
             "Historical Stop requires the exact current owned runtime")
    _require(recovery_keeper._full_identity(expected_keeper_identity["pid"]) == expected_keeper_identity,
             "Historical Stop keeper identity changed")
    _require(runtime["selected"].get("recoveryRelease") == descriptor["release"],
             "Selected runtime does not bind the original reviewed release")
    context = _historical_configuration(manager, descriptor)
    release, target = _historical_release(manager, descriptor["release"], historical_configuration=context)
    _live_historical_operators(manager, release)
    operation, host = runtime["operation"], runtime["host"]
    _require(operation.get("state") == "running" and operation.get("component") == "lifecycle-host"
             and operation.get("controlPlaneOnly") is False and host.get("controlPlaneOnly") is False
             and operation.get("homePath") == str(home) and host.get("status") == "running"
             and operation.get("generation") == operation["operationId"] == host.get("operationId") == host.get("generation")
             and operation.get("hostPid") == operation.get("pid") == host.get("hostPid")
             and host.get("agentLaunchAttempted") is True, "Historical Stop requires the full executing lifecycle owner")
    owner = {"supervisor": _identity(operation, repo / ".jenkins/runtime/python/python.exe"),
             "controller": _identity(host.get("controllerIdentity") or host.get("identity"), manager.java),
             "agent": _identity(host.get("agentIdentity"), manager.java)}
    _require(runtime.get("owner") == owner and len({identity["pid"] for identity in owner.values()}) == 3,
             "Historical Stop native owner identities conflict")
    for name in _LEGACY:
        path = state / name
        _require((_sha(_raw(path)) if path.exists() else None) == release["expectedLifecycleFence"].get(name),
                 "Historical keeper or acceptance lineage changed")
    origin = recovery_keeper._runtime_receipt(manager, descriptor, release, target, historical_configuration=context)
    _committed_command(state, descriptor, origin, command)
    recovery_keeper._keeper_argv(repo, descriptor["keeper"], descriptor["release"], release["config"]["sha256"])
    for name, expected in (("stoppedOperation", runtime["operation"]), ("stoppedHost", runtime["host"]),
                           ("previousSelector", runtime["selected"])):
        _require(_document(_raw(state / _ARTIFACTS[name])) == expected, "Historical Stop cannot adopt another owner")
    _external_delivery_inventory(state, historical_configuration=context)
    _require(_raw(descriptor_path) == descriptor_raw and _raw(command_path) == command_raw,
             "Historical Stop descriptor or command changed during review")
    value = {"schemaVersion": 1, "kind": "historical-external-native-stop-preflight",
        "descriptor": descriptor, "originalDescriptor": _ref(descriptor_path, descriptor_raw),
        "committedCommand": _ref(command_path, command_raw), "release": descriptor["release"], "targetDriver": target,
        "historicalConfiguration": context.document, **{name: False for name in _FLAGS}}
    return HistoricalExternalStop(canonical_json(value), _HISTORICAL_CONFIG_ISSUED,
                                  manager=manager, configuration=context)


def _sealed_target(repo, target):
    _require(isinstance(target, dict), "A complete sealed target is required")
    checksum = require_digest(target.get("driverDigest"), "upgradeDriverDigest")
    root = repo / ".jenkins/runtime/drivers" / checksum
    _require(target.get("root") == str(root), "Upgrade driver escaped its sealed namespace")
    verified = verify_driver(root, expected_digest=checksum)
    _require(target == verified, "Upgrade driver summary changed")
    return verified


def _historical_support(repo, state, release, target):
    """Read the earlier tested launch literal without executing old source."""
    receipt, _ = _reference(state, release.get("regression"))
    from .support_regression import SCOPED_SCOPE, verify_support_regression
    if receipt.get("scope") == SCOPED_SCOPE:
        receipt = verify_support_regression(repo, release["regression"], target, current=False, state_root=state,
            required_operators=("tools/jenkins/contracts/__init__.py", "tools/jenkins/deployment/driver.py",
                "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/recovery_keeper.py",
                "tools/jenkins/deployment/start_transaction.py", "tools/jenkins/processes/native.py",
                "tools/jenkins/processes/job.py"))
        for key, name in (("deploymentSpec", ".jenkins/deployment-spec.json"), ("workflowSpec", ".jenkins/workflow-spec.json")):
            _require(receipt["sourceFiles"][name] == release[key]["sha256"], "Historical tested specifications changed")
        return receipt
    _require(Path(release["regression"]["path"]).parent == state / "migration-baseline"
             and receipt.get("scope") == "complete-current-Jenkins-support-pytest-discovery"
             and receipt.get("status") == "passed" and receipt.get("sourceIdentityStable") is True
             and type(receipt.get("testsRun")) is int and receipt["testsRun"] > 0 and receipt.get("exitCode") == 0
             and receipt.get("failures") == 0 and receipt.get("errors") == 0 and receipt.get("collectionErrors") == [],
             "The exact complete historical support regression did not pass")
    log = Path(receipt.get("log", "")).absolute()
    _require(str(log) == receipt.get("log") and ".." not in log.parts and log.parent == state / "migration-baseline"
             and _sha(_raw(log)) == require_digest(receipt.get("logSha256"), "historicalUpgradeLogSha256"),
             "Historical support regression output changed")
    sources = receipt.get("sourceFiles")
    _require(isinstance(sources, dict) and sources, "Historical tested inventory is missing")
    expected, tests = {}, []
    for name, checksum in sources.items():
        relative = PurePosixPath(name)
        require_digest(checksum, "historicalUpgradeSourceSha256")
        _require(relative.as_posix() == name and not relative.is_absolute() and "\\" not in name and ":" not in name
                 and not any(part in {".", ".."} for part in relative.parts)
                 and ((name.startswith("tools/jenkins/") and name.endswith(".py"))
                      or (name.startswith(".jenkins/pipeline/") and name.endswith(".groovy"))
                      or name in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}),
                 "Historical inventory contains an undeclared source path")
        if name.startswith("tools/jenkins/") and "/tests/" in name:
            tests.append(name)
        elif name not in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}:
            expected[name] = checksum
    required = {"tools/jenkins/contracts/__init__.py", "tools/jenkins/deployment/driver.py",
                "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/recovery_keeper.py",
                "tools/jenkins/deployment/start_transaction.py", "tools/jenkins/processes/native.py",
                "tools/jenkins/processes/job.py", ".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}
    _require(tests and required <= set(sources), "Historical regression lacks its service containment inventory")
    driver_raw = _raw(Path(target["root"]) / "tools/jenkins/deployment/driver.py")
    _require(_sha(driver_raw) == sources["tools/jenkins/deployment/driver.py"], "Historical launcher source changed")
    tree = ast.parse(driver_raw)
    assignments = [node.value for node in tree.body if isinstance(node, ast.Assign)
                   and any(isinstance(name, ast.Name) and name.id == "_LAUNCHER" for name in node.targets)]
    _require(len(assignments) == 1 and isinstance(assignments[0], ast.Constant)
             and isinstance(assignments[0].value, str), "Historical launcher must have one tested literal")
    expected["launcher.py"] = _sha(assignments[0].value.encode("utf-8"))
    expected["tools/__init__.py"] = _sha(b'"""This package belongs to one verified Jenkins snapshot."""\n')
    manifest = _document(_raw(Path(target["manifest"])))
    _require(manifest.get("schemaVersion") == 1 and manifest.get("driverDigest") == target["driverDigest"]
             and isinstance(manifest.get("files"), dict), "Historical sealed manifest identity changed")
    observed = {".jenkins/" + name if name.startswith("pipeline/") else name: entry.get("sha256")
                for name, entry in manifest["files"].items()}
    _require(observed == expected, "Historical sealed driver differs from its full passed inventory")
    for key, name in (("deploymentSpec", ".jenkins/deployment-spec.json"), ("workflowSpec", ".jenkins/workflow-spec.json")):
        _require(sources[name] == release[key]["sha256"], "Historical tested specifications changed")
    return receipt


def _historical_release(manager, reference, *, historical_configuration=None):
    """Earlier passing inventories bind sealed bytes, not newer canonical files."""
    repo, state, _ = _paths(manager)
    release, _ = _reference(state, reference, release=True)
    _require(release.get("schemaVersion") == 1
             and release.get("kind") in {_OLD_RELEASE_KIND, RELEASE_KIND}
             and release.get("scope") == "execution-runtime"
             and release.get("automaticRestart") is False and release.get("queueReplayRequested") is False
             and release.get("runtimeAcceptance") is False and release.get("cargoAcceptance") is False,
             "The predecessor needs its complete reviewed release")
    if historical_configuration is None:
        _configuration(manager, release)
    else:
        _require(type(historical_configuration) is _HistoricalExternalConfiguration,
                 "A typed historical external Stop context is required")
        historical_configuration.validate(manager, reference)
    target = _sealed_target(repo, release.get("targetDriver"))
    historical = _historical_support(repo, state, release, target)
    operators = release.get("operatorFiles")
    _require(isinstance(operators, dict) and operators, "Predecessor operator inventory is missing")
    for name, checksum in operators.items():
        path = Path(name).absolute()
        require_digest(checksum, "previousOperatorSha256")
        _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo),
                 "Predecessor operator inventory is malformed")
        if path.is_relative_to(repo / "tools/jenkins"):
            _require(historical["sourceFiles"].get(path.relative_to(repo).as_posix()) == checksum,
                     "Predecessor operator differs from its passed sealed implementation")
    preparation, _ = _reference(state, release.get("preparation"))
    _require(preparation.get("schemaVersion") == 1 and preparation.get("state") == "prepared-not-selected"
             and preparation.get("driver") == target and preparation.get("protectedHolds") == release.get("protectedHolds")
             and preparation.get("supportRegression", preparation.get("regression")) == release.get("regression"),
             "Predecessor preparation changed")
    if historical_configuration is not None:
        _historical_closure(manager, release)
    return release, target


def _successor_release(manager, reference, *, request_id, marker, owner, fence, selected, holds, old_target, old_release):
    from . import recovery_keeper
    repo, state, _ = _paths(manager)
    release, _ = _reference(state, reference, release=True)
    _require(release.get("schemaVersion") == 1 and release.get("kind") == RELEASE_KIND
             and release.get("scope") == "execution-runtime" and release.get("requestId") == request_id
             and release.get("ownedUpgrade") == marker and release.get("failedOwnStart") is None
             and release.get("automaticRestart") is False and release.get("queueReplayRequested") is False
             and release.get("runtimeAcceptance") is False and release.get("cargoAcceptance") is False
             and release.get("expectedPreviousOwner") == owner and release.get("expectedLifecycleFence") == fence
             and release.get("protectedHolds") == holds,
             "Successor release is not bound to this exact owned transfer")
    _configuration(manager, release)
    target = _sealed_target(repo, release.get("targetDriver"))
    _require(target["driverDigest"] != old_target["driverDigest"], "An upgrade needs a distinct reviewed target")
    current = verify_regression(repo, release.get("regression", {}), target)
    historical = _historical_support(repo, state, release, target)
    _require(current == historical, "Successor regression differs from its complete sealed inventory")
    operators = release.get("operatorFiles")
    required = {str(repo / name) for name in (*recovery_keeper._OPERATOR_FILES,
                "tools/jenkins/deployment/owned_upgrade.py")}
    required.add(str(state / "migration-baseline/jenkins_desktop_holder.py"))
    _require(isinstance(operators, dict) and required <= set(operators), "Successor recovery operators are not fully pinned")
    for name, checksum in operators.items():
        path = Path(name).absolute()
        require_digest(checksum, "upgradeOperatorSha256")
        _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo)
                 and _sha(_raw(path)) == checksum, "Successor operator bytes changed")
        if path.is_relative_to(repo / "tools/jenkins"):
            _require(historical["sourceFiles"].get(path.relative_to(repo).as_posix()) == checksum,
                     "Successor operator differs from its complete passing regression")
    runtime = release.get("runtimeFiles")
    required_runtime = {str(repo / ".jenkins/runtime/python" / name) for name in
        ("python.exe", "pythonw.exe", "python3.dll", "python314.dll", "vcruntime140.dll", "vcruntime140_1.dll")}
    required_runtime |= {str(manager.java.absolute()), str(manager.war.absolute())}
    required_runtime |= {str(repo / ".jenkins/runtime/jdk" / name) for name in ("bin/server/jvm.dll", "lib/modules", "release")}
    _require(isinstance(runtime, dict) and required_runtime <= set(runtime)
             and runtime == old_release.get("runtimeFiles"), "Successor runtime closure must retain the reviewed installed runtime")
    for name, checksum in runtime.items():
        path = Path(name).absolute()
        require_digest(checksum, "upgradeRuntimeSha256")
        _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo / ".jenkins/runtime")
                 and _sha(_raw(path)) == checksum, "Reviewed runtime bytes changed")
    host = release.get("sandboxHost", {})
    path = Path(host.get("path", "")).absolute()
    _require(host == old_release.get("sandboxHost")
             and str(path) == host.get("path") and ".." not in path.parts and path.name == "codex.exe"
             and host.get("profile") == "zircon-strict" and host.get("includeManagedConfig") is True
             and _sha(_raw(path)) == require_digest(host.get("sha256"), "upgradeSandboxHostSha256"),
             "Successor must use its pinned official strict host")
    preparation, _ = _reference(state, release.get("preparation"))
    _require(preparation.get("schemaVersion") == 1 and preparation.get("state") == "prepared-not-selected"
             and preparation.get("driver") == target and preparation.get("protectedHolds") == holds
             and preparation.get("supportRegression", preparation.get("regression")) == release.get("regression")
             and preparation.get("previousSelectedDriver") == selected
             and preparation.get("previousSelectorSha256") == fence["deployment/driver.json"],
             "Successor preparation does not bind this exact stopped selector")
    return release, target


def _departure_record(value, identity):
    _require(isinstance(value, dict) and value.get("pid") == identity["pid"]
             and value.get("expectedCreationTime") == identity["creationTime"], "Departure identity changed")
    if value.get("status") == "absent":
        _require(set(value) == {"pid", "expectedCreationTime", "status"}, "Conflicting departure evidence")
    else:
        observed = value.get("observedIdentity", {})
        _require(value.get("status") == "pid-reused" and observed.get("pid") == identity["pid"]
                 and str(observed.get("creationTime", "")).isascii()
                 and str(observed.get("creationTime", "")).isdecimal()
                 and int(observed["creationTime"]) > int(identity["creationTime"])
                 and isinstance(observed.get("executable"), str) and bool(observed["executable"]),
                 "Live or inconclusive ownership cannot authorize transfer")
    return value


def _departure(identity):
    return _departure_record(observe_departed_identity(identity), identity)


def _owner(manager, operation, host, selected):
    repo, _, home = _paths(manager)
    runtime = operation.get("operationId")
    _require(operation.get("state") == "stopped" and host.get("status") == "stopped"
             and isinstance(runtime, str) and runtime and operation.get("generation") == runtime
             and host.get("operationId") == host.get("generation") == runtime
             and operation.get("component") == "lifecycle-host" and operation.get("homePath") == str(home)
             and operation.get("pid") == operation.get("hostPid") == host.get("hostPid")
             and operation.get("controlPlaneOnly") is False and host.get("controlPlaneOnly") is False
             and host.get("agentLaunchAttempted") is True and selected.get("runtimeOperationId") == runtime,
             "Only a fully stopped execution runtime can transfer ownership")
    identities = {"supervisor": _identity(operation, repo / ".jenkins/runtime/python/python.exe"),
                  "controller": _identity(host.get("controllerIdentity") or host.get("identity"), manager.java),
                  "agent": _identity(host.get("agentIdentity"), manager.java)}
    _require(host.get("identity", identities["controller"]) == identities["controller"]
             and len({identity["pid"] for identity in identities.values()}) == 3,
             "Stopped native identities conflict")
    _require(operation.get("nativeTerminationProof") == host.get("nativeTerminationProof")
             and operation.get("agentTerminationProof") == host.get("agentTerminationProof")
             and has_terminal_lifecycle_proofs(operation, host, str(home), _departure(identities["supervisor"])),
             "The genuine owned Stop lacks complete component NativeJob proofs")
    return {"operation": {key: operation[key] for key in
            ("operationId", "generation", "hostPid", "homePath", "component", "controlPlaneOnly")}, **identities}


def _stop_transport(state, stop, origin, operation, host):
    binding, _ = _reference(state, stop.get("stopBinding"))
    captured = binding.get("capturedOwner", {})
    _require(binding.get("schemaVersion") == 1 and binding.get("kind") == "owned-stop-metadata-binding"
             and captured.get("operation") == origin.get("runtime", {}).get("operation")
             and captured.get("host") == origin.get("runtime", {}).get("host")
             and captured.get("homePath") == operation["homePath"] and binding.get("terminalHost") == host
             and binding.get("nativeProofSynthesized") is False and binding.get("historicalHoldsReleased") is False
             and binding.get("runtimeAcceptance") is False and operation.get("stopBinding") == stop["stopBinding"],
             "The owned Stop transport has no exact captured owner")
    raw_result = binding.get("rawStopResult", {})
    result_path = Path(raw_result.get("path", "")).absolute()
    _require(str(result_path) == raw_result.get("path") and ".." not in result_path.parts
             and result_path.parent == state / "deployment/commands"
             and re.fullmatch(r"stop-[0-9]+\.result\.json", result_path.name), "Stop transport path changed")
    raw = _raw(result_path)
    archived = {"path": raw_result.get("archivePath"), "sha256": raw_result.get("sha256")}
    result, archived_raw = _reference(state, archived)
    _require(Path(archived["path"]).parent == state / "deployment/history" and raw == archived_raw
             and raw == base64.b64decode(raw_result.get("base64", ""), validate=True)
             and binding.get("terminalHostRawSha256") == _sha(raw) and operation.get("stopResult") == archived,
             "Actual stop result bytes or their immutable archive changed")
    command_path = result_path.with_name(result_path.name.replace(".result.json", ".json"))
    command_raw = _raw(command_path)
    command = _document(command_raw)
    _require(command == binding.get("command") and _sha(command_raw) == binding.get("commandSha256")
             and command.get("action") == "stop" and command.get("operationId") == operation["operationId"]
             and command.get("generation") == operation["generation"] and command.get("hostPid") == operation["hostPid"]
             and command.get("controllerIdentity") == host["controllerIdentity"]
             and result.get("status") == "stopped" and result.get("operationId") == operation["operationId"]
             and result.get("generation") == operation["generation"], "Stop command is not bound to its genuine result")
    _require(all(name not in result or result[name] == host[name] for name in
        ("hostPid", "controllerIdentity", "agentIdentity", "agentLaunchAttempted", "controlPlaneOnly")),
        "Raw Stop metadata conflicts with its exact captured native owner")
    projected = {**result, **{name: host[name] for name in
        ("hostPid", "controllerIdentity", "agentIdentity", "agentLaunchAttempted", "controlPlaneOnly")}}
    _require(projected == host and has_terminal_lifecycle_proofs(operation, host, operation["homePath"],
        _departure_record(binding.get("hostDeparture"), _identity(operation))), "Stop result projection changed")
    return {"binding": stop["stopBinding"], "command": _ref(command_path, command_raw), "rawResult": archived}


def _terminal_command(state, descriptor, stop, command):
    _require(command.get("schemaVersion") == 1 and command.get("kind") == _COMMAND_KIND
             and command.get("status") == "stopped" and command.get("keeper") == descriptor["keeper"]
             and command.get("release") == descriptor["release"] and command.get("request") == stop.get("request")
             and command.get("previousRuntimeReceipt") == stop.get("previousRuntimeReceipt")
             and command.get("recoveryDigest") == descriptor.get("recoveryDigest")
             and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False,
             "The old public command must have its genuine stopped outcome")
    request, _ = _reference(state, command.get("request"))
    request_id = request.get("requestId")
    _require(request.get("schemaVersion") == 1 and request.get("kind") == _REQUEST_KIND
             and request.get("action") == "stop" and re.fullmatch("[0-9a-f]{32}", str(request_id))
             and request.get("release") == descriptor["release"] and request.get("keeper") == descriptor["keeper"]
             and request.get("runtimeReceipt") == stop["previousRuntimeReceipt"]
             and request.get("protectedHoldsDigest") == digest(descriptor["protectedHolds"])
             and request.get("runtimeAcceptance") is False, "Public Stop request is not the old actor's exact request")
    origin, _ = _reference(state, stop["previousRuntimeReceipt"])
    running_descriptor = {**descriptor, "runtime": origin["runtime"], "runtimeReceipt": stop["previousRuntimeReceipt"]}
    _require(request.get("descriptorSha256") == _sha(canonical_json(running_descriptor) + b"\n"),
             "Public Stop request does not bind the exact preceding running descriptor")
    result, _ = _reference(state, command.get("result"))
    _require(result.get("schemaVersion") == 1 and result.get("kind") == "recovery-keeper-command-result"
             and result.get("status") == "stopped" and result.get("request") == command["request"]
             and result.get("release") == descriptor["release"] and result.get("keeper") == descriptor["keeper"]
             and result.get("runtimeReceipt") == descriptor["runtimeReceipt"]
             and result.get("operationId") == descriptor["runtime"]["operation"]["operationId"]
             and all(result.get(name) is False for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased")),
             "Public Stop command lacks its immutable complete result")
    for area, expected in (("requests", command["request"]), ("results", command["result"])):
        _require(_document(_raw(state / "recovery-lifecycle" / area / (request_id + ".json"))) == expected,
                 "Public Stop delivery marker changed")


def _external_native_stop(manager, descriptor, reference):
    """Check the public manager's actual Stop without relabelling its issuer."""
    _, state, home = _paths(manager)
    runtime = descriptor.get("runtime", {})
    binding, _ = _reference(state, reference)
    captured = binding.get("capturedOwner", {})
    _require(descriptor.get("admission") in {"idle", "blocked"} and runtime.get("state") == "running"
             and binding.get("kind") == "owned-stop-metadata-binding"
             and captured.get("operation") == runtime.get("operation")
             and captured.get("host") == runtime.get("host") and captured.get("homePath") == str(home),
             "External NativeStop must capture this exact blocked actor's running issuance")
    expected = [_identity(runtime["operation"]), _identity(runtime["host"]["controllerIdentity"]),
                _identity(runtime["host"]["agentIdentity"])]
    _require(captured.get("liveIdentities") == expected,
             "External Stop lacks the actual live birth, executable and command observations")
    host = binding.get("terminalHost", {})
    result = binding.get("rawStopResult", {})
    operation = {**runtime["operation"], "state": "stopped", "serviceReady": False, "health": False,
                 "nativeTerminationProof": host.get("nativeTerminationProof"),
                 "agentTerminationProof": host.get("agentTerminationProof"), "stopBinding": reference,
                 "stopResult": {"path": result.get("archivePath"), "sha256": result.get("sha256")}}
    selected = runtime.get("selected", {})
    owner = _owner(manager, operation, host, selected)
    _require(runtime.get("owner") == {key: owner[key] for key in ("supervisor", "controller", "agent")},
             "External Stop changed the physically created runtime owner")
    transport = _stop_transport(state, {"stopBinding": reference}, {"runtime": runtime}, operation, host)
    return {"operation": operation, "host": host, "owner": owner, "transport": transport}


def _committed_command(state, descriptor, origin, command):
    _require(command.get("schemaVersion") == 1 and command.get("kind") == _COMMAND_KIND
             and command.get("status") == "committed" and command.get("keeper") == descriptor["keeper"]
             and command.get("release") == descriptor["release"] and command.get("request") == origin.get("request")
             and command.get("previousRuntimeReceipt") == origin.get("previousRuntimeReceipt")
             and command.get("recoveryDigest") == descriptor.get("recoveryDigest")
             and command.get("ownedUpgradeDigest") == descriptor.get("ownedUpgradeDigest")
             and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False,
             "External transfer preserves the actor's exact committed command")
    request, _ = _reference(state, command.get("request"))
    result, _ = _reference(state, command.get("result"))
    request_id = request.get("requestId")
    _require(request.get("schemaVersion") == 1 and request.get("kind") == _REQUEST_KIND
             and re.fullmatch("[0-9a-f]{32}", str(request_id))
             and request.get("action") in {"recover", "upgrade", "start", "restart"}
             and request.get("keeper") == descriptor["keeper"] and request.get("release") == descriptor["release"]
             and request.get("runtimeAcceptance") is False
             and result.get("schemaVersion") == 1 and result.get("kind") == "recovery-keeper-command-result"
             and result.get("status") == "running-acceptance-pending" and result.get("request") == command["request"]
             and result.get("keeper") == descriptor["keeper"] and result.get("release") == descriptor["release"]
             and result.get("runtimeReceipt") == descriptor["runtimeReceipt"]
             and result.get("operationId") == descriptor["runtime"]["operation"]["operationId"]
             and all(result.get(name) is False for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased")),
             "Committed start lacks its original immutable runtime result")
    _require(_document(_raw(state / "recovery-lifecycle/results" / (request_id + ".json"))) == command["result"],
             "Committed result delivery marker changed")
    marker = state / "recovery-lifecycle/requests" / (request_id + ".json")
    if marker.exists():
        _require(_document(_raw(marker)) == command["request"], "Committed request marker changed")


def _external_delivery_inventory(state, *, historical_configuration=None):
    """Retain deliveries and admit only positively archived interruptions."""
    from .lost_recovery_runtime import interrupted_request
    inventory = {}
    requests = state / "recovery-lifecycle/requests"
    _plain(requests)
    for marker in sorted(requests.glob("*.json")):
        raw = _raw(marker)
        reference = _document(raw)
        request, _ = _reference(state, reference)
        _require(request.get("requestId") == marker.stem, "Original request marker changed")
        result_path = state / "recovery-lifecycle/results" / marker.name
        if result_path.exists():
            result_ref = _document(_raw(result_path))
            result, _ = _reference(state, result_ref)
            _require(result.get("kind") == "recovery-keeper-command-result" and result.get("request") == reference
                     and result.get("status") in {"stopped", "running-acceptance-pending"}
                     and result.get("keeper") == request.get("keeper") and result.get("release") == request.get("release")
                     and result.get("runtimeAcceptance") is False, "Historical request has a conflicting delivery result")
        else:
            interrupted = (interrupted_request(state, reference) if historical_configuration is None else
                interrupted_request(state, reference, historical_configuration=historical_configuration))
            _require(interrupted, "An unresolved request lacks exact interruption ancestry")
        inventory[str(marker)] = _sha(raw)
    results = state / "recovery-lifecycle/results"
    _plain(results)
    for marker in sorted(results.glob("*.json")):
        raw = _raw(marker)
        reference = _document(raw)
        result, _ = _reference(state, reference)
        request, _ = _reference(state, result.get("request"))
        _require(request.get("requestId") == marker.stem, "Original result marker changed its request identity")
        inventory[str(marker)] = _sha(raw)
    return inventory


def _retirement(manager, reference, *, request_id, descriptor, stopped_descriptor, owned_stop, boot, external=False):
    repo, state, _ = _paths(manager)
    value, _ = _reference(state, reference)
    keeper = _identity(descriptor["keeper"], repo / ".jenkins/runtime/python/python.exe")
    descriptor_key, _, stop_key = _transfer_keys({"variant": EXTERNAL_VARIANT} if external else {})
    _require((not external or value.get("variant") == EXTERNAL_VARIANT)
             and keeper == descriptor["keeper"] and value.get("schemaVersion") == 1 and value.get("kind") == RETIREMENT_KIND
             and value.get("requestId") == request_id and value.get("keeper") == keeper
             and value.get("release") == descriptor["release"] and value.get(descriptor_key) == stopped_descriptor
             and value.get(stop_key) == owned_stop and value.get("bootFileTime") == boot
             and value.get("beforeIdentity") == keeper and value.get("handleIdentity") ==
                 {key: keeper[key] for key in ("pid", "creationTime", "executable")}
             and value.get("handleIdentityVerified") is True and value.get("nativeHandleRetained") is True
             and type(value.get("waitResult")) is int and value["waitResult"] == 0
             and type(value.get("processExitCode")) is int and 0 <= value["processExitCode"] < 2 ** 32
             and value["processExitCode"] != 259
             and value.get("serviceStoppedBeforeRetirement") is True
             and all(value.get(name) is False for name in _FLAGS), "Keeper retirement lacks its exact held-handle evidence")
    args = [str(repo / ".jenkins/runtime/python/python.exe"), "-B", str(repo / "tools/dev/jenkins_recovery_keeper.py"),
            "--config-sha256", descriptor["configurationSha256"], "--release-file", descriptor["release"]["path"],
            "--release-sha256", descriptor["release"]["sha256"]]
    actual_args = value.get("beforeArguments")
    _require(actual_args in (args, [*args, "--no-tray"]) and isinstance(actual_args, list)
             and _sha("\0".join(actual_args).encode("utf-8")) == keeper["commandDigest"],
             "Retired keeper arguments do not bind the exact reviewed entry")
    if value.get("method") == "forced-terminate-held-handle":
        forced = value.get("terminateProcess", {})
        _require(value.get("forced") is True and value.get("graceful") is False
                 and forced.get("api") == "TerminateProcess" and forced.get("succeeded") is True
                 and type(forced.get("requestedExitCode")) is int and forced["requestedExitCode"] == value["processExitCode"],
                 "Forced keeper retirement must record the actual native termination method")
    else:
        signal = value.get("consoleSignal", {})
        group = _identity(signal.get("groupIdentity"))
        _require(value.get("method") == "graceful-console-break" and value.get("forced") is False
                 and value.get("graceful") is True and value["processExitCode"] == 0
                 and "terminateProcess" not in value
                 and signal.get("api") == "GenerateConsoleCtrlEvent" and signal.get("event") == "CTRL_BREAK_EVENT"
                 and signal.get("succeeded") is True and signal.get("groupVerified") is True
                 and isinstance(signal.get("memberPids"), list) and keeper["pid"] in signal["memberPids"]
                 and group["pid"] in signal["memberPids"], "Graceful retirement needs its real verified console group")
    _departure_record(value.get("afterDeparture"), keeper)
    _require(all(name not in value for name in ("nativeTerminationProof", "agentTerminationProof", "stdoutEof", "stderrEof")),
             "Keeper exit cannot manufacture service NativeJob termination or EOF")
    return value


def _request_unused(state, request_id):
    for area in ("requests", "results"):
        _require(not (state / "recovery-lifecycle" / area / (request_id + ".json")).exists(),
                 "A public upgrade request may be used only once", "owned_recovery_upgrade_replay")


def _replacement_command(state, transition, upgrade_digest):
    command = _document(_raw(state / "recovery-lifecycle/command.json"))
    descriptor_key, command_key, stop_key = _transfer_keys(transition)
    external = transition.get("variant") == EXTERNAL_VARIANT
    descriptor_field = "replacesOriginalBlockedDescriptor" if external else "replacesStoppedDescriptor"
    command_field = "replacesCommittedCommand" if external else "replacesTerminalCommand"
    _require(command.get("schemaVersion") == 1 and command.get("kind") == _COMMAND_KIND
             and command.get("status") in {"pending", "start-attempted"}
             and command.get("keeper") == transition["keeper"] and command.get("release") == transition["release"]
             and command.get("request") == transition["request"] and command.get("ownedUpgradeDigest") == upgrade_digest
             and command.get(descriptor_field) == transition[descriptor_key]
             and command.get(command_field) == transition[command_key]
             and (not external or command.get("variant") == EXTERNAL_VARIANT
                  and command.get(stop_key) == transition[stop_key])
             and command.get("ownedUpgradeHandoff") == transition["handoff"] and command.get("previousRuntimeReceipt") is None
             and command.get("runtimeAcceptance") is False and command.get("historicalHoldsReleased") is False
             and all(key not in command for key in ("result", "createdSupervisor", "failedOwnStartInterruption", "replacesOriginalCommand")),
             "Only the exact authorized successor command may replace the stopped command")
    return command


def _transition_evidence(manager, previous, transition, upgrade_digest, artifacts):
    """Recheck immutable handoff bytes and the actual new actor each time."""
    from . import recovery_keeper
    repo, state, _ = _paths(manager)
    keeper = _identity(transition["keeper"], repo / ".jenkins/runtime/python/python.exe")
    _require(keeper == transition["keeper"] and keeper["pid"] == os.getpid()
             and recovery_keeper._full_identity(keeper["pid"]) == keeper
             and transition["release"] == previous["successorRelease"],
             "The bound successor capability must be used by its actual keeper process")
    recovery_keeper._keeper_argv(repo, keeper, transition["release"], previous["binding"]["configuration"]["sha256"])
    request, _ = _reference(state, transition["request"])
    _require(request.get("schemaVersion") == 1 and request.get("kind") == _REQUEST_KIND
             and request.get("action") == "upgrade" and request.get("requestId") == previous["requestId"]
             and request.get("keeper") == keeper and request.get("release") == transition["release"]
             and request.get("ownedUpgradeDigest") == upgrade_digest
             and request.get("protectedHoldsDigest") == previous["executionQuarantine"]["protectedHoldsDigest"]
             and request.get("runtimeAcceptance") is False, "The immutable successor request changed")
    descriptor_key, command_key, stop_key = _transfer_keys(previous)
    _require(transition.get("variant") == previous.get("variant"), "Owned transfer variant changed")
    for name in (descriptor_key, command_key):
        _, raw = _reference(state, transition[name])
        _require(Path(transition[name]["path"]).parent == state / "recovery-lifecycle/history"
                 and raw == artifacts[name][1], "The exact stopped evidence archive changed")
    handoff, _ = _reference(state, transition["handoff"])
    _require(handoff.get("schemaVersion") == 1 and handoff.get("kind") == HANDOFF_KIND
             and handoff.get("outcome") == "owned-transfer" and handoff.get("ownedUpgradeDigest") == upgrade_digest
             and handoff.get("keeper") == keeper and handoff.get("release") == transition["release"]
             and handoff.get("request") == transition["request"] and handoff.get("previousKeeper") == previous["previousKeeper"]
             and handoff.get("previousRelease") == previous["previousRelease"]
             and handoff.get("variant") == previous.get("variant")
             and all(handoff.get(name) == transition[name] for name in (descriptor_key, command_key))
             and handoff.get(stop_key) == previous[stop_key] and handoff.get("keeperRetirement") == previous["keeperRetirement"]
             and all(handoff.get(name) is False for name in _FLAGS), "The immutable owned handoff changed")


def _capture(manager, *, request_id, successor_release_ref, stopped_descriptor_ref, owned_stop_ref,
             keeper_retirement_ref, terminal_command_ref=None, transition=None, upgrade_digest=None,
             external_native_stop_ref=None):
    from . import recovery_keeper
    repo, state, home = _paths(manager)
    _require(isinstance(request_id, str) and re.fullmatch("[0-9a-f]{32}", request_id), "An explicit unique upgrade request is required")
    _request_unused(state, request_id)
    boot = str(windows_boot_filetime())
    external = external_native_stop_ref is not None
    variant = {"variant": EXTERNAL_VARIANT} if external else {}
    descriptor_key, command_key, stop_key = _transfer_keys(variant)
    descriptor, descriptor_raw = _reference(state, stopped_descriptor_ref)
    _require(_raw(state / _ARTIFACTS["stoppedDescriptor"]) == descriptor_raw, "Current stopped descriptor changed")
    _require(descriptor.get("schemaVersion") == 1 and descriptor.get("kind") == _DESCRIPTOR_KIND
             and (descriptor.get("admission") in {"idle", "blocked"} if external else descriptor.get("admission") == "idle")
             and descriptor.get("runtime", {}).get("state") == ("running" if external else "stopped")
             and descriptor.get("automaticRestart") is False and descriptor.get("bootFileTime") == boot
             and all(descriptor.get(name) is False for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased")),
             "The exact idle stopped recovery keeper is required")
    historical_configuration = (_external_historical_configuration(manager, descriptor, external_native_stop_ref)
                                if external else None)
    historical_kwargs = ({} if historical_configuration is None else
                         {"historical_configuration": historical_configuration})
    old_release, old_target = _historical_release(manager, descriptor.get("release"), **historical_kwargs)
    _require((external or descriptor.get("runtimeReceipt") == owned_stop_ref)
             and descriptor.get("protectedHolds") == old_release["protectedHolds"],
             "Stopped descriptor differs from its exact public owned Stop")
    runtime = descriptor["runtime"]
    artifacts = {descriptor_key: (_ref(state / _ARTIFACTS["stoppedDescriptor"], descriptor_raw), descriptor_raw)}
    for name in ("stoppedOperation", "stoppedHost", "previousSelector"):
        path = state / _ARTIFACTS[name]
        raw = _raw(path)
        artifacts[name] = (_ref(path, raw), raw)
    operation, host, selected = (_document(artifacts[name][1]) for name in
                                 ("stoppedOperation", "stoppedHost", "previousSelector"))
    external_stop = _external_native_stop(manager, descriptor, external_native_stop_ref) if external else None
    _require(operation == (external_stop["operation"] if external else runtime.get("operation"))
             and host == (external_stop["host"] if external else runtime.get("host")) and selected == runtime.get("selected")
             and all(selected.get(key) == value for key, value in old_target.items()), "Stopped runtime anchors changed")
    owner = _owner(manager, operation, host, selected)
    _require(runtime.get("owner") == {key: owner[key] for key in ("supervisor", "controller", "agent")},
             "Stopped descriptor identities changed")
    if external:
        _require(owned_stop_ref == external_native_stop_ref and terminal_command_ref is not None,
                 "External NativeStop and exact committed command references are required")
        origin = recovery_keeper._runtime_receipt(manager, descriptor, old_release, old_target,
                                                 external_native_stop_ref=external_native_stop_ref, **historical_kwargs)
        stop, transport = origin, external_stop["transport"]
    else:
        stop, _ = _reference(state, owned_stop_ref)
        _require(recovery_keeper._runtime_receipt(manager, descriptor, old_release, old_target) == stop,
                 "The old public NativeJob receipt chain changed")
        origin, _ = _reference(state, stop.get("previousRuntimeReceipt"))
        transport = _stop_transport(state, stop, origin, operation, host)
    if transition is None:
        raw = _raw(state / _ARTIFACTS["terminalCommand"])
        command_ref = _ref(state / _ARTIFACTS["terminalCommand"], raw)
        if terminal_command_ref is not None:
            if external:
                _, original_raw = _reference(state, terminal_command_ref)
                _require(raw == original_raw, "Original committed command bytes changed")
                command_ref = terminal_command_ref
            else:
                _require(command_ref == terminal_command_ref, "Old terminal command changed")
    else:
        _replacement_command(state, transition, upgrade_digest)
        _, raw = _reference(state, transition[command_key])
        command_ref = terminal_command_ref
        _require(isinstance(command_ref, dict) and command_ref.get("sha256") == _sha(raw), "Old command archive changed")
    command = _document(raw)
    artifacts[command_key] = (_ref(state / _ARTIFACTS["terminalCommand"], raw) if external else command_ref, raw)
    command_validator = _committed_command if external else _terminal_command
    command_validator(state, descriptor, stop, command)
    deliveries = _external_delivery_inventory(state, **historical_kwargs) if external else None
    fence = lost_owner._lifecycle_fence(manager)
    _require(all(fence.get(name) == old_release.get("expectedLifecycleFence", {}).get(name) for name in _LEGACY),
             "Historical owner journals must retain their exact original binding")
    successor_claim, _ = _reference(state, successor_release_ref, release=True)
    failed_start = successor_claim.get("failedStartReconciliation") is not None
    if failed_start:
        from .failed_upgrade_recovery import validate_failed_upgrade_restoration_gate as validate_restoration_gate
    else:
        from .prelaunch_upgrade_recovery import validate_prelaunch_restoration_gate as validate_restoration_gate
    reconciliation = validate_restoration_gate(manager, successor_release_ref, successor_claim, request_id=request_id,
        descriptor_sha256=artifacts[descriptor_key][0]["sha256"], command_sha256=artifacts[command_key][0]["sha256"])
    retirement_request_id = request_id if reconciliation is None else reconciliation["eligibility"][
        "predecessorRetirementRequestId" if failed_start else "originalRequestId"]
    retirement_descriptor = {**descriptor, "configurationSha256": old_release["config"]["sha256"]}
    retirement = _retirement(manager, keeper_retirement_ref, request_id=retirement_request_id, descriptor=retirement_descriptor,
                             stopped_descriptor=stopped_descriptor_ref, owned_stop=owned_stop_ref, boot=boot, external=external)
    keeper = _identity(descriptor["keeper"], repo / ".jenkins/runtime/python/python.exe")
    identities = {"keeper": keeper, **{key: owner[key] for key in ("supervisor", "controller", "agent")}}
    _require(len({value["pid"] for value in identities.values()}) == 4
             and all(int(value["creationTime"]) >= int(boot) for value in identities.values()),
             "Owned retirement requires exact distinct identities in the same real boot")
    departures = {key: _departure(value) for key, value in identities.items()}
    runtime_review_ref = successor_claim.get("runtimeRoleReview")
    runtime_review = None
    if runtime_review_ref is not None:
        from .runtime_roles import prepare_runtime_role_review
        runtime_review = prepare_runtime_role_review(manager, review_reference=runtime_review_ref)
        _require(runtime_review.document["reviewedTarget"] == successor_claim.get("targetDriver"),
                 "The foreign runtime review belongs to another successor target")
    def owner_inventory():
        if runtime_review is None:
            return runtime_owner_inventory(repo)
        observation = runtime_review.observe(manager)
        _require(observation["blockingOwners"] == [], "Another repository runtime still owns the deployment")
        # The full original review inventory remains immutable evidence.
        # Every current inventory is freshly checked; natural client departure
        # cannot adopt a replacement or rewrite the reviewed initial snapshot.
        return {"review": runtime_review_ref, "blockingOwners": [],
                "originalRuntimeInventory": runtime_review.document["originalRuntimeInventory"],
                "reviewedClients": runtime_review.document["clients"]}
    inventory = owner_inventory()
    _require(inventory == [] or runtime_review is not None, "Another repository runtime still owns the deployment")
    home_policy = lost_owner._home_policy(home, "execution-runtime")
    home_observation = observe_home_available(home)
    _require(home_observation.get("staleMarkersDeleted") is False, "Home probes may not erase ownership evidence")
    endpoint = {"address": manager.spec.controller["listenAddress"], "port": int(manager.spec.controller["httpPort"])}
    check_port_available(endpoint["address"], endpoint["port"])
    from types import SimpleNamespace
    execution = lost_owner._execution_gate(manager, {"owner": owner, "selected": selected,
        "anchor": SimpleNamespace(value={"protectedHolds": old_release["protectedHolds"]})})
    holds = protected_holds(state)
    _require(len(holds) == 4 and holds == old_release["protectedHolds"] == descriptor["protectedHolds"]
             == execution["protectedHolds"] and execution["classification"]["activeExecutions"] == [],
             "All four historical holds and existing native quarantine must remain exact")
    marker = {**variant, descriptor_key: stopped_descriptor_ref, stop_key: owned_stop_ref,
              "keeperRetirement": keeper_retirement_ref}
    if external:
        marker[command_key] = command_ref
    successor, target = _successor_release(manager, successor_release_ref, request_id=request_id, marker=marker,
        owner=owner, fence=fence, selected=selected, holds=holds, old_target=old_target, old_release=old_release)
    for key, (reference, original_raw) in artifacts.items():
        if transition is not None and key == command_key:
            _replacement_command(state, transition, upgrade_digest)
        else:
            _require(_raw(Path(reference["path"])) == original_raw, "Current upgrade anchor changed during observation")
    _require(str(windows_boot_filetime()) == boot and owner_inventory() == inventory
             and lost_owner._lifecycle_fence(manager) == fence, "Boot, process inventory, or lifecycle fence changed")
    _require({key: _departure(value) for key, value in identities.items()} == departures, "Departed identity observation changed")
    checked = (recovery_keeper._runtime_receipt(manager, descriptor, old_release, old_target,
                   external_native_stop_ref=external_native_stop_ref, **historical_kwargs) if external
               else recovery_keeper._runtime_receipt(manager, descriptor, old_release, old_target))
    _require(checked == stop and (_external_native_stop(manager, descriptor, external_native_stop_ref)["transport"] if external
             else _stop_transport(state, stop, origin, operation, host)) == transport,
             "The genuine old Stop chain changed during observation")
    command_validator(state, descriptor, stop, command)
    if external:
        _require(_external_delivery_inventory(state, **historical_kwargs) == deliveries,
                 "Historical request deliveries changed during transfer")
    _require(_retirement(manager, keeper_retirement_ref, request_id=retirement_request_id, descriptor=retirement_descriptor,
        stopped_descriptor=stopped_descriptor_ref, owned_stop=owned_stop_ref, boot=boot, external=external) == retirement,
        "Keeper retirement changed during observation")
    _reference(state, descriptor["release"], release=True)
    _reference(state, successor_release_ref, release=True)
    value = {"schemaVersion": 1, "kind": KIND, "requestId": request_id, "scope": "execution-runtime",
             "outcome": "owned-transfer", "owner": owner, "previousKeeper": keeper,
             "previousRelease": descriptor["release"], "successorRelease": successor_release_ref,
             **variant, descriptor_key: stopped_descriptor_ref, "descriptorSha256": _sha(descriptor_raw),
             command_key: command_ref, "ownedStop": owned_stop_ref, "keeperRetirement": keeper_retirement_ref,
             "retirementMethod": retirement["method"], "bootFileTime": boot, "lifecycleFence": fence,
             "operationDigest": digest(operation), "hostRecordDigest": digest(host), "selectedDriverDigest": digest(selected),
             "binding": {"repository": str(repo), "homePath": str(home), "endpoint": endpoint,
                 "configuration": successor["config"], "deploymentSpec": successor["deploymentSpec"],
                 "workflowSpec": successor["workflowSpec"]}, "previousTarget": old_target, "targetDriver": target,
             "departedIdentities": departures, "runtimeOwnerInventory": inventory, "homePolicy": home_policy,
             "homeObservation": home_observation, "stopTransport": transport, "executionQuarantine": execution,
             "rawAnchors": {key: reference for key, (reference, _) in artifacts.items()},
             **{key: False for key in (*_FLAGS, "nativeProofIssued", "executionHoldsReleaseAllowed", "automaticRetry")}}
    if runtime_review_ref is not None:
        value["runtimeRoleReview"] = runtime_review_ref
    if external:
        value[stop_key] = external_native_stop_ref
        value["deliveryInventory"] = deliveries
    validate_restoration_gate(manager, successor_release_ref, successor, request_id=request_id,
        descriptor_sha256=artifacts[descriptor_key][0]["sha256"], command_sha256=artifacts[command_key][0]["sha256"])
    if transition is not None:
        _transition_evidence(manager, value, transition, upgrade_digest, artifacts)
    return value, tuple((key, canonical_json(reference), raw) for key, (reference, raw) in sorted(artifacts.items()))


@dataclass(frozen=True, init=False)
class OwnedRecoveryUpgradePlan:
    """Frozen, exact-type authority with private process-local provenance."""
    snapshot: bytes
    _token: object
    _artifacts: tuple
    _transition: bytes | None

    def __init__(self, snapshot, token=None, *, artifacts=(), transition=None):
        _require(token is _ISSUED and type(snapshot) is bytes and type(artifacts) is tuple
                 and (transition is None or type(transition) is bytes), "Only read-only preparation may issue upgrade authority")
        object.__setattr__(self, "snapshot", snapshot)
        object.__setattr__(self, "_token", token)
        object.__setattr__(self, "_artifacts", artifacts)
        object.__setattr__(self, "_transition", transition)

    @property
    def document(self):
        return _document(self.snapshot)

    @property
    def upgrade_digest(self):
        return _sha(self.snapshot)

    @property
    def original_artifacts(self):
        _require(type(self) is OwnedRecoveryUpgradePlan and self._token is _ISSUED, "Typed issued upgrade authority is required")
        return {name: {"reference": _document(reference), "raw": raw} for name, reference, raw in self._artifacts}

    def authorize_keeper_command(self, manager, *, keeper, release_ref, request, descriptor_archive, command_archive, handoff_ref):
        """Bind exact raw archives before the actor's command-pointer CAS.

        The returned capability permits only its new pending/start-attempted
        command.  Its immutable evidence and digest remain the original review.
        The stopped descriptor is still required byte for byte until startup.
        """
        _require(type(self) is OwnedRecoveryUpgradePlan and self._token is _ISSUED and self._transition is None,
                 "Only a fresh typed upgrade capability may bind a successor command")
        self.revalidate(manager)
        previous, state, repo = self.document, manager.paths.state.absolute(), manager.paths.repo.absolute()
        issued_keeper = _identity(keeper, repo / ".jenkins/runtime/python/python.exe")
        _require(issued_keeper == keeper and keeper != previous["previousKeeper"]
                 and int(keeper["creationTime"]) > max(int(value["expectedCreationTime"]) for value in previous["departedIdentities"].values())
                 and release_ref == previous["successorRelease"], "Successor keeper or release differs from the reviewed transfer")
        request_body, _ = _reference(state, request)
        _require(request_body.get("schemaVersion") == 1 and request_body.get("kind") == _REQUEST_KIND
                 and request_body.get("action") == "upgrade" and request_body.get("requestId") == previous["requestId"]
                 and request_body.get("keeper") == keeper and request_body.get("release") == release_ref
                 and request_body.get("ownedUpgradeDigest") == self.upgrade_digest
                 and request_body.get("protectedHoldsDigest") == previous["executionQuarantine"]["protectedHoldsDigest"]
                 and request_body.get("runtimeAcceptance") is False, "Successor request is not the exact reviewed owned upgrade")
        descriptor_key, command_key, stop_key = _transfer_keys(previous)
        external = previous.get("variant") == EXTERNAL_VARIANT
        archives = {descriptor_key: descriptor_archive, command_key: command_archive}
        for key, reference in archives.items():
            _, raw = _reference(state, reference)
            _require(Path(reference["path"]).parent == state / "recovery-lifecycle/history"
                     and raw == self.original_artifacts[key]["raw"], "Successor must archive exact stopped descriptor and command bytes")
        handoff, _ = _reference(state, handoff_ref)
        _require(handoff.get("schemaVersion") == 1 and handoff.get("kind") == HANDOFF_KIND
                 and handoff.get("outcome") == "owned-transfer" and handoff.get("ownedUpgradeDigest") == self.upgrade_digest
                 and handoff.get("keeper") == keeper and handoff.get("release") == release_ref and handoff.get("request") == request
                 and handoff.get("previousKeeper") == previous["previousKeeper"]
                 and handoff.get("previousRelease") == previous["previousRelease"]
                 and all(handoff.get(key) == reference for key, reference in archives.items())
                 and handoff.get("variant") == previous.get("variant") and handoff.get(stop_key) == previous[stop_key]
                 and handoff.get("keeperRetirement") == previous["keeperRetirement"]
                 and all(handoff.get(key) is False for key in _FLAGS), "Owned transfer requires its exact honest handoff receipt")
        transition = {"keeper": keeper, "release": release_ref, "request": request, **archives, "handoff": handoff_ref}
        if external:
            transition.update(variant=EXTERNAL_VARIANT, externalNativeStop=previous[stop_key])
        _transition_evidence(manager, previous, transition, self.upgrade_digest,
            {key: (_document(reference), raw) for key, reference, raw in self._artifacts})
        return OwnedRecoveryUpgradePlan(self.snapshot, _ISSUED, artifacts=self._artifacts, transition=canonical_json(transition))

    def revalidate(self, manager):
        _require(type(self) is OwnedRecoveryUpgradePlan and self._token is _ISSUED, "A typed issued owned-upgrade capability is required")
        previous = self.document
        descriptor_key, command_key, stop_key = _transfer_keys(previous)
        try:
            current, artifacts = _capture(manager, request_id=previous["requestId"], successor_release_ref=previous["successorRelease"],
                stopped_descriptor_ref=previous[descriptor_key], owned_stop_ref=previous[stop_key],
                keeper_retirement_ref=previous["keeperRetirement"], terminal_command_ref=previous[command_key],
                external_native_stop_ref=previous.get("externalNativeStop"),
                transition=_document(self._transition) if self._transition is not None else None, upgrade_digest=self.upgrade_digest)
        except JenkinsError:
            raise
        except (OSError, ValueError, TypeError, KeyError, AttributeError, SyntaxError, sqlite3.Error, ET.ParseError) as error:
            raise JenkinsError("owned_recovery_upgrade_unproven", "Owned upgrade evidence is unavailable", retryable=True) from error
        _require(canonical_json(current) == self.snapshot and artifacts == self._artifacts,
                 "Owned upgrade evidence changed before the pointer transaction", "owned_recovery_upgrade_changed")
        return {**current, "ownedUpgradeDigest": self.upgrade_digest, "observedAt": time.time()}


def prepare_owned_upgrade_start(manager, *, request_id, successor_release_ref, stopped_descriptor_ref, owned_stop_ref, keeper_retirement_ref):
    """Review exact stopped ownership under the caller's retained start lock."""
    try:
        value, artifacts = _capture(manager, request_id=request_id, successor_release_ref=successor_release_ref,
            stopped_descriptor_ref=stopped_descriptor_ref, owned_stop_ref=owned_stop_ref, keeper_retirement_ref=keeper_retirement_ref)
    except JenkinsError:
        raise
    except (OSError, ValueError, TypeError, KeyError, AttributeError, SyntaxError, sqlite3.Error, ET.ParseError) as error:
        raise JenkinsError("owned_recovery_upgrade_unproven", "Exact owned Stop and keeper retirement could not be verified", retryable=True) from error
    return OwnedRecoveryUpgradePlan(canonical_json(value), _ISSUED, artifacts=artifacts)


def prepare_external_native_stop_upgrade_start(manager, *, request_id, successor_release_ref,
        original_blocked_descriptor_ref, committed_command_ref, external_native_stop_ref, keeper_retirement_ref):
    """Issue a transfer only after a real public NativeStop and keeper retirement.

    The predecessor retains its original idle/blocked running descriptor. Its
    committed command remains committed. This distinct authority records the
    manager's actual Stop binding, never a Stop performed by that blocked actor.
    """
    try:
        value, artifacts = _capture(manager, request_id=request_id, successor_release_ref=successor_release_ref,
            stopped_descriptor_ref=original_blocked_descriptor_ref, owned_stop_ref=external_native_stop_ref,
            keeper_retirement_ref=keeper_retirement_ref, terminal_command_ref=committed_command_ref,
            external_native_stop_ref=external_native_stop_ref)
    except JenkinsError:
        raise
    except (OSError, ValueError, TypeError, KeyError, AttributeError, SyntaxError, sqlite3.Error, ET.ParseError) as error:
        raise JenkinsError("owned_recovery_upgrade_unproven", "External NativeStop and exact retirement are unproven", retryable=True) from error
    return OwnedRecoveryUpgradePlan(canonical_json(value), _ISSUED, artifacts=artifacts)

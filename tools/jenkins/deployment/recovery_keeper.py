"""Independent restricted keeper for an explicitly reviewed lost-owner recovery.

Recovery is an interruption, never an old NativeJob termination or acceptance.
Only this retained actor may create a supervisor in its existing outer Job.
Unknown commands remain blocked with their original request and evidence.
"""
from __future__ import annotations

import ast
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import time
import uuid

from ..contracts import JenkinsError, canonical_json, digest, require_digest
from ..processes.identity import current_identity
from ..processes.boot import windows_boot_filetime
from ..state.locks import process_lock
from .driver import _LAUNCHER, _plain, verify_driver
from .execution_activation import protected_holds, verify_regression
from .keeper_control import _write
from .live_observations import _windows_command_line, _digest_args, observe_live_identity, observe_endpoint_owner
from .lost_owner import LostFormalOwnerRecovery
from .lost_recovery_runtime import LostRecoveryRuntimeInterruption
from .recovery import has_terminal_lifecycle_proofs
from .startup_observations import observe_departed_identity

RELEASE_KIND = "formal-lost-owner-recovery-release"
OWNED_UPGRADE_RELEASE_KIND = "formal-owned-recovery-upgrade-release"
DESCRIPTOR_KIND = "formal-lost-owner-recovery-keeper-descriptor"
COMMAND_KIND = "formal-lost-owner-recovery-command"
REQUEST_KIND = "formal-lost-owner-recovery-request"
CREATION_FLAGS = 0x08000200
_TERMINAL = {"committed", "stopped"}
_LEGACY_FILES = ("deployment/execution-activation.json", "deployment/execution-upgrade-selection.json",
                 "deployment/execution-upgrade-start.json", "formal-lifecycle/keeper.json",
                 "deployment/formal-lifecycle-command.json")
_CAPABILITY = object()
_OPERATOR_FILES = (
    "tools/dev/jenkins_recovery_keeper.py", "tools/dev/start-jenkins-coordinator.ps1", "tools/jenkins/contracts/__init__.py",
    "tools/jenkins/deployment/recovery_keeper.py", "tools/jenkins/deployment/start_transaction.py",
    "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/driver.py", "tools/jenkins/deployment/lost_owner.py",
    "tools/jenkins/deployment/keeper_control.py", "tools/jenkins/deployment/execution_activation.py",
    "tools/jenkins/deployment/live_observations.py", "tools/jenkins/deployment/startup_observations.py",
    "tools/jenkins/deployment/recovery.py", "tools/jenkins/deployment/host.py", "tools/jenkins/deployment/lifecycle.py", "tools/jenkins/deployment/paths.py",
    "tools/jenkins/deployment/spec.py", "tools/jenkins/deployment/bootstrap.py", "tools/jenkins/deployment/job_provisioning.py",
    "tools/jenkins/processes/identity.py", "tools/jenkins/processes/boot.py", "tools/jenkins/processes/native.py",
    "tools/jenkins/processes/job.py", "tools/jenkins/processes/guards.py", "tools/jenkins/state/locks.py", "tools/jenkins/state/atomic.py",
    "tools/jenkins/deployment/owned_upgrade.py", "tools/dev/jenkins_desktop_tray.py",
    *(f"tools/jenkins/tray/{name}.py" for name in (
        "__init__", "__main__", "app", "config", "formal", "formal_app", "formal_native", "formal_startup",
        "launch", "lifecycle", "menu", "native_windows", "operations", "persistence", "service", "startup",
        "terminal", "win32_shell", "worker_environment")),
)
_OWNED_OPERATOR_FILES = {"tools/jenkins/deployment/owned_upgrade.py", "tools/dev/jenkins_desktop_tray.py",
                         *(name for name in _OPERATOR_FILES if name.startswith("tools/jenkins/tray/"))}


def _require(condition, message, code="recovery_keeper_unproven"):
    if not condition:
        raise JenkinsError(code, message)


def _raw(path):
    path = Path(path).absolute()
    _plain(path)
    return path.read_bytes()


def _sha(path):
    return hashlib.sha256(_raw(path)).hexdigest()


def _read(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            _require(key not in result, "Recovery document contains duplicate keys")
            result[key] = value
        return result
    value = json.loads(_raw(path).decode("utf-8"), object_pairs_hook=unique,
                       parse_constant=lambda _: (_ for _ in ()).throw(
                           JenkinsError("recovery_keeper_unproven", "Recovery evidence contains nonfinite values")))
    _require(isinstance(value, dict), "Recovery evidence must be an object")
    return value


def _reference(state, reference, *, release=False):
    _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}, "Exact immutable reference required")
    require_digest(reference["sha256"], "referenceSha256")
    path = Path(reference["path"]).absolute()
    _require(str(path) == reference["path"] and ".." not in path.parts, "Canonical evidence path required")
    allowed = {state / "migration-baseline", state / "deployment/history", state / "recovery-lifecycle/history"}
    if release:
        allowed.add(state / "deployment")
    _require(path.parent in allowed and _sha(path) == reference["sha256"], "Immutable recovery evidence changed")
    return _read(path)


def _archive(state, prefix, value):
    return _archive_bytes(state, prefix, canonical_json(value) + b"\n")


def _archive_bytes(state, prefix, raw):
    """Preserve original interruption bytes, including unknown native outcome."""
    checksum = hashlib.sha256(raw).hexdigest()
    path = state / "recovery-lifecycle/history" / (prefix + "-" + checksum + ".json")
    _plain(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        _require(_raw(path) == raw, "Immutable recovery evidence conflicts")
    else:
        with path.open("xb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
    _require(_sha(path) == checksum, "Recovery evidence write is unknown")
    return {"path": str(path), "sha256": checksum}


def _full_identity(pid):
    before = current_identity(pid).to_dict()
    args = _windows_command_line(pid)
    after = current_identity(pid).to_dict()
    _require(all(before.get(key) == after.get(key) for key in ("pid", "creationTime", "executable")),
             "Recovery process identity changed during observation")
    return {"pid": after["pid"], "creationTime": str(after["creationTime"]),
            "executable": after["executable"], "commandDigest": _digest_args(args)}


def _historical_regression(repo, state, reference, candidate, *, required_operators=None):
    """Bind the passed historical inventory to this exact immutable snapshot."""
    receipt = _reference(state, reference)
    from .support_regression import SCOPED_SCOPE, verify_support_regression
    if receipt.get("scope") == SCOPED_SCOPE:
        return verify_support_regression(repo, reference, candidate, current=False, state_root=state,
                                         required_operators=required_operators or _OPERATOR_FILES)
    _require(Path(reference["path"]).parent == state / "migration-baseline"
             and receipt.get("scope") == "complete-current-Jenkins-support-pytest-discovery"
             and receipt.get("status") == "passed" and receipt.get("sourceIdentityStable") is True
             and type(receipt.get("testsRun")) is int and receipt["testsRun"] > 0
             and receipt.get("exitCode") == 0 and receipt.get("failures") == 0 and receipt.get("errors") == 0
             and receipt.get("collectionErrors") == [], "Complete historical support regression did not pass")
    log = Path(receipt.get("log", "")).absolute()
    _require(str(log) == receipt.get("log") and ".." not in log.parts
             and log.parent == state / "migration-baseline", "Historical support log escaped its immutable namespace")
    require_digest(receipt.get("logSha256"), "regressionLogSha256")
    _require(_sha(log) == receipt["logSha256"], "Historical support regression output changed")
    source_files = receipt.get("sourceFiles")
    _require(isinstance(source_files, dict) and source_files, "Historical support source inventory is missing")
    expected, tests = {}, []
    for name, checksum in source_files.items():
        relative = PurePosixPath(name)
        _require(relative.as_posix() == name and not relative.is_absolute()
                 and not any(part in {".", ".."} for part in relative.parts) and "\\" not in name and ":" not in name
                 and ((name.startswith("tools/jenkins/") and name.endswith(".py"))
                      or (name.startswith(".jenkins/pipeline/") and name.endswith(".groovy"))
                      or name in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}),
                 "Historical support source inventory contains an undeclared path")
        require_digest(checksum, "historicalSourceSha256")
        if name.startswith("tools/jenkins/") and "/tests/" in name:
            tests.append(name)
        elif name not in {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"}:
            expected[name] = checksum
    _require(tests and {".jenkins/deployment-spec.json", ".jenkins/workflow-spec.json"} <= set(source_files)
             and all(name in source_files for name in (required_operators or _OPERATOR_FILES)
                     if name.startswith("tools/jenkins/")),
             "Historical support regression lacks its complete lifecycle inventory")
    manifest = _read(candidate["manifest"])
    source_name = "tools/jenkins/deployment/driver.py"
    source_raw = _raw(Path(candidate["root"]) / source_name)
    _require(hashlib.sha256(source_raw).hexdigest() == expected.get(source_name),
             "Historical launcher source differs from its tested driver")
    try:
        assignments = [node.value for node in ast.parse(source_raw, filename=source_name).body
                       if isinstance(node, ast.Assign) and len(node.targets) == 1
                       and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "_LAUNCHER"]
        launcher = assignments[0].value if len(assignments) == 1 and isinstance(assignments[0], ast.Constant) else None
    except (SyntaxError, UnicodeError, ValueError, TypeError) as error:
        raise JenkinsError("recovery_keeper_unproven", "Historical launcher literal is unproven") from error
    _require(type(launcher) is str, "Historical driver must contain one literal launcher")
    expected["launcher.py"] = hashlib.sha256(launcher.encode("utf-8")).hexdigest()
    expected["tools/__init__.py"] = hashlib.sha256(
        b'"""This package belongs to one verified Jenkins snapshot."""\n').hexdigest()
    _require(manifest.get("schemaVersion") == 1 and manifest.get("driverDigest") == candidate["driverDigest"]
             and isinstance(manifest.get("files"), dict), "Historical driver manifest identity changed")
    observed = {".jenkins/" + name if name.startswith("pipeline/") else name: entry.get("sha256")
                for name, entry in manifest["files"].items()}
    _require(observed == expected, "Sealed runtime does not cover its exact passed historical implementation")
    return receipt


def _failed_start_marker(state, document):
    marker = document.get("failedOwnStart")
    if marker is None:
        return None
    _require(isinstance(marker, dict) and set(marker) == {"plan", "originalCommand", "startTransaction", "attemptedHost"},
             "Failed-start recovery requires its exact explicitly reviewed marker")
    planned = _reference(state, marker["plan"])
    _require(Path(marker["plan"]["path"]).parent == state / "migration-baseline"
             and planned.get("recoveryVariant") == "failed-own-start"
             and planned.get("requestId") == document.get("requestId") and planned.get("scope") == document.get("scope")
             and planned.get("owner") == document.get("expectedPreviousOwner")
             and planned.get("lifecycleFence") == document.get("expectedLifecycleFence"),
             "Failed-start release differs from its exact reviewed interruption plan")
    expected = {"originalCommand": state / "recovery-lifecycle/command.json",
                "attemptedHost": state / "deployment/host.tmp"}
    for name in ("originalCommand", "startTransaction", "attemptedHost"):
        reference = marker[name]
        _require(isinstance(reference, dict) and set(reference) == {"path", "sha256"}
                 and reference == planned.get(name), "Failed-start evidence reference changed")
        require_digest(reference["sha256"], "failedStartArtifactSha256")
        path = Path(reference["path"]).absolute()
        _require(str(path) == reference["path"] and ".." not in path.parts, "Canonical failed-start artifact required")
        if name == "startTransaction":
            _require(path.parent == state / "recovery-lifecycle/start-transactions"
                     and path.name == planned.get("operationId", "") + ".json", "Original failed-start transaction changed")
        else:
            _require(path == expected[name], "Original failed-start command or attempted host path changed")
    return planned


def _owned_upgrade_marker(state, document):
    marker = document.get("ownedUpgrade")
    if document.get("kind") != OWNED_UPGRADE_RELEASE_KIND:
        _require(marker is None, "Owned transfer requires its distinct reviewed release kind")
        return None
    from .owned_upgrade import _is_external, _transfer_keys, UNKNOWN_EXTERNAL_VARIANT
    if isinstance(marker, dict) and _is_external(marker):
        unknown = marker["variant"] == UNKNOWN_EXTERNAL_VARIANT
        _, command_key, _ = _transfer_keys(marker)
        expected = {"variant", "originalBlockedDescriptor", command_key, "externalNativeStop", "keeperRetirement"}
        if unknown:
            expected.add("unknownStopRecovery")
        _require(set(marker) == expected
                 and document.get("failedOwnStart") is None and document.get("automaticRestart") is False
                 and document.get("queueReplayRequested") is False, "External transfer needs its distinct exact reviewed marker")
        descriptor = _reference(state, marker["originalBlockedDescriptor"])
        command = _reference(state, marker[command_key])
        binding = _reference(state, marker["externalNativeStop"])
        retirement = _reference(state, marker["keeperRetirement"])
        _require(descriptor.get("kind") == DESCRIPTOR_KIND and descriptor.get("admission") in {"idle", "blocked"}
                 and descriptor.get("runtime", {}).get("state") == "running"
                 and command.get("kind") == COMMAND_KIND and command.get("status") == ("unknown" if unknown else "committed")
                 and command.get("keeper") == descriptor.get("keeper") and command.get("release") == descriptor.get("release")
                 and binding.get("kind") == "owned-stop-metadata-binding"
                 and binding.get("capturedOwner", {}).get("operation") == descriptor["runtime"].get("operation")
                 and binding.get("capturedOwner", {}).get("host") == descriptor["runtime"].get("host")
                 and retirement.get("kind") == "owned-recovery-keeper-retirement"
                 and retirement.get("variant") == marker["variant"] and retirement.get("keeper") == descriptor.get("keeper")
                 and retirement.get("originalBlockedDescriptor") == marker["originalBlockedDescriptor"]
                 and retirement.get("externalNativeStop") == marker["externalNativeStop"]
                 and all(value.get(name) is False for value in (descriptor, retirement)
                     for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased"))
                 and binding.get("nativeProofSynthesized") is False,
                 "External transfer does not preserve its genuinely issued blocked predecessor")
        if unknown:
            from .unknown_stop_recovery import validate_unknown_command
            origin = _reference(state, descriptor["runtimeReceipt"])
            validate_unknown_command(state, descriptor, origin, command)
            outcome = _reference(state, marker["unknownStopRecovery"])
            _require(outcome.get("kind") == "unknown-stop-native-recovery" and outcome.get("status") == "stopped"
                     and outcome.get("variant") == marker["variant"]
                     and outcome.get("externalNativeStop") == marker["externalNativeStop"]
                     and outcome.get("originalBlockedDescriptor") == marker["originalBlockedDescriptor"]
                     and outcome.get("unknownStopCommand") == marker["unknownStopCommand"]
                     and retirement.get("unknownStopRecovery") == marker["unknownStopRecovery"],
                     "Unknown Stop transfer lacks its separate genuine native outcome")
        return marker
    _require(isinstance(marker, dict) and set(marker) == {"stoppedDescriptor", "ownedStop", "keeperRetirement"}
             and document.get("failedOwnStart") is None and document.get("automaticRestart") is False
             and document.get("queueReplayRequested") is False, "Owned transfer needs its exact reviewed stop and retirement")
    descriptor = _reference(state, marker["stoppedDescriptor"])
    stop = _reference(state, marker["ownedStop"])
    retirement = _reference(state, marker["keeperRetirement"])
    _require(descriptor.get("kind") == DESCRIPTOR_KIND and descriptor.get("admission") == "idle"
             and descriptor.get("runtime", {}).get("state") == "stopped" and descriptor.get("runtimeReceipt") == marker["ownedStop"]
             and stop.get("kind") == "recovery-keeper-owned-stop" and stop.get("keeper") == descriptor.get("keeper")
             and stop.get("release") == descriptor.get("release") and stop.get("operation") == descriptor["runtime"].get("operation")
             and stop.get("host") == descriptor["runtime"].get("host")
             and retirement.get("kind") == "owned-recovery-keeper-retirement"
             and retirement.get("keeper") == descriptor.get("keeper")
             and retirement.get("ownedStop") == marker["ownedStop"]
             and all(value.get(name) is False for value in (descriptor, stop, retirement)
                     for name in ("runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased")),
             "Owned transfer marker differs from its genuine stopped predecessor")
    return marker


def _operator_path(repo, target, name, *, sealed):
    path = Path(name).absolute()
    _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo),
             "Canonical reviewed operator path required")
    if sealed and path.is_relative_to(repo / "tools/jenkins"):
        return Path(target["root"]) / path.relative_to(repo)
    return path


def _previous_stop_reference(release):
    marker = release.get("ownedUpgrade", {})
    from .owned_upgrade import _is_external
    return marker.get("externalNativeStop") if _is_external(marker) else marker.get("ownedStop")


def _release(manager, reference, *, current_source=True):
    repo, state = manager.paths.repo.absolute(), manager.paths.state.absolute()
    document = _reference(state, reference, release=True)
    _require(document.get("schemaVersion") == 1 and document.get("kind") in {RELEASE_KIND, OWNED_UPGRADE_RELEASE_KIND}
             and document.get("scope") == "execution-runtime"
             and re.fullmatch("[0-9a-f]{32}", str(document.get("requestId", "")))
             and document.get("runtimeAcceptance") is False and document.get("cargoAcceptance") is False,
             "Only a bounded reviewed execution recovery release may start")
    for field, path in (("config", repo / ".codex/config.toml"), ("deploymentSpec", manager.spec.path.absolute()),
                        ("workflowSpec", repo / ".jenkins/workflow-spec.json")):
        value = document.get(field)
        _require(isinstance(value, dict) and set(value) == {"path", "sha256"}
                 and value["path"] == str(path) and _sha(path) == value["sha256"],
                 "Recovery configuration or specification changed")
        require_digest(value["sha256"], field)
    target = document.get("targetDriver")
    _require(isinstance(target, dict), "Recovery target driver is missing")
    root = repo / ".jenkins/runtime/drivers" / require_digest(target.get("driverDigest"), "targetDriverDigest")
    _require(target.get("root") == str(root), "Recovery target escaped its canonical sealed namespace")
    verified = verify_driver(root, expected_digest=target["driverDigest"])
    _require(target == verified, "Recovery release must pin the complete sealed target snapshot")
    code_root = document.get("actorCodeRoot")
    _require(code_root is None or code_root == target["root"], "Actor code root must be its exact sealed target")
    if document["kind"] == OWNED_UPGRADE_RELEASE_KIND:
        _require(code_root == target["root"], "An owned successor must retain its verified sealed actor implementation")
    module_root = Path(__file__).absolute().parents[3]
    sealed = not current_source and code_root is not None
    if sealed:
        _require(module_root == root, "Lifecycle reader must load the exact sealed actor generation")
    operators = document.get("operatorFiles")
    lost_runtime = document.get("lostRecoveryRuntime") is not None
    required_operators = tuple(name for name in _OPERATOR_FILES
                               if name not in _OWNED_OPERATOR_FILES or document["kind"] == OWNED_UPGRADE_RELEASE_KIND
                               or lost_runtime)
    if lost_runtime:
        required_operators += ("tools/jenkins/deployment/lost_recovery_runtime.py",)
        _require(code_root == target["root"], "A lost recovery runtime successor needs its exact sealed actor implementation")
    required = {repo / name for name in required_operators} | {state / "migration-baseline/jenkins_desktop_holder.py"}
    _require(isinstance(operators, dict) and {str(path) for path in required} <= set(operators),
             "Recovery release must pin its canonical entry and all recovery operators")
    for name, checksum in operators.items():
        require_digest(checksum, "operatorSha256")
        path = _operator_path(repo, verified, name, sealed=sealed)
        _require(_sha(path) == checksum, "Reviewed recovery operator bytes changed")
    if current_source:
        verify_regression(repo, document.get("regression", {}), verified)
    historical = _historical_regression(repo, state, document.get("regression", {}), verified,
                                        required_operators=required_operators)
    for field, name in (("deploymentSpec", ".jenkins/deployment-spec.json"), ("workflowSpec", ".jenkins/workflow-spec.json")):
        _require(historical["sourceFiles"].get(name) == document[field]["sha256"], "Historical tested specification changed")
    _require(all(historical["sourceFiles"].get(Path(name).relative_to(repo).as_posix()) == checksum
                 for name, checksum in operators.items() if Path(name).is_relative_to(repo / "tools/jenkins")
                 and "/tests/" not in Path(name).relative_to(repo).as_posix()),
             "Reviewed lifecycle operator differs from its tested historical implementation")
    preparation = _reference(state, document.get("preparation"))
    _require(preparation.get("schemaVersion") == 1 and preparation.get("state") == "prepared-not-selected"
             and preparation.get("driver") == target
             and preparation.get("supportRegression", preparation.get("regression")) == document["regression"]
             and preparation.get("protectedHolds") == document.get("protectedHolds"),
             "Recovery target lacks its exact reviewed preparation and regression")
    holds = document.get("protectedHolds")
    _require(isinstance(holds, list) and len(holds) == 4 and protected_holds(state) == holds,
             "The four protected historical holds must remain exact")
    owner, fence = document.get("expectedPreviousOwner"), document.get("expectedLifecycleFence")
    _failed_start_marker(state, document)
    _owned_upgrade_marker(state, document)
    from .lost_recovery_runtime import reviewed_marker
    reviewed_marker(state, document)
    if document["kind"] == OWNED_UPGRADE_RELEASE_KIND:
        components = document.get("runtimeFiles")
        gui = str(repo / ".jenkins/runtime/python/pythonw.exe")
        _require(isinstance(components, dict) and gui in components, "Owned actor must pin its real desktop Python runtime")
        for name, checksum in components.items():
            path = Path(name).absolute()
            require_digest(checksum, "runtimeComponentSha256")
            _require(str(path) == name and ".." not in path.parts and path.is_relative_to(repo / ".jenkins/runtime")
                     and _sha(path) == checksum, "Reviewed owned runtime component changed")
    fence_names = {"deployment-operation.json", "deployment/host.json", "deployment/driver.json", *_LEGACY_FILES}
    _require(isinstance(owner, dict) and isinstance(fence, dict) and set(fence) == fence_names
             and all(fence[name] is not None for name in fence_names - {"deployment/formal-lifecycle-command.json"}),
             "Reviewed complete predecessor binding is missing")
    for checksum in fence.values():
        if checksum is not None:
            require_digest(checksum, "lifecycleFenceSha256")
    previous = preparation.get("previousSelectedDriver")
    _require(isinstance(previous, dict), "Prepared predecessor selector is missing")
    old_root = repo / ".jenkins/runtime/drivers" / require_digest(previous.get("driverDigest"), "previousDriverDigest")
    _require(previous.get("root") == str(old_root)
             and preparation.get("previousSelectorSha256") == fence["deployment/driver.json"]
             and previous.get("runtimeOperationId") == owner.get("operation", {}).get("operationId"),
             "Prepared predecessor selector changed")
    old_verified = verify_driver(old_root, expected_digest=previous["driverDigest"])
    _require(all(previous.get(key) == value for key, value in old_verified.items()), "Prepared predecessor driver bytes changed")
    return document, verified


def _keeper_argv(repo, keeper, release, config_sha):
    expected = [str(repo / ".jenkins/runtime/python/python.exe"), "-B",
                str(repo / "tools/dev/jenkins_recovery_keeper.py"), "--config-sha256", config_sha,
                "--release-file", release["path"], "--release-sha256", release["sha256"]]
    args = list(_windows_command_line(keeper["pid"]))
    _require(args in (expected, [*expected, "--no-tray"])
             and keeper.get("executable") == expected[0] and _digest_args(tuple(args)) == keeper["commandDigest"],
             "Recovery keeper must run the exact canonical reviewed command")


def _restriction(probe, keeper, config):
    configuration, allowed, job = (probe.get(name, {}) for name in ("configuration", "allowedProbe", "associatedJob"))
    identity = probe.get("holderIdentity", {})
    allowed_path = Path(allowed.get("path", "")).absolute()
    probe_root = Path(config["path"]).parent.parent / ".jenkins/state/desktop-holder"
    _plain(allowed_path)
    _require(probe.get("requestedProfile") == "zircon-strict"
             and configuration.get("path") == config["path"] and configuration.get("updateOpenDenied") is True
             and configuration.get("sha256Before") == config["sha256"]
             and configuration.get("sha256After") == config["sha256"]
             and allowed.get("exclusiveCreateWrite") is True and allowed.get("removed") is True
             and allowed.get("path") == str(allowed_path) and allowed_path.parent == probe_root
             and re.fullmatch("probe-[0-9a-f]{32}\\.tmp", allowed_path.name) and not allowed_path.exists()
             and job.get("query") == "current-process-associated-job" and job.get("holderMember") is True
             and keeper["pid"] in job.get("memberPids", [])
             and all(str(identity.get(key)) == str(keeper[key]) for key in ("pid", "creationTime", "executable")),
             "Actual strict file operations and retained outer Job membership are required")
    return probe


class RecoveryKeeperStart:
    """One process-local start capability, bound to its actor and exact command."""
    def __init__(self, actor, plan, request, command_sha, *, token=None):
        _require(token is _CAPABILITY, "Only the retained recovery actor may issue a start capability")
        self.actor, self.plan, self.request, self.command_sha = actor, plan, request, command_sha
        self.used, self.spawn = False, None

    def consume(self, manager, plan):
        _require(type(self) is RecoveryKeeperStart and not self.used and manager is self.actor.manager
                 and plan is self.plan, "Recovery start cannot be replayed or transferred")
        self.actual_preflight = self.actor._fresh()
        _require(_sha(self.actor.command_path) == self.command_sha, "Durable recovery command changed before start")
        command = _read(self.actor.command_path)
        _require(command.get("kind") == COMMAND_KIND and command.get("status") == "start-attempted"
                 and command.get("request") == self.request and command.get("release") == self.actor.release_ref
                 and command.get("keeper") == self.actor.keeper, "Exact explicit recovery start intent is required")
        if plan is None:
            self.verify_terminal(manager)
        else:
            from .owned_upgrade import OwnedRecoveryUpgradePlan
            if type(plan) is OwnedRecoveryUpgradePlan:
                self.actor._reviewed_owned_plan(plan)
                _require(command.get("ownedUpgradeDigest") == plan.upgrade_digest
                         and command.get("ownedUpgradeHandoff") == self.actor.owned_handoff["receipt"],
                         "Owned upgrade differs from its exact successor command")
                plan.revalidate(manager)
                self.used = True
                return
            _require(type(plan) in {LostFormalOwnerRecovery, LostRecoveryRuntimeInterruption}, "Typed lost-owner recovery plan required")
            value = plan.document
            _require(value.get("requestId") == self.actor.release["requestId"]
                     and value.get("scope") == self.actor.release["scope"]
                     and value.get("owner") == self.actor.release["expectedPreviousOwner"]
                     and value.get("lifecycleFence") == self.actor.release["expectedLifecycleFence"]
                     and command.get("recoveryDigest") == plan.recovery_digest,
                     "Lost-owner recovery differs from its explicit reviewed release")
            if value.get("recoveryVariant") == "failed-own-start":
                plan.revalidate(manager)
            elif type(plan) is LostRecoveryRuntimeInterruption:
                self.actor._reviewed_lost_runtime_plan(plan)
                _require(command.get("lostRecoveryRuntimeInterruption") == self.actor.runtime_interruption["receipt"],
                         "Lost runtime replacement must retain its exact interruption archive")
                plan.revalidate(manager)
        self.used = True

    def verify_terminal(self, manager):
        _require(manager is self.actor.manager and self.plan is None, "Only this actor's explicit own stop authorizes restart")
        return self.actor._verify_terminal()

    def target_driver(self, manager):
        _require(self.used and manager is self.actor.manager, "Only a consumed capability may select its target")
        self.actor._fresh()
        if self.plan is None:
            return None
        # Only the exact sealed actor reads its reviewed runtime evidence.
        release, target = _release(manager, self.actor.release_ref, current_source=False)
        _require(release == self.actor.release and target == self.actor.target,
                 "The first recovery target must retain its exact verified sealed implementation")
        selected = _read(self.actor.state / "deployment/driver.json")
        _require(_sha(self.actor.state / "deployment/driver.json")
                 == self.actor.release["expectedLifecycleFence"].get("deployment/driver.json"),
                 "Old selector must be preserved until the reviewed recovery archive")
        if selected.get("driverDigest") == self.actor.target["driverDigest"]:
            return None
        state = ("selected-for-owned-upgrade-start" if self.actor.owned_handoff is not None else "selected-for-recovery-start")
        return {**self.actor.target, "state": state, "runtimeAcceptance": False,
                "cargoAcceptance": False, "supportRegression": self.actor.release["regression"],
                "preparation": self.actor.release["preparation"]["path"], "recoveryRelease": self.actor.release_ref}

    def creation_flags(self):
        _require(self.used and self.spawn is None and os.name == "nt", "One consumed Windows recovery start is required")
        return CREATION_FLAGS

    def record_spawn(self, pid):
        _require(self.used and self.spawn is None and type(pid) is int and pid > 0 and pid != self.actor.keeper["pid"],
                 "Recovery capability must create exactly one supervisor")
        observed = _restriction(self.actor.preflight(self.actor.repo, self.actor.config_sha), self.actor.keeper,
                                self.actor.release["config"])
        _require(pid in observed["associatedJob"]["memberPids"], "Created supervisor escaped the retained restricted outer Job")
        child = _full_identity(pid)
        args = list(_windows_command_line(pid))
        prefix = [str(self.actor.repo / ".jenkins/runtime/python/python.exe"), self.actor.target["launcher"],
                  "__lifecycle-host", "--spec", str(self.actor.manager.spec.path), "--java", str(self.actor.manager.java),
                  "--war", str(self.actor.manager.war), "--operation-id"]
        _require(args[:-1] == prefix and re.fullmatch("start-[0-9]+", args[-1])
                 and child["executable"] == prefix[0] and child["commandDigest"] == _digest_args(tuple(args)),
                 "Recovery supervisor command differs from the pinned target and Home")
        self.spawn = {"creator": self.actor.keeper, "child": child, "creationFlags": CREATION_FLAGS,
                      "actualRestrictionPreflight": self.actual_preflight, "associatedJobAtSpawn": observed,
                      "requestedProfile": "zircon-strict", "childInJob": True, "creatorInJob": True}
        command_sha = _sha(self.actor.command_path)
        command = _read(self.actor.command_path)
        _require(command.get("status") == "start-attempted" and command.get("request") == self.request,
                 "Recovery command changed during native creation")
        _write(self.actor.command_path, {**command, "createdSupervisor": self.spawn}, expected_sha=command_sha)


class RecoveryKeeper:
    def __init__(self, manager, *, release_path, release_sha256, preflight, publish, recovery_plan=None, upgrade_plan=None):
        self.manager, self.preflight, self.publish = manager, preflight, publish
        self.repo, self.state = manager.paths.repo.absolute(), manager.paths.state.absolute()
        self.release_ref = {"path": str(Path(release_path).absolute()), "sha256": release_sha256}
        # Only the exact sealed actor reads its reviewed runtime evidence.
        self.release, self.target = _release(manager, self.release_ref, current_source=False)
        self.config_sha = self.release["config"]["sha256"]
        self.keeper = _full_identity(os.getpid())
        self.directory = self.state / "recovery-lifecycle"
        self.command_path = self.directory / "command.json"
        self.descriptor = self.state / "formal-lifecycle/recovery-keeper.json"
        self.blocked, self.runtime, self.runtime_receipt, self.stop_packet = False, None, None, None
        self.seen = set()
        self.initial_recovery, self.initial_digest, self.boot = None, None, str(windows_boot_filetime())
        self.recovery_plan, self.original_interruption = recovery_plan, None
        self.runtime_interruption = None
        self.upgrade_plan, self.initial_upgrade, self.upgrade_digest, self.owned_handoff = upgrade_plan, None, None, None
        self.descriptor_sha = None
        _require(recovery_plan is None or upgrade_plan is None, "Lost interruption and owned transfer cannot be combined")
        if self.release["kind"] == OWNED_UPGRADE_RELEASE_KIND:
            with process_lock(self.state / "deployment-start.lock", reason="deployment_operation_busy"):
                self._reviewed_owned_plan(upgrade_plan)
                upgrade_plan.revalidate(manager)
                _require(self.descriptor.exists()
                         and _sha(self.descriptor) == upgrade_plan.document["descriptorSha256"],
                         "Only the exact genuinely stopped descriptor may be transferred")
                self.descriptor_sha = _sha(self.descriptor)
                self._idle_command()
        elif self.release.get("lostRecoveryRuntime") is not None:
            with process_lock(self.state / "deployment-start.lock", reason="deployment_operation_busy"):
                _require(upgrade_plan is None, "Lost interruption cannot use owned transfer authority")
                self._reviewed_lost_runtime_plan(recovery_plan)
                recovery_plan.revalidate(manager)
                _require(self.descriptor.exists()
                         and _sha(self.descriptor) == recovery_plan.document["originalDescriptor"]["sha256"],
                         "Only the exact reviewed departed recovery descriptor may be interrupted")
                self.descriptor_sha = _sha(self.descriptor)
                self._idle_command()
        else:
            _require(upgrade_plan is None and not self.descriptor.exists(),
                     "An existing recovery actor requires explicit typed owned transfer; it cannot be adopted")
        if self.release.get("failedOwnStart") is not None:
            with process_lock(self.state / "deployment-start.lock", reason="deployment_operation_busy"):
                self._reviewed_failed_plan(recovery_plan)
                recovery_plan.revalidate(manager)
                self._idle_command()
        elif self.release["kind"] != OWNED_UPGRADE_RELEASE_KIND and self.release.get("lostRecoveryRuntime") is None:
            _require(recovery_plan is None, "A replacement plan requires an explicit reviewed failed-start release")
            self._idle_command()
        self._fresh()

    def _fresh(self):
        release, target = _release(self.manager, self.release_ref, current_source=False)
        _require(release == self.release and target == self.target and str(windows_boot_filetime()) == self.boot,
                 "Reviewed recovery release or Windows boot changed")
        _require(os.getpid() == self.keeper["pid"] and _full_identity(os.getpid()) == self.keeper,
                 "Only the original retained recovery keeper may act")
        _keeper_argv(self.repo, self.keeper, self.release_ref, self.config_sha)
        self._legacy()
        return _restriction(self.preflight(self.repo, self.config_sha), self.keeper, self.release["config"])

    def _legacy(self):
        fence = self.release["expectedLifecycleFence"]
        for name in _LEGACY_FILES:
            path, expected = self.state / name, fence.get(name)
            _plain(path)
            _require((_sha(path) if path.exists() else None) == expected, "Original formal keeper/acceptance lineage changed")

    def _idle_command(self):
        if self.command_path.exists():
            command = _read(self.command_path)
            if self.upgrade_plan is not None and self.runtime is None and self.owned_handoff is None:
                self._reviewed_owned_plan(self.upgrade_plan)
                from .owned_upgrade import _is_external, _transfer_keys, UNKNOWN_EXTERNAL_VARIANT
                planned = self.upgrade_plan.document
                external = _is_external(planned)
                _, key, _ = _transfer_keys(planned)
                expected = "unknown" if planned.get("variant") == UNKNOWN_EXTERNAL_VARIANT else "committed" if external else "stopped"
                _require(command.get("kind") == COMMAND_KIND and command.get("status") == expected
                         and _sha(self.command_path) == self.upgrade_plan.document[key]["sha256"],
                         "Only the reviewed genuine terminal command may be transferred")
                self.upgrade_plan.revalidate(self.manager)
                return
            if (self.release.get("lostRecoveryRuntime") is not None and self.runtime is None
                    and self.runtime_interruption is None):
                self._reviewed_lost_runtime_plan(self.recovery_plan)
                _require(command.get("kind") == COMMAND_KIND
                         and command.get("status") == self.recovery_plan.document["originalCommandStatus"]
                         and _sha(self.command_path) == self.release["lostRecoveryRuntime"]["originalCommand"]["sha256"],
                         "Only the exactly reviewed original recovery command may be interrupted")
                self.recovery_plan.revalidate(self.manager)
                return
            if command.get("status") == "unknown" and self.runtime is None and self.original_interruption is None:
                self._reviewed_failed_plan(self.recovery_plan)
                _require(command.get("kind") == COMMAND_KIND
                         and _sha(self.command_path) == self.release["failedOwnStart"]["originalCommand"]["sha256"],
                         "Only the exactly reviewed original unknown command may be reconciled")
                self.recovery_plan.revalidate(self.manager)
                return
            _require(command.get("kind") == COMMAND_KIND and command.get("status") in _TERMINAL
                     and command.get("keeper") == self.keeper and command.get("release") == self.release_ref,
                     "Preserve and reconcile the original unresolved recovery command", "recovery_keeper_command_unknown")
            result = _reference(self.state, command.get("result"))
            _require(result.get("request") == command.get("request") and result.get("keeper") == self.keeper
                     and result.get("release") == self.release_ref and result.get("runtimeReceipt") == self.runtime_receipt
                     and result.get("runtimeAcceptance") is False,
                     "Completed recovery command lacks its own immutable result")

    def _reviewed_owned_plan(self, plan):
        from .owned_upgrade import OwnedRecoveryUpgradePlan
        _require(type(plan) is OwnedRecoveryUpgradePlan and plan is self.upgrade_plan,
                 "Owned transfer requires its exact issued process-local plan")
        value, marker = plan.document, _owned_upgrade_marker(self.state, self.release)
        _require(marker is not None and value.get("requestId") == self.release["requestId"]
                 and value.get("scope") == self.release["scope"] and value.get("outcome") == "owned-transfer"
                 and value.get("successorRelease") == self.release_ref and value.get("targetDriver") == self.target
                 and value.get("owner") == self.release["expectedPreviousOwner"]
                 and value.get("lifecycleFence") == self.release["expectedLifecycleFence"]
                 and all(value.get(name) == reference for name, reference in marker.items()),
                 "Owned transfer differs from the reviewed successor release")

    def _archive_owned_upgrade(self, plan, request):
        self._reviewed_owned_plan(plan)
        plan.revalidate(self.manager)
        if self.release.get("failedStartReconciliation") is not None:
            from .failed_upgrade_recovery import claim_failed_upgrade_restoration_gate
            claim_failed_upgrade_restoration_gate(self.manager, plan, request, self.keeper)
        else:
            from .prelaunch_upgrade_recovery import claim_prelaunch_restoration_gate
            claim_prelaunch_restoration_gate(self.manager, plan, request, self.keeper)
        artifacts = plan.original_artifacts
        from .owned_upgrade import _transfer_keys, _is_external
        descriptor_key, command_key, stop_key = _transfer_keys(plan.document)
        external = _is_external(plan.document)
        _require(set(artifacts) == {descriptor_key, command_key, "stoppedOperation", "stoppedHost", "previousSelector"},
                 "All original stopped ownership bytes are required")
        archives = {}
        for name, value in artifacts.items():
            reference, raw = value["reference"], value["raw"]
            _require(reference == plan.document["rawAnchors"].get(name) and type(raw) is bytes
                     and hashlib.sha256(raw).hexdigest() == reference["sha256"] and _raw(reference["path"]) == raw,
                     "Original stopped ownership changed before its immutable archive")
            archives[name] = _archive_bytes(self.state, "owned-upgrade-" + name, raw)
            _require(archives[name]["sha256"] == reference["sha256"], "Owned upgrade archive did not preserve original bytes")
        handoff = {
            "schemaVersion": 1, "kind": "owned-recovery-keeper-handoff", "outcome": "owned-transfer",
            "request": request, "keeper": self.keeper, "release": self.release_ref, "ownedUpgradeDigest": plan.upgrade_digest,
            descriptor_key: archives[descriptor_key], command_key: archives[command_key],
            stop_key: plan.document[stop_key], "keeperRetirement": plan.document["keeperRetirement"],
            "previousRelease": plan.document["previousRelease"], "previousKeeper": plan.document["previousKeeper"],
            "originalArtifacts": {name: {"original": artifacts[name]["reference"], "archive": reference}
                                  for name, reference in archives.items()},
            "nativeProofSynthesized": False, "historicalHoldsReleased": False, "runtimeAcceptance": False,
            "cargoAcceptance": False, "queueReplayRequested": False, "automaticRetry": False}
        if external:
            handoff["variant"] = plan.document["variant"]
            if plan.document.get("unknownStopRecovery") is not None:
                handoff["unknownStopRecovery"] = plan.document["unknownStopRecovery"]
        receipt = _archive(self.state, "owned-upgrade-handoff", handoff)
        replacement = plan.authorize_keeper_command(self.manager, keeper=self.keeper, release_ref=self.release_ref,
            request=request, descriptor_archive=archives[descriptor_key], command_archive=archives[command_key],
            handoff_ref=receipt)
        from .owned_upgrade import OwnedRecoveryUpgradePlan
        _require(type(replacement) is OwnedRecoveryUpgradePlan and replacement.document == plan.document,
                 "Owned transfer issuance changed during command replacement")
        self.upgrade_plan = replacement
        self.owned_handoff = {descriptor_key: archives[descriptor_key], command_key: archives[command_key], "receipt": receipt}
        if external:
            self.owned_handoff.update(variant=plan.document["variant"], externalNativeStop=plan.document[stop_key])
            if plan.document.get("unknownStopRecovery") is not None:
                self.owned_handoff["unknownStopRecovery"] = plan.document["unknownStopRecovery"]
        return replacement

    def _reviewed_failed_plan(self, plan):
        _require(type(plan) is LostFormalOwnerRecovery, "Only a typed failed-own-start interruption may replace an unknown command")
        reviewed = _failed_start_marker(self.state, self.release)
        _require(reviewed is not None and plan is self.recovery_plan and plan.document == reviewed,
                 "Failed-own-start capability differs from its explicitly reviewed release")

    def _reviewed_lost_runtime_plan(self, plan):
        from .lost_recovery_runtime import reviewed_marker
        _require(type(plan) is LostRecoveryRuntimeInterruption and plan is self.recovery_plan
                 and plan.document == reviewed_marker(self.state, self.release),
                 "Lost runtime replacement requires its exact issued reviewed capability")

    def _archive_lost_runtime(self, plan, request):
        from .lost_recovery_runtime import KIND, _FLAGS
        self._reviewed_lost_runtime_plan(plan)
        plan.revalidate(self.manager)
        artifacts, archives = plan.original_artifacts, {}
        for name, value in artifacts.items():
            reference, raw = value["reference"], value["raw"]
            _require(reference == plan.document.get(name) and type(raw) is bytes
                     and hashlib.sha256(raw).hexdigest() == reference["sha256"] and _raw(reference["path"]) == raw,
                     "Every original lost runtime byte must remain exact before archiving")
            archives[name] = _archive_bytes(self.state, "lost-recovery-runtime-" + name, raw)
        receipt = _archive(self.state, "lost-recovery-runtime-interruption", {
            "schemaVersion": 1, "kind": KIND, "outcome": "interrupted", "keeper": self.keeper,
            "release": self.release_ref, "request": request, "recoveryDigest": plan.recovery_digest,
            "reviewedPlan": self.release["lostRecoveryRuntime"]["plan"], "operationId": plan.document["operationId"],
            "originalCommandStatus": plan.document["originalCommandStatus"], "originalRequestId": plan.document["originalRequestId"],
            "bootFileTime": self.boot, "originalArtifacts": {
                name: {"original": artifacts[name]["reference"], "archive": reference} for name, reference in archives.items()},
            **{name: False for name in _FLAGS}})
        replacement = plan.authorize_keeper_command(self.manager, keeper=self.keeper, release_ref=self.release_ref,
            request=request, archives=archives, interruption_ref=receipt)
        _require(type(replacement) is LostRecoveryRuntimeInterruption and replacement.document == plan.document,
                 "Lost runtime authority changed while binding its one replacement command")
        self.runtime_interruption = {"originalCommand": archives["originalCommand"],
                                     "originalDescriptor": archives["originalDescriptor"], "receipt": receipt}
        self.recovery_plan = replacement
        return replacement

    def _archive_failed_start(self, plan, request):
        self._reviewed_failed_plan(plan)
        plan.revalidate(self.manager)
        artifacts = plan.original_artifacts
        _require(isinstance(artifacts, dict) and {"originalCommand", "startTransaction", "attemptedHost"} <= set(artifacts),
                 "Complete original failed-start bytes are required before command replacement")
        archives = {}
        for name, value in artifacts.items():
            reference, raw = value["reference"], value["raw"]
            _require(name in {"originalCommand", "startTransaction", "attemptedHost", "committedHost"}
                     and reference == plan.document.get(name) and type(raw) is bytes
                     and hashlib.sha256(raw).hexdigest() == reference["sha256"]
                     and _raw(reference["path"]) == raw, "Original failed-start evidence changed before its immutable archive")
            archives[name] = _archive_bytes(self.state, "failed-own-start-" + name, raw)
            _require(archives[name]["sha256"] == reference["sha256"], "Failed-start archive did not preserve the entire original bytes")
        receipt = _archive(self.state, "failed-own-start-interruption", {
            "schemaVersion": 1, "kind": "failed-recovery-start-interruption", "request": request,
            "release": self.release_ref, "keeper": self.keeper, "recoveryDigest": plan.recovery_digest,
            "reviewedPlan": self.release["failedOwnStart"]["plan"],
            "originalCommand": archives["originalCommand"], "startTransaction": archives["startTransaction"],
            "attemptedHost": archives["attemptedHost"], "originalRequest": plan.document["originalRequest"],
            "operationId": plan.document["operationId"], "nativeProofIssued": False,
            "nativeTerminationProven": False, "historicalEofClaimed": False,
            "originalArtifacts": {name: {"original": artifacts[name]["reference"], "archive": reference}
                                  for name, reference in archives.items()},
            "originalCommandStatus": "unknown", "outcome": "interrupted", "nativeProofSynthesized": False,
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False,
            "queueReplayRequested": False, "automaticRetry": False})
        replacement = plan.authorize_keeper_command(self.manager, keeper=self.keeper, release_ref=self.release_ref,
            request=request, original_archive=archives["originalCommand"], transaction_archive=archives["startTransaction"],
            attempted_host_archive=archives["attemptedHost"], interruption_ref=receipt)
        _require(type(replacement) is LostFormalOwnerRecovery and replacement.document == plan.document,
                 "Original interruption issuance changed during command replacement")
        self.original_interruption = {"originalCommand": archives["originalCommand"], "receipt": receipt}
        self.recovery_plan = replacement
        return replacement

    def _advertise(self):
        _require(self.runtime is not None and self.runtime_receipt is not None, "Recovery runtime issuance is unavailable")
        value = {"schemaVersion": 1, "kind": DESCRIPTOR_KIND, "admission": "blocked" if self.blocked else "idle",
                 "keeper": self.keeper, "release": self.release_ref, "bootFileTime": self.boot,
                 "runtime": self.runtime, "runtimeReceipt": self.runtime_receipt,
                 "initialRecovery": self.initial_recovery, "recoveryDigest": self.initial_digest,
                 "failedOwnStartInterruption": self.original_interruption["receipt"] if self.original_interruption else None,
                 "initialOwnedUpgrade": self.initial_upgrade, "ownedUpgradeDigest": self.upgrade_digest,
                 "ownedUpgradeHandoff": self.owned_handoff["receipt"] if self.owned_handoff else None,
                 "previousStoppedRuntimeReceipt": _previous_stop_reference(self.release),
                 "protectedHolds": self.release["protectedHolds"], "automaticRestart": False,
                 "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        if self.runtime_interruption is not None:
            value["lostRecoveryRuntimeInterruption"] = self.runtime_interruption["receipt"]
        _write(self.descriptor, value, expected_sha=self.descriptor_sha)
        self.descriptor_sha = _sha(self.descriptor)

    def _owner(self, operation, host):
        fields = ("pid", "creationTime", "executable", "commandDigest")
        def identity(value):
            _require(isinstance(value, dict) and type(value.get("pid")) is int and value["pid"] > 0
                     and str(value.get("creationTime", "")).isdigit() and value.get("executable"), "Runtime identity is incomplete")
            require_digest(value.get("commandDigest"), "nativeCommandDigest")
            return {field: str(value[field]) if field == "creationTime" else value[field] for field in fields}
        _require(operation.get("homePath") == str(self.manager.paths.home.absolute())
                 and operation.get("operationId") == operation.get("generation") == host.get("operationId") == host.get("generation")
                 and operation.get("pid") == operation.get("hostPid") == host.get("hostPid")
                 and operation.get("controlPlaneOnly") is False and host.get("controlPlaneOnly") is False
                 and host.get("agentLaunchAttempted") is True, "Full execution runtime ownership is incomplete")
        owner = {"supervisor": identity(operation), "controller": identity(host.get("controllerIdentity") or host.get("identity")),
                 "agent": identity(host.get("agentIdentity"))}
        _require(len({value["pid"] for value in owner.values()}) == 3, "Runtime component identities conflict")
        return owner

    def _running(self, cap):
        operation = self.manager._read()
        host = _read(self.state / "deployment/host.json")
        selected = _read(self.state / "deployment/driver.json")
        _require(isinstance(operation, dict) and operation.get("state") == "running" and host.get("status") == "running"
                 and selected.get("runtimeOperationId") == operation.get("operationId")
                 and all(selected.get(key) == value for key, value in self.target.items()), "Recovery runtime did not start its exact target")
        owner = self._owner(operation, host)
        _require(cap.spawn is not None and owner["supervisor"] == cap.spawn["child"], "Running runtime differs from the created supervisor")
        observations = [observe_live_identity(owner[key]) for key in ("supervisor", "controller", "agent")]
        endpoint = observe_endpoint_owner(self.manager.spec.controller["listenAddress"],
            int(self.manager.spec.controller["httpPort"]), owner["controller"]["pid"])
        health = self.manager.health()
        _require(health.get("ready") is True and all(health.get(key) is True for key in ("controller", "agent", "plugins")),
                 "Recovery runtime health and admission are incomplete")
        _require(self.manager._read() == operation and _read(self.state / "deployment/host.json") == host
                 and _read(self.state / "deployment/driver.json") == selected, "Runtime changed during recovery observation")
        return {"state": "running", "operation": operation, "host": host, "owner": owner, "selected": selected,
                "nativeIdentities": observations, "endpoint": endpoint, "health": health}

    def _verify_runtime(self):
        _require(self.runtime is not None and self.runtime["state"] == "running", "This actor has no current running issuance")
        _require(self.manager._read() == self.runtime["operation"]
                 and _read(self.state / "deployment/host.json") == self.runtime["host"]
                 and _read(self.state / "deployment/driver.json") == self.runtime["selected"], "Current recovery owner changed")
        for key in ("supervisor", "controller", "agent"):
            observe_live_identity(self.runtime["owner"][key])
        observe_endpoint_owner(self.manager.spec.controller["listenAddress"], int(self.manager.spec.controller["httpPort"]),
                               self.runtime["owner"]["controller"]["pid"])

    def _verify_terminal(self):
        packet = self.stop_packet
        _require(packet is not None and self.runtime is not None and self.runtime["state"] == "stopped",
                 "Only a verified actual Stop by this actor authorizes a later Start")
        body = _reference(self.state, packet["receipt"])
        operation, host = self.manager._read(), _read(self.state / "deployment/host.json")
        _require(body.get("kind") == "recovery-keeper-owned-stop" and body.get("keeper") == self.keeper
                 and body.get("runtimeAcceptance") is False and operation == packet["operation"]
                 and host == packet["host"] and body.get("operation") == operation and body.get("host") == host
                 and operation.get("state") == "stopped" and _read(self.state / "deployment/driver.json") == self.runtime["selected"],
                 "The actor's exact stopped owner or evidence changed")
        binding = _reference(self.state, operation.get("stopBinding"))
        _require(binding == packet["stopBinding"] and binding.get("nativeProofSynthesized") is False,
                 "Actual native stop binding is unavailable")
        departure = observe_departed_identity(operation)
        _require(has_terminal_lifecycle_proofs(operation, host, str(self.manager.paths.home.absolute()), departure),
                 "Complete owned native termination and supervisor departure are required")
        return packet["receipt"]

    def _start(self, plan, request):
        command = _read(self.command_path)
        _write(self.command_path, {**command, "status": "start-attempted"}, expected_sha=_sha(self.command_path))
        cap = RecoveryKeeperStart(self, plan, request, _sha(self.command_path), token=_CAPABILITY)
        from .owned_upgrade import OwnedRecoveryUpgradePlan
        authority = ({"owned_upgrade_start": plan} if type(plan) is OwnedRecoveryUpgradePlan
                     else {"lost_owner_recovery": plan})
        started = self.manager._start_locked(self.state / "deployment-start.lock",
                                            recovery_keeper_start=cap, **authority)
        _require(cap.used and cap.spawn is not None and started.get("state") == "running", "Recovery start outcome is unproven")
        self.manager.register_jobs()
        runtime = self._running(cap)
        self._fresh()
        body = {"schemaVersion": 1, "kind": "recovery-keeper-runtime",
            "request": request, "release": self.release_ref, "keeper": self.keeper, "runtime": runtime,
            "initialRecovery": self.initial_recovery, "recoveryDigest": self.initial_digest,
            "failedOwnStartInterruption": self.original_interruption["receipt"] if self.original_interruption else None,
            "initialOwnedUpgrade": self.initial_upgrade, "ownedUpgradeDigest": self.upgrade_digest,
            "ownedUpgradeHandoff": self.owned_handoff["receipt"] if self.owned_handoff else None,
            "previousStoppedRuntimeReceipt": _previous_stop_reference(self.release),
            "previousRuntimeReceipt": self.runtime_receipt, "strictSpawn": cap.spawn, "bootFileTime": self.boot,
            "protectedHolds": self.release["protectedHolds"], "runtimeAcceptance": False, "cargoAcceptance": False,
            "nativeProofSynthesized": False, "historicalHoldsReleased": False, "queueReplayRequested": False}
        if self.runtime_interruption is not None:
            body["lostRecoveryRuntimeInterruption"] = self.runtime_interruption["receipt"]
        receipt = _archive(self.state, "recovery-runtime", body)
        self.runtime, self.runtime_receipt, self.stop_packet = runtime, receipt, None
        return {"status": "running-acceptance-pending", "runtimeReceipt": receipt, "operationId": started["operationId"],
                "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}

    def _stop(self, request):
        self._verify_runtime()
        previous = self.runtime
        command = _read(self.command_path)
        _write(self.command_path, {**command, "status": "stop-attempted"}, expected_sha=_sha(self.command_path))
        stopped = self.manager._stop_locked()
        host = _read(self.state / "deployment/host.json")
        binding = _reference(self.state, stopped.get("stopBinding"))
        _require(stopped.get("state") == "stopped" and stopped.get("operationId") == previous["operation"]["operationId"]
                 and binding.get("kind") == "owned-stop-metadata-binding"
                 and binding.get("capturedOwner", {}).get("operation") == previous["operation"]
                 and binding.get("capturedOwner", {}).get("host") == previous["host"]
                 and binding.get("terminalHost") == host and binding.get("nativeProofSynthesized") is False,
                 "Stop did not return this actor's exact native ownership binding")
        _require(has_terminal_lifecycle_proofs(stopped, host, str(self.manager.paths.home.absolute()),
                                              observe_departed_identity(stopped)), "Owned stop proof is incomplete")
        self._fresh()
        body = {"schemaVersion": 1, "kind": "recovery-keeper-owned-stop",
            "request": request, "release": self.release_ref, "keeper": self.keeper,
            "previousRuntimeReceipt": self.runtime_receipt, "operation": stopped, "host": host,
            "stopBinding": stopped["stopBinding"], "selected": previous["selected"], "bootFileTime": self.boot,
            "initialRecovery": self.initial_recovery, "recoveryDigest": self.initial_digest,
            "failedOwnStartInterruption": self.original_interruption["receipt"] if self.original_interruption else None,
            "initialOwnedUpgrade": self.initial_upgrade, "ownedUpgradeDigest": self.upgrade_digest,
            "ownedUpgradeHandoff": self.owned_handoff["receipt"] if self.owned_handoff else None,
            "previousStoppedRuntimeReceipt": _previous_stop_reference(self.release),
            "protectedHolds": self.release["protectedHolds"],
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False,
            "nativeProofSynthesized": False}
        if self.runtime_interruption is not None:
            body["lostRecoveryRuntimeInterruption"] = self.runtime_interruption["receipt"]
        receipt = _archive(self.state, "recovery-stop", body)
        self.stop_packet = {"receipt": receipt, "operation": stopped, "host": host, "stopBinding": binding}
        self.runtime = {**previous, "state": "stopped", "operation": stopped, "host": host}
        self.runtime_receipt = receipt
        return {"status": "stopped", "runtimeReceipt": receipt, "operationId": stopped["operationId"],
                "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}

    def _begin(self, request, *, plan=None):
        self._idle_command()
        self._fresh()
        previous_sha = _sha(self.command_path) if self.command_path.exists() else None
        self.blocked = True
        from .owned_upgrade import OwnedRecoveryUpgradePlan
        if type(plan) is OwnedRecoveryUpgradePlan:
            plan = self._archive_owned_upgrade(plan, request)
        elif type(plan) is LostRecoveryRuntimeInterruption:
            plan = self._archive_lost_runtime(plan, request)
        elif plan is not None and plan.document.get("recoveryVariant") == "failed-own-start":
            plan = self._archive_failed_start(plan, request)
        command = {"schemaVersion": 1, "kind": COMMAND_KIND, "status": "pending", "request": request,
                   "release": self.release_ref, "keeper": self.keeper,
                   "recoveryDigest": plan.recovery_digest if type(plan) in {LostFormalOwnerRecovery, LostRecoveryRuntimeInterruption} else self.initial_digest,
                   "previousRuntimeReceipt": self.runtime_receipt, "runtimeAcceptance": False,
                   "historicalHoldsReleased": False}
        if plan is not None and self.original_interruption is not None:
            command.update(replacesOriginalCommand=self.original_interruption["originalCommand"],
                           failedOwnStartInterruption=self.original_interruption["receipt"])
        if plan is not None and self.owned_handoff is not None:
            command.update(ownedUpgradeDigest=self.upgrade_digest, ownedUpgradeHandoff=self.owned_handoff["receipt"])
            from .owned_upgrade import _is_external, _transfer_keys, UNKNOWN_EXTERNAL_VARIANT
            if _is_external(self.owned_handoff):
                _, command_key, _ = _transfer_keys(self.owned_handoff)
                field = "replacesUnknownStopCommand" if self.owned_handoff["variant"] == UNKNOWN_EXTERNAL_VARIANT else "replacesCommittedCommand"
                command.update(variant=self.owned_handoff["variant"],
                    replacesOriginalBlockedDescriptor=self.owned_handoff["originalBlockedDescriptor"],
                    externalNativeStop=self.owned_handoff["externalNativeStop"],
                    **{field: self.owned_handoff[command_key]})
                if self.owned_handoff.get("unknownStopRecovery") is not None:
                    command["unknownStopRecovery"] = self.owned_handoff["unknownStopRecovery"]
            else:
                command.update(replacesStoppedDescriptor=self.owned_handoff["stoppedDescriptor"],
                               replacesTerminalCommand=self.owned_handoff["terminalCommand"])
        if plan is not None and self.runtime_interruption is not None:
            command.update(replacesOriginalCommand=self.runtime_interruption["originalCommand"],
                           replacesLostRecoveryDescriptor=self.runtime_interruption["originalDescriptor"],
                           lostRecoveryRuntimeInterruption=self.runtime_interruption["receipt"])
        _write(self.command_path, command, expected_sha=previous_sha)
        if self.runtime is not None:
            self._advertise()
        return plan

    def _finish(self, request, request_id, result):
        self._fresh()
        reference = _archive(self.state, "recovery-result", {"schemaVersion": 1, "kind": "recovery-keeper-command-result",
            **result, "request": request, "keeper": self.keeper, "release": self.release_ref})
        command = _read(self.command_path)
        _require(command.get("request") == request, "Recovery request changed before completion")
        _write(self.command_path, {**command, "status": "stopped" if result["status"] == "stopped" else "committed",
                                   "result": reference}, expected_sha=_sha(self.command_path))
        _write(self.directory / "results" / (request_id + ".json"), reference)
        self.blocked = False
        self._advertise()
        self.publish("holding-recovery-acceptance-pending", {**result, "descriptor": str(self.descriptor), "result": reference})
        return result

    def _unknown(self, error):
        self.blocked = True
        try:
            if self.command_path.exists():
                command = _read(self.command_path)
                if (command.get("status") not in _TERMINAL and command.get("keeper") == self.keeper
                        and command.get("release") == self.release_ref):
                    _write(self.command_path, {**command, "status": "unknown",
                        "reasonCode": getattr(error, "code", "recovery_keeper_command_unknown")}, expected_sha=_sha(self.command_path))
            if self.runtime is not None:
                self._advertise()
        finally:
            self.publish("holding-recovery-unknown", {"reasonCode": getattr(error, "code", "recovery_keeper_command_unknown"),
                                                     "runtimeAcceptance": False})

    def start(self, plan):
        _require(not self.blocked and self.runtime is None and type(plan) in {LostFormalOwnerRecovery, LostRecoveryRuntimeInterruption},
                 "Initial recovery needs the exact fresh typed lost-owner plan")
        if self.release.get("failedOwnStart") is not None:
            self._reviewed_failed_plan(plan)
        elif self.release.get("lostRecoveryRuntime") is not None:
            self._reviewed_lost_runtime_plan(plan)
        with process_lock(self.state / "deployment-start.lock", reason="deployment_operation_busy"):
            self._fresh()
            value = plan.document
            _require(value.get("requestId") == self.release["requestId"]
                     and value.get("owner") == self.release["expectedPreviousOwner"]
                     and value.get("lifecycleFence") == self.release["expectedLifecycleFence"]
                     and value.get("scope") == "execution-runtime", "Recovery plan differs from its reviewed release")
            # Manager revalidates this exact capability again immediately
            # before archiving the old owner or changing deployment pointers.
            plan.revalidate(self.manager)
            self.initial_digest = plan.recovery_digest
            self.initial_recovery = _archive(self.state, "recovery-initial-plan", value)
            request = _archive(self.state, "recovery-initial-request", {"schemaVersion": 1, "kind": REQUEST_KIND,
                "requestId": self.release["requestId"], "action": "recover", "release": self.release_ref,
                "recoveryDigest": self.initial_digest, "keeper": self.keeper, "runtimeAcceptance": False})
            try:
                start_plan = self._begin(request, plan=plan)
                return self._finish(request, self.release["requestId"], self._start(start_plan, request))
            except BaseException as error:
                self._unknown(error)
                raise

    def start_upgrade(self, plan):
        """Transfer a genuine owned Stop through one exact successor command."""
        _require(not self.blocked and self.runtime is None, "Owned upgrade cannot retry an unresolved start")
        self._reviewed_owned_plan(plan)
        with process_lock(self.state / "deployment-start.lock", reason="deployment_operation_busy"):
            self._fresh()
            plan.revalidate(self.manager)
            self.upgrade_digest = plan.upgrade_digest
            self.initial_upgrade = _archive(self.state, "owned-upgrade-initial-plan", plan.document)
            request = _archive(self.state, "owned-upgrade-initial-request", {
                "schemaVersion": 1, "kind": REQUEST_KIND, "requestId": self.release["requestId"], "action": "upgrade",
                "release": self.release_ref, "keeper": self.keeper, "ownedUpgradeDigest": self.upgrade_digest,
                "protectedHoldsDigest": digest(self.release["protectedHolds"]), "runtimeAcceptance": False})
            try:
                start_plan = self._begin(request, plan=plan)
                return self._finish(request, self.release["requestId"], self._start(start_plan, request))
            except BaseException as error:
                self._unknown(error)
                raise

    def _execute(self, reference):
        with process_lock(self.state / "deployment-start.lock", reason="deployment_operation_busy"):
            _require(not self.blocked, "Recovery keeper is blocked by an unresolved request")
            try:
                self._fresh()
                request = _reference(self.state, reference)
                _validate_request(self, request)
                _require(not (self.directory / "results" / (request["requestId"] + ".json")).exists(),
                         "A completed recovery request cannot be replayed")
                self._begin(reference)
                if request["action"] in {"stop", "restart"}:
                    result = self._stop(reference)
                else:
                    self._verify_terminal()
                if request["action"] != "stop":
                    result = self._start(None, reference)
                return self._finish(reference, request["requestId"], result)
            except BaseException as error:
                self._unknown(error)
                raise

    def stop(self):
        """An explicit retained-owner Stop uses the same durable request path."""
        _require(not self.blocked and self.runtime is not None, "An unresolved recovery actor cannot issue a new Stop")
        with process_lock(self.state / "recovery-lifecycle-submit.lock", reason="recovery_keeper_request_busy"):
            self._fresh()
            _submission_idle(self.state, self.directory)
            if self.runtime["state"] == "stopped":
                receipt = self._verify_terminal()
                return {"status": "stopped", "runtimeReceipt": receipt,
                        "operationId": self.runtime["operation"]["operationId"], "runtimeAcceptance": False,
                        "cargoAcceptance": False, "historicalHoldsReleased": False}
            descriptor = _read(self.descriptor)
            _require(descriptor.get("keeper") == self.keeper and descriptor.get("admission") == "idle"
                     and descriptor.get("runtime") == self.runtime and descriptor.get("runtimeReceipt") == self.runtime_receipt,
                     "Own Stop requires this actor's exact current issuance")
            reference, _request_id = _new_request(self.state, descriptor, _sha(self.descriptor), "stop")
            return self._execute(reference)

    def pump(self):
        if self.blocked:
            return
        requests = self.directory / "requests"
        _plain(requests)
        if not requests.exists():
            return
        for marker in sorted(requests.glob("*.json")):
            if marker.stem in self.seen:
                continue
            self.seen.add(marker.stem)
            try:
                reference = _read(marker)
                request = _reference(self.state, reference)
                _require(request.get("requestId") == marker.stem, "Recovery request marker changed")
                result = self.directory / "results" / marker.name
                if result.exists():
                    body = _reference(self.state, _read(result))
                    _require(body.get("request") == reference and body.get("status") in {"stopped", "running-acceptance-pending"},
                             "Existing recovery request result changed")
                    continue
                from .lost_recovery_runtime import interrupted_request
                if interrupted_request(self.state, reference):
                    continue
                self._execute(reference)
            except BaseException as error:
                if not self.blocked:
                    self._unknown(error)
                return


def _validate_request(actor, request):
    _require(request.get("schemaVersion") == 1 and request.get("kind") == REQUEST_KIND
             and re.fullmatch("[0-9a-f]{32}", str(request.get("requestId", "")))
             and request.get("action") in {"start", "stop", "restart"} and request.get("runtimeAcceptance") is False
             and request.get("release") == actor.release_ref and request.get("keeper") == actor.keeper
             and request.get("runtimeReceipt") == actor.runtime_receipt
             and request.get("descriptorSha256") == _sha(actor.descriptor)
             and request.get("protectedHoldsDigest") == digest(actor.release["protectedHolds"]),
             "Explicit request differs from the authoritative recovery keeper issuance")
    _require((request["action"] == "start" and actor.runtime["state"] == "stopped")
             or (request["action"] in {"stop", "restart"} and actor.runtime["state"] == "running"),
             "Requested lifecycle action differs from the issued current state")


def _submission_idle(state, directory):
    command_path = directory / "command.json"
    if command_path.exists():
        command = _read(command_path)
        _require(command.get("kind") == COMMAND_KIND and command.get("status") in _TERMINAL,
                 "Reconcile the original unresolved recovery command; do not submit a new request",
                 "recovery_keeper_command_unknown")
        result = _reference(state, command.get("result"))
        _require(result.get("request") == command.get("request") and result.get("keeper") == command.get("keeper")
                 and result.get("release") == command.get("release") and result.get("runtimeAcceptance") is False,
                 "Recovery command result differs from its original request")
    requests = directory / "requests"
    _plain(requests)
    if requests.exists():
        for marker in sorted(requests.glob("*.json")):
            reference = _read(marker)
            request = _reference(state, reference)
            _require(request.get("requestId") == marker.stem, "Recovery request marker changed")
            result_path = directory / "results" / marker.name
            if not result_path.exists():
                from .lost_recovery_runtime import interrupted_request
                if interrupted_request(state, reference):
                    continue
                raise JenkinsError("recovery_keeper_command_pending", "Reconcile this original request without resubmission",
                                   details={"request": reference}, retryable=True)
            result = _reference(state, _read(result_path))
            _require(result.get("kind") == "recovery-keeper-command-result" and result.get("request") == reference
                     and result.get("status") in {"stopped", "running-acceptance-pending"}
                     and result.get("runtimeAcceptance") is False, "Existing recovery request result changed")


def _new_request(state, descriptor, descriptor_sha, action):
    request_id = uuid.uuid4().hex
    directory = state / "recovery-lifecycle"
    marker = directory / "requests" / (request_id + ".json")
    _require(not marker.exists() and not (directory / "results" / marker.name).exists(), "Recovery request identity was already used")
    request = {"schemaVersion": 1, "kind": REQUEST_KIND, "requestId": request_id, "action": action,
               "release": descriptor["release"], "keeper": descriptor["keeper"],
               "runtimeReceipt": descriptor["runtimeReceipt"], "descriptorSha256": descriptor_sha,
               "protectedHoldsDigest": digest(descriptor["protectedHolds"]), "runtimeAcceptance": False}
    reference = _archive(state, "recovery-request", request)
    _write(marker, reference)
    return reference, request_id


def _runtime_receipt(config, descriptor, release, target, *, external_native_stop_ref=None, historical_configuration=None):
    state, keeper, runtime = config.paths.state.absolute(), descriptor["keeper"], descriptor["runtime"]
    if historical_configuration is not None:
        from .owned_upgrade import _HistoricalExternalConfiguration
        _require(type(historical_configuration) is _HistoricalExternalConfiguration,
                 "A typed historical external Stop context is required")
        historical_configuration.validate(config, descriptor["release"])
    body = _reference(state, descriptor.get("runtimeReceipt"))
    expected_kind = "recovery-keeper-runtime" if runtime["state"] == "running" else "recovery-keeper-owned-stop"
    _require(external_native_stop_ref is None or expected_kind == "recovery-keeper-runtime",
             "An external NativeStop can validate only its original running issuance")
    _require(body.get("schemaVersion") == 1 and body.get("kind") == expected_kind and body.get("keeper") == keeper
             and body.get("release") == descriptor["release"] and body.get("bootFileTime") == descriptor["bootFileTime"]
             and body.get("protectedHolds") == release["protectedHolds"] and body.get("runtimeAcceptance") is False
             and body.get("cargoAcceptance") is False and body.get("historicalHoldsReleased") is False
             and body.get("nativeProofSynthesized") is False and body.get("initialRecovery") == descriptor.get("initialRecovery")
             and body.get("recoveryDigest") == descriptor.get("recoveryDigest")
             and body.get("failedOwnStartInterruption") == descriptor.get("failedOwnStartInterruption")
             and all(body.get(name) == descriptor.get(name) for name in
                     ("initialOwnedUpgrade", "ownedUpgradeDigest", "ownedUpgradeHandoff", "previousStoppedRuntimeReceipt",
                      "lostRecoveryRuntimeInterruption")),
             "Recovery descriptor lacks its exact immutable runtime issuance")
    origin = body
    if expected_kind == "recovery-keeper-owned-stop":
        _require(body.get("operation") == runtime["operation"] and body.get("host") == runtime["host"]
                 and body.get("selected") == runtime["selected"], "Recovery stopped issuance changed")
        origin = _reference(state, body.get("previousRuntimeReceipt"))
        binding = _reference(state, body.get("stopBinding"))
        _require(binding.get("kind") == "owned-stop-metadata-binding" and binding.get("nativeProofSynthesized") is False
                 and binding.get("capturedOwner", {}).get("operation") == origin.get("runtime", {}).get("operation")
                 and binding.get("capturedOwner", {}).get("host") == origin.get("runtime", {}).get("host")
                 and binding.get("terminalHost") == runtime["host"]
                 and runtime["operation"].get("stopBinding") == body["stopBinding"]
                 and has_terminal_lifecycle_proofs(runtime["operation"], runtime["host"], str(config.paths.home.absolute()),
                                                  observe_departed_identity(runtime["operation"])),
                 "Public Start requires this keeper's actual owned native termination")
    else:
        _require(body.get("runtime") == runtime and body.get("queueReplayRequested") is False,
                 "Recovery runtime receipt and descriptor differ")
        if external_native_stop_ref is None:
            for expected in runtime["owner"].values():
                observe_live_identity(expected)
            observe_endpoint_owner(config.spec.controller["listenAddress"], int(config.spec.controller["httpPort"]),
                                   runtime["owner"]["controller"]["pid"])
        else:
            from .owned_upgrade import _external_native_stop
            _external_native_stop(config, descriptor, external_native_stop_ref)
    spawn = origin.get("strictSpawn", {})
    _require(origin.get("kind") == "recovery-keeper-runtime" and origin.get("keeper") == keeper
             and origin.get("release") == descriptor["release"] and origin.get("runtimeAcceptance") is False
             and origin.get("bootFileTime") == descriptor["bootFileTime"] and spawn.get("creator") == keeper
             and spawn.get("child") == origin.get("runtime", {}).get("owner", {}).get("supervisor")
             and spawn.get("creationFlags") == CREATION_FLAGS and spawn.get("requestedProfile") == "zircon-strict"
             and spawn.get("creatorInJob") is True and spawn.get("childInJob") is True,
             "Recovery runtime lacks its retained strict native creation proof")
    _restriction(spawn.get("actualRestrictionPreflight", {}), keeper, release["config"])
    at_spawn = _restriction(spawn.get("associatedJobAtSpawn", {}), keeper, release["config"])
    _require(spawn["child"]["pid"] in at_spawn["associatedJob"]["memberPids"]
             and all(runtime["selected"].get(key) == value for key, value in target.items()),
             "Runtime target or same outer Job child binding changed")
    if release.get("failedOwnStart") is not None:
        interruption = _reference(state, descriptor.get("failedOwnStartInterruption"))
        planned = _failed_start_marker(state, release)
        _require(interruption.get("kind") == "failed-recovery-start-interruption"
                 and interruption.get("keeper") == keeper and interruption.get("release") == descriptor["release"]
                 and interruption.get("reviewedPlan") == release["failedOwnStart"]["plan"]
                 and interruption.get("recoveryDigest") == descriptor["recoveryDigest"]
                 and interruption.get("operationId") == planned["operationId"] and interruption.get("outcome") == "interrupted"
                 and all(interruption.get(name) is False for name in ("nativeProofIssued", "nativeTerminationProven",
                     "historicalEofClaimed", "runtimeAcceptance", "cargoAcceptance", "historicalHoldsReleased", "automaticRetry")),
                 "The original failed start lacks its immutable interruption binding")
        originals = interruption.get("originalArtifacts", {})
        _require(set(originals) >= {"originalCommand", "startTransaction", "attemptedHost"}, "Original failed-start archives are incomplete")
        for name, refs in originals.items():
            _require(name in {"originalCommand", "startTransaction", "attemptedHost", "committedHost"}
                     and refs.get("original") == planned.get(name) and isinstance(refs.get("archive"), dict)
                     and refs["archive"].get("sha256") == refs["original"]["sha256"], "Original failed-start archive changed")
            archived = _reference(state, refs["archive"])
            if name == "originalCommand":
                _require(archived.get("status") == "unknown", "The original unknown command must retain its honest status")
    if release["kind"] == OWNED_UPGRADE_RELEASE_KIND:
        historical_kwargs = ({} if historical_configuration is None else
                             {"historical_configuration": historical_configuration})
        _owned_handoff_receipt(config, descriptor, release, **historical_kwargs)
    else:
        _require(all(descriptor.get(name) is None for name in
                     ("initialOwnedUpgrade", "ownedUpgradeDigest", "ownedUpgradeHandoff", "previousStoppedRuntimeReceipt")),
                 "A lost-owner issuance cannot claim owned transfer authority")
    if release.get("lostRecoveryRuntime") is not None:
        from .lost_recovery_runtime import validate_interruption_receipt
        validate_interruption_receipt(state, descriptor, release)
    else:
        _require(descriptor.get("lostRecoveryRuntimeInterruption") is None,
                 "An ordinary recovery issuance cannot claim a lost runtime interruption")
    return body


def _owned_handoff_receipt(config, descriptor, release, *, historical_configuration=None):
    from .owned_upgrade import _transfer_keys, _is_external, UNKNOWN_EXTERNAL_VARIANT
    state = config.paths.state.absolute()
    planned = _reference(state, descriptor.get("initialOwnedUpgrade"))
    marker = _owned_upgrade_marker(state, release)
    descriptor_key, command_key, stop_key = _transfer_keys(marker)
    external = _is_external(marker)
    _require(planned.get("kind") == "formal-owned-recovery-upgrade-plan" and planned.get("outcome") == "owned-transfer"
             and digest(planned) == descriptor.get("ownedUpgradeDigest") and planned.get("successorRelease") == descriptor["release"]
             and planned.get("requestId") == release["requestId"] and planned.get("owner") == release["expectedPreviousOwner"]
             and planned.get("lifecycleFence") == release["expectedLifecycleFence"] and planned.get("targetDriver") == release["targetDriver"]
             and all(planned.get(name) == reference for name, reference in marker.items())
             and descriptor.get("previousStoppedRuntimeReceipt") == marker[stop_key],
             "Owned runtime lacks its exact immutable stopped predecessor plan")
    handoff = _reference(state, descriptor.get("ownedUpgradeHandoff"))
    _require(handoff.get("kind") == "owned-recovery-keeper-handoff" and handoff.get("outcome") == "owned-transfer"
             and handoff.get("ownedUpgradeDigest") == descriptor["ownedUpgradeDigest"]
             and handoff.get("keeper") == descriptor["keeper"] and handoff.get("release") == descriptor["release"]
             and handoff.get("previousKeeper") == planned["previousKeeper"] and handoff.get("previousRelease") == planned["previousRelease"]
             and handoff.get("variant") == marker.get("variant")
             and handoff.get(stop_key) == marker[stop_key] and handoff.get("keeperRetirement") == marker["keeperRetirement"]
             and all(handoff.get(name) is False for name in ("nativeProofSynthesized", "historicalHoldsReleased",
                 "runtimeAcceptance", "cargoAcceptance", "queueReplayRequested", "automaticRetry")),
             "Owned upgrade handoff changed")
    request = _reference(state, handoff.get("request"))
    _require(request.get("kind") == REQUEST_KIND and request.get("action") == "upgrade"
             and request.get("requestId") == release["requestId"] and request.get("release") == descriptor["release"]
             and request.get("keeper") == descriptor["keeper"] and request.get("ownedUpgradeDigest") == descriptor["ownedUpgradeDigest"]
             and request.get("protectedHoldsDigest") == digest(release["protectedHolds"])
             and request.get("runtimeAcceptance") is False, "Owned upgrade request changed")
    artifacts = handoff.get("originalArtifacts", {})
    _require(set(artifacts) == {descriptor_key, command_key, "stoppedOperation", "stoppedHost", "previousSelector"},
             "Owned handoff must preserve all original ownership bytes")
    for name, refs in artifacts.items():
        _require(refs.get("original") == planned["rawAnchors"].get(name)
                 and refs.get("archive", {}).get("sha256") == refs["original"]["sha256"],
                 "Stopped predecessor archive identity changed")
        _reference(state, refs["archive"])
    old_descriptor = _reference(state, handoff.get(descriptor_key))
    old_command = _reference(state, handoff.get(command_key))
    _require(handoff[descriptor_key] == artifacts[descriptor_key]["archive"]
             and handoff[command_key] == artifacts[command_key]["archive"]
             and old_descriptor == _reference(state, marker[descriptor_key])
             and old_command.get("status") == ("unknown" if marker.get("variant") == UNKNOWN_EXTERNAL_VARIANT else "committed" if external else "stopped")
             and old_command.get("keeper") == planned["previousKeeper"]
             and old_command.get("release") == planned["previousRelease"]
             and (external or old_descriptor.get("runtimeReceipt") == marker[stop_key]),
             "Owned handoff changed its genuine terminal descriptor or command")
    from .owned_upgrade import _historical_release, _external_historical_configuration
    if historical_configuration is None and external:
        historical_configuration = _external_historical_configuration(config, old_descriptor, marker[stop_key])
    historical_kwargs = ({} if historical_configuration is None else
                         {"historical_configuration": historical_configuration})
    previous_release, previous_target = _historical_release(config, planned["previousRelease"], **historical_kwargs)
    failed_start = release.get("failedStartReconciliation") is not None
    if failed_start:
        from .failed_upgrade_recovery import validate_failed_upgrade_restoration_lineage
        reconciliation = validate_failed_upgrade_restoration_lineage(state, release, descriptor["release"])
    else:
        from .prelaunch_upgrade_recovery import validate_prelaunch_restoration_lineage
        reconciliation = validate_prelaunch_restoration_lineage(state, release, descriptor["release"])
    retirement_request_id = planned["requestId"] if reconciliation is None else reconciliation["eligibility"][
        "predecessorRetirementRequestId" if failed_start else "originalRequestId"]
    if external:
        from .owned_upgrade import _external_native_stop, _committed_command, _retirement, _predecessor_boot
        _require(handoff[descriptor_key]["sha256"] == marker[descriptor_key]["sha256"]
                 and handoff[command_key]["sha256"] == marker[command_key]["sha256"]
                 and old_command == _reference(state, marker[command_key]), "Original blocked descriptor or committed command changed")
        origin = _runtime_receipt(config, old_descriptor, previous_release, previous_target,
                                  external_native_stop_ref=marker[stop_key], **historical_kwargs)
        if marker.get("variant") == UNKNOWN_EXTERNAL_VARIANT:
            from .unknown_stop_recovery import validate_completed_recovery
            validate_completed_recovery(config, old_descriptor, old_command, marker["unknownStopRecovery"], marker[stop_key])
            _require(handoff.get("unknownStopRecovery") == marker["unknownStopRecovery"],
                     "Unknown Stop handoff ancestry changed")
        else:
            _committed_command(state, old_descriptor, origin, old_command)
        stopped = _external_native_stop(config, old_descriptor, marker[stop_key])
        _require(_reference(state, artifacts["stoppedOperation"]["archive"]) == stopped["operation"]
                 and _reference(state, artifacts["stoppedHost"]["archive"]) == stopped["host"]
                 and _reference(state, artifacts["previousSelector"]["archive"]) == old_descriptor["runtime"]["selected"],
                 "External NativeStop differs from its preserved actual terminal anchors")
        previous_boot = _predecessor_boot(old_descriptor, planned["bootFileTime"], external=True)
        _require(planned.get("previousBootFileTime", planned["bootFileTime"]) == previous_boot,
                 "Owned handoff changed its verified predecessor boot")
        _retirement(config, marker["keeperRetirement"], request_id=retirement_request_id,
            descriptor={**old_descriptor, "configurationSha256": previous_release["config"]["sha256"]},
            stopped_descriptor=marker[descriptor_key], owned_stop=marker[stop_key], boot=previous_boot, external=True,
            unknown_stop_recovery_ref=marker.get("unknownStopRecovery"))
    else:
        _require(_runtime_receipt(config, old_descriptor, previous_release, previous_target, **historical_kwargs)
                 == _reference(state, marker[stop_key]), "Previous public NativeJob Stop receipt changed")
    return handoff


def request_recovery_keeper_action(config, action, *, timeout=240):
    """Tray submits one explicit request to the exact live retained recovery actor."""
    _require(action in {"start", "stop", "restart"} and timeout > 0, "Explicit bounded recovery lifecycle action required")
    state = config.paths.state.absolute()
    descriptor_path = state / "formal-lifecycle/recovery-keeper.json"
    directory = state / "recovery-lifecycle"
    with process_lock(state / "recovery-lifecycle-submit.lock", reason="recovery_keeper_request_busy"):
        _submission_idle(state, directory)
        descriptor_sha = _sha(descriptor_path)
        descriptor = _read(descriptor_path)
        _require(descriptor.get("schemaVersion") == 1 and descriptor.get("kind") == DESCRIPTOR_KIND
                 and descriptor.get("admission") == "idle" and descriptor.get("automaticRestart") is False
                 and descriptor.get("runtimeAcceptance") is False and descriptor.get("historicalHoldsReleased") is False,
                 "A live idle independently reviewed recovery keeper is required")
        release, target = _release(config, descriptor.get("release"), current_source=False)
        keeper = descriptor.get("keeper")
        _require(observe_live_identity(keeper) is not None, "Recovery keeper identity is unavailable")
        _keeper_argv(config.paths.repo.absolute(), keeper, descriptor["release"], release["config"]["sha256"])
        _require(str(windows_boot_filetime()) == descriptor.get("bootFileTime")
                 and descriptor.get("protectedHolds") == release["protectedHolds"], "Recovery keeper boot or protected holds changed")
        _runtime_receipt(config, descriptor, release, target)
        for name in _LEGACY_FILES:
            path = state / name
            _require((_sha(path) if path.exists() else None) == release["expectedLifecycleFence"].get(name),
                     "Original formal acceptance/keeper lineage changed")
        _require(_read(state / "deployment-operation.json") == descriptor["runtime"]["operation"]
                 and _read(state / "deployment/host.json") == descriptor["runtime"]["host"]
                 and _read(state / "deployment/driver.json") == descriptor["runtime"]["selected"],
                 "Public request cannot adopt a different current runtime")
        _require((action == "start" and descriptor["runtime"]["state"] == "stopped")
                 or (action in {"stop", "restart"} and descriptor["runtime"]["state"] == "running"),
                 "Action does not match the authoritative recovery runtime state")
        _require(_sha(descriptor_path) == descriptor_sha, "Recovery keeper descriptor changed during request verification")
        reference, request_id = _new_request(state, descriptor, descriptor_sha, action)
        deadline = time.monotonic() + timeout
        result_path = directory / "results" / (request_id + ".json")
        while time.monotonic() < deadline:
            if result_path.exists():
                result = _reference(state, _read(result_path))
                _require(result.get("kind") == "recovery-keeper-command-result" and result.get("request") == reference
                         and result.get("keeper") == keeper and result.get("release") == descriptor["release"]
                         and result.get("status") in {"stopped", "running-acceptance-pending"}
                         and result.get("runtimeAcceptance") is False and result.get("historicalHoldsReleased") is False,
                         "Recovery lifecycle result is not bound to this request")
                return result
            command_path = directory / "command.json"
            if command_path.exists() and _read(command_path).get("status") == "unknown":
                raise JenkinsError("recovery_keeper_command_unknown", "Preserve this request for manual proof recovery",
                                   details={"request": reference})
            time.sleep(.1)
        raise JenkinsError("recovery_keeper_command_pending", "Reconcile this original request without resubmission",
                           details={"request": reference}, retryable=True)


__all__ = ["RecoveryKeeper", "RecoveryKeeperStart", "request_recovery_keeper_action"]

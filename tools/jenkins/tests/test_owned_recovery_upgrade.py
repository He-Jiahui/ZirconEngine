"""Owned transfer checks use in-memory evidence and mocked Windows observations."""
from __future__ import annotations

import base64
from contextlib import ExitStack
from copy import deepcopy
from dataclasses import FrozenInstanceError
import hashlib
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import owned_upgrade as upgrade
from tools.jenkins.deployment import lost_owner, recovery_keeper


ROOT = Path(__file__).absolute().parents[3]


class StoppedOwner:
    """Genuine owned Stop structure; no subprocesses, database, or temp files."""
    def __init__(self, *, controller_proof=None, agent_proof=None, strict_spawn=None, retirement=None):
        self.files = {}
        self.state, self.home = ROOT / ".jenkins/state", ROOT / ".jenkins/jenkins_home"
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state, home=self.home),
            spec=SimpleNamespace(path=ROOT / ".jenkins/deployment-spec.json",
                controller={"listenAddress": "127.0.0.1", "httpPort": 18080}),
            java=ROOT / ".jenkins/runtime/jdk/bin/java.exe", war=ROOT / ".jenkins/runtime/jenkins.war")
        self.request_id, self.stop_id, self.boot = "2" * 32, "1" * 32, "134358454705000000"
        self.runtime = "start-200"
        self.paths = {name: self.state / relative for name, relative in upgrade._ARTIFACTS.items()}
        self.identities = {role: {"pid": pid, "creationTime": str(int(self.boot) + index + 100),
            "executable": str(self.manager.java) if role in {"controller", "agent"} else str(ROOT / ".jenkins/runtime/python/python.exe"),
            "commandDigest": str(index + 1) * 64} for index, (role, pid) in enumerate(
                zip(("keeper", "supervisor", "controller", "agent"), (33448, 32656, 32464, 43344)))}
        self.holds = [{"domain": domain, "key": str(index), "version": 1,
                       "payload": {"status": "active", "owner": "legacy-" + str(index // 2)}}
            for index, domain in enumerate(("pool_writer_hold", "resource_reservation") * 2)]
        self.execution = {"recordsDigest": "8" * 64, "protectedHolds": deepcopy(self.holds),
            "protectedHoldsDigest": digest(self.holds), "resourceInventoryDigest": "9" * 64,
            "classification": {"activeExecutions": [], "historicalExecutions": [
                {"executionId": "legacy-0", "status": "quarantined", "nativeTerminationClaimed": False}],
                "protectedHolds": deepcopy(self.holds)}, "nativeProofIssued": False, "executionHoldsReleaseAllowed": False}
        self.targets = {letter * 64: {"driverDigest": letter * 64,
            "root": str(ROOT / ".jenkins/runtime/drivers" / (letter * 64)),
            "launcher": str(ROOT / ".jenkins/runtime/drivers" / (letter * 64) / "launcher.py"),
            "manifest": str(ROOT / ".jenkins/runtime/drivers" / (letter * 64) / "driver-manifest.json"),
            "steps": str(ROOT / ".jenkins/runtime/drivers" / (letter * 64) / "pipeline/steps.groovy")}
            for letter in ("a", "b")}
        self.selected = {**self.targets["a" * 64], "runtimeOperationId": self.runtime}
        config = self.put(ROOT / ".codex/config.toml", {"profile": "zircon-strict"})
        spec = self.put(self.manager.spec.path, {"spec": "deployment"})
        workflow = self.put(ROOT / ".jenkins/workflow-spec.json", {"spec": "workflow"})
        operators = {}
        for name in (*recovery_keeper._OPERATOR_FILES, "tools/jenkins/deployment/owned_upgrade.py"):
            path = ROOT / name
            operators[str(path)] = self.put(path, {"source": name})["sha256"]
        holder = self.state / "migration-baseline/jenkins_desktop_holder.py"
        operators[str(holder)] = self.put(holder, {"source": "restricted-holder"})["sha256"]
        source_files = {Path(name).relative_to(ROOT).as_posix(): checksum for name, checksum in operators.items()
                        if Path(name).is_relative_to(ROOT / "tools/jenkins")}
        source_files.update({".jenkins/deployment-spec.json": spec["sha256"], ".jenkins/workflow-spec.json": workflow["sha256"]})
        self.regression_doc = {"scope": "complete-current-Jenkins-support-pytest-discovery", "status": "passed",
            "sourceIdentityStable": True, "testsRun": 800, "exitCode": 0, "failures": 0, "errors": 0,
            "collectionErrors": [], "sourceFiles": source_files}
        self.regression_ref = self.put(self.state / "migration-baseline/full-passed.json", self.regression_doc)
        self.fence = {}
        for name in upgrade._LEGACY:
            self.fence[name] = self.put(self.state / name, {"legacy": name})["sha256"]
        runtime_files = {}
        for path in [self.manager.java, self.manager.war, *[ROOT / ".jenkins/runtime/python" / name for name in
                ("python.exe", "pythonw.exe", "python3.dll", "python314.dll", "vcruntime140.dll", "vcruntime140_1.dll")],
                *[ROOT / ".jenkins/runtime/jdk" / name for name in ("bin/server/jvm.dll", "lib/modules", "release")]]:
            runtime_files[str(path)] = self.put(path, {"binary": path.name})["sha256"]
        sandbox_path = ROOT / ".jenkins/runtime/test-host/codex.exe"
        sandbox = {"path": str(sandbox_path), "sha256": self.put(sandbox_path, {"binary": "official-host"})["sha256"],
                   "profile": "zircon-strict", "includeManagedConfig": True}
        old_preparation = self.put(self.state / "migration-baseline/old-preparation.json",
            {"schemaVersion": 1, "state": "prepared-not-selected", "driver": self.targets["a" * 64],
             "protectedHolds": self.holds, "supportRegression": self.regression_ref})
        self.old_release = {"schemaVersion": 1, "kind": upgrade._OLD_RELEASE_KIND, "scope": "execution-runtime",
            "requestId": "0" * 32, "automaticRestart": False, "queueReplayRequested": False,
            "runtimeAcceptance": False, "cargoAcceptance": False, "config": config, "deploymentSpec": spec,
            "workflowSpec": workflow, "protectedHolds": self.holds, "targetDriver": self.targets["a" * 64],
            "operatorFiles": operators, "runtimeFiles": runtime_files, "sandboxHost": sandbox,
            "regression": self.regression_ref, "preparation": old_preparation, "expectedLifecycleFence": self.fence}
        self.old_release_ref = self.put(self.state / "deployment/old-release.json", self.old_release)
        self.old_args = [self.identities["keeper"]["executable"], "-B", str(ROOT / "tools/dev/jenkins_recovery_keeper.py"),
            "--config-sha256", config["sha256"], "--release-file", self.old_release_ref["path"],
            "--release-sha256", self.old_release_ref["sha256"]]
        self.identities["keeper"]["commandDigest"] = hashlib.sha256("\0".join(self.old_args).encode()).hexdigest()
        running_operation = {"schemaVersion": 2, "state": "running", "operationId": self.runtime, "generation": self.runtime,
            "component": "lifecycle-host", "controlPlaneOnly": False, "homePath": str(self.home),
            "hostPid": self.identities["supervisor"]["pid"], **self.identities["supervisor"]}
        running_host = {"status": "running", "operationId": self.runtime, "generation": self.runtime,
            "hostPid": self.identities["supervisor"]["pid"], "identity": self.identities["controller"],
            "controllerIdentity": self.identities["controller"], "agentIdentity": self.identities["agent"],
            "agentLaunchAttempted": True, "controlPlaneOnly": False}
        running_runtime = {"state": "running", "operation": running_operation, "host": running_host,
            "selected": self.selected, "owner": {name: self.identities[name] for name in ("supervisor", "controller", "agent")}}
        probe = {"requestedProfile": "zircon-strict", "configuration": {**config, "updateOpenDenied": True,
            "sha256Before": config["sha256"], "sha256After": config["sha256"]},
            "holderIdentity": self.identities["keeper"], "allowedProbe": {"path": str(self.state / "desktop-holder" / ("probe-" + "4" * 32 + ".tmp")),
                "exclusiveCreateWrite": True, "removed": True}, "associatedJob": {"query": "current-process-associated-job",
                "holderMember": True, "memberPids": [self.identities["keeper"]["pid"], self.identities["supervisor"]["pid"]]}}
        spawn = {"creator": self.identities["keeper"], "child": self.identities["supervisor"], "creationFlags": 0x08000200,
            "requestedProfile": "zircon-strict", "creatorInJob": True, "childInJob": True,
            "actualRestrictionPreflight": probe, "associatedJobAtSpawn": probe, **(strict_spawn or {})}
        self.origin_ref = self.put(self.state / "recovery-lifecycle/history/running-receipt.json",
            {"schemaVersion": 1, "kind": "recovery-keeper-runtime", "keeper": self.identities["keeper"],
                "release": self.old_release_ref, "bootFileTime": self.boot, "runtime": running_runtime,
                "strictSpawn": spawn, "queueReplayRequested": False, "runtimeAcceptance": False})
        base_descriptor = {"schemaVersion": 1, "kind": upgrade._DESCRIPTOR_KIND, "admission": "idle",
            "keeper": self.identities["keeper"], "release": self.old_release_ref, "bootFileTime": self.boot,
            "runtime": running_runtime, "runtimeReceipt": self.origin_ref, "initialRecovery": None,
            "recoveryDigest": None, "failedOwnStartInterruption": None, "protectedHolds": self.holds,
            "automaticRestart": False, "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        self.stop_request_ref = self.put(self.state / "recovery-lifecycle/history/public-stop-request.json",
            {"schemaVersion": 1, "kind": upgrade._REQUEST_KIND, "requestId": self.stop_id, "action": "stop",
             "keeper": self.identities["keeper"], "release": self.old_release_ref, "runtimeReceipt": self.origin_ref,
             "descriptorSha256": hashlib.sha256(canonical_json(base_descriptor) + b"\n").hexdigest(),
             "protectedHoldsDigest": digest(self.holds), "runtimeAcceptance": False})
        controller = self.proof(self.identities["controller"], **(controller_proof or {}))
        agent = self.proof(self.identities["agent"], **(agent_proof or {}))
        transport_result = {"status": "stopped", "operationId": self.runtime, "generation": self.runtime,
            "identity": self.identities["controller"], "nativeTerminationProof": controller, "agentTerminationProof": agent}
        self.host = {**transport_result, **{name: running_host[name] for name in
                     ("hostPid", "controllerIdentity", "agentIdentity", "agentLaunchAttempted", "controlPlaneOnly")}}
        result_path = self.state / "deployment/commands/stop-200.result.json"
        result_ref = self.put(result_path, transport_result)
        result_archive = self.copy(result_path, self.state / "deployment/history/stop-result-original.json")
        native_command = {"action": "stop", "operationId": self.runtime, "generation": self.runtime,
                          "hostPid": self.identities["supervisor"]["pid"], "controllerIdentity": self.identities["controller"]}
        command_ref = self.put(self.state / "deployment/commands/stop-200.json", native_command)
        self.binding_ref = self.put(self.state / "deployment/history/owned-stop-binding.json",
            {"schemaVersion": 1, "kind": "owned-stop-metadata-binding", "capturedOwner": {
                "operation": running_operation, "host": running_host, "homePath": str(self.home)},
             "terminalHost": self.host, "command": native_command, "commandSha256": command_ref["sha256"],
             "rawStopResult": {"path": str(result_path), "archivePath": result_archive["path"], "sha256": result_ref["sha256"],
                 "base64": base64.b64encode(self.files[str(result_path)]).decode()}, "terminalHostRawSha256": result_ref["sha256"],
             "hostDeparture": self.absent(self.identities["supervisor"]),
             "nativeProofSynthesized": False, "historicalHoldsReleased": False, "runtimeAcceptance": False})
        self.operation = {**running_operation, "state": "stopped", "serviceReady": False, "health": False,
            "nativeTerminationProof": controller, "agentTerminationProof": agent,
            "stopBinding": self.binding_ref, "stopResult": result_archive}
        stopped_runtime = {**running_runtime, "state": "stopped", "operation": self.operation, "host": self.host}
        self.stop_ref = self.put(self.state / "recovery-lifecycle/history/public-owned-stop.json",
            {"schemaVersion": 1, "kind": "recovery-keeper-owned-stop", "request": self.stop_request_ref,
             "keeper": self.identities["keeper"], "release": self.old_release_ref, "bootFileTime": self.boot,
             "operation": self.operation, "host": self.host, "selected": self.selected, "previousRuntimeReceipt": self.origin_ref,
             "stopBinding": self.binding_ref, "protectedHolds": self.holds, "initialRecovery": None, "recoveryDigest": None,
             "failedOwnStartInterruption": None, **{name: False for name in upgrade._FLAGS}})
        self.descriptor = {**base_descriptor, "runtime": stopped_runtime, "runtimeReceipt": self.stop_ref}
        self.put(self.paths["stoppedDescriptor"], self.descriptor)
        self.stopped_descriptor_ref = self.copy(self.paths["stoppedDescriptor"], self.state / "recovery-lifecycle/history/stopped-descriptor.json")
        stopped_result_ref = self.put(self.state / "recovery-lifecycle/history/public-stop-result.json",
            {"schemaVersion": 1, "kind": "recovery-keeper-command-result", "status": "stopped",
             "request": self.stop_request_ref, "release": self.old_release_ref, "keeper": self.identities["keeper"],
             "runtimeReceipt": self.stop_ref, "operationId": self.runtime,
             "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False})
        self.terminal_command = {"schemaVersion": 1, "kind": upgrade._COMMAND_KIND, "status": "stopped",
            "request": self.stop_request_ref, "release": self.old_release_ref, "keeper": self.identities["keeper"],
            "previousRuntimeReceipt": self.origin_ref, "recoveryDigest": None, "result": stopped_result_ref,
            "runtimeAcceptance": False, "historicalHoldsReleased": False}
        self.put(self.paths["terminalCommand"], self.terminal_command)
        for area, reference in (("requests", self.stop_request_ref), ("results", stopped_result_ref)):
            self.put(self.state / "recovery-lifecycle" / area / (self.stop_id + ".json"), reference)
        for name, document in (("stoppedOperation", self.operation), ("stoppedHost", self.host), ("previousSelector", self.selected)):
            self.put(self.paths[name], document)
        for name in ("deployment-operation.json", "deployment/host.json", "deployment/driver.json"):
            self.fence[name] = self.ref(self.state / name)["sha256"]
        # Persisted predecessor release binds only the five legacy fence names.
        self.retirement = {"schemaVersion": 1, "kind": upgrade.RETIREMENT_KIND, "requestId": self.request_id,
            "keeper": self.identities["keeper"], "release": self.old_release_ref, "stoppedDescriptor": self.stopped_descriptor_ref,
            "ownedStop": self.stop_ref, "bootFileTime": self.boot, "method": "forced-terminate-held-handle",
            "beforeIdentity": self.identities["keeper"], "beforeArguments": self.old_args,
            "handleIdentity": {name: self.identities["keeper"][name] for name in ("pid", "creationTime", "executable")},
            "handleIdentityVerified": True, "nativeHandleRetained": True, "waitResult": 0, "processExitCode": 19,
            "afterDeparture": self.absent(self.identities["keeper"]), "serviceStoppedBeforeRetirement": True,
            "forced": True, "graceful": False, "terminateProcess": {"api": "TerminateProcess", "succeeded": True, "requestedExitCode": 19},
            **{name: False for name in upgrade._FLAGS}, **(retirement or {})}
        self.retirement_path = self.state / "recovery-lifecycle/history/owned-keeper-retirement.json"
        self.retirement_ref = self.put(self.retirement_path, self.retirement)
        self.owner = {"operation": {name: self.operation[name] for name in
                ("operationId", "generation", "hostPid", "homePath", "component", "controlPlaneOnly")},
            **{name: self.identities[name] for name in ("supervisor", "controller", "agent")}}
        preparation_ref = self.put(self.state / "migration-baseline/new-preparation.json",
            {"schemaVersion": 1, "state": "prepared-not-selected", "driver": self.targets["b" * 64],
             "protectedHolds": self.holds, "supportRegression": self.regression_ref,
             "previousSelectedDriver": self.selected, "previousSelectorSha256": self.fence["deployment/driver.json"]})
        self.successor = {**self.old_release, "kind": upgrade.RELEASE_KIND, "requestId": self.request_id,
            "targetDriver": self.targets["b" * 64], "preparation": preparation_ref, "expectedPreviousOwner": self.owner,
            "expectedLifecycleFence": deepcopy(self.fence), "ownedUpgrade": self.marker}
        self.successor_path = self.state / "deployment/successor-release.json"
        self.successor_ref = self.put(self.successor_path, self.successor)
        self.files[str(self.home / "config.xml")] = b"<hudson><numExecutors>0</numExecutors></hudson>"
        self.files[str(self.home / "queue.xml")] = b"<queue><items/><pendingItems/></queue>"

    @staticmethod
    def proof(expected_identity, **changes):
        return {"identity": expected_identity, "complete": True, "childrenGone": True, "activeProcesses": 0,
                "stdoutEof": True, "stderrEof": True, "processExitCode": 0, **changes}

    @staticmethod
    def absent(identity):
        return {"pid": identity["pid"], "expectedCreationTime": identity["creationTime"], "status": "absent"}

    def put(self, path, document):
        self.files[str(path)] = canonical_json(document) + b"\n"
        return self.ref(path)

    def ref(self, path):
        return {"path": str(path), "sha256": hashlib.sha256(self.files[str(path)]).hexdigest()}

    def copy(self, source, destination):
        self.files[str(destination)] = self.files[str(source)]
        return self.ref(destination)

    @property
    def marker(self):
        return {"stoppedDescriptor": self.stopped_descriptor_ref, "ownedStop": self.stop_ref, "keeperRetirement": self.retirement_ref}

    def update_retirement(self, **changes):
        self.retirement.update(changes)
        self.retirement_ref = self.put(self.retirement_path, self.retirement)
        self.successor["ownedUpgrade"] = self.marker
        self.successor_ref = self.put(self.successor_path, self.successor)

    def prepare(self):
        return upgrade.prepare_owned_upgrade_start(self.manager, request_id=self.request_id,
            successor_release_ref=self.successor_ref, stopped_descriptor_ref=self.stopped_descriptor_ref,
            owned_stop_ref=self.stop_ref, keeper_retirement_ref=self.retirement_ref)

    def observations(self):
        stack = ExitStack()
        for module in (upgrade, lost_owner, recovery_keeper):
            stack.enter_context(patch.object(module, "_raw", side_effect=lambda path: self.files[str(Path(path).absolute())]))
            stack.enter_context(patch.object(module, "_plain"))
        stack.enter_context(patch.object(Path, "exists", autospec=True, side_effect=lambda path: str(path) in self.files))
        stack.enter_context(patch.object(upgrade, "verify_driver", side_effect=lambda root, expected_digest: self.targets[expected_digest]))
        self.regression = stack.enter_context(patch.object(upgrade, "verify_regression", return_value=self.regression_doc))
        stack.enter_context(patch.object(upgrade, "_historical_support", return_value=self.regression_doc))
        self.departure = stack.enter_context(patch.object(upgrade, "observe_departed_identity", side_effect=self.absent))
        stack.enter_context(patch.object(recovery_keeper, "observe_departed_identity", side_effect=self.absent))
        self.boot_probe = stack.enter_context(patch.object(upgrade, "windows_boot_filetime", return_value=int(self.boot)))
        self.inventory = stack.enter_context(patch.object(upgrade, "runtime_owner_inventory", return_value=[]))
        self.home_probe = stack.enter_context(patch.object(upgrade, "observe_home_available",
            return_value={"exclusiveReadProbes": [{"file": "config.xml", "bytes": 52, "modifiedNs": 100}], "staleMarkersDeleted": False}))
        self.endpoint = stack.enter_context(patch.object(upgrade, "check_port_available"))
        self.execution_probe = stack.enter_context(patch.object(lost_owner, "_execution_gate", return_value=self.execution))
        stack.enter_context(patch.object(upgrade, "protected_holds", side_effect=lambda state: deepcopy(self.holds)))
        self.self_pid = stack.enter_context(patch.object(upgrade.os, "getpid", return_value=12345))
        self.self_identity = stack.enter_context(patch.object(recovery_keeper, "_full_identity"))
        self.self_argv = stack.enter_context(patch.object(recovery_keeper, "_keeper_argv"))
        return stack

    def replacement(self, plan):
        artifacts = plan.original_artifacts
        archive_refs = {}
        for name in ("stoppedDescriptor", "terminalCommand"):
            path = self.state / "recovery-lifecycle/history" / ("handoff-" + name + ".json")
            self.files[str(path)] = artifacts[name]["raw"]
            archive_refs[name] = self.ref(path)
        keeper = {**self.identities["keeper"], "pid": 12345, "creationTime": str(int(self.boot) + 300)}
        self.self_identity.return_value = keeper
        request = self.put(self.state / "recovery-lifecycle/history/owned-upgrade-request.json",
            {"schemaVersion": 1, "kind": upgrade._REQUEST_KIND, "requestId": self.request_id, "action": "upgrade",
             "keeper": keeper, "release": self.successor_ref, "ownedUpgradeDigest": plan.upgrade_digest,
             "protectedHoldsDigest": digest(self.holds), "runtimeAcceptance": False})
        handoff = self.put(self.state / "recovery-lifecycle/history/owned-upgrade-handoff.json",
            {"schemaVersion": 1, "kind": upgrade.HANDOFF_KIND, "outcome": "owned-transfer", "keeper": keeper,
             "release": self.successor_ref, "request": request, "ownedUpgradeDigest": plan.upgrade_digest,
             "previousKeeper": plan.document["previousKeeper"], "previousRelease": plan.document["previousRelease"],
             "ownedStop": self.stop_ref, "keeperRetirement": self.retirement_ref, **archive_refs,
             **{name: False for name in upgrade._FLAGS}})
        arguments = {"keeper": keeper, "release_ref": self.successor_ref, "request": request,
            "descriptor_archive": archive_refs["stoppedDescriptor"], "command_archive": archive_refs["terminalCommand"], "handoff_ref": handoff}
        command = {"schemaVersion": 1, "kind": upgrade._COMMAND_KIND, "status": "pending", "keeper": keeper,
            "release": self.successor_ref, "request": request, "ownedUpgradeDigest": plan.upgrade_digest,
            "replacesStoppedDescriptor": archive_refs["stoppedDescriptor"], "replacesTerminalCommand": archive_refs["terminalCommand"],
            "ownedUpgradeHandoff": handoff, "previousRuntimeReceipt": None,
            "runtimeAcceptance": False, "historicalHoldsReleased": False}
        return arguments, command


class OwnedRecoveryUpgradeTests(unittest.TestCase):
    def test_genuine_stop_and_forced_keeper_retirement_transfer_without_rewriting_evidence(self):
        scenario = StoppedOwner()
        before = deepcopy(scenario.files)
        with scenario.observations():
            plan = scenario.prepare()
            evidence = plan.revalidate(scenario.manager)
            self.assertEqual("owned-transfer", evidence["outcome"])
            self.assertEqual("forced-terminate-held-handle", evidence["retirementMethod"])
            self.assertEqual(scenario.holds, evidence["executionQuarantine"]["protectedHolds"])
            self.assertEqual(scenario.execution, evidence["executionQuarantine"])
            self.assertEqual(set(upgrade._ARTIFACTS), set(plan.original_artifacts))
            self.assertEqual(scenario.files[str(scenario.paths["terminalCommand"])], plan.original_artifacts["terminalCommand"]["raw"])
            self.assertEqual(2, scenario.regression.call_count)
            for key in (*upgrade._FLAGS, "nativeProofIssued", "executionHoldsReleaseAllowed", "automaticRetry"):
                self.assertIs(evidence[key], False)
        self.assertEqual(before, scenario.files)

    def test_json_and_subclasses_cannot_issue_capability_and_frozen_values_are_detached(self):
        scenario = StoppedOwner()
        with scenario.observations():
            plan = scenario.prepare()
            with self.assertRaises(JenkinsError):
                upgrade.OwnedRecoveryUpgradePlan(canonical_json(plan.document))
            with self.assertRaises(FrozenInstanceError):
                plan.snapshot = b"forged"
            detached = plan.document
            detached["owner"]["controller"]["pid"] = 1
            self.assertEqual(scenario.identities["controller"]["pid"], plan.document["owner"]["controller"]["pid"])
            class Forged(upgrade.OwnedRecoveryUpgradePlan):
                pass
            forged = object.__new__(Forged)
            with self.assertRaises(JenkinsError):
                forged.revalidate(scenario.manager)

    def test_component_native_proofs_require_complete_eof_and_exact_identity(self):
        changes = ({"complete": False}, {"stdoutEof": False}, {"stderrEof": False}, {"childrenGone": False},
                   {"activeProcesses": 1}, {"activeProcesses": False}, {"processExitCode": None}, {"identity": {"pid": 1}})
        for role in ("controller_proof", "agent_proof"):
            for change in changes:
                with self.subTest(role=role, change=change):
                    scenario = StoppedOwner(**{role: change})
                    with scenario.observations(), self.assertRaises(JenkinsError):
                        scenario.prepare()

    def test_pid_absence_cannot_substitute_for_actual_inherited_strict_creation(self):
        for changes in ({"childInJob": False}, {"creatorInJob": False}, {"creationFlags": 0x01000000},
                        {"requestedProfile": "full-access"}):
            with self.subTest(changes=changes):
                scenario = StoppedOwner(strict_spawn=changes)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_keeper_retirement_requires_held_handle_verified_birth_and_completed_wait(self):
        changes = ({"nativeHandleRetained": False}, {"handleIdentityVerified": False}, {"handleIdentity": {}},
            {"waitResult": 258}, {"waitResult": False}, {"processExitCode": 259}, {"serviceStoppedBeforeRetirement": False},
            {"beforeArguments": ["unknown-host.exe"]}, {"nativeProofSynthesized": True}, {"stdoutEof": True})
        for change in changes:
            with self.subTest(change=change):
                scenario = StoppedOwner(retirement=change)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_forced_retirement_cannot_be_relabelled_graceful(self):
        for change in ({"graceful": True}, {"forced": False}, {"method": "graceful-console-break"},
                       {"terminateProcess": {"api": "TerminateProcess", "succeeded": False, "requestedExitCode": 19}}):
            with self.subTest(change=change):
                scenario = StoppedOwner(retirement=change)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_graceful_retirement_requires_real_verified_console_group(self):
        scenario = StoppedOwner()
        signal = {"api": "GenerateConsoleCtrlEvent", "event": "CTRL_BREAK_EVENT", "succeeded": True,
            "groupIdentity": scenario.identities["keeper"], "memberPids": [scenario.identities["keeper"]["pid"]], "groupVerified": True}
        scenario.retirement.pop("terminateProcess")
        scenario.update_retirement(method="graceful-console-break", forced=False, graceful=True,
            processExitCode=0, consoleSignal=signal)
        with scenario.observations():
            self.assertEqual("graceful-console-break", scenario.prepare().document["retirementMethod"])
        scenario.update_retirement(consoleSignal={**signal, "groupVerified": False})
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_every_exact_old_process_must_depart_even_with_native_stop_receipts(self):
        for role in ("keeper", "supervisor", "controller", "agent"):
            with self.subTest(role=role):
                scenario = StoppedOwner()
                with scenario.observations():
                    def departed(identity):
                        if identity == scenario.identities[role]:
                            raise JenkinsError("deployment_owner_live", "Exact owned process remains live")
                        return scenario.absent(identity)
                    scenario.departure.side_effect = departed
                    with self.assertRaises(JenkinsError):
                        scenario.prepare()

    def test_newer_pid_reuse_is_bound_and_older_or_equal_reuse_is_rejected(self):
        scenario = StoppedOwner()
        with scenario.observations():
            def reused(identity):
                return {**scenario.absent(identity), "status": "pid-reused", "observedIdentity": {
                    "pid": identity["pid"], "creationTime": str(int(identity["creationTime"]) + 1000), "executable": "foreign.exe"}}
            scenario.departure.side_effect = reused
            self.assertEqual("pid-reused", scenario.prepare().document["departedIdentities"]["keeper"]["status"])
            scenario.departure.side_effect = lambda identity: {**reused(identity), "observedIdentity": identity}
            with self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_boot_foreign_runtime_home_and_endpoint_changes_block(self):
        for observation in ("boot", "inventory", "home", "endpoint"):
            with self.subTest(observation=observation):
                scenario = StoppedOwner()
                with scenario.observations():
                    if observation == "boot":
                        scenario.boot_probe.side_effect = [int(scenario.boot), int(scenario.boot) + 1]
                    elif observation == "inventory":
                        scenario.inventory.return_value = [{"pid": 77, "creationTime": scenario.boot, "executable": "unknown.exe"}]
                    elif observation == "home":
                        scenario.home_probe.side_effect = JenkinsError("deployment_home_busy", "Home is in use")
                    else:
                        scenario.endpoint.side_effect = JenkinsError("controller_port_unavailable", "Endpoint is owned")
                    with self.assertRaises(JenkinsError):
                        scenario.prepare()

    def test_unknown_or_non_stop_public_command_is_not_an_upgrade_source(self):
        for status in ("unknown", "committed", "stop-attempted"):
            with self.subTest(status=status):
                scenario = StoppedOwner()
                scenario.put(scenario.paths["terminalCommand"], {**scenario.terminal_command, "status": status})
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()

    def test_raw_stop_transport_cannot_be_replaced_by_projected_json(self):
        scenario = StoppedOwner()
        path = scenario.state / "deployment/commands/stop-200.result.json"
        scenario.files[str(path)] += b" "
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_stopped_descriptor_archive_and_canonical_raw_must_be_identical(self):
        scenario = StoppedOwner()
        scenario.files[str(scenario.paths["stoppedDescriptor"])] += b" "
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_full_current_regression_is_mandatory(self):
        scenario = StoppedOwner()
        with scenario.observations():
            scenario.regression.side_effect = JenkinsError("execution_transition_source_changed", "Current source differs")
            with self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_historical_launcher_uses_tested_old_literal_without_executing_old_source(self):
        scenario = StoppedOwner()
        target = scenario.targets["a" * 64]
        driver_path = Path(target["root"]) / "tools/jenkins/deployment/driver.py"
        driver_raw = b"_LAUNCHER = 'original reviewed launcher\\n'\nraise RuntimeError('must not execute historical source')\n"
        scenario.files[str(driver_path)] = driver_raw
        names = ["tools/jenkins/contracts/__init__.py", "tools/jenkins/deployment/driver.py",
                 "tools/jenkins/deployment/manager.py", "tools/jenkins/deployment/recovery_keeper.py",
                 "tools/jenkins/deployment/start_transaction.py", "tools/jenkins/processes/native.py",
                 "tools/jenkins/processes/job.py", "tools/jenkins/tests/test_original.py"]
        sources = {name: "3" * 64 for name in names}
        sources["tools/jenkins/deployment/driver.py"] = hashlib.sha256(driver_raw).hexdigest()
        sources[".jenkins/deployment-spec.json"] = scenario.old_release["deploymentSpec"]["sha256"]
        sources[".jenkins/workflow-spec.json"] = scenario.old_release["workflowSpec"]["sha256"]
        log = scenario.state / "migration-baseline/historical-pass.log"
        scenario.files[str(log)] = b"complete original regression passed"
        historical = {**scenario.regression_doc, "sourceFiles": sources, "log": str(log),
                      "logSha256": hashlib.sha256(scenario.files[str(log)]).hexdigest()}
        reference = scenario.put(scenario.state / "migration-baseline/old-full-support.json", historical)
        expected = {name: {"sha256": checksum} for name, checksum in sources.items() if "/tests/" not in name and not name.startswith(".jenkins/")}
        expected["launcher.py"] = {"sha256": hashlib.sha256(b"original reviewed launcher\n").hexdigest()}
        expected["tools/__init__.py"] = {"sha256": hashlib.sha256(b'"""This package belongs to one verified Jenkins snapshot."""\n').hexdigest()}
        scenario.put(Path(target["manifest"]), {"schemaVersion": 1, "driverDigest": target["driverDigest"], "files": expected})
        with ExitStack() as stack:
            stack.enter_context(patch.object(upgrade, "_raw", side_effect=lambda path: scenario.files[str(Path(path).absolute())]))
            self.assertEqual(historical, upgrade._historical_support(ROOT, scenario.state,
                {**scenario.old_release, "regression": reference}, target))
            expected["launcher.py"] = {"sha256": "f" * 64}
            scenario.put(Path(target["manifest"]), {"schemaVersion": 1, "driverDigest": target["driverDigest"], "files": expected})
            with self.assertRaises(JenkinsError):
                upgrade._historical_support(ROOT, scenario.state, {**scenario.old_release, "regression": reference}, target)

    def test_revalidation_refuses_raw_anchor_and_unknown_native_or_budget_changes(self):
        for change in ("raw-operation", "unknown-native", "resource-budget", "historical-hold"):
            with self.subTest(change=change):
                scenario = StoppedOwner()
                with scenario.observations():
                    plan = scenario.prepare()
                    if change == "raw-operation":
                        scenario.files[str(scenario.paths["stoppedOperation"])] += b" "
                    elif change == "unknown-native":
                        scenario.execution["recordsDigest"] = "f" * 64
                    elif change == "resource-budget":
                        scenario.execution["resourceInventoryDigest"] = "f" * 64
                    else:
                        scenario.holds[0]["version"] += 1
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)

    def test_current_execution_with_unresolved_native_authority_blocks_new_jobs(self):
        scenario = StoppedOwner()
        scenario.execution["classification"]["activeExecutions"] = ["current-native-unknown"]
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_exact_actor_transition_preserves_old_descriptor_and_normal_terminal_stop(self):
        scenario = StoppedOwner()
        with scenario.observations():
            plan = scenario.prepare()
            original_descriptor = scenario.files[str(scenario.paths["stoppedDescriptor"])]
            arguments, command = scenario.replacement(plan)
            transitioned = plan.authorize_keeper_command(scenario.manager, **arguments)
            self.assertEqual(plan.snapshot, transitioned.snapshot)
            self.assertEqual(plan.upgrade_digest, transitioned.upgrade_digest)
            scenario.put(scenario.paths["terminalCommand"], command)
            self.assertEqual("owned-transfer", transitioned.revalidate(scenario.manager)["outcome"])
            scenario.put(scenario.paths["terminalCommand"], {**command, "status": "start-attempted"})
            transitioned.revalidate(scenario.manager)
            self.assertEqual(original_descriptor, scenario.files[str(scenario.paths["stoppedDescriptor"])])
            self.assertEqual("stopped", upgrade._document(transitioned.original_artifacts["terminalCommand"]["raw"])["status"])
            with self.assertRaises(JenkinsError):
                plan.revalidate(scenario.manager)
            with self.assertRaises(JenkinsError):
                transitioned.authorize_keeper_command(scenario.manager, **arguments)

    def test_changed_archive_handoff_request_or_successor_command_is_rejected(self):
        for field in ("keeper", "request", "release", "ownedUpgradeDigest", "replacesStoppedDescriptor",
                      "replacesTerminalCommand", "ownedUpgradeHandoff", "previousRuntimeReceipt", "createdSupervisor", "result"):
            with self.subTest(field=field):
                scenario = StoppedOwner()
                with scenario.observations():
                    plan = scenario.prepare()
                    arguments, command = scenario.replacement(plan)
                    transitioned = plan.authorize_keeper_command(scenario.manager, **arguments)
                    scenario.put(scenario.paths["terminalCommand"], {**command, field: {"forged": True}})
                    with self.assertRaises(JenkinsError):
                        transitioned.revalidate(scenario.manager)

    def test_immutable_handoff_archives_are_rechecked_after_command_replacement(self):
        for key in ("request", "descriptor_archive", "command_archive", "handoff_ref"):
            with self.subTest(key=key):
                scenario = StoppedOwner()
                with scenario.observations():
                    plan = scenario.prepare()
                    arguments, command = scenario.replacement(plan)
                    transitioned = plan.authorize_keeper_command(scenario.manager, **arguments)
                    scenario.put(scenario.paths["terminalCommand"], command)
                    scenario.files[arguments[key]["path"]] += b" "
                    with self.assertRaises(JenkinsError):
                        transitioned.revalidate(scenario.manager)

    def test_successor_actor_must_be_this_exact_process_and_canonical_reviewed_command(self):
        scenario = StoppedOwner()
        with scenario.observations():
            plan = scenario.prepare()
            arguments, _ = scenario.replacement(plan)
            scenario.self_identity.return_value = {**arguments["keeper"], "creationTime": "1"}
            with self.assertRaises(JenkinsError):
                plan.authorize_keeper_command(scenario.manager, **arguments)
            scenario.self_identity.return_value = arguments["keeper"]
            scenario.self_argv.side_effect = JenkinsError("recovery_keeper_unproven", "Exact command differs")
            with self.assertRaises(JenkinsError):
                plan.authorize_keeper_command(scenario.manager, **arguments)

    def test_used_public_request_cannot_issue_another_upgrade_capability(self):
        scenario = StoppedOwner()
        scenario.put(scenario.state / "recovery-lifecycle/results" / (scenario.request_id + ".json"), {"used": True})
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()


if __name__ == "__main__":
    unittest.main()

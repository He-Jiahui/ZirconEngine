"""Exact departed recovery ownership and unknown Stop contracts, in memory."""
from contextlib import ExitStack, nullcontext
from copy import deepcopy
import fnmatch
import hashlib
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import (lost_owner, lost_recovery_runtime as recovery,
                                      owned_upgrade, recovery_keeper, runtime_roles, startup_observations)

ROOT = Path(__file__).absolute().parents[3]


class DepartedRuntime:
    def __init__(self):
        self.state, self.home = ROOT / ".jenkins/state", ROOT / ".jenkins/jenkins_home"
        self.files, self.request_id, self.stop_id = {}, "1" * 32, "2" * 32
        self.management_rows = []
        self.boot, self.old_boot = 134361900000000000, "134361000000000000"
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state, home=self.home),
            spec=SimpleNamespace(path=ROOT / ".jenkins/deployment-spec.json",
                                 controller={"listenAddress": "127.0.0.1", "httpPort": 18080}),
            java=ROOT / ".jenkins/runtime/jdk/bin/java.exe", war=ROOT / ".jenkins/runtime/jenkins.war")
        self.identities = {name: {"pid": i + 11, "creationTime": str(int(self.old_boot) + i + 10),
            "executable": str(ROOT / (".jenkins/runtime/python/python.exe" if i < 2 else ".jenkins/runtime/jdk/bin/java.exe")),
            "commandDigest": str(i + 1) * 64} for i, name in enumerate(("keeper", "supervisor", "controller", "agent"))}
        self.runtime_id = "start-12345"
        self.operation = {"schemaVersion": 2, "state": "running", "operationId": self.runtime_id,
            "generation": self.runtime_id, "component": "lifecycle-host", "homePath": str(self.home),
            "hostPid": 12, "controlPlaneOnly": False, **self.identities["supervisor"]}
        self.host = {"status": "running", "operationId": self.runtime_id, "generation": self.runtime_id,
            "hostPid": 12, "controlPlaneOnly": False, "agentLaunchAttempted": True,
            "controllerIdentity": self.identities["controller"], "identity": self.identities["controller"],
            "agentIdentity": self.identities["agent"]}
        driver_root = ROOT / ".jenkins/runtime/drivers" / ("a" * 64)
        self.target = {"driverDigest": "a" * 64, "root": str(driver_root), "launcher": str(driver_root / "launcher.py"),
                       "manifest": str(driver_root / "driver-manifest.json"), "steps": str(driver_root / "pipeline/steps.groovy")}
        self.old_release_ref = self.put(self.state / "deployment/old-release.json", {"old": True})
        self.old_release = {"kind": "formal-owned-recovery-upgrade-release", "targetDriver": self.target,
            "actorCodeRoot": self.target["root"], "regression": self.put(self.state / "migration-baseline/regression.json", {"passed": True}),
            "preparation": self.put(self.state / "migration-baseline/preparation.json", {"prepared": True}),
            "config": {"path": str(ROOT / ".codex/config.toml"), "sha256": "8" * 64}}
        keeper_args = [str(ROOT / ".jenkins/runtime/python/python.exe"), "-B", str(ROOT / "tools/dev/jenkins_recovery_keeper.py"),
                       "--config-sha256", "8" * 64, "--release-file", self.old_release_ref["path"],
                       "--release-sha256", self.old_release_ref["sha256"]]
        supervisor_args = [str(ROOT / ".jenkins/runtime/python/python.exe"), self.target["launcher"], "__lifecycle-host",
                           "--spec", str(self.manager.spec.path), "--java", str(self.manager.java), "--war", str(self.manager.war),
                           "--operation-id", self.runtime_id]
        for name, args in (("keeper", keeper_args), ("supervisor", supervisor_args)):
            self.identities[name]["commandDigest"] = hashlib.sha256("\0".join(args).encode("utf-8")).hexdigest()
        self.operation["commandDigest"] = self.identities["supervisor"]["commandDigest"]
        self.selected = {**self.target, "runtimeOperationId": self.runtime_id, "recoveryRelease": self.old_release_ref,
                         "supportRegression": self.old_release["regression"], "preparation": self.old_release["preparation"]["path"]}
        self.holds = [{"domain": domain, "key": str(i), "version": 1,
                       "payload": {"owner": "legacy-" + str(i % 2), "status": "active"}}
                      for i, domain in enumerate(("pool_writer_hold", "pool_writer_hold", "resource_reservation", "resource_reservation"))]
        self.rows = [{"domain": "execution", "key": "legacy-" + str(i), "version": 1,
                      "payload": {"operationId": "old-start", "status": "running"}} for i in range(2)] + deepcopy(self.holds)
        runtime = {"state": "running", "operation": self.operation, "host": self.host, "selected": self.selected,
                   "owner": {key: self.identities[key] for key in ("supervisor", "controller", "agent")},
                   "nativeIdentities": [self.identities[key] for key in ("supervisor", "controller", "agent")]}
        self.origin = {"schemaVersion": 1, "kind": "recovery-keeper-runtime", "runtime": runtime,
            "keeper": self.identities["keeper"], "release": self.old_release_ref, "bootFileTime": self.old_boot,
            "protectedHolds": self.holds, "runtimeAcceptance": False, "cargoAcceptance": False,
            "historicalHoldsReleased": False, "nativeProofSynthesized": False, "queueReplayRequested": False,
            "strictSpawn": {"creator": self.identities["keeper"], "child": self.identities["supervisor"],
                "creationFlags": recovery_keeper.CREATION_FLAGS, "requestedProfile": "zircon-strict",
                "creatorInJob": True, "childInJob": True}}
        origin_ref = self.put(self.state / "recovery-lifecycle/history/runtime.json", self.origin)
        self.descriptor = {"schemaVersion": 1, "kind": recovery_keeper.DESCRIPTOR_KIND, "admission": "blocked",
            "runtime": runtime, "runtimeReceipt": origin_ref, "bootFileTime": self.old_boot,
            "keeper": self.identities["keeper"], "release": self.old_release_ref, "protectedHolds": self.holds,
            "automaticRestart": False, "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        request = {"schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "requestId": self.stop_id, "action": "stop",
            "keeper": self.identities["keeper"], "release": self.old_release_ref, "runtimeReceipt": origin_ref,
            "descriptorSha256": hashlib.sha256(canonical_json({**self.descriptor, "admission": "idle"}) + b"\n").hexdigest(),
            "protectedHoldsDigest": digest(self.holds), "runtimeAcceptance": False}
        self.request_ref = self.put(self.state / "recovery-lifecycle/history/stop-request.json", request)
        self.command = {"schemaVersion": 1, "kind": recovery_keeper.COMMAND_KIND, "status": "unknown",
            "reasonCode": "drain_unproven", "request": self.request_ref, "keeper": self.identities["keeper"],
            "release": self.old_release_ref, "previousRuntimeReceipt": origin_ref,
            "runtimeAcceptance": False, "historicalHoldsReleased": False}
        for name, value in (("originalCommand", self.command), ("originalDescriptor", self.descriptor),
                            ("previousOperation", self.operation), ("previousHost", self.host), ("previousSelector", self.selected)):
            self.put(self.state / recovery._POINTERS[name], value)
        self.put(self.state / "recovery-lifecycle/requests" / (self.stop_id + ".json"), self.request_ref)
        self.fence = {name: str(i) * 64 for i, name in enumerate(recovery_keeper._LEGACY_FILES, 1)}
        self.fence.update({name: self.ref(self.state / name)["sha256"] for name in
                          ("deployment-operation.json", "deployment/host.json", "deployment/driver.json")})
        self.old_release["expectedLifecycleFence"] = deepcopy(self.fence)
        runtime_files = [ROOT / ".jenkins/runtime/python" / name for name in
                         ("python.exe", "pythonw.exe", "python3.dll", "python314.dll", "vcruntime140.dll", "vcruntime140_1.dll")]
        runtime_files += [self.manager.java, self.manager.war]
        runtime_files += [ROOT / ".jenkins/runtime/jdk" / name for name in ("bin/server/jvm.dll", "lib/modules", "release")]
        for path in runtime_files:
            self.files[str(path)] = b"pinned runtime"
        self.old_release["runtimeFiles"] = {str(path): self.ref(path)["sha256"] for path in runtime_files}
        self.files[str(self.home / "config.xml")] = b"<hudson><numExecutors>0</numExecutors></hudson>"
        self.files[str(self.home / "queue.xml")] = b"<queue><items/><pendingItems/></queue>"
        self.owner = {"operation": {key: self.operation[key] for key in
                     ("operationId", "generation", "hostPid", "homePath", "component", "controlPlaneOnly")},
                     **{key: self.identities[key] for key in ("supervisor", "controller", "agent")}}

    def committed(self):
        self.boot = int(self.old_boot)
        self.descriptor["admission"] = "idle"
        request = {"schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "requestId": self.stop_id,
            "action": "upgrade", "keeper": self.identities["keeper"], "release": self.old_release_ref,
            "runtimeAcceptance": False}
        self.request_ref = self.put(self.state / "recovery-lifecycle/history/committed-request.json", request)
        self.origin.update(request=self.request_ref, previousRuntimeReceipt=None)
        origin_ref = self.put(self.state / "recovery-lifecycle/history/runtime.json", self.origin)
        self.descriptor["runtimeReceipt"] = origin_ref
        result = {"schemaVersion": 1, "kind": "recovery-keeper-command-result",
            "status": "running-acceptance-pending", "request": self.request_ref,
            "keeper": self.identities["keeper"], "release": self.old_release_ref,
            "runtimeReceipt": origin_ref, "operationId": self.runtime_id,
            "runtimeAcceptance": False, "cargoAcceptance": False, "historicalHoldsReleased": False}
        self.result_ref = self.put(self.state / "recovery-lifecycle/history/committed-result.json", result)
        self.command = {"schemaVersion": 1, "kind": recovery_keeper.COMMAND_KIND, "status": "committed",
            "request": self.request_ref, "keeper": self.identities["keeper"], "release": self.old_release_ref,
            "previousRuntimeReceipt": None, "result": self.result_ref,
            "runtimeAcceptance": False, "historicalHoldsReleased": False}
        self.put(self.state / "recovery-lifecycle/requests" / (self.stop_id + ".json"), self.request_ref)
        self.put(self.state / "recovery-lifecycle/results" / (self.stop_id + ".json"), self.result_ref)
        self.management_rows = [{"domain": "management_operation", "key": str(i), "version": 1,
            "payload": {"requestId": "management-" + str(i), "operationId": "management-" + str(i),
                "phase": phase, "payload": {"runtimeOperationId": self.runtime_id, "driverDigest": self.target["driverDigest"]}}}
            for i, phase in enumerate(("unknown", "running"))]
        self.refresh()
        return self

    def management_native(self):
        identity = {"pid": 40, "creationTime": str(self.boot + 1000), "executable": str(self.manager.java),
                    "commandDigest": "6" * 64}
        proof = {"identity": identity, "complete": True, "childrenGone": True, "activeProcesses": 0,
                 "stdoutEof": True, "stderrEof": True, "processExitCode": 1}
        self.rows.append({"domain": "native_job", "key": "management-0-native", "version": 1,
            "payload": {"owner": "management-0", "operationId": "management-0", "requestId": "management-0",
                "role": "management_owned_cargo", "status": "terminal", "pid": identity["pid"],
                "birthToken": identity["creationTime"], "executable": identity["executable"],
                "commandDigest": identity["commandDigest"], "terminalProof": proof, "completeProof": proof}})
        return identity

    def put(self, path, document):
        self.files[str(path)] = canonical_json(document) + b"\n"
        return self.ref(path)

    def ref(self, path):
        return {"path": str(path), "sha256": hashlib.sha256(self.files[str(path)]).hexdigest()}

    def refresh(self):
        for name, value in (("originalCommand", self.command), ("originalDescriptor", self.descriptor),
                            ("previousOperation", self.operation), ("previousHost", self.host), ("previousSelector", self.selected)):
            self.put(self.state / recovery._POINTERS[name], value)

    @staticmethod
    def absent(identity):
        return {"pid": identity["pid"], "expectedCreationTime": identity["creationTime"], "status": "absent"}

    def observations(self):
        stack = ExitStack()
        # Existing invariant tests need immediate rejection; the wait contracts
        # below use a deterministic clock and explicitly enable their window.
        stack.enter_context(patch.object(recovery, "_RUNTIME_WAIT_SECONDS", 0))
        for module in (recovery, lost_owner, recovery_keeper, owned_upgrade, runtime_roles):
            stack.enter_context(patch.object(module, "_raw", side_effect=lambda path: self.files[str(Path(path).absolute())]))
            stack.enter_context(patch.object(module, "_plain"))
        stack.enter_context(patch.object(Path, "exists", autospec=True, side_effect=lambda path: str(path) in self.files))
        stack.enter_context(patch.object(Path, "glob", autospec=True,
            side_effect=lambda path, pattern: [Path(name) for name in sorted(self.files)
                                             if Path(name).parent == path and fnmatch.fnmatch(Path(name).name, pattern)]))
        self.historical = stack.enter_context(patch.object(owned_upgrade, "_historical_release", return_value=(self.old_release, self.target)))
        self.issuance = stack.enter_context(patch.object(recovery, "_historical_runtime", return_value=self.origin))
        stack.enter_context(patch.object(recovery, "_lifecycle_fence", side_effect=lambda manager: deepcopy(self.fence)))
        self.boot_probe = stack.enter_context(patch.object(recovery, "windows_boot_filetime", return_value=self.boot))
        self.departures = stack.enter_context(patch.object(recovery, "_departure", side_effect=self.absent))
        self.inventory = stack.enter_context(patch.object(startup_observations, "runtime_owner_inventory", return_value=[]))
        self.native_rows = stack.enter_context(patch.object(recovery, "execution_inventory", side_effect=lambda state: deepcopy(self.rows)))
        stack.enter_context(patch.object(recovery, "protected_holds", return_value=self.holds))
        stack.enter_context(patch.object(recovery, "_resource_inventory", return_value=[{"budget": 4}]))
        self.home_probe = stack.enter_context(patch.object(recovery, "observe_home_available", return_value={"staleMarkersDeleted": False}))
        self.endpoint = stack.enter_context(patch.object(recovery, "check_port_available"))
        stack.enter_context(patch.object(recovery, "_management_rows", side_effect=lambda state: deepcopy(self.management_rows)))
        stack.enter_context(patch.object(recovery, "_same_boot_containment", return_value={"nativeTerminationClaimed": False}))
        return stack

    def prepare(self):
        return recovery.prepare_lost_recovery_runtime(self.manager, request_id=self.request_id)

    def current_execution(self):
        key = "3" * 32
        launcher = {"pid": 30, "creationTime": str(int(self.old_boot) + 100),
                    "executable": str(ROOT / ".jenkins/runtime/python/python.exe"), "commandDigest": "4" * 64}
        guard_id = "5" * 32
        native = []
        phases = []
        for i in range(3):
            phase_id = str(i + 6) * 32
            identity = {**launcher, "pid": 31 + i, "creationTime": str(int(self.old_boot) + 101 + i)}
            proof = {"identity": identity, "complete": True, "childrenGone": True, "activeProcesses": 0,
                     "stdoutEof": True, "stderrEof": True, "processExitCode": 0}
            phases.append({"nativeJobId": phase_id, "phase": str(i), "terminalProof": proof})
            native.append({"domain": "native_job", "key": phase_id, "version": 2,
                           "payload": {"pid": identity["pid"], "birthToken": identity["creationTime"],
                            "executable": identity["executable"], "role": "phase", "owner": key, "executionId": key,
                            "guardNativeJobId": guard_id, "status": "terminal", "completeProof": proof, "terminalProof": proof}})
        proof = {"identity": launcher, "complete": True, "childrenGone": True, "activeProcesses": 0,
                 "stdoutEof": True, "stderrEof": True, "processExitCode": 0}
        native.append({"domain": "native_job", "key": guard_id, "version": 2,
                       "payload": {"pid": 30, "birthToken": launcher["creationTime"], "executable": launcher["executable"],
                           "role": "execution_host_guard", "owner": key, "executionId": key, "status": "terminal",
                           "completeProof": proof, "terminalProof": proof}})
        self.rows.extend([{"domain": "execution", "key": key, "version": 10,
            "payload": {"operationId": self.runtime_id, "generation": "1", "driverDigest": "a" * 64,
                        "driver": {"runtimeOperationId": self.runtime_id, "generation": "1", "digest": "a" * 64},
                        "status": "passed", "refs": 0, "consumers": [], "phaseRecords": phases,
                        "terminalProof": phases[-1]["terminalProof"]}},
            {"domain": "execution_launch", "key": key, "version": 6, "payload": {"executionId": key,
                "operationId": self.runtime_id, "generation": "1", "driverDigest": "a" * 64,
                "brokerGeneration": self.runtime_id, "brokerPid": 12, "launcherIdentity": launcher,
                "guardNativeJobId": guard_id, "status": "exited"}},
            {"domain": "execution_host", "key": key, "version": 8, "payload": {"executionId": key,
                "operationId": self.runtime_id, "generation": "1", "hostPid": 30,
                "hostBirthToken": launcher["creationTime"], "status": "terminal", "phaseRecords": phases,
                "terminalProof": phases[-1]["terminalProof"]}}, *native])
        return key

    def successor(self, plan):
        marker = {"plan": self.put(self.state / "migration-baseline/lost-runtime-plan.json", plan.document),
                  "originalCommand": plan.document["originalCommand"], "originalDescriptor": plan.document["originalDescriptor"]}
        release = {"schemaVersion": 1, "kind": recovery_keeper.RELEASE_KIND, "requestId": self.request_id,
            "scope": "execution-runtime", "expectedPreviousOwner": plan.document["owner"],
            "expectedLifecycleFence": plan.document["lifecycleFence"], "protectedHolds": self.holds,
            "config": self.old_release["config"], "lostRecoveryRuntime": marker,
            "runtimeAcceptance": False, "cargoAcceptance": False, "automaticRestart": False, "queueReplayRequested": False}
        return release, self.put(self.state / "deployment/new-release.json", release)

    def actor(self, stack, plan):
        release, release_ref = self.successor(plan)
        new_keeper = {**self.identities["keeper"], "pid": 100, "creationTime": str(self.boot + 1)}
        stack.enter_context(patch.object(recovery_keeper, "_release", return_value=(release, self.target)))
        stack.enter_context(patch.object(recovery_keeper, "_full_identity", return_value=new_keeper))
        stack.enter_context(patch.object(recovery_keeper, "process_lock", side_effect=lambda *args, **kwargs: nullcontext()))
        stack.enter_context(patch.object(recovery_keeper, "windows_boot_filetime", return_value=self.boot))
        stack.enter_context(patch.object(recovery_keeper.RecoveryKeeper, "_fresh", return_value={"strict": True}))
        def archive(state, prefix, raw):
            path = state / "recovery-lifecycle/history" / (prefix + "-" + hashlib.sha256(raw).hexdigest() + ".json")
            self.files[str(path)] = raw
            return self.ref(path)
        stack.enter_context(patch.object(recovery_keeper, "_archive_bytes", side_effect=archive))
        def write(path, value, *, expected_sha=None):
            if expected_sha is not None and self.ref(path)["sha256"] != expected_sha:
                raise JenkinsError("pointer_changed", "CAS changed")
            return self.put(path, value)
        self.write_pointer = stack.enter_context(patch.object(recovery_keeper, "_write", side_effect=write))
        return recovery_keeper.RecoveryKeeper(self.manager, release_path=Path(release_ref["path"]),
            release_sha256=release_ref["sha256"], preflight=Mock(), publish=Mock(), recovery_plan=plan)


class LostRecoveryRuntimeTests(unittest.TestCase):
    def test_ordinary_helper_inventory_survives_review_and_legacy_plan_revalidation(self):
        for committed in (False, True):
            with self.subTest(committed=committed):
                scenario = DepartedRuntime()
                if committed:
                    scenario.committed()
                arguments = [str(ROOT / ".jenkins/runtime/python/python.exe"), "-B",
                             str(Path("E:/cargo-targets/foreign/validate.py"))]
                helper = {"pid": 777, "creationTime": str(scenario.boot + 1), "executable": arguments[0],
                          "commandDigest": runtime_roles._digest_args(tuple(arguments))}
                with scenario.observations(), \
                        patch.object(runtime_roles, "_windows_command_line", return_value=arguments), \
                        patch.object(runtime_roles, "observe_live_identity", side_effect=lambda identity: identity), \
                        patch.object(recovery.time, "sleep") as wait:
                    legacy = scenario.prepare().document
                    legacy.pop("runtimeOwnerObservation")
                    reviewed = scenario.put(scenario.state / "migration-baseline/legacy-plan.json", legacy)
                    before = deepcopy(scenario.files)
                    scenario.inventory.return_value = [helper]
                    plan = scenario.prepare()
                    observation = plan.document["runtimeOwnerObservation"]
                    self.assertEqual([helper], plan.document["runtimeOwnerInventory"])
                    self.assertEqual([helper], observation["runtimeInventory"])
                    self.assertEqual([], observation["blockingOwners"])
                    self.assertEqual(1, len(observation["nonServiceProcesses"]))
                    self.assertTrue(observation["observationOnly"])
                    self.assertFalse(observation["terminalProof"])
                    self.assertFalse(observation["nativeProofIssued"])
                    restored = recovery.prepare_lost_recovery_runtime(scenario.manager,
                        request_id=scenario.request_id, reviewed_plan_ref=reviewed)
                    self.assertEqual(legacy, restored.document)
                    scenario.inventory.return_value = []
                    refreshed = plan.revalidate(scenario.manager)
                    self.assertEqual([], refreshed["runtimeOwnerInventory"])
                    self.assertEqual(plan.recovery_digest, refreshed["recoveryDigest"])
                    wait.assert_not_called()
                    self.assertEqual(before, scenario.files)

    def test_current_service_owner_still_blocks_with_ordinary_helper_present(self):
        for service_kind in ("journal", "control-entry"):
            with self.subTest(service_kind=service_kind):
                scenario = DepartedRuntime().committed()
                helper_arguments = [str(ROOT / ".jenkins/runtime/python/python.exe"), "-B", "-c", "print('helper')"]
                service_arguments = [helper_arguments[0], "-B", str(ROOT / "tools/dev/jenkins_recovery_keeper.py")]
                helper = {"pid": 777, "creationTime": str(scenario.boot + 1), "executable": helper_arguments[0],
                          "commandDigest": runtime_roles._digest_args(tuple(helper_arguments))}
                service = (scenario.identities["supervisor"] if service_kind == "journal" else
                           {**helper, "pid": 778, "commandDigest": runtime_roles._digest_args(tuple(service_arguments))})
                before = deepcopy(scenario.files)
                with scenario.observations(), \
                        patch.object(runtime_roles, "_windows_command_line",
                            side_effect=lambda pid: helper_arguments if pid == helper["pid"] else service_arguments), \
                        patch.object(runtime_roles, "observe_live_identity", side_effect=lambda identity: identity), \
                        patch.object(recovery.time, "sleep") as wait:
                    scenario.inventory.return_value = [helper, service]
                    with self.assertRaises(JenkinsError) as raised:
                        scenario.prepare()
                    self.assertEqual("lost_recovery_runtime_unproven", raised.exception.code)
                    self.assertEqual([helper, service], raised.exception.details["runtimeOwnerInventory"])
                    observation = raised.exception.details["runtimeOwnerObservation"]
                    self.assertEqual([service], observation["blockingOwners"])
                    self.assertEqual(1, len(observation["nonServiceProcesses"]))
                    self.assertFalse(observation["nativeProofIssued"])
                    scenario.home_probe.assert_not_called()
                    scenario.endpoint.assert_not_called()
                    wait.assert_not_called()
                self.assertEqual(before, scenario.files)

    def test_transient_runtime_departure_restarts_full_readonly_review(self):
        scenario = DepartedRuntime().committed()
        before = deepcopy(scenario.files)
        with scenario.observations(), patch.object(recovery, "_RUNTIME_WAIT_SECONDS", 1), \
                patch.object(recovery.time, "monotonic", return_value=0), \
                patch.object(recovery.time, "sleep", side_effect=lambda delay: setattr(scenario.inventory, "return_value", [])) as wait:
            scenario.inventory.return_value = [scenario.identities["agent"]]
            plan = scenario.prepare()
            self.assertEqual([], plan.document["runtimeOwnerInventory"])
            self.assertEqual(scenario.holds, plan.document["executionQuarantine"]["protectedHolds"])
            self.assertTrue(wait.called)
        self.assertEqual(before, scenario.files)

    def test_owner_appearing_at_final_observation_requires_complete_review_again(self):
        scenario = DepartedRuntime().committed()
        before = deepcopy(scenario.files)
        with scenario.observations(), patch.object(recovery, "_RUNTIME_WAIT_SECONDS", 1), \
                patch.object(recovery.time, "monotonic", return_value=0), \
                patch.object(recovery.time, "sleep") as wait:
            scenario.inventory.side_effect = [[], [], [scenario.identities["agent"]], [], [], []]
            plan = scenario.prepare()
            self.assertEqual([], plan.document["runtimeOwnerInventory"])
            self.assertGreaterEqual(scenario.home_probe.call_count, 2)
            self.assertTrue(wait.called)
        self.assertEqual(before, scenario.files)

    def test_persistent_runtime_occupancy_expires_without_writes_or_owner_exclusion(self):
        scenario = DepartedRuntime().committed()
        before, clock = deepcopy(scenario.files), [0.0]
        def advance(delay):
            clock[0] += delay
        with scenario.observations(), patch.object(recovery, "_RUNTIME_WAIT_SECONDS", 1), \
                patch.object(recovery.time, "monotonic", side_effect=lambda: clock[0]), \
                patch.object(recovery.time, "sleep", side_effect=advance):
            scenario.inventory.return_value = [scenario.identities["agent"]]
            with self.assertRaises(JenkinsError) as raised:
                scenario.prepare()
            self.assertEqual("lost_recovery_runtime_unproven", raised.exception.code)
            self.assertEqual([scenario.identities["agent"]], raised.exception.details["runtimeOwnerInventory"])
            self.assertEqual(1.0, clock[0])
        self.assertEqual(before, scenario.files)

    def test_wait_rejects_changed_boot_fence_or_inaccessible_inventory(self):
        for change in ("boot", "fence", "inaccessible"):
            with self.subTest(change=change):
                scenario = DepartedRuntime().committed()
                before = deepcopy(scenario.files)
                def mutate(delay):
                    if change == "boot":
                        scenario.boot_probe.return_value = scenario.boot + 1
                    else:
                        scenario.fence["deployment/driver.json"] = "f" * 64
                with scenario.observations(), patch.object(recovery, "_RUNTIME_WAIT_SECONDS", 1), \
                        patch.object(recovery.time, "monotonic", return_value=0), \
                        patch.object(recovery.time, "sleep", side_effect=mutate) as wait:
                    scenario.inventory.return_value = [scenario.identities["agent"]]
                    if change == "inaccessible":
                        scenario.inventory.side_effect = JenkinsError("deployment_inventory_inconclusive", "inaccessible")
                    with self.assertRaises(JenkinsError):
                        scenario.prepare()
                    self.assertEqual(change != "inaccessible", wait.called)
                self.assertEqual(before, scenario.files)

    def test_reviewed_plan_revalidation_wait_keeps_original_identity_and_digest(self):
        scenario = DepartedRuntime().committed()
        with scenario.observations():
            plan = scenario.prepare()
            before = deepcopy(scenario.files)
            scenario.inventory.return_value = [scenario.identities["agent"]]
            with patch.object(recovery, "_RUNTIME_WAIT_SECONDS", 1), \
                    patch.object(recovery.time, "monotonic", return_value=0), \
                    patch.object(recovery.time, "sleep", side_effect=lambda delay: setattr(scenario.inventory, "return_value", [])):
                current = plan.revalidate(scenario.manager)
            self.assertEqual(plan.recovery_digest, current["recoveryDigest"])
            self.assertEqual(plan.document["owner"], current["owner"])
            self.assertEqual(before, scenario.files)

    def test_same_boot_committed_runtime_preserves_result_management_outcomes_and_holds(self):
        scenario = DepartedRuntime().committed()
        files, rows, management = deepcopy(scenario.files), deepcopy(scenario.rows), deepcopy(scenario.management_rows)
        with scenario.observations():
            plan = scenario.prepare()
            value = plan.revalidate(scenario.manager)
            self.assertEqual(recovery.SAME_BOOT_VARIANT, value["recoveryVariant"])
            self.assertEqual("committed", value["originalCommandStatus"])
            self.assertEqual(scenario.result_ref, value["originalResult"])
            self.assertEqual(scenario.files[scenario.result_ref["path"]], plan.original_artifacts["originalResult"]["raw"])
            self.assertEqual(scenario.holds, value["executionQuarantine"]["protectedHolds"])
            self.assertEqual(2, len(value["managementQuarantine"]["currentRecords"]))
            self.assertFalse(value["containmentReview"]["nativeTerminationClaimed"])
            for flag in recovery._FLAGS:
                self.assertIs(value[flag], False)
        self.assertEqual(files, scenario.files)
        self.assertEqual(rows, scenario.rows)
        self.assertEqual(management, scenario.management_rows)

    def test_same_boot_committed_wrong_admission_boot_or_delivered_result_blocks(self):
        for case in ("blocked", "earlier-boot", "old-birth", "missing-result", "wrong-result"):
            with self.subTest(case=case):
                scenario = DepartedRuntime().committed()
                if case == "blocked":
                    scenario.descriptor["admission"] = "blocked"
                elif case == "earlier-boot":
                    scenario.boot -= 100
                elif case == "old-birth":
                    scenario.identities["agent"]["creationTime"] = str(scenario.boot - 1)
                elif case == "missing-result":
                    del scenario.files[str(scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json"))]
                else:
                    scenario.put(scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json"), scenario.old_release_ref)
                scenario.refresh()
                before = deepcopy(scenario.files)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()
                self.assertEqual(before, scenario.files)

    def test_committed_previous_boot_retains_original_results_proofs_and_protected_holds(self):
        scenario = DepartedRuntime().committed()
        scenario.current_execution()
        scenario.management_native()
        scenario.boot += 10000
        before, rows, management = deepcopy(scenario.files), deepcopy(scenario.rows), deepcopy(scenario.management_rows)
        with scenario.observations():
            plan = scenario.prepare()
            value = plan.revalidate(scenario.manager)
            self.assertEqual(recovery.SAME_BOOT_VARIANT, value["recoveryVariant"])
            self.assertEqual("committed", value["originalCommandStatus"])
            self.assertEqual(scenario.result_ref, value["originalResult"])
            self.assertEqual(scenario.old_boot, value["previousBootFileTime"])
            self.assertEqual(str(scenario.boot), value["bootFileTime"])
            self.assertEqual(scenario.holds, value["executionQuarantine"]["protectedHolds"])
            self.assertFalse(value["containmentReview"]["nativeTerminationClaimed"])
            self.assertTrue(value["managementQuarantine"]["nativeDepartures"][0]["nativeProofReused"])
            for flag in recovery._FLAGS:
                self.assertIs(value[flag], False)
        self.assertEqual(before, scenario.files)
        self.assertEqual(rows, scenario.rows)
        self.assertEqual(management, scenario.management_rows)

    def test_committed_previous_boot_rejects_owner_or_execution_birth_at_or_after_new_boot(self):
        for subject in ("agent", "execution"):
            for offset in (0, 1):
                with self.subTest(subject=subject, offset=offset):
                    scenario = DepartedRuntime().committed()
                    scenario.boot += 10000
                    if subject == "agent":
                        scenario.identities["agent"]["creationTime"] = str(scenario.boot + offset)
                    else:
                        scenario.current_execution()
                        row = next(row for row in scenario.rows if row["domain"] == "native_job" and row["payload"].get("role") == "phase")
                        row["payload"]["completeProof"]["identity"]["creationTime"] = str(scenario.boot + offset)
                        row["payload"]["birthToken"] = str(scenario.boot + offset)
                    scenario.refresh()
                    before = deepcopy(scenario.files)
                    with scenario.observations(), self.assertRaises(JenkinsError):
                        scenario.prepare()
                    self.assertEqual(before, scenario.files)

    def test_committed_previous_boot_rejects_management_native_birth_at_or_after_new_boot(self):
        for offset in (0, 1):
            with self.subTest(offset=offset):
                scenario = DepartedRuntime().committed()
                scenario.boot += 10000
                identity = scenario.management_native()
                identity["creationTime"] = str(scenario.boot + offset)
                scenario.rows[-1]["payload"]["birthToken"] = identity["creationTime"]
                before = deepcopy(scenario.files)
                with scenario.observations(), self.assertRaises(JenkinsError):
                    scenario.prepare()
                self.assertEqual(before, scenario.files)

    def test_same_boot_current_execution_and_management_native_proofs_need_fresh_departure(self):
        scenario = DepartedRuntime().committed()
        scenario.current_execution()
        native = scenario.management_native()
        rows = deepcopy(scenario.rows)
        with scenario.observations():
            plan = scenario.prepare()
            quarantine = plan.document["managementQuarantine"]
            self.assertEqual(native, quarantine["nativeDepartures"][0]["identity"])
            self.assertTrue(quarantine["nativeDepartures"][0]["nativeProofReused"])
            self.assertEqual(1, len(plan.document["executionQuarantine"]["currentExecutions"]))
            observed = {call.args[0]["pid"] for call in scenario.departures.call_args_list}
            self.assertTrue({11, 12, 13, 14, 30, 31, 32, 33, 40} <= observed)
            def departed(identity):
                if identity["pid"] == native["pid"]:
                    raise JenkinsError("native_owner_live", "Original management child still lives")
                return scenario.absent(identity)
            scenario.departures.side_effect = departed
            with self.assertRaises(JenkinsError):
                plan.revalidate(scenario.manager)
        self.assertEqual(rows, scenario.rows)

    def test_same_boot_management_state_change_cannot_relabel_or_reuse_original_plan(self):
        scenario = DepartedRuntime().committed()
        with scenario.observations():
            plan = scenario.prepare()
            scenario.management_rows[0]["version"] += 1
            with self.assertRaises(JenkinsError) as raised:
                plan.revalidate(scenario.manager)
            self.assertEqual("lost_recovery_runtime_changed", raised.exception.code)
            self.assertEqual("unknown", scenario.management_rows[0]["payload"]["phase"])
            self.assertEqual("running", scenario.management_rows[1]["payload"]["phase"])

    def test_real_later_boot_preserves_unknown_raw_and_all_four_holds(self):
        scenario = DepartedRuntime()
        before = deepcopy(scenario.files)
        with scenario.observations():
            plan = scenario.prepare()
            evidence = plan.revalidate(scenario.manager)
            self.assertEqual(recovery.VARIANT, evidence["recoveryVariant"])
            self.assertEqual("unknown", evidence["originalCommandStatus"])
            self.assertEqual(scenario.holds, evidence["executionQuarantine"]["protectedHolds"])
            self.assertEqual(scenario.files[str(scenario.state / recovery._POINTERS["originalCommand"])],
                             plan.original_artifacts["originalCommand"]["raw"])
            for flag in recovery._FLAGS:
                self.assertIs(evidence[flag], False)
            self.assertEqual(2, scenario.historical.call_count)
        self.assertEqual(before, scenario.files)

    def test_verified_absent_and_foreign_pid_reuse_refreshes_keep_original_plan_and_native_proofs(self):
        for initial, current in (("absent", "pid-reused"), ("pid-reused", "absent")):
            with self.subTest(initial=initial, current=current):
                scenario = DepartedRuntime()
                scenario.current_execution()
                files, rows = deepcopy(scenario.files), deepcopy(scenario.rows)
                with scenario.observations() as stack:
                    scenario.departures.side_effect = lost_owner._departure
                    alive = stack.enter_context(patch.object(startup_observations, "process_alive", return_value=initial == "pid-reused"))
                    native_identity = stack.enter_context(patch.object(startup_observations, "current_identity",
                        side_effect=lambda pid: SimpleNamespace(to_dict=lambda: {
                            "pid": pid, "creationTime": str(scenario.boot + pid),
                            "executable": str(ROOT.parent / "foreign/process.exe"), "commandDigest": "f" * 64})))
                    plan = scenario.prepare()
                    snapshot, checksum = plan.snapshot, plan.recovery_digest
                    alive.return_value = current == "pid-reused"
                    evidence = plan.revalidate(scenario.manager)
                    review = evidence["revalidation"]
                    self.assertEqual(snapshot, plan.snapshot)
                    self.assertEqual(checksum, plan.recovery_digest)
                    self.assertEqual(checksum, review["reviewedPlanDigest"])
                    self.assertNotEqual(checksum, review["currentObservationDigest"])
                    self.assertEqual(9, len(review["departureRefreshes"]))
                    observed = recovery._departure_observations(evidence)
                    for change in review["departureRefreshes"]:
                        self.assertEqual(initial, change["before"]["status"])
                        self.assertEqual(current, change["after"]["status"])
                        self.assertEqual(observed[change["path"]], change["after"])
                        self.assertEqual(change["before"]["pid"], change["after"]["pid"])
                        self.assertEqual(change["before"]["expectedCreationTime"], change["after"]["expectedCreationTime"])
                    self.assertEqual(plan.document["executionQuarantine"]["records"], evidence["executionQuarantine"]["records"])
                    self.assertEqual(plan.document["homeObservation"], evidence["homeObservation"])
                    self.assertEqual(plan.document["lifecycleFence"], evidence["lifecycleFence"])
                    for flag in recovery._FLAGS:
                        self.assertIs(evidence[flag], False)
                    native_identity.assert_called()
                self.assertEqual(files, scenario.files)
                self.assertEqual(rows, scenario.rows)

    def test_reviewed_factory_refresh_retains_digest_raw_evidence_and_keeper_authority(self):
        for committed in (False, True):
            for initial, current in (("absent", "pid-reused"), ("pid-reused", "absent")):
                with self.subTest(committed=committed, initial=initial, current=current):
                    scenario = DepartedRuntime()
                    if committed:
                        scenario.committed()
                        scenario.management_native()
                    scenario.current_execution()
                    with scenario.observations() as stack:
                        scenario.departures.side_effect = lost_owner._departure
                        alive = stack.enter_context(patch.object(startup_observations, "process_alive",
                                                                 return_value=initial == "pid-reused"))
                        stack.enter_context(patch.object(startup_observations, "current_identity",
                            side_effect=lambda pid: SimpleNamespace(to_dict=lambda: {
                                "pid": pid, "creationTime": str(scenario.boot + 10000 + pid),
                                "executable": str(ROOT.parent / "foreign/process.exe"), "commandDigest": "f" * 64})))
                        reviewed = scenario.prepare()
                        reference = scenario.put(scenario.state / "migration-baseline/refresh-plan.json", reviewed.document)
                        files, rows, management = deepcopy(scenario.files), deepcopy(scenario.rows), deepcopy(scenario.management_rows)
                        alive.return_value = current == "pid-reused"
                        fresh = recovery.prepare_lost_recovery_runtime(scenario.manager, request_id=scenario.request_id,
                            original_command_ref=reviewed.document["originalCommand"],
                            original_descriptor_ref=reviewed.document["originalDescriptor"], reviewed_plan_ref=reference)
                        self.assertEqual(reviewed.snapshot, fresh.snapshot)
                        self.assertEqual(reviewed.recovery_digest, fresh.recovery_digest)
                        self.assertEqual(reviewed.original_artifacts, fresh.original_artifacts)
                        evidence = fresh.revalidate(scenario.manager)
                        refreshes = evidence["revalidation"]["departureRefreshes"]
                        keeper = next(item for item in refreshes if item["path"] == "/departedIdentities/keeper")
                        self.assertEqual(initial, keeper["before"]["status"])
                        self.assertEqual(current, keeper["after"]["status"])
                        self.assertEqual(reviewed.recovery_digest, evidence["revalidation"]["reviewedPlanDigest"])
                        self.assertNotEqual(reviewed.recovery_digest, evidence["revalidation"]["currentObservationDigest"])
                        self.assertEqual(scenario.holds, evidence["executionQuarantine"]["protectedHolds"])
                        for flag in recovery._FLAGS:
                            self.assertIs(evidence[flag], False)
                        self.assertEqual(files, scenario.files)
                        self.assertEqual(rows, scenario.rows)
                        self.assertEqual(management, scenario.management_rows)
                        actor = scenario.actor(stack, fresh)
                        actor._reviewed_lost_runtime_plan(fresh)
                        for name, original in reviewed.original_artifacts.items():
                            self.assertEqual(original["raw"], scenario.files[original["reference"]["path"]])

    def test_reviewed_factory_rejects_non_departure_differences(self):
        for field in ("requestId", "bootFileTime", "nativeProofIssued", "lifecycleFence", "executionQuarantine"):
            with self.subTest(field=field):
                scenario = DepartedRuntime().committed()
                with scenario.observations():
                    reviewed = scenario.prepare()
                    document = reviewed.document
                    if field == "lifecycleFence":
                        document[field][next(iter(document[field]))] = "0" * 64
                    elif field == "executionQuarantine":
                        document[field]["protectedHolds"][0]["payload"]["status"] = "released"
                    else:
                        document[field] = {"requestId": "f" * 32, "bootFileTime": str(scenario.boot + 1),
                                           "nativeProofIssued": True}[field]
                    reference = scenario.put(scenario.state / "migration-baseline/different-plan.json", document)
                    files, rows = deepcopy(scenario.files), deepcopy(scenario.rows)
                    with self.assertRaises(JenkinsError) as raised:
                        recovery.prepare_lost_recovery_runtime(scenario.manager, request_id=scenario.request_id,
                                                              reviewed_plan_ref=reference)
                    self.assertEqual("lost_recovery_runtime_changed", raised.exception.code)
                    self.assertTrue(raised.exception.details["changedPaths"])
                    self.assertEqual(files, scenario.files)
                    self.assertEqual(rows, scenario.rows)

    def test_reviewed_factory_still_requires_original_hash_and_fresh_native_departure(self):
        scenario = DepartedRuntime().committed()
        with scenario.observations() as stack:
            reviewed = scenario.prepare()
            reference = scenario.put(scenario.state / "migration-baseline/reviewed-plan.json", reviewed.document)
            files, rows = deepcopy(scenario.files), deepcopy(scenario.rows)
            with self.assertRaises(JenkinsError):
                recovery.prepare_lost_recovery_runtime(scenario.manager, request_id=scenario.request_id,
                    reviewed_plan_ref={**reference, "sha256": "0" * 64})
            scenario.departures.side_effect = lost_owner._departure
            stack.enter_context(patch.object(startup_observations, "process_alive",
                side_effect=lambda pid: pid == scenario.identities["keeper"]["pid"]))
            stack.enter_context(patch.object(startup_observations, "current_identity",
                return_value=SimpleNamespace(to_dict=lambda: scenario.identities["keeper"])))
            with self.assertRaises(JenkinsError):
                recovery.prepare_lost_recovery_runtime(scenario.manager, request_id=scenario.request_id,
                                                      reviewed_plan_ref=reference)
            self.assertEqual(files, scenario.files)
            self.assertEqual(rows, scenario.rows)

    def test_departure_refresh_during_capture_keeps_latest_valid_native_observation(self):
        scenario = DepartedRuntime()
        scenario.current_execution()
        counts = {}
        def alive(pid):
            counts[pid] = counts.get(pid, 0) + 1
            return pid >= 30 and counts[pid] > 1
        with scenario.observations() as stack:
            scenario.departures.side_effect = lost_owner._departure
            stack.enter_context(patch.object(startup_observations, "process_alive", side_effect=alive))
            stack.enter_context(patch.object(startup_observations, "current_identity",
                side_effect=lambda pid: SimpleNamespace(to_dict=lambda: {
                    "pid": pid, "creationTime": str(scenario.boot + pid),
                    "executable": str(ROOT.parent / "foreign/process.exe")})))
            plan = scenario.prepare()
            self.assertTrue(all(item["status"] == "pid-reused" for item in
                                plan.document["executionQuarantine"]["currentExecutions"][0]["departures"]))
            self.assertTrue(all(item["status"] == "absent" for item in plan.document["departedIdentities"].values()))

    def test_fresh_live_old_birth_or_unqueryable_identity_still_blocks(self):
        for target, problem in (("keeper", "live"), ("supervisor", "inaccessible"),
                                ("controller", "inconclusive"), ("native", "live"),
                                ("agent", "older-birth")):
            with self.subTest(target=target, problem=problem):
                scenario = DepartedRuntime()
                scenario.current_execution()
                expected = scenario.identities.get(target) or {"pid": 31, "creationTime": str(int(scenario.old_boot) + 101),
                    "executable": str(ROOT / ".jenkins/runtime/python/python.exe")}
                with scenario.observations() as stack:
                    scenario.departures.side_effect = lost_owner._departure
                    alive = stack.enter_context(patch.object(startup_observations, "process_alive", return_value=False))
                    native_identity = stack.enter_context(patch.object(startup_observations, "current_identity"))
                    plan = scenario.prepare()
                    alive.side_effect = lambda pid: (None if problem == "inconclusive" else True) if pid == expected["pid"] else False
                    if problem == "inaccessible":
                        native_identity.side_effect = OSError("identity unavailable")
                    else:
                        native_identity.return_value = SimpleNamespace(to_dict=lambda: {
                            **expected, "creationTime": str(int(expected["creationTime"]) - 1)
                            if problem == "older-birth" else expected["creationTime"]})
                    with self.assertRaises(JenkinsError) as raised:
                        plan.revalidate(scenario.manager)
                    self.assertEqual("deployment_owner_live" if problem == "live" else
                        "deployment_identity_inconclusive" if problem in {"inaccessible", "inconclusive"}
                        else "formal_lost_owner_unproven", raised.exception.code)

    def test_valid_departure_refresh_does_not_hide_real_invariant_changes(self):
        for change in ("boot", "owner", "fence", "ledger", "home", "home-observation", "proof", "inventory"):
            with self.subTest(change=change):
                scenario = DepartedRuntime()
                scenario.current_execution()
                with scenario.observations() as stack:
                    scenario.departures.side_effect = lost_owner._departure
                    alive = stack.enter_context(patch.object(startup_observations, "process_alive", return_value=False))
                    stack.enter_context(patch.object(startup_observations, "current_identity",
                        side_effect=lambda pid: SimpleNamespace(to_dict=lambda: {
                            "pid": pid, "creationTime": str(scenario.boot + pid),
                            "executable": str(ROOT.parent / "foreign/process.exe")})))
                    plan = scenario.prepare()
                    alive.return_value = True
                    if change == "boot":
                        scenario.boot_probe.return_value = scenario.boot + 1
                    elif change == "owner":
                        scenario.command["reason"] = "changed original bytes"
                        scenario.refresh()
                    elif change == "fence":
                        scenario.fence["deployment/driver.json"] = "f" * 64
                    elif change == "ledger":
                        scenario.rows[0]["version"] += 1
                    elif change == "home":
                        scenario.files[str(scenario.home / "config.xml")] += b"\n"
                    elif change == "home-observation":
                        scenario.home_probe.return_value = {"staleMarkersDeleted": False, "exclusiveReadProbes": [{"modifiedNs": 1}]}
                    elif change == "proof":
                        next(row for row in scenario.rows if row["domain"] == "native_job")["payload"]["completeProof"]["stdoutEof"] = False
                    else:
                        scenario.inventory.return_value = [scenario.identities["agent"]]
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)

    def test_real_invariant_drift_reports_paths_and_observation_digest(self):
        scenario = DepartedRuntime()
        with scenario.observations():
            plan = scenario.prepare()
            scenario.rows[0]["version"] += 1
            with self.assertRaises(JenkinsError) as raised:
                plan.revalidate(scenario.manager)
            error = raised.exception
            self.assertEqual("lost_recovery_runtime_changed", error.code)
            self.assertEqual(plan.recovery_digest, error.details["reviewedPlanDigest"])
            self.assertIn("/executionQuarantine/records/0/version", error.details["changedPaths"])
            self.assertNotEqual(plan.recovery_digest, error.details["currentObservationDigest"])
            self.assertFalse(error.details["changedPathsTruncated"])

    def test_same_boot_or_birth_after_boot_each_blocks(self):
        for boot in (int(DepartedRuntime().old_boot), int(DepartedRuntime().old_boot) + 12):
            scenario = DepartedRuntime()
            with scenario.observations():
                scenario.boot_probe.return_value = boot
                with self.assertRaises(JenkinsError):
                    scenario.prepare()
                scenario.endpoint.assert_not_called()

    def test_live_component_runtime_inventory_or_port_blocks(self):
        for role in ("keeper", "supervisor", "controller", "agent"):
            scenario = DepartedRuntime()
            with scenario.observations():
                def observe(identity):
                    if identity == scenario.identities[role]:
                        raise JenkinsError("deployment_owner_live", "still live")
                    return scenario.absent(identity)
                scenario.departures.side_effect = observe
                with self.assertRaises(JenkinsError):
                    scenario.prepare()
        scenario = DepartedRuntime()
        with scenario.observations():
            scenario.inventory.return_value = [scenario.identities["controller"]]
            with self.assertRaises(JenkinsError):
                scenario.prepare()
        with scenario.observations():
            scenario.endpoint.side_effect = JenkinsError("controller_port_unavailable", "busy")
            with self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_unknown_stop_original_request_and_selector_tampering_blocks(self):
        for mutation in (lambda s: s.command.update(status="stopped"),
                         lambda s: s.command.update(reasonCode="native_stop_unknown"),
                         lambda s: s.command.update(previousRuntimeReceipt=s.old_release_ref),
                         lambda s: s.selected.update(runtimeOperationId="start-999"),
                         lambda s: s.descriptor.update(admission="idle"),
                         lambda s: s.command.update(result=s.old_release_ref)):
            scenario = DepartedRuntime()
            mutation(scenario)
            scenario.refresh()
            with scenario.observations(), self.assertRaises(JenkinsError):
                scenario.prepare()
        scenario = DepartedRuntime()
        scenario.put(scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json"), {"fake": True})
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_pending_persisted_queue_runtime_bytes_and_native_stop_each_block(self):
        scenario = DepartedRuntime()
        scenario.files[str(scenario.home / "queue.xml")] = b"<queue><items><item/></items></queue>"
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()
        scenario = DepartedRuntime()
        scenario.files[str(scenario.manager.java)] = b"modified runtime"
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()
        scenario = DepartedRuntime()
        scenario.put(scenario.state / "deployment/commands/stop-123.json", {"operationId": scenario.runtime_id})
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()

    def test_reconciled_native_terminal_proofs_permit_no_artifact_acceptance(self):
        scenario = DepartedRuntime()
        key = scenario.current_execution()
        before = deepcopy(scenario.rows)
        with scenario.observations():
            plan = scenario.prepare()
            observation = plan.document["executionQuarantine"]["currentExecutions"][0]
            self.assertEqual(key, observation["executionId"])
            self.assertEqual("exited", observation["launchStatus"])
            self.assertTrue(observation["launchMetadataReconciled"])
            self.assertFalse(observation["artifactAcceptance"])
        self.assertEqual(before, scenario.rows)

    def test_stranded_started_launch_remains_unchanged_and_requires_separate_reconciliation(self):
        scenario = DepartedRuntime()
        key = scenario.current_execution()
        next(row for row in scenario.rows if row["domain"] == "execution_launch" and row["key"] == key)["payload"]["status"] = "started"
        before = deepcopy(scenario.rows)
        with scenario.observations(), self.assertRaises(JenkinsError):
            scenario.prepare()
        self.assertEqual(before, scenario.rows)

    def test_missing_guard_eof_phase_and_unreleased_consumer_block(self):
        for case in ("guard", "eof", "phase", "refs", "consumer", "reservation"):
            scenario = DepartedRuntime()
            key = scenario.current_execution()
            if case == "guard":
                scenario.rows = [row for row in scenario.rows if row["key"] != "5" * 32]
            elif case == "eof":
                next(row for row in scenario.rows if row["key"] == "5" * 32)["payload"]["completeProof"]["stdoutEof"] = False
            elif case == "phase":
                scenario.rows = [row for row in scenario.rows if row["key"] != "6" * 32]
            elif case == "refs":
                next(row for row in scenario.rows if row["domain"] == "execution" and row["key"] == key)["payload"]["refs"] = 1
            else:
                scenario.rows.append({"domain": "execution_consumer" if case == "consumer" else "resource_reservation",
                    "key": key + ":x", "version": 1, "payload": {"executionId": key, "owner": key, "status": "active"}})
            with scenario.observations(), self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_json_subclass_and_revalidation_races_cannot_issue_authority(self):
        with self.assertRaises(JenkinsError):
            recovery.LostRecoveryRuntimeInterruption(canonical_json({"kind": recovery.KIND}))
        scenario = DepartedRuntime()
        with scenario.observations():
            plan = scenario.prepare()
            copy = plan.document
            copy["outcome"] = "stopped"
            self.assertEqual("interrupted", plan.document["outcome"])
            scenario.boot_probe.return_value = scenario.boot + 1
            with self.assertRaises(JenkinsError):
                plan.revalidate(scenario.manager)

    def test_reviewed_release_marker_requires_exact_pointer_namespace(self):
        scenario = DepartedRuntime()
        with scenario.observations():
            plan = scenario.prepare()
            marker = {"plan": scenario.put(scenario.state / "migration-baseline/plan.json", plan.document),
                      "originalCommand": plan.document["originalCommand"], "originalDescriptor": plan.document["originalDescriptor"]}
            release = {"kind": recovery_keeper.RELEASE_KIND, "requestId": scenario.request_id, "scope": "execution-runtime",
                "expectedPreviousOwner": plan.document["owner"], "expectedLifecycleFence": plan.document["lifecycleFence"],
                "protectedHolds": scenario.holds, "lostRecoveryRuntime": marker}
            self.assertEqual(plan.document, recovery.reviewed_marker(scenario.state, release))
            marker["originalCommand"] = {**marker["originalCommand"], "path": str(scenario.state / "deployment/command.json")}
            with self.assertRaises(JenkinsError):
                recovery.reviewed_marker(scenario.state, release)

    def test_one_typed_replacement_archives_every_raw_byte_and_revalidates_its_own_command(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            plan = scenario.prepare()
            originals = plan.original_artifacts
            actor = scenario.actor(stack, plan)
            request = scenario.put(scenario.state / "recovery-lifecycle/history/new-request.json", {
                "schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "action": "recover",
                "requestId": scenario.request_id, "keeper": actor.keeper, "release": actor.release_ref,
                "recoveryDigest": plan.recovery_digest, "runtimeAcceptance": False})
            replacement = actor._begin(request, plan=plan)
            command = recovery._document(scenario.files[str(actor.command_path)])
            self.assertEqual("pending", command["status"])
            self.assertEqual(plan.document, replacement.document)
            self.assertEqual(plan.recovery_digest, replacement.recovery_digest)
            replacement.revalidate(scenario.manager)
            receipt = recovery._document(scenario.files[actor.runtime_interruption["receipt"]["path"]])
            for name, value in originals.items():
                archive = receipt["originalArtifacts"][name]["archive"]
                self.assertEqual(value["raw"], scenario.files[archive["path"]])
                self.assertEqual(value["reference"]["sha256"], archive["sha256"])
            self.assertEqual("unknown", recovery._document(scenario.files[command["replacesOriginalCommand"]["path"]])["status"])
            with self.assertRaises(JenkinsError):
                actor._begin(request, plan=plan)
            command["status"] = "start-attempted"
            scenario.put(actor.command_path, command)
            capability = recovery_keeper.RecoveryKeeperStart(actor, replacement, request,
                scenario.ref(actor.command_path)["sha256"], token=recovery_keeper._CAPABILITY)
            capability.consume(scenario.manager, replacement)
            self.assertTrue(capability.used)
            with self.assertRaises(JenkinsError):
                capability.consume(scenario.manager, replacement)

    def test_archived_interruption_skips_only_the_original_unknown_request_without_fake_result(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            plan = scenario.prepare()
            actor = scenario.actor(stack, plan)
            request = scenario.put(scenario.state / "recovery-lifecycle/history/new-request.json", {
                "schemaVersion": 1, "kind": recovery_keeper.REQUEST_KIND, "action": "recover",
                "requestId": scenario.request_id, "keeper": actor.keeper, "release": actor.release_ref,
                "recoveryDigest": plan.recovery_digest, "runtimeAcceptance": False})
            actor._begin(request, plan=plan)
            descriptor = {"keeper": actor.keeper, "release": actor.release_ref, "bootFileTime": str(scenario.boot),
                          "recoveryDigest": plan.recovery_digest,
                          "lostRecoveryRuntimeInterruption": actor.runtime_interruption["receipt"]}
            scenario.put(actor.descriptor, descriptor)
            self.assertTrue(recovery.interrupted_request(scenario.state, scenario.request_ref))
            self.assertFalse(recovery.interrupted_request(scenario.state, request))
            self.assertFalse(Path(scenario.state / "recovery-lifecycle/results" / (scenario.stop_id + ".json")).exists())
            archive = actor.runtime_interruption["originalCommand"]
            scenario.files[archive["path"]] = canonical_json({"status": "stopped"})
            with self.assertRaises(JenkinsError):
                recovery.interrupted_request(scenario.state, scenario.request_ref)

    def test_existing_blocked_descriptor_cannot_be_adopted_from_json_or_plain_plan(self):
        scenario = DepartedRuntime()
        with scenario.observations() as stack:
            plan = scenario.prepare()
            actor = scenario.actor(stack, plan)
            with self.assertRaises(JenkinsError):
                actor._reviewed_lost_runtime_plan(plan.document)
            with self.assertRaises(JenkinsError):
                actor.start(plan.document)

class RetainedSelectorBindingTests(unittest.TestCase):
    def _seal(self, case, relative, document):
        path = (case.state / relative).absolute()
        raw = canonical_json(document) + b"\n"
        reference = {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest()}
        case.artifacts[str(path)] = (deepcopy(document), raw)
        return reference

    def _case(self):
        state = Path("E:/in-memory/retained-selector/state").absolute()
        case = SimpleNamespace(state=state, artifacts={}, previous_runtime_id="start-111",
                               runtime_id="start-222")
        previous_release = {"path": str(state / "deployment/previous-release.json"), "sha256": "a" * 64}
        current_release = {"path": str(state / "deployment/current-release.json"), "sha256": "b" * 64}
        case.archived = {
            "driverDigest": "c" * 64,
            "root": str(state / "drivers/previous"),
            "launcher": str(state / "drivers/previous/launcher.py"),
            "manifest": str(state / "drivers/previous/manifest.json"),
            "steps": str(state / "drivers/previous/steps.groovy"),
            "runtimeOperationId": case.previous_runtime_id,
            "recoveryRelease": previous_release,
            "preparation": str(state / "migration-baseline/previous-preparation.json"),
            "supportRegression": {"path": str(state / "migration-baseline/regression.json"), "sha256": "d" * 64},
        }
        case.archive_ref = self._seal(case, "migration-baseline/previous-selector.json", case.archived)
        case.preparation = {
            "previousSelectedDriver": deepcopy(case.archived),
            "previousSelectorSha256": case.archive_ref["sha256"],
            "previousRecoveryRelease": deepcopy(case.archived["recoveryRelease"]),
        }
        preparation_ref = self._seal(case, "migration-baseline/new-preparation.json", case.preparation)
        case.release = {
            "kind": "formal-lost-owner-recovery-release",
            "lostRecoveryRuntime": {"plan": {"path": str(state / "migration-baseline/plan.json"), "sha256": "e" * 64}},
            "preparation": preparation_ref,
            "expectedLifecycleFence": {"deployment/driver.json": case.archive_ref["sha256"]},
        }
        case.interruption_ref = {"path": str(state / "recovery-lifecycle/history/interruption.json"), "sha256": "f" * 64}
        case.interruption = {
            "operationId": case.previous_runtime_id,
            "originalArtifacts": {"previousSelector": {"archive": deepcopy(case.archive_ref)}},
        }
        case.descriptor = {"release": current_release, "lostRecoveryRuntimeInterruption": case.interruption_ref}
        case.selected = {**deepcopy(case.archived), "runtimeOperationId": case.runtime_id}
        return case

    def _artifact(self, case, state, reference, **kwargs):
        if state != case.state or not isinstance(reference, dict) or set(reference) != {"path", "sha256"}:
            raise JenkinsError("lost_recovery_runtime_unproven", "Invalid in-memory artifact reference")
        saved = case.artifacts.get(reference["path"])
        if saved is None or hashlib.sha256(saved[1]).hexdigest() != reference["sha256"]:
            raise JenkinsError("lost_recovery_runtime_unproven", "In-memory artifact reference hash changed")
        return deepcopy(saved[0]), saved[1]

    def _patched(self, case, error=None):
        artifact = Mock(side_effect=lambda state, reference, **kwargs: self._artifact(case, state, reference, **kwargs))
        if error is None:
            validate = Mock(return_value=case.interruption)
        else:
            validate = Mock(side_effect=error)
        stack = ExitStack()
        stack.enter_context(patch.object(recovery, "_artifact", new=artifact))
        stack.enter_context(patch.object(recovery, "validate_interruption_receipt", new=validate))
        return stack, artifact, validate

    def _reject(self, case):
        stack, _, _ = self._patched(case)
        with stack, self.assertRaises(JenkinsError):
            recovery._retained_selector_binding(case.state, case.descriptor, case.release,
                                                case.selected, case.runtime_id)

    def _sync_archive(self, case):
        case.archive_ref = self._seal(case, "migration-baseline/previous-selector.json", case.archived)
        case.interruption["originalArtifacts"]["previousSelector"]["archive"] = deepcopy(case.archive_ref)
        case.release["expectedLifecycleFence"]["deployment/driver.json"] = case.archive_ref["sha256"]
        case.preparation["previousSelectedDriver"] = deepcopy(case.archived)
        case.preparation["previousSelectorSha256"] = case.archive_ref["sha256"]
        case.preparation["previousRecoveryRelease"] = deepcopy(case.archived["recoveryRelease"])
        case.release["preparation"] = self._seal(case, "migration-baseline/new-preparation.json", case.preparation)
        case.selected = {**deepcopy(case.archived), "runtimeOperationId": case.runtime_id}

    def test_direct_binding_returns_none_without_interruption_or_artifact_reads(self):
        case = self._case()
        case.release["kind"] = "legacy-direct-binding"
        case.release.pop("lostRecoveryRuntime")
        case.selected = {
            "recoveryRelease": deepcopy(case.descriptor["release"]),
            "preparation": case.release["preparation"]["path"],
            "driverDigest": "direct-selector",
        }
        stack, artifact, validate = self._patched(case)
        with stack:
            result = recovery._retained_selector_binding(case.state, case.descriptor, case.release,
                                                          case.selected, case.runtime_id)
        self.assertIsNone(result)
        artifact.assert_not_called()
        validate.assert_not_called()

    def test_valid_fallback_returns_only_the_issued_selector_proof(self):
        case = self._case()
        stack, artifact, validate = self._patched(case)
        with stack:
            proof = recovery._retained_selector_binding(case.state, case.descriptor, case.release,
                                                        case.selected, case.runtime_id)
        self.assertEqual({
            "mode": "issued-same-driver-retained-selector",
            "preparation": case.release["preparation"],
            "previousSelector": case.archive_ref,
            "previousRuntimeOperationId": case.previous_runtime_id,
            "runtimeOperationId": case.runtime_id,
        }, proof)
        validate.assert_called_once_with(case.state, case.descriptor, case.release)
        self.assertEqual(2, artifact.call_count)
        self.assertEqual(case.release["preparation"], artifact.call_args_list[0].args[1])
        self.assertEqual(case.archive_ref, artifact.call_args_list[1].args[1])

    def test_fallback_rejects_any_selector_change_or_extra_key(self):
        for mutation in ("change_field", "remove_field", "add_key"):
            with self.subTest(mutation=mutation):
                case = self._case()
                if mutation == "change_field":
                    case.selected["driverDigest"] = "0" * 64
                elif mutation == "remove_field":
                    case.selected.pop("steps")
                else:
                    case.selected["unreviewedSelectorKey"] = True
                self._reject(case)

    def test_fallback_rejects_wrong_reference_hash_fence_and_preparation_ancestry(self):
        for mutation in ("preparation_ref_sha", "archive_ref_sha", "lifecycle_fence_sha",
                         "previous_selector_sha", "previous_selected_driver", "previous_recovery_release"):
            with self.subTest(mutation=mutation):
                case = self._case()
                if mutation == "preparation_ref_sha":
                    case.release["preparation"]["sha256"] = "0" * 64
                elif mutation == "archive_ref_sha":
                    case.interruption["originalArtifacts"]["previousSelector"]["archive"]["sha256"] = "0" * 64
                elif mutation == "lifecycle_fence_sha":
                    case.release["expectedLifecycleFence"]["deployment/driver.json"] = "0" * 64
                elif mutation == "previous_selector_sha":
                    case.preparation["previousSelectorSha256"] = "0" * 64
                    case.release["preparation"] = self._seal(case, "migration-baseline/new-preparation.json", case.preparation)
                elif mutation == "previous_selected_driver":
                    case.preparation["previousSelectedDriver"]["root"] = "foreign-driver-root"
                    case.release["preparation"] = self._seal(case, "migration-baseline/new-preparation.json", case.preparation)
                else:
                    case.preparation["previousRecoveryRelease"] = {"path": "foreign-release", "sha256": "9" * 64}
                    case.release["preparation"] = self._seal(case, "migration-baseline/new-preparation.json", case.preparation)
                self._reject(case)

    def test_fallback_rejects_wrong_predecessor_and_predecessor_equal_to_current_runtime(self):
        for mutation in ("wrong_interruption_predecessor", "same_as_current_runtime"):
            with self.subTest(mutation=mutation):
                case = self._case()
                if mutation == "wrong_interruption_predecessor":
                    case.archived["runtimeOperationId"] = "start-333"
                else:
                    case.archived["runtimeOperationId"] = case.runtime_id
                    case.interruption["operationId"] = case.runtime_id
                self._sync_archive(case)
                self._reject(case)

    def test_fallback_requires_issued_marker_and_propagates_interruption_error(self):
        for mutation in ("wrong_release_kind", "missing_marker"):
            with self.subTest(mutation=mutation):
                case = self._case()
                if mutation == "wrong_release_kind":
                    case.release["kind"] = "foreign-recovery-release"
                else:
                    case.release.pop("lostRecoveryRuntime")
                stack, artifact, validate = self._patched(case)
                with stack, self.assertRaises(JenkinsError):
                    recovery._retained_selector_binding(case.state, case.descriptor, case.release,
                                                        case.selected, case.runtime_id)
                artifact.assert_not_called()
                validate.assert_not_called()

        case = self._case()
        interruption_error = JenkinsError("fixture_interruption_unproven", "Invalid interruption receipt")
        stack, artifact, validate = self._patched(case, error=interruption_error)
        with stack, self.assertRaises(JenkinsError) as raised:
            recovery._retained_selector_binding(case.state, case.descriptor, case.release,
                                                case.selected, case.runtime_id)
        self.assertEqual("fixture_interruption_unproven", raised.exception.code)
        validate.assert_called_once_with(case.state, case.descriptor, case.release)
        artifact.assert_not_called()




if __name__ == "__main__":
    unittest.main()

"""Historical configuration authority is confined to genuine external Stop."""
from copy import deepcopy
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json
from tools.jenkins.deployment import owned_upgrade as upgrade, recovery_keeper, lost_recovery_runtime
from tools.jenkins.tests.test_external_native_stop_upgrade import ExternalStoppedOwner

ROOT = Path(__file__).absolute().parents[3]


class Configuration:
    """Independent archived configuration and reservation fixtures; no runtime."""
    def __init__(self):
        self.state = ROOT / ".jenkins/state"
        self.files = {}
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state,
            home=ROOT / ".jenkins/jenkins_home"), spec=SimpleNamespace(path=ROOT / ".jenkins/deployment-spec.json"),
            java=ROOT / ".jenkins/runtime/jdk/bin/java.exe", war=ROOT / ".jenkins/runtime/jenkins.war")
        self.holds = [
            {"domain": "resource_reservation", "key": str(i), "version": 1, "payload": {
                "owner": "historical-" + str(i), "status": "active", "cpu": 1,
                "memoryBytes": 256, "diskBytes": 64, "heavyWriter": True}}
            for i in range(2)] + [
            {"domain": "pool_writer_hold", "key": str(i), "version": 1,
             "payload": {"owner": "historical-" + str(i), "status": "active"}} for i in range(2)]
        self.old_spec = {"schemaVersion": 1, "repositoryRoot": str(ROOT),
            "controller": {"version": "2.580.1", "java": {"version": "21.0.12.1",
                "executable": str(self.manager.java)}, "listenAddress": "127.0.0.1", "httpPort": 18080,
                "executors": 0, "jenkinsHome": str(self.manager.paths.home), "warPath": str(self.manager.war),
                "logs": str(ROOT / ".jenkins/logs"), "processState": str(self.state)},
            "agent": {"name": "zircon-windows-agent", "executors": 1, "remoteRoot": str(ROOT / ".jenkins/agent")},
            "storage": {"allowedPhysicalRoots": [r"D:\cargo-targets", r"E:\cargo-targets", r"F:\cargo-targets"],
                "buildRoot": r"D:\cargo-targets", "buildNamespace": "zircon-local/zircon-jenkins",
                "resourcePolicy": {"buildRoot": r"D:\cargo-targets", "cpuBudget": 6,
                    "memoryBudget": 12288, "diskBudget": 40960, "maxHeavyWriters": 3,
                    "quarantineReserve": {"executionIds": ["historical-0", "historical-1"], "countedInBudget": True}}},
            "productionResourceDefaults": {"cpuBudget": 2, "maxHeavyWriters": 1, "activateAfter": "pending"}}
        self.old_workflow = {"schemaVersion": 1, "storage": {"home": str(self.manager.paths.home),
            "absoluteBuildRoot": r"D:\cargo-targets\zircon-local\zircon-jenkins"},
            "localResourcePolicy": {"allowedPhysicalRoots": self.old_spec["storage"]["allowedPhysicalRoots"],
                "diskReservationAccounting": "existing shared accounting"}}
        config = self.put(ROOT / ".codex/config.toml", {"profile": "zircon-strict"})
        spec = self.put(self.manager.spec.path, self.old_spec)
        workflow = self.put(ROOT / ".jenkins/workflow-spec.json", self.old_workflow)
        self.archive_spec = self.put(self.state / "deployment/history" /
            ("formal-lifecycle-spec-" + spec["sha256"] + ".json"), self.old_spec)
        self.archive_workflow = self.put(self.state / "deployment/history" /
            ("formal-lifecycle-workflow-" + workflow["sha256"] + ".json"), self.old_workflow)
        self.old_release = {"schemaVersion": 1, "kind": upgrade._OLD_RELEASE_KIND, "scope": "execution-runtime",
            "automaticRestart": False, "queueReplayRequested": False, "runtimeAcceptance": False,
            "cargoAcceptance": False, "config": config, "deploymentSpec": spec, "workflowSpec": workflow,
            "protectedHolds": self.holds}
        self.release_ref = self.put(self.state / "deployment/previous-release.json", self.old_release)
        self.descriptor = {"release": self.release_ref, "protectedHolds": self.holds}
        self.new_spec = deepcopy(self.old_spec)
        self.new_spec["agent"]["executors"] = 2
        policy = self.new_spec["storage"]["resourcePolicy"]
        policy.update(maxHeavyWriters=4, maxNewHeavyWriters=2, skipProvenRootDiskBlockedHeads=True)
        cache = {name: True for name in upgrade._CACHE_PROTECTIONS}
        cache.update(minFreeBytes=35000, maxFiles=64, maxBytes=256, maxSeconds=10)
        self.new_spec["storage"]["postTaskCache"] = cache
        policy["postTaskCache"] = deepcopy(cache)
        self.new_spec["productionResourceDefaults"].update(cpuBudget=6, maxHeavyWriters=2,
            activateAfter="current runtime parallel validation", historicalProtectedReservationsCountAgainstBudget=True)
        self.new_workflow = deepcopy(self.old_workflow)
        self.new_workflow["localResourcePolicy"].update(diskReservationAccounting="per volume incremental growth",
            singleHostAgent={"name": "zircon-windows-agent", "uniquePersistentNode": True, "executors": 2,
                "newHeavyWriters": 2, "protectedHistoricalReservationsCountAgainstAllBudgets": True},
            afterEveryTerminalTask={name: True for name in upgrade._TERMINAL_CACHE_FIELDS})
        self.write_current()

    def put(self, path, value):
        raw = canonical_json(value) + b"\n"
        self.files[str(path)] = raw
        return upgrade._ref(path, raw)

    def write_current(self):
        self.put(self.manager.spec.path, self.new_spec)
        self.put(ROOT / ".jenkins/workflow-spec.json", self.new_workflow)

    def environment(self):
        from contextlib import ExitStack
        stack = ExitStack()
        stack.enter_context(patch.object(upgrade, "_plain"))
        stack.enter_context(patch.object(upgrade, "_raw", side_effect=lambda path: self.files[str(Path(path))]))
        stack.enter_context(patch.object(upgrade, "protected_holds", side_effect=lambda state: deepcopy(self.holds)))
        return stack

    def issue(self):
        return upgrade._historical_configuration(self.manager, self.descriptor)


class HistoricalConfigurationTests(unittest.TestCase):
    def test_authorized_scheduling_retains_archived_layout_and_default_historical_release_is_strict(self):
        memory = Configuration()
        before = deepcopy(memory.files)
        with memory.environment():
            context = memory.issue()
            old_spec = context.validate(memory.manager, memory.release_ref)
            self.assertEqual(old_spec, memory.old_spec)
            self.assertEqual(old_spec["agent"]["executors"], 1)
            self.assertEqual(memory.new_spec["agent"]["executors"], 2)
            with self.assertRaises(JenkinsError):
                upgrade._historical_release(memory.manager, memory.release_ref)
        self.assertEqual(memory.files, before)

    def test_operational_authority_and_unknown_fields_cannot_drift(self):
        mutations = [
            lambda m: m.new_spec["controller"].update(httpPort=18081),
            lambda m: m.new_spec["controller"].update(listenAddress="localhost"),
            lambda m: m.new_spec["controller"].update(jenkinsHome=str(ROOT / ".jenkins/other-home")),
            lambda m: m.new_spec["controller"]["java"].update(executable=str(ROOT / ".jenkins/runtime/other-java.exe")),
            lambda m: m.new_spec["controller"].update(warPath=str(ROOT / ".jenkins/runtime/other.war")),
            lambda m: m.new_spec["controller"].update(credentials={"authenticationMode": "anonymous"}),
            lambda m: m.new_spec["agent"].update(name="second-node"),
            lambda m: m.new_spec["agent"].update(remoteRoot=str(ROOT / ".jenkins/another-agent")),
            lambda m: m.new_spec["storage"].update(buildRoot=r"E:\cargo-targets"),
            lambda m: m.new_spec["storage"].update(buildNamespace="other/namespace"),
            lambda m: m.new_spec["storage"].update(buildRoot=r"D:\cargo-targets\..\cargo-targets"),
            lambda m: m.new_spec["storage"]["resourcePolicy"]["quarantineReserve"].update(executionIds=[]),
            lambda m: m.new_spec["storage"]["resourcePolicy"].update(credentials={"token": "untrusted"}),
            lambda m: m.new_workflow["storage"].update(home=str(ROOT / ".jenkins/other-home")),
            lambda m: m.new_workflow.update(stageContract={"sideEffectsRequire": "caller boolean"}),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                memory = Configuration()
                mutate(memory)
                memory.write_current()
                with memory.environment(), self.assertRaises(JenkinsError):
                    memory.issue()

    def test_budgets_cannot_exclude_protected_holds_and_cleanup_cannot_drop_protections(self):
        mutations = [
            lambda m: m.new_spec["storage"]["resourcePolicy"].update(cpuBudget=1),
            lambda m: m.new_spec["storage"]["resourcePolicy"].update(memoryBudget=511),
            lambda m: m.new_spec["storage"]["resourcePolicy"].update(diskBudget=127),
            lambda m: m.new_spec["storage"]["resourcePolicy"].update(maxHeavyWriters=3),
            lambda m: m.new_spec["productionResourceDefaults"].update(cpuBudget=1),
            lambda m: m.new_spec["productionResourceDefaults"].update(maxHeavyWriters=1),
            lambda m: m.new_spec["agent"].update(executors=True),
            lambda m: m.new_spec["storage"]["postTaskCache"].update(unknownObjectsProtected=False),
            lambda m: m.new_spec["storage"]["resourcePolicy"]["postTaskCache"].update(credentialPath="elsewhere"),
            lambda m: m.new_workflow["localResourcePolicy"]["singleHostAgent"].update(name="migrated"),
            lambda m: m.new_workflow["localResourcePolicy"]["afterEveryTerminalTask"].update(recordActualReclaimedBytes=False),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                memory = Configuration()
                mutate(memory)
                memory.write_current()
                with memory.environment(), self.assertRaises(JenkinsError):
                    memory.issue()

    def test_archive_hash_config_and_exact_current_fence_are_required(self):
        for boundary in ("spec", "workflow", "config", "current"):
            with self.subTest(boundary=boundary):
                memory = Configuration()
                with memory.environment():
                    context = memory.issue()
                    if boundary == "spec":
                        memory.files[memory.archive_spec["path"]] += b" "
                    elif boundary == "workflow":
                        memory.files[memory.archive_workflow["path"]] += b" "
                    elif boundary == "config":
                        memory.put(ROOT / ".codex/config.toml", {"profile": "expanded"})
                    else:
                        memory.new_spec["storage"]["resourcePolicy"]["cpuBudget"] = 7
                        memory.write_current()
                    with self.assertRaises(JenkinsError):
                        context.validate(memory.manager, memory.release_ref)

    def test_interrupted_request_layout_uses_exact_archives_and_preserves_strict_default(self):
        memory = Configuration()
        with memory.environment(), patch.object(lost_recovery_runtime, "_plain"), \
                patch.object(lost_recovery_runtime, "_raw", side_effect=lambda path: memory.files[str(Path(path))]):
            context = memory.issue()
            with self.assertRaises(JenkinsError):
                lost_recovery_runtime._lineage_context(memory.state, memory.old_release)
            inputs = lost_recovery_runtime._lineage_context(memory.state, memory.old_release,
                historical_configuration=context)
            self.assertEqual(inputs.spec.controller, memory.old_spec["controller"])
            self.assertEqual(inputs.java, memory.manager.java)
            self.assertEqual(inputs.war, memory.manager.war)
            for forged in (True, context.document):
                with self.subTest(forged=type(forged)), self.assertRaises(JenkinsError):
                    lost_recovery_runtime.interrupted_request(memory.state, {},
                        historical_configuration=forged)
                with self.subTest(layout=type(forged)), self.assertRaises(JenkinsError):
                    lost_recovery_runtime._lineage_context(memory.state, memory.old_release,
                        historical_configuration=forged)

    def test_json_bool_and_unrelated_release_do_not_issue_context(self):
        memory = Configuration()
        with memory.environment():
            context = memory.issue()
            other = deepcopy(memory.old_release)
            other["requestId"] = "unrelated"
            other_ref = memory.put(memory.state / "deployment/unrelated-release.json", other)
            with self.assertRaises(JenkinsError):
                context.validate(memory.manager, other_ref)
            for forged in (True, {"snapshot": context.snapshot}, context.document):
                with self.subTest(forged=type(forged)), self.assertRaises(JenkinsError):
                    upgrade._historical_release(memory.manager, memory.release_ref, historical_configuration=forged)
            with self.assertRaises(JenkinsError):
                upgrade.HistoricalExternalStop(canonical_json({"claimed": True}), configuration=context)
            with self.assertRaises(JenkinsError):
                upgrade._HistoricalExternalConfiguration(context.snapshot)


class IdleExternalStopTests(unittest.TestCase):
    def test_actual_native_stop_transfers_idle_issuance_without_relabelling_it(self):
        memory = ExternalStoppedOwner()
        memory.descriptor["admission"] = "idle"
        memory.put(memory.paths["stoppedDescriptor"], memory.descriptor)
        memory.stopped_descriptor_ref = memory.copy(memory.paths["stoppedDescriptor"],
            memory.state / "recovery-lifecycle/history/original-idle-descriptor.json")
        memory.retirement["originalBlockedDescriptor"] = memory.stopped_descriptor_ref
        memory.retirement_ref = memory.put(memory.retirement_path, memory.retirement)
        memory.successor["ownedUpgrade"] = memory.marker
        memory.successor_ref = memory.put(memory.successor_path, memory.successor)
        before = deepcopy(memory.files)
        with memory.observations():
            plan = memory.prepare()
            self.assertEqual(plan.document["variant"], upgrade.EXTERNAL_VARIANT)
            self.assertEqual(plan.document["originalBlockedDescriptor"], memory.stopped_descriptor_ref)
            self.assertEqual(memory.descriptor["admission"], "idle")
            self.assertFalse(plan.document["nativeProofSynthesized"])
            self.assertEqual(plan.revalidate(memory.manager)["outcome"], "owned-transfer")
        self.assertEqual(memory.files, before)

    def test_preflight_uses_native_receipt_flag_without_inventing_a_descriptor_flag(self):
        memory = ExternalStoppedOwner()
        memory.descriptor["admission"] = "idle"
        memory.descriptor.pop("nativeProofSynthesized", None)
        runtime = memory.descriptor["runtime"]
        runtime["selected"]["recoveryRelease"] = memory.old_release_ref
        memory.put(memory.paths["stoppedDescriptor"], memory.descriptor)
        for name, value in (("stoppedOperation", runtime["operation"]), ("stoppedHost", runtime["host"]),
                            ("previousSelector", runtime["selected"])):
            memory.put(memory.paths[name], value)
        context = upgrade._HistoricalExternalConfiguration(b"{}", upgrade._HISTORICAL_CONFIG_ISSUED)
        with memory.observations(), patch.object(upgrade, "_historical_configuration", return_value=context), \
                patch.object(upgrade, "_historical_release", return_value=(memory.old_release, memory.targets["a" * 64])), \
                patch.object(recovery_keeper, "_runtime_receipt", return_value={}) as receipt, \
                patch.object(upgrade, "_committed_command"), patch.object(upgrade, "_external_delivery_inventory"):
            memory.self_identity.return_value = memory.identities["keeper"]
            before = deepcopy(memory.files)
            cap = upgrade.prepare_historical_external_stop(memory.manager,
                expected_driver_digest="a" * 64, expected_runtime_operation_id=memory.runtime,
                expected_keeper_identity=memory.identities["keeper"])
            self.assertIs(type(cap), upgrade.HistoricalExternalStop)
            self.assertEqual(cap.document["descriptor"], memory.descriptor)
            self.assertNotIn("nativeProofSynthesized", cap.document["descriptor"])
            self.assertFalse(cap.document["nativeProofSynthesized"])
            self.assertIs(receipt.call_args.kwargs["historical_configuration"], context)
            self.assertEqual(memory.files, before)
            memory.descriptor["runtimeAcceptance"] = True
            memory.put(memory.paths["stoppedDescriptor"], memory.descriptor)
            receipt.reset_mock()
            with self.assertRaises(JenkinsError):
                upgrade.prepare_historical_external_stop(memory.manager,
                    expected_driver_digest="a" * 64, expected_runtime_operation_id=memory.runtime,
                    expected_keeper_identity=memory.identities["keeper"])
            receipt.assert_not_called()

    def test_idle_admission_cannot_substitute_for_a_real_native_stop(self):
        memory = ExternalStoppedOwner()
        memory.descriptor["admission"] = "idle"
        binding = deepcopy(__import__("json").loads(memory.files[memory.binding_ref["path"]]))
        binding["capturedOwner"]["liveIdentities"][1]["creationTime"] = "1"
        reference = memory.put(Path(memory.binding_ref["path"]), binding)
        with memory.observations(), self.assertRaises(JenkinsError):
            upgrade._external_native_stop(memory.manager, memory.descriptor, reference)


if __name__ == "__main__":
    unittest.main()

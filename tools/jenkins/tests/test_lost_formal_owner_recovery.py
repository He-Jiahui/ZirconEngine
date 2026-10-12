"""Lost-owner recovery gates in memory; no service, State, or native work."""
from __future__ import annotations

from contextlib import ExitStack
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.deployment import lost_owner


ROOT = Path(__file__).absolute().parents[3]
_DEFAULT_QUEUE_STATE = (b"<hudson.model.Queue_-State><items/><properties><entry>"
    b"<string>jenkins.model.queue.QueueIdStrategy$DefaultStrategy</string><long>21</long>"
    b"</entry></properties></hudson.model.Queue_-State>")


class Scenario:
    def __init__(self):
        self.state = ROOT / ".jenkins/state"
        self.home = ROOT / ".jenkins/jenkins_home"
        self.request = "1" * 32
        self.manager = SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=self.state, home=self.home))
        self.identities = {}
        for i, name in enumerate(("keeper", "supervisor", "controller", "agent")):
            executable = ROOT / (".jenkins/runtime/python/python.exe" if i < 2
                                 else ".jenkins/runtime/jdk/bin/java.exe")
            self.identities[name] = {"pid": 11 + i, "creationTime": str(134360000000000000 + i),
                                     "executable": str(executable), "commandDigest": str(i + 1) * 64}
        self.owner = {"operation": {"operationId": "start-D", "generation": "start-D", "hostPid": 12,
                                    "homePath": str(self.home), "component": "lifecycle-host", "controlPlaneOnly": False},
                      **{k: self.identities[k] for k in ("supervisor", "controller", "agent")}}
        self.binding = {"repository": str(ROOT), "homePath": str(self.home), "driverDigest": "a" * 64,
                        "driverRoot": str(ROOT / ".jenkins/runtime/drivers" / ("a" * 64)),
                        "endpoint": {"address": "127.0.0.1", "port": 18080}}
        self.holds = [{"domain": domain, "key": key, "version": 7,
                       "payload": {"status": "active", "owner": "legacy", "cpu": 1}}
                      for domain, key in (("resource_reservation", "legacy:legacy"), ("pool_writer_hold", "p"))]
        self.value = {"binding": self.binding, "owner": self.owner, "keeper": self.identities["keeper"],
                      "protectedHolds": self.holds}
        reference = {"path": str(self.state / "deployment/history/current.json"), "sha256": "b" * 64}
        self.issuance = {"anchor": SimpleNamespace(value=self.value, digest="c" * 64), "owner": self.owner,
                         "operation": {**self.owner["operation"], **self.owner["supervisor"], "state": "running"},
                         "host": {"operationId": "start-D", "status": "running"},
                         "selected": {"driverDigest": "a" * 64, "root": self.binding["driverRoot"]},
                         "descriptor": {"runtimeReceipt": reference, "keeperRecord": reference},
                         "descriptorSha256": "d" * 64, "startupBodyDigest": "e" * 64,
                         "reader": SimpleNamespace(repo=ROOT)}
        self.classification = {"activeExecutions": [], "historicalExecutions": [{"executionId": "legacy"}],
                               "protectedHolds": self.holds, "closedBeforeLaunchExecutions": []}
        self.execution = {"recordsDigest": "f" * 64, "classification": self.classification,
                          "protectedHolds": self.holds, "protectedHoldsDigest": digest(self.holds),
                          "resourceInventoryDigest": "9" * 64,
                          "nativeProofIssued": False, "executionHoldsReleaseAllowed": False}
        self.containment = {"driverDigest": "a" * 64, "nativeTerminationClaimed": False}
        self.policy = {"controllerExecutors": 0, "queueSha256": None, "configurationSha256": "8" * 64}
        self.home_observation = {"exclusiveReadProbes": [{"file": "config.xml", "bytes": 20, "modifiedNs": 100}],
                                 "staleMarkersDeleted": False}
        self.fence = {"formal-lifecycle/keeper.json": "d" * 64}

    def observations(self):
        stack = ExitStack()
        self.unused = stack.enter_context(patch.object(lost_owner, "_assert_request_unused"))
        self.issuance_probe = stack.enter_context(patch.object(lost_owner, "_verified_issuance", return_value=self.issuance))
        stack.enter_context(patch.object(lost_owner, "_verified_containment", return_value=self.containment))
        self.fence_probe = stack.enter_context(patch.object(lost_owner, "_lifecycle_fence", return_value=self.fence))
        self.boot_probe = stack.enter_context(patch.object(lost_owner, "windows_boot_filetime", return_value=134359000000000000))
        self.departure_probe = stack.enter_context(patch.object(lost_owner, "observe_departed_identity",
            side_effect=lambda expected: {"pid": expected["pid"], "expectedCreationTime": expected["creationTime"], "status": "absent"}))
        self.inventory_probe = stack.enter_context(patch.object(lost_owner, "runtime_owner_inventory", return_value=[]))
        self.execution_probe = stack.enter_context(patch.object(lost_owner, "_execution_gate", return_value=self.execution))
        stack.enter_context(patch.object(lost_owner, "_home_policy", return_value=self.policy))
        stack.enter_context(patch.object(lost_owner, "observe_home_available", return_value=self.home_observation))
        self.endpoint_probe = stack.enter_context(patch.object(lost_owner, "check_port_available"))
        return stack

    def prepare(self, scope="execution-runtime"):
        return lost_owner.prepare_lost_formal_owner_recovery(self.manager, request_id=self.request, scope=scope)


class LostOwnerRecoveryTests(unittest.TestCase):
    def test_same_boot_departure_issues_only_interruption_authority_and_preserves_holds(self):
        scenario = Scenario()
        before = deepcopy(scenario.holds)
        with scenario.observations():
            plan = scenario.prepare()
            evidence = plan.revalidate(scenario.manager)
            self.assertEqual(16, scenario.departure_probe.call_count)
            scenario.endpoint_probe.assert_called_with("127.0.0.1", 18080)
        self.assertEqual("interrupted", evidence["outcome"])
        self.assertFalse(evidence["controlPlaneOnly"])
        self.assertEqual("start-D", evidence["operationId"])
        self.assertEqual(plan.recovery_digest, evidence["recoveryDigest"])
        self.assertEqual(before, evidence["executionQuarantine"]["protectedHolds"])
        self.assertEqual(before, scenario.holds)
        for field in ("nativeProofIssued", "historicalEofClaimed", "bootBoundaryClaimed",
                      "runtimeAcceptance", "validationAcceptanceAllowed", "executionHoldsReleaseAllowed",
                      "historicalHoldsReleased"):
            self.assertIs(evidence[field], False)
        for field in ("nativeTerminationProof", "agentTerminationProof", "stdoutEof", "stderrEof", "childrenGone"):
            self.assertNotIn(field, evidence)

    def test_quiet_management_is_explicit_and_cannot_be_promoted_by_mutating_document(self):
        scenario = Scenario()
        with scenario.observations():
            plan = scenario.prepare("quiet-management")
            copied = plan.document
            copied["scope"] = "execution-runtime"
            copied["controlPlaneOnly"] = False
            self.assertTrue(plan.revalidate(scenario.manager)["controlPlaneOnly"])

    def test_json_cannot_deserialize_a_recovery_capability(self):
        with self.assertRaises(JenkinsError):
            lost_owner.LostFormalOwnerRecovery(canonical_json({"kind": lost_owner.KIND}))

    def test_each_live_or_inaccessible_component_blocks_without_acceptance(self):
        for role in ("keeper", "supervisor", "controller", "agent"):
            for code in ("deployment_owner_live", "deployment_identity_inconclusive"):
                with self.subTest(role=role, code=code):
                    scenario = Scenario()
                    with scenario.observations():
                        def departure(expected):
                            if expected == scenario.identities[role]:
                                raise JenkinsError(code, "injected live/inaccessible observation")
                            return {"pid": expected["pid"], "expectedCreationTime": expected["creationTime"], "status": "absent"}
                        scenario.departure_probe.side_effect = departure
                        with self.assertRaises(JenkinsError):
                            scenario.prepare()
                        scenario.endpoint_probe.assert_not_called()

    def test_possible_runtime_owner_and_endpoint_conflict_each_block(self):
        scenario = Scenario()
        with scenario.observations():
            scenario.inventory_probe.return_value = [scenario.identities["controller"]]
            with self.assertRaises(JenkinsError):
                scenario.prepare()
        with scenario.observations():
            scenario.endpoint_probe.side_effect = JenkinsError("controller_port_unavailable", "occupied")
            with self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_revalidation_rejects_lineage_change_and_hold_or_budget_change(self):
        for field in ("descriptorSha256", "startupBodyDigest"):
            with self.subTest(field=field):
                scenario = Scenario()
                with scenario.observations():
                    plan = scenario.prepare()
                    scenario.issuance[field] = "0" * 64
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)
        for field in ("protectedHoldsDigest", "resourceInventoryDigest", "recordsDigest"):
            with self.subTest(field=field):
                scenario = Scenario()
                with scenario.observations():
                    plan = scenario.prepare()
                    scenario.execution[field] = "0" * 64
                    with self.assertRaises(JenkinsError):
                        plan.revalidate(scenario.manager)

    def test_inventory_boot_and_lifecycle_races_block_preparation(self):
        for name in ("inventory_probe", "boot_probe", "fence_probe"):
            with self.subTest(name=name):
                scenario = Scenario()
                with scenario.observations():
                    probe = getattr(scenario, name)
                    prior = probe.return_value
                    changed = [scenario.identities["agent"]] if name == "inventory_probe" else (
                        prior + 1 if name == "boot_probe" else {"formal-lifecycle/keeper.json": "0" * 64})
                    probe.side_effect = [prior, changed]
                    with self.assertRaises(JenkinsError):
                        scenario.prepare()

    def test_invalid_request_scope_and_replayed_request_cannot_issue_authority(self):
        scenario = Scenario()
        with scenario.observations():
            for request, scope in (("../request", "execution-runtime"), (scenario.request, "force")):
                with self.assertRaises(JenkinsError):
                    lost_owner.prepare_lost_formal_owner_recovery(scenario.manager, request_id=request, scope=scope)
            scenario.unused.side_effect = JenkinsError("formal_lost_owner_replay", "used")
            with self.assertRaises(JenkinsError):
                scenario.prepare()

    def test_pid_reuse_requires_strictly_later_creation_time(self):
        scenario = Scenario()
        expected = scenario.identities["keeper"]
        for birth in (expected["creationTime"], str(int(expected["creationTime"]) - 1), "unknown"):
            observed = {**expected, "creationTime": birth}
            with patch.object(lost_owner, "observe_departed_identity", return_value={
                "pid": expected["pid"], "expectedCreationTime": expected["creationTime"],
                "status": "pid-reused", "observedIdentity": observed}), self.assertRaises(JenkinsError):
                lost_owner._departure(expected)
        observed = {**expected, "creationTime": str(int(expected["creationTime"]) + 1)}
        with patch.object(lost_owner, "observe_departed_identity", return_value={
            "pid": expected["pid"], "expectedCreationTime": expected["creationTime"],
            "status": "pid-reused", "observedIdentity": observed}):
            self.assertEqual("pid-reused", lost_owner._departure(expected)["status"])

    def test_sealed_source_review_is_independent_of_driver_digest_and_rejects_unknown_bytes(self):
        scenario = Scenario()
        expected = lost_owner._REVIEWED_CONTAINMENT
        manifest = {"files": {name: {"sha256": checksum} for name, checksum in expected.items()}}
        def checksum(path):
            return next((value for name, value in expected.items() if str(path).endswith(str(Path(name)))), "f" * 64)
        with patch.object(lost_owner, "verify_driver") as verify, \
                patch.object(lost_owner, "_read", return_value=manifest), \
                patch.object(lost_owner, "_sha", side_effect=checksum):
            evidence = lost_owner._verified_containment(scenario.issuance)
            self.assertEqual(scenario.issuance["selected"]["driverDigest"], evidence["driverDigest"])
            verify.assert_called_once()
            manifest["files"]["tools/jenkins/deployment/host.py"]["sha256"] = "0" * 64
            with self.assertRaises(JenkinsError):
                lost_owner._verified_containment(scenario.issuance)

    def test_current_execution_and_mutated_historical_hold_each_block(self):
        scenario = Scenario()
        rows = [{"domain": "execution", "key": "current", "version": 1,
                 "payload": {"operationId": "start-D", "status": "running"}}]
        classified = deepcopy(scenario.classification)
        with patch.object(lost_owner, "execution_inventory", return_value=rows), \
                patch.object(lost_owner, "classify_executions", return_value=classified), \
                patch.object(lost_owner, "protected_holds", return_value=scenario.holds), \
                patch.object(lost_owner, "_resource_inventory", return_value=[{"policy": "bounded"}]):
            classified["activeExecutions"] = ["current"]
            with self.assertRaises(JenkinsError):
                lost_owner._execution_gate(scenario.manager, scenario.issuance)
            classified["activeExecutions"] = []
            classified["protectedHolds"][0]["version"] += 1
            with self.assertRaises(JenkinsError):
                lost_owner._execution_gate(scenario.manager, scenario.issuance)

    def test_existing_native_proof_cannot_drop_command_or_stream_identity(self):
        scenario = Scenario()
        native = scenario.identities["agent"]
        proof = {"identity": native, "complete": True, "childrenGone": True, "activeProcesses": 0,
                 "stdoutEof": True, "stderrEof": True, "processExitCode": 0}
        row = {"domain": "native_job", "key": "job", "version": 2,
               "payload": {"pid": native["pid"], "birthToken": native["creationTime"], "executable": native["executable"],
                           "status": "terminal", "completeProof": proof, "terminalProof": proof}}
        with patch.object(lost_owner, "execution_inventory", return_value=[row]), \
                patch.object(lost_owner, "classify_executions", return_value=scenario.classification), \
                patch.object(lost_owner, "protected_holds", return_value=scenario.holds), \
                patch.object(lost_owner, "_resource_inventory", return_value=[]):
            lost_owner._execution_gate(scenario.manager, scenario.issuance)
            row["payload"]["terminalProof"] = {**proof, "stdoutEof": False}
            with self.assertRaises(JenkinsError):
                lost_owner._execution_gate(scenario.manager, scenario.issuance)

    def test_home_queue_requires_reconciliation_before_execution_runtime(self):
        scenario = Scenario()
        config = b"<hudson><numExecutors>0</numExecutors></hudson>"
        for queue in (b"<queue><items><item/></items></queue>", b"<queue><futureItems/></queue>"):
            with patch.object(lost_owner, "_plain"), \
                    patch.object(Path, "exists", return_value=True), \
                    patch.object(lost_owner, "_raw", side_effect=lambda path: config if path.name == "config.xml" else queue), \
                    self.assertRaises(JenkinsError):
                lost_owner._home_policy(scenario.home, "execution-runtime")

    def test_default_strategy_empty_queue_retains_original_bytes_and_counter(self):
        home = ROOT / ".jenkins/jenkins_home"
        config = b"<hudson><numExecutors>0</numExecutors></hudson>"
        for queue in (_DEFAULT_QUEUE_STATE, _DEFAULT_QUEUE_STATE.replace(b"<long>21</long>", b"<long>0</long>"),
                      _DEFAULT_QUEUE_STATE.replace(b"<long>21</long>", b"<long>9223372036854775807</long>"),
                      b"<queue><items/><pendingItems/><counter>21</counter></queue>",
                      b"<hudson.model.Queue_-State><items/><pendingItems/><counter>0</counter></hudson.model.Queue_-State>",
                      b"<queue/>"):
            with self.subTest(queue=queue), ExitStack() as stack:
                stack.enter_context(patch.object(lost_owner, "_plain"))
                stack.enter_context(patch.object(Path, "exists", return_value=True))
                reader = stack.enter_context(patch.object(lost_owner, "_raw", side_effect=lambda path:
                    config if path.name == "config.xml" else queue))
                evidence = lost_owner._home_policy(home, "execution-runtime")
                self.assertEqual(hashlib.sha256(queue).hexdigest(), evidence["queueSha256"])
                self.assertEqual(hashlib.sha256(config).hexdigest(), evidence["configurationSha256"])
                self.assertEqual(0, evidence["controllerExecutors"])
                self.assertEqual([home / "config.xml", home / "queue.xml"], [call.args[0] for call in reader.call_args_list])

    def test_default_queue_metadata_rejects_work_unknown_fields_duplicates_and_attributes(self):
        entry = (b"<entry><string>jenkins.model.queue.QueueIdStrategy$DefaultStrategy</string>"
                 b"<long>21</long></entry>")
        invalid = (
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b"<items><item/></items>"),
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b"<items/><pendingItems><item/></pendingItems>"),
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b"<items>pending</items>"),
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b"<items/><items/>"),
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b"<items/><futureItems/>"),
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b""),
            _DEFAULT_QUEUE_STATE.replace(b"<properties>", b"<properties/><properties>"),
            _DEFAULT_QUEUE_STATE.replace(entry, entry + entry),
            _DEFAULT_QUEUE_STATE.replace(b"</long>", b"</long><long>22</long>"),
            _DEFAULT_QUEUE_STATE.replace(b"<long>21</long>", b"<long><counter>21</counter></long>"),
            _DEFAULT_QUEUE_STATE.replace(b"</string>", b"<unknown/></string>"),
            _DEFAULT_QUEUE_STATE.replace(b"$DefaultStrategy", b"$UnknownStrategy"),
            _DEFAULT_QUEUE_STATE.replace(b"<string>", b"<string> "),
            _DEFAULT_QUEUE_STATE.replace(b"<entry>", b"<entry>unknown"),
            _DEFAULT_QUEUE_STATE.replace(b"<properties>", b"<properties>unknown"),
            _DEFAULT_QUEUE_STATE.replace(b"<properties>", b"<properties><unknown/>"),
            _DEFAULT_QUEUE_STATE.replace(b"<long>", b"<long changed='true'>"),
            _DEFAULT_QUEUE_STATE.replace(b"<entry>", b"<entry class='unknown'>"),
            _DEFAULT_QUEUE_STATE.replace(b"<items/>", b"<items class='unknown'/>"),
            _DEFAULT_QUEUE_STATE.replace(b"<hudson.model.Queue_-State>", b"<hudson.model.Queue_-State changed='true'>"),
            _DEFAULT_QUEUE_STATE.replace(b"</long>", b"</long>unknown"),
            _DEFAULT_QUEUE_STATE.replace(b"Queue_-State", b"Queue_-Unknown"),
            _DEFAULT_QUEUE_STATE.replace(b"hudson.model.Queue_-State", b"queue"),
            b"<queue><items/><pendingItems/><pendingItems/></queue>",
            b"<queue><counter>1</counter><counter>2</counter></queue>",
        )
        config = b"<hudson><numExecutors>0</numExecutors></hudson>"
        for queue in invalid:
            with self.subTest(queue=queue), patch.object(lost_owner, "_plain"), \
                    patch.object(Path, "exists", return_value=True), \
                    patch.object(lost_owner, "_raw", side_effect=lambda path: config if path.name == "config.xml" else queue), \
                    self.assertRaises(JenkinsError) as raised:
                lost_owner._home_policy(ROOT / ".jenkins/jenkins_home", "execution-runtime")
            self.assertEqual("formal_lost_owner_queue_pending", raised.exception.code)

    def test_saved_queue_counter_requires_an_ascii_nonnegative_java_long(self):
        config = b"<hudson><numExecutors>0</numExecutors></hudson>"
        for number in (b"", b" ", b"-1", b"+1", b"1.0", b"1e2", b"twenty-one", b"9223372036854775808",
                       "２１".encode("utf-8"), b"9" * 5000):
            for shape in ("properties", "legacy"):
                queue = (_DEFAULT_QUEUE_STATE.replace(b"<long>21</long>", b"<long>" + number + b"</long>")
                         if shape == "properties" else b"<queue><items/><counter>" + number + b"</counter></queue>")
                with self.subTest(number=number[:30], shape=shape), patch.object(lost_owner, "_plain"), \
                        patch.object(Path, "exists", return_value=True), \
                        patch.object(lost_owner, "_raw", side_effect=lambda path: config if path.name == "config.xml" else queue), \
                        self.assertRaises(JenkinsError) as raised:
                    lost_owner._home_policy(ROOT / ".jenkins/jenkins_home", "execution-runtime")
                self.assertEqual("formal_lost_owner_queue_pending", raised.exception.code)

    def test_malformed_home_xml_has_a_typed_rejection(self):
        valid_config = b"<hudson><numExecutors>0</numExecutors></hudson>"
        for config, queue, code in ((valid_config, b"<hudson.model.Queue_-State><items>", "formal_lost_owner_queue_pending"),
                                    (b"<hudson><numExecutors>0", _DEFAULT_QUEUE_STATE, "formal_lost_owner_home_unproven")):
            with self.subTest(code=code), patch.object(lost_owner, "_plain"), \
                    patch.object(Path, "exists", return_value=True), \
                    patch.object(lost_owner, "_raw", side_effect=lambda path: config if path.name == "config.xml" else queue), \
                    self.assertRaises(JenkinsError) as raised:
                lost_owner._home_policy(ROOT / ".jenkins/jenkins_home", "execution-runtime")
            self.assertEqual(code, raised.exception.code)

    def test_duplicate_recovery_json_is_rejected(self):
        with self.assertRaises(JenkinsError):
            lost_owner._document(b'{"kind":"trusted","kind":"forged"}')


if __name__ == "__main__":
    unittest.main()

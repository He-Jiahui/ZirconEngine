from __future__ import annotations
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from contextlib import ExitStack
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json
from tools.jenkins.deployment import recovery
from tools.jenkins.state import State


class ManagementRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name).absolute()
        self.home = self.repo / ".jenkins/jenkins_home"
        self.home.mkdir(parents=True)
        (self.home / "config.xml").write_text("<hudson><numExecutors>0</numExecutors></hudson>")
        init = self.home / "init.groovy.d/90-zircon-repository-build-root.groovy"
        init.parent.mkdir()
        init.write_bytes(b"Jenkins.get().doQuietDown()")
        self.driver = "a" * 64
        self.root = self.repo / ".jenkins/runtime/drivers" / self.driver
        self.root.mkdir(parents=True)
        (self.root / "driver-manifest.json").write_bytes(b"verified fixture manifest")
        critical = {}
        for name in ("tools/jenkins/processes/native.py", "tools/jenkins/processes/job.py",
                     "tools/jenkins/deployment/host.py", "tools/jenkins/deployment/lifecycle.py"):
            file = self.root / name
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(name.encode())
            critical[name] = hashlib.sha256(file.read_bytes()).hexdigest()
        self.op = {"operationId": "old", "generation": "old", "pid": 11, "hostPid": 11,
                   "creationTime": "100", "executable": str(self.repo / ".jenkins/runtime/python/python.exe"),
                   "commandDigest": "b" * 64, "homePath": str(self.home), "controlPlaneOnly": True}
        controller = {"pid": 12, "creationTime": "110", "executable": str(self.repo / ".jenkins/runtime/jdk/bin/java.exe"),
                      "commandDigest": "c" * 64}
        self.host = {"operationId": "old", "generation": "old", "hostPid": 11, "status": "running",
                     "controllerIdentity": controller, "controlPlaneOnly": True,
                     "agentLaunchAttempted": False, "agentIdentity": None}
        self.selected = {"driverDigest": self.driver, "runtimeOperationId": "old", "root": str(self.root)}
        self.stamp = {"recordId": "123", "startTime": "2026-10-04T03:47:45.5000000Z"}
        baseline = self.repo / ".jenkins/state/migration-baseline"
        baseline.mkdir(parents=True)
        self.probe_path = baseline / "controller-containment-probe-fixture.json"
        self.probe = {"driverDigest": self.driver,
                      "manifestSha256": hashlib.sha256((self.root / "driver-manifest.json").read_bytes()).hexdigest(),
                      "criticalFiles": critical, "accepted": True, "rootAssignedBeforeResume": True,
                      "rootJobMembershipBeforeResume": True, "anonymousJob": True,
                      "lastHandleCloseObserved": True, "stdoutEof": True, "stderrEof": True,
                      "historicalNativeProofIssued": False, "validationAcceptanceAllowed": False,
                      "executionHoldsReleaseAllowed": False, "jobHandleFlags": 0, "limitFlags": 0x2000,
                      "activeProcessesBeforeClose": 2, "membersBeforeClose": [{"pid": 21}, {"pid": 22}],
                      "retainedMemberExitCodes": {"21": 1, "22": 1},
                      "fixtureAnnouncement": {"breakawayRejected": True, "breakawayWinerror": 5}}
        self.probe_path.write_bytes(canonical_json(self.probe))
        self.audit_path = self.repo / ".jenkins/state/deployment/history/audit.json"
        self.audit_path.parent.mkdir(parents=True)
        self.audit = {"schemaVersion": 1, "kind": "sealed-control-plane-management-audit",
                      "scope": "quiet-management-only", "driverDigest": self.driver,
                      "runtimeOperationId": "old", "repositoryRoot": str(self.repo), "homePath": str(self.home),
                      "operationBinding": copy.deepcopy(self.op), "hostBinding": copy.deepcopy(self.host),
                      "nativeProofIssued": False, "historicalEofClaimed": False,
                      "validationAcceptanceAllowed": False, "executionHoldsReleaseAllowed": False,
                      "manifestSha256": self.probe["manifestSha256"], "criticalFiles": critical,
                      "probe": {"path": str(self.probe_path),
                                "sha256": hashlib.sha256(self.probe_path.read_bytes()).hexdigest()},
                      "managementBootStamp": self.stamp,
                      "controlPolicy": {"controllerExecutors": 0, "agentLaunchAttempted": False, "brokerDisabled": True,
                                        "requiredStartEnvironment": {"ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY": "1"},
                                        "quietInitSha256": hashlib.sha256(init.read_bytes()).hexdigest()}}
        self._save_audit()
        (self.repo / ".jenkins/state/deployment-operation.json").write_bytes(canonical_json(self.op))
        (self.repo / ".jenkins/state/deployment/host.json").write_bytes(canonical_json(self.host))
        (self.repo / ".jenkins/state/deployment/driver.json").write_bytes(canonical_json(self.selected))

    def _save_audit(self):
        self.audit_path.write_bytes(canonical_json(self.audit))
        self.map = {self.driver: (self.audit_path.relative_to(self.repo).as_posix(),
                                  hashlib.sha256(self.audit_path.read_bytes()).hexdigest())}

    def _call(self, **overrides):
        return recovery.control_only_management_evidence(
            overrides.get("operation", self.op), overrides.get("host", self.host), str(self.home),
            self.repo, overrides.get("selected", self.selected), "127.0.0.1", 18080)

    def _observations(self):
        stack = ExitStack()
        stack.enter_context(patch.object(recovery, "_CONTROL_PLANE_MANAGEMENT_AUDITS", self.map))
        stack.enter_context(patch("tools.jenkins.deployment.driver.verify_driver"))
        stack.enter_context(patch("tools.jenkins.deployment.management_boot.observe_management_boot_stamp",
                                  return_value=self.stamp))
        stack.enter_context(patch("tools.jenkins.deployment.startup_observations.observe_departed_identity",
                                  side_effect=lambda expected: {"pid": expected["pid"],
                                      "expectedCreationTime": expected["creationTime"], "status": "absent"}))
        stack.enter_context(patch("tools.jenkins.deployment.startup_observations.runtime_owner_inventory", return_value=[]))
        stack.enter_context(patch("tools.jenkins.deployment.startup_observations.observe_home_available", return_value={}))
        stack.enter_context(patch("tools.jenkins.deployment.lifecycle.check_port_available"))
        return stack

    def test_management_evidence_preserves_holds_and_does_not_issue_native_proof(self):
        state = State(self.repo / ".jenkins/state/coordination.sqlite3")
        state.put("resource_reservation", "r", {"owner": "historical", "status": "active"})
        state.put("pool_writer_hold", "p", {"owner": "historical", "status": "active"})
        before = state.list("resource_reservation"), state.list("pool_writer_hold")
        with self._observations():
            evidence = self._call()
        self.assertEqual("interrupted", evidence["outcome"])
        self.assertTrue(evidence["controlPlaneOnly"])
        self.assertTrue(evidence["brokerDisabled"])
        for name in ("nativeProofIssued", "validationAcceptanceAllowed", "executionHoldsReleaseAllowed"):
            self.assertIs(evidence[name], False)
        self.assertNotIn("nativeTerminationProof", evidence)
        self.assertNotIn("stdoutEof", evidence)
        self.assertEqual(before, (state.list("resource_reservation"), state.list("pool_writer_hold")))

    def test_audit_byte_tamper_is_rejected(self):
        self.audit_path.write_bytes(self.audit_path.read_bytes() + b" ")
        with self._observations(), self.assertRaises(JenkinsError):
            self._call()

    def test_changed_owner_driver_generation_agent_and_home_bindings_are_rejected(self):
        cases = [
            {"operation": {**self.op, "generation": "new"}},
            {"operation": {**self.op, "creationTime": "200"}},
            {"operation": {**self.op, "pid": 44}},
            {"operation": {**self.op, "homePath": "other"}},
            {"operation": {**self.op, "controlPlaneOnly": False}},
            {"host": {**self.host, "controllerIdentity": {**self.host["controllerIdentity"], "creationTime": "201"}}},
            {"host": {**self.host, "agentLaunchAttempted": True}},
            {"host": {**self.host, "agentIdentity": self.host["controllerIdentity"]}},
            {"host": {**self.host, "controlPlaneOnly": False}},
            {"selected": {**self.selected, "runtimeOperationId": "new"}},
            {"selected": {**self.selected, "root": str(self.repo / "other")}},
            {"selected": {**self.selected, "driverDigest": "d" * 64}},
        ]
        for value in cases:
            with self.subTest(value=value), self._observations(), self.assertRaises(JenkinsError):
                self._call(**value)

    def test_live_inconclusive_inventory_home_and_endpoint_probes_block_recovery(self):
        for target, options in [
            ("observe_departed_identity", {"side_effect": JenkinsError("live", "Live old owner")}),
            ("observe_departed_identity", {"side_effect": JenkinsError("unknown", "Inaccessible owner")}),
            ("runtime_owner_inventory", {"return_value": [{"pid": 55}]}),
            ("observe_home_available", {"side_effect": JenkinsError("busy", "Home busy")}),
        ]:
            with self.subTest(target=target, options=options), self._observations(), \
                 patch("tools.jenkins.deployment.startup_observations." + target, **options), \
                 self.assertRaises(JenkinsError):
                self._call()
        with self._observations(), patch("tools.jenkins.deployment.lifecycle.check_port_available",
                side_effect=JenkinsError("busy", "Port busy")), self.assertRaises(JenkinsError):
            self._call()

    def test_boot_change_before_or_during_observation_is_rejected(self):
        for stamps in ([{"recordId": "new"}], [self.stamp, {"recordId": "new"}]):
            with self.subTest(stamps=stamps), self._observations(), \
                 patch("tools.jenkins.deployment.management_boot.observe_management_boot_stamp", side_effect=stamps), \
                 self.assertRaises(JenkinsError):
                self._call()

    def test_current_or_ambiguous_execution_owner_is_rejected(self):
        for payload in ({"operationId": "old", "generation": "old", "status": "running"},
                        {"status": "running"}):
            state = State(self.repo / ".jenkins/state/coordination.sqlite3")
            state.put("execution", "e", payload)
            with self.subTest(payload=payload), self._observations(), self.assertRaises(JenkinsError):
                self._call()

    def test_probe_manifest_containment_and_quiet_policy_changes_are_rejected(self):
        for change in (
            {"nativeProofIssued": True}, {"runtimeOperationId": "new"},
            {"criticalFiles": {}}, {"manifestSha256": "0" * 64},
            {"controlPolicy": {**self.audit["controlPolicy"], "brokerDisabled": False}},
        ):
            original = copy.deepcopy(self.audit)
            self.audit.update(change); self._save_audit()
            with self.subTest(change=change), self._observations(), self.assertRaises(JenkinsError):
                self._call()
            self.audit = original; self._save_audit()
        self.probe_path.write_bytes(self.probe_path.read_bytes() + b" ")
        with self._observations(), self.assertRaises(JenkinsError):
            self._call()


    def _client_review(self):
        script = self.repo / "workbench-control.py"
        script.write_bytes(b"foreign client fixture")
        client = {"pid": 71, "creationTime": "710",
                  "executable": str(self.repo / ".jenkins/runtime/python/python.exe"),
                  "commandDigest": ""}
        proof = {"identity": {k: client[k] for k in ("pid", "creationTime", "executable")},
                 "kind": "unrelated-workbench-client",
                 "evidence": {"path": str(script), "sha256": hashlib.sha256(script.read_bytes()).hexdigest()},
                 "commandLineSha256": "d" * 64, "jenkinsNativeRecordReferences": []}
        self.audit["reviewedManagementClients"] = [proof]
        self._save_audit()
        return client, script

    def test_reviewed_client_is_preserved_and_reobserved_without_owner_adoption(self):
        client, script = self._client_review()
        with self._observations(), patch(
                "tools.jenkins.deployment.startup_observations.runtime_owner_inventory",
                return_value=[client]) as inventory:
            evidence = self._call()
        self.assertEqual([], evidence["runtimeOwnerInventory"])
        self.assertEqual([client], evidence["runtimeObservation"]["observedRuntimeProcesses"])
        preserved = evidence["runtimeObservation"]["preservedManagementClients"]
        self.assertEqual(client, preserved[0]["identity"])
        self.assertIs(preserved[0]["stopped"], False)
        self.assertIs(preserved[0]["adopted"], False)
        self.assertEqual(2, inventory.call_count)
        self.assertIs(evidence["nativeProofIssued"], False)
        self.assertIs(evidence["executionHoldsReleaseAllowed"], False)

    def test_unknown_reused_changed_script_and_changing_client_inventory_block(self):
        client, script = self._client_review()
        cases = [[{**client, "pid": 72}], [{**client, "creationTime": "711"}],
                 [{**client, "executable": str(self.repo / ".jenkins/runtime/jdk/bin/java.exe")}]]
        for inventory in cases:
            with self.subTest(inventory=inventory), self._observations(), patch(
                    "tools.jenkins.deployment.startup_observations.runtime_owner_inventory",
                    return_value=inventory), self.assertRaises(JenkinsError):
                self._call()
        script.write_bytes(b"changed foreign client")
        with self._observations(), patch(
                "tools.jenkins.deployment.startup_observations.runtime_owner_inventory",
                return_value=[client]), self.assertRaises(JenkinsError):
            self._call()
        script.write_bytes(b"foreign client fixture")
        with self._observations(), patch(
                "tools.jenkins.deployment.startup_observations.runtime_owner_inventory",
                side_effect=[[client], []]), self.assertRaises(JenkinsError):
            self._call()


    def _unbound_python(self):
        return {"pid": 75, "creationTime": "750",
                "executable": str(self.repo / ".jenkins/runtime/python/python.exe"),
                "commandDigest": ""}

    def test_unbound_python_diagnostic_does_not_block_quiet_management_or_release_holds(self):
        state = State(self.repo / ".jenkins/state/coordination.sqlite3")
        state.put("pool_writer_hold", "historic", {"status": "active", "owner": "unknown-history"})
        before = state.list("pool_writer_hold")
        client = self._unbound_python()
        with self._observations(), patch(
                "tools.jenkins.deployment.startup_observations.runtime_owner_inventory", return_value=[client]):
            evidence = self._call()
        self.assertEqual([], evidence["runtimeOwnerInventory"])
        observation = evidence["runtimeObservation"]["preservedManagementClients"][0]
        self.assertEqual("unbound-python-observation", observation["kind"])
        self.assertEqual(client, observation["identity"])
        for field in ("stopped", "adopted", "homeOwnershipClaimed"):
            self.assertIs(observation[field], False)
        self.assertIs(evidence["controlPlaneOnly"], True)
        self.assertIs(evidence["validationAcceptanceAllowed"], False)
        self.assertIs(evidence["executionHoldsReleaseAllowed"], False)
        self.assertEqual(before, state.list("pool_writer_hold"))

    def test_java_and_durably_bound_or_ambiguous_python_owners_remain_blocking(self):
        state = State(self.repo / ".jenkins/state/coordination.sqlite3")
        client = self._unbound_python()
        cases = [
            ({**client, "executable": str(self.repo / ".jenkins/runtime/jdk/bin/java.exe")}, None),
            (client, {"identity": client}),
            (client, {"pid": client["pid"]}),
        ]
        for owner, binding in cases:
            state.put("native_job", "n", binding or {})
            with self.subTest(owner=owner, binding=binding), self._observations(), patch(
                    "tools.jenkins.deployment.startup_observations.runtime_owner_inventory",
                    return_value=[owner]), self.assertRaises(JenkinsError) as error:
                self._call()
            self.assertEqual("deployment_home_owner_live", error.exception.code)

    def test_historical_reused_pid_is_preserved_without_adopting_current_python(self):
        state = State(self.repo / ".jenkins/state/coordination.sqlite3")
        client = self._unbound_python()
        state.put("native_job", "old", {"identity": {**client, "creationTime": "749"}})
        before = state.list("native_job")
        with self._observations(), patch(
                "tools.jenkins.deployment.startup_observations.runtime_owner_inventory", return_value=[client]):
            evidence = self._call()
        self.assertEqual("unbound-python-observation",
                         evidence["runtimeObservation"]["preservedManagementClients"][0]["kind"])
        self.assertEqual(before, state.list("native_job"))

    def test_selector_mismatch_or_journal_change_during_probe_blocks_management(self):
        selector = self.repo / ".jenkins/state/deployment/driver.json"
        selector.write_bytes(canonical_json({**self.selected, "runtimeOperationId": "other"}))
        with self._observations(), self.assertRaises(JenkinsError) as error:
            self._call()
        self.assertEqual("deployment_management_lifecycle_changed", error.exception.code)
        selector.write_bytes(canonical_json(self.selected))
        journal = self.repo / ".jenkins/state/deployment-operation.json"

        def mutate_journal(_home):
            journal.write_bytes(journal.read_bytes() + b" ")
            return {}

        with self._observations(), patch(
                "tools.jenkins.deployment.startup_observations.observe_home_available",
                side_effect=mutate_journal), self.assertRaises(JenkinsError) as error:
            self._call()
        self.assertEqual("deployment_management_lifecycle_changed", error.exception.code)


    def test_replacement_dispatch_does_not_use_boot_boundary_for_audited_management(self):
        with self._observations(), patch.object(recovery, "control_only_management_evidence", return_value={"ok": True}) as bounded, \
             patch.object(recovery, "previous_boot_deployment_evidence", side_effect=AssertionError("No fallback")):
            self.assertEqual({"ok": True}, recovery.replacement_deployment_evidence(
                self.op, self.host, str(self.home), self.repo, self.selected, "127.0.0.1", 18080))
            bounded.assert_called_once()

if __name__ == "__main__":
    unittest.main()

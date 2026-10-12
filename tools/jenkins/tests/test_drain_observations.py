from __future__ import annotations

import copy
import hashlib
import json
import sqlite3
import tempfile
import unittest
from contextlib import closing
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.deployment import drain_observations as drain
from tools.jenkins.deployment.manager import DeploymentManager


def identity(pid, birth, executable):
    return {"pid": pid, "creationTime": str(birth), "executable": str(executable), "commandDigest": str(pid % 10) * 64}


def proof(value):
    return {"identity": value, "complete": True, "childrenGone": True, "activeProcesses": 0,
            "stdoutEof": True, "stderrEof": True, "processExitCode": 0}


def row(domain, key, payload):
    return {"domain": domain, "key": key, "payload": payload, "version": 1}


class DrainObservationsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.paths = SimpleNamespace(repo=self.root, state=self.root / ".jenkins/state",
                                     home=self.root / ".jenkins/jenkins_home", logs=self.root / ".jenkins/logs")
        self.spec = SimpleNamespace(controller={"listenAddress": "127.0.0.1", "httpPort": 18080}, agent={"name": "agent"})
        self.java = self.root / ".jenkins/runtime/jdk/bin/java.exe"
        self.python = self.root / ".jenkins/runtime/python/python.exe"
        self.manager = DeploymentManager(self.spec, self.paths, self.java, self.root / ".jenkins/runtime/jenkins.war")
        self.owner = {"operationId": "current", "generation": "current", "homePath": str(self.paths.home.absolute()),
                      "supervisor": identity(100, 1000, self.python), "controller": identity(101, 1010, self.java),
                      "agent": identity(102, 1020, self.java), "driverDigest": "a" * 64}
        self.operation = {**self.owner["supervisor"], "component": "lifecycle-host", "operationId": "current",
                          "generation": "current", "homePath": self.owner["homePath"], "hostPid": 100,
                          "state": "running", "controlPlaneOnly": False}
        self.host = {"status": "running", "operationId": "current", "generation": "current", "hostPid": 100,
                     "controllerIdentity": self.owner["controller"], "identity": self.owner["controller"],
                     "agentIdentity": self.owner["agent"], "agentLaunchAttempted": True, "controlPlaneOnly": False}
        self.computers = {"computer": [{"displayName": "Built-In Node", "executors": [], "oneOffExecutors": []},
                        {"displayName": "retired-agent", "numExecutors": 1, "executors": [{"idle": True}], "oneOffExecutors": []},
                        {"displayName": "current-agent", "numExecutors": 1,
                         "executors": [{"idle": True, "currentExecutable": None}], "oneOffExecutors": []}]}
        boot = patch("tools.jenkins.deployment.recovery.windows_boot_filetime", return_value=600)
        self.boot = boot.start()
        self.addCleanup(boot.stop)

    def write(self, path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value), encoding="utf-8")

    def persist_owner(self):
        self.write(self.manager.journal, self.operation)
        self.write(self.paths.state / "deployment/host.json", self.host)
        self.write(self.paths.state / "deployment/driver.json", {"driverDigest": "a" * 64,
            "runtimeOperationId": "current", "root": str(self.root / ".jenkins/runtime/drivers" / ("a" * 64))})

    def execution(self, *, historical=False, status="passed"):
        runtime, checksum, broker = ("old", "b" * 64, 10) if historical else ("current", "a" * 64, 100)
        launcher = identity(13 if historical else 103, 130 if historical else 1030, self.python)
        execution = row("execution", "execution", {"operationId": runtime, "generation": "1", "driverDigest": checksum,
             "driver": {"digest": checksum, "generation": "1", "runtimeOperationId": runtime}, "status": status})
        launch = row("execution_launch", "execution", {"executionId": "execution", "operationId": runtime,
             "generation": "1", "brokerGeneration": runtime, "brokerPid": broker, "driverDigest": checksum,
             "launcherIdentity": launcher, "guardNativeJobId": "guard", "status": "exited"})
        host = row("execution_host", "execution", {"executionId": "execution", "operationId": runtime,
             "generation": "1", "hostPid": launcher["pid"], "hostBirthToken": launcher["creationTime"], "status": "terminal"})
        guard = row("native_job", "guard", {"executionId": "execution", "owner": "execution",
             "role": "execution_host_guard", "pid": launcher["pid"], "birthToken": launcher["creationTime"],
             "executable": launcher["executable"], "status": "terminal", "completeProof": proof(launcher)})
        return [execution, launch, host, guard]

    def history(self, *, legacy=False):
        old = identity(10, 100, self.python)
        controller, agent = identity(11, 110, self.java), identity(12, 120, self.java)
        operation = {**old, "schemaVersion": 2, "component": "lifecycle-host", "operationId": "old",
                     "generation": "old", "hostPid": 10, "homePath": self.owner["homePath"], "state": "running"}
        host = {"operationId": "old", "generation": "old", "hostPid": 10, "status": "running",
                "controllerIdentity": controller, "identity": controller, "agentIdentity": agent, "agentLaunchAttempted": True}
        if legacy:
            record = {**operation, "state": "stopped", "nativeTerminationProof": proof(controller), "agentTerminationProof": proof(agent)}
            path = self.paths.state / "migration-baseline/guarded-runtime-before-stop.json"
        else:
            record = {"operation": operation, "host": host, "bootRecoveryEvidence": {
                "kind": "windows-boot-boundary", "operationId": "old", "homePath": self.owner["homePath"],
                "bootFileTime": "500", "identities": [old, controller, agent], "hostRecordDigest": digest(host),
                "outcome": "interrupted", "validationAcceptanceAllowed": False}}
            path = self.paths.state / "deployment/history" / (digest(record) + ".json")
        self.write(path, record)
        return path, record

    def cancelled_prelaunch(self, *, historical=False):
        runtime, checksum = ("old", "b" * 64) if historical else ("current", "a" * 64)
        key = {"source": "c" * 64, "preparation": "d" * 64, "recipe": "e" * 64, "driver": checksum}
        payload = {"operationId": runtime, "generation": "1", "driverDigest": checksum,
                   "driver": {"digest": checksum, "generation": "1", "runtimeOperationId": runtime},
                   "status": "cancelled", "cancelRequested": True, "cancelledBeforeLaunch": True,
                   "refs": 0, "consumers": [], "sourceDigest": key["source"], "sealedInputRef": key["source"],
                   "preparationKey": key["preparation"], "recipeRef": key["recipe"], "recipeDigest": key["recipe"],
                   "coverageDigest": "f" * 64, "executionKey": key, "executionKeyDigest": digest(key)}
        return [row("execution", "prelaunch", payload),
                row("execution_consumer", "prelaunch:consumer", {"executionId": "prelaunch", "consumerId": "consumer", "status": "released"}),
                row("resource_queue", "prelaunch:prelaunch", {"owner": "prelaunch", "status": "cancelled"})]

    @staticmethod
    def prelaunch_submission(payload, *, delivery="submitted"):
        return row("jenkins_submission", "repository:session:request:execution", {
            "sourceRef": payload["sourceDigest"], "sourceDigest": payload["sourceDigest"],
            "coverageRef": payload["coverageDigest"], "coverageDigest": payload["coverageDigest"],
            "parameters": {"executionId": "prelaunch", "recipeRef": payload["recipeRef"], "generation": payload["generation"]},
            "delivery": delivery})

    @staticmethod
    def departed(value):
        return {"pid": value["pid"], "expectedCreationTime": value["creationTime"], "status": "absent"}

    def test_actual_executor_export_proves_zero_without_busy_executors(self):
        self.assertEqual(0, drain.executor_observation(self.computers)["busyExecutors"])
        value = copy.deepcopy(self.computers)
        value["computer"][0]["oneOffExecutors"] = [{"idle": False, "currentExecutable": {"url": "http://localhost/job/1"}}]
        self.assertEqual(1, drain.executor_observation(value)["busyExecutors"])

    def test_unknown_and_conflicting_executor_exports_block(self):
        bad = [{}, {"computer": []}]
        for change in ({"executors": None}, {"oneOffExecutors": None}, {"numExecutors": -1},
                       {"numExecutors": True}, {"numExecutors": 2}, {"executors": [{}]},
                       {"executors": [{"idle": False}]}, {"executors": [{"idle": 1}]},
                       {"executors": [{"idle": True, "currentExecutable": {"url": "job/1"}}]}):
            value = copy.deepcopy(self.computers)
            value["computer"][2].update(change)
            bad.append(value)
        for value in bad:
            with self.subTest(value=value), self.assertRaises(JenkinsError):
                drain.executor_observation(value)

    def test_owner_reobserves_all_native_components_and_endpoint(self):
        self.persist_owner()
        with patch.object(drain, "verify_driver") as sealed, patch.object(drain, "observe_live_identity") as native, \
             patch.object(drain, "observe_endpoint_owner") as endpoint:
            self.assertEqual(self.owner, drain.owner_observation(self.manager))
        self.assertEqual(3, native.call_count)
        endpoint.assert_called_once_with("127.0.0.1", 18080, 101)
        sealed.assert_called_once_with(self.root / ".jenkins/runtime/drivers" / ("a" * 64), expected_digest="a" * 64)

    def test_owner_mismatch_and_unverified_native_identity_block(self):
        self.persist_owner()
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_live_identity"), patch.object(drain, "observe_endpoint_owner"):
            with self.assertRaises(JenkinsError):
                drain.owner_observation(self.manager, expected={**self.owner, "driverDigest": "b" * 64})
        self.host["agentIdentity"] = {**self.host["agentIdentity"], "commandDigest": ""}
        self.write(self.paths.state / "deployment/host.json", self.host)
        with patch.object(drain, "observe_live_identity") as native, self.assertRaises(JenkinsError):
            drain.owner_observation(self.manager)
        native.assert_not_called()

    def test_current_terminal_requires_bound_guard_and_complete_phase_proofs(self):
        rows = self.execution()
        self.assertEqual([], drain.classify_executions(self.manager, self.owner, rows)["activeExecutions"])
        for field, value in (("complete", False), ("stdoutEof", False), ("activeProcesses", False),
                             ("activeProcesses", 1), ("identity", {}), ("processExitCode", None)):
            changed = copy.deepcopy(rows)
            changed[-1]["payload"]["completeProof"][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(JenkinsError):
                drain.classify_executions(self.manager, self.owner, changed)
        changed = copy.deepcopy(rows)
        changed[1]["payload"]["brokerPid"] = 99
        with self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, changed)

    def test_current_active_execution_is_not_filtered_by_idle_jenkins(self):
        self.assertEqual(["execution"], drain.classify_executions(self.manager, self.owner, self.execution(status="running"))["activeExecutions"])

    def test_pre_agent_current_owner_preserves_historical_holds_and_identity_fences(self):
        path, record = self.history(legacy=True)
        rows = self.execution(historical=True, status="blocked")
        holds = [row(domain, domain, {"owner": "execution", "status": "active"})
                 for domain in ("resource_reservation", "pool_writer_hold")]
        current = {**self.owner, "agent": None}
        before, archive_before = copy.deepcopy(rows + holds), path.read_bytes()
        with patch.object(drain, "verify_driver"), \
             patch.object(drain, "observe_departed_identity", side_effect=self.departed):
            result = drain.classify_executions(self.manager, current, rows + holds)
        self.assertEqual([], result["activeExecutions"])
        self.assertEqual(holds, result["protectedHolds"])
        self.assertEqual(before, rows + holds)
        self.assertEqual(archive_before, path.read_bytes())
        self.assertEqual(1, len(result["historicalExecutions"]))
        self.assertFalse(result["historicalExecutions"][0]["historicalNativeAcceptance"])
        self.assertFalse(result["historicalExecutions"][0]["historicalHoldsReleaseAllowed"])
        historical = {"supervisor": record, "controller": record["nativeTerminationProof"]["identity"],
                      "agent": record["agentTerminationProof"]["identity"]}
        for role, identity_value in (*historical.items(), ("agent", rows[1]["payload"]["launcherIdentity"])):
            with self.subTest(role=role, pid=identity_value["pid"]), patch.object(drain, "verify_driver"), \
                 patch.object(drain, "observe_departed_identity", side_effect=self.departed), self.assertRaises(JenkinsError):
                drain.classify_executions(self.manager, {**current, role: identity_value}, rows + holds)

    def test_historical_boot_binding_preserves_unknown_native_jobs_and_holds(self):
        path, _ = self.history()
        rows = self.execution(historical=True, status="cancel_requested")
        rows[-1]["payload"].update(status="running", completeProof=None)
        holds = [row(domain, domain, {"owner": "execution", "status": "active"})
                 for domain in ("resource_reservation", "pool_writer_hold")]
        before = copy.deepcopy(rows + holds)
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed) as native:
            result = drain.classify_executions(self.manager, self.owner, rows + holds)
        self.assertEqual(before, rows + holds)
        self.assertEqual(holds, result["protectedHolds"])
        native.assert_called_once_with(rows[1]["payload"]["launcherIdentity"])
        self.boot.assert_called_once_with()
        historical = result["historicalExecutions"][0]
        self.assertEqual(str(path), historical["path"])
        self.assertFalse(historical["historicalNativeAcceptance"])
        self.assertFalse(historical["historicalHoldsReleaseAllowed"])

    def test_real_later_boot_does_not_probe_reused_inaccessible_lifecycle_pids(self):
        self.history()
        rows = self.execution(historical=True, status="cancel_requested")
        holds = [row(domain, domain, {"owner": "execution", "status": "active"})
                 for domain in ("resource_reservation", "pool_writer_hold")]
        before = copy.deepcopy(rows + holds)
        def observe(value):
            if value["pid"] != 13:
                raise JenkinsError("deployment_identity_inconclusive", "AccessDenied")
            return self.departed(value)
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=observe) as native:
            result = drain.classify_executions(self.manager, self.owner, rows + holds)
        native.assert_called_once_with(rows[1]["payload"]["launcherIdentity"])
        self.assertEqual(before, rows + holds)
        self.assertEqual(holds, result["protectedHolds"])
        self.assertFalse(result["historicalExecutions"][0]["historicalNativeAcceptance"])
        self.assertFalse(result["historicalExecutions"][0]["historicalHoldsReleaseAllowed"])

    def test_real_boot_must_bind_archive_and_precede_current_supervisor(self):
        self.history()
        rows = self.execution(historical=True, status="blocked")
        for boot in (100, 499, 1000, 1001):
            self.boot.return_value = boot
            with self.subTest(boot=boot), patch.object(drain, "verify_driver"), \
                 patch.object(drain, "observe_departed_identity") as native, self.assertRaises(JenkinsError):
                drain.classify_executions(self.manager, self.owner, rows)
            native.assert_not_called()
        self.boot.side_effect = JenkinsError("boot_unavailable", "Actual Windows boot unavailable")
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity") as native, self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, rows)
        native.assert_not_called()

    def test_live_boot_proof_cannot_change_identity_scope_or_acceptance(self):
        _, record = self.history()
        expected = {**record["bootRecoveryEvidence"], "bootFileTime": "600", "observedAt": 1.0}
        rows = self.execution(historical=True, status="blocked")
        for field, value in (("operationId", "other"), ("homePath", "other"), ("hostRecordDigest", "0" * 64),
                             ("identities", []), ("outcome", "stopped"), ("validationAcceptanceAllowed", True),
                             ("bootFileTime", "0600"), ("unexpected", False)):
            changed = {**expected, field: value}
            with self.subTest(field=field), patch.object(drain, "verify_driver"), \
                 patch.object(drain, "previous_boot_deployment_evidence", return_value=changed), \
                 patch.object(drain, "observe_departed_identity") as native, self.assertRaises(JenkinsError):
                drain.classify_executions(self.manager, self.owner, rows)
            native.assert_not_called()

    def test_later_boot_does_not_replace_independent_launcher_departure(self):
        self.history()
        rows = self.execution(historical=True, status="blocked")
        with patch.object(drain, "verify_driver"), \
             patch.object(drain, "observe_departed_identity", side_effect=JenkinsError("deployment_owner_live", "Launcher alive")), \
             self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, rows)

    def test_stopped_owner_receipt_requires_both_native_proofs_and_departures(self):
        path, record = self.history(legacy=True)
        rows = self.execution(historical=True, status="blocked")
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed):
            self.assertEqual(1, len(drain.classify_executions(self.manager, self.owner, rows)["historicalExecutions"]))
        record["agentTerminationProof"]["stderrEof"] = False
        self.write(path, record)
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed), self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, rows)

    def test_pid_departure_alone_cannot_replace_persisted_historical_provenance(self):
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed) as native, self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, self.execution(historical=True, status="blocked"))
        native.assert_not_called()

    def test_historical_archive_content_and_broker_mismatch_block(self):
        path, record = self.history()
        record["operation"]["hostPid"] = 77
        self.write(path, record)
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed), self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, self.execution(historical=True, status="blocked"))

    def test_preexisting_archive_name_still_requires_complete_owner_evidence(self):
        original, record = self.history()
        path = original.with_name("c" * 64 + ".json")
        original.rename(path)
        rows = self.execution(historical=True, status="blocked")
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed):
            result = drain.classify_executions(self.manager, self.owner, rows)
        self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), result["historicalExecutions"][0]["sha256"])
        record["bootRecoveryEvidence"]["hostRecordDigest"] = "0" * 64
        self.write(path, record)
        with patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed), self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, rows)

    def test_orphaned_native_or_launch_ownership_is_not_ignored(self):
        for value in (row("native_job", "orphan", {"status": "running"}), row("execution_launch", "orphan", {}),
                      row("resource_reservation", "orphan", {"owner": "unknown", "status": "active"})):
            with self.subTest(value=value), self.assertRaises(JenkinsError):
                drain.classify_executions(self.manager, self.owner, [value])

    def test_released_prelaunch_cancel_classifies_current_and_historical_without_native_proof(self):
        for historical in (False, True):
            rows = self.cancelled_prelaunch(historical=historical)
            rows.append(self.prelaunch_submission(rows[0]["payload"]))
            before = copy.deepcopy(rows)
            with self.subTest(historical=historical), patch.object(drain, "verify_driver") as driver, \
                 patch.object(drain, "observe_departed_identity") as native:
                result = drain.classify_executions(self.manager, self.owner, rows)
            self.assertEqual(before, rows)
            self.assertEqual([], result["activeExecutions"])
            self.assertEqual([], result["historicalExecutions"])
            self.assertEqual([], result["protectedHolds"])
            self.assertEqual("prelaunch", result["closedBeforeLaunchExecutions"][0]["executionId"])
            self.assertNotIn("nativeTerminationProof", result["closedBeforeLaunchExecutions"][0])
            native.assert_not_called()
            if historical:
                driver.assert_called_once_with(self.root / ".jenkins/runtime/drivers" / ("b" * 64), expected_digest="b" * 64)
            else:
                driver.assert_not_called()

    def test_prelaunch_cancel_requires_exact_typed_identity_and_released_references(self):
        changes = [("cancelledBeforeLaunch", 1), ("cancelRequested", False), ("status", "failed"), ("refs", False),
                   ("refs", 1), ("consumers", ["consumer"]), ("generation", "0"), ("generation", 1),
                   ("operationId", "other"), ("driver", {}), ("sourceDigest", "z" * 64),
                   ("coverageDigest", ""), ("sealedInputRef", "0" * 64), ("recipeDigest", "0" * 64),
                   ("preparationKey", "0" * 64), ("executionKey", {}), ("executionKeyDigest", "0" * 64),
                   ("phaseRecords", [{}]), ("stageReceipts", {"build": "receipt"})]
        for historical in (False, True):
            for field, value in changes:
                rows = self.cancelled_prelaunch(historical=historical)
                rows[0]["payload"][field] = value
                with self.subTest(historical=historical, field=field, value=value), self.assertRaises(JenkinsError):
                    drain.classify_executions(self.manager, self.owner, rows)
        rows = self.cancelled_prelaunch()
        payload = rows[0]["payload"]
        payload["driverDigest"] = payload["driver"]["digest"] = payload["executionKey"]["driver"] = "b" * 64
        payload["executionKeyDigest"] = digest(payload["executionKey"])
        with self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, rows)
        with patch.object(drain, "verify_driver", side_effect=JenkinsError("driver_corrupt", "Sealed driver changed")), self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, self.cancelled_prelaunch(historical=True))

    def test_existing_failed_before_launch_behavior_is_preserved(self):
        execution = row("execution", "failed", {"operationId": "current", "status": "failed", "failedBeforeLaunch": True})
        self.assertEqual([], drain.classify_executions(self.manager, self.owner, [execution])["activeExecutions"])
        hold = row("resource_reservation", "hold", {"owner": "failed", "status": "active"})
        self.assertEqual(["failed"], drain.classify_executions(self.manager, self.owner, [execution, hold])["activeExecutions"])

    def test_prelaunch_cancel_rejects_launch_consumer_hold_queue_and_unknown_delivery(self):
        extra = [row("execution_launch", "prelaunch", {"status": "requested"}),
                 row("execution_host", "prelaunch", {"status": "terminal"}),
                 row("native_job", "native", {"owner": "prelaunch", "status": "terminal"}),
                 row("execution_consumer", "prelaunch:other", {"executionId": "prelaunch", "consumerId": "other", "status": "active"}),
                 row("execution_consumer", "prelaunch:unknown", {"executionId": "other", "consumerId": "unknown", "status": "released"}),
                 row("resource_reservation", "reservation", {"owner": "prelaunch", "status": "active"}),
                 row("pool_writer_hold", "writer", {"owner": "prelaunch", "status": "active"}),
                 row("resource_queue", "prelaunch:waiting", {"owner": "prelaunch", "status": "waiting"})]
        for delivery in ("unknown", "dispatching", None):
            extra.append(self.prelaunch_submission(self.cancelled_prelaunch()[0]["payload"], delivery=delivery))
        for value in extra:
            for historical in (False, True):
                rows = self.cancelled_prelaunch(historical=historical) + [copy.deepcopy(value)]
                with self.subTest(domain=value["domain"], historical=historical), patch.object(drain, "verify_driver"), self.assertRaises(JenkinsError):
                    drain.classify_executions(self.manager, self.owner, rows)

    def test_prelaunch_cancel_rejects_submission_source_runtime_and_driver_mismatch(self):
        for field, value in (("sourceDigest", "0" * 64), ("coverageDigest", "0" * 64),
                             ("runtimeOperationId", "other"), ("stageImplementationDigest", "0" * 64)):
            rows = self.cancelled_prelaunch()
            submission = self.prelaunch_submission(rows[0]["payload"])
            submission["payload"][field] = value
            with self.subTest(field=field), self.assertRaises(JenkinsError):
                drain.classify_executions(self.manager, self.owner, rows + [submission])
        rows = self.cancelled_prelaunch()
        submission = self.prelaunch_submission(rows[0]["payload"])
        submission["payload"]["parameters"]["RUNTIME_OPERATION_ID"] = "other"
        with self.assertRaises(JenkinsError):
            drain.classify_executions(self.manager, self.owner, rows + [submission])

    def test_inventory_is_read_only_and_requires_existing_supported_schema(self):
        self.paths.state.mkdir(parents=True)
        path = self.paths.state / "coordination.sqlite3"
        with closing(sqlite3.connect(path)) as connection:
            connection.execute("CREATE TABLE metadata (key TEXT,value TEXT)")
            connection.execute("INSERT INTO metadata VALUES ('schemaVersion','1')")
            connection.execute("CREATE TABLE records (domain TEXT,key TEXT,payload TEXT,version INTEGER)")
            connection.execute("INSERT INTO records VALUES ('resource_reservation','hold',?,1)", (json.dumps({"owner": "unknown", "status": "active"}),))
            for domain, payload in (("execution_consumer", {"executionId": "execution", "status": "released"}),
                                    ("resource_queue", {"owner": "execution", "status": "cancelled"}),
                                    ("jenkins_submission", {"parameters": {"executionId": "execution"}, "delivery": "unknown"})):
                connection.execute("INSERT INTO records VALUES (?, ?, ?, 1)", (domain, domain, json.dumps(payload)))
            connection.commit()
        before = hashlib.sha256(path.read_bytes()).hexdigest()
        inventory = drain.execution_inventory(self.paths.state)
        self.assertEqual({"resource_reservation", "execution_consumer", "resource_queue", "jenkins_submission"},
                         {r["domain"] for r in inventory})
        self.assertEqual("hold", next(r["key"] for r in inventory if r["domain"] == "resource_reservation"))
        self.assertEqual(before, hashlib.sha256(path.read_bytes()).hexdigest())
        with self.assertRaises(JenkinsError):
            drain.execution_inventory(self.paths.state / "missing")
        self.assertFalse((self.paths.state / "missing/coordination.sqlite3").exists())

    def simulate_api(self, *, quiet=False, items=None, building=False, computers=None):
        state = {"quiet": quiet, "calls": []}
        def request(endpoint, *, method="GET"):
            state["calls"].append((method, endpoint))
            if method == "POST":
                if endpoint == "quietDown": state["quiet"] = True
                elif endpoint == "cancelQuietDown": state["quiet"] = False
                else: self.fail("Unexpected lifecycle mutation")
                return 302, b""
            if endpoint == drain.COMPUTER_QUERY: value = computers if computers is not None else self.computers
            elif endpoint.startswith("queue/"): value = {"items": items or []}
            elif endpoint == "api/json?tree=quietingDown": value = {"quietingDown": state["quiet"]}
            else: value = {"quietingDown": state["quiet"], "jobs": [{"name": "job", "builds": [{"building": building}]}]}
            return 200, json.dumps(value).encode()
        return state, request

    def test_drained_owner_preserves_holds_and_leaves_quiet_for_owned_stop(self):
        state, request = self.simulate_api()
        self.history()
        hold = row("resource_reservation", "hold", {"owner": "execution", "status": "active"})
        inventory = self.execution(historical=True, status="blocked") + [hold]
        with patch.object(drain, "owner_observation", return_value=self.owner), patch.object(drain, "execution_inventory", return_value=inventory), \
             patch.object(drain, "verify_driver"), patch.object(drain, "observe_departed_identity", side_effect=self.departed), \
             patch.object(self.manager, "_request", side_effect=request):
            result = self.manager.drain(timeout=0)
        self.assertEqual("drained", result["status"])
        self.assertEqual([hold], result["proof"]["protectedHolds"])
        self.assertTrue(state["quiet"])
        self.assertEqual([("POST", "quietDown")], [c for c in state["calls"] if c[0] == "POST"])

    def test_current_execution_or_queue_or_build_blocks_and_restores_same_owner(self):
        for rows, queue, building in ((self.execution(status="running"), [], False), ([], [{"id": 91}], False), ([], [], True)):
            state, request = self.simulate_api(items=queue, building=building)
            with self.subTest(queue=queue, building=building), patch.object(drain, "owner_observation", return_value=self.owner), \
                 patch.object(drain, "execution_inventory", return_value=rows), patch.object(self.manager, "_request", side_effect=request):
                with self.assertRaises(JenkinsError) as caught:
                    self.manager.drain(timeout=0)
            self.assertEqual("drain_timeout", caught.exception.code)
            self.assertFalse(state["quiet"])
            self.assertEqual([("POST", "quietDown"), ("POST", "cancelQuietDown")], [c for c in state["calls"] if c[0] == "POST"])

    def test_closed_prelaunch_cancel_still_requires_empty_actual_jenkins_queue(self):
        for items in ([], [{"id": 91}]):
            state, request = self.simulate_api(items=items)
            rows = self.cancelled_prelaunch()
            with self.subTest(items=items), patch.object(drain, "owner_observation", return_value=self.owner), \
                 patch.object(drain, "execution_inventory", return_value=rows), patch.object(self.manager, "_request", side_effect=request):
                if items:
                    with self.assertRaises(JenkinsError):
                        self.manager.drain(timeout=0)
                    self.assertFalse(state["quiet"])
                else:
                    result = self.manager.drain(timeout=0)
                    self.assertEqual("drained", result["status"])
                    self.assertFalse(result["proof"]["historicalNativeAcceptance"])
                    self.assertFalse(result["proof"]["historicalHoldsReleased"])
                    self.assertNotIn("nativeTerminationProof", result["proof"])

    def test_missing_executor_inventory_does_not_default_to_zero(self):
        state, request = self.simulate_api(computers={"computer": [{"displayName": "node", "busyExecutors": 0}]})
        with patch.object(drain, "owner_observation", return_value=self.owner), patch.object(drain, "execution_inventory", return_value=[]), \
             patch.object(self.manager, "_request", side_effect=request), self.assertRaises(JenkinsError):
            self.manager.drain(timeout=0)
        self.assertFalse(state["quiet"])

    def test_already_quiet_failed_drain_keeps_previous_quiet_state(self):
        state, request = self.simulate_api(quiet=True, items=[{"id": 91}])
        with patch.object(drain, "owner_observation", return_value=self.owner), patch.object(drain, "execution_inventory", return_value=[]), \
             patch.object(self.manager, "_request", side_effect=request), self.assertRaises(JenkinsError):
            self.manager.drain(timeout=0)
        self.assertTrue(state["quiet"])
        self.assertNotIn(("POST", "cancelQuietDown"), state["calls"])

    def test_startup_resumes_only_verified_owner_and_rechecks_acceptance(self):
        state, request = self.simulate_api(quiet=True)
        with patch.object(drain, "owner_observation", return_value=self.owner) as observed, patch.object(self.manager, "_request", side_effect=request):
            result = drain.ensure_accepting_requests(self.manager, "current", expected_owner=self.owner)
        self.assertTrue(result["acceptingRequests"])
        self.assertEqual(3, observed.call_count)
        self.assertEqual(self.owner, observed.call_args_list[0].kwargs["expected"])
        self.assertEqual([("POST", "cancelQuietDown")], [c for c in state["calls"] if c[0] == "POST"])

    def test_startup_unknown_quiet_or_changed_owner_cannot_be_resumed(self):
        for mode in ("quiet-unknown", "owner-changed", "wrong-operation", "wrong-supervisor"):
            state, request = self.simulate_api(quiet=True)
            if mode == "quiet-unknown":
                request = lambda *args, **kwargs: (200, b"{}")
            observed = JenkinsError("drain_owner_changed", "changed") if mode == "owner-changed" else self.owner
            with self.subTest(mode=mode), patch.object(drain, "owner_observation", side_effect=observed if isinstance(observed, Exception) else None,
                 return_value=self.owner), patch.object(self.manager, "_request", side_effect=request) as api, self.assertRaises(JenkinsError):
                drain.ensure_accepting_requests(self.manager, "wrong" if mode == "wrong-operation" else "current", expected_owner=self.owner,
                    expected_supervisor={**self.owner["supervisor"], "pid": 99} if mode == "wrong-supervisor" else None)
            self.assertFalse(any(c.kwargs.get("method") == "POST" for c in api.call_args_list))

    def test_failed_drain_does_not_unquiet_replacement_owner(self):
        state, request = self.simulate_api(items=[{"id": 91}])
        def observe(manager, *, expected=None, allow_starting=False):
            if allow_starting:
                self.assertEqual(self.owner, expected)
                raise JenkinsError("drain_owner_changed", "replacement")
            return self.owner
        with patch.object(drain, "owner_observation", side_effect=observe), patch.object(drain, "execution_inventory", return_value=[]), \
             patch.object(self.manager, "_request", side_effect=request), self.assertRaises(JenkinsError):
            self.manager.drain(timeout=0)
        self.assertTrue(state["quiet"])
        self.assertNotIn(("POST", "cancelQuietDown"), state["calls"])


if __name__ == "__main__":
    unittest.main()

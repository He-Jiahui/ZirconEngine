"""Strict upgrade-holder contracts; no real sandbox or service is launched."""
from __future__ import annotations

import importlib.util
import json
import copy
from pathlib import Path
import unittest
from unittest import mock

from tools.jenkins.contracts import JenkinsError
from tools.tests import test_jenkins_execution_upgrade_selection as fixture_module

REPO = Path(__file__).absolute().parents[2]
ENTRY = REPO / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"
spec = importlib.util.spec_from_file_location("jenkins_upgrade_holder_tested", ENTRY)
holder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(holder)


class UpgradeHolderTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixture_module.UpgradeSelectionTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.repo = self.fixture.repo
        self.selection = self.fixture.selection
        self.manager = self.fixture.manager
        self.path, self.checksum = self.fixture.plan_file()
        self.plan = holder.selector.read_json(self.path)["plan"]
        self.release = {"reviewedUpgradePath": str(self.path), "reviewedUpgradeSha256": self.checksum}
        self.events = []
        self.manager.health.return_value = {"ready": True, "controller": True, "agent": True, "plugins": True}
        self.manager.register_jobs.return_value = {"registered": ["controlled-job"]}
        self.manager._stop_locked.side_effect = lambda: (self.fixture.terminal(), dict(self.fixture.operation))[1]
        self.manager._start_locked.side_effect = self.start_new
        self.fixture.stack.enter_context(mock.patch.object(holder, "observe_live_identity", side_effect=lambda value: dict(value)))
        self.fixture.stack.enter_context(mock.patch.object(holder, "observe_endpoint_owner", return_value={"pid": 202, "ownerCount": 1}))

    def start_new(self, _lock):
        self.fixture.operation.update(operationId="new-operation", generation="new-operation", pid=201, hostPid=201,
                                      creationTime="12401", commandDigest="4" * 64, state="running")
        self.fixture.host.update(operationId="new-operation", generation="new-operation", hostPid=201, status="running")
        self.fixture.host["controllerIdentity"].update(pid=202, creationTime="12402", commandDigest="5" * 64)
        self.fixture.host["agentIdentity"].update(pid=203, creationTime="12403", commandDigest="6" * 64)
        self.fixture.host_path.write_text(json.dumps(self.fixture.host), encoding="utf-8")
        selected = holder.selector.read_json(self.fixture.selector)
        selected["runtimeOperationId"] = "new-operation"
        self.fixture.selector.write_text(json.dumps(selected), encoding="utf-8")
        return dict(self.fixture.operation)

    def run_upgrade(self):
        return holder.perform_upgrade(self.manager, self.selection, self.release, self.plan,
                                      lambda phase, evidence: self.events.append((phase, evidence)))

    def release_file(self):
        baseline = self.repo / ".jenkins/state/migration-baseline/jenkins_desktop_holder.py"
        wrapper = self.repo / "tools/dev/start-jenkins-sandbox.ps1"
        for path in (baseline, wrapper):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"controlled operator fixture\n")
        operators = {str(path): holder.selector.sha(path) for path in (ENTRY, self.selection.helper, baseline, wrapper)}
        preparation = self.fixture.state / "migration-baseline/prepared.json"
        preparation.write_text(json.dumps({"schemaVersion": 1, "state": "prepared-not-selected", "runtimeAcceptance": False,
            "driver": self.plan["candidateDriver"], "regression": self.plan["regression"], "previousSelectedDriver": self.plan["oldDriver"],
            "previousSelectorSha256": self.plan["oldSelectorSha256"], "protectedDeploymentOperationId": "old-operation",
            "protectedHolds": self.plan["protectedHolds"], "boundInputs": {str(self.fixture.spec_path): holder.selector.sha(self.fixture.spec_path)}}), encoding="utf-8")
        release = {**self.release, "schemaVersion": 1, "kind": "desktop-execution-upgrade-release", "repository": str(self.repo),
            "runtimeAcceptance": False, "nativeExecutionHoldsReleaseAllowed": False,
            "configSha256": holder.selector.sha(self.repo / ".codex/config.toml"), "operatorFiles": operators,
            "candidateDriverRoot": self.plan["candidateDriver"]["root"], "candidateDriverDigest": self.plan["candidateDriver"]["driverDigest"],
            "regressionPath": self.plan["regression"]["path"], "regressionSha256": self.plan["regression"]["sha256"],
            "specSha256": holder.selector.sha(self.fixture.spec_path), "expectedPreviousOwner": self.plan["owner"],
            "preparationPath": str(preparation), "preparationSha256": holder.selector.sha(preparation)}
        path = self.fixture.state / "migration-baseline/upgrade-release.json"
        path.write_text(json.dumps(release), encoding="utf-8")
        return path, holder.selector.sha(path), release["configSha256"]

    def test_exact_owner_stop_select_and_default_start_binds_new_owner(self):
        previous = dict(self.fixture.operation)
        result = self.run_upgrade()
        self.assertEqual("new-operation", result["ownedGeneration"]["operationId"])
        self.assertFalse(result["runtimeAcceptance"])
        self.assertTrue(result["previousHolderUntouched"])
        self.assertTrue(result["historicalHoldsUnchanged"])
        self.manager._stop_locked.assert_called_once()
        self.manager._start_locked.assert_called_once_with(self.fixture.state / "deployment-start.lock")
        self.manager.register_jobs.assert_called_once()
        self.assertEqual(["owned-stop-attempted", "old-generation-terminal", "candidate-selected-start-pending",
                          "candidate-start-attempted", "new-generation-owned-acceptance-pending"], [phase for phase, _ in self.events])
        self.assertFalse(holder.baseline.same_owner(holder.baseline.owner_ref(previous), self.manager._read()))
        with self.assertRaises(JenkinsError):
            holder.baseline.stop_exact_owner(self.manager, holder.baseline.owner_ref(previous))
        self.assertEqual(1, self.manager._stop_locked.call_count)
        selection_journal = holder.selector.read_json(self.selection.journal)
        startup = holder.selector.read_json(self.fixture.state / "deployment/execution-upgrade-start.json")
        self.assertEqual("runtime-started-acceptance-pending", selection_journal["status"])
        self.assertEqual("running-acceptance-pending", startup["status"])
        self.assertEqual("new-operation", startup["runtimeOperationId"])
        self.assertFalse(startup["runtimeAcceptance"])
        self.assertEqual(self.fixture.activation_bytes, self.fixture.activation.read_bytes())

    def test_changed_owner_is_not_stopped(self):
        self.fixture.operation["creationTime"] = "999999"
        with self.assertRaises(JenkinsError):
            self.run_upgrade()
        self.manager._stop_locked.assert_not_called()
        self.manager._start_locked.assert_not_called()
        self.fixture.assert_unmodified()

    def test_refused_drain_does_not_select_or_start(self):
        self.manager._stop_locked.side_effect = JenkinsError("drain_timeout", "mocked refused drain")
        with self.assertRaises(JenkinsError):
            self.run_upgrade()
        self.manager._start_locked.assert_not_called()
        self.fixture.assert_unmodified()
        self.assertFalse(self.selection.journal.exists())

    def test_failed_native_terminal_proof_never_selects(self):
        def stop():
            self.fixture.terminal()
            self.fixture.host["nativeTerminationProof"]["activeProcesses"] = 1
            self.fixture.host_path.write_text(json.dumps(self.fixture.host), encoding="utf-8")
            return dict(self.fixture.operation)
        self.manager._stop_locked.side_effect = stop
        with self.assertRaises(JenkinsError):
            self.run_upgrade()
        self.manager._start_locked.assert_not_called()
        self.fixture.assert_unmodified()

    def test_already_terminal_exact_owner_uses_proof_without_reissuing_stop(self):
        self.fixture.terminal()
        result = self.run_upgrade()
        self.manager._stop_locked.assert_not_called()
        self.assertEqual("new-operation", result["ownedGeneration"]["operationId"])

    def test_foreign_new_generation_is_not_adopted(self):
        self.manager._start_locked.side_effect = lambda _lock: {**self.fixture.operation, "operationId": "foreign", "generation": "foreign"}
        with self.assertRaises(JenkinsError) as error:
            self.run_upgrade()
        self.assertEqual("desktop_upgrade_new_owner", error.exception.code)
        self.assertFalse(any("ownedGeneration" in evidence for _, evidence in self.events))
        self.manager.register_jobs.assert_not_called()

    def test_late_registration_failure_preserves_exact_returned_new_owner(self):
        self.manager.register_jobs.side_effect = JenkinsError("job_registration_failed", "mocked registration failure")
        with self.assertRaises(JenkinsError):
            self.run_upgrade()
        owned = [evidence["ownedGeneration"] for _, evidence in self.events if "ownedGeneration" in evidence]
        self.assertEqual(1, len(owned))
        self.assertEqual("new-operation", owned[0]["operationId"])
        self.assertTrue(holder.baseline.same_owner(owned[0], self.manager._read()))
        self.fixture.assert_guarded()
        self.assertEqual("runtime-start-unknown", holder.selector.read_json(self.selection.journal)["status"])
        self.assertEqual("unknown", holder.selector.read_json(self.fixture.state / "deployment/execution-upgrade-start.json")["status"])

    def test_unknown_native_start_blocks_loaded_lifecycle_and_cannot_be_reset(self):
        self.manager._start_locked.side_effect = OSError("mocked uncertain native start")
        with self.assertRaises(OSError):
            self.run_upgrade()
        self.fixture.assert_guarded()
        before = self.fixture.selector.read_bytes()
        with self.assertRaises(JenkinsError):
            self.run_upgrade()
        self.assertEqual(before, self.fixture.selector.read_bytes())
        self.assertEqual(1, self.manager._start_locked.call_count)

    def test_reviewed_release_binds_all_operator_and_preparation_bytes(self):
        path, checksum, config = self.release_file()
        release, plan = holder.release_inputs(self.repo, self.selection, path, checksum, config)
        self.assertEqual(self.plan, plan)
        self.assertEqual(self.plan["candidateDriver"]["driverDigest"], release["candidateDriverDigest"])
        wrapper = self.repo / "tools/dev/start-jenkins-sandbox.ps1"
        wrapper.write_bytes(b"foreign operator bytes\n")
        with self.assertRaises(JenkinsError) as error:
            holder.release_inputs(self.repo, self.selection, path, checksum, config)
        self.assertEqual("desktop_upgrade_operator_changed", error.exception.code)
        self.manager._stop_locked.assert_not_called()

    def test_changed_release_bytes_fail_before_lifecycle(self):
        path, checksum, config = self.release_file()
        path.write_text(path.read_text(encoding="utf-8") + " ", encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            holder.release_inputs(self.repo, self.selection, path, checksum, config)
        self.assertEqual("desktop_upgrade_release_changed", error.exception.code)
        self.manager._stop_locked.assert_not_called()

    def test_restriction_preflight_failure_prevents_manager_construction(self):
        with mock.patch.object(holder, "REPO", self.repo), mock.patch.object(holder, "ENTRY", self.repo / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"), \
             mock.patch.object(holder.baseline, "restriction_preflight", side_effect=JenkinsError("desktop_holder_restriction_missing", "mocked denial probe failure")), \
             mock.patch.object(holder, "UpgradeDeploymentManager") as manager, mock.patch.object(holder.signal, "signal"), mock.patch("builtins.print"):
            result = holder.main(["--config-sha256", "a" * 64, "--release-file", str(self.path), "--release-sha256", self.checksum, "--no-tray"])
        self.assertEqual(2, result)
        manager.assert_not_called()
        self.fixture.assert_unmodified()

    def test_check_only_preflight_records_actual_desktop_and_never_runs_lifecycle(self):
        fixture_entry = self.repo / ".jenkins/state/migration-baseline/jenkins_upgrade_desktop_holder.py"
        diagnostics = {"status": "observed", "windowStation": "WinSta0", "desktop": "CodexSandboxDesktop-test",
                       "uiEnforcementVerified": False, "desktopChanged": False}
        with mock.patch.object(holder, "REPO", self.repo), mock.patch.object(holder, "ENTRY", fixture_entry), \
             mock.patch.object(holder.baseline, "restriction_preflight", return_value={"profileExecutionVerified": False}), \
             mock.patch.object(holder, "desktop_observation", return_value=diagnostics), \
             mock.patch.object(holder, "load_spec", return_value=self.manager.spec), mock.patch.object(holder, "resolve_paths"), \
             mock.patch.object(holder, "UpgradeDeploymentManager", return_value=self.manager), \
             mock.patch.object(holder.selector, "UpgradeSelection", return_value=self.selection), \
             mock.patch.object(holder, "release_inputs", return_value=(self.release, self.plan)), \
             mock.patch.object(holder.signal, "signal"), mock.patch("builtins.print"):
            code = holder.main(["--config-sha256", "a" * 64, "--release-file", str(self.path), "--release-sha256", self.checksum,
                                "--no-tray", "--check-only"])
        self.assertEqual(0, code)
        self.manager._stop_locked.assert_not_called()
        self.manager._start_locked.assert_not_called()
        self.fixture.assert_unmodified()
        receipt = next((self.repo / ".jenkins/state/desktop-upgrade-holder").glob("*.json"))
        state = holder.selector.read_json(receipt)
        self.assertEqual(diagnostics, state["desktopObservation"])
        self.assertFalse(state["holderRetained"])
        self.assertFalse(state["serviceLifecycleExecuted"])

    def test_actual_desktop_diagnostic_does_not_claim_enforcement(self):
        with mock.patch.object(holder.os, "name", "posix"):
            observation = holder.desktop_observation()
        self.assertEqual("unavailable", observation["status"])
        self.assertFalse(observation["uiEnforcementVerified"])

    def test_official_wrapper_upgrade_mode_is_explicit_and_never_changes_profile(self):
        source = (REPO / "tools/dev/start-jenkins-sandbox.ps1").read_text(encoding="utf-8")
        self.assertIn("[string]$UpgradeReleaseFile", source)
        self.assertIn("jenkins_upgrade_desktop_holder.py", source)
        self.assertIn("'sandbox', '-P', 'zircon-strict', '--include-managed-config', '-C'", source)
        self.assertIn("'--release-file', $taskUpgradeRelease, '--release-sha256', $taskUpgradeDigest", source)
        self.assertNotIn("Shell.Application", source)
        self.assertNotIn("CREATE_BREAKAWAY", source)


class HistoricalDrainTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixture_module.UpgradeSelectionTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.manager = self.fixture.manager
        self.selection = self.fixture.selection
        self.path, self.checksum = self.fixture.plan_file()
        self.plan = holder.selector.read_json(self.path)["plan"]
        self.database = self.fixture.database
        self.fixture.stack.enter_context(mock.patch.object(holder, "observe_live_identity", side_effect=lambda value: dict(value)))
        self.fixture.stack.enter_context(mock.patch.object(holder, "observe_endpoint_owner", return_value={"pid": 102, "ownerCount": 1}))
        self.departures = self.fixture.stack.enter_context(mock.patch.object(holder, "observe_departed_identity",
            side_effect=lambda value: {"pid": value["pid"], "status": "absent", "expectedCreationTime": value["creationTime"]}))
        self.history = self.fixture.state / "deployment/history/retired.json"
        self.history.parent.mkdir(exist_ok=True)
        operation = {**self.fixture.operation, "operationId": "retired", "generation": "retired", "hostPid": 301,
                     "pid": 301, "creationTime": "10001"}
        historical_host = {**copy.deepcopy(self.fixture.host), "operationId": "retired", "generation": "retired", "hostPid": 301}
        historical_host["controllerIdentity"].update(pid=302, creationTime="10002")
        historical_host["agentIdentity"].update(pid=303, creationTime="10003")
        identities = [operation, historical_host["controllerIdentity"], historical_host["agentIdentity"]]
        self.history.write_text(json.dumps({"operation": operation, "host": historical_host, "bootRecoveryEvidence": {
            "kind": "windows-boot-boundary", "operationId": "retired", "homePath": str(self.fixture.home),
            "hostRecordDigest": holder.digest(historical_host), "bootFileTime": "11000", "outcome": "interrupted",
            "validationAcceptanceAllowed": False,
            "identities": [{key: value.get(key) for key in ("pid", "creationTime", "executable", "commandDigest")} for value in identities]}}), encoding="utf-8")
        self.install_execution("historical", "retired", 301, status="cancel_requested", terminal=False)
        self.install_execution("current", "old-operation", 101, status="cancelled", terminal=True)
        self.manager._request.side_effect = self.api

    def install_execution(self, key, operation, broker, *, status, terminal):
        checksum = "a" * 64
        launcher = {"pid": 501 if terminal else 401, "creationTime": "12001" if terminal else "10004",
                    "executable": str(self.fixture.python), "commandDigest": "b" * 64}
        self.database.put("execution", key, {"operationId": operation, "generation": "1", "driverDigest": checksum,
            "driver": {"runtimeOperationId": operation, "generation": "1", "digest": checksum}, "status": status})
        self.database.put("execution_launch", key, {"executionId": key, "operationId": operation, "generation": "1",
            "driverDigest": checksum, "brokerGeneration": operation, "brokerPid": broker, "launcherIdentity": launcher,
            "status": "exited" if terminal else "started", "guardNativeJobId": key + "-guard"})
        self.database.put("execution_host", key, {"executionId": key, "operationId": operation, "generation": "1",
            "hostPid": launcher["pid"], "hostBirthToken": launcher["creationTime"], "status": "terminal" if terminal else "running"})
        for role in ("guard", "phase"):
            identity = dict(launcher)
            if role == "phase":
                identity.update(pid=identity["pid"] + 1, executable=str(self.fixture.java))
            proof = {"complete": True, "childrenGone": True, "activeProcesses": 0, "stdoutEof": True,
                     "stderrEof": True, "processExitCode": 0, "identity": identity}
            payload = {"executionId": key, "owner": key, "role": "execution_host_guard" if role == "guard" else "phase",
                "pid": identity["pid"], "birthToken": identity["creationTime"], "executable": identity["executable"],
                "status": "terminal" if terminal else "running", "completeProof": proof if terminal else None,
                "guardNativeJobId": key + "-guard"}
            self.database.put("native_job", key + "-" + role, payload)

    def api(self, endpoint, method="GET"):
        if endpoint == "quietDown":
            self.assertEqual("POST", method)
            return 302, b""
        self.assertEqual("GET", method)
        if endpoint.startswith("computer/"):
            value = {"computer": [{"_class": "hudson.slaves.SlaveComputer", "displayName": "agent",
                                   "executors": [{"_class": "hudson.model.Executor", "idle": True}], "oneOffExecutors": []}]}
        elif endpoint.startswith("queue/"):
            value = {"items": []}
        else:
            value = {"quietingDown": True, "jobs": [{"name": "preserved", "builds": [{"building": False}]}]}
        return 200, json.dumps(value).encode()

    def review(self):
        return holder.review_historical_drain(self.manager, self.selection, self.plan, [self.history])

    def update(self, domain, key, **changes):
        row = self.database.get(domain, key)
        self.database.put(domain, key, {**row["payload"], **changes}, expected_version=row["version"])

    def test_exact_historical_unknown_native_work_is_preserved_while_current_jobs_are_terminal(self):
        before = holder.execution_inventory(self.fixture.state)
        holds = holder.protected_holds(self.fixture.state)
        reviewed = self.review()
        result = holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
        self.assertEqual("drained", result["status"])
        self.assertTrue(result["proof"]["currentNativeTerminationVerified"])
        self.assertFalse(result["proof"]["historicalHoldsReleased"])
        self.assertEqual(before, holder.execution_inventory(self.fixture.state))
        self.assertEqual(holds, holder.protected_holds(self.fixture.state))
        self.assertEqual("cancel_requested", self.database.get("execution", "historical")["payload"]["status"])
        self.manager._stop_locked.assert_not_called()
        self.manager._start_locked.assert_not_called()

    def test_live_retired_owner_cannot_be_filtered(self):
        self.departures.side_effect = JenkinsError("deployment_owner_live", "controlled live historical owner")
        with self.assertRaises(JenkinsError):
            self.review()
        self.manager._request.assert_not_called()

    def stopped_receipt(self):
        archived = holder.selector.read_json(self.history)
        operation = archived["operation"]
        proofs = []
        for identity in (archived["host"]["controllerIdentity"], archived["host"]["agentIdentity"]):
            proofs.append({"complete": True, "childrenGone": True, "activeProcesses": 0, "stdoutEof": True,
                           "stderrEof": True, "processExitCode": 1, "identity": identity})
        path = self.fixture.state / "migration-baseline/retired-stopped-owner.json"
        path.write_text(json.dumps({**operation, "schemaVersion": 2, "state": "stopped",
            "nativeTerminationProof": proofs[0], "agentTerminationProof": proofs[1]}), encoding="utf-8")
        return path

    def test_existing_stopped_owner_receipt_proves_blocked_history_without_native_acceptance(self):
        path = self.stopped_receipt()
        self.update("execution", "historical", status="blocked")
        before = holder.execution_inventory(self.fixture.state)
        holds = holder.protected_holds(self.fixture.state)
        history_files = sorted(self.history.parent.glob("*.json"))
        reviewed = holder.review_historical_drain(self.manager, self.selection, self.plan, [path])
        self.assertFalse(reviewed["historicalNativeAcceptance"])
        self.assertFalse(reviewed["historicalHoldsReleaseAllowed"])
        self.assertEqual("blocked", reviewed["historicalExecutions"][0]["records"]["execution"]["payload"]["status"])
        self.assertEqual("drained", holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)["status"])
        self.assertEqual(before, holder.execution_inventory(self.fixture.state))
        self.assertEqual(holds, holder.protected_holds(self.fixture.state))
        self.assertEqual(history_files, sorted(self.history.parent.glob("*.json")))

    def test_stopped_receipt_requires_exact_owner_and_complete_both_component_proofs(self):
        path = self.stopped_receipt()
        original = holder.selector.read_json(path)
        bad_values = [{**original, "state": "running"}, {**original, "homePath": "foreign"},
            {**original, "hostPid": 101}, {**original, "operationId": "foreign"}, {**original, "agentTerminationProof": None},
            {**original, "nativeTerminationProof": {**original["nativeTerminationProof"], "stderrEof": False}},
            {**original, "agentTerminationProof": {**original["agentTerminationProof"], "activeProcesses": True}}]
        for value in bad_values:
            with self.subTest(value=value):
                path.write_text(json.dumps(value), encoding="utf-8")
                with self.assertRaises(JenkinsError):
                    holder.review_historical_drain(self.manager, self.selection, self.plan, [path])
        self.manager._request.assert_not_called()

    def test_stopped_receipt_requires_actual_departure_of_all_three_components(self):
        path = self.stopped_receipt()
        for pid in (301, 302, 303):
            with self.subTest(pid=pid):
                def departure(value):
                    if value["pid"] == pid:
                        raise JenkinsError("deployment_owner_live", "controlled live historical component")
                    return {"pid": value["pid"], "expectedCreationTime": value["creationTime"], "status": "absent"}
                self.departures.side_effect = departure
                with self.assertRaises(JenkinsError):
                    holder.review_historical_drain(self.manager, self.selection, self.plan, [path])
        self.manager._request.assert_not_called()

    def test_existing_stopped_receipt_bytes_remain_hash_bound(self):
        path = self.stopped_receipt()
        reviewed = holder.review_historical_drain(self.manager, self.selection, self.plan, [path])
        path.write_text(path.read_text(encoding="utf-8") + " ", encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            holder.verify_historical_drain(self.manager, self.selection, self.plan, reviewed)
        self.assertEqual("desktop_upgrade_drain_changed", error.exception.code)
        self.manager._request.assert_not_called()

    def test_historical_launch_driver_or_broker_binding_mismatch_blocks(self):
        for changes in ({"operationId": "foreign"}, {"driverDigest": "c" * 64}, {"brokerPid": 101}):
            with self.subTest(changes=changes):
                row = self.database.get("execution_launch", "historical")["payload"]
                self.update("execution_launch", "historical", **changes)
                with self.assertRaises(JenkinsError):
                    self.review()
                self.update("execution_launch", "historical", **row)
        self.manager._request.assert_not_called()

    def test_exact_historical_row_or_archive_changes_invalidate_review(self):
        reviewed = self.review()
        self.update("execution", "historical", refs=1)
        with self.assertRaises(JenkinsError) as error:
            holder.verify_historical_drain(self.manager, self.selection, self.plan, reviewed)
        self.assertEqual("desktop_upgrade_drain_changed", error.exception.code)
        reviewed = self.review()
        self.history.write_text(self.history.read_text(encoding="utf-8") + " ", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            holder.verify_historical_drain(self.manager, self.selection, self.plan, reviewed)

    def test_new_unknown_active_execution_is_not_automatically_excluded(self):
        reviewed = self.review()
        self.database.put("execution", "unknown", {"status": "pending"})
        with self.assertRaises(JenkinsError):
            holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
        self.manager._request.assert_not_called()

    def test_current_active_execution_blocks_before_quietdown(self):
        reviewed = self.review()
        self.update("execution", "current", status="running")
        with self.assertRaises(JenkinsError) as error:
            holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
        self.assertEqual("desktop_upgrade_current_execution_active", error.exception.code)
        self.manager._request.assert_not_called()

    def test_current_native_guard_or_phase_incomplete_proof_blocks(self):
        for key in ("current-guard", "current-phase"):
            with self.subTest(key=key):
                original = self.database.get("native_job", key)["payload"]
                self.update("native_job", key, completeProof={**original["completeProof"], "stderrEof": False})
                with self.assertRaises(JenkinsError):
                    self.review()
                self.update("native_job", key, **original)
        self.manager._request.assert_not_called()

    def test_orphan_current_native_launch_blocks(self):
        self.database.put("execution_launch", "orphan", {"operationId": "old-operation", "status": "requested"})
        with self.assertRaises(JenkinsError) as error:
            self.review()
        self.assertEqual("desktop_upgrade_execution_ambiguous", error.exception.code)

    def test_current_owner_or_protected_hold_change_blocks(self):
        reviewed = self.review()
        self.fixture.operation["creationTime"] = "foreign"
        with self.assertRaises(JenkinsError):
            holder.verify_historical_drain(self.manager, self.selection, self.plan, reviewed)
        self.fixture.operation["creationTime"] = "12301"
        self.database.put("pool_writer_hold", "legacy", {"status": "released", "owner": "legacy"})
        with self.assertRaises(JenkinsError):
            holder.verify_historical_drain(self.manager, self.selection, self.plan, reviewed)

    def test_incomplete_api_response_never_claims_idle(self):
        reviewed = self.review()
        for bad_endpoint in ("api/", "computer/", "queue/"):
            with self.subTest(endpoint=bad_endpoint):
                self.manager._request.side_effect = lambda endpoint, method="GET": ((403, b"{}") if endpoint.startswith(bad_endpoint) else self.api(endpoint, method))
                with self.assertRaises(JenkinsError) as error:
                    holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
                self.assertEqual("desktop_upgrade_drain_unproven", error.exception.code)
        self.manager._stop_locked.assert_not_called()

    def test_missing_executor_export_never_defaults_to_idle(self):
        reviewed = self.review()
        self.manager._request.side_effect = lambda endpoint, method="GET": ((200, b'{"computer":[{}]}') if endpoint.startswith("computer/") else self.api(endpoint, method))
        with self.assertRaises(JenkinsError) as error:
            holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
        self.assertEqual("desktop_upgrade_drain_unproven", error.exception.code)

    def test_observed_three_computer_api_shape_without_busy_export_drains(self):
        reviewed = self.review()
        computers = {"computer": [
            {"_class": "hudson.model.Hudson$MasterComputer", "displayName": "Built-In Node", "executors": [], "oneOffExecutors": []},
            {"_class": "hudson.slaves.SlaveComputer", "displayName": "old-agent", "executors": [{"idle": True}], "oneOffExecutors": []},
            {"_class": "hudson.slaves.SlaveComputer", "displayName": "current-agent", "executors": [{"idle": True}], "oneOffExecutors": []}]}
        self.manager._request.side_effect = lambda endpoint, method="GET": ((200, json.dumps(computers).encode()) if endpoint.startswith("computer/") else self.api(endpoint, method))
        result = holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
        self.assertEqual(0, result["proof"]["busyExecutors"])
        self.assertEqual(3, len(result["proof"]["executorObservation"]["computer"]))
        computer_call = next(call for call in self.manager._request.call_args_list if call.args[0].startswith("computer/"))
        self.assertIn("executors[idle,currentExecutable[url]]", computer_call.args[0])
        self.assertIn("oneOffExecutors[idle,currentExecutable[url]]", computer_call.args[0])

    def test_regular_and_oneoff_nonidle_executors_block_without_cancellation(self):
        reviewed = self.review()
        for group in ("executors", "oneOffExecutors"):
            with self.subTest(group=group):
                computer = {"displayName": "agent", "executors": [], "oneOffExecutors": []}
                computer[group] = [{"idle": False, "currentExecutable": {"url": "http://localhost/job/unrelated/12/"}}]
                self.manager._request.reset_mock()
                self.manager._request.side_effect = lambda endpoint, method="GET": ((200, json.dumps({"computer": [computer]}).encode()) if endpoint.startswith("computer/") else self.api(endpoint, method))
                with self.assertRaises(JenkinsError) as error:
                    holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
                self.assertEqual("drain_timeout", error.exception.code)
                self.assertEqual([mock.call("quietDown", method="POST")], [call for call in self.manager._request.call_args_list if call.kwargs.get("method") == "POST"])

    def test_malformed_executor_inventory_and_configuration_fail_closed(self):
        base = {"displayName": "agent", "executors": [{"idle": True}], "oneOffExecutors": []}
        for computer in ({"displayName": "agent", "busyExecutors": 0}, {**base, "oneOffExecutors": None},
                {**base, "executors": [{}]}, {**base, "executors": [{"idle": 1}]}, {**base, "executors": [None]},
                {**base, "numExecutors": -1}, {**base, "numExecutors": True}, {**base, "numExecutors": 2}):
            with self.subTest(computer=computer):
                with self.assertRaises(JenkinsError) as error:
                    holder.executor_observation({"computer": [computer]})
                self.assertEqual("desktop_upgrade_drain_unproven", error.exception.code)
        self.assertEqual(0, holder.executor_observation({"computer": [{**base, "numExecutors": 1}]})["busyExecutors"])

    def test_unknown_busy_executable_or_conflicting_idle_export_fails_closed(self):
        for executor in ({"idle": False}, {"idle": False, "currentExecutable": None},
                {"idle": False, "currentExecutable": {}}, {"idle": False, "currentExecutable": {"url": ""}},
                {"idle": True, "currentExecutable": {"url": "http://localhost/job/unrelated/12/"}}):
            with self.subTest(executor=executor):
                with self.assertRaises(JenkinsError) as error:
                    holder.executor_observation({"computer": [{"displayName": "agent", "executors": [executor], "oneOffExecutors": []}]})
                self.assertEqual("desktop_upgrade_drain_unproven", error.exception.code)

    def test_busy_build_queue_or_nonquiet_state_blocks_without_cancel(self):
        reviewed = self.review()
        for prefix, value in (("api/", {"quietingDown": True, "jobs": [{"name": "unrelated", "builds": [{"building": True}]}]}),
                ("api/", {"quietingDown": False, "jobs": []}), ("computer/", {"computer": [{"displayName": "agent",
                    "executors": [{"idle": False, "currentExecutable": {"url": "http://localhost/job/unrelated/12/"}}], "oneOffExecutors": []}]}),
                ("queue/", {"items": [{"id": 12}]})):
            with self.subTest(value=value):
                self.manager._request.reset_mock()
                self.manager._request.side_effect = lambda endpoint, method="GET": ((200, json.dumps(value).encode()) if endpoint.startswith(prefix) else self.api(endpoint, method))
                with self.assertRaises(JenkinsError) as error:
                    holder.classified_drain(self.manager, self.selection, self.plan, reviewed, timeout=0)
                self.assertEqual("drain_timeout", error.exception.code)
                self.assertEqual([mock.call("quietDown", method="POST")], [call for call in self.manager._request.call_args_list if call.kwargs.get("method") == "POST"])

    def test_public_historical_review_is_read_only_without_release_or_retained_holder(self):
        with mock.patch.object(holder, "REPO", self.fixture.repo), mock.patch.object(holder, "load_spec", return_value=self.manager.spec), \
             mock.patch.object(holder, "resolve_paths"), mock.patch.object(holder, "UpgradeDeploymentManager", return_value=self.manager), \
             mock.patch.object(holder.selector, "UpgradeSelection", return_value=self.selection), mock.patch.object(holder.baseline, "save") as save, \
             mock.patch.object(holder, "process_lock") as lock, mock.patch("builtins.print") as output:
            code = holder.main(["--review-historical-drain", str(self.path), "--review-sha256", self.checksum,
                                "--historical-lifecycle-history", str(self.history)])
        self.assertEqual(0, code)
        result = json.loads(output.call_args.args[0])
        self.assertEqual("reviewed-historical-execution-drain", result["status"])
        self.assertFalse(result["serviceLifecycleExecuted"])
        self.assertFalse(result["historicalDrainReview"]["historicalNativeAcceptance"])
        save.assert_not_called()
        lock.assert_not_called()
        self.manager._request.assert_not_called()
        self.manager._stop_locked.assert_not_called()
        self.manager._start_locked.assert_not_called()

    def test_dedicated_manager_consumes_classification_once_even_on_unknown_error(self):
        manager = holder.UpgradeDeploymentManager(self.manager.spec, self.manager.paths, self.fixture.java, self.fixture.war)
        reviewed = self.review()
        progress = mock.Mock()
        context = (self.selection, self.plan, reviewed, progress)
        manager.upgrade_drain_context = context
        with mock.patch.object(holder, "classified_drain", return_value={"status": "drained"}) as drain, \
             mock.patch.object(holder.DeploymentManager, "drain", return_value={"status": "stock"}) as stock:
            self.assertEqual("drained", manager.drain()["status"])
            self.assertIsNone(manager.upgrade_drain_context)
            self.assertEqual("stock", manager.drain()["status"])
        drain.assert_called_once_with(manager, self.selection, self.plan, reviewed, 30.0)
        stock.assert_called_once_with(30.0)
        manager.upgrade_drain_context = context
        with mock.patch.object(holder, "classified_drain", side_effect=OSError("controlled unknown drain")):
            with self.assertRaises(OSError):
                manager.drain()
        self.assertIsNone(manager.upgrade_drain_context)


if __name__ == "__main__":
    unittest.main()

"""Offline upgrade selection contracts; all native observations are mocked."""
from __future__ import annotations

from contextlib import ExitStack
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.driver import seal_driver
from tools.jenkins.deployment.execution_activation import assert_activation_start_allowed, support_sources
from tools.jenkins.deployment.manager import DeploymentManager
from tools.jenkins.processes.identity import ProcessIdentity
from tools.jenkins.state import State

REPO = Path(__file__).absolute().parents[2]
ENTRY = REPO / ".jenkins/state/migration-baseline/select_execution_upgrade.py"
spec = importlib.util.spec_from_file_location("jenkins_execution_upgrade_tested", ENTRY)
upgrade = importlib.util.module_from_spec(spec)
spec.loader.exec_module(upgrade)


def seal_fixture_driver(repo):
    """Retry only a transient Windows rename of this synthetic driver fixture."""
    if repo.parent != REPO / ".jenkins/tmp" or not repo.name.startswith("upgrade-selector-tests-"):
        raise AssertionError("Only the isolated synthetic fixture may use this helper")
    from tools.jenkins.deployment import driver as implementation
    rename = implementation.os.rename
    def bounded_rename(source, destination, *args, **kwargs):
        for attempt in range(6):
            try:
                return rename(source, destination, *args, **kwargs)
            except PermissionError as error:
                if (getattr(error, "winerror", None) != 5 or attempt == 5
                        or Path(source).parent != repo / ".jenkins/runtime/drivers"
                        or Path(destination).parent != Path(source).parent
                        or Path(destination).exists() or not Path(source).exists()
                        or not Path(source).name.startswith(".incoming-")):
                    raise
                time.sleep(.1)
    with mock.patch.object(implementation.os, "rename", side_effect=bounded_rename):
        return seal_driver(repo)


class UpgradeSelectionTests(unittest.TestCase):
    def setUp(self):
        temporary = REPO / ".jenkins/tmp"
        temporary.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="upgrade-selector-tests-", dir=temporary)
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.state = self.repo / ".jenkins/state"
        (self.state / "deployment").mkdir(parents=True)
        (self.state / "migration-baseline").mkdir()
        self.home = self.repo / ".jenkins/jenkins_home"
        (self.home / "init.groovy.d").mkdir(parents=True)
        self.build = self.repo / "empty-build-root-fixture"
        self.build.mkdir()
        self.java = self.repo / ".jenkins/runtime/jdk/bin/java.exe"
        self.python = self.repo / ".jenkins/runtime/python/python.exe"
        self.war = self.repo / ".jenkins/runtime/jenkins.war"
        self.spec_path = self.repo / ".jenkins/deployment-spec.json"
        for path in (self.java, self.python, self.war, self.spec_path,
                     self.repo / ".jenkins/workflow-spec.json", self.repo / ".jenkins/plugin-checksums.json",
                     self.repo / ".codex/config.toml",
                     self.home / "init.groovy.d/90-zircon-repository-build-root.groovy"):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"controlled fixture\n")
        (self.repo / "tools/jenkins").mkdir(parents=True)
        self.source = self.repo / "tools/jenkins/__main__.py"
        self.source.write_text("print('old fixture')\n", encoding="utf-8")
        (self.repo / ".jenkins/pipeline").mkdir()
        (self.repo / ".jenkins/pipeline/steps.groovy").write_text("return this\n", encoding="utf-8")
        self.old = seal_fixture_driver(self.repo)
        self.source.write_text("print('new fixture')\n", encoding="utf-8")
        self.candidate = seal_fixture_driver(self.repo)
        self.old.update(runtimeOperationId="old-operation", state="selected-for-runtime-start", runtimeAcceptance=False)
        self.selector = self.state / "deployment/driver.json"
        self.selector.write_text(json.dumps(self.old, indent=3) + "\n", encoding="utf-8")
        self.old_bytes = self.selector.read_bytes()
        self.activation = self.state / "deployment/execution-activation.json"
        self.activation.write_text(json.dumps({"status": "succeeded", "transitionDigest": "a" * 64,
            "operationId": "old-operation", "candidate": {**self.old, "runtimeAcceptance": False}}, indent=4) + "\n", encoding="utf-8")
        self.activation_bytes = self.activation.read_bytes()
        self.operation = {"component": "lifecycle-host", "operationId": "old-operation", "generation": "old-operation",
            "hostPid": 101, "pid": 101, "creationTime": "12301", "executable": str(self.python), "commandDigest": "1" * 64,
            "homePath": str(self.home), "controlPlaneOnly": False, "state": "running"}
        controller = {"pid": 102, "creationTime": "12302", "executable": str(self.java), "commandDigest": "2" * 64}
        agent = {"pid": 103, "creationTime": "12303", "executable": str(self.java), "commandDigest": "3" * 64}
        self.host = {"operationId": "old-operation", "generation": "old-operation", "hostPid": 101,
                     "status": "running", "controlPlaneOnly": False, "agentLaunchAttempted": True,
                     "controllerIdentity": controller, "agentIdentity": agent}
        self.host_path = self.state / "deployment/host.json"
        self.host_path.write_text(json.dumps(self.host), encoding="utf-8")
        self.manager = mock.Mock()
        self.manager.paths = SimpleNamespace(repo=self.repo, state=self.state, home=self.home, build_root=self.build)
        self.manager.spec = SimpleNamespace(path=self.spec_path, controller={"listenAddress": "127.0.0.1", "httpPort": 1})
        self.manager.java = self.java
        self.manager.war = self.war
        self.manager._read.side_effect = lambda: dict(self.operation)
        self.manager.start.side_effect = AssertionError("Selection may not start runtime")
        self.manager.stop.side_effect = AssertionError("Selection may not stop runtime")
        self.selection = upgrade.UpgradeSelection(self.manager)
        self.database = State(self.state / "coordination.sqlite3")
        self.database.put("resource_reservation", "legacy", {"status": "active", "owner": "legacy"})
        self.database.put("pool_writer_hold", "legacy", {"status": "active", "owner": "legacy"})
        log = self.state / "migration-baseline/regression.log"
        log.write_text("controlled support regression passed\n", encoding="utf-8")
        self.regression = self.state / "migration-baseline/regression.json"
        self.regression.write_text(json.dumps({"scope": "complete-current-Jenkins-support-pytest-discovery",
            "status": "passed", "sourceIdentityStable": True, "testsRun": 1, "failures": 0, "errors": 0,
            "exitCode": 0, "collectionErrors": [], "sourceFiles": support_sources(self.repo),
            "log": str(log), "logSha256": upgrade.sha(log)}), encoding="utf-8")
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(mock.patch.object(upgrade, "observe_live_identity", side_effect=lambda value: dict(value)))
        self.stack.enter_context(mock.patch.object(upgrade, "observe_endpoint_owner", return_value={"pid": 102, "ownerCount": 1}))
        self.departures = self.stack.enter_context(mock.patch.object(upgrade, "observe_departed_identity",
            side_effect=lambda value: {"pid": value["pid"], "expectedCreationTime": value["creationTime"], "status": "absent"}))
        self.home_probe = self.stack.enter_context(mock.patch.object(upgrade, "observe_home_available",
            return_value={"exclusiveReadProbes": [], "staleMarkersDeleted": False}))
        self.port_probe = self.stack.enter_context(mock.patch.object(upgrade, "check_port_available"))

    def review(self):
        return self.selection.review(candidate_root=Path(self.candidate["root"]), regression_path=self.regression)

    def plan_file(self):
        value = self.review()
        path = self.state / "migration-baseline/reviewed-upgrade.json"
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
        return path, upgrade.sha(path)

    def terminal(self):
        self.operation["state"] = "stopped"
        self.host["status"] = "stopped"
        for key, identity in (("nativeTerminationProof", self.host["controllerIdentity"]),
                              ("agentTerminationProof", self.host["agentIdentity"])):
            self.host[key] = {"identity": identity, "complete": True, "childrenGone": True,
                              "activeProcesses": 0, "stdoutEof": True, "stderrEof": True, "processExitCode": 0}
        self.host_path.write_text(json.dumps(self.host), encoding="utf-8")

    def select(self, path, checksum):
        return self.selection.select(review_path=path, review_sha256=checksum)

    def assert_unmodified(self):
        self.assertEqual(self.old_bytes, self.selector.read_bytes())
        self.assertEqual(self.activation_bytes, self.activation.read_bytes())
        self.manager.start.assert_not_called()
        self.manager.stop.assert_not_called()

    def assert_guarded(self):
        guard = upgrade.read_json(self.activation)
        self.assertEqual(upgrade.GUARD_KIND, guard["kind"])
        self.assertIn(guard["status"], {"pending", "unknown"})
        self.assertNotIn("transitionDigest", guard)
        with self.assertRaises(JenkinsError) as error:
            assert_activation_start_allowed(self.manager)
        self.assertEqual("execution_transition_unresolved", error.exception.code)

    def test_review_live_is_read_only_and_selection_refuses_live_owner(self):
        path, checksum = self.plan_file()
        self.assertEqual("reviewed-live-execution-owner", upgrade.read_json(path)["status"])
        self.assertFalse(self.selection.journal.exists())
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_upgrade_owner_live", error.exception.code)
        self.assert_unmodified()

    def test_unknown_native_termination_does_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.host["agentTerminationProof"]["stderrEof"] = False
        self.host_path.write_text(json.dumps(self.host), encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_upgrade_terminal_missing", error.exception.code)
        self.assert_unmodified()

    def test_inconclusive_component_departure_does_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        def departed(value):
            if value["pid"] == 103:
                raise JenkinsError("deployment_identity_inconclusive", "mocked inaccessible identity")
            return {"pid": value["pid"], "expectedCreationTime": value["creationTime"], "status": "absent"}
        self.departures.side_effect = departed
        with self.assertRaises(JenkinsError):
            self.select(path, checksum)
        self.assert_unmodified()

    def test_returned_unknown_component_departure_does_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.departures.side_effect = lambda value: {"pid": value["pid"], "expectedCreationTime": value["creationTime"],
            "status": "inconclusive" if value["pid"] == 103 else "absent"}
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_upgrade_departure_unproven", error.exception.code)
        self.assert_unmodified()

    def test_changed_candidate_bytes_do_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        (Path(self.candidate["root"]) / "tools/jenkins/__main__.py").write_text("print('foreign candidate')\n", encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("driver_integrity_failed", error.exception.code)
        self.assert_unmodified()

    def test_changed_current_support_source_does_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.source.write_text("print('foreign source')\n", encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_transition_source_changed", error.exception.code)
        self.assert_unmodified()

    def test_changed_review_bytes_do_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        path.write_text(path.read_text(encoding="utf-8") + " ", encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_upgrade_review_changed", error.exception.code)
        self.assert_unmodified()

    def test_changed_selector_does_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.selector.write_text(json.dumps({**self.old, "runtimeOperationId": "foreign-operation"}), encoding="utf-8")
        foreign = self.selector.read_bytes()
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_upgrade_selector", error.exception.code)
        self.assertEqual(foreign, self.selector.read_bytes())
        self.assertEqual(self.activation_bytes, self.activation.read_bytes())

    def test_changed_owner_birth_does_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.operation["creationTime"] = "foreign"
        with self.assertRaises(JenkinsError):
            self.select(path, checksum)
        self.assert_unmodified()

    def test_changed_historical_holds_do_not_select(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.database.put("pool_writer_hold", "legacy", {"status": "released", "owner": "legacy"})
        with self.assertRaises(JenkinsError) as error:
            self.select(path, checksum)
        self.assertEqual("execution_upgrade_holds_changed", error.exception.code)
        self.assert_unmodified()

    def test_success_selects_only_after_full_proof_and_restores_exact_history(self):
        path, checksum = self.plan_file()
        self.terminal()
        before = upgrade.protected_holds(self.state)
        result = self.select(path, checksum)
        self.assertEqual("selected-awaiting-runtime-start", result["status"])
        self.assertFalse(result["runtimeAcceptance"])
        selected = upgrade.read_json(self.selector)
        self.assertEqual(self.candidate["driverDigest"], selected["driverDigest"])
        self.assertIsNone(selected["runtimeOperationId"])
        self.assertFalse(selected["runtimeAcceptance"])
        self.assertEqual(self.activation_bytes, self.activation.read_bytes())
        self.assertEqual(before, upgrade.protected_holds(self.state))
        self.assertIsNone(assert_activation_start_allowed(self.manager))
        self.assertEqual("verified-selection-awaiting-runtime-start", self.selection.verify_selected()["status"])
        journal = upgrade.read_json(self.selection.journal)
        intent = json.loads(self.selection._reference(journal["intent"]))
        self.assertEqual(self.old_bytes, self.selection._reference(intent["oldSelector"]))
        self.assertEqual(self.activation_bytes, self.selection._reference(intent["originalActivation"]))
        self.manager.start.assert_not_called()
        self.manager.stop.assert_not_called()

    def test_selector_swap_failure_leaves_recognized_unknown_start_guard(self):
        path, checksum = self.plan_file()
        self.terminal()
        atomic = self.selection._atomic_bytes
        def write(destination, data):
            if destination == self.selector:
                raise OSError("mocked selector exchange failure")
            return atomic(destination, data)
        with mock.patch.object(self.selection, "_atomic_bytes", side_effect=write), self.assertRaises(OSError):
            self.select(path, checksum)
        self.assertEqual(self.old_bytes, self.selector.read_bytes())
        self.assertEqual("unknown", upgrade.read_json(self.selection.journal)["status"])
        self.assert_guarded()
        with self.assertRaises(JenkinsError):
            self.selection.reconcile_selected()
        self.assert_guarded()

    def test_failed_receipt_keeps_applied_selection_guarded_until_exact_reconciliation(self):
        path, checksum = self.plan_file()
        self.terminal()
        immutable = self.selection._immutable
        def archive(name, data):
            if name.startswith("execution-upgrade-selected-"):
                raise OSError("mocked selection receipt failure")
            return immutable(name, data)
        with mock.patch.object(self.selection, "_immutable", side_effect=archive), self.assertRaises(OSError):
            self.select(path, checksum)
        self.assertEqual(self.candidate["driverDigest"], upgrade.read_json(self.selector)["driverDigest"])
        self.assert_guarded()
        self.assertEqual("selected-awaiting-runtime-start", self.selection.reconcile_selected()["status"])
        self.assertEqual(self.activation_bytes, self.activation.read_bytes())
        self.selection.verify_selected()

    def test_failed_guard_restore_keeps_tray_blocked_and_never_retries_selector(self):
        path, checksum = self.plan_file()
        self.terminal()
        atomic = self.selection._atomic_bytes
        def write(destination, data):
            if destination == self.activation and data == self.activation_bytes:
                raise OSError("mocked historical journal restore failure")
            return atomic(destination, data)
        with mock.patch.object(self.selection, "_atomic_bytes", side_effect=write), self.assertRaises(OSError):
            self.select(path, checksum)
        selected_bytes = self.selector.read_bytes()
        self.assert_guarded()
        with mock.patch.object(self.selection, "_atomic_bytes", wraps=atomic) as writing:
            self.selection.reconcile_selected()
        self.assertFalse(any(call.args[0] == self.selector for call in writing.call_args_list))
        self.assertEqual(selected_bytes, self.selector.read_bytes())
        self.assertEqual(self.activation_bytes, self.activation.read_bytes())

    def test_changed_hold_during_unknown_reconciliation_retains_start_guard(self):
        path, checksum = self.plan_file()
        self.terminal()
        with mock.patch.object(self.selection, "_complete_selected", side_effect=OSError("mocked crash after selector")), self.assertRaises(OSError):
            self.select(path, checksum)
        self.database.put("resource_reservation", "legacy", {"status": "released", "owner": "legacy"})
        with self.assertRaises(JenkinsError) as error:
            self.selection.reconcile_selected()
        self.assertEqual("execution_upgrade_holds_changed", error.exception.code)
        self.assert_guarded()

    def test_reconciliation_never_adopts_foreign_selector(self):
        path, checksum = self.plan_file()
        self.terminal()
        with mock.patch.object(self.selection, "_complete_selected", side_effect=OSError("mocked crash after selector")), self.assertRaises(OSError):
            self.select(path, checksum)
        self.selector.write_text(json.dumps({**self.candidate, "runtimeOperationId": "foreign-operation"}), encoding="utf-8")
        foreign = self.selector.read_bytes()
        with self.assertRaises(JenkinsError):
            self.selection.reconcile_selected()
        self.assertEqual(foreign, self.selector.read_bytes())
        self.assert_guarded()

    def test_corrupt_immutable_intent_never_releases_guard(self):
        path, checksum = self.plan_file()
        self.terminal()
        with mock.patch.object(self.selection, "_complete_selected", side_effect=OSError("mocked crash after selector")), self.assertRaises(OSError):
            self.select(path, checksum)
        journal = upgrade.read_json(self.selection.journal)
        Path(journal["intent"]["path"]).write_text("{}", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            self.selection.reconcile_selected()
        self.assert_guarded()

    def test_successful_selected_verification_is_read_only(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.select(path, checksum)
        original = {p: p.read_bytes() for p in (self.selector, self.activation, self.selection.journal)}
        with mock.patch.object(self.selection, "_atomic_bytes", side_effect=AssertionError("read-only verification wrote")):
            self.selection.verify_selected()
        self.assertEqual(original, {p: p.read_bytes() for p in original})

    def test_changed_foreign_guard_is_preserved_on_reconciliation_failure(self):
        path, checksum = self.plan_file()
        self.terminal()
        with mock.patch.object(self.selection, "_complete_selected", side_effect=OSError("mocked crash after selector")), self.assertRaises(OSError):
            self.select(path, checksum)
        self.activation.write_text('{"kind":"foreign-operation","status":"pending"}', encoding="utf-8")
        foreign = self.activation.read_bytes()
        with self.assertRaises(JenkinsError) as error:
            self.selection.reconcile_selected()
        self.assertEqual("execution_upgrade_guard_changed", error.exception.code)
        self.assertEqual(foreign, self.activation.read_bytes())
        with self.assertRaises(JenkinsError):
            assert_activation_start_allowed(self.manager)

    def test_normal_terminal_start_uses_selected_candidate_without_recovery_override(self):
        path, checksum = self.plan_file()
        self.terminal()
        self.select(path, checksum)
        self.selection.verify_selected()
        self.manager.paths.logs = self.repo / ".jenkins/logs"
        manager = DeploymentManager(self.manager.spec, self.manager.paths, self.java, self.war)
        manager._write(dict(self.operation))
        from tools.jenkins.deployment import manager as implementation
        health = {"ready": True, "controller": True, "agent": True, "plugins": True}
        process = mock.Mock(pid=201)
        with ExitStack() as stack:
            stack.enter_context(mock.patch.object(implementation, "check_port_available"))
            spawn = stack.enter_context(mock.patch.object(implementation.subprocess, "Popen", return_value=process))
            stack.enter_context(mock.patch.object(implementation, "_supervisor_creation_flags", return_value=0))
            stack.enter_context(mock.patch.object(implementation, "current_identity",
                return_value=ProcessIdentity(201, "12401", str(self.python), "4" * 64)))
            stack.enter_context(mock.patch.object(implementation, "wait_for_health", return_value=True))
            stack.enter_context(mock.patch.object(manager, "_credentials", return_value=("fixture-user", "fixture-token")))
            stack.enter_context(mock.patch.object(manager, "health", return_value=health))
            stack.enter_context(mock.patch("tools.jenkins.deployment.drain_observations.ensure_accepting_requests"))
            stack.enter_context(mock.patch("tools.jenkins.deployment.startup_observations.observe_departed_identity",
                side_effect=lambda value: {"pid": value["pid"], "expectedCreationTime": value["creationTime"], "status": "absent"}))
            # This path must never invoke incomplete-proof replacement recovery.
            recovery = stack.enter_context(mock.patch("tools.jenkins.deployment.recovery.replacement_deployment_evidence",
                side_effect=AssertionError("Complete native proof must use the normal terminal branch")))
            stack.enter_context(mock.patch.dict(implementation._LIVE_JOBS, {}, clear=True))
            try:
                result = manager._start_locked(self.state / "deployment-start.lock")
                args = spawn.call_args.args[0]
                self.assertEqual(self.candidate["launcher"], args[1])
                self.assertFalse(result["controlPlaneOnly"])
                self.assertEqual("running", result["state"])
                self.assertEqual(self.candidate["driverDigest"], upgrade.read_json(self.selector)["driverDigest"])
                recovery.assert_not_called()
            finally:
                for _process, handle in implementation._LIVE_JOBS.values():
                    handle.close()


if __name__ == "__main__":
    unittest.main()

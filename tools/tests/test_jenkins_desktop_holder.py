"""Focused holder contracts; all service, sandbox and protected probes are mocked."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
from contextlib import ExitStack

REPO = Path(__file__).absolute().parents[2]
ENTRY = REPO / ".jenkins/state/migration-baseline/jenkins_desktop_holder.py"
spec = importlib.util.spec_from_file_location("jenkins_desktop_holder_tested", ENTRY)
holder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(holder)


class DesktopHolderTests(unittest.TestCase):
    def setUp(self):
        temporary = REPO / ".jenkins/tmp"
        temporary.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="holder-tests-", dir=temporary)
        self.root = Path(self.temp.name)
        self.addCleanup(self.temp.cleanup)
        self.runtime = self.root / ".jenkins"
        self.runtime.mkdir()

    def config(self):
        directory = self.root / ".codex"
        directory.mkdir()
        path = directory / "config.toml"
        path.write_text('[windows]\nsandbox = "elevated"\nprivate_desktop = false\n'
                        '[permissions.zircon-strict]\nextends = ":read-only"\n'
                        '[permissions.zircon-strict.filesystem]\n'
                        + repr(str(self.runtime).replace("\\", "/")) + ' = "write"\n', encoding="utf-8")
        return path

    def test_unrestricted_update_open_fails_before_allowed_probe(self):
        path = self.config()
        with mock.patch.object(holder, "process_job_observation") as jobs:
            with self.assertRaises(holder.JenkinsError) as error:
                holder.restriction_preflight(self.root, holder.sha(path))
        self.assertEqual(error.exception.code, "desktop_holder_restriction_missing")
        jobs.assert_not_called()
        self.assertFalse((self.runtime / "state").exists())

    def test_denied_update_and_exclusive_allowed_write_are_required(self):
        path = self.config()
        original_open = Path.open
        def restricted_open(current, mode="r", *args, **kwargs):
            if current == path and mode == "r+b":
                raise PermissionError("simulated restricted token")
            return original_open(current, mode, *args, **kwargs)
        observed = {"holderIdentity": {"pid": 7}, "associatedJob": {"holderMember": True, "memberPids": [7]}}
        with mock.patch.object(Path, "open", restricted_open), mock.patch.object(holder, "process_job_observation", return_value=observed):
            result = holder.restriction_preflight(self.root, holder.sha(path))
        self.assertTrue(result["configuration"]["updateOpenDenied"])
        self.assertTrue(result["allowedProbe"]["exclusiveCreateWrite"])
        self.assertTrue(result["allowedProbe"]["removed"])
        self.assertEqual(result["configuration"]["sha256Before"], result["configuration"]["sha256After"])
        self.assertEqual(result["associatedJob"]["memberPids"], [7])
        self.assertFalse(result["profileExecutionVerified"])

    def test_config_hash_change_prevents_any_update_open(self):
        path = self.config()
        with self.assertRaises(holder.JenkinsError) as error:
            holder.restriction_preflight(self.root, "0" * 64)
        self.assertEqual(error.exception.code, "desktop_holder_config_changed")

    def test_runtime_path_rejects_foreign_output_and_traversal(self):
        for path in (self.root / "foreign.json", self.runtime / ".." / "foreign.json"):
            with self.subTest(path=path), self.assertRaises(holder.JenkinsError):
                holder.runtime_path(self.root, path)

    def test_owner_comparison_requires_every_identity_field(self):
        owner = {field: str(index) for index, field in enumerate(holder.OWNER_FIELDS)}
        self.assertTrue(holder.same_owner(owner, dict(owner)))
        changed = {**owner, "creationTime": "different"}
        self.assertFalse(holder.same_owner(owner, changed))
        self.assertFalse(holder.same_owner({"pid": 1}, {"pid": 1}))

    def test_stop_checks_exact_owner_under_existing_lock(self):
        owner = {field: str(index) for index, field in enumerate(holder.OWNER_FIELDS)}
        manager = mock.Mock()
        manager.paths.state = self.runtime / "state"
        manager._read.return_value = owner
        with mock.patch.object(holder, "process_lock", new_callable=mock.MagicMock) as lock:
            self.assertEqual(holder.stop_exact_owner(manager, owner)["status"], "stopped")
        lock.return_value.__enter__.assert_called_once()
        manager._stop_locked.assert_called_once()

    def test_stop_refuses_changed_generation_without_calling_stop(self):
        owner = {field: str(index) for index, field in enumerate(holder.OWNER_FIELDS)}
        manager = mock.Mock()
        manager.paths.state = self.runtime / "state"
        manager._read.return_value = {**owner, "generation": "foreign"}
        with mock.patch.object(holder, "process_lock", new_callable=mock.MagicMock), self.assertRaises(holder.JenkinsError) as error:
            holder.stop_exact_owner(manager, owner)
        self.assertEqual(error.exception.code, "desktop_holder_owner_changed")
        manager._stop_locked.assert_not_called()

    def release(self):
        candidate = self.runtime / "runtime/drivers" / ("a" * 64)
        candidate.mkdir(parents=True)
        regression = self.runtime / "regression.json"
        regression.write_text('{"tests": "reviewed"}', encoding="utf-8")
        deployment = self.runtime / "deployment-spec.json"
        deployment.write_text('{}', encoding="utf-8")
        release = self.runtime / "release.json"
        release.write_text(json.dumps({"schemaVersion": 1, "candidateDriverRoot": str(candidate),
            "candidateDriverDigest": candidate.name, "regressionPath": str(regression),
            "regressionSha256": holder.sha(regression), "specSha256": holder.sha(deployment)}), encoding="utf-8")
        return release, candidate, regression

    def test_reviewed_release_binds_candidate_regression_and_spec(self):
        release, candidate, regression = self.release()
        self.assertEqual(holder.reviewed_release(self.root, release, holder.sha(release)), (candidate, regression))
        regression.write_text('{}', encoding="utf-8")
        with self.assertRaises(holder.JenkinsError) as error:
            holder.reviewed_release(self.root, release, holder.sha(release))
        self.assertEqual(error.exception.code, "desktop_holder_release_changed")

    def test_probe_only_does_not_construct_lifecycle_manager(self):
        with mock.patch.object(holder, "REPO", self.root), mock.patch.object(holder, "restriction_preflight", return_value={"mocked": True}), \
                mock.patch.object(holder, "DeploymentManager") as manager, mock.patch.object(holder.signal, "signal"), mock.patch("builtins.print"):
            self.assertEqual(holder.main(["--probe-only", "--config-sha256", "a" * 64]), 0)
        manager.assert_not_called()
        receipts = list((self.runtime / "state/desktop-holder").glob("*.json"))
        result = json.loads(receipts[0].read_text(encoding="utf-8"))
        self.assertEqual(result["status"], "restricted-probe-passed")
        self.assertFalse(result["runtimeAcceptance"])

    def lifecycle_fixture(self, stack):
        _, candidate, regression = self.release()
        manager = mock.Mock()
        manager.paths.state = self.runtime / "state"
        baseline = {field: "baseline-" + field for field in holder.OWNER_FIELDS}
        baseline.update(state="running", controlPlaneOnly=True)
        manager._read.return_value = baseline
        fake_spec = mock.Mock()
        fake_spec.path = self.runtime / "deployment-spec.json"
        handlers = {}
        stack.enter_context(mock.patch.object(holder, "REPO", self.root))
        stack.enter_context(mock.patch.object(holder, "restriction_preflight", return_value={"mocked": True}))
        stack.enter_context(mock.patch.object(holder, "verify_driver", return_value={"root": str(candidate), "driverDigest": candidate.name}))
        stack.enter_context(mock.patch.object(holder, "verify_regression"))
        stack.enter_context(mock.patch.object(holder, "load_spec", return_value=fake_spec))
        stack.enter_context(mock.patch.object(holder, "resolve_paths"))
        stack.enter_context(mock.patch.object(holder, "DeploymentManager", return_value=manager))
        stack.enter_context(mock.patch.object(holder, "process_lock", new_callable=mock.MagicMock))
        stack.enter_context(mock.patch.object(holder.signal, "signal", side_effect=lambda code, handler: handlers.update({code: handler})))
        stack.enter_context(mock.patch("builtins.print"))
        arguments = ["--config-sha256", "a" * 64, "--candidate", str(candidate), "--regression", str(regression), "--no-tray"]
        return manager, baseline, handlers, arguments

    def test_stale_running_baseline_calls_manager_once_then_binds_activation(self):
        with ExitStack() as stack:
            manager, baseline, handlers, arguments = self.lifecycle_fixture(stack)
            quiet = {**baseline, "operationId": "quiet-reconciled"}
            active = {**quiet, "operationId": "activated", "controlPlaneOnly": False}
            def start():
                manager._read.return_value = quiet
                return quiet
            def activate(*_args):
                manager._read.return_value = active
                return 0, {"status": "started", "newOwner": holder.owner_ref(active)}
            def health(_manager):
                handlers[holder.signal.SIGINT](holder.signal.SIGINT, None)
                return {"ready": True}
            manager.start.side_effect = start
            activation = stack.enter_context(mock.patch.object(holder, "run_activation", side_effect=activate))
            stack.enter_context(mock.patch.object(holder, "health_summary", side_effect=health))
            stopping = stack.enter_context(mock.patch.object(holder, "stop_exact_owner", return_value={"status": "stopped"}))
            self.assertEqual(holder.main(arguments), 0)
            manager.start.assert_called_once()
            activation.assert_called_once()
            stopping.assert_called_once_with(manager, holder.owner_ref(active))

    def test_failed_activation_does_not_adopt_foreign_current_generation(self):
        with ExitStack() as stack:
            manager, baseline, _handlers, arguments = self.lifecycle_fixture(stack)
            manager.start.return_value = baseline
            def activate(*_args):
                manager._read.return_value = {**baseline, "operationId": "foreign-generation"}
                return 2, {"reasonCode": "reviewed_activation_failed"}
            stack.enter_context(mock.patch.object(holder, "run_activation", side_effect=activate))
            stack.enter_context(mock.patch.object(holder, "health_summary", side_effect=StopIteration("end mocked holding loop")))
            stopping = stack.enter_context(mock.patch.object(holder, "stop_exact_owner"))
            with self.assertRaises(StopIteration):
                holder.main(arguments)
            manager.start.assert_called_once()
            stopping.assert_not_called()
        receipt = next((self.runtime / "state/desktop-holder").glob("*.json"))
        state = json.loads(receipt.read_text(encoding="utf-8"))
        self.assertEqual(state["status"], "holding-failed-start")
        self.assertNotIn("ownedGeneration", state)
        self.assertFalse(state["automaticRestart"])

    def test_wrapper_uses_official_named_profile_and_preserves_codex_home(self):
        source = (REPO / "tools/dev/start-jenkins-sandbox.ps1").read_text(encoding="utf-8")
        self.assertIn("'sandbox', '-P', 'zircon-strict', '--include-managed-config', '-C'", source)
        self.assertIn("EnvironmentVariables['CODEX_HOME'] = $taskOriginalCodexHome", source)
        self.assertIn("usable-candidate.json", source)
        self.assertNotIn("Shell.Application", source)
        self.assertNotIn("CREATE_BREAKAWAY", source)
        self.assertNotIn(".Kill(", source)


if __name__ == "__main__":
    unittest.main()

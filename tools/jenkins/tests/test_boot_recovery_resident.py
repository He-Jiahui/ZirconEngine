"""Bounded contracts for native resident recovery; no Jenkins lifecycle actions."""
import hashlib
import importlib.util
import json
import os
import subprocess
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

_MODULE_PATH = Path(__file__).parents[2] / "dev/jenkins_boot_recovery.py"
_spec = importlib.util.spec_from_file_location("_resident_recovery_contract", _MODULE_PATH)
boot = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(boot)


class ResidentRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="resident-generation-", dir=os.environ["TEMP"])
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.state = self.repo / ".jenkins/state"
        self.report = self.state / "management/reports/resident"
        self.report.mkdir(parents=True)
        self.addCleanup(patch.stopall)
        patch.object(boot, "REPO", self.repo).start()
        patch.object(boot, "STATE", self.state).start()
        self.startup = {"reportRoot": str(self.report)}
        self.startup_ref = {"path": str(self.state / "startup.json"), "sha256": "a" * 64}
        self.boot = "123456789"
        self.release_a = {"path": str(self.state / "release-a.json"), "sha256": "1" * 64}
        self.release_b = {"path": str(self.state / "release-b.json"), "sha256": "2" * 64}
        self.proof_a = {"path": str(self.report / "proof-a.json"), "sha256": "3" * 64}
        self.proof_b = {"path": str(self.report / "proof-b.json"), "sha256": "4" * 64}

    def intent(self, token, release, startup_ref=None):
        root = boot.boot_namespace(self.startup, startup_ref or self.startup_ref, self.boot)
        path = root / ("request-" + token * 32) / "launch-intent.json"
        path.parent.mkdir(parents=True)
        path.write_text(json.dumps({"schemaVersion": 1, "kind": "current-boot-public-start-intent",
                                   "bootFileTime": self.boot, "release": release}), encoding="utf-8")
        return path

    def gate(self, release, proof, write=True):
        return boot.prior_intents(None, self.startup, self.startup_ref, self.boot, release, proof, write=write)

    def verifier(self, manager, ref, *, expected_release):
        pairs = ((self.proof_a, self.release_a), (self.proof_b, self.release_b))
        self.assertIn((ref, expected_release), pairs)
        return {"bootFileTime": self.boot}

    def test_two_distinct_lost_generations_have_distinct_stable_attempt_and_intent_paths(self):
        first = boot.recovery_namespace(self.startup, self.startup_ref, self.boot, {"sha256": "b" * 64})
        same = boot.recovery_namespace(self.startup, self.startup_ref, self.boot, {"sha256": "b" * 64})
        second = boot.recovery_namespace(self.startup, self.startup_ref, self.boot, {"sha256": "c" * 64})
        self.assertEqual(first, same)
        self.assertNotEqual(first, second)
        self.assertEqual("request-" + first[0], first[1].name)
        self.assertNotEqual(first[1] / "launch-intent.json", second[1] / "launch-intent.json")

    def test_unknown_intent_cannot_be_bypassed_by_a_new_descriptor(self):
        path = self.intent("1", self.release_a)
        raw = path.read_bytes()
        with patch.object(boot, "validate_generation_proof") as validate:
            with self.assertRaisesRegex(RuntimeError, "unresolved public"):
                self.gate(self.release_b, self.proof_b)
        validate.assert_not_called()
        self.assertEqual(raw, path.read_bytes())
        self.assertFalse(path.with_name("committed-generation.json").exists())

    def test_same_release_without_fresh_live_proof_is_not_replayed(self):
        path = self.intent("1", self.release_a)
        with self.assertRaisesRegex(RuntimeError, "unresolved public"):
            self.gate(self.release_a, None)
        self.assertFalse(path.with_name("committed-generation.json").exists())

    def test_two_committed_fresh_owners_allow_two_successive_same_boot_recoveries(self):
        first = self.intent("1", self.release_a)
        first_raw = first.read_bytes()
        with patch.object(boot, "validate_generation_proof", side_effect=self.verifier):
            self.gate(self.release_a, self.proof_a)
            second = self.intent("2", self.release_b)
            second_raw = second.read_bytes()
            self.gate(self.release_b, self.proof_b)
        for path, raw, proof in ((first, first_raw, self.proof_a), (second, second_raw, self.proof_b)):
            self.assertEqual(raw, path.read_bytes())
            completed = json.loads(path.with_name("committed-generation.json").read_bytes())
            self.assertEqual(proof, completed["generationProof"])
            self.assertFalse(completed["publicStartReplayed"])
        self.assertEqual(2, len(list(self.report.rglob("launch-intent.json"))))

    def test_completion_cannot_be_rebound_to_mutated_original_intent(self):
        path = self.intent("1", self.release_a)
        with patch.object(boot, "validate_generation_proof", side_effect=self.verifier):
            self.gate(self.release_a, self.proof_a)
            path.write_text(path.read_text() + " ", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "completion changed"):
                self.gate(self.release_b, self.proof_b)

    def test_unknown_intent_in_an_older_startup_version_still_blocks(self):
        old = {**self.startup_ref, "sha256": "f" * 64}
        self.intent("1", self.release_a, old)
        with self.assertRaisesRegex(RuntimeError, "unresolved public"):
            self.gate(self.release_b, self.proof_b)

    def test_dry_run_does_not_mark_intent_completed(self):
        path = self.intent("1", self.release_a)
        with patch.object(boot, "validate_generation_proof", side_effect=self.verifier):
            self.gate(self.release_a, self.proof_a, write=False)
        self.assertFalse(path.with_name("committed-generation.json").exists())

    def test_fresh_proof_from_another_boot_cannot_complete_current_intent(self):
        path = self.intent("1", self.release_a)
        with patch.object(boot, "validate_generation_proof", return_value={"bootFileTime": "999"}):
            with self.assertRaisesRegex(RuntimeError, "current-boot"):
                self.gate(self.release_a, self.proof_a)
        self.assertFalse(path.with_name("committed-generation.json").exists())

    def setup_execute(self, generation):
        descriptor = {"bootFileTime": self.boot, "keeper": {"pid": 23, "creationTime": "456"},
                      "runtimeReceipt": {"path": "receipt", "sha256": "5" * 64}}
        selected = {"runtimeOperationId": "start-123"}
        target = {"driverDigest": "6" * 64}
        docs = {self.state / "formal-lifecycle/recovery-keeper.json": descriptor,
                self.state / "deployment/driver.json": selected,
                self.state / "recovery-lifecycle/command.json": {"status": "committed"}}
        patch.object(boot, "strict_configuration", return_value=True).start()
        patch.object(boot, "startup_document", return_value=(self.startup, self.startup_ref)).start()
        patch.object(boot, "resolved_host", return_value=({}, None)).start()
        patch.object(boot, "read", side_effect=lambda p: docs[Path(p)]).start()
        patch.object(boot, "reference", return_value={"path": "descriptor", "sha256": "7" * 64}).start()
        patch.object(boot, "sealed_runtime", return_value=({}, target)).start()
        patch("tools.jenkins.deployment.spec.load_spec", return_value=None).start()
        patch("tools.jenkins.deployment.paths.resolve_paths", return_value=None).start()
        patch("tools.jenkins.deployment.manager.DeploymentManager", return_value=None).start()
        patch("tools.jenkins.processes.boot.windows_boot_filetime", return_value=int(self.boot)).start()
        if isinstance(generation, Exception):
            patch.object(boot, "generation_proof", side_effect=generation).start()
        else:
            patch.object(boot, "generation_proof", return_value=generation).start()
        return SimpleNamespace(startup_file="startup", startup_sha256="a" * 64,
                               observer_pid=None, observe_only=False, check_only=False)

    def test_existing_owner_returns_no_action_and_no_new_recovery_plan(self):
        args = self.setup_execute(self.proof_a)
        with patch("tools.jenkins.deployment.lost_recovery_runtime.prepare_lost_recovery_runtime") as prepare:
            result = boot.execute(args)
        self.assertEqual("already-running-no-action", result["status"])
        self.assertFalse(result["serviceStartIssued"])
        self.assertTrue(result["strictActualConfigRplusBDenied"])
        prepare.assert_not_called()

    def test_observe_only_never_prepares_recovery_when_current_owner_is_unproven(self):
        args = self.setup_execute(RuntimeError("owner not live"))
        args.observe_only = True
        with patch("tools.jenkins.deployment.lost_recovery_runtime.prepare_lost_recovery_runtime") as prepare:
            with self.assertRaisesRegex(RuntimeError, "observation issues no recovery"):
                boot.execute(args)
        prepare.assert_not_called()

    def test_incomplete_live_generation_marker_is_rejected(self):
        with patch.object(boot, "checked", return_value={
            "schemaVersion": 1, "kind": "strict-committed-live-generation-observation",
            "release": self.release_a, "nativeOwnerObserved": False,
            "strictActualConfigRplusBDenied": True, "archives": {}, "inputs": {}}):
            with self.assertRaisesRegex(RuntimeError, "complete strict live-generation"):
                boot.validate_generation_proof(None, self.proof_a, expected_release=self.release_a)



class RetainedPublicExitPolicyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="resident-exit-policy-", dir=os.environ["TEMP"])
        self.addCleanup(self.temp.cleanup)
        self.fixture = Path(self.temp.name)
        self.dispatcher = _MODULE_PATH.with_name("start-jenkins-on-logon.ps1")
        self.powershell = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        self.logs = Path(os.environ["ZIRCON_RESIDENT_POLICY_REPORT"])
        self.logs.mkdir(parents=True, exist_ok=True)

    def confirmed_facts(self):
        release = {"path": "E:/Git/ZirconEngine/.jenkins/state/deployment/release-test.json",
                   "sha256": "a" * 64}
        return {
            "role": "retained-public", "resident": True,
            "expectedRelease": release,
            "child": {"pid": 3000, "creationTime": "134366000000000000",
                      "retainedExactProcess": True, "nativeExitObserved": True, "exitCode": 1},
            "confirmed": {
                "release": dict(release), "runtimeOperationId": "start-1792125076530147300",
                "runtimeReceipt": {"sha256": "b" * 64},
                "generationProof": {"sha256": "c" * 64},
                "descriptor": {"sha256": "d" * 64},
                "keeper": {"pid": 4000, "creationTime": "134366000010000000"},
                "strictActualConfigRplusBDenied": True, "nativeOwnerObserved": True},
            "currentDescriptorExact": True, "keeperIdentityStatus": "terminated-exact"}

    def run_cases(self, cases):
        case_path = self.fixture / "exit-cases.json"
        case_path.write_text(json.dumps([{"id": name, "facts": facts}
                                        for name, facts, expected in cases]), encoding="utf-8")
        command = [str(self.powershell), "-NoLogo", "-NoProfile", "-NonInteractive",
                   "-ExecutionPolicy", "Bypass", "-File", str(self.dispatcher),
                   "-ExitPolicyCasesFile", str(case_path)]
        prefix = self.logs / self._testMethodName
        try:
            result = subprocess.run(command, capture_output=True, timeout=120, check=False)
        except subprocess.TimeoutExpired as error:
            prefix.with_suffix(".stdout.bin").write_bytes(error.stdout or b"")
            prefix.with_suffix(".stderr.bin").write_bytes(error.stderr or b"")
            raise
        prefix.with_suffix(".stdout.bin").write_bytes(result.stdout)
        prefix.with_suffix(".stderr.bin").write_bytes(result.stderr)
        self.assertEqual(0, result.returncode, result.stderr.decode("utf-8", errors="replace"))
        observed = json.loads(result.stdout.decode("utf-8-sig"))
        self.assertIn(observed["languageMode"], ("ConstrainedLanguage", "FullLanguage"))
        self.assertEqual(len(cases), len(observed["cases"]))
        self.assertEqual({name: expected for name, facts, expected in cases},
                         {row["id"]: row["allowContinue"] for row in observed["cases"]})
        prefix.with_suffix(".json").write_text(json.dumps({
            "kind": "actual-canonical-powershell-exit-policy-test",
            "languageMode": observed["languageMode"], "returncode": result.returncode,
            "source": str(self.dispatcher),
            "sourceSha256": hashlib.sha256(self.dispatcher.read_bytes()).hexdigest(),
            "caseResults": observed["cases"],
            "stdoutSha256": hashlib.sha256(result.stdout).hexdigest(),
            "stderrSha256": hashlib.sha256(result.stderr).hexdigest(),
            "serviceActionsIssued": False}, indent=2), encoding="utf-8")

    def test_confirmed_exact_generation_after_native_child_exit_can_continue(self):
        nonzero = self.confirmed_facts()
        zero = self.confirmed_facts()
        zero["child"]["exitCode"] = 0
        self.run_cases([("confirmed-nonzero", nonzero, True), ("confirmed-zero", zero, True)])

    def test_unconfirmed_and_early_unknown_exit_cannot_continue(self):
        unconfirmed = self.confirmed_facts()
        unconfirmed["confirmed"] = None
        early = self.confirmed_facts()
        early["child"]["creationTime"] = ""
        early["child"]["retainedExactProcess"] = False
        not_ended = self.confirmed_facts()
        not_ended["child"]["nativeExitObserved"] = False
        zero_unknown = self.confirmed_facts()
        zero_unknown["child"]["exitCode"] = 0
        zero_unknown["confirmed"]["nativeOwnerObserved"] = False
        self.run_cases([("unconfirmed", unconfirmed, False), ("early-unknown", early, False),
                        ("not-native-ended", not_ended, False), ("zero-unknown", zero_unknown, False)])

    def test_pid_reuse_live_owner_and_unproven_owner_fail_closed(self):
        cases = []
        for status in ("pid-reused", "alive-exact", "unproven"):
            facts = self.confirmed_facts()
            facts["keeperIdentityStatus"] = status
            cases.append((status, facts, False))
        self.run_cases(cases)

    def test_probe_wrong_release_and_changed_descriptor_cannot_use_public_exit_gate(self):
        probe = self.confirmed_facts()
        probe["role"] = "probe"
        not_resident = self.confirmed_facts()
        not_resident["resident"] = False
        wrong_sha = self.confirmed_facts()
        wrong_sha["expectedRelease"] = {**wrong_sha["expectedRelease"], "sha256": "e" * 64}
        wrong_path = self.confirmed_facts()
        wrong_path["expectedRelease"] = {**wrong_path["expectedRelease"], "path": "E:/other-release.json"}
        changed_descriptor = self.confirmed_facts()
        changed_descriptor["currentDescriptorExact"] = False
        self.run_cases([("probe", probe, False), ("nonresident", not_resident, False),
                        ("wrong-release-sha", wrong_sha, False), ("wrong-release-path", wrong_path, False),
                        ("changed-descriptor", changed_descriptor, False)])

if __name__ == "__main__":
    unittest.main()

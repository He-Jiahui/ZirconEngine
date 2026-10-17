"""Real Git tests for standing Jenkins validation/commit/main integration."""
from contextlib import closing, contextmanager
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import sys
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

spec = importlib.util.spec_from_file_location("validated_delivery_tests", Path(__file__).parents[1] / "delivery.py")
delivery = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = delivery
spec.loader.exec_module(delivery)
git = delivery.commit.git


class DeliveryTests(unittest.TestCase):
    @contextmanager
    def fixture(self):
        # The complete pytest tree is already in the outer retained Windows
        # Job. Fixture Git inherits it; avoid one extra Job per fixture command.
        with tempfile.TemporaryDirectory(prefix="delivery-", dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name, patch.dict(os.environ, {"ZR_COMMIT_NATIVE": "0"}):
            repo = Path(name)
            git(repo, "init", "--quiet", "-b", "main")
            git(repo, "config", "user.name", "Jenkins delivery test")
            git(repo, "config", "user.email", "jenkins-delivery@localhost")
            for file, content in {"owned.py": "value = 1\n", "foreign.txt": "base\n", "dependency.py": "dep = 1\n", ".gitignore": ".jenkins/\n"}.items():
                (repo / file).write_text(content, encoding="utf-8")
            git(repo, "add", ".")
            git(repo, "-c", "commit.gpgSign=false", "commit", "--quiet", "-m", "baseline")
            policy = {"authorization": {"source": "user", "instruction": "Any session may validate/commit/merge via Jenkins"}}
            payload = {"action": "deliver-task", "sessionId": "ordinary-chat", "requestId": "delivery-task",
                       "operationId": "delivery-task", "message": "Implement the task", "paths": ["owned.py"],
                       "dependencies": ["dependency.py"], "validation": [{"kind": "python-static", "paths": ["owned.py"]}]}
            observations = []
            def validate(root, request, names, output, runtime):
                result = {"status": "passed", "head": delivery.text(root, "rev-parse", "HEAD"),
                          "files": delivery.files(root, names), "steps": [{"status": "passed"}]}
                observations.append((root, result))
                return result
            with patch.object(delivery, "OUTPUT", repo / ".jenkins/test-output"):
                yield repo, policy, {str(repo): "write"}, payload, validate, observations

    def execute(self, f, **kwargs):
        repo, policy, grants, payload, validate, _ = f
        return delivery.execute(repo, payload, policy=policy, grants=grants, validate=validate, **kwargs)

    def worktree(self, f):
        repo, _, _, payload, _, _ = f
        root = repo / ".jenkins/worktrees/ordinary-chat-feature"
        git(repo, "worktree", "add", "-b", "codex/jenkins/ordinary-chat/feature", str(root), "HEAD")
        database = repo / ".jenkins/state/coordination.sqlite3"
        database.parent.mkdir(parents=True)
        with closing(sqlite3.connect(database)) as connection:
            connection.execute("CREATE TABLE records(domain TEXT,key TEXT,payload TEXT)")
            connection.execute("INSERT INTO records VALUES(?,?,?)", ("management_worktree", "ordinary-chat-feature",
                json.dumps({"path": str(root), "sessionId": payload["sessionId"], "status": "ready",
                            "branch": "codex/jenkins/ordinary-chat/feature"})))
            connection.commit()
        payload["sourceRoot"] = str(root)
        return root

    def test_any_new_session_commits_on_main_and_preserves_foreign_staging(self):
        with self.fixture() as f:
            repo, _, _, payload, _, calls = f
            payload["sessionId"] = "a-previously-unregistered-chat"
            (repo / "owned.py").write_text("value = 2\n")
            (repo / "foreign.txt").write_text("staged\n")
            git(repo, "add", "foreign.txt")
            (repo / "foreign.txt").write_text("working\n")
            index_before = git(repo, "ls-files", "--stage", "--", "foreign.txt").stdout
            result = self.execute(f)
            self.assertEqual(result["status"], "complete")
            self.assertEqual(result["mergeMethod"], "already-on-main")
            self.assertEqual(len(calls), 1)
            self.assertEqual(git(repo, "show", "HEAD:owned.py").stdout, b"value = 2\n")
            self.assertEqual(git(repo, "show", "HEAD:foreign.txt").stdout, b"base\n")
            self.assertEqual(git(repo, "ls-files", "--stage", "--", "foreign.txt").stdout, index_before)
            self.assertEqual((repo / "foreign.txt").read_text(), "working\n")

    def test_failed_validation_does_not_commit(self):
        with self.fixture() as f:
            before = delivery.text(f[0], "rev-parse", "HEAD")
            failing = lambda *args: {"status": "failed", "files": {}, "head": before}
            result = delivery.execute(f[0], f[3], policy=f[1], grants=f[2], validate=failing)
            self.assertEqual(result["status"], "validation-failed")
            self.assertEqual(delivery.text(f[0], "rev-parse", "HEAD"), before)

    def test_changes_during_validation_wait_and_do_not_commit(self):
        with self.fixture() as f:
            original = f[4]
            def racing(*args):
                result = original(*args)
                (f[0] / "owned.py").write_text("value = 3\n")
                return result
            result = delivery.execute(f[0], f[3], policy=f[1], grants=f[2], validate=racing)
            self.assertEqual(result["status"], "waiting")
            self.assertNotIn("taskCommit", result)

    def test_repeat_reuses_original_result_without_validating_or_committing_again(self):
        with self.fixture() as f:
            (f[0] / "owned.py").write_text("value = 2\n")
            first = self.execute(f)
            (f[0] / "owned.py").write_text("next task\n")
            second = self.execute(f)
            self.assertEqual(first["mainHead"], second["mainHead"])
            self.assertTrue(second["reusedReceipt"])
            self.assertEqual(len(f[5]), 1)

    def test_worktree_fast_forward_validates_actual_candidate(self):
        with self.fixture() as f:
            root = self.worktree(f)
            (root / "owned.py").write_text("value = 2\n")
            result = self.execute(f)
            self.assertEqual(result["status"], "complete", result.get("reason"))
            self.assertEqual(result["mergeMethod"], "validated-fast-forward")
            self.assertEqual(result["mainHead"], delivery.text(root, "rev-parse", "HEAD"))
            self.assertEqual(result["candidateValidation"]["head"], result["mainHead"])
            self.assertEqual(len(f[5]), 2)

    def test_divergent_main_is_merged_and_candidate_revalidated(self):
        with self.fixture() as f:
            root = self.worktree(f)
            (root / "owned.py").write_text("value = 2\n")
            (f[0] / "dependency.py").write_text("dep = 2\n")
            git(f[0], "add", "dependency.py")
            git(f[0], "-c", "commit.gpgSign=false", "commit", "-m", "Other main task")
            result = self.execute(f)
            self.assertEqual(result["status"], "complete", result.get("reason"))
            self.assertEqual(git(f[0], "show", "HEAD:dependency.py").stdout, b"dep = 2\n")
            self.assertEqual(git(f[0], "show", "HEAD:owned.py").stdout, b"value = 2\n")
            self.assertEqual(len(delivery.text(f[0], "show", "-s", "--format=%P", "HEAD").split()), 2)
            self.assertEqual(result["candidateValidation"]["files"]["dependency.py"], delivery.files(f[0], ["dependency.py"])["dependency.py"])

    def test_actual_native_validation_and_worktree_delivery_end_to_end(self):
        with self.fixture() as f:
            root = self.worktree(f)
            (root / "owned.py").write_text("value = 2\n")
            runtime = delivery.commit.read_json(delivery.ROOT / ".jenkins/state/deployment/driver.json")
            runtime = {k: runtime[k] for k in ("driverDigest", "runtimeOperationId")}
            result = delivery.execute(f[0], f[3], policy=f[1], grants=f[2], validate=delivery.run_validation, runtime=runtime)
            self.assertEqual(result["status"], "complete", result.get("reason"))
            for receipt in (result["validation"], result["candidateValidation"]):
                self.assertEqual(receipt["status"], "passed")
                self.assertTrue(receipt["steps"][0]["complete"])
                self.assertEqual(receipt["steps"][0]["native"]["rootExitCode"], 0)
                self.assertEqual(receipt["cleanup"]["status"], "cleaned")
            self.assertEqual(result["mainHead"], delivery.text(root, "rev-parse", "HEAD"))

    def test_recording_same_validated_bytes_in_git_does_not_invalidate_validation(self):
        with self.fixture() as f:
            (f[0] / "owned.py").write_text("value = 2\n")
            runtime = delivery.commit.read_json(delivery.ROOT / ".jenkins/state/deployment/driver.json")
            runtime = {k: runtime[k] for k in ("driverDigest", "runtimeOperationId")}
            original_module = delivery.module
            def load(name, path):
                value = original_module(name, path)
                if name == "delivery_native_command":
                    run = value._run_owned_command
                    def record_after_command(*args):
                        native = run(*args)
                        git(f[0], "add", "owned.py")
                        git(f[0], "-c", "commit.gpgSign=false", "commit", "-m", "Another session records these same validated bytes")
                        return native
                    value._run_owned_command = record_after_command
                return value
            with patch.object(delivery, "module", side_effect=load):
                result = delivery.run_validation(f[0], f[3], ["owned.py"], f[0] / ".jenkins/concurrent-validation", runtime)
            self.assertEqual(result["status"], "passed")
            self.assertTrue(result["inputsUnchanged"])
            self.assertNotEqual(result["headAtStart"], result["head"])
            self.assertEqual(result["committedInputPathsDuringValidation"], ["owned.py"])

    def test_merge_conflict_retains_candidate_and_main_without_merge_state(self):
        with self.fixture() as f:
            root = self.worktree(f)
            (root / "owned.py").write_text("value = 'feature'\n")
            (f[0] / "owned.py").write_text("value = 'main'\n")
            git(f[0], "add", "owned.py")
            git(f[0], "-c", "commit.gpgSign=false", "commit", "-m", "Concurrent main edit")
            main = delivery.text(f[0], "rev-parse", "HEAD")
            result = self.execute(f)
            self.assertEqual(result["status"], "merge-conflict")
            self.assertEqual(delivery.text(f[0], "rev-parse", "HEAD"), main)
            self.assertNotEqual(git(root, "rev-parse", "--verify", "MERGE_HEAD", check=False).returncode, 0)

    def test_foreign_worktree_changes_are_retained_without_committing(self):
        with self.fixture() as f:
            root = self.worktree(f)
            (root / "foreign.txt").write_text("foreign\n")
            with self.assertRaisesRegex(ValueError, "outside this task"):
                self.execute(f)
            self.assertEqual((root / "foreign.txt").read_text(), "foreign\n")

    def test_changed_managed_worktree_branch_is_not_adopted(self):
        with self.fixture() as f:
            root = self.worktree(f)
            git(root, "checkout", "--detach")
            with self.assertRaisesRegex(ValueError, "branch differs"):
                self.execute(f)

    def test_changed_operation_identity_and_missing_validation_are_rejected(self):
        with self.fixture() as f:
            self.execute(f)
            f[3]["message"] = "another task"
            with self.assertRaisesRegex(ValueError, "another delivery"):
                self.execute(f)
            f[3]["validation"] = []
            with self.assertRaisesRegex(ValueError, "validation steps"):
                self.execute(f)

    def test_native_runtime_change_prevents_commit(self):
        with self.fixture() as f:
            def changed():
                raise ValueError("Runtime changed")
            with self.assertRaisesRegex(ValueError, "Runtime changed"):
                self.execute(f, check_runtime=changed)
            self.assertNotIn("taskCommit", delivery.commit.read_json(f[0] / ".jenkins/state/management/deliveries/delivery-task.json"))


class NativeCargoInvocationTests(unittest.TestCase):
    def test_vsdev_wrapper_preserves_native_flags_and_literal_arguments(self):
        with tempfile.TemporaryDirectory(prefix="delivery-native-", dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name:
            destination = Path(name)
            arguments = ["-p", "zircon_runtime", "--features", "graphics,ui", "--", "--exact",
                         "argument with spaces", "single'quote", 'double"quote', "中文",
                         "$([Environment]::Exit(91))", "`literal", "--profile", "dev"]
            native_command = [sys.executable, "-B", "-Xutf8", "-c",
                              "import json,sys; print(json.dumps(sys.argv[1:],ensure_ascii=True))", *arguments]
            command = delivery.vsdev_command(native_command, destination)
            cargo = delivery.module("delivery_invocation_native_test", delivery.ROOT / "tools/dev/local_cargo.py")
            checks = delivery.module("delivery_invocation_checks_test", delivery.ROOT / "tools/jenkins-management/run_checks.py")
            terminal = cargo._run_owned_command(command, destination, dict(os.environ), destination, 120)
            self.assertTrue(checks.native_complete(terminal, command, destination), terminal)
            self.assertEqual(terminal["rootExitCode"], 0, (destination / "stderr.bin").read_bytes())
            self.assertEqual(json.loads((destination / "stdout.bin").read_text(encoding="utf-8")), arguments)


class DeliveryInstallTests(unittest.TestCase):
    def test_extension_preserves_other_routes_and_reinstall_is_idempotent(self):
        class Client:
            def __init__(self):
                self.raw = b'<flow-definition><definition><script>node("zircon-windows-agent") { echo "--capsule core and commit routes" }</script></definition></flow-definition>'
                self.posts = 0
            def request(self, path, method="GET", data=None, **kwargs):
                if method == "POST":
                    self.raw = data
                    self.posts += 1
                return 200, self.raw, {}
        client = Client()
        with patch.object(delivery, "policy_for"):
            delivery.install(Path(__file__).parents[3], client)
            delivery.install(Path(__file__).parents[3], client)
        self.assertEqual(client.posts, 1)
        script = ET.fromstring(client.raw).findtext("definition/script")
        self.assertIn('--capsule core and commit routes', script)
        self.assertEqual(script.count(delivery.BEGIN), 1)


class RuntimeBindingTests(unittest.TestCase):
    def test_unrelated_adapter_edits_do_not_block_but_sealed_code_drift_does(self):
        with tempfile.TemporaryDirectory(prefix="delivery-runtime-", dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name:
            repo = Path(name)
            def write(path, value):
                path.parent.mkdir(parents=True, exist_ok=True)
                data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
                path.write_bytes(data)
                return hashlib.sha256(data).hexdigest()
            data = b"immutable deployed implementation"
            manifest = {"schemaVersion": 1, "files": {"implementation.py": {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}}}
            digest = hashlib.sha256(json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            driver = repo / ".jenkins/runtime/drivers" / digest
            write(driver / "driver-manifest.json", manifest)
            (driver / "implementation.py").write_bytes(data)
            config = {"repositoryRoot": str(repo), "driverDigest": digest, "runtimeOperationId": "runtime-one",
                      "files": {n: hashlib.sha256(data).hexdigest() for n in ("runner.py", "source_preparation.py", "zircon-management.groovy")},
                      "clientSources": {"tools/dev/unused-adapter.ps1": "f" * 64}}
            checksum = hashlib.sha256(json.dumps(config, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            capsule = repo / ".jenkins/runtime/management" / checksum
            write(capsule / "capsule.json", config)
            for path in config["files"]:
                (capsule / path).write_bytes(data)
            write(repo / ".jenkins/state/management/installed.json", {"job": "zircon-management", "capsule": str(capsule),
                  "capsuleSha256": checksum, "driverDigest": digest, "runtimeOperationId": "runtime-one"})
            write(repo / ".jenkins/state/deployment/driver.json", {"driverDigest": digest, "runtimeOperationId": "runtime-one"})
            write(repo / ".jenkins/state/deployment-operation.json", {"operationId": "runtime-one", "state": "running"})
            unused = repo / "tools/dev/unused-adapter.ps1"
            unused.parent.mkdir(parents=True)
            unused.write_text("another session's current adapter")
            self.assertEqual(delivery.runtime_files(repo)[2], driver)
            (driver / "implementation.py").write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "driver implementation changed"):
                delivery.runtime_files(repo)


if __name__ == "__main__":
    unittest.main(verbosity=2)

"""Real Git behavior for commit-first delivery and scheduled integration."""
from contextlib import contextmanager, closing
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import sys
import tempfile
import unittest
from unittest.mock import patch


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).parents[1] / filename)
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


d = load("branch_queue_test_delivery", "delivery.py")
queue = load("branch_queue_tests", "branch_integration.py")


class BranchQueueTests(unittest.TestCase):
    @contextmanager
    def fixture(self):
        with tempfile.TemporaryDirectory(prefix="branch-queue-", dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name, patch.dict(os.environ, {"ZR_COMMIT_NATIVE": "0"}):
            repo = Path(name)
            g = lambda root, *args: queue.git(d, root, *args)
            g(repo, "init", "--quiet", "-b", "main")
            g(repo, "config", "user.name", "Jenkins queue test")
            g(repo, "config", "user.email", "jenkins-queue@localhost")
            for filename, content in {"owned.py": "value = 1\n", "foreign.txt": "base\n", "dependency.py": "dep = 1\n", ".gitignore": ".jenkins/\n"}.items():
                (repo / filename).write_text(content, encoding="utf-8")
            config = repo / ".codex/config.toml"
            config.parent.mkdir()
            config.write_text("[permissions.zircon-strict.filesystem]\n'" + str(repo) + "' = 'write'\n", encoding="utf-8")
            g(repo, "add", ".")
            g(repo, "-c", "commit.gpgSign=false", "commit", "--quiet", "-m", "baseline")
            root = repo / ".jenkins/worktrees/task-source"
            g(repo, "worktree", "add", "-b", "codex/jenkins/task/source", str(root), "HEAD")
            database = repo / ".jenkins/state/coordination.sqlite3"
            database.parent.mkdir(parents=True)
            with closing(sqlite3.connect(database)) as connection:
                connection.execute("CREATE TABLE records(domain TEXT,key TEXT,payload TEXT)")
                connection.execute("INSERT INTO records VALUES(?,?,?)", ("management_worktree", "task-source",
                    json.dumps({"path": str(root), "sessionId": "original-session", "status": "ready", "branch": "codex/jenkins/task/source", "ownedPaths": ["owned.py"]})))
                connection.commit()
            payload = {"action": "deliver-task", "sessionId": "original-session", "requestId": "task-delivery", "operationId": "task-delivery",
                       "ownerThreadId": "owner-thread", "sourceRoot": str(root), "message": "Implement task",
                       "paths": ["owned.py"], "dependencies": ["dependency.py"],
                       "validation": [{"kind": "python-static", "paths": ["owned.py"]}]}
            policy = {"authorization": {"source": "user", "instruction": "Commit own branch first, then scheduled validation/main integration"}, "integration": {"mode": queue.MODE}}
            with patch.object(d, "policy_for", return_value=policy), patch.object(d, "OUTPUT", repo / ".jenkins/test-output"):
                yield repo, root, payload, policy, {str(repo): "write"}, g

    def stage(self, fixture):
        repo, _, payload, policy, grants, _ = fixture
        return queue.stage(d, repo, payload, policy=policy, grants=grants, runtime={})

    def validation(self, root, request, names, output, runtime):
        return {"status": "passed", "head": queue.text(d, root, "rev-parse", "HEAD"), "files": d.files(root, names),
                "steps": [{"complete": True, "status": "passed"}]}

    def integrate(self, fixture, validate=None):
        return queue.integrate(d, fixture[0], queue.directory(fixture[0]) / "requests/task-delivery.json",
                               runtime={}, check_runtime=lambda: None, validate=validate or self.validation)

    def test_branch_commits_before_validation_preserves_foreign_staging_and_main(self):
        with self.fixture() as f:
            repo, root, _, _, _, g = f
            before = queue.text(d, repo, "rev-parse", "HEAD")
            (root / "owned.py").write_text("value = 2\n")
            (root / "foreign.txt").write_text("staged\n")
            g(root, "add", "foreign.txt")
            staged = g(root, "ls-files", "--stage", "--", "foreign.txt").stdout
            (root / "foreign.txt").write_text("working\n")
            with patch.object(d, "run_validation", side_effect=AssertionError("Stage must not run validation")):
                result = self.stage(f)
            self.assertEqual(result["status"], "committed", result.get("reason"))
            self.assertEqual(result["integrationStatus"], "pending")
            self.assertFalse(result["validationPerformed"])
            self.assertEqual(queue.text(d, repo, "rev-parse", "HEAD"), before)
            self.assertEqual(g(root, "show", "HEAD:owned.py").stdout.replace(b"\r\n", b"\n"), b"value = 2\n")
            self.assertEqual(g(root, "show", "HEAD:foreign.txt").stdout.replace(b"\r\n", b"\n"), b"base\n")
            self.assertEqual(g(root, "ls-files", "--stage", "--", "foreign.txt").stdout, staged)
            self.assertEqual((root / "foreign.txt").read_text(), "working\n")

    def test_worktree_index_lock_is_checked_at_real_git_directory_before_commit(self):
        with self.fixture() as f:
            root = f[1]
            (root / "owned.py").write_text("value = 2\n")
            observation = queue.index_observation(d, root)
            lock = Path(observation["path"])
            lock.write_bytes(b"")
            before = queue.text(d, root, "rev-parse", "HEAD")
            result = self.stage(f)
            self.assertEqual(result["status"], "waiting")
            self.assertTrue(result["indexLock"]["exists"])
            self.assertEqual(Path(result["indexLock"]["path"]), lock)
            self.assertTrue(lock.exists())
            self.assertEqual(queue.text(d, root, "rev-parse", "HEAD"), before)

    def test_main_checkout_cannot_use_branch_commit_entry(self):
        with self.fixture() as f:
            f[2]["sourceRoot"] = str(f[0])
            result = self.stage(f)
            self.assertEqual(result["status"], "failed")
            self.assertIn("worktree branch first", result["reason"])
            self.assertTrue((queue.directory(f[0]) / "requests/task-delivery.json").is_file())

    def test_repeat_returns_original_branch_commit_without_committing_later_edits(self):
        with self.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            first = self.stage(f)
            (f[1] / "owned.py").write_text("value = 3\n")
            second = self.stage(f)
            self.assertEqual(first["sourceCommit"], second["sourceCommit"])
            self.assertTrue(second["reusedReceipt"])

    def test_hash_drift_does_not_commit(self):
        with self.fixture() as f:
            f[2]["expectedFileHashes"] = {"owned.py": "a" * 64}
            result = self.stage(f)
            self.assertEqual(result["status"], "failed")
            self.assertNotIn("sourceCommit", result)

    def test_scheduled_candidate_validates_recorded_commit_then_integrates(self):
        with self.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            source = self.stage(f)
            observed = []
            def validate(*args):
                observed.append(queue.text(d, args[0], "rev-parse", "HEAD"))
                self.assertEqual(queue.text(d, f[0], "rev-parse", "HEAD"), source["headBefore"])
                return self.validation(*args)
            result = self.integrate(f, validate)
            self.assertEqual(result["status"], "merged", result.get("reason"))
            self.assertEqual(observed, [result["candidateHead"]])
            self.assertEqual(result["mainHead"], queue.text(d, f[0], "rev-parse", "HEAD"))
            self.assertTrue(result["mainAdvanced"])

    def test_failed_validation_keeps_branch_commit_and_does_not_advance_main(self):
        with self.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            source = self.stage(f)
            def fail(*args):
                return {**self.validation(*args), "status": "failed"}
            result = self.integrate(f, fail)
            self.assertEqual(result["integrationStatus"], "validation-failed")
            self.assertEqual(queue.text(d, f[0], "rev-parse", "HEAD"), source["headBefore"])
            self.assertEqual(queue.text(d, f[1], "rev-parse", "HEAD"), source["sourceCommit"])
            self.assertTrue(Path(result["notification"]["path"]).is_file())
            self.assertFalse(result["notification"]["recipientRead"])

    def test_main_lock_blocks_candidate_creation_and_is_never_deleted(self):
        with self.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            self.stage(f)
            lock = f[0] / ".git/index.lock"
            lock.write_bytes(b"")
            result = self.integrate(f)
            self.assertEqual(result["status"], "committed")
            self.assertEqual(result["integrationStatus"], "waiting")
            self.assertTrue(result["mainIndexLock"]["exists"])
            self.assertTrue(lock.exists())
            self.assertFalse((f[0] / ".jenkins/worktrees/integration-queue").exists())

    def test_later_foreign_branch_commit_is_not_implicitly_included(self):
        with self.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            source = self.stage(f)
            (f[1] / "foreign.txt").write_text("another task\n")
            f[5](f[1], "add", "foreign.txt")
            f[5](f[1], "-c", "commit.gpgSign=false", "commit", "-m", "Unqueued later task")
            result = self.integrate(f)
            self.assertEqual(result["status"], "merged", result.get("reason"))
            self.assertEqual(result["mainHead"], source["sourceCommit"])
            self.assertEqual(f[5](f[0], "show", "HEAD:foreign.txt").stdout.replace(b"\r\n", b"\n"), b"base\n")

    def test_actual_native_candidate_validation_has_exit_eof_and_cleanup(self):
        with self.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            self.stage(f)
            runtime = d.commit.read_json(d.ROOT / ".jenkins/state/deployment/driver.json")
            result = queue.integrate(d, f[0], queue.directory(f[0]) / "requests/task-delivery.json",
                runtime={k: runtime[k] for k in ("driverDigest", "runtimeOperationId")}, check_runtime=lambda: None)
            self.assertEqual(result["status"], "merged", result.get("reason"))
            step = result["candidateValidation"]["steps"][0]
            self.assertTrue(step["complete"])
            self.assertEqual(step["native"]["rootExitCode"], 0)
            self.assertEqual(result["candidateValidation"]["cleanup"]["status"], "cleaned")

    def test_vsdev_wrapper_preserves_native_dash_arguments(self):
        with self.fixture() as f:
            output = f[0] / ".jenkins/vsdev-args"
            output.mkdir()
            command = d.vsdev_command([str(d.ROOT / ".jenkins/runtime/python/python.exe"), "-B", "-c",
                "import sys; assert sys.argv[1:] == ['-p', '-C', '--lib']; print('native-argv-ok')", "-p", "-C", "--lib"], output)
            native_module = d.module("queue_native_argv", d.ROOT / "tools/dev/local_cargo.py")
            native = native_module._run_owned_command(command, f[0], dict(os.environ), output, 90)
            diagnostics = {p.name: p.read_bytes().decode("utf-8", "replace")[-12000:]
                           for p in output.glob("*.bin")}
            self.assertEqual(native["rootExitCode"], 0, json.dumps(diagnostics, ensure_ascii=False))
            self.assertTrue(native["processTree"]["terminal"])


if __name__ == "__main__":
    unittest.main()

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
    def fixture(self, owned_name="owned.py"):
        with tempfile.TemporaryDirectory(prefix="branch-queue-", dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name, patch.dict(os.environ, {"ZR_COMMIT_NATIVE": "0"}):
            repo = Path(name)
            g = lambda root, *args: queue.git(d, root, *args)
            g(repo, "init", "--quiet", "-b", "main")
            g(repo, "config", "user.name", "Jenkins queue test")
            g(repo, "config", "user.email", "jenkins-queue@localhost")
            for filename, content in {owned_name: "value = 1\n", "foreign.txt": "base\n", "dependency.py": "dep = 1\n", ".gitignore": ".jenkins/\n"}.items():
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
                    json.dumps({"path": str(root), "sessionId": "original-session", "status": "ready", "branch": "codex/jenkins/task/source", "ownedPaths": [owned_name.casefold()]})))
                connection.commit()
            payload = {"action": "deliver-task", "sessionId": "original-session", "requestId": "task-delivery", "operationId": "task-delivery",
                       "ownerThreadId": "owner-thread", "sourceRoot": str(root), "message": "Implement task",
                       "paths": [owned_name], "dependencies": ["dependency.py"],
                       "validation": [{"kind": "python-static", "paths": [owned_name]}]}
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
                               runtime={}, check_runtime=lambda: None, validate=validate or self.validation,
                               pull_request=lambda d, repo, root, record, policy: {"status": "merged", "mergeCommit": record["candidateHead"]})

    def test_casefolded_worktree_scope_preserves_actual_git_filename(self):
        with self.fixture("Owned.py") as f:
            repo, root, payload, policy, grants, g = f
            (root / "Owned.py").write_text("value = 2\n")
            result = self.stage(f)
            self.assertEqual(result["status"], "committed", result.get("reason"))
            self.assertEqual(g(root, "show", "HEAD:Owned.py").stdout.replace(b"\r\n", b"\n"), b"value = 2\n")
            self.assertEqual(result["request"]["paths"], ["Owned.py"])

    def test_casefolded_scope_rejects_sibling_prefix(self):
        with self.fixture("Owned.py") as f:
            repo, root, payload, policy, grants, g = f
            payload["paths"] = ["Owned.py.foreign.py"]
            (root / "Owned.py.foreign.py").write_text("foreign = True\n")
            before = queue.text(d, root, "rev-parse", "HEAD")
            result = self.stage(f)
            self.assertEqual(result["status"], "failed")
            self.assertIn("exceeds its original worktree ownership", result["reason"])
            self.assertEqual(queue.text(d, root, "rev-parse", "HEAD"), before)

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

    def test_active_candidate_keeps_other_task_waiting_without_mutation(self):
        with self.fixture() as f:
            repo, source_root, payload, policy, grants, g = f
            (source_root / "owned.py").write_text("value = 2\n")
            first = self.stage(f)
            other_payload = {**payload, "requestId": "other-delivery", "operationId": "other-delivery"}
            (source_root / "owned.py").write_text("value = 3\n")
            other = self.stage((repo, source_root, other_payload, policy, grants, g))
            main = queue.text(d, repo, "rev-parse", "HEAD")
            candidate = repo / ".jenkins/worktrees/integration-queue"
            g(repo, "worktree", "add", "--detach", str(candidate), first["sourceCommit"])
            marker = queue.directory(repo) / "candidate.json"
            d.commit.write_json(marker, {"path": str(candidate), "status": "active",
                "operationId": payload["operationId"], "mainHead": main})
            validation_path = repo / ".jenkins/test-output/original-validation.json"
            validation = {"status": "waiting", "steps": [{"status": "waiting", "complete": False}]}
            d.commit.write_json(validation_path, validation)
            original_path = queue.directory(repo) / "requests/task-delivery.json"
            d.commit.write_json(original_path, {**first, "integrationStatus": "validating",
                "candidateRoot": str(candidate), "candidateHead": first["sourceCommit"],
                "candidateValidation": validation})
            index = Path(queue.index_observation(d, candidate)["path"]).with_name("index")
            preserved = {path: path.read_bytes() for path in (marker, original_path, validation_path,
                index, repo / ".git/index", source_root / "owned.py", candidate / "owned.py")}
            observed = []
            def validate(*args):
                observed.append(args)
                self.fail("Another task must not enter validation while the candidate is active")
            result = queue.integrate(d, repo, queue.directory(repo) / "requests/other-delivery.json",
                runtime={}, check_runtime=lambda: None, validate=validate)
            self.assertEqual(result["integrationStatus"], "waiting", result.get("reason"))
            self.assertIn("original native/merge result needs reconciliation", result["reason"])
            self.assertEqual(observed, [])
            self.assertNotIn("candidateValidation", result)
            self.assertNotIn("candidateGit", result)
            self.assertEqual(queue.text(d, candidate, "rev-parse", "HEAD"), first["sourceCommit"])
            self.assertEqual(queue.text(d, source_root, "rev-parse", "HEAD"), other["sourceCommit"])
            self.assertEqual(queue.text(d, repo, "rev-parse", "HEAD"), main)
            self.assertEqual(preserved, {path: path.read_bytes() for path in preserved})

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

    def test_main_lock_allows_candidate_validation_but_blocks_publication(self):
        with self.fixture() as f:
            repo, root, payload, _, _, g = f
            (root / "owned.py").write_text("value = 2\n")
            source = self.stage(f)
            (repo / "dependency.py").write_text("dep = 2\n")
            g(repo, "add", "dependency.py")
            g(repo, "-c", "commit.gpgSign=false", "commit", "--quiet", "-m", "Advance current main")
            main = queue.text(d, repo, "rev-parse", "HEAD")
            main_ref = repo / ".git/refs/heads/main"
            ref_before = main_ref.read_bytes()
            foreign = repo / "foreign.txt"
            foreign.write_text("foreign staged\n")
            g(repo, "add", "foreign.txt")
            foreign.write_text("foreign working\n")
            index = repo / ".git/index"
            index_before, index_stat = index.read_bytes(), index.stat()
            lock = repo / ".git/index.lock"
            lock.write_bytes(b"foreign index owner\n")
            lock_before, lock_stat = lock.read_bytes(), lock.stat()
            selected = d.commit.read_json(d.ROOT / ".jenkins/state/deployment/driver.json")
            runtime = {k: selected[k] for k in ("driverDigest", "runtimeOperationId")}
            observed = []
            def validate(*args):
                candidate = args[0]
                observed.append(candidate)
                self.assertNotEqual(candidate, repo)
                self.assertNotEqual(Path(queue.index_observation(d, candidate)["path"]), lock)
                self.assertEqual(g(candidate, "show", "HEAD:dependency.py").stdout.replace(b"\r\n", b"\n"), b"dep = 2\n")
                self.assertEqual(g(candidate, "show", "HEAD:owned.py").stdout.replace(b"\r\n", b"\n"), b"value = 2\n")
                self.assertEqual(g(candidate, "show", "HEAD:foreign.txt").stdout.replace(b"\r\n", b"\n"), b"base\n")
                return d.run_validation(*args)
            result = queue.integrate(d, repo, queue.directory(repo) / "requests/task-delivery.json",
                runtime=runtime, check_runtime=lambda: None, validate=validate,
                pull_request=lambda d, repo, root, record, policy: {"status": "merged", "mergeCommit": record["candidateHead"]})
            self.assertEqual(observed, [Path(result["candidateRoot"])])
            self.assertEqual(result["candidateValidation"]["status"], "passed", result.get("reason"))
            step = result["candidateValidation"]["steps"][0]
            self.assertTrue(step["complete"])
            self.assertEqual(step["native"]["rootExitCode"], 0)
            self.assertEqual(result["candidateValidation"]["cleanup"]["status"], "cleaned")
            self.assertEqual(result["status"], "committed")
            self.assertEqual(result["integrationStatus"], "waiting")
            self.assertIn("index.lock requires original-owner reconciliation", result["reason"])
            self.assertTrue(result["mainIndexLock"]["exists"])
            self.assertFalse(result["mainAdvanced"])
            self.assertFalse(result["pushPerformed"])
            self.assertEqual(result["request"], payload)
            self.assertEqual(main_ref.read_bytes(), ref_before)
            self.assertEqual(queue.text(d, repo, "rev-parse", "HEAD"), main)
            self.assertEqual(queue.text(d, root, "rev-parse", "HEAD"), source["sourceCommit"])
            self.assertEqual(index.read_bytes(), index_before)
            self.assertEqual((index.stat().st_dev, index.stat().st_ino), (index_stat.st_dev, index_stat.st_ino))
            self.assertEqual(lock.read_bytes(), lock_before)
            self.assertEqual((lock.stat().st_dev, lock.stat().st_ino, lock.stat().st_mtime_ns),
                             (lock_stat.st_dev, lock_stat.st_ino, lock_stat.st_mtime_ns))
            self.assertEqual(foreign.read_text(), "foreign working\n")

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
                runtime={k: runtime[k] for k in ("driverDigest", "runtimeOperationId")}, check_runtime=lambda: None,
                pull_request=lambda d, repo, root, record, policy: {"status": "merged", "mergeCommit": record["candidateHead"]})
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

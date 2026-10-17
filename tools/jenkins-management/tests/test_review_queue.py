"""Tree reuse and validated origin synchronization regressions (real local Git)."""
import importlib.util
from contextlib import nullcontext
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("review_queue_fixture", Path(__file__).with_name("test_branch_integration.py"))
fixture = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = fixture
spec.loader.exec_module(fixture)
queue, d = fixture.queue, fixture.d


class ReviewQueueTests(unittest.TestCase):
    def setUp(self):
        if os.name != "nt":
            # Portable queue logic only; Windows native read-pin acceptance is
            # covered by the existing native lane, not claimed by these tests.
            pins = patch.object(d.commit, "verified_commit_files", lambda *args: nullcontext())
            pins.start()
            self.addCleanup(pins.stop)

    def test_same_tree_reuses_validation_after_another_candidate_used_worktree(self):
        case = fixture.BranchQueueTests()
        with case.fixture() as f:
            repo, root, payload, _, _, g = f
            (repo / "foreign.txt").write_text("new main\n")
            g(repo, "add", "foreign.txt")
            g(repo, "commit", "-qm", "Another main change")
            (root / "owned.py").write_text("value = 2\n")
            staged = case.stage(f)
            self.assertEqual(staged["status"], "committed", staged)
            path = queue.directory(repo) / "requests/task-delivery.json"
            validate = lambda *args: {**case.validation(*args), "runtime": {}}
            awaiting = lambda *args: {"status": "awaiting-pr"}
            first = queue.integrate(d, repo, path, runtime={}, check_runtime=lambda: None, validate=validate, pull_request=awaiting)
            candidate = Path(first["candidateRoot"])
            g(candidate, "checkout", "--detach", queue.text(d, repo, "rev-parse", "HEAD"))
            original_git = queue.git
            def different_commit(d, root, *args, **kwargs):
                if "merge" in args:
                    args = ("-c", "user.name=Different merge committer", *args)
                return original_git(d, root, *args, **kwargs)
            with patch.object(queue, "git", side_effect=different_commit):
                second = queue.integrate(d, repo, path, runtime={}, check_runtime=lambda: None,
                    validate=lambda *args: self.fail("same tree must reuse validation"), pull_request=awaiting)
            self.assertEqual(second["integrationStatus"], "awaiting-pr", second)
            self.assertNotEqual(first["candidateHead"], second["candidateHead"])
            self.assertTrue(second["validationReused"])

    def origin_fixture(self, f):
        repo, _, _, policy, _, g = f
        before = queue.text(d, repo, "rev-parse", "HEAD")
        upstream = repo / ".jenkins/worktrees/upstream-fixture"
        g(repo, "worktree", "add", "--detach", str(upstream), before)
        (upstream / "foreign.txt").write_text("out of band PR\n")
        g(upstream, "add", "foreign.txt")
        g(upstream, "commit", "-qm", "Out of band PR")
        revision = queue.text(d, upstream, "rev-parse", "HEAD")
        g(repo, "update-ref", "refs/remotes/origin/main", revision)
        g(repo, "remote", "add", "origin", "https://github.com/Owner/Repo.git")
        policy["integration"]["pullRequestRepository"] = "Owner/Repo"
        return before, revision

    def synchronize(self, case, f, *, passed):
        before, revision = self.origin_fixture(f)
        original_git = queue.git
        def local_git(d, root, *args, **kwargs):
            if "fetch" in args:
                return subprocess.CompletedProcess([], 0, b"", b"")
            return original_git(d, root, *args, **kwargs)
        def validate(root, payload, names, output, runtime):
            self.assertEqual(queue.text(d, f[0], "rev-parse", "HEAD"), before)
            self.assertEqual(queue.text(d, root, "rev-parse", "HEAD"), revision)
            return {**case.validation(root, payload, names, output, runtime), "status": "passed" if passed else "failed",
                    "steps": [{"complete": True, "status": "passed" if passed else "failed"}] * len(queue.ORIGIN_MAIN_VALIDATION)}
        with patch.object(queue, "git", side_effect=local_git):
            if passed:
                result = queue.sync_origin_main(d, f[0], runtime={}, check_runtime=lambda: None, validate=validate)
                self.assertEqual(result["status"], "synchronized")
                self.assertEqual(queue.text(d, f[0], "rev-parse", "HEAD"), revision)
            else:
                with self.assertRaises(d.commit.CommitBusy):
                    queue.sync_origin_main(d, f[0], runtime={}, check_runtime=lambda: None, validate=validate)
                self.assertEqual(queue.text(d, f[0], "rev-parse", "HEAD"), before)

    def test_origin_main_is_validated_before_fast_forward(self):
        case = fixture.BranchQueueTests()
        with case.fixture() as f:
            self.synchronize(case, f, passed=True)

    def test_failed_origin_validation_retains_local_main(self):
        case = fixture.BranchQueueTests()
        with case.fixture() as f:
            self.synchronize(case, f, passed=False)

    def test_awaiting_pr_is_successful_scheduled_observation(self):
        case = fixture.BranchQueueTests()
        with case.fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            staged = case.stage(f)
            self.assertEqual(staged["status"], "committed", staged)
            with patch.object(queue, "sync_origin_main", return_value={"status": "current"}), patch.object(
                queue, "integrate", return_value={**staged, "integrationStatus": "awaiting-pr"}
            ):
                result = queue.drain(d, f[0], runtime={}, check_runtime=lambda: None)
            self.assertEqual(result["status"], "complete")
            self.assertEqual(result["results"][0]["integrationStatus"], "awaiting-pr")

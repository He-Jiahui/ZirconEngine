"""Real Git regressions for optional Jenkins validation and immediate commits."""
import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("advisory_branch_fixtures", Path(__file__).with_name("test_branch_integration.py"))
fixtures = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixtures)
d, queue = fixtures.d, fixtures.queue


class AdvisoryValidationTests(unittest.TestCase):
    def fixture(self):
        return fixtures.BranchQueueTests().fixture()

    def advisory(self, f):
        f[3]["integration"].update(validationMode="advisory", validationRequired=False,
                                    blockCommitOnDirtyMain=False)

    def stage(self, f):
        return fixtures.BranchQueueTests().stage(f)

    def integrate(self, f, validate=None, pull_request=None):
        return queue.integrate(d, f[0], queue.directory(f[0]) / "requests/task-delivery.json",
            runtime={}, check_runtime=lambda: None,
            validate=validate or fixtures.BranchQueueTests().validation,
            pull_request=pull_request or (lambda d, repo, root, record, policy:
                                         {"status": "merged", "mergeCommit": record["candidateHead"]}))

    def test_commit_without_recipe_hashes_or_clean_main_preserves_foreign_changes(self):
        with self.fixture() as f:
            self.advisory(f)
            repo, root, payload, _, _, g = f
            payload.pop("validation")
            payload["expectedFileHashes"] = {"owned.py": "a" * 64}
            (repo / "foreign.txt").write_text("another session is editing main\n")
            (root / "owned.py").write_text("value = 2\n")
            (root / "foreign.txt").write_text("another staged task\n")
            g(root, "add", "foreign.txt")
            staged = g(root, "ls-files", "--stage", "--", "foreign.txt").stdout
            before = queue.text(d, repo, "rev-parse", "HEAD")
            with patch.object(d, "files", side_effect=AssertionError("No file hash gate")), \
                    patch.object(d.commit, "verified_commit_files", side_effect=AssertionError("No hash lock")), \
                    patch.object(d, "run_validation", side_effect=AssertionError("No pre-commit validation")):
                result = self.stage(f)
            self.assertEqual(result["status"], "committed", result.get("reason"))
            self.assertFalse(result["sourceHashVerification"])
            self.assertEqual(queue.text(d, repo, "rev-parse", "HEAD"), before)
            self.assertEqual((repo / "foreign.txt").read_text(), "another session is editing main\n")
            self.assertEqual(g(root, "ls-files", "--stage", "--", "foreign.txt").stdout, staged)
            self.assertEqual(g(root, "show", "HEAD:foreign.txt").stdout.strip(), b"base")

    def test_optional_empty_recipe_integrates_without_claiming_validation_pass(self):
        with self.fixture() as f:
            self.advisory(f)
            f[2]["validation"] = []
            (f[1] / "owned.py").write_text("value = 2\n")
            self.assertEqual(self.stage(f)["status"], "committed")
            result = self.integrate(f, validate=lambda *a: self.fail("No validation was requested"))
            self.assertEqual(result["status"], "merged", result.get("reason"))
            self.assertEqual(result["candidateValidation"]["status"], "skipped")
            self.assertFalse(result["formalAcceptance"])

    def test_failed_check_keeps_failure_and_owner_repair_but_allows_merged_pr(self):
        with self.fixture() as f:
            self.advisory(f)
            (f[1] / "owned.py").write_text("value = 2\n")
            self.stage(f)
            def fail(*args):
                value = fixtures.BranchQueueTests().validation(*args)
                value.update(status="failed", steps=[{"complete": True, "status": "failed", "exitCode": 1}])
                return value
            result = self.integrate(f, fail)
            self.assertEqual(result["status"], "merged", result.get("reason"))
            self.assertEqual(result["candidateValidation"]["status"], "failed")
            self.assertTrue(result["repairRequired"])
            notice = json.loads(Path(result["notification"]["path"]).read_text())
            self.assertEqual(notice["ownerThreadId"], "owner-thread")
            self.assertEqual(notice["status"], "repair-required")
            self.assertTrue(notice["repairRequired"])
            self.assertFalse(notice["recipientRead"])

    def test_failed_check_reused_while_waiting_for_pr(self):
        with self.fixture() as f:
            self.advisory(f)
            (f[1] / "owned.py").write_text("value = 2\n")
            self.stage(f)
            calls = []
            def fail(*args):
                calls.append(1)
                value = fixtures.BranchQueueTests().validation(*args)
                return {**value, "status": "failed", "runtime": {}}
            waiting = lambda *a: {"status": "awaiting-pr", "reason": "PR is open"}
            first = self.integrate(f, fail, waiting)
            second = self.integrate(f, fail, waiting)
            self.assertEqual(first["integrationStatus"], "awaiting-pr")
            self.assertTrue(second["validationReused"])
            self.assertEqual(calls, [1])

    def test_current_runtime_may_commit_original_owned_worktree_without_new_acceptance(self):
        with self.fixture() as f:
            self.advisory(f)
            (f[1] / "owned.py").write_text("value = 2\n")
            result = queue.stage(d, f[0], f[2], policy=f[3], grants=f[4], runtime={"driverDigest": "current"})
            self.assertEqual(result["status"], "committed", result.get("reason"))

    def test_original_index_lock_is_preserved_in_advisory_mode(self):
        with self.fixture() as f:
            self.advisory(f)
            (f[1] / "owned.py").write_text("value = 2\n")
            lock = Path(queue.index_observation(d, f[1])["path"])
            lock.write_bytes(b"owned by another live Git command")
            result = self.stage(f)
            self.assertEqual(result["status"], "waiting")
            self.assertEqual(lock.read_bytes(), b"owned by another live Git command")

    def test_interrupted_receipt_recovers_native_git_commit_without_hash_recheck(self):
        with self.fixture() as f:
            self.advisory(f)
            (f[1] / "owned.py").write_text("value = 2\n")
            first = self.stage(f)
            path = queue.directory(f[0]) / "requests/task-delivery.json"
            previous = {**first, "status": "committing"}
            previous.pop("sourceCommit")
            d.commit.write_json(path, previous)
            (f[1] / "owned.py").write_text("value = 3\n")
            result = self.stage(f)
            self.assertEqual(result["sourceCommit"], first["sourceCommit"])
            self.assertTrue(result["recoveredReceipt"])
            self.assertEqual((f[1] / "owned.py").read_text(), "value = 3\n")

    def test_real_pr_observation_accepts_remote_result_without_candidate_tree_gate(self):
        with self.fixture() as f:
            self.advisory(f)
            repo, root, _, policy, _, g = f
            policy["integration"].update(requirePullRequest=True, pullRequestRepository="example/engine")
            g(root, "remote", "add", "origin", "https://github.com/example/engine.git")
            (root / "owned.py").write_text("value = 2\n")
            source = self.stage(f)
            source.update(mainHeadBefore=source["headBefore"], candidateHead=source["sourceCommit"])
            info = {"number": 12, "head": {"sha": source["sourceCommit"], "ref": source["branch"],
                    "repo": {"full_name": "example/engine"}},
                    "base": {"ref": "main", "repo": {"full_name": "example/engine"}}}
            detail = {**info, "merged": True, "state": "closed", "merge_commit_sha": source["headBefore"]}
            original_git = queue.git
            def observe_git(d, path, *args, **kwargs):
                if args[0] == "fetch":
                    return original_git(d, path, "update-ref", "refs/remotes/origin/main", detail["merge_commit_sha"])
                return original_git(d, path, *args, **kwargs)
            with patch.object(queue, "github_json", side_effect=[[info], detail]), \
                    patch.object(queue, "git", side_effect=observe_git):
                result = queue.verified_pr_commit(d, repo, root, source, policy)
            self.assertEqual(result["status"], "merged")
            self.assertEqual(result["mergeCommit"], source["headBefore"])

    def test_active_native_validation_still_retains_candidate_without_blocking_branch_commit(self):
        with self.fixture() as f:
            self.advisory(f)
            (f[1] / "owned.py").write_text("value = 2\n")
            source = self.stage(f)
            def active(*args):
                return {**fixtures.BranchQueueTests().validation(*args), "status": "failed", "steps": [{"complete": False}]}
            result = self.integrate(f, active)
            self.assertEqual(result["status"], "committed")
            self.assertEqual(result["integrationStatus"], "waiting")
            self.assertEqual(queue.text(d, f[1], "rev-parse", "HEAD"), source["sourceCommit"])

    def test_live_policy_accepts_advisory_without_formal_acceptance(self):
        policy = d.policy_for(d.ROOT)
        self.assertTrue(d.advisory_validation(policy))
        self.assertFalse(policy["integration"]["validationRequired"])

    def test_live_runtime_does_not_require_old_capsule_acceptance(self):
        with patch.object(d, "runtime_files", side_effect=AssertionError("No historical capsule gate")):
            context = d.runtime_context(d.ROOT)
        self.assertTrue(context["advisory"])
        self.assertEqual(context["config"]["runtimeOperationId"],
                         d.commit.read_json(d.ROOT / ".jenkins/state/deployment/driver.json")["runtimeOperationId"])


if __name__ == "__main__":
    unittest.main()

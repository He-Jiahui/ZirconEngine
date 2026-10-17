"""Regression coverage for clean main and verified PR publication."""
import importlib.util
from pathlib import Path
import sys
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("main_pr_fixture", Path(__file__).with_name("test_branch_integration.py"))
fixture = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = fixture
spec.loader.exec_module(fixture)
queue, d = fixture.queue, fixture.d


class TestMainPrGate:
    def test_verified_pr_result_advances_main_and_leaves_it_clean(self):
        with fixture.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            case = fixture.BranchQueueTests()
            source = case.stage(f)
            result = case.integrate(f)
            assert result["status"] == "merged", result
            assert result["mainHead"] == source["sourceCommit"]
            assert not queue.text(d, f[0], "status", "--porcelain=v1", "--untracked-files=all")

    def test_dirty_main_blocks_task_commit_and_preserves_both_workspaces(self):
        for kind in ("working", "staged", "untracked"):
            with fixture.BranchQueueTests().fixture() as f:
                (f[1] / "owned.py").write_text("value = 2\n")
                name = "new.txt" if kind == "untracked" else "foreign.txt"
                (f[0] / name).write_text("foreign work\n")
                if kind == "staged":
                    f[5](f[0], "add", name)
                before = queue.text(d, f[1], "rev-parse", "HEAD")
                result = fixture.BranchQueueTests().stage(f)
                assert result["status"] == "waiting", result
                assert "Main must be clean" in result["reason"]
                assert queue.text(d, f[1], "rev-parse", "HEAD") == before
                assert (f[0] / name).read_text() == "foreign work\n"
                assert (f[1] / "owned.py").read_text() == "value = 2\n"

    def test_untracked_main_file_blocks_candidate_creation(self):
        with fixture.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            fixture.BranchQueueTests().stage(f)
            (f[0] / "foreign-new.txt").write_text("retain\n")
            result = fixture.BranchQueueTests().integrate(f)
            assert result["integrationStatus"] == "waiting"
            assert not (f[0] / ".jenkins/worktrees/integration-queue").exists()
            assert (f[0] / "foreign-new.txt").read_text() == "retain\n"

    def test_concurrent_main_edit_blocks_publication(self):
        with fixture.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            case = fixture.BranchQueueTests()
            source = case.stage(f)
            def validate(*args):
                (f[0] / "foreign.txt").write_text("concurrent work\n")
                return case.validation(*args)
            result = case.integrate(f, validate)
            assert result["integrationStatus"] == "waiting"
            assert queue.text(d, f[0], "rev-parse", "HEAD") == source["headBefore"]
            assert (f[0] / "foreign.txt").read_text() == "concurrent work\n"

    def test_validation_pass_without_merged_pr_never_advances_main(self):
        with fixture.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            case = fixture.BranchQueueTests()
            source = case.stage(f)
            result = queue.integrate(d, f[0], queue.directory(f[0]) / "requests/task-delivery.json",
                runtime={}, check_runtime=lambda: None, validate=case.validation,
                pull_request=lambda *args: {"status": "awaiting-pr", "reason": "PR still open"})
            assert result["integrationStatus"] == "awaiting-pr"
            assert result["mainAdvanced"] is False
            assert queue.text(d, f[0], "rev-parse", "HEAD") == source["headBefore"]

    def test_pr_identity_merged_state_and_result_tree(self):
        for state in ("missing", "open", "wrong-source", "wrong-tree", "merged"):
            with fixture.BranchQueueTests().fixture() as f:
                repo, root, _, _, _, g = f
                g(root, "remote", "add", "origin", "https://github.com/example/engine.git")
                (root / "owned.py").write_text("value = 2\n")
                source = fixture.BranchQueueTests().stage(f)
                source.update(mainHeadBefore=source["headBefore"], candidateHead=source["sourceCommit"])
                info = {"number": 12, "head": {"sha": source["sourceCommit"], "ref": source["branch"], "repo": {"full_name": "example/engine"}},
                        "base": {"ref": "main", "repo": {"full_name": "example/engine"}}}
                detail = {**info, "merged": state != "open", "state": "open" if state == "open" else "closed",
                          "merge_commit_sha": source["headBefore"] if state == "wrong-tree" else source["sourceCommit"]}
                if state == "wrong-source":
                    detail = {**detail, "head": {**info["head"], "sha": "0" * 40}}
                original_git = queue.git
                def git(d, path, *args, **kwargs):
                    if args[0] == "fetch":
                        return original_git(d, path, "update-ref", "refs/remotes/origin/main", detail["merge_commit_sha"])
                    return original_git(d, path, *args, **kwargs)
                policy = {"integration": {"requirePullRequest": True, "pullRequestRepository": "example/engine"}}
                with patch.object(queue, "github_json", side_effect=[[] if state == "missing" else [info], detail]), patch.object(queue, "git", side_effect=git):
                    if state in {"wrong-source", "wrong-tree"}:
                        expected = ValueError if state == "wrong-source" else d.commit.CommitBusy
                        try:
                            queue.verified_pr_commit(d, repo, root, source, policy)
                        except expected:
                            pass
                        else:
                            raise AssertionError("Mismatched PR must not be accepted")
                    else:
                        result = queue.verified_pr_commit(d, repo, root, source, policy)
                        assert result["status"] == ("merged" if state == "merged" else "awaiting-pr")

    def test_waiting_for_pr_reuses_exact_candidate_validation(self):
        with fixture.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            case = fixture.BranchQueueTests()
            case.stage(f)
            path = queue.directory(f[0]) / "requests/task-delivery.json"
            def validation(*args):
                return {**case.validation(*args), "runtime": {}}
            waiting = queue.integrate(d, f[0], path, runtime={}, check_runtime=lambda: None, validate=validation,
                pull_request=lambda *args: {"status": "awaiting-pr"})
            assert waiting["integrationStatus"] == "awaiting-pr"
            def must_reuse(*args):
                raise AssertionError("Unchanged source/runtime already has passing validation")
            result = queue.integrate(d, f[0], path, runtime={}, check_runtime=lambda: None, validate=must_reuse,
                pull_request=lambda d, repo, root, record, policy: {"status": "merged", "mergeCommit": record["candidateHead"]})
            assert result["status"] == "merged", result
            assert result["validationReused"] is True

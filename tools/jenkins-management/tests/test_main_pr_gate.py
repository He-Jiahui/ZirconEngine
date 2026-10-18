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

    def test_untracked_main_is_preserved_through_validation_and_real_pr_gates(self):
        with fixture.BranchQueueTests().fixture() as f:
            repo, root, payload, policy, _, g = f
            (root / "owned.py").write_text("value = 2\n")
            case = fixture.BranchQueueTests()
            source = case.stage(f)
            policy["integration"].update(requirePullRequest=True, pullRequestRepository="example/engine")
            g(repo, "remote", "add", "origin", "https://github.com/example/engine.git")
            foreign = repo / "foreign-new.txt"
            foreign.write_bytes(b"retain\n")
            foreign_stat = foreign.stat()
            index = repo / ".git/index"
            index_before, index_stat = index.read_bytes(), index.stat()
            main_ref = repo / ".git/refs/heads/main"
            ref_before = main_ref.read_bytes()
            selected = d.commit.read_json(d.ROOT / ".jenkins/state/deployment/driver.json")
            runtime = {k: selected[k] for k in ("driverDigest", "runtimeOperationId")}
            observed = []
            def validate(*args):
                observed.append(args[0])
                assert args[0] != repo
                return d.run_validation(*args)
            def missing_pr(*args):
                assert len(observed) == 1, "PR observation must follow actual candidate validation"
                return []
            path = queue.directory(repo) / "requests/task-delivery.json"
            with patch.object(queue, "github_json", side_effect=missing_pr) as api:
                waiting = queue.integrate(d, repo, path, runtime=runtime,
                    check_runtime=lambda: None, validate=validate)
            assert api.call_count == 1
            assert observed == [Path(waiting["candidateRoot"])]
            assert waiting["candidateValidation"]["status"] == "passed", waiting
            step = waiting["candidateValidation"]["steps"][0]
            assert step["complete"] and step["native"]["rootExitCode"] == 0
            assert waiting["candidateValidation"]["cleanup"]["status"] == "cleaned"
            assert waiting["integrationStatus"] == "awaiting-pr", waiting
            assert waiting["pullRequest"]["status"] == "awaiting-pr"
            assert waiting["mainAdvanced"] is False and waiting["pushPerformed"] is False
            assert main_ref.read_bytes() == ref_before
            assert index.read_bytes() == index_before
            assert foreign.read_bytes() == b"retain\n"
            info = {"number": 12,
                    "head": {"sha": source["sourceCommit"], "ref": source["branch"], "repo": {"full_name": "example/engine"}},
                    "base": {"ref": "main", "repo": {"full_name": "example/engine"}}}
            detail = {**info, "merged": True, "state": "closed", "merge_commit_sha": waiting["candidateHead"]}
            original_git = queue.git
            def fetch_observation(d, path, *args, **kwargs):
                if args[0] == "fetch":
                    return original_git(d, path, "update-ref", "refs/remotes/origin/main", waiting["candidateHead"])
                return original_git(d, path, *args, **kwargs)
            def must_reuse(*args):
                raise AssertionError("The exact candidate already has native validation")
            with patch.object(queue, "github_json", side_effect=[[info], detail]) as api, patch.object(queue, "git", side_effect=fetch_observation):
                result = queue.integrate(d, repo, path, runtime=runtime,
                    check_runtime=lambda: None, validate=must_reuse)
            assert api.call_count == 2
            assert result["validationReused"] is True
            assert result["pullRequest"]["status"] == "merged" and result["pullRequest"]["number"] == 12
            assert result["status"] == "committed" and result["integrationStatus"] == "waiting", result
            assert "Main must be clean" in result["reason"]
            assert result["mainAdvanced"] is False and result["pushPerformed"] is False
            assert result["request"] == payload and result["sourceCommit"] == source["sourceCommit"]
            assert main_ref.read_bytes() == ref_before
            assert queue.text(d, repo, "rev-parse", "HEAD") == source["headBefore"]
            assert queue.text(d, root, "rev-parse", "HEAD") == source["sourceCommit"]
            assert index.read_bytes() == index_before
            assert (index.stat().st_dev, index.stat().st_ino) == (index_stat.st_dev, index_stat.st_ino)
            assert foreign.read_bytes() == b"retain\n"
            assert (foreign.stat().st_dev, foreign.stat().st_ino, foreign.stat().st_mtime_ns) == (
                foreign_stat.st_dev, foreign_stat.st_ino, foreign_stat.st_mtime_ns)

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

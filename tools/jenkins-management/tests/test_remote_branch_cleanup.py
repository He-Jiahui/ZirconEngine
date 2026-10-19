"""Safety and recovery boundaries for the remote maintenance entry."""
import importlib.util
from contextlib import closing
import copy
import io
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
import time
from unittest.mock import patch

MODULE = Path(__file__).parents[1] / "remote_branch_cleanup.py"
spec = importlib.util.spec_from_file_location("remote_branch_cleanup_under_test", MODULE)
cleanup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cleanup)


class RemoteBranchCleanupTests(unittest.TestCase):
    def reason(self, **changes):
        values = dict(name="codex/finished", default="main", protected=False, refs=set(), oids=set(),
                      oid="a" * 40, open_heads=set(), patterns=["main", "master", "release/*"])
        values.update(changes)
        return cleanup.protected_reason(**values)

    def test_default_and_shared_release_branches_are_retained(self):
        self.assertEqual(self.reason(name="main"), "default-or-preserved-branch")
        self.assertEqual(self.reason(name="release/1"), "default-or-preserved-branch")

    def test_missing_protection_is_not_assumed_unprotected(self):
        self.assertEqual(self.reason(protected=None), "protected-or-protection-unknown")
        self.assertEqual(self.reason(protected=True), "protected-or-protection-unknown")

    def test_active_detached_worktree_protects_its_object(self):
        self.assertEqual(self.reason(oids={"a" * 40}), "worktree-or-unfinished-task")

    def test_active_pr_and_exact_task_branch_prevent_deletion(self):
        self.assertEqual(self.reason(open_heads={"codex/finished"}), "open-pull-request")
        self.assertEqual(self.reason(refs={"codex/finished"}), "worktree-or-unfinished-task")

    def test_refspec_and_option_shaped_names_are_rejected(self):
        for name in ("--all", "topic:main", "topic/../main", "topic//main", "a.lock"):
            with self.subTest(name=name):
                self.assertEqual(self.reason(name=name), "unrecognized-ref-name")

    def test_unknown_deletion_is_never_automatically_replayed(self):
        intent = dict(status="unknown", oid="a" * 40)
        self.assertEqual(cleanup.reconcile_pending(intent, "a" * 40), "original-deletion-outcome-needs-review")
        self.assertEqual(cleanup.reconcile_pending(intent, "b" * 40), "original-ref-changed-preserved")
        self.assertEqual(cleanup.reconcile_pending(intent, None), "original-deletion-confirmed-absent")

    def test_failed_and_unknown_task_inputs_keep_their_original_refs(self):
        value = {"status": "failed", "request": {"branch": "refs/heads/codex/owner", "sourceCommit": "b" * 40}}
        self.assertEqual(cleanup.reference_holds(value), ({"codex/owner"}, {"b" * 40}))
        value.pop("status")
        self.assertEqual(cleanup.reference_holds(value), ({"codex/owner"}, {"b" * 40}))

    def test_completed_records_do_not_permanently_lock_unused_refs(self):
        self.assertEqual(cleanup.reference_holds({"status": "merged", "branch": "codex/finished"}), (set(), set()))

    def test_source_acceptance_or_released_writer_does_not_close_task(self):
        for status in ("accepted", "released", "complete", "archived"):
            with self.subTest(status=status):
                self.assertEqual(cleanup.reference_holds({"status": status, "branch": "codex/owner"}),
                                 ({"codex/owner"}, set()))

    def test_pending_candidate_ref_and_commit_are_protected(self):
        self.assertEqual(cleanup.reference_holds({"status": "waiting", "candidateRef": "refs/heads/codex/stack",
                                                 "commit": "a" * 40}), ({"codex/stack"}, {"a" * 40}))

    def test_stacked_pull_request_keeps_both_head_and_base_branches(self):
        backend = SimpleNamespace(pages=lambda *args: [dict(head=dict(ref="codex/child", repo=None),
                                                           base=dict(ref="codex/parent"))])
        self.assertEqual(cleanup.open_heads(backend, "He-Jiahui/ZirconEngine"), {"codex/child", "codex/parent"})


class RemoteWorkflowTests(unittest.TestCase):
    def test_reference_cache_refreshes_changed_rows_without_reparsing_history(self):
        with closing(cleanup.sqlite3.connect(":memory:")) as connection:
            connection.execute("CREATE TABLE records(domain TEXT,key TEXT,payload TEXT,version INT,updated_at REAL)")
            connection.execute("INSERT INTO records VALUES(?,?,?,?,?)", ("task", "owner",
                json.dumps(dict(status="waiting", branch="codex/original")), 1, 1.0))
            cache = cleanup.StateReferenceCache()
            with patch.object(cleanup, "reference_holds", wraps=cleanup.reference_holds) as parsed:
                self.assertEqual(cache._refresh(connection), ({"codex/original"}, set()))
                self.assertEqual(cache._refresh(connection), ({"codex/original"}, set()))
                self.assertEqual(parsed.call_count, 1)
                connection.execute("UPDATE records SET payload=?,version=2,updated_at=2.0",
                    (json.dumps(dict(status="waiting", branch="codex/new")),))
                self.assertEqual(cache._refresh(connection), ({"codex/new"}, set()))
                self.assertEqual(parsed.call_count, 2)
                connection.execute("DELETE FROM records")
                self.assertEqual(cache._refresh(connection), (set(), set()))

    def test_native_credential_fill_keeps_secret_out_of_record_and_uses_exact_repository_url(self):
        commands = cleanup.Commands(cleanup.REPORT, time.monotonic() + 10, None, cleanup.ROOT / "git.exe")
        for key in ("GH_TOKEN", "GITHUB_TOKEN"):
            commands.environment.pop(key, None)
        with patch.object(cleanup, "save") as saved, \
                patch.object(cleanup.subprocess, "run", return_value=SimpleNamespace(returncode=0,
                    stdout=b"username=fixture\npassword=fixture-token\n", stderr=b"")) as native, \
                patch.object(cleanup.urllib.request, "urlopen", return_value=io.BytesIO(b'{"default_branch":"main"}')) as http:
            self.assertEqual(commands.api("He-Jiahui/ZirconEngine", ""), {"default_branch": "main"})
            self.assertEqual(http.call_args.args[0].full_url, "https://api.github.com/repos/He-Jiahui/ZirconEngine")
            self.assertEqual(native.call_args.args[0][-2:], ["credential", "fill"])
            self.assertNotIn("fixture-token", json.dumps(saved.call_args.args[1]))

    def test_api_failure_is_not_reported_as_git_lock_contention(self):
        commands = cleanup.Commands(cleanup.REPORT, time.monotonic() + 10, None, cleanup.ROOT / "git.exe")
        commands.environment["GH_TOKEN"] = "fixture-token"
        failure = cleanup.urllib.error.HTTPError("https://api.github.com/fixture", 403, "fixture", {}, None)
        with patch.object(cleanup.urllib.request, "urlopen", side_effect=failure):
            with self.assertRaisesRegex(ValueError, "HTTP 403"):
                commands.api("He-Jiahui/ZirconEngine", "")

    def test_existing_github_reader_works_without_github_cli_or_token_output(self):
        facts = {"default_branch": "main"}
        with patch.object(cleanup, "save") as saved, patch.object(cleanup.subprocess, "run") as native:
            commands = cleanup.Commands(cleanup.REPORT, time.monotonic() + 10,
                                        lambda repository, endpoint: facts, cleanup.ROOT / "git.exe")
            self.assertEqual(commands.api("He-Jiahui/ZirconEngine", ""), facts)
            native.assert_not_called()
            self.assertEqual(saved.call_args.args[1]["kind"], "github-read")
            self.assertNotIn("Authorization", json.dumps(saved.call_args.args[1]))

    def run_workflow(self, *, advance_during_push=False, fresh_pr=False, pending=None):
        policy_raw = b'{"test":true}'
        policy = dict(maxBranchesPerRepository=100, maxDeletesPerRepository=2, preservePatterns=["main"])
        journal = copy.deepcopy(pending or {})
        stored, pushes, observed = {}, [], {}
        repository = "He-Jiahui/ZirconEngine"
        a, b, c = "a" * 40, "b" * 40, "c" * 40
        key = repository + ":refs/heads/codex/finished"
        current_oid = a
        pr_queries = 0

        class Backend:
            def git(self, repo, *args, check=True):
                nonlocal current_oid
                if args[:2] == ("remote", "get-url"):
                    return SimpleNamespace(returncode=0, stdout=("https://github.com/" + repository + ".git\n").encode())
                if args[0] == "ls-remote":
                    name = args[-1].removeprefix("refs/heads/")
                    oid = b if name == "main" else current_oid
                    return SimpleNamespace(returncode=0, stdout=(oid + "\trefs/heads/" + name + "\n").encode() if oid else b"")
                if args[0] == "push":
                    pushes.append(args)
                    observed["intentBeforePush"] = copy.deepcopy(stored[str(cleanup.STATE / "journal.json")][key])
                    if advance_during_push:
                        current_oid = c
                        return SimpleNamespace(returncode=1, stdout=b"rejected stale info")
                    current_oid = None
                return SimpleNamespace(returncode=0, stdout=b"")

            def api(self, repository, suffix):
                if suffix == "":
                    return dict(full_name=repository, default_branch="main", archived=False, disabled=False)
                return dict(name="codex/finished", protected=False, commit=dict(sha=a))

            def pages(self, repository, suffix, limit=1000):
                nonlocal pr_queries
                if suffix == "/branches":
                    return [dict(name="main", protected=True, commit=dict(sha=b)),
                            dict(name="codex/finished", protected=False, commit=dict(sha=a))]
                pr_queries += 1
                return [dict(head=dict(ref="codex/finished", repo=None))] if fresh_pr and pr_queries > 1 else []

        def store(path, value, **kwargs):
            stored[str(path)] = copy.deepcopy(value)

        entry = dict(repository=repository, branches=[])
        error = None
        with patch.object(cleanup, "local_holds", return_value=(set(), set())), \
                patch.object(cleanup, "index_available"), patch.object(cleanup, "read", return_value=policy_raw), \
                patch.object(cleanup, "save", side_effect=store):
            try:
                cleanup.cleanup_repository(Backend(), cleanup.ROOT, repository, policy, cleanup.sha(policy_raw),
                                           "apply", cleanup.REPORT / "test", journal, entry, lambda: None)
            except ValueError as failure:
                error = str(failure)
        return SimpleNamespace(entry=entry, journal=journal, pushes=pushes, observed=observed, error=error, key=key)

    def test_deletion_is_journaled_first_and_uses_exact_old_oid(self):
        run = self.run_workflow()
        self.assertIsNone(run.error)
        self.assertEqual(run.observed["intentBeforePush"]["status"], "unknown")
        self.assertEqual(run.pushes, [("push", "--porcelain", "--force-with-lease=refs/heads/codex/finished:" + "a" * 40,
                                      "origin", ":refs/heads/codex/finished")])
        self.assertEqual(run.journal[run.key]["status"], "deleted")
        self.assertTrue(run.journal[run.key]["remoteRefAbsent"])

    def test_branch_advancing_during_delete_keeps_original_unknown_intent(self):
        run = self.run_workflow(advance_during_push=True)
        self.assertIn("original journal", run.error)
        self.assertEqual(len(run.pushes), 1)
        self.assertEqual(run.journal[run.key]["status"], "unknown")
        self.assertEqual(run.entry["branches"][0]["status"], "unknown")

    def test_new_pull_request_between_inventory_and_delete_prevents_push(self):
        run = self.run_workflow(fresh_pr=True)
        self.assertIsNone(run.error)
        self.assertEqual(run.pushes, [])
        self.assertEqual(run.entry["branches"][0]["reason"], "open-pull-request")

    def test_original_unknown_outcome_with_same_tip_never_replays_push(self):
        key = "He-Jiahui/ZirconEngine:refs/heads/codex/finished"
        run = self.run_workflow(pending={key: dict(status="unknown", oid="a" * 40,
            repository="He-Jiahui/ZirconEngine", branch="codex/finished")})
        self.assertIsNone(run.error)
        self.assertEqual(run.pushes, [])
        self.assertEqual(run.entry["branches"][0]["reason"], "original-deletion-outcome-needs-review")


class DeploymentBoundaryTests(unittest.TestCase):
    def deployment(self, changed_observer=False):
        paths = [str(cleanup.ROOT / relative) for relative in (
            "tools/jenkins-management/remote_branch_cleanup.py",
            "tools/jenkins-management/tests/test_remote_branch_cleanup.py",
            "tools/jenkins-management/checkout_commit.py",
            "tools/jenkins-management/run_checks.py", "tools/dev/local_cargo.py")]
        git = "D:/Tools/development/git/Git/mingw64/bin/git.exe"
        plan = dict(gitExecutable=git, sourcePins={path: cleanup.sha(b"pinned") for path in paths + [git]},
                    driverDigest="d" * 64, runtimeOperationId="original-runtime")
        raw = cleanup.encoded(plan)
        def source(path):
            path = Path(path)
            if path == cleanup.STATE / "plan.json":
                return raw
            if path == cleanup.ROOT / ".jenkins/state/deployment/driver.json":
                return cleanup.encoded(dict(driverDigest=plan["driverDigest"],
                                            runtimeOperationId=plan["runtimeOperationId"]))
            if path == cleanup.ROOT / "tools/dev/local_cargo.py" and changed_observer:
                return b"changed executable"
            if str(path) in plan["sourcePins"] or path == Path(git):
                return b"pinned"
            self.fail("Unexecuted client must not become a cleanup dependency: " + str(path))
        with patch.object(cleanup, "read", side_effect=source):
            return cleanup.verify_plan(cleanup.sha(raw))

    def test_canonical_entry_edits_do_not_invalidate_executed_source_plan(self):
        self.assertEqual(self.deployment()["runtimeOperationId"], "original-runtime")

    def test_actual_native_observer_edit_invalidates_executed_source_plan(self):
        with self.assertRaisesRegex(ValueError, "Pinned cleanup dependency changed"):
            self.deployment(changed_observer=True)

    def runtime(self, *, client_observer=None, changed_seal=False, verifier_error=None):
        observer = "tools/dev/local_cargo.py"
        clients = {observer: client_observer or "b" * 64,
                   "tools/jenkins-management/entry.py": "c" * 64}
        raw = cleanup.encoded(dict(clientSources=clients))
        digest = cleanup.sha(raw)
        capsule = cleanup.ROOT / ".jenkins/runtime/management" / digest
        installed = cleanup.encoded(dict(capsuleSha256=digest, capsule=str(capsule)))
        plan = dict(driverDigest="d" * 64, runtimeOperationId="original-runtime",
                    sourcePins={str(cleanup.ROOT / observer): "b" * 64})
        checks = SimpleNamespace(verify_pins=unittest.mock.Mock(side_effect=verifier_error))

        def source(path):
            if Path(path) == cleanup.ROOT / ".jenkins/state/management/installed.json":
                return installed
            if Path(path) == capsule / "capsule.json":
                return raw + (b" " if changed_seal else b"")
            self.fail("Unexpected executable or mutable client read: " + str(path))

        with patch.object(cleanup, "read", side_effect=source):
            result = cleanup.verify_runtime(checks, plan)
        return result, digest, checks, clients

    def test_unexecuted_entry_upgrade_uses_current_sealed_capsule(self):
        result, digest, checks, clients = self.runtime()
        self.assertEqual(result, digest)
        checks.verify_pins.assert_called_once_with(cleanup.ROOT, "d" * 64, "original-runtime", clients)

    def test_changed_native_observer_remains_a_deployment_failure(self):
        with self.assertRaisesRegex(ValueError, "native observer"):
            self.runtime(client_observer="f" * 64)

    def test_changed_capsule_seal_is_rejected_before_runtime_verifier(self):
        with self.assertRaisesRegex(ValueError, "seal changed"):
            self.runtime(changed_seal=True)

    def test_runtime_or_driver_verifier_failure_is_preserved(self):
        with self.assertRaisesRegex(ValueError, "selected_runtime_pin_mismatch"):
            self.runtime(verifier_error=ValueError("selected_runtime_pin_mismatch"))

    def test_preflight_failure_archives_receipt_without_claiming_native_completion(self):
        writes = {}
        output = cleanup.REPORT / "runs/123"
        def saved(path, value, **kwargs):
            writes[Path(path)] = copy.deepcopy(value)
        with patch.dict(cleanup.os.environ, BUILD_NUMBER="123"), \
                patch.object(cleanup, "save", side_effect=saved), \
                patch.object(cleanup, "read", side_effect=lambda path: cleanup.encoded(writes[Path(path)])), \
                patch("builtins.print"):
            self.assertEqual(cleanup.record_preflight_failure(output, SimpleNamespace(mode="apply"),
                                                              ValueError("dependency changed")), 1)
        summary = writes[output / "summary.json"]
        self.assertFalse(summary["nativeStarted"])
        self.assertFalse(summary["nativeComplete"])
        self.assertFalse(summary["remoteCommandsStarted"])
        self.assertIsNone(summary["nativeExitCode"])
        self.assertEqual(summary["remoteBranchesDeleted"], 0)
        self.assertEqual(writes[output / "receipt.json"]["status"], "preflight-failed")
        response = writes[cleanup.ROOT / ".jenkins/records/remote-branch-cleanup-123.response.json"]
        self.assertFalse(response["triggerPermitted"])
        self.assertFalse(response["deleteAuthorized"])

    def test_main_creates_manifest_before_failing_dependency_validation(self):
        writes = {}
        def saved(path, value, **kwargs):
            writes[Path(path)] = copy.deepcopy(value)
        def source(path):
            if Path(path).name == "remote_branch_cleanup.py":
                return b"trusted source"
            return cleanup.encoded(writes[Path(path)])
        with patch.dict(cleanup.os.environ, JOB_NAME=cleanup.JOB, NODE_NAME="zircon-windows-agent", BUILD_NUMBER="124"), \
                patch.object(cleanup, "read", side_effect=source), \
                patch.object(cleanup, "save", side_effect=saved), \
                patch.object(Path, "mkdir"), \
                patch.object(cleanup, "verify_plan", side_effect=ValueError("dependency changed")), \
                patch("builtins.print"):
            result = cleanup.main(["--mode", "apply", "--self-sha", cleanup.sha(b"trusted source"),
                                   "--plan-sha", "a" * 64])
        self.assertEqual(result, 1)
        output = cleanup.REPORT / "runs/124"
        self.assertEqual(list(writes)[0], output / "artifacts.json")
        self.assertEqual(writes[output / "summary.json"]["status"], "preflight-failed")


if __name__ == "__main__":
    unittest.main()

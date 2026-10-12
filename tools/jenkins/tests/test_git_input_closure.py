"""Real isolated Git trees must retain the inputs used by accepted validation."""
from __future__ import annotations

from dataclasses import replace
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.gitops import CandidatePath, GitRepository, IntegrationPlan, handle
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.state import State


class GitInputClosureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=tempfile.gettempdir())
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "closure@example.invalid")
        self.git("config", "user.name", "Input Closure Test")
        for name, content in {"owned.rs": b"before\n", "dependency.rs": b"dependency 1\n",
                              "deleted.rs": b"delete me\n", "generated/input.rs": b"generated 1\n",
                              "foreign.rs": b"foreign\n"}.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        self.git("add", ".")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD")
        self.state = State(self.root / "state.sqlite")
        self.identity = {"repositoryId": "closure-repo", "sessionId": "closure-session"}
        self.owned = ["owned.rs", "deleted.rs", "new.rs"]
        self.state.authorize_session("closure-repo", "closure-session", "fixture", self.owned,
                                     ["commit"], {"source": "user", "task": "isolated input closure regression"})
        self.object_root = canonical_build_root(r"E:\cargo-targets").namespace()
        self.objects = self.object_root / "inputs/objects"
        self.objects.mkdir(parents=True, exist_ok=True)
        self.manifest = {"schemaVersion": 1, "baseHead": self.base,
                         "entries": [self.present("owned.rs", b"accepted owned\n"),
                                     {"path": "deleted.rs", "status": "deleted"},
                                     self.present("new.rs", b"accepted new\n"),
                                     self.present("dependency.rs", b"dependency 1\n"),
                                     self.present("generated/input.rs", b"generated 1\n")],
                         "declaredDependencies": ["dependency.rs"], "external": []}
        self.key = "closure-session:closure-request:1"

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), *args], check=True,
                              capture_output=True, text=True).stdout.strip()

    def present(self, path, data):
        checksum = hashlib.sha256(data).hexdigest()
        (self.objects / checksum).write_bytes(data)
        # Actual source sealing records S_IMODE, not Git's 100644 type bits.
        return {"path": path, "status": "present", "sha256": checksum,
                "objectDigest": checksum, "mode": 0o644, "size": len(data)}

    def register(self):
        self.source = digest(self.manifest)
        self.coverage = digest({"sourceDigest": self.source, "fixture": "input closure"})
        self.state.put("acceptance", "closure-acceptance", {"status": "accepted", "sourceDigest": self.source,
                                                           "coverageDigest": self.coverage})
        self.sealed = {"repositoryRoot": str(self.root), "repositoryId": "closure-repo",
                       "sourceManifest": self.manifest, "sourceDigest": self.source, "coverageDigest": self.coverage,
                       "objectRoot": str(self.object_root), "buildRoot": r"E:\cargo-targets", "ownedPaths": self.owned}
        self.state.put("sealed_input", self.source, self.sealed)
        self.state.put("workflow", self.key, {"repoRoot": str(self.root), "sourceDigest": self.source,
                        "identity": self.identity, "acceptance": "closure-acceptance", "message": "accepted closure"})

    def commit(self):
        return handle("commit-flow", {"workflowKey": self.key, "authorization": self.identity}, self.state, self.root)

    def advance(self, path, data):
        file = self.root / path
        if data is None:
            self.git("rm", path)
        else:
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(data)
            self.git("add", path)
        self.git("commit", "-qm", "foreign advance")

    def assert_revalidation(self, path):
        head, index = self.git("rev-parse", "HEAD"), (self.root / ".git/index").read_bytes()
        with self.assertRaises(JenkinsError) as raised:
            self.commit()
        self.assertEqual("git_input_closure_mismatch", raised.exception.code)
        self.assertTrue(raised.exception.retryable)
        self.assertEqual(path, raised.exception.details["path"])
        self.assertTrue(raised.exception.details["revalidationRequired"])
        self.assertEqual(head, self.git("rev-parse", "HEAD"))
        self.assertEqual(index, (self.root / ".git/index").read_bytes())
        self.assertNotIn("gitReceipt", self.state.get("workflow", self.key)["payload"])
        self.assertEqual(self.sealed, self.state.get("sealed_input", self.source)["payload"])

    def test_dependency_head_drift_preserves_foreign_head_and_index(self):
        self.register()
        self.advance("dependency.rs", b"dependency 2\n")
        (self.root / "foreign.rs").write_bytes(b"staged foreign\n")
        self.git("add", "foreign.rs")
        self.assert_revalidation("dependency.rs")
        self.assertEqual("dependency 2", self.git("show", "HEAD:dependency.rs"))

    def test_tracked_dependency_deletion_requires_revalidation(self):
        self.register()
        self.advance("dependency.rs", None)
        self.assert_revalidation("dependency.rs")

    def test_deleted_dependency_recreated_in_head_requires_revalidation(self):
        self.advance("dependency.rs", None)
        self.manifest["baseHead"] = self.git("rev-parse", "HEAD")
        self.manifest["entries"][3] = {"path": "dependency.rs", "status": "deleted"}
        self.register()
        self.advance("dependency.rs", b"returned dependency\n")
        self.assert_revalidation("dependency.rs")

    def test_new_unowned_input_missing_from_git_tree_requires_revalidation(self):
        self.manifest["entries"].append(self.present("generated/new-input.rs", b"accepted generated input\n"))
        self.register()
        self.assert_revalidation("generated/new-input.rs")

    def test_generated_input_is_checked_even_when_not_in_declared_dependencies(self):
        self.register()
        self.advance("generated/input.rs", b"generated 2\n")
        self.assert_revalidation("generated/input.rs")

    def test_tracked_new_dependency_uses_the_sealed_bytes_instead_of_head_bytes(self):
        self.manifest["entries"].append(self.present("later-dependency.rs", b"sealed new dependency\n"))
        self.manifest["declaredDependencies"].append("later-dependency.rs")
        self.register()
        self.advance("later-dependency.rs", b"foreign new dependency\n")
        self.assert_revalidation("later-dependency.rs")

    def test_deleted_dependency_already_absent_in_candidate_tree_is_valid(self):
        self.advance("dependency.rs", None)
        self.manifest["baseHead"] = self.git("rev-parse", "HEAD")
        self.manifest["entries"][3] = {"path": "dependency.rs", "status": "deleted"}
        self.register()
        result = self.commit()
        self.assertEqual("ref_updated", result["status"])
        self.assertNotEqual(0, subprocess.run(["git", "-C", str(self.root), "cat-file", "-e", "HEAD:dependency.rs"],
                                              capture_output=True).returncode)

    def test_declared_dependency_without_a_sealed_entry_is_rejected(self):
        self.manifest["entries"].pop(3)
        self.register()
        head, index = self.git("rev-parse", "HEAD"), (self.root / ".git/index").read_bytes()
        with self.assertRaises(JenkinsError) as raised:
            self.commit()
        self.assertEqual("git_input_closure_invalid", raised.exception.code)
        self.assertEqual(head, self.git("rev-parse", "HEAD"))
        self.assertEqual(index, (self.root / ".git/index").read_bytes())

    def test_unrelated_advance_owned_new_and_deleted_paths_preserve_foreign_work(self):
        self.register()
        self.advance("foreign.rs", b"unrelated committed\n")
        (self.root / "foreign.rs").write_bytes(b"foreign staged later\n")
        self.git("add", "foreign.rs")
        (self.root / "owned.rs").write_bytes(b"later worktree edit\n")
        result = self.commit()
        self.assertEqual("ref_updated", result["status"])
        self.assertEqual("accepted owned", self.git("show", "HEAD:owned.rs"))
        self.assertEqual("accepted new", self.git("show", "HEAD:new.rs"))
        self.assertNotEqual(0, subprocess.run(["git", "-C", str(self.root), "cat-file", "-e", "HEAD:deleted.rs"],
                                              capture_output=True).returncode)
        self.assertEqual(b"delete me\n", (self.root / "deleted.rs").read_bytes())
        self.assertFalse((self.root / "new.rs").exists())
        self.assertEqual(b"later worktree edit\n", (self.root / "owned.rs").read_bytes())
        self.assertIn("foreign staged later", self.git("diff", "--cached", "--", "foreign.rs"))
        plan = self.state.get_operation(result["operation_id"])["payload"]["plan"]
        self.assertEqual(self.manifest, plan["input_closures"][0]["manifest"])
        before_index = (self.root / ".git/index").read_bytes()
        reconciled = handle("reconcile-flow", {"workflowKey": self.key, "operationId": result["operation_id"]},
                            self.state, self.root)
        self.assertEqual("complete", reconciled["status"])
        self.assertEqual(before_index, (self.root / ".git/index").read_bytes())

    def test_real_planned_tree_must_match_even_without_head_drift(self):
        self.register()
        wrong_oid = subprocess.run(["git", "hash-object", "-w", "--stdin"], cwd=self.root,
                    input=b"unaccepted dependency\n", capture_output=True, check=True).stdout.decode().strip()
        original = GitRepository._tree
        def bad_tree(repo, head, paths):
            return original(repo, head, (*paths, CandidatePath("dependency.rs", wrong_oid)))
        with patch.object(GitRepository, "_tree", bad_tree):
            self.assert_revalidation("dependency.rs")

    def test_reconcile_rechecks_full_authoritative_closure(self):
        self.register()
        result = self.commit()
        changed = json.loads(json.dumps(self.sealed))
        changed["sourceManifest"]["entries"][3] = self.present("dependency.rs", b"replacement accepted input\n")
        self.state.put("sealed_input", self.source, changed)
        head, index = self.git("rev-parse", "HEAD"), (self.root / ".git/index").read_bytes()
        with self.assertRaises(JenkinsError) as raised:
            handle("reconcile-flow", {"workflowKey": self.key, "operationId": result["operation_id"]}, self.state, self.root)
        self.assertEqual("git_input_closure_binding_mismatch", raised.exception.code)
        self.assertEqual(head, self.git("rev-parse", "HEAD"))
        self.assertEqual(index, (self.root / ".git/index").read_bytes())

    def test_registered_plan_cannot_be_replaced_with_a_different_commit_tree(self):
        self.register()
        # Capture the real prepared journal while leaving the ref unmodified.
        prepared = []
        def leave_prepared(repo, plan, **kwargs):
            prepared.append(plan)
            raise RuntimeError("fixture leaves prepared plan")
        with patch.object(GitRepository, "publish", leave_prepared):
            with self.assertRaises(RuntimeError):
                self.commit()
        plan = prepared[0]
        bad_oid = subprocess.run(["git", "hash-object", "-w", "--stdin"], cwd=self.root,
                    input=b"bad dependency\n", capture_output=True, check=True).stdout.decode().strip()
        repo = GitRepository(self.root, state=self.state)
        tree, _ = repo._tree(plan.planned_commit, (CandidatePath("dependency.rs", bad_oid),))
        commit = repo._git("commit-tree", tree, "-p", plan.expected_old_head, "-m", "unaccepted tree")
        forged = replace(plan, planned_tree=tree, planned_commit=commit)
        head, index = repo.head(), repo.index_digest()
        for target, code in ((repo, "git_plan_mismatch"), (GitRepository(self.root), "git_input_closure_mismatch")):
            with self.subTest(code=code):
                authorization = (self.identity if target.state else
                    self.state.get_operation(plan.operation_id)["payload"]["authorization"])
                for action in (lambda: target.publish(forged, authorization=authorization), lambda: target.reconcile(forged)):
                    with self.assertRaises(JenkinsError) as raised:
                        action()
                    self.assertEqual(code, raised.exception.code)
                    self.assertEqual(head, repo.head())
                    self.assertEqual(index, repo.index_digest())

    def test_owned_only_historical_journal_requires_complete_closure_revalidation(self):
        self.register()
        result = self.commit()
        data = dict(self.state.get_operation(result["operation_id"])["payload"]["plan"])
        data.pop("input_closures")
        data["operation_id"] = "historical-owned-only"
        self.state.put_operation("git_integration", data["operation_id"], {"plan": data, "authorization": self.identity})
        head, index = self.git("rev-parse", "HEAD"), (self.root / ".git/index").read_bytes()
        with self.assertRaises(JenkinsError) as raised:
            handle("reconcile-flow", {"workflowKey": self.key, "operationId": data["operation_id"]}, self.state, self.root)
        self.assertEqual("git_input_closure_binding_mismatch", raised.exception.code)
        self.assertTrue(raised.exception.retryable)
        self.assertEqual(head, self.git("rev-parse", "HEAD"))
        self.assertEqual(index, (self.root / ".git/index").read_bytes())


if __name__ == "__main__":
    unittest.main()

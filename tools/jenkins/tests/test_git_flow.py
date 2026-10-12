from __future__ import annotations

import subprocess
import tempfile
import unittest
import hashlib
from pathlib import Path

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.gitops import handle
from tools.jenkins.state import State
from tools.jenkins.resources import canonical_build_root

REPO_ROOT = Path(__file__).resolve().parents[3]
JENKINS_TMP = Path(tempfile.gettempdir())
BUILD_ROOT = Path(r"E:\cargo-targets")
JENKINS_TMP.mkdir(parents=True, exist_ok=True)


class GitFlowFixtureTests(unittest.TestCase):
    def test_typed_preparation_commit_and_reconcile_use_consumer_scope(self):
        """A shared source row's first author cannot supply this flow's paths."""
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as td:
            root = Path(td)
            def git(*args):
                return subprocess.run(["git", "-C", str(root), *args], check=True,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q")
            git("config", "user.email", "flow@example.invalid")
            git("config", "user.name", "Flow Test")
            for name, content in {"owned.txt": "before", "deleted.txt": "remove",
                                  "dependency.txt": "dependency", "foreign.txt": "foreign"}.items():
                (root / name).write_text(content, encoding="utf-8")
            git("add", ".")
            git("commit", "-qm", "base")
            base_head = git("rev-parse", "HEAD")
            (root / "foreign.txt").write_text("foreign staged", encoding="utf-8")
            git("add", "foreign.txt")
            state = State(root / "state.sqlite")
            repo_id, session = "repo-typed", "session-typed"
            identity = {"repositoryId": repo_id, "sessionId": session}
            state.authorize_session(repo_id, session, "fixture", ["owned.txt", "deleted.txt"],
                                    ["commit"], {"source": "user", "task": "typed Git flow"})
            object_root = canonical_build_root(BUILD_ROOT).namespace()
            objects = object_root / "inputs" / "objects"
            objects.mkdir(parents=True, exist_ok=True)
            data = b"typed consumer output\n"
            checksum = hashlib.sha256(data).hexdigest()
            (objects / checksum).write_bytes(data)
            manifest = {"preparationKind": "current-source", "baseHead": base_head,
                        "declaredDependencies": ["dependency.txt"],
                        "entries": [{"path": "owned.txt", "status": "present", "objectDigest": checksum, "mode": 33188},
                                    {"path": "deleted.txt", "status": "deleted"},
                                    {"path": "dependency.txt", "status": "present",
                                     "objectDigest": hashlib.sha256(b"dependency").hexdigest()}]}
            source, coverage = digest(manifest), digest("typed coverage")
            acceptance_id, preparation_ref = "accept-typed", "prepare-typed"
            state.put("acceptance", acceptance_id, {"status": "accepted", "sourceDigest": source,
                                                     "coverageDigest": coverage})
            shared = {"repositoryRoot": str(root), "repositoryId": repo_id,
                      "objectRoot": str(object_root), "buildRoot": str(BUILD_ROOT),
                      "sourceManifest": manifest, "sourceDigest": source, "coverageDigest": coverage,
                      "ownedPaths": ["dependency.txt"]}
            state.put("sealed_input", source, shared)
            state.put_operation("source_preparation", preparation_ref,
                                {"kind": "current-source", "identity": identity}, status="prepared")
            state.transition_operation(preparation_ref, "prepared", "complete",
                                       {"sourceDigest": source, "ownedPaths": ["owned.txt", "deleted.txt"]})
            key = f"{session}:request-typed:1"
            workflow = {"repoRoot": str(root), "sourceDigest": source, "identity": identity,
                        "sourcePreparationRef": preparation_ref, "patchOperationRefs": [],
                        "ownedPaths": ["owned.txt", "deleted.txt"], "acceptance": acceptance_id,
                        "message": "typed scope commit"}
            state.put("workflow", key, workflow)
            control = {"workflowKey": key, "authorization": identity}
            result = handle("commit-flow", control, state, root)
            self.assertEqual("ref_updated", result["status"])
            self.assertEqual("typed consumer output", git("show", "HEAD:owned.txt"))
            self.assertEqual("dependency", git("show", "HEAD:dependency.txt"))
            self.assertNotEqual(0, subprocess.run(["git", "cat-file", "-e", "HEAD:deleted.txt"],
                                                 cwd=root, capture_output=True).returncode)
            self.assertEqual("remove", (root / "deleted.txt").read_text(encoding="utf-8"))
            self.assertIn("foreign staged", git("diff", "--cached", "--", "foreign.txt"))
            self.assertEqual(shared, state.get("sealed_input", source)["payload"])
            current = state.get("workflow", key)
            for changed in ({"ownedPaths": ["dependency.txt"]}, {"sourcePreparationRef": "foreign-preparation"}):
                state.put("workflow", key, {**current["payload"], **changed})
                before_head, before_index = git("rev-parse", "HEAD"), (root / ".git/index").read_bytes()
                with self.assertRaises(JenkinsError):
                    handle("reconcile-flow", {"workflowKey": key, "operationId": result["operation_id"]}, state, root)
                self.assertEqual(before_head, git("rev-parse", "HEAD"))
                self.assertEqual(before_index, (root / ".git/index").read_bytes())
            state.put("workflow", key, current["payload"])
            reconciled = handle("reconcile-flow", {"workflowKey": key, "operationId": result["operation_id"]}, state, root)
            self.assertEqual("complete", reconciled["status"])
            self.assertEqual(shared, state.get("sealed_input", source)["payload"])

    def test_state_bound_commit_flow_preserves_foreign_index(self):
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as td:
            root = Path(td)
            def git(*args):
                return subprocess.run(["git", "-C", str(root), *args], check=True,
                                      capture_output=True, text=True).stdout.strip()
            git("init", "-q")
            git("config", "user.email", "flow@example.invalid")
            git("config", "user.name", "Flow Test")
            (root / "owned.txt").write_text("before")
            (root / "foreign.txt").write_text("foreign")
            git("add", "."); git("commit", "-qm", "base")
            (root / "foreign.txt").write_text("staged foreign"); git("add", "foreign.txt")
            repo_state = root / "state.sqlite"
            state = State(repo_state)
            repo_id, session, request = "repo-flow", "session-flow", "request-flow"
            state.authorize_session(repo_id, session, "fixture", ["owned.txt"], ["commit"],
                                    {"source": "user", "task": "fixture-flow"})
            coverage = digest("coverage")
            acceptance_id = "accept-flow"
            source_manifest = {"schemaVersion": 1, "baseHead": git("rev-parse", "HEAD"),
                               "entries": [{"path": "owned.txt", "status": "present",
                                            "objectDigest": hashlib.sha256(b"after").hexdigest()}],
                               "declaredDependencies": []}
            source = digest(source_manifest)
            state.put("acceptance", acceptance_id, {"status": "accepted", "sourceDigest": source,
                                                     "coverageDigest": coverage})
            candidate = {
                "candidateId": "candidate-flow", "repositoryRoot": str(root),
                "baseHead": git("rev-parse", "HEAD"), "sourceDigest": source,
                "coverageDigest": coverage, "acceptanceReceipts": [acceptance_id],
                "paths": [{"path": "owned.txt", "blobOid": ""}],
            }
            # Replace the temporary blob with a deterministic sealed object.
            blob = subprocess.run(["git", "hash-object", "-w", "--stdin"], cwd=root,
                                  input=b"after", capture_output=True, check=True, text=False).stdout.decode().strip()
            candidate["paths"][0]["blobOid"] = blob
            candidate["inputManifest"] = source_manifest
            key = f"{session}:{request}:1"
            state.put("sealed_input", source, {"repositoryRoot": str(root), "repositoryId": repo_id,
                                                "sourceManifest": source_manifest, "manifest": candidate, "sourceDigest": source,
                                                "coverageDigest": coverage, "ownedPaths": ["owned.txt"]})
            state.put("workflow", key, {"repoRoot": str(root), "sourceDigest": source,
                                         "identity": {"repositoryId": repo_id, "sessionId": session},
                                         "acceptance": acceptance_id, "message": "flow commit",
                                         "stages": [{"name": "commit", "status": "pending"}]})
            result = handle("commit-flow", {"workflowKey": key, "authorization":
                           {"repositoryId": repo_id, "sessionId": session}}, state, root)
            self.assertEqual("ref_updated", result["status"])
            self.assertIn("staged foreign", git("diff", "--cached", "--", "foreign.txt"))
            self.assertEqual("after", git("show", "HEAD:owned.txt"))
            workflow = state.get("workflow", key)["payload"]
            self.assertEqual("passed", next(item for item in workflow["stages"] if item["name"] == "commit")["status"])
            self.assertEqual(result["commit_sha"], workflow["gitReceipt"]["commit_sha"])
            reconciled = handle("reconcile-flow", {"workflowKey": key, "operationId": result["operation_id"]}, state, root)
            self.assertEqual("complete", reconciled["status"])
            self.assertIn("staged foreign", git("diff", "--cached", "--", "foreign.txt"))
            with self.assertRaises(JenkinsError) as error:
                handle("reconcile-flow", {"workflowKey": key, "operationId": result["operation_id"],
                    "plan": {"operation_id": result["operation_id"], "planned_commit": "forged"}}, state, root)
            self.assertEqual("git_plan_mismatch", error.exception.code)

    def test_sealed_sha256_object_is_imported_as_git_blob_and_dependencies_excluded(self):
        """The State source manifest stores SHA-256 bytes, never Git OIDs."""
        with tempfile.TemporaryDirectory(dir=str(JENKINS_TMP)) as td:
            root = Path(td)
            def git(*args, **kwargs):
                return subprocess.run(["git", "-C", str(root), *args], check=True,
                                      capture_output=True, text=True, **kwargs).stdout.strip()
            git("init", "-q")
            git("config", "user.email", "flow@example.invalid")
            git("config", "user.name", "Flow Test")
            (root / "owned.txt").write_text("before", encoding="utf-8")
            (root / "dependency.txt").write_text("dependency", encoding="utf-8")
            git("add", "."); git("commit", "-qm", "base")
            (root / "foreign.txt").write_text("foreign staged", encoding="utf-8"); git("add", "foreign.txt")
            state = State(root / "state.sqlite")
            repo_id, session, request = "repo-sha", "session-sha", "request-sha"
            state.authorize_session(repo_id, session, "fixture", ["owned.txt"], ["commit"], {"source": "user", "task": "sealed sha flow"})
            object_root = canonical_build_root(BUILD_ROOT).namespace()
            object_root.joinpath("inputs", "objects").mkdir(parents=True, exist_ok=True)
            data = b"after from sealed object\n"
            sha = hashlib.sha256(data).hexdigest()
            (object_root / "inputs" / "objects" / sha).write_bytes(data)
            source_manifest = {"schemaVersion": 1, "baseHead": git("rev-parse", "HEAD"),
                               "entries": [{"path": "owned.txt", "status": "present", "objectDigest": sha, "mode": 33188},
                                           {"path": "dependency.txt", "status": "present",
                                            "objectDigest": hashlib.sha256(b"dependency").hexdigest()}],
                               "declaredDependencies": ["dependency.txt"]}
            source = digest(source_manifest); coverage = digest("coverage-sha")
            acceptance_id = "accept-sha"
            state.put("acceptance", acceptance_id, {"status": "accepted", "sourceDigest": source, "coverageDigest": coverage})
            state.put("sealed_input", source, {"repositoryRoot": str(root), "repositoryId": repo_id,
                                                "objectRoot": str(object_root), "buildRoot": str(BUILD_ROOT),
                                                "sourceManifest": source_manifest, "sourceDigest": source,
                                                "coverageDigest": coverage, "ownedPaths": ["owned.txt"]})
            key = f"{session}:{request}:1"
            state.put("workflow", key, {"repoRoot": str(root), "sourceDigest": source,
                                         "identity": {"repositoryId": repo_id, "sessionId": session,
                                                      "requestId": request, "attempt": 1},
                                         "acceptance": acceptance_id, "message": "sealed sha flow"})
            result = handle("commit-flow", {"workflowKey": key,
                            "authorization": {"repositoryId": repo_id, "sessionId": session}}, state, root)
            self.assertEqual("ref_updated", result["status"])
            self.assertEqual("after from sealed object", git("show", "HEAD:owned.txt"))
            self.assertEqual("dependency", git("show", "HEAD:dependency.txt"))
            self.assertTrue(git("diff", "--cached", "--", "foreign.txt"))


if __name__ == "__main__":
    unittest.main()

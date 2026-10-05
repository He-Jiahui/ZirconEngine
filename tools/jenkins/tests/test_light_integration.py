"""Real M3 light-flow acceptance on an isolated Git fixture.

The fixture is deliberately outside the product checkout.  Compiler input
objects use an approved drive-root build namespace while Jenkins Home remains
untouched; the test therefore exercises the same path gates as production.
"""
from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
import unittest
import uuid
from pathlib import Path

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.gitops import CandidatePath, GitRepository
from tools.jenkins.source import handle as source_handle
from tools.jenkins.state import State
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.validation.receipts import validate_acceptance_receipt
from tools.jenkins.workflow.handler import handle

REPO_ROOT = Path(__file__).resolve().parents[3]
JENKINS_TMP = REPO_ROOT / ".jenkins" / "tmp"
BUILD_ROOT = Path(r"E:\cargo-targets")
JENKINS_TMP.mkdir(parents=True, exist_ok=True)


class RealLightAcceptanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(dir=str(JENKINS_TMP))
        self.root = Path(self.temp.name).resolve()
        (self.root / "src").mkdir(parents=True)
        (self.root / "src/lib.rs").write_text("fn main() {}\n", encoding="utf-8")
        self.build_root = BUILD_ROOT
        # The source authority currently derives the canonical object root
        # from buildRoot. Keep the fixture content unique by using a unique
        # Git source and remove only hashes created by this test.
        self.object_root = canonical_build_root(self.build_root).namespace()
        objects = self.object_root / "inputs" / "objects"
        objects.mkdir(parents=True, exist_ok=True)
        self._git("init", "-q")
        self._git("add", "src/lib.rs")
        self._git("-c", "user.email=jenkins-fixture@example.invalid", "-c", "user.name=jenkins-fixture", "commit", "-qm", "base")
        self.state = State(self.root / "state.sqlite3")
        self.repo_id = "fixture-" + self.root.name
        self.session_id = "m3-light-" + self.root.name
        self.owner = "attempt-" + self.root.name
        self.state.authorize_session(self.repo_id, self.session_id, "fixture-user",
                                     ["src/lib.rs", "foreign.txt"],
                                     ["implementation", "validation", "commit"],
                                     {"source": "user", "task": "m3-light-fixture"})

    def tearDown(self) -> None:
        # Content addressed inputs and manifest references are shared immutable
        # data. Keep them for normal GC ownership; only remove the unique Git
        # fixture under .jenkins/tmp.
        self.temp.cleanup()

    def _git(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["git", *args], cwd=self.root, check=check,
                              capture_output=True, text=True)

    def _identity(self, source_digest: str, coverage_digest: str | None = None) -> dict:
        return {"sessionId": self.session_id, "requestId": "req-" + self.root.name,
                "attemptId": self.owner, "generation": 1,
                "sourceInputDigest": source_digest,
                "coverageDigest": coverage_digest or "b" * 64,
                "stageImplementationDigest": "c" * 64,
                "driverDigest": "c" * 64}

    def test_comments_only_flow_accepts_and_commits_fixture(self) -> None:
        """Claim/apply/seal, light receipts, acceptance, and Git integration."""
        original = (self.root / "src/lib.rs").read_bytes()
        before = hashlib.sha256(original).hexdigest()
        claim = source_handle("claim", {
            "repositoryId": self.repo_id, "sessionId": self.session_id,
            "owner": self.owner, "ownedPaths": ["src/lib.rs"],
            "beforeHashes": {"src/lib.rs": before}}, self.state, self.root)
        operation_id = "patch-" + self.root.name
        changed = b"fn main() {}\n// fixture comment\n"
        source_handle("apply", {
            "repositoryId": self.repo_id, "sessionId": self.session_id,
            "owner": self.owner, "patch": {"src/lib.rs": changed.decode()},
            "beforeHashes": {"src/lib.rs": before}, "operationId": operation_id},
            self.state, self.root)
        # Reload the claim after apply: its before hash is advanced to the
        # operation's owned post-image, which is what sealing verifies.
        claims = source_handle("claim", {
            "repositoryId": self.repo_id, "sessionId": self.session_id,
            "owner": self.owner, "ownedPaths": ["src/lib.rs"]}, self.state, self.root)["claims"]
        base_head = self._git("rev-parse", "HEAD").stdout.strip()
        sealed = source_handle("seal", {
            "repositoryId": self.repo_id, "sessionId": self.session_id,
            "owner": self.owner, "claims": claims, "buildRoot": str(self.build_root),
            "objectRoot": str(self.object_root), "baseHead": base_head,
            "coverage": {"sourceManifestRef": "fixture", "preparationKey": "m3-light"},
            "patchOperationRefs": [operation_id]}, self.state, self.root)
        source_digest = sealed["sourceDigest"]
        identity = self._identity(source_digest, self.state.get("sealed_input", source_digest)["payload"]["coverageDigest"])
        # The State record is written by source sealing; no caller-created
        # receipt or source boolean is used below.
        workflow_payload = {"identity": identity, "sealedInputRef": source_digest,
                            "buildRoot": str(self.build_root)}
        handle("register-flow", workflow_payload, self.state, self.root)
        composed = handle("compose-flow", {"identity": identity}, self.state, self.root)
        self.assertEqual("comments_only", composed["template"])
        flow_key = f"{identity['sessionId']}:{identity['requestId']}:{identity['attemptId']}"
        flow_record = self.state.get("workflow", flow_key)["payload"]
        rs_entries = [entry for entry in flow_record["sourceManifest"]["entries"] if entry.get("path") == "src/lib.rs"]
        self.assertTrue(rs_entries, flow_record["sourceManifest"])
        self.assertEqual("present", rs_entries[0].get("status"), rs_entries[0])
        self.assertTrue((Path(flow_record["objectRoot"]) / "inputs" / "objects" / rs_entries[0]["objectDigest"]).is_file(), flow_record)
        handle("run-stage", {"identity": identity, "stage": "patch",
                              "patchJournal": operation_id}, self.state, self.root)
        handle("run-stage", {"identity": identity, "stage": "syntax"}, self.state, self.root)
        handle("run-stage", {"identity": identity, "stage": "format"}, self.state, self.root)
        accepted = handle("run-stage", {"identity": identity, "stage": "acceptance"}, self.state, self.root)
        self.assertEqual("accepted", accepted["status"])
        acceptance = accepted["receipt"]
        validate_acceptance_receipt(acceptance)

        # Stage an unrelated file. Git integration must preserve this foreign
        # index entry while updating only the owned source path.
        (self.root / "foreign.txt").write_text("foreign staged edit\n", encoding="utf-8")
        self._git("add", "foreign.txt")
        git_result = __import__("tools.jenkins.gitops", fromlist=["handle"]).handle(
            "commit-flow", {"workflowKey": flow_key,
                             "authorization": ""},
            self.state, self.root)
        self.assertEqual("ref_updated", git_result["status"])
        self.assertEqual("foreign staged edit\n", (self.root / "foreign.txt").read_text(encoding="utf-8"))
        self.assertTrue(self._git("diff", "--cached", "--", "foreign.txt").stdout)
        self.assertEqual(changed, (self.root / "src/lib.rs").read_bytes())
        self.assertFalse((self.root / "target").exists())

    def test_doc_comment_and_code_change_are_not_comments_only(self) -> None:
        from tools.jenkins.workflow.classification import classify_rust_source
        self.assertNotEqual("comments_only", classify_rust_source("/// docs\nfn main() {}\n")["kind"])
        self.assertNotEqual("comments_only", classify_rust_source("fn main() { println!(\"changed\"); }\n")["kind"])

    def test_foreign_edit_blocks_compensation(self) -> None:
        from tools.jenkins.gitops import CandidateManifest
        repo = GitRepository(self.root, state=self.state)
        before = self._git("rev-parse", "HEAD").stdout.strip()
        old = (self.root / "src/lib.rs").read_bytes()
        candidate = CandidatePath("src/lib.rs", None, before_digest=hashlib.sha256(old).hexdigest())
        (self.root / "src/lib.rs").write_bytes(b"foreign replacement\n")
        with self.assertRaises(JenkinsError) as error:
            repo.compensate_uncommitted((candidate,), before_head=before,
                                        after_hashes={"src/lib.rs": hashlib.sha256(b"owned\n").hexdigest()},
                                        terminal_proof={"terminated": True})
        self.assertEqual("foreign_edit_during_compensation", error.exception.code)


if __name__ == "__main__":
    unittest.main()

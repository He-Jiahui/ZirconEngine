from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.gitops import GitRepository


class GitOpsTests(unittest.TestCase):
    def setUp(self):
        self.parent = Path(__file__).resolve().parents[3] / ".jenkins/state/test-repos"
        self.parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=self.parent)
        self.root = Path(self.temp.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        for key, value in (("user.email", "jenkins-test@example.invalid"), ("user.name", "Jenkins Test")):
            subprocess.run(["git", "-C", str(self.root), "config", key, value], check=True)
        (self.root / "owned.txt").write_text("before", encoding="utf-8")
        (self.root / "foreign.txt").write_text("foreign", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.root), "commit", "-qm", "initial"], check=True)
        self.repo = GitRepository(self.root)

    def tearDown(self):
        self.temp.cleanup()

    def candidate(self, content=b"after"):
        return self.repo.seal_candidate(
            {"owned.txt": content}, source_digest=digest("source"),
            coverage_digest=digest("coverage"), acceptance_receipts=("accept-1",),
        )

    def test_sealed_candidate_publish_and_reconcile(self):
        candidate = self.candidate()
        plan = self.repo.plan((candidate,), authorization={"allowCommit": True}, message="owned change")
        receipt = self.repo.publish(plan, authorization={"allowCommit": True})
        self.assertEqual("ref_updated", receipt.status)
        self.assertEqual("complete", self.repo.reconcile(plan).status)
        self.assertEqual("after", subprocess.run(["git", "-C", str(self.root), "show", "HEAD:owned.txt"],
                                                  check=True, capture_output=True, text=True).stdout)

    def test_duplicate_publish_is_cas_failure_and_reconcile_finds_one_commit(self):
        candidate = self.candidate()
        plan = self.repo.plan((candidate,), authorization={"allowCommit": True}, message="owned change")
        self.repo.publish(plan, authorization={"allowCommit": True})
        with self.assertRaises(JenkinsError) as raised:
            self.repo.publish(plan, authorization={"allowCommit": True})
        self.assertEqual("head_cas_failed", raised.exception.code)
        self.assertEqual("complete", self.repo.reconcile(plan).status)

    def test_head_cas_rejects_foreign_commit(self):
        candidate = self.candidate()
        plan = self.repo.plan((candidate,), authorization={"allowCommit": True}, message="owned change")
        (self.root / "foreign.txt").write_text("later", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.root), "add", "foreign.txt"], check=True)
        subprocess.run(["git", "-C", str(self.root), "commit", "-qm", "foreign"], check=True)
        with self.assertRaises(JenkinsError) as raised:
            self.repo.publish(plan, authorization={"allowCommit": True})
        self.assertEqual("head_cas_failed", raised.exception.code)

    def test_foreign_staged_entry_is_rejected(self):
        (self.root / "owned.txt").write_text("foreign staged", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.root), "add", "owned.txt"], check=True)
        candidate = self.candidate()
        with self.assertRaises(JenkinsError) as raised:
            self.repo.plan((candidate,), authorization={"allowCommit": True}, message="blocked")
        self.assertEqual("foreign_index_entry", raised.exception.code)

    def test_unrelated_foreign_staging_survives_owned_publish(self):
        (self.root / "foreign.txt").write_text("staged later", encoding="utf-8")
        subprocess.run(["git", "-C", str(self.root), "add", "foreign.txt"], check=True)
        candidate = self.candidate()
        plan = self.repo.plan((candidate,), authorization={"allowCommit": True}, message="owned change")
        self.repo.publish(plan, authorization={"allowCommit": True})
        staged = subprocess.run(["git", "-C", str(self.root), "diff", "--cached", "--", "foreign.txt"],
                                check=True, capture_output=True, text=True).stdout
        self.assertIn("staged later", staged)

    def test_compensation_requires_terminal_and_preserves_later_edit(self):
        candidate = self.candidate()
        (self.root / "owned.txt").write_text("failed", encoding="utf-8")
        failed_hash = file_digest(self.root / "owned.txt")
        with self.assertRaises(JenkinsError) as raised:
            self.repo.compensate_uncommitted(candidate.paths, before_head=candidate.base_head,
                                             after_hashes={"owned.txt": failed_hash}, terminal_proof={})
        self.assertEqual("native_not_terminal", raised.exception.code)
        (self.root / "owned.txt").write_text("later edit", encoding="utf-8")
        with self.assertRaises(JenkinsError) as raised:
            self.repo.compensate_uncommitted(candidate.paths, before_head=candidate.base_head,
                                             after_hashes={"owned.txt": failed_hash},
                                             terminal_proof={"terminated": True})
        self.assertEqual("foreign_edit_during_compensation", raised.exception.code)


if __name__ == "__main__":
    unittest.main()

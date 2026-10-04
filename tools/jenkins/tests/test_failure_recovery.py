import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.source import apply_owned_patch
from tools.jenkins.state import State
from tools.jenkins.workflow.recovery import recover_flow


class FailureRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = self.root / "object-store"
        self.store.mkdir()
        self.source = self.root / "src" / "lib.rs"
        self.source.parent.mkdir()
        # The original is intentionally uncommitted, rather than a Git blob.
        self.original = b"fn value() -> u32 { 2 } // uncommitted change\n"
        self.source.write_bytes(self.original)
        self.state = State(self.root / "state.sqlite3")
        self.state.authorize_session("repo", "session", "user", ["src"],
            ["implementation", "validation"], {"source": "user", "task": "isolated recovery acceptance"})
        self.key = "session:request:attempt"
        self.identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                         "attemptId": "attempt", "generation": 1}
        self.auth = {"repositoryId": "repo", "sessionId": "session"}
        self.hash = hashlib.sha256(self.original).hexdigest()

    def tearDown(self):
        self.temp.cleanup()

    def apply(self):
        with patch("tools.jenkins.source._approved_object_root", return_value=self.store):
            return apply_owned_patch(self.state, self.root, "attempt", {"src/lib.rs": "fn value() -> u32 { 9 }\n"},
                {"src/lib.rs": self.hash}, operation_id="patch", authorization=self.auth)

    def workflow(self, **extra):
        self.state.put("workflow", self.key, {"identity": self.identity, "repositoryId": "repo",
            "executionId": "not-launched", "repoRoot": str(self.root), "sourceDigest": "source",
            "sourceManifest": {}, "patchOperationRefs": ["patch"], "buildRoot": "D:/cargo-targets",
            "status": "failed", "stages": [{"name": "acceptance", "status": "pending"}], **extra})

    def recover(self):
        with patch("tools.jenkins.workflow.recovery.canonical_build_root", return_value=object()), \
             patch("tools.jenkins.workflow.recovery.physical_path_under", side_effect=lambda root, path: Path(path)):
            return recover_flow(self.state, self.root, self.key, error="syntax failed")

    def test_uncommitted_before_bytes_are_restored_and_replay_is_idempotent(self):
        self.apply()
        self.workflow()
        self.assertEqual("complete", self.recover()["status"])
        self.assertEqual(self.original, self.source.read_bytes())
        self.assertEqual([], self.state.list("path_claims"))
        self.source.write_bytes(b"later foreign editing\n")
        self.assertEqual("complete", self.recover()["status"])
        self.assertEqual(b"later foreign editing\n", self.source.read_bytes())

    def test_foreign_edit_is_preserved_and_compensation_is_blocked(self):
        self.apply()
        self.workflow()
        self.source.write_bytes(b"foreign\n")
        result = self.recover()
        self.assertEqual("blocked", result["status"])
        self.assertEqual("foreign_edit_during_compensation", result["recovery"]["reasonCode"])
        self.assertEqual(b"foreign\n", self.source.read_bytes())

    def test_unproven_native_process_blocks_compensation(self):
        self.apply()
        self.workflow()
        self.state.put("native_job", "job", {"executionId": "not-launched", "status": "running"})
        self.assertEqual("reconciling", self.recover()["status"])
        self.assertNotEqual(self.original, self.source.read_bytes())

    def test_running_orphan_recovers_only_after_real_later_boot(self):
        self.apply()
        self.workflow()
        eid = 'not-launched'
        self.state.put('resource_policy', 'default', {'cpuBudget': 1, 'memoryBudget': 1, 'diskBudget': 1})
        self.state.put('execution', eid, {'status': 'running', 'refs': 0, 'consumers': []})
        self.state.put('execution_host', eid, {'status': 'running', 'hostBirthToken': '100'})
        self.state.put('execution_launch', eid, {'status': 'started', 'launcherBirthToken': '100'})
        self.state.put('native_job', 'job', {'executionId': eid, 'owner': eid, 'birthToken': '150', 'status': 'running'})
        with patch('tools.jenkins.processes.boot.windows_boot_filetime', return_value=50):
            result = self.recover()
            self.assertEqual('reconciling', result['status'])
            self.assertNotEqual(self.original, self.source.read_bytes())
            self.assertEqual([], self.state.list('boot_termination'))
        with patch('tools.jenkins.processes.boot.windows_boot_filetime', return_value=200):
            result = self.recover()
            self.assertEqual('complete', result['status'])
            self.assertEqual(self.original, self.source.read_bytes())
            self.assertEqual('failed', self.state.get('execution', eid)['payload']['status'])
            self.assertFalse(self.state.get('boot_termination', eid)['payload']['artifactPublicationAllowed'])
            self.assertIsNone(self.state.get('native_job', 'job')['payload'].get('completeProof'))
            self.assertEqual([], self.state.list('acceptance'))

    def test_accepted_source_is_preserved_after_later_stage_failure(self):
        self.apply()
        self.workflow(status="accepted", gitCommitSha="committed")
        self.assertEqual("accepted-source-preserved", self.recover()["compensation"])
        self.assertNotEqual(self.original, self.source.read_bytes())

    def test_corrupt_original_object_is_rejected(self):
        self.apply()
        self.workflow()
        (self.store / "inputs" / "objects" / self.hash).write_bytes(b"corrupt")
        self.assertEqual("blocked", self.recover()["status"])
        self.assertNotEqual(self.original, self.source.read_bytes())

    def test_duplicate_patch_does_not_overwrite_later_edit(self):
        first = self.apply()
        self.source.write_bytes(b"foreign\n")
        self.assertEqual(first, self.apply())
        self.assertEqual(b"foreign\n", self.source.read_bytes())


if __name__ == "__main__":
    unittest.main()

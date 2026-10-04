import tempfile
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError, canonical_json, digest
from tools.jenkins.state import State
from tools.jenkins.workflow.groups import SessionGroups
from tools.jenkins.source import register_sealed_consumer


class MultiSessionGroupsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.state = State(Path(self.tmp.name) / "state.sqlite3")
        self.groups = SessionGroups(self.state)
        self.identity = {
            "repositoryId": "repo-1", "baseHead": "head-1",
            "toolchainDigest": "tool-1", "lockDigest": "lock-1",
            "compilerIdentity": "msvc-1", "wrapperDigest": "wrap-1",
            "profile": "dev", "features": digest([]), "buildRoot": "D:/cargo-targets",
        }

    def tearDown(self):
        self.tmp.cleanup()

    def add(self, sid, paths, *, source="source-1", generation=1, current=None):
        objects = Path(self.tmp.name) / "objects" / "inputs" / "objects"
        objects.mkdir(parents=True, exist_ok=True)
        for p in paths:
            (objects / digest(source + p)).write_bytes(canonical_json(source + p))
        manifest = {"schemaVersion": 1, "baseHead": self.identity["baseHead"],
                    "entries": [{"path": p, "status": "present", "sha256": digest(source + p),
                                 "objectDigest": digest(source + p)} for p in paths],
                    "external": [], "declaredDependencies": []}
        source = digest(manifest)
        claims = [{"path": p, "beforeHash": "before:" + p,
                   **({"currentBeforeHash": current} if current else {})} for p in paths]
        sealed = {"status": "sealed", "sourceDigest": source,
                  "coverage": {"sourceDigest": source, "status": "accepted"},
                  "coverageDigest": "coverage-" + source, "claims": claims,
                  "ownedPaths": list(paths), "sourceManifest": manifest, "manifest": manifest,
                  "repositoryRoot": self.tmp.name, "objectRoot": self.tmp.name + "/objects", **self.identity}
        self.state.put("sealed_input", source, sealed)
        register_sealed_consumer(self.state, repository_id_value="repo-1", session_id=sid,
                                owner="owner-" + sid, source_digest=source, manifest_digest=source)
        return {**self.identity, "sessionId": sid, "requestId": "req-" + sid,
                "attemptId": "owner-" + sid,
                "generation": generation, "sealedInputRef": source, "paths": list(paths)}

    def test_same_input_attaches_three_consumers_and_cancel_is_projection_only(self):
        sessions = [self.add("s1", ["src/a.rs"]), self.add("s2", ["src/a.rs"]), self.add("s3", ["src/a.rs"])]
        # Same sealed input may be consumed independently; overlapping paths are
        # valid only when they are the same immutable input.
        group = self.groups.prepare(sessions)
        self.assertEqual(group.status, "ready")
        cancelled = self.groups.cancel(group.group_id, "s2", 1)
        self.assertEqual(cancelled.status, "ready")
        self.assertEqual(len(cancelled.sessions), 3)
        self.assertNotIn("s2", self.state.get("request_result", "never") or {})
        with self.assertRaises(JenkinsError) as error:
            self.groups.cancel(group.group_id, "s2", 0)
        self.assertEqual(error.exception.code, "group_generation_mismatch")

    def test_disjoint_joint_group_stays_pending_until_actual_joint_coverage(self):
        first = self.add("s1", ["src/a.rs"], source="source-a")
        second = self.add("s2", ["src/b.rs"], source="source-b")
        pending = self.groups.prepare([first, second], mode="joint")
        self.assertEqual(pending.status, "pending-joint-validation")
        with self.assertRaises(JenkinsError) as error:
            self.groups.prepare([first, second], mode="joint",
                joint_validation={"status": "accepted", "coverageDigest": "joint-1"})
        self.assertEqual("joint_acceptance_untrusted", error.exception.code)
        joint = self.state.get("joint_input", pending.joint_manifest_digest)["payload"]
        self.state.put("workflow", "joint-flow", {"status": "accepted", "sourceDigest": joint["sourceDigest"]})
        self.state.put("acceptance", "joint-receipt", {"status": "accepted", "flowId": "joint-flow",
                       "sourceDigest": joint["sourceDigest"], "coverageDigest": joint["coverageDigest"]})
        ready = self.groups.prepare([first, second], mode="joint", joint_validation={"receiptRef": "joint-receipt"})
        self.assertEqual(ready.status, "ready")
        self.assertEqual(pending.group_id, ready.group_id)
        self.assertIsNotNone(ready.joint_manifest_digest)

    def test_path_conflict_and_foreign_edit_are_rejected(self):
        first = self.add("s1", ["src/a.rs"], source="source-a")
        second = self.add("s2", ["src/a.rs"], source="source-b")
        with self.assertRaises(JenkinsError) as error:
            self.groups.prepare([first, second], mode="joint")
        self.assertEqual(error.exception.code, "group_path_conflict")
        foreign = self.add("s3", ["src/c.rs"], source="source-c", current="foreign")
        with self.assertRaises(JenkinsError) as error:
            self.groups.prepare([foreign])
        self.assertEqual(error.exception.code, "foreign_edit_detected")

    def test_late_generation_cancel_does_not_mutate_group(self):
        session = self.add("s1", ["src/a.rs"])
        group = self.groups.prepare([session])
        with self.assertRaises(JenkinsError):
            self.groups.cancel(group.group_id, "s1", 2)
        observed = self.groups.observe(group.group_id)
        self.assertEqual(observed.status, "ready")

    def test_caller_cannot_replace_source_paths_repository_or_build_root(self):
        session = self.add("s1", ["src/a.rs"])
        for field, value in [("paths", ["src/foreign.rs"]), ("repositoryId", "foreign"),
                             ("buildRoot", "F:/cargo-targets")]:
            with self.subTest(field=field), self.assertRaises(JenkinsError):
                self.groups.prepare([{**session, field: value}])

    def test_unregistered_consumer_cannot_claim_an_authored_snapshot(self):
        session = self.add("s1", ["src/a.rs"])
        with self.assertRaises(JenkinsError):
            self.groups.prepare([{**session, "sessionId": "intruder", "attemptId": "intruder"}])

    def test_joint_source_identity_survives_preparation_canonicalization(self):
        from tools.jenkins.workflow.pools import _manifest
        from tools.jenkins.source import compute_source_digest
        first = self.add("s1", ["src/a.rs"], source="a")
        second = self.add("s2", ["Cargo.toml"], source="b")
        group = self.groups.prepare([first, second], mode="joint")
        joint = self.state.get("joint_input", group.joint_manifest_digest)["payload"]
        normalized, _ = _manifest(joint["sourceManifest"], Path(joint["objectRoot"]))
        self.assertEqual(joint["sourceDigest"], compute_source_digest(normalized))


if __name__ == "__main__":
    unittest.main()

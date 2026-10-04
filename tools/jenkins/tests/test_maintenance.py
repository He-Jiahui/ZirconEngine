from pathlib import Path
import hashlib
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.artifacts import ArtifactRegistry, ArtifactStore
from tools.jenkins.maintenance import garbage_collect, inventory
from tools.jenkins.state import State


class MaintenanceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.state = State(root / "state.sqlite3")
        self.store = ArtifactStore(root / "artifacts")
        self.repo, self.session = "repo", "session"
        self.state.authorize_session(self.repo, self.session, "tester", ["src"], ["gc"], {"source": "user", "task": "maintenance test"})

    def tearDown(self):
        self.tmp.cleanup()

    def _payload(self, op):
        return {"identity": {"repositoryId": self.repo, "sessionId": self.session}, "operationId": op,
                "buildRoot": r"D:\cargo-targets", "ownedPaths": ["src"],
                "cleanupScope": {"digests": [], "maxBytes": 1024, "maxObjects": 10}}

    def test_inventory_records_actual_bytes_and_unknown_objects(self):
        src = Path(self.tmp.name) / "payload.bin"; src.write_bytes(b"payload")
        manifest = self.store.publish(src, execution_id="e")
        (self.store.objects / ("f" * 64)).write_bytes(b"unknown")
        with patch("tools.jenkins.maintenance._store", return_value=(Path(r"D:\cargo-targets"), self.store)):
            result = inventory(self._payload("inv"), self.state, Path.cwd())
        self.assertEqual(result["totalBytes"], len(b"payload") + len(b"unknown"))
        self.assertIn("f" * 64, result["unknownDigests"])

    def test_gc_detaches_and_reports_reclaimed_bytes_but_protects_live_and_unknown(self):
        src = Path(self.tmp.name) / "payload.bin"; src.write_bytes(b"payload")
        live = self.store.publish(src, execution_id="live")
        dead_src = Path(self.tmp.name) / "dead.bin"; dead_src.write_bytes(b"dead")
        dead = self.store.publish(dead_src, execution_id="dead")
        registry = ArtifactRegistry(self.state, self.store)
        ref = registry.attach(live.digest, "consumer")
        dead_ref = registry.attach(dead.digest, "dead-consumer")
        self.assertTrue(registry.detach(dead_ref["key"]))
        (self.store.objects / ("f" * 64)).write_bytes(b"unknown")
        with patch("tools.jenkins.maintenance._store", return_value=(Path(r"D:\cargo-targets"), self.store)), patch("tools.jenkins.maintenance.physical_path_under", return_value=self.store.objects / dead.digest):
            payload = self._payload("gc")
            payload["cleanupScope"]["digests"] = [dead.digest]
            result = garbage_collect(payload, self.state, Path.cwd())
        self.assertEqual(result["reclaimedBytes"], len(b"dead"))
        self.assertTrue((self.store.objects / live.digest).exists())
        self.assertFalse((self.store.objects / dead.digest).exists())
        self.assertTrue((self.store.objects / ("f" * 64)).exists())
        self.assertEqual(self.state.get("maintenance_gc", "gc")["payload"]["reclaimedBytes"], len(b"dead"))
        self.assertFalse(registry.detach(ref["key"]) is False)

    def test_gc_requires_scoped_authorization(self):
        self.state = State(Path(self.tmp.name) / "other.sqlite3")
        with patch("tools.jenkins.maintenance._store", return_value=(Path(r"D:\cargo-targets"), self.store)):
            with self.assertRaises(Exception) as error:
                garbage_collect(self._payload("unauthorized"), self.state, Path.cwd())
        self.assertEqual(getattr(error.exception, "code", ""), "action_not_authorized")


if __name__ == "__main__":
    unittest.main()

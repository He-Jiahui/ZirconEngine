"""M1 support boundary integration checks.

These tests deliberately compose the real State, resource, process, artifact,
and path adapters.  They use tiny Python subprocesses and temporary metadata;
no compiler or service fixture is involved.
"""
from __future__ import annotations

import hashlib
import os
import subprocess
import sys
import tempfile
import unittest
import threading
import json
from pathlib import Path

from tools.jenkins.artifacts import ArtifactRegistry, ArtifactStore
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.processes import NativeJob, ProcessRegistry, ProcessIdentity, identity_matches
from tools.jenkins.resources import Capacity, ResourceManager, canonical_build_root
from tools.jenkins.state import State


class M1BoundaryIntegrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="m1-boundary-", dir=str(Path(".Jenkins/tmp")))
        self.root = Path(self.tmp.name)
        self.state = State(self.root / "state.sqlite3")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_resource_request_is_idempotent_and_writer_is_unique(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(2, 1024, 48 * 1024**3))
        first = manager.admit("owner", Capacity(1, 10, 1), inventory_generation="gen", writer_key="pool", request_id="same")
        again = manager.admit("owner", Capacity(1, 10, 1), inventory_generation="gen", writer_key="pool", request_id="same")
        self.assertEqual(first["key"], again["key"])
        with self.assertRaises(JenkinsError) as conflict:
            manager.admit("other", Capacity(1, 10, 1), inventory_generation="gen", writer_key="pool", request_id="different")
        self.assertEqual(conflict.exception.code, "pool_writer_active")

    def test_stale_inventory_and_over_capacity_do_not_leave_reservations(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 8 * 1024**3))
        manager.publish_inventory("tiny", Capacity(1, 1, 36 * 1024**3))
        with self.assertRaises(JenkinsError) as stale:
            manager.admit("owner", Capacity(1, 1, 1), inventory_generation="missing")
        self.assertEqual(stale.exception.code, "storage_snapshot_stale")
        self.assertIsNone(manager.admit("owner", Capacity(2, 2, 1), inventory_generation="tiny", request_id="too-big"))
        self.assertEqual(self.state.list("resource_reservation"), [])

    def test_changed_idempotent_request_payload_is_rejected(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(2, 1024, 48 * 1024**3))
        first = manager.admit("owner", Capacity(1, 10, 1), inventory_generation="gen", request_id="same")
        with self.assertRaises(JenkinsError) as error:
            manager.admit("owner", Capacity(1, 11, 1), inventory_generation="gen", request_id="same")
        self.assertEqual(error.exception.code, "request_payload_mismatch")

    def test_three_sqlite_connections_compete_atomically_for_one_writer(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(2, 1024, 48 * 1024**3))
        results, errors = [], []
        def attempt(index: int) -> None:
            try:
                local = ResourceManager(State(self.root / f"shared-{index}.sqlite3"), Capacity(2, 1024, 48 * 1024**3))
                # All contenders must use the same authoritative DB.
                local.state = self.state
                results.append(local.admit(f"owner-{index}", Capacity(1, 10, 1), inventory_generation="gen", writer_key="pool", request_id=f"r-{index}"))
            except Exception as exc:
                errors.append(exc)
        threads = [threading.Thread(target=attempt, args=(i,)) for i in range(3)]
        for thread in threads: thread.start()
        for thread in threads: thread.join()
        self.assertEqual(sum(item is not None for item in results), 1)
        self.assertLessEqual(len(self.state.list("resource_reservation")), 1)

    def test_three_independent_python_sessions_compete_for_one_writer(self):
        manager = ResourceManager(self.state, Capacity(3, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(3, 1024, 48 * 1024**3))
        script = (
            "import json,sys; from tools.jenkins.state import State; "
            "from tools.jenkins.resources import ResourceManager,Capacity; "
            "s=State(sys.argv[1]); m=ResourceManager(s,Capacity(3,1024,48*1024**3)); "
            "\ntry: r=m.admit(sys.argv[2],Capacity(1,10,1),inventory_generation='gen',writer_key='pool',request_id=sys.argv[2]); ok=r is not None\nexcept Exception: ok=False\nprint(json.dumps({'ok':ok}))"
        )
        # Child sessions need the same DB and inventory, so copy the seed DB first.
        import shutil
        db = self.root / "shared.sqlite3"
        shutil.copy2(self.root / "state.sqlite3", db)
        child_env = dict(os.environ, PYTHONPATH=str(Path.cwd()))
        children = [subprocess.Popen([sys.executable, "-B", "-c", script, str(db), f"child-{i}"], cwd=Path.cwd(), env=child_env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) for i in range(3)]
        outputs = []
        for child in children:
            stdout, stderr = child.communicate(timeout=15)
            self.assertEqual(child.returncode, 0, stderr)
            outputs.append(stdout)
        outcomes = [json.loads(output) for output in outputs]
        self.assertEqual(sum(item["ok"] for item in outcomes), 1)
        self.assertLessEqual(len(State(db).list("resource_reservation")), 1)

    def test_waiting_queue_admits_next_item_after_first_completes(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(2, 1024, 48 * 1024**3))
        first = manager.queue("a", request_id="one", amount=Capacity(1, 10, 1))
        second = manager.queue("b", request_id="two", amount=Capacity(1, 10, 1))
        self.assertEqual(manager.fair_queue()[0]["key"], first["key"])
        admitted = manager.admit("a", Capacity(1, 10, 1), inventory_generation="gen", request_id="one")
        self.assertIsNotNone(admitted)
        manager.state.put("native_job", "done", {"owner":"a", "status":"terminal", "completeProof":{"complete": True}, "executionId":"one"})
        self.assertTrue(manager.release(admitted["key"], native_proof_ref="done"))
        next_item = manager.admit("b", Capacity(1, 10, 1), inventory_generation="gen", request_id="two")
        self.assertIsNotNone(next_item)

    def test_wrong_birth_token_does_not_match_live_process_identity(self):
        with NativeJob.launch((sys.executable, "-B", "-c", "import time; time.sleep(2)"), cwd=self.root) as job:
            wrong = ProcessIdentity(job.identity.pid, "definitely-wrong-birth", job.identity.executable)
            self.assertFalse(identity_matches(wrong))
            self.assertGreater(job.proof(requested=False).active_processes, 0)
            job.terminate(timeout_seconds=10)

    def test_incomplete_native_proof_cannot_release_reservation(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(2, 1024, 48 * 1024**3))
        reservation = manager.admit("owner", Capacity(1, 10, 1), inventory_generation="gen", request_id="unproven")
        self.state.put("native_job", "incomplete", {"owner": "owner", "status": "termination_unproven", "completeProof": False})
        with self.assertRaises(JenkinsError) as error:
            manager.release(reservation["key"], native_proof_ref="incomplete")
        self.assertEqual(error.exception.code, "native_termination_unproven")

    def test_real_native_terminal_proof_releases_capacity_and_writer_hold(self):
        manager = ResourceManager(self.state, Capacity(2, 1024, 48 * 1024**3))
        manager.publish_inventory("gen", Capacity(2, 1024, 48 * 1024**3))
        reservation = manager.admit("owner", Capacity(1, 10, 1), inventory_generation="gen", writer_key="pool", request_id="run")
        with NativeJob.launch((sys.executable, "-B", "-c", "pass"), cwd=self.root) as job:
            registry = ProcessRegistry(self.state)
            record = registry.register(job, execution_id="run", owner="owner")
            self.assertEqual(job.wait(timeout_seconds=10), 0)
            terminal = registry.record_terminal(job)
            self.assertEqual(terminal.payload["status"], "terminal")
            self.assertTrue(manager.release(reservation["key"], writer_key="pool", native_proof_ref=record.native_job_id))
        self.assertEqual(manager.query(reservation["key"])["payload"]["status"], "released")

    def test_artifact_gc_protects_active_and_unknown_refs(self):
        source = self.root / "payload.bin"
        source.write_bytes(b"m1")
        store = ArtifactStore(self.root / "objects")
        registry = ArtifactRegistry(self.state, store)
        manifest = store.publish(source, execution_id="run")
        self.assertEqual(manifest.digest, hashlib.sha256(b"m1").hexdigest())
        with self.assertRaises(JenkinsError) as unknown:
            registry.attach("0" * 64, "owner")
        self.assertEqual(unknown.exception.code, "artifact_missing")
        ref = registry.attach(manifest.digest, "owner")
        self.assertEqual(registry.gc(), [])
        self.assertTrue(registry.detach(ref["key"]))
        self.assertEqual(registry.gc(), [manifest.digest])
        self.assertFalse((store.objects / manifest.digest).exists())

    def test_explicit_invalid_build_root_never_falls_back(self):
        with self.assertRaises(JenkinsError) as error:
            canonical_build_root(str(self.root))
        self.assertEqual(error.exception.code, "build_root_not_approved")


if __name__ == "__main__":
    unittest.main()

import hashlib
from pathlib import Path

import unittest

from tools.jenkins.artifacts import ArtifactStore, ArtifactRegistry
from tools.jenkins.state import State
from tools.jenkins.resources import Capacity, ReservationBook, canonical_build_root

class TestResourcesArtifacts(unittest.TestCase):
 def test_registry_rejects_missing_digest(self):
    import tempfile
    from tools.jenkins.artifacts import ArtifactRegistry
    from tools.jenkins.state import State
    root=Path(tempfile.mkdtemp()); registry=ArtifactRegistry(State(root/'s.db'), ArtifactStore(root/'n'))
    with self.assertRaises(Exception): registry.attach('0'*64, 'x')
 def test_build_root_rejects_explicit_invalid_without_fallback(self):
    with self.assertRaises(Exception) as error: canonical_build_root(r"C:\\temp")
    self.assertEqual(getattr(error.exception, "code", ""), "build_root_not_approved")

 def test_reservation_is_atomic_and_releasable(self):
    book = ReservationBook(Capacity(1, 10, 10))
    first = book.reserve("a", Capacity(1, 10, 10))
    self.assertIsNotNone(first)
    self.assertIsNone(book.reserve("b", Capacity(1, 1, 1)))
    self.assertTrue(book.release(first.id))

 def test_state_resource_idempotent_and_trusted_release(self):
    import tempfile
    from tools.jenkins.state import State
    from tools.jenkins.resources import ResourceManager
    root=Path(tempfile.mkdtemp()); state=State(root/'s.db'); manager=ResourceManager(state, Capacity(2,100,40*1024**3)); manager.publish_inventory('g', Capacity(2,100,40*1024**3))
    first=manager.admit('owner', Capacity(1,10,10), inventory_generation='g', request_id='req')
    second=manager.admit('owner', Capacity(1,10,10), inventory_generation='g', request_id='req')
    self.assertEqual(first['key'], second['key'])
    with self.assertRaises(Exception): manager.release(first['key'], native_proof_ref='bad')
    state.put('native_job','p',{'owner':'owner','status':'terminal','completeProof':{'complete': True},'executionId':'e','birthToken':'b'})
    self.assertTrue(manager.release(first['key'], native_proof_ref='p'))

 def test_artifact_publish_ref_and_gc(self):
    import tempfile
    tmp_path=Path(tempfile.mkdtemp())
    source = tmp_path / "input.bin"
    source.write_bytes(b"artifact")
    store = ArtifactStore(tmp_path / "namespace")
    manifest = store.publish(source, execution_id="exec-1")
    self.assertEqual(manifest.digest, hashlib.sha256(b"artifact").hexdigest())
    registry = ArtifactRegistry(State(tmp_path / "state.db"), store)
    ref = registry.attach(manifest.digest, "consumer-1")
    self.assertEqual(registry.gc(), [])
    self.assertTrue(registry.detach(ref["key"]))
    self.assertEqual(registry.gc(), [manifest.digest])

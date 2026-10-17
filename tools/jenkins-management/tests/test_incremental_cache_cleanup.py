"""Real Windows lock/deletion regressions for the fixed Rust cache policy."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

sys.path.insert(0, str(Path(__file__).parents[1]))
import incremental_cache_cleanup as cleanup


@unittest.skipUnless(os.name == "nt", "Windows rustc session locking")
class IncrementalCacheCleanupTests(unittest.TestCase):
    def fixture(self):
        return tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"])

    def session(self, root, suffix="abc", content=None):
        session = root / "cache/zircon_runtime-abcd" / ("s-abc-def-" + suffix)
        session.mkdir(parents=True)
        lock = session.parent / "s-abc-def.lock"
        lock.write_bytes(b"")
        for name, body in (content or {"dep-graph.bin": b"compiler metadata", "product.o": b"object"}).items():
            (session / name).write_bytes(body)
        old = time.time() - 3600
        os.utime(lock, (old, old))
        os.utime(session, (old, old))
        return session, lock

    def clean(self, root, **kwargs):
        return cleanup.clean(root / "cache", root / "output", trigger_free=2**63, **kwargs)

    def test_deletes_unlocked_session_and_accounts_hardlinks_without_deleting_other_names(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            retained = root / "retained.o"
            os.link(session / "product.o", retained)
            result = self.clean(root)
            self.assertEqual(result["status"], "complete")
            self.assertEqual(result["deletedFiles"], 2)
            self.assertEqual(result["logicalDeletedBytes"], len(b"compiler metadataobject"))
            self.assertFalse(session.exists())
            self.assertEqual(retained.read_bytes(), b"object")
            deleted = [json.loads(line) for line in (root / "output/deletions.jsonl").read_text().splitlines()
                       if json.loads(line)["status"] == "deleted"]
            shared = next(item for item in deleted if item["path"].endswith("product.o"))
            self.assertEqual(shared["hardLinksBefore"], 2)
            self.assertEqual(shared["unlinkedAllocationBytes"], 0)
            self.assertIsNone(result["actualFreedBytes"])

    def test_live_rustc_lock_preserves_every_cache_file(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, lock = self.session(root)
            with cleanup.session_lock(lock):
                result = self.clean(root)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertIn("locked", result["sessions"][0]["reason"])
            self.assertTrue((session / "product.o").exists())

    def test_unknown_source_content_protects_entire_session(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root, content={"product.o": b"object", "source.rs": b"source"})
            result = self.clean(root)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertEqual((session / "source.rs").read_bytes(), b"source")
            self.assertTrue((session / "product.o").exists())

    def test_new_session_and_over_budget_session_are_preserved(self):
        for settings in ({"minimum_age": 7200}, {"max_bytes": 1}):
            with self.subTest(settings=settings), self.fixture() as directory:
                root = Path(directory)
                session, _ = self.session(root)
                result = self.clean(root, **settings)
                self.assertEqual(result["deletedFiles"], 0)
                self.assertTrue(session.exists())

    def test_dry_run_preserves_objects_and_has_no_delete_journal_entries(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            result = self.clean(root, dry_run=True)
            self.assertGreater(result["selectedBytes"], 0)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertTrue(session.exists())
            self.assertEqual((root / "output/deletions.jsonl").read_text(), "")

    def test_changed_file_is_not_deleted_using_an_old_metadata_plan(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            fact = cleanup.cache_files(session)[0]
            Path(fact["path"]).write_bytes(b"changed after selection")
            with self.assertRaisesRegex(ValueError, "changed"):
                cleanup.remove_cache_file(fact)
            self.assertTrue(Path(fact["path"]).exists())

    def test_unexpected_selection_failure_keeps_a_failed_receipt(self):
        with self.fixture() as directory:
            root = Path(directory)
            self.session(root)
            with patch.object(Path, "iterdir", side_effect=RuntimeError("selection failed")):
                with self.assertRaisesRegex(RuntimeError, "selection failed"):
                    self.clean(root)
            receipt = json.loads((root / "output/receipt.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["deletedFiles"], 0)
            self.assertIn("finishedAt", receipt)

    def test_space_threshold_leaves_a_reusable_cache_intact(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            result = cleanup.clean(root / "cache", root / "output", trigger_free=0)
            self.assertEqual(result["status"], "enough-free-space")
            self.assertEqual(result["deletedFiles"], 0)
            self.assertTrue(session.exists())

    def test_install_preserves_original_job_schedule_governance_and_parameters(self):
        class Client:
            raw = b'<flow-definition><properties><cron>H/30 * * * *</cron></properties><definition><script>existingCasAndGovernance()</script></definition></flow-definition>'
            def request(self, path, *, method="GET", data=None, content_type=None):
                if method == "POST":
                    self.raw = data
                return 200, self.raw, {}
        client = Client()
        cleanup.install(cleanup.REPO, client)
        cleanup.install(cleanup.REPO, client)
        tree = ET.fromstring(client.raw)
        script = tree.findtext("definition/script")
        self.assertEqual(script.count(cleanup.BEGIN), 1)
        self.assertTrue(script.endswith("existingCasAndGovernance()"))
        self.assertEqual(tree.findtext("properties/cron"), "H/30 * * * *")

    def test_production_cli_refuses_local_deletion(self):
        with patch.dict(os.environ, {"JOB_NAME": "", "NODE_NAME": ""}), patch.object(
                sys, "argv", ["cleanup", "run", "--run-id", "incremental-1"]):
            with self.assertRaisesRegex(ValueError, "requires.*Jenkins"):
                cleanup.main()


if __name__ == "__main__":
    unittest.main(verbosity=2)

"""Public and interrupted-result contracts for commit-first Jenkins delivery."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("branch_queue_regression_fixtures", Path(__file__).with_name("test_branch_integration.py"))
regression = importlib.util.module_from_spec(spec)
spec.loader.exec_module(regression)
d, queue = regression.d, regression.queue


class EntryPointTests(unittest.TestCase):
    def test_historical_owner_cannot_authorize_a_current_driver_commit(self):
        with regression.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            before = queue.text(d, f[1], "rev-parse", "HEAD")
            result = queue.stage(d, f[0], f[2], policy=f[3], grants=f[4],
                                 runtime={"driverDigest": "current-driver"})
            self.assertEqual(result["status"], "failed")
            self.assertIn("another driver", result["reason"])
            self.assertEqual(queue.text(d, f[1], "rev-parse", "HEAD"), before)

    def test_old_delivery_uses_branch_commit_order_under_current_policy(self):
        with regression.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            with patch.object(d, "run_validation", side_effect=AssertionError("Do not validate before branch commit")):
                result = d.execute(f[0], f[2], policy=f[3], grants=f[4])
            self.assertEqual(result["status"], "committed", result.get("reason"))
            self.assertFalse(result["mainAdvanced"])

    def test_legacy_shared_main_commit_is_rejected_by_current_policy(self):
        with regression.BranchQueueTests().fixture() as f:
            payload = {**f[2], "action": "commit-checkout"}
            with patch.object(d.commit, "ROOT", f[0]), self.assertRaisesRegex(ValueError, "worktree branch first"):
                d.commit.commit_request(f[0], payload, policy=f[3], grants=f[4])

    def test_interrupted_commit_recovers_exact_objects_without_another_commit(self):
        with regression.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            first = regression.BranchQueueTests().stage(f)
            path = queue.directory(f[0]) / "requests/task-delivery.json"
            previous = {**first, "status": "committing"}
            previous.pop("sourceCommit")
            d.commit.write_json(path, previous)
            (f[1] / "owned.py").write_text("value = 3\n")
            recovered = regression.BranchQueueTests().stage(f)
            self.assertEqual(recovered["status"], "committed", recovered.get("reason"))
            self.assertTrue(recovered["recoveredReceipt"])
            self.assertEqual(recovered["sourceCommit"], first["sourceCommit"])
            self.assertEqual(queue.text(d, f[1], "rev-parse", "HEAD"), first["sourceCommit"])

    def test_interrupted_receipt_cannot_adopt_unmatched_objects(self):
        with regression.BranchQueueTests().fixture() as f:
            (f[1] / "owned.py").write_text("value = 2\n")
            first = regression.BranchQueueTests().stage(f)
            path = queue.directory(f[0]) / "requests/task-delivery.json"
            d.commit.write_json(path, {**first, "status": "committing", "expectedGitObjects": {"owned.py": "a" * 40}})
            result = regression.BranchQueueTests().stage(f)
            self.assertEqual(result["status"], "failed")
            self.assertIn("recorded task objects", result["reason"])


if __name__ == "__main__":
    unittest.main()

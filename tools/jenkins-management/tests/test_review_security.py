"""Privilege-boundary regressions from the Claude engineering review."""
import hashlib
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).parents[1] / filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


queue = load("review_queue", "branch_integration.py")
commit = load("review_commit", "checkout_commit.py")


class ReviewSecurityTests(unittest.TestCase):
    def test_generated_audit_cannot_be_added_but_can_be_retired(self):
        with tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name:
            root = Path(name)
            relative = "docs/plans/source-comments/new.json"
            output = root / relative
            output.parent.mkdir(parents=True)
            output.write_text("{}")
            with self.assertRaisesRegex(ValueError, "artifact store"):
                commit.selected_paths(root, [relative], {}, {str(root): "write"})
            output.unlink()
            self.assertEqual(commit.selected_paths(root, [relative], {}, {str(root): "write"}), [relative])

    def test_delivery_pin_matches_reviewed_bytes(self):
        raw = Path(queue.__file__).with_name("delivery.py").read_bytes()
        self.assertEqual(queue.DELIVERY_SHA256, hashlib.sha256(raw).hexdigest())

    def test_modified_delivery_is_rejected_before_execution(self):
        with patch.object(Path, "read_bytes", return_value=b"raise AssertionError('executed')"):
            with self.assertRaisesRegex(ValueError, "reviewed digest"):
                queue.load_delivery()

    def test_git_observation_does_not_execute_checkout_fsmonitor(self):
        with tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as name:
            root = Path(name)
            commit.git(root, "init", "-q")
            marker = root / "executed"
            hook = root / "monitor.sh"
            hook.write_text("#!/bin/sh\necho executed > executed\n", encoding="utf-8")
            hook.chmod(0o755)
            commit.git(root, "config", "core.fsmonitor", str(hook))
            commit.git(root, "status", "--porcelain")
            self.assertFalse(marker.exists())

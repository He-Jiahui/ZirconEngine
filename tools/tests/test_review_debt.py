import unittest
from pathlib import Path
from unittest.mock import patch

from tools.audits.check_conventions import run_conventions
from tools.audits.review_debt import baseline_sources


class ReviewDebtTests(unittest.TestCase):
    def test_frozen_sources_are_available(self):
        root = Path(__file__).resolve().parents[2]
        paths, sources = baseline_sources(root, ["zircon_runtime/src/lib.rs", "absent.rs"])
        self.assertIn("zircon_runtime/src/lib.rs", sources)
        self.assertIn("zircon_runtime/src", paths)
        self.assertNotIn("absent.rs", sources)

    def test_new_missing_reference_is_not_hidden_by_existing_debt(self):
        root = Path(__file__).resolve().parents[2]
        old = {"document": "docs/test.md", "field": "related_code", "path": "old.rs", "reason": "missing path"}
        new = {**old, "path": "new.rs"}
        with patch("tools.audits.check_conventions.audit_document_paths", return_value={"violations": [old, new], "violation_count": 2}), patch(
            "tools.audits.check_conventions.baseline_sources", return_value=(set(), {"docs/test.md": "---\nrelated_code:\n  - old.rs\n---\n"})
        ):
            report = run_conventions(root, ["docs"], dry_run=False)
        self.assertFalse(report["passed"])
        self.assertEqual(report["docs"]["baseline_violation_count"], 1)
        self.assertEqual(report["docs"]["violations"], [new])

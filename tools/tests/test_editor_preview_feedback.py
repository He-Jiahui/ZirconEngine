"""Canned preview feedback must never masquerade as an execution receipt."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
DISPATCH = ROOT / "zircon_editor/src/ui/retained/dispatch"


class PreviewFeedbackReceiptTests(unittest.TestCase):
    def test_canned_statuses_do_not_claim_unreceipted_work(self):
        tables = [DISPATCH / "templates/workbench/feedback.rs"]
        tables += list((DISPATCH / "workbench/feedback").glob("*.rs"))
        tables += list((DISPATCH / "feedback/ui_diagnostics").glob("*.rs"))
        count = 0
        for path in tables:
            for status in re.findall(r'status_text:\s*"([^"\n]*)"', path.read_text()):
                count += 1
                claims_work = re.search(r"\b(queued|applied|compiled|saved)\b", status, re.I)
                has_receipt = "{job_id}" in status or "{receipt_id}" in status
                self.assertFalse(claims_work and not has_receipt, f"{path}: {status}")
        self.assertGreater(count, 200, "include the extension feedback table closure")


if __name__ == "__main__":
    unittest.main()

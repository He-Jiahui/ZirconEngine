"""Unmeasured constants must not masquerade as allocation measurements."""

import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONSTANT_ALLOCATION = re.compile(
    r"\b([A-Za-z_]\w*(?:allocations|allocated_bytes)\w*)=(\d+)\b"
)


class PerfEvidenceLabelsTests(unittest.TestCase):
    def test_literal_allocation_fields_are_explicit_expectations(self):
        paths = subprocess.check_output(
            ["git", "ls-files", "*.rs"], cwd=ROOT, text=True
        ).splitlines()
        violations = []
        for path in paths:
            for number, line in enumerate((ROOT / path).read_text(encoding="utf-8").splitlines(), 1):
                if "PERF_RESULT" not in line:
                    continue
                for match in CONSTANT_ALLOCATION.finditer(line):
                    if not match[1].endswith("_expected"):
                        violations.append(f"{path}:{number}: {match[0]}")
        self.assertEqual(violations, [])

    def test_tab_layout_bytes_are_labeled_as_estimates(self):
        source = (ROOT / "zircon_runtime/src/text/layout/tests/tab.rs").read_text(encoding="utf-8")
        evidence = next(line for line in source.splitlines() if '"PERF_RESULT' in line)
        self.assertIn("legacy_width_estimated_bytes=", evidence)
        self.assertIn("optimized_width_estimated_bytes=", evidence)
        self.assertNotIn("allocated_bytes=", evidence)


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "zircon_export" / "pipeline_report.py"


class PipelineReportSerializationCachePerformanceContractTests(unittest.TestCase):
    def test_prints_the_payload_serialized_by_report_io(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("from .report_io import write_rendered_report_targets", source)
        self.assertEqual(2, source.count("write_rendered_report_targets("))
        self.assertEqual(2, source.count("print(rendered_report)"))
        self.assertNotIn("write_report_targets(", source)


if __name__ == "__main__":
    unittest.main()

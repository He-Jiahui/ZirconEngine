from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "audit_plugin_structure.py"


class PluginAuditParallelPerformanceContractTests(unittest.TestCase):
    def test_independent_audits_run_in_one_bounded_pool(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("with ThreadPoolExecutor(max_workers=len(audits)) as executor:", source)
        self.assertIn("executor.submit(audit, root)", source)
        self.assertIn("for name, audit in audits.items()", source)

    def test_report_waits_for_each_named_result_once(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("results = {name: future.result().to_json()", source)
        self.assertNotIn("audit_plugin_manifest_schema(root).to_json()", source)


if __name__ == "__main__":
    unittest.main()

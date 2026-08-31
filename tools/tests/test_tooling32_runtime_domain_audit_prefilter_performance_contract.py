from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "runtime_domain_dependency_audit.py"


class RuntimeDomainAuditPrefilterPerformanceContractTests(unittest.TestCase):
    def test_reuses_masked_module_attributes(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("masked_attributes = code_view[", source)
        self.assertNotIn("_rust_code_view(original_attributes)", source)

    def test_skips_files_without_dependency_candidates_before_lexing(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        prefilter = source.index('if "crate::" not in source:')
        lexical_view = source.index("code_view = code_views.get(source_path)")

        self.assertLess(prefilter, lexical_view)

    def test_skips_regex_for_lines_without_dependency_candidates(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        line_prefilter = source.index(
            'if "crate::" not in audit_line and not grouped_targets:'
        )
        regex_scan = source.index("CRATE_DOMAIN_REFERENCE.finditer(audit_line)")

        self.assertLess(line_prefilter, regex_scan)

    def test_production_reachability_uses_a_queue(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("production_pending = list(production_reachable)", source)
        self.assertNotIn("while changed:", source)


if __name__ == "__main__":
    unittest.main()

"""Performance contract for build-execution copied_sidecars validation."""

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from tools.zircon_export.pipeline_report_native_dynamic_build_audit_common import (
    string_array_unique_entries_schema_diagnostics,
)
from tools.zircon_export.pipeline_report_native_dynamic_build_execution_packages_schema import (
    native_dynamic_build_execution_copied_sidecars_value_diagnostics,
    native_dynamic_build_execution_package_path_scope_array_diagnostics,
    native_dynamic_build_execution_safe_relative_path_array_diagnostics,
)
from tools.zircon_export.pipeline_report_schema_string_array import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
OWNER = REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_build_execution_packages_schema.py"


def _legacy(label: str, package: dict[str, object], values: object) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(string_array_no_blank_entries_schema_diagnostics(label, values))
    diagnostics.extend(string_array_trimmed_non_empty_entries_schema_diagnostics(label, values))
    diagnostics.extend(string_array_unique_entries_schema_diagnostics(label, values))
    diagnostics.extend(native_dynamic_build_execution_safe_relative_path_array_diagnostics(label, values))
    diagnostics.extend(native_dynamic_build_execution_package_path_scope_array_diagnostics(label, package, values))
    return diagnostics


class NativeDynamicBuildExecutionSidecarsSinglePassPerformanceContractTests(unittest.TestCase):
    def test_mixed_invalid_sidecars_preserve_legacy_diagnostic_order(self) -> None:
        package = {"package_id": "demo"}
        values = ["", " plugins/demo/trim.dll ", "../unsafe.dll", "plugins/other/out.dll", "plugins/demo/same.dll", "plugins/demo/same.dll"]
        self.assertEqual(
            native_dynamic_build_execution_copied_sidecars_value_diagnostics("packages[0].copied_sidecars", package, values),
            _legacy("packages[0].copied_sidecars", package, values),
        )

    def test_non_string_entry_preserves_legacy_short_circuit(self) -> None:
        package = {"package_id": "demo"}
        values = ["../unsafe.dll", 42]
        self.assertEqual(
            native_dynamic_build_execution_copied_sidecars_value_diagnostics("sidecars", package, values),
            _legacy("sidecars", package, values),
        )

    def test_each_clean_sidecar_is_normalized_once(self) -> None:
        package = {"package_id": "demo"}
        values = [f"plugins/demo/sidecar-{index}.dll" for index in range(512)]
        calls = 0

        def counted(value: str) -> str:
            nonlocal calls
            calls += 1
            return value.replace("\\", "/")

        with patch(
            "tools.zircon_export.pipeline_report_native_dynamic_build_execution_packages_schema.normalize_relative_path",
            side_effect=counted,
        ):
            self.assertEqual(native_dynamic_build_execution_copied_sidecars_value_diagnostics("sidecars", package, values), [])
        self.assertEqual(calls, len(values))

    def test_value_projection_has_one_sidecar_loop(self) -> None:
        source = OWNER.read_text(encoding="utf-8")
        helper = source[source.index("def native_dynamic_build_execution_copied_sidecars_value_diagnostics("):source.index("def native_dynamic_build_execution_package_path_scope_array_diagnostics(")]
        self.assertEqual(helper.count("for index, value in enumerate(values):"), 1)


if __name__ == "__main__":
    unittest.main()

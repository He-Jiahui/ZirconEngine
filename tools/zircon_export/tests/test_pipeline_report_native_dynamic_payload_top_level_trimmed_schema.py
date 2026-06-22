from __future__ import annotations

import tempfile
import unittest
from collections.abc import Callable
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class NativeDynamicPayloadTopLevelTrimmedSchemaTests(unittest.TestCase):
    def _assert_payload_schema_diagnostic(
        self,
        mutate_payload: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
        unexpected_diagnostic: str | None = None,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            mutate_payload(payload)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_padded_top_level_string(
        self,
    ) -> None:
        cases = (
            "bundle_path",
            "content_hash",
            "loader_manifest",
            "source",
            "stage_report",
        )
        for field in cases:
            with self.subTest(field=field):

                def mutate(
                    payload: dict[str, object],
                    field: str = field,
                ) -> None:
                    value = payload[field]
                    self.assertIsInstance(value, str)
                    payload[field] = f" {value} "

                self._assert_payload_schema_diagnostic(
                    mutate,
                    f"native_plugins_payload.{field} "
                    "must be a non-empty trimmed string",
                    unexpected_diagnostic=(
                        "native_plugins_payload.content_hash "
                        "must be a SHA-256 hex digest"
                    )
                    if field == "content_hash"
                    else None,
                )


if __name__ == "__main__":
    unittest.main()

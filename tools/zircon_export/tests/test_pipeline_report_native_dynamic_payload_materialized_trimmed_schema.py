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


class NativeDynamicPayloadMaterializedTrimmedSchemaTests(unittest.TestCase):
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

    def test_report_rejects_native_plugins_payload_materialized_package_padded_loadable_artifact(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            artifacts = package["loadable_artifacts"]
            self.assertIsInstance(artifacts, list)
            artifacts[0] = " native/zircon_plugin_animation.dll "

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifacts[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_padded_duplicate_id_before_uniqueness(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            duplicate = dict(package)
            duplicate["package_id"] = " animation "
            packages.append(duplicate)
            payload["package_count"] = len(packages)

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[1].package_id "
            "must be a non-empty trimmed string",
            "native_plugins_payload "
            "materialized_packages[1].package_id "
            "must be unique",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_padded_duplicate_loadable_artifact_before_uniqueness(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            artifacts = package["loadable_artifacts"]
            self.assertIsInstance(artifacts, list)
            artifacts.append(f" {artifacts[0]} ")
            package["loadable_artifact_count"] = len(artifacts)

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifacts[1] "
            "must be a non-empty trimmed string",
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifacts "
            "must not contain duplicate entries",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_padded_string_field(
        self,
    ) -> None:
        cases = (
            ("package_id", " animation "),
            (
                "destination",
                " plugins/animation ",
            ),
            (
                "package_report",
                " plugins/animation/native_dynamic_package.toml ",
            ),
            ("source", " source/animation "),
        )
        for field, value in cases:
            with self.subTest(field=field):

                def mutate(
                    payload: dict[str, object],
                    field: str = field,
                    value: str = value,
                ) -> None:
                    packages = payload["materialized_packages"]
                    self.assertIsInstance(packages, list)
                    package = packages[0]
                    self.assertIsInstance(package, dict)
                    package[field] = value

                self._assert_payload_schema_diagnostic(
                    mutate,
                    "native_plugins_payload "
                    f"materialized_packages[0].{field} "
                    "must be a non-empty trimmed string",
                )


if __name__ == "__main__":
    unittest.main()

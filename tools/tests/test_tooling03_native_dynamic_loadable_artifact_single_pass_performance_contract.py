from __future__ import annotations

import unittest
from unittest import mock

from tools.zircon_export import (
    pipeline_report_native_dynamic_payload_string_array_schema_helpers as subject,
)


class NativeDynamicLoadableArtifactSinglePassPerformanceContractTests(unittest.TestCase):
    def test_single_pass_preserves_legacy_diagnostic_group_order(self) -> None:
        payload = {
            "materialized_packages": [
                {
                    "loadable_artifact_count": 99,
                    "loadable_artifacts": [
                        "plugins/a.dll",
                        "plugins/a.dll",
                        " ",
                        " padded.dll ",
                        "../outside.dll",
                    ],
                },
                {
                    "loadable_artifact_count": 1,
                    "loadable_artifacts": [42],
                },
            ]
        }
        label = "native_dynamic report"
        field = "materialized_packages"
        fields = ("loadable_artifacts",)
        legacy: list[str] = []
        legacy.extend(
            subject.object_array_loadable_artifacts_schema_diagnostics(
                label, payload, field
            )
        )
        legacy.extend(
            subject.object_array_string_array_no_blank_entries_schema_diagnostics(
                label, payload, field, fields
            )
        )
        legacy.extend(
            subject.object_array_string_array_trimmed_non_empty_entries_schema_diagnostics(
                label, payload, field, fields
            )
        )
        legacy.extend(
            subject.object_array_string_array_safe_relative_path_schema_diagnostics(
                label, payload, field, fields
            )
        )
        legacy.extend(
            subject.object_array_string_array_unique_entries_schema_diagnostics(
                label, payload, field, fields
            )
        )
        legacy.extend(
            subject.object_array_integer_matches_string_array_length_schema_diagnostics(
                label,
                payload,
                field,
                "loadable_artifact_count",
                "loadable_artifacts",
            )
        )

        type_diagnostics, value_diagnostics = (
            subject.materialized_package_loadable_artifact_schema_diagnostics(
                label,
                payload,
                field,
                "loadable_artifact_count",
                "loadable_artifacts",
            )
        )

        self.assertEqual(legacy, type_diagnostics + value_diagnostics)

    def test_each_artifact_is_normalized_once(self) -> None:
        package_count = 128
        artifact_count = 32
        payload = {
            "materialized_packages": [
                {
                    "loadable_artifact_count": artifact_count,
                    "loadable_artifacts": [
                        f"plugins/package-{package:03d}/native/artifact-{artifact:03d}.dll"
                        for artifact in range(artifact_count)
                    ],
                }
                for package in range(package_count)
            ]
        }
        original = subject.normalize_relative_path
        normalizations = 0

        def count_normalization(value: str) -> str:
            nonlocal normalizations
            normalizations += 1
            return original(value)

        with mock.patch.object(
            subject,
            "normalize_relative_path",
            count_normalization,
        ):
            diagnostic_groups = (
                subject.materialized_package_loadable_artifact_schema_diagnostics(
                    "native_dynamic report",
                    payload,
                    "materialized_packages",
                    "loadable_artifact_count",
                    "loadable_artifacts",
                )
            )

        self.assertEqual(([], []), diagnostic_groups)
        self.assertEqual(package_count * artifact_count, normalizations)


if __name__ == "__main__":
    unittest.main()

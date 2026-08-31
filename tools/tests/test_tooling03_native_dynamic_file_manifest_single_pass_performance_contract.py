from __future__ import annotations

import unittest
from unittest import mock

from tools.zircon_export import (
    pipeline_report_native_dynamic_payload_file_manifest_schema as subject,
)
from tools.zircon_export.pipeline_report_native_dynamic_payload_schema_helpers import (
    object_array_non_negative_integer_schema_diagnostics,
    object_array_required_non_empty_string_schema_diagnostics,
    object_array_required_trimmed_non_empty_string_schema_diagnostics,
    object_array_safe_relative_path_string_schema_diagnostics,
    object_array_sha256_hex_string_schema_diagnostics,
    object_array_unique_string_field_schema_diagnostics,
)


def _legacy(label: str, payload: dict[str, object]) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            label, payload, "file_manifest", ("path", "sha256")
        )
    )
    diagnostics.extend(
        object_array_required_trimmed_non_empty_string_schema_diagnostics(
            label, payload, "file_manifest", ("path", "sha256")
        )
    )
    diagnostics.extend(
        object_array_sha256_hex_string_schema_diagnostics(
            label, payload, "file_manifest", ("sha256",)
        )
    )
    diagnostics.extend(
        object_array_safe_relative_path_string_schema_diagnostics(
            label, payload, "file_manifest", ("path",)
        )
    )
    diagnostics.extend(
        object_array_unique_string_field_schema_diagnostics(
            label, payload, "file_manifest", "path", normalize_path=True
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            label, payload, "file_manifest", ("bytes",)
        )
    )
    return diagnostics


class NativeDynamicFileManifestSinglePassPerformanceContractTests(unittest.TestCase):
    def test_single_pass_preserves_legacy_diagnostic_order(self) -> None:
        payload = {
            "file_manifest": [
                {"path": " ", "sha256": " ", "bytes": -1},
                {"path": " padded.dll ", "sha256": "x" * 64, "bytes": 1},
                {"path": "../outside.dll", "sha256": "z" * 64, "bytes": 1},
                {"path": "plugins/a.dll", "sha256": "0" * 64, "bytes": 1},
                {"path": "plugins/a.dll", "sha256": "0" * 64, "bytes": 1},
            ]
        }
        label = "native_dynamic report"

        self.assertEqual(
            _legacy(label, payload),
            subject.native_dynamic_file_manifest_value_schema_diagnostics(
                label, payload
            ),
        )

    def test_each_path_is_normalized_once(self) -> None:
        count = 4096
        payload = {
            "file_manifest": [
                {
                    "path": f"plugins/package/file-{index:05d}.dll",
                    "sha256": f"{index:064x}",
                    "bytes": index,
                }
                for index in range(count)
            ]
        }
        original = subject.normalize_relative_path
        calls = 0

        def count_normalization(value: str) -> str:
            nonlocal calls
            calls += 1
            return original(value)

        with mock.patch.object(subject, "normalize_relative_path", count_normalization):
            diagnostics = subject.native_dynamic_file_manifest_value_schema_diagnostics(
                "native_dynamic report", payload
            )

        self.assertEqual([], diagnostics)
        self.assertEqual(count, calls)


if __name__ == "__main__":
    unittest.main()

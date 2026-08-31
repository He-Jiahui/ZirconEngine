from __future__ import annotations

import unittest
from unittest import mock

from tools.zircon_export import pipeline_report_pack_manifest_schema as subject
from tools.zircon_export import pipeline_report_pack_manifest_schema_helpers as helpers
from tools.zircon_export.pipeline_report_schema_primitives import (
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)


class PackManifestSchemaProjectionPerformanceContractTests(unittest.TestCase):
    def test_document_schema_checks_each_chunk_and_asset_once(self) -> None:
        count = 256
        chunks = [
            {
                "hash": [index // 256, index % 256] + [0] * 30,
                "offset": 24 + index,
                "size": 1,
            }
            for index in range(count)
        ]
        assets = [
            {
                "path": f"assets/{index:04d}.bin",
                "chunk_hash": chunks[index]["hash"],
                "size": 1,
            }
            for index in range(count)
        ]
        manifest = {
            "pack": {"version": 1, "total_size": count, "chunks": chunks},
            "assets": assets,
        }
        original_chunk = helpers.pack_chunk_entry_is_schema_clean
        original_asset = helpers.pack_asset_entry_is_schema_clean
        chunk_checks = 0
        asset_checks = 0

        def check_chunk(value: object) -> bool:
            nonlocal chunk_checks
            chunk_checks += 1
            return original_chunk(value)

        def check_asset(value: object) -> bool:
            nonlocal asset_checks
            asset_checks += 1
            return original_asset(value)

        with (
            mock.patch.object(subject, "pack_chunk_entry_is_schema_clean", check_chunk),
            mock.patch.object(subject, "pack_asset_entry_is_schema_clean", check_asset),
            mock.patch.object(helpers, "pack_chunk_entry_is_schema_clean", check_chunk),
            mock.patch.object(helpers, "pack_asset_entry_is_schema_clean", check_asset),
        ):
            diagnostics = subject.pack_document_manifest_schema_diagnostics(
                "manifest",
                manifest,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )

        self.assertEqual([], diagnostics)
        self.assertEqual(count, chunk_checks)
        self.assertEqual(count, asset_checks)


if __name__ == "__main__":
    unittest.main()

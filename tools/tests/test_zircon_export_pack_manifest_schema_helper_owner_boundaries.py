"""Boundary tests for pack manifest schema helper ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_MANIFEST_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_manifest_schema.py"
)
PACK_MANIFEST_SCHEMA_HELPERS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_manifest_schema_helpers.py"
)
PACK_MANIFEST_PATH_HASH_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_pack_manifest_path_hash_schema_helpers.py"
)
PACK_DELTA_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_delta_schema.py"
)
PACK_DELTA_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_delta_semantics.py"
)
PACK_TRIM_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_trim_schema.py"
)

MOVED_CONSTANTS = (
    "PACK_FORMAT_VERSION",
    "PACK_MANIFEST_FIELDS",
    "PACK_MANIFEST_INTEGER_FIELDS",
    "PACK_MANIFEST_REQUIRED_INTEGER_FIELDS",
    "PACK_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS",
    "PACK_MANIFEST_NON_NEGATIVE_INTEGER_FIELDS",
    "PACK_CHUNK_ENTRY_FIELDS",
    "PACK_CHUNK_ENTRY_INTEGER_FIELDS",
    "PACK_CHUNK_ENTRY_REQUIRED_BYTE_ARRAY_FIELDS",
    "PACK_CHUNK_ENTRY_REQUIRED_INTEGER_FIELDS",
    "PACK_CHUNK_ENTRY_NON_NEGATIVE_INTEGER_FIELDS",
    "PACK_ASSET_ENTRY_FIELDS",
    "PACK_ASSET_ENTRY_STRING_FIELDS",
    "PACK_ASSET_ENTRY_INTEGER_FIELDS",
    "PACK_ASSET_ENTRY_REQUIRED_BYTE_ARRAY_FIELDS",
    "PACK_ASSET_ENTRY_REQUIRED_STRING_FIELDS",
    "PACK_ASSET_ENTRY_REQUIRED_INTEGER_FIELDS",
    "PACK_ASSET_ENTRY_NON_NEGATIVE_INTEGER_FIELDS",
)

MOVED_FUNCTIONS = (
    "pack_manifest_is_schema_clean",
    "pack_chunk_entry_is_schema_clean",
    "pack_asset_entry_is_schema_clean",
    "pack_manifest_schema_diagnostics",
    "pack_chunk_entries_schema_diagnostics",
    "pack_total_size_diagnostics",
    "pack_version_diagnostics",
    "pack_chunk_offset_diagnostics",
    "pack_chunk_offset_sort_key",
    "pack_asset_chunk_reference_diagnostics",
    "pack_asset_chunk_size_diagnostics",
    "pack_asset_entries_schema_diagnostics",
    "non_negative_integer_diagnostics",
)

MOVED_PATH_HASH_FUNCTIONS = (
    "pack_chunk_hash_uniqueness_diagnostics",
    "pack_chunk_hash_order_diagnostics",
    "pack_asset_path_schema_diagnostics",
    "pack_asset_path_is_schema_clean",
    "pack_asset_path_uniqueness_diagnostics",
    "pack_asset_path_order_diagnostics",
    "is_safe_asset_package_path",
    "normalized_asset_package_path",
    "is_byte_hash",
    "validate_byte_array_schema_diagnostics",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PackManifestSchemaHelperOwnerBoundaryTests(unittest.TestCase):
    def test_pack_manifest_schema_helper_owner_exists(self):
        self.assertTrue(
            PACK_MANIFEST_SCHEMA_HELPERS.exists(),
            "Pack manifest schema helper owner file is missing",
        )

    def test_pack_manifest_path_hash_helper_owner_exists(self):
        self.assertTrue(
            PACK_MANIFEST_PATH_HASH_HELPERS.exists(),
            "Pack manifest path/hash schema helper owner file is missing",
        )

    def test_reusable_pack_schema_members_are_owned_by_helper_module(self):
        schema_text = PACK_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        helper_text = (
            PACK_MANIFEST_SCHEMA_HELPERS.read_text(encoding="utf-8")
            if PACK_MANIFEST_SCHEMA_HELPERS.exists()
            else ""
        )

        failures: list[str] = []
        for constant_name in MOVED_CONSTANTS:
            definition = f"{constant_name} ="
            if definition in schema_text:
                failures.append(f"manifest schema still owns {constant_name}")
            if definition not in helper_text:
                failures.append(f"helper owner missing {constant_name}")
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"manifest schema still owns {function_name}")
            if definition not in helper_text:
                failures.append(f"helper owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_schema_imports_helpers_without_reverse_import(self):
        schema_text = PACK_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        helper_text = (
            PACK_MANIFEST_SCHEMA_HELPERS.read_text(encoding="utf-8")
            if PACK_MANIFEST_SCHEMA_HELPERS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_pack_manifest_schema_helpers import (",
            schema_text,
        )
        self.assertNotIn(
            ".pipeline_report_pack_manifest_schema",
            helper_text,
        )

    def test_path_hash_helpers_live_in_path_hash_owner(self):
        helper_text = PACK_MANIFEST_SCHEMA_HELPERS.read_text(encoding="utf-8")
        path_hash_text = (
            PACK_MANIFEST_PATH_HASH_HELPERS.read_text(encoding="utf-8")
            if PACK_MANIFEST_PATH_HASH_HELPERS.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in MOVED_PATH_HASH_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in helper_text:
                failures.append(f"schema helper still owns {function_name}")
            if definition not in path_hash_text:
                failures.append(f"path/hash helper owner missing {function_name}")
        for imported_helper in ("is_safe_relative_path", "normalize_relative_path"):
            if imported_helper in helper_text:
                failures.append(
                    f"schema helper still imports path normalizer {imported_helper}"
                )
            if imported_helper not in path_hash_text:
                failures.append(
                    f"path/hash helper owner missing path normalizer {imported_helper}"
                )

        if failures:
            self.fail("\n".join(failures))

    def test_path_hash_consumers_import_owner_directly(self):
        schema_text = PACK_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        delta_text = PACK_DELTA_SCHEMA.read_text(encoding="utf-8")
        trim_text = PACK_TRIM_SCHEMA.read_text(encoding="utf-8")
        import_statement = (
            "from .pipeline_report_pack_manifest_path_hash_schema_helpers import ("
        )

        for consumer_name, consumer_text in (
            ("pack manifest schema", schema_text),
            ("pack delta schema", delta_text),
            (
                "pack delta semantics",
                PACK_DELTA_SEMANTICS.read_text(encoding="utf-8"),
            ),
            ("pack trim schema", trim_text),
        ):
            self.assertIn(
                import_statement,
                consumer_text,
                f"{consumer_name} should import path/hash helpers directly",
            )

        path_hash_text = (
            PACK_MANIFEST_PATH_HASH_HELPERS.read_text(encoding="utf-8")
            if PACK_MANIFEST_PATH_HASH_HELPERS.exists()
            else ""
        )
        self.assertNotIn(
            ".pipeline_report_pack_manifest_schema_helpers",
            path_hash_text,
            "path/hash helper owner must not import the schema helper owner",
        )

    def test_pack_manifest_schema_and_helper_owner_stay_small(self):
        self.assertLess(_line_count(PACK_MANIFEST_SCHEMA), 360)
        self.assertTrue(
            PACK_MANIFEST_SCHEMA_HELPERS.exists(),
            "Pack manifest schema helper owner file is missing",
        )
        self.assertLess(_line_count(PACK_MANIFEST_SCHEMA_HELPERS), 430)
        self.assertTrue(
            PACK_MANIFEST_PATH_HASH_HELPERS.exists(),
            "Pack manifest path/hash schema helper owner file is missing",
        )
        self.assertLess(_line_count(PACK_MANIFEST_PATH_HASH_HELPERS), 180)


if __name__ == "__main__":
    unittest.main()

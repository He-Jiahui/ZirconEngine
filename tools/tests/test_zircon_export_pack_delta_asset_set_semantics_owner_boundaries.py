"""Boundary tests for Pack delta asset-set semantic diagnostics ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_DELTA_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_pack_delta_semantics.py"
)
PACK_DELTA_ASSET_SET_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_pack_delta_asset_set_semantics.py"
)

ASSET_SET_TYPE_ALIASES = (
    "PackChunkFingerprint",
    "PackPlanFingerprint",
    "PackAssetFingerprint",
    "PackDocumentFingerprint",
)

ASSET_SET_FUNCTIONS = (
    "pack_report_delta_asset_set_diagnostics",
    "asset_path_list_is_schema_clean",
    "pack_document_manifest_fingerprint",
    "delta_changed_assets_are_schema_clean",
    "delta_chunks_are_schema_clean",
    "delta_removed_asset_paths",
    "delta_changed_and_reused_asset_paths",
    "delta_changed_asset_entries_match",
    "delta_changed_asset_entries",
    "delta_changed_asset_chunk_hashes_match",
    "delta_manifest_base_chunk_hashes",
    "delta_manifest_assets",
    "manifest_asset_paths",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PackDeltaAssetSetSemanticsOwnerBoundaryTests(unittest.TestCase):
    def test_pack_delta_asset_set_semantics_owner_exists(self) -> None:
        self.assertTrue(
            PACK_DELTA_ASSET_SET_SEMANTICS.exists(),
            "Pack delta asset-set semantics owner file is missing",
        )

    def test_asset_set_semantics_live_in_asset_set_owner(self) -> None:
        parent_text = PACK_DELTA_SEMANTICS.read_text(encoding="utf-8")
        asset_set_text = (
            PACK_DELTA_ASSET_SET_SEMANTICS.read_text(encoding="utf-8")
            if PACK_DELTA_ASSET_SET_SEMANTICS.exists()
            else ""
        )

        failures: list[str] = []
        for alias_name in ASSET_SET_TYPE_ALIASES:
            definition = f"{alias_name} ="
            if definition in parent_text:
                failures.append(f"delta semantics parent still owns {alias_name}")
            if definition not in asset_set_text:
                failures.append(f"asset-set owner missing {alias_name}")
        for function_name in ASSET_SET_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in parent_text:
                failures.append(f"delta semantics parent still owns {function_name}")
            if definition not in asset_set_text:
                failures.append(f"asset-set owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_parent_imports_asset_set_owner_without_reverse_import(self) -> None:
        parent_text = PACK_DELTA_SEMANTICS.read_text(encoding="utf-8")
        asset_set_text = (
            PACK_DELTA_ASSET_SET_SEMANTICS.read_text(encoding="utf-8")
            if PACK_DELTA_ASSET_SET_SEMANTICS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_pack_delta_asset_set_semantics import (",
            parent_text,
        )
        self.assertNotIn(
            "from .pipeline_report_pack_delta_semantics import",
            asset_set_text,
        )

    def test_delta_semantics_parent_budget_stays_tight(self) -> None:
        self.assertLess(_line_count(PACK_DELTA_SEMANTICS), 190)
        self.assertTrue(
            PACK_DELTA_ASSET_SET_SEMANTICS.exists(),
            "Pack delta asset-set semantics owner file is missing",
        )
        self.assertLess(_line_count(PACK_DELTA_ASSET_SET_SEMANTICS), 340)


if __name__ == "__main__":
    unittest.main()

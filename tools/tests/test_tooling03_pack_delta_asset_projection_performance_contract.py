from __future__ import annotations

import unittest
from unittest import mock

from tools.export import pipeline_report_pack_delta_asset_set_semantics as subject
from tools.export import pipeline_report_pack_delta_asset_projection as projection_owner


def _manifest(asset_count: int, hash_offset: int = 0) -> dict[str, object]:
    chunks = [
        {"hash": [((index + hash_offset) % 251)] * 32, "offset": index, "size": 1}
        for index in range(asset_count)
    ]
    return {
        "pack": {"version": 1, "total_size": asset_count, "chunks": chunks},
        "assets": [
            {
                "path": f"assets/{index:05d}.bin",
                "chunk_hash": chunks[index]["hash"],
                "size": 1,
            }
            for index in range(asset_count)
        ],
    }


class PackDeltaAssetProjectionPerformanceContractTests(unittest.TestCase):
    def test_diagnostics_walks_each_document_manifest_once(self) -> None:
        base = _manifest(128)
        target = _manifest(128, hash_offset=1)
        delta_manifest = {
            "base": base,
            "target": target,
            "removed_assets": [],
            "changed_assets": target["assets"],
            "chunks": target["pack"]["chunks"],
        }
        report = {"delta_removed_assets": [], "delta_reused_assets": []}

        # The projection owner now performs the one schema validation pass for
        # the base and target document manifests.  Patch that binding rather
        # than the compatibility import retained by the set-semantics owner.
        original = projection_owner.pack_document_manifest_is_schema_clean
        checked: list[object] = []

        def count_manifest(manifest: dict[str, object]) -> bool:
            checked.append(manifest)
            return original(manifest)

        with mock.patch.object(
            projection_owner,
            "pack_document_manifest_is_schema_clean",
            count_manifest,
        ):
            subject.pack_report_delta_asset_set_diagnostics(report, delta_manifest)

        self.assertEqual([base, target], checked)

    def test_diagnostics_parses_changed_assets_once(self) -> None:
        base = _manifest(64)
        target = _manifest(64, hash_offset=1)
        changed_assets = target["assets"]
        delta_manifest = {
            "base": base,
            "target": target,
            "removed_assets": [],
            "changed_assets": changed_assets,
            "chunks": target["pack"]["chunks"],
        }
        report = {"delta_removed_assets": [], "delta_reused_assets": []}

        original = subject.delta_changed_asset_entries
        calls = 0

        def count_entries(entries: list[object]) -> list[dict[str, object]] | None:
            nonlocal calls
            calls += 1
            return original(entries)

        with mock.patch.object(subject, "delta_changed_asset_entries", count_entries):
            subject.pack_report_delta_asset_set_diagnostics(report, delta_manifest)

        self.assertEqual(1, calls)


if __name__ == "__main__":
    unittest.main()

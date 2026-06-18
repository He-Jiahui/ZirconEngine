from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)
from tools.zircon_export.tests.pack_test_support import pack_manifest


def write_library_embed_reports(out: Path) -> None:
    _write_validate_report_with_strategies(out, ["library_embed"])
    _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
    _write_stage_report(out, "cook_assets", fatal=False)
    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
    _write_stage_report(out, "platform_bundle", fatal=False)


def update_pack_report(
    out: Path,
    *,
    manifest: object | None = None,
    delta_manifest: object | None = None,
    trim_report: object | None = None,
    deduplicated_assets: object | None = None,
) -> None:
    pack_report_path = out / "stages" / "pack" / "report.json"
    pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
    if manifest is not None:
        pack_report["manifest"] = manifest
        pack_report["asset_count"] = manifest_asset_count(manifest)
        pack_report["chunk_count"] = manifest_chunk_count(manifest)
    if delta_manifest is not None:
        pack_report["delta_manifest"] = delta_manifest
        sync_delta_report_counts(pack_report)
        pack_report["delta_removed_assets"] = delta_removed_assets(delta_manifest)
        pack_report["delta_reused_assets"] = delta_reused_assets(delta_manifest)
    if trim_report is not None:
        pack_report["trim_report"] = trim_report
    if deduplicated_assets is not None:
        pack_report["deduplicated_assets"] = deduplicated_assets
    pack_report_path.write_text(
        json.dumps(pack_report, indent=2),
        encoding="utf-8",
    )


def sync_delta_report_counts(pack_report: dict[str, object]) -> None:
    delta_manifest = pack_report.get("delta_manifest")
    if not isinstance(delta_manifest, dict):
        return
    changed_assets = delta_manifest.get("changed_assets")
    if isinstance(changed_assets, list):
        pack_report["delta_asset_count"] = len(changed_assets)
    chunks = delta_manifest.get("chunks")
    if isinstance(chunks, list):
        pack_report["delta_chunk_count"] = len(chunks)


def assert_pack_schema_diagnostic(
    test_case: unittest.TestCase,
    report: dict[str, object],
    expected: str,
) -> None:
    diagnostics = report["diagnostics"]
    test_case.assertTrue(report["fatal"])
    test_case.assertIn("Pack", report["fatal_stages"])
    test_case.assertEqual(report["missing_stages"], [])
    test_case.assertTrue(
        any(expected in diagnostic for diagnostic in diagnostics),
        diagnostics,
    )


def manifest_override(
    override: dict[str, object],
    *,
    hash_value: int = 1,
) -> dict[str, object]:
    manifest = pack_manifest(hash_value=hash_value)
    manifest.update(override)
    return manifest


def trim_report() -> dict[str, object]:
    return {
        "included_assets": ["scenes/main.zscene"],
        "trimmed_assets": [trimmed_asset()],
        "missing_dependencies": [missing_dependency()],
        "duplicate_assets": [],
        "diagnostics": ["trimmed asset textures/unused.png: unreferenced"],
    }


def trimmed_asset() -> dict[str, object]:
    return {
        "path": "textures/unused.png",
        "reason": "Unreferenced",
    }


def missing_dependency() -> dict[str, object]:
    return {
        "owner": "scenes/main.zscene",
        "dependency": "textures/missing.png",
    }


def delta_removed_assets(delta_manifest: object) -> list[str]:
    if not isinstance(delta_manifest, dict):
        return []
    removed_assets = delta_manifest.get("removed_assets")
    if not isinstance(removed_assets, list):
        return []
    return [asset for asset in removed_assets if isinstance(asset, str)]


def delta_reused_assets(delta_manifest: object) -> list[str]:
    if not isinstance(delta_manifest, dict):
        return []
    base = delta_manifest.get("base")
    target = delta_manifest.get("target")
    if not isinstance(base, dict) or not isinstance(target, dict):
        return []
    base_hashes = {
        tuple(chunk["hash"])
        for chunk in manifest_chunks(base)
        if isinstance(chunk.get("hash"), list)
    }
    reused_assets: list[str] = []
    for asset in sorted(manifest_assets(target), key=lambda entry: str(entry["path"])):
        chunk_hash = asset.get("chunk_hash")
        if isinstance(asset.get("path"), str) and isinstance(chunk_hash, list):
            if tuple(chunk_hash) in base_hashes:
                reused_assets.append(str(asset["path"]))
    return reused_assets


def manifest_asset_count(manifest: object) -> int:
    if isinstance(manifest, dict) and isinstance(manifest.get("assets"), list):
        return len(manifest["assets"])
    return 1


def manifest_chunk_count(manifest: object) -> int:
    if not isinstance(manifest, dict):
        return 1
    pack = manifest.get("pack")
    if isinstance(pack, dict) and isinstance(pack.get("chunks"), list):
        return len(pack["chunks"])
    return 1


def manifest_assets(manifest: dict[str, object]) -> list[dict[str, object]]:
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return []
    return [asset for asset in assets if isinstance(asset, dict)]


def manifest_chunks(manifest: dict[str, object]) -> list[dict[str, object]]:
    pack = manifest.get("pack")
    if not isinstance(pack, dict):
        return []
    chunks = pack.get("chunks")
    if not isinstance(chunks, list):
        return []
    return [chunk for chunk in chunks if isinstance(chunk, dict)]

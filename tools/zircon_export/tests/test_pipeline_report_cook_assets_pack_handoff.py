from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.pipeline_report_pack_file_evidence import (
    PACK_BINARY_HEADER_SIZE,
    zrpack_content_hash,
)
from tools.zircon_export.tests.export_test_support import (
    _pack_binary_bytes,
    _write_compile_host_report,
    _write_pack_report,
    _write_validate_report_with_strategies,
    json_dumps,
    json_loads,
)
from tools.zircon_export.tests.platform_bundle_export_test_support import (
    _write_platform_bundle_report_with_native_plugins_payload,
)


class PipelineReportCookAssetsPackHandoffTests(unittest.TestCase):
    def test_report_stage_rejects_pack_asset_manifest_drift_from_cook_assets_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source = root / "source" / "main.scene"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"scene-bytes")
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "source": str(source),
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report manifest.assets[0].size 8 does not match "
                    "CookAssets source byte length 11 for included asset "
                    "scenes/main.zscene"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "pack report manifest.assets[0].chunk_hash does not match "
                    "CookAssets source content hash for included asset "
                    "scenes/main.zscene"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_asset_schema_before_source_byte_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source = root / "source" / "main.scene"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"scene-bytes")
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "source": str(source),
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json_loads(pack_report_path.read_text(encoding="utf-8"))
            assert isinstance(pack_report, dict)
            manifest = pack_report["manifest"]
            assert isinstance(manifest, dict)
            assets = manifest["assets"]
            assert isinstance(assets, list)
            asset = assets[0]
            assert isinstance(asset, dict)
            asset["size"] = -1
            pack = pack_report.get("pack")
            assert isinstance(pack, str)
            Path(pack).write_bytes(
                _pack_binary_bytes(
                    pack_report["manifest"],
                    b"ZRPK",
                    payload=pack_payload_for_asset(0),
                )
            )
            pack_report_path.write_text(
                json_dumps(pack_report),
                encoding="utf-8",
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report manifest.assets[0].size must be non-negative"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "CookAssets source byte length" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_cook_assets_source_path_before_pack_source_byte_semantics(
        self,
    ) -> None:
        cases = (
            (
                "relative_source",
                lambda root: "main.scene",
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].source must be an absolute path"
                ),
            ),
            (
                "missing_source",
                lambda root: str(root / "source" / "missing.scene"),
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].source does not exist"
                ),
            ),
        )
        for label, source_value, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    write_cook_assets_report(
                        out,
                        {
                            "roots": ["scenes/main.zscene"],
                            "assets": [
                                {
                                    "path": "scenes/main.zscene",
                                    "source": source_value(root),
                                    "dependencies": [],
                                    "labels": [],
                                },
                            ],
                        },
                    )
                    _write_pack_report(
                        out,
                        out / "stages" / "pack" / "assets.zrpack",
                    )
                    rewrite_pack_report_for_trim_evidence(
                        out,
                        included_assets=["scenes/main.zscene"],
                        trimmed_assets=[],
                    )
                    _write_platform_bundle_report_with_native_plugins_payload(out, {})

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "CookAssets source" in diagnostic
                            and "could not be read" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_included_asset_missing_cook_assets_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.included_assets contains "
                    "scenes/main.zscene but CookAssets manifest asset is "
                    "missing source"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_trim_included_assets_outside_cook_assets_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "asset_filter": "shipping",
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/hero.png"],
                            "labels": ["shipping"],
                        },
                        {
                            "path": "textures/hero.png",
                            "dependencies": [],
                            "labels": ["shipping"],
                        },
                        {
                            "path": "textures/unused.png",
                            "dependencies": [],
                            "labels": ["shipping"],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_included_assets(
                out,
                ["scenes/main.zscene"],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.included_assets does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_trimmed_assets_outside_cook_assets_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "asset_filter": "shipping",
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/editor.png"],
                            "labels": ["shipping"],
                        },
                        {
                            "path": "textures/editor.png",
                            "dependencies": [],
                            "labels": ["editor"],
                        },
                        {
                            "path": "textures/unused.png",
                            "dependencies": [],
                            "labels": ["shipping"],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[
                    {
                        "path": "textures/editor.png",
                        "reason": "Unreferenced",
                    },
                    {
                        "path": "textures/unused.png",
                        "reason": "Unreferenced",
                    },
                ],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.trimmed_assets does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_success_with_cook_assets_missing_dependency(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/missing.png"],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.missing_dependencies does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_success_with_cook_assets_duplicate_asset(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.duplicate_assets does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_trim_diagnostics_outside_cook_assets_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                        {
                            "path": "textures/unused.png",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[
                    {
                        "path": "textures/unused.png",
                        "reason": "Unreferenced",
                    },
                ],
                diagnostics=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.diagnostics does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def write_cook_assets_report(out: Path, manifest: dict[str, object]) -> None:
    report_dir = out / "stages" / "cook_assets"
    manifest_path = report_dir / "assets.json"
    report_dir.mkdir(parents=True, exist_ok=True)
    manifest_contents = json.dumps(manifest, indent=2, sort_keys=True)
    manifest_path.write_text(manifest_contents, encoding="utf-8", newline="\n")
    assets = manifest.get("assets", [])
    roots = manifest.get("roots", [])
    report = {
        "stage": "CookAssets",
        "profile": "windows-release",
        "fatal": False,
        "diagnostics": [],
        "source_asset_manifest": None,
        "project_manifest": None,
        "generated_from_project": False,
        "project_default_scene": None,
        "cooked_asset_manifest": str(manifest_path),
        "cooked_asset_manifest_sha256": hashlib.sha256(
            manifest_contents.encode("utf-8")
        ).hexdigest(),
        "asset_count": len(assets) if isinstance(assets, list) else 0,
        "root_count": len(roots) if isinstance(roots, list) else 0,
        "asset_filter": manifest.get("asset_filter"),
    }
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def rewrite_pack_report_for_included_assets(
    out: Path,
    included_assets: list[str],
) -> None:
    rewrite_pack_report_for_trim_evidence(
        out,
        included_assets=included_assets,
        trimmed_assets=[],
    )


def rewrite_pack_report_for_trim_evidence(
    out: Path,
    *,
    included_assets: list[str],
    trimmed_assets: list[dict[str, object]],
    diagnostics: list[str] | None = None,
) -> None:
    report_path = out / "stages" / "pack" / "report.json"
    report = json_loads(report_path.read_text(encoding="utf-8"))
    assert isinstance(report, dict)
    report["manifest"], payload = pack_manifest_for_included_assets(included_assets)
    pack = report.get("pack")
    if isinstance(pack, str) and pack.strip():
        Path(pack).write_bytes(
            _pack_binary_bytes(report["manifest"], b"ZRPK", payload=payload)
        )
    report["asset_count"] = len(included_assets)
    report["chunk_count"] = len(included_assets)
    report["trim_report"] = {
        "included_assets": included_assets,
        "trimmed_assets": trimmed_assets,
        "missing_dependencies": [],
        "duplicate_assets": [],
        "diagnostics": diagnostics or [],
    }
    report_path.write_text(json_dumps(report), encoding="utf-8")


def pack_manifest_for_included_assets(
    included_assets: list[str],
) -> tuple[dict[str, object], bytes]:
    payload_chunks = [
        pack_payload_for_asset(index)
        for index, _ in enumerate(included_assets)
    ]
    manifest_assets: list[dict[str, object]] = []
    manifest_chunks: list[dict[str, object]] = []
    payload_offset = PACK_BINARY_HEADER_SIZE
    for path, payload in zip(included_assets, payload_chunks, strict=True):
        content_hash = zrpack_content_hash(payload)
        manifest_assets.append(
            {
                "path": path,
                "chunk_hash": content_hash,
                "size": len(payload),
            }
        )
        manifest_chunks.append(
            {
                "hash": content_hash,
                "offset": payload_offset,
                "size": len(payload),
            }
        )
        payload_offset += len(payload)
    return (
        {
            "pack": {
                "version": 1,
                "chunks": manifest_chunks,
                "total_size": sum(len(chunk) for chunk in payload_chunks),
            },
            "assets": manifest_assets,
        },
        b"".join(payload_chunks),
    )


def pack_payload_for_asset(index: int) -> bytes:
    return f"asset-{index}".encode("ascii").ljust(8, b"-")


if __name__ == "__main__":
    unittest.main()

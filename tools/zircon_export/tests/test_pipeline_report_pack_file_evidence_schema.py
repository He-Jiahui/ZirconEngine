from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.pipeline_report_pack_stage_schema import zrpack_content_hash
from tools.zircon_export.tests.export_test_support import (
    _pack_binary_bytes,
)
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    delta_manifest as _delta_manifest,
    pack_manifest as _pack_manifest,
)


def _payload_pack_manifest(
    payload: bytes,
    *,
    path: str = "scenes/main.zscene",
) -> dict[str, object]:
    chunk_hash = zrpack_content_hash(payload)
    return {
        "pack": {
            "version": 1,
            "chunks": [
                {
                    "hash": chunk_hash,
                    "offset": 24,
                    "size": len(payload),
                }
            ],
            "total_size": len(payload),
        },
        "assets": [
            {
                "path": path,
                "chunk_hash": chunk_hash,
                "size": len(payload),
            }
        ],
    }


def _payload_delta_manifest(
    base_manifest: dict[str, object],
    target_manifest: dict[str, object],
) -> dict[str, object]:
    target_pack = target_manifest["pack"]
    target_assets = target_manifest["assets"]
    if not isinstance(target_pack, dict) or not isinstance(target_assets, list):
        raise AssertionError("test target manifest shape is invalid")
    target_chunks = target_pack["chunks"]
    if not isinstance(target_chunks, list):
        raise AssertionError("test target manifest chunks are invalid")
    target_paths = _manifest_asset_paths(target_manifest)
    return {
        "format_version": 1,
        "base": base_manifest,
        "target": target_manifest,
        "chunks": target_chunks,
        "changed_assets": target_assets,
        "removed_assets": [
            path
            for path in _manifest_asset_paths(base_manifest)
            if path not in target_paths
        ],
    }


def _manifest_asset_paths(manifest: dict[str, object]) -> list[str]:
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return []
    return sorted(
        asset["path"]
        for asset in assets
        if isinstance(asset, dict) and isinstance(asset.get("path"), str)
    )


def _trim_report_for_assets(paths: list[str]) -> dict[str, object]:
    return {
        "included_assets": paths,
        "trimmed_assets": [],
        "missing_dependencies": [],
        "duplicate_assets": [],
        "diagnostics": [],
    }


class PipelineReportPackFileEvidenceSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_required_file_missing(self) -> None:
        cases = ("asset_manifest", "pack")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    missing_path = out / "missing" / f"{field}.json"
                    _write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = str(missing_path)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} {missing_path} does not exist",
                    )

    def test_report_stage_rejects_pack_required_file_directory(self) -> None:
        cases = ("asset_manifest", "pack")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    directory_path = out / "not-a-file" / field
                    directory_path.mkdir(parents=True)
                    _write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = str(directory_path)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} {directory_path} is not a file",
                    )

    def test_report_stage_rejects_pack_required_file_empty(self) -> None:
        cases = ("asset_manifest", "pack")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    empty_path = out / "empty" / f"{field}.json"
                    empty_path.parent.mkdir(parents=True)
                    empty_path.write_bytes(b"")
                    _write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = str(empty_path)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} {empty_path} must not be empty",
                    )

    def test_report_stage_rejects_pack_file_invalid_header(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_path = Path(pack_report["pack"])
            pack_path.write_bytes(b"BAD!" + b"\0" * 20)

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report pack {pack_path} header magic is invalid",
            )

    def test_report_stage_rejects_pack_file_embedded_manifest_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(out, manifest=_pack_manifest())

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report pack embedded manifest does not match manifest",
            )

    def test_report_stage_rejects_pack_file_chunk_schema_before_payload_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            manifest = _payload_pack_manifest(payload)
            pack = manifest["pack"]
            self.assertIsInstance(pack, dict)
            chunks = pack["chunks"]
            self.assertIsInstance(chunks, list)
            chunk = chunks[0]
            self.assertIsInstance(chunk, dict)
            chunk["size"] = -1
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_path = Path(pack_report["pack"])
            pack_path.write_bytes(_pack_binary_bytes(manifest, b"ZRPK"))

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks[0].size must be non-negative",
            )
            self.assertFalse(
                any(
                    "manifest offset" in diagnostic
                    or "payload range is out of bounds" in diagnostic
                    or "payload does not match manifest hash" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_file_payload_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            manifest = _payload_pack_manifest(payload)
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_path = Path(pack_report["pack"])
            pack_path.write_bytes(
                _pack_binary_bytes(
                    manifest,
                    b"ZRPK",
                    payload=b"tampered",
                )
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report pack {pack_path} pack.chunks[0] payload does not "
                "match manifest hash",
            )

    def test_report_stage_rejects_pack_file_payload_manifest_gap(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            manifest = _payload_pack_manifest(payload)
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_path = Path(pack_report["pack"])
            pack_path.write_bytes(
                _pack_binary_bytes(
                    manifest,
                    b"ZRPK",
                    payload=payload + b"gap",
                )
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report pack {pack_path} manifest offset 35 does not match "
                "pack.chunks payload end 32",
            )

    def test_report_stage_rejects_pack_file_manifest_trailing_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            manifest = _payload_pack_manifest(payload)
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_path = Path(pack_report["pack"])
            pack_path.write_bytes(
                _pack_binary_bytes(manifest, b"ZRPK", payload=payload) + b"trail"
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report pack {pack_path} manifest end ",
            )

    def test_report_stage_rejects_pack_delta_file_missing(self) -> None:
        cases = ("delta_pack", "previous_pack")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_manifest = _delta_manifest()
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    previous_pack = out / "pack-output" / "previous.zrpack"
                    missing_path = out / "missing" / f"{field}.zrpack"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )
                    delta_pack.parent.mkdir(parents=True, exist_ok=True)
                    delta_pack.write_text("delta pack placeholder", encoding="utf-8")
                    previous_pack.write_text("previous pack placeholder", encoding="utf-8")
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report["delta_pack"] = str(delta_pack)
                    pack_report["previous_pack"] = str(previous_pack)
                    pack_report["delta_apply_verified"] = True
                    pack_report["trim_report"]["included_assets"] = [
                        asset["path"] for asset in delta_manifest["target"]["assets"]
                    ]
                    pack_report[field] = str(missing_path)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} {missing_path} does not exist",
                    )

    def test_report_stage_rejects_pack_delta_file_empty(self) -> None:
        cases = ("delta_pack", "previous_pack")
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_manifest = _delta_manifest()
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    previous_pack = out / "pack-output" / "previous.zrpack"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )
                    delta_pack.parent.mkdir(parents=True, exist_ok=True)
                    delta_pack.write_text("delta pack placeholder", encoding="utf-8")
                    previous_pack.write_text("previous pack placeholder", encoding="utf-8")
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report["delta_pack"] = str(delta_pack)
                    pack_report["previous_pack"] = str(previous_pack)
                    pack_report["delta_apply_verified"] = True
                    pack_report["trim_report"]["included_assets"] = [
                        asset["path"] for asset in delta_manifest["target"]["assets"]
                    ]
                    empty_path = Path(pack_report[field])
                    empty_path.write_bytes(b"")
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(
                        self,
                        report,
                        f"pack report {field} {empty_path} must not be empty",
                    )

    def test_report_stage_rejects_delta_file_embedded_manifest_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_manifest = _delta_manifest()
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(delta_manifest["target"], b"ZRPK")
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(
                    {**delta_manifest, "removed_assets": []},
                    b"ZRPD",
                )
            )
            previous_pack.write_bytes(_pack_binary_bytes(delta_manifest["base"], b"ZRPK"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report["trim_report"]["included_assets"] = [
                asset["path"] for asset in delta_manifest["target"]["assets"]
            ]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_pack embedded manifest does not match delta_manifest",
            )

    def test_report_stage_rejects_delta_file_payload_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            target_manifest = _payload_pack_manifest(payload)
            base_manifest = _payload_pack_manifest(
                payload,
                path="textures/base.png",
            )
            delta_manifest = _payload_delta_manifest(base_manifest, target_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=target_manifest,
                delta_manifest=delta_manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(target_manifest, b"ZRPK", payload=payload)
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(
                    delta_manifest,
                    b"ZRPD",
                    payload=b"tampered",
                )
            )
            previous_pack.write_bytes(
                _pack_binary_bytes(base_manifest, b"ZRPK", payload=payload)
            )
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report delta_pack {delta_pack} chunks[0] payload does not "
                "match manifest hash",
            )

    def test_report_stage_rejects_delta_file_payload_manifest_gap(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            target_manifest = _payload_pack_manifest(payload)
            base_manifest = {
                "pack": {"version": 1, "chunks": [], "total_size": 0},
                "assets": [],
            }
            delta_manifest = _payload_delta_manifest(base_manifest, target_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=target_manifest,
                delta_manifest=delta_manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(target_manifest, b"ZRPK", payload=payload)
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(
                    delta_manifest,
                    b"ZRPD",
                    payload=payload + b"gap",
                )
            )
            previous_pack.write_bytes(_pack_binary_bytes(base_manifest, b"ZRPK"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report delta_pack {delta_pack} manifest offset 35 does not "
                "match chunks payload end 32",
            )

    def test_report_stage_rejects_delta_file_manifest_trailing_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            target_manifest = _payload_pack_manifest(payload)
            base_manifest = {
                "pack": {"version": 1, "chunks": [], "total_size": 0},
                "assets": [],
            }
            delta_manifest = _payload_delta_manifest(base_manifest, target_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=target_manifest,
                delta_manifest=delta_manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(target_manifest, b"ZRPK", payload=payload)
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(delta_manifest, b"ZRPD", payload=payload)
                + b"trail"
            )
            previous_pack.write_bytes(_pack_binary_bytes(base_manifest, b"ZRPK"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report delta_pack {delta_pack} manifest end ",
            )

    def test_report_stage_rejects_previous_pack_embedded_manifest_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_manifest = _delta_manifest()
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(delta_manifest["target"], b"ZRPK")
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(_pack_binary_bytes(delta_manifest, b"ZRPD"))
            previous_pack.write_bytes(
                _pack_binary_bytes(delta_manifest["target"], b"ZRPK")
            )
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report["trim_report"]["included_assets"] = [
                asset["path"] for asset in delta_manifest["target"]["assets"]
            ]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report previous_pack embedded manifest does not match "
                "delta_manifest.base",
            )

    def test_report_stage_rejects_previous_pack_payload_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            payload = b"expected"
            target_manifest = _payload_pack_manifest(payload)
            base_manifest = _payload_pack_manifest(
                payload,
                path="textures/base.png",
            )
            delta_manifest = _payload_delta_manifest(base_manifest, target_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=target_manifest,
                delta_manifest=delta_manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(target_manifest, b"ZRPK", payload=payload)
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(delta_manifest, b"ZRPD", payload=payload)
            )
            previous_pack.write_bytes(
                _pack_binary_bytes(
                    base_manifest,
                    b"ZRPK",
                    payload=b"tampered",
                )
            )
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report previous_pack {previous_pack} pack.chunks[0] payload "
                "does not match manifest hash",
            )

    def test_report_stage_rejects_previous_pack_payload_manifest_gap(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_payload = b"expected"
            base_payload = b"basefile"
            target_manifest = _payload_pack_manifest(target_payload)
            base_manifest = _payload_pack_manifest(
                base_payload,
                path="textures/base.png",
            )
            delta_manifest = _payload_delta_manifest(base_manifest, target_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=target_manifest,
                delta_manifest=delta_manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(
                    target_manifest,
                    b"ZRPK",
                    payload=target_payload,
                )
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(
                    delta_manifest,
                    b"ZRPD",
                    payload=target_payload,
                )
            )
            previous_pack.write_bytes(
                _pack_binary_bytes(
                    base_manifest,
                    b"ZRPK",
                    payload=base_payload + b"gap",
                )
            )
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report previous_pack {previous_pack} manifest offset 35 "
                "does not match pack.chunks payload end 32",
            )

    def test_report_stage_rejects_previous_pack_manifest_trailing_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            target_payload = b"expected"
            base_payload = b"basefile"
            target_manifest = _payload_pack_manifest(target_payload)
            base_manifest = _payload_pack_manifest(
                base_payload,
                path="textures/base.png",
            )
            delta_manifest = _payload_delta_manifest(base_manifest, target_manifest)
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            previous_pack = out / "pack-output" / "previous.zrpack"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=target_manifest,
                delta_manifest=delta_manifest,
                trim_report=_trim_report_for_assets(["scenes/main.zscene"]),
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            Path(pack_report["pack"]).write_bytes(
                _pack_binary_bytes(
                    target_manifest,
                    b"ZRPK",
                    payload=target_payload,
                )
            )
            delta_pack.parent.mkdir(parents=True, exist_ok=True)
            delta_pack.write_bytes(
                _pack_binary_bytes(
                    delta_manifest,
                    b"ZRPD",
                    payload=target_payload,
                )
            )
            previous_pack.write_bytes(
                _pack_binary_bytes(base_manifest, b"ZRPK", payload=base_payload)
                + b"trail"
            )
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_apply_verified"] = True
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                f"pack report previous_pack {previous_pack} manifest end ",
            )


if __name__ == "__main__":
    unittest.main()

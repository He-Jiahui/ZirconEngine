from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    manifest_override as _manifest_override,
    trim_report as _trim_report,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    asset_entry as _asset_entry,
    chunk_entry as _chunk_entry,
    pack_manifest as _pack_manifest,
    pack_plan as _pack_plan,
)


class PipelineReportPackStageSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_stage_output_outside_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_library_embed_reports(out)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["stage_output"] = str(root / "external" / "pack")
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report stage_output "
                f"{root / 'external' / 'pack'} does not match current "
                f"Pack stage directory {pack_report_path.parent}",
            )

    def test_report_stage_rejects_pack_required_path_blank_string(self) -> None:
        cases = (
            (
                "asset_manifest",
                "pack report asset_manifest must be a non-empty string",
            ),
            (
                "pack",
                "pack report pack must be a non-empty string",
            ),
            (
                "stage_output",
                "pack report stage_output must be a non-empty string",
            ),
        )
        for field, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = " "
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_report_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["sidecar"] = "unexpected"
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report unknown field sidecar",
            )

    def test_report_stage_rejects_pack_manifest_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(out, manifest={**_pack_manifest(), "sidecar": "unexpected"})

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest unknown field sidecar",
            )

    def test_report_stage_rejects_pack_manifest_nested_field_types(self) -> None:
        cases = (
            (
                "pack.version",
                _manifest_override({"pack": {**_pack_plan(), "version": "1"}}),
                "pack report manifest.pack.version must be an integer",
            ),
            (
                "pack.total_size",
                _manifest_override({"pack": {**_pack_plan(), "total_size": "8"}}),
                "pack report manifest.pack.total_size must be an integer",
            ),
            (
                "pack.chunks",
                _manifest_override({"pack": {**_pack_plan(), "chunks": ["bad"]}}),
                "pack report manifest.pack.chunks[0] must be an object",
            ),
            (
                "pack.chunks.hash",
                _manifest_override(
                    {
                        "pack": {
                            **_pack_plan(),
                            "chunks": [{**_chunk_entry(), "hash": [1] * 31}],
                        }
                    }
                ),
                "pack report manifest.pack.chunks[0].hash must be a 32-byte integer array",
            ),
            (
                "pack.assets.path",
                _manifest_override(
                    {"assets": [{**_asset_entry(), "path": 42}]}
                ),
                "pack report manifest.assets[0].path must be a string",
            ),
            (
                "pack.assets.chunk_hash",
                _manifest_override(
                    {"assets": [{**_asset_entry(), "chunk_hash": [True] * 32}]}
                ),
                "pack report manifest.assets[0].chunk_hash must be a 32-byte integer array",
            ),
            (
                "pack.assets.size",
                _manifest_override({"assets": [{**_asset_entry(), "size": "8"}]}),
                "pack report manifest.assets[0].size must be an integer",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_missing_required_field(self) -> None:
        cases = (
            (
                "manifest.pack",
                {"assets": []},
                "pack report manifest.pack must be an object",
            ),
            (
                "manifest.assets",
                {"pack": _pack_plan()},
                "pack report manifest.assets must be an object array",
            ),
            (
                "pack.version",
                {"pack": {"chunks": [_chunk_entry()], "total_size": 8}, "assets": []},
                "pack report manifest.pack.version must be an integer",
            ),
            (
                "pack.total_size",
                {"pack": {"version": 1, "chunks": [_chunk_entry()]}, "assets": []},
                "pack report manifest.pack.total_size must be an integer",
            ),
            (
                "pack.chunks",
                {"pack": {"version": 1, "total_size": 0}, "assets": []},
                "pack report manifest.pack.chunks must be an object array",
            ),
            (
                "pack.chunks.hash",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [{"offset": 24, "size": 8}],
                        "total_size": 8,
                    },
                    "assets": [],
                },
                "pack report manifest.pack.chunks[0].hash "
                "must be a 32-byte integer array",
            ),
            (
                "pack.chunks.offset",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [{"hash": [1] * 32, "size": 8}],
                        "total_size": 8,
                    },
                    "assets": [],
                },
                "pack report manifest.pack.chunks[0].offset must be an integer",
            ),
            (
                "pack.chunks.size",
                {
                    "pack": {
                        "version": 1,
                        "chunks": [{"hash": [1] * 32, "offset": 24}],
                        "total_size": 8,
                    },
                    "assets": [],
                },
                "pack report manifest.pack.chunks[0].size must be an integer",
            ),
            (
                "pack.assets.path",
                _manifest_override({"assets": [{"chunk_hash": [1] * 32, "size": 8}]}),
                "pack report manifest.assets[0].path must be a string",
            ),
            (
                "pack.assets.chunk_hash",
                _manifest_override({"assets": [{"path": "scenes/main.zscene", "size": 8}]}),
                "pack report manifest.assets[0].chunk_hash "
                "must be a 32-byte integer array",
            ),
            (
                "pack.assets.size",
                _manifest_override(
                    {"assets": [{"path": "scenes/main.zscene", "chunk_hash": [1] * 32}]}
                ),
                "pack report manifest.assets[0].size must be an integer",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_asset_empty_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=_manifest_override(
                    {"assets": [{**_asset_entry(), "path": " "}]}
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets[0].path must be a non-empty string",
            )

    def test_report_stage_rejects_pack_manifest_asset_path_shape(self) -> None:
        cases = (
            (
                "unsafe",
                _manifest_override(
                    {"assets": [{**_asset_entry(), "path": "../escape.png"}]}
                ),
                "pack report manifest.assets[0].path "
                "must be a safe relative asset path",
            ),
            (
                "unnormalized",
                _manifest_override(
                    {"assets": [{**_asset_entry(), "path": "textures\\hero.png"}]}
                ),
                "pack report manifest.assets[0].path "
                "must use a normalized relative asset path",
            ),
            (
                "duplicate",
                {
                    "pack": _pack_plan(hash_value=1),
                    "assets": [
                        _asset_entry(hash_value=1, path="scenes/main.zscene"),
                        _asset_entry(hash_value=1, path="scenes/main.zscene"),
                    ],
                },
                "pack report manifest.assets path scenes/main.zscene "
                "is declared more than once",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_negative_layout_numbers(self) -> None:
        cases = (
            (
                "pack.total_size",
                _manifest_override({"pack": {**_pack_plan(), "total_size": -1}}),
                "pack report manifest.pack.total_size must be non-negative",
            ),
            (
                "pack.chunks.offset",
                _manifest_override(
                    {"pack": {**_pack_plan(), "chunks": [{**_chunk_entry(), "offset": -1}]}}
                ),
                "pack report manifest.pack.chunks[0].offset must be non-negative",
            ),
            (
                "pack.chunks.size",
                _manifest_override(
                    {"pack": {**_pack_plan(), "chunks": [{**_chunk_entry(), "size": -1}]}}
                ),
                "pack report manifest.pack.chunks[0].size must be non-negative",
            ),
            (
                "pack.assets.size",
                _manifest_override({"assets": [{**_asset_entry(), "size": -1}]}),
                "pack report manifest.assets[0].size must be non-negative",
            ),
        )
        for label, manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {**_pack_plan(hash_value=1), "version": 2},
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.version 2 is not supported; expected 1",
            )

    def test_report_stage_rejects_pack_manifest_count_mismatch(self) -> None:
        cases = (
            (
                "asset_count",
                99,
                "pack report asset_count 99 does not match manifest.assets length 1",
            ),
            (
                "chunk_count",
                99,
                "pack report chunk_count 99 does not match manifest.pack.chunks length 1",
            ),
        )
        for field, value, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=_pack_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = value
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_report_negative_counts(self) -> None:
        cases = (
            (
                "asset_count",
                "pack report asset_count must be non-negative",
            ),
            (
                "chunk_count",
                "pack report chunk_count must be non-negative",
            ),
        )
        for field, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, manifest=_pack_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = -1
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_manifest_duplicate_chunk_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            _chunk_entry(hash_value=1),
                            {**_chunk_entry(hash_value=1), "offset": 32},
                        ],
                        "total_size": 16,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks contains duplicate chunk hash",
            )

    def test_report_stage_rejects_pack_manifest_unsorted_chunk_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            _chunk_entry(hash_value=2),
                            {**_chunk_entry(hash_value=1), "offset": 32},
                        ],
                        "total_size": 16,
                    },
                    "assets": [_asset_entry(hash_value=2)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks must be sorted by chunk hash",
            )

    def test_report_stage_rejects_pack_manifest_unsorted_assets(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            _chunk_entry(hash_value=1),
                            {**_chunk_entry(hash_value=2), "offset": 32},
                        ],
                        "total_size": 16,
                    },
                    "assets": [
                        _asset_entry(hash_value=2, path="textures/hero.png"),
                        _asset_entry(hash_value=1, path="scenes/main.zscene"),
                    ],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets must be sorted by asset path",
            )

    def test_report_stage_rejects_pack_manifest_asset_missing_chunk_hash(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": _pack_plan(hash_value=1),
                    "assets": [_asset_entry(hash_value=2)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets[0].chunk_hash "
                "is not present in pack report manifest.pack.chunks",
            )

    def test_report_stage_rejects_pack_manifest_asset_chunk_size_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": _pack_plan(hash_value=1),
                    "assets": [{**_asset_entry(hash_value=1), "size": 99}],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.assets[0].size 99 does not match "
                "pack report manifest.pack.chunks size 8",
            )

    def test_report_stage_rejects_pack_manifest_total_size_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {**_pack_plan(hash_value=1), "total_size": 99},
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.total_size 99 does not match "
                "pack report manifest.pack.chunks size sum 8",
            )

    def test_report_stage_rejects_pack_manifest_chunk_offset_gap(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {**_chunk_entry(hash_value=1), "offset": 40},
                        ],
                        "total_size": 8,
                    },
                    "assets": [_asset_entry(hash_value=1)],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report manifest.pack.chunks[0].offset 40 does not match "
                "expected chunk offset 24",
            )

    def test_report_stage_rejects_pack_deduplicated_assets_manifest_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest={
                    **_pack_manifest(),
                    "assets": [
                        _asset_entry(path="scenes/main.zscene"),
                        _asset_entry(path="textures/copy.png"),
                    ],
                },
                trim_report={
                    **_trim_report(),
                    "included_assets": [
                        "scenes/main.zscene",
                        "textures/copy.png",
                    ],
                },
                deduplicated_assets=[],
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report deduplicated_assets does not match "
                "manifest duplicate chunk paths",
            )

    def test_report_stage_rejects_pack_deduplicated_assets_blank_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(out, deduplicated_assets=[" "])

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report deduplicated_assets must not contain blank entries",
            )

    def test_report_stage_rejects_pack_deduplicated_assets_path_shape(
        self,
    ) -> None:
        cases = (
            (
                "unsafe",
                ["../copy.png"],
                "pack report deduplicated_assets[0] "
                "must be a safe relative asset path",
            ),
            (
                "unnormalized",
                ["textures\\copy.png"],
                "pack report deduplicated_assets[0] "
                "must use a normalized relative asset path",
            ),
            (
                "duplicate",
                ["textures/copy.png", "textures/copy.png"],
                "pack report deduplicated_assets path textures/copy.png "
                "is declared more than once",
            ),
        )
        for label, deduplicated_assets, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        deduplicated_assets=deduplicated_assets,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

if __name__ == "__main__":
    unittest.main()

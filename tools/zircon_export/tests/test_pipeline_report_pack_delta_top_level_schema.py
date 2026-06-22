from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import (
    assert_pack_schema_diagnostic as _assert_pack_schema_diagnostic,
    manifest_override as _manifest_override,
    sync_delta_report_counts as _sync_delta_report_counts,
    update_pack_report as _update_pack_report,
    write_library_embed_reports as _write_library_embed_reports,
)
from tools.zircon_export.tests.pack_test_support import (
    delta_manifest as _delta_manifest,
    pack_plan as _pack_plan,
)


class PipelineReportPackDeltaTopLevelSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_delta_manifest_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                delta_manifest={**_delta_manifest(), "sidecar": "unexpected"},
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_manifest unknown field sidecar",
            )

    def test_report_stage_rejects_pack_delta_manifest_field_types(self) -> None:
        cases = (
            (
                "format_version",
                {**_delta_manifest(), "format_version": "1"},
                "pack report delta_manifest.format_version must be an integer",
            ),
            (
                "base",
                {**_delta_manifest(), "base": "bad"},
                "pack report delta_manifest.base must be an object",
            ),
            (
                "target.pack.version",
                {
                    **_delta_manifest(),
                    "target": _manifest_override(
                        {"pack": {**_pack_plan(hash_value=2), "version": "1"}},
                        hash_value=2,
                    ),
                },
                "pack report delta_manifest.target.pack.version must be an integer",
            ),
            (
                "chunks",
                {**_delta_manifest(), "chunks": ["bad"]},
                "pack report delta_manifest.chunks[0] must be an object",
            ),
            (
                "changed_assets",
                {**_delta_manifest(), "changed_assets": ["bad"]},
                "pack report delta_manifest.changed_assets[0] must be an object",
            ),
            (
                "removed_assets",
                {**_delta_manifest(), "removed_assets": ["old.scene", 42]},
                "pack report delta_manifest.removed_assets[1] must be a string",
            ),
        )
        for label, delta_manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=delta_manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_asset_list_non_string_entry_before_array_shape(
        self,
    ) -> None:
        cases = (
            (
                "delta_removed_assets",
                {"delta_removed_assets": ["textures/old.png", 42]},
                "pack report delta_removed_assets[1] must be a string",
                ("pack report delta_removed_assets must be a string array",),
            ),
            (
                "delta_reused_assets",
                {"delta_reused_assets": ["textures/reused.png", False]},
                "pack report delta_reused_assets[1] must be a string",
                ("pack report delta_reused_assets must be a string array",),
            ),
            (
                "delta_manifest.removed_assets",
                {
                    "delta_manifest": {
                        **_delta_manifest(),
                        "removed_assets": ["textures/old.png", None],
                    }
                },
                "pack report delta_manifest.removed_assets[1] must be a string",
                (
                    "pack report delta_manifest.removed_assets must be a string array",
                    "pack report delta_manifest.target does not match manifest",
                    "pack report delta_removed_assets does not match "
                    "delta_manifest.removed_assets",
                ),
            ),
        )
        for label, overrides, expected, unexpected_fragments in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=_delta_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.update(overrides)
                    if "delta_manifest" in overrides:
                        _sync_delta_report_counts(pack_report)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)
                    for unexpected in unexpected_fragments:
                        self.assertFalse(
                            any(
                                unexpected in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_pack_delta_manifest_missing_required_field(
        self,
    ) -> None:
        cases = (
            (
                "format_version",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "format_version"
                },
                "pack report delta_manifest.format_version must be an integer",
            ),
            (
                "base",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "base"
                },
                "pack report delta_manifest.base must be an object",
            ),
            (
                "target",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "target"
                },
                "pack report delta_manifest.target must be an object",
            ),
            (
                "chunks",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "chunks"
                },
                "pack report delta_manifest.chunks must be an object array",
            ),
            (
                "changed_assets",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "changed_assets"
                },
                "pack report delta_manifest.changed_assets must be an object array",
            ),
            (
                "removed_assets",
                {
                    key: value
                    for key, value in _delta_manifest().items()
                    if key != "removed_assets"
                },
                "pack report delta_manifest.removed_assets must be a string array",
            ),
        )
        for label, delta_manifest, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=delta_manifest)

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_path_array_blank_entry(self) -> None:
        cases = (
            (
                "delta_removed_assets",
                {"delta_removed_assets": [" "]},
                "pack report delta_removed_assets must not contain blank entries",
            ),
            (
                "delta_reused_assets",
                {"delta_reused_assets": [" "]},
                "pack report delta_reused_assets must not contain blank entries",
            ),
            (
                "delta_manifest.removed_assets",
                {"delta_manifest": {**_delta_manifest(), "removed_assets": [" "]}},
                "pack report delta_manifest.removed_assets must not contain blank entries",
            ),
        )
        for label, overrides, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=_delta_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.update(overrides)
                    if "delta_manifest" in overrides:
                        _sync_delta_report_counts(pack_report)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_path_array_shape(self) -> None:
        cases = (
            (
                "delta_removed_assets.unsafe",
                {"delta_removed_assets": ["../old.png"]},
                "pack report delta_removed_assets[0] "
                "must be a safe relative asset path",
            ),
            (
                "delta_reused_assets.unnormalized",
                {"delta_reused_assets": ["textures\\reused.png"]},
                "pack report delta_reused_assets[0] "
                "must use a normalized relative asset path",
            ),
            (
                "delta_reused_assets.duplicate",
                {
                    "delta_reused_assets": [
                        "textures/reused.png",
                        "textures/reused.png",
                    ]
                },
                "pack report delta_reused_assets path textures/reused.png "
                "is declared more than once",
            ),
        )
        for label, overrides, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=_delta_manifest())
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.update(overrides)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_missing_report_audit_field(
        self,
    ) -> None:
        cases = (
            (
                "delta_asset_count",
                "pack report delta_asset_count must be an integer",
            ),
            (
                "delta_chunk_count",
                "pack report delta_chunk_count must be an integer",
            ),
            (
                "delta_removed_assets",
                "pack report delta_removed_assets must be a string array",
            ),
            (
                "delta_reused_assets",
                "pack report delta_reused_assets must be a string array",
            ),
        )
        for field, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    delta_manifest = _delta_manifest()
                    _write_library_embed_reports(out)
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report["delta_pack"] = str(delta_pack)
                    pack_report["delta_apply_verified"] = True
                    pack_report["trim_report"]["included_assets"] = [
                        asset["path"] for asset in delta_manifest["target"]["assets"]
                    ]
                    pack_report.pop(field)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_unverified_apply(self) -> None:
        cases = (
            (
                "missing",
                None,
                "pack report delta_apply_verified must be a boolean",
            ),
            (
                "false",
                False,
                "pack report delta_apply_verified must be true when delta_pack is published",
            ),
        )
        for label, value, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    previous_pack = out / "pack-output" / "previous.zrpack"
                    _write_library_embed_reports(out)
                    delta_manifest = _delta_manifest()
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report["delta_pack"] = str(delta_pack)
                    pack_report["previous_pack"] = str(previous_pack)
                    if value is None:
                        pack_report.pop("delta_apply_verified", None)
                    else:
                        pack_report["delta_apply_verified"] = value
                    pack_report["trim_report"]["included_assets"] = [
                        asset["path"] for asset in delta_manifest["target"]["assets"]
                    ]
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_optional_path_blank_string(
        self,
    ) -> None:
        cases = (
            (
                "delta_pack.empty",
                "delta_pack",
                "",
                "pack report delta_pack must be a non-empty string",
            ),
            (
                "delta_pack.whitespace",
                "delta_pack",
                " ",
                "pack report delta_pack must be a non-empty string",
            ),
            (
                "previous_pack.whitespace",
                "previous_pack",
                " ",
                "pack report previous_pack must be a non-empty string",
            ),
        )
        for label, field, value, expected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
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

    def test_report_stage_rejects_pack_delta_unpaired_previous_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_library_embed_reports(out)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["previous_pack"] = str(out / "pack-output" / "previous.zrpack")
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report previous_pack is present but delta_pack is missing",
            )

    def test_report_stage_rejects_pack_delta_negative_counts(self) -> None:
        cases = (
            (
                "delta_asset_count",
                "pack report delta_asset_count must be non-negative",
            ),
            (
                "delta_chunk_count",
                "pack report delta_chunk_count must be non-negative",
            ),
        )
        for field, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    delta_pack = out / "pack-output" / "assets.delta.zrpd"
                    previous_pack = out / "pack-output" / "previous.zrpack"
                    _write_library_embed_reports(out)
                    delta_manifest = _delta_manifest()
                    _update_pack_report(
                        out,
                        manifest=delta_manifest["target"],
                        delta_manifest=delta_manifest,
                    )
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
                    pack_report[field] = -1
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_missing_previous_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            delta_manifest = _delta_manifest()
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["delta_apply_verified"] = True
            pack_report["trim_report"]["included_assets"] = [
                asset["path"] for asset in delta_manifest["target"]["assets"]
            ]
            pack_report.pop("previous_pack", None)
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report previous_pack must be a string",
            )

    def test_report_stage_rejects_pack_delta_blank_previous_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            delta_manifest = _delta_manifest()
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["previous_pack"] = " "
            pack_report["delta_pack"] = str(delta_pack)
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
                "pack report previous_pack must be a non-empty string",
            )

    def test_report_stage_rejects_pack_delta_blank_delta_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            previous_pack = out / "pack-output" / "previous.zrpack"
            delta_manifest = _delta_manifest()
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=delta_manifest["target"],
                delta_manifest=delta_manifest,
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["previous_pack"] = str(previous_pack)
            pack_report["delta_pack"] = " "
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
                "pack report delta_pack must be a non-empty string",
            )


if __name__ == "__main__":
    unittest.main()

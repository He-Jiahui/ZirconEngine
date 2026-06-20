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
    asset_entry as _asset_entry,
    chunk_entry as _chunk_entry,
    delta_manifest as _delta_manifest,
    manifest_for_assets as _manifest_for_assets,
    pack_manifest as _pack_manifest,
    pack_plan as _pack_plan,
)


class PipelineReportPackDeltaSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_pack_delta_manifest_asset_empty_path(self) -> None:
        cases = (
            (
                "base.assets",
                {
                    **_delta_manifest(),
                    "base": _manifest_override(
                        {"assets": [{**_asset_entry(hash_value=1), "path": " "}]}
                    ),
                },
                "pack report delta_manifest.base.assets[0].path "
                "must be a non-empty string",
            ),
            (
                "target.assets",
                {
                    **_delta_manifest(),
                    "target": _manifest_override(
                        {"assets": [{**_asset_entry(hash_value=2), "path": " "}]},
                        hash_value=2,
                    ),
                },
                "pack report delta_manifest.target.assets[0].path "
                "must be a non-empty string",
            ),
            (
                "changed_assets",
                {
                    **_delta_manifest(),
                    "changed_assets": [{**_asset_entry(hash_value=2), "path": " "}],
                },
                "pack report delta_manifest.changed_assets[0].path "
                "must be a non-empty string",
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

    def test_report_stage_rejects_pack_delta_manifest_version_mismatch(
        self,
    ) -> None:
        target = _pack_manifest(hash_value=2)
        cases = (
            (
                "format_version",
                {**_delta_manifest(), "format_version": 2, "target": target},
                target,
                "pack report delta_manifest.format_version 2 is not supported; "
                "expected 1",
            ),
            (
                "base.pack.version",
                {
                    **_delta_manifest(),
                    "base": {
                        "pack": {**_pack_plan(hash_value=1), "version": 2},
                        "assets": [_asset_entry(hash_value=1)],
                    },
                    "target": target,
                    "changed_assets": [_asset_entry(hash_value=2)],
                    "chunks": [_chunk_entry(hash_value=2)],
                    "removed_assets": [],
                },
                target,
                "pack report delta_manifest.base.pack.version 2 is not supported; "
                "expected 1",
            ),
            (
                "target.pack.version",
                {
                    **_delta_manifest(),
                    "target": {
                        "pack": {**_pack_plan(hash_value=2), "version": 2},
                        "assets": [_asset_entry(hash_value=2)],
                    },
                    "changed_assets": [_asset_entry(hash_value=2)],
                    "chunks": [_chunk_entry(hash_value=2)],
                    "removed_assets": ["textures/old.png"],
                },
                {
                    "pack": {**_pack_plan(hash_value=2), "version": 2},
                    "assets": [_asset_entry(hash_value=2)],
                },
                "pack report delta_manifest.target.pack.version 2 is not supported; "
                "expected 1",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_count_mismatch(self) -> None:
        cases = (
            (
                "delta_asset_count",
                99,
                "pack report delta_asset_count 99 does not match "
                "delta_manifest.changed_assets length 1",
            ),
            (
                "delta_chunk_count",
                99,
                "pack report delta_chunk_count 99 does not match "
                "delta_manifest.chunks length 1",
            ),
        )
        for field, value, expected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_library_embed_reports(out)
                    _update_pack_report(out, delta_manifest=_delta_manifest())
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

    def test_report_stage_rejects_pack_delta_manifest_duplicate_chunk_hash(
        self,
    ) -> None:
        target = _manifest_for_assets(
            [
                _asset_entry(hash_value=2, path="scenes/main.zscene"),
                _asset_entry(hash_value=1, path="textures/reused.png"),
            ],
            hash_values=[1, 1],
        )
        cases = (
            (
                "target.pack.chunks",
                {
                    **_delta_manifest(),
                    "target": target,
                    "changed_assets": [],
                    "chunks": [],
                },
                target,
                "pack report delta_manifest.target.pack.chunks "
                "contains duplicate chunk hash",
            ),
            (
                "chunks",
                {
                    **_delta_manifest(),
                    "chunks": [
                        _chunk_entry(hash_value=2),
                        {**_chunk_entry(hash_value=2), "offset": 32},
                    ],
                },
                _pack_manifest(hash_value=2),
                "pack report delta_manifest.chunks contains duplicate chunk hash",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_unsorted_chunk_hash(
        self,
    ) -> None:
        target = _manifest_for_assets(
            [
                _asset_entry(hash_value=2, path="scenes/main.zscene"),
                _asset_entry(hash_value=1, path="textures/reused.png"),
            ],
            hash_values=[2, 1],
        )
        cases = (
            (
                "target.pack.chunks",
                {**_delta_manifest(), "target": target},
                target,
                "pack report delta_manifest.target.pack.chunks "
                "must be sorted by chunk hash",
            ),
            (
                "chunks",
                {
                    **_delta_manifest(),
                    "chunks": [
                        _chunk_entry(hash_value=3),
                        {**_chunk_entry(hash_value=2), "offset": 32},
                    ],
                },
                _pack_manifest(hash_value=2),
                "pack report delta_manifest.chunks must be sorted by chunk hash",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_asset_missing_chunk_hash(
        self,
    ) -> None:
        target = _manifest_for_assets(
            [
                _asset_entry(hash_value=2, path="scenes/main.zscene"),
            ],
            hash_values=[1],
        )
        cases = (
            (
                "base.assets",
                {
                    **_delta_manifest(),
                    "base": _manifest_for_assets(
                        [_asset_entry(hash_value=4)],
                        hash_values=[1],
                    ),
                },
                _pack_manifest(hash_value=2),
                "pack report delta_manifest.base.assets[0].chunk_hash "
                "is not present in pack report delta_manifest.base.pack.chunks",
            ),
            (
                "target.assets",
                {
                    **_delta_manifest(),
                    "target": target,
                    "changed_assets": [],
                    "chunks": [],
                    "removed_assets": ["textures/old.png"],
                },
                target,
                "pack report delta_manifest.target.assets[0].chunk_hash "
                "is not present in pack report delta_manifest.target.pack.chunks",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_asset_chunk_size_mismatch(
        self,
    ) -> None:
        base_with_size_mismatch = _manifest_for_assets(
            [
                {**_asset_entry(hash_value=1, path="scenes/main.zscene"), "size": 99},
                _asset_entry(hash_value=3, path="textures/old.png"),
            ],
            hash_values=[1, 3],
        )
        target = _pack_manifest(hash_value=2)
        cases = (
            (
                "base.assets",
                {
                    **_delta_manifest(),
                    "base": base_with_size_mismatch,
                    "target": target,
                    "changed_assets": [_asset_entry(hash_value=2)],
                    "chunks": [_chunk_entry(hash_value=2)],
                    "removed_assets": ["textures/old.png"],
                },
                target,
                "pack report delta_manifest.base.assets[0].size 99 "
                "does not match pack report delta_manifest.base.pack.chunks size 8",
            ),
            (
                "target.assets",
                {
                    **_delta_manifest(),
                    "target": _manifest_for_assets(
                        [{**_asset_entry(hash_value=2), "size": 99}],
                        hash_values=[2],
                    ),
                    "changed_assets": [{**_asset_entry(hash_value=2), "size": 99}],
                    "chunks": [_chunk_entry(hash_value=2)],
                    "removed_assets": ["textures/old.png"],
                },
                {
                    "pack": _pack_plan(hash_value=2),
                    "assets": [{**_asset_entry(hash_value=2), "size": 99}],
                },
                "pack report delta_manifest.target.assets[0].size 99 "
                "does not match pack report delta_manifest.target.pack.chunks size 8",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_payload_asset_chunk_size_mismatch(
        self,
    ) -> None:
        target = {
            "pack": {
                "version": 1,
                "chunks": [{**_chunk_entry(hash_value=2), "size": 99}],
                "total_size": 99,
            },
            "assets": [{**_asset_entry(hash_value=2), "size": 99}],
        }
        delta_manifest = {
            **_delta_manifest(),
            "target": target,
            "changed_assets": [{**_asset_entry(hash_value=2), "size": 99}],
            "chunks": [_chunk_entry(hash_value=2)],
            "removed_assets": ["textures/old.png"],
        }

        _assert_delta_pack_schema_diagnostic(
            self,
            delta_manifest=delta_manifest,
            manifest=target,
            expected=(
                "pack report delta_manifest.changed_assets[0].size 99 "
                "does not match pack report delta_manifest.chunks size 8"
            ),
        )

    def test_report_stage_rejects_pack_delta_manifest_total_size_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "base.pack.total_size",
                {
                    **_delta_manifest(),
                    "base": {
                        **_manifest_for_assets(
                            [_asset_entry(hash_value=1)],
                            hash_values=[1],
                        ),
                        "pack": {
                            **_pack_plan(hash_value=1),
                            "total_size": 99,
                        },
                    },
                },
                _pack_manifest(hash_value=2),
                "pack report delta_manifest.base.pack.total_size 99 "
                "does not match pack report delta_manifest.base.pack.chunks size sum 8",
            ),
            (
                "target.pack.total_size",
                {
                    **_delta_manifest(),
                    "target": {
                        **_manifest_for_assets(
                            [_asset_entry(hash_value=2)],
                            hash_values=[2],
                        ),
                        "pack": {
                            **_pack_plan(hash_value=2),
                            "total_size": 99,
                        },
                    },
                    "changed_assets": [_asset_entry(hash_value=2)],
                    "chunks": [_chunk_entry(hash_value=2)],
                    "removed_assets": ["textures/old.png"],
                },
                {
                    "pack": {
                        **_pack_plan(hash_value=2),
                        "total_size": 99,
                    },
                    "assets": [_asset_entry(hash_value=2)],
                },
                "pack report delta_manifest.target.pack.total_size 99 "
                "does not match pack report delta_manifest.target.pack.chunks size sum 8",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_chunk_offset_gap(
        self,
    ) -> None:
        cases = (
            (
                "base.pack.chunks",
                {
                    **_delta_manifest(),
                    "base": {
                        "pack": {
                            "version": 1,
                            "chunks": [
                                {**_chunk_entry(hash_value=1), "offset": 40},
                            ],
                            "total_size": 8,
                        },
                        "assets": [_asset_entry(hash_value=1)],
                    },
                },
                _pack_manifest(hash_value=2),
                "pack report delta_manifest.base.pack.chunks[0].offset 40 "
                "does not match expected chunk offset 24",
            ),
            (
                "target.pack.chunks",
                {
                    **_delta_manifest(),
                    "target": {
                        "pack": {
                            "version": 1,
                            "chunks": [
                                {**_chunk_entry(hash_value=2), "offset": 40},
                            ],
                            "total_size": 8,
                        },
                        "assets": [_asset_entry(hash_value=2)],
                    },
                    "changed_assets": [_asset_entry(hash_value=2)],
                    "chunks": [_chunk_entry(hash_value=2)],
                    "removed_assets": ["textures/old.png"],
                },
                {
                    "pack": {
                        "version": 1,
                        "chunks": [
                            {**_chunk_entry(hash_value=2), "offset": 40},
                        ],
                        "total_size": 8,
                    },
                    "assets": [_asset_entry(hash_value=2)],
                },
                "pack report delta_manifest.target.pack.chunks[0].offset 40 "
                "does not match expected chunk offset 24",
            ),
        )
        for label, delta_manifest, manifest, expected in cases:
            with self.subTest(label=label):
                _assert_delta_pack_schema_diagnostic(
                    self,
                    delta_manifest=delta_manifest,
                    manifest=manifest,
                    expected=expected,
                )

    def test_report_stage_rejects_pack_delta_manifest_payload_chunk_offset_gap(
        self,
    ) -> None:
        target = _pack_manifest(hash_value=2)
        delta_manifest = {
            **_delta_manifest(),
            "target": target,
            "chunks": [{**_chunk_entry(hash_value=2), "offset": 40}],
            "changed_assets": [_asset_entry(hash_value=2)],
            "removed_assets": ["textures/old.png"],
        }

        _assert_delta_pack_schema_diagnostic(
            self,
            delta_manifest=delta_manifest,
            manifest=target,
            expected=(
                "pack report delta_manifest.chunks[0].offset 40 "
                "does not match expected chunk offset 24"
            ),
        )

    def test_report_stage_rejects_pack_delta_asset_set_mismatch(self) -> None:
        cases = (
            (
                "delta_removed_assets",
                {"delta_removed_assets": []},
                "pack report delta_removed_assets does not match "
                "delta_manifest.removed_assets",
            ),
            (
                "delta_reused_assets",
                {"delta_reused_assets": []},
                "pack report delta_reused_assets does not match "
                "delta_manifest target assets reused from base chunks",
            ),
            (
                "delta_manifest.removed_assets",
                {"delta_manifest": {**_delta_manifest(), "removed_assets": []}},
                "pack report delta_manifest.removed_assets does not match "
                "base/target asset path difference",
            ),
            (
                "delta_manifest.changed_assets",
                {"delta_manifest": {**_delta_manifest(), "changed_assets": []}},
                "pack report delta_manifest.changed_assets does not match "
                "target assets missing from base chunks",
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

    def test_report_stage_rejects_pack_delta_changed_asset_manifest_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "changed_assets.entry",
                {
                    "delta_manifest": {
                        **_delta_manifest(),
                        "changed_assets": [
                            {**_asset_entry(hash_value=2), "size": 99}
                        ],
                    }
                },
                "pack report delta_manifest.changed_assets does not match "
                "target manifest asset entries",
            ),
            (
                "chunks",
                {
                    "delta_manifest": {
                        **_delta_manifest(),
                        "chunks": [_chunk_entry(hash_value=4)],
                    }
                },
                "pack report delta_manifest.chunks does not match "
                "changed asset chunk hashes",
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
                    _sync_delta_report_counts(pack_report)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)

    def test_report_stage_rejects_pack_delta_target_manifest_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            delta_pack = out / "pack-output" / "assets.delta.zrpd"
            _write_library_embed_reports(out)
            _update_pack_report(
                out,
                manifest=_pack_manifest(hash_value=2),
                delta_manifest={
                    **_delta_manifest(),
                    "target": _manifest_for_assets(
                        [
                            _asset_entry(
                                hash_value=2,
                                path="scenes/not-shipped.zscene",
                            )
                        ],
                        hash_values=[2],
                    ),
                },
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["delta_pack"] = str(delta_pack)
            pack_report["delta_apply_verified"] = True
            _sync_delta_report_counts(pack_report)
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            _assert_pack_schema_diagnostic(
                self,
                report,
                "pack report delta_manifest.target does not match manifest",
            )

    def test_report_stage_rejects_pack_delta_manifest_publication_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "delta_pack_without_manifest",
                {
                    "delta_pack": "pack-output/assets.delta.zrpd",
                    "delta_manifest": None,
                    "delta_apply_verified": True,
                },
                "pack report delta_pack is present but delta_manifest is missing",
            ),
            (
                "manifest_without_delta_pack",
                {
                    "delta_pack": None,
                    "delta_manifest": _delta_manifest(),
                    "delta_apply_verified": True,
                },
                "pack report delta_manifest is present but delta_pack is missing",
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
                    _sync_delta_report_counts(pack_report)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    _assert_pack_schema_diagnostic(self, report, expected)


def _assert_delta_pack_schema_diagnostic(
    test_case: unittest.TestCase,
    *,
    delta_manifest: dict[str, object],
    manifest: dict[str, object],
    expected: str,
) -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        out = Path(temp_dir) / "out"
        delta_pack = out / "pack-output" / "assets.delta.zrpd"
        _write_library_embed_reports(out)
        _update_pack_report(
            out,
            manifest=manifest,
            delta_manifest=delta_manifest,
        )
        pack_report_path = out / "stages" / "pack" / "report.json"
        pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
        pack_report["delta_pack"] = str(delta_pack)
        pack_report["delta_apply_verified"] = True
        _sync_delta_report_counts(pack_report)
        pack_report_path.write_text(
            json.dumps(pack_report, indent=2),
            encoding="utf-8",
        )

        report = build_pipeline_report(out, "windows-release")

        _assert_pack_schema_diagnostic(test_case, report, expected)


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import run_pipeline, run_stage
from tools.zircon_export.native_dynamic_payload import (
    native_dynamic_content_hash,
    native_dynamic_plugins_file_manifest,
)
from tools.zircon_export.platform_bundle import (
    validate_report_allows_native_plugins,
    validate_report_strategy_diagnostics,
    validate_report_uses_strategy,
)
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _compile_host_link_plan,
    _pack_binary_bytes,
    _write_validate_report_with_strategies,
)
from tools.zircon_export.tests.pack_test_support import (
    empty_delta_manifest,
    empty_pack_document_manifest,
)


REPO_ROOT = Path(__file__).resolve().parents[3]


class PlatformBundleInputTests(unittest.TestCase):
    def test_stage_rejects_empty_explicit_handoff_inputs(self) -> None:
        for field, diagnostic in PLATFORM_BUNDLE_EMPTY_ARGUMENTS.items():
            for value in ("", "   "):
                with self.subTest(field=field, value=repr(value)):
                    self.assert_empty_argument_rejected(
                        field,
                        value,
                        diagnostic,
                        pipeline=False,
                    )

    def test_pipeline_preserves_empty_explicit_handoff_inputs(self) -> None:
        for field, diagnostic in PLATFORM_BUNDLE_EMPTY_ARGUMENTS.items():
            for value in ("", "   "):
                with self.subTest(field=field, value=repr(value)):
                    self.assert_empty_argument_rejected(
                        field,
                        value,
                        diagnostic,
                        pipeline=True,
                    )

    def test_pipeline_explicit_pack_file_does_not_inherit_report_delta(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
            explicit_pack = root / "manual" / "manual-assets.zrpack"
            report_pack = root / "pack-output" / "assets.zrpack"
            report_delta = root / "pack-output" / "assets.delta.zrpd"
            for file_path, contents in (
                (host, "host placeholder"),
                (explicit_pack, "explicit pack"),
                (report_pack, "report pack"),
                (report_delta, "report delta"),
            ):
                file_path.parent.mkdir(parents=True, exist_ok=True)
                file_path.write_text(contents, encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, report_pack, report_delta)
            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            args.pack_file = str(explicit_pack)
            args.pack_file_explicit = True

            exit_code = _run_pipeline_quiet(args, "platform_bundle")

            report = _read_json(out / "stages" / "platform_bundle" / "report.json")
            bundle_manifest = _read_json(out / "bundle" / "windows-release" / "bundle.json")
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertEqual(Path(report["pack"]).read_text(encoding="utf-8"), "explicit pack")
            self.assertEqual(Path(report["pack_source"]), explicit_pack)
            self.assertEqual(report["pack_source_origin"], "argument")
            self.assertEqual(Path(bundle_manifest["pack_source"]), explicit_pack)
            self.assertEqual(bundle_manifest["pack_source_origin"], "argument")
            self.assertIsNone(report["delta_pack"])
            self.assertIsNone(bundle_manifest["delta_pack"])
            self.assertFalse(
                (out / "bundle" / "windows-release" / report_delta.name).exists()
            )

    def test_validate_strategy_helpers_reject_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            report_path = out / "stages" / "validate" / "report.json"
            report_path.mkdir(parents=True)

            self.assertFalse(
                validate_report_uses_strategy(out, "windows-release", "native_dynamic")
            )
            self.assertFalse(
                validate_report_allows_native_plugins(out, "windows-release")
            )
            self.assertEqual(
                validate_report_strategy_diagnostics(out, "windows-release"),
                [f"Validate report {report_path} is not a file"],
            )

    def assert_empty_argument_rejected(
        self,
        field: str,
        value: str,
        diagnostic: str,
        *,
        pipeline: bool,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            delta_pack = root / "pack-output" / "assets.delta.zrpd"
            host.parent.mkdir(parents=True)
            pack.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            delta_pack.write_text("delta placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, delta_pack)
            _write_native_dynamic_report(out, native_plugins)
            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            setattr(args, field, value)
            if field == "pack_file":
                args.pack_file_explicit = True
            if field == "delta_pack":
                args.delta_pack_explicit = True

            exit_code = (
                _run_pipeline_quiet(args, "platform_bundle")
                if pipeline
                else _run_stage_quiet(args)
            )

            report = _read_json(out / "stages" / "platform_bundle" / "report.json")
            self.assertEqual(exit_code, 2)
            self.assertEqual(getattr(args, field), value)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["host_executable"])
            self.assertIsNone(report["pack"])
            self.assertIsNone(report["delta_pack"])
            self.assertIsNone(report["native_plugins"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(diagnostic in item for item in report["diagnostics"]),
                report["diagnostics"],
            )


PLATFORM_BUNDLE_EMPTY_ARGUMENTS = {
    "host_executable": "host_executable argument must be a non-empty string",
    "pack_file": "pack_file argument must be a non-empty string",
    "delta_pack": "delta_pack argument must be a non-empty string",
    "native_plugins_dir": "native_plugins_dir argument must be a non-empty string",
}


def _run_stage_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_stage(args)


def _run_pipeline_quiet(args: argparse.Namespace, resume_from: str) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_pipeline(args, resume_from)


def _read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_stage_report(out: Path, stage: str, *, fatal: bool) -> None:
    if stage == "validate" and not fatal:
        _write_validate_report_with_strategies(out, ["library_embed"])
        return
    if stage == "cook_assets" and not fatal:
        _write_cook_assets_report(out)
        return
    report_dir = out / "stages" / stage
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": stage,
                "profile": "windows-release",
                "fatal": fatal,
                "diagnostics": [],
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_cook_assets_report(out: Path) -> None:
    report_dir = out / "stages" / "cook_assets"
    cooked_manifest = report_dir / "assets.json"
    report_dir.mkdir(parents=True, exist_ok=True)
    cooked_manifest.write_text(
        json.dumps({"roots": [], "assets": []}, indent=2),
        encoding="utf-8",
    )
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": "CookAssets",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "source_asset_manifest": None,
                "project_manifest": None,
                "generated_from_project": False,
                "project_default_scene": None,
                "cooked_asset_manifest": str(cooked_manifest),
                "cooked_asset_manifest_sha256": hashlib.sha256(
                    cooked_manifest.read_bytes()
                ).hexdigest(),
                "asset_count": 0,
                "root_count": 0,
                "asset_filter": None,
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_compile_host_report(out: Path, host_executable: Path) -> None:
    report_dir = out / "stages" / "compile_host"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": "CompileHost",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "command": list(_compile_host_plan()["command"]),
                "exit_code": 0,
                "host_executable": str(host_executable),
                "link_plan": _compile_host_link_plan(),
                "stdout_lines": [],
                "stderr_lines": [],
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_pack_report(out: Path, pack: Path, delta_pack: Path) -> None:
    report_dir = out / "stages" / "pack"
    report_dir.mkdir(parents=True, exist_ok=True)
    previous_pack = delta_pack.with_name("previous.zrpack")
    pack.parent.mkdir(parents=True, exist_ok=True)
    pack.write_bytes(_pack_binary_bytes(empty_pack_document_manifest(), b"ZRPK"))
    delta_pack.parent.mkdir(parents=True, exist_ok=True)
    delta_pack.write_bytes(_pack_binary_bytes(empty_delta_manifest(), b"ZRPD"))
    previous_pack.write_bytes(
        _pack_binary_bytes(empty_pack_document_manifest(), b"ZRPK")
    )
    asset_manifest = out / "stages" / "cook_assets" / "assets.json"
    asset_manifest.parent.mkdir(parents=True, exist_ok=True)
    if not asset_manifest.exists():
        asset_manifest.write_text(
            json.dumps({"roots": [], "assets": []}, indent=2),
            encoding="utf-8",
        )
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": "Pack",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "asset_manifest": str(asset_manifest),
                "pack": str(pack),
                "stage_output": str(report_dir),
                "trim_report": {
                    "included_assets": [],
                    "trimmed_assets": [],
                    "missing_dependencies": [],
                    "duplicate_assets": [],
                    "diagnostics": [],
                },
                "manifest": {
                    "pack": {
                        "version": 1,
                        "chunks": [],
                        "total_size": 0,
                    },
                    "assets": [],
                },
                "asset_count": 0,
                "chunk_count": 0,
                "deduplicated_assets": [],
                "deterministic_double_run": False,
                "previous_pack": str(previous_pack),
                "delta_pack": str(delta_pack),
                "delta_manifest": empty_delta_manifest(),
                "delta_asset_count": 0,
                "delta_chunk_count": 0,
                "delta_removed_assets": [],
                "delta_reused_assets": [],
                "delta_apply_verified": True,
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_native_dynamic_stage_plugins(stage_dir: Path) -> Path:
    plugins_dir = stage_dir / "plugins"
    package = plugins_dir / "animation"
    (package / "native").mkdir(parents=True)
    (plugins_dir / "native_plugins.toml").write_text(
        "[[plugins]]\nid = \"animation\"\npackage = \"animation\"\n",
        encoding="utf-8",
    )
    (package / "native" / "zircon_plugin_animation.dll").write_text(
        "native dynamic placeholder",
        encoding="utf-8",
    )
    return plugins_dir


def _write_native_dynamic_report(out: Path, plugins_dir: Path) -> None:
    report_dir = out / "stages" / "native_dynamic"
    report_dir.mkdir(parents=True, exist_ok=True)
    file_manifest = native_dynamic_plugins_file_manifest(
        plugins_dir.parent,
        plugins_dir,
    )
    report_dir.joinpath("report.json").write_text(
        json.dumps(
            {
                "stage": "NativeDynamic",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "plugins_dir": str(plugins_dir),
                "loader_manifest": str(plugins_dir / "native_plugins.toml"),
                "file_manifest": file_manifest,
                "content_hash": native_dynamic_content_hash(file_manifest),
                "materialized_packages": [
                    {
                        "package_id": "animation",
                        "destination": str(plugins_dir / "animation"),
                        "loadable_artifact_count": 1,
                        "loadable_artifacts": [
                            "plugins/animation/native/zircon_plugin_animation.dll"
                        ],
                    }
                ],
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _export_args(*, out: Path, stage: str, dry_run: bool) -> argparse.Namespace:
    return argparse.Namespace(
        profile="windows-release",
        project="zircon-project.toml",
        out=str(out),
        stage=stage,
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=None,
        native_plugin_root=None,
        asset_manifest=None,
        asset_filter=None,
        pack_file=None,
        previous_pack=None,
        delta_pack=None,
        host_executable=None,
        native_plugins_dir=None,
        template_dir=None,
        template_root=None,
        engine_version="0.1.0",
        target_platform=None,
        determinism_check=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=dry_run,
        stage_explicit=True,
        resume_from_explicit=False,
        pack_file_explicit=False,
        delta_pack_explicit=False,
        source_template_build=False,
    )


if __name__ == "__main__":
    unittest.main()

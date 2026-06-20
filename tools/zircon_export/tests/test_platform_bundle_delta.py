from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import json
import shutil
import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.zircon_export.cli import run_pipeline
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _compile_host_link_plan,
    _write_validate_report_with_strategies,
)
from tools.zircon_export.tests.pack_test_support import empty_delta_manifest


REPO_ROOT = Path(__file__).resolve().parents[3]
VALID_TEMPLATE = REPO_ROOT / "export-templates" / "windows-x86_64-library_embed-debug"


class PlatformBundleDeltaTests(unittest.TestCase):
    def test_pipeline_platform_bundle_uses_pack_report_delta_pack_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            delta_pack = root / "pack-output" / "custom-assets.delta.zrpd"
            delta_pack.write_text("delta pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, delta_pack)

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            bundled_delta = out / "bundle" / "windows-release" / delta_pack.name
            platform_report = _read_json(
                out / "stages" / "platform_bundle" / "report.json"
            )
            bundle_manifest = _read_json(out / "bundle" / "windows-release" / "bundle.json")
            pipeline_report = _read_json(out / "report.json")
            self.assertEqual(exit_code, 0, pipeline_report["diagnostics"])
            self.assertTrue(bundled_delta.exists())
            self.assertEqual(Path(platform_report["pack_source"]), pack)
            self.assertEqual(platform_report["pack_source_origin"], "pack_report")
            self.assertEqual(Path(bundle_manifest["pack_source"]), pack)
            self.assertEqual(bundle_manifest["pack_source_origin"], "pack_report")
            self.assertEqual(Path(platform_report["delta_pack"]), bundled_delta)
            self.assertEqual(Path(bundle_manifest["delta_pack"]), bundled_delta)
            self.assertEqual(Path(platform_report["delta_pack_source"]), delta_pack)
            self.assertEqual(platform_report["delta_pack_source_origin"], "pack_report")
            self.assertEqual(Path(bundle_manifest["delta_pack_source"]), delta_pack)
            self.assertEqual(
                bundle_manifest["delta_pack_source_origin"],
                "pack_report",
            )
            self.assertFalse(pipeline_report["fatal"], pipeline_report["diagnostics"])

    def test_template_delta_pack_path_controls_bundle_location(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            template_dir = root / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            _set_template_delta_pack_path(
                template_dir / "template.toml",
                "patches/custom-assets.delta.zrpd",
            )
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            delta_pack = root / "pack-output" / "custom-assets.delta.zrpd"
            delta_pack.write_text("delta pack placeholder", encoding="utf-8")
            host = out / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, delta_pack)
            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            args.template_dir = str(template_dir)
            args.target_platform = "windows-x86_64"

            exit_code = _run_pipeline_quiet(args, "platform_bundle")

            bundled_delta = (
                out
                / "bundle"
                / "windows-release"
                / "patches"
                / "custom-assets.delta.zrpd"
            )
            platform_report = _read_json(
                out / "stages" / "platform_bundle" / "report.json"
            )
            self.assertEqual(exit_code, 0, platform_report["diagnostics"])
            self.assertTrue(bundled_delta.exists())
            self.assertEqual(Path(platform_report["delta_pack"]), bundled_delta)

    def test_checked_in_windows_template_routes_delta_pack_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            delta_pack = root / "pack-output" / "custom-assets.delta.zrpd"
            delta_pack.write_text("delta pack placeholder", encoding="utf-8")
            host = out / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, delta_pack)
            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            args.template_dir = str(VALID_TEMPLATE)
            args.target_platform = "windows-x86_64"

            exit_code = _run_pipeline_quiet(args, "platform_bundle")

            template_manifest = tomllib.loads(
                (VALID_TEMPLATE / "template.toml").read_text(encoding="utf-8")
            )
            expected_delta_path = (
                out
                / "bundle"
                / "windows-release"
                / template_manifest["bundle"]["delta_pack_path"]
            )
            platform_report = _read_json(
                out / "stages" / "platform_bundle" / "report.json"
            )
            bundle_manifest = _read_json(out / "bundle" / "windows-release" / "bundle.json")
            self.assertEqual(exit_code, 0, platform_report["diagnostics"])
            self.assertTrue(expected_delta_path.exists())
            self.assertEqual(Path(platform_report["delta_pack"]), expected_delta_path)
            self.assertEqual(Path(bundle_manifest["delta_pack"]), expected_delta_path)


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
    previous_pack.write_text("previous pack placeholder", encoding="utf-8")
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


def _set_template_delta_pack_path(manifest: Path, delta_pack_path: str) -> None:
    lines = manifest.read_text(encoding="utf-8").splitlines()
    updated: list[str] = []
    replaced = False
    in_bundle = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            if in_bundle and not replaced:
                updated.append(f'delta_pack_path = "{delta_pack_path}"')
                replaced = True
            in_bundle = stripped == "[bundle]"
        if in_bundle and stripped.startswith("delta_pack_path"):
            updated.append(f'delta_pack_path = "{delta_pack_path}"')
            replaced = True
            continue
        updated.append(line)
    if in_bundle and not replaced:
        updated.append(f'delta_pack_path = "{delta_pack_path}"')
        replaced = True
    if not replaced:
        updated.extend(["", "[bundle]", f'delta_pack_path = "{delta_pack_path}"'])
    manifest.write_text("\n".join(updated) + "\n", encoding="utf-8")


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
        source_template_build=False,
    )

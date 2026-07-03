from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
from pathlib import Path

from tools.zircon_export.cli import (
    run_compile_host,
    run_cook_assets,
    run_pipeline,
    run_platform_bundle,
    run_report,
    run_source_template,
    run_stage,
)
from tools.zircon_export.pack_stage import run_pack
from tools.zircon_export.tests import native_dynamic_export_test_support as native_dynamic_support
from tools.zircon_export.tests import platform_bundle_export_test_support as platform_bundle_support
from tools.zircon_export.tests.pack_test_support import (
    empty_delta_manifest,
    empty_pack_document_manifest,
)


REPO_ROOT = Path(__file__).resolve().parents[3]
TEMPLATE_ROOT = REPO_ROOT / "tools" / "zircon_export" / "export-templates"
VALID_TEMPLATE = TEMPLATE_ROOT / "windows-x86_64-library_embed-debug"
LINUX_TEMPLATE = TEMPLATE_ROOT / "linux-x86_64-library_embed-debug"
MACOS_TEMPLATE = TEMPLATE_ROOT / "macos-aarch64-library_embed-debug"


def _template_content_hash(path: str, sha256: str, *, bundle_path: str | None = None) -> str:
    hasher = hashlib.sha256()
    hasher.update(path.encode("utf-8"))
    hasher.update(b"\0")
    hasher.update((bundle_path or path).encode("utf-8"))
    hasher.update(b"\0")
    hasher.update(sha256.lower().encode("ascii"))
    hasher.update(b"\n")
    return hasher.hexdigest()


def _append_template_file_entry(template_dir: Path, *, path: str, sha256: str) -> None:
    manifest = template_dir / "template.toml"
    manifest_text = manifest.read_text(encoding="utf-8")
    entries = [{"path": "bin/zircon_runtime.host-placeholder", "sha256": _file_sha256(template_dir / "bin" / "zircon_runtime.host-placeholder")}]
    entries.append({"path": path, "sha256": sha256})
    hasher = hashlib.sha256()
    for entry in sorted(entries, key=lambda value: value["path"]):
        hasher.update(entry["path"].encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry["path"].encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry["sha256"].lower().encode("ascii"))
        hasher.update(b"\n")
    manifest_text = manifest_text.replace(
        'content_hash = "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"',
        f'content_hash = "{hasher.hexdigest()}"',
    )
    manifest_text += (
        "\n[[files]]\n"
        f'path = "{path}"\n'
        'purpose = "test stale template plugin cleanup"\n'
        f'sha256 = "{sha256}"\n'
    )
    manifest.write_text(manifest_text, encoding="utf-8")


def _file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()



def _platform_bundle_args(
    *,
    out: Path,
    profile: str,
    template_dir: Path | None,
    template_root: Path | None = None,
    pack_file: Path,
    target_platform: str,
) -> argparse.Namespace:
    return argparse.Namespace(
        profile=profile,
        project="zircon-project.toml",
        out=str(out),
        stage="platform_bundle",
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=None,
        native_plugin_root=None,
        asset_manifest=None,
        pack_file=str(pack_file),
        previous_pack=None,
        delta_pack=None,
        host_executable=None,
        host_executable_explicit=False,
        host_executable_source_origin=None,
        native_plugins_dir=None,
        template_dir=str(template_dir) if template_dir else None,
        template_root=str(template_root) if template_root else None,
        engine_version="0.1.0",
        target_platform=target_platform,
        determinism_check=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=False,
        pack_file_explicit=True,
        delta_pack_explicit=False,
    )


def _run_platform_bundle_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_platform_bundle(args)


def _run_cook_assets_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_cook_assets(args)


def _run_pack_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_pack(args)


def _run_report_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_report(args)


def _run_source_template_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_source_template(args)


def _run_stage_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_stage(args)


def _run_pipeline_quiet(args: argparse.Namespace, resume_from: str) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_pipeline(args, resume_from)


def _cook_assets_args(
    *,
    out: Path,
    asset_manifest: Path | None = None,
    project: Path | None = None,
) -> argparse.Namespace:
    args = _export_args(
        out=out,
        stage="cook_assets",
        asset_manifest=asset_manifest,
        dry_run=False,
    )
    if project is not None:
        args.project = str(project)
    return args


def _pack_args(*, out: Path, dry_run: bool = True) -> argparse.Namespace:
    return _export_args(out=out, stage="pack", dry_run=dry_run)


def _report_args(*, out: Path) -> argparse.Namespace:
    return _export_args(out=out, stage="report", dry_run=False)


def _source_template_args(
    *,
    out: Path,
    validate_report: Path | None = None,
    build: bool = False,
    dry_run: bool = True,
) -> argparse.Namespace:
    args = _export_args(out=out, stage="source_template", dry_run=dry_run)
    args.validate_report = str(validate_report) if validate_report else None
    args.source_template_build = build
    return args


def _export_args(
    *,
    out: Path,
    stage: str,
    asset_manifest: Path | None = None,
    dry_run: bool,
) -> argparse.Namespace:
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
        asset_manifest=str(asset_manifest) if asset_manifest else None,
        asset_filter=None,
        pack_file=None,
        previous_pack=None,
        delta_pack=None,
        host_executable=None,
        host_executable_explicit=False,
        host_executable_source_origin=None,
        native_plugins_dir=None,
        template_dir=None,
        template_root=None,
        engine_version="0.1.0",
        target_platform=None,
        determinism_check=False,
        source_template_build=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=dry_run,
        pack_file_explicit=False,
        delta_pack_explicit=False,
    )


def _compile_host_args(
    *,
    out: Path,
    validate_report: Path | None = None,
) -> argparse.Namespace:
    return argparse.Namespace(
        profile="windows-release",
        project="zircon-project.toml",
        out=str(out),
        stage="compile_host",
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=str(validate_report) if validate_report else None,
        native_plugin_root=None,
        asset_manifest=None,
        asset_filter=None,
        pack_file=None,
        host_executable=None,
        native_plugins_dir=None,
        template_dir=None,
        template_root=None,
        engine_version="0.1.0",
        target_platform=None,
        determinism_check=False,
        source_template_build=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=True,
    )


def _compile_host_plan() -> dict[str, object]:
    return {
        "package": "zircon_app",
        "binary": "zircon_runtime",
        "manifest_path": "Cargo.toml",
        "target_dir": "stages/compile_host/target",
        "cargo_profile": "debug",
        "release": False,
        "app_features": ["target-client"],
        "runtime_features": ["target-client"],
        "expected_runtime_plugins": [],
        "linked_runtime_crates": [],
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "Cargo.toml",
            "-p",
            "zircon_app",
            "--bin",
            "zircon_runtime",
            "--no-default-features",
            "--features",
            "target-client",
            "--target-dir",
            "stages/compile_host/target",
        ],
    }


def _compile_host_link_plan() -> dict[str, object]:
    compile_host_plan = _compile_host_plan()
    return {
        "app_features": compile_host_plan["app_features"],
        "runtime_features": compile_host_plan["runtime_features"],
        "expected_runtime_plugins": compile_host_plan["expected_runtime_plugins"],
        "linked_runtime_crates": compile_host_plan["linked_runtime_crates"],
    }


def _source_template_plan() -> dict[str, object]:
    return {
        "manifest_path": "Cargo.toml",
        "target_dir": "stages/source_template/target",
        "cargo_profile": "debug",
        "release": False,
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "Cargo.toml",
            "--target-dir",
            "stages/source_template/target",
        ],
    }


def _validate_runtime_plugin_availability() -> dict[str, object]:
    return {
        "available": [],
        "linked": [],
        "native_dynamic": [],
        "externalized_missing": [],
        "stub": [],
        "blocked_by_target": [],
        "blocked_by_maturity": [],
        "missing_required": [],
    }


def _validate_base_plan_summary() -> dict[str, object]:
    return {
        "enabled_runtime_plugins": [],
        "linked_runtime_crates": [],
        "native_dynamic_packages": [],
        "generated_files": [],
        "runtime_plugin_availability": _validate_runtime_plugin_availability(),
    }


def _validate_profile_summary(
    strategies: object,
    *,
    profile: str,
) -> dict[str, object]:
    return {
        "name": profile,
        "target_mode": "client_runtime",
        "target_platform": "windows-x86_64",
        "build_mode": "debug",
        "strategies": strategies,
        "selected_plugins": [],
        "features": {},
    }


def _validate_plan_summary_for_strategies(strategies: object) -> dict[str, object]:
    plan_summary = _validate_base_plan_summary()
    if export_strategy_is_selected(strategies, "library_embed"):
        plan_summary["library_embed_compile_host"] = _compile_host_plan()
    if export_strategy_is_selected(strategies, "source_template"):
        source_template_plan_summary = _source_template_validate_report()[
            "plan_summary"
        ]
        plan_summary.update(source_template_plan_summary)
    if export_strategy_is_selected(strategies, "native_dynamic"):
        plan_summary["native_dynamic_packages"] = ["animation"]
        plan_summary["native_dynamic_package_exports"] = [
            native_dynamic_support._native_dynamic_package_export()
        ]
    return plan_summary


def _source_template_validate_report() -> dict[str, object]:
    return {
        "stage": "Validate",
        "profile": "windows-release",
        "project_manifest": "zircon-project.toml",
        "stage_output": "stages/validate",
        "profile_found": True,
        "fatal": False,
        "diagnostics": [],
        "fatal_diagnostics": [],
        "profile_summary": _validate_profile_summary(
            ["source_template"],
            profile="windows-release",
        ),
        "plan_summary": {
            **_validate_base_plan_summary(),
            "source_template_build": _source_template_plan(),
            "generated_files": [
                {
                    "path": "Cargo.toml",
                    "purpose": "generated runtime package manifest",
                    "contents": (
                        "[package]\n"
                        "name = \"source-template-smoke\"\n"
                        "version = \"0.1.0\"\n"
                        "edition = \"2021\"\n\n"
                        "[dependencies]\n"
                        "zircon_app = { path = \"../../zircon_app\", default-features = false }\n"
                    ),
                },
                {
                    "path": "src/main.rs",
                    "purpose": "generated runtime entrypoint",
                    "contents": "fn main() {}\n",
                },
            ],
        },
    }


def _run_compile_host_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_compile_host(args)


def json_dumps(value: object) -> str:
    import json

    return json.dumps(value, indent=2)


def json_loads(value: str) -> object:
    import json

    return json.loads(value)


def _write_stage_report(
    out: Path,
    stage: str,
    *,
    fatal: bool,
    profile: str | None = "windows-release",
) -> None:
    if stage == "validate" and profile is not None and not fatal:
        _write_validate_report_with_strategies(out, ["library_embed"], profile=profile)
        return
    if stage == "compile_host" and profile is not None and not fatal:
        _write_compile_host_report(
            out,
            out / "stages" / "compile_host" / "zircon_runtime.exe",
            profile=profile,
        )
        return
    if stage == "cook_assets" and profile is not None and not fatal:
        _write_cook_assets_report(
            out,
            out / "stages" / "cook_assets" / "assets.json",
            profile=profile,
        )
        return
    if stage == "pack" and profile is not None and not fatal:
        _write_pack_report(
            out,
            out / "stages" / "pack" / "assets.zrpack",
            profile=profile,
        )
        return
    if stage == "native_dynamic" and profile is not None and not fatal:
        plugins_dir = native_dynamic_support._write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        native_dynamic_support._write_native_dynamic_report(out, plugins_dir, profile=profile)
        return
    if stage == "platform_bundle" and profile is not None:
        platform_bundle_support._write_platform_bundle_report_with_native_plugins_payload(
            out,
            {},
            fatal=fatal,
            profile=profile,
        )
        return
    if stage == "source_template" and profile is not None:
        _write_source_template_report(out, fatal=fatal, profile=profile)
        return
    report_dir = out / "stages" / stage
    report_dir.mkdir(parents=True, exist_ok=True)
    report: dict[str, object] = {
        "stage": _stage_label(stage),
        "fatal": fatal,
        "diagnostics": ["fatal smoke"] if fatal else [],
    }
    if profile is not None:
        report["profile"] = profile
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def _write_source_template_report(
    out: Path,
    *,
    fatal: bool = False,
    profile: str = "windows-release",
    report_overrides: dict[str, object] | None = None,
) -> Path:
    report_dir = out / "stages" / "source_template"
    project_dir = report_dir / "project"
    generated_files = [
        {
            "path": "Cargo.toml",
            "purpose": "generated runtime package manifest",
        },
        {
            "path": "src/main.rs",
            "purpose": "generated runtime entrypoint",
        },
    ]
    if not fatal:
        (project_dir / "src").mkdir(parents=True, exist_ok=True)
        (project_dir / "Cargo.toml").write_text(
            (
                "[package]\n"
                "name = \"source-template-smoke\"\n"
                "version = \"0.1.0\"\n"
                "edition = \"2021\"\n"
            ),
            encoding="utf-8",
        )
        (project_dir / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
    else:
        project_dir.mkdir(parents=True, exist_ok=True)
    for file in generated_files:
        output = project_dir / str(file["path"])
        if output.exists() and output.is_file():
            contents = output.read_bytes()
            file["size"] = len(contents)
            file["sha256"] = hashlib.sha256(contents).hexdigest()
    build_command = [
        "cargo",
        "build",
        "--manifest-path",
        str(project_dir / "Cargo.toml"),
        "--target-dir",
        str(report_dir / "target"),
    ]
    report: dict[str, object] = {
        "stage": "SourceTemplate",
        "profile": profile,
        "fatal": fatal,
        "diagnostics": ["fatal smoke"] if fatal else [],
        "validate_report": str(out / "stages" / "validate" / "report.json"),
        "project": str(project_dir),
        "generated_files": generated_files,
        "command": build_command,
        "build_executed": True,
        "build_validation": {
            "requested": True,
            "executed": True,
            "status": "passed",
            "exit_code": 0,
            "working_dir": str(project_dir),
            "command": build_command,
            "stdout_lines": ["Finished dev profile"],
            "stderr_lines": [],
        },
        "project_cleaned": False,
        "cleanup_reason": None,
    }
    if report_overrides:
        report.update(report_overrides)
    report_dir.joinpath("report.json").write_text(json_dumps(report), encoding="utf-8")
    return project_dir


def _write_validate_report_with_asset_filter(out: Path, asset_filter: object) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "Validate",
                "profile": "windows-release",
                "project_manifest": str(out / "zircon-project.toml"),
                "stage_output": str(report_dir),
                "profile_found": True,
                "fatal": False,
                "diagnostics": [],
                "fatal_diagnostics": [],
                "profile_summary": {
                    **_validate_profile_summary(
                        ["library_embed"],
                        profile="windows-release",
                    ),
                    "asset_filter": asset_filter,
                },
                "plan_summary": _validate_plan_summary_for_strategies(
                    ["library_embed"]
                ),
            }
        ),
        encoding="utf-8",
    )


def _write_validate_report_with_strategies(
    out: Path,
    strategies: list[str],
    *,
    profile: str = "windows-release",
) -> None:
    _write_validate_report_with_strategies_value(out, strategies, profile=profile)


def _write_validate_report_with_strategies_value(
    out: Path,
    strategies: object,
    *,
    profile: str = "windows-release",
) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    report: dict[str, object] = {
        "stage": "Validate",
        "profile": profile,
        "project_manifest": str(out / "zircon-project.toml"),
        "stage_output": str(report_dir),
        "profile_found": True,
        "fatal": False,
        "diagnostics": [],
        "fatal_diagnostics": [],
        "profile_summary": _validate_profile_summary(strategies, profile=profile),
        "plan_summary": _validate_plan_summary_for_strategies(strategies),
    }
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def export_strategy_is_selected(strategies: object, expected: str) -> bool:
    if not isinstance(strategies, list):
        return False
    return any(
        isinstance(strategy, str)
        and strategy.strip().replace("-", "_").lower() == expected
        for strategy in strategies
    )


def source_template_strategy_is_selected(strategies: object) -> bool:
    return export_strategy_is_selected(strategies, "source_template")


def _write_validate_report_with_native_dynamic_exports(
    out: Path,
    package_export_overrides: dict[str, object] | None = None,
    extra_package_exports: list[dict[str, object]] | None = None,
    native_dynamic_packages: list[str] | None = None,
) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    package_exports = [
        native_dynamic_support._native_dynamic_package_export(package_export_overrides)
    ]
    if extra_package_exports:
        package_exports.extend(extra_package_exports)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "Validate",
                "profile": "windows-release",
                "project_manifest": str(out / "zircon-project.toml"),
                "stage_output": str(report_dir),
                "profile_found": True,
                "fatal": False,
                "diagnostics": [],
                "fatal_diagnostics": [],
                "profile_summary": {
                    **_validate_profile_summary(
                        ["native_dynamic"],
                        profile="windows-release",
                    ),
                },
                "plan_summary": {
                    **_validate_base_plan_summary(),
                    "native_dynamic_packages": (
                        native_dynamic_packages
                        if native_dynamic_packages is not None
                        else ["animation"]
                    ),
                    "native_dynamic_package_exports": package_exports,
                },
            }
        ),
        encoding="utf-8",
    )



def _write_compile_host_report(
    out: Path,
    host_executable: Path,
    *,
    profile: str | None = "windows-release",
    host_value: object | None = None,
) -> None:
    report_dir = out / "stages" / "compile_host"
    report_dir.mkdir(parents=True, exist_ok=True)
    if host_value is None:
        host_executable.parent.mkdir(parents=True, exist_ok=True)
        if not host_executable.exists():
            host_executable.write_text("host placeholder", encoding="utf-8")
    compile_host_plan = _compile_host_plan()
    report: dict[str, object] = {
        "stage": "CompileHost",
        "fatal": False,
        "diagnostics": [],
        "command": list(compile_host_plan["command"]),
        "exit_code": 0,
        "host_executable": str(host_executable) if host_value is None else host_value,
        "link_plan": _compile_host_link_plan(),
        "stdout_lines": [],
        "stderr_lines": [],
    }
    if profile is not None:
        report["profile"] = profile
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def _write_cook_assets_report(
    out: Path,
    cooked_manifest: Path,
    *,
    profile: str | None = "windows-release",
    manifest_value: object | None = None,
) -> None:
    report_dir = out / "stages" / "cook_assets"
    report_dir.mkdir(parents=True, exist_ok=True)
    if manifest_value is None:
        cooked_manifest.parent.mkdir(parents=True, exist_ok=True)
        if not cooked_manifest.exists():
            cooked_manifest.write_text(
                json_dumps(
                    {
                        "roots": [],
                        "assets": [],
                    }
                ),
                encoding="utf-8",
            )
    report: dict[str, object] = {
        "stage": "CookAssets",
        "fatal": False,
        "diagnostics": [],
        "source_asset_manifest": None,
        "project_manifest": None,
        "generated_from_project": False,
        "project_default_scene": None,
        "cooked_asset_manifest": (
            str(cooked_manifest) if manifest_value is None else manifest_value
        ),
        "cooked_asset_manifest_sha256": (
            _file_sha256(cooked_manifest) if manifest_value is None else "0" * 64
        ),
        "asset_count": 0,
        "root_count": 0,
        "asset_filter": None,
    }
    if profile is not None:
        report["profile"] = profile
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def _write_pack_report(
    out: Path,
    pack: Path,
    *,
    delta_pack: Path | None = None,
    delta_apply_verified: bool | None = None,
    profile: str | None = "windows-release",
    pack_value: object | None = None,
    delta_pack_value: object | None = None,
) -> None:
    report_dir = out / "stages" / "pack"
    report_dir.mkdir(parents=True, exist_ok=True)
    asset_manifest = out / "stages" / "cook_assets" / "assets.json"
    asset_manifest.parent.mkdir(parents=True, exist_ok=True)
    if not asset_manifest.exists():
        asset_manifest.write_text(
            json_dumps(
                {
                    "roots": [],
                    "assets": [],
                }
            ),
            encoding="utf-8",
        )
    if pack_value is None:
        pack.parent.mkdir(parents=True, exist_ok=True)
        pack.write_bytes(_pack_binary_bytes(empty_pack_document_manifest(), b"ZRPK"))
    if delta_pack is not None and delta_pack_value is None:
        delta_pack.parent.mkdir(parents=True, exist_ok=True)
        delta_pack.write_bytes(_pack_binary_bytes(empty_delta_manifest(), b"ZRPD"))
    previous_pack = None
    if delta_pack is not None:
        previous_pack = delta_pack.with_name("previous.zrpack")
        previous_pack.write_bytes(
            _pack_binary_bytes(empty_pack_document_manifest(), b"ZRPK")
        )
    delta_manifest = empty_delta_manifest() if delta_pack is not None else None
    report: dict[str, object] = {
        "stage": "Pack",
        "fatal": False,
        "diagnostics": [],
        "asset_manifest": str(asset_manifest),
        "pack": str(pack) if pack_value is None else pack_value,
        "previous_pack": str(previous_pack) if previous_pack else None,
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
        "delta_manifest": delta_manifest,
        "delta_asset_count": 0,
        "delta_chunk_count": 0,
        "delta_removed_assets": [],
        "delta_reused_assets": [],
        "delta_apply_verified": False,
    }
    if profile is not None:
        report["profile"] = profile
    if delta_pack_value is not None:
        report["delta_pack"] = delta_pack_value
    elif delta_pack is not None:
        report["delta_pack"] = str(delta_pack)
    if delta_apply_verified is not None:
        report["delta_apply_verified"] = delta_apply_verified
    report_dir.joinpath("report.json").write_text(json_dumps(report), encoding="utf-8")


def _pack_binary_bytes(
    manifest: object,
    magic: bytes,
    *,
    payload: bytes = b"",
) -> bytes:
    manifest_bytes = json_dumps(manifest).encode("utf-8")
    header = bytearray(24)
    header[0:4] = magic
    header[4:8] = (1).to_bytes(4, "little")
    header[8:16] = (24 + len(payload)).to_bytes(8, "little")
    header[16:24] = len(manifest_bytes).to_bytes(8, "little")
    return bytes(header) + payload + manifest_bytes






def _stage_label(stage: str) -> str:
    return "".join(part.capitalize() for part in stage.split("_"))


def _default_cooked_manifest(out: Path) -> Path:
    return out / "stages" / "cook_assets" / "assets.json"

#!/usr/bin/env python3
"""Stage-oriented Zircon export pipeline command."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any, Sequence

from .cook_assets import default_cooked_asset_manifest, run_cook_assets
from .pipeline_report import run_report
from .pipeline_stages import (
    LIBRARY_EMBED_EXECUTION_STAGES,
    pipeline_stages_after_validate as selected_pipeline_stages_after_validate,
    pipeline_stages_from_resume as selected_pipeline_stages_from_resume,
)
from .native_dynamic import native_dynamic_stage_payload_summary, run_native_dynamic
from .source_template import run_source_template


STAGES = (
    "validate",
    "compile_host",
    "source_template",
    "native_dynamic",
    "cook_assets",
    "pack",
    "platform_bundle",
    "report",
)
RESUMABLE_STAGES = (
    "validate",
    "source_template",
    "native_dynamic",
    "compile_host",
    "cook_assets",
    "pack",
    "platform_bundle",
    "report",
)
DEFAULT_EXECUTION_STAGES = LIBRARY_EMBED_EXECUTION_STAGES
DEFAULT_OUT = "zircon-export"
REPORT_FILE_NAME = "report.json"
EXPORT_TEMPLATE_FORMAT_VERSION = 1
EXPORT_TEMPLATE_MANIFEST_NAME = "template.toml"
EXPORT_TEMPLATE_ALLOWED_HOST_KINDS = {"desktop", "mobile_app", "browser", "headless"}
EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES = {
    "filesystem_bundle",
    "mobile_asset_bundle",
    "browser_fetch",
}
EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES = {
    "native_dynamic_allowed",
    "static_source_or_vm_only",
}
EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS = {
    "directory",
    "app_bundle",
    "zip",
    "web_static",
}


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    if args.resume_from or not args.stage_explicit:
        return run_pipeline(args, args.resume_from or "validate")
    return run_stage(args)


def run_stage(args: argparse.Namespace) -> int:
    if args.stage == "validate":
        return run_validate(args)
    if args.stage == "compile_host":
        return run_compile_host(args)
    if args.stage == "source_template":
        return run_source_template(args)
    if args.stage == "native_dynamic":
        return run_native_dynamic(args)
    if args.stage == "cook_assets":
        return run_cook_assets(args)
    if args.stage == "pack":
        return run_pack(args)
    if args.stage == "platform_bundle":
        return run_platform_bundle(args)
    if args.stage == "report":
        return run_report(args)
    raise SystemExit(f"unsupported export stage: {args.stage}")


def run_pipeline(args: argparse.Namespace, resume_from: str) -> int:
    print(f"zircon_export resume_from={resume_from} profile={args.profile}")

    if resume_from == "validate":
        exit_code = run_pipeline_stage(args, "validate")
        print(f"pipeline_stage=validate exit_code={exit_code}")
        if exit_code != 0:
            return exit_code
        stages = pipeline_stages_after_validate(args)
        print("pipeline_stages=validate," + ",".join(stages))
    else:
        stages = pipeline_stages_from_resume(args, resume_from)
        print("pipeline_stages=" + ",".join(stages))

    for stage in stages:
        exit_code = run_pipeline_stage(args, stage)
        print(f"pipeline_stage={stage} exit_code={exit_code}")
        if exit_code != 0:
            return exit_code
    return 0


def run_pipeline_stage(args: argparse.Namespace, stage: str) -> int:
    stage_args = argparse.Namespace(**vars(args))
    stage_args.stage = stage
    apply_pipeline_stage_defaults(stage_args, stage)
    return run_stage(stage_args)


def pipeline_stages_after_validate(args: argparse.Namespace) -> tuple[str, ...]:
    return selected_pipeline_stages_after_validate(
        resolve_user_path(args.out),
        args.profile,
    )


def pipeline_stages_from_resume(
    args: argparse.Namespace,
    resume_from: str,
) -> tuple[str, ...]:
    return selected_pipeline_stages_from_resume(
        resolve_user_path(args.out),
        args.profile,
        resume_from,
    )


def pipeline_stages_from_validate_report(args: argparse.Namespace) -> tuple[str, ...]:
    return selected_pipeline_stages_after_validate(
        resolve_user_path(args.out),
        args.profile,
    )[:-1]


def apply_pipeline_stage_defaults(args: argparse.Namespace, stage: str) -> None:
    out_root = resolve_user_path(args.out)
    if stage == "cook_assets" and not args.asset_filter:
        asset_filter = validate_report_asset_filter(out_root, args.profile)
        if asset_filter:
            args.asset_filter = asset_filter
        return
    if stage == "pack" and not args.asset_manifest:
        asset_manifest = cook_assets_report_asset_manifest(out_root, args.profile)
        if asset_manifest:
            args.asset_manifest = str(asset_manifest)
        return
    if stage != "platform_bundle":
        return
    if not args.host_executable:
        host_executable = compile_host_report_host_executable(out_root, args.profile)
        if host_executable:
            args.host_executable = str(host_executable)
    if not args.pack_file:
        pack_file = pack_report_pack_file(out_root, args.profile)
        if pack_file:
            args.pack_file = str(pack_file)
    if not args.delta_pack:
        delta_pack = pack_report_delta_pack_file(out_root, args.profile)
        if delta_pack:
            args.delta_pack = str(delta_pack)
    if not args.native_plugins_dir:
        native_plugins_dir = native_dynamic_report_plugins_dir(out_root, args.profile)
        if native_plugins_dir:
            args.native_plugins_dir = str(native_plugins_dir)


def compile_host_report_host_executable(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(out_root, "compile_host", profile, "host_executable")


def cook_assets_report_asset_manifest(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "cook_assets",
        profile,
        "cooked_asset_manifest",
    )


def validate_report_asset_filter(out_root: Path, profile: str) -> str | None:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    if not report_path.exists():
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    if not isinstance(report, dict):
        return None
    if report.get("fatal") or report.get("profile") != profile:
        return None
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    asset_filter = profile_summary.get("asset_filter")
    if not isinstance(asset_filter, str) or not asset_filter:
        return None
    return asset_filter


def pack_report_pack_file(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "pack",
        profile,
        "pack",
        allow_missing_profile=True,
    )


def pack_report_delta_pack_file(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "pack",
        profile,
        "delta_pack",
        allow_missing_profile=True,
    )


def native_dynamic_report_plugins_dir(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "native_dynamic",
        profile,
        "plugins_dir",
    )


def stage_report_path_field(
    out_root: Path,
    stage: str,
    profile: str,
    field: str,
    *,
    allow_missing_profile: bool = False,
) -> Path | None:
    report_path = out_root / "stages" / stage / REPORT_FILE_NAME
    if not report_path.exists():
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    if not isinstance(report, dict):
        return None
    if report.get("fatal"):
        return None
    report_profile = report.get("profile")
    if not isinstance(report_profile, str):
        return None if not allow_missing_profile else field_value_path(report, field)
    if isinstance(report_profile, str) and report_profile != profile:
        return None
    return field_value_path(report, field)


def field_value_path(report: dict[str, Any], field: str) -> Path | None:
    value = report.get(field)
    if not isinstance(value, str) or not value:
        return None
    return resolve_user_path(value)


def parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    argv_list = list(argv) if argv is not None else sys.argv[1:]
    stage_explicit = option_present(argv_list, "--stage")
    resume_from_explicit = option_present(argv_list, "--resume-from")

    parser = argparse.ArgumentParser(
        description="Run the staged Zircon export pipeline.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python -m zircon_export --profile windows-release --project zircon-project.toml --out E:\\zircon-export
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage validate --dry-run
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage compile_host --dry-run
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage source_template --dry-run
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage native_dynamic
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage cook_assets --asset-manifest cooked-assets.json
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage pack
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage platform_bundle --template-dir export-templates\\windows-x86_64-library_embed-debug
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage platform_bundle --template-root export-templates --target-platform windows-x86_64
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage report
  python -m zircon_export --profile windows-release --out E:\\zircon-export --resume-from pack
""".strip(),
    )
    parser.add_argument("--profile", required=True, help="Export profile name.")
    parser.add_argument(
        "--project",
        default="zircon-project.toml",
        help="Project manifest path. Default: zircon-project.toml.",
    )
    parser.add_argument(
        "--out",
        "--output",
        default=DEFAULT_OUT,
        help=f"Export output directory. Default: {DEFAULT_OUT}.",
    )
    parser.add_argument(
        "--stage",
        choices=STAGES,
        default="validate",
        help="Single pipeline stage to run. Omit --stage to run the full main pipeline.",
    )
    parser.add_argument(
        "--resume-from",
        choices=RESUMABLE_STAGES,
        default=None,
        help=(
            "Run the main export pipeline from this stage through report. "
            "Cannot be combined with --stage."
        ),
    )
    parser.add_argument(
        "--repo-root",
        default=None,
        help="Repository root for Cargo commands. Default: auto-detect from this package.",
    )
    parser.add_argument(
        "--cargo",
        default="cargo",
        help="Cargo executable used when --validator is not supplied. Default: cargo.",
    )
    parser.add_argument(
        "--validator",
        default=None,
        help="Prebuilt zircon_export_validate executable. Skips cargo run when supplied.",
    )
    parser.add_argument(
        "--packer",
        default=None,
        help="Prebuilt zircon_export_pack executable. Skips cargo run when supplied.",
    )
    parser.add_argument(
        "--validate-report",
        default=None,
        help=(
            "Validate report consumed by CompileHost. "
            "Default: <out>/stages/validate/report.json."
        ),
    )
    parser.add_argument(
        "--native-plugin-root",
        default=None,
        help=(
            "Root directory containing NativeDynamic plugin packages. "
            "Default: <repo-root>/zircon_plugins."
        ),
    )
    parser.add_argument(
        "--asset-manifest",
        default=None,
        help=(
            "Cooked asset manifest input for CookAssets, or explicit Pack input. "
            "Pack defaults to <out>/stages/cook_assets/assets.json."
        ),
    )
    parser.add_argument(
        "--asset-filter",
        default=None,
        help=(
            "Default asset filter written by CookAssets when the cooked manifest "
            "does not declare asset_filter. Main pipeline defaults to Validate report."
        ),
    )
    parser.add_argument(
        "--pack-file",
        default=None,
        help="Pack output file. Default: <out>/stages/pack/assets.zrpack.",
    )
    parser.add_argument(
        "--previous-pack",
        default=None,
        help="Previous full zrpack used to build a delta pack. Requires --delta-pack.",
    )
    parser.add_argument(
        "--delta-pack",
        default=None,
        help="Delta zrpack output file. Requires --previous-pack.",
    )
    parser.add_argument(
        "--host-executable",
        default=None,
        help="Compiled host executable copied into PlatformBundle when available.",
    )
    parser.add_argument(
        "--native-plugins-dir",
        default=None,
        help=(
            "NativeDynamic plugins directory copied into PlatformBundle. "
            "Main pipeline defaults to <out>/stages/native_dynamic/plugins when reported."
        ),
    )
    parser.add_argument(
        "--template-dir",
        default=None,
        help=(
            "Export template package directory containing template.toml. "
            "When supplied, PlatformBundle validates the template before copying outputs."
        ),
    )
    parser.add_argument(
        "--template-root",
        default=None,
        help=(
            "Root directory containing export-template packages. "
            "Used by PlatformBundle to resolve a compatible template when --template-dir is omitted."
        ),
    )
    parser.add_argument(
        "--engine-version",
        default=None,
        help=(
            "Engine version expected by template.toml. "
            "Default: [workspace.package].version from Cargo.toml."
        ),
    )
    parser.add_argument(
        "--target-platform",
        default=None,
        help="Optional target platform id used to reject mismatched export templates.",
    )
    parser.add_argument(
        "--determinism-check",
        action="store_true",
        help="Run a second in-memory pack write and compare bytes.",
    )
    parser.add_argument(
        "--source-template-build",
        action="store_true",
        help=(
            "Execute cargo build for the materialized SourceTemplate project. "
            "Without this flag, SourceTemplate only writes the generated project and report."
        ),
    )
    parser.add_argument(
        "--native-dynamic-build",
        action="store_true",
        help=(
            "Execute Cargo cdylib builds for NativeDynamic packages and stage the built "
            "loadable artifacts. Without this flag, NativeDynamic only consumes existing "
            "package native artifacts."
        ),
    )
    parser.add_argument(
        "--native-dynamic-build-feature",
        action="append",
        default=[],
        help=(
            "Cargo feature passed to NativeDynamic cdylib package builds. May be repeated; "
            "recorded in the native_build_plan and applied when --native-dynamic-build runs."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-command",
        default=None,
        help=(
            "External signer executable for NativeDynamic loadable artifacts. "
            "Disabled when omitted."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-arg",
        action="append",
        default=[],
        help=(
            "Argument appended to --native-dynamic-sign-command. May be repeated; "
            "supports {artifact}, {package_id}, {package_dir}, {target_platform}, "
            "and {signing_profile}."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-profile",
        default=None,
        help=(
            "Audit label for the NativeDynamic signing profile. This does not select "
            "platform certificate stores by itself; it is passed to the external signer."
        ),
    )
    parser.add_argument(
        "--native-dynamic-sign-platform",
        action="append",
        default=[],
        help=(
            "Target platform prefix allowed by the NativeDynamic signing profile. "
            "May be repeated, for example windows, linux, or macos."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-command",
        default=None,
        help=(
            "External notarization or platform post-processing executable for "
            "NativeDynamic loadable artifacts. Disabled when omitted."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-arg",
        action="append",
        default=[],
        help=(
            "Argument appended to --native-dynamic-notarize-command. May be repeated; "
            "supports {artifact}, {package_id}, {package_dir}, {target_platform}, "
            "{signing_profile}, and {notarization_profile}."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-profile",
        default=None,
        help=(
            "Audit label for the NativeDynamic notarization/profile post-processing "
            "step. This does not select platform services by itself; it is passed to "
            "the external command."
        ),
    )
    parser.add_argument(
        "--native-dynamic-notarize-platform",
        action="append",
        default=[],
        help=(
            "Target platform prefix allowed by the NativeDynamic notarization profile. "
            "May be repeated, for example windows, linux, or macos."
        ),
    )
    parser.add_argument(
        "--target-dir",
        default=None,
        help="Cargo target directory passed through to cargo run.",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Pass --offline to Cargo commands.",
    )
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Do not pass --locked to Cargo. Locked mode is the default.",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Emit pretty JSON from the Rust validate report generator.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the selected stage command without executing it.",
    )
    args = parser.parse_args(argv)
    args.stage_explicit = stage_explicit
    args.resume_from_explicit = resume_from_explicit
    if args.resume_from and stage_explicit:
        parser.error("--resume-from runs the main pipeline and cannot be combined with --stage")
    return args


def option_present(argv: Sequence[str], option: str) -> bool:
    prefix = option + "="
    return any(value == option or value.startswith(prefix) for value in argv)


def run_validate(args: argparse.Namespace) -> int:
    repo_root = resolve_repo_root(args.repo_root)
    project_path = resolve_user_path(args.project)
    out_root = resolve_user_path(args.out)
    stage_dir = out_root / "stages" / "validate"
    report_path = stage_dir / REPORT_FILE_NAME

    command = validate_command(args, repo_root, project_path, stage_dir, report_path)
    print(f"zircon_export stage=Validate profile={args.profile}")
    print(f"report={report_path}")
    print(shell_join(command))
    if args.dry_run:
        return 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    return subprocess.call(command, cwd=repo_root)


def run_pack(args: argparse.Namespace) -> int:
    repo_root = resolve_repo_root(args.repo_root)
    out_root = resolve_user_path(args.out)
    asset_manifest = (
        resolve_user_path(args.asset_manifest)
        if args.asset_manifest
        else default_cooked_asset_manifest(out_root)
    )
    stage_dir = out_root / "stages" / "pack"
    report_path = stage_dir / REPORT_FILE_NAME
    pack_path = resolve_user_path(args.pack_file) if args.pack_file else stage_dir / "assets.zrpack"
    previous_pack = resolve_user_path(args.previous_pack) if args.previous_pack else None
    delta_pack = resolve_user_path(args.delta_pack) if args.delta_pack else None

    command = pack_command(args, repo_root, asset_manifest, stage_dir, report_path, pack_path)
    print(f"zircon_export stage=Pack profile={args.profile}")
    print(f"asset_manifest={asset_manifest}")
    print(f"pack={pack_path}")
    if previous_pack:
        print(f"previous_pack={previous_pack}")
    if delta_pack:
        print(f"delta_pack={delta_pack}")
    print(f"report={report_path}")
    print(shell_join(command))
    if args.dry_run:
        return 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    manifest_diagnostic = pack_asset_manifest_diagnostic(asset_manifest)
    if manifest_diagnostic:
        report = pack_preflight_failure_report(
            args=args,
            asset_manifest=asset_manifest,
            stage_dir=stage_dir,
            pack_path=pack_path,
            previous_pack=previous_pack,
            delta_pack=delta_pack,
            diagnostics=[manifest_diagnostic],
        )
        report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(json.dumps(report, indent=2))
        return 2
    return subprocess.call(command, cwd=repo_root)


def pack_asset_manifest_diagnostic(asset_manifest: Path) -> str | None:
    if not asset_manifest.exists():
        return (
            f"asset manifest {asset_manifest} does not exist; "
            "run CookAssets first or pass --asset-manifest"
        )
    if not asset_manifest.is_file():
        return f"asset manifest {asset_manifest} is not a file"
    return None


def pack_preflight_failure_report(
    *,
    args: argparse.Namespace,
    asset_manifest: Path,
    stage_dir: Path,
    pack_path: Path,
    previous_pack: Path | None,
    delta_pack: Path | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    return {
        "stage": "Pack",
        "profile": args.profile,
        "asset_manifest": str(asset_manifest),
        "pack": str(pack_path),
        "previous_pack": str(previous_pack) if previous_pack else None,
        "delta_pack": str(delta_pack) if delta_pack else None,
        "stage_output": str(stage_dir),
        "fatal": True,
        "diagnostics": diagnostics,
        "trim_report": {
            "included_assets": [],
            "trimmed_assets": [],
            "missing_dependencies": [],
            "diagnostics": [],
        },
        "manifest": None,
        "asset_count": 0,
        "chunk_count": 0,
        "deduplicated_assets": [],
        "deterministic_double_run": False,
        "delta_manifest": None,
        "delta_asset_count": 0,
        "delta_chunk_count": 0,
        "delta_removed_assets": [],
        "delta_reused_assets": [],
    }


def run_compile_host(args: argparse.Namespace) -> int:
    repo_root = resolve_repo_root(args.repo_root)
    out_root = resolve_user_path(args.out)
    validate_report = (
        resolve_user_path(args.validate_report)
        if args.validate_report
        else out_root / "stages" / "validate" / REPORT_FILE_NAME
    )
    stage_dir = out_root / "stages" / "compile_host"
    report_path = stage_dir / REPORT_FILE_NAME

    print(f"zircon_export stage=CompileHost profile={args.profile}")
    print(f"validate_report={validate_report}")
    print(f"report={report_path}")

    diagnostics: list[str] = []
    fatal = False
    compile_plan = load_compile_host_plan(validate_report, args.profile, diagnostics)
    command: list[str] = []
    host_executable = None

    if compile_plan is None:
        fatal = True
    else:
        command = compile_host_command(args, out_root, compile_plan)
        host_executable = compile_host_executable_path(out_root, compile_plan, args)
        print(shell_join(command))
        print(f"host={host_executable}")

    if args.dry_run:
        return 2 if fatal else 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    exit_code = 2
    if fatal:
        exit_code = 2
    else:
        exit_code = subprocess.call(command, cwd=repo_root)
        if exit_code != 0:
            fatal = True
            diagnostics.append(f"CompileHost cargo command exited with code {exit_code}")
        elif host_executable and not host_executable.exists():
            fatal = True
            diagnostics.append(f"CompileHost output {host_executable} does not exist")

    report = {
        "stage": "CompileHost",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report),
        "command": command,
        "host_executable": str(host_executable) if host_executable else None,
        "exit_code": exit_code,
    }
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 2 if fatal else 0


def run_platform_bundle(args: argparse.Namespace) -> int:
    repo_root = resolve_repo_root(args.repo_root)
    out_root = resolve_user_path(args.out)
    pack_path = resolve_user_path(args.pack_file) if args.pack_file else out_root / "stages" / "pack" / "assets.zrpack"
    delta_pack_path = resolve_user_path(args.delta_pack) if args.delta_pack else None
    stage_dir = out_root / "stages" / "platform_bundle"
    bundle_dir = out_root / "bundle" / args.profile
    report_path = stage_dir / REPORT_FILE_NAME
    host_executable = resolve_user_path(args.host_executable) if args.host_executable else None
    native_plugins_dir = (
        resolve_user_path(args.native_plugins_dir)
        if getattr(args, "native_plugins_dir", None)
        else None
    )
    template_dir = resolve_user_path(args.template_dir) if args.template_dir else None
    template_root = resolve_user_path(args.template_root) if args.template_root else None
    diagnostics: list[str] = []
    template_resolution: dict[str, Any] | None = None
    native_plugins_payload = native_dynamic_stage_payload_summary(
        out_root,
        args.profile,
        native_plugins_dir,
        diagnostics,
    )

    if template_root and not template_dir:
        expected_engine_version = args.engine_version or workspace_engine_version(repo_root)
        expected_target_platform = args.target_platform or validated_target_platform(out_root)
        template_resolution = resolve_export_template_from_root(
            template_root=template_root,
            profile=args.profile,
            expected_engine_version=expected_engine_version,
            expected_target_platform=expected_target_platform,
        )
        diagnostics.extend(template_resolution["diagnostics"])
        if not template_resolution["fatal"] and template_resolution.get("template_dir"):
            template_dir = Path(template_resolution["template_dir"])

    print(f"zircon_export stage=PlatformBundle profile={args.profile}")
    print(f"bundle={bundle_dir}")
    print(f"report={report_path}")
    if template_root:
        print(f"template_root={template_root}")
    if template_dir:
        print(f"template={template_dir}")
    if host_executable:
        print(f"host={host_executable}")
    if native_plugins_dir:
        print(f"native_plugins={native_plugins_dir}")
    print(f"pack={pack_path}")
    if delta_pack_path:
        print(f"delta_pack={delta_pack_path}")
    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if diagnostics else 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    if bundle_dir.exists():
        shutil.rmtree(bundle_dir)
    bundle_dir.mkdir(parents=True, exist_ok=True)
    fatal = bool(diagnostics)
    copied_host = None
    copied_pack = None
    copied_delta_pack = None
    copied_native_plugins = None
    copied_native_plugins_payload = None
    copied_template_files: list[dict[str, str]] = []
    template_report: dict[str, Any] | None = None

    if template_dir and not fatal:
        expected_engine_version = args.engine_version or workspace_engine_version(repo_root)
        expected_target_platform = args.target_platform or validated_target_platform(out_root)
        template_report = validate_export_template(
            template_dir=template_dir,
            expected_engine_version=expected_engine_version,
            profile=args.profile,
            expected_target_platform=expected_target_platform,
        )
        diagnostics.extend(template_report["diagnostics"])
        if template_report["fatal"]:
            fatal = True
            diagnostics.append("template validation failed; bundle copy skipped")
        elif not host_executable and template_report.get("host_executable"):
            host_executable = Path(template_report["host_executable"])

    if not fatal:
        materialize_result = materialize_platform_bundle(
            bundle_dir=bundle_dir,
            profile=args.profile,
            host_executable=host_executable,
            pack_path=pack_path,
            delta_pack_path=delta_pack_path,
            native_plugins_dir=native_plugins_dir,
            template_report=template_report,
            diagnostics=diagnostics,
        )
        fatal = materialize_result["fatal"]
        copied_host = materialize_result["host_executable"]
        copied_pack = materialize_result["pack"]
        copied_delta_pack = materialize_result["delta_pack"]
        copied_native_plugins = materialize_result["native_plugins"]
        if copied_native_plugins and native_plugins_payload:
            copied_native_plugins_payload = native_plugins_payload_for_bundle(
                native_plugins_payload,
                copied_native_plugins,
            )
        copied_template_files = materialize_result["template_files"]
        if fatal:
            if bundle_dir.exists():
                shutil.rmtree(bundle_dir)
            copied_host = None
            copied_pack = None
            copied_delta_pack = None
            copied_native_plugins = None
            copied_native_plugins_payload = None
            copied_template_files = []

    manifest = {
        "profile": args.profile,
        "template_resolution": template_resolution,
        "template": template_report,
        "host_executable": str(copied_host) if copied_host else None,
        "pack": str(copied_pack) if copied_pack else None,
        "delta_pack": str(copied_delta_pack) if copied_delta_pack else None,
        "native_plugins": str(copied_native_plugins) if copied_native_plugins else None,
        "native_plugins_payload": copied_native_plugins_payload,
        "template_files": copied_template_files,
    }
    bundle_manifest: Path | None = bundle_dir / "bundle.json"
    bundle_manifest_path = template_bundle_manifest_path(
        bundle_dir,
        template_report,
        diagnostics,
    )
    if bundle_manifest_path:
        bundle_manifest = bundle_manifest_path
    if not fatal:
        bundle_manifest.parent.mkdir(parents=True, exist_ok=True)
        bundle_manifest.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    else:
        if bundle_dir.exists():
            shutil.rmtree(bundle_dir)
        bundle_manifest = None
    report = {
        "stage": "PlatformBundle",
        "profile": args.profile,
        "bundle": str(bundle_dir),
        "fatal": fatal,
        "diagnostics": diagnostics,
        "template_resolution": template_resolution,
        "template": template_report,
        "host_executable": str(copied_host) if copied_host else None,
        "pack": str(copied_pack) if copied_pack else None,
        "delta_pack": str(copied_delta_pack) if copied_delta_pack else None,
        "native_plugins": str(copied_native_plugins) if copied_native_plugins else None,
        "native_plugins_payload": copied_native_plugins_payload,
        "template_files": copied_template_files,
        "bundle_manifest": str(bundle_manifest) if bundle_manifest else None,
    }
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 2 if fatal else 0


def validate_command(
    args: argparse.Namespace,
    repo_root: Path,
    project_path: Path,
    stage_dir: Path,
    report_path: Path,
) -> list[str]:
    validator_args = [
        "--project",
        str(project_path),
        "--profile",
        args.profile,
        "--report",
        str(report_path),
        "--stage-output",
        str(stage_dir),
    ]
    if args.pretty:
        validator_args.append("--pretty")

    if args.validator:
        return [str(resolve_user_path(args.validator)), *validator_args]

    command = [
        args.cargo,
        "run",
        "-p",
        "zircon_runtime",
        "--bin",
        "zircon_export_validate",
    ]
    if not args.no_locked:
        command.append("--locked")
    if args.offline:
        command.append("--offline")
    if args.target_dir:
        command.extend(["--target-dir", str(resolve_user_path(args.target_dir))])
    command.extend(["--", *validator_args])
    return command


def pack_command(
    args: argparse.Namespace,
    repo_root: Path,
    asset_manifest: Path,
    stage_dir: Path,
    report_path: Path,
    pack_path: Path,
) -> list[str]:
    packer_args = [
        "--profile",
        args.profile,
        "--manifest",
        str(asset_manifest),
        "--pack",
        str(pack_path),
        "--report",
        str(report_path),
        "--stage-output",
        str(stage_dir),
    ]
    if args.pretty:
        packer_args.append("--pretty")
    if args.determinism_check:
        packer_args.append("--determinism-check")
    if args.previous_pack:
        packer_args.extend(["--previous-pack", str(resolve_user_path(args.previous_pack))])
    if args.delta_pack:
        packer_args.extend(["--delta-pack", str(resolve_user_path(args.delta_pack))])

    if args.packer:
        return [str(resolve_user_path(args.packer)), *packer_args]

    command = [
        args.cargo,
        "run",
        "-p",
        "zircon_runtime",
        "--bin",
        "zircon_export_pack",
    ]
    if not args.no_locked:
        command.append("--locked")
    if args.offline:
        command.append("--offline")
    if args.target_dir:
        command.extend(["--target-dir", str(resolve_user_path(args.target_dir))])
    command.extend(["--", *packer_args])
    return command


def load_compile_host_plan(
    validate_report: Path,
    profile: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not validate_report.exists():
        diagnostics.append(f"validate report {validate_report} does not exist")
        return None
    try:
        report = json.loads(validate_report.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        diagnostics.append(f"validate report {validate_report} is not valid JSON: {error}")
        return None

    if report.get("fatal"):
        diagnostics.append("validate report is fatal; CompileHost will not run")
        return None
    if report.get("profile") != profile:
        diagnostics.append(
            f"validate report profile {report.get('profile')} does not match requested profile {profile}"
        )
        return None

    plan_summary = report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        diagnostics.append("validate report does not contain plan_summary")
        return None
    compile_plan = plan_summary.get("library_embed_compile_host")
    if not isinstance(compile_plan, dict):
        diagnostics.append("validate report does not contain a LibraryEmbed CompileHost plan")
        return None
    command = compile_plan.get("command")
    if not isinstance(command, list) or any(not isinstance(value, str) for value in command):
        diagnostics.append("CompileHost plan command must be a string array")
        return None
    return compile_plan


def compile_host_command(
    args: argparse.Namespace,
    out_root: Path,
    compile_plan: dict[str, Any],
) -> list[str]:
    command = list(compile_plan["command"])
    if command:
        command[0] = args.cargo
    if not args.no_locked and "--locked" not in command:
        command.append("--locked")
    if args.offline and "--offline" not in command:
        command.append("--offline")
    target_dir = resolve_user_path(args.target_dir) if args.target_dir else compile_host_target_dir(out_root)
    return command_with_option(command, "--target-dir", str(target_dir))


def command_with_option(command: list[str], option: str, value: str) -> list[str]:
    rewritten: list[str] = []
    index = 0
    found = False
    while index < len(command):
        rewritten.append(command[index])
        if command[index] == option and index + 1 < len(command):
            rewritten.append(value)
            index += 2
            found = True
            continue
        index += 1
    if not found:
        rewritten.extend([option, value])
    return rewritten


def compile_host_target_dir(out_root: Path) -> Path:
    return (out_root / "stages" / "compile_host" / "target").resolve()


def compile_host_executable_path(
    out_root: Path,
    compile_plan: dict[str, Any],
    args: argparse.Namespace | None = None,
) -> Path | None:
    binary = compile_plan.get("binary")
    cargo_profile = compile_plan.get("cargo_profile", "debug")
    if not isinstance(binary, str) or not binary:
        return None
    if not isinstance(cargo_profile, str) or not cargo_profile:
        cargo_profile = "debug"
    executable_name = binary + (".exe" if os.name == "nt" else "")
    target_dir = (
        resolve_user_path(args.target_dir)
        if args is not None and getattr(args, "target_dir", None)
        else compile_host_target_dir(out_root)
    )
    return target_dir / cargo_profile / executable_name


def materialize_platform_bundle(
    *,
    bundle_dir: Path,
    profile: str,
    host_executable: Path | None,
    pack_path: Path,
    delta_pack_path: Path | None,
    native_plugins_dir: Path | None,
    template_report: dict[str, Any] | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    fatal = False
    copied_host: Path | None = None
    copied_pack: Path | None = None
    copied_delta_pack: Path | None = None
    copied_native_plugins: Path | None = None
    copied_template_files: list[dict[str, str]] = []
    bundle_root = template_bundle_root(bundle_dir, template_report, diagnostics)
    bundle_root.mkdir(parents=True, exist_ok=True)

    host_destination: Path | None = None
    if host_executable:
        host_destination = template_bundle_output_path(
            bundle_root,
            template_report,
            "host_path",
            host_executable.name,
            diagnostics,
        )
    else:
        diagnostics.append("host executable not supplied; bundle contains assets only")
        fatal = True

    pack_destination = template_bundle_output_path(
        bundle_root,
        template_report,
        "pack_path",
        pack_path.name,
        diagnostics,
    )
    delta_pack_destination = None
    if delta_pack_path:
        delta_pack_destination = template_bundle_output_path(
            bundle_root,
            template_report,
            "delta_pack_path",
            delta_pack_path.name,
            diagnostics,
        )

    if template_report and not fatal:
        for entry in template_report.get("files", []):
            if not isinstance(entry, dict):
                continue
            source = Path(template_report["template_dir"]) / entry["path"]
            destination = resolve_bundle_child(
                bundle_root,
                entry.get("bundle_path", entry["path"]),
                diagnostics,
            )
            if not destination:
                fatal = True
                continue
            if host_destination and source.resolve() == host_executable.resolve():
                continue
            if not source.exists():
                diagnostics.append(f"template file {source} does not exist during bundle copy")
                fatal = True
                continue
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
            copied_template_files.append(
                {
                    "source": str(source),
                    "destination": str(destination),
                }
            )

    if host_executable and host_destination:
        if not host_executable.exists():
            diagnostics.append(f"host executable {host_executable} does not exist")
            fatal = True
        elif not fatal:
            host_destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(host_executable, host_destination)
            copied_host = host_destination

    if not pack_path.exists():
        diagnostics.append(f"pack file {pack_path} does not exist")
        fatal = True
    elif not fatal and pack_destination:
        pack_destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(pack_path, pack_destination)
        copied_pack = pack_destination

    if delta_pack_path:
        if not delta_pack_path.exists():
            diagnostics.append(f"delta pack file {delta_pack_path} does not exist")
            fatal = True
        elif not fatal and delta_pack_destination:
            delta_pack_destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(delta_pack_path, delta_pack_destination)
            copied_delta_pack = delta_pack_destination

    if native_plugins_dir:
        plugins_destination = resolve_bundle_child(bundle_root, "plugins", diagnostics)
        if plugins_destination and native_plugins_dir.exists() and native_plugins_dir.is_dir():
            if plugins_destination.exists():
                shutil.rmtree(plugins_destination)
                copied_template_files = template_files_outside_directory(
                    copied_template_files,
                    plugins_destination,
                )
            copy_dir_contents(native_plugins_dir, plugins_destination)
            copied_native_plugins = plugins_destination
        elif plugins_destination:
            diagnostics.append(f"native plugins directory {native_plugins_dir} does not exist")
            fatal = True

    return {
        "fatal": fatal,
        "profile": profile,
        "bundle_root": bundle_root,
        "host_executable": copied_host,
        "pack": copied_pack,
        "delta_pack": copied_delta_pack,
        "native_plugins": copied_native_plugins,
        "template_files": copied_template_files,
    }


def copy_dir_contents(source: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for child in source.iterdir():
        target = destination / child.name
        if child.is_dir():
            copy_dir_contents(child, target)
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(child, target)


def native_plugins_payload_for_bundle(
    payload: dict[str, Any],
    bundle_plugins_dir: Path,
) -> dict[str, Any]:
    bundled_payload = dict(payload)
    bundled_payload["bundle_path"] = str(bundle_plugins_dir)
    materialized_packages = payload.get("materialized_packages")
    if isinstance(materialized_packages, list):
        source = payload.get("source")
        source_dir = Path(source).expanduser() if isinstance(source, str) else None
        bundled_payload["materialized_packages"] = [
            native_plugins_package_for_bundle(package, source_dir, bundle_plugins_dir)
            for package in materialized_packages
        ]
    return bundled_payload


def template_files_outside_directory(
    template_files: list[dict[str, str]],
    removed_directory: Path,
) -> list[dict[str, str]]:
    retained: list[dict[str, str]] = []
    for entry in template_files:
        destination = entry.get("destination")
        if not destination:
            retained.append(entry)
            continue
        try:
            Path(destination).expanduser().resolve().relative_to(removed_directory.resolve())
        except (OSError, ValueError):
            retained.append(entry)
    return retained


def native_plugins_package_for_bundle(
    package: object,
    source_dir: Path | None,
    bundle_plugins_dir: Path,
) -> object:
    if not isinstance(package, dict):
        return package
    bundled_package = dict(package)
    destination = package.get("destination")
    relative_destination = native_plugins_relative_payload_path(destination, source_dir)
    if relative_destination is None:
        return bundled_package
    bundled_package["destination"] = str(bundle_plugins_dir / relative_destination)
    package_report = package.get("package_report")
    relative_package_report = native_plugins_relative_payload_path(package_report, source_dir)
    if relative_package_report is not None:
        bundled_package["package_report"] = str(bundle_plugins_dir / relative_package_report)
    return bundled_package


def native_plugins_relative_payload_path(
    value: object,
    source_dir: Path | None,
) -> Path | None:
    if not isinstance(value, str) or source_dir is None:
        return None
    try:
        return Path(value).expanduser().resolve().relative_to(source_dir.resolve())
    except (OSError, ValueError):
        return None


def template_bundle_root(
    bundle_dir: Path,
    template_report: dict[str, Any] | None,
    diagnostics: list[str],
) -> Path:
    if not template_report:
        return bundle_dir
    bundle = template_report.get("bundle")
    if not isinstance(bundle, dict):
        return bundle_dir
    root = bundle.get("root")
    if not isinstance(root, str) or not root or root == ".":
        return bundle_dir
    return resolve_bundle_child(bundle_dir, root, diagnostics) or bundle_dir


def template_bundle_output_path(
    bundle_root: Path,
    template_report: dict[str, Any] | None,
    field_name: str,
    fallback_name: str,
    diagnostics: list[str],
) -> Path | None:
    if template_report:
        bundle = template_report.get("bundle")
        if isinstance(bundle, dict):
            value = bundle.get(field_name)
            if isinstance(value, str) and value:
                return resolve_bundle_child(bundle_root, value, diagnostics)
    return bundle_root / fallback_name


def template_bundle_manifest_path(
    bundle_dir: Path,
    template_report: dict[str, Any] | None,
    diagnostics: list[str],
) -> Path | None:
    if not template_report:
        return None
    bundle = template_report.get("bundle")
    if not isinstance(bundle, dict):
        return None
    manifest_path = bundle.get("manifest_path")
    if not isinstance(manifest_path, str) or not manifest_path:
        return None
    return resolve_bundle_child(
        template_bundle_root(bundle_dir, template_report, diagnostics),
        manifest_path,
        diagnostics,
    )


def resolve_export_template_from_root(
    *,
    template_root: Path,
    profile: str,
    expected_engine_version: str | None,
    expected_target_platform: str | None,
) -> dict[str, Any]:
    diagnostics: list[str] = []
    root = template_root.resolve()
    report: dict[str, Any] = {
        "template_root": str(root),
        "profile": profile,
        "expected_engine_version": expected_engine_version,
        "expected_target_platform": expected_target_platform,
        "fatal": False,
        "diagnostics": diagnostics,
        "candidates": [],
        "skipped_candidates": [],
        "template_dir": None,
    }

    if not root.exists():
        diagnostics.append(f"export template root {root} does not exist")
        report["fatal"] = True
        return report
    if not root.is_dir():
        diagnostics.append(f"export template root {root} is not a directory")
        report["fatal"] = True
        return report

    for manifest_path in sorted(root.glob(f"*/{EXPORT_TEMPLATE_MANIFEST_NAME}")):
        candidate_diagnostics: list[str] = []
        manifest = read_template_manifest_for_resolution(manifest_path, candidate_diagnostics)
        if manifest is None:
            if candidate_diagnostics:
                report["skipped_candidates"].append(
                    {
                        "template_dir": str(manifest_path.parent.resolve()),
                        "diagnostics": candidate_diagnostics,
                    }
                )
            continue
        if not template_manifest_matches_resolution(
            manifest,
            profile=profile,
            expected_engine_version=expected_engine_version,
            expected_target_platform=expected_target_platform,
        ):
            continue
        candidate_validation = validate_export_template(
            template_dir=manifest_path.parent,
            expected_engine_version=expected_engine_version,
            profile=profile,
            expected_target_platform=expected_target_platform,
        )
        if candidate_validation["fatal"]:
            report["skipped_candidates"].append(
                {
                    "template_dir": str(manifest_path.parent.resolve()),
                    "diagnostics": candidate_validation["diagnostics"],
                }
            )
            continue
        candidate = template_resolution_candidate(manifest_path.parent, manifest)
        report["candidates"].append(candidate)

    candidates = report["candidates"]
    if not candidates:
        target_note = expected_target_platform or "<any>"
        engine_note = expected_engine_version or "<unresolved>"
        diagnostics.append(
            "no export template under "
            f"{root} matched profile={profile} target_platform={target_note} "
            f"engine_version={engine_note}"
        )
    elif len(candidates) > 1:
        diagnostics.append(
            "multiple export templates matched profile="
            f"{profile}: "
            + ", ".join(str(candidate["template_dir"]) for candidate in candidates)
        )
    else:
        report["template_dir"] = candidates[0]["template_dir"]

    report["fatal"] = bool(diagnostics) and report["template_dir"] is None
    return report


def read_template_manifest_for_resolution(
    manifest_path: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"export template manifest {manifest_path} is not valid TOML: {error}")
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(f"export template manifest {manifest_path} must be a TOML table")
        return None
    return manifest


def template_manifest_matches_resolution(
    manifest: dict[str, Any],
    *,
    profile: str,
    expected_engine_version: str | None,
    expected_target_platform: str | None,
) -> bool:
    if manifest.get("format_version") != EXPORT_TEMPLATE_FORMAT_VERSION:
        return False
    engine_version = manifest.get("engine_version")
    if expected_engine_version and engine_version != expected_engine_version:
        return False
    target_platform = manifest.get("target_platform")
    if expected_target_platform:
        if not isinstance(target_platform, str):
            return False
        if normalize_target_platform(target_platform) != normalize_target_platform(
            expected_target_platform
        ):
            return False
    compatible_profiles = manifest.get("compatible_profiles", [])
    if not compatible_profiles:
        return True
    if not isinstance(compatible_profiles, list):
        return False
    return profile in compatible_profiles


def template_resolution_candidate(
    template_dir: Path,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    return {
        "template_dir": str(template_dir.resolve()),
        "template_id": manifest.get("template_id"),
        "engine_version": manifest.get("engine_version"),
        "target_platform": manifest.get("target_platform"),
        "compatible_profiles": manifest.get("compatible_profiles", []),
        "bundle_format": manifest.get("bundle_format"),
    }


def validate_export_template(
    *,
    template_dir: Path,
    expected_engine_version: str | None,
    profile: str,
    expected_target_platform: str | None,
) -> dict[str, Any]:
    diagnostics: list[str] = []
    template_root = template_dir.resolve()
    manifest_path = template_root / EXPORT_TEMPLATE_MANIFEST_NAME
    report: dict[str, Any] = {
        "template_dir": str(template_root),
        "manifest": str(manifest_path),
        "expected_format_version": EXPORT_TEMPLATE_FORMAT_VERSION,
        "expected_engine_version": expected_engine_version,
        "expected_target_platform": expected_target_platform,
        "profile": profile,
        "fatal": False,
        "diagnostics": diagnostics,
        "host_executable": None,
        "files": [],
    }

    if not template_root.exists():
        diagnostics.append(f"export template directory {template_root} does not exist")
        report["fatal"] = True
        return report
    if not manifest_path.exists():
        diagnostics.append(f"export template manifest {manifest_path} does not exist")
        report["fatal"] = True
        return report

    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"export template manifest is not valid TOML: {error}")
        report["fatal"] = True
        return report

    format_version = manifest.get("format_version")
    report["format_version"] = format_version
    if type(format_version) is not int:
        diagnostics.append("template.toml field format_version must be an integer")
    elif format_version != EXPORT_TEMPLATE_FORMAT_VERSION:
        diagnostics.append(
            "template format_version "
            f"{format_version} is not supported; expected {EXPORT_TEMPLATE_FORMAT_VERSION}"
        )

    engine_version = template_string_field(manifest, "engine_version", diagnostics)
    report["engine_version"] = engine_version
    if not expected_engine_version:
        diagnostics.append("engine version could not be resolved for template validation")
    elif engine_version and engine_version != expected_engine_version:
        diagnostics.append(
            "template engine_version "
            f"{engine_version} does not match engine version {expected_engine_version}"
        )

    template_id = template_string_field(manifest, "template_id", diagnostics)
    target_platform = template_string_field(manifest, "target_platform", diagnostics)
    host_kind = template_string_field(manifest, "host_kind", diagnostics)
    resource_strategy = template_string_field(manifest, "resource_strategy", diagnostics)
    plugin_strategy = template_string_field(manifest, "plugin_strategy", diagnostics)
    bundle_format = template_string_field(manifest, "bundle_format", diagnostics)
    content_hash = template_string_field(manifest, "content_hash", diagnostics)
    report.update(
        {
            "template_id": template_id,
            "target_platform": target_platform,
            "host_kind": host_kind,
            "resource_strategy": resource_strategy,
            "plugin_strategy": plugin_strategy,
            "bundle_format": bundle_format,
            "content_hash": content_hash,
        }
    )

    validate_allowed_field("host_kind", host_kind, EXPORT_TEMPLATE_ALLOWED_HOST_KINDS, diagnostics)
    validate_allowed_field(
        "resource_strategy",
        resource_strategy,
        EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
        diagnostics,
    )
    validate_allowed_field(
        "plugin_strategy",
        plugin_strategy,
        EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
        diagnostics,
    )
    validate_allowed_field(
        "bundle_format",
        bundle_format,
        EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
        diagnostics,
    )

    if (
        expected_target_platform
        and target_platform
        and normalize_target_platform(target_platform)
        != normalize_target_platform(expected_target_platform)
    ):
        diagnostics.append(
            "template target_platform "
            f"{target_platform} does not match requested target platform {expected_target_platform}"
        )

    compatible_profiles = manifest.get("compatible_profiles", [])
    if compatible_profiles is None:
        compatible_profiles = []
    if not isinstance(compatible_profiles, list) or any(
        not isinstance(value, str) for value in compatible_profiles
    ):
        diagnostics.append("template.toml field compatible_profiles must be a string array")
        compatible_profiles = []
    report["compatible_profiles"] = compatible_profiles
    if compatible_profiles and profile not in compatible_profiles:
        diagnostics.append(
            f"template compatible_profiles does not include requested profile {profile}"
        )

    paths = manifest.get("paths")
    host_relative_path = None
    if not isinstance(paths, dict):
        diagnostics.append("template.toml table [paths] is required")
    else:
        host_relative_path = paths.get("host_executable")
        if not isinstance(host_relative_path, str) or not host_relative_path.strip():
            diagnostics.append("template.toml field paths.host_executable must be a non-empty string")
            host_relative_path = None
        else:
            host_relative_path = normalize_relative_path(host_relative_path)
            if not is_safe_relative_path(host_relative_path):
                diagnostics.append(
                    "template.toml field paths.host_executable must be a safe relative path"
                )
                host_relative_path = None

    bundle_config = template_bundle_config(manifest, diagnostics)
    report["bundle"] = bundle_config

    checked_files = template_file_manifest(template_root, manifest, diagnostics)
    report["files"] = checked_files
    if checked_files:
        computed_content_hash = compute_template_content_hash(checked_files)
        report["computed_content_hash"] = computed_content_hash
        if content_hash and not is_sha256_hex(content_hash):
            diagnostics.append("template.toml field content_hash must be a SHA-256 hex digest")
        elif content_hash and content_hash.lower() != computed_content_hash:
            diagnostics.append(
                "template content_hash "
                f"{content_hash} does not match computed hash {computed_content_hash}"
            )
    else:
        diagnostics.append("template.toml must declare at least one [[files]] entry")

    if host_relative_path:
        host_path = resolve_template_child(template_root, host_relative_path, diagnostics)
        if host_path:
            report["host_executable"] = str(host_path)
            if not host_path.exists():
                diagnostics.append(f"template host executable {host_path} does not exist")
            declared_paths = {entry["path"] for entry in checked_files}
            if host_relative_path.replace("\\", "/") not in declared_paths:
                diagnostics.append(
                    "template paths.host_executable must also be listed in [[files]]"
                )

    report["fatal"] = bool(diagnostics)
    return report


def template_string_field(
    manifest: dict[str, Any],
    field_name: str,
    diagnostics: list[str],
) -> str | None:
    value = manifest.get(field_name)
    if isinstance(value, str) and value.strip():
        return value.strip()
    diagnostics.append(f"template.toml field {field_name} must be a non-empty string")
    return None


def validate_allowed_field(
    field_name: str,
    value: str | None,
    allowed_values: set[str],
    diagnostics: list[str],
) -> None:
    if value and value not in allowed_values:
        diagnostics.append(
            f"template.toml field {field_name}={value!r} is not one of "
            f"{', '.join(sorted(allowed_values))}"
        )


def template_bundle_config(
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> dict[str, str]:
    bundle = manifest.get("bundle", {})
    if bundle is None:
        bundle = {}
    if not isinstance(bundle, dict):
        diagnostics.append("template.toml table [bundle] must be a table when present")
        bundle = {}

    config = {
        "root": template_optional_path_field(bundle, "root", ".", diagnostics),
        "host_path": template_optional_path_field(bundle, "host_path", "", diagnostics),
        "pack_path": template_optional_path_field(bundle, "pack_path", "", diagnostics),
        "delta_pack_path": template_optional_path_field(
            bundle,
            "delta_pack_path",
            "",
            diagnostics,
        ),
        "manifest_path": template_optional_path_field(
            bundle,
            "manifest_path",
            "bundle.json",
            diagnostics,
        ),
    }
    return config


def template_optional_path_field(
    table: dict[str, Any],
    field_name: str,
    default: str,
    diagnostics: list[str],
) -> str:
    value = table.get(field_name, default)
    if value is None:
        return default
    if not isinstance(value, str):
        diagnostics.append(f"template.toml field bundle.{field_name} must be a string")
        return default
    normalized = normalize_relative_path(value) if value else default
    if normalized in {"", "."}:
        return normalized
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"template.toml field bundle.{field_name} must be a safe relative path")
        return default
    return normalized


def template_file_manifest(
    template_root: Path,
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> list[dict[str, str]]:
    files = manifest.get("files", [])
    if not isinstance(files, list):
        diagnostics.append("template.toml [[files]] entries must form an array")
        return []

    checked_files: list[dict[str, str]] = []
    seen_paths: set[str] = set()
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            diagnostics.append(f"template.toml [[files]] entry {index} must be a table")
            continue
        relative_path = entry.get("path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            diagnostics.append(f"template.toml [[files]] entry {index} needs a non-empty path")
            continue
        normalized_path = normalize_relative_path(relative_path)
        if not is_safe_relative_path(normalized_path):
            diagnostics.append(
                f"template.toml [[files]] entry {index} path must be a safe relative path"
            )
            continue
        if normalized_path in seen_paths:
            diagnostics.append(f"template file {normalized_path} is declared more than once")
            continue
        seen_paths.add(normalized_path)

        file_path = resolve_template_child(template_root, normalized_path, diagnostics)
        declared_sha256 = entry.get("sha256")
        if not isinstance(declared_sha256, str) or not is_sha256_hex(declared_sha256):
            diagnostics.append(
                f"template file {normalized_path} must declare a SHA-256 hex digest"
            )
            continue
        if not file_path or not file_path.exists():
            diagnostics.append(f"template file {normalized_path} does not exist")
            continue

        actual_sha256 = hashlib.sha256(file_path.read_bytes()).hexdigest()
        if declared_sha256.lower() != actual_sha256:
            diagnostics.append(
                f"template file {normalized_path} sha256 {declared_sha256} "
                f"does not match actual {actual_sha256}"
            )
            continue
        checked_files.append(
            {
                "path": normalized_path,
                "bundle_path": template_bundle_file_path(entry, normalized_path, diagnostics),
                "sha256": actual_sha256,
                "purpose": str(entry.get("purpose", "")),
            }
        )
    return checked_files


def template_bundle_file_path(
    entry: dict[str, Any],
    normalized_path: str,
    diagnostics: list[str],
) -> str:
    value = entry.get("bundle_path", normalized_path)
    if value is None:
        return normalized_path
    if not isinstance(value, str) or not value.strip():
        diagnostics.append(f"template file {normalized_path} has an invalid bundle_path")
        return normalized_path
    normalized = normalize_relative_path(value)
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"template file {normalized_path} bundle_path must be a safe relative path")
        return normalized_path
    return normalized


def resolve_template_child(
    template_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"template path {relative_path} must be relative")
        return None
    resolved = (template_root / child_path).resolve()
    try:
        resolved.relative_to(template_root)
    except ValueError:
        diagnostics.append(f"template path {relative_path} escapes the template directory")
        return None
    return resolved


def resolve_bundle_child(
    bundle_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    normalized = normalize_relative_path(relative_path)
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"bundle path {relative_path} must be a safe relative path")
        return None
    resolved_root = bundle_root.resolve()
    resolved = (resolved_root / Path(normalized)).resolve()
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        diagnostics.append(f"bundle path {relative_path} escapes the bundle directory")
        return None
    return resolved


def normalize_relative_path(value: str) -> str:
    return value.strip().replace("\\", "/")


def is_safe_relative_path(value: str) -> bool:
    path = Path(value)
    if path.is_absolute():
        return False
    parts = value.split("/")
    return bool(value) and all(part not in {"", ".", ".."} for part in parts)


def compute_template_content_hash(files: Sequence[dict[str, str]]) -> str:
    hasher = hashlib.sha256()
    for entry in sorted(files, key=lambda value: value["path"]):
        hasher.update(entry["path"].encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry.get("bundle_path", "").encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry["sha256"].lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def is_sha256_hex(value: str) -> bool:
    if len(value) != 64:
        return False
    return all(character in "0123456789abcdefABCDEF" for character in value)


def workspace_engine_version(repo_root: Path) -> str | None:
    manifest_path = repo_root / "Cargo.toml"
    if not manifest_path.exists():
        return None
    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError:
        return None
    version = (
        manifest.get("workspace", {})
        .get("package", {})
        .get("version")
    )
    return version if isinstance(version, str) and version else None


def validated_target_platform(out_root: Path) -> str | None:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    if not report_path.exists():
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    target_platform = profile_summary.get("target_platform")
    return target_platform if isinstance(target_platform, str) and target_platform else None


def normalize_target_platform(value: str) -> str:
    aliases = {
        "windows": "windows-x86_64",
        "linux": "linux-x86_64",
        "macos": "macos-aarch64",
    }
    return aliases.get(value, value)


def resolve_repo_root(repo_root: str | None) -> Path:
    if repo_root:
        return resolve_user_path(repo_root)
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    return shlex.join(command)

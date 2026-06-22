#!/usr/bin/env python3
"""Stage-oriented Zircon export pipeline command."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any, Sequence

from .compile_host import (
    compile_host_command,
    run_compile_host,
)
from .cook_assets import default_cooked_asset_manifest, run_cook_assets
from .pipeline_report import run_report
from .platform_bundle import run_platform_bundle
from .pipeline_stages import (
    LIBRARY_EMBED_EXECUTION_STAGES,
    pipeline_stages_after_validate as selected_pipeline_stages_after_validate,
    pipeline_stages_from_resume as selected_pipeline_stages_from_resume,
)
from .native_dynamic import run_native_dynamic
from .path_resolve import resolve_stage_optional_path
from .report_io import write_report_targets
from .stage_handoff import (
    compile_host_report_host_executable,
    cook_assets_report_asset_manifest,
    native_dynamic_report_plugins_dir,
    pack_report_delta_pack_file,
    pack_report_pack_file,
    stage_report_path_handoff_diagnostic,
    validate_report_asset_filter,
    validate_report_asset_filter_diagnostic,
    validate_report_requires_bundle_strategy_diagnostics,
)
from .source_template import run_source_template


STAGES = (
    "validate",
    "source_template",
    "native_dynamic",
    "compile_host",
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
    if stage == "cook_assets" and args.asset_filter is None:
        asset_filter_diagnostic = validate_report_asset_filter_diagnostic(
            out_root,
            args.profile,
        )
        if asset_filter_diagnostic:
            args.validate_asset_filter_diagnostic = asset_filter_diagnostic
            return
        asset_filter = validate_report_asset_filter(out_root, args.profile)
        if asset_filter:
            args.asset_filter = asset_filter
        return
    if stage == "pack" and args.asset_manifest is None:
        asset_manifest = cook_assets_report_asset_manifest(out_root, args.profile)
        if asset_manifest:
            args.asset_manifest = str(asset_manifest)
        return
    if stage != "platform_bundle":
        return
    if args.host_executable is None:
        host_executable = compile_host_report_host_executable(out_root, args.profile)
        if host_executable:
            args.host_executable = str(host_executable)
            args.host_executable_source_origin = "compile_host_report"
    if args.pack_file is None:
        pack_file = pack_report_pack_file(out_root, args.profile)
        if pack_file:
            args.pack_file = str(pack_file)
    if (
        args.delta_pack is None
        and not getattr(args, "pack_file_explicit", False)
        and not getattr(args, "delta_pack_explicit", False)
    ):
        delta_pack = pack_report_delta_pack_file(out_root, args.profile)
        if delta_pack:
            args.delta_pack = str(delta_pack)
    if args.native_plugins_dir is None:
        native_plugins_dir = native_dynamic_report_plugins_dir(out_root, args.profile)
        if native_plugins_dir:
            args.native_plugins_dir = str(native_plugins_dir)


def parse_args(argv: Sequence[str] | None) -> argparse.Namespace:
    argv_list = list(argv) if argv is not None else sys.argv[1:]
    stage_explicit = option_present(argv_list, "--stage")
    resume_from_explicit = option_present(argv_list, "--resume-from")
    pack_file_explicit = option_present(argv_list, "--pack-file")
    delta_pack_explicit = option_present(argv_list, "--delta-pack")
    host_executable_explicit = option_present(argv_list, "--host-executable")

    parser = argparse.ArgumentParser(
        description="Run the staged Zircon export pipeline.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python -m zircon_export --profile windows-release --project zircon-project.toml --out E:\\zircon-export
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage validate --dry-run
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage source_template --dry-run
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage native_dynamic
  python -m zircon_export --profile windows-release --out E:\\zircon-export --stage compile_host --dry-run
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
            "Pack defaults from a matching CookAssets report, then "
            "<out>/stages/cook_assets/assets.json."
        ),
    )
    parser.add_argument(
        "--asset-filter",
        default=None,
        help=(
            "Default asset filter written by CookAssets when the cooked manifest "
            "does not declare asset_filter. Defaults from a matching Validate report."
        ),
    )
    parser.add_argument(
        "--pack-file",
        default=None,
        help=(
            "Pack output file. PlatformBundle defaults from a matching Pack report, "
            "then <out>/stages/pack/assets.zrpack."
        ),
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
            "Defaults from a matching NativeDynamic report when reported."
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
    args.pack_file_explicit = pack_file_explicit
    args.delta_pack_explicit = delta_pack_explicit
    args.host_executable_explicit = host_executable_explicit
    if args.resume_from and stage_explicit:
        parser.error("--resume-from runs the main pipeline and cannot be combined with --stage")
    return args


def option_present(argv: Sequence[str], option: str) -> bool:
    prefix = option + "="
    return any(value == option or value.startswith(prefix) for value in argv)


def run_validate(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    stage_dir = out_root / "stages" / "validate"
    report_path = stage_dir / REPORT_FILE_NAME

    diagnostics: list[str] = []
    repo_root = (
        resolve_validate_path(args.repo_root, "repo_root", diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    project_path = resolve_validate_path(args.project, "project", diagnostics)
    validator = resolve_pack_optional_path(args.validator, "validator", diagnostics)
    target_dir = resolve_pack_optional_path(args.target_dir, "target_dir", diagnostics)
    command = (
        validate_command(
            args,
            repo_root,
            project_path,
            stage_dir,
            report_path,
            validator=validator,
            target_dir=target_dir,
        )
        if not diagnostics
        else None
    )
    print(f"zircon_export stage=Validate profile={args.profile}")
    print(f"report={report_path}")
    if command is not None:
        print(shell_join(command))
    else:
        print("command=<skipped>")
    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if diagnostics else 0

    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        report = {
            "stage": "Validate",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": [
                f"Validate stage directory {stage_dir} could not be created: {error}"
            ],
            "project": str(project_path) if project_path else None,
            "stage_output": str(stage_dir),
            "command": command if command is not None else [],
        }
        print(json.dumps(report, indent=2))
        return 2
    if diagnostics:
        report = {
            "stage": "Validate",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": diagnostics,
            "project": str(project_path) if project_path else None,
            "stage_output": str(stage_dir),
            "command": [],
        }
        write_report_targets([("Validate report", report_path)], report)
        print(json.dumps(report, indent=2))
        return 2
    if command is None:
        raise AssertionError("Validate command was not built after preflight passed")
    try:
        exit_code = subprocess.call(command, cwd=repo_root)
        if not report_path.is_file():
            report = validate_preflight_failure_report(
                args=args,
                project_path=project_path,
                stage_dir=stage_dir,
                command=command,
                diagnostics=[
                    f"Validate command exited with code {exit_code} but did not write report {report_path}"
                ],
                exit_code=exit_code,
            )
            write_report_targets([("Validate report", report_path)], report)
            print(json.dumps(report, indent=2))
            return exit_code if exit_code != 0 else 2
        return exit_code
    except OSError as error:
        report = validate_preflight_failure_report(
            args=args,
            project_path=project_path,
            stage_dir=stage_dir,
            command=command,
            diagnostics=[f"Validate command {command[0]} could not start: {error}"],
        )
        write_report_targets([("Validate report", report_path)], report)
        print(json.dumps(report, indent=2))
        return 2


def validate_preflight_failure_report(
    *,
    args: argparse.Namespace,
    project_path: Path | None,
    stage_dir: Path,
    command: list[str],
    diagnostics: list[str],
    exit_code: int | None = None,
) -> dict[str, Any]:
    report: dict[str, Any] = {
        "stage": "Validate",
        "profile": args.profile,
        "fatal": True,
        "diagnostics": diagnostics,
        "project": str(project_path) if project_path else None,
        "stage_output": str(stage_dir),
        "command": command,
    }
    if exit_code is not None:
        report["exit_code"] = exit_code
    return report


def run_pack(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    diagnostics: list[str] = []
    repo_root = (
        resolve_pack_stage_path(args.repo_root, "repo_root", diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    asset_manifest_argument_diagnostic = pack_asset_manifest_argument_diagnostic(args)
    if asset_manifest_argument_diagnostic:
        diagnostics.append(asset_manifest_argument_diagnostic)
    pack_file_diagnostic = pack_file_argument_diagnostic(args)
    if pack_file_diagnostic:
        diagnostics.append(pack_file_diagnostic)
    packer_diagnostic = pack_optional_path_argument_diagnostic(args.packer, "packer")
    if packer_diagnostic:
        diagnostics.append(packer_diagnostic)
    target_dir_diagnostic = pack_optional_path_argument_diagnostic(
        args.target_dir,
        "target_dir",
    )
    if target_dir_diagnostic:
        diagnostics.append(target_dir_diagnostic)
    cook_assets_handoff_diagnostic = None
    diagnostics.extend(pack_delta_argument_diagnostics(args))
    validate_strategy_diagnostics = validate_report_requires_bundle_strategy_diagnostics(
        out_root,
        args.profile,
        "Pack",
    )
    diagnostics.extend(validate_strategy_diagnostics)
    if args.asset_manifest is None:
        cook_assets_handoff_diagnostic = stage_report_path_handoff_diagnostic(
            out_root,
            "cook_assets",
            args.profile,
            "cooked_asset_manifest",
        )
        if cook_assets_handoff_diagnostic:
            diagnostics.append(cook_assets_handoff_diagnostic)
        else:
            reported_asset_manifest = cook_assets_report_asset_manifest(
                out_root,
                args.profile,
            )
            if reported_asset_manifest:
                args.asset_manifest = str(reported_asset_manifest)
    asset_manifest = pack_asset_manifest_path(
        args,
        out_root,
        asset_manifest_argument_diagnostic,
        cook_assets_handoff_diagnostic,
        diagnostics,
    )
    stage_dir = out_root / "stages" / "pack"
    report_path = stage_dir / REPORT_FILE_NAME
    pack_path = pack_output_path(args, stage_dir, pack_file_diagnostic, diagnostics)
    previous_pack = resolve_pack_optional_path(
        args.previous_pack,
        "previous_pack",
        diagnostics,
    )
    delta_pack = resolve_pack_optional_path(
        args.delta_pack,
        "delta_pack",
        diagnostics,
    )
    packer = (
        None
        if packer_diagnostic
        else resolve_pack_optional_path(args.packer, "packer", diagnostics)
    )
    target_dir = (
        None
        if target_dir_diagnostic
        else resolve_pack_optional_path(args.target_dir, "target_dir", diagnostics)
    )

    command = (
        pack_command(
            args,
            repo_root,
            asset_manifest,
            stage_dir,
            report_path,
            pack_path,
            previous_pack=previous_pack,
            delta_pack=delta_pack,
            packer=packer,
            target_dir=target_dir,
        )
        if repo_root is not None
        and asset_manifest is not None
        and pack_path is not None
        and not diagnostics
        else None
    )
    print(f"zircon_export stage=Pack profile={args.profile}")
    print(f"asset_manifest={asset_manifest if asset_manifest else '<invalid>'}")
    print(f"pack={pack_path if pack_path else '<invalid>'}")
    if previous_pack:
        print(f"previous_pack={previous_pack}")
    if delta_pack:
        print(f"delta_pack={delta_pack}")
    print(f"report={report_path}")
    if command is not None:
        print(shell_join(command))
    else:
        print("command=<skipped>")
    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if diagnostics else 0

    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(f"Pack stage directory {stage_dir} could not be created: {error}")
        report = pack_preflight_failure_report(
            args=args,
            asset_manifest=asset_manifest,
            stage_dir=stage_dir,
            pack_path=pack_path,
            previous_pack=previous_pack,
            delta_pack=delta_pack,
            diagnostics=diagnostics,
        )
        print(json.dumps(report, indent=2))
        return 2
    manifest_diagnostic = (
        None if diagnostics else pack_asset_manifest_diagnostic(asset_manifest)
    )
    if manifest_diagnostic:
        diagnostics.append(manifest_diagnostic)
    if diagnostics:
        report = pack_preflight_failure_report(
            args=args,
            asset_manifest=asset_manifest,
            stage_dir=stage_dir,
            pack_path=pack_path,
            previous_pack=previous_pack,
            delta_pack=delta_pack,
            diagnostics=diagnostics,
        )
        write_report_targets([("Pack report", report_path)], report)
        print(json.dumps(report, indent=2))
        return 2
    if command is None:
        raise AssertionError("Pack command was not built after preflight passed")
    try:
        exit_code = subprocess.call(command, cwd=repo_root)
        if not report_path.is_file():
            diagnostics.append(
                f"Pack command exited with code {exit_code} but did not write report {report_path}"
            )
            report = pack_preflight_failure_report(
                args=args,
                asset_manifest=asset_manifest,
                stage_dir=stage_dir,
                pack_path=pack_path,
                previous_pack=previous_pack,
                delta_pack=delta_pack,
                diagnostics=diagnostics,
            )
            write_report_targets([("Pack report", report_path)], report)
            print(json.dumps(report, indent=2))
            return exit_code if exit_code != 0 else 2
        return exit_code
    except OSError as error:
        diagnostics.append(f"Pack command {command[0]} could not start: {error}")
        report = pack_preflight_failure_report(
            args=args,
            asset_manifest=asset_manifest,
            stage_dir=stage_dir,
            pack_path=pack_path,
            previous_pack=previous_pack,
            delta_pack=delta_pack,
            diagnostics=diagnostics,
        )
        write_report_targets([("Pack report", report_path)], report)
        print(json.dumps(report, indent=2))
        return 2


def pack_asset_manifest_argument_diagnostic(args: argparse.Namespace) -> str | None:
    return pack_optional_path_argument_diagnostic(
        getattr(args, "asset_manifest", None),
        "asset_manifest",
    )


def pack_file_argument_diagnostic(args: argparse.Namespace) -> str | None:
    return pack_optional_path_argument_diagnostic(
        getattr(args, "pack_file", None),
        "pack_file",
    )


def pack_optional_path_argument_diagnostic(
    value: object,
    label: str,
) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        return f"{label} argument must be a non-empty string"
    if value.strip() != value:
        return f"{label} argument must be a non-empty trimmed string"
    return None


def pack_asset_manifest_path(
    args: argparse.Namespace,
    out_root: Path,
    asset_manifest_argument_diagnostic: str | None,
    cook_assets_handoff_diagnostic: str | None,
    diagnostics: list[str],
) -> Path | None:
    if asset_manifest_argument_diagnostic or cook_assets_handoff_diagnostic:
        return None
    if args.asset_manifest is not None:
        return resolve_pack_optional_path(args.asset_manifest, "asset_manifest", diagnostics)
    return default_cooked_asset_manifest(out_root)


def pack_output_path(
    args: argparse.Namespace,
    stage_dir: Path,
    pack_file_argument_diagnostic: str | None,
    diagnostics: list[str],
) -> Path | None:
    if pack_file_argument_diagnostic:
        return None
    if args.pack_file is not None:
        return resolve_pack_optional_path(args.pack_file, "pack_file", diagnostics)
    return stage_dir / "assets.zrpack"


def resolve_pack_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def resolve_pack_stage_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="Pack")


def pack_delta_argument_diagnostics(args: argparse.Namespace) -> list[str]:
    previous_pack = getattr(args, "previous_pack", None)
    delta_pack = getattr(args, "delta_pack", None)
    diagnostics: list[str] = []
    previous_pack_diagnostic = pack_optional_path_argument_diagnostic(
        previous_pack,
        "previous_pack",
    )
    if previous_pack_diagnostic:
        diagnostics.append(previous_pack_diagnostic)
    delta_pack_diagnostic = pack_optional_path_argument_diagnostic(
        delta_pack,
        "delta_pack",
    )
    if delta_pack_diagnostic:
        diagnostics.append(delta_pack_diagnostic)
    if not diagnostics and ((previous_pack is None) != (delta_pack is None)):
        diagnostics.append("previous_pack and delta_pack must be supplied together")
    return diagnostics


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
    asset_manifest: Path | None,
    stage_dir: Path,
    pack_path: Path | None,
    previous_pack: Path | None,
    delta_pack: Path | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    return {
        "stage": "Pack",
        "profile": args.profile,
        "asset_manifest": str(asset_manifest) if asset_manifest else None,
        "pack": str(pack_path) if pack_path else None,
        "previous_pack": str(previous_pack) if previous_pack else None,
        "delta_pack": str(delta_pack) if delta_pack else None,
        "stage_output": str(stage_dir),
        "fatal": True,
        "diagnostics": diagnostics,
        "trim_report": {
            "included_assets": [],
            "trimmed_assets": [],
            "missing_dependencies": [],
            "duplicate_assets": [],
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
        "delta_apply_verified": False,
    }


def validate_command(
    args: argparse.Namespace,
    repo_root: Path,
    project_path: Path,
    stage_dir: Path,
    report_path: Path,
    *,
    validator: Path | None = None,
    target_dir: Path | None = None,
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

    if validator:
        return [str(validator), *validator_args]

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
    if target_dir:
        command.extend(["--target-dir", str(target_dir)])
    command.extend(["--", *validator_args])
    return command


def pack_command(
    args: argparse.Namespace,
    repo_root: Path,
    asset_manifest: Path,
    stage_dir: Path,
    report_path: Path,
    pack_path: Path,
    *,
    previous_pack: Path | None = None,
    delta_pack: Path | None = None,
    packer: Path | None = None,
    target_dir: Path | None = None,
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
    if previous_pack:
        packer_args.extend(["--previous-pack", str(previous_pack)])
    if delta_pack:
        packer_args.extend(["--delta-pack", str(delta_pack)])

    if packer:
        return [str(packer), *packer_args]

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
    if target_dir:
        command.extend(["--target-dir", str(target_dir)])
    command.extend(["--", *packer_args])
    return command


def resolve_repo_root(repo_root: str | None) -> Path:
    if repo_root:
        return resolve_user_path(repo_root)
    return default_repo_root()


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def resolve_validate_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="Validate")


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    return shlex.join(command)

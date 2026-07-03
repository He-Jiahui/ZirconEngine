"""Pack stage command, path preflight, and failure report assembly."""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
from pathlib import Path
from typing import Any, Sequence

from .pack_stage_paths import (
    pack_asset_manifest_argument_diagnostic,
    pack_asset_manifest_diagnostic,
    pack_asset_manifest_path,
    pack_delta_argument_diagnostics,
    pack_file_argument_diagnostic,
    pack_optional_path_argument_diagnostic,
    pack_output_path,
    resolve_pack_optional_path,
    resolve_pack_stage_path,
    resolve_user_path,
)
from .report_io import write_report_targets
from .stage_handoff import (
    cook_assets_report_asset_manifest,
    stage_report_path_handoff_diagnostic,
)
from .stage_handoff_strategy import (
    validate_report_requires_bundle_strategy_diagnostics,
)


REPORT_FILE_NAME = "report.json"


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


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def shell_join(command: Sequence[str]) -> str:
    return " ".join(shlex.quote(part) for part in command)

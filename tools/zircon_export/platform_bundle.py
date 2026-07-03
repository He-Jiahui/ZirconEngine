"""PlatformBundle stage and export-template validation for zircon_export."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from .native_dynamic_payload import native_dynamic_stage_payload_summary
from .platform_bundle_arguments import (
    default_repo_root,
    delta_pack_source_origin,
    host_source_origin_from_args,
    pack_source_origin,
    platform_bundle_argument_diagnostics,
    resolve_optional_platform_bundle_path_argument,
    resolve_platform_bundle_path,
    resolve_user_path,
)
from .platform_bundle_materialize import (
    materialize_platform_bundle,
    remove_platform_bundle_dir,
    template_bundle_manifest_path,
)
from .platform_bundle_native_plugins_payload import native_plugins_payload_for_bundle
from .platform_bundle_report_payload import (
    platform_bundle_manifest_payload,
    platform_bundle_stage_directory_failure_report,
    platform_bundle_stage_report_payload,
)
from .platform_bundle_strategy_handoff import (
    platform_bundle_strategy_handoff_diagnostics,
)
from .report_io import write_report_targets
from .export_template import (
    validate_export_template,
)
from .export_template_resolution import (
    resolve_export_template_from_root,
)
from .export_template_manifest import (
    validated_target_platform,
    workspace_engine_version,
)
from .stage_handoff import (
    compile_host_report_host_executable,
    native_dynamic_report_plugins_dir,
    pack_report_delta_pack_file,
    pack_report_pack_file,
    stage_report_metadata_handoff_diagnostic,
    stage_report_optional_path_handoff_diagnostic,
    stage_report_path_handoff_diagnostic,
)


REPORT_FILE_NAME = "report.json"


def run_platform_bundle(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    input_diagnostics = platform_bundle_argument_diagnostics(args)
    diagnostics: list[str] = list(input_diagnostics)
    repo_root = (
        resolve_platform_bundle_path(args.repo_root, "repo_root", diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    pack_path = (
        resolve_optional_platform_bundle_path_argument(args, "pack_file", diagnostics)
        if getattr(args, "pack_file", None)
        else out_root / "stages" / "pack" / "assets.zrpack"
    )
    delta_pack_path = resolve_optional_platform_bundle_path_argument(
        args,
        "delta_pack",
        diagnostics,
    )
    stage_dir = out_root / "stages" / "platform_bundle"
    bundle_dir = out_root / "bundle" / args.profile
    report_path = stage_dir / REPORT_FILE_NAME
    host_executable = resolve_optional_platform_bundle_path_argument(
        args,
        "host_executable",
        diagnostics,
    )
    host_source_origin = host_source_origin_from_args(args)
    native_plugins_dir = resolve_optional_platform_bundle_path_argument(
        args,
        "native_plugins_dir",
        diagnostics,
    )
    template_dir = resolve_optional_platform_bundle_path_argument(
        args,
        "template_dir",
        diagnostics,
    )
    template_root = resolve_optional_platform_bundle_path_argument(
        args,
        "template_root",
        diagnostics,
    )
    template_resolution: dict[str, Any] | None = None
    native_plugins_payload = native_dynamic_stage_payload_summary(
        out_root,
        args.profile,
        native_plugins_dir,
        diagnostics,
    )
    if args.host_executable is None:
        compile_host_handoff_diagnostic = stage_report_path_handoff_diagnostic(
            out_root,
            "compile_host",
            args.profile,
            "host_executable",
        )
        if compile_host_handoff_diagnostic:
            diagnostics.append(compile_host_handoff_diagnostic)
        else:
            reported_host = compile_host_report_host_executable(out_root, args.profile)
            if reported_host:
                host_executable = reported_host
                host_source_origin = "compile_host_report"
    if args.pack_file is None:
        pack_handoff_diagnostic = stage_report_path_handoff_diagnostic(
            out_root,
            "pack",
            args.profile,
            "pack",
        )
        if pack_handoff_diagnostic:
            diagnostics.append(pack_handoff_diagnostic)
        else:
            reported_pack = pack_report_pack_file(out_root, args.profile)
            if reported_pack:
                pack_path = reported_pack
    if (
        not getattr(args, "pack_file_explicit", False)
        and not getattr(args, "delta_pack_explicit", False)
    ):
        delta_pack_handoff_diagnostic = stage_report_optional_path_handoff_diagnostic(
            out_root,
            "pack",
            args.profile,
            "delta_pack",
        )
        if delta_pack_handoff_diagnostic:
            diagnostics.append(delta_pack_handoff_diagnostic)
        else:
            reported_delta_pack = pack_report_delta_pack_file(out_root, args.profile)
            if reported_delta_pack:
                delta_pack_path = reported_delta_pack
    if args.native_plugins_dir is None:
        native_dynamic_handoff_diagnostic = stage_report_metadata_handoff_diagnostic(
            out_root,
            "native_dynamic",
            args.profile,
        )
        if native_dynamic_handoff_diagnostic:
            diagnostics.append(native_dynamic_handoff_diagnostic)
        else:
            reported_native_plugins_dir = native_dynamic_report_plugins_dir(
                out_root,
                args.profile,
            )
            if reported_native_plugins_dir:
                native_plugins_dir = reported_native_plugins_dir
                native_plugins_payload = native_dynamic_stage_payload_summary(
                    out_root,
                    args.profile,
                    native_plugins_dir,
                    diagnostics,
                )
    diagnostics.extend(
        platform_bundle_strategy_handoff_diagnostics(
            out_root,
            args.profile,
            native_plugins_payload,
        )
    )

    if template_root and not template_dir and not diagnostics:
        expected_engine_version = (
            args.engine_version
            or (
                workspace_engine_version(repo_root, diagnostics)
                if repo_root is not None
                else None
            )
        )
        if not diagnostics:
            expected_target_platform = args.target_platform or validated_target_platform(
                out_root,
                args.profile,
            )
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

    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"PlatformBundle stage directory {stage_dir} could not be created: {error}"
        )
        report = platform_bundle_stage_directory_failure_report(
            profile=args.profile,
            bundle_dir=bundle_dir,
            diagnostics=diagnostics,
            template_resolution=template_resolution,
        )
        print(json.dumps(report, indent=2))
        return 2
    if bundle_dir.exists():
        if not remove_platform_bundle_dir(
            "stale PlatformBundle profile bundle",
            bundle_dir,
            diagnostics,
        ):
            diagnostics.append("bundle copy skipped because stale profile bundle cleanup failed")
    fatal = bool(diagnostics)
    if not fatal:
        try:
            bundle_dir.mkdir(parents=True, exist_ok=True)
        except OSError as error:
            diagnostics.append(
                f"PlatformBundle profile bundle {bundle_dir} could not be created: {error}"
            )
            fatal = True
    copied_host = None
    copied_pack = None
    copied_delta_pack = None
    copied_native_plugins = None
    copied_native_plugins_payload = None
    copied_template_files: list[dict[str, str]] = []
    template_report: dict[str, Any] | None = None

    if template_dir and not fatal:
        expected_engine_version = (
            args.engine_version
            or (
                workspace_engine_version(repo_root, diagnostics)
                if repo_root is not None
                else None
            )
        )
        if diagnostics:
            fatal = True
        else:
            expected_target_platform = args.target_platform or validated_target_platform(
                out_root,
                args.profile,
            )
            template_report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version=expected_engine_version,
                profile=args.profile,
                expected_target_platform=expected_target_platform,
            )
            diagnostics.extend(template_report["diagnostics"])
            fatal = bool(template_report["fatal"])
        if fatal:
            fatal = True
            diagnostics.append("template validation failed; bundle copy skipped")
        elif not host_executable and template_report.get("host_executable"):
            host_executable = Path(template_report["host_executable"])
            host_source_origin = "template"

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
                diagnostics,
            )
            if copied_native_plugins_payload is None:
                fatal = True
        copied_template_files = materialize_result["template_files"]
        if fatal:
            if bundle_dir.exists():
                remove_platform_bundle_dir(
                    "partial PlatformBundle profile bundle",
                    bundle_dir,
                    diagnostics,
                )
            copied_host = None
            copied_pack = None
            copied_delta_pack = None
            copied_native_plugins = None
            copied_native_plugins_payload = None
            copied_template_files = []

    payload_inputs = {
        "profile": args.profile,
        "template_resolution": template_resolution,
        "template_report": template_report,
        "copied_host": copied_host,
        "host_executable": host_executable,
        "host_source_origin": host_source_origin,
        "copied_pack": copied_pack,
        "pack_path": pack_path,
        "pack_source_origin": pack_source_origin(args),
        "copied_delta_pack": copied_delta_pack,
        "delta_pack_path": delta_pack_path,
        "delta_pack_source_origin": delta_pack_source_origin(args),
        "copied_native_plugins": copied_native_plugins,
        "copied_native_plugins_payload": copied_native_plugins_payload,
        "copied_template_files": copied_template_files,
    }
    manifest = platform_bundle_manifest_payload(**payload_inputs)
    bundle_manifest: Path | None = bundle_dir / "bundle.json"
    bundle_manifest_path = template_bundle_manifest_path(
        bundle_dir,
        template_report,
        diagnostics,
    )
    if bundle_manifest_path:
        bundle_manifest = bundle_manifest_path
    if not fatal:
        try:
            bundle_manifest.parent.mkdir(parents=True, exist_ok=True)
            bundle_manifest.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        except OSError as error:
            diagnostics.append(
                f"bundle manifest {bundle_manifest} could not be written: {error}"
            )
            fatal = True
            if bundle_dir.exists():
                remove_platform_bundle_dir(
                    "partial PlatformBundle profile bundle",
                    bundle_dir,
                    diagnostics,
                )
            bundle_manifest = None
    else:
        if bundle_dir.exists():
            remove_platform_bundle_dir(
                "partial PlatformBundle profile bundle",
                bundle_dir,
                diagnostics,
            )
        bundle_manifest = None
    report = platform_bundle_stage_report_payload(
        **payload_inputs,
        bundle_dir=bundle_dir,
        fatal=fatal,
        diagnostics=diagnostics,
        bundle_manifest=bundle_manifest,
    )
    report_written = write_report_targets([("PlatformBundle report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0

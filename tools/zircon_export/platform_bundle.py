"""PlatformBundle stage and export-template validation for zircon_export."""

from __future__ import annotations

import argparse
import json
import os
import shutil
from pathlib import Path
from typing import Any

from .native_dynamic_payload import native_dynamic_stage_payload_summary
from .native_dynamic_contract import NATIVE_DYNAMIC_LOADER_MANIFEST
from .path_resolve import resolve_stage_optional_path
from .report_io import write_report_targets
from .export_template import (
    resolve_export_template_from_root,
    resolve_bundle_child,
    validate_export_template,
    validated_target_platform,
    workspace_engine_version,
)
from .stage_handoff import (
    compile_host_report_host_executable,
    export_strategy_diagnostics,
    export_strategies_from_validate_report,
    load_stage_report_object,
    native_dynamic_payload_allowed,
    native_dynamic_report_plugins_dir,
    pack_report_delta_pack_file,
    pack_report_pack_file,
    stage_report_diagnostics_diagnostic,
    stage_report_fatal_diagnostic,
    stage_report_metadata_handoff_diagnostic,
    stage_report_identity_diagnostic,
    stage_report_optional_path_handoff_diagnostic,
    stage_report_path_handoff_diagnostic,
    validate_report_requires_bundle_strategy_diagnostics,
)


REPORT_FILE_NAME = "report.json"


def validate_report_uses_strategy(out_root: Path, profile: str, strategy: str) -> bool:
    report = load_trusted_validate_strategy_report(out_root, profile)
    if report is None:
        return False
    return strategy in export_strategies_from_validate_report(report)


def validate_report_allows_native_plugins(out_root: Path, profile: str) -> bool:
    report, diagnostic = load_trusted_validate_strategy_report_with_diagnostic(
        out_root,
        profile,
    )
    if diagnostic:
        return False
    return native_dynamic_payload_allowed(report)


def validate_report_strategy_diagnostics(
    out_root: Path,
    profile: str,
) -> list[str]:
    report, diagnostic = load_trusted_validate_strategy_report_with_diagnostic(
        out_root,
        profile,
    )
    if diagnostic:
        return [diagnostic]
    return export_strategy_diagnostics(report)


def load_trusted_validate_strategy_report(
    out_root: Path,
    profile: str,
) -> dict[str, Any] | None:
    report, _diagnostic = load_trusted_validate_strategy_report_with_diagnostic(
        out_root,
        profile,
    )
    return report


def load_trusted_validate_strategy_report_with_diagnostic(
    out_root: Path,
    profile: str,
) -> tuple[dict[str, Any] | None, str | None]:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    if not report_path.exists():
        return None, None
    report, diagnostic = load_stage_report_object(report_path, "Validate")
    if diagnostic:
        return None, diagnostic
    if report is None:
        return None, None
    metadata_diagnostic = (
        stage_report_identity_diagnostic(report, "validate")
        or stage_report_fatal_diagnostic(report, "validate")
        or stage_report_diagnostics_diagnostic(report, "validate")
    )
    if metadata_diagnostic:
        return None, metadata_diagnostic
    if report.get("fatal") or report.get("profile") != profile:
        return None, None
    return report, None


def validate_report_strategy_handoff_diagnostic(
    out_root: Path,
    profile: str,
) -> str | None:
    return stage_report_metadata_handoff_diagnostic(out_root, "validate", profile)


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
    validate_strategy_diagnostic = validate_report_strategy_handoff_diagnostic(
        out_root,
        args.profile,
    )
    if validate_strategy_diagnostic:
        diagnostics.append(validate_strategy_diagnostic)
    else:
        bundle_strategy_diagnostics = validate_report_requires_bundle_strategy_diagnostics(
            out_root,
            args.profile,
            "PlatformBundle",
        )
        if bundle_strategy_diagnostics:
            diagnostics.extend(bundle_strategy_diagnostics)
        else:
            diagnostics.extend(validate_report_strategy_diagnostics(out_root, args.profile))
        if native_plugins_payload is not None and not validate_report_allows_native_plugins(
            out_root,
            args.profile,
        ):
            diagnostics.append(
                "PlatformBundle report native_plugins requires the native_dynamic strategy"
            )
        elif native_plugins_payload is None:
            if validate_report_uses_strategy(out_root, args.profile, "native_dynamic"):
                diagnostics.append(
                    "NativeDynamic profile requires native plugins from a matching non-fatal "
                    "NativeDynamic stage report or --native-plugins-dir"
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
        report = {
            "stage": "PlatformBundle",
            "profile": args.profile,
            "bundle": str(bundle_dir),
            "fatal": True,
            "diagnostics": diagnostics,
            "template_resolution": template_resolution,
            "template": None,
            "host_executable": None,
            "host_source": None,
            "host_source_origin": None,
            "pack": None,
            "pack_source": None,
            "pack_source_origin": None,
            "delta_pack": None,
            "delta_pack_source": None,
            "delta_pack_source_origin": None,
            "native_plugins": None,
            "native_plugins_payload": None,
            "template_files": [],
            "bundle_manifest": None,
        }
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

    manifest = {
        "profile": args.profile,
        "template_resolution": template_resolution,
        "template": template_report,
        "host_executable": str(copied_host) if copied_host else None,
        "host_source": str(host_executable) if copied_host else None,
        "host_source_origin": host_source_origin if copied_host else None,
        "pack": str(copied_pack) if copied_pack else None,
        "pack_source": str(pack_path) if copied_pack else None,
        "pack_source_origin": pack_source_origin(args) if copied_pack else None,
        "delta_pack": str(copied_delta_pack) if copied_delta_pack else None,
        "delta_pack_source": str(delta_pack_path) if copied_delta_pack else None,
        "delta_pack_source_origin": (
            delta_pack_source_origin(args) if copied_delta_pack else None
        ),
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
    report = {
        "stage": "PlatformBundle",
        "profile": args.profile,
        "bundle": str(bundle_dir),
        "fatal": fatal,
        "diagnostics": diagnostics,
        "template_resolution": template_resolution,
        "template": template_report,
        "host_executable": str(copied_host) if copied_host else None,
        "host_source": str(host_executable) if copied_host else None,
        "host_source_origin": host_source_origin if copied_host else None,
        "pack": str(copied_pack) if copied_pack else None,
        "pack_source": str(pack_path) if copied_pack else None,
        "pack_source_origin": pack_source_origin(args) if copied_pack else None,
        "delta_pack": str(copied_delta_pack) if copied_delta_pack else None,
        "delta_pack_source": str(delta_pack_path) if copied_delta_pack else None,
        "delta_pack_source_origin": (
            delta_pack_source_origin(args) if copied_delta_pack else None
        ),
        "native_plugins": str(copied_native_plugins) if copied_native_plugins else None,
        "native_plugins_payload": copied_native_plugins_payload,
        "template_files": copied_template_files,
        "bundle_manifest": str(bundle_manifest) if bundle_manifest else None,
    }
    report_written = write_report_targets([("PlatformBundle report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0


def host_source_origin_from_args(args: argparse.Namespace) -> str | None:
    origin = getattr(args, "host_executable_source_origin", None)
    if isinstance(origin, str) and origin:
        return origin
    if getattr(args, "host_executable_explicit", False):
        return "argument"
    if getattr(args, "host_executable", None) is not None:
        return "argument"
    return None


def pack_source_origin(args: argparse.Namespace) -> str:
    return "argument" if getattr(args, "pack_file_explicit", False) else "pack_report"


def delta_pack_source_origin(args: argparse.Namespace) -> str:
    return "argument" if getattr(args, "delta_pack_explicit", False) else "pack_report"


def platform_bundle_argument_diagnostics(args: argparse.Namespace) -> list[str]:
    diagnostics: list[str] = []
    for field in (
        "host_executable",
        "pack_file",
        "delta_pack",
        "native_plugins_dir",
    ):
        value = getattr(args, field, None)
        if value is not None and (not isinstance(value, str) or not value):
            diagnostics.append(f"{field} argument must be a non-empty string")
    return diagnostics


def resolve_optional_platform_bundle_path_argument(
    args: argparse.Namespace,
    field: str,
    diagnostics: list[str],
) -> Path | None:
    value = getattr(args, field, None)
    if value is None:
        return None
    if not isinstance(value, str) or not value:
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{field} {value} could not be resolved: {error}")
        return None


def resolve_platform_bundle_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="PlatformBundle")


def resolve_platform_bundle_copy_path(
    label: str,
    path: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        diagnostics.append(
            f"{label} {path} could not be resolved during bundle copy: {error}"
        )
        return None


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
    try:
        bundle_root.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"PlatformBundle bundle root {bundle_root} could not be created: {error}"
        )
        return {
            "fatal": True,
            "profile": profile,
            "bundle_root": bundle_root,
            "host_executable": None,
            "pack": None,
            "delta_pack": None,
            "native_plugins": None,
            "template_files": [],
        }

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
    if host_executable and not host_destination:
        fatal = True
    if not pack_destination:
        fatal = True
    if delta_pack_path and not delta_pack_destination:
        fatal = True

    if host_executable:
        host_diagnostic = platform_bundle_file_input_diagnostic(
            "host executable",
            host_executable,
        )
        if host_diagnostic:
            diagnostics.append(host_diagnostic)
            fatal = True

    pack_diagnostic = platform_bundle_file_input_diagnostic("pack file", pack_path)
    if pack_diagnostic:
        diagnostics.append(pack_diagnostic)
        fatal = True

    if delta_pack_path:
        delta_pack_diagnostic = platform_bundle_file_input_diagnostic(
            "delta pack file",
            delta_pack_path,
        )
        if delta_pack_diagnostic:
            diagnostics.append(delta_pack_diagnostic)
            fatal = True

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
            if host_destination and host_executable:
                resolved_source = resolve_platform_bundle_copy_path(
                    "template file",
                    source,
                    diagnostics,
                )
                resolved_host = resolve_platform_bundle_copy_path(
                    "host executable",
                    host_executable,
                    diagnostics,
                )
                if resolved_source is None or resolved_host is None:
                    fatal = True
                    continue
                if resolved_source == resolved_host:
                    continue
            if not source.exists():
                diagnostics.append(f"template file {source} does not exist during bundle copy")
                fatal = True
                continue
            if not source.is_file():
                diagnostics.append(f"template file {source} is not a file during bundle copy")
                fatal = True
                continue
            if not copy_platform_bundle_file(
                "template file",
                source,
                destination,
                diagnostics,
            ):
                fatal = True
                continue
            copied_template_files.append(
                {
                    "source": str(source),
                    "destination": str(destination),
                }
            )

    if host_executable and host_destination and not fatal:
        if copy_platform_bundle_file(
            "host executable",
            host_executable,
            host_destination,
            diagnostics,
        ):
            copied_host = host_destination
        else:
            fatal = True

    if not fatal and pack_destination:
        if copy_platform_bundle_file(
            "pack file",
            pack_path,
            pack_destination,
            diagnostics,
        ):
            copied_pack = pack_destination
        else:
            fatal = True

    if delta_pack_path and not fatal and delta_pack_destination:
        if copy_platform_bundle_file(
            "delta pack file",
            delta_pack_path,
            delta_pack_destination,
            diagnostics,
        ):
            copied_delta_pack = delta_pack_destination
        else:
            fatal = True

    if native_plugins_dir:
        plugins_destination = resolve_bundle_child(bundle_root, "plugins", diagnostics)
        if plugins_destination and native_plugins_dir.exists() and native_plugins_dir.is_dir():
            if plugins_destination.exists():
                try:
                    shutil.rmtree(plugins_destination)
                except OSError as error:
                    diagnostics.append(
                        f"native plugins destination {plugins_destination} could not be removed: {error}"
                    )
                    fatal = True
            if not fatal:
                filtered_template_files = template_files_outside_directory(
                    copied_template_files,
                    plugins_destination,
                    diagnostics,
                )
                if filtered_template_files is None:
                    fatal = True
                else:
                    copied_template_files = filtered_template_files
                if not fatal and copy_dir_contents(
                    native_plugins_dir,
                    plugins_destination,
                    diagnostics,
                ):
                    copied_native_plugins = plugins_destination
                else:
                    fatal = True
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


def platform_bundle_file_input_diagnostic(label: str, path: Path) -> str | None:
    if not path.exists():
        return f"{label} {path} does not exist"
    if not path.is_file():
        return f"{label} {path} is not a file"
    return None


def copy_platform_bundle_file(
    label: str,
    source: Path,
    destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    except OSError as error:
        diagnostics.append(
            f"{label} {source} could not be copied to {destination}: {error}"
        )
        return False
    return True


def remove_platform_bundle_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> bool:
    try:
        shutil.rmtree(directory)
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be removed: {error}")
        return False
    return True


def copy_dir_contents(
    source: Path,
    destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        destination.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"native plugins directory {destination} could not be created: {error}"
        )
        return False
    try:
        children = list(source.iterdir())
    except OSError as error:
        diagnostics.append(
            f"native plugins directory {source} could not be listed: {error}"
        )
        return False

    copied = True
    for child in children:
        target = destination / child.name
        if child.is_dir():
            if not copy_dir_contents(child, target, diagnostics):
                copied = False
        else:
            if not copy_platform_bundle_file(
                "native plugins file",
                child,
                target,
                diagnostics,
            ):
                copied = False
    return copied


def native_plugins_payload_for_bundle(
    payload: dict[str, Any],
    bundle_plugins_dir: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    bundled_payload = dict(payload)
    bundled_payload["bundle_path"] = str(bundle_plugins_dir)
    bundled_payload["loader_manifest"] = str(
        bundle_plugins_dir / NATIVE_DYNAMIC_LOADER_MANIFEST
    )
    materialized_packages = payload.get("materialized_packages")
    if isinstance(materialized_packages, list):
        source = payload.get("source")
        source_dir = Path(source).expanduser() if isinstance(source, str) else None
        bundled_packages: list[object] = []
        for index, package in enumerate(materialized_packages):
            bundled_package = native_plugins_package_for_bundle(
                package,
                source_dir,
                bundle_plugins_dir,
                diagnostics,
                index,
            )
            if bundled_package is None:
                return None
            bundled_packages.append(bundled_package)
        bundled_payload["materialized_packages"] = bundled_packages
    return bundled_payload


def template_files_outside_directory(
    template_files: list[dict[str, str]],
    removed_directory: Path,
    diagnostics: list[str],
) -> list[dict[str, str]] | None:
    try:
        resolved_removed_directory = removed_directory.resolve()
    except OSError as error:
        diagnostics.append(
            "PlatformBundle template_files removed directory "
            f"{removed_directory} could not be resolved: {error}"
        )
        return None
    retained: list[dict[str, str]] = []
    for entry in template_files:
        destination = entry.get("destination")
        if not destination:
            retained.append(entry)
            continue
        try:
            resolved_destination = Path(destination).expanduser().resolve()
        except OSError as error:
            diagnostics.append(
                "PlatformBundle template_files destination "
                f"{destination} could not be resolved: {error}"
            )
            return None
        try:
            resolved_destination.relative_to(resolved_removed_directory)
        except ValueError:
            retained.append(entry)
    return retained


def native_plugins_package_for_bundle(
    package: object,
    source_dir: Path | None,
    bundle_plugins_dir: Path,
    diagnostics: list[str],
    index: int,
) -> object | None:
    if not isinstance(package, dict):
        return package
    bundled_package = dict(package)
    destination = package.get("destination")
    relative_destination = native_plugins_relative_payload_path(
        destination,
        source_dir,
        diagnostics,
        f"native_plugins_payload materialized_packages[{index}] destination",
    )
    if relative_destination is None:
        return None
    bundled_package["destination"] = str(bundle_plugins_dir / relative_destination)
    package_report = package.get("package_report")
    relative_package_report = native_plugins_relative_payload_path(
        package_report,
        source_dir,
        diagnostics,
        f"native_plugins_payload materialized_packages[{index}] package_report",
    )
    if package_report is not None and relative_package_report is None:
        return None
    if relative_package_report is not None:
        bundled_package["package_report"] = str(bundle_plugins_dir / relative_package_report)
    return bundled_package


def native_plugins_relative_payload_path(
    value: object,
    source_dir: Path | None,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    if not isinstance(value, str) or source_dir is None:
        return None
    try:
        return Path(value).expanduser().resolve().relative_to(source_dir.resolve())
    except OSError as error:
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None
    except ValueError:
        diagnostics.append(
            f"{label} {value} is outside native_plugins_payload source {source_dir}"
        )
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


def resolve_repo_root(repo_root: str | None) -> Path:
    if repo_root:
        return resolve_user_path(repo_root)
    return default_repo_root()


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()

"""NativeDynamic package export report stage."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .native_build import native_dynamic_build_plan
from .native_build_execution import execute_native_dynamic_build_plan
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_STAGE,
    REPORT_FILE_NAME,
)
from .native_dynamic_cli_options import (
    default_repo_root,
    native_dynamic_cli_optional_trimmed_string,
    native_dynamic_cli_string_array,
    native_dynamic_signing_platforms,
    native_dynamic_signing_profile,
    resolve_user_path,
)
from .native_dynamic_materialize import materialize_native_dynamic_packages
from .native_dynamic_materialize_io import (
    native_dynamic_cas_scope,
    reset_native_dynamic_plugins_dir,
)
from .native_dynamic_stage_payload_finalize import (
    finalize_native_dynamic_stage_payload,
)
from .report_io import write_report_targets
from .native_signing import (
    execute_native_dynamic_notarization,
    execute_native_dynamic_signing,
    native_dynamic_signing_command_template,
)
from .native_dynamic_plan import (
    load_validate_report,
    native_dynamic_artifact_extensions,
    native_dynamic_loadable_artifact_extensions,
    native_dynamic_package_exports,
    native_dynamic_package_ids,
    native_dynamic_target_platform,
    resolve_native_dynamic_path,
    validate_package_selection_matches_exports,
)


def run_native_dynamic(args: argparse.Namespace) -> int:
    mutates_native_artifacts = (
        getattr(args, "native_dynamic_sign_command", None) is not None
        or getattr(args, "native_dynamic_notarize_command", None) is not None
    )
    with native_dynamic_cas_scope(
        allow_hardlinks=not mutates_native_artifacts,
    ):
        return _run_native_dynamic(args)


def _run_native_dynamic(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    diagnostics: list[str] = []
    repo_root = resolve_native_dynamic_path(
        args.repo_root,
        "repo_root",
        diagnostics,
    ) if args.repo_root else default_repo_root()
    validate_report = (
        resolve_native_dynamic_path(
            args.validate_report,
            "validate_report",
            diagnostics,
        )
        if args.validate_report
        else out_root / "stages" / "validate" / REPORT_FILE_NAME
    )
    stage_dir = out_root / "stages" / NATIVE_DYNAMIC_STAGE
    plugin_root = (
        resolve_native_dynamic_path(
            args.native_plugin_root,
            "native_plugin_root",
            diagnostics,
        )
        if getattr(args, "native_plugin_root", None)
        else repo_root / "zircon_plugins" if repo_root else None
    )
    loader_manifest = stage_dir / "plugins" / NATIVE_DYNAMIC_LOADER_MANIFEST
    report_path = stage_dir / REPORT_FILE_NAME

    print(f"zircon_export stage=NativeDynamic profile={args.profile}")
    print(f"validate_report={validate_report if validate_report else '<invalid>'}")
    print(f"native_plugin_root={plugin_root if plugin_root else '<invalid>'}")
    print(f"stage_output={stage_dir}")
    print(f"loader_manifest={loader_manifest}")
    print(f"report={report_path}")

    validate_payload = (
        None
        if validate_report is None
        else load_validate_report(validate_report, args.profile, diagnostics)
    )
    native_dynamic_packages = native_dynamic_package_ids(validate_payload, diagnostics)
    package_exports = native_dynamic_package_exports(validate_payload, diagnostics)
    if package_exports is not None:
        validate_package_selection_matches_exports(
            native_dynamic_packages,
            package_exports,
            diagnostics,
        )
    target_platform = native_dynamic_target_platform(validate_payload, diagnostics)
    artifact_extensions = native_dynamic_artifact_extensions(target_platform)
    fatal = (
        repo_root is None
        or validate_report is None
        or plugin_root is None
        or validate_payload is None
        or package_exports is None
        or bool(diagnostics)
    )
    native_dynamic_build_enabled = bool(getattr(args, "native_dynamic_build", False))
    native_build_plan: dict[str, object] | None = None
    native_build_execution: dict[str, object] = {
        "enabled": native_dynamic_build_enabled,
        "fatal": False,
        "skipped": False,
        "diagnostics": [],
        "package_count": 0,
        "packages": [],
    }
    native_dynamic_build_features = native_dynamic_cli_string_array(
        getattr(args, "native_dynamic_build_feature", []),
        "NativeDynamic native build features",
        diagnostics,
    )
    native_signing_command = native_dynamic_cli_optional_trimmed_string(
        getattr(args, "native_dynamic_sign_command", None),
        "NativeDynamic signing command",
        diagnostics,
    )
    native_signing_enabled = getattr(args, "native_dynamic_sign_command", None) is not None
    native_signing_args = native_dynamic_cli_string_array(
        getattr(args, "native_dynamic_sign_arg", []),
        "NativeDynamic signing args",
        diagnostics,
    )
    native_signing_profile = native_dynamic_signing_profile(
        getattr(args, "native_dynamic_sign_profile", None),
        "NativeDynamic signing profile",
        diagnostics,
    )
    native_signing_platforms = native_dynamic_signing_platforms(
        getattr(args, "native_dynamic_sign_platform", []),
        "NativeDynamic signing allowed platforms",
        diagnostics,
    )
    native_signing: dict[str, object] = {
        "enabled": native_signing_enabled,
        "profile": native_signing_profile,
        "target_platform": target_platform,
        "allowed_platforms": native_signing_platforms,
        "platform_allowed": True,
        "fatal": False,
        "diagnostics": [],
        "package_count": 0,
        "packages": [],
    }
    native_notarization_command = native_dynamic_cli_optional_trimmed_string(
        getattr(args, "native_dynamic_notarize_command", None),
        "NativeDynamic notarization command",
        diagnostics,
    )
    native_notarization_enabled = (
        getattr(args, "native_dynamic_notarize_command", None) is not None
    )
    native_notarization_args = native_dynamic_cli_string_array(
        getattr(args, "native_dynamic_notarize_arg", []),
        "NativeDynamic notarization args",
        diagnostics,
    )
    native_notarization_profile = native_dynamic_signing_profile(
        getattr(args, "native_dynamic_notarize_profile", None),
        "NativeDynamic notarization profile",
        diagnostics,
    )
    native_notarization_platforms = native_dynamic_signing_platforms(
        getattr(args, "native_dynamic_notarize_platform", []),
        "NativeDynamic notarization allowed platforms",
        diagnostics,
    )
    native_notarization: dict[str, object] = {
        "enabled": native_notarization_enabled,
        "profile": native_notarization_profile,
        "target_platform": target_platform,
        "allowed_platforms": native_notarization_platforms,
        "platform_allowed": True,
        "fatal": False,
        "diagnostics": [],
        "package_count": 0,
        "packages": [],
    }
    fatal = fatal or bool(diagnostics)

    if args.dry_run:
        return 2 if fatal else 0

    materialized_packages: list[dict[str, object]] = []
    file_manifest: list[dict[str, object]] = []
    content_hash: str | None = None
    payload_cleaned = False
    cleanup_reason: str | None = None
    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic stage directory {stage_dir} could not be created: {error}"
        )
        report = {
            "stage": "NativeDynamic",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": diagnostics,
            "validate_report": str(validate_report) if validate_report else None,
            "stage_output": str(stage_dir),
            "native_plugin_root": str(plugin_root) if plugin_root else None,
            "target_platform": target_platform,
            "artifact_extensions": sorted(artifact_extensions),
            "plugins_dir": None,
            "loader_manifest": None,
            "native_dynamic_packages": native_dynamic_packages,
            "package_exports": package_exports or [],
            "package_count": len(package_exports or []),
            "native_build_plan": native_build_plan,
            "native_build_execution": native_build_execution,
            "native_signing": native_signing,
            "native_notarization": native_notarization,
            "materialized_packages": materialized_packages,
            "file_manifest": file_manifest,
            "content_hash": content_hash,
            "payload_cleaned": payload_cleaned,
            "cleanup_reason": cleanup_reason,
        }
        print(json.dumps(report, indent=2))
        return 2
    if not fatal and package_exports is not None:
        if not reset_native_dynamic_plugins_dir(stage_dir, diagnostics):
            fatal = True
            cleanup_reason = "stale_payload_cleanup_failed"
        if not fatal:
            loadable_artifact_extensions = native_dynamic_loadable_artifact_extensions(
                artifact_extensions
            )
            source_packages: dict[str, Path] = {}
            materialized_packages = materialize_native_dynamic_packages(
                package_exports,
                plugin_root,
                stage_dir,
                artifact_extensions,
                loadable_artifact_extensions,
                source_packages,
                diagnostics,
                require_source_native_artifacts=not native_dynamic_build_enabled,
            )
            build_plan_diagnostics: list[str] = []
            native_build_plan = native_dynamic_build_plan(
                repo_root=repo_root,
                stage_dir=stage_dir,
                target_dir=(
                    Path(args.target_dir).expanduser()
                    if getattr(args, "target_dir", None)
                    else None
                ),
                package_exports=package_exports,
                source_packages=source_packages,
                validate_payload=validate_payload,
                target_platform=target_platform,
                cargo=getattr(args, "cargo", "cargo"),
                locked=not getattr(args, "no_locked", False),
                offline=bool(getattr(args, "offline", False)),
                build_features=native_dynamic_build_features,
                diagnostics=build_plan_diagnostics,
            )
            if native_dynamic_build_enabled:
                if diagnostics:
                    native_build_execution["skipped"] = True
                    native_build_execution["skip_reason"] = "materialization_diagnostics"
                elif native_build_plan.get("fatal"):
                    diagnostics.extend(build_plan_diagnostics)
                    native_build_execution = {
                        "enabled": True,
                        "fatal": True,
                        "skipped": False,
                        "diagnostics": list(build_plan_diagnostics),
                        "package_count": 0,
                        "packages": [],
                    }
                else:
                    build_execution_diagnostics: list[str] = []
                    native_build_execution = execute_native_dynamic_build_plan(
                        native_build_plan=native_build_plan,
                        repo_root=repo_root,
                        materialized_packages=materialized_packages,
                        diagnostics=build_execution_diagnostics,
                    )
                    diagnostics.extend(build_execution_diagnostics)
            fatal = bool(diagnostics)
            if not fatal:
                signing_command_template = native_dynamic_signing_command_template(
                    command=native_signing_command,
                    extra_args=native_signing_args,
                )
                if native_signing_enabled and not signing_command_template:
                    diagnostics.append(
                        "NativeDynamic signing command is enabled but has no command parts"
                    )
                elif native_signing_enabled:
                    signing_diagnostics: list[str] = []
                    native_signing = execute_native_dynamic_signing(
                        materialized_packages=materialized_packages,
                        loadable_artifact_extensions=loadable_artifact_extensions,
                        command_template=signing_command_template,
                        target_platform=target_platform,
                        signing_profile=native_signing_profile,
                        allowed_platforms=native_signing_platforms,
                        diagnostics=signing_diagnostics,
                    )
                    diagnostics.extend(signing_diagnostics)
            fatal = bool(diagnostics)
            if not fatal:
                notarization_command_template = native_dynamic_signing_command_template(
                    command=native_notarization_command,
                    extra_args=native_notarization_args,
                )
                if native_notarization_enabled and not notarization_command_template:
                    diagnostics.append(
                        "NativeDynamic notarization command is enabled but has no command parts"
                    )
                elif native_notarization_enabled:
                    notarization_diagnostics: list[str] = []
                    native_notarization = execute_native_dynamic_notarization(
                        materialized_packages=materialized_packages,
                        loadable_artifact_extensions=loadable_artifact_extensions,
                        command_template=notarization_command_template,
                        target_platform=target_platform,
                        signing_profile=native_signing_profile,
                        notarization_profile=native_notarization_profile,
                        allowed_platforms=native_notarization_platforms,
                        diagnostics=notarization_diagnostics,
                    )
                    diagnostics.extend(notarization_diagnostics)
            fatal = bool(diagnostics)
            if not fatal:
                file_manifest, content_hash = finalize_native_dynamic_stage_payload(
                    package_exports,
                    stage_dir,
                    materialized_packages,
                    loadable_artifact_extensions,
                    diagnostics,
                )
            fatal = bool(diagnostics)
            if fatal:
                cleanup_succeeded = reset_native_dynamic_plugins_dir(
                    stage_dir,
                    diagnostics,
                )
                materialized_packages = []
                payload_cleaned = cleanup_succeeded
                cleanup_reason = (
                    "fatal_diagnostics" if cleanup_succeeded else "fatal_cleanup_failed"
                )

    report = {
        "stage": "NativeDynamic",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report) if validate_report else None,
        "stage_output": str(stage_dir),
        "native_plugin_root": str(plugin_root) if plugin_root else None,
        "target_platform": target_platform,
        "artifact_extensions": sorted(artifact_extensions),
        "plugins_dir": str(stage_dir / "plugins") if not fatal else None,
        "loader_manifest": str(loader_manifest) if not fatal else None,
        "native_dynamic_packages": native_dynamic_packages,
        "package_exports": package_exports or [],
        "package_count": len(package_exports or []),
        "native_build_plan": native_build_plan,
        "native_build_execution": native_build_execution,
        "native_signing": native_signing,
        "native_notarization": native_notarization,
        "materialized_packages": materialized_packages,
        "file_manifest": file_manifest,
        "content_hash": content_hash,
        "payload_cleaned": payload_cleaned,
        "cleanup_reason": cleanup_reason,
    }
    report_written = write_report_targets([("NativeDynamic report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0

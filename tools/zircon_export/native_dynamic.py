"""NativeDynamic package export report stage."""

from __future__ import annotations

import argparse
import json
import shutil
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .native_build import (
    execute_native_dynamic_build_plan,
    native_dynamic_build_plan,
)
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    NATIVE_DYNAMIC_RESOURCE_DIRS,
    NATIVE_DYNAMIC_STAGE,
    REPORT_FILE_NAME,
)
from .native_dynamic_payload import (
    native_dynamic_content_hash,
    native_dynamic_file_manifest,
    native_dynamic_package_loadable_artifacts,
    native_dynamic_package_payload_file_manifest,
)
from .report_io import write_report_targets
from .native_signing import (
    execute_native_dynamic_notarization,
    execute_native_dynamic_signing,
    native_dynamic_signing_command_template,
)
from .native_dynamic_templates import (
    native_dynamic_package_report_template,
    native_plugin_load_manifest_template,
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


@dataclass(frozen=True)
class PackageManifestRead:
    manifest_id: str | None = None
    error: str | None = None


def run_native_dynamic(args: argparse.Namespace) -> int:
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
                finalize_native_dynamic_package_reports(
                    package_exports,
                    stage_dir,
                    materialized_packages,
                    loadable_artifact_extensions,
                    diagnostics,
                )
            fatal = bool(diagnostics)
            if not fatal:
                try:
                    loader_manifest.parent.mkdir(parents=True, exist_ok=True)
                    loader_manifest.write_text(
                        native_plugin_load_manifest_template(package_exports),
                        encoding="utf-8",
                    )
                except OSError as error:
                    diagnostics.append(
                        f"NativeDynamic loader manifest {loader_manifest} could not be written: {error}"
                    )
                else:
                    file_manifest = native_dynamic_file_manifest(stage_dir, diagnostics)
                fatal = bool(diagnostics)
            if not fatal:
                content_hash = native_dynamic_content_hash(file_manifest)
            else:
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


def native_dynamic_cli_optional_trimmed_string(
    value: object,
    field: str,
    diagnostics: list[str],
) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str):
        diagnostics.append(f"{field} must be a string")
        return None
    if not value or value.strip() != value:
        diagnostics.append(f"{field} must be a non-empty trimmed string")
        return None
    return value


def native_dynamic_cli_string_array(
    value: object,
    field: str,
    diagnostics: list[str],
    *,
    lowercase: bool = False,
) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        value = [value]
    values: list[str] = []
    seen: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str):
            diagnostics.append(f"{field}[{index}] must be a string")
            continue
        if not item or item.strip() != item:
            diagnostics.append(f"{field}[{index}] must be a non-empty trimmed string")
            continue
        normalized = item.lower() if lowercase else item
        if normalized in seen:
            continue
        values.append(normalized)
        seen.add(normalized)
    return values


def native_dynamic_signing_profile(
    value: object,
    field: str,
    diagnostics: list[str],
) -> str | None:
    return native_dynamic_cli_optional_trimmed_string(value, field, diagnostics)


def native_dynamic_signing_platforms(
    value: object,
    field: str,
    diagnostics: list[str],
) -> list[str]:
    return native_dynamic_cli_string_array(value, field, diagnostics, lowercase=True)

def reset_native_dynamic_plugins_dir(
    stage_dir: Path,
    diagnostics: list[str],
) -> bool:
    plugins_dir = stage_dir / "plugins"
    if plugins_dir.exists():
        if not remove_native_dynamic_dir(
            "NativeDynamic plugins directory",
            plugins_dir,
            diagnostics,
        ):
            return False
    try:
        plugins_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic plugins directory {plugins_dir} could not be created: {error}"
        )
        return False
    return True


def remove_native_dynamic_dir(
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


def list_native_dynamic_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> list[Path] | None:
    try:
        return list(directory.iterdir())
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be listed: {error}")
        return None

def materialize_native_dynamic_packages(
    package_exports: list[dict[str, Any]],
    plugin_root: Path,
    stage_dir: Path,
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
    source_packages: dict[str, Path],
    diagnostics: list[str],
    *,
    require_source_native_artifacts: bool,
) -> list[dict[str, object]]:
    materialized_packages: list[dict[str, object]] = []
    package_root = stage_dir / "plugins"
    copied_directories: set[str] = set()
    for package_export in package_exports:
        package_id = str(package_export["package_id"])
        directory = str(package_export["directory"])
        if directory in copied_directories:
            diagnostics.append(
                f"native dynamic package {package_id} resolves to duplicate output directory plugins/{directory}"
            )
            continue
        copied_directories.add(directory)

        diagnostics_before_source_lookup = len(diagnostics)
        source = find_native_package_dir(plugin_root, package_id, diagnostics)
        if source is None:
            if len(diagnostics) == diagnostics_before_source_lookup:
                diagnostics.append(
                    f"native dynamic package {package_id} was selected but no plugin.toml was found under {plugin_root}"
                )
            continue
        source_packages[package_id] = source
        destination = resolve_stage_child(package_root, directory, diagnostics)
        if destination is None:
            continue
        if not copy_native_dynamic_package(
            source,
            destination,
            package_id,
            artifact_extensions,
            loadable_artifact_extensions,
            diagnostics,
            require_source_native_artifacts=require_source_native_artifacts,
        ):
            if destination.exists():
                remove_native_dynamic_dir(
                    f"NativeDynamic package {package_id} partial package",
                    destination,
                    diagnostics,
                )
            continue
        package_report = destination / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            stage_dir,
            destination,
            loadable_artifact_extensions,
            diagnostics,
        )
        materialized_packages.append(
            {
                "package_id": package_id,
                "source": str(source),
                "destination": str(destination),
                "package_report": str(package_report),
                "loadable_artifact_count": len(loadable_artifacts),
                "loadable_artifacts": loadable_artifacts,
            }
        )
    return materialized_packages


def finalize_native_dynamic_package_reports(
    package_exports: list[dict[str, Any]],
    stage_dir: Path,
    materialized_packages: list[dict[str, object]],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
) -> None:
    package_exports_by_id = {
        str(package_export["package_id"]): package_export
        for package_export in package_exports
    }
    for materialized_package in materialized_packages:
        package_id = materialized_package.get("package_id")
        destination = materialized_package.get("destination")
        if not isinstance(package_id, str) or not isinstance(destination, str):
            diagnostics.append("NativeDynamic materialized package entry is malformed")
            continue
        package_export = package_exports_by_id.get(package_id)
        if package_export is None:
            diagnostics.append(
                f"NativeDynamic materialized package {package_id} has no package export"
            )
            continue
        package_dir = Path(destination)
        package_report = package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        payload_file_manifest = native_dynamic_package_payload_file_manifest(
            package_dir,
            diagnostics,
        )
        if diagnostics:
            continue
        try:
            package_report.write_text(
                native_dynamic_package_report_template(package_export, payload_file_manifest),
                encoding="utf-8",
            )
        except OSError as error:
            diagnostics.append(
                f"NativeDynamic package {package_id} report {package_report} could not be written: {error}"
            )
            continue
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            stage_dir,
            package_dir,
            loadable_artifact_extensions,
            diagnostics,
        )
        if diagnostics:
            continue
        materialized_package["package_report"] = str(package_report)
        materialized_package["loadable_artifact_count"] = len(loadable_artifacts)
        materialized_package["loadable_artifacts"] = loadable_artifacts


def find_native_package_dir(
    plugin_root: Path,
    package_id: str,
    diagnostics: list[str],
) -> Path | None:
    if not plugin_root.exists() or not plugin_root.is_dir():
        return None

    direct = plugin_root / package_id
    direct_manifest = direct / "plugin.toml"
    if direct_manifest.exists():
        manifest_read = read_package_manifest_id(direct_manifest)
        if manifest_read.error is not None:
            diagnostics.append(
                f"native dynamic package {package_id} direct manifest {manifest_read.error}"
            )
            return None
        manifest_id = manifest_read.manifest_id
        if manifest_id == package_id:
            return direct
        if manifest_id is not None:
            diagnostics.append(
                f"native dynamic package {package_id} direct manifest id {manifest_id} does not match selected package {package_id}"
            )
            return None
        diagnostics.append(
            f"native dynamic package {package_id} direct manifest id must be a non-empty string"
        )
        return None

    matches: list[Path] = []
    manifest_diagnostics: list[str] = []
    stack = [plugin_root]
    while stack:
        current = stack.pop()
        children = list_native_dynamic_dir(
            f"native dynamic package {package_id} source search directory",
            current,
            diagnostics,
        )
        if children is None:
            return None
        for child in children:
            if not child.is_dir():
                continue
            manifest_path = child / "plugin.toml"
            if manifest_path.exists():
                manifest_read = read_package_manifest_id(manifest_path)
                if manifest_read.error is not None:
                    manifest_diagnostics.append(
                        f"native dynamic package {package_id} source manifest {manifest_path} {manifest_read.error}"
                    )
                elif manifest_read.manifest_id == package_id:
                    matches.append(child)
                elif manifest_read.manifest_id is None:
                    manifest_diagnostics.append(
                        f"native dynamic package {package_id} source manifest {manifest_path} id must be a non-empty string"
                    )
            stack.append(child)
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        diagnostics.append(
            f"native dynamic package {package_id} has multiple source package manifests: "
            + ", ".join(str(match) for match in sorted(matches))
        )
    if manifest_diagnostics:
        diagnostics.extend(manifest_diagnostics)
    return None


def read_package_manifest_id(path: Path) -> PackageManifestRead:
    if not path.exists():
        return PackageManifestRead()
    if not path.is_file():
        return PackageManifestRead(error=f"{path} is not a file")
    try:
        with path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        return PackageManifestRead(error=f"could not be parsed: {error}")
    except OSError as error:
        return PackageManifestRead(error=f"could not be read: {error}")
    if "id" not in manifest:
        return PackageManifestRead()
    manifest_id = manifest.get("id")
    if isinstance(manifest_id, str):
        if not manifest_id:
            return PackageManifestRead()
        if manifest_id.strip() != manifest_id:
            return PackageManifestRead(error="id must be a non-empty trimmed string")
        return PackageManifestRead(manifest_id=manifest_id)
    return PackageManifestRead(error="id must be a string")


def copy_native_dynamic_package(
    source: Path,
    destination: Path,
    package_id: str,
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
    *,
    require_source_native_artifacts: bool,
) -> bool:
    if destination.exists():
        if not remove_native_dynamic_dir(
            f"NativeDynamic package {package_id} destination",
            destination,
            diagnostics,
        ):
            return False
    try:
        destination.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic package {package_id} destination {destination} could not be created: {error}"
        )
        return False

    saw_native_dir = False
    copied_native_artifacts = 0
    copied_loadable_artifacts = 0
    children = list_native_dynamic_dir(
        f"NativeDynamic package {package_id} source directory",
        source,
        diagnostics,
    )
    if children is None:
        return False
    for child in children:
        destination_child = destination / child.name
        if child.is_dir():
            if child.name == "native":
                saw_native_dir = True
                copy_result = copy_native_artifacts(
                    child,
                    destination_child,
                    package_id,
                    artifact_extensions,
                    loadable_artifact_extensions,
                    diagnostics,
                )
                copied_native_artifacts += copy_result["copied"]
                copied_loadable_artifacts += copy_result["loadable"]
                if copy_result["fatal"]:
                    return False
                if copy_result["copied"] == 0:
                    if require_source_native_artifacts:
                        diagnostics.append(
                            f"native dynamic package {package_id} has no dynamic library artifacts under {child}"
                        )
                elif copy_result["loadable"] == 0:
                    if require_source_native_artifacts:
                        diagnostics.append(
                            f"native dynamic package {package_id} has no loadable native library artifacts under {child}"
                        )
            elif child.name in NATIVE_DYNAMIC_RESOURCE_DIRS:
                if not copy_native_dynamic_tree(
                    child,
                    destination_child,
                    diagnostics,
                    f"NativeDynamic package {package_id} resource directory",
                ):
                    return False
        elif child.name == "plugin.toml":
            if not copy_native_dynamic_file(
                child,
                destination_child,
                diagnostics,
                f"NativeDynamic package {package_id} manifest",
            ):
                return False
    if not saw_native_dir:
        if require_source_native_artifacts:
            diagnostics.append(
                f"native dynamic package {package_id} has no native artifact directory under {source}"
            )
            return False
        return True
    if not require_source_native_artifacts:
        return True
    return copied_native_artifacts > 0 and copied_loadable_artifacts > 0


def copy_native_artifacts(
    source: Path,
    destination: Path,
    package_id: str,
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
) -> dict[str, int | bool]:
    copied = 0
    copied_loadable = 0
    try:
        destination.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic package {package_id} artifact directory {destination} could not be created: {error}"
        )
        return {"copied": copied, "loadable": copied_loadable, "fatal": True}
    children = list_native_dynamic_dir(
        f"NativeDynamic package {package_id} artifact directory",
        source,
        diagnostics,
    )
    if children is None:
        return {"copied": copied, "loadable": copied_loadable, "fatal": True}
    for child in children:
        extension = child.suffix.lower()
        if child.is_dir():
            if (
                extension not in artifact_extensions
                or extension not in NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS
            ):
                continue
            if not copy_native_dynamic_tree(
                child,
                destination / child.name,
                diagnostics,
                f"NativeDynamic package {package_id} artifact",
            ):
                return {"copied": copied, "loadable": copied_loadable, "fatal": True}
            copied += 1
            continue
        if not child.is_file():
            continue
        if extension not in artifact_extensions:
            continue
        if not copy_native_dynamic_file(
            child,
            destination / child.name,
            diagnostics,
            f"NativeDynamic package {package_id} artifact",
        ):
            return {"copied": copied, "loadable": copied_loadable, "fatal": True}
        copied += 1
        if extension in loadable_artifact_extensions:
            copied_loadable += 1
    return {"copied": copied, "loadable": copied_loadable, "fatal": False}


def copy_native_dynamic_file(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
) -> bool:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    except OSError as error:
        diagnostics.append(f"{label} {source} could not be copied to {destination}: {error}")
        return False
    return True


def copy_native_dynamic_tree(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
) -> bool:
    try:
        shutil.copytree(source, destination)
    except OSError as error:
        diagnostics.append(f"{label} {source} could not be copied to {destination}: {error}")
        return False
    return True


def resolve_stage_child(
    stage_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"native dynamic package directory {relative_path} must be relative")
        return None
    try:
        resolved_root = stage_root.resolve()
        resolved = (resolved_root / child_path).resolve()
    except OSError as error:
        diagnostics.append(
            f"native dynamic package directory {relative_path} could not be resolved: {error}"
        )
        return None
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        diagnostics.append(
            f"native dynamic package directory {relative_path} escapes the NativeDynamic stage"
        )
        return None
    return resolved


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()

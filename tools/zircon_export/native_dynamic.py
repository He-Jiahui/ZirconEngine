"""NativeDynamic package export report stage."""

from __future__ import annotations

import argparse
import hashlib
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
from .native_signing import (
    execute_native_dynamic_notarization,
    execute_native_dynamic_signing,
    native_dynamic_signing_command_template,
)


REPORT_FILE_NAME = "report.json"
NATIVE_DYNAMIC_STAGE = "native_dynamic"
NATIVE_DYNAMIC_PACKAGE_REPORT_FILE = "native_dynamic_package.toml"
NATIVE_DYNAMIC_LOADER_MANIFEST = "native_plugins.toml"
NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS = {".dll", ".so", ".dylib"}
NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS = {".pdb", ".dbg", ".dsym"}
NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS = (
    NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS | NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS
)
NATIVE_DYNAMIC_PLATFORM_ARTIFACT_EXTENSIONS = {
    "windows": {".dll", ".pdb"},
    "linux": {".so", ".dbg"},
    "macos": {".dylib", ".dsym"},
}
NATIVE_DYNAMIC_RESOURCE_DIRS = {"assets", "asset", "resources", "resource"}
NATIVE_DYNAMIC_ABI_STRING_FIELDS = (
    "descriptor_symbol",
    "descriptor_contract",
    "runtime_entry_source",
    "editor_entry_source",
    "host_function_table",
    "entry_report_contract",
    "behavior_contract",
    "state_snapshot_contract",
    "bridge_method_table",
)
NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS = {
    "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
    "descriptor_contract": "NativePluginAbiV3",
    "runtime_entry_source": "NativePluginAbiV3.runtime_entry_name",
    "editor_entry_source": "NativePluginAbiV3.editor_entry_name",
    "host_function_table": "NativePluginHostFunctionTableV3",
    "entry_report_contract": "NativePluginEntryReportV3",
    "behavior_contract": "NativePluginBehaviorV3",
    "state_snapshot_contract": "NativePluginBehaviorV3.save_state/restore_state",
    "bridge_method_table": "NativePluginBridgeMethodTableV3",
}


@dataclass(frozen=True)
class PackageManifestRead:
    manifest_id: str | None = None
    error: str | None = None


def run_native_dynamic(args: argparse.Namespace) -> int:
    repo_root = resolve_user_path(args.repo_root) if args.repo_root else default_repo_root()
    out_root = resolve_user_path(args.out)
    validate_report = (
        resolve_user_path(args.validate_report)
        if args.validate_report
        else out_root / "stages" / "validate" / REPORT_FILE_NAME
    )
    stage_dir = out_root / "stages" / NATIVE_DYNAMIC_STAGE
    plugin_root = (
        resolve_user_path(args.native_plugin_root)
        if getattr(args, "native_plugin_root", None)
        else repo_root / "zircon_plugins"
    )
    loader_manifest = stage_dir / "plugins" / NATIVE_DYNAMIC_LOADER_MANIFEST
    report_path = stage_dir / REPORT_FILE_NAME

    print(f"zircon_export stage=NativeDynamic profile={args.profile}")
    print(f"validate_report={validate_report}")
    print(f"native_plugin_root={plugin_root}")
    print(f"stage_output={stage_dir}")
    print(f"loader_manifest={loader_manifest}")
    print(f"report={report_path}")

    diagnostics: list[str] = []
    validate_payload = load_validate_report(validate_report, args.profile, diagnostics)
    native_dynamic_packages = native_dynamic_package_ids(validate_payload, diagnostics)
    package_exports = native_dynamic_package_exports(validate_payload, diagnostics)
    if package_exports is not None:
        validate_package_selection_matches_exports(
            native_dynamic_packages,
            package_exports,
            diagnostics,
        )
    target_platform = native_dynamic_target_platform(validate_payload)
    artifact_extensions = native_dynamic_artifact_extensions(target_platform)
    fatal = validate_payload is None or package_exports is None or bool(diagnostics)
    native_dynamic_build_enabled = bool(getattr(args, "native_dynamic_build", False))
    native_build_plan: dict[str, object] | None = None
    native_build_execution: dict[str, object] = {
        "enabled": native_dynamic_build_enabled,
        "fatal": False,
        "diagnostics": [],
        "package_count": 0,
        "packages": [],
    }
    native_signing_enabled = bool(getattr(args, "native_dynamic_sign_command", None))
    native_signing_profile = native_dynamic_signing_profile(
        getattr(args, "native_dynamic_sign_profile", None),
    )
    native_signing_platforms = native_dynamic_signing_platforms(
        getattr(args, "native_dynamic_sign_platform", []),
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
    native_notarization_enabled = bool(
        getattr(args, "native_dynamic_notarize_command", None)
    )
    native_notarization_profile = native_dynamic_signing_profile(
        getattr(args, "native_dynamic_notarize_profile", None),
    )
    native_notarization_platforms = native_dynamic_signing_platforms(
        getattr(args, "native_dynamic_notarize_platform", []),
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

    if args.dry_run:
        return 2 if fatal else 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    materialized_packages: list[dict[str, object]] = []
    file_manifest: list[dict[str, object]] = []
    content_hash: str | None = None
    payload_cleaned = False
    cleanup_reason: str | None = None
    if not fatal and package_exports is not None:
        reset_native_dynamic_plugins_dir(stage_dir)
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
                resolve_user_path(args.target_dir)
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
            build_features=getattr(args, "native_dynamic_build_feature", []),
            diagnostics=build_plan_diagnostics,
        )
        if native_dynamic_build_enabled:
            if diagnostics:
                native_build_execution["skipped"] = True
                native_build_execution["skip_reason"] = "materialization_diagnostics"
            elif native_build_plan.get("fatal"):
                diagnostics.extend(native_build_plan_diagnostics)
                native_build_execution = {
                    "enabled": True,
                    "fatal": True,
                    "diagnostics": list(native_build_plan_diagnostics),
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
                command=getattr(args, "native_dynamic_sign_command", None),
                extra_args=getattr(args, "native_dynamic_sign_arg", []),
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
                command=getattr(args, "native_dynamic_notarize_command", None),
                extra_args=getattr(args, "native_dynamic_notarize_arg", []),
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
            loader_manifest.parent.mkdir(parents=True, exist_ok=True)
            loader_manifest.write_text(
                native_plugin_load_manifest_template(package_exports),
                encoding="utf-8",
            )
            file_manifest = native_dynamic_file_manifest(stage_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
        else:
            reset_native_dynamic_plugins_dir(stage_dir)
            materialized_packages = []
            payload_cleaned = True
            cleanup_reason = "fatal_diagnostics"

    report = {
        "stage": "NativeDynamic",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report),
        "stage_output": str(stage_dir),
        "native_plugin_root": str(plugin_root),
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
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 2 if fatal else 0


def load_validate_report(
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
    if not isinstance(report, dict):
        diagnostics.append(f"validate report {validate_report} must be a JSON object")
        return None
    if report.get("fatal"):
        diagnostics.append("validate report is fatal; NativeDynamic will not export packages")
        return None
    if report.get("profile") != profile:
        diagnostics.append(
            f"validate report profile {report.get('profile')} does not match requested profile {profile}"
        )
        return None
    return report


def native_dynamic_package_ids(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> list[str]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        diagnostics.append("validate report does not contain plan_summary")
        return []
    packages = plan_summary.get("native_dynamic_packages", [])
    if packages is None:
        return []
    if not isinstance(packages, list) or any(not isinstance(value, str) for value in packages):
        diagnostics.append("validate report native_dynamic_packages must be a string array")
        return []
    package_id_indexes: dict[str, int] = {}
    for index, package_id in enumerate(packages):
        previous_index = package_id_indexes.get(package_id)
        if previous_index is not None:
            diagnostics.append(
                f"native_dynamic_packages entry {package_id} duplicates entry {previous_index}"
            )
        else:
            package_id_indexes[package_id] = index
    return list(packages)


def native_dynamic_package_exports(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> list[dict[str, Any]] | None:
    if validate_payload is None:
        return None
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    package_exports = plan_summary.get("native_dynamic_package_exports")
    if package_exports is None:
        diagnostics.append("validate report does not contain native_dynamic_package_exports")
        return None
    if not isinstance(package_exports, list):
        diagnostics.append("validate report native_dynamic_package_exports must be an array")
        return None

    normalized_exports: list[dict[str, Any]] = []
    package_id_indexes: dict[str, int] = {}
    for index, package_export in enumerate(package_exports):
        if not isinstance(package_export, dict):
            diagnostics.append(f"native_dynamic_package_exports entry {index} must be an object")
            return None
        validate_package_export_shape(index, package_export, diagnostics)
        package_id = package_export.get("package_id")
        if isinstance(package_id, str) and package_id:
            previous_index = package_id_indexes.get(package_id)
            if previous_index is not None:
                diagnostics.append(
                    f"native_dynamic_package_exports entry {index} package_id {package_id} duplicates entry {previous_index}"
                )
            else:
                package_id_indexes[package_id] = index
        normalized_exports.append(dict(package_export))
    if diagnostics:
        return None
    return normalized_exports


def validate_package_selection_matches_exports(
    package_ids: list[str],
    package_exports: list[dict[str, Any]],
    diagnostics: list[str],
) -> None:
    selected_ids = set(package_ids)
    exported_ids = {
        str(package_export["package_id"])
        for package_export in package_exports
    }
    for package_id in sorted(exported_ids - selected_ids):
        diagnostics.append(
            f"native_dynamic package_export {package_id} is not present in native_dynamic_packages"
        )
    for package_id in sorted(selected_ids - exported_ids):
        diagnostics.append(
            f"native_dynamic_packages entry {package_id} has no package_export"
        )


def native_dynamic_target_platform(validate_payload: dict[str, Any] | None) -> str | None:
    if validate_payload is None:
        return None
    profile_summary = validate_payload.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    target_platform = profile_summary.get("target_platform") or profile_summary.get("platform")
    if isinstance(target_platform, str) and target_platform:
        return target_platform
    return None


def native_dynamic_artifact_extensions(target_platform: str | None) -> set[str]:
    if not target_platform:
        return set(NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS)
    platform_key = target_platform.split("-", maxsplit=1)[0].lower()
    return set(
        NATIVE_DYNAMIC_PLATFORM_ARTIFACT_EXTENSIONS.get(
            platform_key,
            NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS,
        )
    )


def native_dynamic_signing_profile(value: object) -> str | None:
    if value is None:
        return None
    profile = str(value).strip()
    return profile or None


def native_dynamic_signing_platforms(value: object) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        value = [value]
    platforms: list[str] = []
    seen: set[str] = set()
    for item in value:
        platform = str(item).strip().lower()
        if not platform or platform in seen:
            continue
        platforms.append(platform)
        seen.add(platform)
    return platforms


def native_dynamic_loadable_artifact_extensions(artifact_extensions: set[str]) -> set[str]:
    return set(artifact_extensions) & NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS


def reset_native_dynamic_plugins_dir(stage_dir: Path) -> None:
    plugins_dir = stage_dir / "plugins"
    if plugins_dir.exists():
        shutil.rmtree(plugins_dir)
    plugins_dir.mkdir(parents=True, exist_ok=True)


def native_dynamic_package_directory(package_id: str) -> str:
    sanitized = "".join(
        character
        if character.isascii() and (character.isalnum() or character in "-_")
        else "_"
        for character in package_id
    )
    return sanitized if sanitized else "_"


def validate_package_export_shape(
    index: int,
    package_export: dict[str, Any],
    diagnostics: list[str],
) -> None:
    for field in ("package_id", "directory", "path", "manifest"):
        value = package_export.get(field)
        if not isinstance(value, str) or not value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field {field} must be a non-empty string"
            )
    package_id = package_export.get("package_id")
    directory = package_export.get("directory")
    path = package_export.get("path")
    manifest = package_export.get("manifest")
    package_report = package_export.get("package_report")
    if (
        isinstance(package_id, str)
        and package_id
        and isinstance(directory, str)
        and directory
    ):
        expected_directory = native_dynamic_package_directory(package_id)
        if directory != expected_directory:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} directory must be {expected_directory} for package_id {package_id}"
            )
    if isinstance(directory, str) and directory:
        expected_path = f"plugins/{directory}"
        expected_manifest = f"{expected_path}/plugin.toml"
        expected_package_report = f"{expected_path}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}"
        if isinstance(path, str) and path and path != expected_path:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} path must be {expected_path} for directory {directory}"
            )
        if isinstance(manifest, str) and manifest and manifest != expected_manifest:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} manifest must be {expected_manifest} for directory {directory}"
            )
        if package_report is None:
            package_export["package_report"] = expected_package_report
        elif not isinstance(package_report, str) or not package_report:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field package_report must be a non-empty string"
            )
        elif package_report != expected_package_report:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} package_report must be {expected_package_report} for directory {directory}"
            )
    abi = package_export.get("abi")
    if not isinstance(abi, dict):
        diagnostics.append(f"native_dynamic_package_exports entry {index} field abi must be an object")
        return
    abi_version = abi.get("abi_version")
    if type(abi_version) is not int:
        diagnostics.append(
            f"native_dynamic_package_exports entry {index} abi.abi_version must be an integer"
        )
    elif abi_version != 3:
        diagnostics.append(
            f"native_dynamic_package_exports entry {index} abi.abi_version must be 3"
        )
    for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
        value = abi.get(field)
        if not isinstance(value, str) or not value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be a non-empty string"
            )
            continue
        expected_value = NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]
        if value != expected_value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be {expected_value}"
            )


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
                shutil.rmtree(destination)
            continue
        package_report = destination / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            stage_dir,
            destination,
            loadable_artifact_extensions,
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
        payload_file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
        package_report.write_text(
            native_dynamic_package_report_template(package_export, payload_file_manifest),
            encoding="utf-8",
        )
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            stage_dir,
            package_dir,
            loadable_artifact_extensions,
        )
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
    stack = [plugin_root]
    while stack:
        current = stack.pop()
        for child in current.iterdir():
            if not child.is_dir():
                continue
            if package_manifest_matches(child / "plugin.toml", package_id):
                matches.append(child)
            stack.append(child)
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        diagnostics.append(
            f"native dynamic package {package_id} has multiple source package manifests: "
            + ", ".join(str(match) for match in sorted(matches))
        )
    return None


def package_manifest_matches(path: Path, package_id: str) -> bool:
    return package_manifest_id(path) == package_id


def package_manifest_id(path: Path) -> str | None:
    manifest_read = read_package_manifest_id(path)
    if manifest_read.error is not None:
        return None
    return manifest_read.manifest_id


def read_package_manifest_id(path: Path) -> PackageManifestRead:
    if not path.exists():
        return PackageManifestRead()
    try:
        with path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        return PackageManifestRead(error=f"could not be parsed: {error}")
    manifest_id = manifest.get("id")
    if isinstance(manifest_id, str) and manifest_id:
        return PackageManifestRead(manifest_id=manifest_id)
    return PackageManifestRead()


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
        shutil.rmtree(destination)
    destination.mkdir(parents=True, exist_ok=True)

    saw_native_dir = False
    copied_native_artifacts = 0
    copied_loadable_artifacts = 0
    for child in source.iterdir():
        destination_child = destination / child.name
        if child.is_dir():
            if child.name == "native":
                saw_native_dir = True
                copy_result = copy_native_artifacts(
                    child,
                    destination_child,
                    artifact_extensions,
                    loadable_artifact_extensions,
                )
                copied_native_artifacts += copy_result["copied"]
                copied_loadable_artifacts += copy_result["loadable"]
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
                shutil.copytree(child, destination_child)
        elif child.name == "plugin.toml":
            shutil.copy2(child, destination_child)
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
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
) -> dict[str, int]:
    copied = 0
    copied_loadable = 0
    destination.mkdir(parents=True, exist_ok=True)
    for child in source.iterdir():
        extension = child.suffix.lower()
        if child.is_dir():
            if (
                extension not in artifact_extensions
                or extension not in NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS
            ):
                continue
            shutil.copytree(child, destination / child.name)
            copied += 1
            continue
        if not child.is_file():
            continue
        if extension not in artifact_extensions:
            continue
        shutil.copy2(child, destination / child.name)
        copied += 1
        if extension in loadable_artifact_extensions:
            copied_loadable += 1
    return {"copied": copied, "loadable": copied_loadable}


def native_dynamic_file_manifest(stage_dir: Path) -> list[dict[str, object]]:
    manifest_root = stage_dir.resolve()
    plugins_dir = manifest_root / "plugins"
    return native_dynamic_plugins_file_manifest(manifest_root, plugins_dir)


def native_dynamic_plugins_bundle_file_manifest(
    plugins_dir: Path,
) -> list[dict[str, object]]:
    return native_dynamic_plugins_file_manifest(
        plugins_dir.resolve(),
        plugins_dir,
        root_prefix="plugins",
    )


def native_dynamic_plugins_file_manifest(
    manifest_root: Path,
    plugins_dir: Path,
    *,
    root_prefix: str | None = None,
) -> list[dict[str, object]]:
    manifest_root = manifest_root.resolve()
    plugins_dir = plugins_dir.resolve()
    if not plugins_dir.exists():
        return []

    file_manifest: list[dict[str, object]] = []
    for file_path in sorted(plugins_dir.rglob("*")):
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(manifest_root).as_posix()
        if root_prefix is not None:
            relative_path = f"{root_prefix}/{file_path.relative_to(plugins_dir).as_posix()}"
        payload = file_path.read_bytes()
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def native_dynamic_package_payload_file_manifest(package_dir: Path) -> list[dict[str, object]]:
    package_dir = package_dir.resolve()
    file_manifest: list[dict[str, object]] = []
    for file_path in sorted(package_dir.rglob("*")):
        if not file_path.is_file() or file_path.name == NATIVE_DYNAMIC_PACKAGE_REPORT_FILE:
            continue
        relative_path = file_path.relative_to(package_dir).as_posix()
        payload = file_path.read_bytes()
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def native_dynamic_package_loadable_artifacts(
    stage_dir: Path,
    package_dir: Path,
    loadable_artifact_extensions: set[str],
) -> list[str]:
    stage_dir = stage_dir.resolve()
    package_dir = package_dir.resolve()
    artifacts: list[str] = []
    for file_path in sorted(package_dir.rglob("*")):
        if not file_path.is_file():
            continue
        if file_path.suffix.lower() not in loadable_artifact_extensions:
            continue
        artifacts.append(file_path.relative_to(stage_dir).as_posix())
    return artifacts


def native_dynamic_content_hash(file_manifest: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in file_manifest:
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["bytes"]).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def native_dynamic_stage_payload_summary(
    out_root: Path,
    profile: str,
    plugins_dir: Path | None,
    diagnostics: list[str] | None = None,
) -> dict[str, Any] | None:
    if plugins_dir is None:
        return None

    report_path = out_root / "stages" / NATIVE_DYNAMIC_STAGE / REPORT_FILE_NAME
    if not report_path.exists():
        return native_dynamic_directory_payload_summary(plugins_dir)
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {report_path} is not valid JSON: {error}"
            )
        return None
    if not isinstance(report, dict):
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic report {report_path} must be a JSON object")
        return None
    if report.get("fatal") or report.get("profile") != profile:
        return native_dynamic_directory_payload_summary(plugins_dir)

    reported_plugins_dir = report.get("plugins_dir")
    if not isinstance(reported_plugins_dir, str):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report plugins_dir is missing or invalid")
        return None
    try:
        reported_plugins_path = Path(reported_plugins_dir).expanduser().resolve()
        current_plugins_path = plugins_dir.resolve()
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic report plugins_dir could not be resolved: {error}")
        return None
    if reported_plugins_path != current_plugins_path:
        return native_dynamic_directory_payload_summary(plugins_dir)

    content_hash = report.get("content_hash")
    file_manifest = normalized_file_manifest(report.get("file_manifest"))
    materialized_packages = normalized_materialized_packages(
        report.get("materialized_packages")
    )
    if not isinstance(content_hash, str):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report content_hash is missing or invalid")
        return None
    if file_manifest is None:
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report file_manifest is malformed")
        return None
    if materialized_packages is None:
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report materialized_packages are malformed"
            )
        return None

    actual_file_manifest = native_dynamic_plugins_file_manifest(
        plugins_dir.parent,
        plugins_dir,
    )
    actual_content_hash = native_dynamic_content_hash(actual_file_manifest)
    if actual_content_hash != content_hash:
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report content_hash "
                f"{content_hash} does not match current plugins directory "
                f"{plugins_dir} content_hash {actual_content_hash}"
        )
        return None

    if not materialized_package_loadable_artifacts_match_manifest(
        materialized_packages,
        file_manifest,
        plugins_dir,
    ):
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report loadable_artifacts are not present in file_manifest"
            )
        return None

    payload_summary = {
        "stage_report": str(report_path),
        "source": str(plugins_dir),
        "content_hash": content_hash,
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": len(materialized_packages),
        "materialized_packages": materialized_packages,
    }
    signing_summary = normalized_native_dynamic_operation_audit(
        report.get("native_signing")
    )
    if signing_summary is not None:
        payload_summary["native_signing"] = signing_summary
    notarization_summary = normalized_native_dynamic_operation_audit(
        report.get("native_notarization")
    )
    if notarization_summary is not None:
        payload_summary["native_notarization"] = notarization_summary
    return payload_summary


def native_dynamic_directory_payload_summary(plugins_dir: Path) -> dict[str, Any] | None:
    try:
        plugins_dir = plugins_dir.resolve()
    except OSError:
        return None
    if not plugins_dir.exists() or not plugins_dir.is_dir():
        return None
    file_manifest = native_dynamic_plugins_bundle_file_manifest(plugins_dir)
    materialized_packages = native_dynamic_directory_materialized_packages(plugins_dir)
    return {
        "stage_report": None,
        "source": str(plugins_dir),
        "content_hash": native_dynamic_content_hash(file_manifest),
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": len(materialized_packages),
        "materialized_packages": materialized_packages,
    }


def native_dynamic_directory_materialized_packages(
    plugins_dir: Path,
) -> list[dict[str, object]]:
    materialized_packages: list[dict[str, object]] = []
    for package_dir in sorted(plugins_dir.iterdir(), key=lambda path: path.name):
        if not package_dir.is_dir():
            continue
        package_report = package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        package_summary: dict[str, object] = {
            "package_id": native_dynamic_package_report_id(package_report) or package_dir.name,
            "destination": str(package_dir),
            "loadable_artifact_count": 0,
            "loadable_artifacts": [],
        }
        if package_report.exists():
            package_summary["package_report"] = str(package_report)
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            plugins_dir,
            package_dir,
            NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
        )
        loadable_artifacts = [f"plugins/{path}" for path in loadable_artifacts]
        package_summary["loadable_artifact_count"] = len(loadable_artifacts)
        package_summary["loadable_artifacts"] = loadable_artifacts
        materialized_packages.append(package_summary)
    return materialized_packages


def native_dynamic_package_report_id(package_report: Path) -> str | None:
    if not package_report.exists():
        return None
    try:
        with package_report.open("rb") as report_file:
            report = tomllib.load(report_file)
    except tomllib.TOMLDecodeError:
        return None
    package_id = report.get("package_id")
    if isinstance(package_id, str) and package_id:
        return package_id
    return None


def normalized_native_dynamic_operation_audit(
    value: object,
) -> dict[str, object] | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        return None
    enabled = value.get("enabled")
    profile = value.get("profile")
    target_platform = value.get("target_platform")
    allowed_platforms = value.get("allowed_platforms")
    platform_allowed = value.get("platform_allowed")
    fatal = value.get("fatal")
    package_count = value.get("package_count")
    if (
        type(enabled) is not bool
        or (profile is not None and not isinstance(profile, str))
        or (target_platform is not None and not isinstance(target_platform, str))
        or not isinstance(allowed_platforms, list)
        or any(not isinstance(platform, str) for platform in allowed_platforms)
        or type(platform_allowed) is not bool
        or type(fatal) is not bool
        or type(package_count) is not int
    ):
        return None
    return {
        "enabled": enabled,
        "profile": profile,
        "target_platform": target_platform,
        "allowed_platforms": list(allowed_platforms),
        "platform_allowed": platform_allowed,
        "fatal": fatal,
        "package_count": package_count,
    }


def normalized_file_manifest(value: object) -> list[dict[str, object]] | None:
    if not isinstance(value, list):
        return None
    normalized: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, dict):
            return None
        path = entry.get("path")
        byte_count = entry.get("bytes")
        sha256 = entry.get("sha256")
        if not isinstance(path, str) or type(byte_count) is not int or not isinstance(sha256, str):
            return None
        normalized.append(
            {
                "path": path,
                "bytes": byte_count,
                "sha256": sha256,
            }
        )
    return normalized


def normalized_materialized_packages(value: object) -> list[dict[str, object]] | None:
    if not isinstance(value, list):
        return None
    normalized: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, dict):
            return None
        package_id = entry.get("package_id")
        destination = entry.get("destination")
        loadable_artifact_count = entry.get("loadable_artifact_count")
        loadable_artifacts = entry.get("loadable_artifacts")
        if (
            not isinstance(package_id, str)
            or not isinstance(destination, str)
            or type(loadable_artifact_count) is not int
            or not isinstance(loadable_artifacts, list)
        ):
            return None
        if any(not isinstance(path, str) for path in loadable_artifacts):
            return None
        if loadable_artifact_count != len(loadable_artifacts):
            return None
        package_summary: dict[str, object] = {
            "package_id": package_id,
            "destination": destination,
            "loadable_artifact_count": loadable_artifact_count,
            "loadable_artifacts": list(loadable_artifacts),
        }
        source = entry.get("source")
        if source is not None:
            if not isinstance(source, str):
                return None
            package_summary["source"] = source
        package_report = entry.get("package_report")
        if package_report is not None:
            if not isinstance(package_report, str):
                return None
            package_summary["package_report"] = package_report
        normalized.append(package_summary)
    return normalized


def materialized_package_loadable_artifacts_match_manifest(
    materialized_packages: list[dict[str, object]],
    file_manifest: list[dict[str, object]],
    plugins_dir: Path,
) -> bool:
    manifest_paths = {str(entry["path"]) for entry in file_manifest}
    for package in materialized_packages:
        destination = str(package["destination"])
        try:
            destination_path = Path(destination).expanduser().resolve()
            relative_destination = destination_path.relative_to(plugins_dir.resolve())
        except (OSError, ValueError):
            return False
        package_prefix = f"plugins/{relative_destination.as_posix().rstrip('/')}/"
        loadable_artifacts = package["loadable_artifacts"]
        if not isinstance(loadable_artifacts, list):
            return False
        for artifact_path in loadable_artifacts:
            if not isinstance(artifact_path, str):
                return False
            if artifact_path not in manifest_paths:
                return False
            if not artifact_path.startswith(package_prefix):
                return False
    return True


def native_plugin_load_manifest_template(package_exports: list[dict[str, Any]]) -> str:
    output = "# Generated by Zircon export. Native dynamic plugins are loaded from these packages.\n"
    for package_export in package_exports:
        output += "\n[[plugins]]\n"
        output += f"id = {toml_string(package_export['package_id'])}\n"
        output += f"path = {toml_string(package_export['path'])}\n"
        output += f"manifest = {toml_string(package_export['manifest'])}\n"
        output += f"package_report = {toml_string(package_export['package_report'])}\n"
        output += native_dynamic_abi_contract_toml("plugins.abi", package_export["abi"])
    return output


def native_dynamic_package_report_template(
    package_export: dict[str, Any],
    payload_file_manifest: list[dict[str, object]],
) -> str:
    output = "# Generated by Zircon export. Native dynamic package report.\n"
    output += "format_version = 1\n"
    output += f"package_id = {toml_string(package_export['package_id'])}\n"
    output += f"directory = {toml_string(package_export['directory'])}\n"
    output += f"path = {toml_string(package_export['path'])}\n"
    output += f"manifest = {toml_string(package_export['manifest'])}\n"
    output += native_dynamic_abi_contract_toml("abi", package_export["abi"])
    output += native_dynamic_payload_toml("payload", payload_file_manifest)
    return output


def native_dynamic_payload_toml(
    table_name: str,
    file_manifest: list[dict[str, object]],
) -> str:
    output = f"\n[{table_name}]\n"
    output += f"file_count = {len(file_manifest)}\n"
    output += f"content_hash = {toml_string(native_dynamic_content_hash(file_manifest))}\n"
    for entry in file_manifest:
        output += f"\n[[{table_name}.files]]\n"
        output += f"path = {toml_string(entry['path'])}\n"
        output += f"bytes = {entry['bytes']}\n"
        output += f"sha256 = {toml_string(entry['sha256'])}\n"
    return output


def native_dynamic_abi_contract_toml(table_name: str, abi: dict[str, Any]) -> str:
    output = f"\n[{table_name}]\n"
    output += f"abi_version = {abi['abi_version']}\n"
    for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
        output += f"{field} = {toml_string(abi[field])}\n"
    return output


def toml_string(value: object) -> str:
    return json.dumps(str(value))


def resolve_stage_child(
    stage_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"native dynamic package directory {relative_path} must be relative")
        return None
    resolved_root = stage_root.resolve()
    resolved = (resolved_root / child_path).resolve()
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

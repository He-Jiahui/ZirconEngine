"""NativeDynamic cdylib build-plan discovery for export reports."""

from __future__ import annotations

import os
import shutil
import subprocess
import tomllib
from pathlib import Path
from typing import Any


NATIVE_BUILD_DEFAULT_MODE = "debug"
NATIVE_BUILD_CDYLIB_CRATE_TYPE = "cdylib"


def native_dynamic_build_plan(
    *,
    repo_root: Path,
    stage_dir: Path,
    target_dir: Path | None = None,
    package_exports: list[dict[str, Any]],
    source_packages: dict[str, Path],
    validate_payload: dict[str, Any] | None,
    target_platform: str | None,
    cargo: str,
    locked: bool,
    offline: bool,
    build_features: list[str],
    diagnostics: list[str],
) -> dict[str, object]:
    """Build a non-executing Cargo plan for selected NativeDynamic cdylibs.

    The NativeDynamic stage can still consume prebuilt package artifacts, but
    the report must also expose the exact cdylib Cargo commands needed for the
    same package selection so later execution/signing stages do not infer them.
    """

    plugins_workspace = repo_root / "zircon_plugins" / "Cargo.toml"
    crate_index = native_dynamic_cdylib_crate_index(plugins_workspace, diagnostics)
    cargo_profile = native_dynamic_cargo_profile(validate_payload)
    features = normalized_native_dynamic_build_features(build_features)
    target_dir = resolve_native_build_path(
        "native dynamic build target directory",
        target_dir.expanduser() if target_dir else stage_dir / "target",
        diagnostics,
    )
    resolved_plugins_workspace = resolve_native_build_path(
        "native dynamic plugin workspace manifest",
        plugins_workspace,
        diagnostics,
    )
    packages: list[dict[str, object]] = []

    if target_dir is not None and resolved_plugins_workspace is not None:
        for package_export in package_exports:
            package_id = str(package_export["package_id"])
            source_package = source_packages.get(package_id)
            if source_package is None:
                continue
            crate_name = native_dynamic_source_cdylib_crate_name(
                source_package / "plugin.toml",
                crate_index,
                package_id,
                diagnostics,
            )
            if crate_name is None:
                continue
            crate = crate_index.get(crate_name)
            if crate is None:
                diagnostics.append(
                    f"native dynamic package {package_id} crate {crate_name} is not a cdylib workspace member"
                )
                continue
            command = native_dynamic_cargo_build_command(
                cargo=cargo,
                workspace_manifest=resolved_plugins_workspace,
                crate_name=crate_name,
                target_dir=target_dir,
                cargo_profile=cargo_profile,
                locked=locked,
                offline=offline,
                features=features,
            )
            packages.append(
                {
                    "package_id": package_id,
                    "crate_name": crate_name,
                    "manifest_path": str(crate["manifest_path"]),
                    "workspace_manifest": str(resolved_plugins_workspace),
                    "target_dir": str(target_dir),
                    "cargo_profile": cargo_profile,
                    "release": cargo_profile == "release",
                    "features": features,
                    "command": command,
                    "expected_loadable_artifact": str(
                        native_dynamic_expected_loadable_artifact(
                            target_dir,
                            cargo_profile,
                            crate_name,
                            target_platform,
                        )
                    ),
                }
            )

    return {
        "fatal": bool(diagnostics),
        "diagnostics": list(diagnostics),
        "workspace_manifest": str(resolved_plugins_workspace or plugins_workspace),
        "target_dir": str(target_dir or stage_dir / "target"),
        "cargo_profile": cargo_profile,
        "release": cargo_profile == "release",
        "build_features": features,
        "package_count": len(packages),
        "packages": packages,
    }


def resolve_native_build_path(
    label: str,
    path: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def execute_native_dynamic_build_plan(
    *,
    native_build_plan: dict[str, object],
    repo_root: Path,
    materialized_packages: list[dict[str, object]],
    diagnostics: list[str],
) -> dict[str, object]:
    packages = native_build_plan.get("packages")
    if not isinstance(packages, list):
        diagnostics.append("NativeDynamic native build plan packages are malformed")
        return native_dynamic_build_execution_report(diagnostics, [])

    materialized_package_dirs = native_dynamic_materialized_package_dirs(
        materialized_packages
    )
    package_results: list[dict[str, object]] = []
    for package_plan in packages:
        package_result = execute_native_dynamic_package_build(
            package_plan,
            repo_root,
            materialized_package_dirs,
            diagnostics,
        )
        package_results.append(package_result)

    return native_dynamic_build_execution_report(diagnostics, package_results)


def execute_native_dynamic_package_build(
    package_plan: object,
    repo_root: Path,
    materialized_package_dirs: dict[str, Path],
    diagnostics: list[str],
) -> dict[str, object]:
    if not isinstance(package_plan, dict):
        diagnostics.append("NativeDynamic native build plan package entry is malformed")
        return {
            "package_id": None,
            "crate_name": None,
            "command": [],
            "exit_code": None,
            "stdout": "",
            "stderr": "",
            "expected_loadable_artifact": None,
            "copied_loadable_artifact": None,
            "copied_sidecars": [],
        }

    package_id = package_plan.get("package_id")
    crate_name = package_plan.get("crate_name")
    command = package_plan.get("command")
    expected_loadable_artifact = package_plan.get("expected_loadable_artifact")
    command_parts = (
        list(command)
        if isinstance(command, list) and all(isinstance(part, str) for part in command)
        else []
    )
    result: dict[str, object] = {
        "package_id": package_id if isinstance(package_id, str) else None,
        "crate_name": crate_name if isinstance(crate_name, str) else None,
        "command": command_parts,
        "exit_code": None,
        "stdout": "",
        "stderr": "",
        "expected_loadable_artifact": (
            expected_loadable_artifact
            if isinstance(expected_loadable_artifact, str)
            else None
        ),
        "copied_loadable_artifact": None,
        "copied_sidecars": [],
    }

    if not isinstance(package_id, str) or not package_id:
        diagnostics.append("NativeDynamic native build plan package_id is missing")
        return result
    if not command_parts:
        diagnostics.append(
            f"NativeDynamic native build plan for package {package_id} has malformed command"
        )
        return result
    if not isinstance(expected_loadable_artifact, str) or not expected_loadable_artifact:
        diagnostics.append(
            f"NativeDynamic native build plan for package {package_id} has no expected loadable artifact"
        )
        return result

    try:
        completed = subprocess.run(
            command_parts,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic native build for package {package_id} could not start: {error}"
        )
        return result

    result["exit_code"] = completed.returncode
    result["stdout"] = completed.stdout
    result["stderr"] = completed.stderr
    if completed.returncode != 0:
        diagnostics.append(
            f"NativeDynamic native build for package {package_id} exited with code {completed.returncode}"
        )
        return result

    source_artifact = Path(expected_loadable_artifact).expanduser()
    if not source_artifact.is_absolute():
        source_artifact = repo_root / source_artifact
    resolved_source_artifact = resolve_native_build_path(
        f"NativeDynamic native build for package {package_id} expected artifact",
        source_artifact,
        diagnostics,
    )
    if resolved_source_artifact is None:
        return result
    source_artifact = resolved_source_artifact
    if not source_artifact.exists() or not source_artifact.is_file():
        diagnostics.append(
            f"NativeDynamic native build for package {package_id} expected artifact {source_artifact} does not exist"
        )
        return result

    package_dir = materialized_package_dirs.get(package_id)
    if package_dir is None:
        diagnostics.append(
            f"NativeDynamic native build for package {package_id} has no materialized package destination"
        )
        return result

    native_dir = package_dir / "native"
    try:
        native_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic native build for package {package_id} native artifact directory {native_dir} could not be created: {error}"
        )
        return result

    destination_artifact = native_dir / source_artifact.name
    try:
        shutil.copy2(source_artifact, destination_artifact)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic native build for package {package_id} artifact {source_artifact} could not be copied to {destination_artifact}: {error}"
        )
        return result
    result["copied_loadable_artifact"] = str(destination_artifact)
    result["copied_sidecars"] = copy_native_dynamic_build_sidecars(
        source_artifact,
        native_dir,
        package_id,
        diagnostics,
    )
    return result


def native_dynamic_materialized_package_dirs(
    materialized_packages: list[dict[str, object]],
) -> dict[str, Path]:
    package_dirs: dict[str, Path] = {}
    for package in materialized_packages:
        package_id = package.get("package_id")
        destination = package.get("destination")
        if not isinstance(package_id, str) or not isinstance(destination, str):
            continue
        package_dirs[package_id] = Path(destination)
    return package_dirs


def copy_native_dynamic_build_sidecars(
    source_artifact: Path,
    native_dir: Path,
    package_id: str,
    diagnostics: list[str],
) -> list[str]:
    copied: list[str] = []
    candidates = [
        source_artifact.with_suffix(".pdb"),
        source_artifact.with_suffix(".dbg"),
        Path(str(source_artifact) + ".dSYM"),
    ]
    seen: set[Path] = set()
    for sidecar in candidates:
        if sidecar in seen or not sidecar.exists():
            continue
        seen.add(sidecar)
        destination = native_dir / sidecar.name
        if sidecar.is_dir():
            if destination.exists():
                try:
                    shutil.rmtree(destination)
                except OSError as error:
                    diagnostics.append(
                        f"NativeDynamic native build for package {package_id} sidecar destination {destination} could not be removed: {error}"
                    )
                    continue
            try:
                shutil.copytree(sidecar, destination)
            except OSError as error:
                diagnostics.append(
                    f"NativeDynamic native build for package {package_id} sidecar {sidecar} could not be copied to {destination}: {error}"
                )
                continue
            copied.append(str(destination))
        elif sidecar.is_file():
            try:
                shutil.copy2(sidecar, destination)
            except OSError as error:
                diagnostics.append(
                    f"NativeDynamic native build for package {package_id} sidecar {sidecar} could not be copied to {destination}: {error}"
                )
                continue
            copied.append(str(destination))
    return copied


def native_dynamic_build_execution_report(
    diagnostics: list[str],
    packages: list[dict[str, object]],
) -> dict[str, object]:
    return {
        "enabled": True,
        "fatal": bool(diagnostics),
        "diagnostics": list(diagnostics),
        "package_count": len(packages),
        "packages": packages,
    }


def native_dynamic_cdylib_crate_index(
    workspace_manifest: Path,
    diagnostics: list[str],
) -> dict[str, dict[str, object]]:
    if not workspace_manifest.exists():
        diagnostics.append(
            f"native dynamic plugin workspace manifest {workspace_manifest} does not exist"
        )
        return {}
    workspace = read_toml(workspace_manifest, diagnostics)
    if workspace is None:
        return {}
    members = workspace.get("workspace", {}).get("members", [])
    if not isinstance(members, list):
        diagnostics.append("native dynamic plugin workspace members must be an array")
        return {}

    crates: dict[str, dict[str, object]] = {}
    plugins_root = workspace_manifest.parent
    for member in members:
        if not isinstance(member, str) or not member:
            diagnostics.append("native dynamic plugin workspace member must be a non-empty string")
            continue
        member_manifest = resolve_native_build_path(
            f"native dynamic workspace member {member} manifest",
            plugins_root / Path(member) / "Cargo.toml",
            diagnostics,
        )
        if member_manifest is None:
            continue
        crate_manifest = read_toml(member_manifest, diagnostics)
        if crate_manifest is None:
            continue
        package = crate_manifest.get("package", {})
        crate_name = package.get("name") if isinstance(package, dict) else None
        if not isinstance(crate_name, str) or not crate_name:
            diagnostics.append(f"native dynamic crate manifest {member_manifest} package.name is missing")
            continue
        crate_types = crate_manifest.get("lib", {}).get("crate-type", [])
        if not isinstance(crate_types, list):
            continue
        if NATIVE_BUILD_CDYLIB_CRATE_TYPE not in crate_types:
            continue
        crates[crate_name] = {
            "member": member,
            "manifest_path": member_manifest,
        }
    return crates


def native_dynamic_source_cdylib_crate_name(
    plugin_manifest_path: Path,
    crate_index: dict[str, dict[str, object]],
    package_id: str,
    diagnostics: list[str],
) -> str | None:
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return None
    modules = manifest.get("modules", [])
    if modules is None:
        modules = []
    if not isinstance(modules, list):
        diagnostics.append(
            f"native dynamic package {package_id} plugin.toml modules must be an array"
        )
        return None
    crate_names = [
        crate_name
        for module in modules
        if isinstance(module, dict)
        if isinstance(crate_name := module.get("crate_name"), str)
        if crate_name in crate_index
    ]
    crate_names = dedupe(crate_names)
    if len(crate_names) == 1:
        return crate_names[0]
    if len(crate_names) > 1:
        diagnostics.append(
            f"native dynamic package {package_id} declares multiple cdylib crates: "
            + ", ".join(crate_names)
        )
        return None
    diagnostics.append(
        f"native dynamic package {package_id} declares no cdylib crate in plugin.toml modules"
    )
    return None


def native_dynamic_cargo_profile(validate_payload: dict[str, Any] | None) -> str:
    if isinstance(validate_payload, dict):
        profile_summary = validate_payload.get("profile_summary")
        if isinstance(profile_summary, dict):
            build_mode = profile_summary.get("build_mode")
            if isinstance(build_mode, str) and build_mode.lower() == "release":
                return "release"
    return NATIVE_BUILD_DEFAULT_MODE


def native_dynamic_cargo_build_command(
    *,
    cargo: str,
    workspace_manifest: Path,
    crate_name: str,
    target_dir: Path,
    cargo_profile: str,
    locked: bool,
    offline: bool,
    features: list[str],
) -> list[str]:
    command = [
        cargo,
        "build",
        "--manifest-path",
        str(workspace_manifest),
        "-p",
        crate_name,
        "--target-dir",
        str(target_dir),
    ]
    if locked:
        command.append("--locked")
    if features:
        command.extend(["--features", ",".join(features)])
    if cargo_profile == "release":
        command.append("--release")
    if offline:
        command.append("--offline")
    return command


def native_dynamic_expected_loadable_artifact(
    target_dir: Path,
    cargo_profile: str,
    crate_name: str,
    target_platform: str | None,
) -> Path:
    return target_dir / cargo_profile / platform_dynamic_library_name(
        crate_name,
        target_platform,
    )


def platform_dynamic_library_name(crate_name: str, target_platform: str | None) -> str:
    if target_platform:
        platform = target_platform.split("-", maxsplit=1)[0].lower()
        if platform == "windows":
            return f"{crate_name}.dll"
        if platform == "macos":
            return f"lib{crate_name}.dylib"
        if platform == "linux":
            return f"lib{crate_name}.so"
    if os.name == "nt":
        return f"{crate_name}.dll"
    if hasattr(os, "uname") and os.uname().sysname.lower() == "darwin":
        return f"lib{crate_name}.dylib"
    return f"lib{crate_name}.so"


def normalized_native_dynamic_build_features(features: list[str]) -> list[str]:
    result: list[str] = []
    for feature in features:
        if not isinstance(feature, str):
            continue
        normalized = feature.strip()
        if normalized and normalized not in result:
            result.append(normalized)
    return result


def read_toml(path: Path, diagnostics: list[str]) -> dict[str, Any] | None:
    if not path.exists():
        diagnostics.append(f"TOML file {path} does not exist")
        return None
    if not path.is_file():
        diagnostics.append(f"TOML file {path} is not a file")
        return None
    try:
        with path.open("rb") as toml_file:
            payload = tomllib.load(toml_file)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"TOML file {path} could not be parsed: {error}")
        return None
    except OSError as error:
        diagnostics.append(f"TOML file {path} could not be read: {error}")
        return None
    if not isinstance(payload, dict):
        diagnostics.append(f"TOML file {path} must contain a table")
        return None
    return payload


def dedupe(values: list[str]) -> list[str]:
    result: list[str] = []
    for value in values:
        if value not in result:
            result.append(value)
    return result

"""NativeDynamic Cargo execution and artifact copy helpers."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from .native_build_workspace import resolve_native_build_path


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
        "skipped": False,
        "diagnostics": list(diagnostics),
        "package_count": len(packages),
        "packages": packages,
    }

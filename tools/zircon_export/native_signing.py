"""NativeDynamic loadable-artifact signing execution."""

from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path
from typing import Any


def execute_native_dynamic_signing(
    *,
    materialized_packages: list[dict[str, object]],
    loadable_artifact_extensions: set[str],
    command_template: list[str],
    target_platform: str | None,
    signing_profile: str | None,
    allowed_platforms: list[str],
    diagnostics: list[str],
) -> dict[str, object]:
    return execute_native_dynamic_artifact_command_batch(
        operation="signing",
        materialized_packages=materialized_packages,
        loadable_artifact_extensions=loadable_artifact_extensions,
        command_template=command_template,
        target_platform=target_platform,
        profile=signing_profile,
        signing_profile=signing_profile,
        notarization_profile=None,
        allowed_platforms=allowed_platforms,
        diagnostics=diagnostics,
    )


def execute_native_dynamic_notarization(
    *,
    materialized_packages: list[dict[str, object]],
    loadable_artifact_extensions: set[str],
    command_template: list[str],
    target_platform: str | None,
    signing_profile: str | None,
    notarization_profile: str | None,
    allowed_platforms: list[str],
    diagnostics: list[str],
) -> dict[str, object]:
    return execute_native_dynamic_artifact_command_batch(
        operation="notarization",
        materialized_packages=materialized_packages,
        loadable_artifact_extensions=loadable_artifact_extensions,
        command_template=command_template,
        target_platform=target_platform,
        profile=notarization_profile,
        signing_profile=signing_profile,
        notarization_profile=notarization_profile,
        allowed_platforms=allowed_platforms,
        diagnostics=diagnostics,
    )


def native_dynamic_signing_command_template(
    *,
    command: object,
    extra_args: object,
) -> list[str]:
    if not command:
        return []
    if extra_args is None:
        extra_args = []
    if not isinstance(extra_args, list):
        extra_args = [str(extra_args)]
    return [str(command), *(str(value) for value in extra_args)]


def execute_native_dynamic_artifact_command_batch(
    *,
    operation: str,
    materialized_packages: list[dict[str, object]],
    loadable_artifact_extensions: set[str],
    command_template: list[str],
    target_platform: str | None,
    profile: str | None,
    signing_profile: str | None,
    notarization_profile: str | None,
    allowed_platforms: list[str],
    diagnostics: list[str],
) -> dict[str, object]:
    platform_allowed = native_dynamic_signing_platform_allowed(
        target_platform,
        allowed_platforms,
    )
    report: dict[str, object] = {
        "enabled": True,
        "profile": profile,
        "target_platform": target_platform,
        "allowed_platforms": allowed_platforms,
        "platform_allowed": platform_allowed,
        "fatal": False,
        "diagnostics": [],
        "package_count": 0,
        "packages": [],
    }
    if not platform_allowed:
        diagnostics.append(
            f"NativeDynamic {operation} profile "
            f"{profile or '<unnamed>'} does not allow target platform "
            f"{target_platform or '<unknown>'}"
        )
        report["fatal"] = True
        report["diagnostics"] = list(diagnostics)
        return report

    packages: list[dict[str, object]] = []
    for materialized_package in materialized_packages:
        package_result = execute_native_dynamic_package_artifact_command(
            operation=operation,
            materialized_package=materialized_package,
            loadable_artifact_extensions=loadable_artifact_extensions,
            command_template=command_template,
            target_platform=target_platform,
            signing_profile=signing_profile,
            notarization_profile=notarization_profile,
            diagnostics=diagnostics,
        )
        packages.append(package_result)

    report["fatal"] = bool(diagnostics)
    report["diagnostics"] = list(diagnostics)
    report["package_count"] = len(packages)
    report["packages"] = packages
    return report


def execute_native_dynamic_package_artifact_command(
    *,
    operation: str,
    materialized_package: dict[str, object],
    loadable_artifact_extensions: set[str],
    command_template: list[str],
    target_platform: str | None,
    signing_profile: str | None,
    notarization_profile: str | None,
    diagnostics: list[str],
) -> dict[str, object]:
    package_id = materialized_package.get("package_id")
    destination = materialized_package.get("destination")
    package_result: dict[str, object] = {
        "package_id": package_id if isinstance(package_id, str) else None,
        "artifact_count": 0,
        "artifacts": [],
    }
    if not isinstance(package_id, str) or not package_id:
        diagnostics.append(f"NativeDynamic {operation} package_id is missing")
        return package_result
    if not isinstance(destination, str) or not destination:
        diagnostics.append(
            f"NativeDynamic {operation} package {package_id} destination is missing"
        )
        return package_result

    package_dir = Path(destination)
    artifact_results: list[dict[str, object]] = []
    signable_artifacts = native_dynamic_signable_artifacts(
        package_dir,
        loadable_artifact_extensions,
        diagnostics,
        operation=operation,
        package_id=package_id,
    )
    if signable_artifacts is None:
        return package_result
    for artifact in signable_artifacts:
        artifact_results.append(
            execute_native_dynamic_artifact_command(
                operation=operation,
                artifact=artifact,
                package_id=package_id,
                package_dir=package_dir,
                command_template=command_template,
                target_platform=target_platform,
                signing_profile=signing_profile,
                notarization_profile=notarization_profile,
                diagnostics=diagnostics,
            )
        )

    package_result["artifact_count"] = len(artifact_results)
    package_result["artifacts"] = artifact_results
    return package_result


def execute_native_dynamic_artifact_command(
    *,
    operation: str,
    artifact: Path,
    package_id: str,
    package_dir: Path,
    command_template: list[str],
    target_platform: str | None,
    signing_profile: str | None,
    notarization_profile: str | None,
    diagnostics: list[str],
) -> dict[str, object]:
    command = native_dynamic_signing_command(
        command_template=command_template,
        artifact=artifact,
        package_id=package_id,
        package_dir=package_dir,
        target_platform=target_platform,
        signing_profile=signing_profile,
        notarization_profile=notarization_profile,
    )
    artifact_result: dict[str, object] = {
        "artifact": str(artifact),
        "package_relative_artifact": artifact.relative_to(package_dir).as_posix(),
        "command": command,
        "exit_code": None,
        "stdout": "",
        "stderr": "",
        "before_sha256": None,
        "after_sha256": None,
    }
    before_sha256 = file_sha256_or_diagnostic(
        artifact,
        diagnostics,
        (
            f"NativeDynamic {operation} for package {package_id} artifact "
            f"{artifact} could not be read before command"
        ),
    )
    if before_sha256 is None:
        return artifact_result
    artifact_result["before_sha256"] = before_sha256
    artifact_result["after_sha256"] = before_sha256

    try:
        completed = subprocess.run(
            command,
            cwd=package_dir,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic {operation} for package {package_id} artifact {artifact} could not start: {error}"
        )
        return artifact_result

    artifact_result["exit_code"] = completed.returncode
    artifact_result["stdout"] = completed.stdout
    artifact_result["stderr"] = completed.stderr
    if completed.returncode != 0:
        diagnostics.append(
            f"NativeDynamic {operation} for package {package_id} artifact {artifact} exited with code {completed.returncode}"
        )
        return artifact_result

    if not artifact.exists() or not artifact.is_file():
        diagnostics.append(
            f"NativeDynamic {operation} for package {package_id} artifact {artifact} removed the loadable file"
        )
        return artifact_result

    after_sha256 = file_sha256_or_diagnostic(
        artifact,
        diagnostics,
        (
            f"NativeDynamic {operation} for package {package_id} artifact "
            f"{artifact} could not be read after command"
        ),
    )
    if after_sha256 is None:
        artifact_result["after_sha256"] = None
        return artifact_result
    artifact_result["after_sha256"] = after_sha256
    return artifact_result


def native_dynamic_signable_artifacts(
    package_dir: Path,
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
    *,
    operation: str,
    package_id: str,
) -> list[Path] | None:
    artifacts: list[Path] = []
    try:
        file_paths = sorted(package_dir.rglob("*"))
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic {operation} for package {package_id} package directory {package_dir} could not be listed: {error}"
        )
        return None
    for file_path in file_paths:
        if not file_path.is_file():
            continue
        if file_path.suffix.lower() not in loadable_artifact_extensions:
            continue
        artifacts.append(file_path)
    return artifacts


def native_dynamic_signing_command(
    *,
    command_template: list[str],
    artifact: Path,
    package_id: str,
    package_dir: Path,
    target_platform: str | None,
    signing_profile: str | None,
    notarization_profile: str | None = None,
) -> list[str]:
    replacements: dict[str, str] = {
        "artifact": str(artifact),
        "package_id": package_id,
        "package_dir": str(package_dir),
        "target_platform": target_platform or "",
        "signing_profile": signing_profile or "",
        "notarization_profile": notarization_profile or "",
    }
    return [
        replace_native_dynamic_signing_tokens(part, replacements)
        for part in command_template
    ]


def replace_native_dynamic_signing_tokens(
    value: str,
    replacements: dict[str, str],
) -> str:
    result = value
    for key, replacement in replacements.items():
        result = result.replace("{" + key + "}", replacement)
    return result


def native_dynamic_signing_platform_allowed(
    target_platform: str | None,
    allowed_platforms: list[str],
) -> bool:
    if not allowed_platforms:
        return True
    if not target_platform:
        return False
    normalized_target = target_platform.lower()
    for platform in allowed_platforms:
        normalized_platform = platform.lower()
        if normalized_target == normalized_platform:
            return True
        if normalized_target.startswith(normalized_platform + "-"):
            return True
    return False


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def file_sha256_or_diagnostic(
    path: Path,
    diagnostics: list[str],
    label: str,
) -> str | None:
    try:
        return file_sha256(path)
    except OSError as error:
        diagnostics.append(f"{label}: {error}")
        return None

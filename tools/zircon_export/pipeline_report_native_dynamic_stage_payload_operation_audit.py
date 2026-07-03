"""NativeDynamic stage payload operation-audit artifact diagnostics."""

from __future__ import annotations

from pathlib import Path


def materialized_package_relative_artifacts(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, list[str]]:
    package_artifacts: dict[str, list[str]] = {}
    for package in materialized_packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(
                plugins_dir.resolve(),
            )
        except (OSError, ValueError):
            continue
        package_prefix = f"plugins/{relative_destination.as_posix().rstrip('/')}/"
        artifacts: list[str] = []
        for artifact in package["loadable_artifacts"]:
            artifact_path = str(artifact)
            if artifact_path.startswith(package_prefix):
                artifacts.append(artifact_path.removeprefix(package_prefix))
        package_artifacts[package_id] = artifacts
    return package_artifacts


def native_dynamic_audit_artifacts_are_schema_clean(
    artifacts: list[object],
) -> bool:
    return all(
        isinstance(artifact, dict)
        and isinstance(artifact.get("artifact"), str)
        and artifact["artifact"].strip()
        and artifact["artifact"].strip() == artifact["artifact"]
        and isinstance(artifact.get("package_relative_artifact"), str)
        and artifact["package_relative_artifact"].strip()
        and (
            artifact["package_relative_artifact"].strip()
            == artifact["package_relative_artifact"]
        )
        for artifact in artifacts
    )


def native_dynamic_operation_audit_artifact_diagnostics(
    table: object,
    field: str,
    materialized_package_artifacts: dict[str, list[str]],
) -> list[str]:
    if not isinstance(table, dict):
        return []
    packages = table.get("packages")
    if not isinstance(packages, list):
        return []

    diagnostics: list[str] = []
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        artifacts = package.get("artifacts")
        artifact_count = package.get("artifact_count")
        if not isinstance(package_id, str) or not isinstance(artifacts, list):
            continue
        if type(artifact_count) is int and artifact_count < 0:
            continue
        if not native_dynamic_audit_artifacts_are_schema_clean(artifacts):
            continue
        if not all(
            isinstance(artifact, dict)
            and isinstance(artifact.get("package_relative_artifact"), str)
            for artifact in artifacts
        ):
            continue
        for artifact_index, artifact in enumerate(artifacts):
            artifact_path = artifact.get("artifact")
            package_relative_artifact = artifact.get("package_relative_artifact")
            if not (
                isinstance(artifact_path, str)
                and isinstance(package_relative_artifact, str)
                and package_id.strip()
                and package_id.strip() == package_id
                and package_relative_artifact.strip()
                and package_relative_artifact.strip() == package_relative_artifact
            ):
                continue
            expected_artifact = f"plugins/{package_id}/{package_relative_artifact}"
            if artifact_path != expected_artifact:
                diagnostics.append(
                    f"native_dynamic report {field} package {package_id} "
                    f"artifacts[{artifact_index}].artifact {artifact_path} "
                    "does not match package_relative_artifact "
                    f"{expected_artifact}"
                )
        package_relative_artifacts = [
            str(artifact["package_relative_artifact"]) for artifact in artifacts
        ]
        if type(artifact_count) is int:
            if artifact_count < 0:
                continue
            if artifact_count != len(artifacts):
                diagnostics.append(
                    f"native_dynamic report {field} package {package_id} "
                    f"artifact_count {artifact_count} does not match artifacts "
                    f"{len(artifacts)}"
                )
        expected_artifacts = materialized_package_artifacts.get(package_id)
        if expected_artifacts is None:
            continue
        if package_relative_artifacts != expected_artifacts:
            diagnostics.append(
                f"native_dynamic report {field} package {package_id} "
                f"package_relative_artifacts {package_relative_artifacts} do "
                f"not match materialized loadable artifacts {expected_artifacts}"
            )
    return diagnostics

"""NativeDynamic build execution payload diagnostics."""

from __future__ import annotations

from pathlib import Path


def native_dynamic_build_execution_plan_diagnostics(
    build_plan: object,
    build_execution: object,
) -> list[str]:
    if not isinstance(build_plan, dict) or not isinstance(build_execution, dict):
        return []
    plan_packages = native_dynamic_build_package_rows_by_id(
        build_plan.get("packages")
    )
    execution_packages = build_execution.get("packages")
    if not plan_packages or not isinstance(execution_packages, list):
        return []

    diagnostics: list[str] = []
    for execution_package in execution_packages:
        if not isinstance(execution_package, dict):
            continue
        package_id = execution_package.get("package_id")
        if not native_dynamic_trimmed_non_empty_string_is_schema_clean(
            package_id
        ):
            continue
        plan_package = plan_packages.get(package_id)
        if plan_package is None:
            continue
        diagnostics.extend(
            native_dynamic_build_execution_plan_field_diagnostics(
                package_id,
                "crate_name",
                plan_package,
                execution_package,
            )
        )
        diagnostics.extend(
            native_dynamic_build_execution_plan_field_diagnostics(
                package_id,
                "command",
                plan_package,
                execution_package,
            )
        )
        diagnostics.extend(
            native_dynamic_build_execution_plan_field_diagnostics(
                package_id,
                "expected_loadable_artifact",
                plan_package,
                execution_package,
            )
        )
    return diagnostics


def native_dynamic_build_package_rows_by_id(
    packages: object,
) -> dict[str, dict[str, object]]:
    if not isinstance(packages, list):
        return {}
    rows: dict[str, dict[str, object]] = {}
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        if isinstance(package_id, str) and package_id.strip():
            rows[package_id] = package
    return rows


def native_dynamic_build_execution_plan_field_diagnostics(
    package_id: str,
    field: str,
    plan_package: dict[str, object],
    execution_package: dict[str, object],
) -> list[str]:
    plan_value = plan_package.get(field)
    execution_value = execution_package.get(field)
    if field == "command":
        if not (
            native_dynamic_command_array_is_schema_clean(plan_value)
            and native_dynamic_command_array_is_schema_clean(execution_value)
        ):
            return []
    elif not (
        native_dynamic_trimmed_non_empty_string_is_schema_clean(plan_value)
        and native_dynamic_trimmed_non_empty_string_is_schema_clean(
            execution_value
        )
    ):
        return []
    if execution_value == plan_value:
        return []
    return [
        "native_dynamic report native_build_execution package "
        f"{package_id} {field} {execution_value} does not match "
        f"native_build_plan package {field} {plan_value}"
    ]


def native_dynamic_build_execution_artifact_diagnostics(
    table: object,
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
    current_file_manifest: list[dict[str, object]],
) -> list[str]:
    if not isinstance(table, dict):
        return []
    packages = table.get("packages")
    if not isinstance(packages, list):
        return []

    expected_artifacts = materialized_package_loadable_artifact_paths(
        materialized_packages
    )
    expected_native_dirs = materialized_package_native_dir_paths(
        materialized_packages,
        plugins_dir,
    )
    current_file_paths = native_dynamic_file_manifest_paths(current_file_manifest)
    diagnostics: list[str] = []
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        copied_artifact = package.get("copied_loadable_artifact")
        if (
            not native_dynamic_trimmed_non_empty_string_is_schema_clean(
                package_id
            )
            or not native_dynamic_trimmed_non_empty_string_is_schema_clean(
                copied_artifact
            )
        ):
            continue
        package_expected_artifacts = expected_artifacts.get(package_id)
        if package_expected_artifacts is None:
            continue
        copied_artifact_path = native_dynamic_copied_artifact_bundle_path(
            copied_artifact,
            plugins_dir,
        )
        if copied_artifact_path is None:
            continue
        if copied_artifact_path not in package_expected_artifacts:
            diagnostics.append(
                "native_dynamic report native_build_execution package "
                f"{package_id} copied_loadable_artifact {copied_artifact_path} "
                "does not match materialized loadable artifacts "
                f"{package_expected_artifacts}"
            )
        copied_sidecars = package.get("copied_sidecars")
        if not isinstance(copied_sidecars, list):
            continue
        package_native_dir = expected_native_dirs.get(package_id)
        for sidecar_index, copied_sidecar in enumerate(copied_sidecars):
            if not native_dynamic_trimmed_non_empty_string_is_schema_clean(
                copied_sidecar
            ):
                continue
            copied_sidecar_path = native_dynamic_copied_artifact_bundle_path(
                copied_sidecar,
                plugins_dir,
            )
            if copied_sidecar_path is None:
                continue
            if (
                package_native_dir is not None
                and not copied_sidecar_path.startswith(package_native_dir)
            ):
                diagnostics.append(
                    "native_dynamic report native_build_execution package "
                    f"{package_id} copied_sidecars[{sidecar_index}] "
                    f"{copied_sidecar_path} is not inside materialized "
                    f"native package directory {package_native_dir}"
                )
                continue
            if not native_dynamic_file_manifest_contains_path_or_directory(
                copied_sidecar_path,
                current_file_paths,
            ):
                diagnostics.append(
                    "native_dynamic report native_build_execution package "
                    f"{package_id} copied_sidecars[{sidecar_index}] "
                    f"{copied_sidecar_path} is not present in current "
                    "NativeDynamic plugins file_manifest"
                )
    return diagnostics


def materialized_package_loadable_artifact_paths(
    materialized_packages: list[dict[str, object]],
) -> dict[str, list[str]]:
    package_artifacts: dict[str, list[str]] = {}
    for package in materialized_packages:
        package_id = str(package["package_id"])
        loadable_artifacts = package.get("loadable_artifacts")
        if not isinstance(loadable_artifacts, list):
            continue
        package_artifacts[package_id] = [
            str(artifact)
            for artifact in loadable_artifacts
            if isinstance(artifact, str)
        ]
    return package_artifacts


def materialized_package_native_dir_paths(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, str]:
    package_native_dirs: dict[str, str] = {}
    for package in materialized_packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(
                plugins_dir.resolve(),
            )
        except (OSError, ValueError):
            continue
        package_native_dirs[
            package_id
        ] = f"plugins/{relative_destination.as_posix().rstrip('/')}/native/"
    return package_native_dirs


def native_dynamic_file_manifest_paths(
    file_manifest: list[dict[str, object]],
) -> set[str]:
    return {
        str(entry["path"])
        for entry in file_manifest
        if isinstance(entry.get("path"), str)
    }


def native_dynamic_file_manifest_contains_path_or_directory(
    bundle_path: str,
    file_manifest_paths: set[str],
) -> bool:
    normalized_path = bundle_path.rstrip("/")
    return normalized_path in file_manifest_paths or any(
        path.startswith(f"{normalized_path}/") for path in file_manifest_paths
    )


def native_dynamic_command_array_is_schema_clean(value: object) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(
            isinstance(entry, str) and entry.strip() and entry.strip() == entry
            for entry in value
        )
    )


def native_dynamic_copied_artifact_bundle_path(
    copied_artifact: str,
    plugins_dir: Path,
) -> str | None:
    copied_path = Path(copied_artifact).expanduser()
    if copied_path.is_absolute():
        try:
            relative_path = copied_path.resolve().relative_to(plugins_dir.resolve())
        except (OSError, ValueError):
            return copied_path.as_posix()
        return f"plugins/{relative_path.as_posix()}"
    return copied_path.as_posix()


def native_dynamic_trimmed_non_empty_string_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value

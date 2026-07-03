"""PlatformBundle NativeDynamic payload package path diagnostics."""

from __future__ import annotations

from pathlib import Path

from .pipeline_report_native_dynamic_payload_package_report import (
    platform_bundle_native_plugins_package_report_content_diagnostics,
)


def _resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()


def _resolve_user_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return _resolve_user_path(path)
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def platform_bundle_native_plugins_package_path_diagnostics(
    packages: list[dict[str, object]],
    plugins_dir: Path,
    *,
    stage_backed_payload: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    plugins_root = _resolve_user_path_or_diagnostic(
        plugins_dir,
        diagnostics,
        "PlatformBundle report native_plugins",
    )
    if plugins_root is None:
        return diagnostics
    for index, package in enumerate(packages):
        destination = str(package["destination"])
        destination_path = _resolve_user_path_or_diagnostic(
            destination,
            diagnostics,
            "PlatformBundle report native_plugins_payload "
            f"materialized_packages[{index}] destination",
        )
        if destination_path is None:
            continue
        try:
            destination_path.relative_to(plugins_root)
        except ValueError:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] destination {destination} "
                f"is outside native_plugins {plugins_root}"
            )
            continue
        package_report = package.get("package_report")
        if package_report is None:
            if stage_backed_payload:
                diagnostics.append(
                    "PlatformBundle report native_plugins_payload "
                    f"materialized_packages[{index}] package_report "
                    "is required for stage-backed payloads"
                )
            continue
        package_report_path = _resolve_user_path_or_diagnostic(
            str(package_report),
            diagnostics,
            "PlatformBundle report native_plugins_payload "
            f"materialized_packages[{index}] package_report",
        )
        if package_report_path is None:
            continue
        try:
            package_report_path.relative_to(destination_path)
        except ValueError:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] package_report {package_report} "
                f"is outside package destination {destination_path}"
            )
            continue
        if not package_report_path.exists():
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] package_report {package_report_path} "
                "does not exist"
            )
            continue
        if not package_report_path.is_file():
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] package_report {package_report_path} "
                "is not a file"
            )
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_package_report_content_diagnostics(
                index,
                package,
                plugins_root,
                destination_path,
                package_report_path,
            )
        )
    return diagnostics

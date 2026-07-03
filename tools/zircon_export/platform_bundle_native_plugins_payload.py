"""PlatformBundle native plugins payload rewriting helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_contract import NATIVE_DYNAMIC_LOADER_MANIFEST


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

"""NativeDynamic stage materialized package report diagnostics."""

from __future__ import annotations

import tomllib
from pathlib import Path

from .native_dynamic_contract import NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
from .pipeline_report_native_dynamic_payload_package_report import (
    platform_bundle_native_plugins_package_report_content_diagnostics,
)


def native_dynamic_package_report_diagnostics(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
    native_plugin_root: object,
) -> list[str]:
    diagnostics: list[str] = []
    try:
        plugins_root = plugins_dir.expanduser().resolve()
    except OSError as error:
        return [
            f"native_dynamic report plugins_dir {plugins_dir} could not be resolved: {error}"
        ]
    native_plugin_root_path: Path | None = None
    if native_dynamic_trimmed_non_empty_string_is_schema_clean(
        native_plugin_root
    ):
        try:
            native_plugin_root_path = Path(native_plugin_root).expanduser().resolve()
        except OSError as error:
            diagnostics.append(
                "native_dynamic report native_plugin_root "
                f"{native_plugin_root} could not be resolved: {error}"
            )

    for index, package in enumerate(materialized_packages):
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        source_label = (
            "native_dynamic report "
            f"materialized_packages[{index}] source"
        )
        label = (
            "native_dynamic report "
            f"materialized_packages[{index}] package_report"
        )
        if package.get("source") is None:
            diagnostics.append(
                f"{source_label} is required for NativeDynamic stage materialized packages"
            )
        source = package.get("source")
        if isinstance(source, str) and not source:
            diagnostics.append(
                f"{source_label} must be a non-empty string for "
                "NativeDynamic stage materialized packages"
            )
        if isinstance(source, str) and source:
            try:
                source_path = Path(source).expanduser().resolve()
            except OSError as error:
                diagnostics.append(f"{source_label} {source} could not be resolved: {error}")
            else:
                if native_plugin_root_path is not None:
                    try:
                        source_path.relative_to(native_plugin_root_path)
                    except ValueError:
                        diagnostics.append(
                            f"{source_label} {source_path} is outside "
                            f"native_plugin_root {native_plugin_root_path}"
                        )
                if not source_path.exists():
                    diagnostics.append(f"{source_label} {source_path} does not exist")
                elif not source_path.is_dir():
                    diagnostics.append(f"{source_label} {source_path} is not a directory")
                else:
                    source_manifest = source_path / "plugin.toml"
                    if not source_manifest.exists():
                        diagnostics.append(
                            f"{source_label} manifest {source_manifest} does not exist"
                        )
                    elif not source_manifest.is_file():
                        diagnostics.append(
                            f"{source_label} manifest {source_manifest} is not a file"
                        )
                    else:
                        manifest_id = native_dynamic_source_manifest_id(
                            source_manifest,
                            source_label,
                            diagnostics,
                        )
                        if manifest_id is not None and manifest_id != package_id:
                            diagnostics.append(
                                f"{source_label} manifest id {manifest_id} "
                                f"does not match materialized package {package_id}"
                            )
        try:
            package_dir = destination.resolve()
        except OSError as error:
            diagnostics.append(
                "native_dynamic report "
                f"materialized_packages[{index}] destination {destination} "
                f"could not be resolved: {error}"
            )
            continue
        try:
            package_dir.relative_to(plugins_root)
        except ValueError:
            diagnostics.append(
                "native_dynamic report "
                f"materialized_packages[{index}] destination {destination} "
                f"is outside plugins_dir {plugins_root}"
            )
            continue

        declared_package_report = package.get("package_report")
        expected_package_report = package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        if declared_package_report is None:
            diagnostics.append(
                f"{label} is required for NativeDynamic stage materialized packages"
            )
            continue
        if isinstance(declared_package_report, str) and not declared_package_report:
            diagnostics.append(
                f"{label} must be a non-empty string for "
                "NativeDynamic stage materialized packages"
            )
            continue
        declared_path = Path(str(declared_package_report)).expanduser()
        try:
            declared_path = declared_path.resolve()
        except OSError as error:
            diagnostics.append(
                f"{label} {declared_package_report} could not be resolved: {error}"
            )
            continue
        if declared_path != expected_package_report:
            diagnostics.append(
                f"{label} {declared_path} does not match expected "
                f"{expected_package_report} for package {package_id}"
            )
            continue
        package_report_path = expected_package_report
        if not package_report_path.exists():
            diagnostics.append(f"{label} {package_report_path} does not exist")
            continue
        if not package_report_path.is_file():
            diagnostics.append(f"{label} {package_report_path} is not a file")
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_package_report_content_diagnostics(
                index,
                package,
                plugins_root,
                package_dir,
                package_report_path,
                label=label,
            )
        )
    return diagnostics


def native_dynamic_source_manifest_id(
    source_manifest: Path,
    source_label: str,
    diagnostics: list[str],
) -> str | None:
    try:
        with source_manifest.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"{source_label} manifest {source_manifest} could not be parsed: {error}"
        )
        return None
    except OSError as error:
        diagnostics.append(
            f"{source_label} manifest {source_manifest} could not be read: {error}"
        )
        return None
    if "id" not in manifest:
        diagnostics.append(f"{source_label} manifest id must be a non-empty string")
        return None
    manifest_id = manifest.get("id")
    if not isinstance(manifest_id, str):
        diagnostics.append(f"{source_label} manifest id must be a string")
        return None
    if not manifest_id:
        diagnostics.append(f"{source_label} manifest id must be a non-empty string")
        return None
    if manifest_id.strip() != manifest_id:
        diagnostics.append(f"{source_label} manifest id must be a non-empty trimmed string")
        return None
    return manifest_id


def native_dynamic_trimmed_non_empty_string_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value

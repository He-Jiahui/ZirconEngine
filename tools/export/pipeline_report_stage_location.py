"""Stage-location diagnostics for Zircon export final reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any


STAGE_OUTPUT_LOCATION_STAGES = {
    "native_dynamic": ("native_dynamic report", "NativeDynamic"),
    "pack": ("pack report", "Pack"),
    "validate": ("validate report", "Validate"),
}


def _is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def stage_output_location_diagnostics(
    stage_key: str,
    report: dict[str, Any],
    report_path: Path,
) -> list[str]:
    location_labels = STAGE_OUTPUT_LOCATION_STAGES.get(stage_key)
    if location_labels is None or report.get("fatal") is not False:
        return []
    report_label, stage_label = location_labels
    stage_output = report.get("stage_output")
    if not _is_non_empty_trimmed_string(stage_output):
        return []

    try:
        resolved_stage_output = Path(stage_output).expanduser().resolve()
    except OSError as error:
        return [
            f"{report_label} stage_output {stage_output} could not be resolved: "
            f"{error}"
        ]
    try:
        expected_stage_directory = report_path.expanduser().parent.resolve()
    except OSError as error:
        return [
            f"{report_label} stage report directory {report_path.parent} could "
            f"not be "
            f"resolved: {error}"
        ]

    if resolved_stage_output != expected_stage_directory:
        return [
            f"{report_label} stage_output {resolved_stage_output} does not "
            f"match current {stage_label} stage directory "
            f"{expected_stage_directory}"
        ]
    return []


def native_dynamic_plugins_dir_location_diagnostics(
    stage_key: str,
    report: dict[str, Any],
    report_path: Path,
) -> list[str]:
    if stage_key != "native_dynamic" or report.get("fatal") is not False:
        return []

    plugins_dir = report.get("plugins_dir")
    if not _is_non_empty_trimmed_string(plugins_dir):
        return []

    try:
        resolved_plugins_dir = Path(plugins_dir).expanduser().resolve()
    except OSError as error:
        return [
            f"native_dynamic report plugins_dir {plugins_dir} could not be "
            f"resolved: {error}"
        ]
    try:
        expected_plugins_dir = report_path.expanduser().parent.resolve() / "plugins"
    except OSError as error:
        return [
            f"native_dynamic report stage report directory {report_path.parent} "
            f"could not be resolved: {error}"
        ]

    if resolved_plugins_dir != expected_plugins_dir:
        return [
            f"native_dynamic report plugins_dir {resolved_plugins_dir} does not "
            f"match current NativeDynamic plugins directory {expected_plugins_dir}"
        ]
    return []


def native_dynamic_loader_manifest_location_diagnostics(
    stage_key: str,
    report: dict[str, Any],
    report_path: Path,
) -> list[str]:
    if stage_key != "native_dynamic" or report.get("fatal") is not False:
        return []

    loader_manifest = report.get("loader_manifest")
    if not _is_non_empty_trimmed_string(loader_manifest):
        return []

    try:
        resolved_loader_manifest = Path(loader_manifest).expanduser().resolve()
    except OSError as error:
        return [
            f"native_dynamic report loader_manifest {loader_manifest} could "
            f"not be resolved: {error}"
        ]
    try:
        expected_loader_manifest = (
            report_path.expanduser().parent.resolve()
            / "plugins"
            / "native_plugins.toml"
        )
    except OSError as error:
        return [
            f"native_dynamic report stage report directory {report_path.parent} "
            f"could not be resolved: {error}"
        ]

    if resolved_loader_manifest != expected_loader_manifest:
        return [
            "native_dynamic report loader_manifest "
            f"{resolved_loader_manifest} does not match current NativeDynamic "
            f"loader manifest {expected_loader_manifest}"
        ]
    return []

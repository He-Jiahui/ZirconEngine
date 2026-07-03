"""PlatformBundle template release-evidence diagnostics."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from .export_template_manifest import is_safe_relative_path, normalize_relative_path
from .pipeline_report_platform_bundle_template_copied_files_schema import (
    PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS,
    PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
)
from .pipeline_report_platform_bundle_template_schema import (
    platform_bundle_template_report_schema_diagnostics,
    table_required_non_empty_string_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)
from .pipeline_report_platform_bundle_template_resolution_schema import (
    platform_bundle_template_resolution_diagnostics,
)


def platform_bundle_template_files_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    template_files = report.get("template_files")
    if template_files is None:
        return []
    if not isinstance(template_files, list):
        return ["PlatformBundle report template_files must be a list"]
    if not template_files:
        return []
    template = report.get("template")
    if not isinstance(template, dict):
        return [
            "PlatformBundle report template_files are present but template report is missing"
        ]

    diagnostics: list[str] = []
    diagnostics.extend(platform_bundle_template_report_schema_diagnostics(template))
    host_executable = report.get("host_executable")
    host_path = (
        resolve_user_path_or_diagnostic(
            host_executable,
            diagnostics,
            "PlatformBundle report host_executable",
        )
        if isinstance(host_executable, str) and host_executable
        else None
    )
    expected_hashes = platform_bundle_template_file_hashes(template, diagnostics)
    expected_destinations = platform_bundle_template_file_expected_destinations(
        report,
        template,
        diagnostics,
    )

    for index, entry in enumerate(template_files):
        if not isinstance(entry, dict):
            diagnostics.append(
                f"PlatformBundle report template_files entry {index} must be an object"
            )
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"PlatformBundle report template_files[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS,
            )
        )
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"PlatformBundle report template_files[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
        destination = entry.get("destination")
        if not isinstance(destination, str) or not destination:
            diagnostics.append(
                f"PlatformBundle report template_files entry {index} destination must be a non-empty string"
            )
            continue
        destination_path = resolve_user_path_or_diagnostic(
            destination,
            diagnostics,
            "PlatformBundle report template_files destination",
        )
        if destination_path is None:
            continue
        if host_path is not None and destination_path == host_path:
            continue
        if not destination_path.exists():
            diagnostics.append(
                f"PlatformBundle report template_files destination {destination_path} does not exist"
            )
            continue
        if not destination_path.is_file():
            diagnostics.append(
                f"PlatformBundle report template_files destination {destination_path} is not a file"
            )
            continue
        expected_sha256 = platform_bundle_template_file_expected_hash(
            entry,
            expected_hashes,
            diagnostics,
        )
        if expected_sha256 is None:
            diagnostics.append(
                f"PlatformBundle report template_files entry {index} cannot be matched to template file sha256"
            )
            continue
        destination_diagnostic = platform_bundle_template_file_destination_diagnostic(
            index,
            entry,
            destination_path,
            expected_destinations,
        )
        if destination_diagnostic:
            diagnostics.append(destination_diagnostic)
            continue
        actual_sha256 = platform_bundle_file_sha256(
            destination_path,
            diagnostics,
            f"PlatformBundle report template_files destination {destination_path}",
        )
        if actual_sha256 is None:
            continue
        if actual_sha256 != expected_sha256:
            diagnostics.append(
                "PlatformBundle report template_files destination "
                f"{destination_path} sha256 {actual_sha256} does not match "
                f"template sha256 {expected_sha256}"
            )
    return diagnostics


def platform_bundle_template_file_expected_destinations(
    report: dict[str, Any],
    template: dict[str, Any],
    diagnostics: list[str],
) -> dict[str, tuple[Path, int]]:
    bundle_root = platform_bundle_template_bundle_root(report, template, diagnostics)
    if bundle_root is None:
        return {}
    template_dir = template.get("template_dir")
    files = template.get("files")
    if not isinstance(template_dir, str) or not template_dir:
        return {}
    if not isinstance(files, list):
        return {}
    template_root = resolve_user_path_or_diagnostic(
        template_dir,
        diagnostics,
        "PlatformBundle report template.template_dir",
    )
    if template_root is None:
        return {}
    expected: dict[str, tuple[Path, int]] = {}
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            continue
        relative_path = entry.get("path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            continue
        source_path = resolve_user_path_or_diagnostic(
            template_root / relative_path,
            diagnostics,
            f"PlatformBundle report template.files entry {index} path",
        )
        if source_path is None:
            continue
        bundle_path = entry.get("bundle_path", relative_path)
        if not isinstance(bundle_path, str) or not bundle_path.strip():
            continue
        normalized_bundle_path = normalize_relative_path(bundle_path)
        if not is_safe_relative_path(normalized_bundle_path):
            continue
        expected_destination = resolve_user_path_or_diagnostic(
            bundle_root / normalized_bundle_path,
            diagnostics,
            f"PlatformBundle report template.files[{index}].bundle_path",
        )
        if expected_destination is None:
            continue
        expected[str(source_path)] = (expected_destination, index)
    return expected


def platform_bundle_template_bundle_root(
    report: dict[str, Any],
    template: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    bundle = report.get("bundle")
    if not isinstance(bundle, str) or not bundle:
        return None
    bundle_root = resolve_user_path_or_diagnostic(
        bundle,
        diagnostics,
        "PlatformBundle report bundle",
    )
    if bundle_root is None:
        return None
    template_bundle = template.get("bundle")
    if not isinstance(template_bundle, dict):
        return bundle_root
    root = template_bundle.get("root")
    if not isinstance(root, str) or not root or root == ".":
        return bundle_root
    normalized_root = normalize_relative_path(root)
    if not is_safe_relative_path(normalized_root):
        return bundle_root
    return resolve_user_path_or_diagnostic(
        bundle_root / normalized_root,
        diagnostics,
        "PlatformBundle report template.bundle.root",
    )


def platform_bundle_template_file_destination_diagnostic(
    index: int,
    entry: dict[str, Any],
    destination_path: Path,
    expected_destinations: dict[str, tuple[Path, int]],
) -> str | None:
    source = entry.get("source")
    if not isinstance(source, str) or not source:
        return None
    try:
        source_path = resolve_user_path(source)
    except OSError:
        return None
    expected = expected_destinations.get(str(source_path))
    if expected is None:
        return None
    expected_destination, template_index = expected
    if destination_path == expected_destination:
        return None
    return (
        f"PlatformBundle report template_files[{index}].destination "
        f"does not match template.files[{template_index}].bundle_path "
        f"{expected_destination}"
    )


def platform_bundle_template_file_hashes(
    template: dict[str, Any],
    diagnostics: list[str],
) -> dict[str, str]:
    template_dir = template.get("template_dir")
    files = template.get("files")
    if not isinstance(template_dir, str) or not template_dir:
        diagnostics.append(
            "PlatformBundle report template.template_dir must be a non-empty string when template_files are present"
        )
        return {}
    if not isinstance(files, list):
        diagnostics.append(
            "PlatformBundle report template.files must be a list when template_files are present"
        )
        return {}

    hashes: dict[str, str] = {}
    root = resolve_user_path_or_diagnostic(
        template_dir,
        diagnostics,
        "PlatformBundle report template.template_dir",
    )
    if root is None:
        return hashes
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            diagnostics.append(
                f"PlatformBundle report template.files entry {index} must be an object"
            )
            continue
        relative_path = entry.get("path")
        sha256 = entry.get("sha256")
        if not isinstance(relative_path, str) or not relative_path:
            diagnostics.append(
                f"PlatformBundle report template.files entry {index} path must be a non-empty string"
            )
            continue
        if not isinstance(sha256, str) or not sha256:
            diagnostics.append(
                f"PlatformBundle report template.files entry {index} sha256 must be a non-empty string"
            )
            continue
        file_path = resolve_user_path_or_diagnostic(
            root / relative_path,
            diagnostics,
            f"PlatformBundle report template.files entry {index} path",
        )
        if file_path is None:
            continue
        hashes[str(file_path)] = sha256.lower()
    return hashes


def platform_bundle_template_file_expected_hash(
    entry: dict[str, Any],
    expected_hashes: dict[str, str],
    diagnostics: list[str],
) -> str | None:
    source = entry.get("source")
    if not isinstance(source, str) or not source:
        return None
    source_path = resolve_user_path_or_diagnostic(
        source,
        diagnostics,
        "PlatformBundle report template_files source",
    )
    if source_path is None:
        return None
    return expected_hashes.get(str(source_path))


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()


def resolve_user_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return resolve_user_path(path)
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def platform_bundle_file_sha256(
    path: Path,
    diagnostics: list[str],
    label: str,
) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        diagnostics.append(f"{label} could not be read: {error}")
        return None

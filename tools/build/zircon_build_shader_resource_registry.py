"""Shader resource registry export contract helpers for build tooling."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from uuid import UUID

_RESOURCE_RECORD_KINDS = frozenset(
    (
        "Data",
        "Model",
        "Mesh",
        "Material",
        "MaterialGraph",
        "Texture",
        "Shader",
        "Scene",
        "Sound",
        "Font",
        "PhysicsMaterial",
        "NavMesh",
        "NavigationSettings",
        "Terrain",
        "TerrainLayerStack",
        "TileSet",
        "TileMap",
        "Prefab",
        "AnimationSkeleton",
        "AnimationClip",
        "AnimationSequence",
        "AnimationGraph",
        "AnimationStateMachine",
        "UiLayout",
        "UiWidget",
        "UiStyle",
    )
)
_RESOURCE_RECORD_STATES = frozenset(("Pending", "Ready", "Error", "Reloading"))
_RESOURCE_LOCATOR_SCHEMES = frozenset(("res", "lib", "package", "builtin", "mem"))
_RESOURCE_REGISTRY_BACKED_LOCATOR_SCHEMES = frozenset(("res", "lib", "package", "mem"))
_U32_MAX = 2**32 - 1
_U64_MAX = 2**64 - 1
_INCOMPLETE_RESOURCE_RECORD_ENTRY = "incomplete ResourceRecord entry at index"
_MISSING_RESOURCE_RECORD_LOCATORS = "missing ResourceRecord locators for report sources"
_MISSING_USABLE_SHADER_RECORD_REVISIONS = (
    "missing usable shader ResourceRecord revisions for report sources"
)
_RESOURCE_RECORD_REQUIRED_FIELDS = (
    "id",
    "kind",
    "primary_locator",
    "artifact_locator",
    "revision",
    "state",
    "dependency_ids",
    "diagnostics",
    "source_hash",
    "importer_id",
    "importer_version",
    "config_hash",
)


def validate_shader_resource_registry_export_contract(
    registry_path: Path,
    *,
    report_path: Path | None = None,
    require_usable_shader_records: bool = False,
    require_report_registry_backed_sources: bool = False,
) -> None:
    registry_path = Path(registry_path)
    try:
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(
            "shader prewarm resource registry export unavailable "
            f"({registry_path}: {error})"
        ) from error
    except json.JSONDecodeError as error:
        raise RuntimeError(
            "shader prewarm resource registry export is not valid JSON "
            f"({registry_path}: {error})"
        ) from error

    records = _resource_registry_record_array(registry)
    if records is None:
        raise RuntimeError(
            "shader prewarm resource registry export did not produce "
            "a ResourceRecord array"
        )
    has_usable_shader_record, locators, usable_locators = (
        _validate_and_index_resource_records(
            records,
            detect_usable_shader_records=(
                require_usable_shader_records or report_path is not None
            ),
            index_report_locators=report_path is not None,
        )
    )
    if require_usable_shader_records:
        _validate_registry_export_has_usable_shader_records(
            has_usable_shader_record,
            registry_path,
        )
    if report_path is None and require_report_registry_backed_sources:
        raise RuntimeError(
            "shader prewarm resource registry export requires report_path when "
            "requiring registry-backed report sources"
        )
    if report_path is not None:
        report = _read_report_for_registry_export_contract(report_path)
        _validate_registry_export_matches_report_sources(
            locators,
            usable_locators,
            report,
            require_report_registry_backed_sources=require_report_registry_backed_sources,
        )


def _resource_registry_record_array(registry: object) -> list[object] | None:
    if isinstance(registry, list):
        return registry
    if not isinstance(registry, Mapping):
        return None
    resources = registry.get("resources")
    if isinstance(resources, list):
        return resources
    records = registry.get("records")
    if isinstance(records, list):
        return records
    return None


def _validate_and_index_resource_records(
    records: list[object],
    *,
    detect_usable_shader_records: bool,
    index_report_locators: bool,
) -> tuple[bool, set[str], set[str]]:
    has_usable_shader_record = False
    locators: set[str] = set()
    usable_locators: set[str] = set()
    for index, record in enumerate(records):
        if not isinstance(record, Mapping):
            raise RuntimeError(
                "shader prewarm resource registry export contains "
                "non-object ResourceRecord entries"
            )
        primary_locator, artifact_locator, record_is_usable_shader = (
            _validate_resource_record_shape(record, index)
        )
        is_usable_shader_record = (
            detect_usable_shader_records and record_is_usable_shader
        )
        if is_usable_shader_record:
            has_usable_shader_record = True
        if not index_report_locators:
            continue

        if isinstance(primary_locator, str):
            locators.add(primary_locator)
        if isinstance(artifact_locator, str):
            locators.add(artifact_locator)
        if is_usable_shader_record:
            if isinstance(primary_locator, str):
                usable_locators.add(primary_locator)
            if isinstance(artifact_locator, str):
                usable_locators.add(artifact_locator)
    return has_usable_shader_record, locators, usable_locators


def _read_report_for_registry_export_contract(
    report_path: Path,
) -> Mapping[str, object]:
    report_path = Path(report_path)
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(
            "shader prewarm report unavailable for resource registry "
            f"correlation contract ({report_path}: {error})"
        ) from error
    except json.JSONDecodeError as error:
        raise RuntimeError(
            "shader prewarm report is not valid JSON for resource registry "
            f"correlation contract ({report_path}: {error})"
        ) from error
    if not isinstance(report, Mapping):
        raise RuntimeError(
            "shader prewarm report did not provide resource registry "
            "correlation data"
        )
    return report


def _validate_resource_record_shape(
    record: Mapping[str, object],
    index: int,
) -> tuple[object, object, bool]:
    missing = [
        field
        for field in _RESOURCE_RECORD_REQUIRED_FIELDS
        if field not in record
    ]
    invalid = []
    if not _is_resource_id_string(record.get("id")):
        invalid.append("id")
    kind = record.get("kind")
    if not _resource_record_kind_is_known(kind):
        invalid.append("kind")
    primary_locator = record.get("primary_locator")
    if not _is_resource_locator_string(primary_locator):
        invalid.append("primary_locator")
    artifact_locator = record.get("artifact_locator")
    if artifact_locator is not None and not _is_resource_locator_string(
        artifact_locator
    ):
        invalid.append("artifact_locator")
    revision = record.get("revision")
    if not _is_unsigned_int_within(revision, _U64_MAX):
        invalid.append("revision")
    state = record.get("state")
    if not _resource_record_state_is_known(state):
        invalid.append("state")
    if not isinstance(record.get("dependency_ids"), list):
        invalid.append("dependency_ids")
    elif not all(_is_resource_id_string(value) for value in record["dependency_ids"]):
        invalid.append("dependency_ids")
    if not isinstance(record.get("diagnostics"), list):
        invalid.append("diagnostics")
    elif not all(
        _is_resource_diagnostic_record(value) for value in record["diagnostics"]
    ):
        invalid.append("diagnostics")
    if not isinstance(record.get("source_hash"), str):
        invalid.append("source_hash")
    if not isinstance(record.get("importer_id"), str):
        invalid.append("importer_id")
    importer_version = record.get("importer_version")
    if not _is_unsigned_int_within(importer_version, _U32_MAX):
        invalid.append("importer_version")
    if not isinstance(record.get("config_hash"), str):
        invalid.append("config_hash")

    problems = _unique_in_order([*missing, *invalid])
    if problems:
        raise RuntimeError(
            "shader prewarm resource registry export contains "
            f"{_INCOMPLETE_RESOURCE_RECORD_ENTRY} {index}: "
            f"{', '.join(problems)}"
        )
    is_usable_shader_record = (
        _resource_record_kind_is_shader(kind)
        and state == "Ready"
        and isinstance(revision, int)
        and revision > 0
    )
    return primary_locator, artifact_locator, is_usable_shader_record


def _validate_registry_export_matches_report_sources(
    locators: set[str],
    usable_locators: set[str],
    report: Mapping[str, object],
    *,
    require_report_registry_backed_sources: bool = False,
) -> None:
    source_labels = _report_resource_source_labels(report)
    if not source_labels:
        if require_report_registry_backed_sources:
            raise RuntimeError(
                "shader prewarm resource registry export requires at least one "
                "registry-backed report source for project/plugin asset roots"
            )
        return
    missing = [
        source_label
        for source_label in source_labels
        if source_label not in locators
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm resource registry export is "
            f"{_MISSING_RESOURCE_RECORD_LOCATORS}: "
            + ", ".join(missing)
        )
    unusable = [
        source_label
        for source_label in source_labels
        if source_label not in usable_locators
    ]
    if unusable:
        raise RuntimeError(
            "shader prewarm resource registry export is "
            f"{_MISSING_USABLE_SHADER_RECORD_REVISIONS}: "
            + ", ".join(unusable)
        )


def _validate_registry_export_has_usable_shader_records(
    has_usable_shader_record: bool,
    registry_path: Path,
) -> None:
    if not has_usable_shader_record:
        raise RuntimeError(
            "shader prewarm resource registry export requires at least one "
            "usable Shader ResourceRecord for project/plugin asset roots: "
            f"{registry_path}"
        )


def _report_resource_source_labels(report: Mapping[str, object]) -> tuple[str, ...]:
    seen: set[str] = set()
    labels: list[str] = []

    provenance = report.get("source_provenance")
    if isinstance(provenance, Mapping):
        sources = provenance.get("sources")
        if isinstance(sources, Mapping):
            for source in sources.values():
                if not isinstance(source, Mapping):
                    continue
                _append_report_resource_source_label(
                    labels,
                    seen,
                    source.get("source_label"),
                )

    written_variants = report.get("written_variants")
    if isinstance(written_variants, list):
        for variant in written_variants:
            if not isinstance(variant, Mapping):
                continue
            _append_report_resource_source_label(
                labels,
                seen,
                variant.get("source_label"),
            )
    return tuple(labels)


def _append_report_resource_source_label(
    labels: list[str],
    seen: set[str],
    source_label: object,
) -> None:
    if (
        _is_registry_backed_resource_locator_string(source_label)
        and source_label not in seen
    ):
        seen.add(source_label)
        labels.append(source_label)


def _is_resource_id_string(value: object) -> bool:
    if not isinstance(value, str) or not value:
        return False
    try:
        UUID(value)
    except ValueError:
        return False
    return True


def _is_resource_locator_string(value: object) -> bool:
    if not isinstance(value, str):
        return False
    scheme, separator, remainder = value.partition("://")
    if separator != "://" or scheme not in _RESOURCE_LOCATOR_SCHEMES:
        return False

    path, has_valid_label = _split_resource_locator_label(remainder)
    if not has_valid_label:
        return False

    if scheme == "package":
        return _is_package_resource_locator_path(path)
    return _is_resource_locator_relative_path(path)


def _is_registry_backed_resource_locator_string(value: object) -> bool:
    if not isinstance(value, str):
        return False
    scheme, separator, _ = value.partition("://")
    return (
        separator == "://"
        and scheme in _RESOURCE_REGISTRY_BACKED_LOCATOR_SCHEMES
        and _is_resource_locator_string(value)
    )


def _split_resource_locator_label(value: str) -> tuple[str, bool]:
    path, separator, label = value.partition("#")
    if separator == "#" and not label:
        return path, False
    return path, True


def _is_package_resource_locator_path(path: str) -> bool:
    package_id, separator, package_path = path.replace("\\", "/").partition("/")
    return (
        separator == "/"
        and _is_plain_resource_locator_segment(package_id)
        and _is_resource_locator_relative_path(package_path)
    )


def _is_resource_locator_relative_path(path: str) -> bool:
    normalized_path = path.replace("\\", "/")
    if (
        not normalized_path
        or normalized_path.startswith("/")
        or _contains_resource_locator_drive_prefix(normalized_path)
    ):
        return False
    normalized: list[str] = []
    for segment in normalized_path.split("/"):
        if segment in ("", "."):
            continue
        if segment == "..":
            if not normalized:
                return False
            normalized.pop()
            continue
        normalized.append(segment)
    return bool(normalized)


def _is_plain_resource_locator_segment(value: str) -> bool:
    return (
        bool(value)
        and "/" not in value
        and "\\" not in value
        and ":" not in value
        and value not in (".", "..")
    )


def _contains_resource_locator_drive_prefix(path: str) -> bool:
    for segment in path.split("/"):
        if len(segment) >= 2 and segment[1] == ":" and segment[0].isalpha():
            return True
    return False


def _is_resource_diagnostic_record(value: object) -> bool:
    if not isinstance(value, Mapping):
        return False
    severity = value.get("severity")
    message = value.get("message")
    return severity in ("Info", "Warning", "Error") and isinstance(message, str)


def _is_unsigned_int_within(value: object, max_value: int) -> bool:
    return (
        isinstance(value, int)
        and not isinstance(value, bool)
        and 0 <= value <= max_value
    )


def _resource_record_locators(records: list[object]) -> set[str]:
    locators: set[str] = set()
    for record in records:
        if not isinstance(record, Mapping):
            continue
        for field in ("primary_locator", "artifact_locator"):
            locator = record.get(field)
            if isinstance(locator, str):
                locators.add(locator)
    return locators


def _usable_shader_resource_record_locators(records: list[object]) -> set[str]:
    locators: set[str] = set()
    for record in records:
        if not isinstance(record, Mapping) or not _is_usable_shader_record(record):
            continue
        for field in ("primary_locator", "artifact_locator"):
            locator = record.get(field)
            if isinstance(locator, str):
                locators.add(locator)
    return locators


def _is_usable_shader_record(record: Mapping[str, object]) -> bool:
    revision = record.get("revision")
    return (
        _resource_record_kind_is_shader(record.get("kind"))
        and record.get("state") == "Ready"
        and isinstance(revision, int)
        and not isinstance(revision, bool)
        and revision > 0
    )


def _resource_record_kind_is_shader(kind: object) -> bool:
    return kind == "Shader"


def _resource_record_kind_is_known(kind: object) -> bool:
    return isinstance(kind, str) and kind in _RESOURCE_RECORD_KINDS


def _resource_record_state_is_known(state: object) -> bool:
    return isinstance(state, str) and state in _RESOURCE_RECORD_STATES


def _unique_in_order(values: Sequence[str]) -> tuple[str, ...]:
    seen: set[str] = set()
    ordered: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        ordered.append(value)
    return tuple(ordered)

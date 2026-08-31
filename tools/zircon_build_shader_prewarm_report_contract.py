"""Shader prewarm report summary and contract helpers."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path


_DIMENSION_SUMMARY_GROUPS = (
    ("pass_types", "pass types"),
    ("geometry_source_ids", "geometry source ids"),
    ("shading_model_ids", "shading model ids"),
    ("quality_tiers", "quality tiers"),
)
_DIMENSION_COUNT_FIELDS = ("requested", "written", "failed")


def shader_prewarm_report_dimension_summary_lines(report_path: Path) -> tuple[str, ...]:
    try:
        report = json.loads(Path(report_path).read_text(encoding="utf-8"))
    except OSError as error:
        return (
            "shader prewarm dimension summary: "
            f"unavailable ({Path(report_path)}: {error})",
        )
    except json.JSONDecodeError as error:
        return (
            "shader prewarm dimension summary: "
            f"unavailable ({Path(report_path)}: {error})",
        )
    if not isinstance(report, Mapping):
        return ()
    return shader_prewarm_dimension_summary_lines(report)


def shader_prewarm_dimension_summary_lines(
    report: Mapping[str, object],
) -> tuple[str, ...]:
    summary = report.get("dimension_summary")
    validation_lines = tuple(
        line
        for line in (
            _format_wgpu_validation(
                report.get("wgpu_module_validation"),
                "module",
            ),
            _format_wgpu_validation(
                report.get("wgpu_pipeline_validation"),
                "render pipeline",
            ),
        )
        if line
    )
    provenance_line = _format_source_provenance(report.get("source_provenance"))
    if not isinstance(summary, Mapping):
        lines = ["shader prewarm dimension summary:"]
        lines.extend(validation_lines)
        if provenance_line:
            lines.append(provenance_line)
        return tuple(lines) if len(lines) > 1 else ()
    lines = ["shader prewarm dimension summary:"]
    lines.extend(validation_lines)
    if provenance_line:
        lines.append(provenance_line)
    for field, label in _DIMENSION_SUMMARY_GROUPS:
        entries = _format_dimension_group(summary.get(field))
        if entries:
            lines.append(f"  {label}: {'; '.join(entries)}")
    if len(lines) == 1:
        return ()
    return tuple(lines)


def validate_shader_prewarm_report_contract(
    report_path: Path,
    *,
    require_wgpu_module_validation: bool = False,
    require_wgpu_pipeline_validation: bool = False,
    require_source_provenance: bool = False,
    expected_pass_types: Sequence[str] = (),
    expected_quality_tiers: Sequence[str] = (),
    expected_geometry_sources: Sequence[str] = (),
    expected_geometry_source_ids: Sequence[str] = (),
    expected_shading_model_ids: Sequence[str] = (),
) -> None:
    if (
        not require_wgpu_module_validation
        and not require_wgpu_pipeline_validation
        and not require_source_provenance
        and not expected_pass_types
        and not expected_quality_tiers
        and not expected_geometry_sources
        and not expected_geometry_source_ids
        and not expected_shading_model_ids
    ):
        return

    report_path = Path(report_path)
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(
            "shader prewarm report unavailable for WGPU module validation "
            f"contract ({report_path}: {error})"
        ) from error
    except json.JSONDecodeError as error:
        raise RuntimeError(
            "shader prewarm report is not valid JSON for WGPU module validation "
            f"contract ({report_path}: {error})"
        ) from error

    if not isinstance(report, Mapping):
        raise RuntimeError("shader prewarm report did not confirm WGPU module validation")

    if require_wgpu_module_validation:
        _validate_wgpu_validation_contract(
            report,
            "wgpu_module_validation",
            "module",
        )
    if require_wgpu_pipeline_validation:
        _validate_wgpu_validation_contract(
            report,
            "wgpu_pipeline_validation",
            "render pipeline",
        )
    if require_source_provenance:
        _validate_source_provenance_contract(report)
    _validate_expected_dimension_contract(
        report,
        expected_pass_types=expected_pass_types,
        expected_quality_tiers=expected_quality_tiers,
        expected_geometry_sources=expected_geometry_sources,
        expected_geometry_source_ids=expected_geometry_source_ids,
        expected_shading_model_ids=expected_shading_model_ids,
    )


def parse_shader_id_record(raw_value: str, label: str) -> tuple[str, int]:
    token, separator, id_text = raw_value.partition("=")
    token = token.strip()
    id_text = id_text.strip()
    if not token or separator != "=" or not id_text:
        raise ValueError(f"{label} must use custom:name=ID or name=ID: {raw_value}")
    try:
        id_value = int(id_text, 10)
    except ValueError as error:
        raise ValueError(f"{label} id must be an integer: {raw_value}") from error
    if id_value < 0 or id_value > 255:
        raise ValueError(f"{label} id must fit in u8: {raw_value}")
    if not token.startswith("custom:"):
        token = f"custom:{token}"
    return token, id_value


def _validate_wgpu_validation_contract(
    report: Mapping[str, object],
    field: str,
    label: str,
) -> None:
    validation = report.get(field)
    if not isinstance(validation, Mapping) or not bool(validation.get("enabled", False)):
        raise RuntimeError(_wgpu_validation_missing_message(label))

    requested = _count_value(validation, "requested")
    validated = _count_value(validation, "validated")
    failed = _count_value(validation, "failed")
    skipped = _count_value(validation, "skipped")
    if requested <= 0:
        raise RuntimeError(_wgpu_validation_missing_message(label))
    if validated != requested or failed or skipped:
        raise RuntimeError(
            f"shader prewarm WGPU {label} validation did not validate every "
            "requested variant: "
            f"requested={requested} validated={validated} "
            f"failed={failed} skipped={skipped}"
        )
    report_requested = _count_value(report, "requested")
    if report_requested <= 0:
        return
    report_written = _count_value(report, "written")
    report_failed = _count_value(report, "failed")
    if (
        requested != report_requested
        or validated != report_written
        or failed != report_failed
    ):
        raise RuntimeError(
            f"shader prewarm WGPU {label} validation counts did not match "
            "report totals: "
            f"requested={report_requested}/{requested} "
            f"written={report_written}/{validated} failed={report_failed}/{failed}"
        )


def _wgpu_validation_missing_message(label: str) -> str:
    messages = {
        "module": "shader prewarm report did not confirm WGPU module validation",
        "render pipeline": "shader prewarm report did not confirm WGPU render pipeline validation",
    }
    return messages.get(label, f"shader prewarm report did not confirm WGPU {label} validation")


def _validate_source_provenance_contract(report: Mapping[str, object]) -> None:
    provenance = report.get("source_provenance")
    if not isinstance(provenance, Mapping):
        raise RuntimeError("shader prewarm report did not confirm shader source provenance")
    sources = provenance.get("sources")
    if not isinstance(sources, Mapping) or not sources:
        raise RuntimeError("shader prewarm report did not confirm shader source provenance")

    source_count = _count_value(provenance, "source")
    variant_count = _count_value(provenance, "variant")
    requested_count = _count_value(report, "requested")
    if source_count <= 0 or variant_count <= 0 or source_count != len(sources):
        raise RuntimeError("shader prewarm report did not confirm shader source provenance")
    if requested_count > 0 and variant_count != requested_count:
        raise RuntimeError(
            "shader prewarm source provenance did not cover every "
            "requested variant: "
            f"requested={requested_count} provenance_variants={variant_count}"
        )

    entry_requested_count = 0
    entry_written_count = 0
    entry_failed_count = 0
    for source in sources.values():
        if not isinstance(source, Mapping):
            raise RuntimeError(
                "shader prewarm report did not confirm shader source provenance"
            )
        source_label = source.get("source_label")
        source_hash = source.get("source_hash")
        template_revision = source.get("template_revision")
        requested = _count_value(source, "requested")
        written = _count_value(source, "written")
        failed = _count_value(source, "failed")
        if (
            not _is_nonblank_string(source_label)
            or not _is_nonblank_string(source_hash)
            or not _is_nonblank_string(template_revision)
            or requested <= 0
            or written + failed != requested
        ):
            raise RuntimeError(
                "shader prewarm report did not confirm shader source provenance"
            )
        entry_requested_count += requested
        entry_written_count += written
        entry_failed_count += failed
    if entry_requested_count != variant_count:
        raise RuntimeError(
            "shader prewarm source provenance did not cover every "
            "reported variant: "
            f"variants={variant_count} source_entries={entry_requested_count}"
        )
    written_count = _count_value(report, "written")
    failed_count = _count_value(report, "failed")
    if (
        requested_count > 0
        and (
            entry_requested_count != requested_count
            or entry_written_count != written_count
            or entry_failed_count != failed_count
        )
    ):
        raise RuntimeError(
            "shader prewarm source provenance counts did not match report totals: "
            f"requested={requested_count}/{entry_requested_count} "
            f"written={written_count}/{entry_written_count} "
            f"failed={failed_count}/{entry_failed_count}"
        )


def _validate_expected_dimension_contract(
    report: Mapping[str, object],
    *,
    expected_pass_types: Sequence[str],
    expected_quality_tiers: Sequence[str],
    expected_geometry_sources: Sequence[str],
    expected_geometry_source_ids: Sequence[str],
    expected_shading_model_ids: Sequence[str],
) -> None:
    summary = report.get("dimension_summary")
    if not isinstance(summary, Mapping):
        if expected_pass_types:
            raise RuntimeError(
                "shader prewarm report is missing requested pass types: "
                + ", ".join(
                    _normalize_dimension_token(pass_type)
                    for pass_type in expected_pass_types
                )
            )
        if expected_quality_tiers:
            raise RuntimeError(
                "shader prewarm report is missing requested quality tiers: "
                + ", ".join(expected_quality_tiers)
            )
        if expected_geometry_sources:
            raise RuntimeError(
                "shader prewarm report is missing requested geometry sources: "
                + ", ".join(expected_geometry_sources)
            )
        if expected_geometry_source_ids:
            raise RuntimeError(
                "shader prewarm report is missing requested shader geometry source ids: "
                + ", ".join(
                    _normalized_shader_id_specs(
                        expected_geometry_source_ids,
                        "shader geometry source id",
                    )
                )
            )
        if expected_shading_model_ids:
            raise RuntimeError(
                "shader prewarm report is missing requested shader shading model ids: "
                + ", ".join(
                    _normalized_shader_id_specs(
                        expected_shading_model_ids,
                        "shader shading model id",
                    )
                )
            )
        return

    _validate_expected_pass_types(summary, expected_pass_types)
    _validate_expected_quality_tiers(summary, expected_quality_tiers)
    _validate_expected_geometry_sources(summary, expected_geometry_sources)
    _validate_expected_shader_dimension_ids(
        summary,
        "geometry_source_ids",
        "shader geometry source id",
        expected_geometry_source_ids,
    )
    _validate_expected_shader_dimension_ids(
        summary,
        "shading_model_ids",
        "shader shading model id",
        expected_shading_model_ids,
    )
    _validate_dimension_summary_totals_match_report(report, summary)


def _validate_dimension_summary_totals_match_report(
    report: Mapping[str, object],
    summary: Mapping[str, object],
) -> None:
    report_requested = _count_value(report, "requested")
    if report_requested <= 0:
        return
    report_written = _count_value(report, "written")
    report_failed = _count_value(report, "failed")
    for field, label in _DIMENSION_SUMMARY_GROUPS:
        group = summary.get(field)
        if not isinstance(group, Mapping) or not group:
            continue
        group_requested, group_written, group_failed = _dimension_group_totals(group)
        if (
            group_requested != report_requested
            or group_written != report_written
            or group_failed != report_failed
        ):
            raise RuntimeError(
                f"shader prewarm report {label[:-1]} counts did not match "
                "report totals: "
                f"requested={report_requested}/{group_requested} "
                f"written={report_written}/{group_written} "
                f"failed={report_failed}/{group_failed}"
            )


def _dimension_group_totals(group: Mapping[str, object]) -> tuple[int, int, int]:
    requested = 0
    written = 0
    failed = 0
    for counts in group.values():
        if not isinstance(counts, Mapping):
            continue
        requested += _count_value(counts, "requested")
        written += _count_value(counts, "written")
        failed += _count_value(counts, "failed")
    return requested, written, failed


def _validate_expected_pass_types(
    summary: Mapping[str, object],
    expected_pass_types: Sequence[str],
) -> None:
    if not expected_pass_types:
        return
    expected = tuple(
        _normalize_dimension_token(pass_type) for pass_type in expected_pass_types
    )
    missing, incomplete = _expected_dimension_count_failures(
        summary.get("pass_types"),
        tuple((pass_type, pass_type) for pass_type in expected),
    )
    if missing:
        raise RuntimeError(
            "shader prewarm report is missing requested pass types: "
            + ", ".join(missing)
        )
    if incomplete:
        raise RuntimeError(
            "shader prewarm report did not fully write requested pass types: "
            + ", ".join(incomplete)
        )


def _validate_expected_quality_tiers(
    summary: Mapping[str, object],
    expected_quality_tiers: Sequence[str],
) -> None:
    if not expected_quality_tiers:
        return
    expected = tuple(_normalize_dimension_token(tier) for tier in expected_quality_tiers)
    missing, incomplete = _expected_dimension_count_failures(
        summary.get("quality_tiers"),
        tuple((tier, tier) for tier in expected),
    )
    if missing:
        raise RuntimeError(
            "shader prewarm report is missing requested quality tiers: "
            + ", ".join(missing)
        )
    if incomplete:
        raise RuntimeError(
            "shader prewarm report did not fully write requested quality tiers: "
            + ", ".join(incomplete)
        )


def _validate_expected_geometry_sources(
    summary: Mapping[str, object],
    expected_geometry_sources: Sequence[str],
) -> None:
    if not expected_geometry_sources:
        return
    expected = tuple(
        _geometry_source_dimension_id(source) for source in expected_geometry_sources
    )
    missing, incomplete = _expected_dimension_count_failures(
        summary.get("geometry_source_ids"),
        tuple((source_id, source_id) for source_id in expected),
    )
    if missing:
        raise RuntimeError(
            "shader prewarm report is missing requested geometry sources: "
            + ", ".join(missing)
        )
    if incomplete:
        raise RuntimeError(
            "shader prewarm report did not fully write requested geometry sources: "
            + ", ".join(incomplete)
        )


def _validate_expected_shader_dimension_ids(
    summary: Mapping[str, object],
    field: str,
    label: str,
    expected_specs: Sequence[str],
) -> None:
    if not expected_specs:
        return
    expected_records = _shader_dimension_id_records(expected_specs, label)
    missing, incomplete = _expected_dimension_count_failures(
        summary.get(field),
        tuple(
            (_format_shader_id_record(token, id_value), str(id_value))
            for token, id_value in expected_records
        ),
    )
    if missing:
        raise RuntimeError(
            _shader_dimension_missing_message(label)
            + ": "
            + ", ".join(missing)
        )
    if incomplete:
        raise RuntimeError(
            _shader_dimension_incomplete_message(label)
            + ": "
            + ", ".join(incomplete)
        )


def _shader_dimension_missing_message(label: str) -> str:
    messages = {
        "shader geometry source id": "shader prewarm report is missing requested shader geometry source ids",
        "shader shading model id": "shader prewarm report is missing requested shader shading model ids",
    }
    return messages.get(label, f"shader prewarm report is missing requested {label}s")


def _shader_dimension_incomplete_message(label: str) -> str:
    messages = {
        "shader geometry source id": "shader prewarm report did not fully write requested shader geometry source ids",
        "shader shading model id": "shader prewarm report did not fully write requested shader shading model ids",
    }
    return messages.get(
        label,
        f"shader prewarm report did not fully write requested {label}s",
    )


def _shader_dimension_id_records(
    raw_values: Sequence[str],
    label: str,
) -> tuple[tuple[str, int], ...]:
    return tuple(parse_shader_id_record(raw_value, label) for raw_value in raw_values)


def _normalized_shader_id_specs(raw_values: Sequence[str], label: str) -> tuple[str, ...]:
    return tuple(
        _format_shader_id_record(token, id_value)
        for token, id_value in _shader_dimension_id_records(raw_values, label)
    )


def _format_shader_id_record(token: str, id_value: int) -> str:
    return f"{token}={id_value}"


def _expected_dimension_count_failures(
    group: object,
    expected: Sequence[tuple[str, str]],
) -> tuple[tuple[str, ...], tuple[str, ...]]:
    if not isinstance(group, Mapping):
        return tuple(display for display, _ in expected), ()

    missing: list[str] = []
    incomplete: list[str] = []
    for display, key in expected:
        counts = group.get(key)
        if not isinstance(counts, Mapping):
            missing.append(display)
            continue
        requested = _count_value(counts, "requested")
        if requested <= 0:
            missing.append(display)
            continue
        written = _count_value(counts, "written")
        failed = _count_value(counts, "failed")
        if written != requested or failed != 0:
            incomplete.append(
                f"{key} requested={requested} written={written} failed={failed}"
            )
    return tuple(missing), tuple(incomplete)


def _dimension_has_requested_count(group: Mapping[str, object], key: str) -> bool:
    counts = group.get(key)
    return isinstance(counts, Mapping) and _count_value(counts, "requested") > 0


def _incomplete_dimension_counts(
    group: Mapping[str, object],
    keys: Sequence[str],
) -> tuple[str, ...]:
    _, incomplete = _expected_dimension_count_failures(
        group,
        tuple((key, key) for key in keys),
    )
    return incomplete


def _geometry_source_dimension_id(source: str) -> str:
    token = _normalize_dimension_token(source).replace("_", "-")
    ids = {
        "static": "0",
        "static-mesh": "0",
        "skinned": "1",
        "skinned-mesh": "1",
        "morphed": "2",
        "morphed-mesh": "2",
        "skinned-morphed": "3",
        "skinned-morphed-mesh": "3",
    }
    return ids.get(token, token)


def _normalize_dimension_token(value: str) -> str:
    return str(value).strip().lower()


def _format_source_provenance(provenance: object) -> str | None:
    if not isinstance(provenance, Mapping):
        return None
    sources = provenance.get("sources")
    if not isinstance(sources, Mapping):
        return None
    entries: list[str] = []
    for source_key, source in sources.items():
        if not isinstance(source, Mapping):
            continue
        if not any(
            field in source or f"{field}_count" in source
            for field in _DIMENSION_COUNT_FIELDS
        ):
            continue
        label = source.get("source_label")
        source_hash = source.get("source_hash")
        template_revision = source.get("template_revision")
        count_text = " ".join(
            f"{field}={_count_value(source, field)}"
            for field in _DIMENSION_COUNT_FIELDS
        )
        entries.append(
            " ".join(
                str(part)
                for part in (
                    label if isinstance(label, str) and label else source_key,
                    (
                        f"source_hash={source_hash}"
                        if isinstance(source_hash, str) and source_hash
                        else None
                    ),
                    (
                        f"template={template_revision}"
                        if isinstance(template_revision, str) and template_revision
                        else None
                    ),
                    count_text,
                )
                if part
            )
        )
    if not entries:
        return None
    return f"  source provenance: {'; '.join(entries)}"


def _format_wgpu_validation(validation: object, label: str) -> str | None:
    if not isinstance(validation, Mapping):
        return None
    enabled = bool(validation.get("enabled", False))
    requested = _count_value(validation, "requested")
    validated = _count_value(validation, "validated")
    failed = _count_value(validation, "failed")
    skipped = _count_value(validation, "skipped")
    if not enabled and not any((requested, validated, failed, skipped)):
        return None
    state = "enabled" if enabled else "disabled"
    return (
        f"  WGPU {label} validation: "
        f"{state} requested={requested} validated={validated} "
        f"failed={failed} skipped={skipped}"
    )


def _format_dimension_group(group: object) -> tuple[str, ...]:
    if not isinstance(group, Mapping):
        return ()
    entries: list[str] = []
    for dimension, counts in group.items():
        if not isinstance(counts, Mapping):
            continue
        if not any(
            field in counts or f"{field}_count" in counts
            for field in _DIMENSION_COUNT_FIELDS
        ):
            continue
        count_text = " ".join(
            f"{field}={_count_value(counts, field)}"
            for field in _DIMENSION_COUNT_FIELDS
        )
        entries.append(f"{dimension} {count_text}")
    return tuple(entries)


def _count_value(counts: Mapping[str, object], field: str) -> int:
    value = counts.get(field)
    if value is None:
        value = counts.get(f"{field}_count", 0)
    return _non_negative_int(value)


def _is_nonblank_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value == value.strip()


def _non_negative_int(value: object) -> int:
    if isinstance(value, bool):
        return 0
    if isinstance(value, int) and value >= 0:
        return value
    return 0

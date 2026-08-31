"""Shader prewarm cache artifact contract helpers."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path

try:
    from .zircon_build_shader_prewarm_report_contract import parse_shader_id_record
    from .zircon_build_shader_prewarm_written_variants import (
        ReportedWrittenVariant,
        reported_written_variants,
        validate_cache_hash_shape,
        validate_written_variant_source_labels,
    )
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_shader_prewarm_report_contract import parse_shader_id_record
    from zircon_build_shader_prewarm_written_variants import (
        ReportedWrittenVariant,
        reported_written_variants,
        validate_cache_hash_shape,
        validate_written_variant_source_labels,
    )


_WGSL_ARTIFACT_SUFFIX = ".wgsl.zst"
_META_ARTIFACT_SUFFIX = ".meta"
_SHADER_VARIANT_CACHE_SCHEMA_VERSION = 1


def validate_shader_prewarm_cache_artifact_contract(
    cache_root: Path,
    *,
    report_path: Path,
    expected_pass_types: Sequence[str] = (),
    expected_quality_tiers: Sequence[str] = (),
    expected_geometry_sources: Sequence[str] = (),
    expected_geometry_source_ids: Sequence[str] = (),
    expected_shading_model_ids: Sequence[str] = (),
) -> None:
    report = _read_shader_prewarm_report(report_path)
    written_count = _count_value(report, "written")
    if written_count <= 0:
        return

    artifact_pairs = _shader_cache_artifact_pairs(Path(cache_root))
    if artifact_pairs.missing_metadata:
        missing = ", ".join(str(path) for path in artifact_pairs.missing_metadata)
        raise RuntimeError(
            "shader prewarm cache artifacts are missing metadata for WGSL artifacts: "
            + missing
        )
    written_variants = reported_written_variants(report)
    if written_variants is not None:
        if len(written_variants) != written_count:
            raise RuntimeError(
                "shader prewarm report written cache variant count does not match "
                f"written_count: written={written_count} "
                f"written_variants={len(written_variants)}"
            )
        artifact_pairs.validate_reported_variants(written_variants)
        validate_written_variant_source_labels(report, written_variants)
        _validate_expected_written_variant_dimensions(
            written_variants,
            expected_pass_types=expected_pass_types,
            expected_quality_tiers=expected_quality_tiers,
            expected_geometry_sources=expected_geometry_sources,
            expected_geometry_source_ids=expected_geometry_source_ids,
            expected_shading_model_ids=expected_shading_model_ids,
        )
    elif (
        expected_pass_types
        or expected_quality_tiers
        or expected_geometry_sources
        or expected_geometry_source_ids
        or expected_shading_model_ids
    ):
        raise RuntimeError(
            "shader prewarm report did not include written cache variants for "
            "requested shader ids"
        )
    if artifact_pairs.pair_count < written_count:
        raise RuntimeError(
            "shader prewarm cache artifacts do not cover written variants: "
            f"written={written_count} cache_pairs={artifact_pairs.pair_count} "
            f"cache_root={Path(cache_root)}"
        )


class _ShaderCacheArtifactPairs:
    def __init__(
        self,
        *,
        pair_count: int,
        missing_metadata: tuple[Path, ...],
        metadata_by_hash: Mapping[str, Mapping[str, object]],
    ) -> None:
        self.pair_count = pair_count
        self.missing_metadata = missing_metadata
        self.metadata_by_hash = metadata_by_hash

    def validate_reported_variants(
        self,
        variants: tuple[ReportedWrittenVariant, ...],
    ) -> None:
        missing = [
            variant.cache_hash
            for variant in variants
            if variant.cache_hash not in self.metadata_by_hash
        ]
        if missing:
            raise RuntimeError(
                "shader prewarm cache artifacts are missing reported cache variants: "
                + ", ".join(missing)
            )
        mismatched = [
            variant.describe_mismatch(self.metadata_by_hash[variant.cache_hash])
            for variant in variants
            if not variant.matches_metadata(self.metadata_by_hash[variant.cache_hash])
        ]
        if mismatched:
            raise RuntimeError(
                "shader prewarm cache reported cache variant mismatch: "
                + "; ".join(mismatched)
            )


class _WrittenVariantDimensionIndex:
    def __init__(self, variants: tuple[ReportedWrittenVariant, ...]) -> None:
        self.values_by_field: dict[str, set[str]] = {}
        self.variant_combinations: set[tuple[str | None, str | None, str | None]] = (
            set()
        )
        self.custom_id_combinations: set[
            tuple[str | None, str | None, str | None, str | None]
        ] = set()
        for variant in variants:
            dimensions = _canonical_dimension_values(
                variant.canonical_string,
                values_by_field=self.values_by_field,
            )
            pass_type = dimensions.get("pass")
            quality_tier = dimensions.get("quality")
            geometry_id = dimensions.get("geometry")
            shading_id = dimensions.get("shading")
            self.variant_combinations.add(
                (pass_type, quality_tier, geometry_id)
            )
            for indexed_pass_type in (pass_type, None):
                for indexed_quality_tier in (quality_tier, None):
                    self.custom_id_combinations.add(
                        (
                            indexed_pass_type,
                            indexed_quality_tier,
                            geometry_id,
                            shading_id,
                        )
                    )


def _validate_expected_written_variant_dimensions(
    variants: tuple[ReportedWrittenVariant, ...],
    *,
    expected_pass_types: Sequence[str],
    expected_quality_tiers: Sequence[str],
    expected_geometry_sources: Sequence[str],
    expected_geometry_source_ids: Sequence[str],
    expected_shading_model_ids: Sequence[str],
) -> None:
    if not (
        expected_pass_types
        or expected_quality_tiers
        or expected_geometry_sources
        or expected_geometry_source_ids
        or expected_shading_model_ids
    ):
        return
    dimension_index = _WrittenVariantDimensionIndex(variants)
    _validate_expected_written_pass_types(dimension_index, expected_pass_types)
    _validate_expected_written_quality_tiers(
        dimension_index,
        expected_quality_tiers,
    )
    _validate_expected_written_geometry_sources(
        dimension_index,
        expected_geometry_sources,
    )
    _validate_expected_written_variant_combinations(
        dimension_index,
        expected_pass_types=expected_pass_types,
        expected_quality_tiers=expected_quality_tiers,
        expected_geometry_sources=expected_geometry_sources,
    )
    _validate_expected_written_dimension(
        dimension_index,
        expected_geometry_source_ids,
        label="shader geometry source",
        canonical_field="geometry",
    )
    _validate_expected_written_dimension(
        dimension_index,
        expected_shading_model_ids,
        label="shader shading model",
        canonical_field="shading",
    )
    _validate_expected_written_custom_id_combinations(
        dimension_index,
        expected_pass_types=expected_pass_types,
        expected_quality_tiers=expected_quality_tiers,
        expected_geometry_source_ids=expected_geometry_source_ids,
        expected_shading_model_ids=expected_shading_model_ids,
    )


def _validate_expected_written_pass_types(
    dimension_index: _WrittenVariantDimensionIndex,
    expected_pass_types: Sequence[str],
) -> None:
    requested = tuple(
        _normalize_dimension_token(pass_type) for pass_type in expected_pass_types
    )
    if not requested:
        return
    missing = [
        pass_type
        for pass_type in requested
        if not _canonical_has_dimension_value(
            dimension_index,
            canonical_field="pass",
            expected_value=pass_type,
        )
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm cache artifacts are missing requested shader pass types: "
            + ", ".join(missing)
        )


def _validate_expected_written_quality_tiers(
    dimension_index: _WrittenVariantDimensionIndex,
    expected_quality_tiers: Sequence[str],
) -> None:
    requested = tuple(
        _normalize_dimension_token(quality_tier)
        for quality_tier in expected_quality_tiers
    )
    if not requested:
        return
    missing = [
        quality_tier
        for quality_tier in requested
        if not _canonical_has_dimension_value(
            dimension_index,
            canonical_field="quality",
            expected_value=quality_tier,
        )
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm cache artifacts are missing requested shader quality tiers: "
            + ", ".join(missing)
        )


def _validate_expected_written_geometry_sources(
    dimension_index: _WrittenVariantDimensionIndex,
    expected_geometry_sources: Sequence[str],
) -> None:
    requested = tuple(
        _geometry_source_dimension_id(source) for source in expected_geometry_sources
    )
    if not requested:
        return
    missing = [
        source_id
        for source_id in requested
        if not _canonical_has_dimension_value(
            dimension_index,
            canonical_field="geometry",
            expected_value=source_id,
        )
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm cache artifacts are missing requested shader geometry sources: "
            + ", ".join(missing)
        )


def _validate_expected_written_variant_combinations(
    dimension_index: _WrittenVariantDimensionIndex,
    *,
    expected_pass_types: Sequence[str],
    expected_quality_tiers: Sequence[str],
    expected_geometry_sources: Sequence[str],
) -> None:
    requested_pass_types = tuple(
        _normalize_dimension_token(pass_type) for pass_type in expected_pass_types
    )
    requested_quality_tiers = tuple(
        _normalize_dimension_token(quality_tier)
        for quality_tier in expected_quality_tiers
    )
    requested_geometry_sources = tuple(
        _geometry_source_dimension_id(source) for source in expected_geometry_sources
    )
    if not (
        requested_pass_types
        and requested_quality_tiers
        and requested_geometry_sources
    ):
        return

    missing = [
        f"pass={pass_type}|quality={quality_tier}|geometry={geometry_source}"
        for pass_type in requested_pass_types
        for quality_tier in requested_quality_tiers
        for geometry_source in requested_geometry_sources
        if (
            pass_type,
            quality_tier,
            geometry_source,
        )
        not in dimension_index.variant_combinations
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm cache artifacts are missing requested shader variant combinations: "
            + ", ".join(missing)
        )


def _validate_expected_written_custom_id_combinations(
    dimension_index: _WrittenVariantDimensionIndex,
    *,
    expected_pass_types: Sequence[str],
    expected_quality_tiers: Sequence[str],
    expected_geometry_source_ids: Sequence[str],
    expected_shading_model_ids: Sequence[str],
) -> None:
    requested_geometry_ids = _expected_shader_id_records(
        expected_geometry_source_ids,
        "shader geometry source",
    )
    requested_shading_ids = _expected_shader_id_records(
        expected_shading_model_ids,
        "shader shading model",
    )
    if not (requested_geometry_ids and requested_shading_ids):
        return

    requested_pass_types = tuple(
        _normalize_dimension_token(pass_type) for pass_type in expected_pass_types
    )
    requested_quality_tiers = tuple(
        _normalize_dimension_token(quality_tier)
        for quality_tier in expected_quality_tiers
    )
    missing = [
        _custom_id_combination_label(
            pass_type=pass_type,
            quality_tier=quality_tier,
            geometry_id=geometry_id,
            shading_id=shading_id,
        )
        for pass_type in (requested_pass_types or (None,))
        for quality_tier in (requested_quality_tiers or (None,))
        for _, geometry_id in requested_geometry_ids
        for _, shading_id in requested_shading_ids
        if not _custom_id_combination_matches(
            dimension_index,
            pass_type=pass_type,
            quality_tier=quality_tier,
            geometry_id=geometry_id,
            shading_id=shading_id,
        )
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm cache artifacts are missing requested shader custom id combinations: "
            + ", ".join(missing)
        )


def _custom_id_combination_matches(
    dimension_index: _WrittenVariantDimensionIndex,
    *,
    pass_type: str | None,
    quality_tier: str | None,
    geometry_id: int,
    shading_id: int,
) -> bool:
    return (
        pass_type,
        quality_tier,
        str(geometry_id),
        str(shading_id),
    ) in dimension_index.custom_id_combinations


def _custom_id_combination_label(
    *,
    pass_type: str | None,
    quality_tier: str | None,
    geometry_id: int,
    shading_id: int,
) -> str:
    parts: list[str] = []
    if pass_type is not None:
        parts.append(f"pass={pass_type}")
    if quality_tier is not None:
        parts.append(f"quality={quality_tier}")
    parts.append(f"geometry={geometry_id}")
    parts.append(f"shading={shading_id}")
    return "|".join(parts)


def _validate_expected_written_dimension(
    dimension_index: _WrittenVariantDimensionIndex,
    expected_ids: Sequence[str],
    *,
    label: str,
    canonical_field: str,
) -> None:
    requested = _expected_shader_id_records(expected_ids, label)
    if not requested:
        return
    missing = [
        f"{token}={id_value}"
        for token, id_value in requested
        if not _canonical_has_dimension_value(
            dimension_index,
            canonical_field=canonical_field,
            expected_value=str(id_value),
        )
    ]
    if missing:
        raise RuntimeError(
            f"shader prewarm cache artifacts are missing requested {label} ids: "
            + ", ".join(missing)
        )


def _expected_shader_id_records(
    expected_ids: Sequence[str],
    label: str,
) -> tuple[tuple[str, int], ...]:
    records: list[tuple[str, int]] = []
    for raw_value in expected_ids:
        try:
            records.append(parse_shader_id_record(str(raw_value), label))
        except ValueError as error:
            raise RuntimeError(str(error)) from error
    return tuple(records)


def _canonical_has_dimension_value(
    dimension_index: _WrittenVariantDimensionIndex,
    *,
    canonical_field: str,
    expected_value: str,
) -> bool:
    values = dimension_index.values_by_field.get(canonical_field)
    return values is not None and expected_value in values


def _canonical_dimension_values(
    canonical_string: str,
    *,
    values_by_field: dict[str, set[str]] | None = None,
) -> Mapping[str, str]:
    dimensions: dict[str, str] = {}
    for part in canonical_string.split("|"):
        field, separator, value = part.partition("=")
        if separator:
            dimensions[field] = value
            if values_by_field is not None:
                values_by_field.setdefault(field, set()).add(value)
    return dimensions


def _normalize_dimension_token(value: str) -> str:
    return str(value).strip().lower()


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


def _shader_cache_artifact_pairs(cache_root: Path) -> _ShaderCacheArtifactPairs:
    if not cache_root.exists() or not cache_root.is_dir():
        return _ShaderCacheArtifactPairs(
            pair_count=0,
            missing_metadata=(),
            metadata_by_hash={},
        )

    pair_count = 0
    missing_metadata: list[Path] = []
    metadata_by_hash: dict[str, Mapping[str, object]] = {}
    for wgsl_path in sorted(cache_root.rglob(f"*{_WGSL_ARTIFACT_SUFFIX}")):
        meta_path = _meta_path_for_wgsl_artifact(wgsl_path)
        if meta_path.exists() and meta_path.is_file():
            metadata = _validate_cache_metadata(
                meta_path,
                wgsl_path=wgsl_path,
                cache_root=cache_root,
            )
            metadata_by_hash[str(metadata["hash"])] = metadata
            pair_count += 1
        else:
            missing_metadata.append(wgsl_path)
    return _ShaderCacheArtifactPairs(
        pair_count=pair_count,
        missing_metadata=tuple(missing_metadata),
        metadata_by_hash=metadata_by_hash,
    )


def _meta_path_for_wgsl_artifact(path: Path) -> Path:
    name = path.name
    if not name.endswith(_WGSL_ARTIFACT_SUFFIX):
        return path.with_suffix(_META_ARTIFACT_SUFFIX)
    artifact_stem = name[: -len(_WGSL_ARTIFACT_SUFFIX)]
    return path.with_name(f"{artifact_stem}{_META_ARTIFACT_SUFFIX}")


def _validate_cache_metadata(
    meta_path: Path,
    *,
    wgsl_path: Path,
    cache_root: Path,
) -> Mapping[str, object]:
    try:
        metadata = json.loads(meta_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(
            f"shader prewarm cache artifact metadata unavailable ({meta_path}: {error})"
        ) from error
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"shader prewarm cache artifact has invalid cache metadata ({meta_path}: {error})"
        ) from error
    if not isinstance(metadata, Mapping):
        raise RuntimeError(
            f"shader prewarm cache artifact has invalid cache metadata ({meta_path})"
        )

    expected_hash = _hash_from_wgsl_artifact(wgsl_path)
    validate_cache_hash_shape(expected_hash, source=f"artifact={wgsl_path}")
    _validate_runtime_cache_layout(
        wgsl_path,
        cache_root=cache_root,
        hash_value=expected_hash,
    )
    actual_hash = metadata.get("hash")
    if actual_hash != expected_hash:
        raise RuntimeError(
            "shader prewarm cache metadata hash mismatch: "
            f"artifact={wgsl_path} metadata={meta_path} "
            f"expected={expected_hash} actual={actual_hash}"
        )
    actual_schema_version = metadata.get("schema_version")
    if not isinstance(actual_schema_version, int) or isinstance(
        actual_schema_version, bool
    ):
        raise RuntimeError(
            "shader prewarm cache artifact has invalid cache metadata "
            f"({meta_path}: invalid field types: schema_version)"
        )
    if actual_schema_version != _SHADER_VARIANT_CACHE_SCHEMA_VERSION:
        raise RuntimeError(
            "shader prewarm cache metadata schema mismatch: "
            f"artifact={wgsl_path} metadata={meta_path} "
            f"expected={_SHADER_VARIANT_CACHE_SCHEMA_VERSION} "
            f"actual={actual_schema_version}"
        )

    required_fields = (
        "canonical_string",
        "template_revision",
        "naga_version",
        "wgpu_version",
        "created_unix_seconds",
    )
    missing = [
        field
        for field in required_fields
        if field not in metadata or metadata.get(field) in ("", None)
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm cache artifact has invalid cache metadata "
            f"({meta_path}: missing {', '.join(missing)})"
        )
    invalid_string_fields = [
        field
        for field in (
            "canonical_string",
            "template_revision",
            "naga_version",
            "wgpu_version",
        )
        if not isinstance(metadata.get(field), str)
    ]
    invalid_integer_fields = [
        field
        for field in ("created_unix_seconds",)
        if not isinstance(metadata.get(field), int)
        or isinstance(metadata.get(field), bool)
    ]
    invalid_fields = invalid_string_fields + invalid_integer_fields
    if invalid_fields:
        raise RuntimeError(
            "shader prewarm cache artifact has invalid cache metadata "
            f"({meta_path}: invalid field types: {', '.join(invalid_fields)})"
        )
    return metadata


def _validate_runtime_cache_layout(
    wgsl_path: Path,
    *,
    cache_root: Path,
    hash_value: str,
) -> None:
    expected_path = _runtime_cache_wgsl_artifact_path(cache_root, hash_value)
    if wgsl_path != expected_path:
        raise RuntimeError(
            "shader prewarm cache artifact is not in runtime cache layout: "
            f"artifact={wgsl_path} expected={expected_path}"
        )


def _runtime_cache_wgsl_artifact_path(cache_root: Path, hash_value: str) -> Path:
    shard = hash_value[:2] or "00"
    return (
        cache_root
        / f"v{_SHADER_VARIANT_CACHE_SCHEMA_VERSION}"
        / shard
        / f"{hash_value}{_WGSL_ARTIFACT_SUFFIX}"
    )


def _hash_from_wgsl_artifact(path: Path) -> str:
    name = path.name
    if name.endswith(_WGSL_ARTIFACT_SUFFIX):
        return name[: -len(_WGSL_ARTIFACT_SUFFIX)]
    return path.stem


def _read_shader_prewarm_report(report_path: Path) -> Mapping[str, object]:
    report_path = Path(report_path)
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(
            "shader prewarm report unavailable for cache artifact contract "
            f"({report_path}: {error})"
        ) from error
    except json.JSONDecodeError as error:
        raise RuntimeError(
            "shader prewarm report is not valid JSON for cache artifact contract "
            f"({report_path}: {error})"
        ) from error
    if not isinstance(report, Mapping):
        raise RuntimeError(
            "shader prewarm report did not provide cache artifact contract data"
        )
    return report


def _count_value(counts: Mapping[str, object], field: str) -> int:
    value = counts.get(field)
    if value is None:
        value = counts.get(f"{field}_count", 0)
    if isinstance(value, bool):
        return 0
    if isinstance(value, int) and value >= 0:
        return value
    return 0

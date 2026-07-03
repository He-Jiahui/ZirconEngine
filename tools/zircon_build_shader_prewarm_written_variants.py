"""Shared staged shader prewarm written variant identity helpers."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

_BLAKE3_HEX_LENGTH = 64
_WRITTEN_VARIANT_SOURCE_LABELS_MISSING_SOURCE_PROVENANCE = (
    "written variant source labels missing source provenance"
)


class ReportedWrittenVariant:
    def __init__(
        self,
        *,
        cache_hash: str,
        canonical_string: str,
        source_label: str | None,
        template_revision: str,
        naga_version: str,
        wgpu_version: str,
    ) -> None:
        self.cache_hash = cache_hash
        self.canonical_string = canonical_string
        self.source_label = source_label
        self.template_revision = template_revision
        self.naga_version = naga_version
        self.wgpu_version = wgpu_version

    def matches_metadata(self, metadata: Mapping[str, object]) -> bool:
        return (
            metadata.get("hash") == self.cache_hash
            and metadata.get("canonical_string") == self.canonical_string
            and metadata.get("template_revision") == self.template_revision
            and metadata.get("naga_version") == self.naga_version
            and metadata.get("wgpu_version") == self.wgpu_version
        )

    def describe_mismatch(self, metadata: Mapping[str, object]) -> str:
        return (
            f"{self.cache_hash} expected canonical={self.canonical_string} "
            f"template={self.template_revision} naga={self.naga_version} "
            f"wgpu={self.wgpu_version} but artifact canonical="
            f"{metadata.get('canonical_string')} template="
            f"{metadata.get('template_revision')} naga={metadata.get('naga_version')} "
            f"wgpu={metadata.get('wgpu_version')}"
        )


def validate_cache_hash_shape(hash_value: str, *, source: str) -> None:
    if len(hash_value) != _BLAKE3_HEX_LENGTH or any(
        character not in "0123456789abcdef" for character in hash_value
    ):
        raise RuntimeError(
            "shader prewarm cache hash shape mismatch: "
            f"{source} expected {_BLAKE3_HEX_LENGTH} lowercase hex characters "
            f"actual={hash_value}"
        )


def reported_written_variants(
    report: Mapping[str, object],
) -> tuple[ReportedWrittenVariant, ...] | None:
    raw_variants = report.get("written_variants")
    if raw_variants is None:
        return None
    if not isinstance(raw_variants, list):
        raise RuntimeError(
            "shader prewarm report contains invalid written cache variants"
        )
    variants: list[ReportedWrittenVariant] = []
    for index, raw_variant in enumerate(raw_variants):
        if not isinstance(raw_variant, Mapping):
            raise RuntimeError(
                "shader prewarm report contains invalid written cache variant "
                f"entry at index {index}"
            )
        required = (
            "cache_hash",
            "canonical_string",
            "template_revision",
            "naga_version",
            "wgpu_version",
        )
        missing = [
            field
            for field in required
            if not isinstance(raw_variant.get(field), str) or not raw_variant.get(field)
        ]
        if missing:
            raise RuntimeError(
                "shader prewarm report contains invalid written cache variant "
                f"entry at index {index}: missing {', '.join(missing)}"
            )
        cache_hash = str(raw_variant["cache_hash"])
        validate_cache_hash_shape(
            cache_hash,
            source=f"written_variants[{index}].cache_hash",
        )
        source_label = raw_variant.get("source_label")
        if source_label is not None and not _is_trimmed_nonblank_string(
            source_label
        ):
            raise RuntimeError(
                "shader prewarm report contains invalid written cache variant "
                f"entry at index {index}: invalid source_label"
            )
        variants.append(
            ReportedWrittenVariant(
                cache_hash=cache_hash,
                canonical_string=str(raw_variant["canonical_string"]),
                source_label=source_label if isinstance(source_label, str) else None,
                template_revision=str(raw_variant["template_revision"]),
                naga_version=str(raw_variant["naga_version"]),
                wgpu_version=str(raw_variant["wgpu_version"]),
            )
        )
    validate_unique_written_variant_identity(variants)
    return tuple(variants)


def validate_unique_written_variant_identity(
    variants: tuple[ReportedWrittenVariant, ...],
    *,
    report_path: Path | None = None,
    message_prefix: str = "shader prewarm report rejects",
) -> None:
    duplicate_cache_hashes = _duplicates(variant.cache_hash for variant in variants)
    duplicate_canonical_strings = _duplicates(
        variant.canonical_string for variant in variants
    )
    if duplicate_cache_hashes or duplicate_canonical_strings:
        details = []
        if duplicate_cache_hashes:
            details.append("cache_hash=" + ", ".join(duplicate_cache_hashes))
        if duplicate_canonical_strings:
            details.append("canonical_string=" + ", ".join(duplicate_canonical_strings))
        location = f" report={report_path}" if report_path is not None else ""
        raise RuntimeError(
            f"{message_prefix} duplicate written cache variant identity: "
            f"{'; '.join(details)}{location}"
        )


def validate_written_variant_source_labels(
    report: Mapping[str, object],
    variants: tuple[ReportedWrittenVariant, ...],
) -> None:
    source_labels = _source_provenance_labels(report)
    if source_labels is None:
        return
    missing = [variant.cache_hash for variant in variants if variant.source_label is None]
    if missing:
        raise RuntimeError(
            "shader prewarm cache written variants are missing source labels "
            "for provenance correlation: "
            + ", ".join(missing)
        )
    unknown = sorted(
        {
            variant.source_label
            for variant in variants
            if variant.source_label not in source_labels
        }
    )
    if unknown:
        raise RuntimeError(
            "shader prewarm cache "
            f"{_WRITTEN_VARIANT_SOURCE_LABELS_MISSING_SOURCE_PROVENANCE}: "
            + ", ".join(unknown)
        )


def _source_provenance_labels(report: Mapping[str, object]) -> set[str] | None:
    provenance = report.get("source_provenance")
    if not isinstance(provenance, Mapping):
        return None
    sources = provenance.get("sources")
    if not isinstance(sources, Mapping) or not sources:
        return None
    labels = {
        source.get("source_label")
        for source in sources.values()
        if isinstance(source, Mapping)
        and _is_trimmed_nonblank_string(source.get("source_label"))
    }
    return set(labels)


def _is_trimmed_nonblank_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value == value.strip()


def _duplicates(values) -> list[str]:
    seen: set[str] = set()
    duplicates: list[str] = []
    for value in values:
        if value in seen:
            duplicates.append(value)
        else:
            seen.add(value)
    return duplicates

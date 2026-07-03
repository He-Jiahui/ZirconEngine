"""Pack delta report semantic diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_pack_delta_asset_set_semantics import (
    delta_changed_assets_are_schema_clean,
    delta_chunks_are_schema_clean,
    pack_document_manifest_fingerprint,
)


def pack_report_delta_publication_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    delta_pack = report.get("delta_pack")
    delta_manifest = report.get("delta_manifest")
    previous_pack = report.get("previous_pack")
    delta_pack_present = report_path_is_present(delta_pack)
    previous_pack_present = report_path_is_present(previous_pack)
    if delta_pack_present:
        if delta_manifest is None:
            diagnostics.append(
                "pack report delta_pack is present but delta_manifest is missing"
            )
    elif isinstance(delta_manifest, dict):
        diagnostics.append(
            "pack report delta_manifest is present but delta_pack is missing"
        )
    if previous_pack_present and not delta_pack_present:
        diagnostics.append(
            "pack report previous_pack is present but delta_pack is missing"
        )
    return diagnostics


def report_path_is_present(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def pack_report_delta_manifest_count_diagnostics(
    report: dict[str, Any],
    delta_manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    changed_assets = delta_manifest.get("changed_assets")
    delta_asset_count = report.get("delta_asset_count")
    if (
        isinstance(changed_assets, list)
        and delta_changed_assets_are_schema_clean(changed_assets)
        and isinstance(delta_asset_count, int)
        and not isinstance(delta_asset_count, bool)
        and delta_asset_count != len(changed_assets)
    ):
        diagnostics.append(
            f"pack report delta_asset_count {delta_asset_count} does not match "
            f"delta_manifest.changed_assets length {len(changed_assets)}"
        )
    chunks = delta_manifest.get("chunks")
    delta_chunk_count = report.get("delta_chunk_count")
    if (
        isinstance(chunks, list)
        and delta_chunks_are_schema_clean(chunks)
        and isinstance(delta_chunk_count, int)
        and not isinstance(delta_chunk_count, bool)
        and delta_chunk_count != len(chunks)
    ):
        diagnostics.append(
            f"pack report delta_chunk_count {delta_chunk_count} does not match "
            f"delta_manifest.chunks length {len(chunks)}"
        )
    return diagnostics


def pack_report_delta_target_manifest_diagnostics(
    manifest: dict[str, Any],
    delta_manifest: dict[str, Any],
) -> list[str]:
    target = delta_manifest.get("target")
    if not isinstance(target, dict):
        return []
    parsed_manifest = pack_document_manifest_fingerprint(manifest)
    parsed_target = pack_document_manifest_fingerprint(target)
    if parsed_manifest is None or parsed_target is None:
        return []
    if parsed_manifest == parsed_target:
        return []
    return ["pack report delta_manifest.target does not match manifest"]

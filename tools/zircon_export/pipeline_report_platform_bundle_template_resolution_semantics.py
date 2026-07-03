"""PlatformBundle template-resolution semantic diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_platform_bundle_template_resolution_path_semantics import (
    template_resolution_path_containment_diagnostics,
    template_resolution_template_dir_uniqueness_diagnostics,
)


def template_resolution_selected_candidate_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not False:
        return []
    template_dir = resolution.get("template_dir")
    candidates = resolution.get("candidates")
    if (
        not isinstance(template_dir, str)
        or not template_dir.strip()
        or not isinstance(candidates, list)
    ):
        return []
    candidate_count = sum(1 for candidate in candidates if isinstance(candidate, dict))
    if candidate_count != 1:
        return [
            f"{label} non-fatal resolution must contain exactly one candidate"
        ]
    matching_candidates = [
        candidate
        for candidate in candidates
        if isinstance(candidate, dict)
        and isinstance(candidate.get("template_dir"), str)
        and candidate["template_dir"] == template_dir
    ]
    if len(matching_candidates) != 1:
        return [
            f"{label}.template_dir must match exactly one candidates[].template_dir"
        ]
    return []


def template_resolution_fatal_selection_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not True:
        return []
    template_dir = resolution.get("template_dir")
    if isinstance(template_dir, str) and template_dir.strip():
        return [f"{label} fatal resolution must not select template_dir"]
    return []


def template_resolution_non_fatal_diagnostics_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not False:
        return []
    diagnostics = resolution.get("diagnostics")
    if (
        isinstance(diagnostics, list)
        and any(isinstance(entry, str) and entry.strip() for entry in diagnostics)
    ):
        return [f"{label} non-fatal resolution must not include diagnostics"]
    return []


def template_resolution_non_fatal_expected_identity_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not False:
        return []
    diagnostics: list[str] = []
    for field in ("expected_engine_version", "expected_target_platform"):
        value = resolution.get(field)
        if not isinstance(value, str) or not value.strip():
            diagnostics.append(
                f"{label} non-fatal resolution must include {field}"
            )
    return diagnostics

def template_resolution_non_fatal_selection_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not False:
        return []
    template_dir = resolution.get("template_dir")
    if not isinstance(template_dir, str) or not template_dir.strip():
        return [f"{label} non-fatal resolution must select template_dir"]
    return []


def template_resolution_skipped_candidate_diagnostics_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    skipped_candidates = resolution.get("skipped_candidates")
    if not isinstance(skipped_candidates, list):
        return []
    diagnostics: list[str] = []
    for index, skipped_candidate in enumerate(skipped_candidates):
        if not isinstance(skipped_candidate, dict):
            continue
        candidate_diagnostics = skipped_candidate.get("diagnostics")
        if not isinstance(candidate_diagnostics, list):
            continue
        if not any(
            isinstance(entry, str) and entry.strip()
            for entry in candidate_diagnostics
        ):
            diagnostics.append(
                f"{label} skipped_candidates[{index}].diagnostics "
                "must include diagnostics"
            )
    return diagnostics

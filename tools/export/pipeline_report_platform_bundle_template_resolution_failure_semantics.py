"""PlatformBundle template-resolution failure semantic diagnostics."""

from __future__ import annotations

from typing import Any


def template_resolution_fatal_candidate_count_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not True:
        return []
    if template_resolution_object_row_count(resolution, "candidates") == 1:
        return [f"{label} fatal resolution must not contain exactly one candidate"]
    return []


def template_resolution_fatal_diagnostics_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    diagnostics = resolution.get("diagnostics")
    if (
        resolution.get("fatal") is True
        and isinstance(diagnostics, list)
        and not _trimmed_diagnostic_entries(resolution)
    ):
        return [f"{label} fatal resolution must include diagnostics"]
    return []


def template_resolution_fatal_diagnostic_family_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not True:
        return []
    candidates = resolution.get("candidates")
    diagnostics = resolution.get("diagnostics")
    if not isinstance(candidates, list) or not isinstance(diagnostics, list):
        return []

    candidate_count = template_resolution_object_row_count(resolution, "candidates")
    has_multiple_match = _has_diagnostic_prefix(
        resolution, "multiple export templates matched profile="
    )
    has_root_failure = _has_diagnostic_prefix(resolution, "export template root ")
    has_no_match = _has_diagnostic_prefix(resolution, "no export template under ")
    if candidate_count > 1 and (has_root_failure or has_no_match):
        return [
            f"{label} fatal resolution with multiple candidates must not include "
            "root-failure or no-match diagnostics"
        ]
    if candidate_count == 0 and has_multiple_match:
        return [
            f"{label} fatal resolution with no candidates must not include "
            "multiple-match diagnostics"
        ]
    if candidate_count == 0 and has_root_failure and has_no_match:
        return [
            f"{label} fatal resolution with no candidates must not mix "
            "root-failure and no-match diagnostics"
        ]
    return []


def template_resolution_fatal_multiple_candidate_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if (
        resolution.get("fatal") is not True
        or template_resolution_object_row_count(resolution, "candidates") <= 1
    ):
        return []
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    profile = resolution.get("profile")
    expected_prefix = (
        f"multiple export templates matched profile={profile}:"
        if isinstance(profile, str) and profile.strip()
        else "multiple export templates matched profile="
    )
    matching_entries = _diagnostic_entries_with_prefix(resolution, expected_prefix)
    if not matching_entries:
        profile_note = (
            f" for profile {profile}"
            if isinstance(profile, str) and profile.strip()
            else ""
        )
        return [
            f"{label} fatal resolution with multiple candidates "
            f"must include multiple-match diagnostics{profile_note}"
        ]
    candidate_dirs = _candidate_template_dirs(resolution)
    if candidate_dirs and not any(
        all(candidate_dir in entry for candidate_dir in candidate_dirs)
        for entry in matching_entries
    ):
        return [
            f"{label} fatal multiple-match diagnostics must include "
            "all candidate template_dir values"
        ]
    return []


def template_resolution_fatal_no_candidate_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if (
        resolution.get("fatal") is not True
        or template_resolution_object_row_count(resolution, "candidates") != 0
    ):
        return []
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    if not any(
        _has_diagnostic_prefix(resolution, prefix)
        for prefix in ("export template root ", "no export template under ")
    ):
        return [
            f"{label} fatal resolution with no candidates "
            "must include root-failure or no-match diagnostics"
        ]
    return []


def template_resolution_no_match_profile_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    profile = resolution.get("profile")
    if not isinstance(profile, str) or not profile.strip():
        return []
    no_match_entries = _diagnostic_entries_with_prefix(resolution, "no export template under ")
    if no_match_entries and not any(
        f"profile={profile}" in entry for entry in no_match_entries
    ):
        return [f"{label} fatal no-match diagnostics must include profile {profile}"]
    return []


def template_resolution_no_match_identity_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    no_match_entries = _diagnostic_entries_with_prefix(resolution, "no export template under ")
    if not no_match_entries:
        return []
    expected_fields = (
        ("expected_target_platform", "target_platform", "<any>"),
        ("expected_engine_version", "engine_version", "<unresolved>"),
    )
    for source_field, diagnostic_field, unresolved_marker in expected_fields:
        expected_value = resolution.get(source_field)
        if isinstance(expected_value, str) and expected_value.strip():
            expected_token = f"{diagnostic_field}={expected_value}"
            expected_label = expected_value
        else:
            expected_token = f"{diagnostic_field}={unresolved_marker}"
            expected_label = unresolved_marker
        if not any(expected_token in entry for entry in no_match_entries):
            return [
                f"{label} fatal no-match diagnostics must include "
                f"{diagnostic_field} {expected_label}"
            ]
    return []


def template_resolution_no_match_root_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    template_root = resolution.get("template_root")
    if not isinstance(template_root, str) or not template_root.strip():
        return []
    expected_prefix = f"no export template under {template_root} "
    no_match_entries = _diagnostic_entries_with_prefix(resolution, "no export template under ")
    if no_match_entries and not any(
        entry.startswith(expected_prefix) for entry in no_match_entries
    ):
        return [f"{label} fatal no-match diagnostics must include template_root"]
    return []


def template_resolution_root_failure_candidate_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if not _has_diagnostic_prefix(resolution, "export template root "):
        return []
    candidate_count = template_resolution_object_row_count(resolution, "candidates")
    skipped_count = template_resolution_object_row_count(resolution, "skipped_candidates")
    if candidate_count or skipped_count:
        return [f"{label} root-failure resolution must not include candidate rows"]
    return []


def template_resolution_root_failure_root_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    template_root = resolution.get("template_root")
    if not isinstance(template_root, str) or not template_root.strip():
        return []
    expected_prefix = f"export template root {template_root} "
    root_failure_entries = _diagnostic_entries_with_prefix(resolution, "export template root ")
    if root_failure_entries and not any(
        entry.startswith(expected_prefix) for entry in root_failure_entries
    ):
        return [f"{label} root-failure diagnostics must include template_root"]
    return []


def template_resolution_object_row_count(resolution: dict[str, Any], field: str) -> int:
    value = resolution.get(field)
    if not isinstance(value, list):
        return 0
    return sum(1 for entry in value if isinstance(entry, dict))


def _diagnostic_entries_with_prefix(resolution: dict[str, Any], prefix: str) -> list[str]:
    return [
        entry
        for entry in _trimmed_diagnostic_entries(resolution)
        if entry.startswith(prefix)
    ]


def _has_diagnostic_prefix(resolution: dict[str, Any], prefix: str) -> bool:
    return bool(_diagnostic_entries_with_prefix(resolution, prefix))


def _trimmed_diagnostic_entries(resolution: dict[str, Any]) -> list[str]:
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    return [
        entry.strip()
        for entry in diagnostics
        if isinstance(entry, str) and entry.strip()
    ]


def _candidate_template_dirs(resolution: dict[str, Any]) -> list[str]:
    candidates = resolution.get("candidates")
    if not isinstance(candidates, list):
        return []
    return [
        candidate["template_dir"]
        for candidate in candidates
        if isinstance(candidate, dict)
        and isinstance(candidate.get("template_dir"), str)
        and candidate["template_dir"].strip()
    ]

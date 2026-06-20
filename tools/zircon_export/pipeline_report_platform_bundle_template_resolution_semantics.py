"""PlatformBundle template-resolution semantic diagnostics."""

from __future__ import annotations

from os.path import normcase
from pathlib import Path
from typing import Any

from .export_template import (
    EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    normalize_target_platform,
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


def template_resolution_fatal_candidate_count_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not True:
        return []
    candidates = resolution.get("candidates")
    if not isinstance(candidates, list):
        return []
    candidate_count = sum(1 for candidate in candidates if isinstance(candidate, dict))
    if candidate_count == 1:
        return [f"{label} fatal resolution must not contain exactly one candidate"]
    return []


def template_resolution_fatal_diagnostics_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    if resolution.get("fatal") is not True:
        return []
    diagnostics = resolution.get("diagnostics")
    if (
        isinstance(diagnostics, list)
        and not any(isinstance(entry, str) and entry.strip() for entry in diagnostics)
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
    candidate_count = sum(1 for candidate in candidates if isinstance(candidate, dict))
    has_multiple_match = any(
        isinstance(entry, str)
        and entry.strip().startswith("multiple export templates matched profile=")
        for entry in diagnostics
    )
    has_root_failure = any(
        isinstance(entry, str)
        and entry.strip().startswith("export template root ")
        for entry in diagnostics
    )
    has_no_match = any(
        isinstance(entry, str)
        and entry.strip().startswith("no export template under ")
        for entry in diagnostics
    )
    has_no_candidate_failure = has_root_failure or has_no_match
    if candidate_count > 1 and has_no_candidate_failure:
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
    if resolution.get("fatal") is not True:
        return []
    candidates = resolution.get("candidates")
    candidate_count = (
        sum(1 for candidate in candidates if isinstance(candidate, dict))
        if isinstance(candidates, list)
        else 0
    )
    if candidate_count <= 1:
        return []
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    profile = resolution.get("profile")
    if isinstance(profile, str) and profile.strip():
        expected_prefix = f"multiple export templates matched profile={profile}:"
    else:
        expected_prefix = "multiple export templates matched profile="
    has_multiple_match_diagnostic = any(
        isinstance(entry, str)
        and entry.strip().startswith(expected_prefix)
        for entry in diagnostics
    )
    if not has_multiple_match_diagnostic:
        profile_note = (
            f" for profile {profile}"
            if isinstance(profile, str) and profile.strip()
            else ""
        )
        return [
            f"{label} fatal resolution with multiple candidates "
            f"must include multiple-match diagnostics{profile_note}"
        ]
    candidate_dirs = [
        candidate.get("template_dir")
        for candidate in candidates
        if isinstance(candidate, dict)
        and isinstance(candidate.get("template_dir"), str)
        and candidate["template_dir"].strip()
    ]
    matching_diagnostic_entries = [
        entry.strip()
        for entry in diagnostics
        if isinstance(entry, str)
        and entry.strip().startswith(expected_prefix)
    ]
    if candidate_dirs and not any(
        all(candidate_dir in entry for candidate_dir in candidate_dirs)
        for entry in matching_diagnostic_entries
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
    if resolution.get("fatal") is not True:
        return []
    candidates = resolution.get("candidates")
    candidate_count = (
        sum(1 for candidate in candidates if isinstance(candidate, dict))
        if isinstance(candidates, list)
        else 0
    )
    if candidate_count != 0:
        return []
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    prefixes = (
        "export template root ",
        "no export template under ",
    )
    has_resolution_failure_diagnostic = any(
        isinstance(entry, str)
        and any(entry.strip().startswith(prefix) for prefix in prefixes)
        for entry in diagnostics
    )
    if not has_resolution_failure_diagnostic:
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
    diagnostics = resolution.get("diagnostics")
    if not isinstance(profile, str) or not profile.strip():
        return []
    if not isinstance(diagnostics, list):
        return []
    no_match_entries = [
        entry.strip()
        for entry in diagnostics
        if isinstance(entry, str)
        and entry.strip().startswith("no export template under ")
    ]
    if not no_match_entries:
        return []
    expected_profile = f"profile={profile}"
    if not any(expected_profile in entry for entry in no_match_entries):
        return [f"{label} fatal no-match diagnostics must include profile {profile}"]
    return []


def template_resolution_no_match_identity_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    no_match_entries = [
        entry.strip()
        for entry in diagnostics
        if isinstance(entry, str)
        and entry.strip().startswith("no export template under ")
    ]
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
    diagnostics = resolution.get("diagnostics")
    if not isinstance(template_root, str) or not template_root.strip():
        return []
    if not isinstance(diagnostics, list):
        return []
    expected_prefix = f"no export template under {template_root} "
    no_match_entries = [
        entry.strip()
        for entry in diagnostics
        if isinstance(entry, str)
        and entry.strip().startswith("no export template under ")
    ]
    if no_match_entries and not any(
        entry.startswith(expected_prefix) for entry in no_match_entries
    ):
        return [f"{label} fatal no-match diagnostics must include template_root"]
    return []


def template_resolution_root_failure_candidate_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    diagnostics = resolution.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    has_root_failure = any(
        isinstance(entry, str)
        and entry.strip().startswith("export template root ")
        for entry in diagnostics
    )
    if not has_root_failure:
        return []
    candidate_count = template_resolution_object_row_count(resolution, "candidates")
    skipped_candidate_count = template_resolution_object_row_count(
        resolution,
        "skipped_candidates",
    )
    if candidate_count or skipped_candidate_count:
        return [f"{label} root-failure resolution must not include candidate rows"]
    return []


def template_resolution_root_failure_root_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    template_root = resolution.get("template_root")
    diagnostics = resolution.get("diagnostics")
    if not isinstance(template_root, str) or not template_root.strip():
        return []
    if not isinstance(diagnostics, list):
        return []
    root_failure_entries = [
        entry.strip()
        for entry in diagnostics
        if isinstance(entry, str)
        and entry.strip().startswith("export template root ")
    ]
    expected_prefix = f"export template root {template_root} "
    if root_failure_entries and not any(
        entry.startswith(expected_prefix) for entry in root_failure_entries
    ):
        return [f"{label} root-failure diagnostics must include template_root"]
    return []


def template_resolution_object_row_count(
    resolution: dict[str, Any],
    field: str,
) -> int:
    value = resolution.get(field)
    if not isinstance(value, list):
        return 0
    return sum(1 for entry in value if isinstance(entry, dict))


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


def template_resolution_candidate_profile_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    profile = resolution.get("profile")
    candidates = resolution.get("candidates")
    if not isinstance(profile, str) or not profile.strip() or not isinstance(candidates, list):
        return []
    diagnostics: list[str] = []
    for index, candidate in enumerate(candidates):
        if not isinstance(candidate, dict):
            continue
        compatible_profiles = candidate.get("compatible_profiles")
        if (
            not isinstance(compatible_profiles, list)
            or not compatible_profiles
            or any(
                not isinstance(value, str) or not value.strip()
                for value in compatible_profiles
            )
        ):
            continue
        if profile not in compatible_profiles:
            diagnostics.append(
                f"{label} candidates[{index}].compatible_profiles "
                f"does not include profile {profile}"
            )
    return diagnostics


def template_resolution_candidate_identity_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    expected_engine_version = resolution.get("expected_engine_version")
    expected_target_platform = resolution.get("expected_target_platform")
    candidates = resolution.get("candidates")
    if not isinstance(candidates, list):
        return []
    diagnostics: list[str] = []
    for index, candidate in enumerate(candidates):
        if not isinstance(candidate, dict):
            continue
        engine_version = candidate.get("engine_version")
        if (
            isinstance(engine_version, str)
            and engine_version.strip()
            and isinstance(expected_engine_version, str)
            and expected_engine_version.strip()
            and engine_version != expected_engine_version
        ):
            diagnostics.append(
                f"{label} candidates[{index}].engine_version {engine_version} "
                f"does not match expected_engine_version {expected_engine_version}"
            )
        target_platform = candidate.get("target_platform")
        if (
            isinstance(target_platform, str)
            and target_platform.strip()
            and isinstance(expected_target_platform, str)
            and expected_target_platform.strip()
            and normalize_target_platform(target_platform)
            != normalize_target_platform(expected_target_platform)
        ):
            diagnostics.append(
                f"{label} candidates[{index}].target_platform {target_platform} "
                f"does not match expected_target_platform {expected_target_platform}"
            )
    return diagnostics


def template_resolution_candidate_bundle_format_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    candidates = resolution.get("candidates")
    if not isinstance(candidates, list):
        return []
    diagnostics: list[str] = []
    for index, candidate in enumerate(candidates):
        if not isinstance(candidate, dict):
            continue
        bundle_format = candidate.get("bundle_format")
        if (
            isinstance(bundle_format, str)
            and bundle_format.strip()
            and bundle_format not in EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS
        ):
            diagnostics.append(
                f"{label} candidates[{index}].bundle_format={bundle_format!r} "
                "is not one of "
                f"{', '.join(sorted(EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS))}"
            )
    return diagnostics


def template_resolution_path_containment_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    template_root = resolution.get("template_root")
    if not isinstance(template_root, str) or not template_root.strip():
        return []
    try:
        resolved_root = Path(template_root).expanduser().resolve()
    except OSError as error:
        return [f"{label}.template_root could not be resolved: {error}"]

    diagnostics: list[str] = []
    diagnostics.extend(
        template_resolution_entries_inside_root_diagnostics(
            label,
            resolved_root,
            resolution,
            "candidates",
        )
    )
    diagnostics.extend(
        template_resolution_entries_inside_root_diagnostics(
            label,
            resolved_root,
            resolution,
            "skipped_candidates",
        )
    )
    return diagnostics


def template_resolution_template_dir_uniqueness_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    seen: dict[str, tuple[str, int]] = {}
    for field in ("candidates", "skipped_candidates"):
        entries = resolution.get(field)
        if not isinstance(entries, list):
            continue
        for index, entry in enumerate(entries):
            if not isinstance(entry, dict):
                continue
            template_dir = entry.get("template_dir")
            if not isinstance(template_dir, str) or not template_dir.strip():
                continue
            try:
                key = normcase(str(Path(template_dir).expanduser().resolve()))
            except OSError:
                continue
            if key in seen:
                seen_field, seen_index = seen[key]
                diagnostics.append(
                    f"{label} {field}[{index}].template_dir duplicates "
                    f"{seen_field}[{seen_index}].template_dir"
                )
                continue
            seen[key] = (field, index)
    return diagnostics


def template_resolution_entries_inside_root_diagnostics(
    label: str,
    resolved_root: Path,
    resolution: dict[str, Any],
    field: str,
) -> list[str]:
    entries = resolution.get(field)
    if not isinstance(entries, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            continue
        template_dir = entry.get("template_dir")
        if not isinstance(template_dir, str) or not template_dir.strip():
            continue
        try:
            resolved_template_dir = Path(template_dir).expanduser().resolve()
            relative_template_dir = resolved_template_dir.relative_to(resolved_root)
        except ValueError:
            diagnostics.append(
                f"{label} {field}[{index}].template_dir must be inside template_root"
            )
        except OSError as error:
            diagnostics.append(
                f"{label} {field}[{index}].template_dir could not be resolved: {error}"
            )
        else:
            if len(relative_template_dir.parts) != 1:
                diagnostics.append(
                    f"{label} {field}[{index}].template_dir "
                    "must be a direct child of template_root"
                )
    return diagnostics

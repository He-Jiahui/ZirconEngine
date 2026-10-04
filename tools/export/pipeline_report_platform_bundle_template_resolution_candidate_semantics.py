"""PlatformBundle template-resolution candidate semantic diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template import (
    EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
)
from .export_template_manifest import normalize_target_platform


def template_resolution_candidate_profile_diagnostics(
    label: str,
    resolution: dict[str, Any],
) -> list[str]:
    profile = resolution.get("profile")
    candidates = resolution.get("candidates")
    if (
        not isinstance(profile, str)
        or not profile.strip()
        or not isinstance(candidates, list)
    ):
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
        host_artifact = candidate.get("host_artifact")
        if (
            isinstance(host_artifact, str)
            and host_artifact.strip()
            and host_artifact not in EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS
        ):
            diagnostics.append(
                f"{label} candidates[{index}].host_artifact={host_artifact!r} "
                "is not one of "
                f"{', '.join(sorted(EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS))}"
            )
    return diagnostics

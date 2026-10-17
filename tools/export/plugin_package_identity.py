"""Shared plugin package identity helpers."""

from __future__ import annotations

from typing import Any

from .native_dynamic_contract import native_dynamic_package_directory


def optional_feature_directory(package_id: str | None, feature_id: str) -> str:
    """Keep public feature IDs stable while honoring the engine path budget."""
    suffix = (
        feature_id.removeprefix(f"{package_id}.")
        if package_id is not None and feature_id.startswith(f"{package_id}.")
        else feature_id
    )
    short_directories = {
        ("rendering", "subsurface_scattering"): "subsurface",
        ("sound", "timeline_animation_track"): "timeline",
        ("sound", "ray_traced_convolution_reverb"): "ray_reverb",
    }
    return short_directories.get((package_id, suffix), suffix.replace(".", "/"))


def feature_provider_package_id(feature: dict[str, Any], feature_id: str) -> str | None:
    provider_package_id = feature.get("provider_package_id")
    if provider_package_id is None:
        return native_dynamic_package_directory(feature_id)
    if not isinstance(provider_package_id, str) or not provider_package_id.strip():
        return None
    if provider_package_id.strip() != provider_package_id:
        return None
    return provider_package_id

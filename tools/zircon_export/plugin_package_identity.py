"""Shared plugin package identity helpers."""

from __future__ import annotations

from typing import Any

from .native_dynamic_contract import native_dynamic_package_directory


def feature_provider_package_id(feature: dict[str, Any], feature_id: str) -> str | None:
    provider_package_id = feature.get("provider_package_id")
    if provider_package_id is None:
        return native_dynamic_package_directory(feature_id)
    if not isinstance(provider_package_id, str) or not provider_package_id.strip():
        return None
    if provider_package_id.strip() != provider_package_id:
        return None
    return provider_package_id

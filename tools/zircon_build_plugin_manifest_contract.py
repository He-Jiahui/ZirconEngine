"""Plugin manifest contract helpers for zircon_build."""

from __future__ import annotations

from pathlib import Path
from typing import Iterable


PLUGIN_DISTRIBUTION_FORM_DIST = "dist"
PLUGIN_DISTRIBUTION_FORM_EMBED = "embed"
PLUGIN_DISTRIBUTION_FORMS = (
    PLUGIN_DISTRIBUTION_FORM_EMBED,
    PLUGIN_DISTRIBUTION_FORM_DIST,
)


def distribution_table(data: dict) -> dict:
    distribution = data.get("distribution", {})
    if isinstance(distribution, dict):
        return distribution
    return {}


def require_distribution_forms(
    manifest_path: Path,
    distribution: dict,
) -> tuple[str, ...]:
    raw_forms = distribution.get("forms")
    if not isinstance(raw_forms, list) or not raw_forms:
        raise SystemExit(
            f"{manifest_path}: distribution.forms must be a non-empty array "
            f"containing only {', '.join(PLUGIN_DISTRIBUTION_FORMS)}"
        )
    forms: list[str] = []
    for index, value in enumerate(raw_forms, start=1):
        form = str(value).strip().lower()
        if form not in PLUGIN_DISTRIBUTION_FORMS:
            raise SystemExit(
                f"{manifest_path}: distribution.forms[{index}] must be one of "
                f"{', '.join(PLUGIN_DISTRIBUTION_FORMS)}"
            )
        forms.append(form)
    return tuple(_unique_in_order(forms))


def normalize_optional_string(value: object) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def collect_module_crate_names(data: dict) -> list[str]:
    crate_names: list[str] = []
    for module in data.get("modules", []):
        append_module_crate(crate_names, module)
    for feature_key in ("optional_features", "feature_extensions"):
        for feature in data.get(feature_key, []):
            for module in feature.get("modules", []):
                append_module_crate(crate_names, module)
    return crate_names


def append_module_crate(crate_names: list[str], module: object) -> None:
    if not isinstance(module, dict):
        return
    crate_name = module.get("crate_name")
    if crate_name:
        crate_names.append(str(crate_name))


def _unique_in_order(values: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result

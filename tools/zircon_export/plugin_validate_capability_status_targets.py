"""Target-mode checks for plugin capability status manifests."""

from __future__ import annotations

from typing import Any

Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_MODES = (
    "client_runtime",
    "server_runtime",
    "editor_host",
)
PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_DUPLICATE_MESSAGE = (
    "duplicates capability_status target_modes"
)
PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_COVERAGE_MESSAGE = (
    "should be covered by package supported_targets"
)


def plugin_validate_capability_status_supported_targets(manifest: Manifest) -> set[str]:
    supported_targets = manifest.get("supported_targets")
    if not isinstance(supported_targets, list):
        return set()
    return {
        target
        for target in supported_targets
        if isinstance(target, str) and target.strip() and target.strip() == target
    }


def validate_plugin_capability_status_targets(
    status: Manifest,
    row_label: str,
    supported_targets: set[str],
    diagnostics: Diagnostics,
) -> None:
    target_modes = plugin_validate_capability_status_target_modes(
        status, f"{row_label}.target_modes", diagnostics
    )
    if target_modes is None:
        return
    seen: dict[str, int] = {}
    allowed_targets = set(PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_MODES)
    expected = ", ".join(PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_MODES)
    for index, target_mode in enumerate(target_modes):
        item_label = f"{row_label}.target_modes[{index}]"
        if target_mode not in allowed_targets:
            diagnostics.append(
                f'{item_label} "{target_mode}" is unsupported; expected one of '
                f"{expected}"
            )
            continue
        previous_index = seen.get(target_mode)
        if previous_index is not None:
            diagnostics.append(
                f"{item_label} {target_mode} "
                f"{PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_DUPLICATE_MESSAGE} "
                f"target_modes[{previous_index}]"
            )
        else:
            seen[target_mode] = index
        if supported_targets and target_mode not in supported_targets:
            diagnostics.append(
                f"{item_label} {target_mode} "
                f"{PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_COVERAGE_MESSAGE}"
            )


def plugin_validate_capability_status_target_modes(
    status: Manifest,
    label: str,
    diagnostics: Diagnostics,
) -> list[str] | None:
    if "target_modes" not in status:
        return None
    value = status["target_modes"]
    if not isinstance(value, list):
        diagnostics.append(f"{label} must be an array")
        return None
    if not value:
        diagnostics.append(f"{label} must not be empty when declared")
        return None
    values: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            diagnostics.append(f"{label}[{index}] must be a non-empty trimmed string")
            continue
        values.append(item)
    return values

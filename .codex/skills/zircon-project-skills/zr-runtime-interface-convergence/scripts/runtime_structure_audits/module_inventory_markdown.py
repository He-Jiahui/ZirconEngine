from __future__ import annotations


def render_module_descriptor_distribution_markdown(
    distribution: dict[str, list[dict[str, object]]],
) -> list[str]:
    lines = ["## Module Descriptor Distribution"]
    for crate_name, locations in distribution.items():
        if not locations:
            continue
        lines.append(f"- `{crate_name}`")
        for location in locations:
            lines.append(f"  - `{location['path']}:{location['line']}`")
    return lines


def render_stub_module_descriptor_usage_markdown(
    stub_usage: dict[str, list[dict[str, object]]],
) -> list[str]:
    lines = ["## Stub Module Descriptor Usage"]
    if not stub_usage:
        lines.append("- none")
    else:
        for crate_name, locations in stub_usage.items():
            lines.append(f"- `{crate_name}`")
            for location in locations:
                lines.append(f"  - `{location['path']}:{location['line']}`")
    return lines


def render_engine_module_owner_coverage_markdown(
    coverage: dict[str, list[str]],
) -> list[str]:
    lines = ["## EngineModule Owner Coverage"]
    for crate_name, owners in coverage.items():
        if owners:
            lines.append(f"- `{crate_name}`: {', '.join(sorted(owners))}")
        else:
            lines.append(f"- `{crate_name}`: missing")
    return lines


def render_module_classification_markdown(
    classifications: dict[str, dict[str, object]],
) -> list[str]:
    lines = ["## Module Classification"]
    for crate_name, entry in classifications.items():
        reasons = entry["reasons"]
        if reasons:
            lines.append(f"- `{crate_name}`: {entry['status']} ({', '.join(reasons)})")
        else:
            lines.append(f"- `{crate_name}`: {entry['status']}")
    return lines


def render_support_crates_markdown(support_crates: list[str]) -> list[str]:
    if not support_crates:
        return []
    return [
        "## Support Crates Outside Module Classification",
        f"- {', '.join(support_crates)}",
    ]

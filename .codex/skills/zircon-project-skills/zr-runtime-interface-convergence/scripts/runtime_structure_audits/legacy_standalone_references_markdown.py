from __future__ import annotations


def render_legacy_standalone_references_markdown(
    legacy_refs: dict[str, object],
) -> list[str]:
    lines = ["## Legacy Standalone References In Architecture Docs"]
    if not legacy_refs["counts"]:
        lines.append("- none")
    else:
        for crate_name, count in legacy_refs["counts"].items():
            lines.append(f"- `{crate_name}`: {count} line reference(s)")
    return lines

from __future__ import annotations


def render_plugin_runtime_gaps_markdown(gaps: list[str]) -> list[str]:
    lines = ["## Plugin Runtime Gaps"]
    if not gaps:
        lines.append("- none")
    else:
        lines.extend(f"- {gap}" for gap in gaps)
    return lines

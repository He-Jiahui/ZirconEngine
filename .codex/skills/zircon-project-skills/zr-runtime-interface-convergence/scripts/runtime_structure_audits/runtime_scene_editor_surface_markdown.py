from __future__ import annotations


def render_runtime_scene_editor_surface_markdown(surface: dict[str, object]) -> list[str]:
    editor_named_paths = surface["editor_named_paths"]
    public_editor_locations = surface["public_editor_named_locations"]
    lines = [
        "## Runtime Scene Editor Surface",
        "- editor-named production paths "
        f"({len(editor_named_paths)}): "
        f"{', '.join(editor_named_paths) if editor_named_paths else 'none'}",
    ]
    if not public_editor_locations:
        lines.append("- public editor-named locations: none")
    else:
        lines.append(f"- public editor-named locations ({len(public_editor_locations)}):")
        for location in public_editor_locations:
            lines.append(
                f"  - `{location['path']}:{location['line']}` {location['snippet']}"
            )

    for risk in surface["risks"]:
        lines.append(f"- risk: {risk}")

    return lines

from __future__ import annotations

import re
from pathlib import Path


SCENE_PROJECT_SERIALIZATION_FILES = (
    "zircon_runtime/src/scene/world/world.rs",
    "zircon_runtime/src/scene/world/project_io.rs",
    "zircon_runtime/src/scene/world/project_io/camera.rs",
    "zircon_runtime/src/scene/world/project_io/physics.rs",
    "zircon_runtime/src/scene/world/project_io/post_process.rs",
    "zircon_runtime/src/scene/world/project_io/references.rs",
    "zircon_runtime/src/scene/world/project_io/script.rs",
    "zircon_runtime/src/scene/world/project_io/transform.rs",
    "zircon_runtime/src/scene/dynamic_scene/document/mod.rs",
    "zircon_runtime/src/scene/dynamic_scene/document/read.rs",
    "zircon_runtime/src/scene/dynamic_scene/document/migration/mod.rs",
    "zircon_runtime/src/scene/dynamic_scene/document/migration/project_world.rs",
    "zircon_runtime/src/scene/dynamic_scene/document/write.rs",
    "zircon_runtime/src/scene/dynamic_scene/entity/dynamic_component.rs",
    "zircon_runtime/src/scene/dynamic_scene/entity/dynamic_entity.rs",
    "zircon_runtime/src/scene/dynamic_scene/entity/dynamic_resource.rs",
    "zircon_runtime/src/scene/dynamic_scene/entity/mod.rs",
    "zircon_runtime/src/scene/dynamic_scene/scene/capture.rs",
    "zircon_runtime/src/scene/dynamic_scene/scene/mod.rs",
    "zircon_runtime/src/scene/dynamic_scene/scene/spawn.rs",
    "zircon_runtime/src/scene/dynamic_scene/scene/validation.rs",
    "zircon_runtime/src/scene/dynamic_scene/value/json.rs",
    "zircon_runtime/src/scene/dynamic_scene/value/mod.rs",
    "zircon_runtime/src/scene/dynamic_scene/value/remap.rs",
    "zircon_runtime/src/asset/assets/scene/mod.rs",
)

SCENE_PROJECT_SERIALIZATION_FORBIDDEN_PATTERNS = {
    "selection-state": re.compile(
        r"\b(selected|selection|selected_entity|selected_node|set_selected)\b"
    ),
    "viewport-authoring-state": re.compile(
        r"\b(SceneViewportSettings|SceneViewportTool|TransformSpace|GridMode|"
        r"ViewOrientation|viewport_camera|ViewportCameraSnapshot)\b"
    ),
    "overlay-authoring-state": re.compile(
        r"\b(RenderOverlayExtract|SelectionHighlightExtract|SelectionAnchorExtract|"
        r"GridOverlayExtract|HandleOverlayExtract|SceneGizmoOverlayExtract|"
        r"SceneGizmoKind|scene_gizmos|selection_anchors|gizmo|overlay)\b",
        re.I,
    ),
    "preview-authoring-state": re.compile(
        r"\b(active_camera_override|camera_override|preview_lighting|preview_skybox|"
        r"display_mode|DisplayMode)\b"
    ),
    "editor-pane-state": re.compile(r"\b(pane|Pane)\b"),
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _location(
    path: str,
    line: int,
    snippet: str,
    max_snippet: int | None = None,
) -> dict[str, object]:
    if max_snippet is not None and len(snippet) > max_snippet:
        snippet = f"{snippet[:max_snippet].rstrip()}..."
    return {
        "path": path,
        "line": line,
        "snippet": snippet,
    }


def _find_locations(
    root: Path,
    files: list[Path],
    pattern: re.Pattern[str],
    max_snippet: int | None = None,
) -> list[dict[str, object]]:
    results: list[dict[str, object]] = []
    for path in files:
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            if pattern.search(line):
                results.append(
                    _location(
                        _relative(root, path),
                        line_no,
                        line.strip(),
                        max_snippet=max_snippet,
                    )
                )
    return results


def scene_project_serialization_boundary_audit(root: Path) -> dict[str, object]:
    files = [
        root / relative_path
        for relative_path in SCENE_PROJECT_SERIALIZATION_FILES
        if (root / relative_path).exists()
    ]
    forbidden_locations: dict[str, list[dict[str, object]]] = {}
    for label, pattern in SCENE_PROJECT_SERIALIZATION_FORBIDDEN_PATTERNS.items():
        locations = _find_locations(root, files, pattern, max_snippet=220)
        if locations:
            forbidden_locations[label] = locations

    forbidden_location_count = sum(
        len(locations) for locations in forbidden_locations.values()
    )
    risks: list[str] = []
    if forbidden_locations:
        risks.append(
            "runtime scene project serialization sources contain editor authoring-state names; "
            "M3 should keep selection, viewport tool state, viewport camera overrides, overlays, gizmos, panes, and preview overrides in zircon_editor or host-local state."
        )

    return {
        "files": [_relative(root, path) for path in files],
        "forbidden_locations": dict(sorted(forbidden_locations.items())),
        "forbidden_location_count": forbidden_location_count,
        "risks": risks,
    }

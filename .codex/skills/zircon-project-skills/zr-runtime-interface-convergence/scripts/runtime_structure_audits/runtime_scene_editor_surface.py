from __future__ import annotations

import re
from pathlib import Path


EDITOR_NAMED_SYMBOL_RE = re.compile(r"\b(editor_projection|SceneEditor[A-Za-z0-9_]*)\b")


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _is_production_file(path: Path) -> bool:
    return "tests" not in path.parts and path.name != "tests.rs"


def _location(path: str, line: int, snippet: str) -> dict[str, object]:
    return {
        "path": path,
        "line": line,
        "snippet": snippet,
    }


def _find_locations(
    root: Path,
    files: list[Path],
    pattern: re.Pattern[str],
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
                    )
                )
    return results


def runtime_scene_editor_surface_audit(root: Path) -> dict[str, object]:
    scene_root = root / "zircon_runtime" / "src" / "scene"
    if not scene_root.exists():
        return {
            "editor_named_paths": [],
            "public_editor_named_locations": [],
            "editor_named_locations": [],
            "risks": [],
        }

    scene_files = sorted(path for path in scene_root.rglob("*.rs") if _is_production_file(path))
    editor_named_paths = [
        _relative(root, path)
        for path in scene_files
        if "editor" in path.relative_to(scene_root).as_posix().lower()
    ]
    editor_named_locations = _find_locations(root, scene_files, EDITOR_NAMED_SYMBOL_RE)
    public_editor_named_locations = [
        location
        for location in editor_named_locations
        if str(location["snippet"]).startswith("pub mod")
        or str(location["snippet"]).startswith("pub use")
        or str(location["snippet"]).startswith("pub struct")
        or str(location["snippet"]).startswith("pub fn")
    ]

    risks: list[str] = []
    if editor_named_paths:
        risks.append(
            "zircon_runtime::scene still has editor-named production paths; "
            "M3 should move this to neutral inspection/reflection naming."
        )
    if public_editor_named_locations:
        risks.append(
            "zircon_runtime::scene publicly exposes editor-named DTOs or modules; "
            "editor hierarchy/inspector/viewport DTOs should live in zircon_editor."
        )

    return {
        "editor_named_paths": editor_named_paths,
        "public_editor_named_locations": public_editor_named_locations,
        "editor_named_locations": editor_named_locations[:20],
        "risks": risks,
    }

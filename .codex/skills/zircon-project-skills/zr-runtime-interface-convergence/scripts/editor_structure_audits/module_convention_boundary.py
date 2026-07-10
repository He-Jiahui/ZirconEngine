from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any


EDITOR_SRC = Path("zircon_editor/src")
PRODUCTION_LINE_LIMIT = 1000
VISUAL_STYLE_OWNER_FILES = (
    "mod.rs",
    "button.rs",
    "component.rs",
    "model.rs",
    "surface.rs",
)


@dataclass(frozen=True)
class EditorModuleConventionAudit:
    production_file_count: int
    oversized_production_files: list[dict[str, Any]]
    production_dead_code_suppressions: list[str]
    banned_name_modules: list[str]
    ui_module_owner_boundary_violations: list[str]
    duplicate_test_trees: list[str]
    stale_visual_style_file_exists: bool
    visual_style_missing_owner_files: list[str]

    def to_json(self) -> dict[str, Any]:
        migration_debt_count = (
            len(self.oversized_production_files)
            + len(self.production_dead_code_suppressions)
            + len(self.banned_name_modules)
            + len(self.ui_module_owner_boundary_violations)
            + len(self.duplicate_test_trees)
            + int(self.stale_visual_style_file_exists)
            + len(self.visual_style_missing_owner_files)
        )
        return {
            "m1_gate_status": (
                "classified-and-clear"
                if migration_debt_count == 0
                else "migration-debt-present"
            ),
            "production_file_count": self.production_file_count,
            "migration_debt_count": migration_debt_count,
            "oversized_production_file_count": len(self.oversized_production_files),
            "oversized_production_files": self.oversized_production_files,
            "production_dead_code_suppression_count": len(
                self.production_dead_code_suppressions
            ),
            "production_dead_code_suppressions": self.production_dead_code_suppressions,
            "banned_name_module_count": len(self.banned_name_modules),
            "banned_name_modules": self.banned_name_modules,
            "ui_module_owner_boundary_violation_count": len(
                self.ui_module_owner_boundary_violations
            ),
            "ui_module_owner_boundary_violations": (
                self.ui_module_owner_boundary_violations
            ),
            "duplicate_test_tree_count": len(self.duplicate_test_trees),
            "duplicate_test_trees": self.duplicate_test_trees,
            "visual_style_owner_tree": {
                "old_file_exists": self.stale_visual_style_file_exists,
                "missing_owner_files": self.visual_style_missing_owner_files,
                "owner_files": list(VISUAL_STYLE_OWNER_FILES),
            },
        }


def editor_module_convention_audit(repo_root: Path) -> EditorModuleConventionAudit:
    editor_src = repo_root / EDITOR_SRC
    production_files = sorted(
        path
        for path in editor_src.rglob("*.rs")
        if "tests" not in path.relative_to(editor_src).parts
    )
    return EditorModuleConventionAudit(
        production_file_count=len(production_files),
        oversized_production_files=oversized_production_files(
            repo_root,
            production_files,
        ),
        production_dead_code_suppressions=production_dead_code_suppressions(
            repo_root,
            production_files,
        ),
        banned_name_modules=banned_name_modules(repo_root, editor_src),
        ui_module_owner_boundary_violations=ui_module_owner_boundary_violations(
            repo_root,
            editor_src,
        ),
        duplicate_test_trees=duplicate_test_trees(repo_root, editor_src),
        stale_visual_style_file_exists=(
            editor_src
            / "ui/retained_host/ui/pane_data_conversion/pane_component_projection/visual_style.rs"
        ).exists(),
        visual_style_missing_owner_files=visual_style_missing_owner_files(
            repo_root,
            editor_src,
        ),
    )


def oversized_production_files(
    repo_root: Path,
    production_files: list[Path],
) -> list[dict[str, Any]]:
    oversized = []
    for path in production_files:
        line_count = line_count_for(path)
        if line_count > PRODUCTION_LINE_LIMIT:
            oversized.append(
                {
                    "path": display_path(repo_root, path),
                    "lines": line_count,
                    "owner_class": owner_class_for(path),
                }
            )
    return oversized


def production_dead_code_suppressions(
    repo_root: Path,
    production_files: list[Path],
) -> list[str]:
    return [
        display_path(repo_root, path)
        for path in production_files
        if "#[allow(dead_code)]" in path.read_text(encoding="utf-8")
    ]


def banned_name_modules(repo_root: Path, editor_src: Path) -> list[str]:
    banned = []
    banned_tokens = ("_inner", "_impl", "_helper")
    for path in editor_src.rglob("*"):
        if not path.is_file() and not path.is_dir():
            continue
        if "tests" in path.relative_to(editor_src).parts:
            continue
        name = path.stem if path.is_file() else path.name
        if any(token in name for token in banned_tokens) or name in {
            "util",
            "utils",
            "misc",
        }:
            banned.append(display_path(repo_root, path))
    return sorted(banned)


def ui_module_owner_boundary_violations(repo_root: Path, editor_src: Path) -> list[str]:
    ui_root = editor_src / "ui"
    allowed_root_files = {"mod.rs", "prelude.rs"}
    if not ui_root.exists():
        return []
    return sorted(
        display_path(repo_root, path)
        for path in ui_root.glob("*.rs")
        if path.name not in allowed_root_files
    )


def duplicate_test_trees(repo_root: Path, editor_src: Path) -> list[str]:
    crate_tests = editor_src / "tests"
    ui_tests = editor_src / "ui"
    if not crate_tests.exists() or not ui_tests.exists():
        return []
    crate_test_names = {
        path.stem
        for path in crate_tests.rglob("*.rs")
        if path.name != "mod.rs"
    }
    ui_test_names = {
        path.stem.removesuffix("_tests")
        for path in ui_tests.rglob("*tests.rs")
    }
    overlaps = sorted(crate_test_names.intersection(ui_test_names))
    return [f"{display_path(repo_root, crate_tests)} <-> ui::{name}" for name in overlaps]


def visual_style_missing_owner_files(repo_root: Path, editor_src: Path) -> list[str]:
    owner_root = (
        editor_src
        / "ui/retained_host/ui/pane_data_conversion/pane_component_projection/visual_style"
    )
    return [
        display_path(repo_root, owner_root / file_name)
        for file_name in VISUAL_STYLE_OWNER_FILES
        if not (owner_root / file_name).exists()
    ]


def line_count_for(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def owner_class_for(path: Path) -> str:
    path_text = path.as_posix()
    if "/ui/retained_host/" in path_text:
        return "editor-retained-host"
    if "/ui/" in path_text:
        return "editor-ui"
    if "/core/" in path_text:
        return "editor-core"
    if "/scene/" in path_text:
        return "editor-scene"
    return "editor"


def display_path(repo_root: Path, path: Path) -> str:
    return path.relative_to(repo_root).as_posix()

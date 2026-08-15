#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from editor_structure_audits.module_convention_boundary import (
    editor_module_convention_audit,
)


def repo_root() -> Path:
    current = Path(__file__).resolve()
    for parent in (current, *current.parents):
        if (parent / "zircon_editor" / "Cargo.toml").exists():
            return parent
    raise RuntimeError("Could not find repository root from audit script location.")


def build_report(root: Path) -> dict[str, Any]:
    module_convention_gate = editor_module_convention_audit(root).to_json()
    report = {
        "module_convention_gate": module_convention_gate,
        "summary": {
            "m1_gate_status": module_convention_gate["m1_gate_status"],
            "migration_debt_count": module_convention_gate["migration_debt_count"],
            "oversized_production_file_count": module_convention_gate[
                "oversized_production_file_count"
            ],
            "oversized_test_file_count": module_convention_gate[
                "oversized_test_file_count"
            ],
            "production_dead_code_suppression_count": module_convention_gate[
                "production_dead_code_suppression_count"
            ],
        },
    }
    return report


def render_markdown(report: dict[str, Any]) -> str:
    gate = report["module_convention_gate"]
    lines = [
        "# Editor Structure Audit",
        "",
        f"- M1 gate status: `{gate['m1_gate_status']}`",
        f"- Migration debt count: {gate['migration_debt_count']}",
        f"- Production files: {gate['production_file_count']}",
        f"- Oversized production files: {gate['oversized_production_file_count']}",
        f"- Oversized test files: {gate['oversized_test_file_count']}",
        f"- Production dead-code suppressions: {gate['production_dead_code_suppression_count']}",
        f"- Banned name modules: {gate['banned_name_module_count']}",
        f"- UI owner boundary violations: {gate['ui_module_owner_boundary_violation_count']}",
        f"- Duplicate test trees: {gate['duplicate_test_tree_count']}",
        f"- Visual-style old file exists: {gate['visual_style_owner_tree']['old_file_exists']}",
    ]
    for title, key in [
        ("Oversized Production Files", "oversized_production_files"),
        ("Oversized Test Files", "oversized_test_files"),
        ("Production Dead-Code Suppressions", "production_dead_code_suppressions"),
        ("Banned Name Modules", "banned_name_modules"),
        ("UI Owner Boundary Violations", "ui_module_owner_boundary_violations"),
        ("Duplicate Test Trees", "duplicate_test_trees"),
    ]:
        entries = gate[key]
        if entries:
            lines.append("")
            lines.append(f"## {title}")
            for entry in entries:
                lines.append(f"- `{entry}`")
    missing_visual_style = gate["visual_style_owner_tree"]["missing_owner_files"]
    if missing_visual_style:
        lines.append("")
        lines.append("## Missing Visual Style Owner Files")
        lines.extend(f"- `{path}`" for path in missing_visual_style)
    test_exemptions = gate["oversized_test_file_exemptions"]
    if test_exemptions:
        lines.append("")
        lines.append("## Oversized Test File Exemptions")
        lines.extend(
            f"- `{entry['path']}`: {entry['reason']}"
            for entry in test_exemptions
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit zirconEngine editor structure.")
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of Markdown.")
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="Repository root override.",
    )
    args = parser.parse_args()

    root = (args.repo_root or repo_root()).resolve()
    report = build_report(root)
    if args.json:
        print(json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True))
    else:
        print(render_markdown(report), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

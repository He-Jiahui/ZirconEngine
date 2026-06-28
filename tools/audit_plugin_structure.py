#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from plugin_structure_audits.capability import audit_plugin_capability_conformance
from plugin_structure_audits.dependency_boundary import (
    audit_plugin_dependency_boundary,
)
from plugin_structure_audits.manifest_schema import audit_plugin_manifest_schema
from plugin_structure_audits.registration import audit_plugin_registration_conformance
from plugin_structure_audits.skeleton import audit_plugin_skeleton_conformance


def repo_root() -> Path:
    current = Path(__file__).resolve()
    for parent in (current, *current.parents):
        if (parent / "zircon_plugins" / "Cargo.toml").exists():
            return parent
    raise RuntimeError("Could not find repository root from audit script location.")


def build_report(root: Path) -> dict[str, Any]:
    manifest_schema = audit_plugin_manifest_schema(root).to_json()
    skeleton_conformance = audit_plugin_skeleton_conformance(root).to_json()
    registration_conformance = audit_plugin_registration_conformance(root).to_json()
    capability_conformance = audit_plugin_capability_conformance(root).to_json()
    dependency_boundary = audit_plugin_dependency_boundary(root).to_json()
    skeleton_sample_is_clean = (
        skeleton_conformance["sample_conformance_status"] == "sample-clean"
    )
    skeleton_migration_debt_count = skeleton_conformance["migration_debt_count"]
    skeleton_gate_status = (
        "sample-clean-migration-debt-clear"
        if skeleton_sample_is_clean and skeleton_migration_debt_count == 0
        else "sample-clean-migration-debt-present"
        if skeleton_sample_is_clean
        else "sample-violations-present"
    )
    report = {
        "plugin_manifest_schema_uniform": manifest_schema,
        "skeleton_conformance": skeleton_conformance,
        "registration_conformance": registration_conformance,
        "capability_conformance": capability_conformance,
        "standalone_distribution_conformance": dependency_boundary,
        "plugin_skeleton_gate": {
            "m2_gate_status": skeleton_gate_status,
            "migration_debt_count": skeleton_migration_debt_count,
        },
        "summary": {
            "missing_plugin_toml": manifest_schema["missing_plugin_toml"],
            "manifest_schema_violations": manifest_schema[
                "manifest_schema_violations"
            ],
            "skeleton_migration_debt_count": skeleton_conformance[
                "migration_debt_count"
            ],
            "asset_importer_family_free_function_registration_sites": (
                registration_conformance[
                    "asset_importer_family_free_function_registration_sites"
                ]
            ),
            "split_importer_free_function_registration_sites": (
                registration_conformance[
                    "split_importer_free_function_registration_sites"
                ]
            ),
            "importer_free_function_registration_sites": (
                registration_conformance["importer_free_function_registration_sites"]
            ),
            "runtime_registration_builder_violation_count": (
                registration_conformance[
                    "runtime_registration_builder_violation_count"
                ]
            ),
            "m3_t2_runtime_registration_builder_status": (
                registration_conformance[
                    "m3_t2_runtime_registration_builder_status"
                ]
            ),
            "capability_source_mismatches": capability_conformance[
                "capability_source_mismatches"
            ],
            "capability_audited_runtime_root_count": capability_conformance[
                "audited_runtime_root_count"
            ],
            "m4_runtime_capability_gate_status": capability_conformance[
                "m4_runtime_capability_gate_status"
            ],
            "sdk_builder_mirror_violations": capability_conformance[
                "sdk_builder_mirror_violations"
            ],
            "m4_t2_builder_mirror_gate_status": capability_conformance[
                "m4_t2_builder_mirror_gate_status"
            ],
            "editor_runtime_mirror_violations": capability_conformance[
                "editor_runtime_mirror_violations"
            ],
            "d9_editor_runtime_mirror_gate_status": capability_conformance[
                "d9_editor_runtime_mirror_gate_status"
            ],
            "dist_capable_plugin_count": dependency_boundary[
                "dist_capable_plugin_count"
            ],
            "dist_build_matrix_count": dependency_boundary[
                "dist_build_matrix_count"
            ],
            "distribution_section_violations": dependency_boundary[
                "distribution_section_violations"
            ],
            "dist_dependency_boundary_violations": dependency_boundary[
                "dist_dependency_boundary_violations"
            ],
            "m1_dist_dependency_boundary_gate_status": dependency_boundary[
                "m1_dist_dependency_boundary_gate_status"
            ],
        },
    }
    report["m1_gate_status"] = (
        "classified-and-clear"
        if report["summary"]["missing_plugin_toml"] == 0
        and report["summary"]["manifest_schema_violations"] == 0
        else "migration-debt-present"
    )
    return report


def render_markdown(report: dict[str, Any]) -> str:
    manifest_schema = report["plugin_manifest_schema_uniform"]
    capability = report["capability_conformance"]
    standalone = report["standalone_distribution_conformance"]
    lines = [
        "# Plugin Structure Audit",
        "",
        f"- M1 gate status: `{report['m1_gate_status']}`",
        f"- Expected plugin manifests: {manifest_schema['expected_manifest_count']}",
        f"- Present plugin manifests: {manifest_schema['manifest_count']}",
        f"- Missing plugin.toml: {manifest_schema['missing_plugin_toml']}",
        f"- Manifest schema violations: {manifest_schema['manifest_schema_violations']}",
        f"- Skeleton sample status: `{report['skeleton_conformance']['sample_conformance_status']}`",
        f"- Core workspace dependency status: `{report['skeleton_conformance']['core_workspace_dependency_status']}`",
        f"- Core workspace dependency count: {report['skeleton_conformance']['core_workspace_dependency_count']}",
        f"- Core workspace dependency violations: {report['skeleton_conformance']['core_workspace_dependency_violation_count']}",
        f"- Skeleton migration debt roots: {report['skeleton_conformance']['migration_debt_count']}",
        f"- Skeleton migration debt details: {report['skeleton_conformance']['migration_debt_detail_count']}",
        f"- Asset importer family free-function registration sites: {report['registration_conformance']['asset_importer_family_free_function_registration_sites']}",
        f"- Registration M3/T1 gate status: `{report['registration_conformance']['m3_t1_gate_status']}`",
        f"- Split importer free-function registration sites: {report['registration_conformance']['split_importer_free_function_registration_sites']}",
        f"- Split importer registration gate status: `{report['registration_conformance']['m3_split_importer_gate_status']}`",
        f"- Importer registration gate status: `{report['registration_conformance']['m3_importer_gate_status']}`",
        f"- Runtime registration builder roots: {len(report['registration_conformance']['runtime_registration_builder_roots'])}",
        f"- Runtime registration builder violations: {report['registration_conformance']['runtime_registration_builder_violation_count']}",
        f"- Runtime registration builder gate status: `{report['registration_conformance']['m3_t2_runtime_registration_builder_status']}`",
        f"- Capability audited runtime roots: {capability['audited_runtime_root_count']}",
        f"- Capability source mismatches: {capability['capability_source_mismatches']}",
        f"- M4 runtime capability gate status: `{capability['m4_runtime_capability_gate_status']}`",
        f"- SDK builder/mirror violations: {capability['sdk_builder_mirror_violations']}",
        f"- M4/T2 builder mirror gate status: `{capability['m4_t2_builder_mirror_gate_status']}`",
        f"- Editor-runtime mirror roots: {capability['editor_runtime_mirror_root_count']}",
        f"- Editor-runtime mirror violations: {capability['editor_runtime_mirror_violations']}",
        f"- D9 editor/runtime mirror gate status: `{capability['d9_editor_runtime_mirror_gate_status']}`",
        f"- Dist-capable plugins: {standalone['dist_capable_plugin_count']}",
        f"- Dist build matrix entries: {standalone['dist_build_matrix_count']}",
        f"- Distribution section violations: {standalone['distribution_section_violations']}",
        f"- Dist dependency boundary violations: {standalone['dist_dependency_boundary_violations']}",
        f"- Plugins 13 M1 dist boundary gate status: `{standalone['m1_dist_dependency_boundary_gate_status']}`",
    ]
    if manifest_schema["missing_plugin_toml_paths"]:
        lines.append("")
        lines.append("## Missing Manifests")
        lines.extend(f"- `{path}`" for path in manifest_schema["missing_plugin_toml_paths"])
    if manifest_schema["manifest_schema_violation_details"]:
        lines.append("")
        lines.append("## Manifest Schema Violations")
        lines.extend(
            f"- {violation}"
            for violation in manifest_schema["manifest_schema_violation_details"]
        )
    skeleton = report["skeleton_conformance"]
    if skeleton["sample_violations"]:
        lines.append("")
        lines.append("## Skeleton Sample Violations")
        lines.extend(f"- {violation}" for violation in skeleton["sample_violations"])
    if skeleton["core_workspace_dependency_violations"]:
        lines.append("")
        lines.append("## Core Workspace Dependency Violations")
        lines.extend(
            f"- {violation}"
            for violation in skeleton["core_workspace_dependency_violations"]
        )
    if skeleton["migration_debt_roots"]:
        lines.append("")
        lines.append("## Skeleton Migration Debt Roots")
        lines.extend(f"- `{root}`" for root in skeleton["migration_debt_roots"])
    registration = report["registration_conformance"]
    if registration["asset_importer_family_free_function_registration_site_details"]:
        lines.append("")
        lines.append("## Asset Importer Family Free-Function Registration Sites")
        lines.extend(
            f"- `{site}`"
            for site in registration[
                "asset_importer_family_free_function_registration_site_details"
            ]
        )
    if registration["split_importer_free_function_registration_site_details"]:
        lines.append("")
        lines.append("## Split Importer Free-Function Registration Sites")
        lines.extend(
            f"- `{site}`"
            for site in registration[
                "split_importer_free_function_registration_site_details"
            ]
        )
    if registration["runtime_registration_builder_violations"]:
        lines.append("")
        lines.append("## Runtime Registration Builder Violations")
        lines.extend(
            f"- `{violation}`"
            for violation in registration["runtime_registration_builder_violations"]
        )
    if capability["capability_source_mismatch_details"]:
        lines.append("")
        lines.append("## Capability Source Mismatches")
        lines.extend(
            f"- `{mismatch}`"
            for mismatch in capability["capability_source_mismatch_details"]
        )
    if capability["sdk_builder_mirror_violation_details"]:
        lines.append("")
        lines.append("## SDK Builder Mirror Violations")
        lines.extend(
            f"- `{violation}`"
            for violation in capability["sdk_builder_mirror_violation_details"]
        )
    if capability["editor_runtime_mirror_violation_details"]:
        lines.append("")
        lines.append("## Editor Runtime Mirror Violations")
        lines.extend(
            f"- `{violation}`"
            for violation in capability["editor_runtime_mirror_violation_details"]
        )
    if standalone["distribution_section_violation_details"]:
        lines.append("")
        lines.append("## Distribution Section Violations")
        lines.extend(
            f"- `{violation}`"
            for violation in standalone["distribution_section_violation_details"]
        )
    if standalone["dist_dependency_boundary_violation_details"]:
        lines.append("")
        lines.append("## Dist Dependency Boundary Violations")
        lines.extend(
            f"- `{violation}`"
            for violation in standalone["dist_dependency_boundary_violation_details"]
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit zirconEngine plugin structure.")
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

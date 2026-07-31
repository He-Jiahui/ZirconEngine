from __future__ import annotations


def render_tech_stack_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    manifests = boundary["manifest_files"]
    lines = [
        "## Runtime 01 Tech Stack Boundary",
        "- audited manifest files "
        f"({len(manifests)}/{boundary['expected_manifest_count']}): "
        f"{len(manifests)} files",
        "- corrected non-dependencies protected: "
        f"{boundary['expected_non_dependency_count']}",
        "- ZIP archive dependency declarations: "
        f"{boundary['zip_dependency_count']}/"
        f"{boundary['expected_zip_dependency_count']}",
        "- tech-stack Rust/static guard anchors: "
        f"{boundary['tech_stack_guard_count'] - len(boundary['missing_tech_stack_guards'])}/"
        f"{boundary['tech_stack_guard_count']}",
        "- behavior test anchors: "
        f"{boundary['behavior_test_anchor_count'] - len(boundary['missing_behavior_test_anchors'])}/"
        f"{boundary['behavior_test_anchor_count']}",
        "- editor-only dependency candidates protected: "
        f"{boundary['editor_only_candidate_count']}",
        "- Jolt visible-unavailable feature slots: "
        f"{boundary['jolt_feature_slot_count']}",
        "- removed/editor-only dependencies declared in manifests: "
        f"{len(boundary['declared_removed_dependencies'])}",
        "- Kira dependency owners: "
        f"{', '.join(boundary['kira_dependency_owners'])}",
        "- Sound runtime Kira 0.12.2 pin: "
        f"{'current' if boundary['kira_owner_version_pinned'] else 'drifted'}",
        "- Sound runtime Kira declarations: "
        f"{boundary['kira_owner_dependency_declaration_count']} "
        f"(versions: {', '.join(version or '<missing>' for version in boundary['kira_owner_dependency_versions'])})",
        "- Rapier/Avian dependencies declared in manifests: "
        f"{len(boundary['rapier_or_avian_dependencies'])}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
        "- ZrVM plugin-owned backend manifest: "
        f"{'present' if boundary['zr_vm_plugin_manifest_present'] else 'missing'}",
        "- ZrVM plugin binding dependencies: "
        f"{boundary['zr_vm_plugin_binding_dependency_count']}",
        "- zircon_runtime concrete ZrVM owner: "
        f"{'absent' if boundary['runtime_zr_vm_owner_absent'] else 'present'}",
    ]

    if boundary["missing_manifest_files"]:
        lines.append(
            "- missing manifest files: "
            f"{', '.join(boundary['missing_manifest_files'])}"
        )
    if boundary["manifest_scan_errors"]:
        lines.append(
            "- product manifest scan errors: "
            f"{', '.join(boundary['manifest_scan_errors'])}"
        )
    if boundary["declared_removed_dependencies"]:
        lines.append(
            "- declared removed/editor-only dependencies: "
            f"{', '.join(boundary['declared_removed_dependencies'])}"
        )
    if boundary["missing_version_anchors"]:
        lines.append(
            "- missing version anchors: "
            f"{', '.join(boundary['missing_version_anchors'])}"
        )
    if boundary["dependency_boundary_violations"]:
        lines.append(
            "- dependency boundary violations: "
            f"{', '.join(boundary['dependency_boundary_violations'])}"
        )
    if boundary["zip_dependency_violations"]:
        lines.append(
            "- ZIP archive dependency violations: "
            f"{', '.join(boundary['zip_dependency_violations'])}"
        )
    if boundary["missing_tech_stack_doc_anchors"]:
        lines.append(
            "- missing tech-stack doc anchors: "
            f"{', '.join(boundary['missing_tech_stack_doc_anchors'])}"
        )
    if boundary["missing_kira_tech_stack_doc_anchors"]:
        lines.append(
            "- missing Kira Sound owner doc anchors: "
            f"{', '.join(boundary['missing_kira_tech_stack_doc_anchors'])}"
        )
    if boundary["kira_owner_violations"]:
        lines.append(
            "- Kira owner violations: "
            f"{', '.join(boundary['kira_owner_violations'])}"
        )
    if boundary["missing_text_doc_anchors"]:
        lines.append(
            "- missing text doc anchors: "
            f"{', '.join(boundary['missing_text_doc_anchors'])}"
        )
    if boundary["missing_physics_decision_anchors"]:
        lines.append(
            "- missing physics decision anchors: "
            f"{', '.join(boundary['missing_physics_decision_anchors'])}"
        )
    if boundary["missing_editor_backlog_anchors"]:
        lines.append(
            "- missing editor backlog anchors: "
            f"{', '.join(boundary['missing_editor_backlog_anchors'])}"
        )
    if boundary["missing_tech_stack_guards"]:
        lines.append(
            "- missing tech-stack guard anchors: "
            f"{', '.join(boundary['missing_tech_stack_guards'])}"
        )
    if boundary["missing_behavior_test_anchors"]:
        lines.append(
            "- missing behavior test anchors: "
            f"{', '.join(boundary['missing_behavior_test_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["rapier_or_avian_dependencies"]:
        lines.append(
            "- Rapier/Avian manifest dependencies: "
            f"{', '.join(boundary['rapier_or_avian_dependencies'])}"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines

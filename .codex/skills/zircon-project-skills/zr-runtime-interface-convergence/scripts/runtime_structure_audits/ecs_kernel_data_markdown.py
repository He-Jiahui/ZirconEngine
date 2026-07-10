from __future__ import annotations


def render_ecs_kernel_data_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    source_files = boundary["source_files"]
    test_files = boundary["test_files"]
    lines = [
        "## Runtime 08 ECS Kernel Data Boundary",
        "- audited ECS data-kernel source files "
        f"({len(source_files)}/{boundary['expected_source_file_count']}): "
        f"{len(source_files)} files",
        "- audited Runtime 08 guard/test files "
        f"({len(test_files)}/{boundary['expected_test_file_count']}): "
        f"{len(test_files)} files",
        "- archetype anchors: "
        f"{boundary['archetype_anchor_count'] - len(boundary['missing_archetype_anchors'])}/"
        f"{boundary['archetype_anchor_count']}",
        "- storage anchors: "
        f"{boundary['storage_anchor_count'] - len(boundary['missing_storage_anchors'])}/"
        f"{boundary['storage_anchor_count']}",
        "- component storage private re-export anchors: "
        f"{boundary['component_storage_private_reexport_anchor_count'] - len(boundary['missing_component_storage_private_reexport_anchors'])}/"
        f"{boundary['component_storage_private_reexport_anchor_count']}",
        "- unexpected component storage private re-exports: "
        f"{len(boundary['unexpected_component_storage_private_reexports'])}",
        "- component identity anchors: "
        f"{boundary['component_identity_anchor_count'] - len(boundary['missing_component_identity_anchors'])}/"
        f"{boundary['component_identity_anchor_count']}",
        "- entity lifecycle anchors: "
        f"{boundary['entity_lifecycle_anchor_count'] - len(boundary['missing_entity_lifecycle_anchors'])}/"
        f"{boundary['entity_lifecycle_anchor_count']}",
        "- observer anchors: "
        f"{boundary['observer_anchor_count'] - len(boundary['missing_observer_anchors'])}/"
        f"{boundary['observer_anchor_count']}",
        "- deferred command anchors: "
        f"{boundary['command_anchor_count'] - len(boundary['missing_command_anchors'])}/"
        f"{boundary['command_anchor_count']}",
        "- event/message anchors: "
        f"{boundary['event_message_anchor_count'] - len(boundary['missing_event_message_anchors'])}/"
        f"{boundary['event_message_anchor_count']}",
        "- change tick anchors: "
        f"{boundary['change_tick_anchor_count'] - len(boundary['missing_change_tick_anchors'])}/"
        f"{boundary['change_tick_anchor_count']}",
        "- resource identity anchors: "
        f"{boundary['resource_identity_anchor_count'] - len(boundary['missing_resource_identity_anchors'])}/"
        f"{boundary['resource_identity_anchor_count']}",
        "- Runtime 08 guard anchors: "
        f"{boundary['test_anchor_count'] - len(boundary['missing_test_anchors'])}/"
        f"{boundary['test_anchor_count']}",
        "- Runtime 08 behavior test anchors: "
        f"{boundary['behavior_test_anchor_count'] - len(boundary['missing_behavior_test_anchors'])}/"
        f"{boundary['behavior_test_anchor_count']}",
        "- Runtime 08 doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/"
        f"{boundary['doc_anchor_count']}",
        "- pending Cargo gate anchors: "
        f"{boundary['cargo_gate_anchor_count'] - len(boundary['missing_cargo_gate_anchors'])}/"
        f"{boundary['cargo_gate_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["missing_source_files"]:
        lines.append(
            "- missing Runtime 08 source files: "
            f"{', '.join(boundary['missing_source_files'])}"
        )
    if boundary["missing_test_files"]:
        lines.append(
            "- missing Runtime 08 guard/test files: "
            f"{', '.join(boundary['missing_test_files'])}"
        )
    if boundary["missing_archetype_anchors"]:
        lines.append(
            "- missing archetype anchors: "
            f"{', '.join(boundary['missing_archetype_anchors'])}"
        )
    if boundary["missing_storage_anchors"]:
        lines.append(
            "- missing storage anchors: "
            f"{', '.join(boundary['missing_storage_anchors'])}"
        )
    if boundary["missing_component_storage_private_reexport_anchors"]:
        lines.append(
            "- missing component storage private re-export anchors: "
            f"{', '.join(boundary['missing_component_storage_private_reexport_anchors'])}"
        )
    if boundary["unexpected_component_storage_private_reexports"]:
        lines.append(
            "- unexpected component storage private re-exports: "
            f"{', '.join(boundary['unexpected_component_storage_private_reexports'])}"
        )
    if boundary["missing_component_identity_anchors"]:
        lines.append(
            "- missing component identity anchors: "
            f"{', '.join(boundary['missing_component_identity_anchors'])}"
        )
    if boundary["missing_entity_lifecycle_anchors"]:
        lines.append(
            "- missing entity lifecycle anchors: "
            f"{', '.join(boundary['missing_entity_lifecycle_anchors'])}"
        )
    if boundary["missing_observer_anchors"]:
        lines.append(
            "- missing observer anchors: "
            f"{', '.join(boundary['missing_observer_anchors'])}"
        )
    if boundary["missing_command_anchors"]:
        lines.append(
            "- missing deferred command anchors: "
            f"{', '.join(boundary['missing_command_anchors'])}"
        )
    if boundary["missing_event_message_anchors"]:
        lines.append(
            "- missing event/message anchors: "
            f"{', '.join(boundary['missing_event_message_anchors'])}"
        )
    if boundary["missing_change_tick_anchors"]:
        lines.append(
            "- missing change tick anchors: "
            f"{', '.join(boundary['missing_change_tick_anchors'])}"
        )
    if boundary["missing_resource_identity_anchors"]:
        lines.append(
            "- missing resource identity anchors: "
            f"{', '.join(boundary['missing_resource_identity_anchors'])}"
        )
    if boundary["missing_test_anchors"]:
        lines.append(
            "- missing Runtime 08 test anchors: "
            f"{', '.join(boundary['missing_test_anchors'])}"
        )
    if boundary["missing_behavior_test_anchors"]:
        lines.append(
            "- missing Runtime 08 behavior test anchors: "
            f"{', '.join(boundary['missing_behavior_test_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 08 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines

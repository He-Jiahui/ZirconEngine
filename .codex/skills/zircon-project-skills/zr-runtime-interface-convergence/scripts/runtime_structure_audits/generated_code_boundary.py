from __future__ import annotations

import re
from pathlib import Path


GENERATED_BEHAVIOR_PATTERNS = {
    "entry-bootstrap": re.compile(r"EntryRunner::|bootstrap_with_runtime_plugin|bootstrap_export_runtime", re.I),
    "native-loader": re.compile(r"NativePluginLoader|load_runtime_from_load_manifest"),
    "plugin-registration": re.compile(
        r"plugin_registration\(\)|plugin_feature_registration\(\)|runtime_plugin_registrations\(\)|runtime_plugin_feature_registrations\(\)"
    ),
    "runtime-selection-function": re.compile(r"pub fn (target_mode|export_profile|project_plugins)\("),
    "generated-main": re.compile(r"fn main\(\)"),
}

GENERATED_BEHAVIOR_DECISIONS = {
    "entry-bootstrap": {
        "classification": "handwritten-owner-required",
        "target_owner": "handwritten export bootstrap owner",
        "allowed_generated_shape": "a thin call into one stable export bootstrap facade",
        "required_action": "move EntryRunner assembly and runtime bootstrap sequencing out of generated templates",
    },
    "generated-main": {
        "classification": "entry-glue-review",
        "target_owner": "handwritten export entry facade",
        "allowed_generated_shape": "minimal main function that forwards to a stable facade",
        "required_action": "remove direct runtime/plugin assembly from generated main scaffolds",
    },
    "native-loader": {
        "classification": "native-loader-isolation",
        "target_owner": "isolated native plugin loader namespace",
        "allowed_generated_shape": "native manifest data or a call into an isolated loader facade",
        "required_action": "move NativePluginLoader use behind the M4 native loader boundary",
    },
    "plugin-registration": {
        "classification": "handwritten-owner-required",
        "target_owner": "runtime-owned plugin catalog or export provider owner",
        "allowed_generated_shape": "provider ids, manifest rows, or registration tables without direct calls",
        "required_action": "replace generated plugin_registration calls with data consumed by a handwritten owner",
    },
    "runtime-selection-function": {
        "classification": "data-adapter-review",
        "target_owner": "generated data table adapter",
        "allowed_generated_shape": "pure DTO/table functions with no lifecycle, loader, or registration side effects",
        "required_action": "keep only data/table construction and split mixed registration behavior away",
    },
}

GENERATED_BEHAVIOR_CLASSIFICATION_ORDER = (
    "handwritten-owner-required",
    "native-loader-isolation",
    "entry-glue-review",
    "data-adapter-review",
    "unclassified-generated-behavior",
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _location(path: str, line: int, snippet: str, max_snippet: int | None = None) -> dict[str, object]:
    if max_snippet is not None and len(snippet) > max_snippet:
        snippet = f"{snippet[:max_snippet].rstrip()}..."
    return {
        "path": path,
        "line": line,
        "snippet": snippet,
    }


def _classify_behavior_location(label: str, snippet: str) -> dict[str, object]:
    is_debt = False
    action = "allowed generated adapter"

    if label == "entry-bootstrap":
        is_debt = "EntryRunner::" in snippet or "bootstrap_with_runtime_plugin" in snippet
        action = (
            "move direct EntryRunner/bootstrap sequencing into zircon_app::entry::export_bootstrap"
            if is_debt
            else "keep only the stable zircon_app::bootstrap_export_runtime facade call"
        )
    elif label == "native-loader":
        is_debt = True
        action = "move NativePluginLoader use behind the app/native-loader facade"
    elif label == "plugin-registration":
        is_debt = True
        action = "replace generated registration execution with app-owned provider-table handoff"
    elif label == "generated-main":
        action = "keep generated main as a minimal call into the app export facade"
    elif label == "runtime-selection-function":
        action = "keep selection/profile functions side-effect-free table adapters"
    else:
        is_debt = True
        action = "classify this generated behavior before accepting the boundary"

    return {
        "status": "migration-debt" if is_debt else "allowed-adapter",
        "requires_migration": is_debt,
        "required_action": action,
    }


def _find_locations(
    root: Path,
    files: list[Path],
    label: str,
    pattern: re.Pattern[str],
    max_snippet: int | None = None,
) -> list[dict[str, object]]:
    results: list[dict[str, object]] = []
    for path in files:
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            if pattern.search(line):
                location = _location(
                    _relative(root, path),
                    line_no,
                    line.strip(),
                    max_snippet=max_snippet,
                )
                location.update(_classify_behavior_location(label, str(location["snippet"])))
                results.append(location)
    return results


def _behavior_decisions(
    behavior_locations: dict[str, list[dict[str, object]]],
) -> list[dict[str, object]]:
    decisions: list[dict[str, object]] = []
    for label in sorted(behavior_locations):
        decision = GENERATED_BEHAVIOR_DECISIONS.get(
            label,
            {
                "classification": "unclassified-generated-behavior",
                "target_owner": "unknown",
                "allowed_generated_shape": "none until classified",
                "required_action": "classify this generated behavior before accepting the boundary",
            },
        )
        decisions.append(
            {
                "label": label,
                "classification": decision["classification"],
                "target_owner": decision["target_owner"],
                "allowed_generated_shape": decision["allowed_generated_shape"],
                "required_action": decision["required_action"],
                "location_count": len(behavior_locations[label]),
                "allowed_adapter_count": sum(
                    1
                    for location in behavior_locations[label]
                    if not bool(location.get("requires_migration"))
                ),
                "migration_debt_count": sum(
                    1
                    for location in behavior_locations[label]
                    if bool(location.get("requires_migration"))
                ),
            }
        )
    return decisions


def _group_behavior_decisions(
    decisions: list[dict[str, object]],
) -> dict[str, list[str]]:
    grouped: dict[str, list[str]] = {
        classification: [] for classification in GENERATED_BEHAVIOR_CLASSIFICATION_ORDER
    }
    for decision in decisions:
        grouped.setdefault(str(decision["classification"]), []).append(str(decision["label"]))
    return {key: sorted(value) for key, value in grouped.items() if value}


def _generated_boundary_migration_debt(
    decisions: list[dict[str, object]],
) -> list[str]:
    return [
        f"{decision['label']}: {decision['required_action']} "
        f"({decision['migration_debt_count']} location(s))"
        for decision in decisions
        if int(decision["migration_debt_count"]) > 0
    ]


def generated_code_boundary_audit(root: Path) -> dict[str, object]:
    export_plan_root = root / "zircon_runtime" / "src" / "plugin" / "export_build_plan"
    if not export_plan_root.exists():
        return {
            "template_files": [],
            "template_file_count": 0,
            "behavior_locations": {},
            "behavior_location_count": 0,
            "allowed_adapter_location_count": 0,
            "migration_debt_location_count": 0,
            "behavior_decisions": [],
            "behavior_decision_count": 0,
            "behavior_decision_groups": {},
            "unclassified_behavior_labels": [],
            "unclassified_behavior_label_count": 0,
            "generated_boundary_migration_debt": [],
            "generated_boundary_migration_debt_count": 0,
            "m1_gate_status": "classified-and-clear",
            "risks": [],
        }

    template_files = sorted(
        path
        for path in export_plan_root.rglob("*.rs")
        if any(
            token in path.name
            for token in (
                "template",
                "generated_files",
                "platform_host_files",
            )
        )
        or "platform_host_files" in path.relative_to(export_plan_root).as_posix()
    )
    behavior_locations: dict[str, list[dict[str, object]]] = {}
    for label, pattern in GENERATED_BEHAVIOR_PATTERNS.items():
        locations = _find_locations(root, template_files, label, pattern, max_snippet=220)
        if locations:
            behavior_locations[label] = locations

    behavior_location_count = sum(len(locations) for locations in behavior_locations.values())
    allowed_adapter_location_count = sum(
        1
        for locations in behavior_locations.values()
        for location in locations
        if not bool(location.get("requires_migration"))
    )
    migration_debt_location_count = sum(
        1
        for locations in behavior_locations.values()
        for location in locations
        if bool(location.get("requires_migration"))
    )
    behavior_decisions = _behavior_decisions(behavior_locations)
    unclassified_behavior_labels = [
        str(decision["label"])
        for decision in behavior_decisions
        if decision["classification"] == "unclassified-generated-behavior"
    ]
    generated_boundary_migration_debt = _generated_boundary_migration_debt(behavior_decisions)
    template_file_paths = [_relative(root, path) for path in template_files]
    risks: list[str] = []
    if migration_debt_location_count > 0:
        risks.append(
            "export source templates generate runtime behavior or architecture-sensitive entry code; "
            "M1/M2 should move behavior into runtime-owned owners and keep generated output to leaf data, tables, or adapters."
        )
    if unclassified_behavior_labels:
        risks.append(
            "generated behavior labels are not classified by the M1 boundary gate: "
            + ", ".join(unclassified_behavior_labels)
        )

    return {
        "template_files": template_file_paths,
        "template_file_count": len(template_file_paths),
        "behavior_locations": {
            label: locations for label, locations in sorted(behavior_locations.items())
        },
        "behavior_location_count": behavior_location_count,
        "allowed_adapter_location_count": allowed_adapter_location_count,
        "migration_debt_location_count": migration_debt_location_count,
        "behavior_decisions": behavior_decisions,
        "behavior_decision_count": len(behavior_decisions),
        "behavior_decision_groups": _group_behavior_decisions(behavior_decisions),
        "unclassified_behavior_labels": unclassified_behavior_labels,
        "unclassified_behavior_label_count": len(unclassified_behavior_labels),
        "generated_boundary_migration_debt": generated_boundary_migration_debt,
        "generated_boundary_migration_debt_count": len(generated_boundary_migration_debt),
        "m1_gate_status": (
            "unclassified-behavior-present"
            if unclassified_behavior_labels
            else "migration-debt-present"
            if generated_boundary_migration_debt
            else "classified-and-clear"
        ),
        "risks": risks,
    }

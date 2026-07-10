from __future__ import annotations


HotspotMap = dict[str, list[tuple[str, int]]]

LARGE_FILE_OWNER_CLASSIFICATION_ORDER = (
    "runtime-module-assembly",
    "runtime-framework-render",
    "runtime-other",
    "editor-retained-host",
    "editor-ui",
    "editor-other",
    "support-hub",
    "support-or-plugin",
)

LARGE_FILE_OWNER_DECISIONS = {
    "runtime-module-assembly": {
        "target_owner": "runtime module/profile/plugin assembly owners",
        "required_action": (
            "keep runtime assembly split by ids, manifests, availability, extension "
            "aggregation, plugin-domain mapping, core module construction, and orchestration"
        ),
    },
    "runtime-framework-render": {
        "target_owner": "runtime framework render DTO/diagnostic owners",
        "required_action": (
            "split render framework DTO, debug snapshot, projection, and diagnostic "
            "records by stable contract owner during the M6 render slice"
        ),
    },
    "runtime-other": {
        "target_owner": "runtime module owner for each hotspot",
        "required_action": (
            "split runtime hotspots by behavior family and public contract before "
            "doing allocation or clone optimization"
        ),
    },
    "editor-retained-host": {
        "target_owner": "editor retained-host workflow owners",
        "required_action": (
            "split painter, lifecycle, native pointer, pane conversion, and presentation "
            "hotspots by host-contract workflow during the M7 editor/UI slice"
        ),
    },
    "editor-ui": {
        "target_owner": "editor UI layout and asset-editor owners",
        "required_action": (
            "split editor UI hotspots by window projection, asset-editor workflow, "
            "and template/runtime ownership after active editor UI work settles"
        ),
    },
    "editor-other": {
        "target_owner": "editor module owner",
        "required_action": (
            "classify the editor hotspot into a concrete authoring workflow before "
            "accepting more behavior in that file"
        ),
    },
    "support-hub": {
        "target_owner": "Hub support application owner",
        "required_action": (
            "coordinate with the active Hub session and split Hub runtime/view-model "
            "hotspots by app state, command, and projection owner"
        ),
    },
    "support-or-plugin": {
        "target_owner": "support or plugin owner",
        "required_action": (
            "classify the support/plugin hotspot with its concrete package owner before "
            "adding more behavior"
        ),
    },
}


def large_file_owner_class(path: str) -> str:
    if path == "zircon_runtime/src/builtin/runtime_modules.rs":
        return "runtime-module-assembly"
    if path.startswith("zircon_runtime/src/core/framework/render/"):
        return "runtime-framework-render"
    if path.startswith("zircon_runtime/"):
        return "runtime-other"
    if path.startswith("zircon_editor/src/ui/retained_host/"):
        return "editor-retained-host"
    if path.startswith("zircon_editor/src/ui/"):
        return "editor-ui"
    if path.startswith("zircon_editor/"):
        return "editor-other"
    if path.startswith("zircon_hub/"):
        return "support-hub"
    return "support-or-plugin"


def large_file_hotspot_entries(
    hotspots: HotspotMap,
    max_hotspots: int = 10,
) -> list[dict[str, object]]:
    all_hotspots = sorted(
        (
            {
                "crate": crate_name,
                "path": path,
                "lines": lines,
                "line_count": lines,
                "owner_class": large_file_owner_class(path),
            }
            for crate_name, entries in hotspots.items()
            for path, lines in entries
        ),
        key=lambda item: item["lines"],
        reverse=True,
    )
    return all_hotspots[:max_hotspots]


def large_file_ownership_classes(
    hotspots: HotspotMap,
) -> dict[str, dict[str, object]]:
    summary: dict[str, dict[str, object]] = {}
    for hotspot in large_file_hotspot_entries(hotspots, max_hotspots=sum(len(v) for v in hotspots.values())):
        owner_class = large_file_owner_class(str(hotspot["path"]))
        entry = summary.setdefault(
            owner_class,
            {
                "count": 0,
                "max_lines": 0,
                "sample_paths": [],
            },
        )
        entry["count"] = int(entry["count"]) + 1
        entry["max_lines"] = max(int(entry["max_lines"]), int(hotspot["lines"]))
        if len(entry["sample_paths"]) < 5:
            entry["sample_paths"].append(hotspot["path"])
    return dict(sorted(summary.items()))


def _all_large_file_hotspots(hotspots: HotspotMap) -> list[dict[str, object]]:
    return large_file_hotspot_entries(
        hotspots,
        max_hotspots=sum(len(entries) for entries in hotspots.values()),
    )


def _large_file_decision_groups(
    hotspots: list[dict[str, object]],
) -> dict[str, list[str]]:
    grouped = {classification: [] for classification in LARGE_FILE_OWNER_CLASSIFICATION_ORDER}
    for hotspot in hotspots:
        owner_class = str(hotspot.get("owner_class") or large_file_owner_class(str(hotspot["path"])))
        grouped.setdefault(owner_class, []).append(
            f"{hotspot['path']}:{hotspot['lines']}"
        )
    return {key: value for key, value in grouped.items() if value}


def _large_file_migration_debt(
    classification_counts: dict[str, int],
) -> list[str]:
    debt: list[str] = []
    for classification in LARGE_FILE_OWNER_CLASSIFICATION_ORDER:
        count = classification_counts.get(classification, 0)
        if not count:
            continue
        decision = LARGE_FILE_OWNER_DECISIONS[classification]
        debt.append(
            f"{classification}: {decision['required_action']} ({count} file(s))"
        )
    return debt


def large_file_ownership_gate(
    hotspots: HotspotMap,
    threshold: int,
) -> dict[str, object]:
    all_hotspots = _all_large_file_hotspots(hotspots)
    ownership_classes = large_file_ownership_classes(hotspots)
    classification_counts = {
        owner_class: int(summary["count"])
        for owner_class, summary in ownership_classes.items()
    }
    decision_groups = _large_file_decision_groups(all_hotspots)
    unclassified_hotspots = [
        hotspot
        for hotspot in all_hotspots
        if str(hotspot["owner_class"]) not in LARGE_FILE_OWNER_DECISIONS
    ]
    migration_debt = _large_file_migration_debt(classification_counts)
    risks: list[str] = []
    if migration_debt:
        risks.append(
            "large production files remain above the owner budget; split by ownership "
            "before adding behavior or doing hot-path optimization."
        )
    if unclassified_hotspots:
        risks.append(
            "large production-file hotspots are not classified by the ownership gate: "
            f"{len(unclassified_hotspots)} file(s)"
        )

    return {
        "threshold": threshold,
        "hotspot_count": len(all_hotspots),
        "hotspots": all_hotspots,
        "m1_gate_status": (
            "migration-debt-present" if migration_debt else "classified-and-clear"
        ),
        "classification_counts": classification_counts,
        "classification_count": len(classification_counts),
        "decision_groups": decision_groups,
        "decision_group_count": len(decision_groups),
        "owner_decisions": {
            owner_class: LARGE_FILE_OWNER_DECISIONS[owner_class]
            for owner_class in LARGE_FILE_OWNER_CLASSIFICATION_ORDER
            if owner_class in classification_counts
        },
        "large_file_migration_debt": migration_debt,
        "large_file_migration_debt_count": len(migration_debt),
        "unclassified_hotspots": unclassified_hotspots,
        "unclassified_hotspot_count": len(unclassified_hotspots),
        "risks": risks,
    }

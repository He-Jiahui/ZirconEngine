from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path


MIGRATION_TOKEN_RE = re.compile(r"\b(legacy|compat|shim|bridge)\b", re.I)
MIGRATION_BRIDGE_CONTEXT_RE = re.compile(
    r"\b(legacy|compat|shim|deprecated|compatibility|forwarding)\b",
    re.I,
)

HARD_CUTOVER_CLASSIFICATION_ORDER = (
    "hard-cutover-compat-shim-blocker",
    "migration-bridge-smell-blocker",
    "legacy-runtime-ui-input-debt",
    "legacy-runtime-graphics-debt",
    "legacy-runtime-ui-template-debt",
    "legacy-runtime-asset-debt",
    "legacy-hub-message-archived-text-debt",
    "legacy-texture-importer-dds-debt",
    "legacy-net-hyper-client-api-debt",
    "external-hyper-http1-client-policy",
    "legacy-runtime-scene-document-debt",
    "legacy-editor-ui-fixture-debt",
    "legacy-runtime-ui-layout-debt",
    "legacy-runtime-interface-diagnostics-debt",
    "editor-typography-point-as-pixel-migration-policy",
    "unclassified-hard-cutover-smell",
)

HARD_CUTOVER_CLASSIFICATION_DECISIONS = {
    "hard-cutover-compat-shim-blocker": {
        "target_owner": "hard-cutover migration owner",
        "required_action": (
            "remove compatibility or shim wording and cut callers directly to the "
            "current owner path"
        ),
    },
    "migration-bridge-smell-blocker": {
        "target_owner": "hard-cutover migration owner",
        "required_action": (
            "delete migration-only bridge wording or replace it with the real "
            "business owner term"
        ),
    },
    "legacy-runtime-ui-input-debt": {
        "target_owner": "runtime UI input dispatch route owner",
        "required_action": (
            "rename legacy dispatch route variables and helper wording to current "
            "pointer routing terminology during the runtime UI input slice"
        ),
    },
    "legacy-runtime-graphics-debt": {
        "target_owner": "runtime graphics/render extraction owner",
        "required_action": (
            "rename legacy graphics feature, viewport packet, and render-product "
            "test fixture wording during the M6 graphics/RHI slice"
        ),
    },
    "legacy-runtime-ui-template-debt": {
        "target_owner": "runtime/editor UI template schema owner",
        "required_action": (
            "finish the UI template schema cutover and remove legacy schema/cache "
            "wording from runtime and editor template paths"
        ),
    },
    "legacy-runtime-asset-debt": {
        "target_owner": "runtime asset import and asset schema owner",
        "required_action": (
            "replace legacy asset fallback/import wording with explicit versioned "
            "schema migration or remove the fallback path"
        ),
    },
    "legacy-hub-message-archived-text-debt": {
        "target_owner": "Hub message archived-text compatibility owner",
        "required_action": (
            "rename HubMessage legacy text constructors and fixtures to explicit "
            "archived/raw text policy terminology during the Hub support slice"
        ),
    },
    "legacy-texture-importer-dds-debt": {
        "target_owner": "texture importer DDS container owner",
        "required_action": (
            "replace legacy DDS container wording with explicit DDS caps policy "
            "or delete the old parser path"
        ),
    },
    "legacy-net-hyper-client-api-debt": {
        "target_owner": "Net plugin HTTP backend dependency owner",
        "required_action": (
            "wrap or rename the third-party hyper legacy client path as an explicit "
            "HTTP backend policy when the Net plugin backend is next touched"
        ),
    },
    "external-hyper-http1-client-policy": {
        "target_owner": "Net plugin HTTP backend dependency owner",
        "required_action": (
            "keep the third-party Hyper HTTP/1 client API path isolated behind the "
            "explicit Net HTTP backend policy owner"
        ),
    },
    "legacy-runtime-scene-document-debt": {
        "target_owner": "runtime scene dynamic document owner",
        "required_action": (
            "rename legacy dynamic project document modules to explicit archived "
            "or v1 project document policy terminology during the Runtime 05 scene "
            "serialization cutover"
        ),
    },
    "legacy-editor-ui-fixture-debt": {
        "target_owner": "editor retained-host fixture and UI projection owner",
        "required_action": (
            "rename stale legacy/deprecated UI fixture labels when the retained-host "
            "or view projection owner is next touched"
        ),
    },
    "legacy-runtime-ui-layout-debt": {
        "target_owner": "runtime UI layout pass owner",
        "required_action": (
            "rename the stale wrap/layout compatibility comment to the current Flow "
            "slot behavior"
        ),
    },
    "legacy-runtime-interface-diagnostics-debt": {
        "target_owner": "runtime interface diagnostics compatibility owner",
        "required_action": (
            "replace legacy diagnostic stage names with an explicit archived-format "
            "version policy or remove the compatibility parse path"
        ),
    },
    "editor-typography-point-as-pixel-migration-policy": {
        "target_owner": "editor typography preference migration owner",
        "required_action": (
            "keep the one-way point-as-pixel persisted-value upgrade isolated in "
            "the editor preference migration leaf; do not expose an alias API"
        ),
    },
    "unclassified-hard-cutover-smell": {
        "target_owner": "unknown",
        "required_action": (
            "classify this migration-smell reference with an owner or remove it "
            "before accepting the boundary"
        ),
    },
}

HARD_CUTOVER_ALLOWED_CLASSIFICATIONS = {
    "external-hyper-http1-client-policy",
    "editor-typography-point-as-pixel-migration-policy",
}


@dataclass
class HardCutoverReferenceDecision:
    path: str
    line: int
    snippet: str
    term: str
    classification: str
    target_owner: str
    required_action: str


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _is_production_rust_file(root: Path, path: Path) -> bool:
    relative_parts = _relative(root, path).split("/")
    return (
        "tests" not in relative_parts
        and "target" not in relative_parts
        and path.name != "tests.rs"
        and not path.name.endswith("_tests.rs")
    )


def _top_level_zircon_src_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for crate_root in root.iterdir():
        src_root = crate_root / "src"
        if (
            crate_root.is_dir()
            and crate_root.name.startswith("zircon_")
            and src_root.exists()
        ):
            files.extend(src_root.rglob("*.rs"))
    return files


def _plugin_runtime_src_files(root: Path) -> list[Path]:
    plugin_root = root / "zircon_plugins"
    if not plugin_root.exists():
        return []
    files: list[Path] = []
    for src_root in plugin_root.rglob("src"):
        if src_root.is_dir():
            files.extend(src_root.rglob("*.rs"))
    return files


def _production_rust_files(root: Path) -> list[Path]:
    return sorted(
        {
            path
            for path in [*_top_level_zircon_src_files(root), *_plugin_runtime_src_files(root)]
            if _is_production_rust_file(root, path)
        }
    )


def _production_source_lines(source: str) -> list[tuple[int, str]]:
    """Return lines outside items explicitly compiled only for tests."""
    lines: list[tuple[int, str]] = []
    pending_test_cfg = False
    skipped_item_depth = 0

    for line_no, line in enumerate(source.splitlines(), start=1):
        stripped = line.strip()

        if skipped_item_depth:
            skipped_item_depth += line.count("{") - line.count("}")
            if skipped_item_depth <= 0:
                skipped_item_depth = 0
            continue

        if stripped == "#[cfg(test)]":
            pending_test_cfg = True
            continue

        if pending_test_cfg:
            if stripped.startswith("#["):
                continue
            item_depth = line.count("{") - line.count("}")
            if item_depth > 0:
                skipped_item_depth = item_depth
            pending_test_cfg = False
            continue

        lines.append((line_no, line))

    return lines


def _classify_reference(relative_path: str, line: str, term: str) -> str:
    normalized = relative_path.replace("\\", "/")
    lower_term = term.lower()
    if lower_term in {"compat", "shim"}:
        return "hard-cutover-compat-shim-blocker"
    if lower_term == "bridge":
        if MIGRATION_BRIDGE_CONTEXT_RE.search(line):
            return "migration-bridge-smell-blocker"
        return "allowed-business-bridge-reference"
    if normalized.startswith("zircon_runtime/src/ui/surface/input/"):
        return "legacy-runtime-ui-input-debt"
    if normalized.startswith("zircon_plugins/texture_importer/"):
        return "legacy-texture-importer-dds-debt"
    if normalized.startswith("zircon_runtime/src/graphics/") or normalized.startswith(
        "zircon_runtime/src/core/framework/render/"
    ):
        return "legacy-runtime-graphics-debt"
    if normalized.startswith("zircon_runtime/src/asset/"):
        return "legacy-runtime-asset-debt"
    if normalized.startswith("zircon_hub/src/"):
        return "legacy-hub-message-archived-text-debt"
    if normalized == "zircon_plugins/net/features/http/runtime/src/backend/client.rs":
        return "legacy-net-hyper-client-api-debt"
    if normalized == "zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs":
        return "external-hyper-http1-client-policy"
    if normalized.startswith("zircon_runtime/src/scene/dynamic_scene/document/"):
        return "legacy-runtime-scene-document-debt"
    if (
        normalized.startswith("zircon_runtime/src/ui/template/")
        or normalized.startswith("zircon_editor/src/ui/template_runtime/")
        or normalized.startswith("zircon_editor/src/ui/asset_editor/")
    ):
        return "legacy-runtime-ui-template-debt"
    if normalized.startswith("zircon_editor/src/ui/retained_host/") or normalized.startswith(
        "zircon_editor/src/ui/layouts/"
    ):
        return "legacy-editor-ui-fixture-debt"
    if normalized.startswith("zircon_runtime/src/ui/layout/"):
        return "legacy-runtime-ui-layout-debt"
    if normalized == "zircon_runtime_interface/src/ui/pipeline/stage.rs":
        return "legacy-runtime-interface-diagnostics-debt"
    if normalized == "zircon_editor/src/ui/preferences/typography_migration.rs":
        return "editor-typography-point-as-pixel-migration-policy"
    return "unclassified-hard-cutover-smell"


def _decision(
    relative_path: str,
    line_no: int,
    line: str,
    term: str,
) -> HardCutoverReferenceDecision:
    classification = _classify_reference(relative_path, line, term)
    decision = HARD_CUTOVER_CLASSIFICATION_DECISIONS[classification]
    return HardCutoverReferenceDecision(
        path=relative_path,
        line=line_no,
        snippet=line.strip(),
        term=term.lower(),
        classification=classification,
        target_owner=decision["target_owner"],
        required_action=decision["required_action"],
    )


def _classification_counts(
    decisions: list[HardCutoverReferenceDecision],
) -> dict[str, int]:
    counts = {classification: 0 for classification in HARD_CUTOVER_CLASSIFICATION_ORDER}
    for decision in decisions:
        counts[decision.classification] = counts.get(decision.classification, 0) + 1
    return {key: value for key, value in counts.items() if value}


def _group_decisions(
    decisions: list[HardCutoverReferenceDecision],
) -> dict[str, list[str]]:
    grouped = {classification: [] for classification in HARD_CUTOVER_CLASSIFICATION_ORDER}
    for decision in decisions:
        grouped.setdefault(decision.classification, []).append(
            f"{decision.path}:{decision.line}"
        )
    return {key: value for key, value in grouped.items() if value}


def _classification_samples(
    decisions: list[HardCutoverReferenceDecision],
    max_samples: int = 5,
) -> dict[str, list[dict[str, object]]]:
    samples: dict[str, list[dict[str, object]]] = {}
    for decision in decisions:
        bucket = samples.setdefault(decision.classification, [])
        if len(bucket) < max_samples:
            bucket.append(asdict(decision))
    return samples


def _migration_debt(classification_counts: dict[str, int]) -> list[str]:
    debt: list[str] = []
    for classification in HARD_CUTOVER_CLASSIFICATION_ORDER:
        count = classification_counts.get(classification, 0)
        if not count:
            continue
        if classification in HARD_CUTOVER_ALLOWED_CLASSIFICATIONS:
            continue
        decision = HARD_CUTOVER_CLASSIFICATION_DECISIONS[classification]
        debt.append(
            f"{classification}: {decision['required_action']} ({count} location(s))"
        )
    return debt


def hard_cutover_migration_smells_audit(
    root: Path,
    max_allowed_bridge_samples: int = 8,
) -> dict[str, object]:
    files = _production_rust_files(root)
    term_counts = {
        "legacy": 0,
        "compat": 0,
        "shim": 0,
        "bridge": 0,
    }
    decisions: list[HardCutoverReferenceDecision] = []
    allowed_business_bridge_count = 0
    allowed_business_bridge_references: list[dict[str, object]] = []
    migration_bridge_smell_count = 0

    for path in files:
        relative_path = _relative(root, path)
        for line_no, line in _production_source_lines(_read_text(path)):
            terms = sorted({match.group(1).lower() for match in MIGRATION_TOKEN_RE.finditer(line)})
            for term in terms:
                term_counts[term] = term_counts.get(term, 0) + 1
                classification = _classify_reference(relative_path, line, term)
                if classification == "allowed-business-bridge-reference":
                    allowed_business_bridge_count += 1
                    if len(allowed_business_bridge_references) < max_allowed_bridge_samples:
                        allowed_business_bridge_references.append(
                            {
                                "path": relative_path,
                                "line": line_no,
                                "snippet": line.strip(),
                                "term": term,
                                "classification": classification,
                            }
                        )
                    continue
                if classification == "migration-bridge-smell-blocker":
                    migration_bridge_smell_count += 1
                decisions.append(_decision(relative_path, line_no, line, term))

    classification_counts = _classification_counts(decisions)
    smell_decision_groups = _group_decisions(decisions)
    classification_samples = _classification_samples(decisions)
    unclassified_locations = [
        asdict(decision)
        for decision in decisions
        if decision.classification == "unclassified-hard-cutover-smell"
    ]
    migration_debt = _migration_debt(classification_counts)
    risks: list[str] = []
    if term_counts["compat"] or term_counts["shim"]:
        risks.append(
            "compat or shim wording exists in production Rust files; hard-cutover "
            "migration slices should remove these instead of preserving old paths."
        )
    if migration_bridge_smell_count:
        risks.append(
            "migration-context bridge wording exists in production Rust files and "
            "should be removed or renamed to the owning business concept."
        )
    non_allowed_legacy_count = sum(
        1
        for decision in decisions
        if decision.term == "legacy"
        and decision.classification not in HARD_CUTOVER_ALLOWED_CLASSIFICATIONS
    )
    if non_allowed_legacy_count:
        risks.append(
            "legacy wording remains in production Rust files; each owner group must "
            "decide whether to delete the old behavior or rename it to an explicit "
            "versioned schema/diagnostic policy."
        )
    if unclassified_locations:
        risks.append(
            "hard-cutover migration-smell references are not classified by the gate: "
            f"{len(unclassified_locations)} location(s)"
        )

    return {
        "source_file_count": len(files),
        "legacy_reference_count": term_counts["legacy"],
        "compat_reference_count": term_counts["compat"],
        "shim_reference_count": term_counts["shim"],
        "bridge_reference_count": term_counts["bridge"],
        "allowed_business_bridge_reference_count": allowed_business_bridge_count,
        "allowed_business_bridge_samples": allowed_business_bridge_references,
        "migration_bridge_smell_count": migration_bridge_smell_count,
        "smell_decisions": [asdict(decision) for decision in decisions],
        "smell_decision_count": len(decisions),
        "smell_decision_groups": smell_decision_groups,
        "smell_decision_group_count": len(smell_decision_groups),
        "classification_counts": classification_counts,
        "classification_count": len(classification_counts),
        "classification_samples": classification_samples,
        "unclassified_locations": unclassified_locations,
        "unclassified_location_count": len(unclassified_locations),
        "hard_cutover_migration_debt": migration_debt,
        "hard_cutover_migration_debt_count": len(migration_debt),
        "hard_cutover_gate_status": (
            "migration-debt-present" if migration_debt else "classified-and-clear"
        ),
        "risks": risks,
    }

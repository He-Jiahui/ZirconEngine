from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path


EDITOR_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_]*editor[A-Za-z0-9_]*\b", re.I)
LEGACY_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_]*legacy[A-Za-z0-9_]*\b", re.I)

NAMING_CLASSIFICATION_ORDER = (
    "test-fixture",
    "runtime-profile-editor-host-target",
    "dynamic-api-editor-host-mode",
    "runtime-ui-component-catalog-editor-controls",
    "runtime-ui-template-editor-profile",
    "runtime-asset-editor-metadata",
    "framework-editor-facing-descriptor",
    "graphics-editor-facing-metadata",
    "platform-editor-target-diagnostic",
    "rhi-editor-surface-label",
    "scene-reflection-editor-visible-metadata",
    "curated-runtime-facade-editor-reference",
    "legacy-runtime-ui-input-debt",
    "legacy-runtime-ui-render-table-debt",
    "legacy-runtime-graphics-debt",
    "legacy-runtime-dds-container-policy",
    "legacy-runtime-ui-template-schema-debt",
    "legacy-runtime-ui-layout-debt",
    "legacy-runtime-input-event-debt",
    "legacy-runtime-asset-schema-debt",
    "legacy-dynamic-api-migration-debt",
    "legacy-scene-schema-render-debt",
    "curated-runtime-facade-legacy-reference",
    "unclassified-runtime-naming-reference",
)

NAMING_CLASSIFICATION_DECISIONS = {
    "test-fixture": {
        "target_owner": "runtime tests",
        "required_action": "test-only naming is allowed by file boundary",
    },
    "runtime-profile-editor-host-target": {
        "target_owner": "runtime profile and plugin target mode owner",
        "required_action": "keep as explicit editor-host target/profile vocabulary",
    },
    "dynamic-api-editor-host-mode": {
        "target_owner": "dynamic API runtime-session profile owner",
        "required_action": "keep as explicit editor-host session mode vocabulary",
    },
    "runtime-ui-component-catalog-editor-controls": {
        "target_owner": "runtime UI component catalog owner",
        "required_action": (
            "keep as component-catalog editor/control metadata; do not store scene "
            "authoring state here"
        ),
    },
    "runtime-ui-template-editor-profile": {
        "target_owner": "runtime UI compiled-asset profile owner",
        "required_action": "keep as UI asset compiler profile vocabulary",
    },
    "runtime-asset-editor-metadata": {
        "target_owner": "runtime asset metadata owner",
        "required_action": (
            "keep only asset authoring metadata or display labels; move behavior to "
            "editor-owned code"
        ),
    },
    "framework-editor-facing-descriptor": {
        "target_owner": "runtime framework descriptor and diagnostics owner",
        "required_action": "keep only neutral descriptors consumed by editor/tooling",
    },
    "graphics-editor-facing-metadata": {
        "target_owner": "graphics renderer data and diagnostics owner",
        "required_action": "keep as metadata/reporting until the graphics/RHI slice",
    },
    "platform-editor-target-diagnostic": {
        "target_owner": "platform capability matrix owner",
        "required_action": "keep as runtime target-mode diagnostic vocabulary",
    },
    "rhi-editor-surface-label": {
        "target_owner": "RHI UI surface resource-label owner",
        "required_action": "keep as resource label until the UI/RHI resource naming slice",
    },
    "scene-reflection-editor-visible-metadata": {
        "target_owner": "scene reflection and inspection metadata owner",
        "required_action": (
            "keep only neutral reflection metadata such as visibility hints; editor "
            "authoring state remains forbidden in serialization"
        ),
    },
    "curated-runtime-facade-editor-reference": {
        "target_owner": "runtime facade or diagnostic owner",
        "required_action": "keep as curated facade/doc or diagnostic channel vocabulary",
    },
    "legacy-runtime-ui-input-debt": {
        "target_owner": "runtime UI input dispatch route owner",
        "required_action": "rename legacy input route variables during the runtime UI input slice",
    },
    "legacy-runtime-ui-render-table-debt": {
        "target_owner": "runtime UI collection row render owner",
        "required_action": "rename table text split fallback away from legacy terminology",
    },
    "legacy-runtime-graphics-debt": {
        "target_owner": "runtime graphics/render extraction owner",
        "required_action": "rename legacy render masks and feature fixtures during the graphics slice",
    },
    "legacy-runtime-dds-container-policy": {
        "target_owner": "texture importer DDS container owner",
        "required_action": "replace legacy DDS wording with explicit DDS capability policy",
    },
    "legacy-runtime-ui-template-schema-debt": {
        "target_owner": "runtime UI template schema owner",
        "required_action": "finish schema migration naming and remove legacy template wording",
    },
    "legacy-runtime-ui-layout-debt": {
        "target_owner": "runtime UI layout pass owner",
        "required_action": "rename the remaining legacy layout backend vocabulary",
    },
    "legacy-runtime-input-event-debt": {
        "target_owner": "runtime input event owner",
        "required_action": "rename legacy input delta helpers when the input route is cut over",
    },
    "legacy-runtime-asset-schema-debt": {
        "target_owner": "runtime asset schema owner",
        "required_action": "replace legacy asset defaults with explicit versioned migration names",
    },
    "legacy-dynamic-api-migration-debt": {
        "target_owner": "dynamic API input adapter owner",
        "required_action": "rename legacy input adapter helper after the current route stabilizes",
    },
    "legacy-scene-schema-render-debt": {
        "target_owner": "runtime scene schema/render owner",
        "required_action": "replace legacy scene schema/render names with explicit version names",
    },
    "curated-runtime-facade-legacy-reference": {
        "target_owner": "runtime facade/accessibility owner",
        "required_action": "keep temporarily as assistive/facade debt with owner classification",
    },
    "unclassified-runtime-naming-reference": {
        "target_owner": "unknown",
        "required_action": (
            "classify this editor/legacy reference with an owner or remove it before "
            "accepting the runtime naming boundary"
        ),
    },
}


@dataclass
class RuntimeNamingReferenceDecision:
    path: str
    line: int
    snippet: str
    term: str
    tokens: list[str]
    classification: str
    target_owner: str
    required_action: str


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _is_test_path(relative_path: str) -> bool:
    parts = relative_path.split("/")
    name = parts[-1]
    return "tests" in parts or name == "tests.rs" or name.endswith("_tests.rs")


def _runtime_source_files(root: Path) -> list[Path]:
    runtime_src = root / "zircon_runtime" / "src"
    if not runtime_src.exists():
        return []
    return sorted(runtime_src.rglob("*.rs"))


def _classify_editor_reference(relative_path: str) -> str:
    if _is_test_path(relative_path):
        return "test-fixture"
    if relative_path == "zircon_runtime/src/core/runtime/lifecycle.rs":
        return "runtime-profile-editor-host-target"
    if relative_path.startswith("zircon_runtime/src/plugin/") or relative_path.startswith(
        "zircon_runtime/src/builtin/runtime_modules/"
    ):
        return "runtime-profile-editor-host-target"
    if relative_path.startswith("zircon_runtime/src/dynamic_api/"):
        return "dynamic-api-editor-host-mode"
    if relative_path.startswith(
        "zircon_runtime/src/ui/component/catalog/"
    ) or relative_path.startswith(
        "zircon_runtime/src/ui/component/state_reducer/"
    ) or relative_path.startswith("zircon_runtime/src/ui/v2/surface_tree/"):
        return "runtime-ui-component-catalog-editor-controls"
    if relative_path.startswith("zircon_runtime/src/ui/template/"):
        return "runtime-ui-template-editor-profile"
    if relative_path.startswith("zircon_runtime/src/asset/"):
        return "runtime-asset-editor-metadata"
    if relative_path.startswith("zircon_runtime/src/core/framework/") or relative_path.startswith(
        "zircon_runtime/src/core/runtime/diagnostics/"
    ):
        return "framework-editor-facing-descriptor"
    if relative_path.startswith("zircon_runtime/src/graphics/"):
        return "graphics-editor-facing-metadata"
    if relative_path.startswith("zircon_runtime/src/platform/"):
        return "platform-editor-target-diagnostic"
    if relative_path.startswith("zircon_runtime/src/rhi"):
        return "rhi-editor-surface-label"
    if relative_path.startswith(
        "zircon_runtime/src/scene/reflect/"
    ) or relative_path.startswith("zircon_runtime/src/scene/inspection/"):
        return "scene-reflection-editor-visible-metadata"
    if relative_path in {
        "zircon_runtime/src/diagnostic_log/sink.rs",
        "zircon_runtime/src/prelude.rs",
    }:
        return "curated-runtime-facade-editor-reference"
    return "unclassified-runtime-naming-reference"


def _classify_legacy_reference(relative_path: str) -> str:
    if _is_test_path(relative_path):
        return "test-fixture"
    if (
        relative_path.startswith("zircon_runtime/src/ui/surface/input/")
        or relative_path == "zircon_runtime/src/ui/surface/property_mutation.rs"
        or relative_path == "zircon_runtime/src/ui/surface/surface/default_interactions.rs"
    ):
        return "legacy-runtime-ui-input-debt"
    if relative_path == "zircon_runtime/src/ui/surface/render/collection_rows/table.rs":
        return "legacy-runtime-ui-render-table-debt"
    if relative_path.startswith("zircon_runtime/src/graphics/") or relative_path.startswith(
        "zircon_runtime/src/core/framework/render/"
    ):
        return "legacy-runtime-graphics-debt"
    if relative_path == "zircon_runtime/src/asset/assets/texture/upload_support/dds.rs":
        return "legacy-runtime-dds-container-policy"
    if relative_path.startswith("zircon_runtime/src/ui/template/"):
        return "legacy-runtime-ui-template-schema-debt"
    if relative_path.startswith("zircon_runtime/src/ui/layout/"):
        return "legacy-runtime-ui-layout-debt"
    if relative_path.startswith("zircon_runtime/src/input/") or relative_path.startswith(
        "zircon_runtime/src/core/framework/input/"
    ):
        return "legacy-runtime-input-event-debt"
    if relative_path.startswith("zircon_runtime/src/asset/"):
        return "legacy-runtime-asset-schema-debt"
    if relative_path.startswith("zircon_runtime/src/dynamic_api/"):
        return "legacy-dynamic-api-migration-debt"
    if relative_path.startswith("zircon_runtime/src/scene/"):
        return "legacy-scene-schema-render-debt"
    if relative_path in {
        "zircon_runtime/src/prelude.rs",
        "zircon_runtime/src/ui/accessibility/extract.rs",
    }:
        return "curated-runtime-facade-legacy-reference"
    return "unclassified-runtime-naming-reference"


def _classification_counts(
    decisions: list[RuntimeNamingReferenceDecision],
) -> dict[str, int]:
    counts = {classification: 0 for classification in NAMING_CLASSIFICATION_ORDER}
    for decision in decisions:
        counts[decision.classification] = counts.get(decision.classification, 0) + 1
    return {key: value for key, value in counts.items() if value}


def _group_decisions(
    decisions: list[RuntimeNamingReferenceDecision],
) -> dict[str, list[str]]:
    grouped = {classification: [] for classification in NAMING_CLASSIFICATION_ORDER}
    for decision in decisions:
        grouped.setdefault(decision.classification, []).append(
            f"{decision.path}:{decision.line}"
        )
    return {key: value for key, value in grouped.items() if value}


def _classification_samples(
    decisions: list[RuntimeNamingReferenceDecision],
    max_samples: int = 5,
) -> dict[str, list[dict[str, object]]]:
    samples: dict[str, list[dict[str, object]]] = {}
    for decision in decisions:
        bucket = samples.setdefault(decision.classification, [])
        if len(bucket) < max_samples:
            bucket.append(asdict(decision))
    return samples


def _migration_debt(
    classification_counts: dict[str, int],
) -> list[str]:
    debt: list[str] = []
    for classification in NAMING_CLASSIFICATION_ORDER:
        count = classification_counts.get(classification, 0)
        if not count:
            continue
        if classification in {
            "test-fixture",
            "runtime-profile-editor-host-target",
            "dynamic-api-editor-host-mode",
            "runtime-ui-component-catalog-editor-controls",
            "runtime-ui-template-editor-profile",
            "runtime-asset-editor-metadata",
            "framework-editor-facing-descriptor",
            "graphics-editor-facing-metadata",
            "platform-editor-target-diagnostic",
            "rhi-editor-surface-label",
            "scene-reflection-editor-visible-metadata",
            "curated-runtime-facade-editor-reference",
            "curated-runtime-facade-legacy-reference",
        }:
            continue
        decision = NAMING_CLASSIFICATION_DECISIONS[classification]
        debt.append(
            f"{classification}: {decision['required_action']} ({count} location(s))"
        )
    return debt


def _decisions_for_term(
    root: Path,
    pattern: re.Pattern[str],
    term: str,
) -> list[RuntimeNamingReferenceDecision]:
    decisions: list[RuntimeNamingReferenceDecision] = []
    for path in _runtime_source_files(root):
        relative_path = _relative(root, path)
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            tokens = sorted({match.group(0) for match in pattern.finditer(line)})
            if not tokens:
                continue
            classification = (
                _classify_editor_reference(relative_path)
                if term == "editor"
                else _classify_legacy_reference(relative_path)
            )
            decision = NAMING_CLASSIFICATION_DECISIONS[classification]
            decisions.append(
                RuntimeNamingReferenceDecision(
                    path=relative_path,
                    line=line_no,
                    snippet=line.strip(),
                    term=term,
                    tokens=tokens,
                    classification=classification,
                    target_owner=decision["target_owner"],
                    required_action=decision["required_action"],
                )
            )
    return decisions


def _term_report(decisions: list[RuntimeNamingReferenceDecision]) -> dict[str, object]:
    classification_counts = _classification_counts(decisions)
    unclassified = [
        decision
        for decision in decisions
        if decision.classification == "unclassified-runtime-naming-reference"
    ]
    return {
        "location_count": len(decisions),
        "file_count": len({decision.path for decision in decisions}),
        "classification_counts": classification_counts,
        "classification_groups": _group_decisions(decisions),
        "classification_samples": _classification_samples(decisions),
        "migration_debt": _migration_debt(classification_counts),
        "migration_debt_count": len(_migration_debt(classification_counts)),
        "unclassified_locations": [asdict(decision) for decision in unclassified],
        "unclassified_location_count": len(unclassified),
        "gate_status": "blocked" if unclassified else "classified",
    }


def runtime_naming_boundary_audit(root: Path) -> dict[str, object]:
    editor_decisions = _decisions_for_term(root, EDITOR_TOKEN_RE, "editor")
    legacy_decisions = _decisions_for_term(root, LEGACY_TOKEN_RE, "legacy")
    editor_report = _term_report(editor_decisions)
    legacy_report = _term_report(legacy_decisions)
    return {
        "editor": editor_report,
        "legacy": legacy_report,
        "gate_status": (
            "blocked"
            if editor_report["unclassified_location_count"]
            or legacy_report["unclassified_location_count"]
            else "classified"
        ),
    }

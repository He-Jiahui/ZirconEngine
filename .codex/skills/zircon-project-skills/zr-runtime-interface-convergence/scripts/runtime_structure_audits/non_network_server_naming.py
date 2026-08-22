from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


SERVER_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_]*server[A-Za-z0-9_]*\b", re.I)
UNC_SERVER_PLACEHOLDER_RE = re.compile(
    r"\\{2,4}(?:\?\\{1,2}unc\\{1,2})?server\\{1,2}share",
    re.I,
)
SERVER_RUNTIME_TOKEN_RE = re.compile(r"\bserver_?runtime\b", re.I)
RUNTIME_PROFILE_SERVER_RE = re.compile(r"\bruntimeprofileid::server\b", re.I)
TARGET_SERVER_RE = re.compile(r"\btarget[-_]server\b", re.I)
HEADLESS_SERVER_RE = re.compile(r"\bheadless(?:_|\s+)server\b", re.I)
DEDICATED_SERVER_RE = re.compile(r"\b(?:dedicatedserver|listenserver)\b", re.I)
SERVER_CLIENT_TARGETS_RE = re.compile(r"\bserver_client_targets\b", re.I)
QUOTED_SERVER_RE = re.compile(r'"server"', re.I)
SERVER_CONFIG_RE = re.compile(r"\b(?:server config|dev server|server:)", re.I)
SERVER_HEADLESS_POLICY_RE = re.compile(
    r"\b(?:server/headless|server or headless|server runtime)\b",
    re.I,
)
RUNTIME_TARGET_SERVER_LABEL_RE = re.compile(
    r'\bruntimetargetmode::serverruntime\s*=>\s*"server"',
    re.I,
)
SCENE_SERVER_RE = re.compile(r"\bscene\s+server\b", re.I)

SERVER_REFERENCE_CLASSIFICATION_ORDER = (
    "editor-asset-resource-owner-debt",
    "editor-scene-comment-debt",
    "unclassified-non-network-server",
)

SERVER_REFERENCE_CLASSIFICATION_DECISIONS = {
    "editor-asset-resource-owner-debt": {
        "target_owner": "editor asset/resource access facade",
        "required_action": (
            "rename editor asset/resource `*_server` fields and parameters to manager, "
            "catalog, or access facade terminology during the M7 editor/UI or asset slice"
        ),
    },
    "editor-scene-comment-debt": {
        "target_owner": "editor scene/runtime inspection boundary documentation",
        "required_action": (
            "remove stale `scene server` wording from editor comments when the touched "
            "editor state file is next edited"
        ),
    },
    "unclassified-non-network-server": {
        "target_owner": "unknown",
        "required_action": (
            "classify this server reference with an owner or remove the non-network "
            "server naming before accepting the boundary"
        ),
    },
}


@dataclass
class Location:
    path: str
    line: int
    snippet: str


@dataclass
class ServerReferenceDecision:
    path: str
    line: int
    snippet: str
    tokens: list[str]
    classification: str
    target_owner: str
    required_action: str


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _token_has_server_component(token: str) -> bool:
    lower = token.lower()
    for match in re.finditer("server", lower):
        # `observer` contains the letters "server" but is a separate ECS/UI term.
        if match.start() >= 2 and lower[match.start() - 2 : match.end()] == "observer":
            continue
        if match.start() >= 2 and lower[match.start() - 2 : match.end() + 1] == "observers":
            continue
        return True
    return False


def _server_tokens(line: str) -> list[str]:
    return [
        match.group(0)
        for match in SERVER_TOKEN_RE.finditer(line)
        if _token_has_server_component(match.group(0))
    ]


def _is_test_owned_source(path: Path) -> bool:
    parts = path.as_posix().split("/")
    return (
        "tests" in parts
        or "test_sources" in parts
        or path.name == "tests.rs"
        or path.name.endswith("_tests.rs")
    )


def _token_is_inside_pattern(
    token_match: re.Match[str],
    line: str,
    pattern: re.Pattern[str],
) -> bool:
    return any(
        context.start() <= token_match.start() and token_match.end() <= context.end()
        for context in pattern.finditer(line)
    )


def _is_allowed_server_token(
    path: Path,
    line: str,
    token_match: re.Match[str],
) -> bool:
    normalized = path.as_posix()
    if "/net/" in normalized or "/network" in normalized or "/net_features/" in normalized:
        return True
    if any(
        _token_is_inside_pattern(token_match, line, pattern)
        for pattern in (
            SERVER_RUNTIME_TOKEN_RE,
            UNC_SERVER_PLACEHOLDER_RE,
            RUNTIME_PROFILE_SERVER_RE,
            TARGET_SERVER_RE,
            HEADLESS_SERVER_RE,
            DEDICATED_SERVER_RE,
            SERVER_CLIENT_TARGETS_RE,
            RUNTIME_TARGET_SERVER_LABEL_RE,
        )
    ):
        return True
    if normalized.endswith(
        "zircon_runtime/src/core/framework/project/runtime_profile_id.rs"
    ) and line.strip() == "Server," and token_match.group(0).lower() == "server":
        return True
    if normalized.endswith(
        "zircon_runtime/src/plugin/export_build_plan/default_profile.rs"
    ) and _token_is_inside_pattern(token_match, line, QUOTED_SERVER_RE):
        return True
    if normalized.endswith(
        "zircon_runtime/src/bin/zircon_export_validate/run.rs"
    ) and _token_is_inside_pattern(token_match, line, QUOTED_SERVER_RE):
        return True
    if normalized.endswith(
        "zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs"
    ) and _token_is_inside_pattern(token_match, line, SERVER_CONFIG_RE):
        return True
    if normalized.endswith(
        "zircon_runtime/src/ui/component/catalog/material_foundation/mui_x.rs"
    ) and "mui_enum_prop" in line and _token_is_inside_pattern(
        token_match, line, QUOTED_SERVER_RE
    ):
        return True
    if normalized.endswith(
        "zircon_runtime/src/ui/component/state_reducer/table.rs"
    ) and 'Some("server")' in line and _token_is_inside_pattern(
        token_match, line, QUOTED_SERVER_RE
    ):
        return True
    if normalized.endswith(
        "zircon_runtime/src/ui/surface/surface/default_interactions/table/mod.rs"
    ) and 'Some("server")' in line and _token_is_inside_pattern(
        token_match, line, QUOTED_SERVER_RE
    ):
        return True
    if normalized.endswith(
        "zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs"
    ) and 'Some("server")' in line and _token_is_inside_pattern(
        token_match, line, QUOTED_SERVER_RE
    ):
        return True
    if normalized.endswith(
        "zircon_runtime/src/platform/capability/matrix/mod.rs"
    ) and _token_is_inside_pattern(token_match, line, SERVER_HEADLESS_POLICY_RE):
        return True
    return False


def _classify_server_token(
    relative_path: str,
    line: str,
    token_match: re.Match[str],
) -> str:
    normalized = relative_path.replace("\\", "/")
    if token_match.group(0).lower() == "render_server":
        return "unclassified-non-network-server"
    if _token_is_inside_pattern(token_match, line, SCENE_SERVER_RE):
        return "editor-scene-comment-debt"
    if normalized in {
        "zircon_editor/src/ui/host/resource_access.rs",
        "zircon_editor/src/ui/retained_host/app.rs",
        "zircon_editor/src/ui/retained_host/app/assets.rs",
        "zircon_editor/src/ui/retained_host/app/host_lifecycle.rs",
    }:
        return "editor-asset-resource-owner-debt"
    return "unclassified-non-network-server"


def _server_reference_decision(
    relative_path: str,
    line_no: int,
    line: str,
    tokens: list[str],
    classification: str,
) -> ServerReferenceDecision:
    decision = SERVER_REFERENCE_CLASSIFICATION_DECISIONS[classification]
    return ServerReferenceDecision(
        path=relative_path,
        line=line_no,
        snippet=line.strip(),
        tokens=tokens,
        classification=classification,
        target_owner=decision["target_owner"],
        required_action=decision["required_action"],
    )


def _classification_counts(
    decisions: list[ServerReferenceDecision],
) -> dict[str, int]:
    counts = {classification: 0 for classification in SERVER_REFERENCE_CLASSIFICATION_ORDER}
    for decision in decisions:
        counts[decision.classification] = counts.get(decision.classification, 0) + 1
    return {key: value for key, value in counts.items() if value}


def _group_reference_decisions(
    decisions: list[ServerReferenceDecision],
) -> dict[str, list[str]]:
    grouped = {classification: [] for classification in SERVER_REFERENCE_CLASSIFICATION_ORDER}
    for decision in decisions:
        grouped.setdefault(decision.classification, []).append(
            f"{decision.path}:{decision.line}"
        )
    return {key: value for key, value in grouped.items() if value}


def _classification_samples(
    decisions: list[ServerReferenceDecision],
    max_samples: int = 5,
) -> dict[str, list[dict[str, object]]]:
    samples: dict[str, list[dict[str, object]]] = {}
    for decision in decisions:
        bucket = samples.setdefault(decision.classification, [])
        if len(bucket) < max_samples:
            bucket.append(asdict(decision))
    return samples


def _sample_source_locations(
    decisions: list[ServerReferenceDecision],
    max_samples: int,
) -> list[dict[str, object]]:
    samples: dict[tuple[str, int], dict[str, object]] = {}
    for decision in decisions:
        key = (decision.path, decision.line)
        sample = samples.get(key)
        if sample is None:
            if len(samples) >= max_samples:
                continue
            sample = asdict(decision)
            sample["target_owners"] = [sample.pop("target_owner")]
            sample["required_actions"] = [sample.pop("required_action")]
            samples[key] = sample
            continue
        sample["classification"] = (
            f"{sample['classification']}+{decision.classification}"
        )
        sample_tokens = sample["tokens"]
        if isinstance(sample_tokens, list):
            sample_tokens.extend(decision.tokens)
        target_owners = sample["target_owners"]
        if isinstance(target_owners, list):
            target_owners.append(decision.target_owner)
        required_actions = sample["required_actions"]
        if isinstance(required_actions, list):
            required_actions.append(decision.required_action)
    return list(samples.values())


def _migration_debt(
    classification_counts: dict[str, int],
) -> list[str]:
    debt: list[str] = []
    for classification in SERVER_REFERENCE_CLASSIFICATION_ORDER:
        count = classification_counts.get(classification, 0)
        if not count:
            continue
        decision = SERVER_REFERENCE_CLASSIFICATION_DECISIONS[classification]
        debt.append(
            f"{classification}: {decision['required_action']} ({count} location(s))"
        )
    return debt


def non_network_server_references(
    root: Path,
    files: Iterable[Path],
    max_locations: int = 20,
) -> dict[str, object]:
    observer_false_positive_count = 0
    allowed_context_count = 0
    test_owned_source_file_count = 0
    test_owned_server_reference_count = 0
    decisions: list[ServerReferenceDecision] = []
    for path in files:
        if _is_test_owned_source(path):
            test_owned_source_file_count += 1
            test_owned_server_reference_count += sum(
                bool(_server_tokens(line)) for line in _read_text(path).splitlines()
            )
            continue
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            raw_matches = list(SERVER_TOKEN_RE.finditer(line))
            if not raw_matches:
                continue
            token_matches = [
                match
                for match in raw_matches
                if _token_has_server_component(match.group(0))
            ]
            if not token_matches:
                observer_false_positive_count += 1
                continue
            disallowed_matches = [
                match
                for match in token_matches
                if not _is_allowed_server_token(path, line, match)
            ]
            if len(disallowed_matches) != len(token_matches):
                allowed_context_count += 1
            if not disallowed_matches:
                continue
            relative_path = _relative(root, path)
            classified_tokens: dict[str, list[str]] = {}
            for match in disallowed_matches:
                classification = _classify_server_token(relative_path, line, match)
                classified_tokens.setdefault(classification, []).append(match.group(0))
            for classification in SERVER_REFERENCE_CLASSIFICATION_ORDER:
                tokens = classified_tokens.get(classification)
                if not tokens:
                    continue
                decisions.append(
                    _server_reference_decision(
                        relative_path=relative_path,
                        line_no=line_no,
                        line=line,
                        tokens=tokens,
                        classification=classification,
                    )
                )

    classification_counts = _classification_counts(decisions)
    reference_decision_groups = _group_reference_decisions(decisions)
    classification_samples = _classification_samples(decisions)
    migration_debt = _migration_debt(classification_counts)
    unclassified_locations = [
        asdict(decision)
        for decision in decisions
        if decision.classification == "unclassified-non-network-server"
    ]
    source_location_count = len({(decision.path, decision.line) for decision in decisions})
    sample_locations = _sample_source_locations(decisions, max_locations)
    risks: list[str] = []
    if unclassified_locations:
        risks.append(
            "non-network server naming references are not classified by the M1 gate: "
            f"{len(unclassified_locations)} location(s)"
        )
    if decisions:
        remaining_groups = ", ".join(sorted(classification_counts))
        risks.append(
            "non-network server naming remains in these owner groups: "
            f"{remaining_groups}. Rename each group during its bounded owner slice."
        )

    return {
        "count": source_location_count,
        "source_location_count": source_location_count,
        "sample_locations": sample_locations,
        "sample_location_count": len(sample_locations),
        "reference_decisions": [asdict(decision) for decision in decisions],
        "reference_decision_count": len(decisions),
        "reference_decision_groups": reference_decision_groups,
        "reference_decision_group_count": len(reference_decision_groups),
        "classification_counts": classification_counts,
        "classification_count": len(classification_counts),
        "classification_samples": classification_samples,
        "observer_false_positive_count": observer_false_positive_count,
        "allowed_context_count": allowed_context_count,
        "test_owned_source_file_count": test_owned_source_file_count,
        "test_owned_server_reference_count": test_owned_server_reference_count,
        "unclassified_locations": unclassified_locations,
        "unclassified_location_count": len(unclassified_locations),
        "non_network_server_migration_debt": migration_debt,
        "non_network_server_migration_debt_count": len(migration_debt),
        "m1_gate_status": (
            "migration-debt-present" if migration_debt else "classified-and-clear"
        ),
        "risks": risks,
    }

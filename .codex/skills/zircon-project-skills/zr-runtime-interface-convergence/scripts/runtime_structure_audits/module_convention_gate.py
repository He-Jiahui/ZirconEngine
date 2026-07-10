from __future__ import annotations

from collections.abc import Mapping


RENDER_SCOPED_MIGRATION_DEBT_PREFIXES = (
    "runtime-naming:legacy: legacy-runtime-graphics-debt:",
    "hard-cutover: legacy-runtime-graphics-debt:",
)


def _prefixed_counts(
    prefix: str,
    counts: Mapping[str, object],
) -> dict[str, int]:
    return {
        f"{prefix}:{classification}": int(count)
        for classification, count in counts.items()
        if int(count)
    }


def _extend_prefixed_debt(
    output: list[str],
    prefix: str,
    debt_items: object,
) -> None:
    if not isinstance(debt_items, list):
        return
    output.extend(f"{prefix}: {item}" for item in debt_items)


def _module_classification_counts(
    module_classification: Mapping[str, Mapping[str, object]],
) -> dict[str, int]:
    counts: dict[str, int] = {}
    for entry in module_classification.values():
        status = str(entry["status"])
        counts[status] = counts.get(status, 0) + 1
    return {
        f"module-classification:{status}": count
        for status, count in sorted(counts.items())
    }


def _module_classification_debt(
    module_classification: Mapping[str, Mapping[str, object]],
) -> list[str]:
    return [
        f"module-classification: {crate_name} remains {entry['status']} "
        f"({', '.join(str(reason) for reason in entry['reasons'])})"
        for crate_name, entry in sorted(module_classification.items())
        if entry["status"] != "converged"
    ]


def _split_render_scoped_migration_debt(
    migration_debt: list[str],
) -> tuple[list[str], list[str]]:
    render_scoped: list[str] = []
    non_render: list[str] = []
    for item in migration_debt:
        if item.startswith(RENDER_SCOPED_MIGRATION_DEBT_PREFIXES):
            render_scoped.append(item)
        else:
            non_render.append(item)
    return render_scoped, non_render


def module_convention_gate(
    *,
    module_classification: Mapping[str, Mapping[str, object]],
    large_file_ownership_gate: Mapping[str, object],
    runtime_naming_boundary: Mapping[str, object],
    hard_cutover_migration_smells: Mapping[str, object],
    non_network_server_references: Mapping[str, object],
) -> dict[str, object]:
    classification_counts: dict[str, int] = {}
    classification_counts.update(_module_classification_counts(module_classification))
    classification_counts.update(
        _prefixed_counts(
            "large-file",
            large_file_ownership_gate["classification_counts"],
        )
    )

    runtime_naming_migration_debt_count = 0
    for term in ("editor", "legacy"):
        term_report = runtime_naming_boundary[term]
        classification_counts.update(
            _prefixed_counts(
                f"runtime-naming:{term}",
                term_report["classification_counts"],
            )
        )
        runtime_naming_migration_debt_count += int(term_report["migration_debt_count"])

    classification_counts.update(
        _prefixed_counts(
            "hard-cutover",
            hard_cutover_migration_smells["classification_counts"],
        )
    )
    classification_counts.update(
        _prefixed_counts(
            "non-network-server",
            non_network_server_references["classification_counts"],
        )
    )

    migration_debt = _module_classification_debt(module_classification)
    _extend_prefixed_debt(
        migration_debt,
        "large-file",
        large_file_ownership_gate["large_file_migration_debt"],
    )
    for term in ("editor", "legacy"):
        _extend_prefixed_debt(
            migration_debt,
            f"runtime-naming:{term}",
            runtime_naming_boundary[term]["migration_debt"],
        )
    _extend_prefixed_debt(
        migration_debt,
        "hard-cutover",
        hard_cutover_migration_smells["hard_cutover_migration_debt"],
    )
    _extend_prefixed_debt(
        migration_debt,
        "non-network-server",
        non_network_server_references["non_network_server_migration_debt"],
    )
    render_scoped_migration_debt, non_render_migration_debt = (
        _split_render_scoped_migration_debt(migration_debt)
    )

    violation_fields = {
        "module_classification_debt_count": len(
            _module_classification_debt(module_classification)
        ),
        "large_file_migration_debt_count": int(
            large_file_ownership_gate["large_file_migration_debt_count"]
        ),
        "runtime_naming_migration_debt_count": runtime_naming_migration_debt_count,
        "hard_cutover_migration_debt_count": int(
            hard_cutover_migration_smells["hard_cutover_migration_debt_count"]
        ),
        "non_network_server_migration_debt_count": int(
            non_network_server_references["non_network_server_migration_debt_count"]
        ),
    }

    risks = []
    for source in (
        large_file_ownership_gate,
        hard_cutover_migration_smells,
        non_network_server_references,
    ):
        risks.extend(str(risk) for risk in source.get("risks", []))

    return {
        "m1_gate_status": (
            "migration-debt-present" if migration_debt else "classified-and-clear"
        ),
        "classification_counts": classification_counts,
        "classification_count": len(classification_counts),
        "migration_debt": migration_debt,
        "migration_debt_count": len(migration_debt),
        "render_scoped_migration_debt": render_scoped_migration_debt,
        "render_scoped_migration_debt_count": len(render_scoped_migration_debt),
        "non_render_migration_debt": non_render_migration_debt,
        "non_render_migration_debt_count": len(non_render_migration_debt),
        "exempt": [],
        "exempt_count": 0,
        "source_gate_statuses": {
            "large_file_ownership_gate": large_file_ownership_gate["m1_gate_status"],
            "runtime_naming_boundary": runtime_naming_boundary["gate_status"],
            "hard_cutover_migration_smells": hard_cutover_migration_smells[
                "hard_cutover_gate_status"
            ],
            "non_network_server_references": non_network_server_references[
                "m1_gate_status"
            ],
        },
        "violation_fields": violation_fields,
        "risk_count": len(risks),
        "risks": risks,
    }

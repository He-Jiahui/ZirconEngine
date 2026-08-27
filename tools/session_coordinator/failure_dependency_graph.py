from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from typing import Mapping


@dataclass(frozen=True, slots=True)
class GraphDiagnostic:
    code: str
    message: str
    paths: tuple[str, ...] = ()
    details: dict[str, object] = field(default_factory=dict)


def failure_graph_diagnostics(
    edges: Mapping[str, set[str]],
    edge_artifacts: Mapping[tuple[str, str], set[str]],
    *,
    max_depth: int,
) -> list[GraphDiagnostic]:
    diagnostics = _cycle_diagnostics(edges, edge_artifacts)
    diagnostics.extend(_depth_diagnostics(edges, max_depth=max_depth))
    return diagnostics


def _cycle_diagnostics(
    edges: Mapping[str, set[str]],
    edge_artifacts: Mapping[tuple[str, str], set[str]],
) -> list[GraphDiagnostic]:
    diagnostics: list[GraphDiagnostic] = []
    for plans in _strongly_connected_components(edges):
        members = set(plans)
        internal_edges = [
            (origin, fixing)
            for origin in plans
            for fixing in _sorted(edges.get(origin, set()))
            if fixing in members
        ]
        if len(plans) == 1 and not internal_edges:
            continue
        component_id = hashlib.sha256(
            json.dumps(plans, ensure_ascii=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        edge_details = [
            {
                "originPlan": origin,
                "fixingPlan": fixing,
                "artifacts": _sorted(edge_artifacts.get((origin, fixing), set())),
            }
            for origin, fixing in internal_edges
        ]
        artifact_paths = tuple(
            _sorted(
                {
                    artifact
                    for edge in edge_details
                    for artifact in edge["artifacts"]
                }
            )
        )
        diagnostics.append(
            GraphDiagnostic(
                "cycle",
                (
                    f"Failure dependency SCC {component_id[:12]} contains "
                    f"{len(plans)} plan(s) and {len(internal_edges)} edge(s)"
                ),
                artifact_paths,
                {
                    "componentId": component_id,
                    "plans": list(plans),
                    "edges": edge_details,
                },
            )
        )
    return diagnostics


def _strongly_connected_components(
    edges: Mapping[str, set[str]],
) -> list[tuple[str, ...]]:
    nodes = _sorted(set(edges) | {target for targets in edges.values() for target in targets})
    next_index = 0
    indices: dict[str, int] = {}
    low_links: dict[str, int] = {}
    stack: list[str] = []
    on_stack: set[str] = set()
    components: list[tuple[str, ...]] = []

    def connect(node: str) -> None:
        nonlocal next_index
        indices[node] = next_index
        low_links[node] = next_index
        next_index += 1
        stack.append(node)
        on_stack.add(node)

        for target in _sorted(edges.get(node, set())):
            if target not in indices:
                connect(target)
                low_links[node] = min(low_links[node], low_links[target])
            elif target in on_stack:
                low_links[node] = min(low_links[node], indices[target])

        if low_links[node] != indices[node]:
            return
        component: list[str] = []
        while True:
            member = stack.pop()
            on_stack.remove(member)
            component.append(member)
            if member == node:
                break
        components.append(tuple(_sorted(component)))

    for node in nodes:
        if node not in indices:
            connect(node)
    return sorted(components, key=lambda component: _sort_key(component[0]))


def _depth_diagnostics(
    edges: Mapping[str, set[str]], *, max_depth: int
) -> list[GraphDiagnostic]:
    diagnostics: list[GraphDiagnostic] = []
    visiting: list[str] = []
    depths: dict[str, int] = {}

    def visit(node: str) -> int:
        if node in visiting:
            return 0
        if node in depths:
            return depths[node]
        visiting.append(node)
        depth = 0
        for target in _sorted(edges.get(node, set())):
            depth = max(depth, 1 + visit(target))
        visiting.pop()
        depths[node] = depth
        if depth > max_depth:
            diagnostics.append(
                GraphDiagnostic(
                    "excessive_depth",
                    f"Failure dependency depth {depth} exceeds {max_depth}",
                    (node,),
                )
            )
        return depth

    nodes = set(edges) | {target for targets in edges.values() for target in targets}
    for node in _sorted(nodes):
        visit(node)
    return diagnostics


def _sorted(values) -> list[str]:
    return sorted(values, key=_sort_key)


def _sort_key(value: str) -> tuple[str, str]:
    return value.casefold(), value

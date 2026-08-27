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
    visited: set[str] = set()
    finish_order: list[str] = []
    for root in nodes:
        if root in visited:
            continue
        visited.add(root)
        stack: list[tuple[str, list[str], int]] = [
            (root, _sorted(edges.get(root, set())), 0)
        ]
        while stack:
            node, targets, target_index = stack[-1]
            if target_index == len(targets):
                finish_order.append(node)
                stack.pop()
                continue
            target = targets[target_index]
            stack[-1] = (node, targets, target_index + 1)
            if target in visited:
                continue
            visited.add(target)
            stack.append((target, _sorted(edges.get(target, set())), 0))

    reverse_edges = {node: set() for node in nodes}
    for origin, targets in edges.items():
        for target in targets:
            reverse_edges[target].add(origin)

    assigned: set[str] = set()
    components: list[tuple[str, ...]] = []
    for root in reversed(finish_order):
        if root in assigned:
            continue
        assigned.add(root)
        pending = [root]
        component: list[str] = []
        while pending:
            node = pending.pop()
            component.append(node)
            for target in _sorted(reverse_edges[node]):
                if target in assigned:
                    continue
                assigned.add(target)
                pending.append(target)
        components.append(tuple(_sorted(component)))
    return sorted(components, key=lambda component: _sort_key(component[0]))


def _depth_diagnostics(
    edges: Mapping[str, set[str]], *, max_depth: int
) -> list[GraphDiagnostic]:
    diagnostics: list[GraphDiagnostic] = []
    depths: dict[str, int] = {}
    nodes = set(edges) | {target for targets in edges.values() for target in targets}
    for root in _sorted(nodes):
        if root in depths:
            continue
        active = {root}
        stack: list[tuple[str, list[str], int, int]] = [
            (root, _sorted(edges.get(root, set())), 0, 0)
        ]
        while stack:
            node, targets, target_index, depth = stack[-1]
            if target_index < len(targets):
                target = targets[target_index]
                stack[-1] = (node, targets, target_index + 1, depth)
                if target in active:
                    stack[-1] = (node, targets, target_index + 1, max(depth, 1))
                    continue
                if target in depths:
                    stack[-1] = (
                        node,
                        targets,
                        target_index + 1,
                        max(depth, 1 + depths[target]),
                    )
                    continue
                active.add(target)
                stack.append((target, _sorted(edges.get(target, set())), 0, 0))
                continue

            stack.pop()
            active.remove(node)
            depths[node] = depth
            if depth > max_depth:
                diagnostics.append(
                    GraphDiagnostic(
                        "excessive_depth",
                        f"Failure dependency depth {depth} exceeds {max_depth}",
                        (node,),
                    )
                )
            if stack:
                parent, parent_targets, parent_index, parent_depth = stack[-1]
                stack[-1] = (
                    parent,
                    parent_targets,
                    parent_index,
                    max(parent_depth, 1 + depth),
                )
    return diagnostics


def _sorted(values) -> list[str]:
    return sorted(values, key=_sort_key)


def _sort_key(value: str) -> tuple[str, str]:
    return value.casefold(), value

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path

from ..models import CoordinatorError
from ..plans import PlanRepository


_FENCE = re.compile(
    r"^```zircon-workflow[ \t]*\r?\n(?P<body>.*?)^```[ \t]*\r?$",
    re.MULTILINE | re.DOTALL,
)
_MILESTONE = re.compile(
    r"^##\s+Milestone\s+(?P<id>M\d+)\s*:\s*(?P<title>.+?)\s*$",
    re.MULTILINE | re.IGNORECASE,
)
_LEGACY_NUMBERED_MILESTONE = re.compile(
    r"^#{2,6}\s+(?:[A-Za-z]{2,}[A-Za-z0-9]*\d*)-(?P<id>M\d+)\s+(?P<title>.+?)\s*$",
    re.MULTILINE,
)
_SLICE = re.compile(
    r"^-\s*\[[ xX]\]\s*\*\*(?P<id>M\d+\.\d+)\s+(?P<title>.+?)\.\*\*",
    re.MULTILINE,
)
_NODE_ID = re.compile(r"^M[1-9]\d*$")
_WORKFLOW_ID = re.compile(r"^[a-z0-9][a-z0-9-]{2,127}$")
_MAX_PLAN_BYTES = 2 * 1024 * 1024
_MAX_MILESTONES = 200
_MAX_SLICES = 5_000
_MAX_EDGES = 10_000
_MAX_TITLE_CHARS = 500
_MAX_GOAL_CHARS = 2_000


@dataclass(frozen=True, slots=True)
class TopologyNode:
    node_id: str
    title: str
    depends_on: tuple[str, ...] = ()
    milestone_id: str | None = None


@dataclass(frozen=True, slots=True)
class WorkflowTopology:
    schema_version: int
    workflow_id: str
    goal: str
    plan_path: str
    plan_id: str
    content_hash: str
    milestones: tuple[TopologyNode, ...]
    slices: tuple[TopologyNode, ...]
    source: str

    def semantic_json(self) -> str:
        value = {
            "schema": self.schema_version,
            "workflow_id": self.workflow_id,
            "goal": self.goal,
            "source": self.source,
            "milestones": [asdict(item) for item in self.milestones],
            "slices": [asdict(item) for item in self.slices],
        }
        return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))

    @property
    def topology_hash(self) -> str:
        return hashlib.sha256(self.semantic_json().encode("utf-8")).hexdigest()

    def canonical_json(self) -> str:
        value = {
            "schema": self.schema_version,
            "workflow_id": self.workflow_id,
            "goal": self.goal,
            "plan_path": self.plan_path,
            "plan_id": self.plan_id,
            "content_hash": self.content_hash,
            "topology_hash": self.topology_hash,
            "source": self.source,
            "milestones": [asdict(item) for item in self.milestones],
            "slices": [asdict(item) for item in self.slices],
        }
        return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


class TopologyParser:
    """Parse a numbered plan without granting write access to its definition."""

    def __init__(self, repo_root: str | Path):
        self.repo_root = Path(repo_root).resolve()
        self.plans = PlanRepository(self.repo_root)

    def parse(self, plan_path: str | Path) -> WorkflowTopology:
        owner = self.plans.resolve_owner(plan_path)
        absolute = (self.repo_root / owner.path).resolve()
        raw = absolute.read_bytes()
        if len(raw) > _MAX_PLAN_BYTES:
            raise CoordinatorError(
                "workflow_topology_too_large",
                "Workflow plan exceeds the parser size limit",
                details={"byte_count": len(raw), "maximum": _MAX_PLAN_BYTES},
            )
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError as error:
            raise CoordinatorError(
                "workflow_plan_encoding", "Workflow plan must be UTF-8"
            ) from error
        fences = list(_FENCE.finditer(text))
        marker_count = text.count("```zircon-workflow")
        if marker_count != len(fences):
            raise CoordinatorError(
                "workflow_topology_malformed_fence",
                "zircon-workflow fence is not closed or is malformed",
            )
        if len(fences) > 1:
            raise CoordinatorError(
                "workflow_topology_fence_count",
                "A plan may contain exactly one zircon-workflow block",
                details={"count": len(fences)},
            )
        if fences:
            schema, workflow_id, goal, milestones = self._parse_fenced(
                fences[0].group("body")
            )
            source = "zircon-workflow"
        else:
            schema = 1
            workflow_id = self._fallback_workflow_id(owner.path)
            goal = self._fallback_goal(text, absolute.stem)
            milestones = self._parse_fallback_milestones(text)
            source = "headings"
        slices = self._parse_slices(text, {item.node_id for item in milestones})
        self._validate_graph(milestones)
        return WorkflowTopology(
            schema_version=schema,
            workflow_id=workflow_id,
            goal=goal,
            plan_path=owner.path,
            plan_id=owner.plan_id,
            content_hash=hashlib.sha256(raw).hexdigest(),
            milestones=milestones,
            slices=slices,
            source=source,
        )

    def _parse_fenced(
        self, body: str
    ) -> tuple[int, str, str, tuple[TopologyNode, ...]]:
        try:
            value = json.loads(body)
        except (json.JSONDecodeError, RecursionError) as error:
            raise CoordinatorError(
                "workflow_topology_json", "zircon-workflow block is not valid JSON"
            ) from error
        if not isinstance(value, dict) or value.get("schema") != 1:
            raise CoordinatorError(
                "workflow_topology_schema", "Only zircon-workflow schema 1 is supported"
            )
        workflow_id = value.get("workflow_id")
        goal = value.get("goal")
        raw_milestones = value.get("milestones")
        if not isinstance(workflow_id, str) or not _WORKFLOW_ID.fullmatch(workflow_id):
            raise CoordinatorError(
                "workflow_topology_id", "workflow_id must be a stable lowercase slug"
            )
        if not isinstance(goal, str) or not goal.strip():
            raise CoordinatorError("workflow_topology_goal", "Workflow goal is required")
        if len(goal) > _MAX_GOAL_CHARS:
            raise CoordinatorError(
                "workflow_topology_goal_too_large", "Workflow goal is too long"
            )
        if not isinstance(raw_milestones, list) or not raw_milestones:
            raise CoordinatorError(
                "workflow_topology_milestones", "Workflow milestones must be a non-empty list"
            )
        if len(raw_milestones) > _MAX_MILESTONES:
            raise CoordinatorError(
                "workflow_topology_too_many_nodes",
                "Workflow has too many milestones",
                details={"count": len(raw_milestones), "maximum": _MAX_MILESTONES},
            )
        milestones: list[TopologyNode] = []
        for item in raw_milestones:
            if not isinstance(item, dict):
                raise CoordinatorError(
                    "workflow_topology_milestone", "Each milestone must be an object"
                )
            node_id = item.get("id")
            title = item.get("title")
            dependencies = item.get("depends_on")
            if not isinstance(node_id, str) or not _NODE_ID.fullmatch(node_id):
                raise CoordinatorError(
                    "workflow_topology_milestone_id", "Milestone IDs must match M<number>"
                )
            if not isinstance(title, str) or not title.strip():
                raise CoordinatorError(
                    "workflow_topology_milestone_title", f"Milestone {node_id} needs a title"
                )
            if len(title) > _MAX_TITLE_CHARS:
                raise CoordinatorError(
                    "workflow_topology_title_too_large",
                    f"Milestone {node_id} title is too long",
                )
            if not isinstance(dependencies, list) or any(
                not isinstance(value, str) or not _NODE_ID.fullmatch(value)
                for value in dependencies
            ):
                raise CoordinatorError(
                    "workflow_topology_dependencies",
                    f"Milestone {node_id} dependencies must be milestone IDs",
                )
            milestones.append(
                TopologyNode(node_id, title.strip(), tuple(dependencies))
            )
        return 1, workflow_id, goal.strip(), tuple(milestones)

    @staticmethod
    def _parse_fallback_milestones(text: str) -> tuple[TopologyNode, ...]:
        # Older numbered plans used headings such as ``### SH03-M2 Title``.
        # Treat that stable plan-local prefix as presentation only; workflow
        # nodes remain canonical M<n> IDs and the plan file stays immutable.
        matches = sorted(
            (*_MILESTONE.finditer(text), *_LEGACY_NUMBERED_MILESTONE.finditer(text)),
            key=lambda match: match.start(),
        )
        if not matches:
            raise CoordinatorError(
                "workflow_topology_missing", "Plan has no zircon-workflow block or milestone headings"
            )
        if len(matches) > _MAX_MILESTONES:
            raise CoordinatorError(
                "workflow_topology_too_many_nodes", "Workflow has too many milestones"
            )
        result: list[TopologyNode] = []
        for index, match in enumerate(matches):
            section_end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
            section = text[match.end() : section_end]
            dependencies: tuple[str, ...] = ()
            dependency_line = re.search(
                r"\*\*Dependencies:\*\*\s*(.+)$", section, re.MULTILINE | re.IGNORECASE
            )
            if dependency_line:
                dependencies = tuple(
                    dict.fromkeys(re.findall(r"\bM[1-9]\d*\b", dependency_line.group(1)))
                )
            title = match.group("title").strip()
            if len(title) > _MAX_TITLE_CHARS:
                raise CoordinatorError(
                    "workflow_topology_title_too_large",
                    f"Milestone {match.group('id').upper()} title is too long",
                )
            result.append(
                TopologyNode(
                    match.group("id").upper(),
                    title,
                    dependencies,
                )
            )
        return tuple(result)

    @staticmethod
    def _parse_slices(text: str, milestone_ids: set[str]) -> tuple[TopologyNode, ...]:
        seen: set[str] = set()
        result: list[TopologyNode] = []
        for match in _SLICE.finditer(text):
            node_id = match.group("id").upper()
            if node_id in seen:
                raise CoordinatorError(
                    "workflow_topology_duplicate_id", f"Duplicate workflow node ID: {node_id}"
                )
            milestone_id = node_id.split(".", 1)[0]
            if milestone_id not in milestone_ids:
                raise CoordinatorError(
                    "workflow_topology_slice_owner",
                    f"Slice {node_id} has no owning milestone {milestone_id}",
                )
            seen.add(node_id)
            title = match.group("title").strip()
            if len(title) > _MAX_TITLE_CHARS:
                raise CoordinatorError(
                    "workflow_topology_title_too_large", f"Slice {node_id} title is too long"
                )
            result.append(
                TopologyNode(
                    node_id=node_id,
                    title=title,
                    depends_on=(),
                    milestone_id=milestone_id,
                )
            )
            if len(result) > _MAX_SLICES:
                raise CoordinatorError(
                    "workflow_topology_too_many_nodes", "Workflow has too many slices"
                )
        return tuple(result)

    @staticmethod
    def _validate_graph(milestones: tuple[TopologyNode, ...]) -> None:
        ids = [item.node_id for item in milestones]
        if len(ids) != len(set(ids)):
            duplicate = next(item for item in ids if ids.count(item) > 1)
            raise CoordinatorError(
                "workflow_topology_duplicate_id", f"Duplicate workflow node ID: {duplicate}"
            )
        known = set(ids)
        edge_count = sum(len(item.depends_on) for item in milestones)
        if edge_count > _MAX_EDGES:
            raise CoordinatorError(
                "workflow_topology_too_many_edges", "Workflow has too many dependencies"
            )
        incoming = {node_id: 0 for node_id in ids}
        dependents: dict[str, list[str]] = {node_id: [] for node_id in ids}
        for item in milestones:
            missing = sorted(set(item.depends_on) - known)
            if missing:
                raise CoordinatorError(
                    "workflow_topology_missing_dependency",
                    f"Milestone {item.node_id} has unknown dependencies",
                    details={"missing": missing},
                )
            if item.node_id in item.depends_on:
                raise CoordinatorError(
                    "workflow_topology_cycle", f"Milestone {item.node_id} depends on itself"
                )
            incoming[item.node_id] = len(item.depends_on)
            for dependency in item.depends_on:
                dependents[dependency].append(item.node_id)
        ready = [node_id for node_id, count in incoming.items() if count == 0]
        visited_count = 0
        while ready:
            node_id = ready.pop()
            visited_count += 1
            for dependent in dependents[node_id]:
                incoming[dependent] -= 1
                if incoming[dependent] == 0:
                    ready.append(dependent)
        if visited_count != len(ids):
            raise CoordinatorError(
                "workflow_topology_cycle", "Workflow dependency graph contains a cycle"
            )

    @staticmethod
    def _fallback_workflow_id(plan_path: str) -> str:
        stem = Path(plan_path).stem.lower()
        value = re.sub(r"[^a-z0-9]+", "-", stem).strip("-")
        if not _WORKFLOW_ID.fullmatch(value):
            raise CoordinatorError(
                "workflow_topology_id", "Fallback workflow_id is not a stable lowercase slug"
            )
        return value

    @staticmethod
    def _fallback_goal(text: str, default: str) -> str:
        heading = re.search(r"^#\s+(.+?)\s*$", text, re.MULTILINE)
        goal = heading.group(1).strip() if heading else default
        if len(goal) > _MAX_GOAL_CHARS:
            raise CoordinatorError(
                "workflow_topology_goal_too_large", "Workflow goal is too long"
            )
        return goal

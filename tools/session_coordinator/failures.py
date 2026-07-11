from __future__ import annotations

import importlib.util
import json
import os
import re
import sys
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from types import ModuleType
from typing import Any

from .database import Database
from .models import CoordinatorError, utc_text


MARKDOWN_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")


@dataclass(frozen=True, slots=True)
class FailureNode:
    node_id: int
    lifecycle_key: str
    artifact_path: str
    kind: str
    status: str
    created_at: str
    resolved_at: str | None
    summary_slug: str
    origin_plan: str
    fixing_plan: str
    origin_child_dir: str
    fixing_child_dir: str
    priority: int


@dataclass(frozen=True, slots=True)
class GraphDiagnostic:
    code: str
    message: str
    paths: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class FailureGraphAudit:
    nodes: tuple[FailureNode, ...]
    diagnostics: tuple[GraphDiagnostic, ...]

    @property
    def node_count(self) -> int:
        return len(self.nodes)


@dataclass(frozen=True, slots=True)
class FailureResolution:
    root_cause: str
    architecture_fix: str
    validation: str
    return_summary: str

    def validate(self) -> None:
        for field_name, value in (
            ("root_cause", self.root_cause),
            ("architecture_fix", self.architecture_fix),
            ("validation", self.validation),
            ("return_summary", self.return_summary),
        ):
            if not value.strip():
                raise ValueError(f"Failure resolution field {field_name} cannot be empty")


class FailureGraphService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        *,
        max_depth: int = 8,
        validator_script: str | Path | None = None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.max_depth = max_depth
        source_root = Path(__file__).resolve().parents[2]
        self.validator_script = Path(validator_script).resolve() if validator_script else (
            source_root
            / ".codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py"
        )
        self._validator: ModuleType | None = None

    def import_repository(self) -> FailureGraphAudit:
        validator = self._validator_module()
        records, parse_errors = validator.parse_handoff_records(self.repo_root)
        validation_errors = validator.validate_repository(self.repo_root)
        diagnostics: list[GraphDiagnostic] = [
            GraphDiagnostic("parse_error", error) for error in parse_errors
        ]
        diagnostics.extend(
            GraphDiagnostic("schema_validation", error) for error in validation_errors
        )

        by_lifecycle: dict[str, list[Any]] = {}
        edges: dict[str, set[str]] = {}
        for record in records:
            by_lifecycle.setdefault(record.lifecycle_key, []).append(record)
            origin = self._relative(record.origin_plan)
            fixing = self._relative(record.fixing_plan)
            edges.setdefault(origin, set()).add(fixing)
            if origin.casefold() == fixing.casefold():
                diagnostics.append(
                    GraphDiagnostic(
                        "self_edge",
                        f"Failure {record.summary_slug} routes a plan to itself",
                        (record.relative_path,),
                    )
                )
        for lifecycle_key, lifecycle_records in by_lifecycle.items():
            if len(lifecycle_records) > 1:
                diagnostics.append(
                    GraphDiagnostic(
                        "duplicate_lifecycle",
                        f"Lifecycle {lifecycle_key} has {len(lifecycle_records)} artifacts",
                        tuple(item.relative_path for item in lifecycle_records),
                    )
                )
        diagnostics.extend(self._graph_diagnostics(edges))

        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute("DELETE FROM failure_nodes")
            connection.execute("DELETE FROM failure_diagnostics")
            for record in records:
                canonical_status = "open" if record.kind == "failure" else "fixed"
                connection.execute(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, imported_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        record.lifecycle_key,
                        record.relative_path,
                        record.kind,
                        canonical_status,
                        record.created_at,
                        record.resolved_at,
                        record.summary_slug,
                        self._relative(record.origin_plan),
                        self._relative(record.fixing_plan),
                        self._relative(record.origin_child_dir),
                        self._relative(record.fixing_child_dir),
                        self._priority(record.kind, canonical_status),
                        now,
                    ),
                )
            for diagnostic in diagnostics:
                connection.execute(
                    """
                    INSERT INTO failure_diagnostics(code, message, paths_json, created_at)
                    VALUES (?, ?, ?, ?)
                    """,
                    (
                        diagnostic.code,
                        diagnostic.message,
                        json.dumps(diagnostic.paths),
                        now,
                    ),
                )
        return self.audit()

    def audit(self) -> FailureGraphAudit:
        with self.database.connect() as connection:
            node_rows = connection.execute(
                "SELECT * FROM failure_nodes ORDER BY priority, created_at, summary_slug, artifact_path"
            ).fetchall()
            diagnostic_rows = connection.execute(
                "SELECT * FROM failure_diagnostics ORDER BY code, message"
            ).fetchall()
        nodes = tuple(self._node_from_row(row) for row in node_rows)
        diagnostics = tuple(
            GraphDiagnostic(
                code=row["code"],
                message=row["message"],
                paths=tuple(json.loads(row["paths_json"])),
            )
            for row in diagnostic_rows
        )
        return FailureGraphAudit(nodes, diagnostics)

    def open_for_plan(self, fixing_plan: str | Path) -> list[FailureNode]:
        relative = self._relative(self._resolve_repo_path(fixing_plan))
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT * FROM failure_nodes
                WHERE fixing_plan = ? AND kind = 'failure' AND status = 'open'
                ORDER BY priority, created_at, summary_slug, artifact_path
                """,
                (relative,),
            ).fetchall()
        return [self._node_from_row(row) for row in rows]

    def validator_errors(self) -> list[str]:
        return list(self._validator_module().validate_repository(self.repo_root))

    def return_fixed(
        self,
        lifecycle_key: str,
        resolution: FailureResolution,
        *,
        resolved_at: date,
    ) -> Path:
        resolution.validate()
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT * FROM failure_nodes WHERE lifecycle_key = ?", (lifecycle_key,)
            ).fetchall()
        if len(rows) != 1:
            raise CoordinatorError(
                "ambiguous_failure_lifecycle",
                f"Expected one open artifact for lifecycle {lifecycle_key}; found {len(rows)}",
            )
        node = self._node_from_row(rows[0])
        if node.kind != "failure" or node.status != "open":
            raise CoordinatorError("failure_not_open", "Only an open failure can be returned")

        source = self.repo_root / node.artifact_path
        origin_plan = self.repo_root / node.origin_plan
        fixing_plan = self.repo_root / node.fixing_plan
        destination = (
            self.repo_root
            / node.origin_child_dir
            / f"fixed-{resolved_at.isoformat()}-{node.summary_slug}.md"
        )
        if destination.exists():
            raise CoordinatorError(
                "fixed_destination_exists",
                f"Fixed destination already exists: {self._relative(destination)}",
            )
        source_text = source.read_text(encoding="utf-8")
        origin_text = origin_plan.read_text(encoding="utf-8")
        fixing_text = fixing_plan.read_text(encoding="utf-8")
        fixed_text = self._fixed_content(source_text, resolution, resolved_at)
        fixed_origin = self._replace_handoff_link(
            origin_text, origin_plan, source, destination, node.summary_slug
        )
        fixed_fixing = self._replace_handoff_link(
            fixing_text, fixing_plan, source, destination, node.summary_slug
        )

        originals = {
            source: source_text,
            origin_plan: origin_text,
            fixing_plan: fixing_text,
        }
        try:
            self._atomic_write(destination, fixed_text)
            self._atomic_write(origin_plan, fixed_origin)
            self._atomic_write(fixing_plan, fixed_fixing)
            source.unlink()
            self.import_repository()
        except BaseException:
            self._restore_text(source, source_text)
            self._restore_text(origin_plan, origin_text)
            self._restore_text(fixing_plan, fixing_text)
            if destination not in originals:
                destination.unlink(missing_ok=True)
            self.import_repository()
            raise
        return destination

    def _graph_diagnostics(self, edges: dict[str, set[str]]) -> list[GraphDiagnostic]:
        diagnostics: list[GraphDiagnostic] = []
        visiting: list[str] = []
        visited: set[str] = set()
        reported_cycles: set[tuple[str, ...]] = set()

        def visit(node: str) -> int:
            if node in visiting:
                start = visiting.index(node)
                cycle = tuple(visiting[start:] + [node])
                normalized = tuple(sorted(set(cycle), key=str.casefold))
                if normalized not in reported_cycles:
                    reported_cycles.add(normalized)
                    diagnostics.append(
                        GraphDiagnostic("cycle", "Failure dependency cycle detected", cycle)
                    )
                return 0
            if node in visited:
                return 0
            visiting.append(node)
            depth = 0
            for target in sorted(edges.get(node, ()), key=str.casefold):
                depth = max(depth, 1 + visit(target))
            visiting.pop()
            visited.add(node)
            if depth > self.max_depth:
                diagnostics.append(
                    GraphDiagnostic(
                        "excessive_depth",
                        f"Failure dependency depth {depth} exceeds {self.max_depth}",
                        (node,),
                    )
                )
            return depth

        for node in sorted(set(edges) | {item for targets in edges.values() for item in targets}, key=str.casefold):
            visit(node)
        return diagnostics

    def _validator_module(self) -> ModuleType:
        if self._validator is not None:
            return self._validator
        module_name = "zircon_plan_failure_handoff_validator"
        spec = importlib.util.spec_from_file_location(module_name, self.validator_script)
        if spec is None or spec.loader is None:
            raise CoordinatorError("validator_unavailable", f"Cannot load {self.validator_script}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[module_name] = module
        spec.loader.exec_module(module)
        self._validator = module
        return module

    def _fixed_content(
        self, content: str, resolution: FailureResolution, resolved_at: date
    ) -> str:
        lines = content.splitlines()
        if not lines or lines[0].strip() != "---":
            raise CoordinatorError("invalid_handoff", "Handoff is missing frontmatter")
        end = next(
            (index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---"),
            None,
        )
        if end is None:
            raise CoordinatorError("invalid_handoff", "Handoff frontmatter is unterminated")
        metadata_lines = lines[1:end]
        updates = {
            "handoff_kind": "fixed",
            "status": "fixed",
            "resolved_at": resolved_at.isoformat(),
        }
        found: set[str] = set()
        rewritten: list[str] = []
        for line in metadata_lines:
            key = line.split(":", 1)[0].strip() if ":" in line else ""
            if key in updates:
                rewritten.append(f"{key}: {updates[key]}")
                found.add(key)
            else:
                rewritten.append(line)
        for key in ("handoff_kind", "status", "resolved_at"):
            if key not in found:
                rewritten.append(f"{key}: {updates[key]}")
        body = "\n".join(lines[end + 1 :])
        heading = "## 修复结果与回传"
        if heading not in body:
            raise CoordinatorError("invalid_handoff", f"Handoff is missing {heading}")
        prefix = body.split(heading, 1)[0].rstrip()
        result = (
            f"- 根因：{resolution.root_cause.strip()}\n"
            f"- 架构修复：{resolution.architecture_fix.strip()}\n"
            f"- 验证：{resolution.validation.strip()}\n"
            f"- 回传：{resolution.return_summary.strip()}"
        )
        return (
            "---\n"
            + "\n".join(rewritten)
            + "\n---\n\n"
            + prefix
            + f"\n\n{heading}\n\n{result}\n"
        )

    def _replace_handoff_link(
        self,
        content: str,
        plan_path: Path,
        source: Path,
        destination: Path,
        slug: str,
    ) -> str:
        replacement = (
            f"- fixed 已修复：[{slug}]({Path(os.path.relpath(destination, plan_path.parent)).as_posix()})"
        )
        lines = content.splitlines()
        replaced = False
        output: list[str] = []
        for line in lines:
            matches_source = False
            for raw_target in MARKDOWN_LINK.findall(line):
                target = raw_target.strip().strip("<>").split("#", 1)[0]
                if not target:
                    continue
                candidate = (plan_path.parent / Path(target)).resolve()
                if candidate == source.resolve():
                    matches_source = True
                    break
            if matches_source:
                output.append(replacement)
                replaced = True
            else:
                output.append(line)
        if not replaced:
            raise CoordinatorError(
                "handoff_link_missing",
                f"Plan {self._relative(plan_path)} does not link to {self._relative(source)}",
            )
        return "\n".join(output).rstrip() + "\n"

    @staticmethod
    def _priority(kind: str, status: str) -> int:
        return 0 if kind == "failure" and status == "open" else 100

    def _atomic_write(self, path: Path, content: str) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name(f".{path.name}.coordinator-{os.getpid()}.tmp")
        temporary.write_text(content, encoding="utf-8")
        os.replace(temporary, path)

    @staticmethod
    def _restore_text(path: Path, content: str) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def _resolve_repo_path(self, value: str | Path) -> Path:
        raw = Path(value)
        candidate = raw.resolve() if raw.is_absolute() else (self.repo_root / raw).resolve()
        if not candidate.is_relative_to(self.repo_root):
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}")
        return candidate

    def _relative(self, path: Path) -> str:
        return path.resolve().relative_to(self.repo_root).as_posix()

    @staticmethod
    def _node_from_row(row) -> FailureNode:
        return FailureNode(
            node_id=int(row["node_id"]),
            lifecycle_key=row["lifecycle_key"],
            artifact_path=row["artifact_path"],
            kind=row["kind"],
            status=row["status"],
            created_at=row["created_at"],
            resolved_at=row["resolved_at"],
            summary_slug=row["summary_slug"],
            origin_plan=row["origin_plan"],
            fixing_plan=row["fixing_plan"],
            origin_child_dir=row["origin_child_dir"],
            fixing_child_dir=row["fixing_child_dir"],
            priority=int(row["priority"]),
        )

from __future__ import annotations

import importlib.util
import hashlib
import json
import os
import re
import sys
import tempfile
from contextlib import nullcontext
from dataclasses import dataclass
from dataclasses import replace
from datetime import date
from pathlib import Path, PurePosixPath
from sqlite3 import Connection
from types import ModuleType
from typing import Any

from .database import Database
from .models import CoordinatorError, utc_text
from .plans import PlanRepository


MARKDOWN_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
# Keep the immutable-action snapshot aligned with the validator: a dated output
# record mentioning failure is not a Failure-graph artifact without this suffix.
DATE_FIRST_HANDOFF = re.compile(
    r"^\d{4}-\d{2}-\d{2}-(?:[a-z0-9]+-)*(?:failure|fixed)-handoff\.md$",
    re.IGNORECASE,
)
WORKFLOW_NODE_ID = re.compile(r"M[1-9]\d*(?:\.[1-9]\d*)?")
SUMMARY_SLUG = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
LOCAL_FAILURE_SCOPE = "local"


def _is_failure_artifact(path: Path) -> bool:
    name = path.name.casefold()
    return (
        name.startswith("failure-")
        or name.startswith("fixed-")
        or bool(DATE_FIRST_HANDOFF.match(path.name))
        or "failure-handoff" in name
    )


def failure_artifact_snapshot(repo_root: str | Path) -> list[dict[str, str]]:
    root = Path(repo_root).resolve()
    plans_root = (root / "docs" / "plans").resolve()
    if not plans_root.is_dir() or not plans_root.is_relative_to(root):
        return []
    paths = {path.resolve() for path in plans_root.rglob("*.md") if path.is_file() and _is_failure_artifact(path)}
    artifacts: list[dict[str, str]] = []
    for path in sorted(paths, key=lambda item: str(item).casefold()):
        if not path.is_relative_to(plans_root):
            raise CoordinatorError(
                "failure_artifact_outside_plans", "Failure artifact escaped plans root"
            )
        artifacts.append(
            {
                "path": path.relative_to(root).as_posix(),
                "hash": hashlib.sha256(path.read_bytes()).hexdigest(),
            }
        )
    return artifacts


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
    origin_workflow_node: str | None
    fixing_plan: str
    origin_child_dir: str
    fixing_child_dir: str
    priority: int
    plan_link_mode: str
    related_code: tuple[str, ...]


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

    def import_repository(
        self,
        *,
        expected_artifacts: list[dict[str, str]] | None = None,
        connection: Connection | None = None,
    ) -> FailureGraphAudit:
        validator = self._validator_module()
        records, parse_errors, validation_errors = self._parse_immutable_snapshot(
            validator, expected_artifacts
        )
        diagnostics: list[GraphDiagnostic] = [
            GraphDiagnostic("parse_error", error) for error in parse_errors
        ]
        diagnostics.extend(
            GraphDiagnostic("schema_validation", error) for error in validation_errors
        )

        by_lifecycle: dict[str, list[Any]] = {}
        edges: dict[str, set[str]] = {}
        origin_workflow_nodes: dict[str, str | None] = {}
        handoff_scopes: dict[str, tuple[str, tuple[str, ...]]] = {}
        for record in records:
            by_lifecycle.setdefault(record.lifecycle_key, []).append(record)
            raw_workflow_node = record.metadata.get("origin_workflow_node", "").strip()
            workflow_node = raw_workflow_node or None
            if workflow_node is not None and not WORKFLOW_NODE_ID.fullmatch(workflow_node):
                diagnostics.append(
                    GraphDiagnostic(
                        "invalid_origin_workflow_node",
                        f"Failure {record.summary_slug} has an invalid origin workflow node",
                        (record.relative_path,),
                    )
                )
                workflow_node = None
            origin_workflow_nodes[record.relative_path] = workflow_node
            raw_link_mode = record.metadata.get("_coordinator_plan_link_mode", "")
            link_mode = raw_link_mode if isinstance(raw_link_mode, str) else ""
            raw_related_code = record.metadata.get("_coordinator_related_code", ())
            related_code = tuple(raw_related_code) if isinstance(raw_related_code, tuple) else ()
            handoff_scopes[record.relative_path] = (link_mode, related_code)
            canonical_status = "open" if record.kind == "failure" else "fixed"
            # Only unresolved handoffs express a live execution dependency.
            # Fixed artifacts remain in the index for audit history, but their
            # former routing must not manufacture a current SCC or depth block.
            if canonical_status == "open":
                origin = self._relative(record.origin_plan)
                fixing = self._relative(record.fixing_plan)
                failure_scope = record.metadata.get("failure_scope", "")
                if failure_scope == LOCAL_FAILURE_SCOPE:
                    if origin.casefold() != fixing.casefold():
                        diagnostics.append(
                            GraphDiagnostic(
                                "invalid_local_failure_scope",
                                f"Failure {record.summary_slug} declares local scope across plans",
                                (record.relative_path,),
                            )
                        )
                else:
                    edges.setdefault(origin, set()).add(fixing)
                if origin.casefold() == fixing.casefold() and failure_scope != LOCAL_FAILURE_SCOPE:
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
        with (
            self.database.transaction()
            if connection is None
            else nullcontext(connection)
        ) as connection:
            connection.execute("DELETE FROM failure_nodes")
            connection.execute("DELETE FROM failure_diagnostics")
            for record in records:
                canonical_status = "open" if record.kind == "failure" else "fixed"
                connection.execute(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, origin_workflow_node, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, plan_link_mode,
                        related_code_json, imported_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                        origin_workflow_nodes[record.relative_path],
                        self._relative(record.fixing_plan),
                        self._relative(record.origin_child_dir),
                        self._relative(record.fixing_child_dir),
                        self._priority(record.kind, canonical_status),
                        handoff_scopes[record.relative_path][0],
                        json.dumps(handoff_scopes[record.relative_path][1]),
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
            return self.audit(connection=connection)

    def _parse_immutable_snapshot(
        self, validator: ModuleType, expected_artifacts: list[dict[str, str]] | None
    ) -> tuple[list[Any], list[str], list[str]]:
        """Read once, hash the same bytes, then parse only an immutable plan copy."""
        plans_root = self.repo_root / "docs" / "plans"
        captured: dict[str, bytes] = {}
        for path in sorted(plans_root.rglob("*.md"), key=lambda item: str(item).casefold()):
            if path.is_file():
                captured[path.relative_to(self.repo_root).as_posix()] = path.read_bytes()
        actual = [
            {"path": path, "hash": hashlib.sha256(content).hexdigest()}
            for path, content in captured.items()
            if _is_failure_artifact(Path(path))
        ]
        if expected_artifacts is not None and actual != expected_artifacts:
            raise CoordinatorError(
                "action_state_changed",
                "Failure artifacts changed after controlled-action confirmation",
            )
        with tempfile.TemporaryDirectory(prefix="zircon-failure-snapshot-") as temporary:
            snapshot_root = Path(temporary)
            for relative, content in captured.items():
                destination = snapshot_root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(content)
            records, parse_errors = validator.parse_handoff_records(snapshot_root)
            validation_errors = validator.validate_repository(snapshot_root)
            self._annotate_handoff_scopes(records, captured)
            normalized = [
                replace(
                    record,
                    artifact_path=self.repo_root / record.artifact_path.relative_to(snapshot_root),
                    origin_plan=self.repo_root / record.origin_plan.relative_to(snapshot_root),
                    fixing_plan=self.repo_root / record.fixing_plan.relative_to(snapshot_root),
                    origin_child_dir=self.repo_root / record.origin_child_dir.relative_to(snapshot_root),
                    fixing_child_dir=self.repo_root / record.fixing_child_dir.relative_to(snapshot_root),
                )
                for record in records
            ]
        return normalized, parse_errors, validation_errors

    def _annotate_handoff_scopes(
        self, records: list[Any], captured: dict[str, bytes] | None = None
    ) -> None:
        """Attach child-record scope metadata from the same imported artifact bytes."""
        for record in records:
            if captured is None:
                try:
                    content = record.artifact_path.read_bytes()
                except OSError:
                    content = b""
            else:
                content = captured.get(record.relative_path, b"")
            link_mode, related_code = self._parse_handoff_scope(content)
            record.metadata["_coordinator_plan_link_mode"] = link_mode
            record.metadata["_coordinator_related_code"] = related_code

    def _parse_handoff_scope(self, content: bytes) -> tuple[str, tuple[str, ...]]:
        """Parse only the two gate-owned frontmatter fields; malformed scope fails closed."""
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError:
            return "", ()
        match = re.match(r"\A---\r?\n(?P<header>.*?)\r?\n---(?:\r?\n|\Z)", text, re.DOTALL)
        if match is None:
            return "", ()
        lines = match.group("header").splitlines()
        link_mode = ""
        values: list[str] = []
        reading_related_code = False
        for line in lines:
            if line.startswith("plan_link_mode:"):
                link_mode = line.partition(":")[2].strip()
                reading_related_code = False
                continue
            if line.startswith("related_code:"):
                reading_related_code = True
                continue
            if reading_related_code:
                item = re.match(r"\s*-\s*(?P<path>\S(?:.*\S)?)\s*$", line)
                if item is None:
                    if line and not line[0].isspace():
                        reading_related_code = False
                    continue
                values.append(item.group("path").strip("`'\""))
        if link_mode != "child_record_only" or not values:
            return link_mode, ()
        try:
            normalized = tuple(
                self._relative(self._resolve_repo_path(value)) for value in values
            )
        except CoordinatorError:
            return link_mode, ()
        return link_mode, normalized

    def audit(self, *, connection: Connection | None = None) -> FailureGraphAudit:
        with (
            self.database.connect()
            if connection is None
            else nullcontext(connection)
        ) as connection:
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

    def open_for_plan(
        self, fixing_plan: str | Path, *, connection: Connection | None = None
    ) -> list[FailureNode]:
        relative = self._relative(self._resolve_repo_path(fixing_plan))
        with (
            self.database.connect()
            if connection is None
            else nullcontext(connection)
        ) as connection:
            rows = connection.execute(
                """
                SELECT * FROM failure_nodes
                WHERE fixing_plan = ? AND kind = 'failure' AND status = 'open'
                ORDER BY priority, created_at, summary_slug, artifact_path
                """,
                (relative,),
            ).fetchall()
        return [self._node_from_row(row) for row in rows]

    @staticmethod
    def _complete_child_record_source_slice(
        node: FailureNode,
        fixing_plan: str,
        manifest_paths: tuple[str, ...],
    ) -> bool:
        """Permit a verified source slice to precede its separate fixed return.

        This narrow exception applies only to a child-record-only handoff whose
        immutable imported ``related_code`` scope is wholly present in the
        candidate manifest.  The failure stays open and continues to block all
        other manifests until a later verified fixed return is recorded.
        """
        return (
            node.fixing_plan == fixing_plan
            and node.plan_link_mode == "child_record_only"
            and bool(node.related_code)
            and set(node.related_code) <= set(manifest_paths)
            and FailureGraphService._has_exact_child_record_evidence(node, manifest_paths)
        )

    @staticmethod
    def _has_exact_child_record_evidence(
        node: FailureNode, manifest_paths: tuple[str, ...]
    ) -> bool:
        extras = set(manifest_paths) - set(node.related_code)
        if len(extras) != 1:
            return False
        record = Path(next(iter(extras)))
        return (
            record.suffix.casefold() == ".md"
            and bool(re.fullmatch(r"\d{4}-\d{2}-\d{2}-.+\.md", record.name))
            and not _is_failure_artifact(record)
            and record.as_posix().startswith(node.fixing_child_dir.rstrip("/") + "/")
        )

    def open_related_to_plan(self, plan_path: str | Path) -> list[FailureNode]:
        relative = self._relative(self._resolve_repo_path(plan_path))
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT * FROM failure_nodes
                WHERE (fixing_plan = ? OR origin_plan = ?)
                  AND kind = 'failure' AND status = 'open'
                ORDER BY priority, created_at, summary_slug, artifact_path
                """,
                (relative, relative),
            ).fetchall()
        return [self._node_from_row(row) for row in rows]

    def open_related_to_workflow_nodes(
        self, plan_path: str | Path, workflow_node_keys: tuple[str, ...]
    ) -> list[FailureNode]:
        """Return fixer-priority failures plus origin failures for exact workflow nodes."""
        return self.open_for_manifest(plan_path, workflow_node_keys, ())

    def open_for_manifest(
        self,
        plan_path: str | Path,
        workflow_node_keys: tuple[str, ...],
        manifest_paths: tuple[str, ...] | list[str],
    ) -> list[FailureNode]:
        """Select only node-applicable failures for a verified fixed-return manifest."""
        relative = self._relative(self._resolve_repo_path(plan_path))
        node_keys = tuple(sorted(set(workflow_node_keys), key=str.casefold))
        if not node_keys:
            raise CoordinatorError(
                "workflow_failure_nodes_empty",
                "Workflow Failure filtering requires at least one node key",
            )
        normalized_manifest = tuple(
            sorted(
                {
                    self._relative(self._resolve_repo_path(path))
                    for path in manifest_paths
                },
                key=str.casefold,
            )
        )
        placeholders = ", ".join("?" for _ in node_keys)
        fixed_return = False
        with self.database.connect() as connection:
            if normalized_manifest:
                manifest_placeholders = ", ".join("?" for _ in normalized_manifest)
                fixed_return = connection.execute(
                    f"""SELECT 1 FROM failure_nodes
                        WHERE kind='fixed' AND status='fixed' AND fixing_plan=?
                          AND artifact_path IN ({manifest_placeholders})
                        LIMIT 1""",
                    (relative, *normalized_manifest),
                ).fetchone() is not None
            conditions = [
                f"""(
                    origin_plan = ?
                    AND (
                      origin_workflow_node IS NULL
                      OR origin_workflow_node IN ({placeholders})
                    )
                )"""
            ]
            parameters: list[object] = [relative, *node_keys]
            if not fixed_return:
                conditions.insert(0, "fixing_plan = ?")
                parameters.insert(0, relative)
            rows = connection.execute(
                f"""
                SELECT * FROM failure_nodes
                WHERE kind = 'failure' AND status = 'open'
                  AND ({' OR '.join(conditions)})
                ORDER BY priority, created_at, summary_slug, artifact_path
                """,
                tuple(parameters),
            ).fetchall()
        nodes = [self._node_from_row(row) for row in rows]
        return [
            node
            for node in nodes
            if not self._complete_child_record_source_slice(
                node, relative, normalized_manifest
            )
        ]

    def validator_errors(self) -> list[str]:
        return list(self._validator_module().validate_repository(self.repo_root))

    def validator_errors_for_plan(self, plan_path: str | Path) -> list[str]:
        validator = self._validator_module()
        plan = self._resolve_repo_path(plan_path)
        plan_relative = self._relative(plan)
        match = re.match(r"^(\d+)-", plan.name)
        child_relative = self._relative(plan.parent / match.group(1)) if match else ""
        records, _parse_errors = validator.parse_handoff_records(self.repo_root)
        related_artifacts = {
            record.relative_path.casefold()
            for record in records
            if plan.resolve() in {record.origin_plan.resolve(), record.fixing_plan.resolve()}
        }
        markers = {plan_relative.casefold(), *related_artifacts}
        if child_relative:
            markers.add(child_relative.casefold() + "/")
        return [
            error
            for error in validator.validate_repository(self.repo_root)
            if any(marker in error.replace("\\", "/").casefold() for marker in markers)
        ]

    def materialize_local_validation_failure(
        self,
        *,
        origin_plan: str | Path,
        summary_slug: str,
        source_slice: str,
        reproduction: str,
        lowest_known_cause: str,
        acceptance_criteria: tuple[str, ...] | list[str],
        related_code: tuple[str, ...] | list[str],
        created_at: date | None = None,
    ) -> Path:
        """Persist a validation failure as a same-plan, forward-fix work item.

        Local failures deliberately have no graph dependency edge: the same
        numbered Plan owns both the already-integrated snapshot and its repair.
        The child-only record lets the coordinator wake that Plan without
        mutating the shared plan definition or blocking unrelated Sessions.
        """
        owner = PlanRepository(self.repo_root).resolve_owner(origin_plan)
        normalized_slug = self._require_failure_text("summary_slug", summary_slug)
        if SUMMARY_SLUG.fullmatch(normalized_slug) is None:
            raise CoordinatorError(
                "failure_summary_slug_invalid",
                "Validation failure summary_slug must be lowercase hyphenated text",
            )
        normalized_slice = self._require_failure_text("source_slice", source_slice)
        normalized_reproduction = self._require_failure_text("reproduction", reproduction)
        normalized_cause = self._require_failure_text("lowest_known_cause", lowest_known_cause)
        normalized_acceptance = tuple(
            self._require_failure_text("acceptance_criteria", item)
            for item in acceptance_criteria
        )
        if not normalized_acceptance:
            raise CoordinatorError(
                "failure_acceptance_empty",
                "Validation failure requires at least one forward-repair acceptance criterion",
            )
        normalized_related_code = self._normalize_related_code(related_code)
        stamp = created_at or date.today()
        child_dir = self.repo_root / owner.child_dir
        if not child_dir.is_dir():
            raise CoordinatorError(
                "failure_child_directory_missing",
                f"Numbered Plan child directory does not exist: {owner.child_dir}",
            )
        artifact = child_dir / f"failure-{stamp.isoformat()}-{normalized_slug}.md"
        if artifact.exists():
            content = self._update_local_validation_failure(
                artifact,
                owner_path=owner.path,
                source_slice=normalized_slice,
                reproduction=normalized_reproduction,
            )
        else:
            content = self._local_validation_failure_content(
                owner_path=owner.path,
                child_dir=owner.child_dir,
                created_at=stamp,
                summary_slug=normalized_slug,
                source_slice=normalized_slice,
                reproduction=normalized_reproduction,
                lowest_known_cause=normalized_cause,
                acceptance_criteria=normalized_acceptance,
                related_code=normalized_related_code,
            )
        self._atomic_write(artifact, content)
        self._index_local_validation_failure(
            artifact=artifact,
            owner_path=owner.path,
            child_dir=owner.child_dir,
            created_at=stamp,
            summary_slug=normalized_slug,
            related_code=normalized_related_code,
        )
        return artifact

    def _index_local_validation_failure(
        self,
        *,
        artifact: Path,
        owner_path: str,
        child_dir: str,
        created_at: date,
        summary_slug: str,
        related_code: tuple[str, ...],
    ) -> None:
        """Upsert the coordinator-owned local failure without a global rescan.

        Validation completion is on the latency-sensitive path.  The generated
        artifact has already passed the same input validation as its frontmatter,
        so rebuilding the entire failure graph here only delays the caller and
        can turn a successfully observed test failure into a client timeout.
        A later explicit graph import remains the full-repository audit path.
        """
        relative_artifact = self._relative(artifact)
        owner_absolute = (self.repo_root / owner_path).resolve().as_posix().casefold()
        lifecycle_key = "|".join((owner_absolute, owner_absolute, summary_slug))
        imported_at = utc_text()
        with self.database.transaction() as connection:
            duplicate = connection.execute(
                """
                SELECT artifact_path FROM failure_nodes
                WHERE lifecycle_key = ? AND artifact_path <> ?
                """,
                (lifecycle_key, relative_artifact),
            ).fetchone()
            if duplicate is not None:
                raise CoordinatorError(
                    "failure_lifecycle_conflict",
                    "A local validation failure already owns this lifecycle key",
                    details={
                        "artifact": relative_artifact,
                        "existingArtifact": str(duplicate["artifact_path"]),
                    },
                )
            connection.execute(
                """
                INSERT INTO failure_nodes(
                    lifecycle_key, artifact_path, kind, status, created_at,
                    resolved_at, summary_slug, origin_plan, origin_workflow_node,
                    fixing_plan, origin_child_dir, fixing_child_dir, priority,
                    plan_link_mode, related_code_json, imported_at
                ) VALUES (?, ?, 'failure', 'open', ?, NULL, ?, ?, NULL, ?, ?, ?, 0, ?, ?, ?)
                ON CONFLICT(artifact_path) DO UPDATE SET
                    lifecycle_key=excluded.lifecycle_key,
                    kind=excluded.kind,
                    status=excluded.status,
                    created_at=excluded.created_at,
                    resolved_at=excluded.resolved_at,
                    summary_slug=excluded.summary_slug,
                    origin_plan=excluded.origin_plan,
                    origin_workflow_node=excluded.origin_workflow_node,
                    fixing_plan=excluded.fixing_plan,
                    origin_child_dir=excluded.origin_child_dir,
                    fixing_child_dir=excluded.fixing_child_dir,
                    priority=excluded.priority,
                    plan_link_mode=excluded.plan_link_mode,
                    related_code_json=excluded.related_code_json,
                    imported_at=excluded.imported_at
                """,
                (
                    lifecycle_key,
                    relative_artifact,
                    created_at.isoformat(),
                    summary_slug,
                    owner_path,
                    owner_path,
                    child_dir,
                    child_dir,
                    "child_record_only",
                    json.dumps(related_code),
                    imported_at,
                ),
            )

    @staticmethod
    def _require_failure_text(field_name: str, value: object) -> str:
        if not isinstance(value, str) or not value.strip():
            raise CoordinatorError(
                "failure_materialization_invalid",
                f"Validation failure {field_name} must be non-empty text",
            )
        return value.strip()

    @staticmethod
    def _normalize_related_code(values: tuple[str, ...] | list[str]) -> tuple[str, ...]:
        normalized: set[str] = set()
        for value in values:
            if not isinstance(value, str):
                raise CoordinatorError(
                    "failure_related_code_invalid",
                    "Validation failure related_code paths must be text",
                )
            candidate = value.strip().replace("\\", "/")
            path = PurePosixPath(candidate)
            if (
                not candidate
                or path.is_absolute()
                or any(part in {"", ".", ".."} for part in path.parts)
            ):
                raise CoordinatorError(
                    "failure_related_code_invalid",
                    "Validation failure related_code paths must be safe repo-relative paths",
                )
            normalized.add(path.as_posix())
        if not normalized:
            raise CoordinatorError(
                "failure_related_code_empty",
                "Validation failure requires at least one owned related_code path",
            )
        return tuple(sorted(normalized, key=str.casefold))

    def _update_local_validation_failure(
        self,
        artifact: Path,
        *,
        owner_path: str,
        source_slice: str,
        reproduction: str,
    ) -> str:
        content = artifact.read_text(encoding="utf-8")
        header, _errors = self._validator_module()._parse_frontmatter(artifact, content)
        if (
            header.get("failure_scope") != LOCAL_FAILURE_SCOPE
            or header.get("origin_plan") != owner_path
            or header.get("fixing_plan") != owner_path
        ):
            raise CoordinatorError(
                "failure_artifact_conflict",
                "Existing validation failure artifact has incompatible ownership or scope",
                details={"artifact": self._relative(artifact)},
            )
        evidence = f"- 验证回写：`{source_slice}` — {reproduction}"
        if evidence in content:
            return content
        marker = "## 最低共享层根因"
        if marker not in content:
            raise CoordinatorError(
                "failure_artifact_conflict",
                "Existing local validation failure is missing its root-cause section",
                details={"artifact": self._relative(artifact)},
            )
        return content.replace(marker, f"{evidence}\n\n{marker}", 1)

    @staticmethod
    def _local_validation_failure_content(
        *,
        owner_path: str,
        child_dir: str,
        created_at: date,
        summary_slug: str,
        source_slice: str,
        reproduction: str,
        lowest_known_cause: str,
        acceptance_criteria: tuple[str, ...],
        related_code: tuple[str, ...],
    ) -> str:
        related_lines = "\n".join(f"  - {path}" for path in related_code)
        acceptance_lines = "\n".join(f"- {criterion}" for criterion in acceptance_criteria)
        return f"""---
handoff_kind: failure
status: open
failure_scope: local
created_at: {created_at.isoformat()}
summary_slug: {summary_slug}
origin_plan: {owner_path}
fixing_plan: {owner_path}
origin_child_dir: {child_dir}
fixing_child_dir: {child_dir}
plan_link_mode: child_record_only
related_code:
{related_lines}
---

# {summary_slug}: 验证失败回写

## 来源执行者

- 来源计划：`{owner_path}`
- 来源执行切片：{source_slice}
- 修复责任计划：`{owner_path}`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`{source_slice}` — {reproduction}

## 最低共享层根因

{lowest_known_cause}

## 架构修复验收

{acceptance_lines}

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.
"""

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
        if self._is_child_record_only(source_text):
            return self._return_child_record_only(
                node,
                source=source,
                source_text=source_text,
                destination=destination,
                resolution=resolution,
                resolved_at=resolved_at,
            )
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

    def _return_child_record_only(
        self,
        node: FailureNode,
        *,
        source: Path,
        source_text: str,
        destination: Path,
        resolution: FailureResolution,
        resolved_at: date,
    ) -> Path:
        """Return a child-only handoff without turning parent plans into shared write hotspots."""
        receipt = (
            self.repo_root
            / node.fixing_child_dir
            / f"{resolved_at.isoformat()}-{node.summary_slug}-return.md"
        )
        if receipt.exists():
            raise CoordinatorError(
                "return_receipt_exists",
                f"Child-record return receipt already exists: {self._relative(receipt)}",
            )
        fixed_text = self._fixed_content(source_text, resolution, resolved_at)
        receipt_link = Path(os.path.relpath(destination, receipt.parent)).as_posix()
        receipt_text = "\n".join(
            (
                "---",
                "record_kind: failure_return_status",
                "status: fixed",
                f"resolved_at: {resolved_at.isoformat()}",
                f"summary_slug: {node.summary_slug}",
                f"origin_plan: {node.origin_plan}",
                f"fixing_plan: {node.fixing_plan}",
                "plan_link_mode: child_record_only",
                f"source_artifact: {node.artifact_path}",
                "---",
                "",
                f"# {node.summary_slug} 回传摘要",
                "",
                "- 状态：`fixed`",
                f"- 回传工件：[{destination.name}]({receipt_link})",
                f"- 摘要：{resolution.return_summary}",
                "",
            )
        )
        try:
            self._atomic_write(destination, fixed_text)
            self._atomic_write(receipt, receipt_text)
            source.unlink()
            self.import_repository()
        except BaseException:
            self._restore_text(source, source_text)
            destination.unlink(missing_ok=True)
            receipt.unlink(missing_ok=True)
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

    @staticmethod
    def _is_child_record_only(content: str) -> bool:
        lines = content.splitlines()
        if not lines or lines[0].strip() != "---":
            return False
        end = next(
            (index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---"),
            None,
        )
        if end is None:
            return False
        for line in lines[1:end]:
            key, separator, value = line.partition(":")
            if separator and key.strip() == "plan_link_mode":
                return value.strip() == "child_record_only"
        return False

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
        destination_link = Path(os.path.relpath(destination, plan_path.parent)).as_posix()
        source_resolved = source.resolve()
        lines = content.splitlines()
        replaced = False
        output: list[str] = []
        for line in lines:
            link_count = len(MARKDOWN_LINK.findall(line))
            replaced_on_line = 0

            def replace_link(match: re.Match[str]) -> str:
                nonlocal replaced, replaced_on_line
                raw_target = match.group(1)
                target = raw_target.strip().strip("<>").split("#", 1)[0]
                if not target:
                    return match.group(0)
                candidate = (plan_path.parent / Path(target)).resolve()
                if candidate != source_resolved:
                    return match.group(0)
                replaced = True
                replaced_on_line += 1
                return f"[fixed 已修复：{slug}]({destination_link})"

            rewritten = MARKDOWN_LINK.sub(replace_link, line)
            if rewritten == line:
                output.append(line)
            elif line.lstrip().startswith("|") or replaced_on_line < link_count:
                # Tables and multi-link bullets carry evidence beyond this one
                # handoff. Preserve the line and rewrite only matching tokens.
                output.append(rewritten)
            else:
                # Ordinary handoff bullets deliberately collapse to one concise
                # fixed summary once the canonical artifact has moved.
                output.append(replacement)
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
            origin_workflow_node=row["origin_workflow_node"],
            fixing_plan=row["fixing_plan"],
            origin_child_dir=row["origin_child_dir"],
            fixing_child_dir=row["fixing_child_dir"],
            priority=int(row["priority"]),
            plan_link_mode=row["plan_link_mode"],
            related_code=tuple(json.loads(row["related_code_json"])),
        )

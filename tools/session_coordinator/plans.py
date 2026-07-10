from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from .models import CoordinatorError


PLAN_DEFINITION = re.compile(r"^(\d{2}[a-z]?)-.+\.md$", re.IGNORECASE)
NUMBERED_CHILD_DIR = re.compile(r"^\d{2}[a-z]?$", re.IGNORECASE)


@dataclass(frozen=True, slots=True)
class PlanDocument:
    path: str
    plan_id: str
    child_dir: str


@dataclass(frozen=True, slots=True)
class PlanInventory:
    formal_plans: tuple[PlanDocument, ...]
    legacy_documents: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class PlanWriteDecision:
    allowed: bool
    code: str
    message: str
    owner_child_dir: str | None


class PlanRepository:
    def __init__(self, repo_root: str | Path):
        self.repo_root = Path(repo_root).resolve()
        self.formal_root = self.repo_root / "docs" / "plans"
        self.legacy_root = self.repo_root / ".codex" / "plans"

    def scan(self) -> PlanInventory:
        formal: list[PlanDocument] = []
        if self.formal_root.is_dir():
            for path in self.formal_root.rglob("*.md"):
                match = PLAN_DEFINITION.match(path.name)
                if match is None or NUMBERED_CHILD_DIR.fullmatch(path.parent.name):
                    continue
                plan_id = match.group(1).lower()
                child = path.parent / plan_id
                formal.append(
                    PlanDocument(
                        path=self._relative(path),
                        plan_id=plan_id,
                        child_dir=self._relative(child),
                    )
                )
        legacy = (
            [self._relative(path) for path in self.legacy_root.rglob("*.md")]
            if self.legacy_root.is_dir()
            else []
        )
        return PlanInventory(
            formal_plans=tuple(sorted(formal, key=lambda item: item.path.casefold())),
            legacy_documents=tuple(sorted(legacy, key=str.casefold)),
        )

    def resolve_owner(self, plan_path: str | Path) -> PlanDocument:
        absolute = self._resolve_repo_path(plan_path)
        if not absolute.is_file():
            raise CoordinatorError("plan_not_found", f"Plan does not exist: {self._relative(absolute)}")
        match = PLAN_DEFINITION.match(absolute.name)
        if match is None or NUMBERED_CHILD_DIR.fullmatch(absolute.parent.name):
            raise CoordinatorError(
                "not_numbered_plan",
                f"Plan must be a numbered definition: {self._relative(absolute)}",
            )
        plan_id = match.group(1).lower()
        return PlanDocument(
            path=self._relative(absolute),
            plan_id=plan_id,
            child_dir=self._relative(absolute.parent / plan_id),
        )

    def authorize_write(
        self,
        session_plan_path: str | Path,
        target_path: str | Path,
        *,
        maintenance: bool = False,
    ) -> PlanWriteDecision:
        try:
            target = self._resolve_repo_path(target_path)
            owner = self.resolve_owner(session_plan_path)
        except CoordinatorError as error:
            return PlanWriteDecision(False, error.code, error.message, None)
        owner_child = self.repo_root / owner.child_dir
        owner_relative = owner.child_dir
        if not target.is_relative_to(self.formal_root):
            return PlanWriteDecision(
                False,
                "outside_plan_root",
                "Plan authorization applies only under docs/plans",
                owner_relative,
            )
        if maintenance:
            return PlanWriteDecision(
                True,
                "maintenance_allowed",
                "Explicit maintenance mode may update repository-bounded plan files",
                owner_relative,
            )
        if self._is_plan_definition(target):
            return PlanWriteDecision(
                False,
                "protected_plan_definition",
                "Numbered plan definitions are read-only for business Sessions",
                owner_relative,
            )
        if target.name.casefold() == "index.md" or target.name.casefold().startswith(
            "engine-code-"
        ):
            return PlanWriteDecision(
                False,
                "protected_global_plan",
                "Global plan indexes and engine-code summaries are read-only",
                owner_relative,
            )
        if not target.is_relative_to(owner_child):
            return PlanWriteDecision(
                False,
                "outside_registered_child",
                f"Session output must stay under {owner_relative}",
                owner_relative,
            )
        return PlanWriteDecision(
            True,
            "child_output_allowed",
            f"Write belongs to {owner_relative}",
            owner_relative,
        )

    def protected_reason(self, target_path: str | Path) -> str | None:
        target = self._resolve_repo_path(target_path)
        if self._is_plan_definition(target):
            return "protected_plan_definition"
        if target.name.casefold() == "index.md" or target.name.casefold().startswith(
            "engine-code-"
        ):
            return "protected_global_plan"
        return None

    def _is_plan_definition(self, path: Path) -> bool:
        return (
            path.is_relative_to(self.formal_root)
            and PLAN_DEFINITION.match(path.name) is not None
            and NUMBERED_CHILD_DIR.fullmatch(path.parent.name) is None
        )

    def _resolve_repo_path(self, value: str | Path) -> Path:
        raw = Path(value)
        candidate = raw.resolve() if raw.is_absolute() else (self.repo_root / raw).resolve()
        if not candidate.is_relative_to(self.repo_root):
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}")
        return candidate

    def _relative(self, path: Path) -> str:
        return path.resolve().relative_to(self.repo_root).as_posix()

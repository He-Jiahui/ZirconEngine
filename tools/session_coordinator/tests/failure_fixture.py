from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True, slots=True)
class FixturePlan:
    path: Path
    child: Path


class FailureGraphFixture:
    def __init__(self, root: Path):
        self.root = root

    def add_plan(self, relative: str) -> FixturePlan:
        path = self.root / relative
        plan_id = path.name.split("-", 1)[0]
        child = path.parent / plan_id
        child.mkdir(parents=True, exist_ok=True)
        path.write_text(f"# {path.stem}\n", encoding="utf-8")
        return FixturePlan(path, child)

    def add_handoff(
        self,
        origin: FixturePlan,
        fixing: FixturePlan,
        slug: str,
        *,
        kind: str = "failure",
        created_at: str = "2026-07-11",
        resolved_at: str = "2026-07-12",
    ) -> Path:
        artifact_date = created_at if kind == "failure" else resolved_at
        parent = fixing.child if kind == "failure" else origin.child
        artifact = parent / f"{kind}-{artifact_date}-{slug}.md"
        metadata = {
            "handoff_kind": kind,
            "status": "open" if kind == "failure" else "fixed",
            "created_at": created_at,
            "summary_slug": slug,
            "origin_plan": origin.path.relative_to(self.root).as_posix(),
            "fixing_plan": fixing.path.relative_to(self.root).as_posix(),
            "origin_child_dir": origin.child.relative_to(self.root).as_posix(),
            "fixing_child_dir": fixing.child.relative_to(self.root).as_posix(),
        }
        if kind == "fixed":
            metadata["resolved_at"] = resolved_at
        frontmatter = "\n".join(f"{key}: {value}" for key, value in metadata.items())
        result = (
            "待修复"
            if kind == "failure"
            else "- 根因：shared owner selected the wrong identity.\n"
            "- 架构修复：one canonical constructor now owns the identity.\n"
            "- 验证：lower, reproduction, and upward gates passed.\n"
            "- 回传：origin gate resumed."
        )
        body = f"""# Cross-plan handoff

## 来源执行者

- 来源计划：`{metadata['origin_plan']}`
- 来源执行切片：M3 fixture
- 修复责任计划：`{metadata['fixing_plan']}`
- 交接原因：lowest shared owner

## 失败现象与复现证据

The exact reproduction failed before repair.

## 最低共享层根因

The fixing plan owns the shared identity constructor.

## 架构修复验收

- Lower-layer regression passes.
- Original reproduction passes.
- Upward gate passes.

## 禁止临时方案

- No fallback, alias, shim, or test bypass.

## 修复结果与回传

{result}
"""
        artifact.write_text(f"---\n{frontmatter}\n---\n\n{body}", encoding="utf-8")
        status = "open 待修复" if kind == "failure" else "fixed 已修复"
        for plan in (origin.path, fixing.path):
            link = Path(os.path.relpath(artifact, plan.parent)).as_posix()
            with plan.open("a", encoding="utf-8") as stream:
                stream.write(f"\n- {status}：[{slug}]({link})\n")
        return artifact

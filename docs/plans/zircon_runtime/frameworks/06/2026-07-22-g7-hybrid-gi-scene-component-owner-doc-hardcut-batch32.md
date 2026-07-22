---
related_code:
  - docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md
  - zircon_runtime/src/scene/components/scene/mod.rs
implementation_files:
  - docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md
tests:
  - python -B tools/check_conventions.py --repo-root E:\Git\ZirconEngine --only docs
  - git diff --check -- docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md docs/plans/zircon_runtime/frameworks/06/2026-07-22-g7-hybrid-gi-scene-component-owner-doc-hardcut-batch32.md
---

# Frameworks06 G7 Hybrid GI Scene Component Owner 文档硬切 Batch 32

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: ["docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md"]
Date: 2026-07-22
Session: `frameworks06-g7-hybrid-gi-scene-owner-batch32-20260722`

## Scope Delivered

- 将 Hybrid GI 架构文档 `related_code` 与 `implementation_files` 中两条已删除的 flat `zircon_runtime/src/scene/components/scene.rs` 机器路径，硬切到现存 folder-backed facade `zircon_runtime/src/scene/components/scene/mod.rs`。
- `scene/mod.rs` 是 scene component family 的唯一 curated facade，直接声明并再导出 identity、hierarchy、transform、activation、camera、mesh renderer、lighting、physics、post-process 与 animation 等当前 owners，符合该 Hybrid GI 文档对整组 scene component contract 的引用粒度。
- 旧 flat owner 继续物理不存在；本批不创建 alias、shim、兼容 include 或重复聚合 owner，也不修改 Runtime、Render 或 Hybrid GI 行为。

## Fresh Testing Evidence

- 修改前 fresh G7：全局 `583` violations；目标文档正好 `2` 个 missing-path violations，均指向删除的 `scene/components/scene.rs`。
- 修改后 fresh G7：全局 `581` violations / `1,875` documents / `67,815` checked paths；目标文档 focused violations 为 `0`。
- 目标文档中退役 flat owner 计数为 `0`，current folder facade 计数为 `2`，且 `scene/components/scene/mod.rs` 实际存在。

## Review

- coordinator snapshot `758` exact2 独立复审为 `Critical 0 / Important 0 / Minor 0 — ready`。
- 复审确认 `scene/mod.rs` 声明并 curated re-export Hybrid GI 文档引用的完整 scene component family；本批只替换两条机器 owner 路径，未增加 alias、shim、compatibility 或重复 owner。

## Milestone Decision

本批 focused G7 与独立复审均已通过，当前状态为 `accepted`，等待协调器管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`；其余 `581` 条 current-source violations 属于独立或并发 owner 范围，不以本切片冒充完成。

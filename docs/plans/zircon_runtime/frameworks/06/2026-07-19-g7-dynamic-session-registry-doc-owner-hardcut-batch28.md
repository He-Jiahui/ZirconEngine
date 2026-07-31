---
related_code:
  - docs/zircon_runtime/dynamic_api/session.md
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/registry/mod.rs
implementation_files:
  - docs/zircon_runtime/dynamic_api/session.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/zircon_runtime/dynamic_api/session.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-dynamic-session-registry-doc-owner-hardcut-batch28.md
---

# Frameworks06 G7 Dynamic Session Registry 文档 Owner 硬切 Batch 28

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M1
Status: accepted
Files: ["docs/zircon_runtime/dynamic_api/session.md"]
Date: 2026-07-19
Session: `frameworks06-g7-dynamic-session-registry-doc-owner-hardcut-batch28-20260719`

## Scope Delivered

- 将 Dynamic API session 文档中两条指向已删除 flat `dynamic_api/session/registry.rs` 的机器路径硬切到 folder-backed `registry/mod.rs` current owner。
- 同步正文中的 global registry、handle allocation、poison-safe lock 与 `with_session` dispatch owner；`session.rs` 继续只拥有 ABI session lifecycle，不恢复 flat owner、alias、shim 或兼容 include。
- 本批只修正文档中已经失真的 cross-module owner 事实，不复制 registry 子文件实现细节，也不修改 Runtime 行为。

## Fresh Testing Evidence

- 修改前 fresh G7：目标文档有 `2` 个 missing-path violations，均指向已删除 `dynamic_api/session/registry.rs`；同一时序全局 `588` violations / `139` documents / `67,426` checked paths。
- 修改后 fresh G7：目标文档与本记录 focused `0` violations；同一时序全局降到 `586` violations / `138` documents / `67,430` checked paths，继续保持 RED。
- 首轮独立复审在稳定 exact2 指纹 `1708b31ec0c52b1004acd61cc6e0ccc6296b140dd70306cd265c8c0849298bd4` 上返回 `C0 / I1 / M0`：机器路径虽已清零，但正文仍有两处把 registry/lock helper 归给 `session.rs` 的语义旧 owner；本批随后将两处统一硬切到 `session/registry/mod.rs`，没有恢复旧文件或兼容出口。
- 语义修正后的 fresh G7 仍为 focused `0` violations；同一时序全局 `590` violations / `139` documents / `67,470` checked paths，继续保持 RED，变化来自并发外部文档输入，不能归因于本 exact2。
- 目标文档中退役 flat owner 的 machine path、首轮复审指出的两种 stale-owner 句式均为 `0`；current owner 声明与 `registry/mod.rs` 中 `SESSION_REGISTRY`、`SessionRegistry`、`lock_registry`、`lock_session`、`with_session` 定义全部对齐；exact-scope `git diff --check` 通过，仅输出工作树既有 LF/CRLF 提示。

## Review

- 首轮 exact2 独立复审 `C0 / I1 / M0` 的唯一阻塞项已修正。
- successor immutable snapshot `647` 独立双遍复审为 `C0 / I0 / M0 — Ready`；record `731bf3527a009099fd7553c939cf0d20a23db2f6c9d56f0f79ad6393d66dcf14`、session doc `3214c0731a6203d1c9d673ff5f10be38e7d8b96faefb273a6a8ec08d2548063d` 与 ordinal fingerprint `a69a6ec4cceecd1915fb72f33d750ec783e7ba4bcd1d4a43c8ed4d766286f45d` 两遍稳定无漂移；current source owner、no alias/shim 与 plan status 均通过。

## Milestone Decision

本批 focused G7 与独立复审已通过，状态记为 `accepted`，等待通过协调器纳入 Frameworks06 管理提交。Frameworks06 M1、全局 G7 与计划 06 均保持 `in_progress`，不得以单份 durable 文档同步冒充里程碑完成。

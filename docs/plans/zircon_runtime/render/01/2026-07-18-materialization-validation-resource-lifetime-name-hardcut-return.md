---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-07-18
summary_slug: materialization-validation-resource-lifetime-name-hardcut
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
plan_link_mode: child_record_only
---

# materialization-validation-resource-lifetime-name-hardcut 回传摘要

Plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
Milestone: M2
Status: completed
Files: ["zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs", "docs/plans/zircon_runtime/runtime/12/fixed-2026-07-18-materialization-validation-resource-lifetime-name-hardcut.md"]

## Scope Delivered

- 状态：`fixed`
- 回传工件：[fixed-2026-07-18-materialization-validation-resource-lifetime-name-hardcut.md](../../runtime/12/fixed-2026-07-18-materialization-validation-resource-lifetime-name-hardcut.md)
- 摘要：Runtime12 current-source canonical library check can resume past the Render01 materialization lifetime lookup compile blocker; no Runtime12 or Render17 source was absorbed.

## Fresh Testing Evidence

- Runtime12 source-bound canonical job `f6841642e70c4a43b8674c92f9f18461` / run `230eaecd12ce4bfe97d92753efff6cdc` 在 630-path manifest 上 exit 0；post-run 630/630 hashes match，relevant dirty outside manifest 为 0。
- `rustfmt +1.94.1 --check zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs` exit 0。
- 精确三路径 `git diff --check` exit 0。

## Review

- 独立只读审查结论为 critical 0 / important 0 / minor 0；确认调用最低真实索引 owner `resource_lifetime_by_name`，未引入兼容 shim，且未吸收 Runtime12 或 Render17 源文件。

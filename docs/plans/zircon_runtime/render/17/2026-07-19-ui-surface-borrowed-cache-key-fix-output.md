---
record_kind: milestone_output
status: completed
completed_at: 2026-07-19
plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
milestone: M4
slice: ui-surface-borrowed-cache-key-compile-fix
source_manifest_fingerprint: 42216a3cdd88bbed30369bcce67ad5698effc2dfccfca6f08722e22ce27b1e44
related_code:
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1
---

# UI surface borrowed cache key compile fix

Plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
Milestone: M4
Status: completed
Files: ["zircon_runtime/src/rhi_wgpu/ui_surface.rs", "docs/plans/zircon_runtime/render/17/2026-07-19-rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker-return.md", "docs/plans/zircon_runtime/runtime/12/fixed-2026-07-19-rhi-wgpu-ui-surface-borrowed-cache-key-compile-blocker.md"]

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M4 | `WgpuUiSurfaceRenderer` 图片缓存借用键最低修复 | `completed` | 2026-07-19 | job `f6841642e70c4a43b8674c92f9f18461` / run `230eaecd12ce4bfe97d92753efff6cdc`，630 路径 fingerprint `42216a3c...`，released exit 0/no PID，post-run 630/630 匹配 |

## Scope Delivered

- 将 `HashMap<String, _>` 查找从多余的 `get(&cache_key)` 硬切为真实借用合同 `get(cache_key)`，不增加 clone、兼容 shim 或缓存 owner。
- canonical failure 已返回 Runtime12 fixed 工件；独立复审为 `C0/I0/M0`。

## Fresh Testing Evidence

- `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1`：job `f6841642e70c4a43b8674c92f9f18461` / run `230eaecd12ce4bfe97d92753efff6cdc`，released exit 0/no PID，21.53s。
- 630 路径 source manifest fingerprint 为 `42216a3cdd88bbed30369bcce67ad5698effc2dfccfca6f08722e22ce27b1e44`；post-run 630/630 hash 匹配且 manifest 外 relevant dirty 为 0。

## Review

- 独立 exact4 复审：`C0/I0/M0`。

## 边界

- 本记录只关闭 `ui-surface-borrowed-cache-key-compile-fix` slice，不宣称完整 Render17 M4/PF-M4 完成。
- Render17 pairwise-overlap batching failure 仍保持 open，未被本次修复吸收。

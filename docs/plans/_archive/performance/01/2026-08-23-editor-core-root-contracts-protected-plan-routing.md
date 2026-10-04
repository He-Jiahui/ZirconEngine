---
related_code:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/core/mod.rs
canonical_review:
  - docs/plans/performance/01/2026-08-23-editor-core-root-contracts-currentness-revalidation.md
protected_targets:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Editor core根合同保护计划路由（2026-08-23）

## 请求索引更新

将4个root contracts记录为4/4 Rust文件、635 physical lines、16,505 bytes、0 inline tests、SHA
`3d05a00c16f8c6cc659f14f48619868015572e262caf1be9beb6148481ba7f97`，状态为
`static_current_revalidated / dynamic_pending`。不新增performance ID：

- authoring descriptor的逐builder normalize并入Editor12/`PERF-MVP-538`的一次prepared candidate；
- operation payload clone/retention继续归Editor03/Editor04的`PERF-MVP-067/551`；
- startup filesystem/authority工作继续归Editor16/Editor10的`PERF-MVP-075/100`；
- `mod.rs`无运行时性能任务。

`pending.md`应只保留一个简短module row；在descriptor 10K scale、payload retention、F0 startup、managed
Cargo和F0/F4动态门通过前不写入`review.md`。本会话不修改受保护文件或owner plans。

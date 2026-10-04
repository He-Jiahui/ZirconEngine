---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/125-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/125/2026-09-19-tool-scheduler-revoke-capacity.md
---

# Editor828 · Tool scheduler revoke capacity

| Slice | Status | Local evidence | Managed gate |
| --- | --- | --- | --- |
| Active-lease and queued-request revoke scratch/output capacity | implemented_pending_validation | TDD source/model contract `3/3`; lower owner/kind filtering regression and `EDITOR828_TOOL_SCHEDULER_REVOKE_CAPACITY_BENCH_V1` marker wired; lazy no-match paths and exact output bounds preserve revoke order and semantics; batched Runtime/Editor receipt is recorded in the async admission log | Managed Cargo/Windows Release and scheduler product p50/p95/p99 remain pending behind external `E:\Git\zr_vm` dirt |

The change is intentionally limited to scheduler allocation shape. Tooling
remains out of scope until the planned Rust migration.

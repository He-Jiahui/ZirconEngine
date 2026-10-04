---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/125-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/125/2026-09-19-tool-scheduler-promotion-capacity.md
---

# Editor827 · Tool scheduler promotion capacity

| Slice | Status | Local evidence | Managed gate |
| --- | --- | --- | --- |
| Queue/set and single-claim promotion vectors lazily reserve known bounds | implemented_pending_validation | RED→GREEN source contract `3/3`; lower FIFO promotion regression and Release marker wired; empty/no-op release remains allocation-free; batched Runtime/Editor receipt recorded in `docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md` | Managed Cargo/Windows Release and product scheduler p50/p95/p99 remain pending behind external `E:\Git\zr_vm` dirt |

The change is intentionally limited to allocation shape. Tooling remains out of
scope until the planned Rust migration.

Editor875 later preserves this activated-output capacity behavior while
removing the complete resource-key clone snapshot from single promotion. The
combined scheduler source-contract batch passes `20/20`; managed gates remain
pending.

Editor876 later changes only shutdown ownership/drain behavior. The promotion
capacity contract remains unchanged, and the current adjacent Editor827/828/875/
876 scheduler batch passes `17/17`; managed gates remain pending.

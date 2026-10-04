---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/layout/pass/inline_widgets.rs
tests:
  - zircon_runtime/src/ui/layout/pass/inline_widgets.rs
  - tools/tests/test_runtime_text_infrastructure_compile_contract.py
  - tools/tests/test_runtime_text_rich_inline_geometry_profile_contract.py
---

# Runtime inline-widget arrangement capacity admission

The fixed-size inline-widget arrangement path now reserves the collection sizes it already knows
at each boundary. Direct-child membership reserves `parent.children.len()`, managed-child
membership reserves the compiled binding count, and the affected-root preorder scratch reserves
the root count. The previous predicates, ordering, duplicate/missing-child rejection, subtree
hiding, and normal arrange/render/hit-test ownership are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime81 inline-widget arrangement | Reserve direct-child, managed-binding, and preorder scratch capacities before the hot-path loops. | Runtime text/infrastructure and UI layout/navigation contract batch passed `105/105`; Rustfmt, source guard, and scoped diff checks pass. | implemented_pending_validation |

## Complexity and allocation boundary

The route remains `O(affected tree nodes + rich runs + graphemes + direct children)`. The change
does not add an index or a second UI-tree authority; it only prevents geometric growth of the
known-bounded `HashSet`/`Vec` containers during a warm arrangement. The capacities are lower bounds
(`roots.len()`, direct-child count, and binding count), so malformed or deeper trees retain the
existing growth and fail-closed behavior.

## Local evidence

- TDD source probe was RED before the production capacity calls existed and GREEN after the patch.
- The focused Runtime text/inline-widget and retained-layout contract batch passed `105/105` in a
  single invocation (`5.900s`).
- A subsequent single-invocation non-tooling Runtime/Editor performance-plus-pressure loader
  covered 343 modules and passed `1320/1320` tests in `79.827s`.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` pass; the repository reports only
  its existing LF/CRLF line-ending notices.
- No tooling source was changed. This record reports source/contract evidence only; no CPU,
  allocator, RSS, or input-to-present percentile is inferred from capacity admission.

The current single-process Runtime/Editor performance-contract discovery also passes `1823/1823`
tests in `22.612s`; this broader receipt is local source/model evidence and does not replace the
managed Cargo/Release allocation and latency gate.

## Managed acceptance gate

Managed Windows Cargo behavior plus a Release workload with cold/warm inline-widget arrangements,
allocation counters, and p50/p95/p99 input-to-present evidence remain pending the owner-attributed
asynchronous validation lane recorded in `696`. No coordinator state was polled in this slice,
and the row remains `implemented_pending_validation` until that gate is completed.

---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/702-runtime-ui-residual-test-api-compile-repairs.md
---

# Runtime Render/Text Test-API Compile Repairs

This batch aligns remaining Runtime render, text, and visibility tests with
the current optimized ownership boundaries:

- render-graph alias assertions now read the alias from the graph lifetime
  metadata instead of a resolved RHI texture descriptor;
- segmented screen-space UI plan tests inspect the prepared segment product;
- SDF report fixtures supply the explicit vertex-count field;
- visibility fixtures use stable `ResourceId` values and the hash benchmark
  passes its `HashMap` to the optimized lookup;
- vertical text tests import the canonical `VerticalMode` and use the current
  left-to-right/right-to-left direction variants.

These are test-boundary repairs only; production fallback, ordering, and
resource ownership semantics are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11B/11C text and SDF rendering | Align vertical-text, segmented-plan, and SDF report fixtures with current APIs. | Batched Runtime/Text contract suites and Rustfmt parse checks pass. | implemented_pending_validation |
| Runtime09B visibility and render graph | Keep alias metadata, stable resource identities, and HashMap benchmark inputs congruent with optimized paths. | Focused visibility/render source guards pass; managed Release evidence remains pending. | implemented_pending_validation |

## Batched local evidence

- Latest batched Runtime/Editor/Runtime Text performance-contract runs:
  Runtime `1146/1146` in `5.439s`, Editor `581/581` in `1.248s`, and Runtime
  Text `143/143` in `1.055s` (`1870/1870` aggregate).
- Focused Runtime UI route/layout/navigation/visibility set: `58/58` in
  `0.606s`.
- Batched pressure-model contracts also passed Runtime `154/154` in `3.291s`
  and Editor `129/129` in `2.752s` (`283/283` aggregate).
- Scoped Rustfmt parsing and whitespace checks pass for the directly edited
  boundaries; line-ending notices are non-semantic.

## Managed acceptance gate

The evidence above is source-contract and pressure-model evidence, not a
managed Cargo compile or product-performance receipt. The external
`E:\\Git\\zr_vm` dirty-worktree gate still rejects owner-attributed Windows
Release admission before compilation. Keep this record
`implemented_pending_validation` until the asynchronous owner ticket supplies
compile/test plus p50/p95/p99 evidence.

Tooling changes remain deferred by request.

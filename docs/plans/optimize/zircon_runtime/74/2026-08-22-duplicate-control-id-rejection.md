# Runtime74 Duplicate Control ID Rejection

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-duplicate-control-id-rejection.md","docs/zircon_runtime/ui/template/component_control_scope.md","zircon_runtime/src/ui/surface/control_index.rs","zircon_runtime/src/ui/template/asset/compiler/compile.rs","zircon_runtime/src/ui/template/asset/compiler/control_scope.rs","zircon_runtime/src/ui/template/build/build_error.rs","zircon_runtime/src/ui/template/build/tree_builder.rs","zircon_runtime/src/ui/tests/asset_binding/control_scope.rs","zircon_runtime/src/ui/tests/template_tree_builder.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-005`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- The asset compiler rejects duplicate control ids after component and prototype expansion, so the
  check covers the published identity space rather than only authored local definitions.
- `UiTemplateTreeBuilder` rejects duplicate ids during direct instantiation before inserting the
  duplicate node. The typed error reports the id, first node path, and duplicate node path.
- `UiSurfaceControlIndex` has no smallest-node compatibility lookup. Its string and compiled-slot
  lookups resolve only one node and fail closed if a later runtime mutation creates ambiguity.
- Compiler and direct-instantiation negative regressions lock both public construction paths.

## Validation Contract

The Runtime74 P2 child source guard rejects the former `node_id` compatibility method and locks the
compile and instantiation error variants. It runs both negative regressions as exact tests together
with the existing P2 groups. The 13-task / 16-Cargo-group / 25-behavior-test child SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`. It is pinned by the
84-task / 56-Cargo-group / 18-performance-row super-batch with SHA-256
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Coordinator execution is
pending; no Cargo or behavior pass is claimed.

Duplicate detection is folded into the existing instantiation traversal, avoiding an additional
tree scan. This slice adds no independent timing threshold or performance row. The grouped
Runtime74 super-batch retains 18 existing release measurements; their 21-pair alternating samples
and nearest-rank P95 evidence remain pending coordinator execution.

# Runtime74 Source-Boundary Test Naming

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-runtime-binding-expression-schema-naming.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-source-boundary-test-naming.md","zircon_runtime/src/ui/tests/asset_binding/naming_contract.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-008`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- The source-only naming guard is called
  `binding_validation_source_boundary_uses_schema_name_not_milestone`; it no longer presents a
  substring check as a Runtime binding-expression capability test.
- The grouped validator stage is named `runtime74_binding_schema_source_boundary` and its source
  contract pins the boundary-oriented test name.
- The original P2-001 record now states that this guard reads source text and does not execute the
  parser, compiler, evaluator, or Runtime dispatch path.

## Validation Contract

The renamed test still checks that the production validation owner contains
`is_runtime_binding_expression` and no `M18` milestone vocabulary. Its result is structural evidence
only. Runtime behavior continues to be evidenced by the separate compiled-program and dispatch
tests in the grouped batch. The 13-task / 16-Cargo-group / 25-behavior-test child SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`; the 84-task /
56-Cargo-group / 18-performance-row super-batch SHA-256 is
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Coordinator execution is
pending; no Cargo or behavior pass is claimed.

This evidence-label cleanup changes no production code and adds no performance row. The grouped
Runtime74 batch retains 18 existing release measurements; measured P95 evidence remains pending
coordinator execution.

# Runtime74 Test-Only Template and Router Hard Cut

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-test-only-template-router-hard-cut.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime/src/ui/binding/mod.rs","zircon_runtime/src/ui/binding/router.rs","zircon_runtime/src/ui/template/loader.rs","zircon_runtime/src/ui/template/pipeline.rs","zircon_runtime/src/ui/template/validate.rs","zircon_runtime/src/ui/tests/binding.rs","zircon_runtime/src/ui/tests/boundary/binding_event_roots.rs","zircon_runtime/src/ui/tests/template/loader_instance_validation.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-007`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- The obsolete `UiTemplateLoader`, `UiTemplateValidator`, and `UiTemplateRuntimePipeline` modules,
  their recursive fixture test, and their facade exports remain deleted. Asset loading and
  `UiDocumentCompiler` are the single production compile authority.
- `UiEventRouter<T>` is deleted because repository-wide caller analysis found only one headless unit
  test plus source-boundary assertions. Its module and `zircon_runtime::ui::binding` re-export are
  removed instead of retained as a parallel routing authority.
- The former router-only behavior test and mock command are removed. The binding namespace boundary
  now rejects reintroduction of the router module or symbol.
- Production surface compiled-event dispatch and `UiEventManager` are unchanged; this hard cut does
  not redirect or wrap their routing paths.

## Validation Contract

The P2 child source contract checks all four obsolete source/test paths are absent, rejects the
router module and symbol from the binding facade, and locks the structural namespace regression.
The boundary test is a source-structure guard, not a claim that Runtime event routing executed.
The 13-task / 16-Cargo-group / 25-behavior-test child SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`; the 84-task /
56-Cargo-group / 18-performance-row super-batch SHA-256 is
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Coordinator execution is
pending; no Cargo or behavior pass is claimed.

Deleting the unused exact-route map removes dead generic code and one misleading test surface. The
slice adds no runtime benchmark or performance row. The grouped Runtime74 batch retains 18 existing
release measurements, whose P95 evidence remains pending coordinator execution.

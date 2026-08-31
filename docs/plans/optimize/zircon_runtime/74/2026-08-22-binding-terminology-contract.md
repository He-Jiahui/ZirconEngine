# Runtime74 Binding Terminology Contract

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-binding-terminology-contract.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime/src/ui/template/asset/binding/validation.rs","zircon_runtime/src/ui/tests/asset_binding/schema_naming.rs","zircon_runtime_interface/src/ui/template/asset/binding/mod.rs","zircon_runtime_interface/src/ui/template/asset/binding/schema.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/mod.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-006`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- `UiBindingContractTerm` is the interface-owned vocabulary for event, binding, target, route,
  action, and command. Each term has one stable schema name and one non-overlapping definition.
- Route and action name kinds point back to their contract term. Payload fields remain explicitly
  named `action payload field`; they are not promoted to actions or targets.
- Runtime validation builds diagnostics from the typed name kind, removing caller-supplied noun
  labels that could drift from the field being validated.
- The canonical pipeline document states the event-to-binding-to-target/action flow and reserves
  command for a host operation accepted after routing.

## Validation Contract

The P2 child source guard locks the six-term interface owner, typed diagnostic formatting, and the
pipeline glossary. The schema-naming group includes a regression that checks exact term order,
definitions, route/action mappings, and diagnostic prefixes. The 13-task / 16-Cargo-group /
25-behavior-test child SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`; the 84-task /
56-Cargo-group / 18-performance-row super-batch SHA-256 is
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Coordinator execution is
pending; no Cargo or behavior pass is claimed.

This terminology cleanup adds no runtime work and no independent performance row. The grouped
Runtime74 super-batch retains its 18 existing release measurements; measured P95 evidence remains
pending coordinator execution.

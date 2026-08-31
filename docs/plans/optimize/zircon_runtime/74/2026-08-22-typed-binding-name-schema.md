# Runtime74 Typed Binding Name Schema

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-typed-binding-name-schema.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs","zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs","zircon_runtime/src/ui/template/asset/binding/validation.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/schema_naming.rs","zircon_runtime/src/ui/tests/mod.rs","zircon_runtime_interface/src/ui/component/event.rs","zircon_runtime_interface/src/ui/template/asset/binding/mod.rs","zircon_runtime_interface/src/ui/template/asset/binding/schema.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/mod.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-003`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- `zircon_runtime_interface` is the sole owner for component-event schema names and the typed
  route/action/payload-field naming contract. The previous 35-entry Runtime test mapping and
  payload-kind string table are removed.
- Route/action names preserve product casing and accept one or more ASCII dot-separated segments;
  payload fields use canonical lowercase ASCII/digit/underscore names. Empty segments, whitespace,
  slash separators, non-ASCII characters, and names over 256 bytes fail closed.
- Known payload fields have typed enum identities and Bool/Int hints. Runtime diagnostics and the
  compiled-program interner consume the same owner rather than maintaining local string lists.
- Editor route/action writeback and nested payload-path editing validate through the shared schema.
  Canonical payload suggestions use typed field identities, while dotted fields and array indices
  remain editable through validated path segments.
- A tracked `.zui` asset scan found no current route/action declarations rejected by the new schema.

## Validation Contract

The P2 child validator pins source-owner checks; this slice contributes four Runtime schema
regressions plus one Editor writeback regression to the current 13-task / 16-Cargo-group /
25-behavior-test cleanup batch. Its SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`.
It is pinned by the 84-task / 56-Cargo-group / 18-performance-row Runtime74 super-batch with
SHA-256 `92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`.

This naming cleanup has no independent performance threshold and emits zero performance rows. The
grouped Runtime74 batch retains 18 existing 21-pair release gates. Coordinator execution is pending;
no Cargo, behavior, or performance pass is claimed.

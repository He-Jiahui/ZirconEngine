# Runtime74 Binding Diagnostic Identity

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-binding-diagnostic-identity.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-010`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- `UiBindingDiagnosticCode` now owns one private identity table for the serialized error code,
  stable `ZUI-BIND-0001..0005` diagnostic ID, and `diagnostic.ui.binding.*` localization key. The
  fifth identity is the fail-closed `unsupported_binding_mode` contract from RTB-P1-004.
- `UiBindingDiagnostic` derives all three projections from its typed code; Runtime validation keeps
  contextual messages and does not duplicate identity strings.
- `as_str()` remains an alias for the existing snake_case error code, preserving serialized and Rust
  consumer compatibility.
- The inline interface regression locks the exact mappings, enumeration order, and uniqueness of all
  three identity columns.

## Validation Contract

The TDD red state confirmed the mapping test existed before the identity APIs. Rustfmt, scoped diff
checks, PowerShell AST parsing, and the 12-entry P2 source-contract prefix pass. The added exact
interface Cargo test remains queued for grouped coordinator execution; no Cargo pass is claimed.

The 13-task / 16-Cargo-group / 25-behavior-test P2 child SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`; the 84-task /
56-Cargo-group / 18-performance-row super-batch SHA-256 is
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Coordinator execution is
pending.

This interface identity slice adds no benchmark row and no runtime dispatch work. The grouped
Runtime74 super-batch retains 18 release measurements; measured P95 evidence remains pending
coordinator execution.

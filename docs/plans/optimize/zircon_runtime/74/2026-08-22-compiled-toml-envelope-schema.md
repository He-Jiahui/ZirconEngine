# Runtime74 Compiled TOML Envelope Schema

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py","docs/plans/optimize/zircon_runtime/74/2026-08-22-compiled-toml-envelope-schema.md","docs/ui-and-layout/shared-ui-template-runtime.md","docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md","docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md","docs/zircon_runtime/ui/architecture.md","docs/zircon_runtime/ui/template/pipeline.md","docs/zircon_runtime_interface/ui/mod.md","zircon_runtime_interface/src/ui/component/value.rs","zircon_runtime_interface/src/ui/template/mod.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/artifact.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/package_manifest.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs","zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs","zircon_runtime/src/ui/template/asset/compiler/package/package_manifest.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/asset_package_validation.rs","zircon_runtime/src/ui/tests/template_pipeline.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-002`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- The public schema constant hard-cuts to `UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION`; the old
  binary-artifact constant is removed rather than retained as an alias.
- `UiRuntimeCompiledAssetArtifact` is documented accurately as a fixed byte header followed by a
  deterministic UTF-8 TOML payload. Magic `ZRUIA018` and schema version 3 remain unchanged, so this
  naming cleanup does not invent binary sections the compiler does not emit.
- Current docs, generated-output policy, focused tests, and the existing architecture audit anchor
  use the TOML-envelope capability name. Historical archived evidence remains unchanged.
- Compiled binding IR recursively rejects non-finite float/vector values. This prevents NaN from
  defeating same-generation IR equality and keeps persisted envelope values deterministic.

## Validation Contract

The P2 child validator runs exact naming, envelope round-trip, and non-finite-value rejection tests.
Its SHA-256 is `1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`.
It is pinned by the 84-task / 56-Cargo-group / 18-performance-row Runtime74 super-batch with
SHA-256 `92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`.

This schema cleanup has no independent latency gate and emits zero performance rows. Existing
Runtime74 performance evidence remains pending coordinator execution. No Cargo or behavior pass is
claimed.

# Runtime74 Product Binding Fixture Coverage

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: P2 cleanup
Status: validation_pending
Files: [".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-compiled-endpoint.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p0-p1-endpoint-superbatch.ps1",".codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-p2-schema-cleanup.ps1","docs/plans/optimize/zircon_runtime/74/2026-08-22-product-binding-fixture-coverage.md","docs/zircon_runtime/ui/architecture.md","docs/zircon_runtime/ui/template/pipeline.md","docs/zircon_runtime/ui/v2.md","zircon_editor/assets/ui/editor/product_binding_fixture.zui","zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs","zircon_editor/src/tests/ui/boundary/template_assets.rs","zircon_editor/src/tests/ui/boundary/template_assets/product_binding_fixture.rs","zircon_editor/src/ui/asset_editor/session/lifecycle/v2_projection.rs","zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs","zircon_runtime/src/ui/template/asset/compiler/mod.rs","zircon_runtime/src/ui/template/asset/compiler/value_normalizer.rs","zircon_runtime/src/ui/template/asset/mod.rs","zircon_runtime/src/ui/tests/v2_asset/composite_components.rs","zircon_runtime/src/ui/v2/component_instancer.rs","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime_interface/src/ui/v2/asset.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P2-012`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- UI V2 node instances now own an explicit `params` map. The component instancer validates every
  supplied value against the component parameter schema, rejects unknown or missing parameters, and
  resolves nested values before publishing the compiled arena.
- Prototype nodes consume their instance scope while projected slot children retain the caller
  scope. Props, state, layout, style, slot values, binding targets, and action payloads therefore
  preserve the same typed parameter semantics across local, imported, nested, and repeated
  component instances.
- `product_binding_fixture.zui` is a real product view with two independently parameterized Button
  instances. Pointer dispatch applies the compiled target and emits the resolved action payload;
  reload recompiles changed values, while an invalid Bool override fails closed and preserves the
  last-known-good compiled artifact.
- The Editor authoring projection maps V2 instance parameters to the editable model without
  conflating them with runtime state. Prior V2 state survives a parameter edit round trip.
- Compiler schema 7 invalidates persistent V2 cache entries produced before instance parameter
  resolution became part of the compiled contract. The TOML envelope remains schema version 3.

## Validation Contract

The TDD red state established that the V2 node contract lacked instance parameters and the
instancer had no typed parameter consumption path before the implementation. Rustfmt parsing,
scoped diff checks, all V2 struct-literal checks, PowerShell AST parsing, and the 14-entry P2 source
contract are local static gates. The grouped coordinator batch adds two Runtime component-param
tests, three Editor product/projection tests, and the exact 42-view material inventory test. No
Cargo pass is claimed until that asynchronous validation completes.

The 13-task / 16-Cargo-group / 25-behavior-test P2 child SHA-256 is
`1E8F1D955ED3AA4EDB282B1B253631BB91E19CE57AB81687B115330E698E1459`; the compiled-endpoint child
SHA-256 is `C24767B90070D3446F0127515EC20D2CCDC9B15C3B2BF01DB51445853150F610`. The 84-task /
56-Cargo-group / 18-performance-row super-batch SHA-256 is
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Coordinator execution is
pending.

This product-coverage slice adds no new benchmark row. Compile-time parameter substitution removes
all `ParamRef` lookup work from pointer dispatch; the grouped Runtime74 batch retains 18 existing
21-pair alternating release measurements, with measured P95 evidence pending coordinator execution.

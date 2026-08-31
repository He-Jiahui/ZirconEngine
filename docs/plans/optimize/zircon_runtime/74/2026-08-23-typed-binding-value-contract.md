# Runtime74 Typed Binding Value Contract

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-23-typed-binding-value-contract.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime_interface/src/tests/binding_value_contracts.rs","zircon_runtime_interface/src/tests/mod.rs","zircon_runtime_interface/src/ui/binding/mod.rs","zircon_runtime_interface/src/ui/binding/model/binding_value.rs","zircon_runtime_interface/src/ui/binding/model/binding_value/projection.rs","zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs","zircon_runtime_interface/src/ui/binding/model/binding_value/validation.rs","zircon_runtime_interface/src/ui/binding/model/mod.rs","zircon_runtime_interface/src/ui/binding/model/parse_error.rs","zircon_runtime_interface/src/ui/binding/model/parser.rs","zircon_editor/src/ui/binding_dispatch/inspector/field_value.rs","zircon_editor/src/ui/host/editor_event_control_requests.rs","zircon_editor/src/ui/host/editor_event_dispatch.rs","zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs"]

- Date: 2026-08-23
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-010`
- Delivery state: implementation and static contract complete; grouped coordinator validation pending

## Scope Delivered

- `UiBindingValue` now represents deterministic records, ordered typed-key maps, typed enums with
  optional payloads, validated asset references, generation-qualified entity references, explicit
  optionals, and controlled collection views in addition to the existing scalar and array values.
- JSON and bincode deserialization execute the same value-budget admission. The native binding
  parser round-trips every new variant and rejects an invalid or over-budget value before returning
  an event binding.
- The standard contract admits at most 64 levels, 1,024 nodes, 16 KiB aggregate string data, and
  256 entries in any array, record, or map. It rejects non-finite floats, duplicate map keys,
  empty/oversized typed identities, zero entity generations, zero collection revisions, oversized
  collection windows, and windows outside their declared total length.
- `UiBindingCollectionView` contains a typed model-provider key, typed item-schema key, collection
  revision, and checked window metadata. It never owns row values, so total collection size does
  not expand event payloads.
- Runtime-interface owns one JSON projection. The three Editor event bridges consume it; existing
  scalar/array output is unchanged while new typed values are tagged. Inspector text conversion
  accepts only values that can be converted without discarding type information.

## Reference Evidence and Divergence

- `dev/slint/internal/core/model.rs` exposes collection data through `row_count`/`row_data`, tracks
  row-count and row-data dependencies separately, and documents that model mutation plus
  notification is more efficient than replacing a model property. Zircon follows that reference by
  transporting a bounded view descriptor instead of a materialized row vector.
- `dev/UnrealEngine/Engine/Plugins/Runtime/ModelViewViewModel/Source/ModelViewViewModel/Public/Types/MVVMViewModelCollection.h`
  indexes view-model instances by typed context and publishes one collection-change notification.
  Zircon keeps the same separation between collection identity and change ownership, but uses
  serializable provider/schema generations rather than UObject identity.

Zircon additionally fixes deterministic map-key order and explicit serde/native budgets because its
binding values cross cook, host, replay, and plugin boundaries. Provider row retrieval and
dependency subscriptions remain with later Runtime74 model-execution tasks.

## Regression and Validation Contract

Four focused tests cover JSON/bincode/native rich-value round-trip, every owned budget and identity
rejection, legacy JSON-shape compatibility plus typed tags, and controlled-view serialized size.
The overflow regression also enters through `UiEventBinding::parse_native_binding`, so parser
admission cannot diverge from direct or serde validation.

Local non-Cargo evidence on 2026-08-23:

- `rustfmt +1.94.1`: passed for all P1-010 Rust owners and regressions.
- Runtime74 rich-value `-SourceContractOnly`: 8/8 checks passed.
- Rich-value child and model-values super-batch PowerShell scripts: AST parse passed.
- `git diff --check` for the P1-010 tracked scope: passed (line-ending conversion warnings only).
- Public re-export and exhaustive `UiBindingValue` consumer audit: complete.

The rich-value child validator SHA-256 is
`F507F1987683213F0E10C6BCEAF0770550B9F105F59FD360A45A6012C0789895`; the immutable grouped root
SHA-256 is `EB9810EF20EC49EBC6F786C800A9E71E19D51314BA8E0B0EF30CD667FAA1E450`.
It composes the existing 89-task Runtime74 batch with P1-010 for 90 tasks, 66 Cargo groups, 34
cumulative new tests, and 19 performance rows. Cargo has not run for this revision. No test pass,
measured payload result, commit, or WeCom delivery is claimed until the grouped coordinator
validator returns.

## Performance Gate

The deterministic release row serializes 256 typed records containing `id` and `label` fields, then
serializes a controlled view over 1,000,000 rows. It emits
`materialized_json_bytes`, `controlled_view_json_bytes`, and integer `reduction_percent`; the gate
requires at least 95% reduction and external recomputation from the two raw byte counts. The exact
measurement remains pending coordinator execution.

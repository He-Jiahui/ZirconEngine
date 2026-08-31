---
related_code:
  - zircon_editor/src/ui/binding_dispatch/inspector/field_value.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/validation.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/projection.rs
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/binding/conversion_registry.rs
  - zircon_runtime/src/ui/binding/model_schema_registry.rs
  - zircon_runtime_interface/src/ui/binding/model/conversion.rs
  - zircon_runtime_interface/src/ui/binding/model/model_context.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/control_scope.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/binding/model/model_schema.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/schema.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime/src/ui/tests/asset_schema_migration.rs
  - zircon_runtime/src/ui/tests/asset/fixture_migration.rs
  - zircon_runtime/src/ui/tests/asset_binding/control_scope.rs
  - zircon_runtime/src/ui/tests/asset_binding/schema_naming.rs
  - zircon_runtime/src/ui/tests/boundary/binding_event_roots.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store/control_scope.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_editor/assets/ui/editor/product_binding_fixture.zui
  - zircon_editor/src/tests/ui/boundary/template_assets/product_binding_fixture.rs
implementation_files:
  - zircon_editor/src/ui/binding_dispatch/inspector/field_value.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/validation.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/projection.rs
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/binding/conversion_registry.rs
  - zircon_runtime/src/ui/binding/model_schema_registry.rs
  - zircon_runtime_interface/src/ui/binding/model/conversion.rs
  - zircon_runtime_interface/src/ui/binding/model/model_context.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/control_scope.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/surface/control_index.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/binding/model/model_schema.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/template/asset/binding/schema.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/report.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/ui/tests/asset_binding/schema_naming.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
  - docs/engine-architecture/generated-code-boundary.md
tests:
  - zircon_runtime_interface::tests::binding_value_contracts::binding_value_rich_contract_round_trips_serde_and_native_binding
  - zircon_runtime_interface::tests::binding_value_contracts::binding_value_contract_rejects_every_owned_budget_and_identity_overflow
  - zircon_runtime_interface::tests::binding_value_contracts::binding_value_json_projection_preserves_existing_shapes_and_tags_typed_values
  - zircon_runtime_interface::tests::binding_value_contracts::controlled_collection_view_reduces_serialized_payload_by_at_least_ninety_five_percent
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_74_template_boundary_has_one_compiler_authority
  - zircon_runtime::ui::tests::asset::fixture_migration::ui_asset_loader_rejects_source_template_documents_without_asset_header
  - zircon_runtime::ui::tests::template_pipeline::asset_compiler_is_the_single_template_compile_authority
  - zircon_runtime::ui::tests::template_pipeline::legacy_recursive_template_document_is_not_a_runtime_compile_input
  - zircon_runtime::ui::tests::template_pipeline::template_compiler_authority_has_bounded_p95_latency
  - zircon_runtime::ui::tests::template_pipeline::compiled_template_artifact_stays_toml_envelope_leaf_dto_not_generated_source
  - zircon_runtime::ui::tests::asset_binding::param_ref_compile_resolves_nested_params_and_artifact_roundtrip
  - zircon_runtime::ui::tests::asset_binding::param_ref_compile_rejects_a_missing_referenced_component_param
  - zircon_runtime::ui::tests::asset_prototype_store::param_ref_compile_resolves_prototype_binding_params
  - zircon_runtime::ui::tests::asset_binding::param_ref_compile_preserves_non_param_preview_expressions
  - zircon_runtime::ui::tests::asset_binding::control_scope::component_control_scope_routes_repeated_instances_to_their_own_payloads
  - zircon_runtime::ui::tests::asset_binding::control_scope::component_control_scope_composes_for_nested_instances
  - zircon_runtime::ui::tests::asset_binding::control_scope::component_control_scope_qualifies_one_thousand_instances_linearly
  - zircon_runtime::ui::tests::asset_prototype_store::control_scope::prototype_component_control_scope_matches_tree_compiler_semantics
  - zircon_runtime_interface::ui::template::asset::binding::expression::tests::typed_binding_literal_parser_preserves_supported_value_kinds_and_escapes
  - zircon_runtime_interface::ui::template::asset::binding::expression::tests::typed_binding_literal_param_probe_requires_a_path_root
  - zircon_runtime_interface::ui::template::asset::binding::expression::tests::binding_expression_parse_budgets_reject_oversized_or_deep_input
  - zircon_runtime_interface::ui::template::asset::binding::diagnostic::tests::binding_diagnostic_identity_contract_is_unique_and_stable
  - zircon_runtime_interface::ui::template::document::tests::binding_mode_contract_serializes_trigger_timing_and_write_permissions
  - zircon_runtime_interface::tests::model_schema_contracts::model_schema_contract_round_trips_typed_identity_version_and_field_access
  - zircon_runtime_interface::tests::model_schema_contracts::model_schema_identity_and_versions_reject_invalid_contracts
  - zircon_runtime_interface::tests::model_context_contracts::model_context_patch_round_trips_bind_and_clear_operations
  - zircon_runtime_interface::tests::model_context_contracts::model_context_resolution_is_surface_component_row_item_ordered
  - zircon_runtime_interface::tests::binding_conversion_contracts::binding_conversion_descriptor_round_trips_typed_signature_handle_and_provider_generation
  - zircon_runtime_interface::tests::binding_conversion_contracts::binding_conversion_identity_and_provider_generation_reject_invalid_contracts
  - zircon_runtime_interface::ui::template::asset::binding::expression::evaluator::tests::binding_expression_evaluator_resolves_values_and_preserves_short_circuiting
  - zircon_runtime_interface::ui::template::asset::binding::expression::evaluator::tests::binding_expression_evaluator_rejects_over_depth_programs_without_recursing
  - zircon_runtime::ui::tests::asset_binding::compiled_program::compiler_interns_binding_identity_and_target_endpoints
  - zircon_runtime::ui::tests::asset_binding::compiled_program::compiler_cooks_event_binding_mode_contract_into_binding_program
  - zircon_runtime::ui::tests::asset_binding::compiled_program::compiler_rejects_unimplemented_binding_modes_with_stable_diagnostic
  - zircon_runtime::ui::tests::model_schema_registry::model_schema_registry_resolves_exact_provider_schema_version_and_field_access
  - zircon_runtime::ui::tests::model_schema_registry::model_schema_registry_rejects_missing_schema_duplicate_fields_and_identity_collisions
  - zircon_runtime::ui::tests::model_schema_registry::model_schema_registry_rejects_empty_schemas_and_unknown_resolution_keys
  - zircon_runtime::ui::tests::model_schema_registry::model_schema_registry_keeps_versions_and_large_field_sets_deterministic
  - zircon_runtime::ui::tests::model_context_registry::registry_resolves_layered_context_with_inheritance_override_and_clear
  - zircon_runtime::ui::tests::model_context_registry::registry_rejects_unknown_or_wrong_version_context_providers
  - zircon_runtime::ui::tests::model_context_registry::registry_revalidates_inherited_context_instead_of_trusting_caller_state
  - zircon_runtime::ui::tests::binding_conversion_registry::conversion_registry_resolves_exact_typed_signature_and_idempotent_registration
  - zircon_runtime::ui::tests::binding_conversion_registry::conversion_registry_rejects_signature_mismatch_and_generation_conflicts
  - zircon_runtime::ui::tests::binding_conversion_registry::conversion_registry_upgrade_and_unload_invalidate_old_handles
  - zircon_runtime::ui::tests::binding_conversion_registry::conversion_registry_preserves_provider_errors_and_rejects_wrong_input_or_output_kind
  - zircon_runtime::ui::tests::asset_binding::compiled_program::compiled_artifact_round_trip_preserves_binding_program_generation
  - zircon_runtime::ui::tests::asset_binding::compiled_program::compiler_rejects_over_budget_binding_expression_before_program_publication
  - zircon_runtime::ui::tests::asset_binding::compiled_program::stale_compiled_binding_handle_fails_closed_without_mutating_target
  - zircon_runtime::ui::tests::asset_binding::compiled_program::over_budget_compiled_expression_fails_closed_without_mutating_target
  - zircon_runtime::ui::tests::asset_binding::compiled_program::compiled_binding_endpoint_lookup_improves_nearest_rank_p95_by_at_least_twenty_five_percent
  - zircon_runtime::ui::tests::asset_binding::schema_naming::component_event_schema_names_round_trip_from_the_interface_owner
  - zircon_runtime::ui::tests::asset_binding::schema_naming::action_payload_field_schema_owns_known_names_and_value_kinds
  - zircon_runtime::ui::tests::asset_binding::schema_naming::binding_contract_terms_are_distinct_stable_and_drive_name_diagnostics
  - zircon_runtime::ui::tests::asset_binding::schema_naming::binding_name_schema_preserves_product_routes_and_rejects_ambiguous_names
  - zircon_runtime::ui::tests::asset_binding::schema_naming::compiler_rejects_invalid_route_action_and_payload_field_names
  - zircon_runtime::ui::tests::asset_package_validation::package_report_distinguishes_compilation_from_runtime_binding_execution
  - zircon_runtime::ui::tests::boundary::binding_event_roots::binding_api_moves_under_binding_namespace
  - zircon_editor::ui::asset_editor::binding::binding_inspector::payload_editing::tests::editor_normalization_uses_the_shared_binding_name_schema
doc_type: module-detail
---

# UI Template Compiler Boundary

RTB-P1-001

Status: validation_pending

The runtime has one source-template compiler authority:

```text
UiAssetLoader -> UiDocumentCompiler -> UiCompiledDocument -> UiTemplateSurfaceBuilder
```

`UiAssetLoader` owns source schema migration and parsing. `UiDocumentCompiler` owns validation,
component/slot expansion, binding compilation, style resolution, resource collection, and production
of `UiCompiledDocument`. `UiTemplateSurfaceBuilder` accepts only that compiled document or its
retained `UiTemplateInstance`. The retired `UiTemplateLoader`, `UiTemplateValidator`,
`UiTemplateRuntimePipeline`, `UiTemplateDocument`, and `UiTemplateError` surfaces are deleted; no
compatibility re-export or test-only recursive expander remains.

RTB-P2-007 applies the same hard-cut rule to routing. The generic `UiEventRouter<T>` exact-path map
had no production caller; its only consumer was a headless unit test. The module, public re-export,
and self-justifying behavior test are deleted together. Production component-event dispatch remains
on the surface-owned compiled binding path and `UiEventManager`; the binding namespace retains only
binding update/report behavior. No generic router facade aliases the production authorities.

The acceptance anchors are:

- `asset_compiler_is_the_single_template_compile_authority`
- `legacy_recursive_template_document_is_not_a_runtime_compile_input`
- `template_compiler_authority_has_bounded_p95_latency`
- `compiled_template_artifact_stays_toml_envelope_leaf_dto_not_generated_source`

The performance gate records 21 wall-clock samples of 100 compiles, uses nearest-rank P95, and
emits `PERF-RUNTIME74-COMPILER-AUTHORITY` with all 21 raw samples, the sample count, batch size,
measured P95, limit, and structural authority counts. The external validator sorts those raw values
and independently recomputes nearest-rank P95. Coordinator Cargo validation remains pending; no
measured P95 is claimed in this document yet.

Generated output policy is also explicit. `UiRuntimeCompiledAssetArtifact` records `runtime_09_m3_1_toml_envelope_leaf_dto_not_generated_source` and `requires_generated_source_marker() == false` because the current compiled template artifact is a framed TOML envelope DTO, not a generated Rust/source file. If a future template compiler writes source files, the first line must follow Runtime 02 M4 generated-code policy:

```rust
// @generated <generator> - do not edit by hand
```

Generated source remains limited to leaf DTO/table/adaptor material. Runtime behavior, validation
rules, loader policy, instantiation, and surface mutation stay handwritten in
`zircon_runtime::ui::template`.

## Compiled binding program and endpoint identity

RTB-P1-002 / RTB-P1-003 / RTB-P1-004 / RTB-P1-013 (compiled expression subset) / RTB-P1-015 / RTB-P1-016

After component expansion, parameter substitution, control qualification, and style resolution,
the compiler fingerprints the canonical retained tree and emits `UiCompiledBindingProgram` into
`UiTemplateInstance`. Source strings are interned into separate binding/property/control/route/action
domains. Target and standard action-payload expressions become typed
`UiCompiledBindingExpression` nodes, while binding and target endpoints carry the artifact
generation plus dense node/binding/target slots. Payload literals are stored as `UiValue`; syntax
owned only by the asset-editor preview dialect is marked unavailable and fails closed at Runtime.

`UiTemplateSurfaceBuilder` installs the program beside the retained tree. Pointer component events
carry `UiCompiledBindingHandle`; target execution validates generation and endpoint identity and
does not parse target or action-payload expressions or build a string-keyed binding index at
dispatch time. Default interactions attach the same compiled handle.

`UiBindingMode` owns the serialized `OneTime`, `OneWay`, `TwoWay`, `Event`, and `Command` names.
Each mode has one typed trigger timing and an explicit target/source/command write-permission set.
Existing assets default to `Event`, and compiled bindings retain that mode in the artifact. The
current runtime has a real executor only for `Event`; validation rejects the other four modes with
`unsupported_binding_mode` instead of silently compiling model writeback or command publication.
Those modes become admissible only when their later model/provider and command-gateway milestones
install the matching executor contracts.

TOML envelope schema version 3, compiler schema version 8, and magic `ZRUIA018` prevent earlier
payloads without the compiled payload program and binding-mode identity from being reused as
current artifacts.

## Typed model schema registry

RTB-P1-005

`zircon_runtime_interface::ui::binding` owns validated model-schema, field, and provider IDs plus
non-zero schema/provider versions. `UiModelFieldSchema` pairs a stable field ID with `UiValueKind`
and explicit read-only or read-write access. IDs are bounded to 256 ASCII bytes and use non-empty
dot-separated segments; deserialization applies the same validation as Rust construction.

`zircon_runtime::ui::binding::UiModelSchemaRegistry` owns deterministic runtime registration.
Schemas and providers are keyed by their complete `(id, version)` identity, so multiple schema
versions can coexist without ambiguous latest-version lookup. Re-registering an identical
descriptor is idempotent; changing a descriptor under the same complete key, duplicating a field,
or referencing an absent schema fails with a typed error. Provider-to-field resolution uses
`BTreeMap` indices and never scans model fields or checks concrete product names.

This registry is schema authority only. P1-005 does not install provider objects, subscriptions,
data-context inheritance, or two-way writes. Those remain fail-closed and belong to RTB-P1-006,
RTB-P1-031, RTB-P1-032, and RTB-P1-040. The 1,024-field regression locks deterministic version
ordering and indexed terminal-field resolution; grouped coordinator execution remains pending.

## Layered model data context

RTB-P1-006

`UiResolvedModelContext` has exactly four ordered layers: surface, component, row, and item. A
`UiModelContextPatch` leaves a layer absent to inherit it, binds one complete versioned provider key
to override it, or explicitly clears it. Resolution starts from the inherited context and applies
the patch in canonical layer order, so a component or collection row can override its own model
without silently dropping unrelated surface or item context.

`UiModelSchemaRegistry::resolve_model_context` validates every provider in the resulting context,
including inherited entries. A missing provider or a different unregistered provider version fails
with the exact context layer and key; caller-constructed inherited state is not trusted. Context
composition remains descriptor-only in this slice: live provider objects, subscriptions, field
reads, writeback, and collection virtualization belong to their later Runtime74 milestones.

## Typed binding conversion lifecycle

RTB-P1-008

`UiBindingConversionDescriptor` owns a validated conversion ID, non-zero provider generation, and
an exact `UiValueKind` source/destination signature. Runtime registration assigns a dense slot and
returns `UiBindingConversionHandle { slot, provider_generation }`. Identical registration is
idempotent; changing a signature without increasing provider generation, registering an older
generation, resolving an unloaded handle, or using a handle from an earlier generation fails with
a typed error.

Execution resolves the handle before invoking the stateless provider function. It validates the
actual input kind, preserves a provider's structured failure, and validates the returned output
kind against the registered signature. Upgrading a provider retains the dense slot while making
the old handle stale; explicit unload removes the active ID and also makes its handle stale.

This follows the local Unreal MVVM compiled-library evidence that conversion identity, execution
failure, and load/unload lifetime belong to the compiled binding authority. Zircon intentionally
uses serializable IDs, typed `UiValue`, and explicit provider generations instead of UObject field
paths or reflection-owned functions. Wiring conversion handles into compiled expressions and cache
dependency generations remains subsequent Runtime74 work.

## Binding missing-value policy

RTB-P1-009

Each authored binding target and action payload carries one explicit
`UiBindingMissingValuePolicy`; the compiler retains the target policy beside its dense endpoint and
the payload policy beside the compiled action fields. Legacy targets and actions default to
`Required`. For targets, `Required` rejects the transaction and `Error` keeps a distinct explicit
diagnostic policy. `Optional` omits only the unresolved target while resolved siblings still commit
atomically. `Default` and `Fallback` carry a typed `UiValue` substitute that passes through the
target's normal type and mutation checks.

Action payload resolution no longer collects `Option` values into an implicit all-or-nothing
short circuit. It produces an explicit value, omit, or reject outcome per field: optional fields
are omitted while their route remains publishable, default/fallback values are inserted, and
required/error outcomes suppress action publication. Runtime raw-authoring dispatch, Runtime
compiled dispatch, Editor source preview, and the Editor retained host's compiled action path all
consume the same `UiBindingMissingValueResolution` outcomes.

Artifact admission rejects non-finite substitute values. Runtime does not parse policy text or
look up a fallback registry during event dispatch; it performs one enum branch only after the
compiled expression reports a missing value. Compiler schema 8 prevents persistent caches from
admitting target programs that predate this policy field.

The compiled binding order contract is canonical: nodes use depth-first pre-order with authored
child order; each node's bindings and each binding's targets retain authored vector order; action
payload fields use strict lexical field-name order. Artifact admission requires binding IDs to be
globally contiguous across node/source slots, payload fields to remain in lexical order, and target
endpoint indices to match their stored vector positions. Runtime dispatch consumes those stored
orders directly, so a structurally valid artifact cannot silently remap a source binding slot or
payload update order.

The binding expression owner defines shared 16 KiB source, 2,048 token, 1,024 node, and 64-level
depth ceilings. Source parsing rejects over-budget input before recursive descent; artifact loading
checks the same node/depth limits iteratively; runtime compiled-target evaluation uses an explicit
stack and fails closed on either limit. Evaluator frame/value stacks keep eight entries inline and
spill only for deeper valid expressions, so ordinary shallow bindings add no stack-container heap
allocation. This closes the compiled target/payload portion of RTB-P1-013, not the later model,
command, or collection budgets.

Two release rows use 21 alternating legacy/compiled sample pairs and nearest-rank P95. They require
compiled endpoint lookup and compiled payload dispatch to improve their legacy parse paths by at
least 25% and emit all raw samples for external recomputation. Coordinator measurements remain
pending.

## Typed binding value contract

RTB-P1-010

`UiBindingValue` carries scalar and array values plus deterministic records, ordered typed-key maps,
typed enums with optional payloads, validated asset references, generation-qualified entity
references, explicit optional values, and controlled collection views. The scalar/array native and
JSON projections keep their established shapes. New non-scalar values use explicit native
constructors and tagged JSON projections so an Editor bridge cannot silently erase an enum type,
entity generation, or collection identity.

Every deserialized value is admitted through one depth-bounded contract walk: 64 levels, 1,024 value
nodes, 16 KiB of aggregate string data, and 256 entries per array, record, or map. Typed identities
are non-empty and at most 256 bytes; floats are finite; map keys are deterministic scalar keys and
duplicates fail closed. Native binding parsing invokes the same validation before publishing an
action argument.

A `UiBindingCollectionView` is metadata, not a materialized row array. It binds one typed model
provider and item schema to non-zero versions, a non-zero collection revision, and a checked
`offset/length/total_length` window of at most 256 rows. Row access and invalidation remain provider
responsibilities; the binding value cannot copy an unbounded collection into an event payload.

The release-size row compares the serialized form of 256 materialized typed records with a
controlled view over 1,000,000 rows. It requires at least 95% payload-byte reduction and emits both
byte counts plus the externally recomputable reduction percentage. Coordinator measurement remains
pending.

## Typed binding name schema

RTB-P2-003

`zircon_runtime_interface` owns the stable text contract for component events, action-payload
fields, routes, and actions. Every `UiComponentEventKind` round-trips through its case-sensitive
schema name. `UiActionPayloadFieldName` owns the canonical payload-field vocabulary and the known
Bool/Int type hints previously duplicated in Runtime validation and Editor suggestions.

Payload-field names are non-empty ASCII lowercase/digit/underscore strings. Route and action names
preserve authored ASCII case and contain one or more non-empty dot-separated segments; each segment
accepts ASCII letters, digits, underscore, and hyphen. All three domains share a 256-byte ceiling.
The Runtime compiler validates names before interning them, binding diagnostics use the same
contract, and the Editor validates route/action targets and every key segment in nested payload edit
paths before mutating a document. Invalid empty segments, whitespace, slash-separated names, or
non-ASCII spellings fail closed instead of entering a compiled artifact.

This cleanup has no independent performance threshold. Its Runtime and Editor behavior regressions
ride the grouped Runtime74 coordinator batch; executable results remain pending.

## Binding terminology contract

RTB-P2-006

`UiBindingContractTerm` owns the six terms used across source schema, compiled artifacts, Runtime
diagnostics, and this document. They are not synonyms:

| Term | Sole meaning in the binding pipeline |
|---|---|
| event | A typed input or component occurrence that triggers matching. |
| binding | A compiled declaration that matches one event and owns effects. |
| target | A typed state-mutation endpoint owned by a binding. |
| route | The dispatch destination selected for an action invocation. |
| action | The named invocation and payload emitted by a binding. |
| command | A host operation accepted after action routing. |

An event triggers a binding; the binding may mutate targets and may emit an action. A route chooses
where that action is dispatched. Only a receiving host may convert the action into a command. A
target is never a dispatch destination, a route is never the action name, and an observed event is
not evidence that a command executed.

The source fields retain their established meanings: `UiBindingRef.event` is the event,
`UiBindingRef.targets` are mutation targets, `UiActionRef.route` is the route, and
`UiActionRef.action` is the action name. Name-validation diagnostics derive `route`, `action`, and
`action payload field` from typed schema kinds instead of caller-supplied labels. Current compiled
dispatch does not claim command execution until a host command gateway accepts the routed action.

## Binding diagnostic identity contract

RTB-P2-010

`zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs` is the sole owner of binding
error codes, diagnostic IDs, and localization keys. `UiBindingDiagnosticCode` retains the serialized
snake_case error codes `invalid_target`, `invalid_value_kind`, `unresolved_ref`, and
`unsupported_operator`. The same typed code derives stable diagnostic IDs `ZUI-BIND-0001` through
`ZUI-BIND-0004` and matching `diagnostic.ui.binding.*` localization keys from one private identity
table.

`UiBindingDiagnostic` stores only the typed code and exposes `error_code`, `diagnostic_id`, and
`localization_key` projections. Runtime validation continues to own contextual human-readable
messages, paths, node IDs, and binding IDs, but it must not duplicate identity strings. This keeps
serialized reports backward compatible while allowing Runtime and Editor consumers to localize a
diagnostic without inventing another code registry.

## Package binding lifecycle stage

RTB-P2-011

`UiCompiledAssetPackageValidationReport.binding_lifecycle_stage` records the highest binding stage
proved when that report was emitted. The ordered vocabulary is `Declared`, `Compiled`, `Loaded`,
`Bound`, `Executed`, and `Applied`; serialized reports use the corresponding lowercase names. An old
report without the field defaults conservatively to `Declared`.

Package validation compiles the binding program and therefore emits `Compiled`. Retaining the
`RuntimeBindings` package section does not advance the report to `Loaded`, `Bound`, `Executed`, or
`Applied`. Those stages require evidence from artifact loading, surface binding, execution receipts,
and authoritative apply reports respectively. The package report is not retroactively mutated to
claim runtime work that happened after cooking.

## Shared canonical binding expression evaluator

RTB-P1-029 (canonical AST subset) / RTB-P1-013 (execution-budget subset)

`UiBindingExpression::evaluate_with` in `zircon_runtime_interface` is the single evaluator for the
standard template binding AST. Runtime pointer action projection and Editor template-runtime supply
their property/control resolvers to this entry point; neither consumer owns an operator match tree
or boolean coercion helper. The shared evaluator preserves `&&`/`||` short-circuiting and uses an
explicit frame/value stack with the public 1,024-node and 64-level limits. Its first eight frame and
value entries stay inline; deeper valid expressions use a bounded spill vector.

This does not absorb the asset-editor mock function dialect (`concat`, `coalesce`, `join`, and
similar authoring helpers). Runtime compiled dispatch marks those payload values unavailable;
Editor action-token slots compile the standard AST once when bound. Authoring-only model projection
and uncompiled compatibility surfaces remain outside the cooked artifact path. Provider/model
evaluation remains with later Runtime74 milestones.

## Runtime 09 source-template production migration hard cutover

runtime_09_source_template_fixture_production_migration_path_removed_static_passed_cargo_pending

`UiAssetSchemaMigrator` accepts only sources with an explicit `[asset]` header. Current/older recursive tree assets and supported flat node-table assets still converge on the current tree authority; historical source-template documents without `[asset]` are rejected with `UiAssetError::ParseToml("ui asset source is missing [asset]")`.

The retired `source_template_fixture.rs` conversion module, public fixture migration helpers, report variants and source-string naming guard were deleted together. `ui_asset_loader_rejects_source_template_documents_without_asset_header` is the behavior guard; no test-only converter, alias or compatibility report vocabulary remains on the live path. Historical Runtime15 output records remain archived evidence of the earlier naming cut and are not current API contracts.

## Component binding parameter resolution

Component binding parameters are resolved while the component instance is expanded, before the
retained `UiTemplateNode` or compiled package artifact is published. The ordinary recursive asset
compiler, the flat prototype compiler, and the V2 component instancer use the same value and binding
resolvers. A `param.*` leaf becomes a typed
literal, constant-only boolean/equality branches are folded, and mixed expressions keep their
runtime `prop.*` or `control.*` endpoints without retaining component parameter scope. Bindings on
the component instance itself resolve against the caller's typed parameter scope; bindings inside
the referenced component resolve against the callee's scope.

Action payload expressions that become constants use typed TOML when TOML preserves the `UiValue`
kind. Semantic string kinds and fixed typed values use canonical expression constructors:
`color(...)`, `asset_ref(...)`, `instance_ref(...)`, `enum(...)`, `vec2(...)`, `vec3(...)`,
`vec4(...)`, and `flags(...)`. The runtime and Editor expression parser reconstruct these as typed
`UiValue` literals after artifact serialization. String escaping is canonical and preserves quotes,
backslashes, control escapes, and four-digit Unicode escapes.

The compiler's parameter-reference probe tokenizes the source without requiring the complete
Editor preview dialect to parse. A quoted value such as `"param.title"` therefore does not turn an
otherwise preserved `concat(...)` payload into a Runtime expression parse failure. A real
`param.*` inside an unsupported dialect still fails explicitly because it cannot be compiled into
the supported binding AST.

Array and map parameters can be emitted as whole action-payload constants through TOML, but they
cannot currently be embedded in a mixed target expression. Non-finite or exponent-form floats also
fail closed when the current expression grammar cannot represent them. A referenced parameter that
has neither a default nor an instance value fails compilation with `UiAssetError` rather than
surviving as an expression that later evaluates to `None`.

V2 nodes serialize instance values in `UiV2NodeDefinition::params`. Expansion carries separate
caller and callee scopes, validates every override against `UiComponentParamSchema`, and resolves
props, state, layout, style, slots, target expressions, and action payloads before arena publication.
The Editor product fixture compiles two instances with distinct values, applies a real target on a
retained surface, reloads through the V2 file cache, and rejects a malformed override while keeping
the last-known-good compiled entry. Compiler schema 8 invalidates V2 persistent cache entries made
before this contract.

This section closes only component parameter resolution. Target execution, typed component events,
component control scoping, and transactional hot reload have separate Runtime74 ownership records.

## Component instance control scope

Both asset expansion paths qualify component-private `control_id` values before a retained template
is published. The qualified identity is derived from the full component-call `node_id` path plus the
local control id using a deterministic byte encoding. A component root with an authored instance
`control_id` aliases its local root id to that caller-visible id; other local ids stay private.

Binding target expressions and action payload expressions are parsed into `UiBindingExpression`,
their `ControlPropRef` leaves are rewritten, and they are emitted through the same canonical source
renderer used for parameter resolution. Bindings authored on the instance node and mounted slot
fills stay in the caller scope. Nested components derive a child scope, so their private controls do
not leak into siblings or repeated instances. Recursive document and flat prototype compilation use
the same rules.

Compilation rejects any duplicate `control_id` that remains after expansion. At dispatch time,
`UiSurface` resolves control properties only through the unique incremental control index; the old
smallest-node fallback is removed. The compiler schema is version 6 and the TOML envelope schema
is version 3, so artifacts produced before control qualification and compiled endpoint generation
cannot be reused.

The 1,000-instance regression locks one distinct private id and one matching compiled control
reference per instance, with zero global duplicate fallbacks. Compiled binding handles are now
generation-qualified and stale handles fail closed; model subscription generations and rebind
receipts remain part of the later model/reload boundary.

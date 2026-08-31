# Runtime74 Typed Model Schema Registry

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-23-model-schema-registry.md","docs/zircon_runtime/ui/template/pipeline.md","zircon_runtime/src/ui/binding/mod.rs","zircon_runtime/src/ui/binding/model_schema_registry.rs","zircon_runtime/src/ui/tests/mod.rs","zircon_runtime/src/ui/tests/model_schema_registry.rs","zircon_runtime_interface/src/tests/mod.rs","zircon_runtime_interface/src/tests/model_schema_contracts.rs","zircon_runtime_interface/src/ui/binding/mod.rs","zircon_runtime_interface/src/ui/binding/model/mod.rs","zircon_runtime_interface/src/ui/binding/model/model_schema.rs"]

- Date: 2026-08-23
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-005`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- Interface-owned typed IDs cover model schema, model field, and provider domains. Schema and
  provider versions are separate non-zero types; construction and deserialization share the same
  bounded ASCII dotted-name validation.
- Field schemas carry `UiValueKind` and read-only/read-write access. Complete schema/provider keys
  include both ID and version, avoiding an implicit or mutable "latest" identity.
- The Runtime registry keeps schema versions side by side, requires providers to reference an
  already registered exact schema key, and builds a deterministic field index.
- Identical registrations are idempotent. Empty schemas, duplicate fields, changed descriptors
  under the same key, missing schemas, unknown providers, and unknown fields are typed failures.
- A 1,024-field regression locks deterministic key order and terminal-field lookup without adding
  concrete Editor model names to shared production code.

## Reference Evidence and Divergence

- Slint `internal/compiler/expression_tree.rs` and
  `tests/cases/bindings/two_way_binding_model.slint` support typed property identity, access
  direction, and model-field regression coverage.
- Bevy `crates/bevy_reflect/src/type_registry.rs` supports complete identity indices, idempotent
  same-registration behavior, deterministic lookup APIs, and explicit conflict/overwrite policy.
- Godot `core/object/property_info.h` and `core/object/class_db.cpp` support registering field type
  and access metadata together and rejecting duplicate properties.

Zircon diverges by keying persisted UI model contracts with explicit serializable string IDs and
versions rather than Rust `TypeId`, C++ class identity, or toolkit-generated property references.
That is required for cooked artifact stability and Editor/gameplay parity. The registry deliberately
does not own live provider objects, subscriptions, data contexts, or writes; later Runtime74 tasks
must layer those capabilities on these keys.

## TDD and Validation Contract

Tests were mounted before production types existed. Interface coverage locks serialization,
deserialization, invalid IDs, and zero versions. Runtime coverage locks exact version resolution,
idempotency, all registration conflicts, missing references, deterministic ordering, and the large
field-set boundary.

The grouped Runtime74 submission `caf7bfeb2eed4e3e9452e78fd45aed36` / request
`a97a2f548668430b997b32ec2891c14b` covered 88 tasks, 62 Cargo groups, 20 new behavior tests, and
18 existing performance rows under validator SHA-256
`E93B9E81B8EFA1225CDA3B5CF5632687E7CA29C1A02C20C4614342A91D6BAFB1`. It failed during
validation-copy `closure_planning` with `validation_copy_state_forbidden`, before Cargo started.
No behavior pass, performance result, or commit is claimed; grouped validation remains pending.

The forward grouped submission `a2c39ddcdd944d588daa96cd7c99d512` / request
`d92db795584a4c4e8a561e6d3df175e1` is queued asynchronously without waiting. It covers 89 tasks,
65 Cargo groups, 30 cumulative new behavior tests, and 18 performance rows under root validator
SHA-256 `D84C8CA2B28C1EE4137D0CCC580FB601ED34F7F4E4084081E1AA0BEC75701ACB`; its 245-path,
7-tombstone source manifest is `6d2edcabe8fb82f2971f30f13d908d13899a148aa747ce75ae863a87c2582063`.
This receipt is submission evidence only; acceptance remains pending.

## Performance

Registration is bounded by deterministic `BTreeMap` construction; provider/field resolution uses
logarithmic provider, schema, and field indices. This schema-only slice is not yet on frame or event
execution paths and adds no release benchmark row. The 1,024-field test is a structural scale gate,
not measured P95 evidence; performance status remains pending for later model subscription work.

---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/reflect/type_path.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/reflect/type_kind.rs
  - zircon_runtime_interface/src/reflect/editor_hint.rs
  - zircon_runtime_interface/src/reflect/field_info.rs
  - zircon_runtime_interface/src/reflect/type_info.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/schema.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/reflect/error.rs
implementation_files:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/reflect/type_path.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/reflect/type_kind.rs
  - zircon_runtime_interface/src/reflect/editor_hint.rs
  - zircon_runtime_interface/src/reflect/field_info.rs
  - zircon_runtime_interface/src/reflect/type_info.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/schema.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/reflect/error.rs
plan_sources:
  - user: 2026-05-09 implement M8.1 Reflection Type Registry interface contracts
  - docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md
  - docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime_interface/src/tests/reflect_contracts.rs
  - tests/acceptance/reflection-type-registry.md
  - cargo check -p zircon_runtime_interface --locked --message-format short
  - cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short
doc_type: module-detail
---

# Runtime Interface Reflection Contracts

## Purpose

`zircon_runtime_interface::reflect` owns the neutral serialized reflection contract shared by runtime, editor, plugin tooling, persistence, and future remote projections. The module contains DTOs only: type identity, object addresses, schema metadata, reflected field values, read/write request and response shapes, tagged values, and structured errors.

This interface module must not depend on `zircon_runtime`, `zircon_editor`, `World`, ECS storage, adapters, render code, IO, or service managers. Runtime behavior such as `TypeRegistry`, component adapters, resource adapters, field conversion, dirty-state mutation, and `WorldReflection` dispatch belongs in `zircon_runtime::scene::reflect` in later M8 milestones.

## DTO Ownership

The `reflect` root is structural and re-exports focused child modules:

- `ReflectTypePath` stores canonical `type_path`, short lookup/display path, and optional module/plugin ownership.
- `ReflectTypeKind` describes the reflected type shape.
- `ReflectEditorHint`, `ReflectNumericRange`, and `ReflectEnumOption` describe editor and tooling metadata without requiring editor state.
- `ReflectFieldInfo` stores ordered field schema, editability, serializability, visibility, defaults, numeric ranges, enum options, hints, and documentation.
- `ReflectTypeInfo` groups the type kind and ordered fields.
- `ReflectSerializationStrategy` and `ReflectTypeRegistration` describe registry entries with component/resource, plugin, serialization, editor, remote, and optional plugin-owner flags but without storing runtime adapters.
- `ReflectSchemaFilter`, `ReflectSchemaRequest`, and `ReflectSchemaResponse` define schema listing/filter requests and responses.
- `ReflectFieldValue`, `ReflectFieldsRequest`, `ReflectFieldsResponse`, `ReflectReadRequest`, `ReflectReadResponse`, `ReflectWriteRequest`, and `ReflectWriteResponse` define the shared field access contract.

The constructor helpers on these DTOs preserve simple contract invariants, especially non-empty type paths. They do not perform runtime lookup, world mutation, storage access, or plugin loading.

`ReflectTypeRegistration::plugin_id` is the registry-level plugin owner. `ReflectTypeRegistration::with_plugin_id` keeps `ReflectTypePath::plugin_id` synchronized for DTO consumers that only receive type paths. `serialization` is the persistence strategy, while `serializable` is the explicit visibility/eligibility flag used by filters and tooling; constructors currently initialize it from the strategy and leave it overridable.

## Object Addressing

`ReflectObjectAddress` is the shared address form for reflected operations:

- `Component { entity, type_path }` targets one component on a stable external entity ID.
- `Resource { type_path }` targets one reflected resource by type path.

`ReflectObjectAddress::component` and `ReflectObjectAddress::resource` validate that type paths are not empty. The address is intentionally not editor-specific or transport-specific, so editor inspector, remote read/write, and future persistence code can use the same shape.

## Schema And Read/Write Flow

`ReflectSchemaFilter { type_path, include_components, include_resources, editor_visible, remote_visible, include_plugin_owned }` selects one type or a filtered list of editor-visible, remote-visible, component, resource, and plugin-owned registrations. `ReflectSchemaFilter::for_type` and `ReflectSchemaRequest::for_type` include component and resource registrations by default while leaving editor, remote, and plugin-owned flags unset. `ReflectSchemaRequest { filter }` carries that selector as the schema request contract. Runtime M8.7 applies these flags to registry data; the interface crate only defines the stable request/response contract.

`ReflectSchemaResponse { registrations }` returns deterministic ordered `ReflectTypeRegistration` values. Registration field order is preserved through `Vec<ReflectFieldInfo>` so schema consumers can display or persist fields predictably.

Field enumeration and direct access use shared DTOs:

- `ReflectFieldsRequest { address }`
- `ReflectFieldsResponse { address, fields }`
- `ReflectReadRequest { address, field_name }`
- `ReflectReadResponse { address, field }`
- `ReflectFieldValue { field_name, value }`
- `ReflectWriteRequest { address, field_name, value }`
- `ReflectWriteResponse { address, field, changed }`

Full-field enumeration uses `ReflectFieldsRequest` / `ReflectFieldsResponse`. Single-field reads and writes use explicit `field_name` request DTOs and return a `ReflectFieldValue`. Runtime code is expected to return field values in schema order.

## Reflected Values

`ReflectedValue` uses a stable tagged serde representation with `kind` and `value` fields. Supported shapes are null, booleans, signed and unsigned integers, `f32` scalars, strings, enums, `Vec2`, `Vec3`, `Vec4`, quaternions, optional entity IDs, resource IDs, lists, deterministic string-keyed maps, and raw JSON values.

Maps use `BTreeMap<String, ReflectedValue>` so serialized output is stable across runs. Untagged polymorphic JSON is not used because it would make editor tooling, remote transport, and persistence migrations ambiguous.

## Error Model

`ReflectError` is a serializable structured error enum for schema, lookup, field access, conversion, and registration failures. Variants carry explicit type path, field name, entity, source, target, and reason context as applicable.

The `Display` implementation is diagnostic text only. Callers that need machine-readable data should match the enum instead of parsing display strings.

## M8.1 Validation Commands

M8.1 validation is scoped to the interface contract slice and does not claim workspace-wide readiness:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime_interface --locked --message-format short
cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short
git diff --check -- "zircon_runtime_interface/src/lib.rs" "zircon_runtime_interface/src/reflect" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/reflect_contracts.rs" "docs/zircon_runtime_interface/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md" "docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md" "docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md"
```

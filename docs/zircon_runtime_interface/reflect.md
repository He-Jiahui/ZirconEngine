---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/reflect/type_path.rs
  - zircon_runtime_interface/src/reflect/type_path/validation.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/reflect/type_kind.rs
  - zircon_runtime_interface/src/reflect/editor_hint.rs
  - zircon_runtime_interface/src/reflect/field_id.rs
  - zircon_runtime_interface/src/reflect/field_id_parse_error.rs
  - zircon_runtime_interface/src/reflect/numeric_range.rs
  - zircon_runtime_interface/src/reflect/field_info.rs
  - zircon_runtime_interface/src/reflect/type_info.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/type_role.rs
  - zircon_runtime_interface/src/reflect/script_visibility.rs
  - zircon_runtime_interface/src/reflect/zr_reflect.rs
  - zircon_runtime_interface/src/reflect/zr_reflect_value.rs
  - zircon_runtime_interface/src/reflect/schema.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/mod.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/entry.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/admission.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/field_index.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/fingerprint.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/reflect/value_budget.rs
  - zircon_runtime_interface/src/reflect/value_validation.rs
  - zircon_runtime_interface/src/reflect/error.rs
implementation_files:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/reflect/type_path.rs
  - zircon_runtime_interface/src/reflect/type_path/validation.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/reflect/type_kind.rs
  - zircon_runtime_interface/src/reflect/editor_hint.rs
  - zircon_runtime_interface/src/reflect/field_id.rs
  - zircon_runtime_interface/src/reflect/field_id_parse_error.rs
  - zircon_runtime_interface/src/reflect/numeric_range.rs
  - zircon_runtime_interface/src/reflect/field_info.rs
  - zircon_runtime_interface/src/reflect/type_info.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/type_role.rs
  - zircon_runtime_interface/src/reflect/script_visibility.rs
  - zircon_runtime_interface/src/reflect/zr_reflect.rs
  - zircon_runtime_interface/src/reflect/zr_reflect_value.rs
  - zircon_runtime_interface/src/reflect/schema.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/mod.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/entry.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/admission.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/field_index.rs
  - zircon_runtime_interface/src/reflect/schema_catalog/fingerprint.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/reflect/value_budget.rs
  - zircon_runtime_interface/src/reflect/value_validation.rs
  - zircon_runtime_interface/src/reflect/error.rs
plan_sources:
  - user: 2026-05-09 implement M8.1 Reflection Type Registry interface contracts
  - docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md
  - docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/plans/zircon_plugins/08-zr-vm.md
tests:
  - zircon_runtime_interface/src/tests/reflect_contracts.rs
  - zircon_runtime_interface/src/tests/reflect_value_budget_contracts.rs
  - tests/acceptance/reflection-type-registry.md
  - cargo check -p zircon_runtime_interface --locked --message-format short
  - cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short
doc_type: module-detail
---

# Runtime Interface Reflection Contracts

## Purpose

`zircon_runtime_interface::reflect` owns the neutral serialized reflection contract shared by runtime, editor, plugin tooling, persistence, remote consumers, and script VMs. It contains type identity, object addresses, schema metadata, reflected field values, read/write request and response shapes, tagged values, structured errors, and the runtime-neutral `ZrReflect` / `ZrReflectValue` derive contracts.

This interface module must not depend on `zircon_runtime`, `zircon_editor`, `World`, ECS storage, adapters, render code, IO, or service managers. Runtime behavior such as `TypeRegistry`, component adapters, resource adapters, field conversion, dirty-state mutation, and `WorldReflection` dispatch belongs in `zircon_runtime::scene::reflect` in later M8 milestones.

## DTO Ownership

The `reflect` root is structural and re-exports focused child modules:

- `ReflectTypePath` stores canonical `type_path`, short lookup/display path, and optional module/plugin ownership. Its fields are private; read-only consumers use `type_path()`, `short_type_path()`, `module_path()`, and `plugin_id()`.
- `ReflectTypeKind` describes the reflected type shape.
- `ReflectEditorHint` and `ReflectEnumOption` describe editor and tooling metadata without requiring editor state. `ReflectNumericRange` has a separate declaration/validation owner; its fields are private, its constructor returns `ReflectNumericRangeError`, and serde reuses the same finite/order/positive-step admission before a range can enter schema metadata.
- `ReflectFieldId` is the non-nil canonical 128-bit persistent field identity. `ReflectFieldInfo` stores that ID separately from current name, display name, aliases, ordered schema metadata, editability, serializability, visibility, defaults, numeric ranges, enum options, hints, and documentation.
- `ReflectTypeInfo` groups the type kind and ordered fields.
- `ReflectTypeRole::{Value, Component, Resource}` is the single ECS role classification. `ReflectSerializationStrategy`, `ReflectScriptVisibility`, and `ReflectTypeRegistration` combine that role with serialization, editor, remote, script, and optional documentation policy without storing runtime adapters.
- `ZrReflect` exposes one fallible registration plus generated name-based tooling accessors and numeric field-slot accessors. Dense VM call sites use the numeric surface after load-time resolution; inspector and schema clients retain the name-based surface. `ZrReflectValue` owns neutral conversions between Rust values and `ReflectedValue`; neither trait depends on `World` or runtime storage.
- `ReflectSchemaCatalogEntry`, `ReflectSchemaCatalog`, its immutable snapshot, and
  `ReflectSchemaFingerprint` define the admitted neutral registration set. The catalog owns full/short
  path resolution, explicit ambiguous short names, global field-ID collision admission, scoped legacy
  aliases, dependency closure/order, and the versioned registration-set fingerprint.
- `ReflectSchemaFilter`, `ReflectSchemaRequest`, and `ReflectSchemaResponse` define schema listing/filter
  requests and responses. Every response carries the algorithm version and full catalog fingerprint.
- `ReflectFieldValue`, `ReflectFieldsRequest`, `ReflectFieldsResponse`, `ReflectReadRequest`, `ReflectReadResponse`, `ReflectWriteRequest`, and `ReflectWriteResponse` define the shared field access contract.

`ReflectTypePath::new`, custom deserialization, and its fallible module/plugin builders share one parser. Full paths are at most 512 ASCII bytes and use either Rust `::` or VM `.` separators without mixing them. Rust path segments are identifiers. A VM path uses ASCII key tokens for namespace segments, including canonical plugin IDs with digits or `-`, while its terminal type segment remains an identifier. The short path must equal that terminal segment. An optional module path is at most 384 bytes and must equal the full path prefix. Plugin IDs are at most 128 bytes and use the canonical lowercase ASCII key grammar. Generic arguments are not part of this wire revision. Unknown serialized fields and every state that fails these checks are rejected before a type path reaches a runtime registry. The public byte-limit constants are re-exported from `reflect` so transport and authoring admission can use the same limits.

The other constructor helpers preserve their local contract invariants. `ReflectObjectAddress` reuses the same full type-path parser instead of accepting arbitrary non-empty text. Numeric ranges reject non-finite bounds/steps, `min > max`, and zero or negative steps; open bounds remain explicit `Option` values rather than NaN sentinels. These DTOs do not perform runtime lookup, world mutation, storage access, or plugin loading. Whether a valid range is semantically compatible with a specific field type remains a registry-admission responsibility.

`ReflectTypePath::plugin_id` is the single serialized plugin owner for a registration. The fallible `ReflectTypeRegistration::with_plugin_id` validates and updates that canonical owner; `is_plugin_owned()` is a read-only projection. The retired registration-level identity and ownership fields are not serialized and are rejected as unknown fields during decode. Registry/package consumers project ownership from the type path instead of comparing copies. Likewise, registration wire uses one `role` value instead of independent `is_component`/`is_resource` booleans, so a type cannot decode as both. `serialization` is the persistence strategy, while `serializable` is the explicit visibility/eligibility flag used by filters and tooling; constructors currently initialize it from the strategy and leave it overridable.

`ReflectTypeRegistration::script_visibility` is independent from `remote_visible`. Script host types mark it `Public`, while private runtime metadata remains unavailable to VM projections. Type and field documentation stay on the unified registration, so script ABI descriptors, inspector surfaces, and schema consumers do not maintain parallel field schemas.

## Runtime Field Admission

`ReflectFieldInfo` is a neutral wire DTO. RuntimeInterface `ReflectSchemaCatalog` is the identity,
dependency, path, alias, slot, and fingerprint admission authority; Runtime publication additionally
validates storage/value/editor semantics before changing adapters or schema generations.
`zircon_runtime::scene::reflect::TypeRegistry` contains that catalog plus runtime-only adapter projections.
It rejects over-budget field/enum collections, invalid or duplicate field keys, malformed declared value
types, incompatible defaults or editor hints, numeric metadata on non-numeric declarations, duplicate
enum values, and enum defaults absent from their option set. Registration failures identify the owning
type and field through `ReflectError::InvalidFieldRegistration`.

One bounded declared-value parser owns both policies. General native reflection admits canonical kinds, documented Rust aliases, validated named types, and explicit dynamic `List`/`Map` fields used by heterogeneous native adapters. Strict VM admission accepts only canonical ABI kinds and typed `List<T>` / `Map<String, T>` containers. VM registration constructs and validates its component descriptor once, then publishes the same preflight result into the reflection and component registries. Replaying an identical VM schema is a no-op for catalog generation.

The derive macro infers supported `Vec<T>` fields as typed `List<T>` declarations recursively. It does not infer a bare dynamic list for an unknown element; authors of heterogeneous or domain-specific containers must declare the reflection path and accessors explicitly. Native and script derives generate `ReflectFieldId` from codegen-owned type/field identity keys; an explicit key must be non-empty and already trimmed, and a rename must retain the old key. Runtime never silently hashes a current field name as a fallback.

Numeric field slots remain dense runtime lookup indices, not stable schema identity. The neutral catalog
owns the only ID-to-slot projection: up to 512 fields use a sorted compact index and larger admitted
schemas use a hash index. Runtime registration, VM replacement, removal, and clear mutate that catalog
before the runtime-only adapter projection. Public single-field reads and writes carry `ReflectFieldId`
directly, resolve it through the catalog, and enter the dense component/resource adapter by slot.
`ReflectFieldValue` carries the same stable ID plus the current schema name for authoring and diagnostics;
the name is not a routing identity. Dynamic-scene v3 persistence, editor command history, and VM state use
the same field identity. Generated type and enum-variant identity, generated dependency edges, and an
explicit identity on descriptor-only plugin fields remain follow-on work. Recursive value admission is
owned by the value budget below.

## Object Addressing

`ReflectObjectAddress` is the shared address form for reflected operations:

- `Component { entity, type_path }` targets one component on a stable external entity ID.
- `Resource { type_path }` targets one reflected resource by type path.

`ReflectObjectAddress::component` and `ReflectObjectAddress::resource` apply the canonical full type-path grammar and length budget. The address is intentionally not editor-specific or transport-specific, so editor inspector, remote read/write, and future persistence code can use the same shape.

## Schema And Read/Write Flow

`ReflectSchemaFilter { type_path, include_components, include_resources, editor_visible, remote_visible, include_plugin_owned }` selects one type or a filtered list of editor-visible, remote-visible, component, resource, and plugin-owned registrations. `ReflectSchemaFilter::for_type` and `ReflectSchemaRequest::for_type` include component and resource registrations by default while leaving editor, remote, and plugin-owned flags unset. `ReflectSchemaRequest { filter }` carries that selector as the schema request contract. Runtime M8.7 applies these flags to registry data; the interface crate only defines the stable request/response contract.

`ReflectSchemaResponse { catalog_algorithm_version, catalog_fingerprint, registrations }` returns
deterministic ordered `ReflectTypeRegistration` values bound to one full registration-set identity.
Registration field order is preserved through `Vec<ReflectFieldInfo>` so schema consumers can display or
persist fields predictably. A filtered response does not claim its subset is the whole catalog; clients
compare the full fingerprint before combining pages or cached filters.

Field enumeration and direct access use shared DTOs:

- `ReflectFieldsRequest { address }`
- `ReflectFieldsResponse { address, fields }`
- `ReflectReadRequest { address, field_id }`
- `ReflectReadResponse { address, field }`
- `ReflectFieldValue { field_id, field_name, value }`
- `ReflectWriteRequest { address, field_id, value }`
- `ReflectWriteResponse { address, field, changed }`

Full-field enumeration uses `ReflectFieldsRequest` / `ReflectFieldsResponse` and returns field values in
schema order. Single-field reads and writes route by stable `field_id`. Returned `field_name` values are the
current admitted schema name for display and diagnostics, never a second identity authority.

## Reflected Values

`ReflectedValue` uses a stable tagged serde representation with `kind` and `value` fields. Supported shapes are null, booleans, signed and unsigned integers, `f32` scalars, strings, enums, `Vec2`, `Vec3`, `Vec4`, quaternions, optional entity IDs, resource IDs, lists, deterministic string-keyed maps, and raw JSON values.

Maps use `BTreeMap<String, ReflectedValue>` so serialized output is stable across runs. Untagged polymorphic JSON is not used because it would make editor tooling, remote transport, and persistence migrations ambiguous.

`ReflectValueBudget` is a caller-owned policy with depth, total-node, cumulative UTF-8 string-byte, and per-container entry limits. `ReflectedValue::validate_with_budget` performs one non-recursive flat-work-stack pass over both the tagged value tree and an embedded raw JSON graph. The root tagged value has depth 1; an embedded JSON root is the child of its `Json` wrapper. Tagged values and JSON values count as nodes, while string/enum/resource payloads plus map/object keys count toward cumulative string bytes. Every scalar/vector/quaternion component must be finite. Checked accounting and `ReflectValueValidationError` make rejection machine-readable.

The interface does not define a product-wide default. Runtime owns one boundary policy and applies it before schema publication, editor/remote mutation or response publication, world-query inspection, dynamic component admission, dynamic-scene capture/spawn, reflected JSON persistence, and VM reflected state/schema migration. A per-value budget does not replace an outer response/envelope byte, item, time, paging, cancellation, or backpressure policy. Oversized inline values currently fail closed; a paged/bulk handle requires a separate transport and lifetime owner.

## Error Model

`ReflectError` is a serializable structured error enum for schema, lookup, field access, conversion, registration, and runtime value-admission failures. Variants carry explicit type path, field name, entity, source, target, and reason context as applicable. `InvalidValue` reports the type/field boundary that rejected a `ReflectValueValidationError`; malformed schema defaults remain `InvalidFieldRegistration`.

The `Display` implementation is diagnostic text only. Callers that need machine-readable data should match the enum instead of parsing display strings.

## M8.1 Validation Commands

M8.1 validation is scoped to the interface contract slice and does not claim workspace-wide readiness:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime_interface --locked --message-format short
cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short
git diff --check -- "zircon_runtime_interface/src/lib.rs" "zircon_runtime_interface/src/reflect" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/reflect_contracts.rs" "docs/zircon_runtime_interface/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md" "docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md" "docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md"
```

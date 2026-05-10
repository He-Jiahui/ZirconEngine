---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/ecs/component_registry.rs
  - zircon_runtime/src/scene/ecs/resource_registry.rs
  - zircon_runtime/src/scene/ecs/resource_store.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_type_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_property_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/constructors.rs
implementation_files:
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/reflect/type_path.rs
  - zircon_runtime_interface/src/reflect/type_kind.rs
  - zircon_runtime_interface/src/reflect/editor_hint.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/reflect/schema.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/field_info.rs
  - zircon_runtime_interface/src/reflect/type_info.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/reflect/error.rs
  - zircon_runtime/src/scene/reflect/mod.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/registration.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/reflect_resource.rs
  - zircon_runtime/src/scene/reflect/conversion.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/world_api.rs
  - zircon_runtime/src/scene/reflect/fixed/mod.rs
  - zircon_runtime/src/scene/reflect/fixed/name.rs
  - zircon_runtime/src/scene/reflect/fixed/local_transform.rs
  - zircon_runtime/src/scene/reflect/fixed/active_self.rs
  - zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs
plan_sources:
  - user: 2026-05-08 approved full staged Reflection/TypeRegistry design
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md
tests:
  - zircon_runtime_interface/src/tests/reflect_contracts.rs
  - zircon_runtime/src/scene/tests/ecs_reflect.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api.rs
  - zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs
  - tests/acceptance/reflection-type-registry.md
doc_type: approved-design
---

# Reflection Type Registry Design

## Goal

Build the M8 Reflection And Type Registry foundation from the Bevy-grade ECS roadmap as a full staged architecture. Reflection must become the shared schema, read/write, persistence, editor-inspector, and Remote/BRP capability spine for fixed Rust components, scene resources, and dynamic plugin JSON components while preserving `zircon_runtime::scene::World` authority.

## Approved Direction

Use the full staged design. The first implementation slice is manual registration, runtime registry, field schema, shared object addressing, schema/read/write DTOs, a unified `WorldReflection` facade, fixed component adapters, dynamic JSON component adapters, one resource adapter, and proof that editor and Remote/BRP will share those DTOs. Proc macros, full dynamic scene persistence, editor-wide inspector replacement, and concrete Remote/BRP transport endpoints are staged follow-up milestones that depend on the same contracts.

This design does not import `bevy_ecs` or `bevy_reflect`. Bevy is used as precedent for the split between neutral type metadata and ECS-specific adapters. Zircon owns the runtime model and keeps stable external `EntityId = u64`.

## Current Baseline

`World` already owns typed ECS identity/storage, component/resource registries, fixed component maps, dynamic JSON components, and property-path read/write helpers. The current `ComponentTypeRegistry` stores plugin `ComponentTypeDescriptor` values keyed by string `type_id`; it is not a general reflection registry. Dynamic component writes already validate registered plugin properties, but fixed component schema is duplicated in property access code and resources have no reflection metadata.

The approved architecture adds reflection alongside current ECS state. It does not remove existing scene property APIs in the first slice; those APIs become implementation support and compatibility surfaces while reflection becomes the new authoritative schema path.

## Reference Evidence

### Bevy

Files consulted:

- `dev/bevy/crates/bevy_reflect/src/type_registry.rs`
- `dev/bevy/crates/bevy_reflect/src/reflect.rs`
- `dev/bevy/crates/bevy_ecs/src/reflect/component.rs`
- `dev/bevy/crates/bevy_ecs/src/reflect/resource.rs`

Bevy separates `TypeRegistry` / `TypeRegistration` from ECS-specific behavior. `ReflectComponent` is a function-table adapter stored as type data on a registration. Zircon follows that split, but stores serializable DTOs in `zircon_runtime_interface::reflect` and runtime function tables in `zircon_runtime::scene::reflect`.

### Godot

Files consulted:

- `dev/godot/core/object/property_info.h`
- `dev/godot/core/object/object.h`
- `dev/godot/core/object/script_instance.h`

Godot exposes class properties through metadata plus `set`, `get`, and property-list APIs. Zircon follows the same editor-friendly idea: fields carry type, hint, usage, and editability metadata; object mutation goes through the owning runtime object instead of editor-owned state.

### Fyrox

Files consulted:

- `dev/Fyrox/fyrox-core/src/reflect.rs`
- `dev/Fyrox/fyrox-core-derive/tests/it/reflect.rs`

Fyrox reflection emphasizes field metadata, read-only flags, range/step hints, docs, field access, path resolution, and explicit tests. Zircon adopts these metadata categories while keeping the first slice manually registered instead of proc-macro generated.

## Ownership Boundaries

### `zircon_runtime_interface::reflect`

This module owns neutral, serializable contracts shared by runtime, editor, plugins, and remote tooling. It must not depend on `zircon_runtime::scene::World` or runtime storage. It defines type identity, type kind, field schema, editor hints, serialization strategy, reflected value data, and structured errors.

### `zircon_runtime::scene::reflect`

This module owns runtime behavior. It stores registrations in `TypeRegistry` and attaches `ReflectComponent` / `ReflectResource` adapters that can read and mutate `World`. It may call typed component APIs, resource APIs, dynamic JSON component APIs, and existing property-path conversion code.

### `zircon_runtime::scene::World`

`World` remains the runtime authority for entities, components, resources, hierarchy, transform propagation, serialization, and render extract. `World` owns a runtime-only reflection registry that is skipped by world serialization and rebuilt on construction/deserialization.

### `zircon_editor`

The editor consumes reflected schema and read/write APIs. It does not own runtime component data and does not move editor-only state into runtime world serialization.

### Plugins And Remote Tools

Plugin manifests continue to submit `ComponentTypeDescriptor` as the VM/plugin-facing input format. Runtime registration projects that descriptor into `ReflectTypeRegistration`. Remote/BRP endpoints consume the same reflection registry and structured error model once the runtime side is proven.

## Interface Contracts

`zircon_runtime_interface::reflect` defines these serializable DTOs.

### `ReflectTypePath`

Stable type identity:

- `type_path`: full canonical path such as `zircon_runtime::scene::components::Name` or `weather.Component.CloudLayer`.
- `short_type_path`: short display/lookup name such as `Name` or `CloudLayer`.
- `module_path`: optional Rust-style module path when available.
- `plugin_id`: optional plugin owner for dynamic plugin component types.

### `ReflectTypeKind`

Type shape enum:

- `Struct`
- `TupleStruct`
- `Tuple`
- `Enum`
- `List`
- `Map`
- `Scalar`
- `Opaque`
- `Json`

### `ReflectEditorHint`

Editor/runtime hint enum for field UI and tooling:

- `None`
- `String`
- `MultilineString`
- `Bool`
- `Integer`
- `Unsigned`
- `Scalar`
- `Vec2`
- `Vec3`
- `Vec4`
- `Quaternion`
- `Enum`
- `Entity`
- `Resource`
- `Color`
- `Json`

### `ReflectFieldInfo`

Ordered field schema:

- `name`: canonical field name used by reflection APIs.
- `display_name`: human-readable label.
- `value_type_path`: type path for the field value.
- `editable`: whether reflected writes are allowed.
- `serializable`: whether the field can participate in persistence.
- `editor_visible`: whether editor schema projections should show it.
- `default_value`: optional `ReflectedValue` default.
- `numeric_range`: optional `{ min, max, step, precision }` for numeric UI.
- `enum_options`: ordered allowed enum values.
- `editor_hint`: `ReflectEditorHint`.
- `documentation`: optional doc string.

### `ReflectObjectAddress`

Stable address for a reflected object:

- `Component { entity: u64, type_path: String }`: an entity component identified by stable external `EntityId` plus reflected type path.
- `Resource { type_path: String }`: a scene/runtime resource identified by reflected type path.

The address is intentionally not editor-specific and not transport-specific. Editor inspector, diff/patch, and Remote/BRP requests all use the same object address shape.

### `ReflectTypeInfo`

Type metadata:

- `kind`: `ReflectTypeKind`.
- `fields`: ordered `Vec<ReflectFieldInfo>`.

### `ReflectSerializationStrategy`

Persistence strategy enum:

- `None`: not serializable.
- `Value`: serializable through `ReflectedValue`.
- `Json`: serializable as JSON payload.
- `ResourceHandle`: serializable as a resource identifier string.
- `EntityReference`: serializable as an entity reference requiring remap during dynamic scene operations.

### `ReflectTypeRegistration`

Registry entry:

- `type_path`: `ReflectTypePath`.
- `display_name`: human-readable type name.
- `type_info`: `ReflectTypeInfo`.
- `serialization`: `ReflectSerializationStrategy`.
- `is_component`: component adapter may exist.
- `is_resource`: resource adapter may exist.
- `plugin_owned`: dynamic/plugin-owned type.
- `serializable`: field/value persistence may include it.
- `editor_visible`: editor schema projection may include it.
- `remote_visible`: Remote/BRP may expose it through the shared facade.
- `plugin_id`: optional plugin owner id.

### Shared Schema And Read/Write DTOs

These DTOs are part of M8, not a later transport invention:

- `ReflectSchemaFilter { type_path, include_components, include_resources, editor_visible, remote_visible, include_plugin_owned }`: selects one type or a filtered list of editor-visible, remote-visible, component, resource, and plugin-owned registrations.
- `ReflectSchemaRequest { filter }`: carries the schema selection contract.
- `ReflectSchemaResponse { registrations }`: deterministic list of `ReflectTypeRegistration` values.
- `ReflectFieldsRequest { address }`: asks for every reflected field on the addressed object.
- `ReflectFieldsResponse { address, fields }`: ordered field values for the addressed object.
- `ReflectReadRequest { address, field_name }`: asks for one field value.
- `ReflectReadResponse { address, field }`: returns one reflected field value.
- `ReflectFieldValue { field_name, value }`: one reflected field value in a read response.
- `ReflectWriteRequest { address, field_name, value }`: writes one field value.
- `ReflectWriteResponse { address, field, changed }`: reports whether the write changed state and returns the current field value after the write.

`ReflectError` is the error half of every response. Runtime callers may use `Result<Response, ReflectError>` directly; Remote/BRP later maps that result into transport status without changing the DTO contract.

### `ReflectedValue`

Stable reflected value enum:

- `Null`
- `Bool(bool)`
- `Integer(i64)`
- `Unsigned(u64)`
- `Scalar(f32)`
- `String(String)`
- `Enum(String)`
- `Vec2([f32; 2])`
- `Vec3([f32; 3])`
- `Vec4([f32; 4])`
- `Quaternion([f32; 4])`
- `Entity(Option<u64>)`
- `Resource(String)`
- `List(Vec<ReflectedValue>)`
- `Map(BTreeMap<String, ReflectedValue>)`
- `Json(serde_json::Value)`

`ReflectedValue` serializes through stable tagged JSON. Untagged polymorphic JSON is rejected because it makes remote tooling and persistence migrations ambiguous.

### `ReflectError`

Structured error enum:

- `UnknownType { type_path }`
- `AmbiguousShortTypePath { short_type_path }`
- `DuplicateTypePath { type_path }`
- `InvalidTypePath { type_path, reason }`
- `NoComponentAdapter { type_path }`
- `NoResourceAdapter { type_path }`
- `MissingEntity { entity }`
- `MissingComponent { entity, type_path }`
- `MissingResource { type_path }`
- `UnknownField { type_path, field_name }`
- `NonEditableField { type_path, field_name }`
- `TypeMismatch { type_path, field_name, expected, actual }`
- `UnsupportedConversion { source, target }`
- `AddressKindMismatch { expected, actual }`
- `InvalidRegistration { type_path, reason }`

Errors include enough context for editor diagnostics and Remote/BRP responses without parsing strings.

## Runtime Registry

`zircon_runtime::scene::reflect::TypeRegistry` stores registrations by full type path in deterministic order. It also stores short-path lookup entries when the short path is unambiguous. Ambiguous short paths are recorded and return `ReflectError::AmbiguousShortTypePath` rather than guessing.

The registry rejects duplicate full type paths. Intentional replacement is not in the first slice because hot reload needs explicit unload/reload semantics and active component checks.

Each registration may have runtime type data:

- `ReflectComponent` for component operations.
- `ReflectResource` for resource operations.

The registry is runtime-only. `World::empty`, `World::new`, and `Deserialize for World` rebuild fixed registrations. Plugin descriptor registration projects descriptors into the registry after existing descriptor validation succeeds.

## Component Reflection

`ReflectComponent` is a function-table adapter scoped to Zircon’s `World`:

- `contains(world, entity) -> bool`
- `read_field(world, entity, field_name) -> Result<ReflectedValue, ReflectError>`
- `write_field(world, entity, field_name, value) -> Result<bool, ReflectError>`
- `remove(world, entity) -> Result<bool, ReflectError>`
- `reflect_fields(world, entity) -> Result<Vec<ReflectFieldValue>, ReflectError>`

Adapters must call existing `World` mutation paths so dirty state, change ticks, typed component presence, and dynamic component presence stay coherent.

### Fixed Component Initial Set

Manual fixed adapters in the first implementation slice:

- `Name`: `value` as `String`.
- `LocalTransform`: `translation` as `Vec3`, `rotation` as `Vec4`, `scale` as `Vec3`.
- `ActiveSelf`: `value` as `Bool`.
- `RenderLayerMask`: `mask` as `Unsigned`.
- `CameraComponent`: `fov_y_radians`, `z_near`, `z_far` as `Scalar`.
- `RigidBodyComponent`: `body_type` as `Enum`, `mass`, `linear_damping`, `angular_damping`, `gravity_scale` as `Scalar`, `linear_velocity` and `angular_velocity` as `Vec3`, `can_sleep` as `Bool`. Lock arrays are omitted from first-slice writes to avoid inventing lossy vector semantics for `[bool; 3]`.

If a fixed component has internal state that cannot be represented safely, its field is omitted or marked non-editable rather than approximated.

### Dynamic Plugin Components

`ComponentTypeDescriptor` remains the plugin-facing manifest DTO. Registering a descriptor also creates a `ReflectTypeRegistration`:

- `type_path.type_path = descriptor.type_id`
- `type_path.short_type_path` uses the final `.` segment of `type_id`
- `type_path.plugin_id = descriptor.plugin_id`
- `type_info.kind = Json`
- fields derive from `ComponentPropertyDescriptor`
- `is_component = true`
- `is_plugin_owned = true`
- `serialization = Json`

Dynamic adapter read/write uses existing `World::dynamic_component_property` and `World::set_dynamic_component_property`. Reflection returns structured `ReflectError` values for unknown type, unknown field, missing entity, missing component, non-editable field, and conversion failures. Existing lower-level dynamic component methods may keep their current string errors during this slice.

## Resource Reflection

`ReflectResource` mirrors component reflection for scene resources:

- `contains(world) -> bool`
- `reflect_fields(world) -> Result<Vec<ReflectFieldValue>, ReflectError>`
- `read_field(world, field_name) -> Result<ReflectedValue, ReflectError>`
- `write_field(world, field_name, value) -> Result<bool, ReflectError>`

Resource writes use `World::get_resource_mut` / resource tick paths so change detection remains valid. The first slice proves the API with a manual resource adapter in tests and leaves production resource registration opt-in until stable runtime resources are selected.

## Reflected Value Conversion

`zircon_runtime::scene::reflect::conversion` owns conversions between:

- `ReflectedValue`
- `ScenePropertyValue`
- `serde_json::Value`
- supported math/resource/entity DTOs

Unsupported conversions return `ReflectError::UnsupportedConversion`. Vector dimensions must match exactly. Non-finite scalar JSON encoding returns a structured conversion error.

## WorldReflection Facade

`zircon_runtime::scene::reflect::WorldReflection` is the single runtime facade for schema and read/write operations. It wraps `World` access and exposes these operations:

- `list_reflect_types(world, request: ReflectSchemaRequest) -> Result<ReflectSchemaResponse, ReflectError>`
- `reflect_schema(world, type_path: &str) -> Result<ReflectTypeRegistration, ReflectError>`
- `reflect_fields(world, request: ReflectFieldsRequest) -> Result<ReflectFieldsResponse, ReflectError>`
- `reflect_read(world, request: ReflectReadRequest) -> Result<ReflectReadResponse, ReflectError>`
- `reflect_write(world, request: ReflectWriteRequest) -> Result<ReflectWriteResponse, ReflectError>`

`World` may expose convenience methods, but editor and remote code should be able to use the facade DTOs without learning fixed component maps, dynamic JSON storage, or resource internals. This prevents a second schema/read/write contract from appearing in editor or Remote/BRP code.

## Editor Inspector Path

The editor inspector consumes `ReflectSchemaRequest`, `ReflectFieldsRequest`, `ReflectReadRequest`, and `ReflectWriteRequest` through `WorldReflection`. It does not inspect fixed component maps directly. The M8 implementation must include a runtime-interface contract test or runtime scene test proving editor-mode and remote-mode schema filters use the same DTOs and registry data, even if editor call sites are replaced in a later milestone.

## Diff, Patch, And Persistence Path

Scene diff/patch uses `ReflectTypeRegistration` for field enumeration and `ReflectedValue` for values. Entity references use the `Entity` value variant and `EntityReference` serialization strategy so M9 dynamic scene remapping can happen explicitly. Full `DynamicScene` save/load is owned by the M9 milestone and depends on the M8 registry being stable.

## Remote/BRP Path

Remote schema/read/write endpoints consume the same registry and error types:

- schema: list registrations and fields allowed by `remote_visible`
- read: entity/resource plus type path plus optional field
- write: entity/resource plus type path plus field plus `ReflectedValue`

Remote/BRP is not allowed to invent a separate property schema. Its API is a transport projection over the runtime reflection contract.

M8 does not implement the Remote/BRP transport endpoint, but it must provide and test the shared request/response DTOs that endpoint will carry.

## Staged Milestones

1. Public reflect contracts: create serializable DTOs, object addressing, schema/read/write requests and responses, field-value DTOs, and contract tests in `zircon_runtime_interface::reflect`.
2. Runtime registry: add deterministic `TypeRegistry`, `ReflectTypeRegistration` storage, short-path lookup, duplicate errors, and runtime-only `World` ownership.
3. Reflected values and conversions: add tagged JSON roundtrip plus `ScenePropertyValue` and JSON conversion helpers.
4. Fixed component adapters: manually register, enumerate fields with `reflect_fields`, and read/write selected fixed components through reflection.
5. Dynamic plugin component adapters: project `ComponentTypeDescriptor` into reflection and enumerate/read/write dynamic JSON component fields through adapters.
6. Resource reflection: add manual resource adapter registration, field enumeration, and change-tick-aware resource mutation coverage.
7. WorldReflection shared facade proof: route schema, field enumeration, read, and write through shared DTOs and prove editor/remote filters share the same registry data.
8. Editor inspector projection: add a narrow editor schema/read/write projection that consumes reflection without moving editor state into runtime.
9. Scene diff/patch/persistence groundwork: add reflected field diff and patch DTOs that M9 `DynamicScene` can reuse.
10. Remote/BRP projection: expose schema/read/write over the remote capability surface using reflection types and structured errors.
11. Derive/proc-macro follow-up: add `zircon_reflect_derive` only after manual registration semantics are stable.

## Validation Strategy

Milestone testing stages use scoped validation before broader workspace claims:

- `cargo check -p zircon_runtime_interface --locked --message-format short`
- `cargo test -p zircon_runtime_interface --locked --message-format short`
- `cargo check -p zircon_runtime --lib --locked --message-format short`
- `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short`
- `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short`

Because recent repo-local default `target` validation hit a dep-info race, implementation plans should prefer an isolated target directory such as `E:\cargo-targets\zircon-reflect-m8` for scoped validation.

## Acceptance Criteria

The first implementation slice is accepted when:

- `zircon_runtime_interface::reflect` exposes stable serializable contracts and tests pass.
- `ReflectObjectAddress`, schema/read/write request/response DTOs, and reflected field-value DTOs serialize deterministically.
- `World` owns a runtime-only `TypeRegistry` that is rebuilt after construction and deserialization.
- Fixed scene components, dynamic plugin components, and at least one resource adapter are visible through the registry.
- At least one fixed component, one dynamic plugin component, and one resource can be read and written through reflection APIs.
- `WorldReflection` can enumerate schema, enumerate fields, read, and write using only shared DTOs.
- Editor-visible and remote-readable schema filters are proven to use the same registry and DTO surface.
- Reflection writes use existing `World` mutation paths so dirty state and change ticks remain coherent.
- Unknown type, ambiguous short type path, unknown field, missing component/resource, non-editable field, and type mismatch return structured `ReflectError` values.
- `ReflectedValue` tagged JSON roundtrip coverage passes.
- Runtime/interface docs and acceptance evidence record the implementation files and validation commands.

## Explicit Divergence From References

- Zircon uses serializable DTOs in `zircon_runtime_interface` instead of Bevy’s Rust-only `TypeRegistration` shape because editor, plugin, and Remote/BRP surfaces need stable data contracts.
- Zircon starts with manual registration instead of derive macros because the runtime semantics must be proven before code generation is added.
- Zircon’s `ReflectResource` has read/write functions in the first architecture, unlike Bevy’s current marker-style resource reflection, because resources are first-class for editor, persistence, and Remote/BRP tooling.
- Zircon preserves plugin JSON components as dynamic reflection-backed components rather than converting them into Rust types, because VM/plugin boundaries must not require direct Rust object sharing.

## Out Of Scope For The First Slice

- `zircon_reflect_derive` proc macros.
- Full `DynamicScene` save/load and entity remapping.
- Replacing every editor inspector call site.
- Remote/BRP transport endpoints.
- Removing `ComponentTypeDescriptor` from plugin manifests.

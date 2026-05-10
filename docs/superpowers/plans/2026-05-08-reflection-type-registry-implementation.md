# Reflection Type Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved M8 Reflection/TypeRegistry foundation with neutral DTOs, runtime `TypeRegistry`, unified component/resource reflection, and shared Editor inspector plus Remote/devtools facade entrypoints.

**Architecture:** `zircon_runtime_interface::reflect` owns serializable contracts only. `zircon_runtime::scene::reflect` owns runtime adapters, registry state, and `WorldReflection` facade methods that call existing `World` mutation paths. `zircon_editor` and future Remote/devtools consume the same schema/read/write DTOs instead of owning parallel reflection models.

**Tech Stack:** Rust 2021, Cargo workspace, `serde`, `serde_json`, existing `zircon_runtime::scene` ECS/resource APIs, existing plugin `ComponentTypeDescriptor`, milestone-first validation with scoped Cargo commands and isolated `CARGO_TARGET_DIR`.

---

## Repository Context

- Work directly in the existing `main` checkout. Do not create a branch or worktree.
- The worktree is heavily dirty from parallel sessions. Do not revert or normalize unrelated files.
- Refresh `.codex/sessions/` before implementation if more than 4 hours have passed since this plan was written.
- Reflection design source: `docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md`.
- Roadmap source: `.codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md` M8.
- Runtime/editor boundary source: `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md`.
- Do not add `bevy_ecs` or `bevy_reflect` as runtime dependencies.
- Do not run broad `cargo fmt --all`; format only touched files or let focused Cargo commands report formatting-independent compile/test status.

## Architecture Note

- Owner fixed role: runtime absorption layer owns world authority and runtime adapters; neutral contract layer owns shared DTOs; editor authoring host remains a consumer.
- Required abstractions: `ReflectTypeRegistration`, `ReflectObjectAddress`, schema/read/write DTOs, `TypeRegistry`, `ReflectComponent`, `ReflectResource`, and `WorldReflection`.
- Mainstream precedent: Bevy splits `bevy_reflect` type contracts from `bevy_ecs::reflect::{ReflectComponent, ReflectResource}` ECS adapters. Zircon follows that split locally without importing Bevy crates.
- Boundary depth: adding a facade first prevents Editor inspector, Remote/devtools, dynamic plugin components, and resources from each inventing separate field metadata models.
- Validation layers: interface DTO serialization, registry ownership, fixed component mutation coherence, dynamic JSON property coherence, resource change ticks, shared DTO transport, docs evidence.

## File Structure Map

Create these interface files:

- `zircon_runtime_interface/src/reflect/mod.rs`: structural module root and curated re-exports only.
- `zircon_runtime_interface/src/reflect/type_path.rs`: `ReflectTypePath` declaration and constructors.
- `zircon_runtime_interface/src/reflect/type_kind.rs`: `ReflectTypeKind` enum.
- `zircon_runtime_interface/src/reflect/field_info.rs`: `ReflectFieldInfo` declaration and narrow helpers.
- `zircon_runtime_interface/src/reflect/type_info.rs`: `ReflectTypeInfo` declaration.
- `zircon_runtime_interface/src/reflect/type_registration.rs`: `ReflectTypeRegistration` declaration and visibility/filter helpers.
- `zircon_runtime_interface/src/reflect/reflected_value.rs`: `ReflectedValue` declaration and value-kind helpers.
- `zircon_runtime_interface/src/reflect/object_address.rs`: `ReflectObjectAddress` declaration.
- `zircon_runtime_interface/src/reflect/schema.rs`: `ReflectSchemaFilter`, `ReflectSchemaRequest`, and `ReflectSchemaResponse` declarations.
- `zircon_runtime_interface/src/reflect/read_write.rs`: `ReflectFieldValue`, `ReflectFieldsRequest`, `ReflectFieldsResponse`, `ReflectReadRequest`, `ReflectReadResponse`, `ReflectWriteRequest`, and `ReflectWriteResponse` declarations.
- `zircon_runtime_interface/src/reflect/error.rs`: `ReflectError` declaration and `Display` implementation.
- `zircon_runtime_interface/src/lib.rs`: add `pub mod reflect;`.

Create these runtime files:

- `zircon_runtime/src/scene/reflect/mod.rs`: structural module root and curated re-exports only.
- `zircon_runtime/src/scene/reflect/type_registry.rs`: deterministic registry storage, lookup, short-path ambiguity, duplicate rejection, and metadata-only equality/debug behavior.
- `zircon_runtime/src/scene/reflect/reflect_component.rs`: `ReflectComponent` function-table adapter.
- `zircon_runtime/src/scene/reflect/reflect_resource.rs`: `ReflectResource` function-table adapter.
- `zircon_runtime/src/scene/reflect/world_reflection.rs`: shared schema/read/write facade used by public `World` methods.
- `zircon_runtime/src/scene/reflect/conversion.rs`: `ReflectedValue` conversion to/from `ScenePropertyValue` and JSON.
- `zircon_runtime/src/scene/reflect/dynamic_component.rs`: dynamic plugin descriptor projection and adapter functions.
- `zircon_runtime/src/scene/reflect/registration.rs`: fixed/resource registration bootstrap helpers.
- `zircon_runtime/src/scene/reflect/fixed/mod.rs`: fixed component adapter module wiring.
- `zircon_runtime/src/scene/reflect/fixed/name.rs`: `Name` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/local_transform.rs`: `LocalTransform` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/active_self.rs`: `ActiveSelf` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs`: `RenderLayerMask` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs`: selected `RigidBodyComponent` schema and adapter.

Modify these runtime files:

- `zircon_runtime/src/scene/mod.rs`: add `pub mod reflect;` and narrow re-exports.
- `zircon_runtime/src/scene/world/world.rs`: add skipped runtime-only `type_registry: TypeRegistry` and initialize during deserialization.
- `zircon_runtime/src/scene/world/bootstrap.rs`: initialize the reflection registry in `World::empty()`.
- `zircon_runtime/src/scene/world/dynamic_components.rs`: project `ComponentTypeDescriptor` into reflection during registration.
- `zircon_runtime/src/scene/world/mod.rs`: keep structural exports only; do not place reflection behavior here.

Create or modify tests and docs:

- `zircon_runtime_interface/src/tests/reflect_contracts.rs`: interface DTO and value codec tests.
- `zircon_runtime_interface/src/tests/mod.rs`: add `mod reflect_contracts;`.
- `zircon_runtime/src/scene/tests/ecs_reflect.rs`: runtime registry, adapter, dynamic component, resource reflection, and facade tests.
- `zircon_runtime/src/scene/tests/mod.rs`: add `mod ecs_reflect;`.
- `docs/zircon_runtime_interface/reflect.md`: neutral reflection DTO documentation with related-code header.
- `docs/zircon_runtime/scene/reflect.md`: runtime registry/adapters/facade documentation with related-code header.
- `docs/zircon_runtime/scene/ecs.md`: link ECS typed storage, resources, dynamic components, and reflection registry behavior.
- `docs/zircon_editor/scene/viewport/edit_mode_projection.md`: document the future `WorldReflection` inspector field-source seam.
- `tests/acceptance/reflection-type-registry.md`: acceptance evidence and validation results.

## Milestone M8.1: Interface Reflection Contracts

**Goal:** Add stable serializable reflection DTOs in `zircon_runtime_interface::reflect` without any dependency on runtime world storage.

**In-scope behaviors:** Type paths, type kinds, field metadata, registrations, reflected values, object addresses, schema query/response DTOs, field read/write DTOs, structured errors, deterministic serde roundtrips.

**Dependencies:** Existing `zircon_runtime_interface` serde/serde_json dependencies.

**Implementation slices:**

- [ ] Create `zircon_runtime_interface/src/reflect/mod.rs` with only child modules and re-exports:

```rust
//! Serializable reflection DTOs shared by runtime hosts, editors, and tooling.

mod error;
mod field_info;
mod object_address;
mod reflected_value;
mod read_write;
mod schema;
mod type_info;
mod type_kind;
mod type_path;
mod type_registration;

pub use error::ReflectError;
pub use field_info::ReflectFieldInfo;
pub use object_address::ReflectObjectAddress;
pub use reflected_value::ReflectedValue;
pub use read_write::{
    ReflectFieldValue, ReflectFieldsRequest, ReflectFieldsResponse, ReflectReadRequest,
    ReflectReadResponse, ReflectWriteRequest, ReflectWriteResponse,
};
pub use schema::{ReflectSchemaFilter, ReflectSchemaRequest, ReflectSchemaResponse};
pub use type_info::ReflectTypeInfo;
pub use type_kind::ReflectTypeKind;
pub use type_path::ReflectTypePath;
pub use type_registration::ReflectTypeRegistration;
```

- [ ] Add `pub mod reflect;` to `zircon_runtime_interface/src/lib.rs` near the other top-level DTO modules.
- [ ] Add `ReflectTypePath` with this shape:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectTypePath {
    pub type_path: String,
    pub short_type_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
}
```

- [ ] Add `ReflectTypePath::new(type_path, short_type_path) -> Result<Self, ReflectError>`, `with_module_path`, and `with_plugin_id`. The constructor returns `ReflectError::InvalidTypePath` when either required string is empty after trimming.
- [ ] Add `ReflectTypeKind` with serde snake-case variants: `Struct`, `TupleStruct`, `Tuple`, `Enum`, `List`, `Map`, `Scalar`, `Opaque`, `Json`.
- [ ] Add `ReflectFieldInfo` with fields `name`, `display_name`, `value_type_path`, `editable`, `default_value`, and `documentation`. Keep `default_value` and `documentation` optional with `skip_serializing_if`.
- [ ] Add `ReflectTypeInfo { pub kind: ReflectTypeKind, pub fields: Vec<ReflectFieldInfo> }` plus constructors `struct_with_fields`, `json_with_fields`, and `opaque`.
- [ ] Add `ReflectTypeRegistration` with fields `type_path`, `display_name`, `type_info`, `is_component`, `is_resource`, `plugin_owned`, `serializable`, `editor_visible`, `remote_visible`, and `plugin_id`.
- [ ] Add `ReflectedValue` as a tagged serde enum with variants `Bool(bool)`, `Integer(i64)`, `Unsigned(u64)`, `Scalar(f32)`, `String(String)`, `Enum(String)`, `Vec2([f32; 2])`, `Vec3([f32; 3])`, `Vec4([f32; 4])`, `Quaternion([f32; 4])`, `Entity(Option<u64>)`, `Resource(String)`, `List(Vec<ReflectedValue>)`, `Map(BTreeMap<String, ReflectedValue>)`, `Json(serde_json::Value)`, and `Null`.
- [ ] Add `ReflectObjectAddress` as a tagged serde enum with variants `Component { entity: u64, type_path: String }` and `Resource { type_path: String }`.
- [ ] Add `ReflectSchemaFilter` with fields `type_path: Option<String>`, `include_components: bool`, `include_resources: bool`, `editor_visible: bool`, `remote_visible: bool`, and `include_plugin_owned: bool`. Implement `ReflectSchemaFilter::editor_visible()`, `ReflectSchemaFilter::remote_visible()`, and `ReflectSchemaFilter::for_type()` constructors.
- [ ] Add `ReflectSchemaRequest { pub filter: ReflectSchemaFilter }` with equivalent `editor_visible`, `remote_visible`, and `for_type` constructors.
- [ ] Add `ReflectSchemaResponse { pub registrations: Vec<ReflectTypeRegistration> }` and keep registrations sorted by full type path when constructed by runtime code.
- [ ] Add `ReflectFieldValue { pub field_name: String, pub value: ReflectedValue }`.
- [ ] Add `ReflectFieldsRequest { pub address: ReflectObjectAddress }` and `ReflectFieldsResponse { pub address: ReflectObjectAddress, pub fields: Vec<ReflectFieldValue> }` for full-field enumeration.
- [ ] Add `ReflectReadRequest { pub address: ReflectObjectAddress, pub field_name: String }` and `ReflectReadResponse { pub address: ReflectObjectAddress, pub field: ReflectFieldValue }`.
- [ ] Add `ReflectWriteRequest { pub address: ReflectObjectAddress, pub field_name: String, pub value: ReflectedValue }` and `ReflectWriteResponse { pub address: ReflectObjectAddress, pub field: ReflectFieldValue, pub changed: bool }`.
- [ ] Add `ReflectError` with variants `InvalidTypePath`, `UnknownType`, `UnknownField`, `NonEditableField`, `TypeMismatch`, `MissingEntity`, `MissingComponent`, `MissingResource`, `AddressKindMismatch`, `NoComponentAdapter`, `NoResourceAdapter`, `UnsupportedConversion`, `DuplicateTypePath`, and `AmbiguousShortTypePath`. Each variant that names a type or field must carry the relevant string context.
- [ ] Implement `Display` for `ReflectError` using explicit context from the enum; do not parse legacy string errors here.
- [ ] Add `zircon_runtime_interface/src/tests/reflect_contracts.rs` with tests named:
  - `type_registration_serializes_with_ordered_fields`
  - `reflected_value_tagged_json_roundtrips_all_supported_shapes`
  - `field_metadata_preserves_editability_defaults_and_docs`
  - `reflect_object_address_schema_and_read_write_dtos_roundtrip`
  - `reflect_error_display_includes_type_field_and_entity_context`
- [ ] Update `zircon_runtime_interface/src/tests/mod.rs` with `mod reflect_contracts;`.
- [ ] Create `docs/zircon_runtime_interface/reflect.md` with the required related-code header, DTO ownership rules, value shape, schema/read/write contracts, error model, and validation command list.

**Testing stage:**

- [ ] Set an isolated target dir for local evidence:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
```

- [ ] Run interface checks:

```powershell
cargo check -p zircon_runtime_interface --locked --message-format short
cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short
```

- [ ] If a test fails, fix the lowest DTO/serde/error layer first, then rerun both commands.
- [ ] Record command output summaries in `tests/acceptance/reflection-type-registry.md`.

**Exit evidence:** Both interface commands pass with no unrelated failure claimed as fixed.

## Milestone M8.2: Runtime TypeRegistry And World Ownership

**Goal:** Add runtime-only reflection registry ownership to `World` and deterministic lookup behavior.

**In-scope behaviors:** Registry insert, duplicate full-path rejection, full-path lookup, short-path lookup, ambiguous short-path error, deterministic iteration, runtime-only world field, rebuild on construction and deserialization.

**Dependencies:** M8.1 DTOs.

**Implementation slices:**

- [ ] Create `zircon_runtime/src/scene/reflect/mod.rs` with structural child modules and re-exports:

```rust
mod conversion;
mod dynamic_component;
mod fixed;
mod reflect_component;
mod reflect_resource;
mod registration;
mod type_registry;
mod world_reflection;

pub use reflect_component::ReflectComponent;
pub use reflect_resource::ReflectResource;
pub use registration::register_builtin_reflection;
pub use type_registry::{RuntimeTypeRegistration, TypeRegistry};
pub use world_reflection::WorldReflection;
```

- [ ] Add `pub mod reflect;` to `zircon_runtime/src/scene/mod.rs` and re-export only `ReflectComponent`, `ReflectResource`, `TypeRegistry`, and `WorldReflection` if scene users need them.
- [ ] Implement `TypeRegistry` in `type_registry.rs` with fields:

```rust
use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::reflect::{ReflectError, ReflectTypeRegistration};

#[derive(Clone, Default)]
pub struct TypeRegistry {
    registrations: BTreeMap<String, RuntimeTypeRegistration>,
    short_paths: BTreeMap<String, String>,
    ambiguous_short_paths: BTreeSet<String>,
}

#[derive(Clone)]
pub struct RuntimeTypeRegistration {
    pub registration: ReflectTypeRegistration,
    pub component: Option<crate::scene::reflect::ReflectComponent>,
    pub resource: Option<crate::scene::reflect::ReflectResource>,
}
```

- [ ] Implement `TypeRegistry::register`, `TypeRegistry::registration`, `TypeRegistry::resolve`, `TypeRegistry::runtime_registration`, `TypeRegistry::iter`, `TypeRegistry::contains`, and `TypeRegistry::clear`.
- [ ] `TypeRegistry::resolve` first checks full type path, then unambiguous short type path, then returns `UnknownType` or `AmbiguousShortTypePath`.
- [ ] Implement `Debug` and `PartialEq` manually for `TypeRegistry` and `RuntimeTypeRegistration` by comparing metadata registrations and short-path state only. Do not compare function-table pointer identity.
- [ ] Add `#[serde(skip, default)] pub(super) type_registry: TypeRegistry` to `World` in `zircon_runtime/src/scene/world/world.rs`.
- [ ] Initialize `type_registry` in `World::empty()` by constructing the world, calling `crate::scene::reflect::register_builtin_reflection(&mut world)`, and returning the world.
- [ ] Initialize `type_registry` in `Deserialize for World` before `rebuild_entity_registry()` and `rebuild_typed_component_presence()` return the world. Call the same fixed registration bootstrap used by `World::empty()`.
- [ ] Add `World::type_registry(&self) -> &TypeRegistry` in `world_reflection.rs` through an `impl World` block.
- [ ] Add tests in `zircon_runtime/src/scene/tests/ecs_reflect.rs` named:
  - `empty_world_builds_runtime_only_type_registry`
  - `type_registry_rejects_duplicate_full_type_paths`
  - `type_registry_short_path_lookup_reports_ambiguity`
  - `world_serialization_skips_reflection_registry_and_rebuilds_it_on_load`
- [ ] Create `docs/zircon_runtime/scene/reflect.md` with the required related-code header and registry ownership rules.

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
```

If compile fails because `World` derives require narrower registry equality or debug behavior, fix the registry trait implementations before changing `World` derives.

**Exit evidence:** Registry tests pass and serialized `World` output does not include reflection registry internals.

## Milestone M8.3: Facade DTO Flow And Value Conversion

**Goal:** Add `WorldReflection` facade methods and conversion helpers before concrete component/resource adapters depend on them.

**In-scope behaviors:** Schema filtering, field read/write request routing by address, complete-field response shape, scalar/vector/entity/resource/string/bool/json conversion, unsupported conversion errors, non-finite scalar rejection.

**Dependencies:** M8.1 DTOs and M8.2 registry ownership.

**Implementation slices:**

- [ ] Implement `conversion.rs` functions:

```rust
pub fn reflected_from_scene_value(value: ScenePropertyValue) -> Result<ReflectedValue, ReflectError>;
pub fn scene_value_from_reflected(value: ReflectedValue) -> Result<ScenePropertyValue, ReflectError>;
pub fn reflected_from_json(value: serde_json::Value) -> ReflectedValue;
pub fn json_from_reflected(value: ReflectedValue) -> Result<serde_json::Value, ReflectError>;
```

- [ ] Map `ScenePropertyValue::Quaternion([x, y, z, w])` to `ReflectedValue::Quaternion([x, y, z, w])`. Do not make generic `Vec4` writes become quaternion writes in the conversion helper.
- [ ] Map `ScenePropertyValue::AnimationParameter` to `ReflectError::UnsupportedConversion` until animation parameter reflection has its own value shape.
- [ ] Reject `ReflectedValue::Scalar` and vector values that contain non-finite floats when converting to JSON or `ScenePropertyValue`.
- [ ] Implement `WorldReflection` in `world_reflection.rs` as a zero-sized facade with these methods:

```rust
pub struct WorldReflection;

impl WorldReflection {
    pub fn list_reflect_types(
        world: &World,
        request: ReflectSchemaRequest,
    ) -> Result<ReflectSchemaResponse, ReflectError>;

    pub fn reflect_schema(
        world: &World,
        type_path: &str,
    ) -> Result<ReflectTypeRegistration, ReflectError>;

    pub fn reflect_fields(
        world: &World,
        request: ReflectFieldsRequest,
    ) -> Result<ReflectFieldsResponse, ReflectError>;

    pub fn reflect_read(
        world: &World,
        request: ReflectReadRequest,
    ) -> Result<ReflectReadResponse, ReflectError>;

    pub fn reflect_write(
        world: &mut World,
        request: ReflectWriteRequest,
    ) -> Result<ReflectWriteResponse, ReflectError>;
}
```

- [ ] Add public `World` wrappers with matching names: `list_reflect_types`, `reflect_schema`, `reflect_read`, `reflect_write`, and `reflect_fields`.
- [ ] The facade must route `Component` addresses to component adapters and `Resource` addresses to resource adapters.
- [ ] `ReflectFieldsRequest` must call the adapter `read_fields`; `ReflectReadRequest` must call `read_field` and wrap one `ReflectFieldValue` in the response.
- [ ] `ReflectWriteResponse.field` must be read back after a successful write so Editor and Remote callers receive the normalized value.
- [ ] Add conversion/facade tests named:
  - `scene_property_values_convert_to_reflected_values`
  - `reflected_values_convert_to_scene_property_values_when_supported`
  - `reflected_json_conversion_rejects_non_finite_scalars`
  - `animation_parameter_conversion_returns_structured_error`
  - `world_reflection_routes_component_and_resource_addresses`

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
```

**Exit evidence:** Conversion tests pass and facade routing returns structured errors before concrete adapters are available.

## Milestone M8.4: Fixed Component Reflection Adapters

**Goal:** Read and write selected fixed scene components through the unified `WorldReflection` APIs.

**In-scope behaviors:** Manual fixed registration, `ReflectComponent` adapter calls, selected fixed field schema, read/write with dirty-state/change-tick preservation.

**Dependencies:** M8.2 registry and M8.3 facade/conversion helpers.

**Implementation slices:**

- [ ] Define `ReflectComponent` in `reflect_component.rs`:

```rust
#[derive(Clone, Copy)]
pub struct ReflectComponent {
    pub contains: fn(&World, EntityId) -> bool,
    pub read_field: fn(&World, EntityId, &str) -> Result<ReflectedValue, ReflectError>,
    pub read_fields: fn(&World, EntityId) -> Result<Vec<ReflectFieldValue>, ReflectError>,
    pub write_field: fn(&mut World, EntityId, &str, ReflectedValue) -> Result<bool, ReflectError>,
    pub remove: fn(&mut World, EntityId) -> Result<bool, ReflectError>,
}
```

- [ ] Add forwarding methods on `ReflectComponent` for `contains`, `read_field`, `read_fields`, `write_field`, and `remove`.
- [ ] Implement fixed adapters under `zircon_runtime/src/scene/reflect/fixed/` for `Name`, `LocalTransform`, `ActiveSelf`, `RenderLayerMask`, and `RigidBodyComponent`.
- [ ] Register fixed component type paths exactly as:
  - `zircon_runtime::scene::components::Name`
  - `zircon_runtime::scene::components::LocalTransform`
  - `zircon_runtime::scene::components::ActiveSelf`
  - `zircon_runtime::scene::components::RenderLayerMask`
  - `zircon_runtime::scene::components::RigidBodyComponent`
- [ ] `Name` exposes writable field `value: String` and writes through `World::insert(entity, Name(value))`.
- [ ] `ActiveSelf` exposes writable field `value: Bool` and writes through `World::insert(entity, ActiveSelf(value))` so active dirty state and component ticks are updated.
- [ ] `RenderLayerMask` exposes writable field `mask: Unsigned` and writes through `World::insert(entity, RenderLayerMask(mask as u32))` after rejecting values greater than `u32::MAX` with `TypeMismatch`.
- [ ] `LocalTransform` exposes `translation: Vec3`, `rotation: Vec4`, and `scale: Vec3`. `translation` and `scale` are writable through `World::get_mut::<LocalTransform>(entity)`; `rotation` is readable and registered as non-editable in this slice.
- [ ] `RigidBodyComponent` exposes writable scalar/bool fields `mass`, `linear_damping`, `angular_damping`, `gravity_scale`, and `can_sleep`; it exposes read-only fields `body_type`, `linear_velocity`, `angular_velocity`, `lock_translation`, and `lock_rotation`.
- [ ] Each fixed adapter must return `MissingEntity` when the entity does not exist, `MissingComponent` when the entity lacks the component, `UnknownField` when the field name is absent, and `NonEditableField` when a read-only field is written.
- [ ] Add `register_builtin_reflection(world: &mut World)` in `registration.rs`. It inserts all fixed component registrations into `world.type_registry` and may be called safely on a fresh default registry.
- [ ] Add tests named:
  - `fixed_component_registrations_exist_in_empty_world`
  - `name_component_reads_and_writes_through_world_reflection`
  - `active_self_reflection_write_marks_active_dirty_state`
  - `local_transform_reflection_write_marks_transform_dirty_state`
  - `local_transform_rotation_is_readable_but_not_writable_in_m8`
  - `render_layer_mask_reflection_roundtrips_unsigned_mask`
  - `rigid_body_reflection_exposes_selected_safe_fields`
  - `unknown_fixed_field_returns_structured_error`
  - `missing_fixed_component_returns_structured_error`

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short
```

If dirty-state tests fail, inspect `zircon_runtime/src/scene/world/typed_api.rs`, `zircon_runtime/src/scene/world/change_detection.rs`, and `zircon_runtime/src/scene/world/dirty_state.rs` before adding adapter-local dirty flags.

**Exit evidence:** Fixed reflected writes mutate the same world state as existing typed APIs and do not regress typed ECS tests.

## Milestone M8.5: Dynamic Plugin JSON Component Reflection

**Goal:** Project existing plugin component descriptors into the reflection registry and read/write dynamic JSON fields through the same facade as fixed components.

**In-scope behaviors:** Descriptor projection, plugin-owned JSON registration, dynamic adapter reads/writes, non-editable field errors, unknown field errors, existing unload guard behavior preserved.

**Dependencies:** M8.2 registry, M8.3 facade/conversion, M8.4 `ReflectComponent` API.

**Implementation slices:**

- [ ] Implement `dynamic_component.rs` functions:

```rust
pub fn registration_from_component_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<ReflectTypeRegistration, ReflectError>;

pub fn reflect_component_for_dynamic_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> ReflectComponent;
```

- [ ] Dynamic descriptor projection sets `ReflectTypeKind::Json`, `is_component = true`, `is_resource = false`, `plugin_owned = true`, `serializable = true`, `editor_visible = true`, `remote_visible = true`, and `plugin_id = Some(descriptor.plugin_id.clone())`.
- [ ] Dynamic field metadata maps `ComponentPropertyDescriptor.name` to `ReflectFieldInfo.name`, `display_name` to the same string, `value_type_path` to the descriptor `value_type`, and `editable` to the descriptor `editable` flag.
- [ ] Modify `World::register_component_type` in `dynamic_components.rs` so descriptor projection and reflection-registry duplicate preflight happen before mutating `component_types`.
- [ ] The mutation order must be: create `ReflectTypeRegistration`, check `self.type_registry.contains(&descriptor.type_id)`, call `self.component_types.register(descriptor.clone())`, then insert a `RuntimeTypeRegistration` with the dynamic `ReflectComponent` adapter. If any preflight fails, return a string converted from `ReflectError` and leave both registries unchanged.
- [ ] Dynamic `contains` returns true only when the entity exists and `World::dynamic_component(entity, type_path)` is present.
- [ ] Dynamic `read_field` constructs `ComponentPropertyPath::parse(&format!("{type_path}.{field_name}"))` and calls `World::dynamic_component_property`.
- [ ] Dynamic `read_fields` iterates reflected field metadata in schema order and reads declared fields only.
- [ ] Dynamic `write_field` validates editability through reflected field metadata, converts `ReflectedValue` to `ScenePropertyValue`, and calls `World::set_dynamic_component_property`.
- [ ] Add tests named:
  - `dynamic_component_descriptor_registers_reflected_json_component`
  - `dynamic_component_reflection_reads_json_property_through_facade`
  - `dynamic_component_reflection_writes_json_property_through_facade`
  - `dynamic_component_reflection_rejects_non_editable_property`
  - `dynamic_component_reflection_unknown_type_and_field_are_structured_errors`
  - `plugin_unload_guard_still_counts_reflected_dynamic_components`
- [ ] Keep existing tests in `zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs` unchanged unless reflection insertion exposes a real defect.

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib plugin_extensions::dynamic_components --locked --message-format short
```

**Exit evidence:** Descriptor-backed dynamic components appear in `TypeRegistry`, reflection read/write uses existing JSON property paths, and plugin unload behavior remains intact.

## Milestone M8.6: Resource Reflection

**Goal:** Prove reflection is not component-only by adding resource adapter registration and change-tick-aware resource writes.

**In-scope behaviors:** Manual resource adapter, resource schema, resource read/write through `ReflectObjectAddress::Resource`, missing resource error, resource change tick update.

**Dependencies:** M8.2 registry, M8.3 facade/conversion, and existing `World::{insert_resource,get_resource,get_resource_mut,resource_change_ticks}` APIs.

**Implementation slices:**

- [ ] Define `ReflectResource` in `reflect_resource.rs`:

```rust
#[derive(Clone, Copy)]
pub struct ReflectResource {
    pub contains: fn(&World) -> bool,
    pub read_field: fn(&World, &str) -> Result<ReflectedValue, ReflectError>,
    pub read_fields: fn(&World) -> Result<Vec<ReflectFieldValue>, ReflectError>,
    pub write_field: fn(&mut World, &str, ReflectedValue) -> Result<bool, ReflectError>,
}
```

- [ ] Add forwarding methods on `ReflectResource` for `contains`, `read_field`, `read_fields`, and `write_field`.
- [ ] Add `TypeRegistry::register_resource` convenience helper that stores a `RuntimeTypeRegistration` with `resource: Some(adapter)` and `component: None`.
- [ ] Add a test-only `FrameCounter { value: u32 }` resource adapter in `ecs_reflect.rs`. The adapter registers type path `zircon_runtime::scene::tests::ecs_reflect::FrameCounter`, exposes writable field `value: Unsigned`, reads through `World::get_resource::<FrameCounter>()`, and writes through `World::get_resource_mut::<FrameCounter>()`.
- [ ] Ensure `WorldReflection::reflect_read` and `WorldReflection::reflect_write` route `ReflectObjectAddress::Resource` through `ReflectResource` and return `NoResourceAdapter` if the type registration is not a resource.
- [ ] Add tests named:
  - `manual_resource_registration_adds_reflected_resource_schema`
  - `resource_reflection_reads_and_writes_field_through_facade`
  - `resource_reflection_write_updates_change_tick`
  - `missing_reflected_resource_returns_structured_error`
  - `component_and_resource_reflection_share_address_and_facade_shape`

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short
```

**Exit evidence:** The registry can store resource registrations and reflected writes update resource change ticks.

## Milestone M8.7: Editor Inspector And Remote DTO Reuse Proof

**Goal:** Prove Editor inspector and Remote/devtools can reuse the same runtime reflection surface without rewriting every editor panel or adding network transport.

**In-scope behaviors:** Inspector-style full-field listing through `reflect_fields`, schema/read/write DTO serialization through `serde_json`, docs for editor seam, no direct inspector dependency on `ComponentTypeRegistry` in new tests.

**Dependencies:** M8.1 through M8.6.

**Implementation slices:**

- [ ] Add a runtime test named `inspector_style_field_list_uses_world_reflection_facade`. It must create an entity with `Name`, `ActiveSelf`, and one dynamic plugin component, then call `world.reflect_fields(ReflectFieldsRequest { address: ReflectObjectAddress::Component { ... } })` for each reflected type and assert field names/values without calling `component_type_descriptors()` or fixed component maps.
- [ ] Add a runtime test named `remote_style_schema_read_response_serializes_without_runtime_handles`. It must call `world.list_reflect_types(ReflectSchemaRequest::remote_visible())`, call `world.reflect_read(...)` for one component field, serialize both responses through `serde_json::to_string`, deserialize them, and assert equality.
- [ ] Add a runtime test named `remote_style_write_request_serializes_and_mutates_through_facade`. It must serialize a `ReflectWriteRequest`, deserialize it, pass it to `world.reflect_write(...)`, and verify the world changed.
- [ ] Update `docs/zircon_editor/scene/viewport/edit_mode_projection.md` to state that the future inspector field-source seam is `WorldReflection` producing existing `SceneInspectorField` data, while editor selection/tools/history remain in `zircon_editor`.
- [ ] Do not rewrite `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs` in this milestone. The acceptance proof is runtime-side DTO and facade reuse plus documentation of the editor replacement point.
- [ ] Do not add sockets, HTTP routes, BRP handlers, or session authorization in this milestone. Remote/devtools proof is the serializable schema/read/write DTO shape.

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
```

**Exit evidence:** Inspector-style and remote-style tests pass through `WorldReflection` and serialized DTOs only.

## Milestone M8.8: Documentation And Acceptance Evidence

**Goal:** Make the new reflection module discoverable and record validation evidence for the implemented M8 slice.

**In-scope behaviors:** Module docs, ECS docs linkage, editor seam docs, acceptance evidence, session handoff.

**Dependencies:** M8.1 through M8.7 implementation.

**Implementation slices:**

- [ ] Update `docs/zircon_runtime_interface/reflect.md` with final DTO file list, tests, design source, value codec, address model, schema query model, and request/response semantics.
- [ ] Update `docs/zircon_runtime/scene/reflect.md` with runtime registry/adapters, `World` lifecycle, fixed/dynamic/resource adapter behavior, `WorldReflection` facade rules, and validation commands.
- [ ] Update `docs/zircon_runtime/scene/ecs.md` with the relationship between `ComponentRegistry`, `ResourceRegistry`, dynamic plugin components, and `TypeRegistry`.
- [ ] Update `docs/zircon_editor/scene/viewport/edit_mode_projection.md` with the runtime reflection field-source seam if M8.7 did not already land the final wording.
- [ ] Create or update `tests/acceptance/reflection-type-registry.md` with exact command outputs from each testing stage and known unclaimed workspace validation.
- [ ] Update `.codex/sessions/20260508-2036-reflection-type-registry.md` with status, touched modules, validation evidence, blockers, and next step.

**Testing stage:**

```powershell
git diff --check -- docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md docs/zircon_runtime_interface/reflect.md docs/zircon_runtime/scene/reflect.md docs/zircon_runtime/scene/ecs.md docs/zircon_editor/scene/viewport/edit_mode_projection.md tests/acceptance/reflection-type-registry.md .codex/sessions/20260508-2036-reflection-type-registry.md
```

**Exit evidence:** Docs and acceptance records match implementation files and validation evidence.

## Final Promotion Gate

- [ ] Run scoped runtime/interface validation:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo test -p zircon_runtime_interface --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests --locked --message-format short
```

- [ ] Before claiming workspace-level readiness, run or explicitly defer with evidence:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-workspace"
cargo build --workspace --locked --verbose
cargo test --workspace --locked --verbose
```

- [ ] Record any unrelated pre-existing workspace failures with command output, owning active session if known, and why the M8 scoped acceptance remains valid.

## Plan Self-Review Notes

- Spec coverage: M8.1 covers neutral contracts, including `ReflectObjectAddress` and schema/read/write DTOs. M8.2 covers runtime registry/world ownership. M8.3 covers the `WorldReflection` facade and value conversion. M8.4 covers fixed components. M8.5 covers dynamic plugin JSON components. M8.6 covers resources. M8.7 covers Editor inspector and Remote/devtools shared DTO proof. M8.8 covers docs/evidence.
- Boundary check: interface DTOs stay in `zircon_runtime_interface`; runtime adapters stay in `zircon_runtime::scene::reflect`; editor receives a documented seam without moving editor state into runtime; Remote transport is not implemented in M8.
- Type consistency: public facade names use `list_reflect_types`, `reflect_schema`, `reflect_read`, `reflect_write`, and `reflect_fields` throughout the plan. Component/resource adapters both return `ReflectFieldValue` lists and route through `ReflectObjectAddress`.

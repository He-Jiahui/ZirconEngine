# Reflection Type Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved M8 Reflection/TypeRegistry foundation for fixed components, dynamic plugin JSON components, reflected resources, and staged editor/persistence/remote follow-up work.

**Architecture:** Add stable serializable reflection DTOs in `zircon_runtime_interface::reflect`, then add runtime-only `TypeRegistry` plus component/resource adapters in `zircon_runtime::scene::reflect`. `World` remains authoritative; reflection adapters call existing mutation paths so dirty state, typed presence, dynamic presence, and change ticks stay coherent.

**Tech Stack:** Rust 2021, Cargo workspace, `serde`, `serde_json`, existing `zircon_runtime::scene` ECS/resource APIs, existing plugin `ComponentTypeDescriptor`, milestone-first validation with scoped Cargo commands and isolated `CARGO_TARGET_DIR`.

---

## Repository Context

- Work directly in the existing `main` checkout. Do not create a branch or worktree.
- The worktree is heavily dirty from parallel sessions. Do not revert or normalize unrelated files.
- Active coordination notes currently cover render M4+ design, color foundation design, ECS SystemParam/Commands/Change Detection, app provider wiring, and taskpool foundation. Avoid those areas except when reflection changes directly require compile repair.
- Reflection design source: `docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md`.
- Roadmap source: `.codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md` M8.
- Runtime/editor boundary source: `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md`.

## File Structure Map

Create these interface files:

- `zircon_runtime_interface/src/reflect/mod.rs`: structural module root and curated re-exports only.
- `zircon_runtime_interface/src/reflect/type_path.rs`: `ReflectTypePath` declaration and constructors.
- `zircon_runtime_interface/src/reflect/type_kind.rs`: `ReflectTypeKind` enum.
- `zircon_runtime_interface/src/reflect/editor_hint.rs`: `ReflectEditorHint`, `ReflectNumericRange`, and enum-option DTOs.
- `zircon_runtime_interface/src/reflect/field_info.rs`: `ReflectFieldInfo` declaration and narrow builder-style helpers.
- `zircon_runtime_interface/src/reflect/type_info.rs`: `ReflectTypeInfo` declaration.
- `zircon_runtime_interface/src/reflect/type_registration.rs`: `ReflectSerializationStrategy` and `ReflectTypeRegistration` declaration.
- `zircon_runtime_interface/src/reflect/reflected_value.rs`: `ReflectedValue` declaration and value-name helpers.
- `zircon_runtime_interface/src/reflect/error.rs`: `ReflectError` declaration and `Display` implementation.
- `zircon_runtime_interface/src/lib.rs`: add `pub mod reflect;` and curated re-exports only if needed by existing crate style.

Create these runtime files:

- `zircon_runtime/src/scene/reflect/mod.rs`: structural module root and curated re-exports only.
- `zircon_runtime/src/scene/reflect/type_registry.rs`: deterministic registry storage, lookup, short-path ambiguity, and duplicate rejection.
- `zircon_runtime/src/scene/reflect/registration.rs`: fixed registration bootstrap functions and descriptor projection entry points.
- `zircon_runtime/src/scene/reflect/reflect_component.rs`: `ReflectComponent` function-table adapter.
- `zircon_runtime/src/scene/reflect/reflect_resource.rs`: `ReflectResource` function-table adapter.
- `zircon_runtime/src/scene/reflect/conversion.rs`: `ReflectedValue` conversion to/from `ScenePropertyValue` and JSON.
- `zircon_runtime/src/scene/reflect/dynamic_component.rs`: dynamic plugin component descriptor projection and adapter functions.
- `zircon_runtime/src/scene/reflect/world_api.rs`: public `World` reflection API methods.
- `zircon_runtime/src/scene/reflect/fixed/mod.rs`: fixed component adapter module wiring.
- `zircon_runtime/src/scene/reflect/fixed/name.rs`: `Name` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/local_transform.rs`: `LocalTransform` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/active_self.rs`: `ActiveSelf` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs`: `RenderLayerMask` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/camera_component.rs`: `CameraComponent` schema and adapter.
- `zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs`: selected `RigidBodyComponent` schema and adapter.

Modify these runtime files:

- `zircon_runtime/src/scene/mod.rs`: add `pub mod reflect;` and re-export the reflection types intended for scene users.
- `zircon_runtime/src/scene/world/world.rs`: add `#[serde(skip, default)] pub(super) type_registry: TypeRegistry` and initialize it in deserialization.
- `zircon_runtime/src/scene/world/bootstrap.rs`: initialize the reflection registry in `World::empty()`.
- `zircon_runtime/src/scene/world/dynamic_components.rs`: project `ComponentTypeDescriptor` into reflection during registration.
- `zircon_runtime/src/scene/world/mod.rs`: keep structural exports only; do not place reflection behavior here.

Create/modify tests and docs:

- `zircon_runtime_interface/src/tests/reflect_contracts.rs`: interface DTO and value codec tests.
- `zircon_runtime_interface/src/tests/mod.rs`: add `mod reflect_contracts;`.
- `zircon_runtime/src/scene/tests/ecs_reflect.rs`: runtime registry, adapter, dynamic component, and resource reflection tests.
- `zircon_runtime/src/scene/tests/mod.rs`: add `mod ecs_reflect;`.
- `docs/zircon_runtime_interface/reflect.md`: module documentation with related-code header.
- `docs/zircon_runtime/scene/reflect.md`: runtime registry/adapters documentation with related-code header.
- `docs/zircon_runtime/scene/ecs.md`: link ECS typed storage, dynamic components, and reflection registry behavior.
- `tests/acceptance/reflection-type-registry.md`: acceptance evidence and validation results.

## Milestone M8.1: Interface Reflection Contracts

**Goal:** Add stable serializable reflection DTOs in `zircon_runtime_interface::reflect` without any dependency on runtime world storage.

**In-scope behaviors:** Type path metadata, type kind, editor hint metadata, field info, type info, type registration, reflected value enum, serialization strategy, structured errors, deterministic serde roundtrips.

**Dependencies:** Existing `zircon_runtime_interface` serde/serde_json dependencies.

**Implementation slices:**

- [ ] Create `zircon_runtime_interface/src/reflect/mod.rs` with only child modules and re-exports:

```rust
mod editor_hint;
mod error;
mod field_info;
mod reflected_value;
mod type_info;
mod type_kind;
mod type_path;
mod type_registration;

pub use editor_hint::{ReflectEditorHint, ReflectEnumOption, ReflectNumericRange};
pub use error::ReflectError;
pub use field_info::ReflectFieldInfo;
pub use reflected_value::ReflectedValue;
pub use type_info::ReflectTypeInfo;
pub use type_kind::ReflectTypeKind;
pub use type_path::ReflectTypePath;
pub use type_registration::{ReflectSerializationStrategy, ReflectTypeRegistration};
```

- [ ] Add `pub mod reflect;` to `zircon_runtime_interface/src/lib.rs` near the other top-level DTO modules.
- [ ] Add `ReflectTypePath` with `new(type_path, short_type_path)`, `with_module_path`, and `with_plugin_id` helpers. Validate only non-empty strings in constructors that return `Result<Self, ReflectError>`.
- [ ] Add `ReflectTypeKind`, `ReflectEditorHint`, `ReflectNumericRange`, `ReflectEnumOption`, `ReflectFieldInfo`, `ReflectTypeInfo`, `ReflectSerializationStrategy`, `ReflectTypeRegistration`, `ReflectedValue`, and `ReflectError` exactly as described in the approved spec.
- [ ] Derive `Clone`, `Debug`, `PartialEq`, `Serialize`, and `Deserialize` on public DTOs. Derive `Eq` where fields allow it.
- [ ] Use `BTreeMap<String, ReflectedValue>` for `ReflectedValue::Map` so serde output is deterministic.
- [ ] Implement `Display` for `ReflectError` using explicit context from the enum; do not parse or generate legacy string errors here.
- [ ] Add `zircon_runtime_interface/src/tests/reflect_contracts.rs` with tests named:
  - `type_registration_serializes_with_ordered_fields`
  - `reflected_value_tagged_json_roundtrips_all_supported_shapes`
  - `field_metadata_preserves_editor_hints_defaults_and_ranges`
  - `reflect_error_display_includes_type_field_and_entity_context`
- [ ] Update `zircon_runtime_interface/src/tests/mod.rs` with `mod reflect_contracts;`.
- [ ] Create `docs/zircon_runtime_interface/reflect.md` with the required related-code header, DTO ownership rules, value shape, error model, and validation command list.

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

**In-scope behaviors:** Registry insert, duplicate full-path rejection, short-path lookup, ambiguous short-path error, deterministic iteration, runtime-only world field, rebuild on construction and deserialization.

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
mod world_api;

pub use reflect_component::ReflectComponent;
pub use reflect_resource::ReflectResource;
pub use registration::register_builtin_reflection;
pub use type_registry::TypeRegistry;
```

- [ ] Add `pub mod reflect;` to `zircon_runtime/src/scene/mod.rs` and re-export only `ReflectComponent`, `ReflectResource`, and `TypeRegistry` if scene users need them.
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

- [ ] Add `TypeRegistry::register`, `TypeRegistry::registration`, `TypeRegistry::registration_by_short_path`, `TypeRegistry::runtime_registration`, `TypeRegistry::iter`, and `TypeRegistry::contains`.
- [ ] If tests need debug output, implement `Debug` manually for `TypeRegistry` by printing only registration type paths and ambiguous short paths. Do not require `ReflectComponent` or `ReflectResource` function tables to implement `Debug` or `PartialEq`.
- [ ] Add `#[serde(skip, default)] pub(super) type_registry: TypeRegistry` to `World` in `zircon_runtime/src/scene/world/world.rs`.
- [ ] Initialize `type_registry` in `World::empty()` by creating `TypeRegistry::default()` and calling `crate::scene::reflect::register_builtin_reflection(&mut world)` before returning.
- [ ] Initialize `type_registry` in `Deserialize for World` and call the same fixed registration bootstrap before `rebuild_entity_registry()` and `rebuild_typed_component_presence()` return the world.
- [ ] Add `World::type_registry(&self) -> &TypeRegistry` in `world_api.rs`.
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

If compile fails because `ReflectComponent` or `ReflectResource` needs a narrower clone/debug shape, fix the adapter declarations before changing `World` logic.

**Exit evidence:** Registry tests pass and serialized `World` output does not include reflection registry internals.

## Milestone M8.3: Reflected Value Conversion Layer

**Goal:** Convert between `ReflectedValue`, existing `ScenePropertyValue`, and JSON without panics or lossy guesses.

**In-scope behaviors:** Scalar/vector/entity/resource/string/bool/enum conversion, JSON object/list wrapping, unsupported conversion errors, non-finite scalar rejection.

**Dependencies:** M8.1 DTOs and existing `zircon_runtime::core::framework::scene::ScenePropertyValue`.

**Implementation slices:**

- [ ] Implement `conversion.rs` functions:

```rust
pub fn reflected_from_scene_value(value: ScenePropertyValue) -> ReflectedValue;
pub fn scene_value_from_reflected(value: ReflectedValue) -> Result<ScenePropertyValue, ReflectError>;
pub fn reflected_from_json(value: serde_json::Value) -> ReflectedValue;
pub fn json_from_reflected(value: ReflectedValue) -> Result<serde_json::Value, ReflectError>;
```

- [ ] Map `ScenePropertyValue::Quaternion` to `ReflectedValue::Quaternion`; do not map it to `Vec4`.
- [ ] Map `ScenePropertyValue::AnimationParameter` to `ReflectError::UnsupportedConversion` until animation parameter reflection has a dedicated value shape.
- [ ] Reject `ReflectedValue::Scalar` values that are not finite when converting to JSON.
- [ ] Add tests in `ecs_reflect.rs` named:
  - `scene_property_values_convert_to_reflected_values`
  - `reflected_values_convert_to_scene_property_values_when_supported`
  - `reflected_json_conversion_rejects_non_finite_scalars`
  - `animation_parameter_conversion_returns_structured_error`

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
```

**Exit evidence:** Conversion tests pass and unsupported cases return `ReflectError::UnsupportedConversion`.

## Milestone M8.4: Fixed Component Reflection Adapters

**Goal:** Read and write selected fixed scene components through runtime reflection APIs.

**In-scope behaviors:** Manual fixed registration, `ReflectComponent` adapter calls, selected fixed field schema, read/write with dirty-state/change-tick preservation.

**Dependencies:** M8.2 registry and M8.3 conversion helpers.

**Implementation slices:**

- [ ] Define `ReflectComponent` in `reflect_component.rs`:

```rust
#[derive(Clone)]
pub struct ReflectComponent {
    contains: fn(&World, EntityId) -> bool,
    read_field: fn(&World, EntityId, &str) -> Result<ReflectedValue, ReflectError>,
    write_field: fn(&mut World, EntityId, &str, ReflectedValue) -> Result<bool, ReflectError>,
    remove: fn(&mut World, EntityId) -> Result<bool, ReflectError>,
}
```

- [ ] Add forwarding methods on `ReflectComponent` for `contains`, `read_field`, `write_field`, and `remove`.
- [ ] Implement fixed adapters in separate files under `zircon_runtime/src/scene/reflect/fixed/` for `Name`, `LocalTransform`, `ActiveSelf`, `RenderLayerMask`, `CameraComponent`, and selected `RigidBodyComponent` fields.
- [ ] In each fixed adapter, use existing public `World` APIs where they exist. Use fixed maps only inside `zircon_runtime::scene` when no public setter exists, and call the same dirty-state helpers used by existing property setters.
- [ ] Add `register_builtin_reflection(world: &mut World)` in `registration.rs`. It inserts all fixed component registrations into `world.type_registry`.
- [ ] Add public `World` APIs in `world_api.rs`:

```rust
pub fn read_reflected_component_field(
    &self,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError>;

pub fn write_reflected_component_field(
    &mut self,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError>;

pub fn remove_reflected_component(
    &mut self,
    entity: EntityId,
    type_path: &str,
) -> Result<bool, ReflectError>;
```

- [ ] Add tests named:
  - `fixed_component_registrations_exist_in_empty_world`
  - `name_component_reads_and_writes_through_reflection`
  - `active_self_reflection_write_marks_active_dirty_state`
  - `local_transform_reflection_write_marks_transform_dirty_state`
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

If dirty-state tests fail, inspect `zircon_runtime/src/scene/world/property_access/write.rs`, `zircon_runtime/src/scene/world/typed_api.rs`, and `zircon_runtime/src/scene/world/dirty_state.rs` before adding adapter-local special cases.

**Exit evidence:** Fixed reflected writes mutate the same world state as existing setters and do not regress typed ECS tests.

## Milestone M8.5: Dynamic Plugin JSON Component Reflection

**Goal:** Project existing plugin component descriptors into the reflection registry and read/write dynamic JSON fields through reflection.

**In-scope behaviors:** Descriptor projection, plugin-owned JSON registration, dynamic adapter reads/writes, non-editable field errors, unknown field errors, existing unload guard behavior preserved.

**Dependencies:** M8.2 registry, M8.3 conversion, M8.4 `ReflectComponent` API.

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

- [ ] Map descriptor `value_type` strings to field hints with this first-slice table:
  - `bool` or `boolean` -> `ReflectEditorHint::Bool`
  - `int` or `integer` -> `ReflectEditorHint::Integer`
  - `uint` or `unsigned` -> `ReflectEditorHint::Unsigned`
  - `scalar`, `float`, `number` -> `ReflectEditorHint::Scalar`
  - `string` -> `ReflectEditorHint::String`
  - `vec2` -> `ReflectEditorHint::Vec2`
  - `vec3` -> `ReflectEditorHint::Vec3`
  - `vec4` -> `ReflectEditorHint::Vec4`
  - `entity` -> `ReflectEditorHint::Entity`
  - `resource` -> `ReflectEditorHint::Resource`
  - any other string -> `ReflectEditorHint::Json`
- [ ] Modify `World::register_component_type` in `dynamic_components.rs` so descriptor projection and reflection-registry duplicate preflight happen before mutating `component_types`. Then call `component_types.register(descriptor.clone())` and insert the reflected runtime registration. If reflection projection or duplicate preflight fails, return a string converted from `ReflectError` and do not leave a partially registered descriptor or reflection entry.
- [ ] Dynamic `read_field` constructs `ComponentPropertyPath::parse(&format!("{type_path}.{field_name}"))` and calls `World::dynamic_component_property`.
- [ ] Dynamic `write_field` validates editability through reflected field metadata, converts `ReflectedValue` to `ScenePropertyValue`, and calls `World::set_dynamic_component_property`.
- [ ] Add tests named:
  - `dynamic_component_descriptor_registers_reflected_json_component`
  - `dynamic_component_reflection_reads_json_property`
  - `dynamic_component_reflection_writes_json_property`
  - `dynamic_component_reflection_rejects_non_editable_property`
  - `dynamic_component_reflection_unknown_type_and_field_are_structured_errors`
  - `plugin_unload_guard_still_counts_reflected_dynamic_components`
- [ ] Keep existing tests in `zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs` unchanged unless reflection insertion exposes a real bug.

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

**In-scope behaviors:** Manual resource adapter, resource schema, resource read/write, missing resource error, resource change tick update.

**Dependencies:** M8.2 registry and existing `World::{insert_resource,get_resource,get_resource_mut,resource_change_ticks}` APIs.

**Implementation slices:**

- [ ] Define `ReflectResource` in `reflect_resource.rs`:

```rust
#[derive(Clone)]
pub struct ReflectResource {
    contains: fn(&World) -> bool,
    read_field: fn(&World, &str) -> Result<ReflectedValue, ReflectError>,
    write_field: fn(&mut World, &str, ReflectedValue) -> Result<bool, ReflectError>,
}
```

- [ ] Add forwarding methods on `ReflectResource` for `contains`, `read_field`, and `write_field`.
- [ ] Add public `World` APIs in `world_api.rs`:

```rust
pub fn read_reflected_resource_field(
    &self,
    type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError>;

pub fn write_reflected_resource_field(
    &mut self,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError>;
```

- [ ] Add a test-only `FrameCounter(u32)` resource adapter in `ecs_reflect.rs` that registers `value` as `Unsigned` and writes through `World::get_resource_mut::<FrameCounter>()`.
- [ ] Add tests named:
  - `manual_resource_registration_adds_reflected_resource_schema`
  - `resource_reflection_reads_and_writes_field`
  - `resource_reflection_write_updates_change_tick`
  - `missing_reflected_resource_returns_structured_error`

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short
```

**Exit evidence:** The registry can store resource registrations and reflected writes update resource change ticks.

## Milestone M8.7: Documentation And Acceptance Evidence

**Goal:** Make the new reflection module discoverable and record validation evidence for the implemented M8 slice.

**In-scope behaviors:** Module docs, ECS docs linkage, acceptance evidence, session handoff.

**Dependencies:** M8.1 through M8.6 implementation.

**Implementation slices:**

- [ ] Update `docs/zircon_runtime_interface/reflect.md` with final DTO file list, tests, and design source.
- [ ] Update `docs/zircon_runtime/scene/reflect.md` with runtime registry/adapters, World lifecycle, fixed/dynamic/resource adapter behavior, and validation commands.
- [ ] Update `docs/zircon_runtime/scene/ecs.md` with the relationship between `ComponentRegistry`, `ResourceRegistry`, dynamic plugin components, and `TypeRegistry`.
- [ ] Create or update `tests/acceptance/reflection-type-registry.md` with exact command outputs from each testing stage and known unclaimed workspace validation.
- [ ] Update `.codex/sessions/20260508-2036-reflection-type-registry.md` with status, touched modules, validation evidence, blockers, and next step.

**Testing stage:**

```powershell
git diff --check -- docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md docs/zircon_runtime_interface/reflect.md docs/zircon_runtime/scene/reflect.md docs/zircon_runtime/scene/ecs.md tests/acceptance/reflection-type-registry.md .codex/sessions/20260508-2036-reflection-type-registry.md
```

**Exit evidence:** Docs and acceptance records match implementation files and validation evidence.

## Milestone M8.8: Editor Inspector Projection

**Goal:** Add a narrow editor-facing schema/read/write projection over reflection without replacing every inspector path.

**In-scope behaviors:** Schema projection DTO, one fixed component edit through editor projection, one dynamic plugin component edit through editor projection, runtime/editor boundary preserved.

**Dependencies:** M8.1 through M8.7.

**Implementation slices:**

- [ ] Add an editor-side projection module under the existing inspector/editing structure rather than putting editor state into runtime.
- [ ] The projection reads `World::type_registry()` and calls `World::read_reflected_component_field` / `World::write_reflected_component_field`.
- [ ] Add editor tests that prove fixed and dynamic component edits use reflection and undo/redo command surfaces still own editor history.

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8-editor"
cargo check -p zircon_editor --lib --locked --message-format short
cargo test -p zircon_editor --lib editing::inspector --locked --message-format short
```

**Exit evidence:** Editor projection consumes runtime reflection without adding editor-only state to `World` serialization.

## Milestone M9.1: Reflected Diff And Patch Groundwork

**Goal:** Add reflected diff/patch DTOs that can become the M9 `DynamicScene` persistence substrate.

**In-scope behaviors:** Entity component field diff, resource field diff, entity reference value preservation, structured missing-type diagnostics.

**Dependencies:** M8.1 through M8.7.

**Implementation slices:**

- [ ] Add diff/patch DTOs under `zircon_runtime_interface::reflect` or a dedicated scene persistence module only after confirming ownership with the M9 plan.
- [ ] Use `ReflectedValue::Entity` for entity references and require explicit remap input before applying patches across worlds.
- [ ] Add runtime tests that diff and patch `Name`, `LocalTransform.translation`, one dynamic JSON field, and one reflected resource field.

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m9"
cargo check -p zircon_runtime_interface --locked --message-format short
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
```

**Exit evidence:** Diff/patch tests pass without introducing a parallel schema separate from reflection.

## Milestone Remote.1: Remote/BRP Reflection Projection

**Goal:** Expose reflection schema/read/write over the remote capability surface using the same type registrations and errors.

**In-scope behaviors:** Remote schema list, component field read, component field write, resource field read/write, error translation.

**Dependencies:** M8.1 through M8.7 and the current remote/picking/gizmos plan.

**Implementation slices:**

- [ ] Add remote request/response DTOs that carry `type_path`, optional `entity`, `field_name`, and `ReflectedValue`.
- [ ] Remote schema must filter by `is_remote_readable` and `is_remote_writable`; it must not expose editor-only fields by default.
- [ ] Translate `ReflectError` into the existing remote status/error shape without losing type path, field, entity, or resource context.
- [ ] Add tests for schema, read, write, unknown type, unknown field, missing entity, and non-writable field.

**Testing stage:**

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-remote"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib remote --locked --message-format short
```

**Exit evidence:** Remote/BRP reflection uses the runtime registry and does not introduce a second property metadata authority.

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

- Spec coverage: M8.1 covers interface contracts; M8.2 covers registry/world ownership; M8.3 covers reflected values; M8.4 covers fixed components; M8.5 covers dynamic plugin JSON components; M8.6 covers resources; M8.7 covers docs/evidence; M8.8, M9.1, and Remote.1 cover staged editor, persistence, and remote follow-up work.
- Boundary check: interface DTOs stay in `zircon_runtime_interface`; runtime adapters stay in `zircon_runtime::scene::reflect`; editor projection stays in `zircon_editor`; remote transport consumes reflection instead of defining new schema.
- Type consistency: public API names use `read_reflected_component_field`, `write_reflected_component_field`, `read_reflected_resource_field`, and `write_reflected_resource_field` throughout the plan. Function-table adapter structs derive only `Clone`; debug/partial-equality behavior stays on metadata DTOs and registry views rather than function pointers.

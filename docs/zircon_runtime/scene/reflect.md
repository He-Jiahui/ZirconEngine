---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/reflect/conversion.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/fixed/active_in_hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/active_self.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_runtime/src/scene/reflect/fixed/local_transform.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/scene/reflect/fixed/mobility.rs
  - zircon_runtime/src/scene/reflect/fixed/mod.rs
  - zircon_runtime/src/scene/reflect/fixed/name.rs
  - zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs
  - zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs
  - zircon_runtime/src/scene/reflect/fixed/shared.rs
  - zircon_runtime/src/scene/reflect/mod.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/reflect_resource.rs
  - zircon_runtime/src/scene/reflect/registration.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_property_descriptor.rs
  - zircon_runtime/src/plugin/component_type_descriptor/component_type_descriptor.rs
implementation_files:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/reflect/conversion.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/fixed/active_in_hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/active_self.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_runtime/src/scene/reflect/fixed/local_transform.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/scene/reflect/fixed/mobility.rs
  - zircon_runtime/src/scene/reflect/fixed/mod.rs
  - zircon_runtime/src/scene/reflect/fixed/name.rs
  - zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs
  - zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs
  - zircon_runtime/src/scene/reflect/fixed/shared.rs
  - zircon_runtime/src/scene/reflect/mod.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/reflect_resource.rs
  - zircon_runtime/src/scene/reflect/registration.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/world.rs
plan_sources:
  - user: 2026-05-09 M8.2 Runtime TypeRegistry And World Ownership
  - user: 2026-05-09 M8.3 Facade DTO Flow And Value Conversion
  - user: 2026-05-09 M8.4 Fixed Component Reflection Adapters
  - user: 2026-05-09 M8.5 Dynamic Plugin JSON Component Reflection
  - user: 2026-05-09 M8.6 Resource Reflection
  - user: 2026-05-16 M8.7 Editor Inspector And Remote DTO Reuse Proof
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md
  - docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md
tests:
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/resources.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs
  - tests/acceptance/reflection-type-registry.md
doc_type: module-detail
---

# Scene Reflection Runtime Registry

## M8.2 Scope

M8.2 introduces the runtime-owned reflection registry boundary under `zircon_runtime::scene::reflect` and makes `World` own the registry lifecycle. It does not register concrete fixed component schemas, dynamic plugin component adapters, resource adapters, schema facade methods, or read/write behavior. Those remain later M8 milestones.

The structural module root wires these child modules:

- `reflect_component.rs` contains the placeholder `ReflectComponent` function-table type for future fixed and dynamic component adapters.
- `reflect_resource.rs` contains the placeholder `ReflectResource` function-table type for future resource adapters.
- `registration.rs` owns the crate-visible builtin reflection bootstrap lifecycle.
- `type_registry.rs` owns deterministic runtime registry storage and lookup rules.
- `world_reflection.rs` owns the `WorldReflection` marker and the public immutable `World::type_registry()` accessor.

Later M8 slices keep the same ownership split: `ReflectComponent` and `ReflectResource` become concrete adapter contracts, `dynamic_component.rs` projects plugin JSON descriptors into the registry, and `world_reflection.rs` remains the facade for schema/read/write DTO routing.

## Runtime Boundary

`zircon_runtime_interface::reflect` owns neutral DTOs such as `ReflectTypeRegistration`, `ReflectTypePath`, and `ReflectError`. `zircon_runtime::scene::reflect` owns runtime behavior around those DTOs. The runtime registry stores DTO metadata plus optional adapter slots, but the DTOs remain independent of ECS storage and runtime function tables.

`RuntimeTypeRegistration` has three fields:

- `registration`: the neutral `ReflectTypeRegistration` metadata used by schema, inspector, remote, and serialization contracts.
- `component`: an optional `ReflectComponent` adapter slot reserved for M8.4 and M8.5.
- `resource`: an optional `ReflectResource` adapter slot reserved for M8.6.

M8.2 only uses metadata-only registrations through `RuntimeTypeRegistration::metadata`. Manual `Debug` and `PartialEq` implementations compare neutral metadata plus whether component/resource adapter slots are present. They deliberately do not compare adapter function-table identity, because future adapter function tables are behavior slots rather than semantic metadata.

## Lookup Behavior

`TypeRegistry` stores registrations in `BTreeMap<String, RuntimeTypeRegistration>` keyed by full type path. Iteration is deterministic because the map is ordered by full type path.

Lookup follows this order:

- A full type path match always wins and returns the registry-owned canonical full type path key.
- An unambiguous short type path resolves to the registry-owned canonical full type path key.
- A known ambiguous short type path returns `ReflectError::AmbiguousShortTypePath`.
- A missing path returns `ReflectError::UnknownType`.

Short-path ambiguity is tracked separately from full registrations:

- The first registration for a short path records `short -> full` in the unambiguous lookup map.
- The second different full type path with the same short path removes the unambiguous lookup and records the short path as ambiguous.
- Later registrations with the same short path keep the short path ambiguous.
- `TypeRegistry::clear` removes registrations, unambiguous short-path lookups, and ambiguity state.

Duplicate full type paths are rejected with `ReflectError::DuplicateTypePath` before mutating registry state. `contains_type_path` is the strict full-path membership check intended for duplicate preflight. `contains` is resolve-capable and returns true for either a registered full path or an unambiguous short path; ambiguous short paths return false because `resolve` reports an error for them.

## World Lifecycle

`World` owns a `type_registry: TypeRegistry` field marked `#[serde(skip, default)]`. This keeps the registry runtime-only and prevents reflected metadata or future function-table adapters from entering scene JSON.

`World::empty()` constructs the world, then calls the crate-visible bootstrap function `register_builtin_reflection(&mut world)` before returning. The function is intentionally not publicly re-exported from `scene::reflect` or `scene`, because M8.2 behavior clears and rebuilds the builtin runtime registry and must not become an external clearing API. `World::new()` continues to call `World::empty()` before spawning the default camera/light/cube scene, so the default scene behavior is preserved while reflection lifecycle has one bootstrap path.

Manual `Deserialize for World` initializes an empty registry, then calls the same crate-visible bootstrap function before rebuilding existing derived ECS state. After M8.4, deserialized worlds rebuild the runtime-only registry to the fixed builtin component registrations without changing the scene serialization format.

Public registry access is immutable through `World::type_registry()`. Tests use a crate-internal `cfg(test)` helper to seed runtime-only metadata and verify that serialization skips it; mutable registry access is not part of the public runtime API.

## Validation

M8.2 focused validation commands use the shared reflection target directory:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
git diff --check -- "zircon_runtime/src/scene/reflect" "zircon_runtime/src/scene/mod.rs" "zircon_runtime/src/scene/world/world.rs" "zircon_runtime/src/scene/world/bootstrap.rs" "zircon_runtime/src/scene/tests/mod.rs" "zircon_runtime/src/scene/tests/ecs_reflect.rs" "docs/zircon_runtime/scene/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md"
```

Acceptance evidence is recorded in `tests/acceptance/reflection-type-registry.md`.

## M8.3 Scope

M8.3 adds value conversion helpers, turns the component/resource adapter placeholders into function-table contracts, and implements the `WorldReflection` DTO facade. It intentionally does not add concrete fixed component adapters, dynamic plugin component adapters, production resource adapters, editor integration, or remote transport.

The reflection module root now also wires `conversion.rs`, which owns conversion between runtime scene property values and reflection DTO values. `reflected_from_json` preserves arbitrary input as `ReflectedValue::Json` so this milestone does not invent lossy type inference from JSON.

## Value Conversion

`reflected_from_scene_value` maps supported `ScenePropertyValue` variants directly to matching `ReflectedValue` variants:

- `Bool`, `Integer`, `Unsigned`, `Scalar`, `String`, `Enum`, `Vec2`, `Vec3`, `Vec4`, `Quaternion`, `Entity`, and `Resource` keep their runtime values.
- `AnimationParameter(_)` returns `ReflectError::UnsupportedConversion` with source `ScenePropertyValue::AnimationParameter` and target `ReflectedValue`.

`scene_value_from_reflected` maps the supported scalar, vector, enum, entity, and resource variants back into `ScenePropertyValue`. `Null`, `List`, `Map`, and `Json` are intentionally rejected because they do not have a stable scene property representation in M8.3.

Non-finite float values are rejected before converting `ReflectedValue::Scalar`, `Vec2`, `Vec3`, `Vec4`, or `Quaternion` to `ScenePropertyValue`. The same finite check runs before serializing any reflected value to JSON, including nested values inside `List` and `Map`.

`json_from_reflected` currently preserves the tagged reflection DTO shape by using serde after finite validation. For example, `ReflectedValue::Vec3([1.0, 2.0, 3.0])` serializes as `{"kind":"Vec3","value":[1.0,2.0,3.0]}`, and `ReflectedValue::Entity(Some(7))` serializes as `{"kind":"Entity","value":7}`. This deterministic shape is documented in `scene::tests::ecs_reflect` so later editor or remote callers can rely on the DTO contract instead of ad hoc JSON inference.

## Adapter Function Tables

`ReflectComponent` is now a cloneable function-table contract with adapter-owned `type_path` context and slots for:

- `contains(&World, EntityId, &str) -> bool`
- `read_field(&World, EntityId, &str, &str) -> Result<ReflectedValue, ReflectError>`
- `read_fields(&World, EntityId, &str) -> Result<Vec<ReflectFieldValue>, ReflectError>`
- `write_field(&mut World, EntityId, &str, &str, ReflectedValue) -> Result<bool, ReflectError>`
- `remove(&mut World, EntityId, &str) -> Result<bool, ReflectError>`

The forwarding methods keep the public adapter call shape unchanged for callers while passing the stored full type path into the function table. Fixed adapters ignore that argument and use their compile-time constants; dynamic adapters use it to resolve descriptor-backed JSON metadata and property paths. `ReflectResource` is the matching resource function table with world-level `contains`, `read_field`, `read_fields`, and `write_field` callbacks.

## WorldReflection Facade

`WorldReflection` now exposes schema and value DTO routing:

- `list_reflect_types` iterates `TypeRegistry` deterministically and applies `ReflectSchemaFilter`.
- `reflect_schema` resolves a full or unambiguous short type path and clones the neutral `ReflectTypeRegistration`.
- `reflect_fields` routes `ReflectObjectAddress::Component` to a component adapter and `ReflectObjectAddress::Resource` to a resource adapter.
- `reflect_read` routes through the matching adapter `read_field` and wraps the normalized value in `ReflectReadResponse`.
- `reflect_write` routes through the matching adapter `write_field`, then reads the same field back so callers receive the normalized current value in `ReflectWriteResponse`.

`World` exposes public wrappers with the same facade names: `list_reflect_types`, `reflect_schema`, `reflect_fields`, `reflect_read`, and `reflect_write`. These wrappers keep callers anchored on `World` as the runtime scene authority while centralizing routing rules in `WorldReflection`.

Schema filter behavior is exact:

- If `type_path` is present, the registry resolves that full or unambiguous short path first, then visibility, category, and plugin-owned filters still apply.
- If either `include_components` or `include_resources` is true, only registrations matching at least one requested category are included.
- If both category flags are false, category filtering is disabled.
- `editor_visible = true` requires `registration.editor_visible`.
- `remote_visible = true` requires `registration.remote_visible`.
- `include_plugin_owned = false` excludes plugin-owned registrations.

Missing or incompatible routing remains structured:

- Missing type paths propagate `ReflectError::UnknownType` from the registry.
- Component addresses without a component adapter return `ReflectError::NoComponentAdapter`.
- Resource addresses without a resource adapter return `ReflectError::NoResourceAdapter`.
- Address category mismatches return `ReflectError::AddressKindMismatch` with the resolved type path in the expected and actual strings.

## M8.3 Validation

M8.3 `cargo check -p zircon_runtime --lib --locked --message-format short` passes with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`. The focused `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` command also passes after `SystemState<P>` received a manual diagnostic `Debug` implementation that avoids requiring marker param types such as `ResParam<T>` and `ResMutParam<T>` to implement `Debug`.

That support-layer fix is intentionally narrow: it only affects diagnostic formatting for failed `Result::unwrap_err()` paths and does not change resource parameter access, borrow checking, or reflection routing behavior. The passing focused reflection run reported `10 passed; 0 failed; 0 ignored; 1119 filtered out` with unrelated dead-code warnings in graphics/plugin test helpers.

## M8.4 Fixed Component Adapters

M8.4 adds the first production component adapters under `zircon_runtime::scene::reflect::fixed`. The `fixed/mod.rs` file only wires child modules. Each concrete adapter owns one component schema and function table, while `fixed/shared.rs` owns repeated entity/component lookup and structured error construction.

`register_builtin_reflection(world)` still clears the builtin runtime registry first, then registers these fixed component type paths with component adapters:

- `zircon_runtime::scene::components::Name`
- `zircon_runtime::scene::components::LocalTransform`
- `zircon_runtime::scene::components::ActiveSelf`
- `zircon_runtime::scene::components::RenderLayerMask`
- `zircon_runtime::scene::components::RigidBodyComponent`

The adapters are intentionally manual. They call `World::insert`, `World::get`, `World::get_mut`, and `World::remove` instead of mutating fixed maps directly. Changed writes use the same typed ECS mutation paths as non-reflection writes, while no-op writes return `changed = false` before acquiring mutable component access.

## Fixed Field Schema

`Name` registers one editable field:

- `value: String`, written through `World::insert(entity, Name(value))`.

`ActiveSelf` registers one editable field:

- `value: Bool`, written through `World::insert(entity, ActiveSelf(value))` when the value changes. Changed writes mark active hierarchy and render-extract derived state dirty through the existing typed component mutation path.

`RenderLayerMask` registers one editable field:

- `mask: Unsigned`, written through `World::insert(entity, RenderLayerMask(mask as u32))`. Values above `u32::MAX` return `ReflectError::TypeMismatch` and do not mutate the world.

`LocalTransform` registers three fields:

- `translation: Vec3`, editable through `World::get_mut::<LocalTransform>(entity)`.
- `rotation: Vec4`, readable but non-editable in M8.4.
- `scale: Vec3`, editable through `World::get_mut::<LocalTransform>(entity)`.

The transform adapter compares the current subfield first. Changed writes then use `get_mut` for editable subfield mutation so the existing transform dirty-state and change-tick path is preserved without replacing the whole component; no-op writes return unchanged without dirtying scene state.

`RigidBodyComponent` registers the selected safe field set:

- Editable: `mass`, `linear_damping`, `angular_damping`, and `gravity_scale` as `Scalar`, plus `can_sleep` as `Bool`.
- Read-only: `body_type` as `Enum`, `linear_velocity` and `angular_velocity` as `Vec3`, and `lock_translation` plus `lock_rotation` as `List<Bool>` values preserving the three axis-lock booleans without inventing unsafe writes.

Read-only rigid-body fields are exposed for inspectors and remote diagnostics, but writes return `ReflectError::NonEditableField` until a later physics-authoring slice defines safe mutation semantics.

## M10 Fixed Inspector Coverage

M10 extends `reflect/fixed` so `World::editor_projection` can be the editor viewport inspector source instead of a separate hand-written `SceneNode` field list. The added fixed adapters cover:

- `Hierarchy.parent` as an editable `Entity` field, but with `serializable = false` because `DynamicScene` already stores parent links in `NodeRecord`.
- `ActiveInHierarchy.value` as a read-only, non-serializable derived `Bool` field.
- `CameraComponent.fov_y_radians`, `z_near`, and `z_far` as editable scalar fields.
- `MeshRenderer.model` and `material` as read-only resource fields, plus editable `tint: Vec4`.
- `Mobility.kind` as an editable enum field using the existing runtime mobility validation path.
- `DirectionalLight`, `PointLight`, and `SpotLight` authoring fields for color, direction, intensity, range, and spot cone angles.

The editor crate maps these reflected type paths back to legacy command-compatible property paths such as `Transform.translation`, `Active.enabled`, `MeshRenderer.model`, and plugin paths like `weather.Component.CloudLayer.coverage`. This keeps the runtime reflection schema as the single field source while preserving editor command/history compatibility.

M10 focused validation:

```powershell
cargo test -p zircon_runtime --lib scene::tests::editor_projection --locked --jobs 1 --message-format short
cargo test -p zircon_editor --lib viewport_edit_mode_projection_consumes_runtime_reflection_inspector_fields --locked --jobs 1 --message-format short
```

The runtime command passed 2 tests with 0 failures and 1458 filtered out. The editor command passed 1 test with 0 failures and 1341 filtered out. A crate-level `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short` currently fails outside this reflection slice in `zircon_runtime/src/scene/world/render.rs`, where the active rendering parity work has a `PostProcessExtract` initializer missing `graph` and `stack` fields.

## Fixed Error Model

Every fixed adapter returns structured errors consistently:

- Missing entity: `ReflectError::MissingEntity { entity }`.
- Existing entity without that fixed component: `ReflectError::MissingComponent { entity, type_path }`.
- Absent field name: `ReflectError::UnknownField { type_path, field_name }`.
- Write to a read-only fixed field: `ReflectError::NonEditableField { type_path, field_name }`.
- Wrong reflected value kind, non-finite scalar/vector input, or out-of-range render layer mask: `ReflectError::TypeMismatch { type_path, field_name, expected, actual }`.

`WorldReflection::reflect_write` still reads the field back after the adapter write succeeds, so callers receive the normalized `ReflectWriteResponse.field` value after fixed component mutation.

## M8.4 Validation

M8.4 implementation adds focused coverage in `zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs` for builtin registration, fixed field reads and writes, dirty-state preservation for active and transform writes, read-only rotation behavior, render-layer `u32` bounds, rigid-body selected fields, unknown fields, missing components, and missing entities.

The closeout validation also fixed the lower shared default-name path used by those reflection tests: `World::spawn_node` now computes its per-kind ordinal before inserting the new entity/kind, and `zircon_runtime/src/scene/tests/world_basics.rs` covers first mesh `Mesh 1`, second mesh `Mesh 2`, and first cube `Cube 1`.

Local M8.4 formatting and scoped Cargo validation passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::world_basics::spawn_node_assigns_one_based_kind_ordinals --locked --message-format short
```

The focused `ecs_reflect` filter currently reports 25 passing tests because M8.5 dynamic-component reflection tests already live under the same module. The fixed-adapter slice remains scoped to the `foundation.rs` cases and the typed ECS validation command above. Workspace-wide validation was not run or claimed for M8.4.

## M8.5 Dynamic Plugin JSON Component Reflection

M8.5 adds `reflect/dynamic_component.rs`, which projects plugin-facing `ComponentTypeDescriptor` values into runtime reflection registrations and supplies one dynamic `ReflectComponent` adapter implementation. `ComponentTypeDescriptor` remains the descriptor cache/input path for plugins; `TypeRegistry` becomes the schema/read/write reflection path used by `WorldReflection`.

Descriptor projection is deterministic:

- The full reflected type path is `descriptor.type_id`; the short type path is the last dot segment such as `CloudLayer` for `weather.Component.CloudLayer`.
- The reflected type kind is `ReflectTypeKind::Json` with JSON serialization.
- The registration is a component, plugin-owned, serializable, editor-visible, remote-visible, and carries `plugin_id = Some(descriptor.plugin_id.clone())` both at the registration and nested type-path levels.
- Each `ComponentPropertyDescriptor` becomes one `ReflectFieldInfo` in descriptor order. `name` and `display_name` use the property name, `value_type_path` uses `value_type`, and `editable` uses the descriptor editability flag.
- Empty dynamic field names or value-type paths return `ReflectError::InvalidRegistration` before either registry is mutated.

`World::register_component_type` now creates the reflected registration and checks the reflection registry before mutating the plugin descriptor registry. The mutation sequence is: project `ReflectTypeRegistration`, reject a duplicate reflected path through `TypeRegistry::contains`, register the cloned `ComponentTypeDescriptor`, then insert a `RuntimeTypeRegistration` carrying the dynamic component adapter. Invalid reflection metadata or duplicate reflection paths leave both registries unchanged; descriptor registry errors still occur before reflection insertion, so reflection state is not partially updated.

Dynamic component adapters route through existing world JSON helpers rather than replacing them:

- `contains` is true only when the entity exists and `World::dynamic_component(entity, type_path)` is present.
- `read_field` validates the reflected field metadata, builds `ComponentPropertyPath::parse(&format!("{type_path}.{field_name}"))`, calls `World::dynamic_component_property`, and converts the resulting `ScenePropertyValue` into `ReflectedValue`.
- `read_fields` iterates reflected field metadata in schema order and reads only declared fields.
- `write_field` validates reflected editability, requires the dynamic component instance to exist, converts `ReflectedValue` into `ScenePropertyValue`, and calls `World::set_dynamic_component_property`.
- Missing entities, missing dynamic component instances, undeclared fields, read-only fields, and unsupported value conversions return structured `ReflectError` variants.

M8.5 tests live in `zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs`. The previous reflection tests were moved under `zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs` so the parent `mod.rs` stays structural and new dynamic coverage does not extend the old 1000+ line flat test file.

## M8.5 Validation

M8.5 scoped validation passed with the same target directory:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib plugin_extensions::dynamic_components --locked --message-format short
```

The dynamic plugin regression command passed 6 tests in `zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs`. Workspace-wide validation was not run or claimed for M8.5.

## M8.6 Resource Reflection

M8.6 proves the reflection facade is not component-only. It keeps resources owned by `World` typed resource storage and adds the registry convenience needed to attach resource adapters to neutral `ReflectTypeRegistration` metadata.

`TypeRegistry::register_resource(registration, adapter)` validates that the incoming registration is resource-only, then stores a `RuntimeTypeRegistration` with `resource: Some(adapter)` and `component: None`. Invalid component or metadata-only registrations return `ReflectError::InvalidRegistration` instead of creating a mixed adapter slot.

`WorldReflection` routes `ReflectObjectAddress::Resource { type_path }` through `ReflectResource` symmetrically with component routing. Resource-address routing first rejects resolved component registrations with `ReflectError::AddressKindMismatch`, then returns `ReflectError::NoResourceAdapter` only for resource registrations that have no resource adapter. Component-address routing also rejects resource registrations with `ReflectError::AddressKindMismatch` before component adapter lookup.

## Resource Adapter Contract

`ReflectResource` remains a function-table contract with slots for:

- `contains(&World) -> bool`
- `read_field(&World, &str) -> Result<ReflectedValue, ReflectError>`
- `read_fields(&World) -> Result<Vec<ReflectFieldValue>, ReflectError>`
- `write_field(&mut World, &str, ReflectedValue) -> Result<bool, ReflectError>`

The M8.6 test adapter is intentionally test-only. `FrameCounter { value: u32 }` lives in `zircon_runtime/src/scene/tests/ecs_reflect/resources.rs`, registers type path `zircon_runtime::scene::tests::ecs_reflect::FrameCounter`, and exposes one editable `value: Unsigned` field with `ReflectSerializationStrategy::ResourceHandle`.

The adapter reads through `World::get_resource::<FrameCounter>()` and writes through `World::get_resource_mut::<FrameCounter>()`. That write path calls `resource_mut_with_ticks`, which marks the resource changed at the current mutation tick in `ResourceStore`. No-op writes compare the existing value first and return `changed = false` without acquiring mutable resource access.

## Resource Error Model

The resource tests cover structured resource failures and shared facade shape:

- Missing `FrameCounter` storage returns `ReflectError::MissingResource { type_path }` for both read and write.
- Unknown resource fields return `ReflectError::UnknownField { type_path, field_name }` from the adapter.
- Resource registrations without a resource adapter return `ReflectError::NoResourceAdapter`.
- Component registrations addressed as resources return `ReflectError::AddressKindMismatch` because the resource route rejects non-resource registrations before adapter lookup.
- Resource registrations addressed as components return `ReflectError::AddressKindMismatch` because the component route rejects non-component registrations before adapter lookup.

`WorldReflection::reflect_write` still reads the resource field back after a successful write, so callers receive a normalized `ReflectWriteResponse.field` matching the current world state.

## M8.6 Validation

M8.6 scoped validation passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short
```

The focused `ecs_reflect` filter now reports 32 passing tests, including six resource reflection tests in `resources.rs`. The typed ECS filter reports 4 passing tests. The Cargo test commands emitted two unrelated dead-code warnings in graphics/plugin test helpers. Workspace-wide validation was not run or claimed for M8.6.

## M8.7 Inspector And Remote DTO Reuse Proof

M8.7 adds runtime-side acceptance coverage for the editor-inspector and remote/devtools seams without changing editor production code or adding transport endpoints. The tests live in `zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs` so the existing `ecs_reflect` test root remains structural.

The inspector-style proof creates one entity with fixed `Name` and `ActiveSelf` components plus a dynamic plugin JSON component. It calls `world.reflect_fields(ReflectFieldsRequest::new(...))` for each reflected component type and asserts ordered `ReflectFieldValue` output. The test deliberately uses the `WorldReflection` facade rather than `component_type_descriptors()` or fixed component maps, matching the intended future source for editor `SceneInspectorField` projection.

The remote-style proof keeps transport out of scope and validates the DTO boundary directly:

- Serialized/deserialized `ReflectSchemaRequest` values preserve the default plugin-owned exclusion, and produce serializable plugin-owned schema data only when the request explicitly sets `include_plugin_owned = true`.
- A serialized/deserialized `ReflectReadRequest` passes through `world.reflect_read(...)` and returns a serializable `ReflectReadResponse` for a component field.
- A serialized/deserialized `ReflectWriteRequest` is passed back to `world.reflect_write(...)` and mutates the world through the same reflection facade.
- The resulting `ReflectSchemaResponse`, `ReflectReadResponse`, and `ReflectWriteResponse` values are also serialized/deserialized to prove responses remain transport-shaped DTOs.
- Serialized schema/read/write request and response DTOs are checked for absence of runtime-only tokens such as `World`, `TypeRegistry`, `ReflectComponent`, and `ReflectResource`.

This milestone only documents the editor replacement point in `docs/zircon_editor/scene/viewport/edit_mode_projection.md`; it does not rewrite the current projection builder, editor history, selection, undo/redo, sockets, HTTP routes, BRP handlers, or remote authorization.

## M8.7 Validation

M8.7 scoped validation passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`:

```powershell
$env:CARGO_TARGET_DIR = "E:\cargo-targets\zircon-reflect-m8"
rustfmt --edition 2021 --check "zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs" "zircon_runtime/src/scene/tests/ecs_reflect/mod.rs"
cargo check -p zircon_runtime --lib --locked --message-format short
cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
```

The focused reflection filter now reports 35 passing tests, including the three M8.7 inspector/remote DTO reuse tests in `editor_remote.rs`. After focused review found missing schema/read request roundtrip coverage, the schema/read test now serializes/deserializes `ReflectSchemaRequest` and `ReflectReadRequest` before executing the facade path, and the focused reflection filter passed again with 35 tests. The scoped whitespace check over the M8.7 test/doc/session files passed with LF-to-CRLF warnings only. Workspace-wide validation was not run or claimed for M8.7.

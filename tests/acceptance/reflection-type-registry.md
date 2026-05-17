# Reflection Type Registry Acceptance Evidence

## Scope

This file records scoped acceptance evidence for the Reflection/TypeRegistry milestone series. The M8.1 entry covers only `zircon_runtime_interface::reflect` DTO contracts, tests, and documentation. It does not claim workspace-wide validation.

## M8.1 Interface Reflection Contracts

Implementation files:

- `zircon_runtime_interface/src/lib.rs`
- `zircon_runtime_interface/src/reflect/mod.rs`
- `zircon_runtime_interface/src/reflect/type_path.rs`
- `zircon_runtime_interface/src/reflect/object_address.rs`
- `zircon_runtime_interface/src/reflect/type_kind.rs`
- `zircon_runtime_interface/src/reflect/editor_hint.rs`
- `zircon_runtime_interface/src/reflect/field_info.rs`
- `zircon_runtime_interface/src/reflect/type_info.rs`
- `zircon_runtime_interface/src/reflect/type_registration.rs`
- `zircon_runtime_interface/src/reflect/schema.rs`
- `zircon_runtime_interface/src/reflect/read_write.rs`
- `zircon_runtime_interface/src/reflect/reflected_value.rs`
- `zircon_runtime_interface/src/reflect/error.rs`

Tests and docs:

- `zircon_runtime_interface/src/tests/reflect_contracts.rs`
- `docs/zircon_runtime_interface/reflect.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Compliance fix scope:

- Replaced the stale schema query and value read/write contracts with `ReflectSchemaFilter`, `ReflectSchemaRequest`, `ReflectSchemaResponse`, `ReflectFieldsRequest`, `ReflectFieldsResponse`, `ReflectReadRequest`, `ReflectReadResponse`, `ReflectWriteRequest`, and `ReflectWriteResponse`.
- Replaced the stale schema-query and value-request module files with `schema.rs` and `read_write.rs`.
- Restored `ReflectObjectAddress` to exactly `Component { entity, type_path }` and `Resource { type_path }`.
- Kept reconciled required `ReflectedValue::Enum` and `ReflectedValue::Quaternion` variants and updated tests for them.
- Quality-fix update: `ReflectTypeRegistration::with_plugin_id` now synchronizes registry-level and nested type-path plugin owners.
- Quality-fix update: `ReflectSchemaFilter::for_type` and `ReflectSchemaRequest::for_type` include component and resource registrations by default without forcing editor, remote, or plugin-owned flags.
- Quality-fix update: contract tests now assert exact serde JSON shapes for tagged values, object addresses, field/read/write DTOs, and structured errors.

Commands:

- Passed: `cargo check -p zircon_runtime_interface --locked --message-format short`
- Passed: `cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short`
- Passed: `git diff --check -- "zircon_runtime_interface/src/lib.rs" "zircon_runtime_interface/src/reflect" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/reflect_contracts.rs" "docs/zircon_runtime_interface/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md" "docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md" "docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md"`

Observed output summaries:

- Controller rerun: `cargo check -p zircon_runtime_interface --locked --message-format short` finished successfully in 0.48s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- Controller rerun: `cargo test -p zircon_runtime_interface reflect_contracts --locked --message-format short` finished successfully after a 27.32s compile with 7 passed, 0 failed, 0 ignored, 0 measured, and 83 filtered out.
- Controller rerun: `git diff --check` completed with no whitespace errors. It emitted LF-to-CRLF warnings for `docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md`, `docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md`, `zircon_runtime_interface/src/lib.rs`, and `zircon_runtime_interface/src/tests/mod.rs`.

Result:

- Interface DTO compile, focused contract-test validation, spec compliance review, code-quality review, and scoped whitespace validation passed for M8.1. No workspace-wide validation was run or claimed for M8.1.

## M8.2 Runtime TypeRegistry And World Ownership

Implementation files:

- `zircon_runtime/src/scene/mod.rs`
- `zircon_runtime/src/scene/reflect/mod.rs`
- `zircon_runtime/src/scene/reflect/reflect_component.rs`
- `zircon_runtime/src/scene/reflect/reflect_resource.rs`
- `zircon_runtime/src/scene/reflect/registration.rs`
- `zircon_runtime/src/scene/reflect/type_registry.rs`
- `zircon_runtime/src/scene/reflect/world_reflection.rs`
- `zircon_runtime/src/scene/world/bootstrap.rs`
- `zircon_runtime/src/scene/world/world.rs`

Tests and docs:

- `zircon_runtime/src/scene/tests/mod.rs`
- `zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs`
- `docs/zircon_runtime/scene/reflect.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Scope:

- Added runtime-owned `zircon_runtime::scene::reflect` structural module boundary.
- Added placeholder `ReflectComponent` and `ReflectResource` adapter slot types without behavior methods.
- Added deterministic `TypeRegistry` storage keyed by full type path with duplicate rejection, full-path lookup, short-path lookup, ambiguous short-path tracking, deterministic iteration, clear, and empty checks.
- Added `RuntimeTypeRegistration` metadata plus future component/resource adapter slots.
- Added runtime-only skipped `World::type_registry` ownership with construction and deserialization rebuild path through `register_builtin_reflection`.
- Quality-fix update: `register_builtin_reflection` is crate-visible within `scene` and is not publicly re-exported as an external clearing API.
- Quality-fix update: `TypeRegistry::resolve` returns registry-owned canonical full type path keys for both full-path and short-path matches.
- Quality-fix update: `contains_type_path` is the strict full-path check for duplicate preflight, while `contains` is resolve-capable for full paths and unambiguous short paths.
- Quality-fix update: `RuntimeTypeRegistration` `Debug` and `PartialEq` compare adapter slot presence without comparing adapter function-table identity.
- Kept M8.3 facade/conversion, M8.4 fixed adapters, M8.5 dynamic adapters, M8.6 resources, editor, and remote transport out of scope.

Commands:

- Passed with unrelated warnings: `cargo check -p zircon_runtime --lib --locked --message-format short`
- Blocked by unrelated active worktree test compile errors: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed with line-ending warnings only after quality fixes: `git diff --check -- "zircon_runtime/src/scene/reflect" "zircon_runtime/src/scene/mod.rs" "zircon_runtime/src/scene/world/world.rs" "zircon_runtime/src/scene/world/bootstrap.rs" "zircon_runtime/src/scene/tests/mod.rs" "zircon_runtime/src/scene/tests/ecs_reflect.rs" "docs/zircon_runtime/scene/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md"`

Observed output summaries:

- Initial `cargo check` exposed an in-scope `TypeRegistry::resolve` lifetime error, which was fixed by tying the returned `&str` lifetime to both `&self` and the input full-path borrow.
- Rerun `cargo check` reached `zircon_runtime` and failed only on unrelated UI/accessibility active-worktree errors: `zircon_runtime/src/ui/accessibility/diagnostics.rs:54` borrow conflict and `zircon_runtime/src/ui/surface/input/dispatch.rs:820` missing `&UiInputEvent::Accessibility(_)` match arm.
- `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` could not run the focused tests because the lib test target failed to compile unrelated tests and UI code first. Reported blockers included `zircon_runtime/src/scene/tests/ecs_systems.rs:128` missing `Debug` for `ResParam<MissingScore>`, multiple `UiTemplateNodeMetadata` initializers missing `a11y` and `widget`, the same UI accessibility borrow conflict, and the same missing accessibility input match arm.
- Scoped `git diff --check` completed with no whitespace errors. It emitted LF-to-CRLF warnings for `zircon_runtime/src/scene/mod.rs`, `zircon_runtime/src/scene/tests/mod.rs`, `zircon_runtime/src/scene/world/bootstrap.rs`, and `zircon_runtime/src/scene/world/world.rs`.
- Controller rerun after M8.2 quality re-review: `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully in 16.25s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- Controller rerun after M8.2 quality re-review: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` did not reach the focused tests because the lib test target failed on unrelated `zircon_runtime/src/scene/tests/ecs_systems.rs:128` missing `Debug` for `ResParam<MissingScore>` and `zircon_runtime/src/scene/tests/ecs_systems.rs:129` missing `Debug` for `ResMutParam<MissingScore>`.
- Controller rerun after M8.2 quality re-review: scoped `git diff --check` completed with no whitespace errors and LF-to-CRLF warnings for `zircon_runtime/src/scene/mod.rs`, `zircon_runtime/src/scene/tests/mod.rs`, `zircon_runtime/src/scene/world/bootstrap.rs`, and `zircon_runtime/src/scene/world/world.rs`.

Result:

- M8.2 implementation, spec review, quality review, docs, and scoped whitespace validation are present. Fresh `cargo check` passes for `zircon_runtime --lib`; focused `ecs_reflect` test execution remains blocked by unrelated lib-test compile errors owned by the active ECS SystemParam session, so M8.2 cannot honestly be reported as fully test-passing in this checkout.

## M8.3 Facade DTO Flow And Value Conversion

Implementation files:

- `zircon_runtime/src/scene/mod.rs`
- `zircon_runtime/src/scene/reflect/mod.rs`
- `zircon_runtime/src/scene/reflect/conversion.rs`
- `zircon_runtime/src/scene/reflect/reflect_component.rs`
- `zircon_runtime/src/scene/reflect/reflect_resource.rs`
- `zircon_runtime/src/scene/reflect/world_reflection.rs`
- `zircon_runtime/src/scene/ecs/system/system_state.rs`

Tests and docs:

- `zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs`
- `docs/zircon_runtime/scene/reflect.md`
- `tests/acceptance/reflection-type-registry.md`
- `.codex/sessions/20260508-2036-reflection-type-registry.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Scope:

- Added conversion helpers for `ScenePropertyValue` to `ReflectedValue`, `ReflectedValue` to `ScenePropertyValue`, arbitrary JSON preservation as `ReflectedValue::Json`, and deterministic tagged DTO JSON serialization from reflected values.
- Rejected `ScenePropertyValue::AnimationParameter` with a structured `UnsupportedConversion` error.
- Rejected `ReflectedValue::Null`, `List`, `Map`, and `Json` when converting to `ScenePropertyValue`.
- Rejected non-finite reflected scalars, vectors, and quaternions when converting to scene values or JSON, including nested list/map JSON conversion.
- Replaced placeholder `ReflectComponent` and `ReflectResource` unit structs with function-table contracts and forwarding methods. M8.5 later made `ReflectComponent` cloneable with adapter-owned type-path context for dynamic descriptors.
- Implemented `WorldReflection` schema listing, schema resolution, fields/read/write routing, missing-adapter errors, and address-kind mismatch errors through registry adapter slots.
- Added public `World` wrappers for `list_reflect_types`, `reflect_schema`, `reflect_fields`, `reflect_read`, and `reflect_write`.
- Unblocked focused reflection test execution by replacing `SystemState<P>`'s derived `Debug` with a manual diagnostic `Debug` implementation that does not require marker param types such as `ResParam<T>` or `ResMutParam<T>` to implement `Debug`.
- Kept M8.4 fixed component adapters, M8.5 dynamic plugin adapters, M8.6 production resource adapters, editor integration, and remote transport out of scope.

Commands:

- Passed: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/ecs/system/system_state.rs"`
- Passed: `cargo check -p zircon_runtime --lib --locked --message-format short`
- Passed after rerun with longer compile timeout: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed with line-ending warnings only: `git diff --check -- "zircon_runtime/src/scene/ecs/system/system_state.rs" "zircon_runtime/src/scene/mod.rs" "zircon_runtime/src/scene/reflect" "zircon_runtime/src/scene/tests/ecs_reflect.rs" "docs/zircon_runtime/scene/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md"`

Observed output summaries:

- `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully in 23.23s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- Reproduced the previous blocker: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` failed before tests because `Result::unwrap_err()` required `SystemState<ResParam<MissingScore>>` and `SystemState<ResMutParam<MissingScore>>` to implement `Debug`.
- Root cause: the derived `Debug` implementation on `SystemState<P>` added an unnecessary `P: Debug` bound even though diagnostic output only needs cached state, access, and tick fields. Manual `Debug` now requires only `P::State: Debug` and keeps marker params out of the diagnostic bound.
- `rustfmt --edition 2021 --check "zircon_runtime/src/scene/ecs/system/system_state.rs"` passed.
- `cargo check -p zircon_runtime --lib --locked --message-format short` passed in 15.60s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- First focused test rerun timed out after 120s while compiling. The longer rerun finished in 2m 25s with 2 unrelated dead-code warnings and `10 passed; 0 failed; 0 ignored; 0 measured; 1119 filtered out`.
- Scoped `git diff --check` completed with no whitespace errors. It emitted LF-to-CRLF warnings for `zircon_runtime/src/scene/ecs/system/system_state.rs` and `zircon_runtime/src/scene/mod.rs`.

Result:

- M8.3 implementation, docs, focused compile validation, and focused reflection tests pass in this checkout. Workspace-wide validation was not run or claimed for M8.3.

## M8.4 Fixed Component Reflection Adapters

Implementation files:

- `zircon_runtime/src/scene/world/bootstrap.rs`
- `zircon_runtime/src/scene/reflect/mod.rs`
- `zircon_runtime/src/scene/reflect/registration.rs`
- `zircon_runtime/src/scene/reflect/fixed/mod.rs`
- `zircon_runtime/src/scene/reflect/fixed/shared.rs`
- `zircon_runtime/src/scene/reflect/fixed/name.rs`
- `zircon_runtime/src/scene/reflect/fixed/local_transform.rs`
- `zircon_runtime/src/scene/reflect/fixed/active_self.rs`
- `zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs`
- `zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs`

Tests and docs:

- `zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs`
- `zircon_runtime/src/scene/tests/world_basics.rs`
- `docs/zircon_runtime/scene/reflect.md`
- `tests/acceptance/reflection-type-registry.md`
- `.codex/sessions/20260508-2036-reflection-type-registry.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Scope:

- Added manual fixed component reflection registrations for `Name`, `LocalTransform`, `ActiveSelf`, `RenderLayerMask`, and `RigidBodyComponent` with exact required type paths.
- Wired `register_builtin_reflection(world)` to clear and rebuild builtin fixed registrations with component adapters.
- Added fixed `ReflectComponent` adapters for field enumeration, field reads, field writes, and component removal through existing `World` typed APIs.
- Preserved dirty-state/change-tick behavior for changed writes by routing replacement writes through `World::insert` and in-place mutable subfield writes through `World::get_mut`; no-op in-place writes now return unchanged before mutable access.
- Added structured fixed-adapter errors for missing entities, missing components, unknown fields, non-editable fields, value-kind mismatches, non-finite reflected numeric input, and out-of-range render layer masks.
- Fixed the lower shared `World::spawn_node` default-name ordinal path after reflection validation exposed a first mesh named `Mesh 2`; default names are now computed before the new entity/kind is inserted, and `world_basics` covers one-based per-kind ordinals.
- Kept dynamic plugin component reflection, production resource reflection, editor integration, and remote transport out of M8.4 scope.

Commands:

- Passed: `cargo check -p zircon_runtime --lib --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::world_basics::spawn_node_assigns_one_based_kind_ordinals --locked --message-format short`
- Passed: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/reflect/fixed/shared.rs" "zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs" "zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs" "zircon_runtime/src/scene/reflect/fixed/local_transform.rs" "zircon_runtime/src/scene/reflect/fixed/name.rs" "zircon_runtime/src/scene/reflect/fixed/active_self.rs" "zircon_runtime/src/scene/reflect/fixed/mod.rs" "zircon_runtime/src/scene/reflect/mod.rs" "zircon_runtime/src/scene/reflect/registration.rs" "zircon_runtime/src/scene/tests/ecs_reflect.rs" "zircon_runtime/src/scene/ecs/system/system_state.rs"`
- Passed quality follow-up formatting: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/reflect/fixed/shared.rs" "zircon_runtime/src/scene/reflect/fixed/local_transform.rs" "zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs" "zircon_runtime/src/scene/tests/ecs_reflect.rs"`
- Passed closeout formatting: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/world/bootstrap.rs" "zircon_runtime/src/scene/tests/world_basics.rs" "zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs"`
- Passed after review fixes: M8.4 spec compliance review and code-quality re-review by subagent inspection.

Observed output summaries:

- `rustfmt --check` for touched M8.4 Rust files completed with no output.
- Quality follow-up `rustfmt --check` initially reported formatting drift in `ecs_reflect.rs`; after scoped `rustfmt`, rerun completed with no output.
- The closeout `cargo check -p zircon_runtime --lib --locked --message-format short` passed in 24.62s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- The closeout `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` passed with 25 passed, 0 failed, 0 ignored, 1138 filtered out. This filter now includes both M8.4 fixed-adapter tests in `foundation.rs` and already-present M8.5 dynamic-component tests in `dynamic_components.rs`. It emitted two unrelated dead-code warnings in graphics/plugin test helpers.
- The closeout `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short` passed with 4 passed, 0 failed, 0 ignored, 1159 filtered out. It emitted the same unrelated dead-code warnings.
- The lower-layer ordinal regression was first observed red with `Mesh 2` versus `Mesh 1`, then passed after the shared `spawn_node` fix with 1 passed, 0 failed, 1162 filtered out.
- Closeout `git diff --check` over the reflection/world/docs/session scope completed with no whitespace errors and LF-to-CRLF warnings for `zircon_runtime/src/scene/mod.rs`, `zircon_runtime/src/scene/tests/world_basics.rs`, `zircon_runtime/src/scene/world/bootstrap.rs`, and `zircon_runtime/src/scene/world/dynamic_components.rs`.
- Spec compliance review returned `Spec compliant` for M8.4 after code inspection.
- Code-quality review initially found one test compile issue (`unknown_fixed_field_returns_structured_error` needed mutable `world`) and a minor `LocalTransform.scale` coverage gap. Both were fixed, and code-quality re-review approved with no remaining Critical, Important, or Minor issues.
- Focused code review of the shared `spawn_node` ordinal fix reported no Critical, Important, or Minor findings.

Result:

- M8.4 implementation, fixed-adapter tests, typed ECS regression tests, lower-layer ordinal regression, docs, focused formatting, focused review, and scoped Cargo validation pass in this checkout. Workspace-wide validation was not run or claimed for M8.4.

## M8.5 Dynamic Plugin JSON Component Reflection

Implementation files:

- `zircon_runtime/src/scene/reflect/mod.rs`
- `zircon_runtime/src/scene/reflect/reflect_component.rs`
- `zircon_runtime/src/scene/reflect/dynamic_component.rs`
- `zircon_runtime/src/scene/world/dynamic_components.rs`

Tests and docs:

- `zircon_runtime/src/scene/tests/ecs_reflect/mod.rs`
- `zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs`
- `zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs`
- `docs/zircon_runtime/scene/reflect.md`
- `tests/acceptance/reflection-type-registry.md`
- `.codex/sessions/20260508-2036-reflection-type-registry.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Scope:

- Added dynamic descriptor projection from `ComponentTypeDescriptor` to `ReflectTypeRegistration` with JSON type info, plugin ownership, remote/editor visibility, descriptor plugin ID propagation, and descriptor-order field metadata.
- Added a dynamic `ReflectComponent` adapter that carries type-path context through the existing function-table shape and reads/writes JSON fields via existing dynamic component property helpers.
- Changed `World::register_component_type` so reflection projection and duplicate reflected path checks happen before descriptor-registry mutation, then descriptor registration happens before reflection registry insertion.
- Kept `ComponentTypeDescriptor` as the plugin-facing descriptor cache/input while making `TypeRegistry` the reflection/schema path.
- Moved the flat `scene::tests::ecs_reflect` file into a folder-backed module and placed new M8.5 coverage in `dynamic_components.rs`.

Commands:

- Passed after formatting: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/reflect/reflect_component.rs" "zircon_runtime/src/scene/reflect/dynamic_component.rs" "zircon_runtime/src/scene/reflect/mod.rs" "zircon_runtime/src/scene/reflect/world_reflection.rs" "zircon_runtime/src/scene/reflect/fixed/name.rs" "zircon_runtime/src/scene/reflect/fixed/active_self.rs" "zircon_runtime/src/scene/reflect/fixed/local_transform.rs" "zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs" "zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs" "zircon_runtime/src/scene/world/dynamic_components.rs" "zircon_runtime/src/scene/tests/ecs_reflect/mod.rs" "zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs" "zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs"`
- Passed with line-ending warning only: `git diff --check -- "zircon_runtime/src/scene/reflect/reflect_component.rs" "zircon_runtime/src/scene/reflect/dynamic_component.rs" "zircon_runtime/src/scene/reflect/mod.rs" "zircon_runtime/src/scene/reflect/world_reflection.rs" "zircon_runtime/src/scene/reflect/fixed/name.rs" "zircon_runtime/src/scene/reflect/fixed/active_self.rs" "zircon_runtime/src/scene/reflect/fixed/local_transform.rs" "zircon_runtime/src/scene/reflect/fixed/render_layer_mask.rs" "zircon_runtime/src/scene/reflect/fixed/rigid_body_component.rs" "zircon_runtime/src/scene/world/dynamic_components.rs" "zircon_runtime/src/scene/tests/ecs_reflect" "zircon_runtime/src/scene/tests/ecs_reflect.rs" "docs/zircon_runtime/scene/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md"`
- Passed: `cargo check -p zircon_runtime --lib --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib plugin_extensions::dynamic_components --locked --message-format short`

Observed output summaries:

- Initial `rustfmt --check` reported formatting drift in touched fixed adapter/test files after the adapter context signature change. Scoped `rustfmt` was run on touched Rust files, and the follow-up `rustfmt --check` completed with no output.
- Scoped `git diff --check` reported no whitespace errors and one LF-to-CRLF warning for `zircon_runtime/src/scene/world/dynamic_components.rs`.
- The closeout `cargo check -p zircon_runtime --lib --locked --message-format short` passed in 24.62s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- The closeout `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` passed with 25 passed, 0 failed, 0 ignored, 1138 filtered out, including the six M8.5 dynamic reflection tests in `ecs_reflect/dynamic_components.rs`.
- The closeout `cargo test -p zircon_runtime --lib plugin_extensions::dynamic_components --locked --message-format short` passed with 6 passed, 0 failed, 0 ignored, 1157 filtered out.
- The Cargo test commands emitted two unrelated dead-code warnings in graphics/plugin test helpers.

Reference evidence preserved for M8.5:

- Bevy: `dev/bevy/crates/bevy_ecs/src/reflect/component.rs` uses function-table `ReflectComponent` stored as type data in a registry for runtime-known component operations.
- Bevy: `dev/bevy/crates/bevy_reflect/src/type_registry.rs` indexes full and short type paths and tracks ambiguity.
- Godot: `dev/godot/core/object/property_info.h` carries property names, usage flags, and read-only/editor/storage visibility metadata.
- Zircon intentionally diverges by keeping serializable DTOs in `zircon_runtime_interface::reflect` and manual plugin JSON descriptors in runtime.

Result:

- M8.5 implementation, dynamic reflection tests, plugin dynamic-component regression tests, docs, acceptance evidence, and scoped Cargo validation pass in this checkout. Workspace-wide validation was not run or claimed for M8.5.

## M8.6 Resource Reflection

Implementation files:

- `zircon_runtime/src/scene/reflect/type_registry.rs`
- `zircon_runtime/src/scene/reflect/world_reflection.rs`
- `zircon_runtime/src/scene/reflect/reflect_resource.rs`

Existing implementation files exercised by this milestone:

- `zircon_runtime/src/scene/world/typed_api.rs`
- `zircon_runtime/src/scene/world/change_detection.rs`
- `zircon_runtime/src/scene/ecs/resource_store.rs`

Tests and docs:

- `zircon_runtime/src/scene/tests/ecs_reflect/mod.rs`
- `zircon_runtime/src/scene/tests/ecs_reflect/resources.rs`
- `docs/zircon_runtime/scene/reflect.md`
- `tests/acceptance/reflection-type-registry.md`
- `.codex/sessions/20260508-2036-reflection-type-registry.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Scope:

- Added `TypeRegistry::register_resource` as the resource-adapter convenience helper. It stores resource registrations with `resource: Some(adapter)` and `component: None`, and rejects non-resource or mixed component/resource registrations with `ReflectError::InvalidRegistration`.
- Added M8.6 resource tests in a new `ecs_reflect/resources.rs` module instead of extending the already-large `foundation.rs` test file.
- Added a test-only `FrameCounter { value: u32 }` resource adapter registered as `zircon_runtime::scene::tests::ecs_reflect::FrameCounter` with one editable `value: Unsigned` field and `ReflectSerializationStrategy::ResourceHandle`.
- Proved resource schema listing, full/short type-path schema lookup, field enumeration, field read, field write/readback, no-op writes, missing resource errors, and shared component/resource facade shape through `ReflectObjectAddress` DTOs.
- Proved reflected resource writes update `World::resource_change_ticks::<FrameCounter>()` by writing through `World::get_resource_mut::<FrameCounter>()`.
- Added a metadata-only resource registration regression so resolved resource registrations without a resource adapter return `ReflectError::NoResourceAdapter`.
- Tightened resource-address routing so component registrations addressed as resources return `ReflectError::AddressKindMismatch`; resource registrations addressed as components already return the same category-mismatch error from the component route.
- Kept editor integration, remote transport, proc macros, and broad workspace cleanup out of M8.6 scope.

Commands:

- Passed after formatting: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/reflect/type_registry.rs" "zircon_runtime/src/scene/tests/ecs_reflect/mod.rs" "zircon_runtime/src/scene/tests/ecs_reflect/resources.rs"`
- Passed after review fix: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/reflect/world_reflection.rs" "zircon_runtime/src/scene/tests/ecs_reflect/resources.rs"`
- Passed: `cargo check -p zircon_runtime --lib --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short`
- Passed: `git diff --check -- "zircon_runtime/src/scene/reflect/world_reflection.rs" "zircon_runtime/src/scene/tests/ecs_reflect/resources.rs" "zircon_runtime/src/scene/reflect/reflect_resource.rs" "zircon_runtime/src/scene/reflect/type_registry.rs" "zircon_runtime/src/scene/tests/ecs_reflect/mod.rs" "docs/zircon_runtime/scene/reflect.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md"`

Observed output summaries:

- Initial `rustfmt --check` reported formatting drift in the new `resources.rs`; scoped `rustfmt --edition 2021` was run on the touched M8.6 files, and the follow-up formatting check completed with no output.
- First M8.6 `cargo check -p zircon_runtime --lib --locked --message-format short` passed in 1m 58s with one unrelated warning: `zircon_runtime\src\scene\world\render.rs:183:8` method `collect_render_meshes` is never used.
- First parallel `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` timed out at 300s while waiting on Cargo locks and compiling the lib-test target. It did not produce a pass/fail test result and was rerun serially.
- First parallel `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short` passed with 4 passed, 0 failed, 0 ignored, 1175 filtered out after 4m 32s. It emitted two unrelated dead-code warnings in graphics/plugin test helpers.
- Serial rerun `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` passed with 31 passed, 0 failed, 0 ignored, 1153 filtered out after 1m 45s. It emitted two unrelated dead-code warnings in graphics/plugin test helpers.
- Static M8.6 spec compliance review reported `Spec compliant` with no Cargo validation run by the reviewer.
- Focused code-quality review found one low coverage gap: metadata-only resource registrations were not directly tested for `ReflectError::NoResourceAdapter`. `resource_registration_without_adapter_returns_structured_error` now covers that route.
- Systematic follow-up exposed a resource-route category bug: `component_and_resource_reflection_share_address_and_facade_shape` was changed to expect `AddressKindMismatch` for `Name` addressed as a resource and failed before the fix with `NoResourceAdapter { type_path: "zircon_runtime::scene::components::Name" }`.
- After adding the resource-route category check in `world_reflection.rs`, `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect::resources::component_and_resource_reflection_share_address_and_facade_shape --locked --message-format short` passed with 1 passed, 0 failed, 1184 filtered out.
- `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect::resources::resource_registration_without_adapter_returns_structured_error --locked --message-format short` passed with 1 passed, 0 failed, 1184 filtered out.
- Post-fix review reported no findings.
- Final `rustfmt --edition 2021 --check "zircon_runtime/src/scene/reflect/world_reflection.rs" "zircon_runtime/src/scene/tests/ecs_reflect/resources.rs"` completed with no output.
- Final `cargo check -p zircon_runtime --lib --locked --message-format short` passed in 14.88s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- Final `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` passed with 32 passed, 0 failed, 0 ignored, 1153 filtered out after 54.79s. It emitted two unrelated dead-code warnings in graphics/plugin test helpers.
- Final `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short` passed with 4 passed, 0 failed, 0 ignored, 1181 filtered out after 1.58s. It emitted the same unrelated dead-code warnings.
- Final scoped `git diff --check` completed with no output and no whitespace errors.

Result:

- M8.6 implementation, focused resource reflection tests, typed ECS regression tests, docs, review fix, and scoped Cargo validation pass in this checkout. Workspace-wide validation was not run or claimed for M8.6.

## M8.7 Editor Inspector And Remote DTO Reuse Proof

Implementation files:

- `zircon_runtime/src/scene/tests/ecs_reflect/mod.rs`
- `zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs`

Docs:

- `docs/zircon_runtime/scene/reflect.md`
- `docs/zircon_editor/scene/viewport/edit_mode_projection.md`
- `tests/acceptance/reflection-type-registry.md`
- `.codex/sessions/20260508-2036-reflection-type-registry.md`

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Scope:

- Added runtime-side inspector-style field listing proof through `world.reflect_fields(ReflectFieldsRequest::new(...))` for fixed `Name`, fixed `ActiveSelf`, and dynamic plugin JSON component fields.
- Added remote-style schema/read request-response DTO serialization proof by serializing and deserializing `ReflectSchemaRequest`, `ReflectReadRequest`, `ReflectSchemaResponse`, and `ReflectReadResponse` with `serde_json` and asserting response equality.
- Added remote-style plugin-owned filter proof: `ReflectSchemaRequest::remote_visible()` excludes plugin-owned dynamic types by default, while explicit `include_plugin_owned = true` includes them.
- Added remote-style write request-response proof by serializing and deserializing `ReflectWriteRequest`, passing it to `world.reflect_write(...)`, serializing/deserializing the returned `ReflectWriteResponse`, and verifying the world changed through the reflection facade.
- Checked serialized reflection DTO payloads for absence of runtime-only tokens such as `World`, `TypeRegistry`, `ReflectComponent`, and `ReflectResource`.
- Documented the future editor inspector seam: `WorldReflection` DTOs become the field source for existing `SceneInspectorField` projection, while selection, viewport tools, command history, and undo/redo intent remain in `zircon_editor`.
- Kept actual editor projection rewrites, sockets, HTTP routes, BRP handlers, and remote authorization out of M8.7 scope.

Commands:

- Passed: `rustfmt --edition 2021 --check "zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs" "zircon_runtime/src/scene/tests/ecs_reflect/mod.rs"`
- Passed: `cargo check -p zircon_runtime --lib --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed with line-ending warnings only: `git diff --check -- "zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs" "zircon_runtime/src/scene/tests/ecs_reflect/mod.rs" "docs/zircon_runtime/scene/reflect.md" "docs/zircon_editor/scene/viewport/edit_mode_projection.md" "tests/acceptance/reflection-type-registry.md" ".codex/sessions/20260508-2036-reflection-type-registry.md"`

Observed output summaries:

- M8.7 spec compliance review reported `Spec compliant`. The code-quality review found missing write-response serialization coverage and missing default plugin-owned remote filter coverage; both were fixed in `editor_remote.rs` before validation. A final focused review then found schema/read request roundtrips were still response-only; `editor_remote.rs` now roundtrips `ReflectSchemaRequest` and `ReflectReadRequest` before executing the facade path.
- `rustfmt --check` for the M8.7 test files completed with no output.
- Before Cargo validation, the E drive had 13.33 GB free. The low-disk policy cleanup ran `cargo clean --target-dir "E:\cargo-targets\zircon-reflect-m8"`, removed 903 files / 404.7 MiB, and left 13.73 GB free, so validation stayed serial and scoped.
- `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully in 2m 55s with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-m8`.
- `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` finished successfully after a 6m 57s compile and reported 35 passed, 0 failed, 0 ignored, 1425 filtered out.
- Scoped `git diff --check` completed with no whitespace errors. It emitted LF-to-CRLF warnings for `docs/zircon_editor/scene/viewport/edit_mode_projection.md`, `docs/zircon_runtime/scene/reflect.md`, `tests/acceptance/reflection-type-registry.md`, and `zircon_runtime/src/scene/tests/ecs_reflect/mod.rs`.

Result:

- M8.7 implementation, review fixes, runtime inspector/remote DTO tests, docs, focused formatting, focused compile validation, focused reflection tests, and scoped whitespace validation pass in this checkout. Workspace-wide validation was not run or claimed for M8.7.

## M8.8 Documentation And Acceptance Evidence

Documentation files:

- `docs/zircon_runtime_interface/reflect.md`
- `docs/zircon_runtime/scene/reflect.md`
- `docs/zircon_runtime/scene/ecs.md`
- `docs/zircon_editor/scene/viewport/edit_mode_projection.md`
- `tests/acceptance/reflection-type-registry.md`
- `.codex/sessions/20260508-2036-reflection-type-registry.md`

Scope:

- Confirmed the interface reflection doc owns the final DTO file list, tests, design source, object address model, schema/read/write request-response semantics, tagged value contract, and error model.
- Updated the runtime reflection doc header and body so plan sources include the M8.7 editor/remote proof, boundary plan, and design spec, while the body links the concrete adapter/facade ownership split.
- Updated the ECS doc header and body to explain how `ComponentRegistry`, `ResourceRegistry`, plugin `ComponentTypeRegistry`, dynamic JSON descriptors, runtime `TypeRegistry`, and `WorldReflection` relate without collapsing storage identity into schema identity.
- Confirmed the editor viewport projection doc records the runtime reflection field-source seam while selection, tools, command history, undo/redo intent, and authoring UI state remain editor-owned.
- Preserved scoped validation evidence only; workspace-wide and plugin-workspace validation remain unclaimed until the final promotion gate runs.

Commands:

- Passed with line-ending warnings only: `git diff --check -- docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md docs/zircon_runtime_interface/reflect.md docs/zircon_runtime/scene/reflect.md docs/zircon_runtime/scene/ecs.md docs/zircon_editor/scene/viewport/edit_mode_projection.md tests/acceptance/reflection-type-registry.md .codex/sessions/20260508-2036-reflection-type-registry.md`

Observed output summaries:

- Scoped documentation `git diff --check` completed with no whitespace errors. It emitted LF-to-CRLF warnings for `docs/zircon_editor/scene/viewport/edit_mode_projection.md`, `docs/zircon_runtime/scene/ecs.md`, `docs/zircon_runtime/scene/reflect.md`, and `tests/acceptance/reflection-type-registry.md`.

Result:

- M8.8 documentation cleanup and scoped documentation whitespace validation pass in this checkout. The final promotion gate remains pending; workspace-wide and plugin-workspace validation were not run or claimed in M8.8.

## Final Promotion Gate

Validation target directory:

- `E:\cargo-targets\zircon-reflect-m8`

Commands:

- Low-disk cleanup before final scoped tests: `cargo clean --target-dir "E:\cargo-targets\zircon-reflect-m8"`
- Passed: `cargo test -p zircon_runtime_interface --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short`
- Passed: `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short`
- Later broad-validation attempt failed before root workspace tests: `cargo build --workspace --locked --verbose`
- Not run because root workspace build failed first: `cargo test --workspace --locked --verbose`
- Later broad-validation plugin workspace results are recorded below.

Observed output summaries:

- Before final scoped runtime tests, the E drive had 32.67 GB free, which is below the repository's 50 GB Cargo threshold. No Cargo/rustc process was using `E:\cargo-targets\zircon-reflect-m8`, so cleanup removed 4675 files / 5.4 GiB and left 37.89 GB free.
- `cargo test -p zircon_runtime_interface --locked --message-format short` finished successfully in 1m 01s. The lib tests reported 96 passed, 0 failed, 0 ignored, 0 filtered out; doc tests reported 0 tests.
- `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` finished successfully after an 8m 24s compile and reported 35 passed, 0 failed, 0 ignored, 1430 filtered out.
- `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short` finished successfully in 4m 05s and reported 4 passed, 0 failed, 0 ignored, 1463 filtered out.
- `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short` finished successfully after a 10m 28s compile and reported 114 passed, 0 failed, 0 ignored, 1355 filtered out.
- After the focused review fix for schema/read request roundtrips, `rustfmt --edition 2021 --check "zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs"` completed with no output after scoped formatting. The E drive had 43.19 GB free before the retest cleanup; `cargo clean --target-dir "E:\cargo-targets\zircon-reflect-m8"` removed 4193 files / 7.4 GiB and left 48.01 GB free. The retest `cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short` finished successfully after a 10m 49s compile and reported 35 passed, 0 failed, 0 ignored, 1455 filtered out.
- Workspace-wide and plugin-workspace validation were deferred at the time of the scoped gate because this checkout remained heavily dirty with active editor/UI/render/plugin lanes, and the active drive still remained below the 50 GB build threshold after cleaning the isolated reflection target. No workspace-wide green status was claimed from the scoped gate.

Result:

- Reflection M8 scoped promotion passes for the runtime-interface contract suite, focused reflection tests, typed ECS regression tests, and broader runtime scene tests. Root workspace readiness remains unclaimed until the root workspace CI build/test commands pass in a coordinated validation window.

## Broad Workspace And Plugin Validation

Validation target directories:

- `E:\cargo-targets\zircon-reflect-workspace`
- `E:\cargo-targets\zircon-reflect-plugins`

Commands:

- Failed on Windows before Zircon crates finished compiling: `cargo build --workspace --locked --verbose` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-workspace`
- Not run after the failed root build: `cargo test --workspace --locked --verbose`
- Blocked in WSL/Linux CI parity setup: root workspace build/test commands, because required pkg-config packages are missing and noninteractive sudo cannot install them.
- Passed before this continuation in the same broad-validation lane: `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked --verbose` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-plugins`
- Passed: `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-plugins`
- Passed: `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-reflect-plugins`

Observed output summaries:

- The Windows root workspace build failed in third-party `wgpu-hal 29.0.3` DX12 code before completing the root workspace. The first error was `E0308` in `wgpu-hal-29.0.3/src/dx12/suballocation.rs:83`, where `gpu_allocator::d3d12::ID3D12DeviceVersion::Device(...)` expected the `windows 0.61.3`/`windows-core 0.61.2` `ID3D12Device` type but received the `windows 0.62.2`/`windows-core 0.62.2` `Direct3D12::ID3D12Device` type. The same dependency split also produced `ResourceCategory::from(&D3D12_RESOURCE_DESC)` and `ID3D12Heap: Param<ID3D12Heap>` errors, and rustc reported `could not compile wgpu-hal (lib) due to 10 previous errors`.
- Fresh WSL probe at `2026-05-17 13:59 +08:00` reported `Linux 6.6.87.2-microsoft-standard-WSL2 x86_64 GNU/Linux`, `pkg_config=0.29.2`, missing `pkg:x11`, `pkg:xcursor`, `pkg:xcb`, `pkg:xkbcommon`, `pkg:wayland-client`, and `pkg:alsa`, plus `sudo_noninteractive=failed` with `sudo: a password is required`. This blocks local Linux CI-parity root validation until those system packages are installed or a prepared environment is provided.
- The plugin workspace target started with 57.25 GB free on `E:` before the build, so the isolated target was not cleaned. The plugin build finished with `Finished dev profile [unoptimized + debuginfo] target(s) in 3m 13s`.
- The plugin workspace had 55.06 GB free before tests. The plugin test command finished with `Finished test profile [unoptimized + debuginfo] target(s) in 8m 04s`; all visible unit-test and doc-test summaries in the captured output were `ok` with 0 failures.
- After plugin testing, `E:` had 30.93 GB free. No additional broad Cargo build/test was started after recording evidence because the repo policy requires cleaning before further build/test work at or below the 50 GB threshold, and cleaning the plugin target would discard the just-produced cached evidence target.

Result:

- Plugin workspace build/test validation passed in this checkout, with pre-continuation evidence that the plugin all-targets check had already passed in the same isolated target. Root workspace validation remains blocked, not green, because the Windows CI-shape build fails in third-party `wgpu-hal`/`windows` dependency code and local WSL CI-parity validation cannot proceed without missing system dependencies and sudo access.

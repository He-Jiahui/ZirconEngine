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

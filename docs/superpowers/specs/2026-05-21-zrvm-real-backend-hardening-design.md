---
related_code:
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
  - docs/superpowers/specs/2026-05-20-zrvm-reflection-macro-modularity-design.md
  - docs/superpowers/plans/2026-05-20-zrvm-reflection-macro-modularity.md
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/godot/core/object/script_language_extension.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/sub_app.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
plan_sources:
  - user: 2026-05-21 continue ZrVM lane 1 real backend hardening
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - docs/superpowers/specs/2026-05-20-zrvm-reflection-macro-modularity-design.md
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening real_backend -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
doc_type: design-spec
---

# ZrVM Real Backend Hardening Design

## Purpose

The ZrVM host reflection path now has neutral descriptors, registry validation, macro-generated host modules, and generated host interface documentation. The next narrow follow-up is to harden the real `zr_vm` backend adapter so descriptor-to-native-module lowering fails with precise context and has focused regression coverage.

This slice keeps `zircon_plugins/zr_vm_language/runtime` as the owner of backend-specific behavior. The shared runtime descriptor model remains the source of truth, and the backend adapter remains a consumer that lowers validated descriptors into `zr_vm_rust_binding` module, type, function, and callback registrations.

## Scope

In scope:

- validate backend-owned descriptor constraints before calling `zr_vm_rust_binding` builders when the Rust adapter can report clearer errors;
- preserve the existing `real-zr-vm` feature-gated integration path;
- improve contextual error messages for module/function lowering and native callback conversion;
- add focused unit coverage for backend hardening that does not require a local `zr_vm` dynamic library when possible;
- keep existing real backend integration tests for lifecycle/native module loading under `real-zr-vm`;
- update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` with the backend hardening boundary and validation evidence.

Out of scope:

- JSON, TOML, or binary descriptor export;
- new host modules or manager-backed foundation, asset, scene, or render behavior;
- changes to `zircon_runtime::core::framework::script` descriptor DTOs unless implementation proves a real invariant cannot be expressed;
- exposing Rust object pointers, trait objects, `wgpu` objects, scene worlds, or editor state to ZrVM;
- replacing the global `zr_vm` lock or changing process-level binding assumptions;
- changing hot-reload semantics beyond preserving the existing `saveState`/`restoreState` behavior.

## Chosen Approach

Add small private validation and lowering helpers inside `zircon_plugins/zr_vm_language/runtime/src/real_backend.rs`. Keep the public backend surface unchanged: `load_project_package` remains the entry point and `ZrVmBackend` continues to delegate to it behind the `real-zr-vm` feature.

The adapter should validate backend-specific constraints close to the lowering code:

- function `min_argument_count` and `max_argument_count` must fit `u16` because `zrvm::FunctionBuilder::new` takes `u16` arity bounds;
- `min_argument_count <= max_argument_count` before a function builder is created;
- descriptor parameter count must not exceed `max_argument_count`;
- backend errors should include module and function names so failing native registration identifies the source descriptor;
- supported native argument conversion remains limited to null, bool, int, float, and string;
- supported host return lowering remains null, bool, int, float, string, bytes-as-lossy-string, and host handle as integer.

This design does not duplicate `HostExportRegistry` validation. Non-empty names, duplicate modules/types/functions, capability membership, type-ref trimming, and callback-name coherence stay in the registry. The real backend only validates constraints introduced by the target backend ABI or by the adapter's conversion policy.

## Alternatives Considered

### Expand Built-In Host Modules First

This would make ZrVM more useful from scripts, but it would build new behavior on a backend adapter that still has weak boundary diagnostics. It also risks touching asset, scene, or render areas currently active in other sessions.

### Move More Validation Into `HostExportRegistry`

This would centralize more checks but would incorrectly make registry validation know target backend details such as `u16` arity limits. Those limits belong to the `zr_vm` adapter and should not constrain future VM backends unless they share the same ABI.

### Introduce Public Backend Diagnostic DTOs

This could help future tooling, but it is heavier than this slice needs. Current call sites already use `VmError`; private helpers and contextual messages are enough until an editor or CLI consumer needs structured backend diagnostics.

## Architecture

Ownership stays unchanged:

- `zircon_runtime::core::framework::script` owns neutral descriptors, values, type refs, and conversion traits.
- `zircon_runtime::script::vm::host::HostExportRegistry` owns descriptor validation, capability checks, and callback dispatch.
- `zircon_plugins/zr_vm_language/runtime::real_backend` owns real `zr_vm` lowering, native module registration, value conversion across the binding, and lifecycle export calls.
- `VmPluginManager` and `HotReloadCoordinator` own slot lifecycle; this slice does not change their state migration order.

The feature is a leaf hardening pass inside an existing abstraction. It does not create a new crate boundary, public trait, or cross-crate dependency. The adapter consumes descriptors and capabilities through `VmPluginHostContext`, registers native modules into a local `zrvm::Runtime`, and stores registrations in the plugin instance so native modules stay alive as long as the session lives.

The global `zr_vm` lock remains in place. Current tests and plan notes treat `zr_vm_rust_binding` state as process-level shared state, so concurrent real backend calls must continue to serialize until the binding proves otherwise.

## Reference Evidence

The design follows the current ZirconEngine VM/plugin contracts first and uses reference engines to pressure-test lifecycle and extension-boundary behavior.

- Godot `dev/godot/core/extension/gdextension_manager.cpp` returns explicit load statuses for already-loaded, not-loaded, failed, and restart-required extension states. It also unloads and clears callback state on reload failure paths. This supports keeping contextual backend errors close to the extension/backend boundary instead of allowing opaque lower-level failures.
- Godot `dev/godot/core/extension/gdextension.cpp` stores argument and return metadata in `GDExtensionMethodBind`, rejects invalid cached method calls, and validates call shape before dispatch. This supports validating `zr_vm` function arity and conversion shape before native callback dispatch.
- Godot `dev/godot/core/object/script_language_extension.cpp` exposes script validation, reload, documentation, debug, and profiling hooks through explicit extension callbacks. This supports keeping script-language backend behavior behind a bounded adapter rather than leaking VM details into shared runtime DTOs.
- Bevy `dev/bevy/crates/bevy_app/src/plugin.rs` defines staged plugin lifecycle (`build`, `ready`, `finish`, `cleanup`) and duplicate-plugin rejection by name. This supports treating backend registration as lifecycle-bound setup with clear failure points.
- Bevy `dev/bevy/crates/bevy_app/src/sub_app.rs` tracks plugin names, plugin build depth, readiness, finish, and cleanup state. This supports keeping real backend registration and callback lifetimes tied to the plugin instance lifecycle.

Deliberate ZirconEngine divergence: ZrVM integration uses descriptor DTOs, `HostExportRegistry`, `HostHandle`, and `CapabilitySet` rather than reference-engine object pointers or direct method binds. This preserves the repository roadmap rule that VM plugins cross the host boundary through stable handles, capabilities, and serializable values.

## Error Handling

All new backend hardening errors should return `VmError::Operation` with contextual strings. The messages should identify the module and function where applicable, for example:

- `zr_vm function zr.zircon.math.vec3_length min arity exceeds u16`
- `zr_vm function example.bad min arity 3 exceeds max arity 2`
- `zr_vm function example.bad declares 4 parameters but max arity is 2`
- `unsupported zr_vm native argument kind <kind> for <module>.<function>`

The adapter should not silently clamp arity or skip invalid descriptors. If a descriptor cannot be lowered faithfully, loading should fail before the project session starts.

## Testing Plan

Feature-independent unit tests should live in `zircon_plugins/zr_vm_language/runtime/src/lib.rs` or a child test module near `real_backend.rs`, depending on final visibility. They should avoid requiring the `real-zr-vm` feature unless they instantiate actual `zrvm::Runtime` or binding values.

Required focused coverage:

- arity overflow: a descriptor with `min_argument_count` or `max_argument_count` above `u16::MAX` returns contextual `VmError`;
- invalid arity ordering: `min_argument_count > max_argument_count` returns contextual `VmError`;
- parameter/max mismatch: more declared parameters than `max_argument_count` returns contextual `VmError`;
- callback capability denial: a native callback routed through `HostExportRegistry::call_with_capabilities` reports capability failure with module/function context;
- host return lowering: null, bool, int, float, string, bytes, and host handles lower through the supported policy;
- unsupported ZrVM argument value kinds remain errors rather than lossy conversions.

Existing feature-gated real backend tests should continue to cover:

- native host module registration and lifecycle roundtrip;
- persistent session state through `saveState` and `restoreState`;
- documented minimal ZrVM example load and lifecycle.

Validation commands for the implementation testing stage:

```powershell
rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening real_backend -- --nocapture --test-threads=1
```

The `real-zr-vm` command requires local `zr_vm` binding artifacts and `ZR_VM_RUST_BINDING_LIB_DIR`. If those are unavailable, final reporting must state the exact environment blocker and still run non-real-backend unit tests.

## Documentation Plan

Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` after implementation to describe:

- the split between neutral descriptor validation and `zr_vm` backend-specific lowering validation;
- the `u16` arity limit as a target-backend constraint, not a shared descriptor constraint;
- value conversion policy for native arguments and host return values;
- scoped validation evidence and any missing local `zr_vm` binding blocker.

If implementation adds enough backend-specific documentation to warrant a new file, create it under `docs/zircon_plugins/zr_vm_language/runtime/` only if that directory already fits the docs taxonomy; otherwise keep the existing VM host reflection doc as the owner to avoid duplicate subsystem docs.

## Acceptance Criteria

The slice is accepted when:

- backend-specific descriptor constraints fail before `zr_vm` module registration with contextual `VmError` messages;
- `HostExportRegistry` remains the owner for shared descriptor validation and capability enforcement;
- public backend registration APIs and package protocol stay unchanged;
- non-real-backend unit tests cover arity and conversion hardening without requiring a local `zr_vm` dynamic library;
- feature-gated real backend tests pass when local binding artifacts are configured, or the exact environment blocker is recorded;
- docs record the backend hardening boundary and validation evidence.

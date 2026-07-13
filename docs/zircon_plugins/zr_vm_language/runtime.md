---
related_code:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/compiled_call_site.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/param_layout.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/call_site_error.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/errors.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/lock.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/values.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/support.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/zr_reflect.rs
  - zircon_runtime_interface/src/reflect/zr_reflect_value.rs
  - zircon_reflect_derive/src/derive.rs
  - zircon_reflect_derive/src/fields.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/derived/component_adapter.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/vm_type_backing.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/compiled_call_site.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/param_layout.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/call_site_error.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/errors.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/lock.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/values.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/support.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/zr_reflect.rs
  - zircon_runtime_interface/src/reflect/zr_reflect_value.rs
  - zircon_reflect_derive/src/derive.rs
  - zircon_reflect_derive/src/fields.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/derived/component_adapter.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/vm_type_backing.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
plan_sources:
  - user: 2026-05-15 implement ZrVM language plugin and reflection registration plan
  - user: 2026-05-16 continue precise VM host reflection macro implementation
  - user: 2026-07-14 implement docs/plans/zircon_plugins architecture
  - docs/plans/zircon_plugins/08-zr-vm.md
tests:
  - zircon_plugins/zr_vm_language/runtime/src/tests/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/support.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/scene/reflect/vm_type_backing.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime: passed 2026-05-15"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm: passed 2026-05-15 with ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm: extended 2026-05-16 to load a discovered zr_vm:project package, register Zircon host modules, run lifecycle exports, hot reload, and unload"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue and ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: attempted 2026-05-16; blocked before test execution by concurrent cargo package locks and unrelated in-progress zircon_runtime edits"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 --lib --tests with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: attempted 2026-05-16; Cargo exited during dependency compilation without Rust diagnostics while external workspace builds restarted"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16"
  - "cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime: passed 2026-05-16"
  - "cmake --build E:\\Git\\zr_vm\\build\\codex-msvc-debug --config Debug --target zr_vm_rust_binding_shared --parallel 1: passed 2026-05-16 after CallModuleExport entry-load diagnostics patch"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 --lib --tests with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 real_backend_loads_native_host_modules_and_roundtrips_lifecycle -- --nocapture --test-threads=1: attempted 2026-05-16; blocked in zircon_runtime by unrelated UiWidgetBehavior::RadioGroup/Radio non-exhaustive matches in ui/accessibility/extract.rs and ui/surface/surface/default_interactions.rs"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 real_backend_loads_native_host_modules_and_roundtrips_lifecycle -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed, 3 filtered out"
  - "historical pre-v2 ctypes native probe against E:\\Git\\zr_vm\\build\\codex-msvc-debug\\bin\\Debug\\zr_vm_rust_binding.dll: passed 2026-05-16; project compiled, activate() called foundation.time_unix_millis, math.vec3_dot, and foundation.log_info once each, legacy saveState returned string 'created' (superseded by the M5 full-envelope protocol)"
  - "cmake --build E:\\Git\\zr_vm\\build\\codex-msvc-debug --config Debug --target zr_vm_rust_binding_shared --parallel 1: passed 2026-05-16 after ProjectSession ABI patch"
  - "cargo test --manifest-path E:\\Git\\zr_vm\\zr_vm_rust_binding\\rust\\Cargo.toml -p zr_vm_rust_binding --locked --offline --jobs 1 project_session_preserves_module_state_between_export_calls -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed"
  - "cargo test --manifest-path E:\\Git\\zr_vm\\zr_vm_rust_binding\\rust\\Cargo.toml -p zr_vm_rust_binding --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 9 unit tests passed, 6 native registration integration tests passed"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 --lib --tests with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16 after persistent ProjectSession integration"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 real_backend -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 2 passed, 0 failed, 3 filtered out"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16; 3 passed, 0 failed"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 6 passed, 0 failed"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 real_backend_loads_documented_minimal_example -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed, 5 filtered out"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16; 16 passed, 0 failed, 1487 filtered out"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: attempted 2026-05-16; blocked by unrelated zircon_runtime graphics compile errors E0061/E0499 in render frame submission code"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-17 after active graphics/UI blockers settled"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-17; 3 unit tests plus doc-tests"
  - "cargo test -p zircon_runtime --locked --target-dir F:\\cargo-targets\\zircon-zmeta-validation --lib runtime_backed_workspace_plugin_manifests_are_present_in_builtin_catalog -- --nocapture: passed 2026-05-16"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-18"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-18; 3 unit tests plus doc-tests"
  - "cargo fmt --manifest-path zircon_plugins/Cargo.toml --all --check: attempted 2026-05-18; blocked by unrelated unformatted hybrid_gi/runtime and runtime asset/render/scene files owned by concurrent sessions"
  - "2026-05-31: cargo test --manifest-path .\\zircon_plugins\\zr_vm_language\\runtime\\Cargo.toml zr_vm_language_registration_reports_backend_capability --locked --offline --jobs 1 --target-dir D:\\cargo-targets\\zircon-authoring-runtime-metadata --color never --quiet: red before linked capability-status metadata, then passed with existing runtime warnings"
  - "2026-05-31: cargo test --manifest-path .\\Cargo.toml -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --offline --jobs 1 --target-dir D:\\cargo-targets\\zircon-authoring-runtime-metadata --color never --quiet: passed with existing runtime warnings"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --lib zr_vm_language_registration_reports_backend_capability --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\\cargo-targets\\zircon-vampire-plugins: passed 2026-06-09"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --message-format short --color never with CARGO_TARGET_DIR=E:\\cargo-targets\\zircon-vampire-plugins, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug, PATH including E:\\Git\\zr_vm\\build\\codex-msvc-debug\\bin\\Debug: passed 2026-06-09"
doc_type: module-detail
---

# ZrVM Language Runtime Plugin

`zircon_plugin_zr_vm_language_runtime` contributes the `zr_vm` VM backend family. The backend selector for source projects is `zr_vm:project`. Its runtime module resolves `VmPluginManager` and registers `ZrVmBackendFamily` during module activation.

The plugin also contributes scene runtime hooks for entity-bound ZrVM scripts. The registered hook ids are `zr_vm_language.script.scene.fixed_update` and `zr_vm_language.script.scene.update`; both are plugin-id prefixed to satisfy runtime extension validation. Those hooks use scene `script.bindings` data to call package exports through `VmPluginManager`.

The plugin is optional and disabled by default in project selection. This keeps ZirconEngine buildable on machines that do not have `E:\Git\zr_vm` or the `zr_vm_rust_binding` dynamic library available.

The crate root is a structural plugin surface: it declares backend/module children, exposes the plugin descriptor helpers, and delegates unit coverage to `src/tests/mod.rs`. Default-build registration and module tests live in `tests/registration.rs`; feature-gated native binding lifecycle tests live in `tests/real_backend.rs`; temporary ZrVM package fixtures and host-context helpers live in `tests/support.rs`.

## Unified Reflection And Dense Calls

The Plugins 08 M1 path uses `zircon_runtime_interface::reflect::ReflectTypeRegistration` as its only authoritative type and field schema:

- `ZrReflect` derive output registers built-in component metadata and generated accessors; runtime adapters reinsert changed components through `World::insert`, preserving component-store invariants.
- `TypeRegistry::register_vm_type` accepts VM-origin registrations and can back component-shaped types with the existing dynamic component storage path.
- `ZirconScriptType` creates a public script-visible `ReflectTypeRegistration` first. `ScriptHostTypeDescriptor` is then a fallible ABI projection; only VM-specific value kinds, prototype kind, and construction permission remain outside the unified schema.
- `ScriptCallTable` resolves type/member names once when a module is loaded. Compiled call sites retain only dense numeric slots, parameter layout, and captured adapters; steady-state reads and writes invoke numeric field-slot callbacks and never dispatch through the name-based reflection callback.

The script projection validates visibility and field correspondence and reports `ReflectError::InvalidRegistration` instead of relying on an invariant panic. Registration failures are converted to `VmError` only at the host-module boundary where the VM-facing error contract requires it.

`src/real_backend.rs` is now the structural entry for the feature-gated native backend. `real_backend/package.rs` owns package loading and session startup, `instance.rs` owns `VmPluginInstance` lifecycle forwarding, `host_modules.rs` owns host module/type/function registration, `values.rs` owns host-value lowering and ZrVM argument lifting, `errors.rs` owns binding error normalization, and `lock.rs` owns the process-global runtime mutex. `real_backend/tests.rs` keeps private helper coverage for arity validation, value conversion, callback diagnostics, and unsupported argument rejection.

## Runtime Catalog Registration

`RuntimePluginDescriptor::builtin_catalog()` includes the `zr_vm_language` runtime-backed package so the runtime package manifest tests can reconcile three sources of truth:

- `zircon_plugins/Cargo.toml` workspace membership;
- `zircon_plugins/zr_vm_language/plugin.toml`;
- the runtime builtin catalog used by export and plugin manifest projection.

The descriptor uses package id `zr_vm_language`, runtime id `RuntimePluginId::ZrVmLanguage`, crate `zircon_plugin_zr_vm_language_runtime`, target modes `client_runtime`/`server_runtime`/`editor_host`, and capabilities `runtime.plugin.zr_vm_language` plus `runtime.script.backend.zr_vm_project`. Both capabilities are marked partial because the default build registers the backend family but the real native binding remains feature-gated.

The linked runtime package manifest now carries the same category `runtime`, maturity `experimental`, and partial status rows as static `zircon_plugins/zr_vm_language/plugin.toml` and the built-in catalog. This keeps Plugin Manager/export metadata consistent even when the real ZrVM backend remains opt-in through `backend-zr-vm`.

## Build Modes

Default build:

- compiles the plugin crate and registers the backend family type;
- resolves `zr_vm:project`;
- reports `BackendUnavailable` when a package is loaded, because the native binding is not linked.

Real ZrVM build:

- enable feature `backend-zr-vm`;
- set `ZR_VM_RUST_BINDING_LIB_DIR` to the directory containing the built `zr_vm_rust_binding` dynamic library;
- the crate links `zr_vm_rust_binding` from `E:\Git\zr_vm\zr_vm_rust_binding\rust`.

The real backend serializes access through a process-global mutex because the current binding tests show shared C-side runtime state.

Lifecycle export calls pass the target module name to `ProjectSession::call_module_export` and keep `RunOptions::module_name` empty when the session is started. This matches the current `zr_vm_rust_binding` export-call contract: the binding loads the project entry once, then resolves later `module.export` calls from the same project global.

Scene export calls use the same module-export path. `VmPluginInstance::call_export` returns an optional neutral `ScriptHostValue`, and the real backend converts ZrVM null/bool/int/float/string return values back to the runtime host-value surface. Object/array/native-handle returns are still rejected because scene lifecycle hooks do not yet define ownership semantics for those payloads.

The feature-gated test suite includes a real project fixture when `backend-zr-vm` is enabled. That fixture now lives in `tests/support.rs`; it writes a JSON `.zrp`, imports `zr.zircon.math` and `zr.zircon.foundation`, calls native host functions from `activate()`, then verifies the package can be loaded, hot reloaded through `saveState`/`restoreState`, and unloaded. It also copies the documented minimal example into a temporary package root and loads that copy, so the checked-in example stays aligned with the real backend without writing build artifacts into `docs/`.

## Host Module Translation

When `backend-zr-vm` is enabled, `ZrVmBackend`:

1. Opens the discovered `.zrp` project.
2. Builds a standard `zr_vm` runtime.
3. Delegates host module translation to `real_backend/host_modules.rs`, which converts every `HostExportRegistry` module descriptor into a `zr_vm_rust_binding::ModuleBuilder`.
4. Registers native callbacks that dispatch back into `HostExportRegistry::call_with_capabilities` after `real_backend/values.rs` lifts ZrVM arguments into neutral `ScriptHostValue` records.
5. Compiles the project incrementally.
6. Starts a persistent `zr_vm_rust_binding::ProjectSession`.
7. Maps optional lifecycle exports through `real_backend/instance.rs` to `VmPluginInstance` methods.

Host type registration consumes the ABI descriptor projected from unified reflection metadata without re-infering Rust names:

- `ScriptHostPrototypeKind` maps directly to `zr_vm_rust_binding::PrototypeType`.
- `ScriptHostTypeDescriptor::allow_value_construction` maps to `TypeBuilder::allow_value_construction`.
- Reflected fields use `ScriptHostFieldDescriptor::type_ref.type_name`.
- Native function parameters use `ScriptHostParameterDescriptor::type_ref.type_name`.
- Native function return types use `ScriptHostFunctionDescriptor::return_type.type_name`.

This means a Rust helper such as `fn length(value: f64) -> f64` registers as ZrVM `float -> float` by default, while custom value descriptors can register semantic host types such as `Vec3` without exposing Rust object pointers or requiring the plugin backend to know Rust type spelling. Type names, ordered fields, field type paths, and documentation all originate from `ReflectTypeRegistration`; the descriptor does not own a second field schema.

The native callback bridge deliberately stays descriptor-driven rather than type-name-special-cased. The plugin accepts only the host value kinds that the shared script framework already exposes, rejects unsupported ZrVM argument kinds with the target function label, and wraps return-value lowering failures with the same label. This keeps interface registration and reflection metadata aligned with the shared `HostExportRegistry` contract instead of adding backend-local dispatch branches.

The lifecycle names are optional:

- `activate()`
- `deactivate()`
- `saveState(): string`
- `restoreState(state: string)`
- `stateSchema(): string`

`saveState` and `restoreState` use a hard-cut JSON protocol for the complete versioned `VmStateBlob` envelope: `schema_version`, the authoritative type-path/hash table, and payload bytes travel together. Raw payload-only lifecycle strings are no longer accepted. `stateSchema` is optional; when present it returns `VmStateSchema` JSON and enables reflected field migration. Missing lifecycle exports remain accepted, with a missing `stateSchema` selecting opaque-envelope transfer.

## Current Binding Notes

The local `E:\Git\zr_vm` binding used for 2026-05-16 validation includes a `ZrRustBinding_ProjectSession` ABI:

- `ZrRustBinding_ProjectSession_Start` prepares the project, loads the entry module once, and stores the transferred `SZrGlobalState` in a retained execution owner.
- `ZrRustBinding_ProjectSession_CallModuleExport` dispatches lifecycle exports against that same global so module-level VM state survives across `activate`/`saveState`/`restoreState`/`deactivate`.
- `ZrRustBinding_ProjectSession_Free` releases the session owner while live `ZrRustBindingValue` handles can keep the global alive through the existing value owner retain path.

The older `ZrRustBinding_Project_CallModuleExport` API remains a fresh-capture compatibility path. It still preloads the project entry before resolving the export, returns `NOT_FOUND` when the export is absent, and preserves the current VM exception text in runtime-error diagnostics. Without the entry-load fix, `activate()` can compile but fail at export resolution because the project module was never loaded into the prepared runtime global.

`ZrVmPluginInstance` owns one `ProjectSession` per loaded plugin instance. Hot reload creates a new session for the new project image; the manager decodes the old session's complete `saveState` envelope, activates the new instance, optionally reads `stateSchema`, migrates reflected fields, then sends the complete resulting envelope to `restoreState`. The feature-gated fixture publishes a reflected type table and schema so the test traverses the production migration protocol rather than a payload-only string shortcut.

The real fixture avoids ZrVM string conversion syntax inside `activate()`. A direct probe showed `foundation.log_info("activated:" + <string> now + ":" + <string> dot)` fails inside ZrVM with `GET_MEMBER: receiver must be an object, array, or string`; the fixture now keeps `time_unix_millis` and `vec3_dot` calls for host callback coverage, then logs a static string so the test verifies native dispatch rather than a currently failing ZrVM cast/concatenation edge.

The documented minimal example follows the same constraint. `docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr` imports `zr.zircon.foundation`, queries time during `activate()` to prove host access, logs static lifecycle messages, and returns/accepts the complete JSON `VmStateBlob` envelope. Its empty type table intentionally selects opaque mode while still obeying the v2 wire contract.

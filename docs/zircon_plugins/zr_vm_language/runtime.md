---
related_code:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
  - external:E:/Git/zr_vm/zr_vm_rust_binding/include/zr_vm_rust_binding.h
  - external:E:/Git/zr_vm/zr_vm_rust_binding/src/zr_vm_rust_binding/api.c
  - external:E:/Git/zr_vm/zr_vm_rust_binding/src/zr_vm_rust_binding/internal.h
  - external:E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - external:E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding_sys/src/lib.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
  - external:E:/Git/zr_vm/zr_vm_rust_binding/include/zr_vm_rust_binding.h
  - external:E:/Git/zr_vm/zr_vm_rust_binding/src/zr_vm_rust_binding/api.c
  - external:E:/Git/zr_vm/zr_vm_rust_binding/src/zr_vm_rust_binding/internal.h
  - external:E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - external:E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding_sys/src/lib.rs
plan_sources:
  - user: 2026-05-15 implement ZrVM language plugin and reflection registration plan
  - user: 2026-05-16 continue precise VM host reflection macro implementation
tests:
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime: passed 2026-05-15"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm: passed 2026-05-15 with ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm: extended 2026-05-16 to load a discovered zr_vm:project package, register Zircon host modules, run lifecycle exports, hot reload, and unload"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue and ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: attempted 2026-05-16; blocked before test execution by concurrent cargo package locks and unrelated in-progress zircon_runtime edits"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --lib --tests with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: attempted 2026-05-16; Cargo exited during dependency compilation without Rust diagnostics while external workspace builds restarted"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16"
  - "cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime: passed 2026-05-16"
  - "cmake --build E:\\Git\\zr_vm\\build\\codex-msvc-debug --config Debug --target zr_vm_rust_binding_shared --parallel 1: passed 2026-05-16 after CallModuleExport entry-load diagnostics patch"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --lib --tests with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 real_backend_loads_native_host_modules_and_roundtrips_lifecycle -- --nocapture --test-threads=1: attempted 2026-05-16; blocked in zircon_runtime by unrelated UiWidgetBehavior::RadioGroup/Radio non-exhaustive matches in ui/accessibility/extract.rs and ui/surface/surface/default_interactions.rs"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 real_backend_loads_native_host_modules_and_roundtrips_lifecycle -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed, 3 filtered out"
  - "ctypes native probe against E:\\Git\\zr_vm\\build\\codex-msvc-debug\\bin\\Debug\\zr_vm_rust_binding.dll: passed 2026-05-16; project compiled, activate() called foundation.time_unix_millis, math.vec3_dot, and foundation.log_info once each, saveState returned string 'created'"
  - "cmake --build E:\\Git\\zr_vm\\build\\codex-msvc-debug --config Debug --target zr_vm_rust_binding_shared --parallel 1: passed 2026-05-16 after ProjectSession ABI patch"
  - "cargo test --manifest-path E:\\Git\\zr_vm\\zr_vm_rust_binding\\rust\\Cargo.toml -p zr_vm_rust_binding --locked --offline --jobs 1 project_session_preserves_module_state_between_export_calls -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed"
  - "cargo test --manifest-path E:\\Git\\zr_vm\\zr_vm_rust_binding\\rust\\Cargo.toml -p zr_vm_rust_binding --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 9 unit tests passed, 6 native registration integration tests passed"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --lib --tests with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16 after persistent ProjectSession integration"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 real_backend -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 2 passed, 0 failed, 3 filtered out"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16; 3 passed, 0 failed"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 6 passed, 0 failed"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 real_backend_loads_documented_minimal_example -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed, 5 filtered out"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16; 16 passed, 0 failed, 1487 filtered out"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: attempted 2026-05-16; blocked by unrelated zircon_runtime graphics compile errors E0061/E0499 in render frame submission code"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-17 after active graphics/UI blockers settled"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-17; 3 unit tests plus doc-tests"
  - "cargo test -p zircon_runtime --locked --target-dir F:\\cargo-targets\\zircon-zmeta-validation --lib runtime_backed_workspace_plugin_manifests_are_present_in_builtin_catalog -- --nocapture: passed 2026-05-16"
  - "cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-18"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros-plugins: passed 2026-05-18; 3 unit tests plus doc-tests"
  - "cargo fmt --manifest-path zircon_plugins/Cargo.toml --all --check: attempted 2026-05-18; blocked by unrelated unformatted hybrid_gi/runtime and runtime asset/render/scene files owned by concurrent sessions"
doc_type: module-detail
---

# ZrVM Language Runtime Plugin

`zircon_plugin_zr_vm_language_runtime` contributes the `zr_vm` VM backend family. The backend selector for source projects is `zr_vm:project`. Its runtime module resolves `VmPluginManager` and registers `ZrVmBackendFamily` during module activation.

The plugin is optional and disabled by default in project selection. This keeps ZirconEngine buildable on machines that do not have `E:\Git\zr_vm` or the `zr_vm_rust_binding` dynamic library available.

## Runtime Catalog Registration

`RuntimePluginDescriptor::builtin_catalog()` includes the `zr_vm_language` runtime-backed package so the runtime package manifest tests can reconcile three sources of truth:

- `zircon_plugins/Cargo.toml` workspace membership;
- `zircon_plugins/zr_vm_language/plugin.toml`;
- the runtime builtin catalog used by export and plugin manifest projection.

The descriptor uses package id `zr_vm_language`, runtime id `RuntimePluginId::ZrVmLanguage`, crate `zircon_plugin_zr_vm_language_runtime`, target modes `client_runtime`/`server_runtime`/`editor_host`, and capabilities `runtime.plugin.zr_vm_language` plus `runtime.script.backend.zr_vm_project`. Both capabilities are marked partial because the default build registers the backend family but the real native binding remains feature-gated.

## Build Modes

Default build:

- compiles the plugin crate and registers the backend family type;
- resolves `zr_vm:project`;
- reports `BackendUnavailable` when a package is loaded, because the native binding is not linked.

Real ZrVM build:

- enable feature `real-zr-vm`;
- set `ZR_VM_RUST_BINDING_LIB_DIR` to the directory containing the built `zr_vm_rust_binding` dynamic library;
- the crate links `zr_vm_rust_binding` from `E:\Git\zr_vm\zr_vm_rust_binding\rust`.

The real backend serializes access through a process-global mutex because the current binding tests show shared C-side runtime state.

Lifecycle export calls pass the target module name to `ProjectSession::call_module_export` and keep `RunOptions::module_name` empty when the session is started. This matches the current `zr_vm_rust_binding` export-call contract: the binding loads the project entry once, then resolves later `module.export` calls from the same project global.

The feature-gated test suite includes a real project fixture when `real-zr-vm` is enabled. That fixture writes a JSON `.zrp`, imports `zr.zircon.math` and `zr.zircon.foundation`, calls native host functions from `activate()`, then verifies the package can be loaded, hot reloaded through `saveState`/`restoreState`, and unloaded. It also copies the documented minimal example into a temporary package root and loads that copy, so the checked-in example stays aligned with the real backend without writing build artifacts into `docs/`.

## Host Module Translation

When `real-zr-vm` is enabled, `ZrVmBackend`:

1. Opens the discovered `.zrp` project.
2. Builds a standard `zr_vm` runtime.
3. Converts every `HostExportRegistry` module descriptor into a `zr_vm_rust_binding::ModuleBuilder`.
4. Registers native callbacks that dispatch back into `HostExportRegistry::call_with_capabilities`.
5. Compiles the project incrementally.
6. Starts a persistent `zr_vm_rust_binding::ProjectSession`.
7. Maps optional lifecycle exports to `VmPluginInstance` methods.

Host type registration uses descriptor metadata without re-infering Rust names:

- `ScriptHostPrototypeKind` maps directly to `zr_vm_rust_binding::PrototypeType`.
- `ScriptHostTypeDescriptor::allow_value_construction` maps to `TypeBuilder::allow_value_construction`.
- Reflected fields use `ScriptHostFieldDescriptor::type_ref.type_name`.
- Native function parameters use `ScriptHostParameterDescriptor::type_ref.type_name`.
- Native function return types use `ScriptHostFunctionDescriptor::return_type.type_name`.

This means a Rust helper such as `fn length(value: f64) -> f64` registers as ZrVM `float -> float` by default, while custom value descriptors can register semantic host types such as `Vec3` without exposing Rust object pointers or requiring the plugin backend to know Rust type spelling.

The lifecycle names are optional:

- `activate()`
- `deactivate()`
- `saveState(): string`
- `restoreState(state: string)`

`saveState` and `restoreState` map to `VmStateBlob` UTF-8 bytes. Missing lifecycle exports are accepted.

## Current Binding Notes

The local `E:\Git\zr_vm` binding used for 2026-05-16 validation includes a `ZrRustBinding_ProjectSession` ABI:

- `ZrRustBinding_ProjectSession_Start` prepares the project, loads the entry module once, and stores the transferred `SZrGlobalState` in a retained execution owner.
- `ZrRustBinding_ProjectSession_CallModuleExport` dispatches lifecycle exports against that same global so module-level VM state survives across `activate`/`saveState`/`restoreState`/`deactivate`.
- `ZrRustBinding_ProjectSession_Free` releases the session owner while live `ZrRustBindingValue` handles can keep the global alive through the existing value owner retain path.

The older `ZrRustBinding_Project_CallModuleExport` API remains a fresh-capture compatibility path. It still preloads the project entry before resolving the export, returns `NOT_FOUND` when the export is absent, and preserves the current VM exception text in runtime-error diagnostics. Without the entry-load fix, `activate()` can compile but fail at export resolution because the project module was never loaded into the prepared runtime global.

`ZrVmPluginInstance` owns one `ProjectSession` per loaded plugin instance. Hot reload creates a new session for the new project image; the manager saves state from the old session, activates the new instance, then calls `restoreState` on the new session. The focused real-backend test asserts that `saveState` observes the `activate` mutation and then observes the `restoreState` mutation, covering the state continuity that the earlier fresh-capture path could not provide.

The real fixture avoids ZrVM string conversion syntax inside `activate()`. A direct probe showed `foundation.log_info("activated:" + <string> now + ":" + <string> dot)` fails inside ZrVM with `GET_MEMBER: receiver must be an object, array, or string`; the fixture now keeps `time_unix_millis` and `vec3_dot` calls for host callback coverage, then logs a static string so the test verifies native dispatch rather than a currently failing ZrVM cast/concatenation edge.

The documented minimal example follows the same constraint. `docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr` imports `zr.zircon.foundation`, queries time during `activate()` to prove host access, logs static lifecycle messages, and keeps hot-reload state as plain strings returned by `saveState()` and accepted by `restoreState(state)`.

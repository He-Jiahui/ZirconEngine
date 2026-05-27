---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/reflection_macros/Cargo.toml
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime/reflection_macros/src/tokens.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/options.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/writer.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
implementation_files:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/reflection_macros/Cargo.toml
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime/reflection_macros/src/tokens.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/options.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/writer.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
plan_sources:
  - user: 2026-05-15 implement ZrVM language plugin and reflection registration plan
  - user: 2026-05-16 continue precise VM host reflection macro implementation
  - user: 2026-05-18 modular reflection content/generated reflection interface documentation
  - user: 2026-05-20 continue ZrVM host reflection follow-up with macro modularity
  - user: 2026-05-21 continue ZrVM lane 1 real backend hardening
  - docs/superpowers/specs/2026-05-20-zrvm-reflection-macro-modularity-design.md
  - docs/superpowers/plans/2026-05-20-zrvm-reflection-macro-modularity.md
  - docs/superpowers/plans/2026-05-18-zrvm-host-reflection-docs.md
  - docs/superpowers/specs/2026-05-21-zrvm-real-backend-hardening-design.md
  - docs/superpowers/plans/2026-05-21-zrvm-real-backend-hardening.md
tests:
  - zircon_runtime/src/script/vm/tests.rs
  - "cargo test -p zircon_runtime script::vm: passed 2026-05-15"
  - "cargo test -p zircon_runtime script::vm --locked --target-dir target\\codex-reflection-macros: passed 2026-05-16"
  - "cargo fmt --manifest-path zircon_runtime/reflection_macros/Cargo.toml --check: passed 2026-05-16"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-16"
  - "cargo test -p zircon_runtime script::vm --locked --jobs 1: attempted 2026-05-16 in E:\\cargo-targets\\zircon-zrvm-continue; local machine remained saturated by concurrent cargo jobs before completion"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16; 16 passed, 0 failed, 1487 filtered out"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-runtime-check: passed 2026-05-16"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 real_backend_loads_documented_minimal_example -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed, 5 filtered out"
  - "cargo check --workspace --locked: passed 2026-05-15"
  - "cargo test --workspace --locked: attempted 2026-05-15; stopped by no space on device"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: attempted 2026-05-18; blocked before reflection tests by unrelated graphics test compile errors in zircon_runtime/src/graphics/tests/render_product_ui.rs missing RenderStats UI fields"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18; 0 unit tests and 0 doc-tests"
  - "rustfmt --edition 2021 --check zircon_runtime/src/core/framework/script.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/src/script/vm/host/builtin_host_modules.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-18"
  - "cargo fmt --all --check: attempted 2026-05-18; blocked by unrelated unformatted asset/render/scene files owned by concurrent sessions"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: attempted 2026-05-18 after enum/default-type-ref hardening; blocked by unrelated graphics compile error E0061 in zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18 after enum/default-type-ref hardening; 0 unit tests and 0 doc-tests"
  - "rustfmt --edition 2021 --check zircon_runtime/src/core/framework/script.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/src/script/vm/host/builtin_host_modules.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-18 after enum/default-type-ref hardening"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: red 2026-05-18 for unsupported-input macro tests before guards; async/generic function and generic type tests failed because macros still emitted descriptors"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18 after unsupported-input guards; 3 passed, 0 failed, 0 doc-tests"
  - "rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/src/core/framework/script.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/host/builtin_host_modules.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-18 after unsupported-input guards"
  - "cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- --nocapture --test-threads=1: passed 2026-05-18 during Milestone 2; 4 host_reflection_docs tests passed"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs: passed 2026-05-18 during Milestone 2"
  - "cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 during Milestone 3; generated explicit-output host interface Markdown"
  - "Test-Path -LiteralPath 'F:\\cargo-targets\\codex-reflection-docs\\host-interface.md': passed 2026-05-18 during Milestone 3; generated file existed"
  - "Grep tool search for 'zr\\.zircon\\.math' in F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 during Milestone 3; generated file included zr.zircon.math at line 76"
  - "rustfmt --edition 2021 --check zircon_runtime/src/script/vm/host/reflection_docs/mod.rs zircon_runtime/src/script/vm/host/reflection_docs/options.rs zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs zircon_runtime/src/script/vm/host/reflection_docs/writer.rs zircon_runtime/src/script/vm/host/mod.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/src/bin/zircon_host_reflection_docs.rs: passed 2026-05-18 final validation"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs: passed 2026-05-18 final validation; 3 passed, 0 failed, 0 doc-tests"
  - "cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- --nocapture --test-threads=1: passed 2026-05-18 final validation; 4 host_reflection_docs tests passed, 1561 filtered out"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs: passed 2026-05-18 final validation"
  - "cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 final validation; generated explicit-output host interface Markdown"
  - "Test-Path -LiteralPath 'F:\\cargo-targets\\codex-reflection-docs\\host-interface.md': passed 2026-05-18 final validation; generated file existed"
  - "Grep tool search for 'zr\\.zircon\\.math' in F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 final validation; generated file included zr.zircon.math at line 76"
  - "rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs: passed 2026-05-20 final validation"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity: passed 2026-05-20 final validation; 10 passed, 0 failed, 0 doc-tests"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose: passed 2026-05-20 evidence run; 2 existing warnings in scene ECS helpers"
  - "cargo test -p zircon_runtime --lib rust_reflection_macros_generate_type_function_and_module_descriptors --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-20 final validation; 1 passed, 0 failed, 1745 filtered out"
  - "cargo test -p zircon_runtime --lib host_reflection_docs_include_macro_generated_builtin_math_module --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-20 final validation; 1 passed, 0 failed, 1746 filtered out"
  - "F: free space check before closeout validation: passed 2026-05-21; 66.86 GB free, no target cleanup required for F:\\cargo-targets\\codex-reflection-macro-modularity"
  - "rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs: passed 2026-05-21 closeout validation"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity: passed 2026-05-21 closeout validation; 10 passed, 0 failed, 0 doc-tests"
  - "cargo test -p zircon_runtime --lib rust_reflection_macros_generate_type_function_and_module_descriptors --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-21 closeout validation; 1 passed, 0 failed, 1746 filtered out"
  - "cargo test -p zircon_runtime --lib host_reflection_docs_include_macro_generated_builtin_math_module --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-21 closeout validation; 1 passed, 0 failed, 1746 filtered out"
  - "F: free space check before ZrVM real-backend hardening validation: passed 2026-05-24; 93.32 GB free, no target cleanup required for F:\\cargo-targets\\codex-zrvm-real-backend-hardening"
  - "rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-24"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-zrvm-real-backend-hardening: passed 2026-05-24; 3 passed, 0 failed, 0 doc-tests"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-zrvm-real-backend-hardening real_backend -- --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug and PATH including E:\\Git\\zr_vm\\build\\codex-msvc-debug\\bin\\Debug: passed 2026-05-24; 11 passed, 0 failed, 3 filtered out"
doc_type: module-detail
---

# ZrVM Host Reflection

The VM host surface is split into two layers:

- `zircon_runtime::core::framework::script` owns neutral descriptors and values. VM backends can read `ScriptHostModuleDescriptor`, `ScriptHostFunctionDescriptor`, `ScriptHostTypeDescriptor`, `ScriptHostValueKind`, `ScriptHostCallContext`, and `ScriptHostResult` without depending on concrete runtime managers.
- `zircon_runtime::script::vm::host` owns registration, handle allocation, validation, capability checks, and callback dispatch.

VM code never receives Rust object pointers. Host objects are represented as `HostHandle` values, and framework-level values carry those handles as `u64` so the neutral contract does not depend on the VM subsystem.

`zircon_runtime_reflection_macros` is the convenience layer for Rust-authored host libraries. `ZirconScriptType`, `zircon_host_function`, and `zircon_host_module` emit the same neutral descriptors as handwritten registrations. Function parameters now derive their exported type names from `ScriptHostFromValue::script_host_type_ref`, so Rust `f64` exports as the VM-facing `float` type instead of leaking a Rust-only spelling into ZrVM native module metadata.

## Type Reflection Model

`ScriptHostValueKind` remains the coarse ABI-lowering category used by host calls. `ScriptHostTypeRef` carries the VM-facing type name beside that value kind, allowing a host function to lower as `Float` while still registering a semantic type such as `Vec3`, `ColorRgba`, or `float` with a backend. Function and parameter descriptors default primitive type refs from their value kind, while type descriptors default the type ref name to the descriptor name so handwritten semantic types do not accidentally collapse back to `float` or `int`.

`ScriptHostPrototypeKind` describes the VM prototype that should be used for a host type: module, class, interface, struct, enum, or native. The derive macro defaults Rust structs to `Struct` and Rust enums to `Enum`; callers can still override the prototype with `#[zircon_script(prototype = ...)]` when a host type intentionally maps to another VM shape. `ScriptHostTypeDescriptor::allow_value_construction` records whether the VM may construct values directly from the reflected descriptor. These fields are intentionally descriptor data only; scripts still receive values or `HostHandle` identifiers, not Rust object pointers.

The conversion traits are the Rust-side source of default type refs:

- `ScriptHostFromValue` converts script arguments into Rust parameters and exposes the exported argument type ref.
- `ScriptHostIntoValue` converts Rust return values into `ScriptHostValue` and exposes the exported return type ref.
- `ZirconScriptType` produces a complete `ScriptHostTypeDescriptor` for Rust-authored value types.

The macro entry points reject unsupported Rust shapes instead of emitting descriptors that would fail later through trait-bound or runtime errors. `zircon_host_function` supports synchronous, non-generic free functions with simple identifier parameters. `ZirconScriptType` supports non-generic structs and enums; unions are rejected. Async functions, generic functions, and generic script types must be wrapped in a concrete host export before reflection.

The proc-macro crate is a separate workspace member because Rust procedural macros must live in a `proc-macro` crate. `zircon_runtime` re-exports the macros so runtime-owned host modules can write `#[crate::zircon_host_function]`, `#[crate::zircon_host_module]`, and `#[derive(crate::ZirconScriptType)]` without depending on the macro crate directly.

The macro crate is split by code-generation responsibility. `lib.rs` contains only the Rust-required proc-macro entry points and module declarations. `args.rs` owns attribute argument parsing, `attrs.rs` owns `#[zircon_script]` parsing and item discovery, `derive_type.rs` emits `ZirconScriptType` descriptors, `function.rs` emits host function descriptors and callbacks, `module.rs` emits host module descriptors and registration functions, `tokens.rs` owns shared token helpers, and `tests.rs` covers unsupported input plus descriptor metadata generation. Runtime validation remains in `HostExportRegistry`; the macro crate only rejects Rust shapes that cannot be represented correctly as host descriptors.

## Registry Behavior

`HostExportRegistry` validates a module before it becomes visible:

- module, version, capability, type, function, and parameter names must be non-empty and trimmed;
- module names, type names, function names, and parameter names must not duplicate within their scope;
- type, field, parameter, and return `ScriptHostTypeRef` names must be non-empty and already trimmed;
- every type ref value kind must match the descriptor value kind that will be used for call lowering;
- a registered type descriptor's own `type_ref.type_name` must match its descriptor name;
- field names must not duplicate within a reflected type;
- function arity must be coherent with its parameter list;
- function required capabilities must be declared on the module;
- callbacks must exactly match declared function names.
- duplicate callback names are rejected before callback storage, so a later callback cannot silently replace an earlier one.

Each registered module receives a `HostHandle` through the shared `HostRegistry`, using a `host.module.<module>` capability label. This keeps script-visible handles stable and lets existing handle validation continue to work.

Calls go through `call_with_capabilities`. The registry checks arity and required capabilities before building a `ScriptHostCallContext` and dispatching the callback. Backends should pass the package capability set from `VmPluginHostContext`.

## Built-In Modules

`PluginHostDriver::default()` registers first-wave built-in host modules:

- `zr.zircon.foundation`: time, log, and event helper descriptors.
- `zr.zircon.asset`: locator/status/revision query descriptors.
- `zr.zircon.scene`: default world handle and handle validity helpers.
- `zr.zircon.render`: read-only render metadata descriptors.
- `zr.zircon.math`: pure value descriptors and deterministic vector helpers.

`zr.zircon.math` is registered through the reflection macros. `Vec3` and `ColorRgba` derive `ZirconScriptType`, and pure helpers such as vector length and dot product use `zircon_host_function`, proving that macro-generated descriptors flow through the same registry validation and dispatch path as handwritten modules.

This first wave deliberately favors stable values and handles over concrete manager references. Manager-backed behavior can replace the diagnostic placeholders once the target services expose stable trait-object access through `core::manager`.

## Generated Interface Documentation

Generated ZrVM host interface documentation is descriptor-driven. `ScriptHostModuleDescriptor`, `ScriptHostTypeDescriptor`, `ScriptHostFunctionDescriptor`, `ScriptHostParameterDescriptor`, `ScriptHostFieldDescriptor`, and `ScriptHostTypeRef` remain the source of truth; the Markdown renderer reads those descriptors instead of reflecting Rust implementation details or querying a backend-specific ABI. Built-in documentation uses `builtin_host_module_descriptors()` to register the same first-wave host modules into a local registry and then renders the validated descriptor records.

The renderer keeps output deterministic for review and generated-file comparison. Modules are sorted by module name, capabilities are sorted by capability string, reflected types are sorted by type name, and functions are sorted by function name. Field order and function parameter order stay descriptor-defined because those sequences describe user-facing struct layout and call signatures.

The writer command is explicit-output only:

```powershell
cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- F:\cargo-targets\codex-reflection-docs\host-interface.md
```

The command requires exactly one output Markdown path, creates missing parent directories through the writer API, and does not commit a machine-specific generated artifact path into the repository. Callers choose where generated interface documentation is emitted.

The built-in math module is the proof that handwritten and macro-generated descriptors flow through one documentation path. `zr.zircon.math` is registered through the reflection macros, then rendered from the same descriptor model as the handwritten built-ins; Milestone 3 generated output was inspected and contained `zr.zircon.math` at line 76.

## Real Backend Lowering Boundary

The real `zr_vm` backend treats `HostExportRegistry` records as already validated neutral descriptors, then applies only target-backend lowering checks. Function arity must fit the `zr_vm` native function ABI (`u16` min/max bounds), `min_argument_count` must not exceed `max_argument_count`, and reflected parameter count must fit the maximum arity. These are backend constraints, not shared descriptor constraints for every future VM backend.

Native callbacks convert ZrVM null, bool, int, float, and string arguments into `ScriptHostValue` before dispatching through `HostExportRegistry::call_with_capabilities`. Host return values lower null, bool, int, float, string, bytes as lossy UTF-8 strings, and `HostHandle` as integers. Unsupported ZrVM argument kinds remain errors with module/function context rather than lossy conversions.

## Package Protocol

`discover_vm_plugin_package` still supports bytecode packages. A ZrVM project package uses:

```toml
backend = "zr_vm:project"

[zr_vm]
project = "plugin.zrp"
entry_module = "main"
execution_mode = "binary"
```

Project packages store no bytecode in `VmPluginPackage::bytecode`; instead they populate `VmPluginPackage::zr_vm_project` and `VmPluginPackageSource::zr_vm_project_path`.

The checked-in minimal example lives at `docs/zircon_runtime/script/vm/examples/zr_vm_minimal`. It uses the same package protocol, imports `zr.zircon.foundation`, calls `foundation.time_unix_millis()` and `foundation.log_info()` from `activate()`, and demonstrates hot-reload state through optional `saveState(): string` and `restoreState(state: string)` exports. The real ZrVM plugin tests copy that example to a temporary package root before loading it so validation does not leave compiled artifacts under `docs/`.

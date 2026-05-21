---
related_code:
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
  - docs/superpowers/specs/2026-05-18-zrvm-host-reflection-docs-design.md
  - zircon_runtime/reflection_macros/Cargo.toml
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/tests.rs
implementation_files:
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime/reflection_macros/src/tokens.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
plan_sources:
  - user: 2026-05-20 continue ZrVM host reflection follow-up
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - docs/superpowers/specs/2026-05-18-zrvm-host-reflection-docs-design.md
tests:
  - cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity
  - rustfmt --edition 2021 --check over touched macro crate and runtime docs-related Rust files
doc_type: design-spec
---

# ZrVM Reflection Macro Modularity Design

## Purpose

The ZrVM host reflection surface now has neutral descriptors, registry validation, generated interface documentation, and a macro-generated built-in math module that proves handwritten and macro-generated descriptors share one path. The next safe follow-up is to make the procedural macro crate maintainable before adding more host APIs or backend features.

The goal is to split `zircon_runtime/reflection_macros/src/lib.rs` into focused modules and harden macro-level descriptor validation tests. This keeps descriptor generation predictable while the host reflection surface grows.

JSON descriptor export is intentionally out of scope for this slice. It should wait for a concrete backend, documentation, editor, or tooling consumer so the export contract is not invented prematurely.

## Scope

In scope:

- split the proc-macro crate into responsibility-focused source files;
- preserve the public macro surface: `ZirconScriptType`, `zircon_host_function`, and `zircon_host_module`;
- preserve generated descriptor semantics for current built-in math module and tests;
- add or retain macro tests that reject unsupported inputs before descriptor generation;
- add descriptor-quality tests for metadata that the macros own, such as type names, capabilities, docs, field skipping, prototype kind, and return type refs;
- update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` with the new macro crate structure and fresh validation evidence.

Out of scope:

- JSON, TOML, or binary descriptor export;
- new VM host modules beyond the existing built-ins;
- new manager-backed foundation, asset, scene, or render behavior;
- changes to the real ZrVM backend registration ABI;
- exposing Rust object pointers or trait objects to VM code;
- scene/world `TypeRegistry` reflection, which is a separate reflection domain.

## Chosen Approach

Use a hard cutover from one monolithic macro file to a folder-like module set inside the same proc-macro crate. The crate remains a proc-macro crate and `lib.rs` remains the only file with `#[proc_macro_*]` entry points.

Target source layout:

```text
zircon_runtime/reflection_macros/src/
  lib.rs
  args.rs
  attrs.rs
  derive_type.rs
  function.rs
  module.rs
  tokens.rs
  tests.rs
```

Responsibilities:

- `lib.rs`: proc-macro entry points, module declarations, and narrow dispatch into implementation functions.
- `args.rs`: attribute argument structs and `syn::parse::Parse` implementations for host functions and host modules.
- `attrs.rs`: `#[zircon_script(...)]` parsing plus derive/attribute discovery helpers.
- `derive_type.rs`: `ZirconScriptType` derive expansion and field descriptor generation.
- `function.rs`: `zircon_host_function` expansion, parameter extraction, argument conversion, and callback generation.
- `module.rs`: `zircon_host_module` expansion and module descriptor registration generation.
- `tokens.rs`: shared token helpers such as type-ref generation and path token conversion.
- `tests.rs`: macro crate unit tests for unsupported input and descriptor metadata generation helpers.

This is a source reorganization plus test-hardening slice. It should not change generated runtime descriptors except where a test exposes an existing macro bug.

## Alternatives Considered

### Add More Built-In Host Modules

This would expose more runtime functionality to ZrVM immediately. It was rejected for this slice because the macro crate is already the weakest maintainability point in the current reflection path, and adding more host APIs would increase pressure on a monolithic code generator.

### Backend Registration Hardening First

This would add more real-backend tests for arity overflow, unsupported value kinds, host handle mapping, and repeated registration. It remains valuable follow-up work, but it does not address the current macro crate boundary drift.

### Macro Modularization Plus JSON Export

This would pair the refactor with a new machine-readable descriptor export. It was rejected for this slice because there is no current concrete consumer. The descriptor model already serializes through `serde`, and a dedicated export contract can be designed later when a consumer defines ordering, schema stability, and versioning needs.

## Architecture

Ownership remains unchanged:

- `zircon_runtime::core::framework::script` owns neutral descriptor DTOs and conversion traits.
- `zircon_runtime::script::vm::host` owns registry validation, host handles, capabilities, callbacks, built-ins, and generated Markdown documentation.
- `zircon_runtime_reflection_macros` only generates code that builds the neutral descriptors and host callbacks.
- `zircon_plugins/zr_vm_language/runtime` consumes descriptors through the existing `HostExportRegistry` records when `real-zr-vm` is enabled.

The macro crate must not become a runtime reflection implementation layer. It must not render docs, export files, register built-ins by itself, or know backend-specific ABI details. It only translates Rust-authored host declarations into descriptor-building Rust code.

Root wiring files stay structural. `reflection_macros/src/lib.rs` is allowed to host proc-macro entry points because Rust requires those at the proc-macro crate root, but parsing, expansion, and helper logic move into focused modules.

## Descriptor Quality Rules

The macros should continue to reject unsupported Rust shapes early:

- `zircon_host_function` rejects async functions;
- `zircon_host_function` rejects generic functions;
- `zircon_host_function` rejects methods and non-identifier parameters;
- `zircon_host_module` requires an inline module body;
- `ZirconScriptType` rejects generic types;
- `ZirconScriptType` rejects unions.

The refactor should add focused tests for macro-owned metadata when practical:

- host module name, version, capabilities, and documentation are preserved;
- host function name, capabilities, documentation, return type name, and parameter type refs are preserved;
- script type name, prototype kind, value kind, value construction flag, field docs, and skipped fields are preserved;
- enum script types default to `ScriptHostPrototypeKind::Enum`;
- struct script types default to `ScriptHostPrototypeKind::Struct`.

Runtime descriptor validation remains in `HostExportRegistry`. The macro crate should not duplicate every registry rule. Macro tests should focus on code-generation ownership and on invalid Rust shapes that are cheaper and clearer to reject during expansion.

## Reference Evidence

The local design follows the current ZirconEngine descriptor path first, then uses reference engines only to pressure-test the shape.

- Bevy `dev/bevy/crates/bevy_reflect/src/type_registry.rs` keeps type registration centralized through `TypeRegistry`, `TypeRegistration`, and `GetTypeRegistration`; derived reflection contributes metadata into that registry rather than spreading runtime behavior through the derive entry point.
- Bevy `dev/bevy/crates/bevy_reflect/compile_fail/tests/reflect_derive/generics_fail.rs` shows compile-time rejection coverage for unsupported derive shapes, matching the local macro crate's unsupported-input tests.
- Godot `dev/godot/tests/core/object/test_method_bind.cpp` binds methods through explicit `ClassDB::bind_method` metadata and then calls them through the reflected object surface, matching the local separation between descriptors and callback dispatch.
- Godot `dev/godot/tests/core/object/test_class_db.cpp` validates exposed class, builtin type, and enum metadata centrally through `ClassDB`, matching the local rule that registry validation remains centralized in `HostExportRegistry`.
- Fyrox `dev/Fyrox/template-core/src/lib.rs` generates scripts with `Visit`, `Reflect`, `TypeUuidProvider`, and `ComponentProvider` derives while plugin lifecycle stays in `Plugin`, matching the local split between macro-generated metadata and runtime plugin lifecycle.

Deliberate ZirconEngine divergence: the current VM host reflection model uses ABI-safe `ScriptHostValue` values, `HostHandle`, capability sets, and serialized descriptor DTOs instead of reference-engine object pointers or direct object reflection. This preserves the ZrVM plugin boundary and hot-reload expectations from the repository roadmap.

## Testing Plan

Focused macro crate tests:

- current unsupported-input tests continue to pass;
- add tests for rejected methods, non-identifier parameters, non-inline modules, and unions if they are not already covered;
- add metadata tests that call expansion helper functions or generated descriptors where practical.

Runtime integration checks:

- existing `rust_reflection_macros_generate_type_function_and_module_descriptors` continues to pass;
- existing built-in math descriptor and generated documentation tests continue to prove the macro path participates in runtime host reflection.

Validation commands for the implementation testing stage:

```powershell
rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs
cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity
cargo test -p zircon_runtime rust_reflection_macros_generate_type_function_and_module_descriptors --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity -- --nocapture --test-threads=1
cargo test -p zircon_runtime host_reflection_docs_include_macro_generated_builtin_math_module --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity -- --nocapture --test-threads=1
```

If runtime scoped validation is blocked by unrelated compile errors from active UI/rendering work, record the exact first blocker and do not claim runtime or workspace green.

## Documentation Plan

Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` after implementation to describe:

- the macro crate module layout;
- the boundary between macro generation and runtime descriptor validation;
- unsupported Rust shapes rejected by the macros;
- fresh scoped validation evidence and any unrelated blockers.

The implementation should also keep this spec's machine-readable file header accurate if the final file list changes.

## Acceptance Criteria

The slice is accepted when:

- `reflection_macros/src/lib.rs` is reduced to proc-macro entry points and structural wiring;
- parsing, attribute handling, derive expansion, function expansion, module expansion, token helpers, and macro crate tests live in focused files;
- public macro names and generated descriptor semantics stay compatible with current runtime tests;
- unsupported input and descriptor metadata tests cover the macro-owned boundary;
- generated built-in math reflection and host reflection documentation still pass focused validation when unrelated workspace blockers allow it;
- docs record the new structure, validation evidence, and any open blockers.

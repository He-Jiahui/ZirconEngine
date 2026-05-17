---
related_code:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
implementation_files:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
plan_sources:
  - user: 2026-05-18 requested modular reflection content and generated reflection interface documentation
tests:
  - cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macros
  - cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macros
doc_type: design-spec
---

# ZrVM Host Reflection Interface Documentation Design

## Purpose

The current ZrVM host reflection work can describe host modules, types, fields, functions, parameters, return types, required capabilities, and documentation strings through neutral runtime descriptors. The next step is to make that reflection content modular and generate interface documentation from the same descriptor data that VM backends consume.

The first implementation targets a runtime documentation API plus a small checked-in writer. The API must render deterministic Markdown from `ScriptHostModuleDescriptor` values. The writer must collect registered host descriptors and write the rendered interface document to a user-selected path. This keeps generated documentation aligned with both handwritten host modules and macro-generated host modules.

## Scope

In scope:

- deterministic Markdown rendering for one or more `ScriptHostModuleDescriptor` values;
- file writing through a small runtime-owned helper or tool entry point;
- modular Rust source organization for rendering, sorting, formatting, and writing responsibilities;
- tests that prove the generated document includes module, capability, type, field, function, parameter, and return information;
- documentation updates under `docs/zircon_runtime/script/vm/` and, where backend behavior is affected, `docs/zircon_plugins/zr_vm_language/`.

Out of scope for the first implementation:

- macro-expansion-time documentation file generation;
- exposing Rust object pointers or runtime object references to VM code;
- changing the ZrVM backend registration ABI;
- documenting ECS scene reflection or editor inspector reflection, which are separate reflection domains;
- workspace-wide validation claims while unrelated graphics/UI work blocks compilation.

## Chosen Approach

Use a runtime descriptor-driven documentation pipeline.

`ScriptHostModuleDescriptor` remains the single source of truth. The renderer consumes descriptors after they have been produced by handwritten code, reflection macros, or future plugin registration paths. This avoids duplicating reflection logic in the procedural macro crate and guarantees the generated docs match the data registered with a VM backend.

The writer is intentionally small. It should not own reflection semantics; it should only gather descriptors, call the renderer, and write the output. The first writer target is built-in ZrVM host interface documentation, because built-ins already exercise both handwritten modules and the macro-generated math module.

## Alternatives Considered

### API Only

This would add the Markdown renderer without a file-writing path. It is lower risk, but it does not satisfy the requirement to generate reflection interface documents as files.

### Macro-Generated Documentation

This would make `zircon_runtime_reflection_macros` emit documentation artifacts beside reflected modules, types, and functions. It was rejected for the first implementation because it only covers macro-authored exports, duplicates descriptor formatting logic, and makes the procedural macro crate responsible for behavior that belongs to runtime reflection tooling.

### API Plus Writer

This is the selected approach. It covers handwritten and macro-generated descriptors, keeps reflection semantics in runtime descriptors, provides a concrete file-generation path, and can feed export tooling in a separate follow-up.

## Architecture

Add a folder-backed documentation module under the runtime script/VM host reflection area instead of expanding root wiring files.

The target shape is:

```text
zircon_runtime/src/script/vm/host/reflection_docs/
  mod.rs
  markdown.rs
  options.rs
  writer.rs
```

Responsibilities:

- `mod.rs` exposes the small public surface and remains structural.
- `options.rs` defines renderer options such as title, heading depth, and optional capability display.
- `markdown.rs` renders descriptor data into deterministic Markdown.
- `writer.rs` writes rendered documents to disk and owns filesystem errors.

The first implementation uses the VM host subsystem path because the writer documents VM host modules. Moving the module to `zircon_runtime/src/core/framework/script/reflection_docs/` requires a separate implementation-plan decision that keeps `core::framework::script` as neutral DTO territory and does not turn `script.rs` into an implementation file.

## Public Surface

The first API should stay narrow:

```rust
pub struct ScriptHostInterfaceMarkdownOptions {
    pub title: String,
    pub heading_level: usize,
    pub include_capabilities: bool,
    pub include_empty_sections: bool,
}

pub fn render_script_host_modules_markdown(
    modules: &[ScriptHostModuleDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) -> String;

pub fn write_script_host_modules_markdown(
    path: impl AsRef<Path>,
    modules: &[ScriptHostModuleDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) -> std::io::Result<()>;
```

The writer or tool may add a built-in helper such as `write_builtin_host_interface_markdown` if it materially simplifies usage. That helper should still call the generic renderer and writer.

## Markdown Contract

The generated Markdown must be deterministic so tests and docs diffs remain stable.

Ordering rules:

- modules sorted by module name;
- capabilities sorted lexicographically;
- types sorted by type name;
- fields kept in descriptor order, because field order is part of the reflected type surface;
- functions sorted by function name;
- parameters kept in descriptor order, because argument order is part of the call contract.

Each module section includes:

- module name and version;
- module documentation, when present;
- declared capabilities;
- type descriptors with prototype kind, value kind, type ref, value-construction flag, fields, and field docs;
- function descriptors with parameter list, return type ref, required capabilities, and function docs.

The renderer must display the VM-facing type name from `ScriptHostTypeRef::type_name`, not Rust type names or only `ScriptHostValueKind`. This preserves the current type-ref convergence where `f64` exports as `float`, and semantic host types such as `Vec3` remain visible as `Vec3`.

## Writer Contract

The writer must be explicit and side-effect limited:

- it writes only to the requested path;
- it creates the parent directory when needed;
- it overwrites the requested file through normal create/truncate file writing and does not need a durable transactional protocol;
- it returns `std::io::Result<()>` without converting filesystem failures into VM errors;
- it does not register host modules itself unless called through a built-in-specific helper or tool.

The checked-in tool must use repository-standard paths, keep machine-specific output paths out of committed files, and require an explicit output path in the first implementation.

## Macro Crate Modularity

The proc-macro crate is currently a single implementation file. The documentation feature should not add more behavior there. If macro crate changes are required during implementation, split by responsibility first:

- entry points and public proc-macro functions;
- argument parsing;
- type derive expansion;
- host function expansion;
- host module expansion;
- shared token helpers;
- macro tests.

The macro crate should continue to emit descriptors only. Documentation generation remains descriptor-driven in runtime code.

## Error Handling

Renderer functions should be infallible because descriptors are already validated by `HostExportRegistry` before normal use. Tests may still call the renderer with synthetic descriptors, so the renderer should be defensive about empty optional docs but should not reimplement registry validation.

Filesystem errors belong to the writer and should surface as `std::io::Error`. The writer should not swallow missing parent-directory, permission, or path-type failures.

## Testing Plan

Required focused tests:

- render a synthetic descriptor fixture and assert stable module/function/type text;
- render built-in modules and assert the macro-generated math interface includes `Vec3`, `ColorRgba`, `vec3_length`, VM-facing `float`, and required return/parameter details;
- writer test writes to a temporary path and verifies the file contains deterministic content;
- existing reflection macro tests continue to pass.

Validation commands for the milestone testing stage:

- `rustfmt --edition 2021 --check` over touched Rust files;
- `cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macros`;
- `cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macros`, if unrelated graphics/UI blockers allow it;
- focused `cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macros`, if unrelated graphics/UI blockers allow it.

If scoped runtime validation is blocked by unrelated compile errors, record the exact blocker and do not claim runtime or workspace green.

## Documentation Plan

Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` to describe:

- modular descriptor-driven documentation generation;
- the Markdown contract and sorting rules;
- how the writer is invoked;
- validation evidence and any blocked commands.

Update `docs/zircon_plugins/zr_vm_language/runtime.md` only if generated docs affect backend-facing type/module documentation behavior. Keep both documents' machine-readable headers current.

## Acceptance Criteria

The implementation is accepted when:

- host reflection documentation can be generated from descriptors without using proc-macro internals;
- the writer can write deterministic Markdown to a chosen path;
- the built-in math module documentation proves macro-generated reflection participates in the same pipeline as handwritten modules;
- root wiring files stay structural and new behavior lives in focused modules;
- docs are updated with implementation files, tests, and known validation blockers;
- no workspace-wide success is claimed unless the corresponding commands actually pass.

# ZrVM Host Reflection Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build descriptor-driven ZrVM host reflection interface documentation rendering plus an explicit-output writer command.

**Architecture:** Keep `ScriptHostModuleDescriptor` as the source of truth and add a focused `zircon_runtime::script::vm::host::reflection_docs` subtree for options, Markdown rendering, filesystem writing, and built-in module collection. Keep root wiring files structural and expose only narrow public functions through existing VM host re-exports.

**Tech Stack:** Rust 2021, existing `zircon_runtime` library, existing `zircon_runtime_reflection_macros` crate, standard-library Markdown/string/file APIs, scoped Cargo validation with `--locked --offline --jobs 1` where possible.

---

## Current Baseline

- Repository policy requires staying in the existing `main` checkout; do not create branches or worktrees.
- `zircon_runtime/src/core/framework/script.rs` already defines descriptor DTOs: `ScriptHostModuleDescriptor`, `ScriptHostTypeDescriptor`, `ScriptHostFunctionDescriptor`, `ScriptHostParameterDescriptor`, `ScriptHostFieldDescriptor`, and `ScriptHostTypeRef`.
- `HostExportRegistry` already validates descriptors and exposes sorted registered modules through `modules()`.
- Built-in VM host modules live in `zircon_runtime/src/script/vm/host/builtin_host_modules.rs`; most are handwritten, while `zr.zircon.math` is macro-generated and proves the macro path emits the same descriptor model.
- The macro crate is currently monolithic. This plan does not add behavior there; do not edit `zircon_runtime/reflection_macros/src/lib.rs` for this feature.
- Runtime scoped validation is known to be vulnerable to unrelated graphics/UI compile blockers. Record exact blockers and do not claim workspace green unless the corresponding command passes.

## File Structure

Create:

- `zircon_runtime/src/script/vm/host/reflection_docs/mod.rs`: structural module, public re-exports, and built-in descriptor collection helper.
- `zircon_runtime/src/script/vm/host/reflection_docs/options.rs`: `ScriptHostInterfaceMarkdownOptions` and defaults.
- `zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs`: deterministic Markdown renderer from descriptor slices.
- `zircon_runtime/src/script/vm/host/reflection_docs/writer.rs`: filesystem writer that creates parent directories and writes rendered Markdown.
- `zircon_runtime/src/bin/zircon_host_reflection_docs.rs`: explicit-output command that registers built-in host modules and writes their interface document.

Modify:

- `zircon_runtime/Cargo.toml`: add an explicit `[[bin]]` target for `zircon_host_reflection_docs` so the writer command is stable even if Cargo auto-discovery settings change.
- `zircon_runtime/src/script/vm/host/mod.rs`: add `mod reflection_docs;` and curated public re-exports only.
- `zircon_runtime/src/script/vm/mod.rs`: re-export the public reflection-doc functions and options through the existing VM public surface.
- `zircon_runtime/src/script/mod.rs`: re-export the same public reflection-doc functions for `zircon_runtime::script::*` consumers.
- `zircon_runtime/src/script/vm/tests.rs`: add focused unit coverage for synthetic rendering, built-in math rendering, and writer output.
- `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`: document the new descriptor-driven docs pipeline, writer command, tests, and blockers.
- `.codex/sessions/20260518-0229-zrvm-host-reflection-docs.md`: update live coordination status during implementation and validation; delete or archive it on completion.

Do not modify unless implementation proves it is required:

- `zircon_runtime/src/core/framework/script.rs`: descriptor model is already sufficient.
- `zircon_runtime/reflection_macros/src/lib.rs`: macro output remains descriptor-only.
- `zircon_plugins/zr_vm_language/runtime/src/real_backend.rs`: backend behavior is not changing.

## Milestone 1: Descriptor Markdown API

### Goal

Add deterministic Markdown rendering for `ScriptHostModuleDescriptor` slices without filesystem side effects.

### In-Scope Behaviors

- Render a document title.
- Sort modules by name.
- Sort capabilities by string.
- Sort types by type name while preserving each type's field order.
- Sort functions by function name while preserving each function's parameter order.
- Display VM-facing `ScriptHostTypeRef::type_name` for fields, parameters, and return values.
- Display value kind, prototype kind, value-construction flag, docs, and required capabilities.

### Dependencies

- Existing descriptor DTOs in `zircon_runtime/src/core/framework/script.rs`.
- Existing registry/macro-generated built-in descriptors.

### Implementation Slices

- [ ] Create `zircon_runtime/src/script/vm/host/reflection_docs/options.rs` with this shape:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptHostInterfaceMarkdownOptions {
    pub title: String,
    pub heading_level: usize,
    pub include_capabilities: bool,
    pub include_empty_sections: bool,
}

impl Default for ScriptHostInterfaceMarkdownOptions {
    fn default() -> Self {
        Self {
            title: "ZrVM Host Interface".to_string(),
            heading_level: 1,
            include_capabilities: true,
            include_empty_sections: false,
        }
    }
}
```

- [ ] Create `zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs` with `render_script_host_modules_markdown`. Use helper functions inside this file for headings, kind formatting, type refs, docs, capabilities, types, fields, functions, and parameters. Keep all renderer helpers private.

```rust
use crate::core::framework::script::{
    ScriptHostFieldDescriptor, ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor,
    ScriptHostParameterDescriptor, ScriptHostTypeDescriptor, ScriptHostTypeRef,
};

use super::ScriptHostInterfaceMarkdownOptions;

pub fn render_script_host_modules_markdown(
    modules: &[ScriptHostModuleDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) -> String {
    let mut output = String::new();
    push_heading(&mut output, options.heading_level, &options.title);
    output.push('\n');

    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| left.name.cmp(&right.name));

    for module in modules {
        render_module(&mut output, module, options);
    }

    output
}
```

- [ ] Use this exact Markdown shape so tests have a stable contract:

```markdown
# ZrVM Host Interface

## Module `example.alpha`

- Version: `0.1.0`
- Documentation: Example module docs.
- Capabilities: `alpha.read`, `alpha.write`

### Types

#### Type `Vec3`

- Type ref: `Vec3` (`float`)
- Prototype: `struct`
- Value construction: `true`
- Documentation: Vec3 docs.

Fields:

- `x`: `float` (`float`) - X axis.

### Functions

#### Function `length`

- Return: `float` (`float`)
- Required capabilities: `alpha.read`
- Documentation: Return vector length.

Parameters:

- `x`: `float` (`float`)
```

- [ ] Add `zircon_runtime/src/script/vm/host/reflection_docs/mod.rs` as structural wiring:

```rust
mod markdown;
mod options;

pub use markdown::render_script_host_modules_markdown;
pub use options::ScriptHostInterfaceMarkdownOptions;
```

- [ ] Modify `zircon_runtime/src/script/vm/host/mod.rs` to include and re-export the renderer and options:

```rust
mod reflection_docs;

pub use reflection_docs::{
    render_script_host_modules_markdown, ScriptHostInterfaceMarkdownOptions,
};
```

- [ ] Modify `zircon_runtime/src/script/vm/mod.rs` and `zircon_runtime/src/script/mod.rs` to add the same public re-exports through their existing `pub use` lists. Keep both files as wiring only.

- [ ] Add synthetic renderer coverage to `zircon_runtime/src/script/vm/tests.rs` inside the existing `#[cfg(test)] mod tests`. Extend the current grouped `use super::super` import block so it includes `builtin_host_module_descriptors`, `render_script_host_modules_markdown`, `write_script_host_modules_markdown`, and `ScriptHostInterfaceMarkdownOptions` beside the existing VM test imports. Add this test body:

```rust
#[test]
fn host_reflection_docs_render_synthetic_descriptor_deterministically() {
    let descriptor = ScriptHostModuleDescriptor::new("example.alpha", "0.1.0")
        .with_capability("alpha.write")
        .with_capability("alpha.read")
        .with_type(
            ScriptHostTypeDescriptor::new("Vec3", ScriptHostValueKind::Float)
                .with_prototype_kind(ScriptHostPrototypeKind::Struct)
                .allow_value_construction(true)
                .with_field(
                    crate::core::framework::script::ScriptHostFieldDescriptor::new(
                        "x",
                        ScriptHostValueKind::Float,
                    )
                    .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float"))
                    .with_documentation("X axis."),
                )
                .with_documentation("Vec3 docs."),
        )
        .with_function(
            ScriptHostFunctionDescriptor::new("length", 1, 1, ScriptHostValueKind::Float)
                .with_return_type(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float"))
                .with_required_capability("alpha.read")
                .with_parameter(
                    ScriptHostParameterDescriptor::new("x", ScriptHostValueKind::Float)
                        .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "float")),
                )
                .with_documentation("Return vector length."),
        )
        .with_documentation("Example module docs.");

    let markdown = render_script_host_modules_markdown(
        &[descriptor],
        &ScriptHostInterfaceMarkdownOptions::default(),
    );

    assert!(markdown.contains("# ZrVM Host Interface"));
    assert!(markdown.contains("## Module `example.alpha`"));
    assert!(markdown.contains("- Capabilities: `alpha.read`, `alpha.write`"));
    assert!(markdown.contains("#### Type `Vec3`"));
    assert!(markdown.contains("- Type ref: `Vec3` (`float`)"));
    assert!(markdown.contains("- `x`: `float` (`float`) - X axis."));
    assert!(markdown.contains("#### Function `length`"));
    assert!(markdown.contains("- Return: `float` (`float`)"));
    assert!(markdown.contains("- `x`: `float` (`float`)"));
}
```

### Lightweight Checks

- Run `rustfmt --edition 2021 --check` on the new reflection-doc files and `zircon_runtime/src/script/vm/tests.rs` if editing syntax gets large enough to need an early sanity check.

### Testing Stage

- Run `rustfmt --edition 2021 --check zircon_runtime/src/script/vm/host/reflection_docs/mod.rs zircon_runtime/src/script/vm/host/reflection_docs/options.rs zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs zircon_runtime/src/script/vm/host/mod.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs`.
- Run `cargo test -p zircon_runtime host_reflection_docs_render_synthetic_descriptor_deterministically --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- --nocapture --test-threads=1`. When this fails before the focused test because of unrelated compile errors, record the first unrelated file, line, and error code in the session note and docs, then continue with code/documentation work that does not touch the unrelated area.

### Exit Evidence

- The synthetic rendering test exists and either passes or has a recorded unrelated compile blocker.
- The renderer API is reachable through `zircon_runtime::script` without making root files own behavior.

## Milestone 2: Writer API And Built-In Descriptor Collection

### Goal

Add filesystem writing and prove built-in handwritten and macro-generated modules render through the same descriptor pipeline.

### In-Scope Behaviors

- Write Markdown to an explicit path.
- Create parent directories when they do not exist.
- Return `std::io::Result<()>` for filesystem failures.
- Collect built-in host module descriptors by registering built-ins into a temporary `HostExportRegistry`.
- Prove the generated Markdown includes the macro-generated math module.

### Dependencies

- Milestone 1 renderer API.
- Existing `register_builtin_host_modules`, `HostExportRegistry`, and `HostRegistry`.

### Implementation Slices

- [ ] Create `zircon_runtime/src/script/vm/host/reflection_docs/writer.rs`:

```rust
use std::fs;
use std::io;
use std::path::Path;

use crate::core::framework::script::ScriptHostModuleDescriptor;

use super::{render_script_host_modules_markdown, ScriptHostInterfaceMarkdownOptions};

pub fn write_script_host_modules_markdown(
    path: impl AsRef<Path>,
    modules: &[ScriptHostModuleDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_script_host_modules_markdown(modules, options))
}
```

- [ ] Extend `reflection_docs/mod.rs` to include the writer and a built-in collection helper:

```rust
mod markdown;
mod options;
mod writer;

pub use markdown::render_script_host_modules_markdown;
pub use options::ScriptHostInterfaceMarkdownOptions;
pub use writer::write_script_host_modules_markdown;

use super::{register_builtin_host_modules, HostExportRegistry, HostRegistry};
use crate::core::framework::script::ScriptHostModuleDescriptor;
use crate::script::VmError;

pub fn builtin_host_module_descriptors() -> Result<Vec<ScriptHostModuleDescriptor>, VmError> {
    let registry = HostRegistry::default();
    let exports = HostExportRegistry::new(registry.clone());
    register_builtin_host_modules(&exports, &registry)?;
    Ok(exports
        .modules()
        .into_iter()
        .map(|record| record.descriptor)
        .collect())
}
```

- [ ] Re-export `write_script_host_modules_markdown` and `builtin_host_module_descriptors` through `host/mod.rs`, `vm/mod.rs`, and `script/mod.rs`.

- [ ] Add a writer test to `zircon_runtime/src/script/vm/tests.rs`. Use a unique temp directory to avoid collisions:

```rust
#[test]
fn host_reflection_docs_writer_creates_parent_directory_and_file() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon-host-reflection-docs-{nonce}"));
    let output_path = root.join("nested").join("host-interface.md");
    let descriptor = ScriptHostModuleDescriptor::new("writer.example", "0.1.0")
        .with_documentation("Writer example.");

    write_script_host_modules_markdown(
        &output_path,
        &[descriptor],
        &ScriptHostInterfaceMarkdownOptions::default(),
    )
    .unwrap();

    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("## Module `writer.example`"));
    assert!(contents.contains("Writer example."));

    fs::remove_dir_all(root).unwrap();
}
```

- [ ] Add built-in math coverage to `zircon_runtime/src/script/vm/tests.rs`:

```rust
#[test]
fn host_reflection_docs_include_macro_generated_builtin_math_module() {
    let modules = builtin_host_module_descriptors().unwrap();
    let markdown = render_script_host_modules_markdown(
        &modules,
        &ScriptHostInterfaceMarkdownOptions::default(),
    );

    assert!(markdown.contains("## Module `zr.zircon.math`"));
    assert!(markdown.contains("#### Type `Vec3`"));
    assert!(markdown.contains("#### Type `ColorRgba`"));
    assert!(markdown.contains("#### Function `vec3_length`"));
    assert!(markdown.contains("#### Function `vec3_dot`"));
    assert!(markdown.contains("- Return: `float` (`float`)"));
    assert!(markdown.contains("- `x`: `float` (`float`)"));
}
```

### Lightweight Checks

- Run `rustfmt --edition 2021 --check` on touched reflection-doc files and `zircon_runtime/src/script/vm/tests.rs` after adding writer tests.

### Testing Stage

- Run `rustfmt --edition 2021 --check zircon_runtime/src/script/vm/host/reflection_docs/mod.rs zircon_runtime/src/script/vm/host/reflection_docs/options.rs zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs zircon_runtime/src/script/vm/host/reflection_docs/writer.rs zircon_runtime/src/script/vm/host/mod.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs`.
- Run `cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- --nocapture --test-threads=1`. When this fails before the focused tests because of unrelated compile errors, record the first unrelated file, line, and error code in the session note and docs.
- Run `cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs`. When this fails because of unrelated compile errors, record the first unrelated file, line, and error code in the session note and docs.

### Exit Evidence

- Writer test exists and either passes or has a recorded unrelated compile blocker.
- Built-in math docs test exists and either passes or has a recorded unrelated compile blocker.
- New API does not mutate registry state outside local descriptor collection.

## Milestone 3: Explicit Writer Command

### Goal

Add a small runtime-owned command that writes built-in host reflection documentation to a caller-supplied path.

### In-Scope Behaviors

- Require exactly one output path argument.
- Print usage and return non-zero on missing or extra arguments.
- Register built-in host modules, render descriptors, and write Markdown to the requested path.
- Keep machine-specific output paths out of source and docs.

### Dependencies

- Milestone 2 writer and built-in descriptor helper.

### Implementation Slices

- [ ] Add `zircon_runtime/src/bin/zircon_host_reflection_docs.rs`:

```rust
use std::path::PathBuf;

use zircon_runtime::script::{
    builtin_host_module_descriptors, write_script_host_modules_markdown,
    ScriptHostInterfaceMarkdownOptions,
};

fn main() {
    if let Err(error) = run(std::env::args().skip(1)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let mut args = args.into_iter();
    let output = args
        .next()
        .ok_or_else(|| usage("missing output path"))
        .map(PathBuf::from)?;
    if args.next().is_some() {
        return Err(usage("expected exactly one output path"));
    }

    let modules = builtin_host_module_descriptors()
        .map_err(|error| format!("failed to collect built-in host modules: {error}"))?;
    write_script_host_modules_markdown(
        output,
        &modules,
        &ScriptHostInterfaceMarkdownOptions::default(),
    )
    .map_err(|error| format!("failed to write host interface docs: {error}"))
}

fn usage(message: &str) -> String {
    format!("{message}\nusage: zircon_host_reflection_docs <output-markdown-path>")
}
```

- [ ] Add this explicit `[[bin]]` section to `zircon_runtime/Cargo.toml`:

```toml
[[bin]]
name = "zircon_host_reflection_docs"
path = "src/bin/zircon_host_reflection_docs.rs"
```

- [ ] Keep this bin free of `clap` or new dependencies. The command has one argument and can use `std::env::args`.

### Lightweight Checks

- Run `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_host_reflection_docs.rs` after creating the file.

### Testing Stage

- Run `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_host_reflection_docs.rs`.
- Run `cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- F:\cargo-targets\codex-reflection-docs\host-interface.md`. When this fails because of unrelated compile errors, record the first unrelated file, line, and error code in the session note and docs.
- Inspect the generated file path with `Test-Path -LiteralPath "F:\cargo-targets\codex-reflection-docs\host-interface.md"` after a successful command run. Confirm generated content contains `zr.zircon.math` using `rg "zr\.zircon\.math" "F:\cargo-targets\codex-reflection-docs\host-interface.md"`.

### Exit Evidence

- Command exists and either writes the file successfully or has a recorded unrelated compile blocker.
- The command requires an explicit output path and has no committed machine-specific output path.

## Milestone 4: Documentation And Acceptance Evidence

### Goal

Update repository docs and live coordination notes to describe the new docs pipeline, writer command, validation evidence, and any blockers.

### In-Scope Behaviors

- Keep module docs under `docs/zircon_runtime/script/vm/` current.
- Keep machine-readable headers updated with new implementation files and tests.
- Record scoped validation results and blockers precisely.
- Retire the active coordination note if no handoff is needed.

### Dependencies

- Milestones 1 through 3 implementation.

### Implementation Slices

- [ ] Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` header `related_code` and `implementation_files` with:

```yaml
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/options.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/writer.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
```

- [ ] Add a body section named `Generated Interface Documentation` to `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` covering descriptor source of truth, sorting rules, writer command, and the built-in math proof.

- [ ] Add validation evidence lines for every command run. Use exact status wording:

```markdown
- "cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs: passed 2026-05-18; synthetic, writer, and built-in math docs tests passed"
```

If blocked, write the exact blocker instead of a pass claim:

```markdown
- "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs: attempted 2026-05-18; blocked by unrelated zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs:142 E0061 missing ScreenSpaceUiRenderer argument"
```

- [ ] Update `.codex/sessions/20260518-0229-zrvm-host-reflection-docs.md` with final touched files, validation status, blockers, and next step. Delete it before final closeout if no other session needs the handoff; archive it only if a blocker remains useful to other sessions.

### Lightweight Checks

- Run scoped `git diff --check -- "docs/zircon_runtime/script/vm/zr_vm_host_reflection.md" ".codex/sessions/20260518-0229-zrvm-host-reflection-docs.md" "docs/superpowers/plans/2026-05-18-zrvm-host-reflection-docs.md"` before closeout.

### Testing Stage

- Run all validation commands from Milestones 1 through 3 that have not already been run in this implementation session.
- Run `cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs` to prove macro crate tests still pass.
- Run `cargo fmt --all --check` only if unrelated formatting blockers are resolved; otherwise run scoped `rustfmt --edition 2021 --check` over touched Rust files and document the known workspace fmt blockers.

### Exit Evidence

- Documentation includes the new implementation files and generated-doc behavior.
- Acceptance evidence distinguishes passed scoped checks from unrelated blockers.
- Active session note is deleted or archived according to coordination rules.

## Final Validation Matrix

Use serial commands and one target directory to avoid contention:

```powershell
$env:CARGO_TARGET_DIR = "F:\cargo-targets\codex-reflection-docs"
rustfmt --edition 2021 --check zircon_runtime/src/script/vm/host/reflection_docs/mod.rs zircon_runtime/src/script/vm/host/reflection_docs/options.rs zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs zircon_runtime/src/script/vm/host/reflection_docs/writer.rs zircon_runtime/src/script/vm/host/mod.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/src/bin/zircon_host_reflection_docs.rs
cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs
cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- --nocapture --test-threads=1
cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs
cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- F:\cargo-targets\codex-reflection-docs\host-interface.md
```

If disk free space on `F:` is `<= 50 GB`, run `cargo clean --target-dir F:\cargo-targets\codex-reflection-docs` before Cargo validation. Do not run multiple heavy Cargo commands in parallel against this target directory.

## Completion Criteria

- Renderer API, writer API, built-in descriptor helper, and writer command are implemented.
- Synthetic, writer, and built-in math documentation tests are added.
- Generated Markdown shows VM-facing type names and deterministic ordering.
- Root wiring files remain structural.
- Docs and session evidence are updated.
- Scoped validation passes or unrelated blockers are recorded with exact evidence.

# ZrVM Reflection Macro Modularity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the ZrVM reflection proc-macro crate into focused modules and harden macro-owned descriptor metadata tests without changing the runtime host reflection contract.

**Architecture:** Keep `zircon_runtime_reflection_macros` as descriptor-generation only. `lib.rs` stays limited to required proc-macro entry points and structural module wiring, while parsing, attribute scanning, type derive expansion, function expansion, module expansion, token helpers, and tests move into focused files. Runtime registry validation, host handles, capabilities, backend translation, and Markdown docs remain in `zircon_runtime` and `zircon_plugins` unchanged except for documentation updates.

**Tech Stack:** Rust 2021, `proc_macro`, `proc_macro2`, `quote`, `syn`, existing `zircon_runtime::core::framework::script` descriptor DTOs, scoped Cargo validation with `--locked --offline --jobs 1` when practical.

**Completion Status:** Milestones 1 and 2 are implemented and scoped validation passed on 2026-05-20 with `F:` free space at 87.44 GB, so no target cleanup was required. Milestone 3 documentation is updated in `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`, docs sanity checks passed, the active session note was retired, and closeout validation passed again on 2026-05-21 with `F:` free space at 66.86 GB, so no target cleanup was required.

---

## Current Baseline

- Work in the existing `main` checkout. Do not create a worktree or branch.
- `zircon_runtime/reflection_macros/src/lib.rs` currently owns proc-macro entry points, argument parsing, `#[zircon_script]` attribute parsing, derive expansion, function expansion, module expansion, token helpers, and unit tests in one file.
- Public macros are already re-exported by `zircon_runtime` and used by `zircon_runtime/src/script/vm/host/builtin_host_modules.rs` for `zr.zircon.math`.
- Runtime tests in `zircon_runtime/src/script/vm/tests.rs` already prove generated descriptors and built-in math docs participate in the same descriptor path.
- This plan does not add JSON descriptor export, new host modules, backend ABI changes, or manager-backed host behavior.
- Active coordination note for this task: `.codex/sessions/20260520-0416-zrvm-host-reflection-followup.md`. Update it before implementation and validation stages, then delete it on clean completion if no handoff is needed.

## File Structure

Create:

- `zircon_runtime/reflection_macros/src/args.rs`: `ScriptTypeArgs`, `FieldArgs`, `HostFunctionArgs`, `HostModuleArgs`, `Parse` impls, and literal/path parser helpers used by argument parsing.
- `zircon_runtime/reflection_macros/src/attrs.rs`: `parse_script_type_attrs`, `parse_field_attrs`, `host_attr_function_ident`, `script_type_ident`, derive/attribute detection helpers.
- `zircon_runtime/reflection_macros/src/derive_type.rs`: `derive_zircon_script_type_impl` and field descriptor token generation.
- `zircon_runtime/reflection_macros/src/function.rs`: `host_function_impl`, `HostParam`, and host function parameter extraction.
- `zircon_runtime/reflection_macros/src/module.rs`: `host_module_impl`.
- `zircon_runtime/reflection_macros/src/tokens.rs`: `path_tokens` and `script_host_type_ref_tokens`.
- `zircon_runtime/reflection_macros/src/tests.rs`: macro crate unit tests moved out of `lib.rs`, plus added unsupported-input and metadata-generation tests.

Modify:

- `zircon_runtime/reflection_macros/src/lib.rs`: reduce to module declarations, proc-macro entry points, and narrow calls into implementation modules.
- `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`: update machine-readable header, macro crate structure narrative, unsupported input notes, and validation evidence.
- `.codex/sessions/20260520-0416-zrvm-host-reflection-followup.md`: update active status during implementation and validation; delete on completion if no handoff is needed.

Do not modify unless a compile error proves it is necessary:

- `zircon_runtime/src/core/framework/script.rs`: descriptor DTOs are sufficient.
- `zircon_runtime/src/script/vm/host/host_export_registry.rs`: runtime descriptor validation remains unchanged.
- `zircon_runtime/src/script/vm/host/builtin_host_modules.rs`: built-in math should continue compiling unchanged.
- `zircon_plugins/zr_vm_language/runtime/src/real_backend.rs`: backend descriptor translation is not in scope.

## Milestone 1: Macro Crate Structural Cutover

### Goal

Move existing macro parsing, expansion, helper, and test code into focused files while preserving generated descriptor semantics.

### In-Scope Behaviors

- `ZirconScriptType` macro entry point still accepts the same input and emits the same descriptor-building implementation.
- `zircon_host_function` macro entry point still accepts the same input and emits the same descriptor and callback helpers.
- `zircon_host_module` macro entry point still accepts the same input and emits the same module descriptor and registration helpers.
- Existing unsupported input tests still compile and pass.
- `lib.rs` remains the only file containing `#[proc_macro_derive]` and `#[proc_macro_attribute]` functions.

### Dependencies

- Existing `zircon_runtime/reflection_macros/src/lib.rs` implementation.
- Existing runtime descriptor DTO paths under `::zircon_runtime::core::framework::script`.

### Implementation Slices

- [x] **Update active session note before edits.** Change `.codex/sessions/20260520-0416-zrvm-host-reflection-followup.md` to `status: active-implementing-macro-cutover`, set a fresh `updated_at`, and note that edits are limited to `zircon_runtime/reflection_macros/src/*` until Milestone 1 completes.

- [x] **Create `zircon_runtime/reflection_macros/src/args.rs`.** Move these existing definitions and parser helpers from `lib.rs` without changing behavior: `ScriptTypeArgs`, `FieldArgs`, `HostFunctionArgs`, `HostModuleArgs`, `impl Parse for HostFunctionArgs`, `impl Parse for HostModuleArgs`, `parse_key_values`, `parse_lit_string`, and `parse_expr_path`. Mark items `pub(crate)` only where another module needs them.

```rust
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Ident, LitBool, LitStr, Path, Token};

#[derive(Default)]
pub(crate) struct ScriptTypeArgs {
    pub(crate) name: Option<String>,
    pub(crate) value_kind: Option<Path>,
    pub(crate) prototype: Option<Path>,
    pub(crate) allow_value_construction: Option<bool>,
    pub(crate) documentation: Option<String>,
}

#[derive(Default)]
pub(crate) struct FieldArgs {
    pub(crate) name: Option<String>,
    pub(crate) type_name: Option<String>,
    pub(crate) value_kind: Option<Path>,
    pub(crate) documentation: Option<String>,
    pub(crate) skip: bool,
}

#[derive(Default)]
pub(crate) struct HostFunctionArgs {
    pub(crate) name: Option<String>,
    pub(crate) return_type_name: Option<String>,
    pub(crate) return_value_kind: Option<Path>,
    pub(crate) capability: Vec<String>,
    pub(crate) documentation: Option<String>,
}

#[derive(Default)]
pub(crate) struct HostModuleArgs {
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) capability: Vec<String>,
    pub(crate) documentation: Option<String>,
}
```

- [x] **Create `zircon_runtime/reflection_macros/src/tokens.rs`.** Move `path_tokens` and `script_host_type_ref_tokens` unchanged, with `pub(crate)` visibility.

```rust
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Path, Type};

pub(crate) fn path_tokens(path: Path) -> TokenStream2 {
    quote!(#path)
}

pub(crate) fn script_host_type_ref_tokens(
    ty: &Type,
    value_kind: Option<TokenStream2>,
    type_name: Option<String>,
    trait_path: TokenStream2,
) -> TokenStream2 {
    match (value_kind, type_name) {
        (Some(value_kind), Some(type_name)) => quote! {
            ::zircon_runtime::core::framework::script::ScriptHostTypeRef::new(#value_kind, #type_name)
        },
        (Some(value_kind), None) => quote! {{
            let mut type_ref = <#ty as #trait_path>::script_host_type_ref();
            type_ref.value_kind = #value_kind;
            type_ref
        }},
        (None, Some(type_name)) => quote! {{
            let mut type_ref = <#ty as #trait_path>::script_host_type_ref();
            type_ref.type_name = #type_name.to_string();
            type_ref
        }},
        (None, None) => quote! {
            <#ty as #trait_path>::script_host_type_ref()
        },
    }
}
```

- [x] **Create `zircon_runtime/reflection_macros/src/attrs.rs`.** Move `parse_script_type_attrs`, `parse_field_attrs`, `host_attr_function_ident`, `script_type_ident`, `has_attr_with_last_segment`, `path_last_segment_is`, and `derives_zircon_script_type`. Keep `parse_script_type_attrs`, `parse_field_attrs`, `host_attr_function_ident`, and `script_type_ident` `pub(crate)`.

- [x] **Create `zircon_runtime/reflection_macros/src/derive_type.rs`.** Move `derive_zircon_script_type_impl` and `field_descriptor_tokens`. Import helpers explicitly:

```rust
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

use crate::attrs::{parse_field_attrs, parse_script_type_attrs};
use crate::tokens::{path_tokens, script_host_type_ref_tokens};
```

Keep `derive_zircon_script_type_impl` as `pub(crate)`. Keep `field_descriptor_tokens` private. The moved body must still reject generic parameters and unions, default struct prototype to `ScriptHostPrototypeKind::Struct`, default enum prototype to `ScriptHostPrototypeKind::Enum`, emit `.with_type_ref(ScriptHostTypeRef::new(#value_kind, #type_name))`, and append generated field descriptor tokens plus optional documentation exactly as the current implementation does.

- [x] **Create `zircon_runtime/reflection_macros/src/function.rs`.** Move `host_function_impl`, `HostParam`, and `host_function_params`. Import helpers explicitly:

```rust
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{FnArg, Ident, ItemFn, Pat, ReturnType, Type};

use crate::args::HostFunctionArgs;
use crate::tokens::{path_tokens, script_host_type_ref_tokens};
```

Keep `host_function_impl` as `pub(crate)`. Keep `HostParam` and `host_function_params` private. The moved body must still reject async functions, generic functions, methods, and non-identifier parameters; generate `__zircon_host_function_descriptor_<fn>` and `__zircon_host_export_function_<fn>` helper names; derive parameter and return `ScriptHostTypeRef` values from the conversion traits; preserve required capabilities and documentation; and call through `ScriptHostFromValue`/`ScriptHostIntoValue` exactly as the current implementation does.

- [x] **Create `zircon_runtime/reflection_macros/src/module.rs`.** Move `host_module_impl`. Import helpers explicitly:

```rust
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::ItemMod;

use crate::args::HostModuleArgs;
use crate::attrs::{host_attr_function_ident, script_type_ident};
```

Keep `host_module_impl` as `pub(crate)`. The moved body must still require `name = "..."`, default version to `0.1.0`, reject non-inline modules, discover functions carrying a `zircon_host_function` attribute, discover structs/enums deriving `ZirconScriptType`, generate `<module>_host_module_descriptor`, and generate `register_<module>_host_module` that calls `HostExportRegistry::register_module` with generated function exports.

- [x] **Move existing tests into `zircon_runtime/reflection_macros/src/tests.rs`.** Add `#[cfg(test)] mod tests;` to `lib.rs`. In `tests.rs`, import implementation helpers through `crate::derive_type::derive_zircon_script_type_impl` and `crate::function::host_function_impl`, plus `crate::args::HostFunctionArgs`.

- [x] **Reduce `zircon_runtime/reflection_macros/src/lib.rs` to structural wiring and entry points.** Keep the public proc-macro functions and move all helper definitions out.

```rust
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod args;
mod attrs;
mod derive_type;
mod function;
mod module;
mod tokens;

#[cfg(test)]
mod tests;

#[proc_macro_derive(ZirconScriptType, attributes(zircon_script))]
pub fn derive_zircon_script_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    derive_type::derive_zircon_script_type_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn zircon_host_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as args::HostFunctionArgs);
    let item = parse_macro_input!(item as syn::ItemFn);
    function::host_function_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn zircon_host_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as args::HostModuleArgs);
    let item = parse_macro_input!(item as syn::ItemMod);
    module::host_module_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
```

### Lightweight Checks

- Run `rustfmt --edition 2021 --check` on macro crate files only if manual edits become hard to visually verify before the testing stage.

### Testing Stage

- [x] Check free disk space for `F:\cargo-targets\codex-reflection-macro-modularity`. If free space on `F:` is `<= 50 GB`, clean that target dir before Cargo validation.
- [x] Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs
```

- [x] Run:

```powershell
cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity
```

- [x] No Milestone 1 validation failures required correction before runtime integration.

### Exit Evidence

- `lib.rs` contains only module declarations and proc-macro entry points.
- All moved modules compile through macro crate tests.
- Existing unsupported-input macro tests still pass after the move.

## Milestone 2: Macro-Owned Metadata Hardening

### Goal

Add focused macro crate tests that cover unsupported inputs and descriptor metadata owned by code generation, then verify runtime macro integration still works.

### In-Scope Behaviors

- Unsupported method input is rejected by `zircon_host_function`.
- Unsupported non-identifier parameter input is rejected by `zircon_host_function`.
- Non-inline `zircon_host_module` input is rejected.
- Union input is rejected by `ZirconScriptType`.
- Type metadata generation preserves name, prototype, value kind, value construction, field docs, and skipped fields.
- Function metadata generation preserves exported name, capability, documentation, return type name, and parameter type refs.
- Module metadata generation includes derived types and host functions discovered inside an inline module.

### Dependencies

- Milestone 1 structural cutover complete.
- Existing expansion helper functions available from focused modules to crate tests.

### Implementation Slices

- [x] **Add unsupported-input tests to `zircon_runtime/reflection_macros/src/tests.rs`.** Add tests with exact assertions:

```rust
#[test]
fn host_function_rejects_methods() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn length(&self) -> f64 {
            0.0
        }
    };

    let error = crate::function::host_function_impl(
        crate::args::HostFunctionArgs::default(),
        function,
    )
    .expect_err("methods should be rejected");

    assert!(error.to_string().contains("methods"));
}

#[test]
fn host_function_rejects_non_identifier_parameters() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn unpack((x, y): (f64, f64)) -> f64 {
            x + y
        }
    };

    let error = crate::function::host_function_impl(
        crate::args::HostFunctionArgs::default(),
        function,
    )
    .expect_err("destructuring parameters should be rejected");

    assert!(error.to_string().contains("simple identifiers"));
}

#[test]
fn host_module_rejects_non_inline_modules() {
    let module: syn::ItemMod = syn::parse_quote! {
        mod external_math;
    };
    let args = crate::args::HostModuleArgs {
        name: Some("test.external".to_string()),
        ..Default::default()
    };

    let error = crate::module::host_module_impl(args, module)
        .expect_err("non-inline host modules should be rejected");

    assert!(error.to_string().contains("inline module body"));
}

#[test]
fn script_type_rejects_unions() {
    let input: syn::DeriveInput = syn::parse_quote! {
        union BadValue {
            int_value: i64,
            float_value: f64,
        }
    };

    let error = crate::derive_type::derive_zircon_script_type_impl(input)
        .expect_err("unions should be rejected");

    assert!(error.to_string().contains("unions"));
}
```

- [x] **Add metadata token tests to `zircon_runtime/reflection_macros/src/tests.rs`.** Use token string assertions for generated code ownership. Normalize whitespace by calling `.to_string()` and asserting key fragments rather than comparing entire token streams.

```rust
#[test]
fn script_type_expansion_preserves_metadata_and_skips_fields() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[zircon_script(
            name = "MetaVec3",
            value_kind = ScriptHostValueKind::Float,
            prototype = ScriptHostPrototypeKind::Struct,
            allow_value_construction = true,
            documentation = "vector docs"
        )]
        struct Vec3 {
            #[zircon_script(type_name = "float", documentation = "x docs")]
            x: f64,
            #[zircon_script(skip)]
            cached_length: f64,
        }
    };

    let tokens = crate::derive_type::derive_zircon_script_type_impl(input)
        .expect("script type expansion")
        .to_string();

    assert!(tokens.contains("MetaVec3"));
    assert!(tokens.contains("ScriptHostValueKind :: Float"));
    assert!(tokens.contains("ScriptHostPrototypeKind :: Struct"));
    assert!(tokens.contains("allow_value_construction (true)"));
    assert!(tokens.contains("with_documentation (\"vector docs\")"));
    assert!(tokens.contains("with_documentation (\"x docs\")"));
    assert!(!tokens.contains("cached_length"));
}

#[test]
fn host_function_expansion_preserves_descriptor_metadata() {
    let function: syn::ItemFn = syn::parse_quote! {
        fn length(value: f64) -> f64 {
            value
        }
    };
    let args = crate::args::HostFunctionArgs {
        name: Some("vec_length".to_string()),
        return_type_name: Some("float".to_string()),
        return_value_kind: None,
        capability: vec!["math.read".to_string()],
        documentation: Some("length docs".to_string()),
    };

    let tokens = crate::function::host_function_impl(args, function)
        .expect("host function expansion")
        .to_string();

    assert!(tokens.contains("vec_length"));
    assert!(tokens.contains("math.read"));
    assert!(tokens.contains("length docs"));
    assert!(tokens.contains("with_return_type"));
    assert!(tokens.contains("ScriptHostParameterDescriptor"));
}

#[test]
fn host_module_expansion_collects_types_and_functions() {
    let module: syn::ItemMod = syn::parse_quote! {
        mod math {
            #[derive(ZirconScriptType)]
            struct Vec3 {
                x: f64,
            }

            #[zircon_host_function(name = "length")]
            fn length(x: f64) -> f64 {
                x
            }
        }
    };
    let args = crate::args::HostModuleArgs {
        name: Some("test.math".to_string()),
        version: Some("0.2.0".to_string()),
        capability: vec!["math.read".to_string()],
        documentation: Some("math docs".to_string()),
    };

    let tokens = crate::module::host_module_impl(args, module)
        .expect("host module expansion")
        .to_string();

    assert!(tokens.contains("test.math"));
    assert!(tokens.contains("0.2.0"));
    assert!(tokens.contains("math.read"));
    assert!(tokens.contains("Vec3 as :: zircon_runtime :: core :: framework :: script :: ZirconScriptType"));
    assert!(tokens.contains("__zircon_host_function_descriptor_length"));
    assert!(tokens.contains("__zircon_host_export_function_length"));
}
```

- [x] **If token formatting differs, adjust assertions only to stable semantic fragments.** Do not weaken tests to only check non-empty output; every metadata test must assert at least three concrete fragments tied to descriptor semantics.

- [x] **Run the Milestone 2 testing stage.** Fix failures in the lowest layer first: macro test compile errors, then macro expansion semantics, then runtime integration.

### Lightweight Checks

- No early Cargo loop is required. Use `rustfmt --edition 2021 --check` on `tests.rs` if the test edits become hard to visually inspect before the testing stage.

### Testing Stage

- [x] Check free disk space for `F:\cargo-targets\codex-reflection-macro-modularity`. If free space on `F:` is `<= 50 GB`, clean that target dir before Cargo validation.
- [x] Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs
```

- [x] Run:

```powershell
cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity
```

- [x] Run focused runtime integration tests:

```powershell
cargo test -p zircon_runtime rust_reflection_macros_generate_type_function_and_module_descriptors --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity -- --nocapture --test-threads=1
cargo test -p zircon_runtime host_reflection_docs_include_macro_generated_builtin_math_module --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-macro-modularity -- --nocapture --test-threads=1
```

- [x] Runtime tests were not blocked by unrelated UI/rendering compile errors during the final scoped validation.

### Exit Evidence

- Macro crate unit tests cover unsupported-input and metadata-generation boundaries.
- Runtime macro integration and generated docs tests pass, or exact unrelated blockers are recorded.
- No public macro name or descriptor DTO path changed.

## Milestone 3: Documentation And Closeout

### Goal

Document the new macro crate structure and close the coordination loop with scoped validation evidence.

### In-Scope Behaviors

- `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` reflects the new macro crate structure.
- The docs distinguish macro-owned compile-time rejection from runtime `HostExportRegistry` descriptor validation.
- Fresh validation evidence is added to the document header or body.
- Active session note is deleted if no handoff remains.

### Dependencies

- Milestone 1 and 2 implementation complete.
- Validation evidence or exact blocker records available.

### Implementation Slices

- [x] **Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` machine-readable header.** Add the new macro crate files to both `related_code` and `implementation_files` if they are not already present:

```yaml
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime/reflection_macros/src/tokens.rs
  - zircon_runtime/reflection_macros/src/tests.rs
```

- [x] **Add a macro crate structure section to `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`.** Insert after the existing paragraph that describes the proc-macro crate. Use this content and adjust only if final file names differ:

```markdown
The macro crate is split by code-generation responsibility. `lib.rs` contains only the Rust-required proc-macro entry points and module declarations. `args.rs` owns attribute argument parsing, `attrs.rs` owns `#[zircon_script]` parsing and item discovery, `derive_type.rs` emits `ZirconScriptType` descriptors, `function.rs` emits host function descriptors and callbacks, `module.rs` emits host module descriptors and registration functions, `tokens.rs` owns shared token helpers, and `tests.rs` covers unsupported input plus descriptor metadata generation. Runtime validation remains in `HostExportRegistry`; the macro crate only rejects Rust shapes that cannot be represented correctly as host descriptors.
```

- [x] **Add validation evidence to docs.** Append fresh command results to the `tests:` header entries or a validation paragraph. Include exact commands, `--locked`, `--offline`, `--jobs 1`, target dir, pass/fail status, and first unrelated blocker if any.

- [x] **Run documentation sanity checks.** Use grep/read only; confirm the docs mention `args.rs`, `derive_type.rs`, `HostExportRegistry`, and `unsupported input` or equivalent wording.

- [x] **Update and retire the active session note.** If no handoff is needed, delete `.codex/sessions/20260520-0416-zrvm-host-reflection-followup.md` after final validation and docs updates. If there is an unresolved blocker another session needs, move it to `.codex/sessions/archive/` with `status: completed` and a 2-5 bullet handoff summary.

### Lightweight Checks

- No Cargo command is needed for docs-only edits after Milestone 2 validation unless code changes are made during documentation.

### Testing Stage

- [x] Milestone 3 closeout changed docs/session state only; no additional Rust code edits were made in this closeout step.
- [x] Additional Cargo validation was not required by Milestone 3 after the last passing scoped validation; final closeout still reruns scoped verification before completion reporting.

### Closeout Evidence

- Documentation sanity checks confirmed `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` mentions `args.rs`, `derive_type.rs`, `HostExportRegistry`, and unsupported input wording.
- The active coordination note had no unresolved blocker or handoff requirement, so it was retired from `.codex/sessions/` root.
- Fresh closeout validation on 2026-05-21 passed: macro crate `rustfmt`, macro crate unit tests, and the two focused `zircon_runtime` host reflection integration tests.

### Exit Evidence

- Runtime host reflection docs list the new macro crate files and explain their responsibilities.
- Validation evidence is fresh and scoped.
- Active coordination note is not left stale in `.codex/sessions/` root.

## Final Reporting Requirements

- State that validation was scoped, not workspace-wide, unless a workspace command is actually run.
- List exact commands run and whether `--locked` was included.
- State whether `F:` free space required target cleanup.
- State whether runtime integration tests passed or the first unrelated blocker if they did not.
- State that no JSON export, backend ABI change, or new host module was added.
- Do not commit unless the user explicitly asks for a commit.

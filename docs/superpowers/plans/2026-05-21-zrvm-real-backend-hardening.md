# ZrVM Real Backend Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the real ZrVM backend adapter so descriptor-to-native-module lowering and callback conversion fail with contextual diagnostics and focused coverage.

**Architecture:** Keep `zircon_plugins/zr_vm_language/runtime` as the backend-specific owner. Reuse `zircon_runtime::core::framework::script` descriptors and `HostExportRegistry` for shared validation/capability dispatch, and add only private adapter helpers/tests for `zr_vm`-specific arity and value-conversion constraints. Do not change public package protocol, public backend registration APIs, host modules, or shared descriptor DTOs.

**Tech Stack:** Rust 2021, `zircon_plugins` Cargo workspace, `zircon_runtime::script` VM host contracts, optional `real-zr-vm` feature using `zr_vm_rust_binding`, scoped Cargo validation with `--locked --offline --jobs 1` and `F:\cargo-targets\codex-zrvm-real-backend-hardening`.

---

## Current Baseline

- Repository policy requires staying in the existing `main` checkout. Do not create a branch or worktree.
- Approved spec: `docs/superpowers/specs/2026-05-21-zrvm-real-backend-hardening-design.md`.
- Active coordination note: `.codex/sessions/20260521-0230-zrvm-next-slice-design.md`. Update it during implementation and validation, then delete it on clean completion if no handoff remains.
- `zircon_plugins/zr_vm_language/runtime/src/real_backend.rs` is currently 296 lines and owns real `zr_vm` runtime creation, host module registration, lifecycle export calls, native argument conversion, host return lowering, and optional-export handling.
- `build_native_function` already checks `min_argument_count` and `max_argument_count` for `u16` overflow, but it does not check `min <= max`, parameter count against max arity, or add module/function context to native argument conversion failures.
- `to_zr_value` lowers `ScriptHostValue::{Null,Bool,Int,Float,String,Bytes,HostHandle}` directly to `zrvm::Value`, with bytes lowered through `String::from_utf8_lossy` and host handles lowered as integers.
- Existing non-real tests in `zircon_plugins/zr_vm_language/runtime/src/lib.rs` cover plugin descriptor/capability registration and backend selector wiring.
- Existing `real-zr-vm` feature-gated tests cover native host module registration, lifecycle roundtrip, persistent session state, and the documented minimal example. They require local binding artifacts and `ZR_VM_RUST_BINDING_LIB_DIR`.
- Runtime structure audit reported no plugin runtime gaps. Unrelated large-file hotspots remain outside this slice in editor/runtime UI and render areas; do not edit them.

## File Structure

Modify:

- `zircon_plugins/zr_vm_language/runtime/src/real_backend.rs`: add private backend-local helper functions for function arity validation, native callback context formatting, native argument reading with module/function context, host return lowering wrapper, and unit tests gated by `#[cfg(test)]` inside this file.
- `zircon_plugins/zr_vm_language/runtime/src/lib.rs`: no production changes expected. Only adjust tests here if private helper visibility in `real_backend.rs` proves impractical.
- `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`: update the machine-readable header and body with backend-specific lowering validation, value conversion policy, and validation evidence.
- `.codex/sessions/20260521-0230-zrvm-next-slice-design.md`: keep live status current during implementation/testing; delete on clean completion.

Do not modify unless a compile error or test evidence proves it necessary:

- `zircon_runtime/src/core/framework/script.rs`: shared descriptor DTOs are sufficient.
- `zircon_runtime/src/script/vm/host/host_export_registry.rs`: shared descriptor validation and capability dispatch remain unchanged.
- `zircon_runtime/src/script/vm/host/builtin_host_modules.rs`: no new host modules or manager-backed behavior in this slice.
- `zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs`: hot-reload order remains unchanged.
- `zircon_plugins/zr_vm_language/runtime/src/backend.rs`: backend family wiring should remain unchanged.

## Milestone 1: Backend-Local Validation And Conversion Hardening

### Goal

Make `real_backend.rs` validate `zr_vm`-specific function lowering constraints and produce contextual errors for native callback conversion without changing public APIs or shared descriptors.

### In-Scope Behaviors

- `min_argument_count` and `max_argument_count` above `u16::MAX` still fail before `zrvm::FunctionBuilder::new`.
- `min_argument_count > max_argument_count` fails before `zrvm::FunctionBuilder::new`.
- `function.parameters.len() > max_argument_count` fails before `zrvm::FunctionBuilder::new`.
- Function-lowering errors include `module.function` context.
- Native callback argument conversion errors include `module.function` context.
- Capability-denied callback dispatch errors include `module.function` context when mapped into a `zrvm::Error`.
- Host return lowering policy remains unchanged for null, bool, int, float, string, bytes, and host handle values.
- No public type, trait, feature flag, package protocol, or module descriptor path changes.

### Dependencies

- Existing `ScriptHostFunctionDescriptor` and `ScriptHostValue` model from `zircon_runtime::core::framework::script`.
- Existing `HostExportRegistry::call_with_capabilities` capability enforcement.
- Existing `zrvm::FunctionBuilder` arity API and `zrvm::Value` constructors under the `real-zr-vm` feature.

### Implementation Slices

- [ ] **Update active session note before code edits.** Set `.codex/sessions/20260521-0230-zrvm-next-slice-design.md` to `status: active-zrvm-real-backend-hardening-implementation`, set a fresh `updated_at`, and list `zircon_plugins/zr_vm_language/runtime/src/real_backend.rs` as the only production code edit target for Milestone 1.

- [ ] **Add a private context helper in `real_backend.rs`.** Place it near `build_native_function` so arity validation and callback errors share one spelling.

```rust
fn native_function_label(module_name: &str, function_name: &str) -> String {
    format!("{module_name}.{function_name}")
}
```

- [ ] **Add private arity validation in `real_backend.rs`.** Place this above `build_native_function` or immediately below it. Keep it private and return concrete `u16` values for the builder.

```rust
fn validate_native_function_arity(
    module_name: &str,
    function: &ScriptHostFunctionDescriptor,
) -> Result<(u16, u16), VmError> {
    let label = native_function_label(module_name, &function.name);
    let min = u16::try_from(function.min_argument_count).map_err(|_| {
        VmError::Operation(format!("zr_vm function {label} min arity exceeds u16"))
    })?;
    let max = u16::try_from(function.max_argument_count).map_err(|_| {
        VmError::Operation(format!("zr_vm function {label} max arity exceeds u16"))
    })?;
    if function.min_argument_count > function.max_argument_count {
        return Err(VmError::Operation(format!(
            "zr_vm function {label} min arity {} exceeds max arity {}",
            function.min_argument_count, function.max_argument_count
        )));
    }
    if function.parameters.len() > function.max_argument_count {
        return Err(VmError::Operation(format!(
            "zr_vm function {label} declares {} parameters but max arity is {}",
            function.parameters.len(), function.max_argument_count
        )));
    }
    Ok((min, max))
}
```

- [ ] **Use `validate_native_function_arity` in `build_native_function`.** Replace the current two `u16::try_from(...)` blocks with:

```rust
let function_name = function.name.clone();
let label = native_function_label(module_name, &function_name);
let (min, max) = validate_native_function_arity(module_name, function)?;
```

Then keep `let module_name = module_name.to_string();` after computing `label` or clone the label before moving strings into the closure. The closure must still call `HostExportRegistry::call_with_capabilities` with the original module and function names.

- [ ] **Add contextual native argument reading.** Replace `read_host_arguments(context)?` in the callback closure with a contextual helper:

```rust
let arguments = read_host_arguments_for_function(context, &label)?;
```

Add this helper and keep the existing `read_host_arguments` only if tests or readability benefit from the lower-level helper:

```rust
fn read_host_arguments_for_function(
    context: &zrvm::NativeCallContext,
    function_label: &str,
) -> Result<Vec<ScriptHostValue>, zrvm::Error> {
    let count = context.argument_count().map_err(|error| {
        zr_error(format!(
            "failed to read argument count for {function_label}: {error}"
        ))
    })?;
    let mut arguments = Vec::with_capacity(count);
    for index in 0..count {
        let value = context.argument(index).map_err(|error| {
            zr_error(format!(
                "failed to read argument {index} for {function_label}: {error}"
            ))
        })?;
        arguments.push(from_zr_value_for_function(&value, function_label, index)?);
    }
    Ok(arguments)
}
```

- [ ] **Add contextual ZrVM value conversion.** Keep the supported conversion policy unchanged, but add function and index context for unsupported kinds.

```rust
fn from_zr_value_for_function(
    value: &zrvm::Value,
    function_label: &str,
    index: usize,
) -> Result<ScriptHostValue, zrvm::Error> {
    match value.kind() {
        zrvm::ValueKind::Null => Ok(ScriptHostValue::Null),
        zrvm::ValueKind::Bool => Ok(ScriptHostValue::Bool(value.as_bool()?)),
        zrvm::ValueKind::Int => Ok(ScriptHostValue::Int(value.as_int()?)),
        zrvm::ValueKind::Float => Ok(ScriptHostValue::Float(value.as_float()?)),
        zrvm::ValueKind::String => Ok(ScriptHostValue::String(value.as_string()?)),
        other => Err(zr_error(format!(
            "unsupported zr_vm native argument kind {other:?} at {function_label} argument {index}"
        ))),
    }
}
```

If the binding's accessor methods can fail after the kind check, leave those failures as binding errors unless they can be wrapped without losing status. Do not invent lossy conversions for arrays, objects, handles, or other future `zrvm::ValueKind` variants.

- [ ] **Wrap callback dispatch and return lowering with context.** In the callback closure, map host dispatch and return-lowering errors with the label:

```rust
let value = exports
    .call_with_capabilities(&module_name, &function_name, arguments, &capabilities)
    .map_err(|error| zr_error(format!("zr_vm host callback {label} failed: {error}")))?;
to_zr_value_for_function(value, &label)
```

Add the wrapper:

```rust
fn to_zr_value_for_function(
    value: ScriptHostValue,
    function_label: &str,
) -> Result<zrvm::Value, zrvm::Error> {
    to_zr_value(value).map_err(|error| {
        zr_error(format!(
            "failed to lower host return value for {function_label}: {error}"
        ))
    })
}
```

Keep `to_zr_value` itself as the single supported return policy implementation so tests can call it directly.

- [ ] **Add private tests in `real_backend.rs`.** Append a `#[cfg(test)] mod tests` to `real_backend.rs`. These tests run only when `real_backend.rs` is compiled, which is behind `real-zr-vm`; keep them focused and deterministic.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::script::{
        ScriptHostFunctionDescriptor, ScriptHostParameterDescriptor, ScriptHostValueKind,
    };
    use zircon_runtime::script::{CapabilitySet, HostExportFunction, HostExportRegistry};

    fn descriptor_with_arity(min: usize, max: usize) -> ScriptHostFunctionDescriptor {
        ScriptHostFunctionDescriptor::new("bad", min, max, ScriptHostValueKind::Null)
    }

    #[test]
    fn validate_native_function_arity_rejects_min_overflow() {
        let descriptor = descriptor_with_arity(usize::from(u16::MAX) + 1, usize::from(u16::MAX) + 1);
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error.to_string().contains("min arity exceeds u16"));
    }

    #[test]
    fn validate_native_function_arity_rejects_max_overflow() {
        let descriptor = descriptor_with_arity(0, usize::from(u16::MAX) + 1);
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error.to_string().contains("max arity exceeds u16"));
    }

    #[test]
    fn validate_native_function_arity_rejects_min_greater_than_max() {
        let descriptor = descriptor_with_arity(3, 2);
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error.to_string().contains("min arity 3 exceeds max arity 2"));
    }

    #[test]
    fn validate_native_function_arity_rejects_parameter_count_above_max() {
        let descriptor = descriptor_with_arity(0, 1)
            .with_parameter(ScriptHostParameterDescriptor::new("left", ScriptHostValueKind::Float))
            .with_parameter(ScriptHostParameterDescriptor::new("right", ScriptHostValueKind::Float));
        let error = validate_native_function_arity("example", &descriptor).unwrap_err();

        assert!(error.to_string().contains("example.bad"));
        assert!(error.to_string().contains("declares 2 parameters but max arity is 1"));
    }

    #[test]
    fn to_zr_value_lowers_supported_host_values() {
        assert!(matches!(to_zr_value(ScriptHostValue::Null).unwrap().kind(), zrvm::ValueKind::Null));
        assert_eq!(to_zr_value(ScriptHostValue::Bool(true)).unwrap().as_bool().unwrap(), true);
        assert_eq!(to_zr_value(ScriptHostValue::Int(7)).unwrap().as_int().unwrap(), 7);
        assert_eq!(to_zr_value(ScriptHostValue::Float(1.5)).unwrap().as_float().unwrap(), 1.5);
        assert_eq!(to_zr_value(ScriptHostValue::String("ok".to_string())).unwrap().as_string().unwrap(), "ok");
        assert_eq!(to_zr_value(ScriptHostValue::Bytes(vec![104, 105])).unwrap().as_string().unwrap(), "hi");
        assert_eq!(to_zr_value(ScriptHostValue::HostHandle(42)).unwrap().as_int().unwrap(), 42);
    }

    #[test]
    fn callback_dispatch_errors_include_function_context() {
        let exports = HostExportRegistry::default();
        exports
            .register_module(
                zircon_runtime::core::framework::script::ScriptHostModuleDescriptor::new("example", "0.1.0")
                    .with_capability("allowed")
                    .with_function(
                        ScriptHostFunctionDescriptor::new("secure", 0, 0, ScriptHostValueKind::Null)
                            .with_required_capability("allowed"),
                    ),
                [HostExportFunction::new("secure", |_| Ok(ScriptHostValue::Null))],
            )
            .unwrap();

        let error = exports
            .call_with_capabilities("example", "secure", Vec::new(), &CapabilitySet::default())
            .map_err(|error| zr_error(format!("zr_vm host callback example.secure failed: {error}")))
            .unwrap_err();

        assert!(error.message.contains("example.secure"));
        assert!(error.message.contains("capability"));
    }
}
```

If exact accessor return types differ, adjust only the assertion syntax while preserving semantic coverage. Do not weaken tests to only assert that a value was constructed.

### Lightweight Checks

- No early Cargo loop is required during implementation slices.
- Use `rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/real_backend.rs` if manual edits become hard to inspect before the testing stage.

### Testing Stage

- [ ] Check free disk space for `F:\cargo-targets\codex-zrvm-real-backend-hardening`. If free space on `F:` is `<= 50 GB`, run `cargo clean --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening` before Cargo validation.
- [ ] Run:

```powershell
rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
```

- [ ] Run the non-real plugin package tests:

```powershell
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening
```

- [ ] Run the real-backend focused tests when `ZR_VM_RUST_BINDING_LIB_DIR` is configured and local `zr_vm` binding artifacts exist:

```powershell
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-zrvm-real-backend-hardening real_backend -- --nocapture --test-threads=1
```

- [ ] If the real-backend command cannot run because `ZR_VM_RUST_BINDING_LIB_DIR` or binding artifacts are unavailable, record the exact environment blocker in the session note and docs. Do not claim `real-zr-vm` validation passed.
- [ ] If any test fails, fix the lowest failing backend-local layer first: arity helper, conversion helper, callback wrapper, then integration fixtures.

### Exit Evidence

- Backend-specific arity validation tests cover overflow, invalid ordering, and parameter/max mismatch.
- Return-lowering tests cover all currently supported `ScriptHostValue` variants.
- Callback error mapping test proves module/function context is included for capability denial.
- Non-real plugin tests pass, and real-backend focused tests pass or have an exact local-environment blocker recorded.
- No shared descriptor DTO or host registry behavior changed.

## Milestone 2: Documentation And Closeout

### Goal

Document the backend hardening boundary, record scoped validation evidence, and retire the active coordination note.

### In-Scope Behaviors

- `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` explains the split between shared descriptor validation and `zr_vm` backend-specific lowering validation.
- The docs mention the `u16` arity limit as a target-backend constraint.
- The docs describe the supported native argument and host return value conversion policy.
- The docs record exact validation commands, `--locked`, `--offline`, `--jobs 1`, target dir, pass/fail status, and any real-backend environment blocker.
- Active session note is deleted on clean completion if no handoff remains.

### Dependencies

- Milestone 1 implementation and validation evidence or exact blockers.

### Implementation Slices

- [ ] **Update `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` machine-readable header.** Ensure `related_code` and `implementation_files` include:

```yaml
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
```

If the file is already listed, keep one copy only.

- [ ] **Add a backend lowering paragraph to `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`.** Insert after the generated interface documentation or package protocol section, whichever reads more naturally after implementation. Use this wording as the minimum content:

```markdown
The real `zr_vm` backend treats `HostExportRegistry` records as already validated neutral descriptors, then applies only target-backend lowering checks. Function arity must fit the `zr_vm` native function ABI (`u16` min/max bounds), `min_argument_count` must not exceed `max_argument_count`, and reflected parameter count must fit the maximum arity. These are backend constraints, not shared descriptor constraints for every future VM backend.

Native callbacks convert ZrVM null, bool, int, float, and string arguments into `ScriptHostValue` before dispatching through `HostExportRegistry::call_with_capabilities`. Host return values lower null, bool, int, float, string, bytes as lossy UTF-8 strings, and `HostHandle` as integers. Unsupported ZrVM argument kinds remain errors with module/function context rather than lossy conversions.
```

- [ ] **Append validation evidence to the `tests:` header.** Include the exact Milestone 1 commands and outcomes. If `real-zr-vm` is blocked by missing local artifacts, write a line in this shape:

```yaml
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-zrvm-real-backend-hardening real_backend -- --nocapture --test-threads=1: not run 2026-05-21; ZR_VM_RUST_BINDING_LIB_DIR was not configured"
```

Use the actual date, status, and blocker text from the validation output.

- [ ] **Update the active session note.** Record final touched files, validation commands, pass/fail status, and any environment blocker in `.codex/sessions/20260521-0230-zrvm-next-slice-design.md` before retiring it.

- [ ] **Retire the active session note.** If no handoff remains, delete `.codex/sessions/20260521-0230-zrvm-next-slice-design.md`. If a real-backend environment blocker should remain visible to other sessions, move it to `.codex/sessions/archive/` with `status: completed` and a 2-5 bullet handoff summary.

### Lightweight Checks

- Run grep/read sanity checks only for docs: confirm `u16`, `HostExportRegistry`, `real \`zr_vm\` backend`, and `HostHandle` are mentioned after the doc edit.

### Testing Stage

- [ ] If Milestone 2 changes Rust code, rerun all Milestone 1 validation commands.
- [ ] If Milestone 2 changes only docs/session state, run:

```powershell
git diff --check -- docs/zircon_runtime/script/vm/zr_vm_host_reflection.md docs/superpowers/plans/2026-05-21-zrvm-real-backend-hardening.md docs/superpowers/specs/2026-05-21-zrvm-real-backend-hardening-design.md
```

- [ ] Re-run docs sanity grep checks after final docs edits.

### Exit Evidence

- Host reflection docs explain backend-specific lowering validation and conversion policy.
- Validation evidence is recorded precisely and does not claim workspace-wide green.
- Active coordination note is not left stale in `.codex/sessions/` root.

## Final Reporting Requirements

- State that validation was scoped to the plugin package and docs unless a workspace command is actually run.
- List exact commands run and whether `--locked` was included.
- State whether `F:` free space required target cleanup.
- State whether `real-zr-vm` tests ran; if not, state the exact environment blocker.
- State that no JSON export, new host module, manager-backed behavior, shared descriptor DTO change, or public backend API change was added.
- Do not commit unless the user explicitly asks for a commit.

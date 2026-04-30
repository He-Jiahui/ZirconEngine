# Native Dynamic Fixture Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real `cdylib` native plugin fixture and prove `NativePluginLoader` can load its descriptor plus runtime/editor entries from an actual dynamic library.

**Architecture:** Keep the fixture as a standalone package in the independent `zircon_plugins` workspace. The fixture native crate does not depend on `zircon_runtime`; it duplicates only the stable C ABI structs needed to export symbols and returns manifest TOML through static C strings. Runtime tests build the fixture crate into a temp target dir, copy the platform library into a temp plugin package, then exercise the existing loader.

**Tech Stack:** Rust 2021, `cdylib`, `libloading` through existing `zircon_runtime::NativePluginLoader`, Cargo workspace under `zircon_plugins`.

---

## File Structure

- Modify: `zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs`
  - Adds the red/green integration-style unit test and helper functions for building/copying the fixture dynamic library.
- Modify: `zircon_plugins/Cargo.toml`
  - Adds `native_dynamic_fixture/native` as an independent workspace member.
- Create: `zircon_plugins/native_dynamic_fixture/plugin.toml`
  - Describes one package with runtime and editor modules backed by the same native crate.
- Create: `zircon_plugins/native_dynamic_fixture/native/Cargo.toml`
  - Declares the `cdylib` fixture crate.
- Create: `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`
  - Exports ABI v1 descriptor, runtime entry, and editor entry symbols.
- Modify: `.codex/sessions/20260427-0123-native-dynamic-fixture-closure.md`
  - Records the final touched modules and validation evidence.

## Task 1: Red Test For Real Native Dynamic Fixture Loading

**Files:**
- Modify: `zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs`

- [ ] **Step 1: Add imports needed by the test**

Add these imports at the top of `zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs`:

```rust
use std::process::Command;
```

- [ ] **Step 2: Add the failing test**

Append this test after `native_registration_reports_preserve_per_plugin_loader_diagnostics`:

```rust
#[test]
fn native_loader_calls_real_fixture_descriptor_and_entries() {
    let fixture_target = temp_export_root("native-dynamic-fixture-target");
    let package_root = temp_export_root("native-dynamic-fixture-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path = build_native_dynamic_fixture(&fixture_target);
    fs::copy(
        &library_path,
        native_root.join(platform_library_file_name("zircon_plugin_native_dynamic_fixture_native")),
    )
    .unwrap();
    fs::copy(
        repo_root().join("zircon_plugins/native_dynamic_fixture/plugin.toml"),
        plugin_root.join("plugin.toml"),
    )
    .unwrap();

    let report = NativePluginLoader.load_discovered_all(&package_root);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.loaded.len(), 1);
    let plugin = &report.loaded[0];
    assert_eq!(plugin.plugin_id, "native_dynamic_fixture");
    assert_eq!(
        plugin.descriptor.as_ref().unwrap().runtime_entry_name.as_deref(),
        Some("zircon_native_dynamic_fixture_runtime_entry_v1")
    );
    assert_eq!(
        plugin.descriptor.as_ref().unwrap().editor_entry_name.as_deref(),
        Some("zircon_native_dynamic_fixture_editor_entry_v1")
    );
    assert_eq!(
        plugin.runtime_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_fixture"
    );
    assert_eq!(
        plugin.editor_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_fixture"
    );

    let registrations = report.runtime_plugin_registration_reports();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].package_manifest.id, "native_dynamic_fixture");
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("runtime entry reached")));
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}
```

- [ ] **Step 3: Add helper functions below `temp_export_root`**

Append these helpers at the bottom of the same file:

```rust
fn build_native_dynamic_fixture(target_root: &std::path::Path) -> PathBuf {
    let manifest_path = repo_root().join("zircon_plugins/Cargo.toml");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("-p")
        .arg("zircon_plugin_native_dynamic_fixture_native")
        .arg("--locked")
        .arg("--target-dir")
        .arg(target_root)
        .arg("--quiet")
        .status()
        .unwrap();
    assert!(status.success(), "native dynamic fixture build failed: {status}");
    target_root
        .join("debug")
        .join(platform_library_file_name("zircon_plugin_native_dynamic_fixture_native"))
}

fn platform_library_file_name(crate_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{crate_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{crate_name}.dylib")
    } else {
        format!("lib{crate_name}.so")
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
```

- [ ] **Step 4: Run the test to verify RED**

Run:

```powershell
cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 -- --nocapture
```

Expected: FAIL because package `zircon_plugin_native_dynamic_fixture_native` or `zircon_plugins/native_dynamic_fixture/plugin.toml` does not exist yet.

## Task 2: Add The Native Dynamic Fixture Package

**Files:**
- Modify: `zircon_plugins/Cargo.toml`
- Create: `zircon_plugins/native_dynamic_fixture/plugin.toml`
- Create: `zircon_plugins/native_dynamic_fixture/native/Cargo.toml`
- Create: `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`

- [ ] **Step 1: Add the workspace member**

In `zircon_plugins/Cargo.toml`, append the member near the other packages:

```toml
    "native_dynamic_fixture/native",
```

- [ ] **Step 2: Create the package manifest**

Create `zircon_plugins/native_dynamic_fixture/plugin.toml`:

```toml
id = "native_dynamic_fixture"
version = "0.1.0"
display_name = "Native Dynamic Fixture"
description = "Real dynamic library fixture for ABI v1 native plugin loading."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_dynamic_fixture.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_dynamic_fixture_native"
target_modes = ["client_runtime", "server_runtime", "editor_host"]
capabilities = ["runtime.plugin.native_dynamic_fixture"]

[[modules]]
name = "native_dynamic_fixture.editor"
kind = "editor"
crate_name = "zircon_plugin_native_dynamic_fixture_native"
target_modes = ["editor_host"]
capabilities = ["editor.extension.native_dynamic_fixture"]
```

- [ ] **Step 3: Create the native crate manifest**

Create `zircon_plugins/native_dynamic_fixture/native/Cargo.toml`:

```toml
[package]
name = "zircon_plugin_native_dynamic_fixture_native"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Native dynamic ABI fixture plugin for Zircon."

[lib]
crate-type = ["cdylib"]

[dependencies]
```

- [ ] **Step 4: Create the exported ABI implementation**

Create `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`:

```rust
use std::ffi::c_char;

const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = 1;

const PLUGIN_MANIFEST: &str = concat!(
    r#"id = "native_dynamic_fixture"
version = "0.1.0"
display_name = "Native Dynamic Fixture"
description = "Real dynamic library fixture for ABI v1 native plugin loading."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_dynamic_fixture.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_dynamic_fixture_native"
target_modes = ["client_runtime", "server_runtime", "editor_host"]
capabilities = ["runtime.plugin.native_dynamic_fixture"]

[[modules]]
name = "native_dynamic_fixture.editor"
kind = "editor"
crate_name = "zircon_plugin_native_dynamic_fixture_native"
target_modes = ["editor_host"]
capabilities = ["editor.extension.native_dynamic_fixture"]
"#,
    "\0"
);

const PLUGIN_ID: &[u8] = b"native_dynamic_fixture\0";
const RUNTIME_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_runtime_entry_v1\0";
const EDITOR_ENTRY: &[u8] = b"zircon_native_dynamic_fixture_editor_entry_v1\0";
const RUNTIME_DIAGNOSTICS: &[u8] = b"runtime entry reached\0";
const EDITOR_DIAGNOSTICS: &[u8] = b"editor entry reached\0";

#[repr(C)]
pub struct NativePluginAbiV1 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
}

#[repr(C)]
pub struct NativePluginEntryReportV1 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
}

struct SyncDescriptor(NativePluginAbiV1);
struct SyncEntryReport(NativePluginEntryReportV1);

unsafe impl Sync for SyncDescriptor {}
unsafe impl Sync for SyncEntryReport {}

static DESCRIPTOR: SyncDescriptor = SyncDescriptor(NativePluginAbiV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    plugin_id: PLUGIN_ID.as_ptr().cast(),
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    runtime_entry_name: RUNTIME_ENTRY.as_ptr().cast(),
    editor_entry_name: EDITOR_ENTRY.as_ptr().cast(),
});

static RUNTIME_REPORT: SyncEntryReport = SyncEntryReport(NativePluginEntryReportV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: RUNTIME_DIAGNOSTICS.as_ptr().cast(),
});

static EDITOR_REPORT: SyncEntryReport = SyncEntryReport(NativePluginEntryReportV1 {
    abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    package_manifest_toml: PLUGIN_MANIFEST.as_bytes().as_ptr().cast(),
    diagnostics: EDITOR_DIAGNOSTICS.as_ptr().cast(),
});

#[no_mangle]
pub extern "C" fn zircon_native_plugin_descriptor_v1() -> *const NativePluginAbiV1 {
    &DESCRIPTOR.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_fixture_runtime_entry_v1(
) -> *const NativePluginEntryReportV1 {
    &RUNTIME_REPORT.0
}

#[no_mangle]
pub extern "C" fn zircon_native_dynamic_fixture_editor_entry_v1(
) -> *const NativePluginEntryReportV1 {
    &EDITOR_REPORT.0
}
```

- [ ] **Step 5: Check the fixture crate directly**

Run:

```powershell
cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
```

Expected: PASS.

## Task 3: Green Test And Closeout Evidence

**Files:**
- Modify: `zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs`
- Modify: `.codex/sessions/20260427-0123-native-dynamic-fixture-closure.md`

- [ ] **Step 1: Run the focused native fixture test**

Run:

```powershell
cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 -- --nocapture
```

Expected: PASS. The output should show the focused test passed and should not show native loader diagnostics.

- [ ] **Step 2: Run the existing native loader tests**

Run:

```powershell
cargo test -p zircon_runtime --lib native_loader_ --locked --jobs 1 -- --nocapture
```

Expected: PASS for the existing native loader tests plus the new fixture test.

- [ ] **Step 3: Update the session note**

Update `.codex/sessions/20260427-0123-native-dynamic-fixture-closure.md` with:

```markdown
## Checks / Failing Signals
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1` passed.
- `cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --locked --jobs 1 -- --nocapture` passed.
- `cargo test -p zircon_runtime --lib native_loader_ --locked --jobs 1 -- --nocapture` passed.
```

- [ ] **Step 4: Do not commit unless requested**

Run:

```powershell
git status --short
```

Expected: the new fixture files, plan/spec docs, session note, and native loader test changes are visible. Do not run `git commit` unless the user explicitly asks for a commit.

## Self-Review

- Spec coverage: Task 1 proves the real loader path with a dynamic library; Task 2 adds the fixture package and exported ABI symbols; Task 3 validates the fixture and records evidence.
- Placeholder scan: no placeholder steps remain; every created file has concrete contents.
- Type consistency: symbol names in the descriptor match exported function names and test assertions; package id and crate name match `plugin.toml`, fixture manifest TOML, and Cargo package name.

---
related_code:
  - zircon_ui/src/module/ui_module_descriptor.rs
  - zircon_ui/src/event_ui/manager/ui_event_manager.rs
  - zircon_ui/src/tests/shared_core.rs
  - zircon_core/src/runtime/contexts/plugin_context.rs
  - zircon_core/src/runtime/descriptors/plugin_descriptor.rs
  - zircon_core/src/runtime/descriptors/service_factory.rs
  - zircon_core/src/runtime/handle/registration.rs
  - zircon_core/src/runtime/handle/resolution.rs
  - zircon_core/src/runtime/state/service_entry.rs
  - zircon_module/src/service_factory.rs
  - zircon_script/src/vm/module/module_descriptor.rs
  - zircon_script/src/vm/backend/backend_registry.rs
  - zircon_script/src/vm/backend/vm_backend.rs
  - zircon_script/src/vm/plugin/vm_plugin_instance.rs
  - zircon_script/src/vm/runtime/hot_reload_coordinator.rs
  - zircon_script/src/vm/runtime/vm_plugin_manager.rs
  - zircon_script/src/vm/tests.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/session/mod.rs
  - zircon_editor/src/editing/ui_asset/session/ui_asset_editor_session.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
implementation_files:
  - zircon_ui/src/module/ui_runtime_driver.rs
  - zircon_core/src/runtime/descriptors/plugin_factory.rs
  - zircon_script/src/vm/backend/vm_backend_family.rs
  - zircon_script/src/vm/backend/builtin_vm_backend_family.rs
  - zircon_script/src/vm/host/vm_plugin_host_context.rs
  - zircon_script/src/vm/host/vm_plugin_slot_lifecycle.rs
  - zircon_editor/src/editing/ui_asset/session/lifecycle.rs
  - zircon_editor/src/editing/ui_asset/session/command_entry.rs
  - zircon_editor/src/editing/ui_asset/session/palette_state.rs
  - zircon_editor/src/editing/ui_asset/session/binding_state.rs
plan_sources:
  - user: 2026-04-18 formalize convergence-gap repair spec and detailed implementation plan
  - docs/superpowers/specs/2026-04-18-convergence-gap-closure-design.md
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_script/src/vm/tests.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
doc_type: milestone-detail
---
# Convergence Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 关闭 `zircon_ui`、`zircon_script`、`zircon_editor` 当前仍阻止结构化审计转绿的三个收敛缺口。

**Architecture:** 先把 `zircon_ui` 从 stub descriptor 升到真实 module runtime，再把 `zircon_core` 的 plugin descriptor 改成 context-aware factory，为 `zircon_script` 的 host context 和 backend family 提供下层支点。最后把 `zircon_editor` 的 `UiAssetEditorSession` 真正迁入 `session/` 子树，并恢复 editor 相关测试面。

**Tech Stack:** Rust 2021, Cargo, `zircon_core` descriptor runtime, `zircon_module` helpers, repo-local audit script

---

## Target File Structure

```text
zircon_ui/src/module/
  ui_runtime_driver.rs
  ui_module_descriptor.rs

zircon_core/src/runtime/descriptors/
  plugin_factory.rs
  plugin_descriptor.rs

zircon_script/src/vm/backend/
  vm_backend_family.rs
  builtin_vm_backend_family.rs

zircon_script/src/vm/host/
  vm_plugin_host_context.rs
  vm_plugin_slot_lifecycle.rs

zircon_editor/src/editing/ui_asset/session/
  ui_asset_editor_session.rs
  lifecycle.rs
  command_entry.rs
  palette_state.rs
  binding_state.rs
```

## Validation Baseline

- 审计命令：
  - `python .\.codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_runtime_structure.py --json`
- 当前额外基线：
  - `cargo test -p zircon_editor --no-run --message-format short` 目前先失败于 `zircon_asset/src/pipeline/manager/project_asset_manager/loading/*` 的上游回归。

### Task 1: Promote `zircon_ui` To A Real Runtime Module

**Files:**
- Create: `zircon_ui/src/module/ui_runtime_driver.rs`
- Modify: `zircon_ui/src/module/mod.rs`
- Modify: `zircon_ui/src/module/ui_module_name.rs`
- Modify: `zircon_ui/src/module/ui_module_descriptor.rs`
- Modify: `zircon_ui/src/lib.rs`
- Modify: `zircon_ui/src/tests/shared_core.rs`

- [ ] **Step 1: Write failing tests for real UI driver/manager registration**

```rust
#[test]
fn ui_module_descriptor_registers_real_driver_and_manager_services() {
    let descriptor = module_descriptor();
    assert_eq!(descriptor.drivers.len(), 1);
    assert_eq!(descriptor.managers.len(), 1);
    assert_eq!(descriptor.drivers[0].name.as_str(), UI_RUNTIME_DRIVER_NAME);
    assert_eq!(descriptor.managers[0].name.as_str(), UI_EVENT_MANAGER_NAME);
}

#[test]
fn ui_module_activates_and_resolves_runtime_driver_and_event_manager() {
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    core.register_module(UiModule.descriptor()).unwrap();
    core.activate_module(UI_MODULE_NAME).unwrap();
    core.resolve_driver::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME).unwrap();
    core.resolve_manager::<UiEventManager>(UI_EVENT_MANAGER_NAME).unwrap();
}
```

- [ ] **Step 2: Run the focused UI tests**

Run:

```powershell
cargo test -p zircon_ui shared_core -- --nocapture
```

Expected: fail while `module_descriptor()` still returns a stub descriptor.

- [ ] **Step 3: Implement the new runtime driver and real descriptor**

```rust
// zircon_ui/src/module/ui_runtime_driver.rs
#[derive(Debug, Default)]
pub struct UiRuntimeDriver;
```

```rust
// zircon_ui/src/module/ui_module_name.rs
pub const UI_RUNTIME_DRIVER_NAME: &str = "UiModule.Driver.UiRuntimeDriver";
pub const UI_EVENT_MANAGER_NAME: &str = "UiModule.Manager.UiEventManager";
```

```rust
// zircon_ui/src/module/ui_module_descriptor.rs
ModuleDescriptor::new(UI_MODULE_NAME, "Runtime UI widgets and layout")
    .with_driver(DriverDescriptor::new(
        qualified_name(UI_MODULE_NAME, ServiceKind::Driver, "UiRuntimeDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(UiRuntimeDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(UI_MODULE_NAME, ServiceKind::Manager, "UiEventManager"),
        StartupMode::Immediate,
        vec![dependency_on(UI_MODULE_NAME, ServiceKind::Driver, "UiRuntimeDriver")],
        factory(|_| Ok(Arc::new(UiEventManager::default()) as ServiceObject)),
    ))
```

- [ ] **Step 4: Re-run the UI tests and verify no stub usage remains**

Run:

```powershell
cargo test -p zircon_ui shared_core -- --nocapture
Get-ChildItem zircon_ui/src -Recurse -File | Select-String -Pattern 'stub_module_descriptor'
```

Expected: tests green, no search results.

### Task 2: Make Core Plugin Descriptors Context-Aware

**Files:**
- Create: `zircon_core/src/runtime/descriptors/plugin_factory.rs`
- Modify: `zircon_core/src/runtime/descriptors/mod.rs`
- Modify: `zircon_core/src/runtime/descriptors/plugin_descriptor.rs`
- Modify: `zircon_core/src/runtime/contexts/plugin_context.rs`
- Modify: `zircon_core/src/runtime/state/service_entry.rs`
- Modify: `zircon_core/src/runtime/handle/registration.rs`
- Modify: `zircon_core/src/runtime/handle/resolution.rs`
- Modify: `zircon_core/src/runtime/tests.rs`
- Modify: `zircon_module/src/service_factory.rs`
- Modify: `zircon_module/src/lib.rs`

- [ ] **Step 1: Add a failing test proving plugin resolution needs `PluginContext`**

```rust
#[test]
fn plugin_resolution_builds_plugin_context_instead_of_passing_only_core_handle() {
    let seen = Arc::new(Mutex::new(None::<PluginContext>));
    let seen_for_factory = Arc::clone(&seen);
    let runtime = CoreRuntime::new();
    let core = runtime.handle();

    core.register_module(
        ModuleDescriptor::new("PluginContextSpec", "plugin context test").with_plugin(
            PluginDescriptor::new(
                qualified_name("PluginContextSpec", ServiceKind::Plugin, "RecordedPlugin"),
                StartupMode::Immediate,
                Vec::new(),
                plugin_factory(move |context| {
                    *seen_for_factory.lock().unwrap() = Some(context.clone());
                    Ok(Arc::new(RecordedPlugin(context.clone())) as _)
                }),
            ),
        ),
    )
    .unwrap();

    core.activate_module("PluginContextSpec").unwrap();
    core.resolve_plugin::<RecordedPlugin>("PluginContextSpec.Plugin.RecordedPlugin")
        .unwrap();

    let context = seen.lock().unwrap().clone().unwrap();
    assert_eq!(context.plugin_name, "PluginContextSpec.Plugin.RecordedPlugin");
    assert!(context.package_root.is_none());
}
```

- [ ] **Step 2: Run the focused core tests**

Run:

```powershell
cargo test -p zircon_core runtime -- --nocapture
```

Expected: fail because `PluginDescriptor` still stores `ServiceFactory`.

- [ ] **Step 3: Add `PluginFactory` and extend `PluginContext`**

```rust
pub type PluginFactory =
    Arc<dyn Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync>;
```

```rust
pub struct PluginContext {
    pub plugin_name: String,
    pub core: CoreWeak,
    pub package_root: Option<PathBuf>,
    pub source_root: Option<PathBuf>,
    pub data_root: Option<PathBuf>,
}
```

- [ ] **Step 4: Split service-entry storage between ordinary service factories and plugin factories**

```rust
pub(crate) enum ServiceEntryFactory {
    Service(ServiceFactory),
    Plugin(PluginFactory),
}
```

- [ ] **Step 5: Change plugin registration and resolution to build `PluginContext`**

```rust
ServiceEntryFactory::Plugin(plugin.factory.clone())
```

```rust
let context = PluginContext {
    plugin_name: service_name.to_string(),
    core: self.downgrade(),
    package_root: None,
    source_root: None,
    data_root: None,
};
factory(&context)
```

- [ ] **Step 6: Add `plugin_factory()` helper to `zircon_module` and rerun tests**

Run:

```powershell
cargo test -p zircon_core runtime -- --nocapture
```

Expected: core plugin-resolution tests green.

### Task 3: Add `VmPluginHostContext`, Backend Families, And Slot Lifecycle To `zircon_script`

**Files:**
- Create: `zircon_script/src/vm/backend/vm_backend_family.rs`
- Create: `zircon_script/src/vm/backend/builtin_vm_backend_family.rs`
- Create: `zircon_script/src/vm/host/vm_plugin_host_context.rs`
- Create: `zircon_script/src/vm/host/vm_plugin_slot_lifecycle.rs`
- Modify: `zircon_script/src/vm/backend/mod.rs`
- Modify: `zircon_script/src/vm/backend/backend_registry.rs`
- Modify: `zircon_script/src/vm/backend/vm_backend.rs`
- Modify: `zircon_script/src/vm/plugin/vm_plugin_instance.rs`
- Modify: `zircon_script/src/vm/runtime/hot_reload_coordinator.rs`
- Modify: `zircon_script/src/vm/runtime/vm_plugin_manager.rs`
- Modify: `zircon_script/src/vm/module/module_descriptor.rs`
- Modify: `zircon_script/src/vm/tests.rs`

- [ ] **Step 1: Add failing tests for host-context propagation and backend-family resolution**

```rust
#[test]
fn builtin_backend_family_accepts_qualified_and_legacy_backend_names() {
    let registry = VmBackendRegistry::new();
    registry.register_family(Arc::new(BuiltinVmBackendFamily::default()));
    assert!(registry.resolve("builtin:mock").is_ok());
    assert!(registry.resolve("mock").is_ok());
}

#[test]
fn vm_plugin_manager_propagates_host_context_roots_and_backend_selector() {
    let fixture = PluginFixture::new("sample", "0.1.0", "builtin:mock", &[1, 2, 3]);
    let manager = VmPluginManager::mock();
    let packages = manager.discover_packages(&fixture.root).unwrap();
    let slot = manager.load_discovered_package(&packages[0]).unwrap();
    let record = manager.slot(slot).unwrap();

    assert_eq!(record.backend_name, "builtin:mock");
    assert_eq!(
        record.source.manifest_path.as_deref(),
        Some(fixture.manifest_path.as_path())
    );
}
```

- [ ] **Step 2: Run the focused script tests**

Run:

```powershell
cargo test -p zircon_script vm -- --nocapture
```

Expected: fail because backend family and host context types do not exist yet.

- [ ] **Step 3: Implement `VmBackendFamily` and the built-in family**

```rust
pub trait VmBackendFamily: Send + Sync {
    fn family_name(&self) -> &str;
    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError>;
    fn selectors(&self) -> Vec<String>;
}
```

```rust
match selector {
    "builtin:mock" | "mock" => Ok(Arc::new(MockVmBackend)),
    "builtin:unavailable" | "unavailable" => Ok(Arc::new(UnavailableVmBackend)),
    other => Err(VmError::UnknownBackend(other.to_string())),
}
```

- [ ] **Step 4: Implement `VmPluginSlotLifecycle` and `VmPluginHostContext`**

```rust
pub trait VmPluginSlotLifecycle: Send + Sync {
    fn load_package(&self, backend_selector: &str, package: VmPluginPackage) -> Result<PluginSlotId, VmError>;
    fn hot_reload_slot(&self, slot: PluginSlotId, package: VmPluginPackage) -> Result<(), VmError>;
    fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError>;
    fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError>;
    fn list_slots(&self) -> Vec<VmPluginSlotRecord>;
}
```

```rust
pub struct VmPluginHostContext {
    pub plugin: PluginContext,
    pub capabilities: CapabilitySet,
    pub backend_selector: String,
    pub package_source: VmPluginPackageSource,
    pub host_registry: HostRegistry,
    pub slot_lifecycle: Arc<dyn VmPluginSlotLifecycle>,
}
```

- [ ] **Step 5: Thread the new host context through backend loading and plugin activation**

```rust
fn load_package(
    &self,
    package: &VmPluginPackage,
    host: &VmPluginHostContext,
) -> Result<Box<dyn VmPluginInstance>, VmError>;
```

```rust
fn activate(&mut self, host: &VmPluginHostContext) -> Result<(), VmError>;
```

- [ ] **Step 6: Update `VmPluginManager` to store base `PluginContext`, resolve backend families, and build host contexts**

```rust
pub struct VmPluginManager {
    plugin_context: PluginContext,
    coordinator: HotReloadCoordinator,
    backends: VmBackendRegistry,
    selected_backend: RwLock<String>,
}
```

- [ ] **Step 7: Re-run the script tests**

Run:

```powershell
cargo test -p zircon_script vm -- --nocapture
```

Expected: the new host-context and backend-family tests pass together with existing discovery/hot-reload coverage.

### Task 4: Prove `zircon_script` No Longer Relies On A Core-Only Plugin Path

**Files:**
- Modify: `zircon_script/src/vm/tests.rs`
- Modify: `zircon_script/src/vm/runtime/vm_plugin_manager.rs`

- [ ] **Step 1: Extend tests to assert base `PluginContext` and slot-source roots are visible**

```rust
#[test]
fn core_resolve_plugin_builds_vm_plugin_runtime_with_base_plugin_context() {
    let plugin = core.resolve_plugin::<VmPluginManager>(VM_PLUGIN_RUNTIME_NAME).unwrap();
    assert_eq!(plugin.base_plugin_context().plugin_name, VM_PLUGIN_RUNTIME_NAME);
}
```

```rust
#[test]
fn loading_discovered_package_preserves_manifest_paths_in_slot_records() {
    let slot = manager.load_discovered_package(&packages[0]).unwrap();
    let record = manager.slot(slot).unwrap();
    assert_eq!(record.source.manifest_path.as_deref(), Some(fixture.manifest_path.as_path()));
}
```

- [ ] **Step 2: Run the script tests and the structural audit**

Run:

```powershell
cargo test -p zircon_script vm -- --nocapture
python .\.codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_runtime_structure.py --json
```

Expected: tests green, audit no longer reports `plugin-runtime-gap`.

### Task 5: Move `UiAssetEditorSession` Into The `session/` Subtree For Real

**Files:**
- Create: `zircon_editor/src/editing/ui_asset/session/lifecycle.rs`
- Create: `zircon_editor/src/editing/ui_asset/session/command_entry.rs`
- Create: `zircon_editor/src/editing/ui_asset/session/palette_state.rs`
- Create: `zircon_editor/src/editing/ui_asset/session/binding_state.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session/mod.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session/ui_asset_editor_session.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session/preview_compile.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session/style_inspection.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session/hierarchy_projection.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session/session_state.rs`
- Modify: `zircon_editor/src/editing/ui_asset/session.rs`

- [ ] **Step 1: Replace the shim in `ui_asset_editor_session.rs` with the real type and error definitions**

```rust
#[derive(Debug)]
pub enum UiAssetEditorSessionError {
    InvalidSelectionIndex { index: usize },
    InvalidPreviewIndex { index: usize },
    InvalidSourceBuffer,
    InvalidPaletteIndex { index: usize },
    InvalidBindingIndex { index: usize },
    InvalidStyleTokenIndex { index: usize },
    InvalidStyleRuleIndex { index: usize },
}

pub struct UiAssetEditorSession {
    route: UiAssetEditorRoute,
    source_buffer: UiAssetSourceBuffer,
    diagnostics: Vec<String>,
    undo_stack: UiAssetEditorUndoStack,
    preview_host: Option<UiAssetPreviewHost>,
    state: UiAssetEditorSessionState,
}
```

- [ ] **Step 2: Move lifecycle and command orchestration methods out of the old file**

```rust
impl UiAssetEditorSession {
    pub fn from_source(
        route: UiAssetEditorRoute,
        source: String,
        preview_size: UiSize,
    ) -> Result<Self, UiAssetEditorSessionError> { /* body copied from former session.rs */ }

    fn revalidate(&mut self) -> Result<(), UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    fn rebuild_preview_snapshot(&mut self) -> Result<(), UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    fn refresh_preview_for_current_preset(&mut self) -> Result<(), UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn canonical_source(&self) -> Result<String, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn save_to_canonical_source(&mut self) -> Result<String, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
}
```

```rust
impl UiAssetEditorSession {
    pub fn apply_command(
        &mut self,
        command: UiAssetEditorCommand,
    ) -> Result<(), UiAssetEditorSessionError> { /* body copied from former session.rs */ }

    fn apply_command_with_effects(
        &mut self,
        command: UiAssetEditorCommand,
        effects: UiAssetEditorUndoExternalEffects,
    ) -> Result<(), UiAssetEditorSessionError> { /* body copied from former session.rs */ }

    pub fn undo(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn redo(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
}
```

- [ ] **Step 3: Move palette and binding method families by prefix**

```rust
impl UiAssetEditorSession {
    pub fn select_palette_index(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn update_palette_drag_target(
        &mut self,
        preview_node_id: &str,
        target_index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn clear_palette_drag_target(&mut self) -> bool { /* body copied from former session.rs */ }
    pub fn cycle_palette_drag_target_candidate_next(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn confirm_palette_target_choice(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
}
```

```rust
impl UiAssetEditorSession {
    pub fn select_binding(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn add_binding(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn delete_selected_binding(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn set_selected_binding_event(&mut self, event: &str) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
    pub fn delete_selected_binding_payload(&mut self) -> Result<bool, UiAssetEditorSessionError> { /* body copied from former session.rs */ }
}
```

- [ ] **Step 4: Rewire `session/mod.rs` and remove the old implementation dependency**

```rust
pub(crate) mod binding_state;
pub(crate) mod command_entry;
pub(crate) mod hierarchy_projection;
pub(crate) mod lifecycle;
pub(crate) mod palette_state;
pub(crate) mod preview_compile;
pub(crate) mod session_state;
pub(crate) mod style_inspection;
pub(crate) mod ui_asset_editor_session;
```

- [ ] **Step 5: Run the editor-local session tests**

Run:

```powershell
cargo test -p zircon_editor ui_asset -- --nocapture
cargo test -p zircon_editor ui_asset_palette_drop -- --nocapture
```

Expected: `UiAssetEditorSession::from_source` remains compatible and the split compiles.

### Task 6: Repair Editor Test-Only Drift And Restore `ui_asset_sessions` Structure Coverage

**Files:**
- Modify: `zircon_editor/src/tests/editing/ui_asset.rs`
- Modify: `zircon_editor/src/tests/editing/ui_asset_palette_drop.rs`
- Modify: `zircon_editor/src/tests/host/manager.rs`
- Modify: any editor-local helpers revealed by compile drift

- [ ] **Step 1: Re-run the editor test compile and classify failures**

Run:

```powershell
cargo test -p zircon_editor --no-run --message-format short
```

Expected: either editor-local split regressions, or the existing upstream `zircon_asset` blocker.

- [ ] **Step 2: Fix editor-local drift without changing public session entry points**

```rust
let mut session = UiAssetEditorSession::from_source(route, source, UiSize::new(640.0, 360.0))
    .expect("ui asset editor session");
```

```rust
#[test]
fn editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("host")
        .join("manager")
        .join("ui_asset_sessions");
    for relative in ["mod.rs", "open.rs", "save.rs", "lifecycle.rs", "sync.rs", "imports.rs", "hydration.rs", "preview_refresh.rs"] {
        assert!(root.join(relative).exists());
    }
}
```

- [ ] **Step 3: Run the focused editor tests**

Run:

```powershell
cargo test -p zircon_editor ui_asset -- --nocapture
cargo test -p zircon_editor ui_asset_palette_drop -- --nocapture
cargo test -p zircon_editor editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors -- --nocapture
```

Expected: green, or explicit note that only an upstream `zircon_asset` blocker remains.

### Task 7: Final Audit And Cargo Verification

**Files:**
- Modify: all files touched by Tasks 1-6

- [ ] **Step 1: Run the structural audit**

Run:

```powershell
python .\.codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_runtime_structure.py --json
```

Expected:

- `zircon_ui.status == "converged"`
- `zircon_script.status == "converged"`
- `zircon_editor.status == "converged"`

- [ ] **Step 2: Run the focused cargo tests**

Run:

```powershell
cargo test -p zircon_ui -- --nocapture
cargo test -p zircon_script -- --nocapture
cargo test -p zircon_editor -- --nocapture
```

- [ ] **Step 3: If `zircon_editor` is still blocked by upstream compile failures, record the exact blocker instead of claiming full green**

```text
cargo test -p zircon_editor --no-run --message-format short
fails in zircon_asset/src/pipeline/manager/project_asset_manager/loading/*
with unresolved imports/private method regressions unrelated to the session split.
```

- [ ] **Step 4: Do a final source scan for the three original red flags**

Run:

```powershell
Get-ChildItem zircon_ui/src -Recurse -File | Select-String -Pattern 'stub_module_descriptor'
Get-Item 'zircon_editor/src/editing/ui_asset/session.rs' | Format-List FullName,Length
```

Expected: no `stub_module_descriptor` usage; `session.rs` deleted or reduced to a tiny shim.

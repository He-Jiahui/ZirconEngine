# Plugin Renderer Neutral Readback Execution Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a neutral runtime-owned execution/readback surface so VG/HGI plugin renderer code can register pass executors, publish neutral outputs, and promote deferred tests without restoring old runtime graphics owner paths.

**Architecture:** Work lower-to-upper. First add runtime-owned executor registration contracts and propagate them through plugin catalog, graphics module construction, render framework construction, and `SceneRenderer` executor registry setup. Then add neutral render readback/output DTOs under `zircon_runtime::core::framework::render` and store them in scene-renderer host state. Finally wire VG/HGI plugin sources against plugin-local converters plus neutral runtime DTOs, promote tests, and update rendering docs.

**Tech Stack:** Rust 2021, Cargo workspaces, `zircon_runtime`, `zircon_plugins`, render graph executor ids, runtime plugin extension registry, scene renderer advanced output/readback state, Markdown module docs.

---

## Repository Baseline

- Work from existing `main` checkout at `E:\Git\ZirconEngine`; do not create worktrees or feature branches.
- Do not commit automatically.
- Approved spec: `docs/superpowers/specs/2026-05-02-plugin-renderer-neutral-readback-execution-surface-design.md`.
- Active coordination note: `.codex/sessions/20260502-0132-plugin-renderer-neutral-execution-design.md`.
- Avoid active UI/editor cutover paths: `zircon_runtime_interface`, `zircon_editor`, Runtime UI, editor UI, and app runtime-library wiring unless a later explicit scope change requires them.
- Keep `zircon_runtime` independent from `zircon_plugins`.
- Keep `ViewportRenderFrame` neutral.

## File Structure Map

### Runtime Executor Registration

- Create: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registration.rs` for a neutral `RenderPassExecutorRegistration` declaration.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs` to export `RenderPassExecutorFn`, `RenderPassExecutorRegistration`, `RenderPassExecutorId`, `RenderPassExecutionContext`, and `RenderPassExecutorRegistry` from the graph execution folder.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` to register explicit executor registrations after descriptor-created no-op executors.
- Modify: `zircon_runtime/src/graphics/scene/mod.rs` and `zircon_runtime/src/graphics/mod.rs` to expose the neutral executor registration surface through `zircon_runtime::graphics`.
- Modify: `zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs`, `register.rs`, and `access.rs` to store, register, and expose executor registrations.
- Modify: `zircon_runtime/src/plugin/extension_registry_error.rs` to add duplicate executor id diagnostics.
- Modify: `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs` to merge executor registrations from plugin reports.
- Modify: `zircon_runtime/src/tests/plugin_extensions/extension_registry.rs` to cover executor registration, duplicates, and catalog merge behavior.

### Runtime Construction Propagation

- Modify: `zircon_runtime/src/builtin/runtime_modules.rs` to collect `render_pass_executors()` from plugin registration reports and pass them to `GraphicsModule`.
- Modify: `zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs` to store executor registrations in `GraphicsModule`.
- Modify: `zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs` to capture executor registrations in the render framework manager factory.
- Modify: `zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs` to pass executor registrations into `WgpuRenderFramework`.
- Modify: `zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs` to pass executor registrations into `SceneRenderer`.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new.rs` and `new_with_icon_source.rs` so scene renderer construction installs explicit plugin executor functions after descriptor no-ops.

### Neutral Readback / Output DTOs

- Create: `zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs` for neutral VG/HGI readback/output declarations.
- Modify: `zircon_runtime/src/core/framework/render/mod.rs` to export the new neutral DTOs.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs` and `collect_into_outputs.rs` to carry neutral readback snapshots into host outputs.
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/scene_renderer_advanced_plugin_outputs.rs`, `output_access.rs`, and `output_storage.rs` to store and expose neutral VG/HGI readback/output snapshots.
- Add or modify runtime tests near `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` and `zircon_runtime/src/tests/plugin_extensions/extension_registry.rs`; only create a new runtime test file if the existing test module becomes too mixed.

### Plugin Executor Registration

- Modify: `zircon_plugins/virtual_geometry/runtime/src/lib.rs` to register executor functions for all VG render feature pass ids and assert the registrations in existing plugin tests.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/lib.rs` to register executor functions for all HGI render feature pass ids and assert the registrations in existing plugin tests.

### Plugin Readback / Output Wiring

- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs` to wire `root_state_readbacks` only after its files no longer assume runtime `SceneRenderer` internals.
- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/*.rs` to consume plugin-local readback helpers and neutral `zircon_runtime::core::framework::render` DTOs.
- Modify: `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/*.rs` only for conversion helpers needed to emit neutral VG output DTOs.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/mod.rs` to wire `post_process_sources` and `root_output_sources` only after their imports are neutral/plugin-local.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/post_process_sources/**/*.rs` to consume neutral HGI scene-prepare/readback DTOs rather than runtime frame state.
- Modify: `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/**/*.rs` to remove stale `SceneRenderer` and `scene_renderer_advanced_plugin_readbacks` assumptions.

### Documentation And Coordination

- Modify: `.codex/sessions/20260502-0132-plugin-renderer-neutral-execution-design.md` with implementation status, commands, blockers, and closeout evidence.
- Modify: `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md` with neutral executor/readback ownership, implementation files, plan sources, and tests.
- Modify: `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md` with neutral executor/readback ownership, implementation files, plan sources, and tests.

## Milestone 0: Coordination And Baseline Gates

- Goal: refresh live overlap and establish exact stale-path/search baseline before source edits.
- In-scope behaviors: active-session coordination, branch policy, stale owner path gates, runtime dependency-direction gate.
- Dependencies: approved spec and this plan.
- Lightweight checks: read/search only.

### Implementation Slices

- [ ] Confirm branch is `main`.

Run:

```powershell
git branch --show-current
```

Expected: `main`. If not `main`, stop and ask the user how to reconcile with repository policy.

- [ ] Refresh coordination context.

Run:

```powershell
.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
```

Expected: no active session owns the same graphics/plugin execution/readback files. If runtime-interface or UI sessions are active, keep this plan out of their files.

- [ ] Update `.codex/sessions/20260502-0132-plugin-renderer-neutral-execution-design.md`.

Record:

```markdown
## Current Step
- Implementation plan approved; entering Milestone 0 baseline gates.
```

- [ ] Run old-owner search gates for VG and HGI plugin trees.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
```

Expected: no live Rust code hits from the completed hard-cutover baseline. Any hit is an in-scope blocker unless it is historical text explicitly marked non-code history.

- [ ] Run dependency-direction search gate.

Run:

```powershell
rg --line-number "zircon_plugins" "zircon_runtime/src"
```

Expected: no production import from `zircon_runtime` to a `zircon_plugins` crate. Test fixtures or generated text must be reviewed before accepting them.

### Testing Stage

- [ ] No Cargo commands in Milestone 0. Accept the milestone after commands and findings are recorded in the session note.

### Exit Evidence

- Branch policy confirmed.
- Coordination note refreshed.
- Old-owner and dependency-direction search results recorded.

## Milestone 1: Runtime Executor Registration Surface

- Goal: let runtime plugins register neutral pass executor functions through `RuntimeExtensionRegistry` and merge them into runtime extension catalogs.
- In-scope behaviors: executor registration declaration, duplicate diagnostics, registry accessors, catalog merge tests, no plugin dependency in runtime.
- Dependencies: Milestone 0 gates.
- Lightweight checks: scoped syntax/type check only if needed before testing stage.

### Implementation Slices

- [ ] Create `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registration.rs`.

Target code:

```rust
use super::{RenderPassExecutorFn, RenderPassExecutorId};

#[derive(Clone, Debug)]
pub struct RenderPassExecutorRegistration {
    pub executor_id: RenderPassExecutorId,
    pub executor: RenderPassExecutorFn,
}

impl RenderPassExecutorRegistration {
    pub fn new(
        executor_id: impl Into<RenderPassExecutorId>,
        executor: RenderPassExecutorFn,
    ) -> Self {
        Self {
            executor_id: executor_id.into(),
            executor,
        }
    }

    pub fn executor_id(&self) -> &RenderPassExecutorId {
        &self.executor_id
    }
}
```

- [ ] Update `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs`.

Target exports:

```rust
mod render_graph_execution_record;
mod render_pass_execution_context;
mod render_pass_executor_id;
mod render_pass_executor_registration;
mod render_pass_executor_registry;

pub use render_graph_execution_record::RenderGraphExecutionRecord;
pub use render_pass_execution_context::RenderPassExecutionContext;
pub use render_pass_executor_id::RenderPassExecutorId;
pub use render_pass_executor_registration::RenderPassExecutorRegistration;
pub use render_pass_executor_registry::{RenderPassExecutorFn, RenderPassExecutorRegistry};
```

- [ ] Update `zircon_runtime/src/graphics/scene/mod.rs` and `zircon_runtime/src/graphics/mod.rs` to expose the neutral surface.

`scene/mod.rs` target line:

```rust
RenderPassExecutionContext, RenderPassExecutorFn, RenderPassExecutorId,
RenderPassExecutorRegistration, GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
```

`graphics/mod.rs` target export:

```rust
pub use scene::{
    RenderPassExecutionContext, RenderPassExecutorFn, RenderPassExecutorId,
    RenderPassExecutorRegistration, SceneRenderer,
};
```

Keep non-public scene-renderer internals private.

- [ ] Update `zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs`.

Add import and field:

```rust
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

pub(super) render_pass_executors: Vec<RenderPassExecutorRegistration>,
```

- [ ] Update `zircon_runtime/src/plugin/extension_registry_error.rs`.

Add variant:

```rust
#[error("render pass executor {0} already registered")]
DuplicateRenderPassExecutor(String),
```

- [ ] Update `zircon_runtime/src/plugin/extension_registry/register.rs`.

Add import and method:

```rust
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};

pub fn register_render_pass_executor(
    &mut self,
    registration: RenderPassExecutorRegistration,
) -> Result<(), RuntimeExtensionRegistryError> {
    if self
        .render_pass_executors
        .iter()
        .any(|existing| existing.executor_id() == registration.executor_id())
    {
        return Err(RuntimeExtensionRegistryError::DuplicateRenderPassExecutor(
            registration.executor_id().to_string(),
        ));
    }
    self.render_pass_executors.push(registration);
    Ok(())
}
```

- [ ] Update `zircon_runtime/src/plugin/extension_registry/access.rs`.

Add accessor:

```rust
pub fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
    &self.render_pass_executors
}
```

- [ ] Update `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs`.

After render feature merge, add:

```rust
for executor in registration.extensions.render_pass_executors() {
    push_runtime_extension_result(
        registry.register_render_pass_executor(executor.clone()),
        &mut diagnostics,
    );
}
```

- [ ] Add unit-test code to `zircon_runtime/src/tests/plugin_extensions/extension_registry.rs`.

Add imports:

```rust
use crate::graphics::{
    RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistration,
};
```

Add helper:

```rust
fn weather_render_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Ok(())
}
```

Add test:

```rust
#[test]
fn runtime_extension_registry_collects_render_pass_executor_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let registration = RenderPassExecutorRegistration::new(
        "weather.volumetric-clouds",
        weather_render_executor,
    );

    registry
        .register_render_pass_executor(registration.clone())
        .expect("executor contribution");

    assert_eq!(registry.render_pass_executors().len(), 1);
    assert_eq!(
        registry.render_pass_executors()[0].executor_id(),
        &RenderPassExecutorId::new("weather.volumetric-clouds")
    );
}
```

Add duplicate assertion to `runtime_extension_registry_rejects_duplicate_module_and_render_feature_names` or a focused new test:

```rust
let executor = RenderPassExecutorRegistration::new(
    "weather.volumetric-clouds",
    weather_render_executor,
);
registry
    .register_render_pass_executor(executor.clone())
    .expect("first executor");
let duplicate_executor = registry.register_render_pass_executor(executor).unwrap_err();
assert!(duplicate_executor
    .to_string()
    .contains("render pass executor weather.volumetric-clouds already registered"));
```

Update `WeatherRuntimePlugin::register_runtime_extensions` to register the weather executor and assert catalog merge:

```rust
registry.register_render_pass_executor(RenderPassExecutorRegistration::new(
    "weather.volumetric-clouds",
    weather_render_executor,
))?;
```

Then add to `runtime_plugin_catalog_merges_module_and_render_feature_contributions`:

```rust
assert_eq!(report.registry.render_pass_executors().len(), 1);
assert_eq!(
    report.registry.render_pass_executors()[0].executor_id().as_str(),
    "weather.volumetric-clouds"
);
```

### Testing Stage

- [ ] Run focused runtime plugin extension tests.

Run:

```powershell
cargo test -p zircon_runtime --lib plugin_extensions --locked --offline -- --nocapture
```

Expected: plugin extension registry tests pass. If the filter matches zero tests, run:

```powershell
cargo test -p zircon_runtime --lib runtime_extension_registry --locked --offline -- --nocapture
```

Debug/correction loop: fix the lowest runtime extension registry compile or behavior failure first, then rerun the same command.

### Exit Evidence

- Runtime extension registry stores executor registrations.
- Duplicate executor ids produce explicit diagnostics.
- Catalog merge preserves executor registrations.
- No `zircon_runtime` import from `zircon_plugins` was added.

## Milestone 2: Runtime Construction Propagates Executor Registrations

- Goal: explicit executor registrations reach `SceneRenderer` and replace descriptor-created no-op executors at graph execution time.
- In-scope behaviors: `GraphicsModule`, module descriptor factory, render framework construction, scene renderer construction, executor registry replacement tests.
- Dependencies: Milestone 1.
- Lightweight checks: scoped `cargo check -p zircon_runtime --lib --locked --offline` only if constructor signature churn is unclear.

### Implementation Slices

- [ ] Update `RenderPassExecutorRegistry` in `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs`.

Add method:

```rust
pub fn register_explicit_executors(
    &mut self,
    registrations: impl IntoIterator<Item = RenderPassExecutorRegistration>,
) {
    for registration in registrations {
        self.register(registration.executor_id, registration.executor);
    }
}
```

Add constructor:

```rust
pub fn with_builtin_noop_executors_for_render_features_and_executor_registrations(
    render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    executor_registrations: impl IntoIterator<Item = RenderPassExecutorRegistration>,
) -> Self {
    let mut registry = Self::with_builtin_noop_executors_for_render_features(render_features);
    registry.register_explicit_executors(executor_registrations);
    registry
}
```

Add test executor and test:

```rust
fn explicit_virtual_geometry_executor(
    context: &RenderPassExecutionContext,
) -> Result<(), String> {
    if context.executor_id.as_str() == "virtual-geometry.prepare" {
        return Err("explicit virtual geometry executor called".to_string());
    }
    Ok(())
}

#[test]
fn explicit_executor_registration_replaces_descriptor_noop_executor() {
    let descriptor = plugin_virtual_geometry_descriptor();
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
        [descriptor],
        [RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            explicit_virtual_geometry_executor,
        )],
    );
    let error = registry
        .execute(&RenderPassExecutionContext::new(
            "plugin-virtual-geometry-registry",
            RenderPassExecutorId::new("virtual-geometry.prepare"),
        ))
        .unwrap_err();
    assert_eq!(error, "explicit virtual geometry executor called");
}
```

- [ ] Add `RenderPassExecutorRegistration` fields through construction signatures.

Update these files to pass `Vec<RenderPassExecutorRegistration>` alongside render features and VG providers:

```text
zircon_runtime/src/builtin/runtime_modules.rs
zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs
zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new.rs
zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
```

Use empty `Vec::new()` from no-plugin constructors. In `runtime_modules_for_target_with_plugin_registration_reports`, collect:

```rust
let render_pass_executors = registrations
    .iter()
    .flat_map(|registration| registration.extensions.render_pass_executors().iter().cloned())
    .collect::<Vec<_>>();
```

Pass `&render_pass_executors` into the graphics module construction path.

- [ ] Update `SceneRenderer::new_with_icon_source_and_plugin_render_features` to install executor registrations.

Target assignment:

```rust
render_pass_executors:
    RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
        render_features,
        render_pass_executors,
    ),
```

- [ ] Update existing call sites and tests that construct `WgpuRenderFramework::new_with_plugin_render_features` or `SceneRenderer::new_with_plugin_render_features` to pass `Vec::new()` for executor registrations unless the test is specifically about explicit executors.

Search command:

```powershell
rg --line-number "new_with_plugin_render_features\(|new_with_icon_source_and_plugin_render_features\(|with_render_extensions\(|module_descriptor_with_render_features\(" "zircon_runtime/src"
```

Expected: every call site has the updated executor registration argument.

### Testing Stage

- [ ] Run focused executor registry tests.

Run:

```powershell
cargo test -p zircon_runtime --lib render_pass_executor --locked --offline -- --nocapture
```

Expected: executor registry tests pass, including explicit registration replacement. If constructor churn causes broad compile failures, fix call sites before changing behavior.

### Exit Evidence

- Explicit plugin executor registrations can replace descriptor-created no-op executor functions.
- Runtime construction path carries executor registrations from plugin reports to `SceneRenderer`.
- No runtime dependency on plugin crates.

## Milestone 3: Plugin Executor Registration

- Goal: VG/HGI plugin runtime crates register metadata-safe executor functions for all declared render feature pass ids.
- In-scope behaviors: plugin `lib.rs` executor functions, registration tests, runtime-owned types consumed by plugin crates.
- Dependencies: Milestones 1 and 2.
- Lightweight checks: none before testing stage unless plugin compile breaks before all functions are written.

### Implementation Slices

- [ ] Update `zircon_plugins/virtual_geometry/runtime/src/lib.rs` imports.

Add:

```rust
use zircon_runtime::graphics::{
    RenderPassExecutionContext, RenderPassExecutorRegistration,
};
```

- [ ] In `VirtualGeometryRuntimePlugin::register_runtime_extensions`, register pass executors after `register_render_feature`.

Target shape:

```rust
registry.register_render_feature(render_feature_descriptor())?;
for registration in render_pass_executor_registrations() {
    registry.register_render_pass_executor(registration)?;
}
registry.register_virtual_geometry_runtime_provider(
    virtual_geometry_runtime_provider_registration(),
)
```

- [ ] Add VG executor registration functions.

Target code:

```rust
pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new("virtual-geometry.prepare", virtual_geometry_prepare_executor),
        RenderPassExecutorRegistration::new("virtual-geometry.node-cluster-cull", virtual_geometry_node_cluster_cull_executor),
        RenderPassExecutorRegistration::new("virtual-geometry.page-feedback", virtual_geometry_page_feedback_executor),
        RenderPassExecutorRegistration::new("virtual-geometry.visbuffer", virtual_geometry_visbuffer_executor),
        RenderPassExecutorRegistration::new("virtual-geometry.debug-overlay", virtual_geometry_debug_overlay_executor),
    ]
}

fn virtual_geometry_prepare_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn virtual_geometry_node_cluster_cull_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn virtual_geometry_page_feedback_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn virtual_geometry_visbuffer_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn virtual_geometry_debug_overlay_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}
```

These are intentionally metadata-safe no-op pass bodies until a later neutral GPU command context exists.

- [ ] Update VG plugin test `virtual_geometry_registration_contributes_render_feature_descriptor`.

Add assertions:

```rust
assert_eq!(report.extensions.render_pass_executors().len(), 5);
assert_eq!(
    report
        .extensions
        .render_pass_executors()
        .iter()
        .map(|registration| registration.executor_id().as_str())
        .collect::<Vec<_>>(),
    vec![
        "virtual-geometry.prepare",
        "virtual-geometry.node-cluster-cull",
        "virtual-geometry.page-feedback",
        "virtual-geometry.visbuffer",
        "virtual-geometry.debug-overlay",
    ]
);
```

- [ ] Update `zircon_plugins/hybrid_gi/runtime/src/lib.rs` imports.

Add:

```rust
use zircon_runtime::graphics::{
    RenderPassExecutionContext, RenderPassExecutorRegistration,
};
```

- [ ] In `HybridGiRuntimePlugin::register_runtime_extensions`, register pass executors after `register_render_feature`.

Target shape:

```rust
registry.register_module(module_descriptor())?;
registry.register_render_feature(render_feature_descriptor())?;
for registration in render_pass_executor_registrations() {
    registry.register_render_pass_executor(registration)?;
}
Ok(())
```

- [ ] Add HGI executor registration functions.

Target code:

```rust
pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new("hybrid-gi.scene-prepare", hybrid_gi_scene_prepare_executor),
        RenderPassExecutorRegistration::new("hybrid-gi.trace-schedule", hybrid_gi_trace_schedule_executor),
        RenderPassExecutorRegistration::new("hybrid-gi.resolve", hybrid_gi_resolve_executor),
        RenderPassExecutorRegistration::new("hybrid-gi.history", hybrid_gi_history_executor),
    ]
}

fn hybrid_gi_scene_prepare_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn hybrid_gi_trace_schedule_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn hybrid_gi_resolve_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}

fn hybrid_gi_history_executor(
    _context: &RenderPassExecutionContext,
) -> Result<(), String> {
    Ok(())
}
```

- [ ] Update HGI plugin test `hybrid_gi_registration_contributes_render_feature_descriptor`.

Add assertions:

```rust
assert_eq!(report.extensions.render_pass_executors().len(), 4);
assert_eq!(
    report
        .extensions
        .render_pass_executors()
        .iter()
        .map(|registration| registration.executor_id().as_str())
        .collect::<Vec<_>>(),
    vec![
        "hybrid-gi.scene-prepare",
        "hybrid-gi.trace-schedule",
        "hybrid-gi.resolve",
        "hybrid-gi.history",
    ]
);
```

### Testing Stage

- [ ] Run plugin package tests.

Run:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline -- --nocapture
```

Expected: both plugin test suites pass. Debug/correction loop starts from public runtime export errors first, then plugin registration assertions.

### Exit Evidence

- VG/HGI plugin crates register one executor per declared pass id.
- Plugin tests assert executor registration ids.
- Registered functions remain metadata-safe and do not require runtime-private GPU state.

## Milestone 4: Neutral Readback / Output DTOs

- Goal: define and store neutral VG/HGI renderer output/readback payloads without exposing plugin concrete GPU/resource types.
- In-scope behaviors: DTO declarations under framework render contracts, scene-renderer mailbox storage/accessors, deterministic roundtrip tests.
- Dependencies: Milestones 1 through 3.
- Lightweight checks: scoped runtime check if DTO type fanout is unclear.

### Implementation Slices

- [ ] Create `zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs`.

The file should declare neutral record structs with derives `Clone`, `Debug`, `Default`, `PartialEq`, and `Eq` where all fields are integer/array/vector data. Include at least these declarations:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderPluginRendererOutputs {
    pub virtual_geometry: RenderVirtualGeometryReadbackOutputs,
    pub hybrid_gi: RenderHybridGiReadbackOutputs,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryReadbackOutputs {
    pub page_table_entries: Vec<u32>,
    pub completed_page_assignments: Vec<RenderVirtualGeometryPageAssignmentRecord>,
    pub page_replacements: Vec<RenderVirtualGeometryPageReplacementRecord>,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedClusterRecord>,
    pub visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Record>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    pub node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub launch_worklist_snapshots: Vec<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageAssignmentRecord {
    pub page_id: u64,
    pub physical_slot: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageReplacementRecord {
    pub old_page_id: u64,
    pub new_page_id: u64,
    pub physical_slot: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometrySelectedClusterRecord {
    pub cluster_id: u64,
    pub instance_id: u32,
    pub source: RenderVirtualGeometrySelectedClusterSource,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryVisBuffer64Record {
    pub value: u64,
    pub source: RenderVirtualGeometryVisBuffer64Source,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiReadbackOutputs {
    pub cache_entries: Vec<RenderHybridGiCacheEntryRecord>,
    pub completed_probe_ids: Vec<u32>,
    pub completed_trace_region_ids: Vec<u32>,
    pub probe_irradiance_rgb: Vec<[u16; 3]>,
    pub probe_rt_lighting_rgb: Vec<[u16; 3]>,
    pub scene_prepare: RenderHybridGiScenePrepareReadbackOutputs,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiCacheEntryRecord {
    pub key: u64,
    pub value: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiScenePrepareReadbackOutputs {
    pub occupied_atlas_slots: Vec<u32>,
    pub occupied_capture_slots: Vec<u32>,
    pub atlas_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub capture_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub voxel_clipmap_ids: Vec<u32>,
    pub voxel_samples: Vec<RenderHybridGiScenePrepareSample>,
    pub voxel_occupancy: Vec<u32>,
    pub voxel_cells: Vec<RenderHybridGiVoxelCellRecord>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub texture_layers: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiScenePrepareSample {
    pub index: u32,
    pub rgba8: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiVoxelCellRecord {
    pub clipmap_id: u32,
    pub cell_id: u32,
    pub occupancy: u32,
}
```

Use existing VG framework record types from `virtual_geometry_debug_snapshot.rs` where they already exist. Do not import plugin crate types.

- [ ] Update `zircon_runtime/src/core/framework/render/mod.rs` to declare and export `plugin_renderer_outputs` types.

Add module declaration near other render contracts:

```rust
mod plugin_renderer_outputs;
```

Add public exports for every new DTO type.

- [ ] Add mailbox storage to `SceneRendererAdvancedPluginOutputs`.

In `scene_renderer_advanced_plugin_outputs.rs`, add fields:

```rust
pub(in crate::graphics::scene::scene_renderer::core) plugin_renderer_outputs:
    crate::core::framework::render::RenderPluginRendererOutputs,
```

If `Default` derive no longer works, implement `Default` explicitly and preserve existing VG cull/render-path/indirect fields.

- [ ] Add accessors and storage helpers.

In `output_access.rs`, add:

```rust
pub(crate) fn plugin_renderer_outputs(
    &self,
) -> &crate::core::framework::render::RenderPluginRendererOutputs {
    &self.plugin_renderer_outputs
}

pub(crate) fn has_virtual_geometry_gpu_readback(&self) -> bool {
    !self.plugin_renderer_outputs.virtual_geometry.page_table_entries.is_empty()
        || !self.plugin_renderer_outputs.virtual_geometry.selected_clusters.is_empty()
        || !self
            .plugin_renderer_outputs
            .virtual_geometry
            .visbuffer64_entries
            .is_empty()
}
```

In `output_storage.rs`, add a setter:

```rust
pub(in crate::graphics::scene::scene_renderer::core) fn store_plugin_renderer_outputs(
    &mut self,
    outputs: crate::core::framework::render::RenderPluginRendererOutputs,
) {
    self.plugin_renderer_outputs = outputs;
}
```

- [ ] Update `SceneRendererAdvancedPluginReadbacks` to carry a neutral snapshot.

In `scene_renderer_advanced_plugin_readbacks.rs`, change it from a zero-field struct to:

```rust
use crate::core::framework::render::RenderPluginRendererOutputs;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginReadbacks {
    outputs: RenderPluginRendererOutputs,
}

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn new() -> Self {
        Self {
            outputs: RenderPluginRendererOutputs::default(),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn from_outputs(
        outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self { outputs }
    }
}
```

In `collect_into_outputs.rs`, store the snapshot:

```rust
outputs.store_plugin_renderer_outputs(self.outputs);
```

- [ ] Add runtime tests for neutral DTO roundtrip.

If access is private, place tests inside the nearest existing module that can access it. Cover:

```rust
#[test]
fn advanced_plugin_readbacks_collect_neutral_plugin_renderer_outputs() {
    let mut outputs = SceneRendererAdvancedPluginOutputs::default();
    let readbacks = SceneRendererAdvancedPluginReadbacks::from_outputs(RenderPluginRendererOutputs {
        virtual_geometry: RenderVirtualGeometryReadbackOutputs {
            page_table_entries: vec![1, 2, 3],
            ..RenderVirtualGeometryReadbackOutputs::default()
        },
        hybrid_gi: RenderHybridGiReadbackOutputs {
            completed_probe_ids: vec![7, 9],
            ..RenderHybridGiReadbackOutputs::default()
        },
    });

    readbacks
        .collect_into_outputs(test_device(), &mut outputs)
        .expect("neutral readback collection");

    assert_eq!(outputs.plugin_renderer_outputs().virtual_geometry.page_table_entries, vec![1, 2, 3]);
    assert_eq!(outputs.plugin_renderer_outputs().hybrid_gi.completed_probe_ids, vec![7, 9]);
}
```

If creating a `wgpu::Device` fixture is too expensive for this private test, split storage into a pure helper and test the helper without a device.

### Testing Stage

- [ ] Run focused runtime tests.

Run:

```powershell
cargo test -p zircon_runtime --lib advanced_plugin_readbacks --locked --offline -- --nocapture
cargo test -p zircon_runtime --lib plugin_renderer_outputs --locked --offline -- --nocapture
```

Expected: filters run at least one test each or the plan executor records the exact replacement filters discovered from module names. Debug/correction loop starts with DTO visibility and private test placement.

### Exit Evidence

- Neutral VG/HGI DTOs are public through `zircon_runtime::core::framework::render`.
- Scene renderer host state can store and expose neutral plugin renderer outputs.
- No plugin concrete types appear in runtime DTO declarations.

## Milestone 5: Plugin Readback / Output Wiring

- Goal: wire deferred VG/HGI plugin renderer folders against plugin-local converters plus neutral runtime DTOs without using runtime-private scene-renderer internals.
- In-scope behaviors: VG `root_state_readbacks`, HGI `post_process_sources`, HGI `root_output_sources`, module wiring, old-owner path gates.
- Dependencies: Milestone 4.
- Lightweight checks: plugin package `cargo check` if wiring compile failures are broad.

### Implementation Slices

- [ ] Wire VG `root_state_readbacks` in `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/mod.rs`.

Add:

```rust
mod root_state_readbacks;
```

Only keep this line if all files in that folder compile without stale runtime owner assumptions.

- [ ] Replace VG stale `SceneRenderer` inherent assumptions.

For files such as:

```text
zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/read_node_and_cluster_cull_traversal_records.rs
zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/take_gpu_completion_parts.rs
zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_state_readbacks/take_gpu_readback.rs
```

Use plugin-local functions that take explicit neutral/plugin inputs, for example:

```rust
use zircon_runtime::core::framework::render::RenderVirtualGeometryReadbackOutputs;

pub(crate) fn read_node_and_cluster_cull_traversal_records(
    outputs: &RenderVirtualGeometryReadbackOutputs,
) -> &[RenderVirtualGeometryNodeAndClusterCullTraversalRecord] {
    &outputs.node_cluster_cull.traversal_records
}
```

Do not write `impl SceneRenderer` in plugin crates.

- [ ] Convert VG root output helpers to emit neutral DTOs.

In `root_output_sources/virtual_geometry_readback_outputs.rs` and conversion helpers, add functions that build `RenderVirtualGeometryReadbackOutputs` from plugin-local `VirtualGeometryGpuReadback` or `VirtualGeometryGpuReadbackCompletionParts`.

Accepted shape:

```rust
pub(crate) fn neutral_virtual_geometry_readback_outputs(
    readback: &VirtualGeometryGpuReadback,
) -> RenderVirtualGeometryReadbackOutputs {
    RenderVirtualGeometryReadbackOutputs {
        page_table_entries: readback.page_table_entries().to_vec(),
        selected_clusters: readback
            .selected_clusters()
            .iter()
            .map(|cluster| RenderVirtualGeometrySelectedClusterRecord {
                cluster_id: cluster.cluster_id,
                instance_id: cluster.instance_id,
                source: cluster.source,
            })
            .collect(),
        ..RenderVirtualGeometryReadbackOutputs::default()
    }
}
```

Adjust field names to the actual plugin-local accessor shapes. Keep the neutral output type from runtime and all concrete interpretation in plugin code.

- [ ] Wire HGI `post_process_sources` and `root_output_sources` in `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/mod.rs`.

Add:

```rust
mod post_process_sources;
mod root_output_sources;
```

Only keep these lines if all files in those folders compile against neutral/plugin-local imports.

- [ ] Replace HGI stale runtime frame/scene-renderer assumptions.

For `root_output_sources/scene_prepare_resources.rs` and `scene_renderer_hybrid_gi/*`, remove imports of nonexistent plugin-local `scene_renderer` or runtime-private readback modules. Convert functions to accept explicit plugin-local readback or neutral runtime DTO inputs.

Accepted shape:

```rust
use zircon_runtime::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs;

pub(crate) fn scene_prepare_resources(
    outputs: &RenderHybridGiReadbackOutputs,
) -> &RenderHybridGiScenePrepareReadbackOutputs {
    &outputs.scene_prepare
}
```

- [ ] Convert HGI readback helpers to emit neutral DTOs.

Use plugin-local `HybridGiGpuReadback`, `HybridGiGpuReadbackCompletionParts`, and `HybridGiScenePrepareResourcesSnapshot` accessors. Emit `RenderHybridGiReadbackOutputs` and `RenderHybridGiScenePrepareReadbackOutputs` from `zircon_runtime::core::framework::render`.

- [ ] Run old-owner search gates after each plugin subtree is wired.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
```

Expected: no live Rust code hits.

### Testing Stage

- [ ] Run plugin package checks first.

Run:

```powershell
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline
```

Expected: both checks pass. Debug/correction loop starts from old-owner imports and visibility scope errors before changing behavior.

- [ ] Run plugin package tests.

Run:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline -- --nocapture
```

Expected: both package suites pass or exact in-scope blockers are documented.

### Exit Evidence

- VG `root_state_readbacks` is wired without `impl SceneRenderer` or runtime-private output access.
- HGI `post_process_sources` and `root_output_sources` are wired without HGI-specific runtime frame state.
- Plugin package checks/tests pass or exact blockers are recorded.

## Milestone 6: Deferred Test Promotion And Documentation

- Goal: promote renderer tests that now use neutral/plugin-local boundaries, update docs, and close the session with evidence.
- In-scope behaviors: moved test module declarations, package validation, docs, session closeout.
- Dependencies: Milestone 5.
- Lightweight checks: no additional checks before testing stage.

### Implementation Slices

- [ ] Promote VG renderer test sources only when imports are clean.

Candidate files under `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/`:

```text
virtual_geometry_gpu.rs
virtual_geometry_prepare_render.rs
virtual_geometry_args_source_authority.rs
virtual_geometry_execution_args_authority.rs
virtual_geometry_execution_stats.rs
virtual_geometry_node_and_cluster_cull_execution.rs
virtual_geometry_submission_authority.rs
virtual_geometry_submission_execution_order.rs
virtual_geometry_unified_indirect.rs
```

Add `#[cfg(test)]` module declarations in `virtual_geometry/mod.rs` only for files that compile with plugin-local and neutral runtime imports.

- [ ] Promote HGI renderer test sources only when imports are clean.

Candidate files under `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/`:

```text
hybrid_gi_gpu.rs
hybrid_gi_gpu_scene_light_seed.rs
hybrid_gi_gpu_runtime_source.rs
hybrid_gi_resolve_render.rs
hybrid_gi_resolve_history.rs
hybrid_gi_resolve_dynamic_lights.rs
hybrid_gi_resolve_surface_cache.rs
hybrid_gi_scene_prepare_material_fixtures.rs
```

Add `#[cfg(test)]` module declarations in `hybrid_gi/mod.rs` only for files that compile with plugin-local and neutral runtime imports.

- [ ] Update `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md`.

Ensure frontmatter includes:

```yaml
plan_sources:
  - docs/superpowers/specs/2026-05-02-plugin-renderer-neutral-readback-execution-surface-design.md
  - docs/superpowers/plans/2026-05-02-plugin-renderer-neutral-readback-execution-surface.md
```

Add implementation files for executor registration, neutral DTOs, and VG plugin readback/output wiring. Add the final VG validation commands and pass/fail evidence.

- [ ] Update `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md`.

Ensure frontmatter includes the same spec/plan sources. Add implementation files for executor registration, neutral DTOs, and HGI plugin post-process/root-output wiring. Add the final HGI validation commands and pass/fail evidence.

- [ ] Update `.codex/sessions/20260502-0132-plugin-renderer-neutral-execution-design.md` with final status.

If complete, move it to `.codex/sessions/archive/20260502-0132-plugin-renderer-neutral-execution-design.md` and set `status: completed`. If blockers remain, keep it active and record exact blocker commands/output summaries.

### Testing Stage

- [ ] Run final scoped runtime validation.

Run:

```powershell
cargo test -p zircon_runtime --lib render_pass_executor --locked --offline -- --nocapture
cargo test -p zircon_runtime --lib plugin_extensions --locked --offline -- --nocapture
cargo test -p zircon_runtime --lib advanced_plugin_readbacks --locked --offline -- --nocapture
```

Expected: all focused runtime suites pass or exact blockers are recorded.

- [ ] Run final plugin validation.

Run:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --offline -- --nocapture
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline -- --nocapture
```

Expected: both plugin suites pass.

- [ ] Run final search and whitespace gates.

Run:

```powershell
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry"
rg --line-number "crate::graphics::types|crate::graphics::scene::scene_renderer|pub\(in crate::graphics|crate::graphics::runtime|crate::graphics::tests" "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi"
rg --line-number "zircon_plugins" "zircon_runtime/src"
git diff --check
```

Expected: old-owner searches have no live Rust code hits; runtime dependency-direction search has no production imports; `git diff --check` has no whitespace errors. LF-to-CRLF warnings may appear and must be reported separately.

### Exit Evidence

- Runtime executor, plugin extension, and neutral readback tests pass.
- VG/HGI plugin package tests pass.
- Docs list implementation files, plan sources, and validation evidence.
- Session note is archived on success or records exact blockers if not complete.

## Final Acceptance Checklist

- [ ] Plugin crates register pass executor functions through runtime-owned APIs.
- [ ] Runtime graph execution validates and dispatches explicit executor ids without importing plugin crates.
- [ ] Neutral readback/output DTOs live under `zircon_runtime::core::framework::render` and contain no plugin concrete types.
- [ ] Scene renderer host state stores neutral plugin renderer outputs.
- [ ] VG `root_state_readbacks` no longer assumes inherent methods on runtime `SceneRenderer`.
- [ ] HGI `post_process_sources` and `root_output_sources` no longer assume HGI-specific runtime frame state.
- [ ] `ViewportRenderFrame` remains neutral.
- [ ] Deferred renderer tests are promoted only after old-owner path gates are clean.
- [ ] No compatibility shims, bridge modules, alias re-exports, or old owner paths are introduced.
- [ ] Scoped runtime/plugin validation passes or exact blockers are documented.

# Render Plugin Boundary Hardening Milestone Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Treat HGI resolve-runtime fixture construction, `SceneRendererAdvancedPluginOutputs` aggregate narrowing, and the remaining VG/GI runtime/renderer DTO field-layout audit as one independent render-plugin boundary-hardening milestone.

**Architecture:** Work bottom-up: first remove raw fixture construction of `HybridGiResolveRuntime`, then tighten the renderer advanced-output aggregate owner, then audit remaining runtime/readback/completion/stat DTO seams. All crossings must use constructor, accessor, builder, or one-shot `into_parts(...)` handoff methods instead of reading or constructing field layouts from sibling packages.

**Tech Stack:** Rust workspace under `zircon_runtime`, VG/HGI runtime plugin crates under `zircon_plugins`, Markdown module docs under `docs/assets-and-rendering`, live coordination under `.codex/sessions`.

---

## Mandatory Skill Workflow

Before any implementation message or code edit for this milestone, load and follow these skills when applicable:

- `using-superpowers`: check skills before responding or acting.
- `cross-session-coordination`: run the 4-hour coordination scan before overlap-sensitive edits and update `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`.
- `continuous-milestone-execution`: keep advancing this independent milestone until complete unless a real product-level branch ambiguity appears.
- `layered-milestone-development`: keep work ordered from lower shared DTO/fixture seams to renderer aggregate seams to broader audit/validation.
- `zircon-dev`, `zircon-dev-workflow`, and `zircon-dev-validation`: stay on the current `main` checkout, avoid worktrees/branches unless the user changes policy, and use repository Cargo validation rules.
- `zr-module-boundary-discipline` and `modularize-large-files`: split helpers into folder-backed modules instead of adding large mixed sections to existing files.
- `code-module-docs-maintenance`: update `docs/assets-and-rendering/render-framework-architecture.md`, `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md`, or `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md` when code boundaries change.
- `verification-before-completion`: no green/completion claim without fresh command evidence.
- `test-driven-development`: use red/green tests only when the user lifts the current no-`cargo test` gate; otherwise record the deferred test command and run compile-only validation.

Current user constraint to preserve unless explicitly changed: do not run new `cargo test` during milestone implementation; use targeted `rustfmt --check`, scoped `git diff --check`, `cargo check`, and plugin-crate `cargo check`. Final milestone acceptance may run tests only after the user allows the test gate.

Do not create commits unless the user explicitly requests a commit.

## File Structure

### Existing files to modify

- `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/mod.rs`: re-export the test-only fixture builder.
- `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/resolve_runtime.rs`: keep production `HybridGiResolveRuntime::new(...)` and eventually make runtime fields private after fixture migration.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/scene_renderer_advanced_plugin_outputs.rs`: add aggregate-local subowner accessors and make aggregate fields private.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs`: replace aggregate field reads with subowner accessors.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs`: replace aggregate field writes with mutable subowner accessors.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_cull_access.rs`: replace aggregate field reads with subowner accessors.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_render_path_access.rs`: replace aggregate field reads with subowner accessors.
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_indirect_access.rs`: replace aggregate field reads with subowner accessors.
- `docs/assets-and-rendering/render-framework-architecture.md`: record renderer advanced-output aggregate narrowing and final DTO audit evidence.
- `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md`: record HGI resolve-runtime fixture-builder and private-field closure.
- `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`: keep live state, blockers, validation evidence, and next step current.

### New file to create

- `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/test_builder.rs`: `#[cfg(test)]` builder for HGI resolve-runtime fixtures, keeping tests away from the owner field layout.

### Fixture/test files to migrate in batches

- `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/trace_region_inputs.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_trace_regions/encode.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs`
- `zircon_runtime/src/graphics/tests/hybrid_gi_resolve_history.rs`
- `zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs`
- `zircon_runtime/src/graphics/tests/hybrid_gi_runtime.rs`

## Task 1: Add HGI Resolve Runtime Test Builder

**Files:**
- Create: `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/test_builder.rs`
- Modify: `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/mod.rs`
- Modify: `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`

- [ ] **Step 1: Confirm the raw-literal baseline**

Run:

```powershell
Select-String -Path zircon_runtime\src\**\*.rs -Pattern 'HybridGiResolveRuntime\s*\{' | Measure-Object
```

Expected: a non-zero count, currently broad enough to require batched migration.

- [ ] **Step 2: Add the test builder module**

Create `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/test_builder.rs` with this content:

```rust
use std::collections::{BTreeMap, BTreeSet};

use super::{
    HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};

#[derive(Default)]
pub(crate) struct HybridGiResolveRuntimeTestBuilder {
    probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
    trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    probe_parent_probes: BTreeMap<u32, u32>,
    probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
    probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    probe_scene_driven_hierarchy_irradiance_ids: BTreeSet<u32>,
    probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet<u32>,
    probe_scene_driven_hierarchy_irradiance_quality_q8: BTreeMap<u32, u8>,
    probe_scene_driven_hierarchy_irradiance_freshness_q8: BTreeMap<u32, u8>,
    probe_scene_driven_hierarchy_irradiance_revision: BTreeMap<u32, u32>,
    probe_scene_driven_hierarchy_rt_lighting_quality_q8: BTreeMap<u32, u8>,
    probe_scene_driven_hierarchy_rt_lighting_freshness_q8: BTreeMap<u32, u8>,
    probe_scene_driven_hierarchy_rt_lighting_revision: BTreeMap<u32, u32>,
}

impl HybridGiResolveRuntimeTestBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_probe_scene_data(
        mut self,
        probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
    ) -> Self {
        self.probe_scene_data = probe_scene_data;
        self
    }

    pub(crate) fn with_trace_region_scene_data(
        mut self,
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    ) -> Self {
        self.trace_region_scene_data = trace_region_scene_data;
        self
    }

    pub(crate) fn with_probe_parent_probes(
        mut self,
        probe_parent_probes: BTreeMap<u32, u32>,
    ) -> Self {
        self.probe_parent_probes = probe_parent_probes;
        self
    }

    pub(crate) fn with_probe_rt_lighting_rgb(
        mut self,
        probe_rt_lighting_rgb: BTreeMap<u32, [u8; 3]>,
    ) -> Self {
        self.probe_rt_lighting_rgb = probe_rt_lighting_rgb;
        self
    }

    pub(crate) fn with_probe_hierarchy_resolve_weight_q8(
        mut self,
        probe_hierarchy_resolve_weight_q8: BTreeMap<u32, u16>,
    ) -> Self {
        self.probe_hierarchy_resolve_weight_q8 = probe_hierarchy_resolve_weight_q8;
        self
    }

    pub(crate) fn with_probe_hierarchy_irradiance_rgb_and_weight(
        mut self,
        probe_hierarchy_irradiance_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    ) -> Self {
        self.probe_hierarchy_irradiance_rgb_and_weight =
            probe_hierarchy_irradiance_rgb_and_weight;
        self
    }

    pub(crate) fn with_probe_hierarchy_rt_lighting_rgb_and_weight(
        mut self,
        probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap<u32, [u8; 4]>,
    ) -> Self {
        self.probe_hierarchy_rt_lighting_rgb_and_weight =
            probe_hierarchy_rt_lighting_rgb_and_weight;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_ids(
        mut self,
        probe_scene_driven_hierarchy_irradiance_ids: BTreeSet<u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_ids =
            probe_scene_driven_hierarchy_irradiance_ids;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_ids(
        mut self,
        probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet<u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_ids =
            probe_scene_driven_hierarchy_rt_lighting_ids;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_quality_q8(
        mut self,
        quality: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_quality_q8 = quality;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_freshness_q8(
        mut self,
        freshness: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_freshness_q8 = freshness;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_irradiance_revision(
        mut self,
        revision: BTreeMap<u32, u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_irradiance_revision = revision;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_quality_q8(
        mut self,
        quality: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_quality_q8 = quality;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_freshness_q8(
        mut self,
        freshness: BTreeMap<u32, u8>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_freshness_q8 = freshness;
        self
    }

    pub(crate) fn with_probe_scene_driven_hierarchy_rt_lighting_revision(
        mut self,
        revision: BTreeMap<u32, u32>,
    ) -> Self {
        self.probe_scene_driven_hierarchy_rt_lighting_revision = revision;
        self
    }

    pub(crate) fn build(self) -> HybridGiResolveRuntime {
        HybridGiResolveRuntime::new(
            self.probe_scene_data,
            self.trace_region_scene_data,
            self.probe_parent_probes,
            self.probe_rt_lighting_rgb,
            self.probe_hierarchy_resolve_weight_q8,
            self.probe_hierarchy_irradiance_rgb_and_weight,
            self.probe_hierarchy_rt_lighting_rgb_and_weight,
            self.probe_scene_driven_hierarchy_irradiance_ids,
            self.probe_scene_driven_hierarchy_rt_lighting_ids,
            self.probe_scene_driven_hierarchy_irradiance_quality_q8,
            self.probe_scene_driven_hierarchy_irradiance_freshness_q8,
            self.probe_scene_driven_hierarchy_irradiance_revision,
            self.probe_scene_driven_hierarchy_rt_lighting_quality_q8,
            self.probe_scene_driven_hierarchy_rt_lighting_freshness_q8,
            self.probe_scene_driven_hierarchy_rt_lighting_revision,
        )
    }
}
```

- [ ] **Step 3: Re-export the test builder**

Modify `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/mod.rs`:

```rust
mod packing;
mod probe_scene_data;
mod resolve_runtime;
mod scene_data_access;
mod scene_truth_access;
#[cfg(test)]
mod test_builder;
mod topology;
mod trace_region_scene_data;

pub(crate) use probe_scene_data::HybridGiResolveProbeSceneData;
pub(crate) use resolve_runtime::HybridGiResolveRuntime;
#[cfg(test)]
pub(crate) use test_builder::HybridGiResolveRuntimeTestBuilder;
pub(crate) use trace_region_scene_data::HybridGiResolveTraceRegionSceneData;
```

- [ ] **Step 4: Run compile-only validation**

Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime\src\graphics\types\hybrid_gi_resolve_runtime\*.rs
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: both `cargo check` commands finish successfully. If the builder emits unused warnings, keep it `#[cfg(test)]` and migrate at least one fixture in Task 2 before rechecking.

## Task 2: Migrate HGI GPU-Prepare Resolve Runtime Fixtures

**Files:**
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/trace_region_inputs.rs`

- [ ] **Step 1: Replace imports in each fixture module**

In each touched file, replace test imports of `HybridGiResolveRuntime` where a raw literal is only used for fixture construction with:

```rust
use crate::graphics::types::HybridGiResolveRuntimeTestBuilder;
```

Keep `HybridGiResolveRuntime` imported only when the file has helper return signatures that still name it.

- [ ] **Step 2: Replace a simple empty raw literal**

Replace this pattern:

```rust
let runtime = HybridGiResolveRuntime {
    probe_scene_data: BTreeMap::new(),
    trace_region_scene_data: BTreeMap::new(),
    probe_parent_probes: BTreeMap::new(),
    probe_rt_lighting_rgb: BTreeMap::new(),
    probe_hierarchy_resolve_weight_q8: BTreeMap::new(),
    probe_hierarchy_irradiance_rgb_and_weight: BTreeMap::new(),
    probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap::new(),
    probe_scene_driven_hierarchy_irradiance_ids: BTreeSet::new(),
    probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet::new(),
    probe_scene_driven_hierarchy_irradiance_quality_q8: BTreeMap::new(),
    probe_scene_driven_hierarchy_irradiance_freshness_q8: BTreeMap::new(),
    probe_scene_driven_hierarchy_irradiance_revision: BTreeMap::new(),
    probe_scene_driven_hierarchy_rt_lighting_quality_q8: BTreeMap::new(),
    probe_scene_driven_hierarchy_rt_lighting_freshness_q8: BTreeMap::new(),
    probe_scene_driven_hierarchy_rt_lighting_revision: BTreeMap::new(),
};
```

with:

```rust
let runtime = HybridGiResolveRuntimeTestBuilder::new().build();
```

- [ ] **Step 3: Replace populated raw literals with builder chains**

For a literal that sets maps such as `probe_scene_data`, `trace_region_scene_data`, `probe_parent_probes`, or `probe_rt_lighting_rgb`, use this pattern:

```rust
let runtime = HybridGiResolveRuntimeTestBuilder::new()
    .with_probe_scene_data(probe_scene_data)
    .with_trace_region_scene_data(trace_region_scene_data)
    .with_probe_parent_probes(probe_parent_probes)
    .with_probe_rt_lighting_rgb(probe_rt_lighting_rgb)
    .build();
```

Only include builder methods for maps/sets that are non-empty or semantically asserted by the fixture.

- [ ] **Step 4: Compile this batch**

Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime\src\graphics\types\hybrid_gi_resolve_runtime\*.rs zircon_runtime\src\graphics\scene\scene_renderer\hybrid_gi\gpu_resources\execute_prepare\runtime_trace_source.rs zircon_runtime\src\graphics\scene\scene_renderer\hybrid_gi\gpu_resources\execute_prepare\probe_quantization.rs zircon_runtime\src\graphics\scene\scene_renderer\hybrid_gi\gpu_resources\execute_prepare\execute\collect_inputs.rs zircon_runtime\src\graphics\scene\scene_renderer\hybrid_gi\gpu_resources\execute_prepare\trace_region_inputs.rs
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: compile-only test target succeeds.

## Task 3: Migrate HGI Post-Process Resolve Runtime Fixtures

**Files:**
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_trace_regions/encode.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs`

- [ ] **Step 1: Migrate trace-region encoder fixtures first**

In `encode_hybrid_gi_trace_regions/encode.rs`, replace raw `HybridGiResolveRuntime { ... }` fixtures with `HybridGiResolveRuntimeTestBuilder::new().with_trace_region_scene_data(...).build()` and add `.with_probe_scene_data(...)` only when the fixture asserts probe-scene fallback behavior.

- [ ] **Step 2: Migrate parent-chain helper constructors**

In `runtime_parent_chain.rs`, replace helper bodies returning raw literals with builder output:

```rust
fn resolve_runtime_with_parent_topology(
    probe_parent_probes: BTreeMap<u32, u32>,
) -> HybridGiResolveRuntime {
    HybridGiResolveRuntimeTestBuilder::new()
        .with_probe_parent_probes(probe_parent_probes)
        .build()
}
```

Keep helper names stable where existing tests call them.

- [ ] **Step 3: Migrate resolve-weight fixtures**

In `hybrid_gi_hierarchy_resolve_weight.rs`, replace literals that set `probe_hierarchy_resolve_weight_q8` with:

```rust
let runtime = HybridGiResolveRuntimeTestBuilder::new()
    .with_probe_hierarchy_resolve_weight_q8(probe_hierarchy_resolve_weight_q8)
    .build();
```

Add `.with_probe_parent_probes(...)` when the expected behavior depends on ancestor lookup.

- [ ] **Step 4: Migrate probe encoder fixtures in bounded groups**

In `encode_hybrid_gi_probes/encode.rs`, migrate raw literals in groups of no more than 8 fixtures per edit. After each group, run `rustfmt --edition 2021 --check` for this file and `cargo check -p zircon_runtime --tests ...` before continuing.

- [ ] **Step 5: Compile this batch**

Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\scene_renderer\post_process\resources\execute_post_process\encode_hybrid_gi_trace_regions\encode.rs zircon_runtime\src\graphics\scene\scene_renderer\post_process\resources\execute_post_process\encode_hybrid_gi_probes\runtime_parent_chain.rs zircon_runtime\src\graphics\scene\scene_renderer\post_process\resources\execute_post_process\encode_hybrid_gi_probes\hybrid_gi_hierarchy_resolve_weight.rs zircon_runtime\src\graphics\scene\scene_renderer\post_process\resources\execute_post_process\encode_hybrid_gi_probes\encode.rs
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: compile-only test target succeeds.

## Task 4: Migrate Graphics HGI Resolve Fixtures and Close Runtime Field Privacy

**Files:**
- Modify: `zircon_runtime/src/graphics/tests/hybrid_gi_resolve_history.rs`
- Modify: `zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs`
- Modify: `zircon_runtime/src/graphics/tests/hybrid_gi_runtime.rs`
- Modify: `zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime/resolve_runtime.rs`

- [x] **Step 1: Replace direct field mutation in graphics tests**

Replace mutation patterns such as:

```rust
runtime.probe_parent_probes = runtime_parent_topology([(probe_id, ancestor_probe_id)]);
```

with reconstruction through the builder:

```rust
let runtime = HybridGiResolveRuntimeTestBuilder::new()
    .with_probe_parent_probes(runtime_parent_topology([(probe_id, ancestor_probe_id)]))
    .build();
```

If the existing test starts with a populated runtime and then mutates one map, preserve the original helper by changing that helper to accept the map before `build()`.

- [x] **Step 2: Replace direct field assertions with owner queries**

Replace empty-map checks:

```rust
assert!(resolve_runtime.probe_scene_data.is_empty());
assert!(resolve_runtime.trace_region_scene_data.is_empty());
```

with existing query behavior:

```rust
assert!(resolve_runtime.probe_scene_data(1).is_none());
assert!(resolve_runtime.trace_region_scene_data(1).is_none());
```

If the test must assert global emptiness, add narrowly named owner methods in `scene_data_access.rs`:

```rust
pub(crate) fn has_probe_scene_data(&self) -> bool {
    !self.probe_scene_data.is_empty()
}

pub(crate) fn has_trace_region_scene_data(&self) -> bool {
    !self.trace_region_scene_data.is_empty()
}
```

Then assert `!resolve_runtime.has_probe_scene_data()` or `!resolve_runtime.has_trace_region_scene_data()`.

- [x] **Step 3: Narrow `HybridGiResolveRuntime` fields to owner-module visibility**

After grep shows no external raw literals or field reads, change `resolve_runtime.rs` from:

```rust
pub(crate) probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
```

to:

```rust
probe_scene_data: BTreeMap<u32, HybridGiResolveProbeSceneData>,
```

Apply the same removal of `pub(crate)` to every field in `HybridGiResolveRuntime`.

- [x] **Step 4: Verify no raw runtime field coupling remains**

Run:

```powershell
Select-String -Path zircon_runtime\src\**\*.rs -Pattern 'HybridGiResolveRuntime\s*\{'
Select-String -Path zircon_runtime\src\**\*.rs -Pattern '\.probe_scene_data|\.trace_region_scene_data|\.probe_parent_probes|\.probe_rt_lighting_rgb|\.probe_hierarchy_resolve_weight_q8'
```

Expected: constructor literals appear only in `resolve_runtime.rs`; field hits appear only inside `hybrid_gi_resolve_runtime/**` owner methods or unrelated HGI runtime-state owner maps with the same field names.

- [x] **Step 5: Compile this batch**

Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime\src\graphics\types\hybrid_gi_resolve_runtime\*.rs zircon_runtime\src\graphics\tests\hybrid_gi_resolve_history.rs zircon_runtime\src\graphics\tests\hybrid_gi_resolve_render.rs zircon_runtime\src\graphics\tests\hybrid_gi_runtime.rs
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: both compile-only checks succeed.

## Task 5: Narrow `SceneRendererAdvancedPluginOutputs` Aggregate Owner

**Files:**
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/scene_renderer_advanced_plugin_outputs.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_cull_access.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_render_path_access.rs`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/virtual_geometry_indirect_access.rs`

- [x] **Step 1: Add aggregate-local subowner accessors**

In `scene_renderer_advanced_plugin_outputs.rs`, replace the struct declaration and add methods:

```rust
#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginOutputs {
    hybrid_gi_readback: HybridGiReadbackOutputs,
    virtual_geometry_readback: VirtualGeometryReadbackOutputs,
    virtual_geometry_cull: VirtualGeometryCullOutputs,
    virtual_geometry_render_path: VirtualGeometryRenderPathOutputs,
    virtual_geometry_indirect: VirtualGeometryIndirectOutputs,
}

impl SceneRendererAdvancedPluginOutputs {
    pub(super) fn hybrid_gi_readback(&self) -> &HybridGiReadbackOutputs {
        &self.hybrid_gi_readback
    }

    pub(super) fn hybrid_gi_readback_mut(&mut self) -> &mut HybridGiReadbackOutputs {
        &mut self.hybrid_gi_readback
    }

    pub(super) fn virtual_geometry_readback(&self) -> &VirtualGeometryReadbackOutputs {
        &self.virtual_geometry_readback
    }

    pub(super) fn virtual_geometry_readback_mut(&mut self) -> &mut VirtualGeometryReadbackOutputs {
        &mut self.virtual_geometry_readback
    }

    pub(super) fn virtual_geometry_cull(&self) -> &VirtualGeometryCullOutputs {
        &self.virtual_geometry_cull
    }

    pub(super) fn virtual_geometry_cull_mut(&mut self) -> &mut VirtualGeometryCullOutputs {
        &mut self.virtual_geometry_cull
    }

    pub(super) fn virtual_geometry_render_path(&self) -> &VirtualGeometryRenderPathOutputs {
        &self.virtual_geometry_render_path
    }

    pub(super) fn virtual_geometry_render_path_mut(
        &mut self,
    ) -> &mut VirtualGeometryRenderPathOutputs {
        &mut self.virtual_geometry_render_path
    }

    pub(super) fn virtual_geometry_indirect(&self) -> &VirtualGeometryIndirectOutputs {
        &self.virtual_geometry_indirect
    }

    pub(super) fn virtual_geometry_indirect_mut(&mut self) -> &mut VirtualGeometryIndirectOutputs {
        &mut self.virtual_geometry_indirect
    }
}
```

- [x] **Step 2: Update access files to call aggregate methods**

Replace reads like:

```rust
self.virtual_geometry_indirect.indirect_draw_count()
```

with:

```rust
self.virtual_geometry_indirect().indirect_draw_count()
```

Replace mutations like:

```rust
self.virtual_geometry_indirect.clear_indirect_args_buffer();
```

with:

```rust
self.virtual_geometry_indirect_mut().clear_indirect_args_buffer();
```

Apply the same pattern for `hybrid_gi_readback`, `virtual_geometry_readback`, `virtual_geometry_cull`, and `virtual_geometry_render_path`.

- [x] **Step 3: Update storage file to call mutable aggregate methods**

Replace:

```rust
self.virtual_geometry_cull.store(update);
```

with:

```rust
self.virtual_geometry_cull_mut().store(update);
```

Apply the same pattern for HGI readback, VG readback, render path, and indirect storage.

- [x] **Step 4: Verify aggregate fields are private**

Run:

```powershell
Select-String -Path zircon_runtime\src\graphics\scene\scene_renderer\core\scene_renderer\advanced_plugin_outputs\*.rs -Pattern 'pub\(super\) [A-Za-z_]+:'
```

Expected: no matches.

- [x] **Step 5: Compile this batch**

Run:

```powershell
rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\scene_renderer\core\scene_renderer\advanced_plugin_outputs\*.rs
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: compile-only checks succeed.

## Task 6: Run Remaining VG/GI DTO Field-Layout Leak Audit

**Files:**
- Read/audit: `zircon_runtime/src/graphics/runtime/**`
- Read/audit: `zircon_runtime/src/graphics/scene/scene_renderer/**`
- Modify docs only when no new code leak is fixed: `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`
- Modify docs plus code when a concrete leak is fixed: `docs/assets-and-rendering/render-framework-architecture.md`, `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md`, or `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md`

- [x] **Step 1: Search for raw construction of known DTO owners**

Run targeted searches:

```powershell
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern 'HybridGiGpuReadback\s*\{'
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern 'VirtualGeometryGpuReadback\s*\{'
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern 'RuntimeFeedbackBatch\s*\{'
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern 'PreparedRuntimeSubmission\s*\{'
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern 'SubmissionRecordUpdate\s*\{'
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern 'SceneRendererAdvancedPluginReadbacks\s*\{'
```

Expected: matches are either owner declarations, owner constructors, or intentionally deferred test helpers recorded in the session note.

- [x] **Step 2: Search for raw field reads of known owner DTOs**

Run targeted searches:

```powershell
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern '\.hybrid_gi_gpu_readback|\.virtual_geometry_gpu_readback|\.runtime_feedback|\.prepared_submission|\.submission_record_update'
Select-String -Path zircon_runtime\src\graphics\**\*.rs -Pattern '\.node_and_cluster_cull_|\.indirect_execution_|\.mesh_draw_submission_'
```

Expected: raw reads are inside owner methods or update DTO application methods only.

- [x] **Step 3: Fix the smallest concrete leak only, if found**

If a leak appears in production code, choose the lowest shared owner and add one of these narrow boundaries:

```rust
pub(crate) fn field_name(&self) -> FieldType {
    self.field_name
}
```

or:

```rust
pub(crate) fn into_parts(self) -> OwnerParts {
    OwnerParts { field_name: self.field_name }
}
```

Use an accessor for borrowed/read-only repeated observations and `into_parts(...)` for one-shot storage handoff. Do not widen field visibility.

- [x] **Step 4: Update docs and session state for each fixed leak**

Record the exact owner, caller, validation command, and any deferred test gate in the matching asset-rendering doc and in `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`.

- [x] **Step 5: Compile-only validation for the audit batch**

Run:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: all compile-only checks succeed.

## Task 7: Final Milestone Acceptance Gate

**Files:**
- Modify: `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`
- Modify: relevant docs touched by the milestone

- [x] **Step 1: Confirm user permission for test execution**

Ask only if the current no-`cargo test` gate is still active. If the user keeps the gate closed, record that test acceptance is deferred and do not claim full milestone completion.

- [x] **Step 2: Run final compile-only baseline regardless of test gate**

Run:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never
```

Expected: all compile-only checks succeed.

- [x] **Step 3: Run tests after the user lifted the gate**

Run focused tests first:

```powershell
cargo test -p zircon_runtime --lib hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never -- --nocapture
cargo test -p zircon_runtime --lib hybrid_gi_resolve_history --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never -- --nocapture
cargo test -p zircon_runtime --lib render_framework_bridge --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never -- --nocapture
```

Then run plugin runtime tests:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-boundary-hardening --color never -- --nocapture
```

Expected: focused tests pass. If a failure appears in an upper-layer test, apply `support-first-regression-testing` before editing upper-layer code.

Actual: after the user continued the milestone, the focused runtime tests and VG/HGI plugin runtime tests passed with fresh evidence recorded in `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md`.

- [x] **Step 4: Run scoped whitespace validation**

Run:

```powershell
git diff --check -- .codex/sessions/20260426-2305-render-feature-plugin-cutover.md docs/assets-and-rendering/render-framework-architecture.md docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md docs/assets-and-rendering/virtual-geometry-nanite-foundation.md zircon_runtime/src/graphics/types/hybrid_gi_resolve_runtime zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs
```

Expected: no whitespace errors. LF-to-CRLF warnings are acceptable if no whitespace error line is emitted.

- [x] **Step 5: Record completion or blocker status**

Update `.codex/sessions/20260426-2305-render-feature-plugin-cutover.md` with exact command evidence. If all acceptance gates pass and no handoff is needed, retire or archive the active session note according to `cross-session-coordination`.

## Self-Review

- Spec coverage: the plan covers all three user-named independent milestone areas: HGI resolve-runtime fixture construction cleanup, `SceneRendererAdvancedPluginOutputs` aggregate narrowing, and continued VG/GI DTO leak auditing.
- Placeholder scan: no `TBD`, `TODO`, or unspecified validation command is left in the task steps.
- Type consistency: all planned HGI helper names use `HybridGiResolveRuntimeTestBuilder`; renderer aggregate methods consistently use immutable and mutable subowner accessor naming.

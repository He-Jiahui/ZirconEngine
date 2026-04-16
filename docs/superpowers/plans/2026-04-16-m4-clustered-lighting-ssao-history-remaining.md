---
related_code:
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/lib.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
implementation_files:
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/lib.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
plan_sources:
  - user: 2026-04-16 continue the next clustered-lighting/SSAO/history slice and finish the remaining tasks as much as possible in one pass
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history.md
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
tests:
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/tests.rs
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_render_server --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M4 Clustered Lighting SSAO History Remaining Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把当前 `clustered lighting / SSAO / history` 主链从“有骨架”推进到“可配置、可降级、可验证”，让 quality profile 和 capability 真正参与 built-in pipeline 编译与 runtime 选择。

**Architecture:** 在 `RenderPipelineAsset` 上增加 compile options，把可选 feature 和 async compute fallback 固定成显式编译参数；再让 `RenderQualityProfile + RenderBackendCaps` 在 `WgpuRenderServer` 里映射到 compile options，并通过 `RenderStats` 暴露有效 feature 与 async-compute 退化结果。这样可以在不伪造完整 SSAO/cluster shader 的前提下，把 M4 主链收束成稳定行为边界。

**Tech Stack:** Rust, `zircon_graphics`, `zircon_render_server`, `zircon_render_graph`, headless `wgpu`

---

### Task 1: Add Compile Options For Feature Gating And Queue Fallback

**Files:**
- Modify: `zircon_graphics/src/pipeline/mod.rs`
- Test: `zircon_graphics/src/tests/pipeline_compile.rs`

- [ ] **Step 1: Write the failing compile-option tests**

```rust
#[test]
fn compile_options_can_disable_clustered_ssao_and_history_features() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
        .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
        .with_feature_disabled(BuiltinRenderFeature::HistoryResolve);

    let compiled = pipeline.compile_with_options(&test_extract(), &options).unwrap();

    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"ssao-evaluate"));
    assert!(!pass_names.contains(&"clustered-light-culling"));
    assert!(!pass_names.contains(&"history-resolve"));
    assert!(compiled.history_bindings.is_empty());
}
```

```rust
#[test]
fn compile_options_fallback_async_compute_passes_to_graphics_queue() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let options = RenderPipelineCompileOptions::default().with_async_compute(false);

    let compiled = pipeline.compile_with_options(&test_extract(), &options).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .filter(|pass| pass.queue == QueueLane::AsyncCompute)
            .count(),
        0
    );
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "ssao-evaluate" && pass.queue == QueueLane::Graphics));
}
```

- [ ] **Step 2: Run the targeted pipeline test to verify it fails**

Run: `cargo test -p zircon_graphics pipeline_compile --locked`
Expected: FAIL because `RenderPipelineCompileOptions` and `compile_with_options(...)` do not exist yet.

- [ ] **Step 3: Implement minimal compile options**

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPipelineCompileOptions {
    pub disabled_features: BTreeSet<BuiltinRenderFeature>,
    pub allow_async_compute: bool,
}

impl Default for RenderPipelineCompileOptions {
    fn default() -> Self {
        Self {
            disabled_features: BTreeSet::new(),
            allow_async_compute: true,
        }
    }
}
```

- [ ] **Step 4: Re-run the pipeline test suite**

Run: `cargo test -p zircon_graphics pipeline_compile --locked`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add zircon_graphics/src/pipeline/mod.rs zircon_graphics/src/tests/pipeline_compile.rs
git commit -m "feat: add configurable pipeline compile options"
```

### Task 2: Map Quality Profile And Capabilities Into Effective Pipeline Compilation

**Files:**
- Modify: `zircon_render_server/src/types.rs`
- Modify: `zircon_render_server/src/lib.rs`
- Modify: `zircon_render_server/src/tests.rs`
- Modify: `zircon_graphics/src/runtime/server/mod.rs`
- Test: `zircon_graphics/src/tests/render_server_bridge.rs`

- [ ] **Step 1: Write the failing render-server tests**

```rust
#[test]
fn headless_wgpu_server_falls_back_async_compute_passes_to_graphics() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server.submit_frame_extract(viewport, test_extract()).unwrap();
    let stats = server.query_stats().unwrap();

    assert!(!stats.capabilities.supports_async_compute);
    assert_eq!(stats.last_async_compute_pass_count, 0);
    assert!(stats
        .last_effective_features
        .contains(&"clustered_lighting".to_string()));
}
```

```rust
#[test]
fn quality_profile_can_disable_ssao_clustered_and_history_features() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server.submit_frame_extract(viewport, test_extract()).unwrap();
    let before = server.query_stats().unwrap().last_frame_history;

    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("forward-lite")
                .with_screen_space_ambient_occlusion(false)
                .with_clustered_lighting(false)
                .with_history_resolve(false),
        )
        .unwrap();
    server.submit_frame_extract(viewport, test_extract()).unwrap();
    let stats = server.query_stats().unwrap();

    assert_ne!(before, stats.last_frame_history);
    assert!(!stats
        .last_effective_features
        .contains(&"screen_space_ambient_occlusion".to_string()));
}
```

- [ ] **Step 2: Run the targeted render-server tests to verify they fail**

Run: `cargo test -p zircon_graphics render_server_bridge --locked`
Expected: FAIL because `RenderQualityProfile` and `RenderStats` do not expose the new toggles/effective results yet.

- [ ] **Step 3: Add façade quality toggles and effective stats**

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureQualitySettings {
    pub clustered_lighting: bool,
    pub screen_space_ambient_occlusion: bool,
    pub history_resolve: bool,
    pub allow_async_compute: bool,
}
```

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub active_viewports: usize,
    pub submitted_frames: u64,
    pub captured_frames: u64,
    pub last_generation: Option<u64>,
    pub last_pipeline: Option<RenderPipelineHandle>,
    pub last_frame_history: Option<FrameHistoryHandle>,
    pub last_quality_profile: Option<String>,
    pub last_effective_features: Vec<String>,
    pub last_async_compute_pass_count: usize,
    pub capabilities: RenderCapabilitySummary,
}
```

- [ ] **Step 4: Compile built-in pipelines through quality/capability options inside `WgpuRenderServer`**

```rust
let compile_options = compile_options_for_profile(
    record.quality_profile.as_ref(),
    &state.stats.capabilities,
);
let compiled_pipeline = pipeline_asset
    .compile_with_options(&extract, &compile_options)
    .map_err(RenderServerError::Backend)?;
```

- [ ] **Step 5: Re-run the targeted render-server and façade tests**

Run: `cargo test -p zircon_graphics render_server_bridge --locked`
Expected: PASS

Run: `cargo test -p zircon_render_server --locked`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add zircon_render_server/src/types.rs zircon_render_server/src/lib.rs zircon_render_server/src/tests.rs zircon_graphics/src/runtime/server/mod.rs zircon_graphics/src/tests/render_server_bridge.rs
git commit -m "feat: gate m4 behavior features through quality profiles"
```

### Task 3: Update Architecture Docs And Run Expanded Validation

**Files:**
- Modify: `docs/assets-and-rendering/srp-rhi-render-server-architecture.md`

- [ ] **Step 1: Update the architecture document**

```markdown
- `RenderQualityProfile` 现在除了 pipeline override，还能控制 clustered lighting / SSAO / history resolve 与 async-compute 偏好
- `WgpuRenderServer` 会把 quality profile 和 capability 映射成 `RenderPipelineCompileOptions`
- headless `wgpu` 基线在 `supports_async_compute = false` 时会把 M4 compute pass cleanly 退化到 graphics queue，而不是泄漏 backend 约束到 façade
```

- [ ] **Step 2: Run validation**

Run: `cargo test -p zircon_graphics pipeline_compile --locked`
Expected: PASS

Run: `cargo test -p zircon_graphics render_server_bridge --locked`
Expected: PASS

Run: `cargo test -p zircon_render_server --locked`
Expected: PASS

Run: `cargo test -p zircon_graphics --lib --locked`
Expected: PASS

Run: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add docs/assets-and-rendering/srp-rhi-render-server-architecture.md
git commit -m "docs: record configurable m4 behavior layer"
```

## Self-Review

- 这份计划补的是当前主链真正剩下的行为闭环：`quality/caps -> compile options -> effective runtime result`。
- 它没有越级伪造完整 SSAO 或 clustered shader，而是把当前架构下最关键的配置与降级边界真正打通。
- 命名保持和现有骨架一致：`RenderPipelineCompileOptions`、`RenderFeatureQualitySettings`、`last_effective_features`、`last_async_compute_pass_count`。

## Inline Execution Choice

用户已经明确要求继续下一条并尽量一次性完成剩余任务，因此本计划写完后直接进入 inline execution，不等待额外确认。

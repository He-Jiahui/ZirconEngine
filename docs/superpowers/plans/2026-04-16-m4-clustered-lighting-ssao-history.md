---
related_code:
  - zircon_graphics/src/extract/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
implementation_files:
  - zircon_graphics/src/extract/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
plan_sources:
  - user: 2026-04-16 clustered lighting / SSAO / history 作为下一条 M4 行为层主链并要求生成详细 task 后持续执行
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
tests:
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/tests.rs
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_render_server --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M4 Clustered Lighting SSAO History Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在现有 Forward+ / Deferred built-in pipeline 之上，落地可编译、可统计、可持续演进的 `frame history + clustered lighting + SSAO` M4 行为层骨架，并把 viewport runtime history 正式挂进 `RenderServer`。

**Architecture:** 先把跨帧资源需求从 `FrameHistoryHandle` 空壳扩成显式 contract，再让 `RenderFeatureDescriptor` / `RenderPipelineAsset` 能声明和聚合 history binding，最后把 `WgpuRenderServer` 升级为每 viewport 持有 history state 和 visibility history 的稳定宿主。`clustered lighting` 与 `SSAO` 只落 feature/pass/runtime 边界，不在这一轮伪造完整 shader 行为。

**Tech Stack:** Rust, `zircon_graphics`, `zircon_render_server`, `zircon_scene`, `zircon_render_graph`, headless `wgpu`

---

### Task 1: Define Frame History Contracts

**Files:**
- Create: `docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history.md`
- Modify: `zircon_graphics/src/extract/mod.rs`
- Modify: `zircon_graphics/src/feature/mod.rs`
- Modify: `zircon_graphics/src/pipeline/mod.rs`
- Test: `zircon_graphics/src/tests/pipeline_compile.rs`

- [ ] **Step 1: Write the failing history-compile test**

```rust
#[test]
fn deferred_pipeline_compiles_history_bindings_for_ssao_and_history_resolve() {
    let pipeline = RenderPipelineAsset::default_deferred();

    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .history_bindings
            .iter()
            .map(|binding| (binding.slot, binding.access))
            .collect::<Vec<_>>(),
        vec![
            (FrameHistorySlot::AmbientOcclusion, FrameHistoryAccess::ReadWrite),
            (FrameHistorySlot::SceneColor, FrameHistoryAccess::ReadWrite),
        ]
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p zircon_graphics deferred_pipeline_compiles_history_bindings_for_ssao_and_history_resolve --locked`
Expected: FAIL because `CompiledRenderPipeline` does not expose `history_bindings` and the built-in deferred feature set does not declare any history usage yet.

- [ ] **Step 3: Implement minimal frame-history types and compile aggregation**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameHistorySlot {
    SceneColor,
    AmbientOcclusion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameHistoryAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameHistoryBinding {
    pub slot: FrameHistorySlot,
    pub access: FrameHistoryAccess,
}

impl FrameHistoryBinding {
    pub const fn read_write(slot: FrameHistorySlot) -> Self {
        Self {
            slot,
            access: FrameHistoryAccess::ReadWrite,
        }
    }
}
```

- [ ] **Step 4: Run the targeted pipeline test to verify it passes**

Run: `cargo test -p zircon_graphics deferred_pipeline_compiles_history_bindings_for_ssao_and_history_resolve --locked`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history.md zircon_graphics/src/extract/mod.rs zircon_graphics/src/feature/mod.rs zircon_graphics/src/pipeline/mod.rs zircon_graphics/src/tests/pipeline_compile.rs
git commit -m "feat: add frame history compile contracts"
```

### Task 2: Wire Built-In Clustered Lighting SSAO History Features

**Files:**
- Modify: `zircon_graphics/src/feature/mod.rs`
- Modify: `zircon_graphics/src/pipeline/mod.rs`
- Test: `zircon_graphics/src/tests/pipeline_compile.rs`

- [ ] **Step 1: Write the failing pipeline-order tests**

```rust
#[test]
fn default_forward_plus_pipeline_compiles_clustered_ssao_and_history_passes() {
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Opaque,
            RenderPassStage::Transparent,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "ssao-evaluate",
            "clustered-light-culling",
            "opaque-mesh",
            "transparent-mesh",
            "post-process",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cargo test -p zircon_graphics pipeline_compile --locked`
Expected: FAIL because `AmbientOcclusion` stage and the new built-in features do not exist yet.

- [ ] **Step 3: Implement the new built-in feature descriptors**

```rust
pub enum BuiltinRenderFeature {
    Mesh,
    DeferredGeometry,
    DeferredLighting,
    ClusteredLighting,
    ScreenSpaceAmbientOcclusion,
    HistoryResolve,
    Shadows,
    PostProcess,
    DebugOverlay,
    // ...
}
```

```rust
RenderFeatureDescriptor::new(
    "clustered_lighting",
    vec!["view".into(), "lighting".into(), "visibility".into()],
    vec![RenderFeaturePassDescriptor::new(
        RenderPassStage::Lighting,
        "clustered-light-culling",
        QueueLane::Compute,
    )],
)
```

- [ ] **Step 4: Update built-in Forward+ and Deferred pipelines**

```rust
stages: vec![
    RenderPassStage::DepthPrepass,
    RenderPassStage::Shadow,
    RenderPassStage::AmbientOcclusion,
    RenderPassStage::Lighting,
    RenderPassStage::Opaque,
    RenderPassStage::Transparent,
    RenderPassStage::PostProcess,
    RenderPassStage::Overlay,
]
```

- [ ] **Step 5: Re-run the pipeline test suite**

Run: `cargo test -p zircon_graphics pipeline_compile --locked`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add zircon_graphics/src/feature/mod.rs zircon_graphics/src/pipeline/mod.rs zircon_graphics/src/tests/pipeline_compile.rs
git commit -m "feat: wire clustered lighting and ssao built-in pipelines"
```

### Task 3: Persist Viewport Frame History In Render Server

**Files:**
- Modify: `zircon_graphics/src/runtime/mod.rs`
- Modify: `zircon_graphics/src/runtime/server/mod.rs`
- Modify: `zircon_graphics/src/visibility/mod.rs`
- Modify: `zircon_render_server/src/types.rs`
- Test: `zircon_graphics/src/tests/render_server_bridge.rs`
- Test: `zircon_render_server/src/tests.rs`

- [ ] **Step 1: Write the failing render-server history tests**

```rust
#[test]
fn render_server_reuses_history_handle_until_pipeline_or_viewport_shape_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderServer::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    server.submit_frame_extract(viewport, test_extract()).unwrap();
    let first = server.query_stats().unwrap().last_frame_history;

    server.submit_frame_extract(viewport, test_extract()).unwrap();
    let second = server.query_stats().unwrap().last_frame_history;

    assert_eq!(first, second);
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cargo test -p zircon_graphics render_server_bridge --locked`
Expected: FAIL because `RenderStats` does not expose frame history and `WgpuRenderServer` does not persist viewport history state.

- [ ] **Step 3: Add runtime viewport history state**

```rust
struct ViewportFrameHistory {
    handle: FrameHistoryHandle,
    viewport_size: UVec2,
    pipeline: RenderPipelineHandle,
    generation: u64,
    bindings: Vec<FrameHistoryBinding>,
    visibility: VisibilityHistorySnapshot,
}
```

```rust
let visibility = VisibilityContext::from_extract_with_history(
    &extract,
    record.history.as_ref().map(|history| &history.visibility),
);
```

- [ ] **Step 4: Expose the last frame-history handle through render stats**

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
    pub capabilities: RenderCapabilitySummary,
}
```

- [ ] **Step 5: Re-run the render-server tests**

Run: `cargo test -p zircon_graphics render_server_bridge --locked`
Expected: PASS

Run: `cargo test -p zircon_render_server --locked`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add zircon_graphics/src/runtime/mod.rs zircon_graphics/src/runtime/server/mod.rs zircon_graphics/src/visibility/mod.rs zircon_graphics/src/tests/render_server_bridge.rs zircon_render_server/src/types.rs zircon_render_server/src/tests.rs
git commit -m "feat: persist viewport frame history in render server"
```

### Task 4: Synchronize Architecture Docs And Validate M4 Slice

**Files:**
- Modify: `docs/assets-and-rendering/srp-rhi-render-server-architecture.md`

- [ ] **Step 1: Update the architecture document for the new M4 slice**

```markdown
## M4 Clustered Lighting SSAO History Baseline

- `FrameHistoryHandle` 不再是空壳，而是由 `WgpuRenderServer` 为每个 viewport 持有并在 pipeline/viewport shape 变化时轮换
- `RenderFeatureDescriptor` 现在除了 extract section，还会声明 history binding
- built-in Forward+ / Deferred renderer 会固定编译 `ssao-evaluate`、`clustered-light-culling`、`history-resolve`
```

- [ ] **Step 2: Run the narrowed validation matrix**

Run: `cargo test -p zircon_graphics pipeline_compile --locked`
Expected: PASS

Run: `cargo test -p zircon_graphics render_server_bridge --locked`
Expected: PASS

Run: `cargo test -p zircon_render_server --locked`
Expected: PASS

Run: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add docs/assets-and-rendering/srp-rhi-render-server-architecture.md
git commit -m "docs: record m4 history and ssao architecture"
```

## Self-Review

- 计划覆盖了当前这条主链的三个依赖层次：`history contract -> built-in feature wiring -> viewport runtime history`。
- 没有留 `TODO` / `TBD` 占位，也没有把“写测试”写成空话；每个任务都给了目标测试、命令和期待结果。
- 命名在整个计划里保持一致：`FrameHistoryBinding`、`FrameHistorySlot`、`ScreenSpaceAmbientOcclusion`、`HistoryResolve`、`last_frame_history`。

## Inline Execution Choice

用户已明确要求生成 detailed task 后持续执行，并且当前会话禁止为了确认而停下。因此本计划默认按 inline execution 直接进入实现，不等待额外选择。

---
related_code:
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/history/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_render_server/src/types.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/history/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/tests/project_render.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
plan_sources:
  - user: 2026-04-16 下一段真正剩下的是把 SSAO、clustered lighting、history resolve 的真实 shader/resource/runtime 数据路径接进固定 pass 和 profile/capability 边界
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history-remaining.md
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
tests:
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_graphics history_resolve_blends_previous_scene_color_when_enabled --locked
  - cargo test -p zircon_graphics ssao_quality_profile_darkens_scene_when_enabled --locked
  - cargo test -p zircon_graphics clustered_lighting_quality_profile_applies_runtime_tile_lighting --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M4 Runtime Shader Resource Paths

**Goal:** 把 `history resolve / SSAO / clustered lighting` 从现有 compile/runtime skeleton 推进到真实可执行的 shader、resource、runtime 数据路径，并继续通过 `RenderServer + RenderPipelineCompileOptions + RenderQualityProfile` 控制启闭与退化。

**Architecture:** 保持现有 compile graph 和 façade 边界不动，真实 GPU 行为全部落在当前 legacy-compatible `SceneRenderer` 内部，作为 `RenderServer` 消费 `CompiledRenderPipeline` 的第一条 runtime 执行路径。这样可以先让 M4 行为层真正跑起来，同时不把 `wgpu` 类型泄漏回 `zircon_render_server` 或 asset SRP 层。

## Slice Order

### Task 1: History Resolve Runtime Path

- [x] 在 `SceneRenderer` 增加 server-only runtime 入口，消费 `CompiledRenderPipeline` 和 `FrameHistoryHandle`
- [x] 把 `OffscreenTarget` 拆成 `scene_color + final_color + depth`，不再直接把 base scene 画进 readback 目标
- [x] 为每个 `FrameHistoryHandle` 持有稳定的 `scene_color_history` 纹理资源
- [x] 新增 fullscreen history-resolve shader/pipeline，输入 `scene_color + previous_scene_color_history`，输出 `final_color`
- [x] render 后把 `final_color` copy 回 `scene_color_history`，形成下一帧输入
- [x] 用 integration test 证明：同一 viewport 第一帧绿色、第二帧黑色时，开启 history resolve 的第二帧仍保留可测量的绿色残留；关闭 history resolve 时不存在该残留

### Task 2: SSAO Runtime Path

- [x] 新增 normal/depth prepass 资源：`normal_texture` 和可采样 `depth_texture`
- [x] 新增 SSAO compute pass，输入 `depth + normal + previous_ao_history`，输出 `ao_texture`
- [x] 为每个 `FrameHistoryHandle` 持有稳定的 `ambient_occlusion_history` 纹理资源
- [x] post-process shader 实际采样 `ao_texture` 并调制当前 scene color
- [x] render 后把 `ao_texture` copy 回 `ambient_occlusion_history`
- [x] 用 integration test 证明：同一 scene 在 `screen_space_ambient_occlusion=true` 时平均亮度明显低于关闭 SSAO 的同场景输出

### Task 3: Clustered Lighting Runtime Path

- [x] 新增 directional-light structured buffer，把 extract lighting 数据编码进 GPU buffer
- [x] 新增 clustered-light compute pass，按 tile 写入 `cluster_lighting_buffer`
- [x] post-process shader 按像素 uv 映射到 cluster tile，并实际消费 `cluster_lighting_buffer`
- [x] cluster 数据路径必须只依赖 `CompiledRenderPipeline` feature 集和 current extract，不绕回 façade 或 ECS
- [x] 用 integration test 证明：`clustered_lighting=true` 时输出会出现可测量的亮度/色调变化；关闭 clustered lighting 时该变化消失

### Task 4: Docs And Validation

- [x] 更新 `docs/assets-and-rendering/srp-rhi-render-server-architecture.md`，把 compile skeleton 提升为真实 runtime 资源链说明
- [x] 跑 targeted tests，再跑 `cargo test -p zircon_graphics --lib --locked`
- [x] 跑 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`

## Constraints

- 不改变 `zircon_render_server` 的稳定 façade 形状；runtime 真实 GPU 资源只留在 `zircon_graphics`
- 不把 `wgpu` 原生类型塞进 `RenderStats` 或 scene extract
- `RenderQualityProfile` 和 capability 仍然只通过 `RenderPipelineCompileOptions` 生效，renderer 只消费已经编译好的有效 feature 集
- 这次不伪造完整 deferred renderer，也不提前接 Nanite/Lumen；只把 M4 这三条行为链接通

## Completion Gate

- `history resolve / SSAO / clustered lighting` 三条路径都必须有真实 shader 和真实 GPU 资源
- 行为必须能通过 `RenderQualityProfile` 启闭
- 行为必须经 `RenderServer` 驱动，而不是只在局部 demo path 生效
- 文档和验证必须同步更新

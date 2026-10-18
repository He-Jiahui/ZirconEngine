# Zircon SRP/RHI 渲染管线补全计划

## Summary

本计划把现有渲染系统从“已有 RHI/SRP/GI/VG 雏形”推进到可执行的架构闭环：`zircon_runtime::rhi -> zircon_runtime::render_graph -> zircon_runtime::graphics::pipeline/feature -> SceneRenderer 后端执行`。实现目标不是逐行移植 Unreal 或 Unity Graphics，而是复刻其结构：Unreal 的 RHI/RDG/Renderer 分层，Unity URP/HDRP 的 PipelineAsset/RendererFeature/Pass/RenderGraph 资产化组织，并把现有 Hybrid GI 与 Virtual Geometry 收束为可开关的 RenderFeature 家族。

默认采用当前仓库收束规则：不新增 `zircon_render_server`，不使用非网络 `server` 命名；公共渲染门面继续落在 `zircon_runtime::core::framework::render::RenderFramework`，底层基础设施继续作为 `zircon_runtime::{rhi,rhi_wgpu,render_graph}` 内部模块推进。

## Key Changes

- **路线图先校正**
  - 更新 `.codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md`，删除新增 `zircon_render_server` / 独立 crate 的旧表述。
  - 明确最终落点：`RenderFramework` 是外部 façade，`rhi` 是底层硬件接口，`render_graph` 是帧资源图，`graphics` 是 SRP/Renderer 实现层。
  - 同步更新 `docs/assets-and-rendering/render-framework-architecture.md`，把旧 `zircon_graphics/zircon_rhi` crate 路径改成当前 `zircon_runtime/src/...` 路径。

- **M1：补齐 RHI v1 合同**
  - 扩展 `zircon_runtime/src/rhi`：加入稳定资源句柄 `BufferHandle`、`TextureHandle`、`SamplerHandle`、`ShaderModuleHandle`、`PipelineHandle`，并把 `BufferUsage` / `TextureUsage` 改为可组合 usage flags。
  - 扩展 `RenderDevice`：提供资源创建、销毁、shader/pipeline 创建、command list 创建、queue submit、readback/fence 查询的 trait 方法。
  - `rhi_wgpu` 作为首个实现，包装现有 wgpu 能力；短期允许具体 wgpu 资源仍只在 `graphics/backend` 和 renderer GPU resources 内部可见，但禁止 editor/runtime/framework 层直接依赖 wgpu。
  - 新增边界测试：framework/editor/app 不得 import `wgpu`；RHI descriptors 不得出现 Mesh/Material/Light/Scene 等上层语义。

- **M2：把 RenderGraph 从拓扑图升级为资源图**
  - 扩展 `RenderGraphBuilder`：支持 transient texture/buffer descriptor、external resource import、pass read/write 声明、manual dependency、pass flags、queue lane。
  - `compile()` 需要产出：拓扑顺序、资源生命周期、pass culling 结果、queue lane 统计、资源读写验证错误。
  - 增加验证：cycle rejection、unknown pass/resource、read-before-producer、write-after-write 缺依赖、transient lifetime、external output root、async compute fallback。
  - 参考 Unity Core RenderGraph 的 resource registry/import/create 结构，以及 Unreal `FRDGBuilder` 的 create/register/extract/pass culling 模型。

- **M3：把 SRP asset/feature/pass 接到真实 RenderGraph**
  - 扩展 `RendererFeatureAsset`：保留 `BuiltinRenderFeature`，增加 feature-local config、quality gate、capability requirements。
  - 扩展 `RenderFeatureDescriptor`：每个 feature 声明 required extract sections、history bindings、pass descriptors、resource reads/writes、executor id。
  - `RenderPipelineAsset::compile_with_options(...)` 不再只按 stage 串 pass 名称，而是用 feature descriptors 构建完整 RenderGraph。
  - 默认 Forward+ 管线固定阶段：DepthPrepass、Shadow、Lighting、Opaque、Transparent、PostProcess、Overlay。
  - 默认 Deferred 管线固定阶段：DepthPrepass、Shadow、GBuffer、AmbientOcclusion、Lighting、Transparent、PostProcess、Overlay。

- **M4：新增 pass executor 层，接管现有 SceneRenderer 执行**
  - 在 `graphics::scene::scene_renderer` 下新增 graph execution 子模块：`RenderPassExecutorId`、`RenderPassExecutionContext`、`RenderPassExecutorRegistry`。
  - `SceneRenderer::render_frame_with_pipeline(...)` 执行流程改为：compile pipeline asset -> prepare frame runtime data -> execute compiled graph passes -> collect outputs/stats。
  - 初期 executor 复用现有渲染函数，不做大规模 shader 重写；重点是让每个实际 pass 都有 RenderGraph 节点、资源 IO 和 executor id。
  - 旧 monolithic render path 只保留为内部迁移入口，不能继续作为新的公共主链路扩展点。

- **M5：GI/VG 收束为正式 RenderFeature 家族**
  - Hybrid GI 拆成 feature runtime：scene prepare、trace schedule/update、resolve、history 四类 pass descriptor；继续消费现有 `RenderHybridGiExtract`、runtime state、surface cache、voxel scene 数据。
  - Virtual Geometry 拆成 feature runtime：prepare、node-and-cluster cull、page request/uploader feedback、visbuffer/hardware raster fallback、debug overlay pass descriptor。
  - GI/VG 都必须通过 `RenderQualityProfile` + backend caps opt-in；baseline wgpu headless 默认可编译关闭，高级路径只在 profile 和 caps 同时允许时启用。
  - `RenderStats` 继续暴露现有 GI/VG 统计，但统计来源改为 graph execution / feature runtime record，而不是散落在 submit-frame 专用路径里。

- **M6：公共 façade 与资产注册闭环**
  - 扩展 `RenderFramework`：新增 `register_pipeline_asset(RenderPipelineAsset) -> RenderPipelineHandle`，并让 `reload_pipeline(handle)` 做 graph compile validation。
  - `set_pipeline_asset(viewport, handle)` 保持现有行为，但失败时返回明确错误：unknown pipeline、capability mismatch、graph compile failure。
  - `RenderPipelineAsset::default_forward_plus()` 和 `default_deferred()` 继续作为内建资产；后续项目资产加载只注册同一结构，不走单独 renderer 分支。
  - 保持 editor/runtime/script 只依赖 `core::framework::render` 的 DTO、handle 和 façade。

- **M7：文档与模块边界维护**
  - 更新 `docs/assets-and-rendering/render-framework-architecture.md`，记录新的 RHI/RenderGraph/SRP/Feature execution 主链。
  - 更新 `docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md` 和 `virtual-geometry-nanite-foundation.md`，说明 GI/VG 已从 submit-frame 专用路径收束为 RenderFeature runtime。
  - 文档头部 `related_code` 必须包含实际改动路径，测试命令必须写入对应 docs。

## Test Plan

- **RHI**
  - `cargo test -p zircon_runtime --locked rhi::tests`
  - 覆盖 caps、usage flags、resource descriptor roundtrip、wgpu caps 降级、unsupported acceleration structures fallback。
  - 新增源码边界测试，阻止 editor/app/framework 上层直接 import `wgpu`。

- **RenderGraph**
  - `cargo test -p zircon_runtime --locked render_graph::tests`
  - 覆盖 pass ordering、cycle rejection、resource read/write dependency、transient lifetime、external output root、pass culling、async queue fallback。

- **SRP Pipeline**
  - `cargo test -p zircon_runtime --locked graphics::tests::pipeline_compile`
  - 覆盖 Forward+ 与 Deferred 的 graph pass 序列、feature opt-in/out、capability gate、history bindings、required extract sections。
  - 新增测试证明 `RenderFeature::register_passes` 或等价 descriptor path 真的改变 compiled graph，而不是只改变 feature name list。

- **GI/VG Featureization**
  - 继续跑现有 Hybrid GI 和 Virtual Geometry focused suites：
    - `cargo test -p zircon_runtime --locked hybrid_gi_`
    - `cargo test -p zircon_runtime --locked virtual_geometry_`
    - `cargo test -p zircon_runtime --locked node_and_cluster_cull_pass_`
  - 新增测试证明关闭 GI/VG 时 graph 不包含对应 pass；开启时 pass、history、stats、debug snapshot 全部仍可观测。

- **Integration**
  - `cargo test -p zircon_runtime --locked render_framework_bridge`
  - 覆盖 viewport create/destroy、pipeline register/set/reload、submit frame extract、capture frame、query stats。
  - 最终跑：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime`。
  - 若公共 DTO 或 app/editor 边界改变，再扩展到 workspace validator。

## Assumptions

- 当前仓库继续在 `main` 现有 checkout 上开发，不创建 worktree 或分支。
- 本计划优先完成架构闭环，不要求一次性实现完整 Lumen、完整 Nanite、完整硬件 RT 或完整 ShaderGraph/VFXGraph。
- Unreal/Unity 作为结构参考，不复制其 API 命名；Zircon 保持 Rust 模块边界和当前 `zircon_runtime` 收束形态。
- 现有 Hybrid GI 与 Virtual Geometry 行为尽量保留，通过 feature runtime 迁移执行入口，而不是重写算法。
- wgpu 仍是唯一必备后端；RHI 先形成稳定抽象和 wgpu adapter，后续再逐步减少 renderer 内部直接 wgpu 使用面。

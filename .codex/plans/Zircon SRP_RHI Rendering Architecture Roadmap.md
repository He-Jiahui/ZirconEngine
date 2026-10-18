# Zircon SRP/RHI Rendering Architecture Roadmap

## Summary

当前路线已经从“新增独立渲染 crate/server”收束为 `zircon_runtime` 内部闭环：

- `zircon_runtime::core::framework::render::RenderFramework` 是 editor/runtime/script 可见的公共 facade。
- `zircon_runtime::rhi` 定义无场景语义的 RHI 资源、句柄、usage flags 与设备合同。
- `zircon_runtime::rhi_wgpu` 是第一个 wgpu 后端适配层，wgpu 细节不得越过 runtime graphics/backend 与 renderer GPU resources 边界。
- `zircon_runtime::render_graph` 负责编译帧资源图、pass 依赖、资源生命周期、pass culling 与 queue lane 统计。
- `zircon_runtime::graphics::{pipeline,feature,scene}` 负责 SRP asset/feature/pass 描述与 `SceneRenderer` 后端执行。

本路线参考 Unreal 的 RHI/RDG/Renderer 分层和 Unity URP/HDRP 的 PipelineAsset/RendererFeature/Pass/RenderGraph 组织方式，但不复制其 API 命名。

## Landed Code Direction

- RHI v1 已加入稳定资源句柄、可组合 usage flags、资源/管线/command/fence/readback 设备合同与 wgpu 基线实现。
- RenderGraph 已从 pass 拓扑图升级为资源图，支持 transient/import resource、read/write 声明、manual dependency、pass flags、queue lane、编译验证和 culling。
- `RenderFeaturePassDescriptor` 现在声明 executor id 与资源读写；`RenderPipelineAsset::compile_with_options(...)` 会把 feature descriptor 编译为完整 `CompiledRenderGraph`。
- Hybrid GI 与 Virtual Geometry 已作为 opt-in feature pass 家族进入 graph：关闭时 graph 不含对应 pass，开启时保留 history/stats/debug 可观测入口。
- `SceneRenderer::render_frame_with_pipeline(...)` 先执行 compiled graph 的 pass executor registry 钩子，再落回现有实际渲染路径，作为旧单体路径迁移入口。
- `RenderFramework` 支持 `register_pipeline_asset(...)`、`reload_pipeline(...)` graph validation、viewport pipeline set/submit/stats 闭环。

## Milestones

- M1 RHI v1 合同：已落地，继续减少 renderer 内部直接 wgpu 面。
- M2 RenderGraph 资源图：已落地，后续补资源 aliasing 与真实 command recording 对接。
- M3 SRP asset/feature/pass：已落地 graph 编译主链，后续增加项目资产加载同构注册。
- M4 pass executor 层：已落地 no-op registry 迁移钩子，后续逐个 pass 接管真实 SceneRenderer 函数。
- M5 GI/VG featureization：已落地 descriptor/graph opt-in，后续继续把 submit-frame 专用统计迁入 feature runtime record。
- M6 facade 与资产注册：已落地 register/reload/set/submit focused path，后续补 capability mismatch 细分错误。

## Focused Validation

- `cargo test -p zircon_runtime --locked rhi::tests --lib --target-dir target/codex-srp-rhi --jobs 1`
- `cargo test -p zircon_runtime --locked render_graph::tests --lib --target-dir target/codex-srp-rhi --jobs 1`
- `cargo test -p zircon_runtime --locked graphics::tests::pipeline_compile --lib --target-dir target/codex-srp-rhi --jobs 1`
- `cargo test -p zircon_runtime --locked render_framework_bridge --lib --target-dir target/codex-srp-rhi --jobs 1`
- `cargo test -p zircon_runtime --locked hybrid_gi_ --lib --target-dir target/codex-srp-rhi --jobs 1`
- `cargo test -p zircon_runtime --locked virtual_geometry_ --lib --target-dir target/codex-srp-rhi --jobs 1`
- `cargo test -p zircon_runtime --locked node_and_cluster_cull_pass_ --lib --target-dir target/codex-srp-rhi --jobs 1`

## Constraints

- 不新增 `zircon_render_server`，不使用非网络 `server` 命名。
- 不新增 `zircon_rhi` / `zircon_render_graph` 独立 crate；当前落点是 `zircon_runtime/src/{rhi,rhi_wgpu,render_graph}`。
- 上层 RHI descriptors 不出现 Mesh/Material/Light/Scene 等语义。
- editor/app/framework 层不得直接 import `wgpu`。

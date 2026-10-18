# ZirconEngine ECS 到渲染链路完善里程碑计划

## Summary
按“主链优先”推进：先把 `zircon_runtime::scene` 的自研 ECS 调度、Transform/Active 派生状态、RenderExtract 主链打通，再把材质、后处理、SRP renderer asset、WGPU 提交流程接上，最后补强 Lumen/Nanite 风格 Hybrid GI 与 Virtual Geometry 的接口深度。Bevy/Unity/Unreal 只作为结构参考，不直接引入源码或替换 Zircon 自研架构。

当前关键事实：Zircon 已有 `World` 组件表、`RenderFrameExtract`、SRP pipeline asset、VG/GI runtime provider 与 visibility planning，但 `SceneViewportRenderPacket -> RenderFrameExtract` 仍会把 VG/GI/postprocess 填成默认值，`LateUpdate` 还未硬切到 Bevy 风格 `PostUpdate`，场景派生状态仍以 `rebuild_derived_state()` 隐式重建为主。

## Reference Anchors
- Zircon 当前实现：[`SystemStage`](E:/Git/ZirconEngine/zircon_runtime/src/scene/components/schedule.rs:4)、[`World` component maps](E:/Git/ZirconEngine/zircon_runtime/src/scene/world/world.rs:16)、[`rebuild_derived_state`](E:/Git/ZirconEngine/zircon_runtime/src/scene/world/derived_state.rs:22)、[`RenderFrameExtract`](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/frame_extract.rs:127)、[`RenderVirtualGeometryExtract`](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/scene_extract.rs:225)、[`RenderHybridGiExtract`](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/scene_extract.rs:330)、[`RenderFramework`](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/framework.rs:9)。
- Bevy 参考：main schedule 顺序 [`First/PreUpdate/Update/PostUpdate/Last`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_app/src/main_schedule.rs:81)，Transform 在 `PostUpdate` 传播 [`TransformPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_transform/src/plugins.rs:22)，render extract 模式 [`ExtractComponent`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_render/src/extract_component.rs:30)，材质 GPU bind contract [`Material`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_pbr/src/material.rs:144)，winit runner/event loop [`WinitPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_winit/src/lib.rs:72)，任务池参考 [`ComputeTaskPool`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_tasks/README.md:29)。
- Unity SRP 参考：RenderGraph pass/resource 生命周期 [`RenderGraph`](E:/Git/ZirconEngine/dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs:331)，VolumeStack/VolumeManager per-camera blending [`VolumeManager.Update`](E:/Git/ZirconEngine/dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs:765)，RendererFeature 注入点 [`ScriptableRendererFeature`](E:/Git/ZirconEngine/dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRendererFeature.cs:50)，renderer pass queue [`EnqueuePass`](E:/Git/ZirconEngine/dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRenderer.cs:1008)。
- Unreal 参考：RDG builder/pass execution [`FRDGBuilder`](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h:47)，Lumen scene/card/surface-cache data [`FLumenSceneData`](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneData.h:970)，screen probe gather [`LumenScreenProbeGather`](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp:23)，Nanite packed view/raster/shading pipeline [`NaniteShared`](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteShared.h:34)，Nanite cull/raster interfaces [`NaniteCullRaster`](E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteCullRaster.h:58)。

## Key API And Data-Flow Changes
- ECS/schedule：新增 `zircon_runtime::scene::ecs`，包含 `ResourceStore`、typed `Events<T>`、`SceneSystemDescriptor`、`SceneSystemRegistry`、`SceneScheduleRunner`；`SystemStage` 硬切为 `First -> PreUpdate -> FixedUpdate -> Update -> PostUpdate -> Last -> RenderExtract`，删除 `LateUpdate` 兼容路径。
- Built-in systems：把 active hierarchy、transform propagation、node cache refresh、render extract prep 从隐式 `rebuild_derived_state()` 迁到调度系统；`PostUpdate` 先跑用户/插件 hook，再跑核心派生系统，`RenderExtract` 最后构建渲染帧。
- Render extract：`RenderFrameExtract` 成为 renderer 唯一主入口；`World::build_render_frame_extract` 直接填充 geometry/materials/lighting/post_process/visibility/VG/GI，不再通过会丢失字段的 snapshot 默认转换。
- Materials：新增 neutral `MaterialExtract` 与 `RenderMaterialSnapshot`，从现有 `MaterialAsset`/`MaterialGraphAsset` 解析 PBR 参数、alpha、double-sided、domain、shader/texture refs、pipeline key；renderer 以 material key 缓存 bind/layout，不让 `MeshRenderer` 只携带 handle+tint。
- Postprocess/Volume：新增场景侧 `PostProcessVolumeComponent` 与 `PostProcessSettings`，按 camera/world position 做 priority/weight/global blending，输出 bloom/color grading/tonemap/exposure 到 `PostProcessExtract`。
- Renderer/SRP：继续使用现有 `RenderPipelineAsset`、`RendererAsset`、`RendererFeatureAsset`，补齐 project asset load、feature descriptor validation、history/resource binding 到 `WgpuRenderFramework::submit_frame_extract` 的实际消费链。
- VG/GI：保留 opt-in capability；补强 `VirtualGeometryRuntimeProvider` 与 `HybridGiRuntimeProvider` 输入，使其能接收 material/camera/lights/visibility/update-plan/debug/state generation，并从 scene components 或 model-derived data 生成非空 extract、stats、debug snapshot。

## Milestones
- M0 Baseline Evidence：冻结当前 failing/passing 测试基线；更新引用矩阵，确认 ECS/render/material/postprocess/VG/GI 已有代码与缺口，输出到 docs 但不改行为。
- M1 ECS Schedule Foundation：实现 ResourceStore、Events、system registry/runner；硬切 `LateUpdate -> PostUpdate`；`WorldDriver::tick_level` 改为跑 native systems + legacy hooks；迁移现有 schedule tests。
- M2 Transform And Active Systems：引入 dirty flags；spawn/setter 只标脏，`PostUpdate` 统一验证 hierarchy、传播 active、传播 world transform、刷新 node cache；覆盖 parent reorder、inactive parent、static/dynamic mobility、large hierarchy。
- M3 Canonical Render Extract：重写 scene producer 直接构造 `RenderFrameExtract`；填充 visibility、camera aspect、lights、materials placeholder、postprocess defaults、VG/GI sidebands；保留 `to_scene_snapshot` 仅用于 preview/test roundtrip。
- M4 Material Runtime Contract：把 `MaterialAsset` 与 `MaterialGraphAsset` 编译成 `RenderMaterialSnapshot`；定义 PBR/domain/alpha/pipeline key；renderer 侧建立 material cache 与 missing asset fallback；补 forward/deferred material parity tests。
- M5 Postprocess And SRP Volume Stack：实现 global/local postprocess volume blending；把 bloom/color grading/tonemap/exposure 接入 `PostProcessExtract`；让 SRP feature descriptor 根据 extract requirements 决定是否记录 postprocess pass。
- M6 Renderer Submit Integration：让 `WgpuRenderFramework` 消费 compiled pipeline、material extract、postprocess extract、history bindings、viewport resize；winit/app loop 顺序固定为 window events -> scene tick -> render extract -> submit frame。
- M7 Virtual Geometry Interface Depth：用 Nanite 参考补齐 view/cull/page/feedback 概念：per-mesh VG enable、cluster/page budget、provider-generated instances/clusters/pages、visibility plan、page upload plan、debug readback；默认仍 opt-out。
- M8 Hybrid GI Interface Depth：用 Lumen 参考补齐 scene representation：probe/trace-region/card/surface-cache/voxel budget、camera-relative update plan、lights/material participation、history/debug stats；默认仍 opt-out，CPU/mock provider 可验证链路。
- M9 Documentation And Acceptance：更新 `docs/assets-and-rendering/render-framework-architecture.md`、`virtual-geometry-nanite-foundation.md`、`hybrid-gi-lumen-scene-representation.md`，新增/更新 scene ECS/render-extract 文档，并在每篇文档记录 related code、参考源码、测试证据。

## Test Plan
- 每个里程碑最后运行对应 focused tests：scene schedule/world/render extract、asset material roundtrip、pipeline compile、runtime provider、visibility planning、render framework submit/capture。
- M1-M3 重点命令：`cargo test -p zircon_runtime --lib scene::tests --locked`，并新增 transform/active/render_extract 回归。
- M4-M6 重点命令：`cargo test -p zircon_runtime --lib graphics::tests --locked`，覆盖 material key、postprocess extract、pipeline feature requirements、submit record。
- M7-M8 重点命令：现有 `m5_flagship_slots`、VG/GI runtime provider、visibility planning tests 扩展为非空 extract、debug snapshot、history/readback stats。
- 最终验收：`cargo build --workspace --locked --verbose`、`cargo test --workspace --locked --verbose`，再跑 `zircon_plugins/Cargo.toml` 的 check/build/test；若触及导出目标，补跑 export target policy tests。

## Assumptions
- 不直接引入 `bevy_ecs`；Zircon 继续保留自研 `zircon_runtime::scene::World`。
- `LateUpdate` 是旧语义，实施时硬切为 `PostUpdate`，不保留 re-export/shim。
- VG/GI 是 capability-gated flagship path：默认 pipeline 不启用，启用必须显式 opt-in 并注册 provider/descriptor。
- Unity SRP 只吸收 RenderGraph、RendererFeature、VolumeStack 的结构思想；Unreal Lumen/Nanite 只作为接口和数据流参考，不承诺同等完整算法质量。

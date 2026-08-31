---
title: Runtime Direct Lighting、Photometry、Light Grid、Shadow Atlas、Cookie、IES 与 Contact Shadow 当前工作树复核
category: zircon_runtime
report_id: Runtime185
review_date: 2026-08-30
baseline_head: 79ff31b5e6f3cf8319f809013b2f960493a1a96a
verification_head: 79ff31b5e6f3cf8319f809013b2f960493a1a96a
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/71-runtime-scene-light-directional-point-spot-rect-photometry-layer-shadow-cookie-ies-extract-authoring-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/245-editor-direct-lighting-shadow-cookie-ies-lighting-bake-current-working-tree-review.md
related_code:
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/core/framework/render/light
  - zircon_runtime/src/core/framework/render/advanced_lighting/cookie.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src
tests:
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs
  - zircon_runtime/src/graphics/tests/render_product_shadows/many_point_lights.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_wide.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/scene/tests/render_extract/lighting_postprocess.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowDepthRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LocalLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/RectLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/IESTextureManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RectLightTextureManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapArray.h
  - dev/bevy/crates/bevy_light/src/cluster/mod.rs
  - dev/bevy/crates/bevy_light/src/cluster/assign.rs
  - dev/bevy/crates/bevy_pbr/src/cluster/gpu.rs
  - dev/bevy/crates/bevy_pbr/src/render/shadow_sampling.wgsl
  - dev/godot/servers/rendering/renderer_rd/cluster_builder_rd.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/light_storage.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/area_lights_inc.glsl
  - dev/Fyrox/fyrox-impl/src/scene/light/directional.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/point.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/spot.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightLoop/LightLoop.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Light/HDAdditionalLightData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/LightUnitUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightCookieManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightLoop/CookieSampling.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/LTCAreaLight/LTCAreaLight.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/ContactShadows.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDShadowManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDShadowAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/HDCachedShadowAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ForwardLights.cs
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime185 · Direct Lighting / Shadow 当前工程化差距

## 1. 结论

当前工作树有可保留的直接光照骨架：四类 light component、scene asset 的基础颜色/强度/范围/形状字段，`GpuLightData` 的 128-byte ABI，CPU light-grid 的 tile/z-bin 结果，directional cascade 与 point/spot shadow atlas allocator，cookie metadata packing，以及 contact-shadow 的 render-graph descriptor。它们仍是局部实现，不是可交付的 Unreal/Unity 级 lighting product。

最关键的事实是：四类灯光在 `World::collect_*_lights` 中都将 `shadow` 写成 `None`；`RectLight` 明确标记 `renderer_degraded`，且 `SceneEntityAsset` 的 light schema 没有 shadow、cookie、IES、photometric unit 或 channel 字段。shadow planner 因而只能消费手工构造的 snapshot。`GpuLightData` 虽有 shadow/cookie 槽位，slot 仍以 `SHADOW_SLOT_NONE` 起始，64-bit light id 只写入低 32 位；directional cookie metadata 还会复用 position/spot ABI 字段。

light grid 在 CPU 每帧重新分配 z-bin、tile mask、min/max 与统计 scratch，并把灯数静默截断到 `u16::MAX`。spot、rect 统一按 sphere 做 bounds，async-compute descriptor 与真实 CPU assignment 并存；因此不能把它称为唯一的 clustered Forward+ GPU owner。shadow cache 只维护 identity decision，未接入 atlas 内容保留/复制，atlas 仍有整幅清除路径；contact shadow shader 只依据 depth/normal/HZB 生成全屏 occlusion，没有 light id、light direction、channel 或 per-light shadow visibility 语义。

Runtime95 的 10 项 Runtime09E P0 与 Runtime71 的两项 authoring P0 在当前树仍开放，本报告不重新登记这些唯一 P0，只刷新当前证据并拆出后续工作。本文新增 **0 项 P0、30 项 P1、12 项 P2、26 道资格门**；P1 30 Open，P2 12 Open，资格门为 24 Fail、2 Partial、0 Pass。在 source -> validated descriptor -> generation -> cluster/shadow/cookie -> submit -> capture/readiness 闭合，并有同硬件 Unreal/Unity/Fyrox/Godot 对照前，不得宣称性能或表现达到当前 Unreal。

目标架构：

```text
Light source/asset + photometry/shadow/cookie/IES/channel
  -> validated EffectiveLightDescriptor (source/effective/provenance)
  -> PreparedLightingGeneration (dense IDs, bounds, dependencies)
  -> one GPU assignment owner (cluster/tile/froxel)
  -> ShadowPolicy + PlannedShadowViewGeneration + persistent atlas/cache
  -> direct/deferred/volumetric/cookie consumers
  -> submit/readiness/fault/capture receipts
```

## 2. 当前源码证据

### 2.1 Source、scene extract 与 roundtrip

- `scene/components/scene/lighting.rs` 只有 color/intensity/range/size/angles/volumetric；没有 `LightShadowSettings`、photometric unit、source radius、temperature、IES、cookie、lighting channel 或 per-light feature mask。
- `asset/assets/scene/lighting.rs` 与 component 字段基本同构，默认值仍是 unitless（例如 rect intensity 1,000,000）；spot 默认使用独立字段但没有 schema version/migration 证明。
- `scene/world/render/lights.rs` 逐 family 收集并执行 camera layer 粗过滤，directional、point、spot、rect 全部构造 `shadow: None`；rect 同时写入 “rect light renderer shading is not implemented yet” 的 degraded reason。
- `scene/world/project_io/scene_asset.rs` 能将基础 light 字段 load/save，但 `SceneEntityAsset` 没有阴影、cookie、IES、unit、channel 的持久化字段；同一文件仍把 terrain/tilemap 等领域固定为 `None`，说明 scene schema 是增量拼接而非完整 descriptor owner。
- `Mobility` 和 `RenderLayerMask` 已进入 snapshot，但 layer 只在 GPU ABI 中保存，尚未成为 receiver/caster/direct/volumetric 的共同 admission contract。

### 2.2 GPU ABI、packing 与 readiness

- `GpuLightData` 固定 128 bytes，包含 position/range、color/intensity、direction/type、spot/rect shape、shadow slot/layer、shadow params、cookie rect/misc；这是一项真实 ABI 底座，应保留并改为 versioned reflected layout。
- `light_buffer.rs` 的 pack 函数按 family 重复解释字段，`shadow_slot_layer[0]` 初始为 `SHADOW_SLOT_NONE`，shadow plan 之后再以 light index 回写；任何中途 consumer 都可能看到未分配 slot。
- `shadow_slot_layer[2]` 只保存 `light_id as u32`，stable identity/generation 不足以抵抗超过 32-bit、复用或跨 session wrap；`cookie_misc` 同时承载 cookie projection 和 volumetric membership。
- directional cookie 的 offset/scale 会覆盖 `position_range` 与 `spot_angles_size`，说明 ABI 槽位没有按 shape/cookie/shadow 维度分离，版本演进会破坏已有 shader 解释。
- `RenderLightReadinessReport::from_light_slices` 对 directional/point/spot 直接以总数作为 ready，只有 ambient/rect 的显式 degraded 标记参与计数；它不能代表 dependency、pipeline、slot、device 或 submit readiness。

### 2.3 Light grid / clustered Forward+

- `light_grid_builder.rs` 每帧创建 `zbins`、`tile_masks`、`zbin_min_max` 与统计遍历；tile budget 通过放大 tile size 解决，不能表达 GPU compact list、cell overflow 或 quality policy。
- `lights.len()` 被 `.min(u16::MAX)` 静默截断；调用方没有 accepted/rejected list、overflow receipt 或可预测 fallback。bitmask 宽度随 light count 线性增长，dense scenes 的 VRAM/上传成本没有上限证明。
- directional 覆盖全部 tile/bin；point、spot、rect 都进入 `sphere_influence`，因此聚光锥和矩形 emitter 会过度分配或在边界漏分配，不能支持真实 oriented bounds。
- near-plane/camera-inside 使用中心投影和 `clip.w` 判断，projection mode、reversed-Z、jitter、sub-viewport、stereo/view family 没有单一 canonical projection owner；orthographic 与 visibility/cascade 的约定也没有差分证明。
- `light_grid_pass.rs` 只把 CPU 输出打包到三份 buffer upload；descriptor 中的 async compute path 与 CPU assignment 并存，不能声称有真实 GPU light assignment、barrier、queue overlap 或 shader-produced index list。
- 统计包含 non-empty/peak/average 等 CPU 重扫结果，但没有 GPU cell overflow、accepted light、assignment generation、bytes/time 或 per-view history。

### 2.4 Shadow policy、views、atlas 与 cache

- `shadow/plan.rs` 只选择第一盏 shadow-casting directional；additional lights 只处理 point 六面和 spot 单面，rect 没有 shadow request，directional cascades 直接生成 allocation/key/generation。
- cascade near plane、split config、默认级数和距离仍由 plan 内常量决定，未由 asset/quality/device/view policy 验证；directional 与 local allocation 没有同一 generation authority。
- planner 和 `graphics/visibility/view_context/build_views.rs` 各自生成 shadow view/frustum/caster query，allocation 之后才知道实际 accepted slots，存在 planned/visibility 双 authority。
- atlas allocator 有 retention/preemption/slot generation 和 tier，但没有 gutter/filter footprint、receiver bias 单位、safe UV、multi-page/virtual page 预算；point face 的原子分配失败策略也没有产品 receipt。
- `ShadowCache` 能比较 light params、static caster revision 和 atlas generation，但当前模块主要被自身 tests/plan inputs 消费；没有证明 atlas depth 内容跨帧保留、reuse hit 能跳过 static pass、dynamic overlay 只重绘移动 caster。
- `shadow_map_renderer` 为每个 slot 重放 view/pass，pipeline/resource 缺失路径仍可能 `expect`；render graph 的 clear/load policy 与 cache decision 没有统一提交合同。

### 2.5 Cookie、IES 与 contact shadow

- cookie framework/atlas 有 metadata 和固定容量路径，但 scene component/asset 没有 cookie source producer、dependency generation、IES profile 或 authoring roundtrip；cookie texture residency、mip/gutter/eviction 没有 receipt。
- IES 没有 runtime resource kind、photometric profile、normalization、spot/point projection 或 shader consumer；不能以 cookie metadata 代替 IES。
- contact-shadow plugin 注册 HZB/normal/depth 到 storage texture 的 compute pass，shader 做 12 个 screen-space depth samples 加 HZB furthest 与 grazing normal；输出是全屏 visibility scalar，不读取 light buffer、shadow slot、channel、world position、light ray 或 per-light history。
- 因此 contact shadow 与 direct-light visibility 没有乘法/优先级/temporal disocclusion 的定义，开启后会把同一 occlusion 施加给所有灯和环境，属于错误能力名而非逐光接触阴影。

### 2.6 测试与产品证据

- shadow 产品测试大量手工构造 `RenderSceneSnapshot`/light snapshot，能覆盖 allocator/layout/graph contract，却不能证明 asset -> scene -> World -> extract -> plan -> GPU capture 的 authoring 可达性。
- many-light tests 主要覆盖小规模 point lights；没有 65,535+ overflow、dense tile pressure、camera-inside、near-crossing、ortho/reversed-Z、multi-directional、rect/LTC、cookie/IES reload、cache hit/miss 或 device loss。
- 目前没有统一 per-light allocation/readiness/cookie/cluster receipt，也没有同硬件同场景与 Unreal/Unity/Fyrox/Godot 的 GPU time、VRAM、像素误差和 capture provenance 对照。

## 3. 参考引擎差异

Unreal 的 LightComponent/LocalLight/RectLight 将强度单位、mobility、channels、shadow policy、IES/cookie 与 renderer registration 作为一条 authority，并由 LightGrid、ShadowSetup、VirtualShadowMap cache 和 depth submission 共同消费。Unity HDRP 的 LightLoop、HDAdditionalLightData、LightUnitUtils、LTC area light、LightCookieManager 与 cached shadow atlas 体现了“validated source -> per-frame GPU data -> persistent allocation/cache”的分层；URP 也明确追加灯光 shadow atlas 的 capacity/fallback。Bevy 的 cluster assignment/GPU buffers 与 shadow sampling 是轻量但一致的 render extraction 对照，Godot 的 cluster builder/storage 与 area-light shader 展示了 shape-aware bounds，Fyrox 的 light/CSM 是较小的 scene/render 对照。Zircon 当前最接近“多个局部 descriptor + CPU staging”，尚未形成上述单一 authority。

## 4. P1 重构任务

| ID | 当前差异 | 必须完成 |
|---|---|---|
| RT-LGT-01 | component 没有 effective descriptor | 建立 `EffectiveLightDescriptor`，统一 shape、mobility、channels、photometry、shadow/cookie/IES 与 provenance。 |
| RT-LGT-02 | asset schema 缺 shadow source | 为四类 light 增加 versioned shadow settings、migration、unknown-field 保留和 partial-load diagnostics。 |
| RT-LGT-03 | asset schema 缺 cookie/IES | 引入 cookie/IES resource reference、projection、normalization、dependency generation 和 reload receipt。 |
| RT-LGT-04 | unitless intensity | 支持 lux/lumen/candela/nit、temperature/tint、working color space、finite/negative admission 与 roundtrip。 |
| RT-LGT-05 | extract 全部 shadow=None | source -> World -> extract 必须发布 validated shadow presence、reason、generation，禁止 renderer 猜默认。 |
| RT-LGT-06 | rect 明确 degraded | 以 LTC/area integration 消费 width/height/orientation/sidedness/source texture，并保留可解释 fallback。 |
| RT-LGT-07 | light channels 只停在 ABI | 建立 receiver/light/caster/volumetric 的共同 `LightingChannelMask`，camera 只做粗过滤。 |
| RT-LGT-08 | pack 多份解释 | 单一 `PreparedLightingGeneration` 完成 dense index、family ranges、GPU ABI、bounds 和 all-consumer indirection。 |
| RT-LGT-09 | ID 截断为 u32 | 使用 generation-qualified dense index + stable handle map，检测 reuse/wrap/cross-world collision。 |
| RT-LGT-10 | ABI 槽位复用 | 版本化 reflected layout，独立 shape/cookie/IES/shadow/channel 字段，GPU/CPU layout 差分为零。 |
| RT-LGT-11 | readiness 按 count | per-light readiness 必须依赖 validation、resource generation、pipeline、allocation、device 和 submit receipt。 |
| RT-LGT-12 | CPU grid 每帧分配 | per-view persistent workspace、capacity growth、steady-frame zero-allocation 与 upload budget。 |
| RT-LGT-13 | u16 静默截断 | 设备/quality capacity admission，返回 accepted/rejected light IDs、reason、fallback 和 telemetry。 |
| RT-LGT-14 | bitmask 线性膨胀 | 设计 compact index list/two-level/linked list 策略，cell overflow 可 resize 或按 policy degrade。 |
| RT-LGT-15 | bounds 统一 sphere | spot 使用 conservative cone，rect 使用 oriented box/frustum，GPU/CPU bounds differential 无漏光。 |
| RT-LGT-16 | projection 分散 | canonical projection/bounds library 覆盖 perspective/ortho/reversed-Z/jitter/subrect/stereo/near/camera-inside。 |
| RT-LGT-17 | 假 async compute contract | hard cutover 到唯一 assignment owner；graph access、queue、barrier、consumer read 与 encoder 命令一致。 |
| RT-LGT-18 | CPU stats 重扫 | 热路径改用 GPU counters/readback 或采样统计，debug 统计不得改变 assignment。 |
| RT-LGT-19 | directional 只取第一盏 | ShadowPolicy 按 view/importance/cost/quality 接受或拒绝全部 directional，并发布 typed receipt。 |
| RT-LGT-20 | cascade 常量化 | near/distance/count/split/fade/stabilization 来自 validated policy，支持 camera cut、quality 和 XR view family。 |
| RT-LGT-21 | views 双 authority | allocation 后发布唯一 `PlannedShadowViewGeneration`，visibility、caster query、renderer 只消费它。 |
| RT-LGT-22 | atlas 无 safe rect | allocator 纳入 gutter/padding/filter footprint、tier、multi-page pressure、UV safe rect 和 generation。 |
| RT-LGT-23 | shadow cache 无 consumer | persistent cache manager 持有物理 depth/page 内容，evaluate/commit/retain 接入 graph 和 dynamic overlay。 |
| RT-LGT-24 | per-slot command replay | planned caster packet 按 view/phase 预编译或 GPU cull，减少 uniform/bind/pass/string 分配并有预算。 |
| RT-LGT-25 | cookie atlas 固定重建 | persistent cookie residency、mip/gutter/priority/eviction、incremental upload、missing/last-good receipt。 |
| RT-LGT-26 | IES 不存在 | IES artifact/import/normalization/projection/shader sampling、cache、device fallback 与 editor bridge。 |
| RT-LGT-27 | contact shadow 全屏 | 迁入逐光 direct visibility，重建 world position，沿 light ray march，支持 channels/history/disocclusion；否则删除能力声明。 |
| RT-LGT-28 | error 可能 panic | pipeline/resource/device/OOM/resize 走 typed outcome、last-good 或显式 degrade，渲染线程不 `expect`。 |
| RT-LGT-29 | 产品测试为 snapshot fixture | 增加 asset/component -> save/reopen -> World -> extract -> plan -> GPU capture 的四类灯光矩阵。 |
| RT-LGT-30 | 缺性能与 fault corpus | 建 1/64/128/4K/65,535+/overflow、cache thrash、device loss、resize、same-hardware visual/perf corpus。 |

## 5. P2 增强任务

| ID | 演进方向 | 前置资格 |
|---|---|---|
| RT-LGT-P2-01 | Virtual Shadow Map page table/clipmap/physical page cache | allocator、generation、cache hit 和 GPU Scene 先闭合。 |
| RT-LGT-P2-02 | stochastic many-light/MegaLights/reservoir/denoise | 正确 cluster、物理光度和 reference capture 先稳定。 |
| RT-LGT-P2-03 | hardware ray-traced/hybrid per-light shadows | BLAS/TLAS、ray-query capability、fallback parity 先闭合。 |
| RT-LGT-P2-04 | ray-query contact/soft shadow | per-light contact semantics、history/disocclusion 和 budget 先闭合。 |
| RT-LGT-P2-05 | EVSM/MSM/VSM moment filtering | atlas boundary、precision、bleeding 与 temporal metrics 先具备。 |
| RT-LGT-P2-06 | disk/tube/sphere/polygon/mesh emitters | LTC rect、solid angle、photometry 与 oriented bounds 先闭合。 |
| RT-LGT-P2-07 | spectral/SPD photometry | RGB units、working space、BRDF energy conservation oracle 先闭合。 |
| RT-LGT-P2-08 | adaptive light LOD/importance | accepted/rejected receipt、hysteresis、overflow 和 profile 先闭合。 |
| RT-LGT-P2-09 | explicit copy/async queue shadow streaming | queue/fence ownership、graph barriers 和 device recovery 先闭合。 |
| RT-LGT-P2-10 | XR multiview/foveated cluster/shadow allocation | view-family、foveation map 和双眼 consistency gate 先闭合。 |
| RT-LGT-P2-11 | path-traced direct-light oracle | asset units、BRDF、exposure、capture provenance 和 tolerance 先稳定。 |
| RT-LGT-P2-12 | neural/stochastic shadow reconstruction | 正确非神经 baseline、模型 provenance、fallback 和平台成本先闭合。 |

## 6. 资格门

| 门 | 当前结果 | 关闭证据 |
|---|---|---|
| light source schema | Fail | 四类 light 的 shadow/cookie/IES/unit/channel 字段有 versioned roundtrip。 |
| effective descriptor | Fail | component/asset 到 renderer 只经过一份 validated descriptor。 |
| shadow authoring reachability | Fail | asset -> World -> extract 能产生非 None shadow 并保留 reason/generation。 |
| rect area light | Fail | LTC/等价 area integration、orientation、sidedness、bounds 与 pixel regression。 |
| photometry/color | Fail | lux/lumen/candela/nit、temperature、working space、finite/negative validation。 |
| lighting channels | Fail | receiver/caster/cluster/direct/volumetric/camera mask 一致。 |
| GPU ABI | Partial | 128-byte ABI 与 layout tests 存在，但槽位复用、ID 截断和 generation 未解决。 |
| readiness | Fail | 每灯依赖、pipeline、slot、device、submit receipt 驱动。 |
| light-grid capacity | Fail | 设备预算、overflow receipt、accepted/rejected 名单和 fallback。 |
| light-grid bounds | Fail | spot/rect oriented conservative bounds，near/camera-inside/ortho/reversed-Z differential。 |
| GPU assignment owner | Fail | CPU/async compute 二选一且 graph/queue/barrier/consumer 一致。 |
| grid memory/perf | Fail | persistent workspace、steady allocation、bytes/time/overflow metrics。 |
| shadow policy | Fail | multi-directional、importance/cost/quality/hysteresis receipt。 |
| planned shadow views | Fail | allocation 后唯一 view generation 被 visibility/caster/renderer 消费。 |
| cascade policy | Fail | validated near/split/count/fade/stabilization 与 camera/XR policy。 |
| atlas allocator | Partial | tier/retention/preemption 存在，但 gutter/safe rect/page pressure 不完整。 |
| shadow cache | Fail | persistent depth/cache content、hit/miss/invalidation、dynamic overlay 进入 graph。 |
| shadow submission | Fail | per-slot pass 无 panic、批量 caster packet、pipeline/device outcome。 |
| cookie residency | Fail | scene producer、persistent atlas、mip/gutter/eviction/reload receipt。 |
| IES | Fail | resource/import/normalization/projection/shader/editor roundtrip。 |
| contact shadow semantics | Fail | per-light world-space ray visibility、mask/history/disocclusion 或移除能力。 |
| scene product path | Fail | save/reopen/PIE/standalone 到 GPU capture 的四类灯光闭环。 |
| tests | Fail | overflow/extreme projection/cache/rect/cookie/IES/device-loss 产品矩阵。 |
| diagnostics | Fail | per-light allocation/readiness/degrade/cache/cluster/shadow/cookie metrics。 |
| fault recovery | Fail | OOM/device loss/resize/stale generation 无黑帧谎报或 panic。 |
| performance parity | Fail | 同硬件同场景与 Unreal/Unity/Fyrox/Godot 的 CPU/GPU/VRAM/像素误差报告。 |

## 7. 实施顺序

1. 先完成 Runtime09E/Runtime71 的 parent P0 characterizations，并让真实 scene asset 能产生 shadow settings。
2. 建立 `EffectiveLightDescriptor` 与 `PreparedLightingGeneration`，同步修正 ABI、ID、channel、readiness 和 scene roundtrip。
3. 选择唯一 light assignment owner，完成 conservative bounds、overflow receipt 和 GPU/CPU differential。
4. 收敛 shadow policy/view/atlas/cache，随后接入 cookie/IES 与逐光 contact visibility。
5. 最后用 asset-to-capture、fault、scale、soak 和同硬件 reference corpus 关闭产品资格门。

本轮仅写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Cargo/GPU/PIE 动态验证。

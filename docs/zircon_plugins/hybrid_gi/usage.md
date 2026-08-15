---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/participation.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/source_ledger.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures/project_documents.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/types/hybrid_gi_prepare/voxel_cell.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
implementation_files:
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/participation.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/source_ledger.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures/project_documents.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/types/hybrid_gi_prepare/voxel_cell.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
plan_sources:
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/Hybrid GI 计算机图形学合集工程映射.md
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - user: 2026-07-12 参照计算机图形学合集完善 HybridGI 与渲染功能使用
tests:
  - tools/tests/test_hybrid_gi_editor_profile.py
  - tools/tests/test_hybrid_gi_m4_contract.py
  - zircon_runtime/tests/hybrid_gi_m4_source_ledger.rs
  - zircon_runtime/tests/hybrid_gi_m4_source_ledger_wgpu.rs
  - zircon_plugins/hybrid_gi/runtime/tests/hybrid_gi_m4_invalidation_profiles.rs
  - zircon_plugins/hybrid_gi/runtime/tests/hybrid_gi_m4_profile_matrix_wgpu.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/advanced_providers.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/hybrid_gi/runtime/src/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_render_framework_stats.rs
  - docs/tests/runtime/render/plan18_hybrid_gi_scene_representation_only_forward_deferred_wgpu_20260710.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_source_ledger_wgpu_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_profile_forward_deferred_wgpu_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_mobility_roundtrip_wgpu_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_moving_emissive_wgpu_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_profile_forward_deferred_renderdoc_20260713_capture.rdc
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_profile_forward_deferred_renderdoc_20260713_capture_8.rdc
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_renderdoc_ui_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_renderdoc_hybrid_passes_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_renderdoc_handoff_passes_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_m4_renderdoc_api_validation_20260713.png
  - docs/tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_20260714.md
  - docs/tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_actual_20260714.png
  - docs/tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_fallback_20260714.png
doc_type: workflow-detail
---

# Hybrid GI 使用指南

## 当前产品边界

当前可用路径是 `DynamicOnly` 语义的 Lumen-style Hybrid GI。它从普通 mesh、材质、方向光、点光、聚光和 Emissive 场景抽取派生 card、Surface Cache、screen probe、Radiance Cache 与 voxel fallback，不要求项目手工布置 probe 或 trace region。

`BakedStaticDynamic` 已进入 HGI-M4 核心代码候选：Plan 11 EL-M3 提供版本化 lightmap/probe baseline，HGI 消费只读合同；surface participation、per-probe source ledger、dynamic-delta 权重、generation/epoch 时序签名、场景失效签名和四个 profile 预算已经接入。Plan 11 的外部 fixture、真实 WGPU readback 和产品 PNG 详见 [`Environment Lightmap And Probe Consumption`](../../assets-and-rendering/environment-lightmap-probe-consumption.md)。

这仍不代表混合模式通过产品总门。当前已验证 Runtime production build、静态合同、完整 post-process WGSL Naga 校验、生产 29-binding 后处理来源账本 WGPU readback、四 Profile 的 Forward+/Deferred 2x4 固定场景，以及 mobility 与 moving Emissive 双管线跨帧往返。HybridGI runtime 已显式启用它实际导入的 Runtime `graphics` 特性；standalone official production build 已通过。完整 crate test phase 执行 library 119、invalidation/profile 2、Forward+/Deferred WGPU 产品 3，共 124 项，0 failed / 0 ignored。Editor10 项目 fixture 已切到 manifest-owned roots 与公开 project writer，拆分独立 `project_documents.rs` 后由 coordinator 将 [`hybrid-gi-project-fixture-api-drift`](../../plans/zircon_runtime/render/18/fixed-2026-07-13-hybrid-gi-project-fixture-api-drift.md) 回传 Render18。合法 baked+dynamic delta 增加能量，非法 baked+full dynamic 与 baked baseline 逐像素一致；四档双管线 parity MAE 均低于 `0.02`，两个往返恢复帧 MAE 均为 `0`。同一四 Profile 产品测试已经由 RenderDoc 1.44/DX12 留存 8 份可重放 `.rdc` 并完成 pass/API validation 审计。Editor 实际值诊断已完成 provider -> runtime stats -> pane payload 代码链、Runtime fallback/stale-state 行为测试和 Editor unit。Navigation 05 已修复回传；真实窗口验证了第一方 provider 注册、WGPU viewport、异步首帧重试和组合图 `scene-velocity` 合同。2026-07-14 当前源码产品构建与 Custom actual / IndoorStatic missing-bake fallback 两张真实 PNG 已通过，解析后预算与回退可见且 viewport 非空。`FullyDynamic`/`Custom DynamicOnly` 可直接用于产品验证；`BakedStaticDynamic` 只有在有效 Plan 11 baked contract 存在时生效，缺失时确定性降级。broad/full 验证仍开放，因此不关闭整个 HGI-M4。

## 编辑器启用

标准编辑器目标 `target-editor-host` 会编译第一方高级渲染 provider 目录。`EntryProfile::Editor` 默认请求 `HybridGlobalIllumination`，但不请求 `VirtualGeometry`；provider 目录虽同时编译 HGI、Solari 和 Virtual Geometry，运行时注册仍按 render profile 选择，默认只链接 HGI。

每个 retained editor viewport 会安装 `editor-viewport-default` 质量配置，并在场景抽取没有显式预算时填入：

| 参数 | 编辑器默认值 | 作用 |
|---|---:|---|
| `trace_budget` | 32 | screen probe 数量与每 probe ray 分配的总预算 |
| `card_budget` | 64 | 本帧可驻留/更新的 Surface Cache card/page 预算 |
| `voxel_budget` | 16 | 软件 voxel clipmap 的驻留/更新预算 |
| `quality` | `Medium` | 每 trace tile 的实际方向数，Low/Medium/High 为 4/8/16 |

显式非零预算、质量档和 debug-view 请求不会被编辑器默认值覆盖。当前执行器把 `debug_view` 编码进 HGI resolve uniform，并在最终 HGI lighting 合成前选择当前帧诊断输出；非 `None` 模式禁用时域复用，避免旧历史污染调试颜色。

### 编辑器快速切换 Profile

编辑器启动前设置 `ZIRCON_EDITOR_HYBRID_GI_PROFILE`，即可让所有 retained viewport 使用同一产品 Profile：

```powershell
$env:ZIRCON_EDITOR_HYBRID_GI_PROFILE = 'fully-dynamic'
$env:ZIRCON_EDITOR_HYBRID_GI_PROFILE = 'indoor-static'
$env:ZIRCON_EDITOR_HYBRID_GI_PROFILE = 'open-world'
$env:ZIRCON_EDITOR_HYBRID_GI_PROFILE = 'cinematic'
$env:ZIRCON_EDITOR_HYBRID_GI_PROFILE = 'custom'
```

连字符和下划线写法都可用于前三个复合名称。未设置、空值或无法识别的值按 `custom` 处理。`custom` 使用上表的编辑器默认预算；其他 Profile 把 viewport override 保持为零，由 provider 解析为下节列出的产品预算，避免编辑器默认值覆盖 Profile。

切换后应在 `Runtime Diagnostics` 核对 `Hybrid GI effective` 和 `Hybrid GI budgets`，不要只根据环境变量判断实际生效状态。`indoor-static`、`open-world`、`cinematic` 在项目没有有效 Plan 11 baked contract 时会显示请求的 Profile，但实际模式降级为 `dynamic-only`，并报告 `baked-lighting-unavailable`。

构建标准编辑器：

```powershell
python tools/zircon_build.py --target editor
```

等价 Cargo 聚合目标是 `zircon_app --no-default-features --features target-editor-host`。不要只启用 `zircon_runtime/target-editor-host`，那只提供 runtime/editor 域合同，不提供 App 侧第一方 HGI provider 目录。

## 运行时显式启用

客户端运行时默认不启用 HGI。嵌入式入口需要同时满足两个条件：

1. 构建 `zircon_app` 时加入 `first-party-advanced-render-runtime-plugins`。
2. 项目或入口的 `RenderProfileBundle` 请求 `RenderProductFeature::HybridGlobalIllumination`。

```rust
let render_profile = RenderProfileBundle::default_render()
    .with_features([RenderProductFeature::HybridGlobalIllumination]);
let config = EntryConfig::new(EntryProfile::Runtime)
    .with_render_profile(render_profile);
```

只编译 provider 而不请求产品 feature 不会建立 HGI Pass；只请求 feature 而未编译/注册 provider 会得到结构化降级报告，不会偷偷改走测试 fixture。

## 每视口配置

`RenderFrameExtract::lighting.hybrid_global_illumination` 接受 `RenderHybridGiExtract`：

```rust
extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
    enabled: true,
    mode: RenderHybridGiMode::DynamicOnly,
    profile: RenderHybridGiProfile::Custom,
    quality: RenderHybridGiQuality::High,
    trace_budget: 64,
    card_budget: 128,
    voxel_budget: 32,
    debug_view: RenderHybridGiDebugView::None,
});
```

`Custom` 的预算为零表示该工作族没有可用工作量；在编辑器中零值会被 viewport 默认值补齐，在客户端运行时中应由项目明确给出。质量档会改变真实 ray direction 数，不只是改标签或 march 常量。

开发验证 baked + dynamic：

```rust
extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
    enabled: true,
    mode: RenderHybridGiMode::BakedStaticDynamic,
    profile: RenderHybridGiProfile::IndoorStatic,
    ..RenderHybridGiExtract::default()
});
```

该模式要求 `EnvironmentExtract` 同帧提供 Plan 11 的 `LightmapConsumeContract`。静态灯不会再次进入 HGI seed，movable directional/point/spot 与动态 surface/emissive 作为 delta；每个 probe 还携带 stable primitive key、source mask 和 dynamic weight。缺失 baked 合同时实际模式会降级为 `DynamicOnly`，`fallback_reason` 为 `BakedLightingUnavailable`，不得把请求值显示成已生效值。

## Profile 与实际预算

| Profile | 实际模式 | trace | card | voxel | 说明 |
|---|---|---:|---:|---:|---|
| `FullyDynamic` | `DynamicOnly` | 96 | 192 | 96 | 可破坏场景和动态昼夜 |
| `IndoorStatic` | `BakedStaticDynamic` | 64 | 256 | 64 | lightmap baseline + 近场动态 delta |
| `OpenWorld` | `BakedStaticDynamic` | 64 | 192 | 128 | 静态远场与较高 voxel streaming 预算 |
| `Cinematic` | `BakedStaticDynamic` | 192 | 512 | 192 | 固定 warmup/采集的高质量预算 |
| `Custom` | 显式 `mode` | 显式 | 显式 | 显式 | 调用方负责预算 |

profile 给出默认预算；调用方提供的非零预算覆盖对应默认值。可用 `RenderHybridGiExtract::resolved_settings(baked_lighting_available)` 查询最终模式、预算和降级原因；renderer neutral sideband 也保存同一 `resolved_settings`，供产品诊断显示实际值。

`voxel_budget` 是 clipmap 驻留/更新的工作预算，不等于 clipmap 层数。当前 V1 最多驻留 8 层；更大的 profile 预算用于更新选择与后续分页治理，不能通过指数扩大层级半径消耗。运行时会拒绝非有限的 clipmap center、half extent 和 mesh bounds，避免错误配置把 `inf/NaN` 传播到 GPU packet。

## 输入与 Pass 顺序

生产路径需要 scene depth、GBuffer normal、furthest HZB、velocity 和 GI history。启用后 compiled graph 应依次包含：

1. `hybrid-gi-scene-prepare`
2. `hybrid-gi-trace-schedule`
3. `hybrid-gi-resolve`
4. `hybrid-gi-history`

前三个节点产生当前帧 scene packet、trace packet 和 HDR GI lighting；history 节点保存 lighting 与 temporal metadata。feature 关闭、能力不足或 provider 缺失时，这四个节点都不应进入有效执行路径。

`scene-velocity` 是 HGI resolve 的必需只读输入。组合图可以把它声明为 external 或 transient；执行器对两种来源执行相同的名称、访问模式和纹理校验。即使产品 Profile 设置 `temporal_history=false`，只要 HGI capability 和 provider 实际生效，compile options 仍保留内置 Temporal feature 以产生 velocity；这不会隐式打开 TAA，TAA 仍由 post-process stack 独立过滤。

## Debug View

`RenderHybridGiDebugView` 是稳定的序列化请求合同。HGI executor 从当前 `RenderFrameExtract` 读取它，通过 resolve uniform 的稳定数值 ABI 传给 WGSL，并在空间/时域滤波前选择当前 GPU trace authority 的诊断颜色。各枚举的当前含义如下：

| 枚举 | 当前输出 |
|---|---|
| `None` | 最终 GI 合成 |
| `Cards` | 对 Surface Cache 命中的 support/card signature 生成稳定伪彩色，未命中像素写黑 |
| `SurfaceCache` | 只显示 Surface Cache 命中的当前 radiance，其他 source 写黑 |
| `VoxelClipmap` | 只显示 Surface Cache miss 后由软件 voxel fallback 提供的当前 radiance，其他 source 写黑 |
| `InputSet` | 按 Surface Cache、Voxel、screen hit、depth fallback 使用固定颜色显示当前 source 分类 |

这些模式写入 `hybrid-gi-lighting` 后仍经过正常 post-uber 产品路径，不另建调试 pipeline；因此关闭调试时没有额外 pipeline/cache 分裂。当前源码包含直接 resolve WGPU 像素合同和五视图产品矩阵导出器；`docs/tests/runtime/render/plan18_hybrid_gi_debug_views_wgpu_20260810.png` 只有在真实 WGPU 运行、五个 panel 非空且四个调试模式都与 `None` 产生像素差异后才会生成。该文件和对应 report 在实际生成前仍是待验证证据，不能用旧 PNG 或枚举标签替代。

## 降级与诊断

- backend 不声明 `HybridGlobalIllumination` capability：保留基础渲染，HGI provider report 为 disabled/degraded。
- product profile 未请求 HGI：不注册有效 HGI 工作，属于正常关闭，不是错误。
- provider 未链接：请求值保留在配置层，但执行层不得生成 HGI Pass。
- history 不兼容、motion/depth/source/support/normal 不匹配：拒绝该像素历史并使用当前帧结果。
- Surface Cache miss：按当前 V1 合同进入 voxel clipmap fallback；全层无有效 source 时保持确定性黑色/关闭结果，不读取未初始化 radiance。
- Mesh/Global SDF 不可用、形变中或超过对象/页面/上传预算：Global SDF 页面保持不可采样，继续进入 voxel clipmap fallback；不得把对象 AABB 当作有效 signed-distance payload。终端回退会清除该 generation 的 dirty request，直到对象或资源 revision 再次使页变脏；页面失效投影与 Global SDF 候选使用相同的扩张 influence band，因此邻页的有效贡献者也会重新标脏。纯页面/上传预算延后则保留 dirty request 并轮转到后续有界批次。

运行时应读取 resolved provider report 和 render stats，而不是仅检查用户请求值。至少核对 executed pass count、probe trace tile count、Surface Cache resident page count、voxel resident clipmap count与 capability fallback reason。Radiance Cache 的真实 GPU 工作使用 `last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts`，固定顺序为 `mark / allocate / trace / filter / border-mip / consume`；该数组来自异步 GPU readback，表示最近完成的样本而非当前 CPU 提交帧。稳定复用应为前五项零、`consume` 非零。

软件追踪 provenance 位于 `RenderHybridGiScenePrepareReadbackOutputs::probe_trace_diagnostics`。每条记录对应一个实际 dispatch 的 probe，包含按 tile 贡献加权的 dominant intersection/lighting source、实际使用的 intersection backend 与 lighting source bitmask、distance/confidence、typed fallback reason，以及 texture/page/SDF/voxel/hardware-ray cost counters。Voxel route 的 distance 是按贡献权重汇总的 probe-to-cell-AABB 世界距离，不会以占位零值伪装命中；其中心坐标以 producer 相同的有符号 `64x` 量化与 `+2048` 偏置解码。记录数受本帧 admitted probe 数限制；它是异步 GPU readback，不应被当作当前 CPU 提交帧的同步完成信号。

`Runtime Diagnostics` 面板从 provider 的 neutral prepared frame 读取同一份实际生效设置，不在 Editor 侧重新推算。启用 HGI 时会显示：

- `Hybrid GI effective`：实际 `profile`、降级后的 `mode` 和 `quality`；
- `Hybrid GI budgets`：最终 `trace/card/voxel` 预算，包含 profile 默认值与非零 override；
- `Hybrid GI fallback`：无降级时为 `none`，缺少 Plan 11 baked 合同时为 `baked-lighting-unavailable`；
- `Hybrid GI active probes`：当前可见 probe 数量。

provider 关闭或 extract 移除后，`RenderStats.last_hybrid_gi_resolved_settings` 会在同一帧清空，面板显示 `Hybrid GI effective: unavailable`，不会沿用上一帧的 Profile 或回退原因。

HybridGI 数据链已经通过 Runtime production build、IndoorStatic 缺 baked 的结构化 fallback 1/1、provider-missing/stale-state 2/2、Editor pane unit、组合图速度纹理双来源用例 1/1 和当前源码 `target-editor-host` 产品构建。实际产品证据为 [`Custom actual`](../../tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_actual_20260714.png) 与 [`IndoorStatic missing-bake fallback`](../../tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_fallback_20260714.png)，完整哈希与判读见 [`evidence report`](../../tests/runtime/render/plan18_hybrid_gi_editor_runtime_diagnostics_20260714.md)。两图均为 1688x980、viewport 非空、stderr 为空；前者显示 `custom/dynamic-only/medium`、`32/64/16`、`none`，后者显示 `indoor-static/dynamic-only/high`、`64/256/64`、`baked-lighting-unavailable`。该二进制的紧凑 pane 将 Render 帧行放在可见折叠下方，因此证据报告不虚构文字帧计数；后续源码已用顺序单测把 Render 行移入前五项。

### RenderDoc 产品捕获

当前 M4 产品捕获使用 `D:\Tools\renderdoc\renderdoccmd.exe` 1.44、DX12 backend、WGPU debug/validation 和 RenderDoc API validation。四 Profile Forward+/Deferred 产品矩阵会产生 8 个 panel/framework frame，对应 `docs/tests/runtime/render/plan18_hybrid_gi_m4_profile_forward_deferred_renderdoc_20260713_capture*.rdc`。

qrenderdoc 已重放首尾捕获。首个捕获可检索 `HybridGiSceneDepthHandoffPass`、`HybridGiTraceScheduleHandoffPass`、`HybridGiResolveTraceDepthSourcePass` 和 `HybridGiCompletionPass`；末个捕获可见 completion、`zircon::RenderGraphPass::hybrid-gi-history` 与 `zircon::DeferredLighting`。API validation 的 71/80 条提示均为 Source API、Medium、State Creation、ID 926 的 wgpu alias-heap CBV GPU VA 重叠诊断；这些提示需要保留在审计结果中，不得写成零告警，也不能等同于 HGI shader、barrier 或 dispatch 错误。

## 场景使用建议

| 场景 | 当前推荐 | 尚未成立的能力 |
|---|---|---|
| 可破坏场景、动态昼夜 | DynamicOnly，按 GPU 预算选择 Medium/High | Mesh/Global SDF 分层源码已接入；当前源码 WGPU/RenderDoc 与性能证据仍待协调器验收 |
| 室内固定结构 | 有有效 Plan 11 baked contract 时使用 IndoorStatic；缺失时观察确定性 DynamicOnly 回退 | 固定场景、mobility/Emissive 往返、RenderDoc、124 项 crate behavior 和 actual/fallback 产品图已通过；broad/full 仍开放 |
| 开放世界 | 有有效 baked contract 时使用 OpenWorld；重点观察 generation/streaming 失效 | Global SDF 分页与 generation compare-and-commit 已实现；跨区动态产品与性能验收仍开放 |
| 电影预览 | 使用 Cinematic + 固定 warmup 帧；无 baked contract 时按诊断降级 | 高预算固定场景、跨帧残影和 `.rdc` 已验收；更长时间稳定性与 broad/full 验证仍开放 |

公众号合集提供了混合工作流、Lumen Pass 和 SDF/clipmap 的工程线索，但不作为算法正确性或性能数字的依据。实际实现以本仓库代码、`dev/LumenInUE5.5.4WithComputeShader`、Unreal 源码结构和 WGPU capability 验证为准。

## 验收

最小验收不能只看 compiled graph 名称，应同时具备：

- provider 已按 profile 链接且 capability gate 允许执行；
- Deferred/Forward+ 场景都产生非空真实 WGPU 图；
- feature-off 与 provider-missing 路径保持基线；
- PNG 写入 `docs/tests/runtime/render` 并人工目检；
- report 记录 Pass、预算、cache/voxel/probe stats 与哈希；
- M3/M4/M5 总门另外要求 RenderDoc `.rdc`，没有 `.rdc` 不得把整个 Hybrid GI 产品化标为完成。M4 当前已有 8 份 DX12 `.rdc` 并完成重放/API 审计，Editor actual/fallback 与 crate suite 子门也已关闭；broad/full 验证仍不可由这些局部证据替代。

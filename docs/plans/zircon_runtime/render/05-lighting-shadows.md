---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs
  - zircon_runtime/src/graphics/tests/render_product_shadows/many_point_lights.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_shadows_many_point_lights_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_shadow_captures_directional_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_wide.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/rows.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowDepthRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Shadows/ShadowSceneRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapArray.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ForwardLights.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/DeferredLights.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ShadowUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/MainLightShadowCasterPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs
  - dev/bevy/crates/bevy_light/src/cluster/assign.rs
  - dev/bevy/crates/bevy_light/src/cascade.rs
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/bevy/crates/bevy_pbr/src/render/clustered_forward.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/shadows.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/shadow_sampling.wgsl
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/point.rs
plan_sources:
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
---

# 计划 05:光照与阴影管理

## 目标

解除"灯光走场景 uniform、上限 4 点光"的限制,建成 clustered light grid + GPUScene light buffer 的光照底座,
并把阴影从单一 shadow map 升级为 cascade 完整化 + 多光源 shadow atlas 的管理体系。完成后:

1. Forward+ 与 Deferred 共用同一份 light grid 与 light buffer(对齐 HGI 计划"同一套 GI 算法"原则)。
2. 场景灯光数量仅受 buffer 容量约束(数百级),支持 directional/point/spot/rect 全类型进 grid。
3. 方向光 CSM(级联划分、稳定化、过渡带)与 point/spot 阴影共享 atlas,槽位按优先级与距离动态分配。
4. 阴影渲染 view 经计划 04 独立剔除、计划 02 的 ShadowPassProcessor 生成命令。

## 现状与差距

- 灯光快照类型齐全(`light/snapshots.rs`),但消费端落在 BASIC_SCENE_UNIFORM,数量受限;cluster grid 已创建却没有 per-cluster light list 注入,clustered 描述符是空壳。
- `shadow_map_renderer.rs` 有方向光级联与点光基础,但级联划分/稳定化(texel snapping)/级联过渡未成体系,无 atlas 管理,多光源阴影互斥。
- 无 per-light 阴影参数面(bias/normal bias/分辨率档位)契约。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Renderer/Private/LightGridInjection.cpp` | froxel grid 构建 compute:cluster AABB 与灯光形状求交、灯光索引链表压缩为紧凑列表 |
| `dev/UnrealEngine/.../Renderer/Private/LightRendering.cpp` | deferred 光照 pass 的组织:per-light 体积光栅化 vs tiled/clustered 全屏 pass 的取舍 |
| `dev/UnrealEngine/.../Renderer/Private/ShadowSetup.cpp` | 级联划分(对数/线性混合 split)、阴影视锥构造、caster 收集边界 |
| `dev/UnrealEngine/.../Renderer/Private/ShadowDepthRendering.cpp` | shadow depth pass 与专用 pass processor(bias、slope-scale 状态) |
| `dev/Graphics/.../Runtime/ForwardLights.cs` | URP Forward+ 的 light buffer/cluster 打包(zbin + tile 列表),比 UE 更贴近本引擎规模,推荐作为 grid 数据布局首选样板 |
| `dev/Graphics/.../Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs` + `ShadowUtils.cs` | 多光源 shadow atlas 槽位分配、按分辨率档位打包、texel snapping 稳定化 |
| `dev/UnrealEngine/.../VirtualShadowMaps/VirtualShadowMapArray.cpp` | VSM 远期预研(本计划只读不实施) |

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_light/src/cluster/assign.rs` | `LightGridPass` 的 CPU 聚簇构建 | `assign_objects_to_clusters`:CPU 端灯光-cluster 求交与紧凑索引打包(Rust 同型先例,数据布局取 URP zbin+tile) |
| `dev/bevy/crates/bevy_pbr/src/render/light.rs` | `GpuLightData` 打包与 shadow view 生成 | `prepare_lights`:灯光 extract → GPU buffer 打包、级联/点光 shadow view 批量创建、`ViewShadowBindings` |
| `dev/bevy/crates/bevy_pbr/src/render/clustered_forward.wgsl` | `zr_light_grid.wgsl` include | cluster offset/count 查询函数与着色端灯光遍历骨架的 WGSL 形态 |
| `dev/bevy/crates/bevy_light/src/cascade.rs` | `cascade.rs` 级联分割与稳定化 | `CascadeShadowConfigBuilder::build`/`calculate_cascade_bounds`、`Cascade.texel_size`(snapping 用世界单位 texel) |
| `dev/bevy/crates/bevy_pbr/src/render/shadows.wgsl` | `zr_shadow.wgsl` CSM 选择与槽采样 | `get_cascade_index`/`world_to_directional_light_local`/`fetch_point_shadow`/`fetch_spot_shadow` |
| `dev/bevy/crates/bevy_pbr/src/render/shadow_sampling.wgsl` | PCF 质量分档(LS-M4) | `sample_shadow_map_hardware`(硬件 2x2)与 `sample_shadow_map_castano_thirteen`(高质量核)的 WGSL 写法 |
| `dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs` | CSM 最小可用 Rust 实现 | `CSM_NUM_CASCADES` 级联数组、`split_options` 分割、逐级联正交投影 + framebuffer 组织 |
| `dev/Fyrox/fyrox-impl/src/renderer/shadow/point.rs` | point 光立方体 6 面阴影 | cube map 六面(`CubeMapFaceDescriptor::cube_faces`)shadow pass 组织,LS-M3 point 光 6 槽参照 |

`ShadowAtlas` shelf 槽位分配器无 Rust 同类参照(bevy 走 texture array 分层、Fyrox 走独立 framebuffer,均无 atlas 打包),实现时以 URP `AdditionalLightsShadowAtlasLayout.cs` 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:`graphics/scene/scene_renderer/` 下新增 `lighting/`(grid 构建与光照 pass)并升级 `shadow/`;灯光契约扩展在 `core/framework/render/light/`。

核心设计:

- `GpuLightData`:进计划 03 GpuScene 的 light buffer(类型、位置/方向、颜色强度、范围/锥角、阴影槽位索引、layer mask);extract 端 `light/snapshots.rs` 增加稳定 light id。
- `LightGridPass`:URP 风格 zbin + tile 灯光列表(比 UE 链表更简单,wgpu storage buffer 友好);V1 由 CPU 构建并经 graph executor 上传到瞬态 buffer,IO 合同保留为后续 GPU compute 构建可替换形态。forward 着色与 deferred lighting pass 同读此 grid。
- deferred lighting 改造:从"场景 uniform 灯光循环"切到"grid 驱动循环";forward+ 同一 WGSL include 共享采样函数。
- `ShadowAtlas`:单张大 depth 纹理 + 槽位分配器(2 的幂档位,按光源屏幕占比/优先级分配与降档);方向光 CSM 占独立 atlas 区段;级联划分采用对数线性混合 + texel snapping + 级联间过渡带。
- per-light 阴影参数契约:bias/normal bias/分辨率偏好/是否投影进 `light/` 契约与编辑器面板对接;`casts_shadow` relevance 由计划 04 提供。
- 多 render layer:灯光 layer mask 与计划 09 的 RenderLayer 对齐,着色时按 mask 过滤(Unity Light culling mask 语义)。

## 里程碑

### LS-M1 GpuLightData 与灯光上限解除

进度(2026-06-13):
- 已完成第一段数据底座:新增 framework 侧 `GpuLightData`/`GpuLightType`/`LightShadowSettings`;
  `RenderDirectionalLightSnapshot`/`RenderPointLightSnapshot`/`RenderSpotLightSnapshot`/`RenderRectLightSnapshot`
  已携带稳定 `light_id`、`layer_mask` 与可选 shadow 设置,真实 scene extract 以 `node_id` 作为稳定 id 并继承实体 render layer mask。
- 已新增 `graphics/scene/scene_renderer/lighting/light_buffer.rs`,按 Bevy light packing 与 URP structured additional-light buffer 思路,
  将 directional/point/spot/rect 全量打包为 96B `GpuLightData`;该打包路径不再按 scene-uniform 点光上限截断。
- `GpuScene` 现已拥有真实 light storage buffer、CPU shadow、容量增长、整段上传和 bind group 重建路径;
  `build_mesh_draws` 会在 primitive/instance 同步前把当前帧 packed lights 写入 GPUScene,由既有 GPUScene flush 一并上传。
- LS-M1 shader 硬切换已落地:forward fallback 与 deferred lighting 均 prepend `zr_gpu_scene.wgsl`,从 group3 的
  `GpuLightData` storage buffer 做全量灯光循环;deferred lighting pipeline layout 绑定同一 GPUScene group3。
- `SceneUniform` 已删除 `light_dir`/`light_color`/点光数组/点光参数字段,非光照 shader 只声明自己实际读取的 camera/ambient/motion
  prefix;`readiness.rs` 已移除旧 scene-uniform directional/point 上限,方向光/点光/聚光按 light-buffer 消费路径报告 ready。
- LS-M1 窄范围验收已补齐:core-min `cargo check` 通过,已构建 lib-test 二进制并直接跑通 light ABI、light packer、
  GPUScene light upload、forward/deferred WGSL validity 与 GPU light shader source 断言;`render_product_many_point_lights`
  已补源码产品合同,证明 64 点光打包不截断。2026-06-21 新增
  `render_product_many_point_lights_forward_deferred_capture_parity`,用真实 WGPU 捕获证明 64 点光在 Forward+/Deferred
  两管线都会提高中心 mesh 亮度并保持同一产品范围。

实施切片:
1. light buffer 进 GpuScene(依赖计划 03 GS-M1);extract 增加 light id 与脏更新。
2. 内建 shader 灯光循环改读 buffer(暂全量循环,不分簇);删除 BASIC_SCENE_UNIFORM 灯光段。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime light --locked` 与 `render_product` 回归
- 验收证据:>4 点光场景全部生效(产物对拍);灯光增删改增量上传断言。

### LS-M2 clustered light grid

进度(2026-06-13):
- 已完成 light grid 数据底座:新增 `lighting/light_grid_builder.rs`,按 URP zbin + tile mask 结构构建
  `LightGridParams`、zbin header、tile bitmask 与 `LightGridStats`;tile 从 8px 起按预算倍增,
  zbin/tile word 上限分别固定为 4096/8192,方向光覆盖全 grid,point/spot/rect 以 view-space sphere 近似求交。
- 已新增 `lighting/light_grid_pass.rs`,从当前 extract 与 camera 生成 frame light grid 并写入 zbin/tile buffers;
  `light-grid-build` graph 节点声明 `LIGHT_ZBINS`、`LIGHT_TILE_MASKS`,并暂时继续写 legacy `LIGHT_LIST`
  供旧 tile tint/post-process 兼容路径消费。
- 已新增 `lighting/shaders/zr_light_grid.wgsl`,提供 grid 参数、zbin header、tile base、mask word 与灯光计数查询函数;
  该 include 已被 forward fallback、deferred lighting 与内建 PBR shader 共同引用;三条着色路径已从 LS-M1 全量 light-buffer
  循环切到 `zbin_word & tile_word` 的 grid mask 交集遍历,并在 light index 越界时保护返回。
- graph executor 注册名已从 `lighting.clustered-cull` 收束为 `lighting.light-grid`,pipeline 编译器为
  `LIGHT_GRID_PARAMS`/`LIGHT_ZBINS`/`LIGHT_TILE_MASKS` 分配 transient buffer,mesh/deferred pass 描述符声明读取这些资源。
  forward group1 layout 与 deferred lighting group1 layout 都绑定 light-grid uniform/storage buffers;motion-vector 与 overlay
  路径使用 disabled/empty fallback buffers。
- `LightGridStats` 已接入 graph execution record、`SceneRenderer::last_light_grid_report()`、`RenderStats` 与产品诊断;
  指标记录在 `render.light_grid.*`,包括 light/tile/zbin 数、非空 tile/zbin/cluster、峰值每簇灯光数与平均每簇灯光数。
- 64 点光产品合同切片(2026-06-14):新增 `render_product_many_point_lights`,在 render product 层证明 64 个 point light
  进入 `GpuLightData` packer、light grid 使用第二个 32-bit word、zbin/tile mask 保留高位灯光,且默认 Forward+/Deferred 编译图
  都经 `light-grid-build` 写入并由 mesh/deferred lighting pass 读取 `LIGHT_GRID_PARAMS`/`LIGHT_ZBINS`/`LIGHT_TILE_MASKS`。
- 验证(2026-06-13):`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` 通过
  (仓库既有 warnings);临时 Naga 验证器直接解析并验证了 fallback mesh、deferred lighting、builtin PBR 三份拼接后 WGSL。
  过滤 `cargo test` 目标在 Windows lib-test 代码生成阶段两次超时,未返回测试结果;本切片保留源码断言与 check/WGSL 验证作为当前证据。
- 64 点光合同验证(2026-06-14):`cargo fmt --all`、`cargo fmt --all -- --check` 通过;`cargo check -p zircon_runtime
  --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-many-point-contracts-coremin --message-format short --color never`
  通过(66 个仓库既有 warnings)。`cargo test -p zircon_runtime --lib render_product_many_point_lights --no-default-features
  --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-many-point-contracts-coremin
  --message-format short --color never --no-run` 完成 lib-test 编译;随后直接运行生成的
  `zircon_runtime-5d2828c2001649f6.exe render_product_many_point_lights --nocapture` 通过 1 个过滤测试。该切片仍不替代真实
  forward/deferred 捕获对拍。
- 64 点光真实捕获对拍(2026-06-21):新增
  `render_product_many_point_lights_forward_deferred_capture_parity`,注册 lit PBR 材质和真实 cube mesh,分别提交 Forward+
  与 Deferred 基线/64 点光场景,断言 `lighting.light-grid` executor、64 点 light-grid stats、dense cluster peak 以及中心区域
  捕获亮度提升。冷 target-dir Cargo wrapper 在 Windows 共享 lib-test 编译窗口内超时未产出二进制;复用预热
  `target\codex-runtime-hzb-storage-limit-0620` 的 `cargo test -p zircon_runtime --lib
  render_product_many_point_lights_forward_deferred_capture_parity --no-default-features --features core-min --locked --jobs 1
  --target-dir target\codex-runtime-hzb-storage-limit-0620 --message-format short --color never -- --test-threads=1 --nocapture`
  通过 1 个过滤测试(仓库既有 warnings)。同一二进制直接复跑 `render_product_many_point_lights`、`render_product_csm_directional`
  与 `render_product_multi_spot_shadows` 均通过。
- 百灯局部密度统计(2026-06-21):新增
  `render_product_hundred_point_lights_report_local_density_stats`,构造总数同为 128 的 dense/spread 两组 point lights,
  先直接检查 CPU light-grid 的峰值和平均每 cluster 灯数,再经真实 WGPU Forward+ 产品提交验证 `lighting.light-grid`
  executor、128 点 light-grid stats、非空 grid 占用、峰值与平均 cluster 负载差异。该用例明确证明当前统计随局部密度变化,
  不把总灯数相同的场景误判为同等着色压力。

实施切片:
1. zbin+tile grid CPU 构建 + graph 上传节点;grid 资源经计划 01 瞬态池,IO 合同保留后续 GPU compute 替换空间。
2. forward 与 deferred 着色切换为 grid 驱动;共享 WGSL include。
3. grid 统计(平均/峰值每簇灯光数)进 RenderStats。

测试阶段:
- `cargo test -p zircon_runtime lighting --locked`(grid builder zbin/tile mask、预算扩张、WGSL include 解析)
- 验收证据:百灯场景着色成本与灯光局部密度相关而非总数(统计)已由 2026-06-21 产品守卫覆盖;两管线产物一致性对拍已由同日 64 点光真实捕获覆盖。

### LS-M3 CSM 完整化与 shadow atlas

进度(2026-06-13):

- LS-M3 第一切片已完成底层支撑:新增 `shadow/atlas/allocator.rs` 与 `shadow/cascade.rs`,并给 `ShadowResolutionTier`
  补齐降档辅助方法。atlas 分配器支持 4096 默认 atlas、顶部 CSM 保留行、URP 风格整体面积降档、优先级压力驱逐、
  上一帧槽位复用、8 帧滞回、连续 4 帧且高 25% 的抢占门槛,并输出 frame allocation/rejection 报告。
- CSM 支撑已实现 log/linear 混合 split、fade range、camera frustum slice bounds、light-space texel snapping 与
  snapped orthographic view-projection helper。该底座现在同时服务 `ShadowFramePlan` 的 atlas slot 矩阵和 Plan 04
  direction cascade shadow view camera,让 atlas pass 与 visibility 使用同一套 split/bounds 口径。
- 已新增 `shadow/slot.rs` 固定 `GpuShadowSlot`/`GpuShadowGlobals` CPU-side GPU ABI:`ZrShadowSlot` 为 96 B
  (`view_proj` 0、`atlas_scale_bias` 64、`params` 80),globals 为 48 B(cascade far split、fade length、atlas size/
  inv-size)。已新增 `shadow/atlas/resources.rs`,由 `SceneRendererCore` 持有持久 WGPU depth atlas、atlas view、
  comparison sampler、`shadow_slots` storage buffer 与 `shadow_globals` uniform buffer;默认 4096x4096/256 slot,
  设备 2D 纹理上限不足时降到 2048。`shadow/atlas/bindings.rs` 已把最终 group1 binding 固定为
  8/9/10/11,避开 legacy shadow 与 light-grid 槽位。当前每帧上传 `ShadowFramePlan` 产出的 slot/global payload,
  并已把 atlas entries 插进 forward/deferred group1 layout 与 bind group;motion-vector/overlay 等无 atlas 输入路径使用 disabled fallback
  slot/global buffer。
- 已新增 `shadow/plan.rs` 作为 LS-M3 计划桥接层:根据整帧 `ViewportRenderFrame` 构建 `ShadowFramePlan`,方向光写连续 CSM slot,
  point 光要求 6 个 face slot 全部分配成功才写入,spot 光写 1 个 slot,并在 `build_mesh_draws` 写入 GPUScene light buffer 前
  patch `GpuLightData.shadow_slot_layer.x` 与 `shadow_params.w`。当前 slot `view_proj` 已从 identity 占位推进为真实矩阵:
  方向光基于主相机 frustum slice bounds 生成 texel-snapped CSM 正交矩阵,spot 基于位置/方向/锥角/range 生成透视矩阵,
  point 光写 6 个 90 度 face 透视矩阵。
- 已新增 `shadow/shaders/zr_shadow.wgsl`,由 forward fallback、deferred lighting 与 builtin PBR shader 共享 include。shader 端
  通过 `GpuLightData.shadow_slot_layer`/`shadow_params.w` 选择方向光级联、point 光 cube face 或 spot 单槽,投影到
  `ZrShadowSlot.view_proj` 与 atlas scale/bias 后按 slot flag 进行 1/5/9 tap comparison PCF;默认 authoring 档位为 Low,
  现有产品合同显式设 High 以保留旧 3x3 采样行为。
- 已完成 graph-declared atlas depth 写入闭环:`PostProcessGraphResourceNames::SHADOW_ATLAS` 是 graph-visible
  外部资源,内建 shadows descriptor 已收束为 `shadow-atlas` pass / `shadow.atlas` executor;forward mesh、deferred lighting
  与 deferred transparent mesh 显式读取 `SHADOW_ATLAS` 以建立 graph 顺序。
  `import_frame_targets()` 把持久 `ShadowAtlasResources::atlas_view()` 导入 execution resources,Shadow 阶段把
  `ShadowFramePlan` 传入 `RenderPassGpuExecutionContext`,并由 `ShadowMapRenderer::record_atlas_commands_with_attachment_ops()`
  按 `ShadowAtlasSlotPass { slot_index, rect, view_proj, view_key }` 逐槽设置 viewport/scissor 后 replay shadow command stream。
  本轮已让方向光 cascade、point face、spot slot pass 分别绑定 `VisibilityViewKey::ShadowCascade { light, cascade }`、
  `ShadowPointFace { light, face }`、`ShadowSpot { light }`;shadow 命令携带 source entity,atlas 写入回放时会按对应 Plan 04
  shadow view 的 visible entity set 跳过不可见命令。Plan 04 现在生成 atlas 所需 view key,且方向光各 cascade shadow view
  使用同一套 CSM frustum slice bounds。pass/executor 命名已收束为最终 atlas 形态。
- atlas receiver 硬切片(2026-06-14):已删除 graph 级 legacy `SHADOW_MAP` 资源、forward/deferred 旧单贴图 receiver bindings
  (`shadow_map_tex`/`ShadowReceiverUniform`/`shadow_compare_sampler`)与 Rust receiver uniform buffers;`shadow-atlas` pass
  现在只写 external `SHADOW_ATLAS`,所有 forward/deferred receiver shader 只通过 atlas slot 采样。`RenderShadowExecutionReport`
  的写入统计改为 `shadow_atlas_write_count`,graph/test 夹具也收束到 `shadow-atlas`。
- caster/receiver 源码保护切片(2026-06-14):新增 `mesh_visibility_states_preserve_shadow_only_casters`,在 mesh draw
  visibility state 层证明"主视图不可见但 shadow view 可见"的 caster 会保留 `shadow_view_visible=true`,不会被
  main-view culling 误删。该切片只覆盖源码级 caster 保留边界,不替代 forward/deferred 对拍、CSM 平移稳定性或 multi-spot
  产品捕获。
- 产品阴影合同切片(2026-06-14):新增 `render_product_csm_directional` 与 `render_product_multi_spot_shadows`,
  将方向光 4 级联 slot 生成与 ≥3 spot 阴影同帧 atlas 槽位不重叠写入 render product 源码合同。该切片仍不替代真实
  forward/deferred 捕获对拍、CSM 平移稳定性抓帧和 receiver 覆盖验证。
- 方向光 shadow-atlas 真实 WGPU 捕获(2026-06-21):新增
  `render_product_directional_shadow_atlas_capture_records_receiver_path` 与
  `render_product_directional_shadow_atlas_darkens_receiver_capture`,在真实 Forward+ 产品提交中注册 receiver/caster
  材质与 cube mesh,断言 `shadow.atlas` executor、atlas graph write、receiver 读取、caster draw、可见 receiver sample,
  并用同色 receive-shadow 开关对拍证明 receiver 区域产生可见暗化。调试确认 atlas depth 与投影有效后,
  `ShadowAtlasResources` 的比较采样器改为 `GreaterEqual`,并由
  `render_shadow_atlas_compare_function_matches_forward_depth_contract` 锁定深度比较合同。CSM 平移稳定、多
  spot、forward/deferred shadow parity、PCF 质量与 contact shadow 捕获已由后续 2026-06-21 行补齐。
- 多 spot shadow-atlas 真实捕获守卫起步(2026-06-21):新增
  `render_product_multi_spot_shadow_atlas_darkens_receivers_capture`,在真实 Forward+ 产品提交中注册 3 个 spot 光、
  3 个 caster 与同一 receiver,用 receive-shadow 开关对拍统计全帧暗化像素、luma delta 与 RGB delta,并断言
  `shadow.atlas` executor、atlas graph write、receiver 读取、spot ready count、caster draw 和新的
  `RenderShadowExecutionReport.shadowed_light_count == 3`。本切片还把 `RenderShadowExecutionReport` 的阴影灯数量
  从旧 direction-only 统计补正为当前 atlas 支持的 directional/point/spot shadow-casting light 总数。当前已通过
  core-min library check;后续长窗口 lib-test `--no-run` 已产出可执行测试二进制,直接 WGPU 执行也已通过。
- 混合 shadow-atlas 宽场景真实 WGPU 守卫(2026-06-22):新增
  `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture`,在新的
  `render_product_shadow_wide.rs` 中保持宽场景产品守卫独立,避免继续扩大既有 capture 文件。该用例用真实 Forward+
  产品提交同帧注册 1 个 directional 与 3 个 spot shadow-casting lights、宽 receiver 与多组 caster,对同色
  receive-shadow 开关做全帧和左/中/右区域暗化对拍,并断言 `shadow.atlas`、`lighting.light-grid`、directional/spot
  ready counts、`RenderShadowExecutionReport.shadowed_light_count == 4`、receiver availability 与 caster draw count。
  验证通过 `cargo test -p zircon_runtime --lib render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture --locked --jobs 1 --target-dir target\codex-shadow-wide-0622 --message-format short --color never -- --test-threads=1 --nocapture`
  1/1(首次默认特性 lib-test 构建 28m46s,测试执行 5.21s,仓库既有 warnings)。RenderDoc 证据、更宽 contact-shadow
  场景与 root wider checks 仍未完成。
- 验证(2026-06-13):`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` 通过
  (仓库既有 warnings)。`cargo test -p zircon_runtime shadow --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --no-run --message-format short --color never`
  在 Windows 共享 lib-test 编译/代码生成阶段 600s 超时,未返回测试二进制;残留 cargo/rustc 进程已确认清理。真实矩阵切片随后已通过同一 core-min
  `cargo check` 形态。本轮 atlas binding/shader sampling 切片再次通过同一 core-min `cargo check`;focused
  `fallback_mesh_shader_is_valid_wgsl` 测试在共享 lib-test 编译阶段 600s 超时,残留 cargo/rustc 进程已清理。本轮
  graph-declared atlas depth 写入切片已运行同一 core-min `cargo check` 并通过,仍为 70 个仓库既有 warnings。随后尝试
  `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked render_shadow_frame_plan_assigns_first_directional_cascade_slots --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never -- --exact --nocapture`
  在 600s 内未返回测试结果,仍表现为 Windows 共享 lib-test 编译超时;本次超时命令未留下对应 target-dir 的 cargo/rustc 残留进程。
  随后的 atlas shadow-view 扩展切片已再次通过 `cargo fmt --all -- --check` 与同一 core-min `cargo check`,覆盖 `ShadowCascade`、
  `ShadowPointFace`、`ShadowSpot` view key 生成和 atlas slot pass 绑定,仍为仓库既有 warnings。focused
  `visibility_context_builds_shadow_views_for_atlas_light_slots` 测试尝试在 600s 内未返回结果,仍表现为 Windows 共享 lib-test
  编译超时;对应 target-dir 的残留 cargo/rustc 进程已停止或已退出。方向光 frustum-slice follow-up 随后在同一 target dir 下完成
  shared lib-test `--no-run` 编译;`cargo test ... render_shadow_ -- --nocapture` 通过 27 个 shadow 过滤测试,覆盖新增
  `render_shadow_cascade_bounds_follow_camera_slice_depth` 与 `render_shadow_frame_plan_builds_distinct_directional_cascade_matrices`;
  `cargo test ... visibility_context_builds_shadow_views_for_atlas_light_slots -- --nocapture` 通过 1 个 visibility 过滤测试。
- pass/executor 命名收束切片(2026-06-13):内建 shadows descriptor、执行器注册、产品阴影合同与图统计合同已从
  `shadow-map`/`shadow.map` 收束为 `shadow-atlas`/`shadow.atlas`,当时仍继续写 legacy `SHADOW_MAP` 与 external
  `SHADOW_ATLAS`;该 legacy 写入已在 2026-06-14 receiver hard cut 中删除。`cargo fmt --all -- --check`、scoped `git diff --check` 通过;`cargo check -p zircon_runtime
  --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin
  --message-format short --color never` 通过(66 个仓库既有 warnings)。`cargo test ... shadow_atlas -- --test-threads=1 --nocapture`
  通过 16 个过滤测试,1 个 `render_framework_stats_report_shadow_atlas_graph_execution` 因当前 HZB occlusion WGPU
  storage-buffer 上限 blocker 被标记 ignored;直接运行已构建的 core-min lib-test 二进制 `render_product_shadows`
  通过 4 个产品阴影合同测试。`render_product_shadows` 的 Cargo wrapper 在复用 target dir 时遇到 dep-info 写入错误,
  干净 target dir 重新编译 15 分钟超时;默认 Forward+/Deferred pipeline compile 过滤的重新编译也在共享 lib-test 链接阶段超时,
  但最终 core-min `cargo check` 已重新通过且无残留 cargo/rustc 进程。
- hard cut 验证(2026-06-14):`cargo fmt --all`、`cargo fmt --all -- --check`、本切片 scoped `git diff --check` 通过;
  `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-shadow-atlas-cutover-coremin --message-format short --color never`
  通过(66 个仓库既有 warnings)。`cargo test -p zircon_runtime --lib shadow_atlas --no-default-features --features core-min --locked ...`
  在启动前被当前 `Cargo.lock` 与测试目标解析不一致阻止,未修改锁文件,无测试结果。
- 产品阴影合同验证(2026-06-14):`cargo fmt --all`、`cargo fmt --all -- --check` 通过;`cargo check -p zircon_runtime
  --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-vc3-product-shadow-contracts-coremin --message-format short --color never`
  通过(66 个仓库既有 warnings)。`cargo test -p zircon_runtime --lib render_product_multi_spot_shadows --no-default-features
  --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-product-shadow-contracts-coremin
  --message-format short --color never --no-run` 在 904s 后仍未返回 shared lib-test 编译结果,无过滤测试结果;无残留
  target-dir cargo/rustc 进程,未修改 `Cargo.lock`。

实施切片:
1. 级联划分/稳定化/过渡带;`ShadowAtlas` 槽位分配器与降档策略。
2. point(立方体 6 面或双抛物面)/spot 阴影入 atlas;`GpuLightData` 写阴影槽位与矩阵。
3. 已接入计划 04 per-light/per-slot view 剔除与方向光 frustum-slice 精化:atlas replay 按 `ShadowAtlasSlotPass.view_key` 过滤 source entity,覆盖方向光 cascade、point face、spot;方向光 cascade view 与 atlas slot 矩阵共享 CSM frustum-slice bounds。
4. 已收束 graph pass/executor 命名并完成 receiver 硬切:`shadow-atlas` / `shadow.atlas` 为当前图合同;旧 `SHADOW_MAP` graph 资源与单贴图 receiver 绑定已删除。

测试阶段:
- `cargo test -p zircon_runtime shadow --locked`(分配器单测:档位、驱逐、降档)+ `render_product` 阴影场景
- 验收证据:相机平移时方向光阴影边缘无 swimming(texel snapping 生效,抓帧对比);多 spot 阴影并存。

### LS-M4 PCF 质量与 contact shadow(可选 feature)

进度(2026-06-14):
- 已完成 PCF 质量源码切片:新增 `ShadowPcfQuality { Low, Medium, High }` 并接入 `LightShadowSettings.pcf_quality`,
  `GpuShadowSlot.params.w` 的 flags 以 bit 8..9 编码质量档位,`ShadowFramePlan` 按 per-light shadow settings 写入 slot。
    `zr_shadow.wgsl` 现在按 slot flag 分流 Low=1 tap、Medium=5 tap 十字、High=9 tap 宽核;forward fallback、deferred lighting
    与 builtin PBR shader source 断言均覆盖该 include 合同。
- 已完成 PCF 质量真实 WGPU 捕获(2026-06-21):新增
  `render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture`,构造 spot receiver/caster 场景并分别提交 Low/High
  PCF 与 receive-shadow 关闭基线。早期 directional 场景 Low/High 截图完全相同;临时 diagnostic 证明 High flag 已进 shader,
  最终改用 spot 边缘场景并将 High kernel 半径显式扩展到 8 texel 后,真实截图能同时证明两档均产生 receiver 暗化且质量切换改变边缘产物。
- 已完成 contact shadow 描述符闭环:新增 `rendering.contact_shadow` 可选 feature,同步 `zircon_plugins/Cargo.toml`、
  `plugin.toml`、Rendering runtime provider、内置 Rendering catalog 行与编辑器 capability;runtime crate 注册
  `contact-shadow` / `lighting.contact-shadow` async compute pass,读取 `scene-depth`、`gbuffer-normal`、计划 04 `hzb-furthest`,
  写入 `contact-shadow-occlusion` transient storage texture。`plugin_render_features.rs` 将该 feature 插到 `hzb-build`
  之后、`light-grid-build` 之前;插件测试覆盖 feature 关闭时 compiled graph 无 `contact-shadow` pass。
- 已完成 contact shadow 最小真实执行器:runtime crate 用插件自有 `RenderPassExecutor` 替换 no-op,缓存 WGPU compute pipeline,
  绑定 depth/normal/HZB 与 `contact-shadow-occlusion` storage texture,执行 `contact_shadow.wgsl` 的 8x8 ray-march pass,
  并通过 `RenderPassGpuExecutionContext::record_compute_dispatch(...)` 记录实际 dispatch 与 storage write 资源。该执行器不依赖
  runtime 私有 post-process stack;仍保持 feature 默认关闭。
- 已完成 contact shadow 后处理消费闭环:`post.stack` graph pass 现在声明读取 `contact-shadow-occlusion`,保证该 transient
  texture 的生命周期覆盖到最终后处理;feature 关闭或资源缺席时 execution 绑定白色 fallback。`SceneRuntimeFeatureFlags` 新增
  `contact_shadow_enabled`,由插件 feature 名 `contact_shadow` 驱动;`PostProcessParams::lighting_flags.x` 独立控制 contact shadow,
  不复用 SSAO 标志。`post_process.wgsl` 新增 binding 27 `contact_shadow_tex`,在最终颜色阶段把 contact visibility 与 AO
  分开相乘;SSR 内部 pass 复用同一 bind group 布局时绑定白色 fallback,不采样 contact shadow 槽位。
- 已完成 contact shadow 真实 WGPU 产品捕获(2026-06-21):新增
  `contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region`,用同一 Forward+ receiver/blocker 场景分别提交
  plugin-enabled 与 baseline pipeline。测试断言 `rendering.contact_shadow` effective feature、`contact-shadow` graph pass、
  `lighting.contact-shadow` executor、compute dispatch/workload 与零 coverage gap,并用最终帧对拍证明接触区域产生可见暗化。
  插件 executor 改为经 `RenderPassGpuExecutionContext::require_texture_view(...)` 解析 pass-declared texture views,
  不再直接访问 runtime 私有 execution resources。
- 已完成 contact shadow 更宽真实 WGPU 产品捕获(2026-06-22):新增
  `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions`,用 192x128 Forward+ 宽 receiver 与
  左/中/右三组 blocker 分别提交 plugin-enabled 与 baseline pipeline。测试继续断言 `rendering.contact_shadow` feature、
  `contact-shadow` pass、`lighting.contact-shadow` executor、compute dispatch/workload 与零 coverage gap,并要求全帧和
  左/中/右三个接触窗口均产生可见暗化,同时检查 open receiver 区域不会吞掉主要暗化统计,防止把全屏色差误判为接触阴影。
  本切片按 `docs/plans/engine-code-structure-convention.md` 与 `docs/plans/engine-code-review-findings-2026-06.md`
  复核:只扩展测试层,未新增生产 `Result<_, String>`、`#[allow(dead_code)]`、FFI 边界或 builder/API 债,测试文件 531 行低于
  800 行拆分阈值。
- 验证(2026-06-14):`cargo fmt --all -- --check` 与 scoped `git diff --check` 通过;`cargo check -p zircon_runtime
  --lib --no-default-features --features core-min --locked --jobs 1
  --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never` 通过(67 个仓库既有 warnings)。
  `cargo test -p zircon_runtime --lib pcf_quality --no-default-features --features core-min --locked --jobs 1
    --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never --no-run`
    在 Windows 共享 lib-test 编译/链接阶段超时 904 秒,未产出过滤测试结果;残留的该 target-dir cargo/rustc 进程已停止。
    轻量源码合同检查确认 12 个关键符号/断言存在。
  - contact shadow 验证(2026-06-14):scoped `rustfmt --edition 2021 --check` 通过;scoped `git diff --check` 通过
    (仅 CRLF 提示);`cargo metadata --manifest-path zircon_plugins\Cargo.toml --no-deps --format-version 1 --locked`
    确认 `zircon_plugin_rendering_contact_shadow_runtime` 与 `zircon_plugin_rendering_contact_shadow_editor` 已进入插件 workspace;
    16 项 contact-shadow 源码合同检查通过。真实执行器切片追加 scoped `rustfmt --edition 2021 --check`、scoped
    `git diff --check` 与 15 项源码合同检查,覆盖 WGPU executor、storage texture binding、shader binding 和 dispatch 记录入口。
    后处理消费切片追加 scoped `rustfmt --edition 2021`、scoped `git diff --check`、12 项源码合同检查与 4 个 SSR fallback
    调用点检查,覆盖 runtime flag、post.stack graph 读取、binding 27、WGSL contact shadow 采样和插件侧 post-process 读依赖断言。
    本切片复跑 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1
    --target-dir D:\cargo-targets\zircon-runtime-contact-shadow-post-0614` 与
    `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_rendering_contact_shadow_runtime --locked --jobs 1
    --target-dir D:\cargo-targets\zircon-contact-shadow-post-0614`,两者均在编译前因对应 lock 文件需要刷新而被 `--locked`
    拦截;当前未修改 lock 文件,本切片不强行刷新/恢复 `Cargo.lock`。
  - PCF 质量真实捕获验证(2026-06-21):`cargo test -p zircon_runtime --lib
    render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture --no-default-features --features core-min --locked
    --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture`
    通过 1/1(测试执行 4.48s,仓库既有 warnings);`cargo test -p zircon_runtime --lib shadow_atlas_resources
    --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621
    --message-format short --color never -- --test-threads=1 --nocapture` 通过 3/3,覆盖 Forward fallback、Deferred lighting
    与 builtin PBR 三条 shader source 的 PCF 半径常量合同。
  - contact shadow 真实捕获验证(2026-06-21):`cargo test -p zircon_plugin_rendering_contact_shadow_runtime
    contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region --locked --jobs 1
    --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture`
    通过 1/1(测试执行 13.84s,仓库既有 warnings)。随后 `cargo test -p zircon_plugin_rendering_contact_shadow_runtime
    --lib --locked --jobs 1 --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never
    -- --test-threads=1 --nocapture` 通过 7/7。`zircon_plugins/Cargo.lock` 已同步以满足插件 workspace 的 `--locked`
    验证,新增的 `typed-path`/`zip 9.0.0-pre2` 来自当前 `zircon_runtime` 依赖解析。
  - contact shadow 更宽真实捕获验证(2026-06-22):`cargo test -p zircon_plugin_rendering_contact_shadow_runtime
    contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions --locked --jobs 1
    --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture`
    通过 1/1(测试执行 8.92s,首次该 target-dir 编译 27m42s,仓库既有 warnings)。随后
    `cargo test -p zircon_plugin_rendering_contact_shadow_runtime --lib --locked --jobs 1
    --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture`
    通过 8/8(测试执行 13.58s,仓库既有 warnings)。scoped `rustfmt --edition 2021 --check
    zircon_plugins\rendering\features\contact_shadow\runtime\src\wgpu_product_tests.rs` 通过。

实施切片:
1. 已完成 PCF 核(质量分档:1/5/9 tap)与 per-light bias/pcf quality 槽位合同定稿。
2. 已完成 PCF 质量真实 WGPU 捕获:spot receiver-edge 场景证明 Low/High 档位均产生可见暗化,且 High 宽核会改变截图边缘。
3. 已完成:contact shadow(屏幕空间短距离 ray march,读 HZB)作为 rendering 插件可选 feature 接入,并已通过 post.stack 读取
   `contact-shadow-occlusion` 乘入最终颜色。
4. 已完成 contact shadow 真实 WGPU 产品捕获:plugin-enabled 与 baseline Forward+ 帧对拍证明接触区域暗化;2026-06-22 已补更宽多 blocker/多接触区域场景,后续保留 RenderDoc 证据与 root wider/locked checks。

测试阶段:
- `cargo test -p zircon_runtime shadow --locked` 与 `cargo test --manifest-path zircon_plugins/Cargo.toml -p <rendering feature crate> --locked`
- 验收证据:质量档位切换产物差异符合预期;feature 关闭时 graph 无对应 pass。

## 工程落地细化

本章为计划 05 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、std430 约定、WGSL include 前缀、测试命名等全局约定直接引用 index.md §8,不在此重定义。跨计划契约名原样使用:计划 01 `RgTextureHandle`/`RgBufferHandle`/`TransientResourcePool`、计划 02 `MeshPassProcessor`(语义槽命令)、计划 03 `GpuScene`/`GpuSceneIdAllocator`、计划 04 `ViewVisibilityContext`/`PrimitiveRelevance`、计划 09 `RenderLayer`、计划 10 `RendererCommon`。

### 模块与文件落点

新增文件(facade 固定 `zircon_runtime::core::framework::render`,实现归 `graphics/`):

| 路径 | 内容 | 层 |
|------|------|----|
| `zircon_runtime/src/core/framework/render/light/gpu_light.rs` | `GpuLightData`、`GpuLightType`、`SHADOW_SLOT_NONE`(纯 POD + bytemuck,无 wgpu) | framework 契约 |
| `zircon_runtime/src/core/framework/render/light/shadow_settings.rs` | `LightShadowSettings`(bias/normal bias/分辨率偏好/strength,编辑器面板对接面) | framework 契约 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/mod.rs` | 模块 wiring(thin) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs` | light buffer 进 `GpuScene`:打包 snapshots → `GpuLightData`、脏更新、容量增长 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs` | CPU zbin+tile 构建(URP Jobs 形态的 Rust 移植)+ `LightGridParams` 计算 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_pass.rs` | `lighting.light-grid` executor:buffer 写入与 grid 统计进 RenderStats | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl` | 共享 include:grid 查询函数(无 entry point) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/mod.rs` | atlas 模块 wiring、allocator/resources 出口 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs` | `ShadowAtlasAllocator`(shelf 分配器)+ 滞回/降档策略 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs` | `shadow_atlas` group1 binding ABI:8 atlas/9 sampler/10 slots/11 globals | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs` | `ShadowAtlasResources`:持久 WGPU atlas、compare sampler、slot storage/global uniform buffer owner | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs` | `CascadeSplitConfig`、log/linear 混合分割、texel snapping、过渡带计算 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs` | `ShadowFramePlan`:extract 灯光 → atlas allocation、shadow slot/global payload、atlas slot-pass descriptors 与 light buffer slot 写回;不再持有 view-projection helper | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/view_projection.rs` | directional cascade、point cube face 与 spot shadow view-projection 矩阵构造;同时拥有方向/距离 sanitizing 和 stable up-vector 规则 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs` | `GpuShadowSlot`/`GpuShadowGlobals` POD、atlas allocation 到 slot 打包、flags/layout 常量 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl` | 共享 include:slot 采样、CSM 选择、PCF 核 | WGSL |
| `zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs` | `rendering.contact_shadow` 可选 feature runtime:注册 HZB 驱动 `contact-shadow` graph pass、WGPU compute executor 与契约测试 | rendering plugin |
| `zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs` | contact shadow 真实 WGPU 产品捕获:plugin-enabled/baseline Forward+ 帧对拍、executor dispatch、graph coverage 与更宽多接触区域守卫 | rendering plugin test |
| `zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl` | contact shadow compute shader:读 scene depth/normal/HZB,写 Rgba8Unorm visibility 输出 | WGSL |
| `zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs` | `rendering.contact_shadow` editor capability/manifest 投影 | rendering plugin |

修改文件:

| 路径 | 改动 |
|------|------|
| `zircon_runtime/src/core/framework/render/light/snapshots.rs` | 各 snapshot 增加 `light_id: u64`(稳定 id,extract 端生成)、`layer_mask: u32`、`shadow: Option<LightShadowSettings>` |
| `zircon_runtime/src/core/framework/render/light/readiness.rs` | 删除 `BASIC_SCENE_UNIFORM_*_LIMIT` 与 `ready_point_light_count` 等截断逻辑;readiness 改为 buffer 容量导向(LS-M1 硬切换) |
| `zircon_runtime/src/core/framework/render/light/mod.rs` | re-export 更新(thin) |
| `zircon_runtime/src/core/framework/render/post_process/stack.rs` | `PostProcessGraphResourceNames` 新增/保留 `LIGHT_ZBINS`、`LIGHT_TILE_MASKS`、`SHADOW_ATLAS`、`CONTACT_SHADOW_OCCLUSION`;`LIGHT_LIST` 仅作为旧 tile tint/post-process 过渡资源保留,grid 着色切换后删除;LS-M3 已删除旧 `SHADOW_MAP` graph 资源 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs` | `post.stack` 声明读取 `CONTACT_SHADOW_OCCLUSION`,让 contact shadow transient 生命周期覆盖最终后处理 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs` | 为插件 executor 暴露 resolver-aware `require_texture_view(...)`,让 pass 声明的 texture view 可经公共 GPU context 解析 |
| `zircon_runtime/src/graphics/scene/scene_renderer/post_process/**` | `SceneRuntimeFeatureFlags::contact_shadow_enabled`、`PostProcessParams::lighting_flags`、post-process bind group binding 27 与 WGSL contact shadow 采样/合成路径 |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs` | 删除 `light_dir`/`light_color`/`point_light_position_range`/`point_light_color_intensity`/`point_light_params` 字段(ambient 保留) |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs` | 删除 `authored_point_light_data`/`empty_point_light_data` |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl` + `mesh/mesh_pipeline/fallback_mesh_shader_source.rs` | 灯光循环改读 light buffer(LS-M1 全量循环→LS-M2 grid 驱动);shadow 采样按计划 03 GS-M2 槽位重排进 group1 |
| `zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/{create.rs,shader_source.rs}` + `deferred/lighting_bind_group_layout/` | deferred lighting 从 uniform 灯光循环切 grid 驱动;group1 layout 与 forward 共用同一布局常量 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs` | atlas caster renderer:按 `ShadowAtlasSlotPass` 更新 scene uniform、viewport/scissor 并 replay shadow stream 写 atlas;旧单 shadow map 直录路径、`shadow_light`/`shadow_scene_uniform` 等 legacy receiver 辅助已删除 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs` | pass 改名 `light-grid-build`,IO:`write_buffer(LIGHT_ZBINS)` + `write_buffer(LIGHT_TILE_MASKS)`(不再 read SCENE_DEPTH,zbin/tile 为 CPU 构建,见帧时序);过渡期仍写 `LIGHT_LIST` 供旧调试/tint executor 消费 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs` | pass 已收束为 `shadow-atlas`/`shadow.atlas`,只 `write_external_with_ops(SHADOW_ATLAS, ...)`;旧 `SHADOW_MAP` 写入已删除 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs` | 对插件公开 `record_compute_dispatch(...)`,允许插件 executor 把真实 compute dispatch 与 storage 写资源汇入帧执行记录 |
| `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/{default_forward_plus.rs,default_deferred.rs}` | feature 列表不变(`Shadows`/`ClusteredLighting` 复用),无新增 stage |

### 核心类型与接口

framework 契约层(`light/gpu_light.rs`,无 wgpu):

```rust
#[repr(u32)]
pub enum GpuLightType { Directional = 0, Point = 1, Spot = 2, Rect = 3 }

pub const SHADOW_SLOT_NONE: u32 = 0xFFFF_FFFF;

/// std430,96 B/灯;偏移见下节布局表。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq)]
pub struct GpuLightData {
    pub position_range: [f32; 4],     // xyz 位置(directional 不用),w = range
    pub color_intensity: [f32; 4],    // rgb 线性色,w = intensity
    pub direction_type: [f32; 4],     // xyz 方向,w = f32::from_bits(GpuLightType)
    pub spot_angles_size: [f32; 4],   // x cos(inner) y cos(outer);rect: zw 半宽/半高
    pub shadow_slot_layer: [u32; 4],  // x 首 shadow 槽(或 SHADOW_SLOT_NONE),y layer_mask,z light_id 低 32 位,w flags
    pub shadow_params: [f32; 4],      // x strength,y depth_bias,z normal_bias,w 级联数/槽数
}
```

```rust
// light/shadow_settings.rs(framework 契约)
pub struct LightShadowSettings {
    pub casts_shadow: bool,
    pub depth_bias: f32,          // 以 shadow texel 世界尺寸为单位(URP GetShadowBias 语义)
    pub normal_bias: f32,
    pub strength: f32,
    pub resolution_preference: ShadowResolutionTier, // 见 atlas 档位表
}
```

graphics 实现层:

```rust
// lighting/light_grid_builder.rs
pub(crate) struct LightGridParams {
    pub zbin_scale: f32, pub zbin_offset: f32,
    pub bin_count: u32, pub words_per_tile: u32,
    pub tile_resolution: [u32; 2], pub tile_size_px: u32, pub light_count: u32,
}
pub(crate) struct LightGridCpuOutput { pub zbins: Vec<u32>, pub tile_masks: Vec<u32>, pub params: LightGridParams }
pub(crate) fn build_light_grid(lights: &[GpuLightData], view: &LightGridViewInfo) -> LightGridCpuOutput;

// shadow/atlas/allocator.rs
pub(crate) enum ShadowResolutionTier { T128 = 128, T256 = 256, T512 = 512, T1024 = 1024, T2048 = 2048 }
pub(crate) struct ShadowSlotRequest { pub light_id: u64, pub face_index: u8, pub tier: ShadowResolutionTier, pub priority_score: f32 }
pub(crate) struct ShadowSlotAllocation { pub slot_index: u32, pub atlas_rect_px: [u32; 4], pub tier: ShadowResolutionTier }
impl ShadowAtlasAllocator {
    pub fn allocate_frame(&mut self, requests: &[ShadowSlotRequest]) -> Vec<ShadowSlotAllocation>; // 含滞回与降档
}

// shadow/cascade.rs
pub(crate) struct CascadeSplitConfig { pub cascade_count: u32 /*<=4*/, pub max_distance: f32, pub log_linear_lambda: f32 /*默认 0.7*/, pub fade_fraction: f32 /*默认 0.1*/ }
pub(crate) fn compute_cascade_splits(cfg: &CascadeSplitConfig, near: f32) -> [f32; 5];
pub(crate) fn snapped_cascade_view_proj(light_dir: Vec3, frustum_slice: &FrustumSlice, resolution: u32) -> Mat4; // texel snapping

// shadow/slot.rs
#[repr(C, align(16))]
pub(crate) struct GpuShadowSlot { pub view_proj: [[f32; 4]; 4], pub atlas_scale_bias: [f32; 4], pub params: [f32; 4] } // 96 B
#[repr(C, align(16))]
pub(crate) struct GpuShadowGlobals { pub cascade_splits: [f32; 4], pub cascade_fade_lengths: [f32; 4], pub atlas_params: [f32; 4] } // 48 B

// shadow/atlas/resources.rs
pub(crate) struct ShadowAtlasResourceConfig { pub width: u32, pub height: u32, pub slot_capacity: u32 }
pub(crate) struct ShadowAtlasResources; // owns atlas texture/view, compare sampler, shadow_slots storage, shadow_globals uniform

// shadow/atlas/bindings.rs
pub(crate) const SHADOW_ATLAS_BINDING: u32 = 8;
pub(crate) const SHADOW_ATLAS_SAMPLER_BINDING: u32 = 9;
pub(crate) const SHADOW_ATLAS_SLOT_BUFFER_BINDING: u32 = 10;
pub(crate) const SHADOW_GLOBALS_BINDING: u32 = 11;

// shadow/plan.rs
pub(crate) struct ShadowFramePlan; // owns frame slots/globals and per-light first-slot assignments
pub(crate) struct ShadowLightSlotAssignments; // patches GpuLightData.shadow_slot_layer.x + shadow_params.w
```

阴影 view 剔除经计划 04 `ViewVisibilityContext`(per-light view,`casts_shadow` relevance 由 `PrimitiveRelevance` 提供);shadow pass 命令经计划 02 `MeshPassProcessor` 生成语义槽命令,不直接 `set_pipeline`。

### GPU 数据布局与 WGSL 约定

`GpuLightData` std430 偏移(96 B/灯,light buffer 为 `array<GpuLightData>` storage,进 `GpuScene` 管理,id 由 `GpuSceneIdAllocator` 分配):

| 偏移 | 字段 | 类型 |
|------|------|------|
| 0 | position_range | vec4f |
| 16 | color_intensity | vec4f |
| 32 | direction_type | vec4f(w 位转 u32 类型) |
| 48 | spot_angles_size | vec4f |
| 64 | shadow_slot_layer | vec4u |
| 80 | shadow_params | vec4f |

zbin buffer(`array<u32>`):每 bin 跨度 `2 + words_per_tile` 个 u32。word0 = 灯光 header `min_index & 0xFFFF | max_index << 16`(URP `ZBinningJob.EncodeHeader` 同构;空 bin 为 `0xFFFF | 0 << 16`);word1 = 保留给计划 11 反射探针 header;word2.. = 灯光位掩码(bit i = 灯 i 与该 bin 相交)。bin 索引(透视):`bin = log2(view_z) * zbin_scale + zbin_offset`,其中 `zbin_scale = ZR_MAX_ZBIN_WORDS / ((log2(far) - log2(near)) * (2 + words_per_tile))`、`zbin_offset = -log2(near) * zbin_scale`(URP ForwardLights.cs 同式;正交用线性 z)。常量:`ZR_MAX_ZBIN_WORDS = 4096`、`ZR_MAX_TILE_WORDS = 8192`。

tile mask buffer(`array<u32>`):`tile_masks[(tile_y * tile_res.x + tile_x) * words_per_tile + word]`,bit i = 灯 i 覆盖该 tile。`words_per_tile = (light_count + 31) / 32`;tile 边长从 8px 起倍增直至 `tile_res.x * tile_res.y * words_per_tile <= ZR_MAX_TILE_WORDS`(URP `actualTileWidth` 同策略)。着色时灯光遍历集合 = `zbin_word & tile_word` 的按位与(URP 双重裁剪同构)。

group1(pass 级,index.md §8;LS-M3 已完成 atlas receiver hard cut。forward 着色 pass 与
deferred lighting pass 都从 group3 `GpuScene` 读取 `array<GpuLightData>` light buffer,group1 只放 pass-local 纹理/采样器/
light-grid 资源):

Forward group1 当前布局:

| binding | 资源 | 类型 |
|---------|------|------|
| 8 | shadow_atlas | `texture_depth_2d` |
| 9 | shadow_sampler | `sampler_comparison` |
| 10 | shadow_slots | `var<storage, read> array<ZrShadowSlot>` |
| 11 | shadow_globals | `var<uniform> ZrShadowGlobals` |
| 20 | light_grid_params | `var<uniform> ZrLightGridParams` |
| 21 | light_zbins | `var<storage, read> array<u32>` |
| 22 | light_tile_masks | `var<storage, read> array<u32>` |

Deferred lighting group1 当前布局:

| binding | 资源 | 类型 |
|---------|------|------|
| 0..3 | gbuffer_albedo/normal/background/material | `texture_2d<f32>` |
| 4 | scene_depth | `texture_depth_2d` |
| 8 | shadow_atlas | `texture_depth_2d` |
| 9 | shadow_sampler | `sampler_comparison` |
| 10 | shadow_slots | `var<storage, read> array<ZrShadowSlot>` |
| 11 | shadow_globals | `var<uniform> ZrShadowGlobals` |
| 20 | light_grid_params | `var<uniform> ZrLightGridParams` |
| 21 | light_zbins | `var<storage, read> array<u32>` |
| 22 | light_tile_masks | `var<storage, read> array<u32>` |

LS-M3 atlas binding ABI 已在 `shadow/atlas/bindings.rs` 固化,并已插入当前 forward/deferred layout:

| binding | 资源 | 类型 |
|---------|------|------|
| 8 | shadow_atlas | `texture_depth_2d` |
| 9 | shadow_sampler | `sampler_comparison` |
| 10 | shadow_slots | `var<storage, read> array<ZrShadowSlot>` |
| 11 | shadow_globals | `var<uniform> ZrShadowGlobals` |

这些 binding 避开 light-grid 的 20/21/22,并保留 group1 低号位给 forward material/deferred gbuffer。layout 接入、`zr_shadow.wgsl`
和 `GpuLightData.shadow_slot_layer` 写回已完成;`SHADOW_ATLAS` 现在由 graph 声明为 external 资源,并由
`shadow-atlas` pass / `shadow.atlas` executor 逐 slot 写入真实 depth atlas。Plan 04 per-light/per-cascade shadow view 剔除已接入;仍需捕获对拍,
处理必要的 caster expansion/receiver slice 覆盖风险。

`ZrShadowSlot`(std430,96 B,CPU POD 已落地为 `GpuShadowSlot`):`view_proj: mat4x4f`(offset 0)、
`atlas_scale_bias: vec4f`(64,xy scale zw offset,归一化 atlas UV)、`params: vec4f`(80,x depth_bias y normal_bias
z slot_texel_size w flags)。`ZrShadowGlobals`(CPU POD `GpuShadowGlobals`,48 B):`cascade_splits: vec4f`、
`cascade_fade_lengths: vec4f`、`atlas_params: vec4f(width,height,inv_width,inv_height)`。

`zr_light_grid.wgsl`(include,只含函数与 struct):

```wgsl
struct ZrLightGridParams { zbin_scale: f32, zbin_offset: f32, bin_count: u32, words_per_tile: u32,
                           tile_resolution: vec2<u32>, tile_size_px: u32, light_count: u32 }
fn zr_light_zbin_index(view_z: f32, p: ZrLightGridParams) -> u32
fn zr_light_tile_base(frag_coord: vec2<f32>, p: ZrLightGridParams) -> u32   // tile_masks 基址
fn zr_light_zbin_header(bin: u32, p: ZrLightGridParams) -> vec2<u32>        // (min_index, max_index)
fn zr_light_mask_word(tile_base: u32, bin: u32, word: u32, p: ZrLightGridParams) -> u32 // zbin_word & tile_word
fn zr_light_count(frag_coord: vec2<f32>, view_z: f32, p: ZrLightGridParams) -> u32      // 统计/debug 用
```

WGSL 无闭包,`for_each_light` 以调用方循环骨架落地(forward 与 deferred 共用同一段拼接模板,由计划 08 模板机制注入):外层 `for word in 0..words_per_tile`,取 `zr_light_mask_word`,内层 `while mask != 0u { let i = firstTrailingBit(mask); mask &= mask - 1u; shade(light_buffer[word*32u+i]); }`,并以 `zr_light_zbin_header` 的 min/max 截断 word 区间。layer 过滤:`(light.shadow_slot_layer.y & view_layer_mask) != 0u`(`RenderLayer` 语义,计划 09)。

`zr_shadow.wgsl`(include):

```wgsl
fn zr_sample_shadow_slot(slot_index: u32, world_position: vec3<f32>) -> f32
fn zr_shadow_cascade_index(view_z: f32, cascade_count: u32) -> u32
fn zr_sample_directional_shadow(light: ZrGpuLightData, world_position: vec3<f32>, view_z: f32) -> f32
fn zr_point_shadow_face_index(light_position: vec3<f32>, world_position: vec3<f32>) -> u32
fn zr_gpu_light_shadow_visibility(light: ZrGpuLightData, light_type: u32, world_position: vec3<f32>, view_z: f32) -> f32
```

CSM 约定:方向光的 `shadow_slot_layer.x` 是首个 cascade slot,`shadow_params.w` 是 cascade 数;`zr_sample_directional_shadow`
按 `zr_shadow_globals.cascade_splits` 选择级联,并在 `split[i] - fade` 区间内对 i 与 i+1 级联结果 lerp(UE `SplitNearFadeRegion`/
`FadePlaneOffset` 语义的单带简化)。分割公式(log/linear 混合):`split_i = lerp(near + (far-near)*i/N,
near*(far/near)^(i/N), lambda)`,`lambda` 默认 0.7(UE 等价物为 `ComputeAccumulatedScale` 指数分布,exponent 默认 4;
两者曲线接近,取公式更简的混合式)。texel snapping:级联包围球半径取 slice 八角点最大距(半径跨帧锁定到 1% 步进),
`world_per_texel = 2*radius/resolution`,将 light view 空间下的视锥中心 `fmod` 对齐到 `world_per_texel` 网格再回变换
(UE ShadowSetup.cpp `SnapX/SnapY` 同构)。

PCF 档位表(进 `ZrShadowSlot.params.w` flags 与 pipeline 常量):

| 档位 | taps | 适用 |
|------|------|------|
| Low | 1(硬件 comparison 双线性) | 默认/低端 |
| Medium | 5(十字) | spot/point |
| High | 9(3x3) | 方向光近级联 |

`ShadowPcfQuality::default()` 为 Low;需要保持旧 3x3 行为的产品/测试合同必须显式写 `ShadowPcfQuality::High`。
CPU slot flags 与 WGSL 常量共享 bit 8..9 作为质量字段,bit 0 仍为 valid flag,bit 1..3 保留方向光/spot/point-face 类型标记。

ShadowAtlas:单张 `4096x4096` D32Float(capability 不足降 2048,档位整体降一档)。方向光 CSM 固定占顶部 `4096x1024` 行(4 x 1024²);其余区域 shelf 分配。

### 帧时序与集成点

帧内顺序(全部经 graph 节点 + executor id,无旁路):

1. extract:`RenderFrameExtract` 灯光 snapshots(带 light_id/layer_mask/shadow settings)。
2. prepare(CPU):`light_buffer.rs` 打包 + 脏更新上传(GpuScene grow);`cascade.rs` 计算级联;`ShadowAtlasAllocator::allocate_frame` 产出槽位;`build_light_grid` CPU 构建 zbin/tile(对齐 URP:URP 的 zbin/tile 即 CPU Jobs 构建,不依赖 depth;故 `light-grid-build` 节点不读 SCENE_DEPTH)。
3. graph 节点 `light-grid-build`(executor `lighting.light-grid`,QueueLane::AsyncCompute 保留):LS-M2 形态为 CPU builder 输出的 zbin/tile buffer 上传 + 统计;后续若切 GPU compute 构建,IO 合同不变。zbin/tile buffer 经计划 01 `TransientResourcePool` 以 `RgBufferHandle` 申请。
4. graph 节点 `shadow-atlas`(executor `shadow.atlas`)已启用;只写 external `SHADOW_ATLAS`,按 `ShadowAtlasSlotPass` 槽位批次渲染。命令暂复用 shadow command stream,但 replay 已按 `ShadowAtlasSlotPass.view_key` 消费计划 04 的 directional cascade/point face/spot shadow view 可见集。
5. depth prepass → forward 着色 / deferred geometry + lighting:同读 group1 的 grid 与 atlas;当前 forward/deferred pass 已声明读取 `SHADOW_ATLAS` 以建立 graph 顺序。

硬切换删除清单(LS-M1/M3 各自变更内完成,不留双路径):

- `scene_uniform.rs`:`light_dir`、`light_color`、`point_light_position_range`、`point_light_color_intensity`、`point_light_params` 字段及 `from_frame.rs` 的 `authored_point_light_data`/`empty_point_light_data`。
- `readiness.rs`:`BASIC_SCENE_UNIFORM_DIRECTIONAL_LIGHT_LIMIT`、`BASIC_SCENE_UNIFORM_POINT_LIGHT_LIMIT`、`ready_directional_light_count`、`ready_point_light_count`(spot `ready=0` 的硬编码一并删除)。
- `fallback_mesh.wgsl` 与 deferred `lighting_pipeline/shader_source.rs`:scene uniform 灯光循环段。
- `shadow_map_renderer.rs`:`shadow_light`、`shadow_light_from_directional`、`shadow_scene_uniform`、`shadow_view_projection`、`shadow_bounds_from_frame`、`ShadowMapRenderer` 旧单贴图 `record_commands_with_attachment_ops` 直录路径(LS-M3 已完成)。
- `post_process/stack.rs`:`LIGHT_LIST` 资源名在 shader grid 驱动与 tile tint 兼容路径收束后删除;`SHADOW_MAP` 已在 LS-M3 `SHADOW_ATLAS` 切换时删除,消费点同变更内改名。

### 实施切片细化

LS-M1(灯光上限解除):
1. 触碰:`gpu_light.rs`(新)、`snapshots.rs`、`light_buffer.rs`(新)、`readiness.rs`。要点:`GpuLightData` 定稿 + 打包/脏更新;readiness 截断逻辑删除。判据:`cargo check -p zircon_runtime --lib --locked` 过;layout 单测(96 B + 偏移断言)过。
2. 触碰:`fallback_mesh.wgsl`、deferred `shader_source.rs`、`scene_uniform.rs`/`from_frame.rs`、`stack.rs`。要点:着色循环改读 light buffer 全量循环(group1 binding0/1);scene uniform 灯光段删除。判据:`render_product` 既有场景产物不回归;>8 点光生效。

LS-M2(clustered grid):
1. 触碰:`light_grid_builder.rs`(新)、`light_grid_pass.rs`(新)、`clustered_lighting.rs`、`zr_light_grid.wgsl`(新)。要点:CPU zbin/tile 构建 + transient buffer 上传 + graph IO 改 `LIGHT_ZBINS`/`LIGHT_TILE_MASKS`。判据:CPU 参考(暴力逐灯求交)与 builder 输出 mask 全等单测过。
2. 触碰:`fallback_mesh.wgsl`、deferred lighting shader、两处 bind group layout。要点:全量循环切 grid 驱动(同一 include),保留全量循环作为 capability 回落档(同一 buffer ABI)。判据:forward/deferred 产物对拍一致;grid 统计进 RenderStats。

LS-M3(CSM + atlas):
1. 已完成:触碰 `cascade.rs`(新)、`atlas/`(新)、`slot.rs`(新)、`shadow_settings.rs`(新)。要点:分割/snapping/fade 计算与 shelf 分配器(滞回:槽位保留 8 帧,抢占需连续 4 帧得分高 25%;降档:URP `EstimateScaleFactorNeededToFitAllShadowsInAtlas` 同策略整体除 2),并定稿 `GpuShadowSlot`/`GpuShadowGlobals` ABI。判据:分配器/级联/slot layout 源码单测覆盖(档位/驱逐/滞回/降档/offset),core-min `cargo check` 通过。
2. 已完成 bridge 写入:触碰 `shadows.rs` descriptor、`post_process/stack.rs`、`execute_graph_stage.rs`、`shadow_map_renderer.rs`、`zr_shadow.wgsl`、`GpuLightData.shadow_slot_layer` 写入。要点:`SHADOW_ATLAS` 作为 external graph resource 被导入并写入,forward/deferred shader 通过 atlas slot 采样;point 光取立方体 6 面(6 槽,双抛物面留作降档预研不实施)。判据:core-min `cargo check` 通过。
3. 已完成 shadow view 过滤切片:触碰 `view_context/mod.rs`、`view_context/build_views.rs`、`construct.rs`、`shadow/plan.rs`、`shadow_map_renderer.rs`、`mesh_pass/mesh_draw_command.rs`、`mesh_pass/mesh_pass_processor.rs`。要点:`VisibilityViewKey` 可拷贝进 `ShadowAtlasSlotPass`;方向光 cascade、point face、spot slot pass 分别记录对应 Plan 04 shadow view;shadow replay 以 command source entity 对 view visible set 做逐命令过滤。indirect shadow stream 在需要 view 过滤时仍走逐命令 direct replay,避免按槽位过滤错误复用整批 indirect args。判据:core-min `cargo check` 通过。
4. 已完成方向光 frustum-slice 精化:触碰 `shadow/cascade.rs`、`shadow/plan.rs`、`scene/mod.rs`、`scene_renderer/mod.rs` 与 Plan 04 `view_context/build_views.rs`/`construct.rs`。要点:`cascade_shadow_bounds_from_camera_slice(...)` 从主相机 frustum slice 八角点推导每 cascade center/radius,`ShadowFramePlan` 和 visibility shadow view 共享该 split/bounds 口径,atlas slot pass 第 0/3 级联矩阵不再相同。判据:core-min check 与 focused shadow/visibility tests 通过。
5. 已完成 receiver 硬切:删除 legacy `SHADOW_MAP` graph resource、forward/deferred 单贴图 receiver bindings 与 Rust receiver uniform buffers;forward/deferred/builtin PBR 只通过 atlas slot 采样。判据:core-min `cargo check` 通过。
6. 已完成 caster/receiver 源码保护:触碰 `mesh/build_mesh_draws/build/build.rs`。要点:`mesh_visibility_states(...)` 将 shadow-only caster 映射为 `main_view_visible=false` 且 `shadow_view_visible=true`,证明主视图剔除不会切掉 shadow view 可见 caster。判据:source test 已添加,core-min `cargo check` 通过。
7. 已完成产品合同源码断言:触碰 `graphics/tests/render_product_shadows.rs`。要点:`render_product_csm_directional` 锁住方向光 CSM 4 槽与 per-cascade view key,`render_product_multi_spot_shadows` 锁住 ≥3 spot 阴影同帧 atlas 槽位不重叠。判据:core-min `cargo check` 通过。
8. 已完成方向光 atlas 路径与可见暗化真实捕获:触碰 `shadow/atlas/resources.rs`、`graphics/tests/render_product_shadow_captures.rs` 与测试模块注册。要点:真实 WGPU Forward+ 产品提交中证明 `shadow.atlas` executor、atlas 写入、receiver 读取、caster draw、receiver sample 同帧成立,并用同色 receive-shadow 开关对拍证明 receiver 可见变暗;比较采样器固定为 `GreaterEqual` 以匹配 depth atlas 可见性合同。判据:comparison contract Cargo filter 与两个产品 direct exact rerun 均通过。
9. 已完成 multi-spot 与 CSM 平移稳定真实视觉捕获验收:多 spot 真实捕获守卫已接入并通过源码格式、core-min lib/`--tests` 类型检查、lib-test `--no-run` 和直接 WGPU 执行;CSM subtexel 相机平移守卫用同相机 shadowed/unshadowed 对拍抵消投影位移后比较暗化统计,证明轻微相机移动没有让 receiver 阴影大幅 swimming。判据:多 spot 阴影并存;相机平移抓帧无 swimming。
10. 已完成 forward/deferred shadow parity 真实 WGPU 对拍:新增 `render_product_directional_shadow_atlas_forward_deferred_darkening_parity`,复用 directional receiver/caster 场景分别跑 Forward+ 与 Deferred shadowed/unshadowed 捕获,并把 Deferred G-buffer material alpha 扩展为低 7 位 shading model + 高位 receive-shadow flag,让 `lighting.deferred` 与 Forward+ 一样尊重 receiver 的 receive-shadow 开关。剩余 caster expansion/receiver slice 与 RenderDoc 风险仍待补。

LS-M4(PCF 与 contact shadow):
1. 已完成:触碰 `zr_shadow.wgsl`、`shadow_settings.rs`、`shadow/slot.rs`、`shadow/plan.rs` 及三处 shader source 断言。要点:1/5/9 tap 档位 + per-light pcf quality/bias 槽位合同定稿(URP GetShadowBias 的 texel 尺度语义保留在 bias 字段)。判据:slot flags 与 shader source 单测覆盖。
2. 已完成 PCF 质量真实捕获:触碰 `render_product_shadow_captures.rs` 与三处 shader source 断言。要点:spot receiver-edge 场景跑 Low/High/unshadowed 三帧并比较 darkened pixels、luma delta 与 RGB delta;High PCF 半径常量纳入 shader source 合同。判据:exact WGPU capture 过滤测试 + `shadow_atlas_resources` 过滤测试通过。
3. 已完成 contact shadow 最小执行闭环:触碰 `zircon_plugins/` rendering feature crate、`contact_shadow.wgsl` 与 `RenderPassGpuExecutionContext` 插件 dispatch 记录入口。要点:经 RenderFeature descriptor 接入;feature 关闭时 compiled graph 无该 pass;启用时 WGPU compute executor 读计划 04 HZB 和 depth/normal,写 `CONTACT_SHADOW_OCCLUSION`,并记录实际 dispatch。判据:插件测试 + graph 节点存在性断言 + executor/shader 源码合同。
4. 已完成 contact shadow 真实 WGPU 产品捕获:触碰 `wgpu_product_tests.rs` 与 `gpu/resource_lookup.rs`。要点:同一 Forward+ receiver/blocker 场景跑 plugin-enabled 与 baseline pipeline,断言 feature/pass/executor/dispatch/coverage 并比较最终帧暗化。2026-06-22 追加宽 receiver + 左/中/右多 blocker 场景,验证三个接触窗口都暗化且 open receiver 不吞掉主要统计。判据:contact shadow exact 过滤测试与插件 runtime 全量 lib 测试均通过。

切片期一律 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime --lib --locked`(过滤词 `light` / `lighting` / `shadow`)。

### 测试与验收清单

| 测试函数 | 断言要点 | 位置 |
|----------|---------|------|
| `render_light_buffer_layout_matches_wgsl_offsets` | `size_of::<GpuLightData>() == 96` + `offset_of!` 六字段 | `light/gpu_light.rs` |
| `render_light_buffer_uploads_only_dirty_lights` | 增删改后上传区间仅含脏灯 | `lighting/light_buffer.rs` |
| `render_light_grid_zbin_header_encodes_min_max` | header 编码/空 bin 哨兵与 URP 语义一致 | `lighting/light_grid_builder.rs` |
| `render_light_grid_cpu_matches_brute_force_reference` | 随机灯集:builder mask == 暴力求交 mask | 同上 |
| `render_light_grid_tile_words_respect_budget` | tile 边长倍增至满足 `ZR_MAX_TILE_WORDS` | 同上 |
| `render_light_grid_layer_mask_filters_lights` | mask 不匹配的灯不进着色集合 | 同上 |
| `render_shadow_atlas_allocates_tiers_descending` | shelf 按档位降序铺排,无重叠 | `shadow/atlas/allocator/tests.rs` |
| `render_shadow_atlas_global_downgrade_fits_pressure` | URP 风格整体面积估算导致统一降档后仍能铺满 | 同上 |
| `render_shadow_atlas_evicts_lowest_priority_on_pressure` | 超容时低分槽被驱逐/降档 | 同上 |
| `render_shadow_atlas_hysteresis_prevents_flapping` | 分数交替互换时槽位 8 帧内不换主 | 同上 |
| `render_shadow_atlas_preempts_after_confirmed_priority_margin` | 连续 4 帧且高 25% 才允许抢占既有槽位 | 同上 |
| `render_shadow_atlas_scale_bias_matches_slice_transform` | atlas scale/bias 与 URP `ApplySliceTransform` 的归一化形态一致 | 同上 |
| `render_shadow_cascade_splits_blend_log_linear` | lambda=0/1 退化为线性/对数;单调递增 | `shadow/cascade.rs` |
| `render_shadow_cascade_ranges_are_monotonic_and_have_fade_bands` | split range 单调且 fade_start 落在级联范围内 | 同上 |
| `render_shadow_cascade_snapping_quantizes_origin` | 平移相机半 texel,snapped 矩阵不变 | 同上 |
| `render_shadow_cascade_view_projection_is_stable_under_half_texel_motion` | half-texel light-space motion 不改变 snapped VP 矩阵 | 同上 |
| `render_shadow_cascade_bounds_follow_camera_slice_depth` | 远 cascade slice 的 bounds 半径与中心距离大于近 cascade slice | 同上 |
| `render_shadow_slot_layout_matches_plan_05_std430_contract` | `GpuShadowSlot` 96 B + offset 0/64/80 | `shadow/slot.rs` |
| `render_shadow_slot_from_allocation_writes_atlas_slice_and_flags` | allocation 写入 atlas scale/bias、bias、texel size 和 flags | 同上 |
| `render_shadow_slot_encodes_pcf_quality_in_flags` | Low/Medium/High 稳定编码到 `GpuShadowSlot.params.w` bit 8..9 | 同上 |
| `render_shadow_globals_layout_and_atlas_params_are_stable` | `GpuShadowGlobals` 48 B + split/fade/atlas 参数布局 | 同上 |
| `render_shadow_atlas_resource_config_uses_plan_05_defaults` | atlas resource 默认 4096x4096/256 slots | `shadow/atlas/resources.rs` |
| `render_shadow_atlas_compare_function_matches_forward_depth_contract` | atlas 比较采样器使用 `GreaterEqual`,保持 depth atlas clear=1 与 `map_depth >= receiver_depth` 可见性合同 | 同上 |
| `render_shadow_atlas_resource_config_downgrades_to_capability_limit` | 设备纹理上限不足时降到 2048 fallback | 同上 |
| `render_shadow_atlas_upload_report_describes_cleared_tail` | 上传报告记录写入 slot 与清理 stale tail | 同上 |
| `render_shadow_atlas_group1_bindings_avoid_legacy_shadow_and_light_grid_slots` | atlas binding 8..11 不与现有 legacy shadow/light-grid 冲突 | `shadow/atlas/bindings.rs` |
| `render_shadow_atlas_group1_layout_entries_match_plan_05_resource_types` | 8 texture depth、9 compare sampler、10 storage、11 uniform | 同上 |
| `fallback_mesh_shader_receives_shadow_atlas_resources` | forward shader 声明 atlas 8..11 资源、调用 `zr_gpu_light_shadow_visibility`,覆盖 PCF 质量 helper,并断言旧单贴图 receiver 不存在 | `mesh/mesh_pipeline/fallback_mesh_shader_source.rs` |
| `deferred_lighting_shader_receives_shadow_atlas_resources` | deferred shader 声明 atlas 8..11 资源、调用 `zr_gpu_light_shadow_visibility`,覆盖 PCF 质量 helper,并断言旧单贴图 receiver 不存在 | `deferred/lighting_pipeline/tests.rs` |
| `builtin_pbr_shader_receives_shadow_atlas_resources` | builtin PBR shader 拼接 `zr_shadow.wgsl`、声明 atlas 8..11 资源,覆盖 PCF 质量 helper,并断言旧单贴图 receiver 不存在 | `asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs` |
| `render_shadow_frame_plan_assigns_first_directional_cascade_slots` | 首个方向光写入连续 CSM slot、atlas slot pass 与 globals split/fade | `shadow/plan/tests.rs` |
| `render_shadow_frame_plan_caps_directional_cascade_tier_to_atlas_row` | CSM tier 按实际 atlas 宽度/级联数与预留行高度降档 | 同上 |
| `render_shadow_frame_plan_builds_distinct_directional_cascade_matrices` | 方向光第 0/3 cascade atlas pass 与 slot view-projection 矩阵不同,证明不再共享 frame-bounds 投影 | 同上 |
| `render_shadow_frame_plan_assigns_point_light_contiguous_face_slots` | point 光 6 面 slot/atlas slot pass 完整分配后才建立 first-slot assignment,且每个 face 带 `ShadowPointFace` view key | 同上 |
| `render_shadow_frame_plan_assigns_spot_light_slot_view_key` | spot 光单槽 slot pass 带 `ShadowSpot` view key | 同上 |
| `render_shadow_frame_plan_encodes_per_light_pcf_quality` | point/spot shadow settings 的 Low/Medium 档位分别写入各自 atlas slot flags | 同上 |
| `contact_shadow_feature_registers_hzb_ray_march_pass` | `rendering.contact_shadow` 注册 HZB 驱动的 async compute pass、workload、输入/输出资源和默认关闭 manifest | `zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs` |
| `contact_shadow_graph_pass_is_absent_when_plugin_feature_is_disabled` | 插件 feature 禁用时 compiled graph 无 `contact-shadow`;启用后 pass 位于 `hzb-build` 之后并读 `HZB_FURTHEST`、写 `CONTACT_SHADOW_OCCLUSION` | 同上 |
| `contact_shadow_executor_accepts_declared_pass_contract` | executor 契约接受 async compute pass 名称、队列、side-effect flags 与 depth/normal/HZB/output 资源 | 同上 |
| `contact_shadow_executor_requires_gpu_after_contract_validation` | 契约通过后 metadata-only context 会明确要求 renderer GPU context,证明 executor 不再是 no-op | 同上 |
| `contact_shadow_executor_rejects_resource_contract_drift` | 输出资源缺失时报告 resource contract mismatch | 同上 |
| `contact_shadow_shader_declares_expected_compute_bindings` | WGSL 声明 depth/normal/HZB 输入、Rgba8Unorm storage 输出、8x8 workgroup 和 `textureStore` | 同上 |
| `contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region` | 真实 WGPU Forward+ plugin-enabled/baseline 对拍证明 contact shadow pass 执行、dispatch 记录、coverage 零缺口并使接触区域变暗 | `wgpu_product_tests.rs` |
| `visibility_context_builds_shadow_views_for_atlas_light_slots` | Plan 04 生成方向光 4 cascade、point 6 face、spot 1 view key,且方向光 cascade camera 随 slice 深度变化,供 atlas slot pass 消费 | `visibility/context/from_extract_with_history/construct.rs` |
| `shadow_atlas_view_filter_keeps_only_visible_source_entities` | atlas slot replay 过滤掉不在对应 shadow view visible set 中的 source entity | `shadow_map_renderer.rs` |
| `mesh_visibility_states_preserve_shadow_only_casters` | main-view 不可见但 shadow-view 可见的 caster 仍写成 `shadow_view_visible=true`,供 shadow pass 生成命令 | `mesh/build_mesh_draws/build/build.rs` |
| `mesh_batch_ref_emits_gpu_scene_instance_command` | `MeshDrawCommand` 保留 source entity,供 atlas slot view 过滤消费且不破坏 instance span | `mesh_pass/mesh_draw_command_list.rs` |
| `render_shadow_light_slot_assignments_patch_packed_light_contract` | assignment patch `GpuLightData.shadow_slot_layer.x` 与 `shadow_params.w` | 同上 |
| `render_product_many_point_lights` | 源码合同已覆盖 64 点光打包、light-grid 第二 word 与 Forward+/Deferred 编译图消费 | render_product 套件 |
| `render_product_many_point_lights_forward_deferred_capture_parity` | 真实 WGPU 捕获覆盖 Forward+/Deferred 基线与 64 点光 lit 场景,证明 light-grid stats 与中心区域亮度贡献保持一致产品范围 | 同上 |
| `render_product_hundred_point_lights_report_local_density_stats` | 真实 WGPU 产品提交覆盖 128 点 dense/spread 等总数场景,证明 peak/average cluster 负载随局部密度变化而非总灯数 | 同上 |
| `render_product_directional_shadow_atlas_capture_records_receiver_path` | 真实 WGPU Forward+ 捕获证明 directional `shadow.atlas` executor、atlas 写入、receiver 读取、caster draw 与可见 receiver sample 同帧成立 | `render_product_shadow_captures/directional.rs` |
| `render_product_directional_shadow_atlas_darkens_receiver_capture` | 同色 receiver receive-shadow 开关对拍证明 directional atlas 采样会使 receiver 区域可见变暗 | 同上 |
| `render_product_csm_directional_remains_stable_under_subtexel_camera_shift` | 真实 WGPU Forward+ CSM 场景在 baseline/subtexel camera shift 下分别做 shadowed/unshadowed 对拍,证明暗化像素和 luma delta 保持稳定、无明显 shadow swimming | `render_product_shadow_captures/directional.rs` |
| `render_product_directional_shadow_atlas_forward_deferred_darkening_parity` | 真实 WGPU directional receiver/caster 场景分别跑 Forward+ 与 Deferred shadowed/unshadowed 对拍,证明两管线都执行 atlas shadow 并产生同产品范围的 receiver 暗化 | `render_product_shadow_captures/directional.rs` |
| `render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture` | 真实 WGPU spot receiver/caster 场景用 Low/High PCF 与 unshadowed baseline 对拍,证明两档均产生 receiver 暗化且 High 宽核改变边缘截图产物 | `render_product_shadow_captures.rs` |
| `render_product_csm_directional` | 4 级联边界 fade 无硬缝 | 同上 |
| `render_product_multi_spot_shadows` | ≥3 spot 阴影同帧并存 | 同上 |
| `render_product_multi_spot_shadow_atlas_darkens_receivers_capture` | 真实 WGPU Forward+ 多 spot 场景断言 3 个 spot shadowed light、atlas 写/读、caster draw,并用 receive-shadow 开关统计全帧暗化像素;当前已通过 core-min lib/`--tests` 类型检查、lib-test `--no-run` 与直接 WGPU 捕获执行 | `render_product_shadow_captures.rs` |

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-24 | Project render quality/deferred tests owner split | render_project_render_quality_deferred_tests_owner_split_static_passed_cargo_deferred_implementation_cadence | Project render quality/deferred tests owner split 已把 `graphics/tests/project_render.rs` 中的 temporal material history、SSAO shared-HZB、clustered lighting quality profile 与 Deferred G-buffer product tests 迁入 `graphics/tests/project_render/render_quality.rs`；父测试 owner 从 1387 行降到 774 行，只保留基础 project render/export/material/wire tests、shared asset fixtures 与 `mod render_quality;`。新增 `runtime_15_project_render_quality_tests_are_child_owner` 锁定 moved test/helper ownership、父子 800 行预算和 Plan 04/05/08/render index/structure/review/render submit docs anchors。 | scoped rustfmt、static owner scan、line-count scan、docs-anchor scan、stale-path scan、touched-file whitespace scan 和 scoped `git diff --check` 已通过；line-count 当前为 parent 774、child 631、guard 98。Cargo/WGPU/RenderDoc 按 milestone implementation cadence 暂缓，不计通过。 | Cargo lane 空闲后补跑 `project_render` quality/deferred focused product group；Plan 05 剩余验收仍是 RenderDoc 与 root wider locked checks。 |
| 2026-06-24 | Render product shadow captures directional test owner split | render_plan05_shadow_capture_directional_tests_owner_split_static_passed_cargo_deferred_implementation_cadence | `graphics/tests/render_product_shadow_captures.rs` 从 1069 行 oversized directional/spot/multi-spot capture owner 收敛为 608 行 spot/multi-spot parent,保留 spot PCF、多 spot receiver capture、shared material/mesh/profile/darkening helpers 与 `mod directional;` 挂载；新增 538 行 `graphics/tests/render_product_shadow_captures/directional.rs` 承接 directional atlas path/darkening、CSM subtexel stability、Forward+/Deferred directional parity、directional frame extraction/capture helpers 与 pipeline executor assertions。新增 `runtime_15_render_product_shadow_captures_directional_tests_are_child_owner` 锁定 moved test/helper ownership、父子 800 行预算和 docs/status anchors。 | scoped rustfmt/static owner scans、line-count/docs-anchor scans、stale-path scan、touched-file whitespace scan 与 scoped diff-check 通过；Cargo/WGPU/RenderDoc 按 milestone implementation cadence 暂缓，不计通过。 | Cargo lane 空闲后补跑 directional shadow capture group、CSM subtexel stability 与 Forward+/Deferred directional parity；继续补 Plan 05 RenderDoc/root wider locked evidence。 |
| 2026-06-24 | Render product shadows many-point lights test owner split | render_plan05_product_shadows_many_point_lights_test_owner_split_static_passed_cargo_deferred_implementation_cadence | `graphics/tests/render_product_shadows.rs` 从 912 行 oversized mixed product-test owner 收敛为 300 行 shadow graph/CSM/spot parent，只保留 shadow-atlas graph contract、CSM directional、multi-spot shadow plan tests 和 shared shadow helpers；新增 571 行 `graphics/tests/render_product_shadows/many_point_lights.rs` 承接 `render_product_many_point_lights`、Forward+/Deferred 64 点光捕获对拍、128 点光局部密度统计、light-grid graph resource assertions、material/mesh/viewport fixtures 与 light-grid stats assertions。新增 `runtime_15_render_product_shadows_many_point_lights_tests_are_child_owner` 锁定 moved test/helper ownership、父子 800 行预算和 docs/status anchors。 | scoped rustfmt/static owner scans、line-count/docs-anchor scans、touched-file whitespace scan 与 scoped diff-check 通过；Cargo/WGPU/RenderDoc 按 milestone implementation cadence 暂缓，不计通过。 | Cargo lane 空闲后补跑 `render_product_many_point_lights*`、`render_product_hundred_point_lights_report_local_density_stats` 和 Plan 05 shadow product group；继续补 RenderDoc/root wider locked evidence。 |
| 2026-06-24 | Plan 05 shadow atlas/plan test owner split | render_plan05_shadow_atlas_plan_test_owner_split_static_passed_cargo_deferred_active_compile_lane | 按 R1.4/R4.3 收束 LS-M3 shadow atlas/CSM 测试归属:`graphics/scene/scene_renderer/shadow/atlas/allocator.rs` 从 899 行降到 683 行并保留 atlas request/downgrade/retention/packing production owner,新增 215 行 `graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs` 承接 allocator pressure/scale-bias tests;`graphics/scene/scene_renderer/shadow/plan.rs` 从 868 行降到 506 行并保留 frame-plan/cascade/point/spot slot orchestration,新增 360 行 `graphics/scene/scene_renderer/shadow/plan/tests.rs` 承接 cascade/point/spot/PCF/light-slot writeback tests。新增结构守卫 `runtime_15_shadow_atlas_plan_tests_are_child_owners` 锁定 moved-test ownership、四文件 800 行预算和 docs/status 锚点。 | scoped rustfmt/static scans、line-count scan、docs-anchor scan、touched-file whitespace scan 和 scoped diff-check 通过;当前存在 active compile lane,不启动新的 Cargo/WGPU/RenderDoc,不计通过。 | Plan 05 剩余验收仍是 RenderDoc 证据、root wider/locked checks,以及后续 allocator allocation/rejection pressure 产品统计接入切片。 |
| 2026-06-24 | Plan 05/F12 shadow dead-code suppression cleanup | render_plan05_shadow_dead_code_suppression_cleanup_static_passed_cargo_deferred_active_lanes | 按 E6/F12 收敛 shadow 生产子树 suppression:`shadow/mod.rs` 不再用模块级 `#[allow(dead_code)]` 遮盖 atlas/cascade/plan/slot owner,`shadow/atlas/mod.rs` 不再用 `#[allow(unused_imports)]` 维持宽 re-export 面,只导出当前外部消费的 allocator/resources/binding 合同。`ShadowFramePlan` 删除未消费的 `atlas_allocation` 存储字段和 `atlas_allocation()` getter;allocator 的 `allocation_for(...)`、`last_frame()` 与 `ShadowSlotAllocation::was_downgraded()` 收到 test cfg,保留分配器产品路径的 frame allocation 返回和 `ShadowFramePlan` 的 slot/pass/light assignment 行为。 | scoped `rustfmt --edition 2021 --check`、shadow 子树 suppression scan、`atlas_allocation(` stale scan 与 scoped diff-check 通过;diff-check 仅报告 LF/CRLF 提示。当前存在其他 cargo/rustc active lane,不启动新的 heavy Cargo/WGPU/RenderDoc,不计通过。 | Plan 05 剩余验收仍是 RenderDoc 证据、root wider locked checks,以及后续需要把 allocator allocation/rejection pressure 诊断接入产品统计时的独立切片。 |
| 2026-06-23 | Plan 05/09 shadow view-projection owner split | render_plan05_09_shadow_view_projection_owner_split_static_passed | 按 R1.4 先收敛 `shadow/plan.rs` 的近阈值职责:新增 `graphics/scene/scene_renderer/shadow/view_projection.rs`,承接 `directional_cascade_view_projection(...)`、`spot_light_view_projection(...)`、`point_light_face_view_projection(...)`、`point_light_face_axes(...)`、方向/距离 sanitizing、`Transform::looking_at` 与 `Mat4::perspective_rh` 阴影视图矩阵构造;`shadow/plan.rs` 保留 ShadowFramePlan 编排、atlas request/allocation、slot pass 与 light-buffer assignment owner。新增 `runtime_15_shadow_plan_view_projection_is_child_owner`,锁定 `mod view_projection;` 挂载、moved helpers 不回流和 docs/status 锚点。 | scoped `rustfmt --edition 2021 --check`、line-count、moved-helper/static scans、docs anchor scans 与 diff-check 通过。`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-plan05-shadow-view-projection-0623 --message-format short --color never` 在 304s 工具窗口超时且无输出,未生成测试二进制,不计 Cargo/WGPU 通过。 | 后续阴影规划新职责继续落到 folder-backed child owner;Plan 05 剩余验收仍是 RenderDoc 证据与 root wider locked checks。 |
| 2026-06-23 | Render index 当前状态总览拆分 | light buffer/light grid 完成,2026-06-21 `render_product_many_point_lights_forward_deferred_capture_parity` 已补齐 64 点光 Forward+/Deferred 真实 WGPU 捕获对拍,`render_product_hundred_point_lights_report_local_density_stats` 已补齐百灯局部密度统计守卫,directional/multi-spot shadow-atlas receiver darkening、`render_product_csm_directional_remains_stable_under_subtexel_camera_shift`、`render_product_directional_shadow_atlas_forward_deferred_darkening_parity`、`render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture`、`contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region` 与 `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions` 已补齐真实 WGPU 捕获证据,2026-06-22 `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture` 已补齐 mixed directional+spot wider shadow-atlas 产品守卫 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 05 Lighting/Shadows 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：RenderDoc 与 root wider locked checks 风险；验收缺口：需要 RenderDoc、root wider locked checks |
| 2026-06-15 | LS-M1 GpuLightData and light-limit removal | 已完成(核心 buffer/着色路径接入;真实多光源捕获见 2026-06-21 行) | `GpuLightData`/light buffer packer 与 dirty upload 接入;scene uniform 旧方向光/点光截断字段和 readiness 常量删除;forward fallback、deferred lighting 与内建 PBR 通过 GPUScene/light buffer 读取灯光,为 64 点光产品源码合同提供统一 ABI。 | core-min `cargo check` 通过;已构建 lib-test binary 并直接跑通 light ABI、packer、GPUScene light upload 和 shader source 过滤项;`render_product_many_point_lights` 已有 64 点光源码/graph 覆盖;真实 WGPU 多点光 forward/deferred captured parity 由 2026-06-21 行补齐。 | 仍需更宽 render-product 回归与非多点光阴影/contact shadow 更宽场景覆盖。 |
| 2026-06-15 | LS-M2 clustered light grid | 已完成(网格构建与着色消费接入;宽图回归待后续) | CPU zbin/tile builder、`zr_light_grid.wgsl`、transient zbin/tile buffers、`lighting.light-grid` executor 与 stats 接入;forward fallback、deferred lighting、内建 PBR 共同引用 grid include,保留全量 light-buffer 作为 capability fallback。 | `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` 通过;Naga validator 解析并验证 fallback mesh、deferred lighting、builtin PBR 与 `zr_gpu_scene.wgsl`/`zr_light_grid.wgsl`;后续 focused `light_grid` builder tests 已通过,早期宽过滤测试曾因 Windows lib-test 编译/codegen 超时未返回。 | Broader graph/pipeline validation 仍需处理 `ssao-evaluate` async compute workload 等无关图声明漂移后补跑。 |
| 2026-06-21 | LS-M1/LS-M2 many point light captured parity | 已完成 | `render_product_many_point_lights_forward_deferred_capture_parity` 在真实 WGPU 产品路径中提交 Forward+ 与 Deferred 基线/64 点光场景,验证 light-grid executor、64 点灯光 stats、dense cluster peak 与中心区域亮度贡献;同轮复跑既有 many-point/CSM/multi-spot 产品合同。 | `rustfmt --edition 2021 zircon_runtime\src\graphics\tests\render_product_shadows.rs` 通过;预热 target-dir `cargo test -p zircon_runtime --lib render_product_many_point_lights_forward_deferred_capture_parity --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1;直接二进制 exact 复跑 `render_product_many_point_lights`、`render_product_csm_directional`、`render_product_multi_spot_shadows` 均通过。冷 target-dir Cargo wrapper 15 分钟超时未产出二进制,不计为通过。 | 百灯成本/局部密度统计由下一行补齐;contact shadow WGPU 捕获由 LS-M4 行补齐;仍需更宽 render-product sweep 与 RenderDoc 证据。 |
| 2026-06-21 | LS-M2 hundred-light local-density stats product guard | 已完成 | `render_product_hundred_point_lights_report_local_density_stats` 覆盖 128 点 dense/spread 等总数场景,同时断言 CPU light-grid 与真实 WGPU Forward+ 产品 stats 的 peak/average cluster 负载差异,防止回退到按总灯数解释着色压力。 | `rustfmt --edition 2021 zircon_runtime\src\graphics\tests\render_product_shadows.rs` 通过;`cargo test -p zircon_runtime --lib render_product_hundred_point_lights_report_local_density_stats --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1(仓库既有 warnings);直接二进制 exact 复跑该用例与 `render_product_many_point_lights_forward_deferred_capture_parity` 均通过;`rustfmt --edition 2021 --check zircon_runtime\src\graphics\tests\render_product_shadows.rs` 通过。 | 仍需更宽 render-product sweep、RenderDoc 证据与 locked/wider checks。 |
| 2026-06-21 | LS-M3 directional shadow-atlas WGPU visible darkening guard | 已完成 | `render_product_directional_shadow_atlas_capture_records_receiver_path` 保留真实 WGPU Forward+ receiver/caster 路径级守卫;新增 `render_product_directional_shadow_atlas_darkens_receiver_capture` 对同色 receive-shadow 开关做截图对拍,证明 directional atlas 采样使 receiver 区域可见变暗;`ShadowAtlasResources` 比较采样合同改为 `GreaterEqual`,匹配 `map_depth >= receiver_depth` 的深度可见性语义。 | `rustfmt --edition 2021 zircon_runtime\src\graphics\scene\scene_renderer\shadow\atlas\resources.rs zircon_runtime\src\graphics\tests\render_product_shadow_captures.rs` 通过;`cargo test -p zircon_runtime --lib render_shadow_atlas_compare_function_matches_forward_depth_contract --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --quiet -- --test-threads=1 --nocapture` 通过 1/1(仓库既有 warnings);最新 lib-test 二进制 direct exact 复跑 `render_product_directional_shadow_atlas_capture_records_receiver_path` 与 `render_product_directional_shadow_atlas_darkens_receiver_capture` 均通过;`rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\scene_renderer\shadow\atlas\resources.rs zircon_runtime\src\graphics\tests\render_product_shadow_captures.rs zircon_runtime\src\graphics\tests\mod.rs` 通过。 | 仍需 RenderDoc 证据与更宽 product/locked checks。 |
| 2026-06-21 | LS-M3 multi-spot shadow-atlas WGPU guard source/check | 已完成 | `render_product_multi_spot_shadow_atlas_darkens_receivers_capture` 构造 3 spot/3 caster/1 receiver 场景,对 receive-shadow 开关做全帧暗化像素与 luma/RGB delta 守卫;`RenderShadowExecutionReport` 新增 `shadowed_light_count`,由 submit stats 按 directional/point/spot 的 shadow-casting light 总数填充,避免多 spot 场景继续只报告 direction-ready 数。 | `rustfmt --edition 2021 --check zircon_runtime\src\core\framework\render\shadow.rs zircon_runtime\src\graphics\runtime\render_framework\submit_frame_extract\update_stats\base_stats.rs zircon_runtime\src\graphics\tests\render_product_shadow_captures.rs zircon_runtime\src\graphics\tests\mod.rs` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` 通过(仓库既有 warnings);同步 `virtual_geometry_debug_snapshot_contract.rs` 的 4 个直接 `RenderMeshSnapshot` 夹具后,`cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` 通过(仓库既有 warnings);长窗口 `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` 通过并生成 `zircon_runtime-c339c28ec98a5de7.exe`(18m11s);直接二进制过滤 `render_product_multi_spot_shadow_atlas_darkens_receivers_capture --nocapture --test-threads=1` 通过 1/1(8.60s)。 | 仍需 RenderDoc 证据与更宽 product/locked checks。 |
| 2026-06-22 | LS-M3 mixed shadow-atlas wider WGPU product guard | 已完成 | 新增 `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture`,真实 Forward+ 场景同帧提交 1 directional + 3 spot shadow-casting lights、宽 receiver 与多 caster groups,对 receive-shadow 开关做全帧和左/中/右区域暗化守卫,并断言 `shadow.atlas`、`lighting.light-grid`、ready counts、`shadowed_light_count == 4` 和 caster draw count。 | `cargo test -p zircon_runtime --lib render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture --locked --jobs 1 --target-dir target\codex-shadow-wide-0622 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1(首次默认特性 lib-test 构建 28m46s,测试执行 5.21s,仓库既有 warnings)。 | contact-shadow 宽场景由 2026-06-22 LS-M4 行补齐;仍需 RenderDoc 证据与 root wider checks。 |
| 2026-06-21 | LS-M3 CSM subtexel camera-shift WGPU guard | 已完成 | `render_product_csm_directional_remains_stable_under_subtexel_camera_shift` 复用 directional receiver/caster 产品场景,分别在 baseline 与 x=0.006 subtexel camera shift 下提交 shadowed/unshadowed 对拍,用同相机 unshadowed 基线抵消普通投影位移,再比较暗化像素数和 luma delta,防止 CSM snapping 失效造成阴影 swimming。 | `rustfmt --edition 2021 zircon_runtime\src\graphics\tests\render_product_shadow_captures.rs` 通过;`cargo test -p zircon_runtime --lib render_product_csm_directional_remains_stable_under_subtexel_camera_shift --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1(编译/运行 6m34s,测试执行 3.50s,仓库既有 warnings)。 | 仍需 RenderDoc 证据与更宽 product/locked checks。 |
| 2026-06-21 | LS-M3 forward/deferred shadow parity WGPU guard | 已完成 | 新增 `render_product_directional_shadow_atlas_forward_deferred_darkening_parity`,同一 directional receiver/caster 场景分别跑 Forward+ 与 Deferred 的 shadowed/unshadowed 对拍,并修复 Deferred G-buffer material alpha:低 7 位继续编码 shading model,高位编码 receive-shadow flag,`lighting.deferred` 解码后才决定是否采样 atlas shadow。生产修复前该产品守卫失败为 Deferred 暗化像素 0。 | `cargo test -p zircon_runtime --lib render_product_directional_shadow_atlas_forward_deferred_darkening_parity --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 先失败(Deferred pixels=0)后通过 1/1(编译/运行 6m38s,测试执行 5.92s,仓库既有 warnings);`cargo test -p zircon_runtime --lib deferred_geometry_shader --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 7/7;`cargo test -p zircon_runtime --lib deferred_lighting_shader --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 6/6;`cargo test -p zircon_runtime --lib deferred_material_gbuffer_shaders_encode_and_decode_material_channels --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1。 | 仍需 RenderDoc 证据与更宽 product/locked checks。 |
| 2026-06-21 | LS-M3 multi-spot test-build support unblock | 已完成 | `virtual_geometry_debug_snapshot_contract.rs` 的 4 个直接 `RenderMeshSnapshot` 夹具同步 `stable_instance_key`、`transform_revision`、`mesh_lod`、`static_state` 到当前 DTO 合同,解除 Plan 05 multi-spot guard 的 `--tests` 构建阻塞。 | `rustfmt --edition 2021 --check zircon_runtime\tests\virtual_geometry_debug_snapshot_contract.rs zircon_runtime\src\core\framework\render\shadow.rs zircon_runtime\src\graphics\runtime\render_framework\submit_frame_extract\update_stats\base_stats.rs zircon_runtime\src\graphics\tests\render_product_shadow_captures.rs zircon_runtime\src\graphics\tests\mod.rs` 通过;`cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` 通过(仓库既有 warnings);随后同一 target-dir 的 lib-test `--no-run` 与 multi-spot 直接二进制执行均通过。 | 已解除本轮测试构建门;继续 RenderDoc 与更宽检查。 |
| 2026-06-21 | LS-M4 PCF quality WGPU receiver-edge guard | 已完成 | 新增 `render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture`,在真实 WGPU spot receiver/caster 场景中提交 Low、High 与 unshadowed baseline 三帧,并用 darkened pixels、luma delta 与 RGB delta 证明 PCF 档位影响 receiver 边缘产物。`zr_shadow.wgsl` 的 High 9-tap kernel 半径改为 8 texel,三处 shader source 测试新增 Medium/High 半径常量合同。 | `cargo test -p zircon_runtime --lib render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1(编译/运行 7m12s,测试执行 4.48s,仓库既有 warnings);`cargo test -p zircon_runtime --lib shadow_atlas_resources --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 3/3(仓库既有 warnings)。 | contact shadow WGPU 捕获由下一行补齐;仍需 RenderDoc 证据、更宽 product sweep 与 locked/wider checks。 |
| 2026-06-21 | LS-M4 contact shadow WGPU product capture | 已完成 | 新增 `contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region`,同一 Forward+ receiver/blocker 场景跑 plugin-enabled 与 baseline pipeline,断言 `rendering.contact_shadow` feature、`contact-shadow` pass、`lighting.contact-shadow` executor、compute dispatch、coverage 零缺口,并用最终帧暗化统计证明接触阴影进入产品输出。`RenderPassGpuExecutionContext::require_texture_view(...)` 为插件 executor 暴露 pass-declared texture view 解析入口。 | `cargo test -p zircon_plugin_rendering_contact_shadow_runtime contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region --locked --jobs 1 --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1;`cargo test -p zircon_plugin_rendering_contact_shadow_runtime --lib --locked --jobs 1 --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 7/7。 | 仍需 RenderDoc 证据、更宽 contact-shadow 场景与 root wider checks。 |
| 2026-06-22 | LS-M4 contact shadow wider WGPU product guard | 已完成 | 新增 `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions`,用 192x128 Forward+ 宽 receiver 与左/中/右三组 blocker 对比 plugin-enabled/baseline pipeline,断言 feature/pass/executor/dispatch/coverage,并要求全帧和三个接触窗口均暗化且 open receiver 区域不吞掉主要统计。按 `engine-code-structure-convention.md` 与 `engine-code-review-findings-2026-06.md` 复核,本切片只扩展测试层,未新增生产 String error、dead-code suppression、FFI 或 builder/API 债,测试文件 531 行低于拆分阈值。 | `cargo test -p zircon_plugin_rendering_contact_shadow_runtime contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions --locked --jobs 1 --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1(首次该 target-dir 编译 27m42s,测试执行 8.92s,仓库既有 warnings);`cargo test -p zircon_plugin_rendering_contact_shadow_runtime --lib --locked --jobs 1 --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 8/8(测试执行 13.58s)。 | 仍需 RenderDoc 证据与 root wider locked checks。 |
| 2026-06-15 | LS-M3 CSM and shadow atlas hard cut | 部分完成: atlas/receiver/source contracts 已落地,2026-06-21 已补 directional/multi-spot/CSM/forward-deferred shadow parity 真实 WGPU 捕获 | `shadow/atlas`、`cascade`、`slot`、`shadow_settings`、`shadow/plan` 接入;`SHADOW_ATLAS` external graph resource 写入并替代旧 `SHADOW_MAP`;forward/deferred/builtin PBR 只通过 atlas slot 采样;directional cascade/point face/spot view key 从计划 04 shadow views 消费;shadow-only caster 与 product shadow source contracts 已加保护。 | core-min check 覆盖 atlas/bridge/view-filter/receiver/source contracts;`render_shadow_` 过滤测试通过 27 个,`visibility_context_builds_shadow_views_for_atlas_light_slots` 通过 1 个,`shadow_atlas` 命名过滤曾通过 16 个,直接 lib-test 二进制通过 4 个 `render_product_shadows` 合同测试;2026-06-21 补齐 directional/multi-spot receiver darkening、CSM subtexel camera-shift 与 forward/deferred shadow parity 真实 WGPU 捕获。 | caster expansion/receiver slice 风险与 RenderDoc 抓帧验收仍待后续。 |
| 2026-06-15 | LS-M4 PCF quality and contact shadow feature | 已完成(核心合同与真实 WGPU 捕获;RenderDoc 待后续) | `zr_shadow.wgsl`、shadow settings/slot/plan 写入 Low/Medium/High PCF quality flags;contact shadow 作为 rendering plugin feature 接入,默认关闭时 compiled graph 无 pass,启用后 async compute executor 读 depth/normal/HZB 并写 `CONTACT_SHADOW_OCCLUSION`,executor contract 与 shader binding 有源码测试;2026-06-21 已补 PCF 与 contact shadow 真实 WGPU 产品捕获,2026-06-22 已补更宽多接触区域产品守卫。 | PCF core-min `cargo check` 通过并报告既有 warnings;slot flags、shader source、contact-shadow descriptor/executor/shader contract 有源码覆盖;PCF spot receiver-edge 捕获、三条 shader source 半径常量测试、contact shadow exact 过滤测试与插件 runtime 全量 lib 测试均已通过。 | 仍需 root 更宽 locked checks 与 RenderDoc 证据。 |

### 参考实现精读笔记

- `ForwardLights.cs::CreateShadowCullingDataAsync`(zbin/tile 主流程):`zBinScale/zBinOffset` 公式(透视 log2、正交线性)、`wordsPerTile = (itemsPerTile + 31) / 32`、tile 边长自 8 起 `actualTileWidth <<= 1` 直至满足 `maxTileWords`、`LightMinMaxZJob → ZBinningJob` 与 `TilingJob → TileRangeExpansionJob` 双链并行。Zircon 对应:`build_light_grid` 单函数串行先行(灯数百级无需 Jobs),公式与 word 布局原样移植;探针项暂不进 grid,header word1 保留。
- `Tiling/ZBinningJob.cs`:`EncodeHeader(min,max) = min&0xFFFF | max<<16`、空 header `(0xFFFF, 0)`、`FillZBins` 对每灯 `[minBin, maxBin]` 区间 OR 位掩码并收紧 header。Zircon 原样移植为 `zr_light_zbin_header` 的逆操作;取舍:不做 batch 切分(无 Jobs 并行需求)。
- `AdditionalLightsShadowAtlasLayout.cs`:`ShadowResolutionRequest{visibleLightIndex, perLightShadowSliceIndex, requestedResolution, offsetX/Y, allocatedResolution}`、按分辨率排序后 `EstimateScaleFactorNeededToFitAllShadowsInAtlas` 整体降档、过小请求整灯剔除(按 `GetPunctualLightShadowSlicesCount` 回退)、`m_VisibleLightIndexToSortedShadowResolutionRequestsFirstSliceIndex` 反查表。Zircon 对应:`ShadowSlotRequest`/`allocate_frame`;取舍:URP 每帧全量重排,Zircon 加跨帧滞回(URP 无此机制,是其阴影闪烁来源之一)。
- `ShadowUtils.cs::GetShadowBias`:bias 以 shadow texel 世界尺寸为单位缩放(`texelSize = frustumSize / shadowResolution`),透视投影下随深度变化;`ApplySliceTransform` 将 slice 矩阵乘 atlas scale/bias。Zircon 对应:`LightShadowSettings.depth_bias/normal_bias` 取同语义,`ZrShadowSlot.atlas_scale_bias` 即 slice transform。
- `LightGridInjection.cpp`:`GetLightGridZParams`(`DepthDistributionScale = 4.05` 的非线性 z 切片)、`RWCulledLightLinks` 链表 + 紧凑化为 `CulledLightDataGrid`(16bit 元素)、`NumCulledLightsGridStride = 2`。取舍:UE 的 3D froxel + GPU 链表压缩需要两道 compute 与原子分配,Zircon 取 URP zbin+tile(2D tile x 1D zbin 近似 3D),无压缩 pass、storage buffer 一次写成,wgpu 友好;`PackRG16/PackRGB10` 的灯数据压缩暂不采用(96 B 直存,数百灯无带宽压力)。
- `ShadowSetup.cpp` + `DirectionalLightComponent.cpp`:`ComputeAccumulatedScale(Exponent, CascadeIndex, CascadeCount)` 指数级联分布(`GetEffectiveCascadeDistributionExponent` 无烘焙时固定 4)、`GetSplitDistance` 由累计比例插值 near/far、`CascadeSettings.SplitNearFadeRegion/SplitFarFadeRegion/FadePlaneOffset/FadePlaneLength` 过渡带、snapping 以 `FMath::Fmod` 在 light view 空间求 `SnapX/SnapY` 后回变换。Zircon 对应:`compute_cascade_splits`(取 log/linear lambda 混合式替代指数式)、`snapped_cascade_view_proj` 同构 snapping;fade 取单带简化(仅 far 端),UE 双带不引入。
- `VirtualShadowMapArray.cpp`:仅记录差距 —— VSM 为页表化 16k 虚拟分辨率 + 按需物理页分配 + 静态/动态缓存双层,依赖 GPU 页表 compute 生态;Zircon atlas 方案与其无共享 ABI,远期若引入按插件走,不在本计划范围。

## 风险与回退

- grid 构建 compute 在低端设备成本高:保留 LS-M1 的全量循环作为 capability 回落档(同一 buffer ABI,只是不分簇),不维护双 shader 语义。
- atlas 槽位抖动(灯光优先级频繁互换导致阴影闪烁):分配器加滞回(槽位保留若干帧);单测覆盖。
- VSM 不在本计划范围:仅在 LS-M3 文档中记录与 `VirtualShadowMapArray.cpp` 的差距,避免过度设计。

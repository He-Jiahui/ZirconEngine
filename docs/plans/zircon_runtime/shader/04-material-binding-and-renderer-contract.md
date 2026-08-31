# 计划 SH-04:材质绑定与渲染器契约

## 目标

1. `.zmaterial` v2:对 `surface` shader 的绑定契约定稿——属性覆盖强校验、option 选择、贴图绑定、queue 偏移覆写、可选 `parent` 材质继承链。
2. 参数三层覆盖模型:shader 默认值 → 材质资产 → 运行时 per-renderer 覆盖块(MaterialPropertyBlock 等价物),各层职权与失效语义定稿。
3. renderer 侧变体组装接线:options → `ShaderVariantKey.material_option_bits` 新维度、shading_model/queue/disabled_passes 从 shader 声明解析、预热清单扩展 option 组合——全部作为计划 08 的登记式增维。
4. `compute` / `fullscreen` 的消费契约:按名字绑定资源的 dispatch/pass builder,经 render graph 节点接入,不进材质变体空间。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-shader-material-renderer-contract",
  "goal": "完成材质绑定、运行时覆盖、渲染器变体和全局 compute/fullscreen 契约",
  "milestones": [
    {"id": "M1", "title": "zmaterial v2 与三层覆盖", "depends_on": []},
    {"id": "M2", "title": "变体增维与 renderer 接线", "depends_on": ["M1"]},
    {"id": "M3", "title": "compute/fullscreen 契约与样例", "depends_on": []}
  ]
}
```

## 现状与差距

- `ZMaterialDocument`(`asset/assets/material/zmaterial.rs`)只有 overrides/textures,无 option 选择、无 queue 覆写、无继承;overrides 校验依赖 `accepts_value` 的宽松检查。
- 自定义材质的 shading model、pass 参与目前无声明来源(standard material 走 `lighting_model` 字段,自定义 shader 缺位)。
- 无 per-renderer 覆盖:动态改一个实例的颜色需要克隆整个材质。
- compute 无资产级派发契约:引擎内 compute(HZB、particles)全部手写 pipeline 与 bind group。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Engine/Public/Materials/Material.h`(及 MaterialInstance 族) | Material → MaterialInstanceConstant → MaterialInstanceDynamic 的参数继承与覆盖链;StaticSwitch(改变体)与 Scalar/Vector(不改变体)的二分——`parent` 链与"option 改变体、属性不改"的职权划分样板 |
| `dev/Graphics/.../universal/Runtime/Passes/DrawObjectsPass.cs` | FilteringSettings(renderQueueRange)+ ShaderTagId 匹配 + RenderStateBlock——renderer 按 queue/pass 选择绘制集合的样板 |
| Unity `MaterialPropertyBlock` 用法(`dev/Graphics/.../Runtime/MipGen/MipGenerator.cs`、`Decal/Entities/DecalEntityManager.cs`) | per-instance 覆盖不改批处理布局、不可含 keyword——运行时覆盖块的边界样板 |
| `dev/bevy/crates/bevy_material/src/key.rs` + `bevy_pbr/src/material.rs` | 材质数据进 pipeline key 的类型擦除形态、DirtySpecializations 脏跟踪——option bits 进变体键与失效的 Rust 落地形态 |
| `dev/Fyrox/fyrox-material/src/shader/mod.rs` | material 引用 shader + 按名覆盖属性值的资产形态(现 zmaterial 同构,v2 增量部分对照) |
| 本仓库 `render/08-material-shader-permutation.md` 工程落地细化 | `ShaderVariantKey`/帧时序/prewarm——本计划全部增维的登记目标 |

## 目标架构

### zmaterial v2

```toml
version = 2
name = "Hero Cloth Red"
shader = "shaders/hero_cloth.zshader"
parent = ""                          # 可选: 另一 .zmaterial,单链继承,shader 必须同源

[options]                            # 改变体: 等价 UE StaticSwitch / Unity shader_feature keyword
detail_layer = true

[overrides]                          # 不改变体: 按 MaterialPropertyLayout 强校验类型
base_color = [0.8, 0.1, 0.1, 1.0]
fuzz = 0.5

[textures]
base_color = "textures/cloth_albedo.ztex"

[queue]                              # 可选: 覆写 shader 声明,±100 段内偏移(render index §8 第 4 条)
offset = 10
```

- 校验升级:overrides/textures/options 逐项对照 shader artifact 的 `MaterialPropertyLayout`/`MaterialOptionTable` 校验(名字存在、类型匹配、enum 值合法),未知键报诊断——替换 `accepts_value` 宽松路径。
- `parent` 链(UE MIC 语义):子材质未声明的键沿链继承;链上 shader 引用必须一致;链深与环由 importer 限制。选项与属性都可被子层覆盖;解析在 import/加载期折叠为扁平有效值集,运行时不留链式查找。
- shading_model / disabled_passes / render_state 不允许材质覆写(它们是 shader 的身份,材质只选值与开关;需要不同 render state 即写不同 shader 或用 option)。

### 参数三层覆盖模型

| 层 | 载体 | 可改 | 变体影响 | 失效粒度 |
|----|------|------|---------|---------|
| L0 shader 默认 | `.zshader` properties/options default | 属性默认值、option 默认 | — | shader revision(全材质) |
| L1 材质资产 | `.zmaterial`(含 parent 链折叠) | 属性值、贴图、option 选择、queue 偏移 | option 改变体键;属性/贴图只改 group2 内容 | material revision(该材质) |
| L2 运行时覆盖块 | `MaterialPropertyOverrideBlock`(per-renderer 组件数据,经 scene extract 进 `RenderFrameExtract`) | **仅属性标量/向量值** | 禁止改 option/贴图/queue——不触变体、不触 bind group | 当帧上传(uniform 动态段或 per-draw 上传,批处理键不变) |

L2 是 Unity MaterialPropertyBlock 的等价物:名字 → 布局表槽位写入,渲染侧对携带覆盖块的 draw 以材质 uniform 副本(帧环形分配)替换 binding0,layout 不变故 pipeline/bind group layout 均复用。运行时改 option 的合法路径是显式 API 克隆材质(等价 UE 的 MID 不能改 StaticSwitch)。

### renderer 契约(surface)

接入计划 08 帧时序 Prepare 中段的 key 组装(登记式增维,均需镜像进 08 工程落地细化):

1. `ShaderVariantKey` 增字段 `material_option_bits: u32`(L1 折叠后的 option 位包);`canonical_string` 增 layout_hash 与 option bits 参与项;`packed_dims` 不变(材质维度不打包,维持 08 约定)。
2. shading model 解析:custom surface shader 由 zshader `shading_model` token → `ShadingModelId`(替代 standard-only 的 `lighting_model` 路径,standard material 内建 shader 走同一声明);token 未注册 → 诊断并回落 fallback 材质。
3. pass 过滤:draw build 按 shader `disabled_passes` 跳过对应 mesh pass 的命令生成(Fyrox disabled_passes 语义);Shadow 被禁用时不进 shadow pass 队列而非绑空管线。
4. queue:shader `queue` 声明 + 材质偏移 → `RenderQueueValue`,进计划 09 排序键的既有 queue 位段。
5. 预热:`zircon_shader_prewarm` 清单从 zmaterial 收集(shader × 实际 option 组合)× 适用几何源 × 启用 pass——`shader_feature_local` 稀疏语义,不做 option 全组合枚举。

### compute / fullscreen 消费契约

```rust
// core/framework/render/shader/compute_dispatch.rs(契约层)
pub struct ComputeKernelRef { pub shader: AssetReference, pub kernel: String }
// graphics 层 builder:按名字设参与绑资源,布局由 artifact 生成
let mut d = ComputeDispatchBuilder::new(kernel_ref);
d.set_f32("delta_time", dt);
d.bind_storage("particles", particle_buffer);
d.dispatch_groups(ctx, [n, 1, 1]);   // 经 render graph compute 节点录制
```

- compute ABI:group0 binding0 = 生成 params uniform(SH-02 打包器),binding1.. = `resources` 声明序分配;compute pipeline cache 键 = (shader, kernel, options, content_hash),独立于材质变体缓存但复用同一磁盘缓存设施(键前缀区分)。
- 所有派发必须经 render graph 节点(render index §6 第 3 条:不允许旁路提交);引擎内建 compute(HZB/particles 等)分批迁移到该契约是后续各 owner 计划的事,本计划交付契约与首个迁移样例。
- fullscreen ABI:group0 = frame/view(既有 scene uniform),group1 = pass 输入(`resources` 声明的源纹理/采样器,由 graph 边供给),group2 = 生成 params;vs 由全屏三角模板提供;供 postprocess/renderer feature 消费(计划 07 的 pass 实现可渐进迁移)。

## 里程碑

### SH04-M1 zmaterial v2 与三层覆盖

实施切片:
1. zmaterial v2 解析(options/queue/parent)与布局表强校验;parent 链折叠;`accepts_value` 宽松路径删除。
2. `MaterialPropertyOverrideBlock` 组件与 extract → 渲染侧 per-draw uniform 副本路径;L2 禁改项校验。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime material --locked`
- 验收证据:校验诊断矩阵(未知键/类型错/enum 非法/parent 环);L2 覆盖渲染正确且 pipeline/bind group layout 指针级复用断言。

### SH04-M2 变体增维与 renderer 接线

实施切片:
1. `ShaderVariantKey.material_option_bits` + canonical_string 增项(同步登记计划 08);draw build 组装接线;shading_model token 解析统一;disabled_passes 过滤。
2. prewarm 清单扩展 option 组合;缺失 option 组合进 `VariantMissReport`。

测试阶段:
- `cargo test -p zircon_runtime shader --locked`(键稳定性含 option 维度)+ `render_product` 三模型/双 option 场景对拍
- 验收证据:同材质不同 option 产出不同变体且正确;禁用 Shadow 的材质不产 shadow 命令;预热后 option 场景零 miss。

### SH04-M3 compute/fullscreen 契约与样例

实施切片:
1. `ComputeDispatchBuilder`/compute pipeline cache/graph 节点接入;一个内建 compute(选 HZB 或 particle 一处)迁移为样例并删除其手写绑定路径。
2. fullscreen 模板与 builder;一个后处理 pass 样例迁移。

测试阶段:
- `cargo test -p zircon_runtime compute --locked` + 对应 `render_product`/`render_perf` 回归
- 验收证据:样例迁移前后产物一致;按名绑定错误(缺资源/类型错)有帧诊断而非 wgpu 崩溃。

## 测试与验收清单

- `render_material_v2_validation_*`:zmaterial 校验矩阵与 parent 折叠。
- `render_material_override_block_*`:L2 三禁(option/贴图/queue)与 layout 复用。
- `render_shader_variant_option_bits_*`:键稳定性、prewarm 稀疏枚举、miss 报告。
- `render_shader_pass_filter_*`:disabled_passes 命令生成过滤。
- `render_compute_dispatch_builder_*`:按名绑定、kernel 选择、graph 节点约束。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 日期 | 里程碑 | 状态 | 完成项目 | 验证与证据 | 后续 |
|---|---|---|---|---|---|
| 2026-07-04 | SH04 / Plan 11 PBR environment sampling | PBR 环境反射产品矩阵已验证 | `zr_environment.wgsl` 已作为共享 include 进入 material pass support includes;fallback mesh、standard PBR forward 与 deferred lighting 都调用 `zr_environment_pbr_indirect(...)`;8x8 `.zmaterial` 临时项目覆盖 metallic 0..1 与 smoothness 0..1,并在 procedural skybox 环境中导出截图。 | `cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::deferred::lighting_pipeline --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1` 13/13 通过；`cargo test -p zircon_runtime --lib scene_renderer::mesh::mesh_pipeline::fallback_mesh_shader_source --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1` 14/14 通过；`cargo test -p zircon_runtime --lib graphics::shader::template --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1` 21/21 通过；ignored export `graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_metallic_smoothness_matrix_png` 通过 1/1,输出 `docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png`,SHA256 `E883A3BDF657025EAD16A7F39B1F8BE5D7FFCDA1FDEF0243A8636A05C217030D`。 | 本行只关闭 procedural skybox 环境下的 PBR 反射矩阵;真实 cubemap IBL prefilter、probe blending、lightmaps/probes、fog、RenderDoc/product capture 与更广 product/perf sweep 仍待 Plan 11 后续里程碑。 |
| 2026-07-04 | SH04 / Plan 11 real HDRI sampled reflection | 真实 HDRI 环境反射矩阵已验证 | `SkyboxMode::SampledEquirectangular` 与 `SampledEquirectangularEnvironment` 让 material pass 共用真实 HDRI 下采样环境；`SceneUniform` 新增 `environment_sample_params`/`environment_samples`,`zr_environment.wgsl` 与 `skybox_procedural.wgsl` 采样同一 16x8 equirectangular table；ignored export 使用 Poly Haven `lakes` HDRI 验证 metallic/smoothness PBR 反射真实场景条带。 | Poly Haven HDR `docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr` 本地 SHA256 `FAF3ECE79216E568A29F0D8FC176A795C66EB9C312C3CF3EE18D9AC04A71DECB`;`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-real-hdri-reflection-0704 --message-format short --color never` 通过；direct binary `export_runtime_shader_pbr_real_hdri_reflection_png` 通过 1/1,输出 `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png`,1280x960,132232 bytes,SHA256 `958E3B200EC56BCA16BF9596B1F05D872179F51CEB9A64925E10FC2D41792DEE`;同名 target/cargo-target 扫描 0。 | 本行只关闭真实 HDRI sampled environment 验证；不标记 cubemap 资产导入、GPU `texture_cube`、GGX prefilter mip 链、reflection probe capture/blending、SH irradiance bake、RenderDoc/product capture 或更广 product/perf sweep 完成。 |
| 2026-07-03 | SH04-M1 zmaterial v2 与三层覆盖 | L1/L2 主路径完成 | `.zmaterial` 已硬切换 `version = 2`，默认解析拒绝非 v2；`ZMaterialDocument`/`MaterialAsset` 支持 `parent`、`options`、`queue`，仓库内 tracked `.zmaterial` 均迁移到 v2；材质加载折叠 parent 链，限制深度并诊断缺失、循环、shader mismatch；属性、贴图、option、queue offset 对照 shader `MaterialPropertyLayout`/`MaterialOptionTable` 强校验；新增 `MaterialPropertyOverrideBlock`，`MeshRenderer` 可携带运行时属性覆盖，scene extract 写入 `GeometryExtract.material_property_overrides`，ResourceStreamer 合成覆盖后的 material uniform payload，draw 构建为覆盖 draw 创建 per-draw binding0 uniform，静态批处理与静态命令缓存排除覆盖 draw。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` 于 2026-07-03 通过；direct generated lib-test binary `E:\cargo-targets\zircon-runtime-shader-sh02-m2\debug\deps\zircon_runtime-90029178d239f47b.exe` 通过 `material_property_uniform_payload_applies_runtime_override_block`、`material_property_uniform_payload_reports_invalid_override_block_entries`、`geometry_extract_excludes_material_override_entities_from_static_batches` 共 3/3；`git grep -n "version = 1" -- "*.zmaterial"` 复扫无匹配。 | L2 当前关闭 CPU payload、extract、per-draw uniform 与静态批/缓存排除；仍需更广 render_product 双材质覆盖画面对拍和 perf 计数。 |
| 2026-07-03 | SH04-M2 变体增维与 renderer 接线 | renderer 与 asset-root prewarm 主路径完成；material-pass 二次启动 miss=0 聚焦验证已补跑 | material option bits 已由 `MaterialOptionTable` 计算并进入 `ShaderVariantKey`/`PipelineKey`；canonical string 包含 layout_hash；shader `shading_model` token 接到 runtime material shading model；shader/material queue 进入 render queue；`disabled_passes` 已传入 draw/batch/cache key，并在 depth/shadow/velocity/TAA/base/transparent processor 中过滤；所有 material pass 模板源码现在通过 ResourceStreamer 获得 generated material WGSL、模块 include 与 option define；材质 bind group layout/创建、标准材质模板、fallback mesh WGSL、builtin PBR WGSL 与管线测试布局已硬切到 group2 binding0 uniform + binding1..10 标准贴图/采样器；`zircon_shader_prewarm` 的 asset-root manifest 现在从 `.zshader` 生成 material layout hash/option table，并按 `.zmaterial` 实际 option 值稀疏写入 `material_option_bits`，重复材质组合由 canonical key 去重，相关 prewarm fixture 已迁移到 zshader/zmaterial v2。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` 通过；`cargo check -p zircon_runtime --bin zircon_shader_prewarm --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` 通过；direct generated bin-test binary `E:\cargo-targets\zircon-runtime-shader-sh02-m2\debug\deps\zircon_shader_prewarm-1eec6d2f27a12a1f.exe` 通过 `manifest::tests::shader_prewarm_asset_root_manifest_uses_sparse_material_option_keys`、`manifest::tests::shader_prewarm_asset_root_manifest_reads_compound_zshader_package`、`manifest::tests::shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source` 共 3/3，随后 `manifest::tests` 21/21 通过；`cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest_uses_sparse_material_option_keys ... -- --exact` 因短名 exact 过滤为 0 tests，不计通过。vampire 验证图 `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png` 为 1280x720、54403 bytes、SHA256 `1526BE245965025596FA6098C495D85DCCBDA90E295C1B80489F4649740B5CE0`，同名 target/cargo-targets 扫描无匹配。direct generated lib-test binary `E:\cargo-targets\zircon-runtime-03-gates-0704\debug\deps\zircon_runtime-33095b46939b64fc.exe` 精确运行 `graphics::tests::render_product_mesh_cache::project_plugin_registry_material_passes_staged_cache::second_launch::render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss` 通过 1/1、6400 filtered、10.59s，验证 first/second launch staged cache 只读、material pass request 无 compile miss。`renderer_material_layout_diagnostics` focused test 在 Windows lib-test harness 编译阶段 300s 超时，未计通过。 | `VariantMissReport` 已有 material-pass 二次启动 miss=0 聚焦产品验证；端到端双 option/disabled pass/render queue 命令生成断言、更广 product/perf sweep 与 RenderDoc/product capture 仍待补；compute/fullscreen builder 属 SH04-M3。 |
| 2026-07-04 | SH04-M2 prewarm 组合 WGPU module+pipeline 验证 | public wrapper、runtime lib 过滤与 owner budget 已关闭 | `zircon_shader_prewarm` 同时启用 `--validate-wgpu-modules` 与 `--validate-wgpu-pipelines` 时改走组合 validation API，先 module 后 pipeline，任一 module 失败会跳过对应 pipeline 并记录 skipped；公共 dynamic API 复用一个 offscreen backend/device，report 同时开启 module 与 pipeline 两个 summary。`variant_cache/prewarm.rs` 的组合成功/失败测试下沉到 `variant_cache/prewarm/tests/combined_validation_tests.rs`，父文件维持 672 行；`manifest/tests.rs` 的 raw source revision 测试下沉到 `manifest/tests/raw_revision.rs`，父测试文件维持 721 行；结构守卫同步读取 `manifest/material_sources.rs` 与 raw revision child，避免再把状态或预算锚点写回父 route。 | `cargo test -p zircon_runtime --lib --no-run --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-combined-validation-0704 --message-format short --color never` 通过；直接运行 `E:\cargo-targets\zircon-shader-combined-validation-0704\debug\deps\zircon_runtime-6bef7a696c15c9a5.exe shader --nocapture` 通过 366/366、1 ignored；`python -m unittest tools.tests.test_zircon_build_shader_prewarm_wrapper_orchestration.ZirconBuildShaderPrewarmWrapperOrchestrationTests.test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu_pipelines` 通过，report 为 requested/written/failed 18/18/0，WGPU module validation 18/18，WGPU pipeline validation 18/18；prewarm bin `target-server` rebuild 通过。 | RenderDoc/product capture、workspace/full CI 与更广 product/perf sweep 仍不是本行完成项；本行只关闭 public wrapper 组合 WGPU validation 与结构预算同步。 |
| 2026-07-03 | SH04-M3 compute/fullscreen 契约与样例 | 契约与首个 descriptor 样例完成；GPU executor 全量迁移与真实 pipeline cache 未关闭 | 新增 `core/framework/render/shader/compute_dispatch.rs` 与 `fullscreen_pass.rs`，锁定 compute ABI group0 binding0 params、binding1.. resources，以及 fullscreen ABI group0 frame、group1 pass inputs、group2 params；`RenderGraphComputeWorkload::from_shader_dispatch(...)` 让 graph compute workload 可消费 `ComputeDispatchPlan`；`RenderFeaturePassDescriptor` 新增 compute/fullscreen plan 消费入口；`clustered_lighting` descriptor 改为消费 `ComputeDispatchBuilder` 计划，`motion-vector-tile-max` 后处理 pass 改为消费 `FullscreenPassBuilder` 计划；`builtin://shader/pbr.wgsl` 即使被 ResourceStreamer 记录为 Surface，fallback key 仍强制走 standard material template，避免 Base/非 Base pipeline 误走 generated material path；bridge async-compute fallback stats 改为由 compiled graph workload 派生期望。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` 通过；direct generated lib-test binary `zircon_runtime-90029178d239f47b.exe compute --nocapture --test-threads=1` 通过 36/36；`... fullscreen --nocapture --test-threads=1` 通过 3/3；`cargo test -p zircon_runtime --lib graphics::tests::render_framework_bridge::pipeline_profiles::headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never -- --exact --nocapture --test-threads=1` 通过 1/1；direct binary 通过 `runtime_base_mesh_pipeline_keeps_builtin_fallback_on_standard_template_after_shader_stream` 1/1 与 `runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss` 1/1；ignored vampire screenshot test 通过 1/1，验证图 `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png` 为 1280x720、54403 bytes、SHA256 `1526BE245965025596FA6098C495D85DCCBDA90E295C1B80489F4649740B5CE0`，同名 target/cargo-targets 扫描无匹配。 | 仍需把 HZB/particles/更多 postprocess executor 从手写绑定逐步迁到该契约；compute pipeline 目前完成中立 cache key，不等于真实 GPU pipeline disk cache；更广 product/perf sweep、RenderDoc/更广产品截图仍未关闭。 |
| 2026-08-26 | SH04-M2 / 09C P0-4 material generation publication | mesh 范围源码实施完成，受管验证与统一 PipelineService 仍进行中 | `PreparedMaterialBundle` 作为不可拆 draw-facing generation；cold/hot candidate 均先 staged，按当前 scene geometry 与当前 graph 的 Base/GBuffer/Depth/Shadow/Velocity/TAA/OIT exact requirements 全就绪后一次发布。当前帧保持完整旧 bundle；无 last-good 时成对使用 default pipeline、fallback bindings/textures 与洋红 error proxy。terminal failure 保留 last-good/error proxy 与候选诊断；完整候选身份抑制同一失败的逐帧两次 uniform 重建。静态 command cache 的 material identity 同时覆盖 draw generation、最终 `PipelineKey`、六项纹理及两项 uniform 资源身份。 | 仅完成限定 Rust `rustfmt`、源码契约与 scoped diff integrity；两次托管 Windows 尝试均在 Cargo 前停止（acquire reconciliation timeout；既有 D/E/F unmanaged artifacts），未运行 Cargo/WGPU、热重载产品矩阵、PNG、RenderDoc、timing、RSS 或功耗。可由源码证明的稳态新增 census scan 为 0；候选期为 `O(P + D + R)`，不是实测性能结论。 | M2 历史主路径不回退，但本行不关闭 09C P0-4/M6。仍需 cold/hot invalid/recovery 300 帧、跨 pass 不混代、命令缓存精确失效、统一 PSO authority 与 submission-ticket retirement 验收。 |

2026-08-26 camera-boundary addendum: material PSO admission now aggregates observed/deferred state over the complete viewport camera sequence and publishes only at the viewport-terminal camera. A candidate with no referencing draw remains staged but leaves the active census until later material preparation reactivates it. Stable added work is O(1) and candidate work is `O(P + D + R)` per camera with two bools per candidate; these are source complexity bounds, not measurements. Multi-viewport unseen graph/quality/fog/geometry contexts still require context-qualified last-good selection, so 09C P0-4/M6 remain open and no milestone is upgraded.

2026-07-04 final recheck note: after docs sync, touched-file rustfmt, the two exact combined WGPU validation tests, the broad runtime `shader` filter, screenshot hash checks, and same-name target scans passed. The wrapper live-WGPU unittest re-run skipped because no `zircon_shader_prewarm.exe` was present and `ZR_TEST_SHADER_PREWARM_EXE` was unset; a follow-up bin rebuild timed out after 244s under active cargo/rustc lanes, so neither is counted as additional pass evidence beyond the earlier counted wrapper/bin runs.

## 风险与回退

- option 位宽 32 上限:enum 位宽按声明序打包,超限 import 诊断;确有超限需求时扩 u64 并 bump canonical schema 版本,不做隐式截断。
- L2 覆盖的上传带宽:每携带覆盖块的 draw 一份 uniform 副本,量大时退化明显——帧环形分配 + 计数进 `render_perf_*` 观测;GPU 侧 per-instance 属性(DOTS Instancing 形态)列为后续能力档。
- parent 链与热重载交互(改 parent 传播子材质):按 revision 沿链传播失效,折叠缓存重建;链深限制(≤4)控制爆炸面。
- compute 契约推广节奏:仅交付契约 + 单样例,避免一次性迁移全部内建 compute 造成大回归面;各 owner 计划按里程碑自行迁移。

## Open Failure Handoffs

- fixed 已修复：[compute-fullscreen-descriptor-compile-regression](../../zircon_editor/editor/02/fixed-2026-07-14-compute-fullscreen-descriptor-compile-regression.md)
- fixed 已修复：[material-abi-layout-expectation-drift](../../zircon_editor/editor/02/fixed-2026-07-14-material-abi-layout-expectation-drift.md)

# Zircon Shader 资产、材质绑定与渲染器契约计划集

本目录是 shader 资产形态、属性代码生成、WGSL 模块化与 material/renderer 绑定契约的权威计划集。渲染管线内部机制(变体键、模板拼接、几何源、shading model、变体缓存与预热)的权威仍是 `docs/plans/zircon_runtime/render/08-material-shader-permutation.md`(下称"计划 08");本计划集只定义**资产面向开发者的契约层**,并在需要扩展计划 08 既定结构(如变体键增维)时显式登记镜像条目,不在此重定义计划 08 已定稿的机制。

## 1. 定位与问题

当前 `.zshader` 是"整段 WGSL + 手写 pipeline_layout"的单一形态(`zircon_runtime/src/asset/assets/shader/zshader.rs`),开发者写一个自定义材质 shader 需要:

1. 手写 group0-3 全部 bind group 声明(与引擎 ABI 逐 binding 对齐,错一个槽位即失败);
2. 手写 WGSL 里的属性 uniform struct,并与 `properties` 声明、Rust 侧 `property_uniform.rs` 三处手工同步;
3. 无法声明变体开关(options),无法按材质裁剪 pass;
4. 除内嵌 `zr_*` include 外,用户资产之间无法互相引用;
5. compute / 纯头文件模块没有资产类型位,只能伪装成"整段 shader"。

目标:开发者只写**一份 `.zshader` 声明 + 一段只含语义代码的 WGSL**,其余(属性 struct、绑定声明、打包布局、Rust 侧上传、编辑器 Inspector、变体枚举)全部由引擎生成与推导。

## 2. Shader 资产类型体系(总览)

`.zshader` v2 增加必填 `kind` 字段,四种类型:

| kind | 语义 | 可绑定材质 | entry point | 参与材质变体空间 | 参照 |
|------|------|-----------|-------------|----------------|------|
| `surface` | 材质表面着色:提供 surface 函数(及可选顶点位移函数),由计划 08 模板按(几何源 × pass × shading model)拼出最终 WGSL | 是(`.zmaterial` 引用) | 无(模板提供) | 是 | Unity Lit/Unlit.shader、UE Material、Fyrox shader 资产 |
| `include` | 纯模块/函数库:只暴露 struct 与函数,供其他 shader 跨模块引用 | 否 | 禁止 | 否(内容哈希进引用者缓存键) | Unity ShaderLibrary/*.hlsl、bevy `#define_import_path` 模块 |
| `compute` | 计算着色:一个文件多个 kernel,经 render graph compute 节点派发 | 否 | `stage = "compute"` 的 kernel 列表 | 否(独立变体空间) | Unity `.compute` + `#pragma kernel`、UE GlobalShader |
| `fullscreen` | 全屏 pass(后处理/合成):只写 fragment,vs 由全屏三角模板提供 | 否 | fragment 一个 | 否(独立变体空间) | UE GlobalShader、URP fullscreen blit |

分型原则(对齐 UE `MaterialShader` / `GlobalShader` 的划分):**只有 `surface` 进材质变体空间**(材质 × 几何源 × pass × shading model × options);`compute` / `fullscreen` 是全局 shader,变体只由自身 options 与引擎 defines 决定,避免排列爆炸。`unlit` / `pbr` 不是资产类型,而是 `surface` shader 的 `shading_model` 声明(对接计划 08 MS-M3 的 `ShadingModelId` 注册体系)。

## 3. 开发者视角的目标工作流

```
my_shader.zshader        ← 声明: kind/shading_model/properties/options/texture_slots/imports/passes
my_shader.wgsl           ← 本体: 只写 zr_material_surface 函数与私有辅助函数
                            开头由拼接器自动注入生成的属性声明模块(不手写任何 binding)
my_material.zmaterial    ← 绑定: shader 引用 + 属性覆盖 + option 选择 + 贴图绑定
```

- 属性可声明打包压缩:标量按 vec4 槽自动合并,布局对所有变体恒定(Unity "不许 ifdef properties" 的 SRP Batcher 纪律)。
- WGSL 内通过生成的访问函数取属性(`zr_mat_base_color()`),打包细节对开发者透明。
- options 生成 `ZR_OPT_*` defines,材质在 Inspector 勾选,变体按"实际被材质使用的组合"稀疏编译(Unity `shader_feature_local` 语义),预热工具从 `.zmaterial` 扫描组合。
- 跨模块引用:`include` 型 shader 声明 `import_path`(默认由资产路径推导,见子计划 05),其他 shader 经 `imports` + WGSL 内 `#include <path>` 引用,与内嵌 `zr_*` include 同一注册表解析。
- IDE 可识别:引擎生成 shader IDE 解析环境(模块 stub 树 + `module_map.json` + 拼接预览);用户 WGSL 可写一行 `#include <self::material>` 锚点让生成符号对 IDE 可见(拼接产物不变);编译诊断行号回映到用户源(子计划 05)。

## 4. 子计划地图与执行顺序

| 子计划 | 内容 | 前置 |
|--------|------|------|
| [01 shader 资产类型与 zshader v2](01-shader-asset-kinds-and-zshader-v2.md) | `kind` 四型、schema v2 字段定稿、importer 校验与硬切换 | 无 |
| [02 属性代码生成与 vec4 打包](02-property-codegen-and-packing.md) | properties/options → 生成 WGSL 属性模块 + `MaterialPropertyLayout` 表;打包算法;Rust 上传侧单一来源 | 01 |
| [03 WGSL 模块 import 与跨引用](03-module-imports-and-cross-references.md) | 资产级 include 注册、依赖图与环检测、内容哈希进变体缓存键 | 01 |
| [04 材质绑定与渲染器契约](04-material-binding-and-renderer-contract.md) | zmaterial v2、三层参数覆盖、options 进变体键、pass 参与声明、compute/fullscreen 的 renderer 侧消费 | 01/02/03 |
| [05 编写 DX 与 IDE 可识别性](05-ide-and-authoring-dx.md) | import_path 路径推导、IDE 解析环境(stub 树 + module_map + 拼接预览)、`self::material` 生成符号锚点、诊断行号回映 | 02/03 |
| [06 环境 IBL 资产链与 PBR 正确性验收](06-environment-ibl-and-pbr-correctness.md) | equirect→cube/GGX 预滤波(FIS)/SH9/BRDF LUT 算法定稿(cmft/UE5 精读)、`zr_environment.wgsl` v2 采样契约、8×8 材质球矩阵量化验收;文件落点/帧时序实施权威仍是 render 计划 11 | 01(资产依赖 render 计划 13 TX-M3) |

执行顺序 01 → (02 ∥ 03) → (04 ∥ 05) → 06(06 内部 EC-M1→M4 顺序执行,与 04/05 无相互依赖)。与计划 08 里程碑的衔接:

- 02 的生成属性模块插进计划 08 的固定拼接顺序(shading include 之后、材质 surface 函数之前),供 08 模板路径消费;
- 04 给计划 08 的 `ShaderVariantKey` 增加 `material_option_bits: u32` 维度与 `canonical_string` 的 layout_hash 参与项——该扩展必须同步登记进计划 08 "工程落地细化"(render index §8 第 7 条:细化章节为实施权威);
- 预热(`zircon_shader_prewarm`)的变体枚举清单从"材质 × 几何源 × pass"扩展为"× option 组合(仅 zmaterial 实际使用)",归 04。

## 4.1 当前执行状态

| 日期 | 子计划 | 状态 | 已完成 | 未完成/待验证 |
|---|---|---|---|---|
| 2026-07-05 | SH06 EC-M1a sampled HDRI mip 链纠偏 | 中间实现,待最终验证 | 16x8 sampled equirect table 改为 128x64 base + 8 mip storage buffer;scene uniform 删除内嵌 `environment_samples`;skybox/PBR helper 改用 bilinear/lod 采样;roughness→mip 对齐 UE reflection capture 公式;真实 HDRI 导出新文件名 `runtime_shader_pbr_real_hdri_lakes_pmrem_reflection_20260705.png`。 | `cargo check`、shader/Naga tests 与 ignored export 仍待补跑;完整 GPU `texture_cube`、GGX FIS prefilter、SH9、BRDF LUT、probe capture 与 Plan 13 `.zcube` 仍按 SH06 EC-M1/EC-M2 后续推进。 |
| 2026-07-05 | SH06 环境 IBL 资产链与 PBR 正确性验收 | 计划建立:源码研究与算法定稿完成,未实施 | cmft/cmftStudio 与 UE5 精读结论、equirect→cube/GGX prefilter(filtered importance sampling)/SH9(立体角权重+预乘 A_l)/BRDF LUT(128×128 RG16F) 算法定稿、`zr_environment.wgsl` v2 契约、离线烘焙三来源持久化(资产伴生→缓存回写→现场 compute)与 IEM 替代 SH 消费选项、8×8 材质球矩阵六条量化判据;定位当前马赛克根源:16×8 sampled equirect table + 最近邻采样 + 两方向 lerp 伪粗糙度 + 无 Cube 纹理维度。 | EC-M1..M4 全部待实施;依赖 render 计划 13 TX-M3 `.zcube` 与计划 11 EL-M1/M2 落点;镜像条目已登记进计划 11/13 状态表。 |
| 2026-07-04 | Plan 11 / SH04 PBR environment reflection matrix | procedural skybox + PBR environment matrix 已聚焦验证 | `EnvironmentExtract` 已进入 render frame/viewport packet;formal procedural skybox 由 `EnvironmentExtract.skybox` 驱动;`zr_environment.wgsl` 进入 fallback mesh、standard PBR 与 deferred lighting,standard PBR 可按 metallic/smoothness 采样 procedural sky environment;ignored export 生成 8x8 metallic 0..1 / smoothness 0..1 材质球矩阵。 | `cargo test -p zircon_runtime --lib graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_metallic_smoothness_matrix_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m4-pbr-matrix-0704 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1` 通过 1/1；截图 `docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png`,1280x960,109556 bytes,SHA256 `E883A3BDF657025EAD16A7F39B1F8BE5D7FFCDA1FDEF0243A8636A05C217030D`;同名 target/cargo-target 扫描 0。cubemap prefilter、reflection probes、lightmaps/probes、fog 与 RenderDoc/product capture 仍待 Plan 11 后续里程碑。 |
| 2026-07-04 | Plan 11 / SH04 real HDRI sampled environment reflection | 真实 HDRI 采样天空与 PBR 反射矩阵已验证 | Poly Haven `lakes` 1K HDRI 已保存到 `docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr`;`SampledEquirectangularEnvironment`/`SkyboxMode::SampledEquirectangular` 通过 scene uniform 上传 16x8 sample table,skybox 与 standard PBR/fallback/deferred 环境 helper 共用真实 HDR-derived 采样；ignored export 生成真实 HDRI 8x8 PBR 反射矩阵。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-real-hdri-reflection-0704 --message-format short --color never` 通过；direct binary `E:\cargo-targets\zircon-real-hdri-reflection-0704-server\debug\deps\zircon_runtime-0a7825d39d44b0c4.exe graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_real_hdri_reflection_png --ignored --exact --nocapture --test-threads=1` 通过 1/1；截图 `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png`,1280x960,132232 bytes,SHA256 `958E3B200EC56BCA16BF9596B1F05D872179F51CEB9A64925E10FC2D41792DEE`;同名 target/cargo-target 扫描 0。完整 cubemap asset import、GPU `texture_cube`、GGX prefilter、probe blending、SH bake 与 RenderDoc/product capture 仍待 Plan 11/13 后续里程碑。 |
| 2026-07-03 | SH01 shader 资产类型与 zshader v2 | 已完成 runtime 硬切换 | `.zshader` v2 kind 分型、surface-only prewarm 过滤、存量 `.zshader` 迁移、v1/pipeline_layout/user shader_defs 拒绝诊断。 | `zircon_plugins` workspace check 仍待最终集成阶段单独跑。 |
| 2026-07-03 | SH02 属性代码生成与 vec4 打包 | 主路径完成，截图验收通过；focused 布局/模板/Naga 验证已补跑通过 | `MaterialPropertyLayout`/`MaterialOptionTable`、generated material WGSL、layout_hash、ResourceStreamer material uniform 上传、模板 generated module 注入、option define 注入；renderer material group2 ABI 已统一为 binding0 uniform + binding1..10 标准贴图/采样器。 | `cargo check` 与 vampire 离屏截图导出通过；本轮 `cargo test -p zircon_runtime --lib property_layout` 4/4 通过，`cargo test -p zircon_runtime --lib render_shader_template` 首跑暴露 status mirror 缺口、修复后 18/18 通过；更宽的 `shader`/`material` filter、产品级 RenderDoc/capture 与 product miss=0 仍待补跑。 |
| 2026-07-03 | SH03 WGSL 模块 import 与跨引用 | M1 完成，M2 redirect、asset-root include 失效传播、插件 manifest 注册、source-only 项目扫描依赖传播、redirect 材质 readiness 与产品侧 ResourceStreamer 诊断已通过聚焦验证 | `include_registry` 硬切换为 `module_registry`；内建/资产模块统一解析、传递依赖、去重、环诊断、WGSL include 扫描/剥离、include asset ResourceStreamer 供给、importer 一致性与词法诊断；focused `shader_module` tests 6/6 通过；`ShaderImportReadiness.source_diagnostic` 现在为 redirect import 行记录 import path -> redirect locator 来源说明；`zircon_shader_prewarm` asset-root 扫描通过 `manifest/module_dependencies.rs` 把同 root 下 include 模块依赖 hash 混入引用者 request 与 material revision，旁观者 shader revision 保持不变；插件 manifest 新增 `[[shader_permutation.shader_modules]]`，工具链会导出 registry `shader_modules`，prewarm overlay 会把外部插件模块 hash 作为 fallback 纳入项目 shader revision；`scan_and_import/shader_import_dependencies.rs` 现在把 source-only 项目 import_path 解析为 include shader 的 `ResourceRecord.dependency_ids`；`MaterialAsset::readiness_report_with_shader_contract(...)` 现在把 shader redirect import 依赖解析失败投射为材质 `UnresolvedShaderReference` 与 shader fallback usage；产品侧 `ResourceStreamer` 已覆盖 resolved redirect include prepare/module source 供给与 missing redirect fallback 诊断。 | redirect 来源诊断 1/1；asset-root include revision exact 1/1，`shader_prewarm_asset_root_manifest` 10/10；插件注册工具链 Python 回归 52/52、py_compile、Rust fmt、prewarm bin check 通过，直接运行生成 test binary 的两个新增 exact tests 各 1/1；source-only import 依赖传播 `cargo check` 与 `shader_import_dependency_contract` 1/1 通过；redirect 材质 readiness 补口 `cargo check` 与 `material_shader_redirect_dependency_contract` 2/2 通过，覆盖内存合成依赖与 `ProjectManager::scan_and_import` imported artifact 链路；产品侧 ResourceStreamer redirect 子测试 direct generated runtime binary 通过 2/2。focused product material-pass 二次启动 miss=0 已由 SH04 用例关闭；产品级 RenderDoc/product capture、更广 product/perf sweep 与 editor project-sync refresh exact Cargo result 仍待完成。 |
| 2026-07-03 | SH04 材质绑定与渲染器契约 | M1/L1/L2、M2 renderer/prewarm 主路径与 M3 compute/fullscreen 契约样例完成；material-pass 二次启动 miss=0 聚焦验证已补跑 | `.zmaterial` v2、parent/options/queue、强校验、material option bits、layout_hash variant key、shading_model/queue/disabled_passes renderer 接线、所有 material pass 模板源码经 ResourceStreamer 注入 generated material/modules/options；标准/备用/内置 WGSL、材质 bind group layout 与 bind group 创建已同步新版 group2 ABI；`MaterialPropertyOverrideBlock` 已接入 scene extract、per-draw material uniform override payload、静态批/命令缓存排除和覆盖 payload 窄测；`zircon_shader_prewarm` 已从 `.zshader` 生成 material layout hash/option table，并按 `.zmaterial` 实际 option 值稀疏生成 variant key；`ComputeDispatchBuilder`/`FullscreenPassBuilder`、graph compute workload bridge、RenderFeaturePassDescriptor 消费入口、clustered lighting compute descriptor 与 motion-vector fullscreen descriptor 样例已接线。 | SH03 focused `shader_module` tests 与 SH05 IDE/stub 聚焦验证已关闭；截图证据 `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png` 已归档；prewarm manifest 21/21、compute filter 36/36、fullscreen filter 3/3 聚焦测试通过；direct generated lib-test binary `E:\cargo-targets\zircon-runtime-03-gates-0704\debug\deps\zircon_runtime-33095b46939b64fc.exe` 精确运行 `render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss` 通过 1/1、6400 filtered、10.59s，覆盖 product material-pass staged cache 二次启动 `compile_miss_count == 0`。HZB/particles/更多 postprocess executor 全量迁移、真实 compute pipeline disk cache、更广 product/perf sweep 与 RenderDoc/product capture 仍需产品级验收。 |
| 2026-07-04 | Plan 08 / SH04 shader prewarm 组合 WGPU 校验 | 组合 module+pipeline WGPU 路径与 owner budget 已聚焦关闭 | `zircon_shader_prewarm` 同时启用 `--validate-wgpu-modules` 与 `--validate-wgpu-pipelines` 时改走组合入口，复用一个 offscreen backend/device，先执行 shader module validation 再执行 render pipeline validation；module validation 失败时 pipeline validation 记 skipped，report 同时保持 `wgpu_module_validation.enabled = true` 与 `wgpu_pipeline_validation.enabled = true`。结构上，`variant_cache/prewarm.rs` 的组合行为测试下沉到 `variant_cache/prewarm/tests/combined_validation_tests.rs`，`manifest/tests.rs` 的 raw source revision 测试下沉到 `manifest/tests/raw_revision.rs`，相关结构守卫同步读取 `manifest/material_sources.rs` 与 raw revision child。 | `cargo test -p zircon_runtime --lib --no-run --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-combined-validation-0704 --message-format short --color never` 通过；直接运行同 target runtime lib-test binary 的 `shader` filter 通过 366/366、1 ignored；public wrapper live WGPU prewarm 回归通过，report 为 requested/written/failed 18/18/0，module validation 18/18，pipeline validation 18/18；prewarm bin rebuild 通过。截图证据仍在 `docs/tests/runtime/shader`，同名 target/cargo-target 扫描无命中。RenderDoc/product capture 与更广 product/perf sweep 仍待后续补跑。 |
| 2026-07-03 | SH05 编写 DX 与 IDE | M1 截图验收、M2 共享生成器/CLI/默认与显式非默认 preview/增量 diff/Naga parse/validate gate/editor refresh hook typecheck/批量 preview matrix、M3 诊断回映完成 | import_path 项目命名空间推导、同名目录折叠、显式冗余 warning、`self::`/`zr_` 保留段校验、扫描级重复 import_path 诊断、`self::material` 锚点字节等价测试；`graphics/shader/ide_env_generation.rs` 成为 CLI/editor 共享生成 owner,`zircon_shader_ide_env/run.rs` 收缩为参数解析/项目扫描/report 薄壳,生成 `module_map.json`、`modules/` stub 树、per-shader `generated/*.material.wgsl`、`preview/<shader>.<variant>.wgsl` 和 `preview/<shader>.<variant>.segments.json`,并改为 content-aware 写入:内容未变不重写、受管目录内过期产物可清理、module 一字节变化只改对应 stub 与 map；写入前所有 stub 通过 Naga WGSL parse,默认与显式非默认 preview 通过完整 Naga validate,report 输出 `naga_parsed_stub_count`/`naga_validated_preview_count`,并新增 validation-only builtin/feature/context/material option defines 让跨 stub 依赖和 `ZR_OPT_*` 可解析但不污染写出文件；`--variants` 保留默认 preview 快捷入口,`--variant <pass[:options=bits]>`/`ShaderIdePreviewVariant` 可生成 `gbuffer_options_0x00000001` 等非默认组合并拒绝同名 preview variant；默认 editor refresh 仍只请求 static mesh + Forward pass + option bits 0,所有 preview 都把 defines/include/generated/user/pass-template 段表导出给 IDE/人工回映；`DefaultEditorAssetManager::sync_from_project(...)` 在导入后扫描到 Ready shader 时刷新同一 `.zircon-cache/shader_ide/v1` 产物,editor lib typecheck 已通过；`zr_surface_types.wgsl` 现在统一拥有 `ZrSurfaceInput` alias、`zr_surface_default(...)`、`ZrDeferredGBufferOutput` 与 deferred material flag encode helper,GBuffer template 不再重复声明共享输出结构；`MaterialShaderTemplateAssembly` 记录 source segment 表,Naga parse/validation 诊断可回映到 module id + 局部行号,运行时 Forward/GBuffer/TAA material pass 会把 surface shader 的 `import_path` 作为用户源 module id；材质球 surface shader 通过 compound zshader 导入并离屏渲染到 `docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png`；本轮补入并执行通过 `shader_ide_env_batches_preview_matrix_for_all_surface_shaders` 批量矩阵回归,覆盖两个 surface shader × default/GBuffer options=1/DepthPrepass/Shadow/Velocity/TAA preview 路径与 stale 清理,并修复多 shader generated `self::material` scope 与 TAA material-mask 新 ABI 预览缺口。 | 已复跑通过 `cargo check -p zircon_editor --lib --no-default-features`、`cargo test -p zircon_runtime --lib shader_ide_env` 6/6、`cargo test -p zircon_runtime --lib shader_template_` 20/20、`cargo test -p zircon_runtime --lib shader_module` 6/6,以及本轮非默认 preview 聚焦 tests/CLI parse tests；截图证据 `docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png` 与 `runtime_shader_material_vampire_offscreen_20260703.png` 已复核在 docs 目录且非空。新增 preview matrix 先通过 `cargo check -p zircon_runtime --tests` 编译门,随后 `cargo test -p zircon_runtime --lib shader_ide_env_batches_preview_matrix_for_all_surface_shaders` 在 `F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b` 通过 1/1；`cargo test -p zircon_runtime --lib taa_reactive_mask` 同 target 通过 13/13；product material-pass 二次启动 miss=0 已由 SH04 聚焦用例补跑通过；RenderDoc/product capture 与更广 product/perf sweep 仍需后续补跑；M3 结构预算已把 template surface module tests、mesh ensure pipeline tests 和 IDE generator tests 下沉到 child owner。 |

2026-07-04 final recheck note: after this status sync, direct runtime `shader` filter, touched-file rustfmt, screenshot hash check, and same-name target scan were re-run successfully. The wrapper live-WGPU unittest re-run was skipped because no `zircon_shader_prewarm.exe` was present and `ZR_TEST_SHADER_PREWARM_EXE` was unset; a follow-up bin rebuild attempt timed out after 244s while other cargo/rustc lanes were active, so those two commands are not counted as additional pass evidence beyond the earlier counted wrapper/bin runs in the table.

## 5. 全局约定(各子计划共享)

1. **bind group ABI** 直接引用 render index §8 第 1 条:group0 = frame/view、group1 = pass 输入、group2 = material、group3 = object/instance。`surface` shader 的 group2 声明**全部由生成**,zshader v2 不再允许手写 group0-3 布局;`compute`/`fullscreen` 的参数与资源声明也走生成(详见子计划 01/04)。
2. **布局恒定纪律**:同一 shader 的属性 uniform 布局与 group2 bind group layout 对所有变体恒定;options 只影响代码路径,不改布局;未启用 option 关联的贴图槽绑定引擎默认贴图(white/normal 等),不裁剪 binding。
3. **变体维度归属**:全局维度(pass、quality、引擎 feature bits、几何源、shading model)唯一归计划 08;`.zshader` 只能声明材质局部 options(`shader_feature_local` 语义),不得定义全局 keyword。
4. **生成物单一来源**:属性布局表(`MaterialPropertyLayout`)在 import 期生成一次,WGSL 声明模块、Rust 上传路径、编辑器 Inspector、变体缓存键全部消费同一份 artifact,不允许任何一侧手工重复声明。
5. **命名保留段**:生成符号占用 `zr_`、`ZR_OPT_`、`ZrMaterial*`/`ZrCompute*` 前缀;用户 WGSL 内全局符号撞前缀时 naga 解析后诊断拒绝(沿用计划 08 的 surface 函数改名与前缀检查通道)。
6. **硬切换**:每个子计划的落地切片内迁移调用方并删除被取代路径(手写 pipeline_layout、三处手工同步的属性声明等),不留兼容双轨;`.zshader` v1 文档由 importer 报"需迁移"诊断,不做静默兼容解析。
7. **测试命名**沿用 render index §8 第 6 条(`render_shader_*` 单测、`render_product_*` 对拍);资产/导入侧单测归 asset owner 模块 `#[cfg(test)]`。
8. **参考对照纪律**同 render index §8 第 8 条;各子计划"参考代码"表列出必读文件,动手前先读。

## 6. 与既有文档的关系

- `docs/plans/zircon_runtime/render/08-material-shader-permutation.md`:变体键/模板拼接/缓存预热的实施权威;本计划集是其资产契约层前置与增维来源。
- `docs/plans/zircon_runtime/render/index.md` §8:全局工程约定,本计划集第 5 节全部约定以其为基。
- `docs/plans/engine-code-structure-convention.md`:模块归属与结构规范;本计划集文件落点遵循"契约进 `core/framework/render`、实现进 `graphics`/`asset`"。
- `docs/assets-and-rendering/render-framework-architecture.md` 等 docs 模块文档:各子计划落地后按源路径镜像规则更新。

## 7. 全局验收基线

1. 开发者新建一个自定义 PBR surface shader(带 8 个属性、2 个 options、3 张贴图)所需手写内容 ≤ 一份 zshader 声明 + 一个 surface 函数;不出现任何手写 binding/layout/上传代码。
2. 同一 shader 的所有变体共享 group2 bind group layout(布局恒定纪律可断言)。
3. options 组合只按 `.zmaterial` 实际使用稀疏编译;预热后二次启动 miss 计数为 0(对齐计划 08 MS-M4 验收)。
4. `include`/`compute`/`fullscreen` 三型资产各有端到端用例:跨模块引用的 surface shader、graph 节点派发的 compute、后处理 fullscreen pass。
5. 属性布局表变更(增删属性、改打包)能被 `material_revision`/layout_hash 捕获,旧磁盘缓存不误命中。
6. IDE 环境生成后:`#include` 的每个模块与生成属性符号在 stub 树中有物理对应且可跳转;拼接/编译错误报(用户文件或模块 id, 局部行号),不出现拼接后行号。

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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Shader 总索引只保留计划入口与当前执行概述；逐条产出记录改由对应子计划目录维护。

- 迁入记录：[`06/2026-07-09-index-output-records.md`](06/2026-07-09-index-output-records.md)

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

测试调度遵循 [`milestone-validation-policy.md`](../../milestone-validation-policy.md)：资产 schema、代码生成、import、renderer 消费与 IDE 产物各自按里程碑聚合验证；只有跨这些边界的波次才运行完整 shader/renderer 回归。

1. 开发者新建一个自定义 PBR surface shader(带 8 个属性、2 个 options、3 张贴图)所需手写内容 ≤ 一份 zshader 声明 + 一个 surface 函数;不出现任何手写 binding/layout/上传代码。
2. 同一 shader 的所有变体共享 group2 bind group layout(布局恒定纪律可断言)。
3. options 组合只按 `.zmaterial` 实际使用稀疏编译;预热后二次启动 miss 计数为 0(对齐计划 08 MS-M4 验收)。
4. `include`/`compute`/`fullscreen` 三型资产各有端到端用例:跨模块引用的 surface shader、graph 节点派发的 compute、后处理 fullscreen pass。
5. 属性布局表变更(增删属性、改打包)能被 `material_revision`/layout_hash 捕获,旧磁盘缓存不误命中。
6. IDE 环境生成后:`#include` 的每个模块与生成属性符号在 stub 树中有物理对应且可跳转;拼接/编译错误报(用户文件或模块 id, 局部行号),不出现拼接后行号。

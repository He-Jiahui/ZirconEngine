# 计划 SH-02:属性代码生成与 vec4 打包

## 目标

1. 从 `.zshader` 的 properties/options/texture_slots 在 import 期生成三件套:**WGSL 属性声明模块**(struct + 访问函数 + 贴图/采样器声明)、**`MaterialPropertyLayout` 布局表**(每属性的槽位/偏移/标量类,含 layout_hash)、**option 位表**。
2. vec4 打包压缩:标量与向量按确定性算法合并进 vec4 槽,布局对所有变体恒定;开发者经语义访问函数取值,打包对其透明。
3. Rust 上传侧(`property_uniform.rs` 路径)与编辑器 Inspector 改为消费同一份布局表 artifact,删除三处手工同步;compute/fullscreen 的 params 生成复用同一打包器。
4. 生成模块由计划 08 拼接器自动前置注入,WGSL 本体零手写绑定。

## 现状与差距

- group2 material uniform 的布局散在三处:zshader 手写 pipeline_layout、WGSL 手写 struct、`core/framework/render/material/property_uniform.rs` 的 Rust 侧映射;任何一处改动需手工同步。
- 无打包:每个属性直觉上各占一段,浪费 uniform 空间且无 `_st`(纹理 transform)自动位。
- options 概念缺失:`shader_defs` 是静态注入,材质不可选择,不进变体键。
- 编辑器 Inspector 依赖 properties 的自由 editor 字段,无类型化 min/max/枚举值来源。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/Graphics/.../universal/Shaders/LitInput.hlsl` | `CBUFFER_START(UnityPerMaterial)` 恒定布局纪律("不许 ifdef properties")、`_BaseMap_ST`/`_TexelSize` 自动辅助属性——生成模块的输出形态样板 |
| `dev/Graphics/Packages/com.unity.shadergraph/Editor/Generation/Processors/PropertyCollector.cs` + `Generator.cs` | 属性收集 → HLSLProperty 中间表示 → 声明代码生成的三段架构——本计划生成器的结构样板 |
| `dev/UnrealEngine/.../RenderCore/Public/ShaderParameterMacros.h`、`ShaderParameterStruct.h` | `FShaderParametersMetadata`:名称/偏移/类型元数据一次生成、上传与校验多处消费——`MaterialPropertyLayout` 的语义样板(本计划用数据驱动而非宏,因资产在运行期加载) |
| `dev/bevy/crates/bevy_render/macros/src/as_bind_group.rs` | 字段 → BindGroupLayoutEntry 自动生成、绑定槽冲突检查(`BindingState`)——Rust 侧生成 bind group layout 的落地形态 |
| `dev/Graphics/.../Shaders/Lit.shader` pragma 段 | `shader_feature_local`(稀疏材质变体)与 `multi_compile`(全局)的二分——options 语义定位 |
| 本仓库 `core/framework/render/material/property_uniform.rs`、`graphics/scene/resources/gpu_material_uniform/` | 被替换的手工映射路径与上传时序 |

## 目标架构

归属:布局契约类型在 `core/framework/render/shader/`;打包算法与 WGSL 生成在 `asset/assets/shader/`(import 期执行,产物进 artifact);上传侧在 `graphics/scene/resources/gpu_material_uniform/` 消费 artifact;拼接注入点在 `graphics/shader/template/assemble.rs`。

### 属性 kind 定稿与打包算法

属性 kind 枚举定稿(替代自由字符串):`float | vec2 | vec3 | vec4 | color`(f32 类)、`int | uint | bool`(u32 类,bool 上传为 0/1)。贴图槽声明 `st = true` 时自动追加一个 vec4 `_st` 属性(scale.xy + offset.zw,Unity `_ST` 先例)。

打包算法(确定性,双侧共享同一实现):

1. 按文档声明序遍历,f32 类与 u32 类分别装箱(WGSL struct 成员为 `vec4<f32>` / `vec4<u32>` 槽数组段,f32 段在前)。
2. 槽内 first-fit:`vec4/color` 独占一槽;`vec3` 占 `.xyz`,其 `.w` 可被**声明序在其后**的首个同类标量回填;`vec2` 对齐 `.xy` 或 `.zw` 半槽;标量依序填空分量。
3. 回填只向后看不重排:同一声明序永远产出同一布局(布局稳定性 = 声明序 + 算法版本号,二者共同参与 layout_hash)。
4. 产物 `packed_size` 16 字节对齐;空文档生成 4 字节占位(保持 group2 binding0 恒存在,bind group 结构稳定)。

### 生成 WGSL 模块(示例)

对 SH-01 的 Hero Cloth 示例,生成模块(拼接期注入,开发者不可见亦不可改):

```wgsl
// -- zr generated: material properties (layout_hash = 0x84f3...) --
struct ZrMaterialProperties {
    s0: vec4<f32>,   // base_color.rgba
    s1: vec4<f32>,   // fuzz | detail_st? -- 注: 打包结果由布局表描述,注释由生成器输出
}
@group(2) @binding(0) var<uniform> zr_material: ZrMaterialProperties;
@group(2) @binding(1) var zr_tex_base_color: texture_2d<f32>;
@group(2) @binding(2) var zr_smp_base_color: sampler;
@group(2) @binding(3) var zr_tex_detail: texture_2d<f32>;
@group(2) @binding(4) var zr_smp_detail: sampler;

fn zr_mat_base_color() -> vec4<f32> { return zr_material.s0; }
fn zr_mat_fuzz() -> f32 { return zr_material.s1.x; }
fn zr_sample_base_color(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(zr_tex_base_color, zr_smp_base_color, uv);
}
fn zr_sample_detail(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(zr_tex_detail, zr_smp_detail, uv);
}
```

- 访问函数是唯一稳定 API:打包槽位变化不破坏用户代码;`_st` 属性生成 `zr_uv_<slot>(uv)` 变换函数。
- 贴图/采样器 binding 自 1 起按声明序分配;option 关联槽**常驻声明**(index 全局约定 2),未启用时 Rust 侧绑默认贴图。
- options 以值型 define 注入编译请求(`ZR_OPT_DETAIL_LAYER = 1`、enum 为 `ZR_OPT_DETAIL_MODE = 2`),复用既有 `RenderShaderDefinitionValue` 通道;用户代码用现有条件编译语法按 define 分支。
- 注入位置:计划 08 固定拼接顺序中"材质 surface 函数"细分为"**生成属性模块 → imports 展开(SH-03)→ surface 函数**",此细分登记进计划 08 工程落地细化。
- 生成模块同时登记为局部模块 id `self::material` 并物化为 IDE stub 文件:用户可选写 `#include <self::material>` 作 IDE 锚点,拼接器按 id 去重、产物不变(锚点/stub/诊断行号回映的机制归 SH-05)。

### 布局表契约

```rust
// core/framework/render/shader/material_property_layout.rs(契约层,可序列化,无 wgpu)
pub enum PropertyScalarClass { F32, U32 }
pub struct MaterialPropertySlotRef {
    pub name: String,
    pub kind: MaterialPropertyKind,      // 定稿枚举
    pub scalar_class: PropertyScalarClass,
    pub slot: u16,                       // vec4 槽索引(类内)
    pub component: u8,                   // 起始分量 0..=3
    pub component_count: u8,             // 1..=4
}
pub struct MaterialPropertyLayout {
    pub properties: Vec<MaterialPropertySlotRef>,
    pub f32_slot_count: u16,
    pub u32_slot_count: u16,
    pub packed_size: u32,                // 字节,16 对齐
    pub texture_bindings: Vec<MaterialTextureBindingRef>, // name → (binding, option 关联)
    pub layout_hash: u64,                // blake3 截断:声明序 + kinds + 算法版本
}
pub struct MaterialOptionTable {
    pub options: Vec<MaterialOptionRef>, // name/kind/bit_offset/bit_width/enum values/default
    pub total_bits: u8,                  // ≤ 32,超限 import 诊断
}
```

`MaterialPropertyLayout` + `MaterialOptionTable` + 生成 WGSL 源一并存入 `ShaderAsset` artifact;`layout_hash` 参与变体 `canonical_string`(经 SH-04 登记进计划 08)。

### Rust 上传侧单一来源

- `gpu_material_uniform` 重构:按 `MaterialPropertyLayout` 写 CPU 暂存(f32 段 + u32 段),属性名 → 槽位查表;删除 `property_uniform.rs` 中与 standard material 字段一一手写的映射(standard material 的 PBR 属性表由内建 surface shader 的同一条 zshader 声明产出——内建材质与用户材质走同一代码路径,这是"减少重复工作"的关键闭环)。
- group2 bind group layout 由布局表推导(uniform + 贴图/采样器对),`mesh_pipeline_cache` 建管线时从 `ResolvedVariant.layout` 取,不再读手写 pipeline_layout。
- 编辑器 Inspector(zircon_editor 侧)按 `properties`+`editor` 元数据与 `MaterialOptionTable` 自动生成控件;本计划只保证 artifact 中元数据完备,编辑器落地归 editor 计划集。

## 里程碑

### SH02-M1 打包器与布局表

实施切片:
1. `MaterialPropertyKind` 定稿枚举替换自由字符串(含 v2 解析接线);打包算法与 `MaterialPropertyLayout`/`MaterialOptionTable` 生成;layout_hash。
2. WGSL 属性模块文本生成器(struct/绑定/访问函数/`_st`);naga 验证生成产物(空属性、全类型覆盖、64 属性上限用例)。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime property_layout --locked`
- 验收证据:打包确定性(同输入跨进程同 hash)、回填正确性(vec3+float 共槽)、生成 WGSL 全部过 naga、声明序不变时增删无关属性不动既有槽位以外的断言。

### SH02-M2 拼接注入与上传侧切换

实施切片:
1. `assemble.rs` 注入生成模块(细分拼接顺序);options → define 注入编译请求;前缀撞名诊断扩展到 `zr_mat_*`/`zr_tex_*`/`zr_smp_*`。
2. `gpu_material_uniform` 改为布局表驱动;内建 standard material 属性改由内建 surface zshader 声明产出;删除手写映射与 zshader 手写 group2 消费路径。

测试阶段:
- `cargo test -p zircon_runtime shader --locked` + `cargo test -p zircon_runtime material --locked` + `render_product` 回归
- 验收证据:内建 PBR 场景产物不回归;自定义 shader 属性经访问函数取值渲染正确;同 shader 全变体 group2 layout 相等的断言测试。

## 测试与验收清单

- `render_shader_property_packing_*`:装箱/回填/对齐/hash 稳定性。
- `render_shader_generated_module_naga_*`:生成模块编译验证(含 option define 组合)。
- `render_shader_layout_single_source_*`:上传偏移与 WGSL 槽位一致性(布局表为共同断言基准)。
- `render_product_custom_material_properties`:端到端产物对拍。

## 状态与产出记录

| 日期 | 里程碑 | 状态 | 完成项目 | 验证与证据 | 后续 |
|---|---|---|---|---|---|
| 2026-07-03 | SH02-M1 打包器与布局表 | 已完成 runtime/asset 首段，focused 布局/Naga 验证通过 | `core/framework/render/shader/material_property_layout.rs` 定稿 `MaterialPropertyLayout`、`MaterialOptionTable`、属性槽位与 layout_hash；`asset/assets/shader/property_layout.rs` 从 `.zshader` v2 properties/options/texture_slots 生成布局、option 位表与 generated material WGSL；空属性、全类型、纹理绑定、option enum/bool 均进入 artifact。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` 于 2026-07-03 通过；`cargo test -p zircon_runtime --lib graphics::tests::project_render::project_scenes::export_example_vampire_scene_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1` 通过 1/1，并归档 `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png`；`cargo test -p zircon_runtime --lib property_layout --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1` 通过 4/4，覆盖 deterministic hash、vec3+scalar 回填、option bit packing 和 generated module Naga。 | 截图已归档，SHA256 `1526BE245965025596FA6098C495D85DCCBDA90E295C1B80489F4649740B5CE0`；M1 focused `property_layout`/Naga 生成模块验证已关闭。 |
| 2026-07-03 | SH02-M2 拼接注入与上传侧切换 | 已完成主路径接线并通过截图验证，template focused 验证通过 | `assemble.rs`/GBuffer/TAA 模板支持 generated material module、`self::material` 锚点去重与 material option defines；`resource_streamer_ensure_material.rs` 改由 `MaterialPropertyLayout` 组装 uniform payload；mesh material pass 源码构造把 generated material WGSL 传入 Forward/GBuffer/Depth/Shadow/Velocity/TAA；`ShaderVariantKey`/`PipelineKey` 加入 layout_hash，磁盘变体键参与布局变化；renderer material group2 ABI 同步 generated material 的 binding0 uniform。 | 同上 `cargo check` 通过；vampire 离屏截图导出测试通过；`cargo test -p zircon_runtime --lib render_shader_template --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1` 首跑暴露 status mirror 缺口并在补齐后复跑通过 18/18，覆盖 template assembly、surface modules、Naga 标准材质验证与结构 owner anchors。 | 继续补跑更宽的 `shader`、`material` focused tests；截图已写入 `docs/tests/runtime/shader`，同名文件未出现在 target/cargo-targets。 |

## 风险与回退

- uniform 空间上限:group2 uniform 单 buffer,属性槽超预算(>256 vec4)时 import 诊断;bindless/storage 化属于后续能力档,不在本计划双轨。
- 打包算法演进:算法版本号进 layout_hash,升级即全量失效重编(宁可多失效,与计划 08 磁盘缓存口径一致);不做旧算法兼容读取。
- 访问函数内联开销:naga/wgpu 下游会内联平凡函数,不做手工展开;若抓帧证实开销,再评估生成期展开,不改用户 API。

## Code Review 建议 (2026-07-30)

基于对布局契约与上传侧实现的实际阅读。

### 与代码现状不符，需修订

- 「Rust 上传侧单一来源」与 SH02-M2 切片写「删除 `property_uniform.rs`」,但 `zircon_runtime/src/core/framework/render/material/property_uniform.rs` 仍存在,并已重构为布局表驱动的 `RenderMaterialPropertyUniformPayload::from_layout_and_values(...)`(`property_uniform.rs:24-52`),按 `MaterialPropertyLayout.properties` 遍历写偏移,不再逐 standard 字段手写。实现方向与计划一致,但「删除该文件」的措辞已过时。建议把描述改为「`property_uniform.rs` 改为消费 `MaterialPropertyLayout` 的 payload builder,删除逐字段手写映射」,与保留下来的文件对齐。

### 设计优化建议

- `property_uniform.rs:36-40` 写出的 `RenderMaterialPropertyUniformField.alignment` 恒为 4,`size` 为 `component_count*4`。这对 vec3(component_count=3)会报 size=12/alignment=4,与 WGSL `vec4` 槽的 16 字节对齐语义有落差。SH02-M1 的打包器已用 vec4 槽 first-fit,但该 field 元数据把对齐降级为标量粒度。若下游(Inspector/校验)按此 alignment 推断布局会与 WGSL 实际 16 对齐不符,建议在计划里明确该 field 元数据的 alignment 语义(标量偏移粒度 vs vec4 槽对齐),避免消费方误用。

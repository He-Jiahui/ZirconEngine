# 计划 SH-01:shader 资产类型体系与 zshader schema v2

## 目标

1. `.zshader` 增加必填 `kind`(`surface` / `include` / `compute` / `fullscreen`),四型各自的字段合法性由 importer 强校验。
2. `surface` 型定稿材质契约字段:`shading_model`、`render_state` 白名单覆写、`queue`、`disabled_passes`、properties/options/texture_slots。
3. 删除 `surface` 型的手写 `pipeline_layout`(group0-3 全部由引擎 ABI + 生成推导);`compute`/`fullscreen` 的资源声明改为语义化 `resources` 列表(binding 编号由生成分配)。
4. `.zshader` v1 硬切换:importer 对无 `kind` 或含手写 `pipeline_layout` 的 surface 文档报迁移诊断,仓库内既有 fixture/example 同切片迁移。

## 现状与差距

- `ZShaderDocument`(`zircon_runtime/src/asset/assets/shader/zshader.rs:26`)无类型区分:所有文档共享 wgsl_files/entry_points/pipeline_layout 字段,compute 与"材质 shader"无法从资产层面区分,include 型模块无资产位。
- `pipeline_layout` 要求开发者手写 group/binding/visibility(`examples/vampire/assets/shaders/default_pbr/default_pbr.zshader`),与 render index §8 第 1 条的固定 ABI 重复且易错。
- pass 参与不可声明:自定义 shader 无法表达"不投影/不进 GBuffer"(Fyrox `disabled_passes` 的能力缺失)。
- `properties.kind` 为自由字符串,合法集合无枚举定稿;`texture_slots.group` 允许自由指定组,违反 ABI 固定纪律。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/Fyrox/fyrox-material/src/shader/mod.rs` 与 `dev/Fyrox/editor/resources/shaders/gizmo.shader` | 数据驱动 shader 定义:resources/PropertyGroup、`disabled_passes`、pass 级 draw_parameters ——`.zshader` v2 surface 型的直接同类 |
| `dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/Lit.shader` | Properties 块 + pass 集合(LightMode tag)+ render state 声明的组织;`_QueueOffset`/renderQueue 的材质可覆写语义 |
| `dev/Graphics` 中任一 `.compute`(如 `com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeVolumeUploadData.compute`) | kernel 为首级编译单元、kernel 级变体(`multi_compile_local`)——compute 型 entry_points 的语义样板 |
| `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/GlobalShader.h` | 非材质全局 shader 独立于材质变体空间管理——`compute`/`fullscreen` 分型依据 |
| 本仓库 `zircon_runtime/src/asset/assets/shader/zshader.rs`、`asset/importer/ingest/import_shader_package.rs` | 现有解析/导入双层;v2 在此硬切换 |

## 目标架构

归属:schema 与解析在 `zircon_runtime/src/asset/assets/shader/`;kind 契约枚举与 render_state 白名单契约在 `core/framework/render/shader/`;importer 校验在 `asset/importer/ingest/`。

### schema v2 字段矩阵

| 字段 | surface | include | compute | fullscreen |
|------|---------|---------|---------|------------|
| `kind` | 必填 | 必填 | 必填 | 必填 |
| `name` / `version` | ✓ | ✓ | ✓ | ✓ |
| `import_path` | 可选(供他人引用其辅助函数时) | **必填**(模块身份) | 禁止 | 禁止 |
| `wgsl_files` | ✓(surface 函数源) | ✓(模块本体) | ✓ | ✓(fragment 源) |
| `entry_points` | 禁止(模板提供) | 禁止 | 必填 ≥1,全部 `stage="compute"` | 可选(默认 `fs_main`) |
| `imports` | ✓ | ✓(仅可依赖 include) | ✓ | ✓ |
| `shading_model` | 必填(token,对接 `ShadingModelId`) | 禁止 | 禁止 | 禁止 |
| `properties` / `options` | ✓ | 禁止 | ✓(生成 dispatch params) | ✓(生成 pass params) |
| `texture_slots` | ✓ | 禁止 | 禁止(用 `resources`) | 禁止(用 `resources`) |
| `resources` | 禁止(ABI 固定) | 禁止 | ✓(storage/texture/sampler 语义槽) | ✓(pass 输入槽) |
| `render_state` | ✓(白名单覆写) | 禁止 | 禁止 | ✓(blend 白名单) |
| `queue` | ✓(段名 + ±100 偏移,对齐 render index §8 第 4 条) | 禁止 | 禁止 | 禁止 |
| `disabled_passes` | ✓(Forward/GBuffer/DepthPrepass/Shadow/Velocity 白名单) | 禁止 | 禁止 | 禁止 |
| `pipeline_layout` | **删除** | 禁止 | 禁止(由 `resources` 生成) | 禁止(同左) |
| `editor` | ✓ | ✓ | ✓ | ✓ |

`shader_defs` / `shader_def_values` 保留为引擎/工具注入通道,但用户文档内声明视为迁移诊断(用户侧变体一律走 `options`,见 SH-02;静态常量直接写 WGSL `const`)。

### surface 型示例(目标形态)

```toml
version = 2
kind = "surface"
name = "Hero Cloth"
shading_model = "standard_pbr"      # 或 "unlit" / "blinn_phong" / 插件注册 token
queue = { segment = "geometry", offset = 0 }
disabled_passes = []                 # 例: ["Velocity"]
wgsl_files = ["hero_cloth.wgsl"]
imports = ["zircon::noise"]          # include 型资产的 import_path

[render_state]                       # 白名单: cull_mode/depth_write/depth_compare/blend 预设
cull_mode = "back"

[[properties]]
name = "base_color"
kind = "color"
default = [1.0, 1.0, 1.0, 1.0]

[[properties]]
name = "fuzz"
kind = "float"
default = 0.3
editor = { min = "0", max = "1" }

[[options]]
name = "detail_layer"
kind = "bool"
default = false

[[texture_slots]]
name = "base_color"
kind = "texture_2d"
default = "white"

[[texture_slots]]
name = "detail"
kind = "texture_2d"
default = "white"
option = "detail_layer"              # 仅该 option 启用时被采样;绑定槽常驻
```

对应 `hero_cloth.wgsl` 只含:

```wgsl
// 生成的属性模块(SH-02)由拼接器自动前置注入,此处零手写声明。
fn zr_material_surface(in: ZrSurfaceInput) -> ZrSurfaceOutput { ... }
// 可选: fn zr_material_displace(pos: vec3<f32>, in: ZrVertexInput) -> vec3<f32>
```

`render_state` 是**白名单覆写**而非自由管线描述:仅 `cull_mode`、`depth_write`、`depth_compare`、`blend`(预设枚举 `opaque/alpha/additive/premultiplied`)四项,双面/alpha 语义与计划 08 feature bits(DOUBLE_SIDED/ALPHA_TEST)的换算由 importer 归一,避免与变体维度双轨。

### compute 型示例

```toml
version = 2
kind = "compute"
name = "Particle Sim"
wgsl_files = ["particle_sim.wgsl"]

[[entry_points]]
name = "simulate"
stage = "compute"

[[entry_points]]
name = "emit"
stage = "compute"

[[properties]]                       # 生成 ZrComputeParams uniform(SH-02 同一打包器)
name = "delta_time"
kind = "float"

[[resources]]                        # binding 编号由生成分配,开发者只用名字
name = "particles"
kind = "storage_buffer"
access = "read_write"

[[resources]]
name = "spawn_requests"
kind = "storage_buffer"
access = "read"
```

workgroup size 由 WGSL `@workgroup_size` 本体声明(Unity `[numthreads]` 同位);kernel 即 entry point,Rust 侧按 `(shader, kernel_name)` 派发(消费契约归 SH-04)。

### kind 契约与解析形态

```rust
// core/framework/render/shader/shader_kind.rs(契约层,无 wgpu)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderAssetKind { Surface, Include, Compute, Fullscreen }

// asset/assets/shader/zshader.rs(v2 解析:tagged 文档 → kind 专属结构)
pub enum ZShaderDocumentV2 {
    Surface(ZSurfaceShaderDocument),
    Include(ZIncludeShaderDocument),
    Compute(ZComputeShaderDocument),
    Fullscreen(ZFullscreenShaderDocument),
}
```

解析用 `kind` 字段手动分发(先读 `kind` 再按型反序列化,型内 `deny_unknown_fields`),字段矩阵违规在反序列化层即报错并带字段名,不留运行期才发现的自由字段。

## 里程碑

### SH01-M1 schema v2 与 kind 分型解析

实施切片:
1. `ShaderAssetKind` 契约 + `ZShaderDocumentV2` 四型结构与 TOML 分发解析;字段矩阵校验(违规字段/缺失必填的诊断含字段名与文档 URI)。
2. `render_state` 白名单类型与 queue 段声明类型(复用既有 `RenderQueueValue` 段常量);`resources` 语义槽声明类型。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime zshader --locked`
- 验收证据:四型文档 roundtrip(parse → serialize → parse 等值)单测;12 类字段矩阵违规各有诊断单测。

### SH01-M2 importer 硬切换与存量迁移

实施切片:
1. `import_shader_package.rs` 按 kind 分流构建 `ShaderAsset`(artifact 带 kind 与型专属数据);v1 文档(无 kind / 带 pipeline_layout 的材质文档)报"需迁移"诊断并拒绝导入。
2. 迁移仓库内全部存量 `.zshader`(examples、editor 工程模板、测试 fixture、`zircon_plugins` 内引用),删除 v1 解析路径与 `RenderShaderPipelineLayoutDescriptor` 在 zshader 文档中的暴露;`zircon_shader_prewarm/manifest.rs` 的资产扫描同步按 kind 过滤(仅 surface 进材质变体清单)。

测试阶段:
- `cargo test -p zircon_runtime --lib --locked`(asset 域全量)+ `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- 验收证据:存量资产全部以 v2 导入成功;v1 文档诊断快照测试;prewarm 清单只含 surface 型。

## 测试与验收清单

- `zshader_v2_kind_dispatch_*`:四型解析与字段矩阵拒绝。
- `zshader_v2_surface_render_state_*`:白名单覆写与 feature bits 归一(double_sided/alpha 换算唯一)。
- `zshader_v2_migration_diagnostic_*`:v1 文档报迁移诊断且含 URI。
- `render_shader_prewarm_manifest_kind_filter`:清单按 kind 过滤。

## 状态与产出记录

| 日期 | 里程碑 | 状态 | 完成项目 | 验证与证据 | 后续 |
|---|---|---|---|---|---|
| 2026-07-02 | SH01-M1 schema v2 与 kind 分型解析 | 已完成解析层首段 | 在 `core/framework/render/shader` 增加 `ShaderAssetKind`、render_state、queue、resources 契约子模块；在 `asset/assets/shader/zshader.rs` 增加 `ZShaderDocumentV2` 四型解析、字段矩阵、必填字段和 compute/fullscreen entry stage 校验；顶层 asset/render 导出同步。 | `rustfmt --edition 2021` 覆盖本切片文件；`cargo test -p zircon_runtime --lib zshader_v2 --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh01-m1 --message-format short --color never -- --nocapture --test-threads=1` 通过，3 passed / 0 failed / 5948 filtered，只有既有 warning。 | SH01-M2 尚未完成：importer 硬切换、仓库 `.zshader` v2 迁移、v1 拒绝诊断、prewarm surface-only 过滤仍待实现；截图验收属于后续渲染集成切片。 |
| 2026-07-02 | SH01-M2 importer 硬切换与存量迁移 | 已完成 runtime 硬切换首段 | `ShaderAsset` 持久化 `kind`、`options`、`shading_model`、`render_state`、`queue`、`disabled_passes`、`resources`；compound importer 改为只消费 `ZShaderDocumentV2`，无 `kind`、手写 `pipeline_layout`、用户侧 `shader_defs` / `shader_def_values` 都进入迁移诊断；删除旧 `ZShaderDocument` / `ZShaderDefinition*` 用户文档解析出口；仓库内 examples、editor 模板、fixture `.zshader` 全部迁移到 `version = 2` + `kind = "surface"`；`zircon_shader_prewarm` 扫描按 kind 过滤，仅 surface 进入材质 pass 清单，include/compute/fullscreen 不再投影为材质变体。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` 通过；`cargo check -p zircon_runtime --tests --locked --jobs 1 --message-format short --color never` 通过；`cargo test -p zircon_runtime --lib zshader_v2 --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh01-m2 --message-format short --color never -- --nocapture --test-threads=1` 通过，5 passed / 0 failed / 5952 filtered，只有既有 warning；`git ls-files '*.zshader'` + 静态扫描确认无 `version = 1`、`pipeline_layout`、`shader_defs`、`shader_def_values` 残留；审查守卫 `review_f5_zshader_v2_replaces_user_shader_definitions` 已转绿。 | `cargo test -p zircon_runtime --lib --locked` 与 `zircon_plugins` workspace check 尚未单独声明通过，后续会在 SH02-SH04 渲染集成后合并跑；截图验收仍属于 SH04/渲染方案接受门。 |

## 风险与回退

- 存量迁移面:仓库内 `.zshader` 数量有限(examples + fixtures),一次切片内可完成;若外部项目存量成为阻塞,提供 `zircon_export`/编辑器侧一次性迁移命令(生成 v2 文档草稿),仍不做运行时双轨解析。
- `render_state` 白名单过窄的风险:超出四项的需求(stencil、color mask)按"先进白名单评审再扩枚举"处理,禁止回退到自由 pipeline 描述。

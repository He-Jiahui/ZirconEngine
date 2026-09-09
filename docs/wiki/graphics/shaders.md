---
related_code:
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/graphics/material/shading_models/mod.rs
  - zircon_runtime/src/plugin/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/shader/template
  - zircon_runtime/src/graphics/shader/invocation
  - zircon_runtime/src/graphics/shader/variant_cache
  - zircon_runtime/src/graphics/shader/ide_validation.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/shader/index.md
tests:
  - zircon_runtime/src/graphics/shader/ide_env_generation/tests.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs
  - tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py
doc_type: module-detail
---

# 着色器系统

## 概览

ZirconEngine 当前以 WGSL 为运行时着色语言，围绕它提供四层能力：资产与依赖描述、模板组装与 ABI、变体/管线缓存、IDE 辅助与验证。材质着色器不会把任意字符串直接塞进管线；模板层会组合 engine include、几何源、着色模型、材质 surface 和 Pass 特定片段。

## 资产类型

- `ShaderProgramAsset`：可执行 shader program 及 entry point/依赖/渲染状态。
- `ShaderGraphAsset`：shader graph 的资产表示。
- `MaterialGraphAsset`：材质图资产表示。
- `ShaderVariantKey` / `RenderShaderVariantKey`：由 feature、quality、pass、geometry、material option 等组成的稳定变体身份。
- `PluginShaderModuleSource`：插件提供的命名 WGSL 模块源。

资产导入与运行时编译是两阶段：导入器负责 URI、依赖、格式和 artifact；graphics shader 层负责为具体管线目标组装/验证/缓存。

## Module 与 Include

WGSL 源可以通过 Zircon 的 include token 引用内置或插件模块。`builtin_shader_ide_module_sources()` 返回 IDE 所需的内置 module source 集；插件源必须拥有稳定 token/namespace，且不能覆盖保留的 builtin/generated/self namespace。

模板组装返回 `ShaderAssemblySegment` 列表，使错误能定位到原始片段，而不是只报告拼接后大文件行号。`validate_material_shader_template_wgsl_with_segments` 将 Naga 验证错误映射回 segment。

## 材质模板与几何源

`MaterialShaderTemplateRequest` 指定几何源、材质 source、shader pass 和 feature。内置几何源覆盖 static、morphed、skinned、skinned+morphed mesh；插件可注册 `GeometrySourceDescriptor`，但必须声明 vertex attributes 和 binding requirements。

`ShadingModelDescriptor` 与 `ShadingModelRegistry` 管理 Standard PBR、Unlit、Blinn-Phong 及插件扩展。插件 ID 必须从 `SHADING_MODEL_PLUGIN_ID_START` 开始。着色模型 include 被 token 化并参与 cache identity，修改 source 必须导致变体失效。

## Fullscreen 与 Compute 调用

`FullscreenPassBuilder` 和 `ComputeDispatchBuilder` 是公开的计划构建器，用于声明 shader ref、参数、命名资源 binding 与 dispatch extent。输出为 `FullscreenPassPlan` / `ComputeDispatchPlan`，供 feature lowering 和 Render Graph 使用。

关键 ABI 常量包括 fullscreen frame/pass-input group、params binding，以及 compute params/resource binding 起点。插件不要复制数字；从 `zircon_runtime::graphics` 导入常量，避免引擎 ABI 变化后静默错绑。

## Shader IDE

```rust
use std::path::Path;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::graphics::{
    parse_shader_ide_wgsl_module,
    validate_shader_ide_wgsl_module,
    write_shader_ide_env_for_project,
};

let project = ProjectManager::open("my_project")?;
let _report = write_shader_ide_env_for_project(
    &project,
    Some(Path::new("target/shader-ide")),
    &[],
)?;
let wgsl = "@compute @workgroup_size(1) fn main() {}";
let _parsed = parse_shader_ide_wgsl_module("example.compute", wgsl)?;
let _validation = validate_shader_ide_wgsl_module("example.compute", wgsl)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

IDE env 生成缓存位于约定目录，并写 module map、preview map 和 generated material stub。它辅助补全、跳转和预览，不替代最终目标设备上的 WGPU pipeline validation。

## 变体预热与缓存

变体缓存 key 包含源内容、模板/接口版本、Naga/WGPU 版本、目标和编译维度。预热可分别执行：仅落盘、Naga module validation、WGPU pipeline validation，或二者结合；budget 变体限制工作量并生成 summary。

`WgpuRenderFramework::prewarm_shader_pipelines` 是产品级入口。预热失败通过 `RuntimeShaderPipelinePrewarmReport`/failure 列表报告；调用者应展示诊断，不要把“部分变体写入”当作全部成功。

## 质量与 fallback

`ShaderQualityTier`、`ShaderPipelineTarget` 和 material feature 参与变体选择。运行时缺失变体时可按 `ShaderPipelineFallbackAction/State` 使用明确 fallback 或拒绝；不得静默用不兼容 shader。材质 `fallback_policy` 与 shader pipeline fallback 是相邻但不同的合同。

## 实现状态

- WGSL 模板、内置 include、Shader IDE、Naga validation、磁盘变体缓存和 WGPU pipeline validation 为**默认可用**。
- 插件 module/geometry/shading-model 为**可选可用**，必须注册且满足 ABI。
- Shader Graph / Material Graph 有资产和预览/编译支持，但编辑器完整节点体验不应仅从资产类型存在推断。
- Ray tracing/neural shader slot 为**实验或契约**，取决于后端能力与实际 executor。

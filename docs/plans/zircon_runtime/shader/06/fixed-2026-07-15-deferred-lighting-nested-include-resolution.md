---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: deferred-lighting-nested-include-resolution
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_irradiance_volume.wgsl
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs
tests:
  - F:/cargo-targets/zircon-engine/pool/832e9caf94cdbc5bb2fbeb3ffd49f9e6d203390ac62d5f892b18bcb9fe6d3c30/debug/deps/runtime_shader_pbr_hdri_export-fc1a1989f9d8ecf5.exe pbr_matrix::render_product_environment_pbr_matrix_quantitative --ignored --exact --nocapture --test-threads=1
  - cargo test -p zircon_runtime shader_module_registry --locked
resolved_at: 2026-07-15
---


# Render18: deferred lighting leaves a nested WGSL include unresolved

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：M1 / EC-M1 当前源码真实 HDRI 8x8 PBR 产品门禁
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：失败由 Render18 AF-M2 新增的 irradiance-volume 模块依赖触发，最低共享原因位于 Render18 正在改造的 deferred shader source assembly，而非 Shader06 环境采样或产品测试。

## 失败现象与复现证据

当前源码已成功编译 integration test binary。直接执行精确 ignored 产品测试时实际运行 1 项，并在约 29.74 秒后于创建 `zircon-deferred-lighting-shader` 时失败；Naga 报告最终 WGSL 第 327 行仍包含原始 `#include <zr_irradiance_volume.wgsl>`，因此测试在 GPU 管线创建阶段终止，尚未进入新增的左右掠射对称断言。

复现命令：

```powershell
& 'F:\cargo-targets\zircon-engine\pool\832e9caf94cdbc5bb2fbeb3ffd49f9e6d203390ac62d5f892b18bcb9fe6d3c30\debug\deps\runtime_shader_pbr_hdri_export-fc1a1989f9d8ecf5.exe' 'pbr_matrix::render_product_environment_pbr_matrix_quantitative' --ignored --exact --nocapture --test-threads=1
```

结果：`1 failed; 0 passed`。期望是 deferred lighting 通过统一模块依赖图产出无任何 include directive 的最终 WGSL，然后 Shader06 产品测试继续渲染、readback 与量化验收。

## 最低共享层根因

Render18 已在 `ShaderModuleRegistry` 中把 `zr_lightmap.wgsl -> zr_irradiance_volume.wgsl` 建模为依赖，并在 `ShaderTemplateInclude::new` 中剥离 include directive；但 `assemble_deferred_lighting_shader_source` 仍绕过注册器，手工把原始 `LIGHTMAP_INCLUDE` 拼进最终源码。预先拼接 irradiance-volume 源码不会消费 lightmap 内的文本 include，导致 Naga 收到未预处理语法。该问题与现有 Shader IDE failure 同属依赖图收敛，但发生在不同运行时消费者和不同来源计划，需独立生命周期。

## 架构修复验收

- deferred lighting 组装消费唯一 `ShaderModuleRegistry` 依赖排序，最终提交给 WGPU/Naga 的源码不包含任何有效 `#include` directive。
- focused lower-layer test覆盖 lightmap 的传递依赖只解析一次、顺序为 irradiance-volume 在 lightmap 之前，并验证 deferred 最终源码已剥离 nested include。
- 重跑 Render18 AF-M2 deferred/light-cookie/irradiance-volume focused tests，确认绑定 33..37 和函数可见性不回退。
- 重跑上述 Shader06 精确 8x8 PBR 产品测试，必须实际执行 1 项并通过左右掠射、粗糙度、F0、镜面 SSIM 与截图产物验收。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止仅删除 `zr_lightmap.wgsl` 的依赖声明、在 Shader06 fixture 中跳过 deferred pipeline，或继续维护手工预拼接与 module registry 两份依赖真源。

## 修复结果与回传

- 根因：Deferred lighting manually concatenated raw lightmap WGSL outside ShaderModuleRegistry, leaving the irradiance-volume transitive include unresolved for Naga.
- 架构修复：Render18 routes lightmap transitive dependencies through ShaderModuleRegistry, strips nested include directives, orders irradiance-volume before lightmap, and uses explicit textureSampleLevel for the cookie loop.
- 验证：Render18 managed AF-M2 WGPU job 4108772f5a3b4f0784cfab0925a914fd passed exit 0; Shader06 upward exact ignored PBR job 44dad6ba11c04a6ea61c91054037a4fa passed 1/1 with grazing delta 0.049672 and mirror SSIM 0.998674.
- 回传：Deferred lighting now resolves transitive WGSL modules through the registry and both Render18 WGPU plus Shader06 HDRI/PBR upward gates pass.

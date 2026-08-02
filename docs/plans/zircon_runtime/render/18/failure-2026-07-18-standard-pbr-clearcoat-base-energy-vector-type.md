---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: standard-pbr-clearcoat-base-energy-vector-type
origin_plan: docs/plans/zircon_runtime/render/09-camera-render-ordering.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_runtime/render/09
fixing_child_dir: docs/plans/zircon_runtime/render/18
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  - zircon_runtime/src/graphics/shader/template/tests.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib graphics::tests::render_product_camera_targets::visual_export::export_camera_custom_target_overlay_wgpu_png --locked --jobs 1 -- --ignored --exact --nocapture --test-threads=1
---

# Render18: Standard PBR clearcoat base energy 必须保持 vec3 类型

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/09-camera-render-ordering.md`
- 来源执行切片：CO-M2 camera custom-target overlay WGPU PNG + DX12 RenderDoc evidence gate
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 修复责任切片：AF-M1 advanced PBR material family
- 交接原因：最低共享错误位于 Standard PBR clearcoat shader，不属于 Camera Target、overlay 排序或 Render09 产品证据 scope。

## 失败现象与复现证据

- Render09 managed GPU reservation `5e1363a4f4fb48a5b7f718227346f531`、job `6511cefa089448e2b1f89008eac47f57`、run `b9fba2a659794c71a739d9c09dc41bcb` 在当前源码完成约 48 分钟 Rust 1.94.1 构建后执行 exact ignored exporter 1 项，结果 `0 passed / 1 failed / 8473 filtered`，job terminal/released exit 1，无 live PID。
- `Device::create_shader_module(label = "zircon-mesh-shader")` 被 WGPU/Naga 拒绝：`zr_standard_pbr_gpu_light_lighting` 的 expression 75 类型与 expression 54 的存储类型不匹配；诊断落在 composed shader 对 `zr_pbr_clearcoat_base_energy_scale(surface, world_view)` 的赋值。
- `zr_shading_standard_pbr.wgsl:251` 以 `var direct_base_energy = 1.0` 将变量推断为标量 `f32`，而 `zr_pbr_extras.wgsl:121` 的 clearcoat energy scale 返回 `vec3<f32>`；随后赋值形成确定的标量/向量类型冲突。
- exact exporter 在 shader module 创建阶段终止，`plan09_camera_custom_target_overlay_wgpu_20260718.png` 与 `plan09_camera_custom_target_overlay_dx12_renderdoc_20260718_capture.rdc` 均不存在，不能把该 job 记为视觉或捕获证据。

## 最低共享层根因

Standard PBR direct-light accumulator 的默认 base-energy 表达式仍保留旧标量形态，但 AF-M1 clearcoat 为按 RGB Fresnel 衰减引入了向量返回值。模板静态文本测试确认调用存在，却没有对启用 clearcoat include 后的完整 composed WGSL 做类型验证，因此错误直到真实产品 shader 创建才暴露。

## 架构修复验收

- Render18 AF-M1 owner 让 `direct_base_energy` 的默认值、clearcoat 返回值和下游乘法保持同一 `vec3<f32>` 合同，不改变 Blinn-Phong 跳过 clearcoat、Standard PBR diffuse/specular 能量分配或环境光路径。
- 增加聚焦合同，至少让包含 `zr_pbr_extras.wgsl` 的完整 Standard PBR composed shader 经过 Naga/WGPU 类型验证，并覆盖 clearcoat 关闭与开启路径；不能只做字符串包含断言。
- 在 immutable current source 上通过 shader/template focused gate，并重新执行本记录 frontmatter 中的 Render09 exact product reproduction。
- Render09 owner 获得 fixed return 后重新生成并目检 exact PNG，再以 DX12 RenderDoc 生成可重放 RDC；两个 exact artifact 同时存在才可关闭 CO-M2 visual evidence slice。

## 禁止临时方案

- 不得删除 clearcoat base-energy 衰减、把向量返回值强行截成单通道，或关闭 WGPU validation 来绕过错误。
- 不得弱化 Camera Target 产品断言、复用旧 test binary/PNG/RDC，或把无产物的 exit 1 记为通过。
- 不得由 Render09 Camera 或 HGI RC-S1 会话吸收 Standard PBR shader/test 路径；修复必须由 AF-M1 owner 独立 lease、验证、review、commit 并 fixed return。

## 修复结果与回传

Open state: `current-source shader repair present; managed validation pending`.

- Standard PBR initializes `direct_base_energy` as `vec3<f32>(1.0)`, matching the RGB clearcoat Fresnel scale and preserving the existing downstream energy multiplication.
- The template suite assembles and Naga-validates the complete Standard PBR forward WGSL with clearcoat disabled, clearcoat enabled, and Blinn-Phong selected, so the vector contract is no longer protected only by a text assertion.
- The Render09 managed product exporter, exact PNG inspection, and DX12 RenderDoc capture remain required before this handoff can close; it remains `open`.

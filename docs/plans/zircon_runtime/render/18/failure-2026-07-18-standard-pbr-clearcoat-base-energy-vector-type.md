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
  - zircon_runtime/src/graphics/shader/template/tests/material_template_assembly.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_pbr_specialization.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --locked standard_pbr_clearcoat_base_energy_variants_validate_with_naga -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked standard_pbr_specialization -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked graphics::tests::render_product_camera_targets::visual_export::export_camera_custom_target_overlay_wgpu_png -- --ignored --exact --nocapture --test-threads=1
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

- Render18 AF-M1 owner 让前向阶段的共享 `clearcoat_base_energy` 默认值、clearcoat 返回值和传入 direct-light `direct_base_energy` 的参数保持同一 `vec3<f32>` 合同，不改变 Blinn-Phong 跳过 clearcoat、Standard PBR diffuse/specular 能量分配或环境光路径。
- 增加聚焦合同，至少让包含 `zr_pbr_extras.wgsl` 的完整 Standard PBR composed shader 经过 Naga/WGPU 类型验证，并覆盖 clearcoat 关闭与开启路径；不能只做字符串包含断言。
- 在 immutable current source 上通过 shader/template focused gate，并重新执行本记录 frontmatter 中的 Render09 exact product reproduction。
- Render09 owner 获得 fixed return 后重新生成并目检 exact PNG，再以 DX12 RenderDoc 生成可重放 RDC；两个 exact artifact 同时存在才可关闭 CO-M2 visual evidence slice。

## 禁止临时方案

- 不得删除 clearcoat base-energy 衰减、把向量返回值强行截成单通道，或关闭 WGPU validation 来绕过错误。
- 不得弱化 Camera Target 产品断言、复用旧 test binary/PNG/RDC，或把无产物的 exit 1 记为通过。
- 不得由 Render09 Camera 或 HGI RC-S1 会话吸收 Standard PBR shader/test 路径；修复必须由 AF-M1 owner 独立 lease、验证、review、commit 并 fixed return。

## 修复结果与回传

Open state: `current-source shader repair present; managed validation pending`.

- Standard PBR initializes the forward-owned `clearcoat_base_energy` as `vec3<f32>(1.0)` and passes it as the direct-light `direct_base_energy` argument, matching the RGB clearcoat Fresnel scale and preserving the existing downstream energy multiplication.
- The template suite assembles and Naga-validates the complete Standard PBR forward WGSL with clearcoat disabled, clearcoat enabled, and Blinn-Phong selected, so the vector contract is no longer protected only by a text assertion.
- The Render09 managed product exporter, exact PNG inspection, and DX12 RenderDoc capture remain required before this handoff can close; it remains `open`.

### 2026-09-11 current-source re-admission

- Fixing Session `failure-roll-01a084c8-render18-clearcoat-r1` froze snapshot `3490` at
  `c37155ba304740b3762b20585f77fb53a6da47fb`. The owned forward shader, clearcoat include,
  and template test root have SHA-256 `c1a9ee72c0c8ae50ab72dd8b7c1d6908529c7b3755561868ae4bc9d680d8a03e`,
  `a50064e197658e029889a0c364a7950dc42cd4326bec2b2b4eccada012345b85`, and
  `b268b73aaa6a3dfec4563537985fd7e76aeebeb5c0e0f3ca4f49676d62c9c53d` respectively.
- The owner submitted the focused composed-WGSL Naga regression
  `standard_pbr_clearcoat_base_energy_variants_validate_with_naga` through the Windows
  coordinator. Admission was rejected before a validation ticket, validation copy, Cargo run,
or test execution because external repository `E:\Git\zr_vm` is dirty. This is not a dynamic
pass and does not replace the required Render09 exporter/PNG/RenderDoc evidence.

### 2026-09-19 rolling source-contract recheck

Successor Session `failure-roll-01a084c8-render18-clearcoat-r2` reclaimed the
complete clearcoat shader/template slice after an audited ownership transfer at
baseline epoch `611`. Current source checks pass:

- `zr_shading_standard_pbr.wgsl` initializes `clearcoat_base_energy` as
  `vec3<f32>(1.0)`, applies the normalized RGB clearcoat scale, and forwards
  that vector to direct, environment, transmission, and emission base-layer
  paths;
- `zr_pbr_extras.wgsl` keeps the clearcoat base-energy helpers vector-valued;
- the focused template tests retain the complete composed WGSL Naga variants
  for clearcoat disabled/enabled and Blinn-Phong, plus specialization coverage;
- the local WGSL/source contract probe passes for the current bytes.

Current immutable manifest hashes are:

```text
docs/plans/zircon_runtime/render/18/failure-2026-07-18-standard-pbr-clearcoat-base-energy-vector-type.md
  4e991192037cd598aab2eb1fb71b55afa5d095356183cc3c1df915bfb466fc25
zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  c1a9ee72c0c8ae50ab72dd8b7c1d6908529c7b3755561868ae4bc9d680d8a03e
zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  a50064e197658e029889a0c364a7950dc42cd4326bec2b2b4eccada012345b85
zircon_runtime/src/graphics/shader/template/tests.rs
  47c519014a794722748ad62193701be3a37010088b4934a7566df61b66db8e64
zircon_runtime/src/graphics/shader/template/tests/material_template_assembly.rs
  0bf6f528b060e03feb88060a627b359af654e87bdc7b46fcbe810395524c1bd4
zircon_runtime/src/graphics/shader/template/tests/standard_pbr_specialization.rs
  0030342fc4f0e1d02a5e083b90af1b6eaea2b600e82302b38f8d528cca8a6131
```

The managed Rust/Naga gate and the originating Render09 exact WGPU PNG plus
DX12 RenderDoc replay remain required. External `E:/Git/zr_vm` dirt previously
blocked admission before Cargo; no dynamic or visual pass is claimed here.
This lifecycle remains `open` with no fixed return or closeout.

Successor static ticket `15861a1eb182491ba825afc58671e096` completed on
managed job/run `344f0abc671d46fe8d6bdd050b9106c7` with exit code 0 and marker
`RENDER18_CLEARCOAT_SOURCE_CONTRACT_PASS`; coordinator cleanup completed. The
receipt covers only the current WGSL/template source contract. Managed
Rust/Naga, Render09 WGPU PNG and DX12 RenderDoc gates, independent review,
fixed return and closeout remain pending; no dynamic or visual acceptance is
claimed.

### 2026-09-20 independent review

- Reviewer Session `review-render18-clearcoat-r2` inspected the source-sealed plan,
  failure record, Standard PBR WGSL, clearcoat helpers, template assembly tests, and
  specialization tests. The owned source hashes are `zr_shading_standard_pbr.wgsl`
  `c1a9ee72c0c8ae50ab72dd8b7c1d6908529c7b3755561868ae4bc9d680d8a03e`,
  `zr_pbr_extras.wgsl` `a50064e197658e029889a0c364a7950dc42cd4326bec2b2b4eccada012345b85`,
  `template/tests.rs` `47c519014a794722748ad62193701be3a37010088b4934a7566df61b66db8e64`,
  `material_template_assembly.rs` `0bf6f528b060e03feb88060a627b359af654e87bdc7b46fcbe810395524c1bd4`,
  and `standard_pbr_specialization.rs`
  `0030342fc4f0e1d02a5e083b90af1b6eaea2b600e82302b38f8d528cca8a6131`.
  The reviewer held the failure-document lease during the audit.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. The forward shader initializes
  `clearcoat_base_energy` as `vec3<f32>(1.0)`, assigns the vector-valued normalized clearcoat
  helper, forwards it through `direct_base_energy`, and reuses it for environment, transmission,
  and emission base-layer paths. The focused test assembles and invokes Naga validation for
  clearcoat-disabled, clearcoat-enabled, and Blinn-Phong variants; specialization tests preserve
  the non-clearcoat and KHR base-layer semantics.
- The independent textual vector probe passed and `git diff --check` passed. Scoped Rustfmt for
  the three test owners reports existing import/order and assertion-format drift in those shared
  files; the review did not edit or normalize that foreign/pre-existing drift. No dynamic Naga,
  Cargo, WGPU, PNG, or RenderDoc result is inferred from this receipt. Render09 product evidence,
  managed shader validation, fixed return, and closeout remain pending.

### 2026-09-25 successor current-source reconciliation r3

Fixing Session `failure-roll-01a084c8-render18-clearcoat-r3` reclaimed the
complete six-path clearcoat/template slice with no active lease conflict. The
current source hashes are:

```text
zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  c1a9ee72c0c8ae50ab72dd8b7c1d6908529c7b3755561868ae4bc9d680d8a03e
zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  a50064e197658e029889a0c364a7950dc42cd4326bec2b2b4eccada012345b85
zircon_runtime/src/graphics/shader/template/tests.rs
  47c519014a794722748ad62193701be3a37010088b4934a7566df61b66db8e64
zircon_runtime/src/graphics/shader/template/tests/material_template_assembly.rs
  0bf6f528b060e03feb88060a627b359af654e87bdc7b46fcbe810395524c1bd4
zircon_runtime/src/graphics/shader/template/tests/standard_pbr_specialization.rs
  0030342fc4f0e1d02a5e083b90af1b6eaea2b600e82302b38f8d528cca8a6131
```

The current-source probe passed:
`RENDER18_CLEARCOAT_CURRENT_SOURCE_PASS 5 paths`. It confirms the forward
`vec3<f32>(1.0)` accumulator and vector direct-light contract, vector-valued
clearcoat helpers, complete composed-template Naga variant test anchors for
clearcoat disabled/enabled and Blinn-Phong, and specialization reuse across
environment/transmission/emission paths. Scoped `git diff --check` passed.
Rustfmt remains non-passing only on the existing import/order and assertion
format drift in the three Rust template test owners; the WGSL files require no
Rustfmt and no formatting-only edit was made.

This is source/static evidence only. Managed Rust/Naga Cargo, the originating
Render09 exact WGPU PNG and DX12 RenderDoc replay, independent review, canonical
`failure return`, fixed status, closeout and WeCom remain pending.

### 2026-09-25 independent current-source review r3

Reviewer Session `review-render18-clearcoat-r3` rechecked coordinator snapshot
`3824` without editing or absorbing foreign changes. All five source hashes
match the manifest. Static inspection confirms the WGSL `vec3<f32>(1.0)`
`clearcoat_base_energy`, vector `direct_base_energy`, vector-valued clearcoat
helpers and propagation through direct/environment/transmission/emission paths;
the focused Naga anchor explicitly assembles clearcoat-disabled,
clearcoat-enabled and Blinn-Phong variants. Scoped `git diff --check` is clean;
the documented rustfmt drift remains confined to the three Rust template test
owners, with WGSL unaffected.

Independent result: **Critical=0 / Important=0 / Moderate=0**. This is static
source evidence only; no Cargo/Naga dynamic pass, WGPU product export, PNG, or
RenderDoc replay is inferred. The source manifest was sealed in snapshot 3824
before this receipt, so the current document hash is a post-snapshot doc-only
change. Managed/product gates, canonical return, fixed status, closeout and
WeCom remain pending.

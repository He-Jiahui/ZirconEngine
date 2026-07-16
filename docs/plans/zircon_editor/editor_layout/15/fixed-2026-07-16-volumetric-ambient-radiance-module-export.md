---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: volumetric-ambient-radiance-module-export
plan_link_mode: child_record_only
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/light_scatter.rs
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -VerboseOutput
  - cargo test -p zircon_runtime --lib --locked volumetric_ambient_radiance -- --exact
resolved_at: 2026-07-16
---


# Render18：体积雾环境辐射函数缺失模块出口

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行者：`editor-layout15-visual-refinement-20260714`
- 来源执行切片：ConfirmDialog 紧凑 body/action gap 的 fresh managed GREEN 验证
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：最低共享原因位于 Render18 正在修改的 froxel 模块出口；Editor Layout15 不拥有 advanced-lighting 的可见性合同。

## 失败现象与复现证据

Windows 受管 job `e6f0fac2d09143adbea90e4ced69350a` 执行
`.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -VerboseOutput`
时以 exit 1 正常 finish/release。编译在进入 `zircon_editor` 前失败：

```text
error[E0432]: unresolved import `super::super::volumetric_ambient_radiance`
 --> zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/light_scatter.rs:12:39
```

定义已经存在于 `froxel/light_scatter.rs`，但 `froxel/mod.rs` 没有向 sibling executor 暴露该函数。

## 最低共享层根因

Render18 在 executor 中从 `froxel` 根导入 `volumetric_ambient_radiance`，同时只在私有
`light_scatter` 子模块内声明了 `pub(crate)` 函数，没有同步 froxel 模块的 canonical
re-export。调用点和模块出口因此不一致。

## 架构修复验收

- 在 froxel 模块边界建立单一、明确的函数出口；不得在 executor 复制环境辐射计算。
- Render18 的 `volumetric_ambient_radiance` 聚焦测试必须通过。
- 原始 `zircon_editor` 受管验证必须越过 `zircon_runtime` 编译，并生成包含 Layout15 当前源码的 fresh test binary。
- Layout15 随后重跑 5 个 ConfirmDialog exact tests 和 workbench atlas ignored capture。

## 禁止临时方案

- 不得在 Editor/UI 路径添加 alias、fallback、条件编译绕过或重复的环境辐射实现。
- 不得删除 ambient-radiance 调用、削弱 Render18 测试或复用旧 `zircon_editor` binary 冒充 fresh GREEN。
- 不得由 Layout15 接管、提交或 release Render18 的受管 job。

## 修复结果与回传

- 根因：Render18 imported volumetric_ambient_radiance from the froxel root without re-exporting the existing light_scatter implementation.
- 架构修复：Added the canonical pub(crate) froxel module re-export; no duplicate implementation, alias, fallback, or Editor-side workaround was introduced.
- 验证：Managed job edd25ded210548dbabfea57f6fcf2087 compiled zircon_runtime and zircon_editor and produced fresh binary zircon_editor-7cbf6e3f9c684171.exe at 2026-07-16 04:58:03 +08. Five ConfirmDialog exact tests passed 5/5. Atlas ignored capture passed 1/1 and wrote docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png SHA-256 2334AB05CDF5FC6744870B79F4A71A79D9164FBCA7D4E6475186164D19A6AA99. The broad package runner entered its known idle-thread stall and was truthfully finished/released with exit 124; no full-package pass is claimed.
- 回传：E0432 is removed from the Layout15 upward gate; Render18 owns the returned froxel/mod.rs patch for its AF-M3 manifest, while Layout15 resumes its ConfirmDialog visual closeout.

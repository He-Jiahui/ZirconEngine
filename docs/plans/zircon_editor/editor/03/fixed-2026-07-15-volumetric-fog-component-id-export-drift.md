---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: volumetric-fog-component-id-export-drift
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_extract.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild
resolved_at: 2026-07-15
---


# Render18：Volumetric Fog 组件标识导出漂移

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Editor03 的 Navigation Editor 受管回归门在编译共享 `zircon_runtime` 时报告 E0432：`post_process/volume_extract.rs` 从 `core::framework::render` 导入 `VOLUMETRIC_FOG_COMPONENT_ID`，但 Render18 advanced-lighting 导出链只发布 `VOLUMETRIC_FOG_VOLUME_COMPONENT`。该共享编译失败先于 Editor03 测试执行，门禁不计通过。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M3.2 operation factory/runtime wiring 的 Navigation Editor Windows 受管回归门
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：失败符号与消费点都位于 Render18 的 volumetric fog/Volume 注册链，Editor03 不拥有该标识或导出边界。
- 生命周期键：`volumetric-fog-component-id-export-drift`

## 失败现象与复现证据

Windows 受管命令 `./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild` 在编译 `zircon_runtime` 时返回 E0432：

- `zircon_runtime/src/core/framework/render/post_process/volume_extract.rs:3` 导入 `crate::core::framework::render::VOLUMETRIC_FOG_COMPONENT_ID`；
- `zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs` 定义该常量；
- `advanced_lighting/mod.rs` 与 `render/mod.rs` 当前只导出 `VOLUMETRIC_FOG_VOLUME_COMPONENT`，没有发布消费点所引用的标识。

该错误出现在 Editor03/Navigation Editor 自身编译与测试之前，因此不能用此前生成的测试二进制替代当前源码门禁。

## 最低共享层根因

Render18 AF-M3 的 volumetric fog Volume 注册接线在新增消费点时没有同步稳定标识的模块边界：定义、descriptor 与 `volume_extract` 消费者之间的单一公开路径不一致。最低修复层是 Render18 自有的 advanced-lighting/Volume 导出合同。

## 架构修复验收

- 由 Render18 owner 确定并保留一个 canonical volumetric fog component identifier，定义、descriptor 注册和 `volume_extract` 必须通过同一 owner 路径消费。
- 不得同时保留两个语义重叠的公开常量；若 `VOLUMETRIC_FOG_COMPONENT_ID` 只供 render 内部消费，应使用最窄的显式模块路径/可见性。
- 重跑 Render18 volumetric/advanced-lighting 聚焦测试，再重跑上述 Navigation Editor 受管包门，确认当前源码越过共享 runtime 编译并实际执行 Editor03 测试。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在 unrelated crate root 增加第二份常量、用字符串字面量绕过 owner，或删除/跳过 `volume_extract` 的 volumetric override 解析。
- 禁止弱化 Editor03/Navigation Editor 包门来隐藏共享 Runtime 编译失败。

## 修复结果与回传

- 根因：The volumetric fog component identifier was defined in volumetric.rs but omitted from the advanced_lighting and render curated export chain consumed by VolumeComponentOverride.
- 架构修复：Published the single canonical VOLUMETRIC_FOG_COMPONENT_ID through advanced_lighting/mod.rs and render/mod.rs; volume extraction consumes that canonical owner without aliases or duplicate constants.
- 验证：.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild completed on Windows with managed job cd3c852c7abf4e0dbbb511935dfa037d and exit_code=0.
- 回传：Editor03 Navigation Editor current-source gate is green and may resume operation factory/runtime wiring acceptance.

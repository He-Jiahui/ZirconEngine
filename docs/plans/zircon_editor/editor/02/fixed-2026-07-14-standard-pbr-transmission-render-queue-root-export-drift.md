---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: standard-pbr-transmission-render-queue-root-export-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/tests/assets/material/advanced_features.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
resolved_at: 2026-07-14
---


# Render 18：Standard PBR transmission render queue 根导出漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 测试阶段，Runtime15 handoff 返回后的 exact core-min scene 复验
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：transmission queue 常量的定义、框架根导出与 material consumer 接缝均属于 Render18 正在实施的 AF-M1/AF-M2 边界；Editor02 不得复制 `2900`、下钻私有 child module 或恢复临时别名。

## 失败现象与复现证据

受管 Windows job `ba86a72b259340338ab60dbb54f3d5d0` 执行：

```powershell
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

在测试执行前以 exit `101` 停止，出现 2 条 `E0432`：

- `zircon_runtime/src/asset/assets/material/material_asset.rs:11` 无法从 `crate::core::framework::render` 导入 `STANDARD_PBR_TRANSMISSION_RENDER_QUEUE`；
- `zircon_runtime/src/asset/tests/assets/material/advanced_features.rs:3` 同样失败。

定义已位于 `advanced_lighting/material_features.rs`，且 `advanced_lighting/mod.rs` 已导出；`core/framework/render/mod.rs` 的公开 advanced-lighting 列表遗漏该常量。完整日志：`E:\ZirconBuilds\editor02-m1-coremin-final-after-runtime15-20260714.log`。

## 最低共享层根因

Render18 在新增 transmission queue typed constant 与 material consumer 时，只完成 child owner 导出，没有同步 canonical `core::framework::render` 根表面。消费者已按框架根边界引用，因此这是同一 Render18 切片的公开表面不完整，而非 Editor02 或 asset domain 的替代所有权。

## 架构修复验收

- `STANDARD_PBR_TRANSMISSION_RENDER_QUEUE` 由唯一 typed definition owner 提供，并从 Render18 约定的 canonical framework render 表面可见。
- transmission material production 与测试消费者编译通过，常量仍为 typed `RenderQueueValue`，不得复制裸 `2_900`。
- 重跑 Render18 focused transmission/advanced-lighting tests，再重跑 Editor02 原始 exact core-min scene 门禁；Runtime15 新生命周期守卫也必须在该门禁内通过。

## 禁止临时方案

- 不得在 material consumer 复制 `2_900`、添加本地常量或从私有 child path 下钻导入。
- 不得添加旧名称 alias、兼容 re-export 层、cfg(test) 特例或静默 fallback。
- 不得削弱测试或计划验收条件来隐藏失败。

## 修复结果与回传

- 根因：Render18 added the typed transmission render-queue definition and child export but omitted it from the canonical core::framework::render root surface.
- 架构修复：Export STANDARD_PBR_TRANSMISSION_RENDER_QUEUE from the existing framework render root while retaining the unique typed definition and canonical consumer imports.
- 验证：Managed Windows job 1d651b687cf647fe8498321d7095c731: cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked; 596 passed, 0 failed.
- 回传：The transmission queue canonical export is complete and the original Editor02 exact core-min gate is green without aliases or copied constants.

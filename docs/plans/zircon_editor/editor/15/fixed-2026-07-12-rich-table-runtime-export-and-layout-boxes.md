---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: rich-table-runtime-export-and-layout-boxes
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/text/mod.rs
  - zircon_runtime/src/text/model/rich.rs
  - zircon_runtime/src/text/rich/bbcode_table.rs
  - zircon_runtime/src/text/rich/bbcode_table/attributes.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline_vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
tests:
  - cargo check -p zircon_runtime --lib --locked
  - cargo test -p zircon_runtime --lib text_rich --locked -- --test-threads=1
  - cargo test -p zircon_editor --lib --no-run --locked --jobs 1 --target-dir <coordinator-managed-target>
resolved_at: 2026-07-12
---


# Editor UI 03：Runtime rich-table export 与 resolved-layout boxes 编译失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor 15 M1.1 Windows 当前源码 lib-test no-run
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 交接原因：最低共享故障位于 Text07 rich-table DTO 公开面与 shared resolved-layout 构造契约，Editor 15 只消费 Runtime UI，不拥有文本类型导出或 layout boxes。

## 失败现象与复现证据

Editor 15 在 Windows 受管测试池执行当前源码 `zircon_editor` lib-test no-run 时，`zircon_runtime` 文本层先于 Editor 15 代码编译失败：

- `graphics/text/rich/bbcode_table.rs` 与 `bbcode_table/attributes.rs` 无法从 `core::framework::render` 导入 `RichTableCellBoxStyle` / `RichTableCellPadding`；
- `ui/text/layout_engine/rich_inline.rs` 与 `rich_inline_vertical.rs` 构造 `UiResolvedTextLayout` 时遗漏新字段 `boxes`。

原始复现：

```powershell
cargo test -p zircon_editor --lib --no-run --locked --jobs 1 --target-dir <coordinator-managed-target>
```

## 最低共享层根因

Text07 同一切片增加了 neutral rich-table cell box DTO 与 `UiResolvedTextLayout.boxes`，但共享公开面和所有完整 layout constructors 没有原子更新。类型定义本身存在，消费 owner 仍只依赖约定的 `core::framework::render` 公开面；horizontal/vertical rich-inline 也必须显式声明其当前不产生 table boxes，不能让 Editor 调用点补字段或添加导出旁路。

## 架构修复验收

- Rich-table DTO 只从既有 Runtime render text 公开面导出，BBCode consumer 使用同一唯一路径。
- 普通 horizontal、VerticalRl 与 rich-inline layout constructors 显式建立 `boxes`；table owner 才生成语义 box frames。
- Runtime Interface production/test、Runtime current-source check 与 `text_rich` focused tests通过。
- 晚于相关源码的 Editor lib-test binary 成功产出，证明原始 no-run 编译边界越过该失败点。

## 禁止临时方案

- 禁止旧路径兼容 re-export、Editor 专用转发导出、条件编译或调用点类型别名。
- 禁止通过删掉 `boxes`、serde-only 默认或弱化 table layout 测试掩盖构造契约。

## 修复结果与回传

- 根因：Text07 added RichTableCellBoxStyle/RichTableCellPadding and UiResolvedTextLayout.boxes without atomically updating the existing render-text export surface and every horizontal/vertical rich-inline constructor.
- 架构修复：Export the neutral table-cell DTOs only through the existing core framework render text surface; initialize empty boxes in non-table horizontal and VerticalRl layouts; let rich-table layout alone emit resolved semantic box frames. No Editor forwarding export, compatibility re-export, cfg bypass, or renderer reconstruction was added.
- 验证：Runtime Interface production/test passed; current Runtime locked lib check passed; text_rich 68/68 and renderer rich-table exact 1/1 passed; a managed Editor binary newer than all related sources exists and executed the original HUD framebuffer exact 1/1. Fresh Editor relink timed out after 1204.4s without diagnostics and is not counted as green.
- 回传：Editor 15 can resume beyond the rich-table export/layout-box compile boundary. The Text07 fixes are accepted; unrelated Editor 15 and global handoff validation failures remain independently owned.

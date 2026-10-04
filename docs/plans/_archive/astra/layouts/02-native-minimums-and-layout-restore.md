---
status: in_progress
review_date: 2026-09-05
plan_sources:
  - docs/plans/astra/layouts/01-responsive-layout.md
  - docs/plans/optimize/zircon_editor/193-editor-scene-viewport-layout-split-view-orthographic-quadrant-maximize-link-sync-slot-focus-toolbar-persistence-performance-product-integration-current-source-review.md
---

# Editor 原生最小尺寸与布局恢复

## 范围

本计划只处理 Editor workbench 的三个共享边界：把已计算的逻辑最小尺寸转换为物理尺寸并在 event-loop owner 上同步到原生窗口；为低于正常最小高度的合成输入定义确定性 band 优先级；在布局加载后的共享 normalize 边界修复非法 split ratio，同时保留合法递归拓扑和 ratio。

多 leaf 产品投影、per-slot viewport、toolbar/focus/render identity 和 Hub 不在本计划内。现有 split 产品入口在独立计划通过双 leaf 产品测试前不得据此宣称完成。

## 里程碑

### M1：原生最小尺寸

- 原生窗口创建时直接应用 geometry 已转换、shell 已发布的 physical minimum；仅发布值无效时才把 logical fallback 乘以 scale factor。
- layout tier、drawer constraint 或 DPI/scale 变化导致 minimum 改变时，只在 event-loop owner 更新原生窗口。
- 非有限、非正 scale 使用 `1.0`；发布的物理值直接向上取整并饱和到 `u32`，禁止再次按 DPI 缩放。

### M2：极短窗口优先级

- underflow 按 `top bar -> host bar -> document minimum -> bottom strip -> status` 分配。
- 低优先级 band 只有在 separator 与至少 1 logical pixel 同时可用时才出现；separator 只存在于相邻正高度 band 之间。
- 正常高度继续使用现有 constraint solver 和 compact bottom 合同。
- authoritative layout 的零高 band 穿过 retained publication/root frames 时保持为零；只有整份布局缺失时才走 legacy fallback。

### M3：布局恢复 ratio

- 共享 normalize 边界将非有限 ratio 恢复为 `0.5`，并把有限 ratio 限制到 `[0.1, 0.9]`。
- 递归处理每个 split node，不改变合法 ratio、axis、leaf 顺序、tab assignment 或 active tab。

### M4：验证阶段

- 运行源码格式、diff check 和 focused source guards；Cargo 留给后续受管批次。
- focused production tests 覆盖创建/DPI换算、短高度边界、嵌套合法 topology round-trip 和非法 ratio normalize。
- 后续 Windows 产品验收必须实际尝试把窗口缩到 physical minimum 以下，并在 DPI 切换后检查 client rect 与更新后的 minimum。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|

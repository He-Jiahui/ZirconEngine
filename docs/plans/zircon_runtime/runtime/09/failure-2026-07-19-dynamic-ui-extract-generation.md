---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: dynamic-ui-extract-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
tests:
  - stable UI extract build/visit counters
  - menu/HUD F2 pixel parity
---

# Runtime09：dynamic UI extract generation

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-433 dynamic UI extract generation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：menu/HUD component generation 与 viewport-owned UI extract 的最低共享 owner 是 Runtime09。

## 失败现象与复现证据

每次capture/present都全扫World寻找menu或HUD，稳定文本仍clone并重建command Vec与style color Strings。HUD分类的临时token Vec已局部删除，但UI artifact没有component/viewport generation owner。

## 最低共享层根因

menu/HUD 尚未发布 component/viewport generation-owned UI artifact，稳定帧只能重新扫描 World 并重建命令与样式数据。

## 架构修复验收

- 按menu/HUD component generation与viewport size发布唯一Arc UI extract，scene change set只失效目标entity。
- stable capture只借用handle，world visits/build/String/Vec alloc均0；changed generation build≤1。
- 1/1k/100k nodes与1/60/240Hz验证menu click、HUD抑制、resize、layout和像素等价；回传PERF-MVP-433并联动Render14/EditorUI08。

## 禁止临时方案

不得保留每帧 World 全扫、引入可漂移的 String cache，或在稳定 generation 上继续全量重建 command Vec。

## 修复结果与回传

Open state: `待 Runtime09 发布generation-owned menu/HUD extract并回传规模证据`。

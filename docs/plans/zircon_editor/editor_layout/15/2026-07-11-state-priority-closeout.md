---
status: complete
completed_at: 2026-07-11
plan_source: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime/src/ui/tests/render_painter_state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes
  - zircon_editor/src/tests/host/retained_menu_pointer/state_priority_visual_screenshot.rs
evidence:
  - docs/tests/editor/editor-components-state-priority-900x360.png
---

# Layout 15：交互状态优先级闭环

## 完成范围

- 关闭 `S15.4gf/S15.6fg` 至 `S15.4km/S15.6jn`、`S15.4ft/S15.6eu`、`S15.4fr/S15.6es`、`S15.4fp/S15.6eq`、`S15.4fo/S15.6ep`、`S15.4fn/S15.6eo`、`S15.4fl/S15.6em`、`S15.4fk/S15.6el`、`S15.4fj/S15.6ek` 的 focused-only、pressed/drag、selected/checked 与 chrome/row/tab 状态优先级证据。
- `focused` 只表达键盘焦点，不再把 alert、notification、TreeView、DataGrid、generic surface、page/dock/module tab 提升成 pressed、hovered 或 selected 表面。
- pressed/drag/open/disabled/loading 与 selected/checked identity 继续按共享 `UiPainterResolvedState` 决议；runtime render extract、retained painter 与截图 fixture 使用同一语义。

## 回归与诊断

- Runtime Interface painter-state 合同：6/6 通过。
- Runtime 当前源码 binary：aggregate render-extract 1/1，通过；Material button、collection row、selection control、chrome、alert、notification focused matrix 8/8，通过。
- Editor retained focused matrix：此前完整执行 16/16；本次重新抽验 pressed、drag、focused alert 与视觉 route 4/4，通过。
- RED 诊断发现 aggregate fixture 仍断言旧高饱和色：button hover `#43ccd8`、field focus `#35c7d0`、slider halo `#35c7d03a`。生产 owner 与专用测试已硬切为低强调 palette，故只把陈旧 fixture 收敛到 `#263d43`、`#2aa6b8`、`#d8e3e71a`；未修改生产行为、未增加兼容分支。
- `rustfmt --check zircon_runtime/src/ui/tests/render_painter_state.rs` 与 scoped `git diff --check` 通过。

## 截图证据

- 文件：`docs/tests/editor/editor-components-state-priority-900x360.png`
- 尺寸：900×360；大小：41982 bytes。
- SHA256：`F7621E64E030358A22E78992C5CEF29DB8D577FF67D87D2F4CA4D14D42F2491C`。
- 人工复核：tone alert 保留色调边界；focused row/tab 不冒充 selected/hot；pressed/drag/checked 层级可辨且保持低强调 Unreal/AI workbench 暗色风格。
- 存放扫描：`docs/tests/editor` 精确命中 1；仓库 `target`、托管 Cargo lane、外部 Editor Cargo target 精确命中均为 0。

## 外部信号

- 全仓 handoff 审计仍报告其他计划的并发 schema/link/self-edge 问题；本次 Editor M1 的 ZUI governance、plugin provider、font discovery 三个 fixed lifecycle 均已单独复验并通过，未把外部诊断计入本切片失败。
- 整体 Editor Layout 目标保持 active；下一阶段继续从原子控件进入紧凑复合控件与自适应窗口组合。

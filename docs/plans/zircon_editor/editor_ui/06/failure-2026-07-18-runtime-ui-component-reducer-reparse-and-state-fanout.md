---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-component-reducer-reparse-and-state-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/component/state_reducer/command_palette.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/submenu.rs
  - zircon_runtime/src/ui/component/state_reducer/windowing.rs
  - dev/slint/internal/core/model.rs
  - dev/slint/internal/core/model/adapters.rs
tests:
  - 10k command and recursive-menu input scale test
  - stable-generation arrow navigation zero-parse test
  - atomic component state patch changed-field test
---

# Runtime UI component reducer每事件重解析与多字段fanout

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：state reducer 22/22逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 联动责任：EditorUI01提供输入/timer budget；EditorUI03拥有text edit-state特化。
- 交接原因：typed component state、options/commands schema与alias兼容边界由EditorUI06统一拥有。

## 失败现象与复现证据

PERF-MVP-265：virtual range约15个alias、submenu 5、world surface 6个独立BTreeMap写，静态key重复分配且alias为第二authority。PERF-MVP-266：CommandPalette/Menu每字符或方向键重建owned DTO/递归树、lowercase全部字段、复制filtered ids；hidden/disabled检查可退化O(N²)并反复clone filtered Vec。

## 最低共享层根因

通用`UiValue`既是authoring输入又被当作热路径索引；没有model generation、compiled entries、canonical field identity或atomic changed patch。

## 架构修复验收

- `UiComponentStatePatch`一次提交flags/values/reference-source，只返回实际changed canonical fields；alias只在投影边界生成。
- commands/options generation维护typed entry、normalized corpus、id/index、disabled set、parent/child与filtered mapping。
- query变化至多一次filter；稳定generation方向键近O(1)，parse/String clone/lowercase/filter rebuild=0。
- 100/1k/10k entries、depth 1/8记录key alloc、state writes、parse/clone/filter/contains与CPU p95；Unicode search、typeahead/submenu/selection及Cargo通过。

## 禁止临时方案

- 不得为每个组件family复制一套无共享generation的cache。
- 不得只删除alias字段而破坏资产兼容；canonical authority和兼容投影必须同时落地。

## 修复结果与回传

Open state: `等待EditorUI06回传atomic state patch、generation-owned command/menu index及规模证据`。

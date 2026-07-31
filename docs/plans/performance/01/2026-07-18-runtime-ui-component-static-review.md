---
related_code:
  - zircon_runtime/src/ui/component
  - zircon_runtime/src/ui/tests/component_catalog/catalog_inventory.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/table.rs
  - zircon_editor/src/ui/asset_editor/palette/build.rs
  - zircon_editor/src/ui/asset_editor/palette/native_slots.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
reference_sources:
  - dev/slint/internal/core/model.rs
  - dev/slint/internal/core/model/adapters.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/IItemsSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h
tests:
  - shared catalog source-level RED to GREEN guard passed
  - DataGrid sort and column-width source-level RED to GREEN guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows component tests pending
  - collection and reducer scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI component逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/component` 59/59：catalog 32、state reducer含root 22、descriptor 2、data binding 2与module root 1；目录原先无外部脏文件。连同前批，`ui`累计生产文件171/783。产品调用追踪覆盖asset palette、document compiler、retained registry、showcase event与v2 surface interaction。

## PERF-MVP-264：OnceLock后仍深clone catalog

editor/material registry只构建一次，但工厂返回owned `Self`，每次复制全部descriptor及其String/Vec/BTreeMap。`available_native_slot_names`会按调用复制，showcase每个组件事件也复制；palette build还在随后再次clone/sort metadata。本轮用源码RED→GREEN增加两个`*_shared()`静态借用入口，保留owned API兼容，并切换palette build、native slot和showcase event三条只读热路径。compiler default与组合registry仍需EditorUI06用Arc/generation handle收口。

## PERF-MVP-265/266：通用state fanout与每事件重解析

单一逻辑动作被展开为多字段BTreeMap写入：virtual range约15个alias、text edit 8+mirror、submenu 5、world surface 6；静态key重复分配，alias成为第二authority。CommandPalette与Menu又在每个字符/方向键从`UiValue`重建owned DTO/递归树、逐字段lowercase，navigation甚至先重跑filter。filtered/disabled使用Vec线性contains，hidden检查可为每candidate复制整份filtered id Vec。

EditorUI06需建立canonical typed state patch及commands/options generation index。Slint明确要求持久model并以`row_changed/added/removed`通知，而非替换整份model；其FilterModel长期保存sorted mapping，单row变化用binary search局部更新，正是本目录缺失的ownership边界。

## PERF-MVP-267/268：TreeView与DataGrid

TreeView每个select/expand/nav/edit都递归重建owned id Vec，`push_unique`线性查重导致最坏O(N²)；begin edit先建全id表再第二遍找label。UE `STreeView`保存linearized items、parent index与sparse expanded state，items由observable source触发refresh，而不是每键从通用值重建索引。

DataGrid client sort在UI线程排序完整rows，原比较器为常见String/Enum也调用`display_text()`分配；column resize原先clone完整width map。本轮已让string-like scalar借用`&str`比较并原地更新width map。typed column comparator、field index、large sort worker与row permutation仍交EditorUI06。

## PERF-MVP-269/270：feedback与文本

NotificationCenter每键多次解析全部entry并重算unread；Toast timeout深cloneraw map、重建余下queue后再次全parse，且无entry/byte hard cap。普通TextInput则每键clone全文构建edit state，再为primary/mirror/caret/selection/composition多字段写回，并在change validation全量grapheme count。feedback model归EditorUI06，文本atomic edit-state与Text09共享结果归EditorUI03；PERF-MVP-270回链AT入口的PERF-MVP-258但不混淆两个产品入口。

## 无新增热区的文件

descriptor validation只在register时运行，data-source builder很小，material/editor showcase descriptor文件均为首建期构造器；它们的主要运行时风险是registry深clone，已统一记入PERF-MVP-264，不按每个constructor重复立项。

## 责任计划与验收

EditorUI06收到catalog、state/filter、tree/table与feedback四份failure，EditorUI03收到普通text edit一份failure。100/1k/10k entries/nodes/rows与10k event序列记录registry/value/String clone bytes、parse/filter/dedupe/compare、state writes、main-thread CPU、queue bytes/age；current-source Cargo和MVP palette/menu/tree/table/text/notification产品trace完成前，59/59仍留pending。

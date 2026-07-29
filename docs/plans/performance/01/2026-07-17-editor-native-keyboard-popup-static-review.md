---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/host_page_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/menu_popup_metrics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/menus.rs
  - dev/slint/internal/core/window.rs
tests:
  - retained popup keyboard and host-page overflow tests
  - context-menu projection tests
  - current-source Windows Cargo pending
  - 1000-key and 1/100/10000-row scale trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native keyboard/popup逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`native_keyboard.rs` + `native_keyboard/**`共 **13** 个Rust文件、**584** 行；`native_popup_dismiss.rs` + 子文件共 **3** 个、**112** 行；`workbench_context_menu.rs` + 子文件/测试共 **6** 个、**203** 行；`host_page_overflow_menu.rs`和`menu_popup_metrics.rs`各1个、合计 **129** 行。总计 **24/24** 个文件、**1,028** 行已逐文件阅读。当前源Cargo、按键storm和规模trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Winit key只映射Next/Previous/First/Last/Accept/Cancel，未消费键继续交给其它输入路径；popup row跳过disabled/separator，支持wrap、selected/focused/hovered初始项、host page overflow、command palette commit、普通option/menu action与top popup cancel。Outside primary press区分trigger/popup containment和damage union。Context menu provider只在secondary hit时按scene/module/generic分类，拒绝popup row嵌套菜单；这是低频命令构造，不是静态主线程热点。Menu text width的重复font/theme成本已分别归PERF-MVP-156/161。

## 热点与计划

PERF-MVP-170：每个popup导航键和typeahead字符都调用`get_host_presentation()`深clone完整host snapshot，然后为Workbench倒序调用`row_data`，逐项clone约160字段的`TemplatePaneNodeData`。每个候选再遍历全部option/menu rows，新建`Vec<PopupKeyboardRow>`并复制action/value/identity/search/frame，之后才选择一个next row；host-page overflow也重建全部hidden tab rows。Typeahead将query lowercase一次，但每个row最多把search/value/identity各lowercase并分配一次。Outside press另走一次全树popup discovery。

EditorUI01应在surface/presentation generation提交时发布stable active-popup stack、navigation rows和current index；按键只移动index或在已提交rows上做一次folded prefix search，dismiss只查询top popup。该模型必须与PERF-MVP-146/147的control/hit index同代提交，并在popup/template/row generation变化时失效，不能由keyboard/dismiss consumer各建私有缓存。Slint的menu shadow tree由`PropertyTracker::evaluate_if_dirty`仅在依赖dirty时重建，window则直接保存`active_popups`，说明popup identity/lifetime应由window/surface owner维护而非按事件反射整树。

两个局部确定项可直接收敛：`popup_text_starts_with`以Unicode lowercase iterator比较prefix，删除每row字符串分配；uniform host-page overflow以Y/stride计算至多一个candidate并保留gap/边界contains检查，删除O(hidden pages) hit loop。两项先补回归测试，再在文本输入跨计划编译修复回传后跑current-source gate。

## 动态验收

对1/100/10,000 controls与popup rows分别执行1,000次Arrow/typeahead/outside press，记录full presentation clones、wide node clones、row Vec builds、visited nodes、lowercase/string allocations和p95。稳定按键的前四项必须为0，visited不随无关tree size增长，typeahead row lowercase allocation为0；outside只查top popup。保持disabled/separator、wrap/first/last、Unicode search、selected/focused/hovered、accept/cancel、command palette/option/menu/overflow callback、damage和视觉等价。

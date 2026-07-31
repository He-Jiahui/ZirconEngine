---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/binding
  - zircon_runtime/src/ui/event_ui
  - zircon_runtime/src/ui/theme
  - zircon_runtime/src/ui/tree
  - zircon_runtime/src/ui/tests/material_button_style.rs
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/material-ui/packages/mui-material/src/Button/Button.js
  - dev/slint/internal/core/properties.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/NavigationMetaData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
tests:
  - button single-map source-level RED to GREEN guard passed
  - StyleProperty legacy entry remains required and built-ins forward to borrowed values
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime UI tests pending
  - navigation/reflection scale counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI root/binding/event/theme/tree逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/ui`当前物理与Git tracked均为783个Rust文件。本轮逐文件完整阅读root 5/5、`binding` 3/3、`event_ui` 10/10、`theme` 1/1、`tree` 9/9，共28/28个生产文件，并定向阅读`ui/tests/material_button_style.rs`与editor control/reflection调用图。外部同时修改的`ui/module.rs`只读；本轮源码写入限定于`ui/style.rs`及其测试。

## PERF-MVP-251：button单表样式重复物化

`resolve_button_style_from_values`被editor asset/view/pane/workbench投影使用。原实现先clone完整`BTreeMap<String, toml::Value>`，构建`Arc<StyleSheetScope>`与`Weak`，再让17个属性各自upgrade/查表；variant/color/size/icon/state字符串还分别复制并lowercase分配。

本轮以源码RED→GREEN将单表入口改为直接借用同一map并一次构造`ResolvedButtonStyle`，内置property的旧`extract(UiV2ResolvedStyle)`入口继续为必实现契约并转发借用入口，避免破坏外部旧实现或引入默认递归；字符串解析使用`&str + eq_ignore_ascii_case`，只有自定义style/role结果保留必要String所有权。Material UI Button同样围绕一次owner state/theme计算variant组合，不要求为每个字段复制整份样式表。剩余stable-generation重复解析与theme snapshot归EditorUI04。

## PERF-MVP-252：失联订阅与无界fanout

`UiControlRequest::SubscribeDiffs`调用`subscribe()`后立即丢弃receiver，只向请求方返回id；sender永久留在`subscriptions`。每次远程请求都会新增无法消费的死订阅，之后reflection和invocation broadcast仍逐订阅clone notification并send。正常本地订阅也使用crossbeam unbounded channel，没有queue bytes、合并、过载或断开清理策略。

`replace_tree`还会在每次snapshot发布后清空并重建所有tree的node index，diff前后对目标tree做全节点比较。editor高频事件触发完整reflection的上游根因已登记PERF-MVP-076/099；本项只登记event manager新增的死订阅/无界队列和跨tree全索引放大，避免重复归因。EditorUI08需让transport拥有真实receiver/sink生命周期，以bounded、按tree generation可合并的diff队列替代永久sender，并从changed set增量维护目标tree索引。

## PERF-MVP-253：每个导航键重建候选

产品输入通过`UiSurface`调用`UiTree::next_navigation_target`。每个Tab、Shift-Tab或方向键都递归遍历所有roots重建focus candidates；每个候选再沿parent链求active MUI modal root并clone navigation group id，随后retain并sort。manual group target又单独调用同一全量构建与排序，最坏为O(N*depth + N log N)/key。

EditorUI01需把tab order、group首项、node→group/modal ancestry与paint order放入tree/layout generation拥有的navigation index；stable generation按键不重建、不排序、不沿祖先链。UE Slate把explicit/custom/wrap/stop/escape放入`FNavigationMetaData`并由应用路由消费；Zircon索引化不能丢失这些边界规则、manual target或nested modal语义。方向导航可先使用scope-local候选索引，规模证据不足前不强制特定空间结构。

## 其他观察、责任计划与验收

binding值/动作类型是轻量契约，未发现独立产品热点。legacy `UiHitTestIndex::hit_test(tree)`会即时arrange，但产品`UiSurface`已有arranged tree与hit grid；radius query的Vec去重/排序先回链后续surface审查，当前不重复立项。`UiEventRouter`当前仅测试引用，也不作为MVP产品瓶颈登记。

EditorUI04/08/01分别收到button style generation、detached subscription与navigation candidate failure记录。1/100/10k style/node/subscriber规模下记录map/String clone bytes、Arc upgrade、property lookup、candidate rebuild/sort/ancestor walk、queue entries/bytes、notification clone/send/coalesce/drop与CPU p50/p95；stable generation的样式重解析、导航候选构建和死订阅fanout应为0。current-source Cargo、editor产品键盘/reflection trace与行为矩阵完成前，本模块仍留`pending.md`。

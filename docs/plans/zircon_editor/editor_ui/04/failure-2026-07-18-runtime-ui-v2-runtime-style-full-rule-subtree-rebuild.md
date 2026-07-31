---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-v2-runtime-style-full-rule-subtree-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/v2/style/tokens.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/slot_contract.rs
  - zircon_runtime/src/ui/template/asset/style.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetHeap.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp
tests:
  - 10k pointer-state affected-node and rule-probe counter
  - stable computed-style zero-map-clone test
  - selector specificity state and token pixel parity matrix
---

# Runtime UI v2伪状态全规则与全子树重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：v2 style/runtime_state/tokens逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md`
- 交接原因：selector compilation、computed style、theme/token与pseudo-state authority由EditorUI04拥有。

## 失败现象与复现证据

PERF-MVP-275/307：hover/focus/press遍历指定root整棵子树，每node扫描全部pseudo rules并重建attributes/overrides/tokens；asset compile style同样为每node复制ancestor path并扫描全部rules，且每rule复制sheet tokens。state path分配String Vec并排序/去重alias。本轮局部止损已删除每node children clone、matched rule深clone和slot owner attributes clone，但主要R×N×depth工作仍在。

## 最低共享层根因

parsed selectors只是按specificity排序的flat Vec，没有component/id/class/state候选索引、rule→node依赖、interned state或computed-style generation；baseline又复制为三张owned map。

## 架构修复验收

- selector编译为候选索引与interned state bitset，state变化只访问affected rules/nodes。
- computed style共享immutable baseline，更新生成typed delta；unchanged metadata write/map clone=0。
- dirty work去重并按layout/render影响传播，参考UE Slate unique invalidation heap，不由单叶事件重扫整子树。
- 1/100/10k nodes×1/100/1k rules及10k状态事件记录rule probes、String/map clone bytes、visited/changed nodes与CPU p95；specificity/theme/token/像素通过。

## 禁止临时方案

- 不得只缓存selector parse结果；当前热点是候选匹配、路径/状态物化与整子树apply。
- 不得把hover等高频事件延迟合并到不可预测帧而缺少work budget和最终状态保证。

## 修复结果与回传

Open state: `等待EditorUI04回传indexed selector、computed-style delta、dirty-work预算与像素证据`。

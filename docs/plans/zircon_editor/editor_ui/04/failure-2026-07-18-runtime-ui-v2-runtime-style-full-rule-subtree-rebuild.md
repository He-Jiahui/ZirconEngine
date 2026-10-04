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
  - zircon_runtime/src/ui/v2/style/attribute_delta.rs
  - zircon_runtime/src/ui/v2/style/attribute_delta/tests.rs
  - zircon_runtime/src/ui/v2/style/rule_index.rs
  - zircon_runtime/src/ui/v2/style/rule_capacity_tests.rs
  - zircon_runtime/src/ui/v2/style/selector_path_tests.rs
  - zircon_runtime/src/ui/v2/style/runtime_state/capacity_tests.rs
  - zircon_runtime/src/ui/v2/style/rule_index/candidate_capacity_tests.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime/attribute_delta.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_v2_style.rs
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

## 2026-09-27 当前源码修复切片（动态验收未完成）

- 沿用现行 primary `failure-roll-01a084c8-editorui04-button-style-r2`，保留其 `6b4bc86089cb4464f850136c8079e90cb6513ebc` / epoch 611 基线与历史 snapshot 3817；该 Session 没有实际已接受或待处理验证票据。r1 的静态票据不迁移，也不作为本次动态验收。
- 审计转移 `38cf7fbed65aa001ff2426e7086fe254fa34f331183132b01c33646912d58d34` 和 `5758a7bb1b1a51332cc3683e37e9823473c53d44e890d1c72b9eca8598b9a5b0` 保留已归档或无归属的现行源码，并补齐实际 `--lib` 编译所需的 indexed-rule 与 capacity/selector 测试模块；未覆盖其他活动 owner。
- 最低共享层 `style/attribute_delta.rs` 借用 baseline、匹配规则及 retained state，按字段原地更新三张 metadata map。未变数组、表和字符串保留原分配；已退出的规则恢复 baseline 或删除其引入的字段。inline override 的保护、retained state alias 的移除和 token 的 `.` / `[` 前缀边界保留原语义。
- 行为回归通过真实 `UiV2Surface` 覆盖 1/100/10k 菜单项、hover 开关、重复 restyle、`mark_dirty=false`；检查菜单数组与代表性 label 地址、背景恢复、outline 删除和 dirty 范围。另有下层属性恢复、inline override、token 边界及 metadata-only dirty 回归。当前只完成 Rustfmt 解析和格式检查，尚未动态执行这些测试。
- 尚余：未变嵌套 `Value` 的相等比较仍访问菜单内容；匹配规则仍物化 resolved declaration。不得把本切片当作 O(1) 点击、零规则值复制或整个 lifecycle 完成。原文 nodes×rules×状态事件的 counters、CPU p95、theme/token/像素与上行验收全部保持 open。
- 共享受管 Cargo 准入当前被外部工作树归档错误阻挡：Editor09 command request `fb23cdfc5d1a4856abf8a67b7dca6d7c` 在提交阶段终态失败，未生成可复用票据。该 generic 错误丢失原 IO/tar 异常原因，尚不能断言发生并发源码漂移；不得手工封包或提供调用方 archive/provenance 绕过准入。

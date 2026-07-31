---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-virtualization-full-child-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
tests:
  - 100k-row fixed-extent visible-range visit-count test
  - variable-extent index correction and cache invalidation test
  - nested scroll focus accessibility behavior matrix
---

# Runtime UI虚拟列表仍全量遍历并隐藏offscreen子树

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：layout scroll/virtualization/arrange逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 联动责任：EditorUI01滚动/focus/accessibility范围；Text09提供动态行高measure cache generation。
- 交接原因：virtual extent索引、range generation与特殊容器布局合同由EditorUI02统一拥有，输入和文本计划只消费该合同。

## 失败现象与复现证据

PERF-MVP-262：每次virtual arrange为全部children构建positions再逐项窗口判定，offscreen节点递归clone children并清零layout。即使固定extent，单步scroll的CPU仍随total rows增长。现有`scroll_virtualization.rs`只在6行fixture断言窗口外frame为default，没有position/hide/measure visit counter，因此“only materializes visible window”不能作为复杂度验收。

## 最低共享层根因

virtual window只输出range，没有可按index寻址的position authority、动态extent前缀索引或进出窗口delta；offscreen状态依赖全量递归写回维持。

## 架构修复验收

- fixed extent以算术O(1)求range和position；variable extent使用与text measure cache同generation的prefix/Fenwick或分块索引。
- 每帧只访问visible+overscan及进入/离开窗口edge，offscreen subtree visited=0；range generation供focus/accessibility/render共同消费。
- 1k/10k/100k rows、visible 10/50连续滚动记录position/slot/layout/hide visits、index update、alloc和CPU p95，per-step不随total rows增长。
- fixed/variable/estimated extent、reverse、clip、nested scroll、focus/accessibility与current-source Cargo/产品trace通过。

## 禁止临时方案

- 不得只减少overscan而保留全部child position扫描。
- 不得让focus/accessibility私建另一份全量visible索引或直接丢失offscreen语义状态。

## 修复结果与回传

Open state: `等待EditorUI02回传indexed extent/range generation、edge delta更新与100k-row规模证据`。

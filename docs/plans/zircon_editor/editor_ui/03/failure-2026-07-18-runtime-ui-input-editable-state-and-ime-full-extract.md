---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-input-editable-state-and-ime-full-extract
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/input/editable_text.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/surface/input/text_state.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
tests:
  - atomic editable text state patch test
  - IME input zero-full-extract and indexed-layout test
  - 100k-character clone and prefix-visit counter test
---

# Runtime UI editable状态事务与IME同步全树extract

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/surface/input` editable/text/IME路径
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 联动责任：EditorUI08拥有render extract generation/index，Text09拥有layout/cache，Runtime11拥有worker budget。
- 交接原因：editable source/state、text layout与IME context由EditorUI03统一整合。

## 失败现象与复现证据

PERF-MVP-295/296：每次编辑从TOML重建并复制完整text/composition，再独立提交最多8个property mutation及多份binding/dirty/diagnostic；active IME的每个keyboard/text/preedit/commit同步刷新全树render extract、线性扫commands，并复制surrounding text。原layout lookup还深clone layout/style。

本轮已让semantic key/constraint token无分配匹配、filter借用TOML String，并让IME lookup借用当前extract layout/style。完整证据见`docs/plans/performance/01/2026-07-18-runtime-ui-surface-input-static-review.md`。

## 最低共享层根因

editable state仍以声明metadata多字段为runtime authority，缺少single patch transaction；text node也没有generation-owned layout index，输入handler只能强制运行render pipeline来获得IME geometry。

## 架构修复验收

- persistent editable state以shared source+ranges表达caret/selection/composition；一次事件只提交一个`TextStatePatch`，unchanged field不mutation、不binding、不dirty。
- change/submit与render/IME共享正文owner；caret-only正文clone=0，单edit最终正文ownership不超过必要一次。
- generation-owned node→text-layout handle近O(1)查询；任何input handler full render extract calls=0，stale generation显式fallback或排队更新。
- surrounding text/composition以shared source+ranges表达，在platform边界按需物化；长文本prefix work有索引或预算。
- 1/10k/100k chars、1/100/10k UI nodes和60/120 Hz preedit记录clone bytes、transactions、binding/dirty、extract calls、command/prefix visits与CPU p95；Unicode/filter/max/selection/composition/wrap/rich/vertical/platform IME、Cargo与像素通过。

## 禁止临时方案

- 不得增加另一个无generation失效的IME layout cache。
- 不得只合并property API但仍逐字段触发binding/dirty。
- 不得把同步全树extract移到另一个input callback或私有线程后继续阻塞结果。

## 修复结果与回传

Open state: `等待EditorUI03联动EditorUI08/Text09/Runtime11回传persistent editable state、atomic patch、indexed layout、zero-full-extract IME与规模/产品证据`。

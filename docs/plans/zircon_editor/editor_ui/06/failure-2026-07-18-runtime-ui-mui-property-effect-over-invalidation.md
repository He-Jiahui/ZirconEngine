---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-mui-property-effect-over-invalidation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/surface_dirty_mui.rs
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs
tests:
  - timer-only metadata zero-pipeline-visit test
  - 120-Hz transition dirty-domain and stage-visit matrix
  - nested slot/sx typed changed-field effect test
---

# Runtime UI MUI属性effect过度失效

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`surface_dirty_mui.rs`、metadata dirty mapping与rebuild消费定向复核
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 联动责任：EditorUI07声明transition effect，EditorUI08消费changed node/range stage delta。
- 交接原因：property到layout/style/text/input/timer effect的权威应来自component schema，不能由surface字符串白名单平行维护。

## 失败现象与复现证据

PERF-MVP-315：Snackbar `autoHideDuration`测试要求layout+hit+render+text+input全dirty，尽管timer按需读取值；所有feedback/customization属性共享粗粒度union。`transition_progress/animation_progress`每tick标hit+render+input，当前input dirty会全量重建arranged tree，形成60/120 Hz O(N)风险。

## 最低共享层根因

component/prop schema没有typed property-effect与changed-field delta，surface只能按String类别猜测最保守dirty union；animation、timer、style和geometry也没有共享effect contract。

## 架构修复验收

- compiled component descriptor声明timer/render/text/style/layout/hit/input/a11y effect及alias canonical identity。
- nested slot/sx patch按实际changed typed fields求一次dirty union；unchanged写入不dirty。
- timer-only更新pipeline visits=0且timer generation恰更新一次；Fade只render，geometry transition只patchaffected range。
- 1/100/10k nodes、60/120 Hz记录各stage visits、dirty commits、timer reschedule、allocation与CPU p95；Cargo、产品toast/transition trace和像素/命中通过。

## 禁止临时方案

- 不得给所有feedback/transition统一改成render-only；Grow/Slide/Zoom必须保留几何和hit语义。
- 不得让surface与component reducer各维护一份String property effect表。

## 修复结果与回传

Open state: `等待EditorUI06回传typed property-effect schema与精确dirty union，EditorUI07/08回传高频transition stage证据`。

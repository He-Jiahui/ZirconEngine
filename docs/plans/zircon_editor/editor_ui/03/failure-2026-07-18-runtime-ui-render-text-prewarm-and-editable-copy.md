---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-render-text-prewarm-and-editable-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/text
tests:
  - stable text generation zero-shape-layout test
  - 100k-character editable ownership test
  - unified prewarm worker budget and stale cancel test
---

# Runtime UI render文本多轮prewarm与editable正文复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：render text prewarm/measure/field/extract
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 联动责任：Text09拥有cache/task ownership，Runtime11拥有worker budget。
- 交接原因：editable text state、UI text layout与render text artifact由EditorUI03整合。

## 失败现象与复现证据

PERF-MVP-292：owner prewarm完整resolve visual，extract重复resolve，command后再prewarm/missing-layout；私有固定2线程池绕过统一budget。TextField clone base style、composition、command text与editable，focus layout再复制完整editable。

## 最低共享层根因

source/style/edit-state/layout没有generation-owned共享handle，prewarm与consumer各自要求owned DTO。

## 架构修复验收

- source/style/prewarm/extract/render共享compiled text/layout identity，stable generation shape/layout=0。
- editable正文Arc共享，selection/composition用range，单edit正文ownership不超过必要最终一次。
- worker纳入Runtime11统一budget/cancel/age，stale generation不发布。
- 1/100/10k nodes、1/10k/100k chars记录clone bytes、passes、queue depth/age与UI CPU p95；IME/rich/vertical/fallback/Cargo/像素通过。

## 禁止临时方案

- 不得增加第三份text cache或继续用private fixed thread pool。
- 不得只减少一个clone而保留同帧多轮shape/layout。

## 修复结果与回传

Open state: `等待EditorUI03联动Text09/Runtime11回传compiled text handle、single-pass layout与worker证据`。

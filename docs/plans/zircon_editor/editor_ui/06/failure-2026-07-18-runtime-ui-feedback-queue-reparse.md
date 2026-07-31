---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-feedback-queue-reparse
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/component/state_reducer/notification_center.rs
  - zircon_runtime/src/ui/component/state_reducer/toast.rs
  - zircon_runtime/src/ui/surface/input/toast_timer.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
tests:
  - 10k notification navigation and mark-read scale test
  - bounded toast storm timeout fairness test
  - stale timeout generation rejection test
---

# Runtime UI feedback队列全量重解析且无硬容量

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：notification/toast reducer与timer接缝
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 联动责任：EditorUI01提供统一deadline queue与frame dispatch budget。
- 交接原因：NotificationCenter/Snackbar的数据模型、current/expired语义和容量策略由EditorUI06组件库拥有。

## 失败现象与复现证据

PERF-MVP-269：NotificationCenter每键/选择多次解析全部entry并重算unread，mark-read递归全值；Toast timeout深cloneraw map、重建余下queue后再次parse。toast queue未定义entry/byte hard cap。

## 最低共享层根因

feedback state只保存通用数组，没有id index、unread aggregate、deque node或timer generation；所有查询和删除都必须重新materialize完整队列。

## 架构修复验收

- generation-owned model持有id→entry/index与unread aggregate，select/read近O(1)，row delta更新aggregate。
- toast采用有界deque/id index，timeout pop/remove不重parse全queue；count+bytes双hard cap并记录drop/coalesce/age。
- timer携带toast id+generation，stale timeout丢弃；同帧到期遵守EditorUI01 count/time budget与age fairness。
- 1/100/10k entries连续10k nav/timeouts记录parse/raw clone、visits、queue bytes/age与CPU p95；popup/current/expired/action/read/disabled及Cargo通过。

## 禁止临时方案

- 不得仅把Vec换成VecDeque而继续每次生成owned `ToastEntry`全副本。
- 不得无声丢通知；容量策略和丢弃/coalesce结果必须可诊断。

## 修复结果与回传

Open state: `等待EditorUI06联动EditorUI01回传indexed feedback model、bounded queue、timer generation及规模证据`。

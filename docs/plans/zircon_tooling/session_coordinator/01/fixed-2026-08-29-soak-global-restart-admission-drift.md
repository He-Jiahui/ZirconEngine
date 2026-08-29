---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: soak-global-restart-admission-drift
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/soak.py
  - tools/session_coordinator/tests/test_soak.py
tests:
  - python -u -B -m unittest tools.session_coordinator.tests.test_soak.SoakTests.test_short_fixture_soak_rolls_over_and_preserves_events -v
  - python -u -B -m unittest tools.session_coordinator.tests.test_soak -v
resolved_at: 2026-08-29
---

# Coordinator01 soak global restart admission drift

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：unoccupied failure-chain sweep for long-running service lifecycle validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns both the deterministic soak harness and the closed service lifecycle action catalog.

## 失败现象与复现证据

`test_short_fixture_soak_restarts_and_preserves_events` failed both in the combined
sessions/rollout/soak sweep and as a focused reproduction. The soak stopped after
about two seconds with `lifecycle_global_shutdown_disabled`; the shorter duration,
single sample, missing restart, unchanged event cursor, and one-instance diagnostics
were all downstream consequences of that primary rejection.

## 最低共享层根因

The soak helper still previewed `service.restart`. Production lifecycle policy now
keeps global stop/restart/force-stop disabled while task admission is open. The
supported code-reload path is `service.rollover`, which preserves admission and
unstarted work while binding predecessor shutdown and successor completion to the
same durable action.

## 架构修复验收

- Use only the closed `service.rollover` action for the soak's single daemon transition.
- Preserve the existing durable action confirmation, awaiting-restart handoff, exact
  successor identity, event continuity, and per-instance resource sampling checks.
- Keep the report schema's `restart_count` compatibility field and require exactly one
  predecessor-to-successor transition.
- Re-run the focused four-second fixture and the complete soak module.

## 禁止临时方案

- Do not enable global shutdown in production or in the fixture merely to admit the
  obsolete action.
- Do not bypass the controlled action service with direct process termination or an
  unjournaled local restart.
- Do not relax duration, sample, event-continuity, or instance-transition acceptance.

## 修复结果与回传

- 根因：The deterministic soak still requested globally disabled service.restart, so production lifecycle admission rejected the harness before it could observe a successor or preserve event continuity.
- 架构修复：The soak now uses the closed service.rollover action, verifies durable awaiting-restart handoff and exact successor identity, and applies a bounded cross-instance evidence window while keeping same-instance cadence strict.
- 验证：Source commit 1f49b1c3bb980a776746f96f6b67c3061924d3c8; fresh transition and rollover-gap regressions passed 2/2; focused real fixture passed; complete soak module passed 10/10 twice (17.171s and 37.898s).
- 回传：Coordinator01 soak now validates the supported admission-preserving rollover lifecycle with exact two-instance sampling, durable action identity, and event continuity.

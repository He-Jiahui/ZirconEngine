---
handoff_kind: failure
status: open
created_at: 2026-08-29
summary_slug: control-health-p95-connection-churn
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_control_load.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_control_load.ControlLoadTests.test_z_health_and_action_preview_p95_targets -v
  - python -m unittest tools.session_coordinator.tests.test_control_load.ControlLoadTests.test_reused_health_connection_observes_fresh_database_state -v
---

# Coordinator01: control health P95 regresses through connection churn

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：unoccupied failure discovery through the M6 control-load regression
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the loopback health transport and the authoritative health projection.

## 失败现象与复现证据

On Windows, the isolated M6 health test repeatedly exceeds its `<100 ms` P95 contract: `221.6 ms` under the initial suite and `138.2 ms` alone. The action-preview half remains within contract. Direct profiling shows server-side health at `208.0 ms` P95 and HTTP identity at `81.9 ms`, so this is not action-service load.

## 最低共享层根因

Every `CoordinatorClient.health()` call creates a new HTTP connection because the server responds as HTTP/1.0 and the client uses one-shot `urlopen`. Each request also opens separate SQLite connections for baseline and supervision, repeating connection setup and producing non-atomic health fields. The connection churn dominates the bounded loopback endpoint.

## 架构修复验收

- One client serializes and reuses its authenticated health connection; transport failures discard it and fall back to the existing bounded read-only request behavior.
- The server supports HTTP/1.1 keep-alive with explicit response lengths.
- Baseline and supervision are projected from one fresh read transaction; no health result is time-cached.
- The isolated M6 health P95 is below `100 ms`, and a reused connection observes an intervening database state change.

## 禁止临时方案

- Do not raise the M6 latency threshold or reduce its sample count.
- Do not cache health fields, weaken blocker freshness, or omit supervision data.
- Do not share an unsynchronized HTTP connection across client threads.

## 修复结果与回传

Source-fixed state: `Coordinator validation and lifecycle return pending`.

- 根因：loopback health 的 HTTP 与 SQLite 连接均按调用反复建立，造成 Windows P95 抖动并切裂 baseline/supervision 读视图。
- 架构修复：客户端用锁保护的 health keep-alive transport，服务端启用 HTTP/1.1，并在一个 deferred read transaction 内投影 baseline 与 supervision。
- 验证：focused P95 从孤立 RED `138.2 ms` 降到 `22.3 ms`，完整 load suite `9/9` 再测为 `12.3 ms`；freshness、client `20/20`、server health smoke `3/3`、baseline `2/2`、真实 HTTP/1.1 framing、`py_compile` 与 scoped diff gate 均通过。
- 回传：待 Coordinator exact-snapshot finalizer 与 canonical `failure return` 完成后关闭。

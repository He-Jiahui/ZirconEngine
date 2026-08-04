---
handoff_kind: fixed
status: fixed
created_at: 2026-07-26
summary_slug: validation-copy-cargo-async-materialization-request-block
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_database.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_database
  - validation_copy.materialize_cargo bounded durable acknowledgement and restart recovery regression
resolved_at: 2026-08-04
---


# Coordinator01: validation-copy Cargo async materialization request block

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / IMPLEMENTATION INTEGRATED / MANAGED RETURN PENDING` | 2026-08-03 | 当前 production 已在 request thread 仅持久化 bounded accepted job metadata，closure planning、disk/Git probe、artifact governance、archive/overlay/hash 均由 worker 执行；startup recovery 对同一 job exactly-once claim，malformed durable payload terminalize 为 typed failure。当前源码精确 8-test 回归 8/8 通过（18.656s），覆盖 ack-before-work、durable-before-probe、bounded no-manifest response、governance 不阻塞 ACK、unowned/external descriptor typed failure、pinned-baseline drift、restart recovery 与 request decode terminalization。生产/既有测试属于先前冻结 validation-copy 切片，本轮未抢改；待 managed receipt、Plugins01 origin replay 与 failure return，不声明 fixed/commit。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：Plugins01 recovery validation-copy materialization
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Cargo closure planning、target-root probing、external/overlay validation、Git archive/hash materialization and HTTP command response shaping all execute in the Coordinator01 control plane. Plugins01 may preserve its immutable materialized job but must not retry, clean up, or modify coordinator source to bypass a control-plane timeout.

## 失败现象与复现证据

Plugins01 request `9733722576484736a7ce58ce9ae9d15a` for `validation_copy.materialize_cargo` was accepted at `2026-07-26T12:58:29Z` and completed at `13:14:29Z`. Its materialized job `5945e3ef29d74bd69602adca02e243b5` has immutable input manifest `595c5d5f02c80dc63e5e289ad2bf3709e4f67639f5bccb272e7efd1e32f9ee27`, no validation-copy run, and no Plugins01 Cargo start.

The command response was about 1.68 MiB and journaled as digest `d2e57c9c…`; while it ran, wrapper-safe `validation-copy status` returned typed `command_preflight_timeout`. Source audit proved the request thread synchronously performed Cargo closure planning plus baseline/archive/overlay/external extraction and full input hashing before returning `record.to_dict()`. The raw ledger phase could remain `planned` even though the status mapper correctly recognized a started materialization as `materializing`.

## 最低共享层根因

`WorkspaceCopyService.materialize_cargo()` was synchronous, and the server dispatch path called it directly before returning a full manifest. In addition, the pre-ack path could probe disk/path/Git, normalize overlays, and invoke recursive artifact governance. A malformed persisted Cargo request could roll back its worker claim and remain indefinitely recoverable instead of becoming terminal.

## 架构修复验收

- `validation_copy.materialize_cargo` persists a bounded durable `accepted` job before any disk probe, path normalization, Git pin, closure plan, artifact scan, archive, overlay, external descriptor, or full hash work.
- A claimed worker alone performs root preparation, governance/ownership checks, closure planning, materialization, and phase/error persistence; restart recovery resumes the same job id exactly once.
- The command response contains durable job metadata only, never the full input manifest; malformed accepted payloads terminalize as typed `request_decode` failures.
- Focused and full Coordinator01 validation must pass, then the origin may inspect the durable result without retrying or mutating its already materialized copy.

## 禁止临时方案

- 不得重试、清理或重建 Plugins01 job `5945e3ef29d74bd69602adca02e243b5`。
- 不得在 HTTP 接收线程保留同步 closure/archive/hash 工作、提高 timeout、截断后仍返回完整 manifest，或以调用方轮询掩盖控制面阻塞。
- 不得由 Plugins01 修改 Coordinator01 源码、直接写 SQLite，或以共享工作树 Cargo 替代 immutable validation copy。

## 修复结果与回传

- 根因：Cargo validation-copy materialization executed closure planning, disk and Git probes, artifact governance, archive and hashing synchronously on the command request thread, so durable acknowledgement was delayed behind unbounded work; malformed persisted requests could also remain recoverable instead of becoming terminal.
- 架构修复：The current coordinator persists a bounded accepted job record before all probes and heavy work, returns job metadata without a full manifest, lets a worker claim and materialize the job exactly once, recovers accepted work after restart, and terminalizes malformed durable requests with typed request_decode evidence.
- 验证：Eight exact local regressions passed in 11.189s. Managed ticket 7d91906cc854499bb3d2fec594915182, source manifest b29de4cf430182e844c8d53b249edff5b3fc0ea6c92e049f94520636dfae15dc, copy job 8f8f3e6fc4fe451486eaec20b1086100 passed 8/8 in 13.817s with exit code 0. Handoff graph before return validated 561 artifacts with 0 errors.
- 回传：The preserved Plugins01 job 5945e3ef29d74bd69602adca02e243b5 remains materialized with immutable input manifest 595c5d5f02c80dc63e5e289ad2bf3709e4f67639f5bccb272e7efd1e32f9ee27 and no validation-copy run, exactly as required; it was inspected without retry or cleanup. Its source session is archived, so no historical Cargo run was fabricated.

---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: welcome-project-probe-admission-budget
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/jobs/limits.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
tests:
  - EditorJobSystem typed admission accepted/merged/backpressured contract
  - 1/1000/1000000 request entry/bytes/oldest-age/RSS admission storm
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
---

# Editor14：Welcome project probe 准入预算交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：`failure-2026-07-22-welcome-project-probe-admission-storm`
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor10 拥有草稿语义与 `ProjectAuthority` 调用边界；队列的 entry/bytes/oldest-age 预算、合并结果和调度指标属于 Editor14 的 `EditorJobSystem` 统一准入契约。
- 生命周期键：`welcome-project-probe-admission-budget`

## 失败现象与复现证据

当前 `WelcomeProjectProbeState` 已能在 host 内按草稿 generation 进行防抖、到期提交、同目标复用与取消；但提交使用普通 `EditorJobSpec { category: Index, priority: Background, cancel }`。`EditorJobSpec` 没有 admission key、payload-byte 或 deadline 信息，`EditorJobLimits` 对 `Index` 默认无上限。快速输入因此仍可被其他调用路径在调度层无界排队，且没有 accepted/merged/backpressured、队列 bytes、oldest age 与 RSS 的统一证据。

该结论来自 Editor10 独立源码复审：Welcome 层不得自己建立线程池、私有预算或第二个 job truth；这样会使 asset import、script build 与 welcome probe 的背压语义再次分裂。

## 最低共享层根因

`EditorJobSystem` 只表达类别、优先级、互斥、依赖和取消，没有让业务调用者声明 typed coalescing key、请求 payload 成本与最大可等待年龄的准入协议，也没有返回可审计的 admission outcome。既有 `pending` 索引只能选择已入队任务，不能对无界提交本身实施 entry/byte/age 背压。

## 架构修复验收

- Editor14 在 `EditorJobSpec`/submission 边界发布 typed admission request 与 result：调用者声明语义 key、payload bytes 与最大 age；结果显式为 accepted、merged 或 backpressured，且不丢弃已开始或终态事件。
- 预算由 `EditorJobSystem` 作为唯一 owner 按类别/全局统计 entry、queued bytes、oldest age、merged、cancelled 与 started；上层读取同一份观测，不建立 Welcome 私有队列或计数器事实源。
- 同 key 的未开始请求在 I/O 前合并为 latest generation；正在执行请求保持协作式取消，并由 job 内检查点在下一段 I/O 前停止。Editor10 仅提供 draft generation/key 与 `ProjectAuthority` probe 实现。
- 背压 contract 覆盖 1、1,000、1,000,000 请求以及 32B/4KiB payload、1ms/1s probe；固定 entry/bytes/oldest-age/RSS 与 UI pump p95 门，并保留 missing/linked/invalid/current generation/submit failure/shutdown 语义。
- 先通过 Editor14 focused scheduling/background-storm tests，再向上复跑 Editor10 Welcome probe tests、current-source Cargo 与 F0 产品 trace。

## 禁止临时方案

- Do not add a Welcome-private worker pool, queue, timer thread, budget counter, or separate scheduler truth.
- Do not satisfy bytes/age budgets with a fixed debounce, category concurrency cap, token cancellation alone, or a test-only smaller pool.
- Do not drop terminal/cancel/error events, relax the 1/1k/1M evidence, or use aliases, compatibility shims, silent fallbacks, duplicated truth, or call-site exceptions.

## 修复结果与回传

Open state: `待修复`; Editor10 retains its local debounce and cancellation work but must not return `welcome-project-probe-admission-storm` as fixed until this shared admission contract is returned and the upward current-source/product gates pass.

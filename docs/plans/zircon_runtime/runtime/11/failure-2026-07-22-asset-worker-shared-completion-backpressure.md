---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-worker-shared-completion-backpressure
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/worker_pool
  - zircon_runtime/src/core/runtime/tasks
tests:
  - cargo test -p zircon_runtime --lib asset::tests::pipeline::worker_pool --locked --jobs 1 -- --nocapture --test-threads=1
  - unique request, duplicate waiter, large payload, stalled consumer, cancel and shutdown matrices
---

# Runtime11：asset worker共享完成结果与背压缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset pipeline逐Rust文件性能审查，PERF-MVP-498
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：共享终态、结果所有权、完成队列背压与cancel/shutdown属于统一任务模型，不能由asset worker再建立一套私有无界完成协议。
- 生命周期键：`asset-worker-shared-completion-backpressure`

## 失败现象与复现证据

产品manager的唯一请求队列已按worker数有界，request诊断也已从每次全扫waiters改为O(1)。但同一AssetId的duplicate waiter不消耗唯一队列capacity，completion channel仍无界；任务完成时按waiter数量逐次`payload.clone()`，Texture/Mesh内部Vec会被深拷贝N次。慢consumer与256MiB payload组合可让一次single-flight decode变成N份常驻结果；owner Drop还会同步等待所有pending jobs。

## 最低共享层根因

worker pool把观察者数量编码成payload消息数量，没有Runtime11统一的共享immutable result owner、observer ticket、completion bytes/age预算和可取消shutdown终态。

## 架构修复验收

- 每个single-flight job只发布一个`Arc`结果/终态；N个waiter只持ticket/cursor，不按观察者复制payload。
- unique request、duplicate waiter和unharvested completion分别有entry/bytes/age硬预算，拒绝/合并/过期原因可观测。
- cancel/deadline/shutdown覆盖queued、running和completed-unharvested；owner-thread Drop不无限同步等待。
- unique/waiters 1/1k/100k、payload 4KiB/256MiB、workers 1/8/64、stall 0/1/60s记录queue age、clone bytes、RSS、cancel/drop wall；同一结果payload owner=1且内存硬有界。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止只把completion channel改为bounded后让worker阻塞在大payload发送。
- 禁止让每个AssetId或consumer私建线程、队列或结果缓存。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

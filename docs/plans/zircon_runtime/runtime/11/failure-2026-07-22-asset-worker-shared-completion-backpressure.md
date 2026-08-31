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

### 当前源码复核（2026-08-25）

- `AssetWorkerPool` 已在 Runtime11 的 IO `TaskPool` 上实现 single-flight：同一请求的多个观察者只取得 `AssetWorkerCompletionTicket`，终态只保有一个 `Arc<CpuAssetPayload>`，不会按 waiter 深拷贝 payload。
- `CompletionRegistry` 对唯一任务、live waiter、已完成条目、已完成字节数以及 request/completion age 分别施加预算；超限、过期、取消和关闭均以可观察终态唤醒 ticket。worker 不通过完成 channel 发送大 payload，也不创建私有线程或队列。
- `Drop` 先切换关闭状态并取消未完成/未收割条目，不等待 IO worker；对应的 source tests 已覆盖共享 owner、1/1k/100k waiter 上限、完成条目/字节预算、过期、取消和非阻塞 Drop。
- 此结构与 Unreal `FStreamableHandle` 的共享句柄及受管活动请求模型、Bevy `Arc<StrongHandle>` 的共享资产寿命模型一致；Zircon 仍保持 Runtime11 统一任务池和 timer 的所有权边界。

### 仍待受管验收

- 尚未运行本记录列出的 Cargo 矩阵，不能将 source tests 记为通过。
- 尚未采集 unique/waiters `1/1k/100k`、payload `4KiB/256MiB`、workers `1/8/64`、stall `0/1/60s` 的 queue age、clone bytes、RSS、cancel/drop wall 数据，也没有功耗或与参考引擎的耗时对比；在这些数据出现前，不开展新的性能微优化或宣称瓶颈已消失。

Open state: `源码实现已覆盖共享结果与背压设计；受管验证和 Performance01 量化采样待完成`; no pass is claimed.

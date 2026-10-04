# 2026-07-17 MVP editor jobs / messaging 静态审查

## 范围与状态

- 已逐文件读取 `zircon_editor/src/core/jobs` 19 个生产 Rust 文件；7 个测试/支持文件已建立清单，并重点读取 1,000-job storm、pump、progress、scheduling 与 thread-ownership 合同。
- 已逐文件读取 `zircon_editor/src/core/editor_message` 25 个生产 Rust 文件。
- 当前仅完成静态审查和状态栏单项低风险修复；Cargo、当前源码 editor trace、队列压力与全部测试验收仍待完成，两个目录继续留在 `pending.md`。

## 已有正确边界

- Editor jobs 复用 Runtime `JobScheduler`，生产路径没有再创建私有线程池。
- Thumbnail/Export 有类别并发上限，mutex group 和显式 dependency 进入 scheduler handle；panic/cancel/shutdown 有测试合同。
- terminal dependency history 默认保留 256 项，仍被 pending dependency 引用时允许暂时超过上限，取消/调度后再收敛。
- worker 只写 job-event MPSC，Editor message bus 发布集中在 retained-host 主线程 tick。

## 已确认问题

### 状态栏每帧克隆全部 active jobs

`sync_editor_job_progress` 每 tick 调用 `progress().snapshot()`，深 clone 所有 active job 的 label/progress，随后只选择最小 JobId 的一项。已增加 `primary_snapshot()`，使状态栏只克隆首个非 terminal entry；完整 snapshot API 保留给任务面板。回归 `primary_snapshot_clones_only_the_smallest_visible_job` 已写入，待 Cargo 验证。

### job pump 无配额

job-event MPSC 是 unbounded；`pump_events()` 用 `try_recv` 循环一直 drain 到空，并对每个事件同步发布到 editor message bus。1,000-job storm 已记录 pump P50/P95/max，但测试明确写着 `numeric_budget=undefined`，所以它只证明功能与可重复测量，不证明主线程帧预算安全。

应同时测量 count budget 和 time budget；progress 高频事件可按 JobId 合并 latest value，但 Started/terminal edge 必须保序且不可丢。

### message inbox 无界且 fanout clone

`EditorMessageBus` 为每 subscriber 保存 `Vec<EditorMessageDelivery>`，没有容量、age、drop 或 stale-subscriber 指标。publish/broadcast 先收集 subscriber id 到新 `Vec`，再为每个 inbox clone topic/message；custom JSON 与 JobEvent 字符串会随 fanout 深拷贝。未 drain 的 subscriber 可长期增长。

### pending admission 接近 O(n²)

pending jobs 存于 `Vec`。每次 promote 全量扫描 admissible jobs并取最小 priority/id，再 `remove(index)`；每个完成任务又触发 promote。长队列下累计扫描/搬移接近 O(n²)，现有 storm 只输出 submit 时间，没有算法预算。

## 验收计划

1. 聚焦验证 primary snapshot、jobs、message bus 与 retained status 投影。
2. 运行 1,000/10,000 job 队列，分别测 submit、promotion、pump count/time、message fanout、RSS 与 queue age。
3. Editor14 定义每 tick job-event 配额与 progress 合并规则；Editor02 定义 inbox delivery/backpressure 与 shared payload 语义。
4. 当前源码编辑器空闲和 background storm 采集 WPR 主线程、worker、allocator、mutex 与 redraw 证据。


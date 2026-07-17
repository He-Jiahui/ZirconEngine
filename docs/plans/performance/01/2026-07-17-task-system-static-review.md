# 2026-07-17 MVP 任务系统静态审查

## 范围与状态

- 已逐文件读取 `zircon_runtime/src/core/runtime/tasks/**` 9 个生产 Rust 文件、`zircon_runtime/src/core/runtime/modules/**` 6 个运行时模块文件，以及 `zircon_runtime/src/tests/tasks.rs`。
- 静态审查完成；聚焦 Cargo 测试、当前源码产品 trace 和压力测试尚未完成，因此这些目录仍留在 `pending.md`。
- 对照实现读取 `dev/bevy/crates/bevy_app/src/task_pool_plugin.rs` 与 `dev/bevy/crates/bevy_tasks/src/**`。Zircon 的进程级 `OnceLock<TaskPools>` 和低核线程分配与 Bevy 的默认任务池策略一致。

## 已确认事实

### 进程级执行 owner

`TaskPools::default()` 复用同一组 compute、async-compute 与 IO pool，常规运行时不会因每个实例重新创建三组线程。`TaskPoolOptions::create_pools()` 仍可显式创建隔离 owner，现有测试覆盖共享与隔离语义。

在可用并行度不超过 2 时，各类 pool 的最小线程数会让实际 worker 数大于 `total_threads`。Bevy 的参考实现明确采用同类策略，因此这里先记录为“有意 oversubscription”，必须用 WPR 上下文切换与真实产品吞吐证明有害后才能调整，不能只凭线程数量直接修改。

### detached panic 会污染完成率

`JobScheduler::spawn` 原先只在任务正常返回后调用 `record_completed()`；detached task panic 时 `tasks.scheduled - tasks.completed` 会永久增大。该差值被用作积压信号时，会把历史 panic 误报成持续队列堆积。

已增加析构守卫，使正常返回和 unwind 都记录完成，同时保持 Rayon 原有 panic 行为；回归测试 `detached_spawn_counts_panicked_tasks_as_completed` 已写入，待协调 Cargo lane 运行。

### 等待指标名超过实现能证明的语义

`JobHandle::wait()` 无条件累加 `tasks.main_thread_wait_ms`，但 handle 不保存创建线程或主线程身份，worker、后台线程和测试线程的调用也会进入同一计数。当前数值只能证明“显式 handle wait 总时长”，不能证明主线程等待。

Runtime 11/07 应先选择一种契约：显式传入/保存 main-thread identity 后只记录主线程，或将指标硬切换为 `tasks.explicit_wait_ms` 并迁移消费者。修改前增加 worker-side wait 回归，防止性能报告继续错误归因。

### 调度器缺少判定堆积所需指标

现有诊断只有 scheduled、completed、dependency wait 与所谓 main-thread wait；没有 queued、active、queue delay、execution duration、steal/yield 或取消/失败计数。因此 `scheduled-completed` 只能给出粗略未终结任务数，无法区分排队、正在执行、panic 和依赖等待。

Runtime 07/11 应增加低成本计数器和可选采样：

- 当前 queued / active / peak queued；
- enqueue-to-start p50/p95/p99 或直方图；
- task execution duration 与 pool kind；
- completed / panicked / cancelled 分项；
- worker assist/yield 次数与主线程显式等待。

这些指标必须支持关闭或采样，不能让诊断本身在高频微任务下成为锁或原子热点。

## 验收计划

1. 运行 detached panic 聚焦测试和 `zircon_runtime` task 测试集。
2. 用 1、2、逻辑核数三种配置提交短任务、长任务和依赖 fanout，记录吞吐、queue delay、上下文切换与 CPU 利用率。
3. 在当前源码 runtime/editor MVP 中采集 WPR 线程时间线，核对 pool 数、空闲唤醒与主线程 wait。
4. 动态证据通过且文档/责任计划回填后，才把任务目录从 `pending.md` 移入 `review.md`。


# Runtime11 VM Discovery Runtime Owner Hard Cut 架构与验证计划

> 日期：2026-08-26
> 状态：`source_implemented_static_passed_managed_validation_pending`
> 所属计划：`runtime/11-job-system-task-model.md`

## 1. 当前源码结论

`VmPluginManager::with_plugin_context_and_host_exports` 的正常产品路径已经从 live `CoreHandle`
取得 Runtime I/O `TaskPool`，再显式构造 `JobScheduler`。但是当前仍有一条隐藏旁路：

1. `with_builtin_backends` 调用 `detached_plugin_context`；
2. helper 创建局部 `CoreRuntime`，只把 `CoreWeak` 放进 context 后立即 drop runtime；
3. manager 构造时 weak upgrade 失败；
4. `unwrap_or_default` 创建 `VmPluginDiscoveryWorker::default`；
5. worker 静默抓取 `TaskPools::process_default` 和 `JobScheduler::process_io`。

因此 mock/unavailable manager 表面拥有一个 runtime context，实际 discovery work 越过该 runtime 的
生命周期，进入进程级永久 owner。外部传入已经失效的 `PluginContext` 也会触发同一静默 fallback。这是
task owner correctness 问题，不是微观性能问题。

## 2. Unreal Engine 对照与本仓裁决

主要参考 Unreal Engine
`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h`：

- `FAsyncTaskBase::StartBackgroundTask` 接收明确的 `FQueuedThreadPool*`；
- task destructor 用 `CheckIdle` 拒绝“worker 仍可能执行而 owner 已销毁”的生命周期；
- `Cancel` 先尝试从同一 queued pool 撤回 work，不在 owner 缺失时偷偷切换到另一个 pool；
- pool shutdown abandon 与调用者 cancel 是不同路径。

Zircon 不复制 UE 的裸指针或隐式全局默认值。现有 Runtime11 已有 injected `TaskPool`、bounded keyed
lane、cancel authority 与 terminal ticket，因此本切片只修复 owner acquisition：live Runtime 才能构造
可执行 worker，缺失 owner 时返回 typed unavailable。

## 3. 目标所有权结构

`VmPluginDiscoveryWorker` 硬切为两个内部 backend：

- `Runtime { lane, io_pool, runtime_owner }`：只能由 `with_runtime(&CoreHandle)` 构造；
- `Unavailable`：不创建 pool、scheduler、thread 或 lane，所有 submit 立即返回 `VmError::Operation`。

删除 worker 的 `new`、`Default` 及 `TaskPools::process_default`/`JobScheduler::process_io` 引用，不保留
alias 或兼容入口。scheduler 只能从该 `CoreHandle` 的 I/O pool 派生，不能再由 caller 传入第二个可能
不一致的执行 owner。Runtime backend 保存 `CoreWeak`；每次 submit 在 generation/admission 前 upgrade，
以短生命周期 `CoreHandle` 保证 admission 线性化期间 owner 存活，activate 后立即释放。该 handle 不捕获
进 queued closure，避免 `runtime services -> manager -> lane work -> runtime` 强引用环；跨 task code lease
仍由统一 ExecutionScope 后续收敛。

`VmPluginManager::with_builtin_backends` 改为一次创建 `(CoreRuntime, PluginContext)`，manager 保存
`Option<CoreRuntime>` owner。字段声明顺序保证 `discovery_worker` 先 drop/drain，owned runtime 后 drop；
context 仍只持 weak，避免 runtime/service 强引用环。

外部 `with_plugin_context*` 不接管调用者 runtime：构造时能 upgrade 就注入该 runtime 的 I/O owner；
不能 upgrade 就安装 `Unavailable` worker。manager 的 backend/reflection 能力仍可使用，但任何 package
discovery 都收到明确错误，不再跨生命周期执行。

## 4. 复杂度、资源与性能假设

| 路径 | 修改前 | 修改后 |
|---|---:|---:|
| live context owner lookup | weak upgrade `O(1)` | 不变 |
| detached manager runtime | 创建后立即 drop | manager lifetime 内持有一个 explicit runtime |
| stale context submit | 静默进入 process pool | `O(1)` typed rejection |
| runtime owner gate | 构造期一次 weak upgrade | 构造期 + 每次 submit 一次 `O(1)` weak upgrade |
| discovery admission/执行 | bounded keyed lane | 不变 |
| process-global discovery fallback | 1 条 API 路径 | 0 |

detached manager 现在真实承担它创建的 runtime worker 生命周期，线程不再转嫁给 process singleton；这是
资源归属修复，不能仅凭源码推断总线程数、CPU 或功耗下降。后续性能验收必须分别测 live product、
detached mock 和 stale-context 三类构造的 runtime count、worker inventory、submit latency、queue age、
shutdown wall、线程基线恢复、CPU、RSS 与功耗。若 mock suite 大量并发创建 manager，再根据实测决定是否
由 test harness 显式共享一个 execution owner，不能恢复进程级 fallback 隐藏成本。

## 5. 确定性验证计划

先添加三个不依赖 sleep 或 filesystem I/O 的回归：

- `with_builtin_backends` 返回后，其 `PluginContext::core` 必须仍能 upgrade，证明 detached runtime owner
  没有在 helper 返回时销毁；
- 人工构造 context 后 drop 原 runtime，再创建 manager；`submit_package_discovery` 必须立即返回包含
  runtime-owner unavailable 的 typed error，不能创建或使用 process pool。
- 用 live external context 构造 manager 后再 drop runtime；后续 submit 同样必须在 generation/admission
  前 typed 拒绝，证明 worker 的 pool clone 不是继续接纳工作的 owner authority。

静态守卫同时断言 VM discovery I/O production 源码不再含 `TaskPools::process_default`、
`JobScheduler::process_io`、worker `new`、`Default` 或 raw `with_io_pool`。执行 scoped
`rustfmt --check`、源码状态机断言、
owned trailing whitespace 和 diff check。受管 Cargo、thread census 与性能/功耗采样保持 pending；没有
匹配哈希的回执前不提交 milestone、不发送手工企微。

## 6. 本切片完成定义

- package discovery 只有 injected runtime lane 或 explicit unavailable 两种状态；
- detached manager 保存 runtime owner，worker shutdown 顺序早于 runtime drop；
- stale external context 不再触发 process-global fallback；
- 三项行为回归与静态 no-fallback 守卫已挂载；
- 状态记录区分 source/static 与 managed/performance evidence。

## 7. 2026-08-26 源码验证结果

- detached builtin manager 已保存 `CoreRuntime` owner；字段顺序保证 discovery worker 在 owned runtime
  之前 drop；
- VM discovery worker 已硬切为 `Runtime / Unavailable` backend，唯一 runtime 构造接收
  `&CoreHandle`，scheduler 从同一 I/O pool 派生，submit 在 generation/admission 前检查 `CoreWeak`；
- `new`、`Default`、raw `with_io_pool`、`TaskPools::process_default` 与
  `JobScheduler::process_io` production 入口均为 0；
- 3 项确定性行为回归与 1 项 no-fallback source guard 已挂载；
- scoped `rustfmt --check`：2/2 Rust 文件通过；
- owner/backend/admission/drop-order 源码断言：19/19 通过；
- 3 个 owned 路径 trailing whitespace：0；tracked owned diff check 通过，仅有 Git 的 LF/CRLF
  工作区提示；
- 文件规模：discovery I/O owner 298 行，manager 675 行，均低于 800 行 production soft limit；
- 受管 Cargo、runtime/thread census、shutdown wall、CPU、RSS 与功耗样本：0，保持 pending。

本切片把 VM package discovery 的 process-global fallback API 数从 1 收敛为 0，并把 stale owner 的
行为从静默执行改为 typed rejection；它没有关闭 Runtime11 的统一 ExecutionScope/code lease、worker join
或全局线程预算 P0。

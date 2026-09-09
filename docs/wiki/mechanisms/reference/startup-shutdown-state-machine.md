---
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation
  - zircon_app/src/entry
implementation_files:
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/core/runtime/lifecycle.rs
plan_sources:
  - docs/wiki/core-runtime/runtime-and-module-lifecycle.md
tests:
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_app/src/entry/tests/runtime_entry_source_guards
doc_type: mechanism-case-study
---

# 从启动到停机：CoreRuntime 状态机案例

本案例描述一个编辑器或游戏宿主从进程入口到模块清理的完整路径。重点不是列出函数，而是说明每个状态拥有的数据、允许的动作、终止条件，以及发生失败后如何恢复到可观察状态。

## 场景

宿主读取 profile，创建 `CoreRuntime`，注册 builtin 与插件模块，等待 renderer 和 asset manager ready，然后每帧驱动时间、世界和渲染。用户关闭窗口时，宿主必须停止新请求、排空 task graph、按反向依赖关闭模块，并在预算耗尽时留下可诊断的 stopping 状态。

```mermaid
stateDiagram-v2
    [*] --> Constructing
    Constructing --> Registered: CoreRuntime::try_new + register_module
    Registered --> Activating: activate_registered_modules
    Activating --> Ready: build + ready all modules
    Activating --> Failed: dependency/error/timeout
    Ready --> Running: first frame admitted
    Running --> Quiescing: close request
    Quiescing --> Draining: close task admission
    Draining --> Cleaning: services drained
    Draining --> Stopping: deadline exceeded
    Cleaning --> Stopped: reverse cleanup complete
    Failed --> [*]
    Stopped --> [*]
```

## 状态与拥有者

| 状态 | 权威拥有者 | 可执行动作 | 禁止动作 |
| --- | --- | --- | --- |
| `Constructing` | `CoreRuntimeInner` | 创建 clock、random、task graph | 解析 manager |
| `Registered` | module registry | 添加 descriptor、校验依赖 | 运行生命周期回调 |
| `Activating` | activation coordinator | build/ready、建立服务 | 对外承诺服务可用 |
| `Ready` | registry + service entries | 打开 admission、resolve | 修改冻结 descriptor |
| `Running` | host frame loop | submit frame/task/event | 绕过 owner 直接写共享状态 |
| `Quiescing` | task graph + module registry | 关闭新 admission | 新建长期任务 |
| `Draining` | `ServiceCallGuard` census | 等待调用释放 | 卸载仍有 guard 的服务 |
| `Cleaning` | `ModuleLifecycle::cleanup` | 释放资源、撤销 watcher | 再次 publish 新服务 |
| `Stopped` | runtime handle | 读取最终诊断 | 重新激活旧 generation |

## 启动调用顺序

```rust
use std::time::Duration;
use zircon_runtime::core::{CoreRuntime, ModuleDescriptor};

fn boot() -> Result<CoreRuntime, Box<dyn std::error::Error>> {
    let runtime = CoreRuntime::try_new()?;
    runtime.register_module(ModuleDescriptor::new("Foundation", "clock and diagnostics"))?;
    runtime.register_module(ModuleDescriptor::new("Gameplay", "world systems"))?;
    runtime.activate_registered_modules_with_ready_timeout(Duration::from_secs(5))?;
    Ok(runtime)
}
```

这段代码展示公开入口和错误传播。真实产品应使用 profile 组装的 descriptor 集合，不能手工遗漏依赖模块。`register_module` 完成前不应启动 frame loop；`activate_registered_modules_with_ready_timeout` 返回成功只表示 ready 合同满足，不代表首帧已呈现。

## 首帧门控

首帧前需要检查三类条件：

1. 模块 activation report 中没有缺失依赖、重复服务或 ready timeout。
2. renderer 已获得 viewport/surface 的 owner 绑定，asset facade 至少可查询根状态。
3. task graph scope 已创建且 admission 计数为零或符合启动任务白名单。

宿主应将这些条件合成一个 `StartupReceipt`（产品层 DTO），并在日志中记录 runtime identity、profile、模块顺序和预算。不要用“窗口已创建”替代 runtime ready。

## 停机顺序

```rust
fn shutdown(runtime: &CoreRuntime) -> Result<(), Box<dyn std::error::Error>> {
    runtime.shutdown_task_graph(Duration::from_secs(2))?;
    runtime.shutdown_registered_modules_with_drain_timeout(Duration::from_secs(5))?;
    Ok(())
}
```

产品入口通常会先停止窗口事件和 frame demand，再关闭 task graph；上例仅表示公开 API 形状。模块 shutdown 内部按活动拓扑的逆序执行：先关闭上层 dependent，再关闭被依赖的 foundation。每次 deactivation 都先关闭 service admission，再等待 guard drain，最后调用 cleanup。

## 失败注入

| 注入点 | 可见错误 | 预期状态 | 恢复动作 |
| --- | --- | --- | --- |
| `try_new` worker 初始化失败 | `EngineTaskGraphInitError` | 无可用 runtime | 调整 worker budget 后重建 |
| descriptor 缺依赖 | `ModuleDependencyMissing` | 仍为 `Registered` | 修正 profile，重新构造 |
| `build` panic | 生命周期错误包装 | 激活事务回滚 | 隔离插件，保留错误摘要 |
| ready 超时 | `ModuleReadyTimeout` | 模块未对外 ready | 等待外部资源或放弃启动 |
| shutdown drain 超时 | `CoreError::ModuleCleanupTimeout` | `Stopping` | 继续 `*_until` 或强制进程级终止 |
| stale service handle | `CoreError::StaleServiceHandle` | 调用被拒绝 | 重新 resolve 当前 generation |

## 不变量

- descriptor 图在批量激活前冻结，激活失败不留下半激活服务。
- 生命周期回调不能在 registry 锁内执行，避免 callback 重入死锁。
- admission 关闭先于 cleanup；新调用永远不能在 cleanup 后进入。
- 每次重新激活生成新的 service identity；旧 handle 只能返回 stale。
- 关闭使用一个绝对 deadline，嵌套层不能把剩余预算重置成新的完整 timeout。
- 任何失败都必须包含阶段、模块名、generation 或 budget，便于自动化恢复。

## API 映射

| 机制概念 | Rust 接口 | 调用方责任 |
| --- | --- | --- |
| 创建 | `CoreRuntime::try_new` | 处理 task graph 初始化错误 |
| 注册 | `register_module` | 保证 descriptor 完整且唯一 |
| 激活 | `activate_registered_modules_with_ready_timeout` | 提供 ready 预算 |
| service admission | `resolve_manager_handle` + `ServiceHandle::enter` | 尽快释放 guard |
| frame 进入 | `tick_time` | 每帧只调用一次 clock authority |
| task 关闭 | `shutdown_task_graph` | 先关闭 scope admission |
| 模块关闭 | `shutdown_registered_modules_with_drain_timeout` | 处理 drain timeout |

## 性能预算

启动阶段应把 descriptor 图校验控制在几十毫秒级；ready 等待由外部资源预算决定，不应无限轮询。运行态每帧的状态转换和 admission 只允许常数级锁竞争；模块 cleanup 的预算必须按模块分解并记录剩余时间。停机日志应包含 `elapsed`, `remaining_budget`, `queued`, `running`, `failed`, `cancelled`。

推荐基线：首帧前图构建小于 100 ms；常规 frame admission 小于 1 ms；正常停机 2 s 内排空 task graph，5 s 内完成模块 cleanup。产品可调整，不应把这些数字写成协议常量。

## 生产检查清单

- [ ] profile 与 target mode 一致。
- [ ] 所有 builtin/plugin descriptor 在冻结前完成注册。
- [ ] activation report 已持久化到启动日志。
- [ ] ready timeout 不会吞掉具体模块名称。
- [ ] frame loop 在 `Ready` 之前不会提交渲染。
- [ ] 关闭流程先停止外部请求，再关闭 task scope。
- [ ] 所有 `ServiceHandle` 调用都有有限作用域。
- [ ] drain timeout 诊断列出未释放 owner。
- [ ] 重启路径不会复用旧 handle 或旧 generation。
- [ ] 集成测试覆盖失败回滚和部分停机。

## 参考与验证

- 源码：`zircon_runtime/src/core/runtime/runtime.rs`、`handle/activation`、`state`。
- 测试：`zircon_runtime/src/core/runtime/tests/activation/behavior/{activation,deactivation,reactivation}.rs`、`zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked`。
- 对照：Unreal 的 module manager 生命周期、Fyrox 的 editor/runtime 启动分层、Bevy 的 schedule startup/exit 阶段。
- 相关 Wiki：[模块激活与服务解析](../module-activation-and-service-resolution.md)、[产品生命周期](../../product-lifecycle.md)。

## 场景变体 A：编辑器冷启动

编辑器冷启动通常包含 project open、asset index、UI workbench 和 runtime preview 四组模块。先注册无窗口依赖的 Foundation、Project、Asset，再注册 Window、UI、Preview。窗口关闭只影响 editor host；preview runtime 可以先执行自己的 deactivation，再由 editor host 关闭共享模块。

推荐顺序：

1. 读取用户设置和 project path，拒绝空路径。
2. 组装 `RuntimeProfileId::Editor` 的 builtin module report。
3. 创建 `CoreRuntime`，注册 editor-only descriptor。
4. 激活并等待 asset index ready；UI 只在 workbench layout 可恢复后创建。
5. 首帧展示 health panel，而不是直接隐藏所有 provider 错误。
6. 关闭时先停止 preview frame demand，再保存布局和 dirty documents。

编辑器应该将 `ModuleReadyTimeout` 映射为面板中的 provider 状态，允许用户重试单个模块；不要因为一个可选 animation toolkit 超时而退出整个 editor。

## 场景变体 B：无窗口服务器

server profile 不注册 winit、surface 或 UI module。`CoreRuntime::try_with_task_graph_options` 仍创建 task graph，但 worker budget 由 server 配置决定。服务解析只允许 headless driver；任何窗口 API 调用都应在编译 feature 或 capability report 阶段被拒绝。

server 关闭流程更短：停止网络 tick、关闭 simulation scope、等待 deterministic jobs，再按逆拓扑 cleanup。禁止使用“创建隐藏窗口再复用 client profile”的捷径，因为这会把 GPU device 和 surface 生命周期带入服务器。

## 状态转移审计表

| 转移 | 前置条件 | 产生的证据 | 失败后是否可重试 |
| --- | --- | --- | --- |
| Constructing -> Registered | task graph 已创建 | runtime identity、descriptor count | 是，重建 runtime |
| Registered -> Activating | registry freeze 成功 | topology hash | 是，修复 descriptor |
| Activating -> Ready | 所有 required ready | activation receipt | 是，扩大预算/修复 provider |
| Ready -> Running | host frame admitted | first-frame demand | 是，关闭后重启 |
| Running -> Quiescing | close request accepted | close reason | 否，当前实例终止 |
| Quiescing -> Draining | admissions closed | census snapshot | 可继续直到 deadline |
| Draining -> Cleaning | census zero | drain receipt | 是，按剩余预算继续 |
| Cleaning -> Stopped | cleanup 全部完成 | shutdown report | 失败需进程级终止 |

## 运维恢复剧本

当启动失败时，先读取 activation report，而不是重复点击启动按钮。若错误是 missing capability，检查 profile/feature；若是 ready timeout，检查资源队列和 worker saturation；若是 factory panic，隔离插件并保存 panic payload 摘要。启动重试必须创建新的 runtime identity，确保旧 task 和 service 不会混入。

当停机卡住时，读取每个 scope 的 queued/running、service guard owner、asset watcher 和 pending allocation。先取消 `CancelOnDrop` 工作，再给 `FinishOnShutdown` 任务剩余预算；只有在报告明确不可恢复时才执行进程级终止。

## 可观测字段

建议为每个状态记录：`runtime_id`、`profile`、`target`、`module`、`state_before`、`state_after`、`generation`、`elapsed_us`、`remaining_budget_us`、`error_kind`。状态转移事件要可关联 host request id，便于把窗口关闭、插件卸载和 task drain 连接到同一条 trace。

## 设计决策

- 选择显式 `Ready` 而不是第一次调用时隐式初始化，换取可预测首帧和清晰错误。
- 选择绝对 deadline 而不是层层相加的相对 timeout，避免深层 cleanup 无限延长。
- 选择 stale handle 错误而不是自动迁移，避免把旧对象写入新 runtime。
- 选择 profile capability report 而不是 `cfg!(feature)` 猜测，保证部署配置可审计。

## 验证矩阵

| 测试族 | 应证明的事实 |
| --- | --- |
| activation/behavior/activation | 依赖先激活且 callback 顺序稳定 |
| activation/behavior/deactivation | admission、drain、cleanup 顺序正确 |
| activation/behavior/reactivation | generation 递增，旧 handle stale |
| activation/behavior/deactivation/blocked | 阻塞 dependent 能被定位 |
| entry/runtime_entry_source_guards | host 不在错误状态调用 runtime |

实现任何新的 module lifecycle 时，应先为每个状态增加失败测试，再更新本页状态表。

## API 前置条件与后置条件

| 接口 | 前置条件 | 成功后 |
| --- | --- | --- |
| `CoreRuntime::try_new` | worker 配置有效 | runtime identity 与 task graph |
| `register_module` | registry 未冻结 | descriptor 纳入拓扑 |
| `activate_registered_modules_with_ready_timeout` | 图验证通过 | required services admitted |
| `tick_time` | runtime running | outer frame snapshot |
| `shutdown_task_graph` | 外部提交已停止 | worker shutdown report |
| `shutdown_registered_modules_with_drain_timeout` | task/guard 可排空 | modules stopped |

## 常见反模式

- 首帧后再注册 required module，导致 topology hash 与启动报告不一致。
- ready timeout 后继续 frame loop，造成部分功能看似可用。
- shutdown 时先 cleanup 后 close admission，形成 use-after-cleanup 风险。
- 为每层重新创建 timeout，导致总停机时间无上限。
- 重启时复用旧 runtime handle，混淆 generation。

## 演练脚本设计

测试宿主应能够注入：worker 创建失败、descriptor missing、factory panic、ready timeout、held guard、stuck FinishOnShutdown task、cleanup error。每个注入点都验证最终 state、report 字段和是否允许 retry。测试完成后确认没有后台线程、watcher 或 allocation 残留。

## 章节验收

- [ ] 启动与停机状态都有 owner、入口和终态。
- [ ] editor/server 两种变体都不依赖隐式窗口能力。
- [ ] deadline 在嵌套层只减少不重置。
- [ ] stale handle 和 blocked drain 有恢复动作。
- [ ] 状态转移与测试目录一一对应。

## 交叉模块契约

app entry 负责 host request 和窗口生命周期，CoreRuntime 负责 module/service/task authority，editor 负责 workbench/document，dynamic API 负责 session/allocation。启动和停机报告要把这些 owner 串在同一个 request/trace id 下。

## 版本升级注意

新增生命周期状态必须明确旧状态的映射、首帧/停机兼容策略和诊断字段。不能仅在 host 层添加布尔开关绕过 runtime state machine。

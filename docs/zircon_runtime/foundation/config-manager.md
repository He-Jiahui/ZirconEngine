---
related_code:
  - zircon_runtime/src/core/framework/foundation.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/persistence.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence.rs
  - zircon_runtime/src/foundation/runtime/config_manager/state.rs
  - zircon_runtime/src/foundation/runtime/config_manager/worker.rs
  - zircon_runtime/src/foundation/runtime/config_manager/writer.rs
  - zircon_runtime/src/foundation/persistence/atomic_file.rs
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
implementation_files:
  - zircon_runtime/src/core/framework/foundation.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/persistence.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence.rs
  - zircon_runtime/src/foundation/runtime/config_manager/state.rs
  - zircon_runtime/src/foundation/runtime/config_manager/worker.rs
  - zircon_runtime/src/foundation/runtime/config_manager/writer.rs
  - zircon_runtime/src/foundation/persistence/atomic_file.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
doc_type: code-module
status: current
---

# Config manager

`DefaultConfigManager`是foundation层的配置服务实现。内存事实源仍位于`CoreHandle`配置存储；manager从用户配置路径加载JSON，并通过`ConfigManager`提供typed boundary之外的`serde_json::Value`读写与显式持久化控制。

## 当前写入语义

- `set_value`只比较并更新内存事实源，然后推进dirty generation并唤醒worker；调用线程不snapshot、不序列化且不执行文件系统写入。
- 同值且没有dirty时不创建工作；持久化失败后dirty独立保留，同值调用会重新请求提交，不能因value equality吞掉失败。
- 所有clone共享一个`zr-config-persist` owner。worker使用25 ms trailing debounce合并burst，snapshot最新完整配置并只确认本次attempt对应的generation；attempt期间出现的新变化继续保持pending。
- `ConfigFileWriter`只在worker线程执行。生产writer先通过foundation共享原子文件owner生成已flush/sync的pending transaction，再进入配置路径的commit fence做平台atomic replace；失败时旧完整JSON保持可读。

## Flush与关闭

- `flush(timeout)`强制调度当前dirty generation并等待它持久化；成功、worker错误和调用方timeout都有确定的`Result`语义。
- 最后一个worker owner析构时请求shutdown强制flush，最多等待2秒。线程在期限内退出则join；超时会取消该worker的commit fence并写入tracing错误，关闭流程不会无界阻塞。
- 每个配置路径使用与目录是否已创建无关的词法绝对key，共享一个进程内epoch gate。新manager注册会推进epoch；旧detached worker即使稍后恢复，也只能清理staging，不能在新manager提交之后覆盖旧快照。若旧worker已经进入replace，新的注册立即返回明确`WouldBlock`配置错误而不是无界等待；提交退出后调用方可重新激活。
- 启动时配置文件不存在且没有transaction backup是合法空状态。Windows若canonical目标缺失且恰有一个同owner backup，启动先恢复backup；多个候选、恢复失败、文件不可读或JSON无效均返回带路径与原因的`CoreError::ConfigParse`，不再静默忽略损坏配置。

## 可观测性

`persistence_report()`返回dirty/persisted generation、当前与峰值pending flush、attempt/success/failure计数、累计serialized bytes、p95与最大flush时延以及最后一次错误。pending depth按“是否存在未持久化generation”计数，因此单owner队列上界为1，不暴露实现内部锁或线程句柄。

配置提交仍由Runtime02单一owner负责；editor、layout和插件不得各自增加timer、持久化线程或绕过`ConfigManager`直接写配置文件。

## 验证重点

聚焦测试覆盖同值零工作、失败后同值重试、并发更新不丢失、1000次burst coalescing、最后owner shutdown flush、显式flush timeout、首次parent不存在时shutdown timeout后的旧writer fencing、已进入commit时replacement activation快速失败、损坏启动文件、Windows单backup启动恢复，以及replace部分失败时canonical旧JSON恢复。受管Cargo门禁通过前，只声明实现与静态检查完成，不声明里程碑验收。

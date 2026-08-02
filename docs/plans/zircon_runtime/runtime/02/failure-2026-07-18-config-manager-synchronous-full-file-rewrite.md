---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: config-manager-synchronous-full-file-rewrite
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/foundation/mod.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence.rs
  - zircon_runtime/src/foundation/runtime/config_manager/state.rs
  - zircon_runtime/src/foundation/runtime/config_manager/worker.rs
  - zircon_runtime/src/foundation/runtime/config_manager/writer.rs
  - zircon_runtime/src/foundation/persistence/atomic_file.rs
  - zircon_runtime/src/foundation/runtime/config_manager_tests.rs
  - zircon_runtime/src/core/runtime/config_store.rs
---

# Config manager synchronous full-file rewrite

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/{build.rs,src/{lib.rs,prelude.rs,foundation/**}}`当前源13/13 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 交接原因：配置存储、生命周期flush和后台任务owner属于Runtime02 core spine，不应由调用方各自防抖或绕过manager。

## 失败现象与复现证据

原实现每次`ConfigManager::set_value`都在调用线程snapshot全部配置、pretty-serialize并`fs::write`整文件，同值调用也产生O(total config bytes)序列化和同步I/O；clone manager并发调用还可让整文件写入交错覆盖。性能切片已直接加入同值检测、dirty失败重试和共享持久化mutex作为止损。

## 最低共享层根因

配置内存状态没有dirty generation、提交调度和原子持久化合同，`set_value`同时承担逻辑变更与磁盘提交。当前止损仍在真实变化时同步整文件写盘，mutex只保证进程内顺序，不能提供批量写、主线程隔离或崩溃一致性。

## 架构修复验收

- `set_value`只更新内存并推进dirty generation；1/1k burst调用方filesystem time=0。
- 单owner worker按明确防抖/批量策略snapshot并序列化；写次数有上界，最新generation最终持久化且并发更新不丢失。
- 写入同目录临时文件，flush成功后atomic replace；任一步失败保留旧完整文件并报告可观测错误。
- shutdown执行有界flush，超时/错误有确定语义；重启只读到旧完整或新完整版本，不读到截断JSON。
- 测试覆盖同值零写、失败后同值重试、并发更新、worker coalescing、shutdown flush和crash consistency；记录write count、bytes、queue depth和p95 latency。

## 禁止临时方案

- 不得让editor/layout等调用方分别加timer或绕过`ConfigManager`直接写文件。
- 不得用“内存值相同即永不重试”吞掉上一次持久化失败；dirty状态必须独立于value equality。
- 不得继续以`fs::write`原地截断目标文件后写入，或把无界写任务投递到通用线程池。

## 修复结果与回传

Runtime02已完成代码层架构修复：

- `DefaultConfigManager::set_value`只更新`ConfigStore`并请求dirty generation；配置snapshot、JSON序列化和文件系统写入均在命名为`zr-config-persist`的单owner worker执行。
- worker使用trailing debounce合并burst，只确认attempt目标generation；并发产生的更新保持pending。失败保留dirty，同值调用可重新请求提交。
- `ConfigManager`公开有界`flush(timeout)`与`ConfigPersistenceReport`，报告generation、pending/peak、attempt/success/failure、serialized bytes、p95/max latency和最后错误。
- 最后owner关闭时强制flush并最多等待2秒；启动阶段对存在但损坏/不可读的配置返回带路径的明确错误。
- 每个配置路径由不依赖目录存在性的词法绝对key与epoch commit gate串行化实际replace/注册；shutdown timeout取消旧fence并记录tracing错误，detached worker不能在新manager提交后覆盖旧快照。已进入replace时，新激活快速返回明确错误而不是无界等待。
- Windows replacement部分失败若只剩backup会立即恢复canonical目标；崩溃后启动仅在目标缺失且backup唯一时恢复，多个候选直接返回明确错误。
- 资产sidecar/registry原子写实现已硬切到`foundation/persistence/atomic_file.rs`，配置writer直接复用同一生产owner；旧`asset/project/meta_io.rs`模块被删除且没有转发兼容层。
- 源级TDD覆盖同值零工作、首次失败后同值重试、4线程并发无丢失、1000次burst合并为一次写入、replace失败保留/恢复旧完整JSON、最后owner shutdown flush、flush timeout、shutdown timeout旧writer fencing、Windows backup启动恢复和损坏启动文件。

Independent review round 1: `Critical 0 / Important 2 / Minor 0`。两项Important分别为Windows部分replacement失败恢复和shutdown timeout detached worker迟到提交；r4已按最低共享层加入canonical backup recovery与per-path epoch commit fence。

Independent review round 2: `Critical 0 / Important 2 / Minor 0`。后续两项Important指出目录创建前后canonical key可能漂移，以及已进入replace时新注册可能无界等待；r4已改为纯词法稳定key与`try_lock` fail-fast激活，并增加parent初始不存在和fence-admitted阻塞两条回归，待round 3复核。

Independent review round 3: `Critical 0 / Important 0 / Minor 0`。复核覆盖稳定path key、`try_lock` fail-fast注册，以及pre-fence/fence-admitted两种shutdown timeout回归；未发现新的阻断项。

Open state: `主架构、两轮review修复、round 3独立复核、精确rustfmt与scoped diff-check已完成。受管Cargo聚焦reservation 77cf6cb7e95b4972aadc39d6a1356d1f等待执行；其sourceManifest为空，只能作shared-current诊断。最终source-bound聚焦/完整门禁、failure fixed return与里程碑提交尚未完成，因此本Failure仍为open`。

### 2026-07-19 性能复审补充

- `foundation/**` current 16/16重新逐文件静态核对；主架构保持单worker、dirty generation、防抖、atomic replace与有界shutdown。
- 新增回归并修复pending dirty generation上的同值调用刷新`last_dirty_at`和重复notify：真实change仍延长debounce，失败后同值调用仍重新请求。
- 源码RED→GREEN守卫、`rustfmt`、scoped `git diff --check`通过；current-source Cargo仍pending，PERF-MVP-223不提前转完成。

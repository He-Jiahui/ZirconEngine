---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: preference-storage-bounded-persistence-lane
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/platform/preferences/storage.rs
  - zircon_runtime/src/core/framework/platform/preferences/mod.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io
  - zircon_runtime/src/platform/service_types/manager.rs
  - zircon_runtime/src/platform/service_types/driver.rs
  - zircon_runtime/src/platform/preferences/backend.rs
  - zircon_runtime/src/platform/preferences/persistence
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_runtime/tests/runtime11_preference_backend_authority.rs
  - examples/woc/native/apps/woc_client/src/preferences/storage.rs
  - examples/woc/native/apps/woc_client/tests/preference_storage_support.rs
tests:
  - 'python -B -c "from pathlib import Path; source = Path(''zircon_runtime/src/platform/service_types/manager.rs'').read_text(encoding=''utf-8''); assert ''preference_storage_backend().read('' not in source and ''preference_storage_backend().write('' not in source and ''preference_storage_backend().remove('' not in source and ''preference_storage_backend().flush('' not in source"'
  - python -B -m unittest tools.tests.test_runtime11_preference_persistence_lane_contract -v
  - cargo +1.94.1 test -p zircon_runtime --lib preference_persistence_lane --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib platform_preference_storage --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --test runtime11_preference_backend_authority --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --doc --locked --jobs 1
  - cargo +1.94.1 test --manifest-path examples/woc/native/Cargo.toml -p woc_client --tests --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime11：偏好存储缺少有界持久化执行通道

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M6 跨平台偏好存储接线前性能/生命周期验收（`PERF-MVP-589`）
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：Frameworks05 拥有 platform-neutral 偏好合同和 host backend 边界；阻塞 I/O 的任务接纳、串行 pipe、背压、取消、deadline、flush/shutdown fence 与诊断属于 Runtime11 统一 JobSystem。继续在 Platform manager 内同步调用 backend 会复制调度真相，WOC/Editor 私建线程则会绕过 Runtime11 的全局线程预算。

## 失败现象与复现证据

当前中立 `PreferenceStorage` 与 host-only `PreferenceStorageBackend` 都暴露同步 `read/write/remove/flush`。`PlatformManager` 从 driver 的短 `RwLock` 中 clone backend `Arc` 后，直接在调用线程执行该同步方法；锁生命周期虽然短，但没有改变 backend I/O 仍由 caller 执行的事实。

Frontmatter 第一条只读 source audit 在 2026-07-31 current source 稳定退出 1：`manager.rs` 同时命中 `preference_storage_backend().read/write/remove/flush` 四条直接转发。这是无需 Cargo 的架构 RED；planned unittest owner 只有在 Runtime11/Platform hard cut 同批落地后才可创建并转 GREEN，不能以 import error 充当失败证据。

Desktop `AtomicFilePreferenceStorageBackend` 的 read 直接访问文件系统；write 会构造 hash/path、写 staging、原子替换、同步已提交文件，并在 Unix 同步父目录；remove 也可能同步父目录。若 WOC 或 Editor 在 frame/UI 路径接入现有 manager 合同，caller filesystem wall 大于零，backend stall 会直接变成帧停顿。当前尚无产品调用方，因此这是一项**接线前阻断**，不是已发生的 F0/F2/F4 热点或 Cargo 失败。

现有 Runtime11 `JobScheduler`/`TaskPools` 提供统一 worker 预算、句柄与诊断，但偏好服务没有唯一 persistence pipe，也没有 queue entries/bytes 上限、per-key generation 合并、read-your-write、deadline/cancel、flush/shutdown fence 或 durability 终态。同步 `flush()` 返回成功只表示 caller 已直接执行 backend，不能证明异步产品接线的有界生命周期。

## 最低共享层根因

当前合同把两个不同责任合并为一条同步调用链：

1. `PreferenceStorage` 应是 runtime consumer 可用的有界提交/观察合同；
2. `PreferenceStorageBackend` 是 host 实现的阻塞 durability primitive，只能由 Runtime11 I/O worker 调用。

Platform manager 直接实现第一条合同并同步转发第二条合同，使线程、背压和终态没有 owner。最低共享修复不是删 fsync、扩大 frame budget 或把同步调用包进每个调用方自己的线程，而是由 Runtime11 提供唯一、**领域无关**的 bounded keyed I/O pipe；Platform 负责把 Preference DTO/策略映射为 opaque key/work、安装 backend、投影 capability 并消费 pipe ticket。

`core::runtime::tasks` 不得 import `core::framework::platform`、`PreferenceKey` 或任何 `Preference*` 类型。Runtime11 只拥有通用 admission/ticket/fence/deadline/terminal、opaque key 的 pending replacement、I/O-pool execution 与中立 queue/wall-time diagnostics；Frameworks05/Platform 继续拥有 PreferenceStorage DTO/error、overlay/tombstone/read-your-write、host backend adapter，以及 hash/path/staged-write/fsync 等 backend-specific diagnostics。该分界避免把 Platform 语义反向下沉进 core task spine。

## 架构修复验收

- **单一执行 owner 与 public hard cut**：硬切 consumer-facing `PreferenceStorage` 为有界提交/快照/终态合同；同步 `PreferenceStorageBackend` 从 `core::framework::platform` neutral surface 删除，唯一迁到公开 host-only `zircon_runtime::platform::preferences` SPI。`PreferenceBackendWorkAuthority` 必须从同一 host route 公开，使外部 host 能命名并实现 trait；每个 `open_read`/write/remove/flush primitive 只接受调用期借用 `&PreferenceBackendWorkAuthority`。token 的私有字段和唯一私有 constructor 由 `platform/preferences/persistence/work.rs` 拥有，禁止 Clone/Copy/Default/public factory，外部实现可编译但不能构造、复制或留存 authority；只有 persistence worker 能取得调用能力。`PlatformManager` 不再持有、接受或直接调用 backend；不得通过 helper 返回、局部变量、改名同步 API、re-export、prelude 或 facade 保留旧 consumer 路径，也不得新建 preference 私有线程或第二套 scheduler。
- **双层有界接纳**：Runtime11 pipe 在复制 opaque key/work、排队或启动 closure 前同时检查 entry 与 retained-byte 上限；Platform overlay 另有独立 entry/retained-byte limits，精确计入 namespace/key/value、tombstone、generation、typed failure projection 与 adapter metadata。Platform 先用 checked arithmetic 生成复合 `PreferencePersistenceQuote`，其中 `overlay_retained_bytes` 与 `lane_retained_bytes` 分别包含各 owner 实际保留的 payload 与 metadata，不得用一个标量替代两层计量；首次 backend read 按 configured `max_value_bytes <= MAX_PREFERENCE_VALUE_BYTES` 预留最大结果。backend 的 `open_read` 只返回 `Read + Send` stream，不返回已分配 Vec；唯一 persistence worker 用 `Read::take(max_value_bytes.checked_add(1))` 消费，超限返回 typed capacity failure。内置 atomic-file backend 必须使用 `File::open`，静态拒绝 `fs::read`。backend 的任意动态 source/message 只存在于 worker stack，发布 terminal 前必须投影成 `PreferencePersistenceFailureProjection` 的固定 fields + 至多 `MAX_PREFERENCE_FAILURE_DETAIL_BYTES` 的 UTF-8-safe detail，并把上限计入两层 quote。先取得 overlay reservation，再以 lane quote 取得不可运行的 lane admission，安装可见 generation 后才激活 work。任一 quote overflow/reservation 失败都在复制/可见前 typed-reject。durable success 且无更新 generation 时可释放 overlay payload；failed/cancelled overlay 继续占 quota，直到 retry 成功或调用显式有损 eviction，并将 eviction 作为可观测终态。
- **generation 与合并**：Platform 为同一 namespace/key 分配单调 generation，并映射为 pipe opaque key；同 key、同一 durability barrier 之前的 queued write/remove 只保留 latest generation，旧操作终态标为 `Superseded { successor }`。正在执行的 primitive 不被伪取消，后继 generation 保持确定序；不同 key 可以在全局预算内并存。
- **read-your-write 与线性化**：Platform 以 `reserve overlay -> admit suspended work -> install generation -> make runnable` 为唯一提交序列，worker/terminal 不可能先于 overlay 发布。随后 read 返回同 key 最新已接纳 write 或 remove tombstone及其 durability 状态，不等待文件系统，也不静默回退到旧 durable value。terminal callback 和首次未加载 read completion 只有在 generation 仍匹配时才能更新 overlay；过时 completion 必须丢弃为 stale observation，不能覆盖更新 generation。
- **global durability fence**：`flush_fence` 在单一 global admission epoch 线性化，捕获所有 pre-fence key obligations，并把 backend `flush()` 作为独占 barrier primitive 排入 Runtime11 lane。barrier 后任意 key 的 work 可以接纳/进入 overlay，但在 pre-fence obligations 成功且 backend flush 返回前不得进入 backend；同 key work也不得跨 barrier supersede/coalesce。任何 backend error、work deadline、允许发生的 pre-start cancellation 或断链 superseded obligation 都使 fence typed-fail；`Superseded` 自身不是 durable success。矩阵必须同时覆盖 fence 后同 key和不同 key submit，证明 post-fence backend work 均不会越过全局 flush。
- **work deadline / observer timeout / cancel authority**：admission-owned `work_deadline` 只决定尚未开始的共享 work 是否进入 `DeadlineBeforeStart`；observer-owned `wait_until` 超时只结束该 waiter，不改变共享 terminal。显式 `cancel_before_start` 只允许持有 mutation cancel capability 的 owner 调用、幂等，work 已开始则返回 `AlreadyStarted`，被 global fence pin 后返回 `FencePinned`；不得由任意 waiter 取消共享 durability obligation。任何 pre-start terminal 都不回滚可见 overlay，而是按 generation 标成 `visible_not_durable`；in-flight backend success/error始终是权威 terminal。
- **shutdown ownership**：shutdown 停止新接纳，按上述 cancel authority 处理未开始且未被 fence 固定的 work，并等待 in-flight backend 返回。deadline 只结束本次 shutdown wait并返回 `BoundedKeyedIoShutdownGuard`；guard 持有 lane state、已接纳/执行中的 `JobHandle`/tickets 与可查询 incomplete/error report，原 lane owner在 guard 完成前不得消失。共享 `TaskPools` 始终拥有 Rayon worker 生命周期，lane 不创建私有 OS thread/pool，也不拥有或 join worker handles；guard Drop 只能 drain 已提交 work 到真实 terminal，不能 detach。调用方可显式继续 `wait_until`/query 后再完成 shutdown。
- **调度安全**：lane 控制锁只保护 bounded state，不跨 backend 调用、ticket wait 或 terminal observer；I/O worker 和 terminal observer 不得同步等待同一 lane。deadline 在任务尚未获得 worker 时也能终止，不能依赖 backend 内部 checkpoint 才生效。
- **可观测性**：Runtime11 公开领域无关的 queue entries/bytes、in-flight、oldest age、submitted/completed/failed/cancelled/superseded/coalesced 与 worker wall；Platform 汇总 namespace/key generation、overlay durability、backend wall、caller filesystem wall、hash/path build 与 staged-write/fsync。frame/UI caller filesystem wall 必须为 0。
- **下层行为矩阵**：以 1/1k/100k 提交、0/10/1000ms backend stall、0/1/64MiB values、同 key storm、跨 key fairness、lane/overlay capacity reject、backend error、pre-start work deadline、shutdown race验证有界性；另覆盖 persisted file 为 `max_value_bytes + 1`、backend error detail 超过 cap、flush-before-later-same-key、flush-before-later-different-key、cancel/deadline-after-overlay、deadline-during-inflight、multiple waiters one timeout、cancel-vs-fence、stale read completion、shutdown timeout 后 query/drop。断言 oversized read 在完整分配前 typed-reject、retained error projection 不超过 cap、barrier 不跨代合并、post-fence backend work 不越过 global flush、逻辑可见性带 durability failure、observer timeout 不伪造 shared terminal，且任何 non-durable terminal 都不能使 flush 成功。
- **上行回传**：Frameworks05 回跑 `platform_preference_storage`，WOC 再接引擎 adapter 并验证 fresh-process、角色 scope 隔离、损坏值、拒绝写与真实平台 backend；在这些证据完成前不得把 Frameworks05 M6 或 `PERF-MVP-589` 标为 accepted。

## 禁止临时方案

- 不得删除或弱化 desktop atomic commit/fsync 来换取 frame time；durability 政策只能由明确产品合同修改。
- 不得在 WOC、Editor、Platform manager 或单个 backend 中私建线程、无界 channel、轮询循环、全局静态 map 或调用点特例。
- 不得用 “spawn 后忘记”、process-local fallback、静默覆盖错误、shutdown 丢队列、同步 read 特例或测试专用 bypass 冒充完成。
- 不得保留同步 consumer API 作为 legacy/compat/shim；hard cut 必须迁移全部产品调用方，并维持 backend-only 同步边界不公开给 frame/UI consumer。

## 修复结果与回传

Current state: `resolving_failure`。Runtime11/Platform/WOC hard cut 已实现，静态契约与精确格式门通过；独立 current-source 二次审查、canonical Rust 1.94.1 受管编译、focused tests、doctest、WOC 上行门、Frameworks05 上行门和 coordinator failure return 尚未完成，因此本 artifact 继续保持 `handoff_kind: failure` / `status: open`，不得提前声明 fixed。

2026-08-01 current-source implementation evidence:

- `core::runtime::tasks::bounded_keyed_io` 现在由 lane 持有所有已接纳 work；Fence 在分配 prerequisite snapshot 前先计算精确 retained bytes 并与 caller quote 一起完成有界接纳，容量拒绝不分配 ticket、不推进 global epoch。连续 Fence 只链向最近一个 Fence，再捕获该 Fence 之后的 non-fence obligation，使 256 个排队 Fence 的 prerequisite records 保持 O(N)，而不是重复复制全部历史形成 O(N^2)。Fence 依赖仍使用可释放 pin；后继同 key durable generation 可替代旧失败，而不同 key 的 non-durable obligation 继续使 Fence 失败。
- work/Fence deadline 通过既有 process `TaskTimer` 独立于饱和 I/O worker 发布 `DeadlineBeforeStart`；work panic 被收敛为 typed `work_panicked` terminal，容量、observer 和 shutdown report 均在终态后释放或可查询。Shutdown 使用 `Condvar` drain，不再 spin，也不依赖 admission owner 后续 activate/drop 才能完成 fence-pinned work。
- lane 出队与 `mark_started` 现在在同一控制锁临界区内线性化，shutdown 不会再遗漏 active-before-start work；active observer 由最终清理路径恰好通知一次，shutdown complete 同时要求 pump 状态和对应 `JobHandle` 已真实 terminal，因此 terminal observer 尚未返回时 guard 不会提前完成。
- Platform adapter 用短 submission mutex 线性化 generation、overlay reservation、lane admission、可见 generation 安装、terminal observer 与 activation；pre-start terminal 无需 ticket polling 即投影 `visible_not_durable`。已失败 generation 会阻断每个 Fence，直到同 key retry durable 或调用显式 lossy eviction。
- `PreferencePersistenceLimits` 对 `max_value_bytes > MAX_PREFERENCE_VALUE_BYTES` typed-reject；lane quote 同时计入 opaque key 与 closure 保留的 `PreferenceKey`。同 key recovery 不重复占用 overlay entry quota，跨 key 容量只能通过明确 eviction 释放。
- Overlay 对同 key replacement 使用旧 generation 退出后的投影 retained bytes 计费，使默认限制下完整 64 MiB 失败值仍可原 key retry；显式 eviction 只接受已经终止的 `VisibleNotDurable` generation，Pending 和 Durable generation 均不可被移除或与仍在执行的 backend work 脱节。
- Platform 复合诊断现在包含 lane/overlay generation durability、backend wall、固定为 0 的 caller filesystem wall，以及 atomic backend 的 hash/path、staged-write/commit、fsync wall 与 operation counters；应用在注册 Platform descriptor 时把 host backend 注入其 canonical driver factory，使 driver 从首次构造起已安装 backend，任何模块激活通知或 consumer resolution 都不会观察临时 `Unavailable`。
- 首次 backend read 仍按最大值 admission，但 completion 会把 overlay quote 收敛到真实 value 长度；空值和 typed failure 收敛为 0 value bytes，不再把最坏 64 MiB 预算永久占用。对应小容量矩阵证明多个小冷读可持续接纳。
- Atomic-file backend 的 namespace/key hash path 由同一 mutex owner 的 4096-entry bounded FIFO cache 复用，并公开 hit/miss/build/eviction/current-entry 诊断；命中只执行 `HashMap` O(1) lookup，不再扫描或移动 FIFO order，容量满时确定逐出最旧 key，不引入全局静态 map 或无界 retained path。
- `PreferencePersistenceAdapter` 的 public bypass 已从 platform route 硬切为 crate-private，`PlatformManager::default` 已删除；模块 cleanup 以 250ms 有名预算等待持久化 lane，超时返回 typed `ModuleCleanupTimeout` 并保留 shutdown guard 和 manager/driver 服务，后续 cleanup 可继续 drain，不 detach、不在 adapter Drop 阻塞启动 shutdown。
- WOC 删除自有同步 `PreferenceStorage` trait 与公开 re-export，stored settings/keybind/gamepad/inventory 直接消费引擎 `PreferenceStorage` snapshot/submission 合同；冷读桥接显式区分 `Pending` 与 `Ready`，stored model 通过 `refresh_from_storage` 接收首次 backend completion，库存读取提供同等的 `try_load_inventory_filter`，不再把启动期 Pending 永久固化为默认。每个 stored model 保留最后一次 `PreferenceMutationSubmission`，库存写直接返回 submission，使 typed durability failure 可由 consumer 观察而不回滚 session-visible overlay。集成测试通过真实 `CoreRuntime -> PlatformDriver -> manager service` 安装 host-memory backend，并用只写 backend、不预热 overlay 的 seed 覆盖 settings/gamepad/keybind/inventory fresh-process 冷启动；测试 backend 为每个 key 提供显式 `Mutex + Condvar` read gate，构造器先被确定性阻塞在 backend read 内，测试断言 Pending/default 后才释放，避免内存 backend 过快完成造成竞态假绿或偶发失败。keybind 另覆盖 scoped miss completion 后才启动 legacy cold read、两次 completion 均不固化中间默认值。backend 后续不可用时 session-visible overlay 不倒退，fresh unavailable storage 才降级默认。
- Rust 行为矩阵已覆盖 1/1k/100k 接纳、1k same-key storm 与 interleaved-key fairness、64 MiB 失败值 retry、1000ms backend stall、active-before-start shutdown、active observer 单次通知、饱和 worker/Fence deadline、pin release、reverse activation、failed-generation repeated Fence、in-flight failure + durable successor、same-key single-slot recovery、terminal-only eviction、stale read、deadline during in-flight、multiple waiters、panic、shutdown timeout/query/drop、bounded stream、failure detail cap、atomic diagnostics、跨 key Fence 顺序、Fence prerequisite 预算拒绝/epoch 回滚和 256-Fence 线性 retained bytes 上界。
- Runtime11 静态契约中 6 个非 lock 用例 GREEN；新增的第 7 个用 `tomllib` 结构化解析 nested `Cargo.lock`，当前按 TDD 精确 RED 于旧 `woc_client` dependency list 缺少 `zircon_runtime`，将在 coordinator 受管生成 lock 后转 GREEN。`tools.tests.test_frameworks_05_preference_storage_boundary` 为 7/7 GREEN；direct manager primitive audit、`failed_epochs`/spin-loop negative audit、Rust 1.94.1 scoped rustfmt 和 `git diff --check` 均 GREEN。Cargo 证据只允许由 immutable managed validation receipt 补充。
- 前一轮只读审查发现的 shutdown 线性化、最大值 retry、terminal-only eviction、行为矩阵、observer/handle drain 和私有 owner import 已逐项前向修复。2026-08-01 首轮扩展二审为 `Critical 0 / Important 2 / Minor 0`：指出 WOC 冷启动只读一次 Pending 且无 refresh，以及 `examples/woc/native/Cargo.lock` 尚未包含新增 `zircon_runtime` 依赖。随后 current-source 复审再指出两个 Important：Fence prerequisite snapshot 未纳入 retained-byte admission 且重复 queued Fence 会形成 O(N^2) 保留，以及内存 backend 过快完成使 cold-read Pending 测试存在竞态。前两项源码问题均已按上述显式 refresh、精确 prerequisite 计费/O(N) Fence 链和 per-key read gate 完成前向修复，当前正对最终源码进行独立二次审查；nested lock 仍禁止手写，必须在二审收敛后由 coordinator 受管 `cargo +1.94.1 generate-lockfile --manifest-path examples/woc/native/Cargo.toml --offline` 生成并纳入精确快照。只有 lock 生成且最终 current-source 审查达到 `Critical 0 / Important 0`，才可提交受管 acceptance 门。

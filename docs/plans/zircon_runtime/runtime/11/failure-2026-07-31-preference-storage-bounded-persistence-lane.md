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
  - zircon_runtime/src/core/framework/platform/preferences/backend.rs
  - zircon_runtime/src/core/framework/platform/preferences/mod.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/platform/service_types/manager.rs
  - zircon_runtime/src/platform/service_types/driver.rs
  - zircon_runtime/src/platform/preferences/backend.rs
  - zircon_runtime/src/platform/preferences/persistence
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
tests:
  - 'python -B -c "from pathlib import Path; source = Path(''zircon_runtime/src/platform/service_types/manager.rs'').read_text(encoding=''utf-8''); assert ''preference_storage_backend().read('' not in source and ''preference_storage_backend().write('' not in source and ''preference_storage_backend().remove('' not in source and ''preference_storage_backend().flush('' not in source"'
  - python -B -m unittest tools.tests.test_runtime11_preference_persistence_lane_contract -v
  - cargo +1.94.1 test -p zircon_runtime --lib preference_persistence_lane --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib platform_preference_storage --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --test runtime11_preference_backend_authority --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --doc --locked --jobs 1
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

Open state: `runtime11_bounded_persistence_lane_tdd_red_implementation_pending`。Frameworks05 的中立合同、Platform manager/backend 注入与 desktop atomic-file primitive 已存在，Runtime11 persistence lane、typed ticket/fence、背压/合并/诊断及动态矩阵尚未实现；没有 Runtime/WOC managed pass、fixed return 或产品接线通过声明。

2026-07-31 architecture-first TDD evidence:

- 新增 `tools/tests/test_runtime11_preference_persistence_lane_contract.py`，锁定 Runtime11 folder-backed `bounded_keyed_io` owner、领域中立性、entry/retained-byte 双上限、ticket/fence/terminal/diagnostics、global admission epoch、独立 work/wait/cancel/shutdown 语义，以及 Platform persistence adapter 的 overlay reservation、generation 条件完成、跨 key flush 顺序与 backend-work ownership。
- shutdown guard 必须复用共享 I/O pool 的 `JobHandle`/tickets 并禁止 `std::thread`、`JoinHandle`、`ThreadPoolBuilder` 或私有 scheduler；Platform 复合 quote 分别暴露 checked `overlay_retained_bytes` 与 `lane_retained_bytes`。
- hard-cut contract 还要求删除旧 neutral `preferences/backend.rs`，真实路由 host backend/persistence modules，并把 `PreferenceStorageBackend` 类型与 read/write/remove/flush primitive 调用限定在精确 Platform allowlist，防止 driver/helper/局部变量改名旁路。
- backend primitive 进一步由 public host-route `PreferenceBackendWorkAuthority` 编译期 capability 锁定：外部 fixture 必须能实现 trait，但 authority 只能在 `persistence/work.rs` 的私有 constructor 构造；完整 derive token 集合、限定/非限定 `Clone`/`Copy`/`Default` impl、authority inherent impl 中返回 `Self`/authority/alias 的公开 factory 或 associated const/static，以及全文件返回 authority/传递 alias 的公开 free factory/value 都必须被常驻负向守卫拒绝，且不得误伤同文件其他 work 类型的公开 `-> Self` constructor。外部集成夹具同时锁定“可实现 host SPI”和“authority 只由 persistence worker 签发”的边界；owner 上的 `compile_fail` doctest 由 Rust 编译器证明外部 crate 不能直接构造私有 token。读取 SPI 只返回 stream，worker 统一 max+1 bounded consume；atomic-file `fs::read` 常驻负向守卫与 `CountingRead` 用例证明 oversized persisted value 不会先形成完整 Vec。bounded read 与 capped failure projection 的上限必须进入复合 quote，新增 oversized persisted value/error-detail 行为用例。
- 同一 contract 要求 consumer-facing `PreferenceStorage` 硬切为 snapshot/mutation-ticket/flush-fence API，neutral framework 不再导出同步 backend SPI，`PlatformManager` 不再接受/引用 backend；同步 primitive 只能存在于新的 host-only `platform::preferences::backend` owner。
- current source 运行该 contract 为预期 0/6：generic lane 缺失、Platform adapter/authority 缺失、同步 consumer signature 仍存在、manager 仍直达 backend、backend SPI 仍从 neutral framework 公开。这是实现前 RED，不是 acceptance 结果。
- Runtime source 存在其他有效写入租约；本切片没有越权编辑 task/platform Rust。待协调器提供 disjoint source wakeup 后，从 generic lane 最低层开始实现并逐层转绿。

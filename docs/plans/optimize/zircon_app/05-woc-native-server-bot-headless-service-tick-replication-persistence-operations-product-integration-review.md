---
related_code:
  - examples/woc/native/Cargo.toml
  - examples/woc/native/Cargo.lock
  - examples/woc/native/apps/woc_server/Cargo.toml
  - examples/woc/native/apps/woc_server/src/main.rs
  - examples/woc/native/apps/woc_server/src/lib.rs
  - examples/woc/native/apps/woc_server/src/fixed_tick_driver.rs
  - examples/woc/native/apps/woc_bot/Cargo.toml
  - examples/woc/native/apps/woc_bot/src/main.rs
  - examples/woc/native/apps/woc_headless/Cargo.toml
  - examples/woc/native/apps/woc_headless/src/main.rs
  - examples/woc/native/plugins/woc_runtime/Cargo.toml
  - examples/woc/native/plugins/woc_runtime/src/lib.rs
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
  - examples/woc/native/crates/woc_protocol/src/contracts.rs
  - examples/woc/native/crates/woc_protocol/src/movement_input.rs
  - examples/woc/native/crates/woc_protocol/src/payload.rs
  - examples/woc/zircon-project.toml
  - examples/woc/README.md
tests:
  - examples/woc/native/apps/woc_server/tests/fixed_tick_driver.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/18-woc-generated-content-catalog-buildset-install-query-runtime-review.md
  - docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/11-woc-parity-oracle-trace-golden-differential-replay-evidence-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetDriver.cpp
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/main/main.cpp
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/main/multiplayer_api.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 05 · WOC Native Server、Bot、Headless Service Tick、Replication、Persistence 与 Operations 工程化差距

## 1. 结论

WOC server 当前有一段值得保留的局部 fixed-tick 骨架。`FixedServerTickDriver` 把20 Hz边界固定为50,000,000 ns，不把host wall time写入authoritative payload；command batch在进入队列前做容量与pending-window重复sequence检查，movement先做完整validate并拒绝同一pending batch中的重复actor；每个tick前对command做canonical sort；catch-up有每次调用上限且不会静默丢弃accumulator backlog；VM fault会把runtime转为`Faulted`并保留一次失败输入诊断。这些都比在`main`里直接调用一轮`fixed_tick()`更接近可审查的scheduler。

但这段driver没有进入server产品。`woc_server` binary只依赖`woc_runtime::identity_report_json()`，打印JSON后正常退出；`woc_server` library target与binary target之间没有调用边。`woc_bot`和`woc_headless`各只有manifest与8行identity reporter，依赖也只有`woc_runtime`。三种角色都没有clock loop、ZrVM implementation、listener/session、replication、persistence、signal、health、admin、metrics或shutdown owner。因此这里的“server/bot/headless”仍只是artifact name，不是可持续运行的产品能力。

即使未来把driver直接接到网络入口，它的服务语义仍不安全。driver接受没有principal、connection、world或shard identity的裸`Vec<Command>`和`Vec<MovementFrame>`；重复窗口只覆盖当前pending queue，tick取走队列后立即清空。更严重的是，driver在authoritative transaction完成前先`mem::take`并清空队列；VM fault后失败batch只进入`last_failed_input`，没有replay journal、commit receipt或恢复API。fault后的下一次`advance()`还会用空batch触发`SessionNotRunning`，覆盖原始失败诊断。输入既不在pending queue，也不在可恢复journal中，不能证明at-most-once、at-least-once或exactly-once。

当前commit边界也只停在进程内`CommittedSnapshot`替换。没有把command outcome、replication publication、durable journal/checkpoint和client ACK绑定为同一commit generation；没有证明server crash、disk full、network partition或VM trap后客户端观察与持久状态一致。queue预算只按item计数，构造器接受任意`usize`并立即`Vec::with_capacity`，没有aggregate payload bytes、decode work、per-principal inflight或process memory admission。把协议字段上限当成服务资源上限会留下直接的OOM与noisy-neighbor路径。

App03继续拥有四角色`WocProductHost`、真实ZrVM adapter、outer world-state transaction和Client/Server总闭环；Runtime08E拥有secure transport，Runtime19拥有command codec/admission/movement/outcome语义。本篇只拥有native server service composition、host clock/queue/fault recovery、authoritative publication/durability orchestration、operations，以及bot/headless runner合同。本轮登记 **4 项 P0、72 项 P1 和 16 项 P2**，不重复计算App03已经登记的“产品角色只打印identity”总P0。

## 2. 审查边界与物理清单

### 2.1 文件级覆盖

| 物理文件 | 行数 | 产品可达性与结论 |
|---|---:|---|
| `woc_server/Cargo.toml` | 10 | 声明`woc_protocol`与`woc_runtime`；无engine host、network、async、persistence、telemetry依赖 |
| `woc_server/src/main.rs` | 8 | 只读取project identity并打印；不引用同package library |
| `woc_server/src/lib.rs` | 3 | 只导出`fixed_tick_driver`；binary无consumer |
| `woc_server/src/fixed_tick_driver.rs` | 259 | 唯一service-like production code；只有integration test消费 |
| `woc_bot/Cargo.toml` | 9 | 只依赖`woc_runtime` |
| `woc_bot/src/main.rs` | 8 | 只打印Bot identity并退出 |
| `woc_headless/Cargo.toml` | 9 | 只依赖`woc_runtime` |
| `woc_headless/src/main.rs` | 8 | 只打印Headless identity并退出 |
| `woc_server/tests/fixed_tick_driver.rs` | 201 | 6个同步test，全部使用`RecordingVm` test double |

三角色共有5个production Rust文件、286行、9,827 bytes；三个manifest为28行、632 bytes；唯一test文件为201行、6,854 bytes。`cargo metadata --no-deps`确认server有lib、bin和一个integration-test target，bot/headless各只有bin target。`FixedServerTickDriver`、queue sets与`last_failed_input`在native production中都只有定义文件自身命中，没有产品caller。

### 2.2 依赖闭包

`woc_runtime::transaction`是driver唯一的authority wrapper依赖；它仍只有generic `WocProjectVm` trait，production没有实现。`woc_protocol`提供`Command`、`MovementFrame`与fixed input codec，但不拥有authenticated connection或server service。三角色production文件中对`std::net`、TCP/UDP socket、async runtime、tracing/metrics、signal/shutdown、health/admin、journal/checkpoint、persistence、replication、principal与session的有效实现命中均为0。

项目manifest把Bot和Headless与Server一并映射到`server_runtime`，只证明三者要求同一组plugin selection；它没有表达agent policy、observation/action、local simulation、network authority、offscreen capability或terminal policy。README明确说明native层应拥有transport、persistence和executable roles，也明确当前partial authored work不代表playable；当前源码与该声明一致，不应抬高capability状态。

### 2.3 动态证据边界

本轮执行的`cargo metadata --no-deps`只证明manifest与target图可解析，不编译crate。WOC native workspace此前已经稳定复现6个`woc_protocol` compile errors，导致workspace tests 0运行；源码与失败条件未变化，本轮不重复运行同一失败lane。App03、Runtime19与Tooling11保留该动态证据的canonical记录。

## 3. 参考引擎约束

- Unreal `FEngineLoop::Tick`把process heartbeat、time/rate、engine tick、exit和frame diagnostics放进统一产品循环；dedicated server走显式能力分支。`UNetDriver`分离`TickDispatch`、`TickFlush`与`Shutdown`，带connection、queue failsafe、hitch与统计语义。WOC无需复制类层次，但listener ingress、simulation、egress flush和shutdown不能压成一个裸`advance(elapsed_ns)`。
- Bevy `ScheduleRunnerPlugin`在无window runner中仍完成plugin finish/cleanup、持续`app.update()`、等待策略与`AppExit`；fixed time保留独立overstep并在fixed schedule中消费。它不提供WOC的network/persistence答案，但证明headless不等于“打印identity后退出”。
- Fyrox `Executor::run_headless`仍等待resource registry、启用plugins、持续loop、处理lag/throttle、sleep并通过loop controller退出。WOC server/headless必须至少达到同等host lifecycle，再叠加authoritative service语义。
- Godot `--headless`显式切换headless display与dummy audio，但仍运行`Main::iteration`、SceneTree physics/process并执行ordered finish；`MultiplayerAPI::poll`保持network pump边界。无图形能力是provider选择，不是省略main loop。
- Unity Graphics `RenderGraph`不是server framework，本篇不从中臆造network设计。它提供的相关约束是work execution与resource lifetime必须显式绑定；同理server/headless artifact必须明确排除不需要的graphics provider，offscreen render则必须作为独立capability与lifetime进入BuildSet，不能仅靠role名称推断。

## 4. 可保留的正确基础

1. 20 Hz boundary使用整数ns，不把host wall time序列化进authoritative input。
2. constructor拒绝0 catch-up、0 command queue和0 movement queue预算。
3. command batch在mutation前整体检查pending容量和当前窗口duplicate sequence。
4. movement batch逐frame validate，发现同actor重复时整批拒绝。
5. command canonical sort固定actor/generation/sequence/command/payload顺序。
6. catch-up每次调用有上限，剩余accumulator可观察为`backlog_ticks`。
7. VM fault转换为server `Faulted`状态，不会把无效candidate提交成snapshot。
8. test覆盖exact boundary、capacity atomicity、canonical ordering、movement duplicate和VM fault基础路径。

这些基础只能作为未来`AuthoritativeTickService`内部的scheduler/admission primitive，不能直接声明为server product。

## 5. P0：Authoritative Service 硬阻断

### WOC-SVC-P0-001 · 失败前破坏性出队，输入没有可恢复提交语义

`advance()`先取走command/movement并清空dedupe set，再调用runtime transaction。VM或decode/budget fault后，batch不回到pending queue，只保留在`last_failed_input`。该字段不是journal，没有batch ID、BuildSet、tick generation、durability、ack或replay API；下一次`advance()`可用空diagnostic覆盖它。服务不能证明输入最终被拒绝、重放或提交。

重构为`IngressBatch -> PreparedTick -> DurableIntent/Commit -> PublishedOutcome`状态机。destructive dequeue只能发生在prepared record可恢复之后；terminal receipt必须标明accepted/rejected/committed/aborted、tick、world generation和replay cursor。crash/fault恢复必须从journal重建pending与outcome，不能依赖内存diagnostic。

### WOC-SVC-P0-002 · Authority queue没有principal、session、world与actor ownership绑定

public enqueue API接受裸command与movement，dedupe key只有actor id/generation/sequence。任何未来transport adapter都可以在未绑定authenticated principal、connection generation、world/shard、actor lease和permission的情况下把输入送入authority queue。这个缺口不会因“后来接了socket”自动消失。

建立`AuthenticatedIngressEnvelope`与`AuthorityAdmissionReceipt`。在producer admission前校验server/build/schema、principal/session/connection generation、world/shard、actor ownership、command capability、sequence/replay window与rate budget；裸protocol DTO不得成为service public API。

### WOC-SVC-P0-003 · Simulation commit、outcome、replication与durability没有统一代际

成功tick只替换进程内`CommittedSnapshot`，`ServerTickAdvance`只返回committed count与backlog。没有command outcome journal、client ACK cursor、replication snapshot/delta generation、durable event/state checkpoint或publication fence。进程在VM commit之后、network/persistence之前退出时，客户端与恢复状态可观察到不同事实。

定义`AuthorityCommitGeneration`和原子publication protocol：simulation candidate、command outcomes、event journal、snapshot/delta、persistence checkpoint与replication receipt共享tick/generation/BuildSet/schema；明确write-ahead、commit、publish、ack与retire顺序，并为每个中断点提供recovery test。

### WOC-SVC-P0-004 · 服务资源资格只按item计数，无法阻止内存与CPU失控

constructor接受任意`usize`并立即按上限预分配两个Vec；enqueue只检查item count，不检查payload bytes、decoded nodes、canonical sort work、clone amplification、per-principal inflight、backlog age或process memory。调用者已经构造完整Vec后才进入driver，超大candidate的分配成本发生在admission之前。VM预算又是执行后的自报usage，不能中止hang。

把budget移到transport decoder和queue producer：限制frame/aggregate bytes、commands、movement、decode steps、allocation、sort work、per-principal tokens、world queue、tick deadline与process memory；使用reservation receipt和bounded channel。配置本身必须有hard maximum并在分配前验证，OOM/timeout进入可观测terminal policy。

## 6. P1：Artifact、Role 与 Capability Truth

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-SVC-P1-001 | server binary不引用同package library | binary composition root必须构造并运行service host，link-map/target test证明driver可达 |
| WOC-SVC-P1-002 | bot/headless各只依赖`woc_runtime` | 每个artifact声明实际runner、protocol、engine adapter与必要provider依赖 |
| WOC-SVC-P1-003 | Bot/Headless折叠为`server_runtime` | role descriptor分别声明network、authority、agent、render、persistence和terminal policy |
| WOC-SVC-P1-004 | identity valid被唯一成功路径使用 | capability state拆成Configured/Materialized/Started/Ready/Degraded/Terminal |
| WOC-SVC-P1-005 | 无versioned CLI/config schema | 定义listen/world/tick/budget/storage/log/admin/seed/max-tick配置、来源优先级与拒绝未知字段 |
| WOC-SVC-P1-006 | 运行receipt不携BuildSet | startup/ready/tick/terminal绑定engine、package、protocol、world schema与artifact digest |
| WOC-SVC-P1-007 | 无target/platform capability矩阵 | dedicated server、local headless、bot worker、offscreen runner分别构建并验证闭包 |
| WOC-SVC-P1-008 | 没有package/plugin materialization门 | 启动前resolve并admit真实ZrVM/WOC/net/storage provider与版本 |
| WOC-SVC-P1-009 | plugin selection不形成lifecycle | ProductHost按依赖启停、quiesce并拒绝缺provider的ready |
| WOC-SVC-P1-010 | `expect` panic是入口错误策略 | config/materialization/start failure返回稳定exit code与structured terminal receipt |
| WOC-SVC-P1-011 | 无starting/ready/draining/terminal协议 | service manager、test harness和orchestrator消费同一operation schema |
| WOC-SVC-P1-012 | 无package/container/service定义 | 产物包含config schema、assets、symbols、health contract、upgrade/rollback与最小权限 |

## 7. P1：Lifecycle、Clock 与 Fixed Schedule

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-SVC-P1-013 | `elapsed_ns`由任意caller注入 | host拥有monotonic clock、sample generation与test-only virtual clock |
| WOC-SVC-P1-014 | 20 Hz写死为裸常量 | qualified simulation profile拥有frequency、period、compatibility与fingerprint |
| WOC-SVC-P1-015 | accumulator使用saturating add | overflow/hitch产生显式fault或clamped receipt，不能静默变成`u64::MAX` |
| WOC-SVC-P1-016 | backlog只有tick数 | 暴露age、wall lag、oldest input、queue/work、catch-up cost与SLO状态 |
| WOC-SVC-P1-017 | 无death-spiral策略 | 定义catch-up、degrade、shed、pause、migrate或terminal阈值及operator action |
| WOC-SVC-P1-018 | catch-up第一tick吞全部pending，后续为空 | ingress记录target tick/arrival generation，明确late-input与multi-tick分配规则 |
| WOC-SVC-P1-019 | 无固定phase graph | network ingest、admission、simulation、outcome、replication、persist、metrics顺序可验证 |
| WOC-SVC-P1-020 | 无pause/drain/stop状态 | lifecycle幂等并禁止drain后新admission，等待inflight commit与flush |
| WOC-SVC-P1-021 | 无OS/service signal owner | Ctrl-C、service stop、fatal、admin drain与orchestrator cancel汇入同一shutdown coordinator |
| WOC-SVC-P1-022 | `runtime_mut()`泄露内部authority owner | driver只暴露受控operation；reload/recover/install snapshot走generation-checked coordinator |
| WOC-SVC-P1-023 | Faulted后没有service recovery policy | 区分retryable world fault、poisoned VM、process fatal、shard evacuation与operator hold |
| WOC-SVC-P1-024 | error只有Rust enum/debug形状 | stable code、operation/tick/world/session context、source chain与redacted diagnostic进入receipt |

## 8. P1：Ingress、Ordering 与 Fairness

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-SVC-P1-025 | command/movement只按item容量 | 同时限制bytes、payload work、actor cardinality、age和per-session share |
| WOC-SVC-P1-026 | 任意capacity直接预分配 | config hard cap、checked reservation、fallible allocation与startup memory budget |
| WOC-SVC-P1-027 | API接收已分配完整Vec | bounded decoder/stream producer在分配前取得queue reservation |
| WOC-SVC-P1-028 | sequence去重只活到pending batch | per-session durable replay window跨tick、reconnect与failover保留 |
| WOC-SVC-P1-029 | dedupe key无world/session/connection | stable ingress key包含tenant/world/shard/principal/session generation |
| WOC-SVC-P1-030 | command-specific validation延后到codec/VM | service admission调用qualified registry并保留validator/schema receipt |
| WOC-SVC-P1-031 | 单一全局Vec无fairness | connection/principal/world队列使用quota、weighted fairness与starvation指标 |
| WOC-SVC-P1-032 | queue full只返回maximum | reject receipt携retry policy、current pressure、server cursor且受anti-amplification约束 |
| WOC-SVC-P1-033 | canonical sort可改变arrival语义 | ordering contract声明actor内序、跨actor独立性、冲突与deterministic tie-break |
| WOC-SVC-P1-034 | 同actor movement一律duplicate | 明确latest-wins/coalesce/sample window；保留被替换sequence与ACK |
| WOC-SVC-P1-035 | movement无跨tickstale/ack窗口 | authority维护last-applied、duplicate/stale/future和resync cursor |
| WOC-SVC-P1-036 | driver只能`&mut self`同步调用 | transport通过bounded MPSC/partition inbox、wakeup与shutdown fencing接入 |

## 9. P1：Commit、Replication、Persistence 与 Recovery

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-SVC-P1-037 | `last_failed_input`无限期保留完整payload | retention/redaction/encryption/size budget明确，secret与PII不得进入Debug/heap dump |
| WOC-SVC-P1-038 | advance receipt没有tick范围/digest | 返回每个committed tick的generation、state/event/outcome digest与duration |
| WOC-SVC-P1-039 | 无per-command terminal outcome | accepted/rejected/duplicate/stale/deferred/committed与reason成为可重放journal |
| WOC-SVC-P1-040 | snapshot没有publication owner | immutable snapshot generation通过publisher安装，reader pin/retire有界 |
| WOC-SVC-P1-041 | 无delta与relevancy pipeline | replication owner按connection baseline、interest、budget与schema生成delta |
| WOC-SVC-P1-042 | 无event/write-ahead journal | intent、RNG、world event、outcome与commit marker形成checksum记录 |
| WOC-SVC-P1-043 | 无checkpoint与compaction | snapshot checkpoint、journal segment、retention、verification与atomic publish明确 |
| WOC-SVC-P1-044 | 无durability级别 | sync/async/ephemeral policy显式进入session与receipt，不能默认假定落盘 |
| WOC-SVC-P1-045 | 无cold-start/recovery install | 从qualified checkpoint+journal恢复并校验BuildSet/schema/tick/digest |
| WOC-SVC-P1-046 | 无world/shard generation | 每个world独立authority owner、clock、queue、storage、replication与teardown |
| WOC-SVC-P1-047 | 无failover/lease fencing | active authority lease、epoch与stale-writer rejection防止双主提交 |
| WOC-SVC-P1-048 | hot reload与service commit无编排 | drain admission、checkpoint、VM migrate、generation switch、replication resync按事务执行 |

## 10. P1：Bot 与 Headless Runner

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-SVC-P1-049 | Bot没有library/composition root | `BotProductHost`拥有agent、connection或local environment、clock与terminal lifecycle |
| WOC-SVC-P1-050 | 无agent policy contract | policy version、model/artifact digest、determinism与resource budget进入BuildSet |
| WOC-SVC-P1-051 | 无typed observation | observation schema、visibility、history、mask、bounds与generation可验证 |
| WOC-SVC-P1-052 | 无typed action adapter | action映射command registry并执行authority/cooldown/payload validation |
| WOC-SVC-P1-053 | 无seed/RNG ownership | environment、policy与sampling stream分离，seed/draw cursor进入replay |
| WOC-SVC-P1-054 | 无episode/rollout lifecycle | reset/start/step/terminal/truncated/reward/result形成versioned receipt |
| WOC-SVC-P1-055 | 无remote与in-process模式边界 | network bot和local deterministic bot使用同语义adapter并分别资格化 |
| WOC-SVC-P1-056 | Bot无principal/account/rate policy | test principal、production bot权限、anti-cheat与audit明确隔离 |
| WOC-SVC-P1-057 | Headless没有simulation runner | 加载package/world，按virtual或real clock持续step并有明确退出条件 |
| WOC-SVC-P1-058 | 无snapshot input/output合同 | headless job可安装qualified state并发布result/artifact，不用stdout临时文本 |
| WOC-SVC-P1-059 | 无max tick/deadline/cancel | CLI/job envelope定义steps、deadline、cancel、checkpoint与terminal reason |
| WOC-SVC-P1-060 | 无batch/vectorized environment | 批量runner隔离world/seed/budget，支持公平调度与per-environment failure |

## 11. P1：Operations、Security、Test 与 Performance

| ID | 当前差距 | 需要重构 |
|---|---|---|
| WOC-SVC-P1-061 | 无structured logging/tracing/metrics | process/world/tick/session/connection correlation与bounded cardinality统一 |
| WOC-SVC-P1-062 | 无startup/readiness/liveness | readiness要求provider/world/listener/storage可用；liveness不掩盖stalled tick |
| WOC-SVC-P1-063 | 无admin/maintenance plane | RBAC保护的drain、snapshot、status、kick、reload、diagnostic operation有audit |
| WOC-SVC-P1-064 | 无graceful shutdown evidence | 停止admission、完成/中止tick、flush replication、checkpoint、close connections、卸载provider |
| WOC-SVC-P1-065 | 无secret/RBAC/security audit | key来源、rotation、redaction、least privilege、auth failure与abuse signal明确 |
| WOC-SVC-P1-066 | 无storage/network故障测试 | disk full、fsync fail、corruption、partition、reorder、duplicate、slow peer均可恢复 |
| WOC-SVC-P1-067 | 无binary/process journey | 启动真实artifact、等ready、连接、发送、观察、drain并核对exit/receipt |
| WOC-SVC-P1-068 | test VM只返回常量state | 真实ZrVM/package/current BuildSet进入server integration lane |
| WOC-SVC-P1-069 | scheduler edge覆盖不足 | overflow、huge hitch、fault后再advance、diagnostic retention、multi-world与shutdown race测试 |
| WOC-SVC-P1-070 | 无fuzz/property/concurrency test | codec-admission-queue、ordering、replay、journal recovery与channel race系统化生成 |
| WOC-SVC-P1-071 | 无benchmark/soak/capacity证据 | tick p50/p95/p99、queue age、alloc/copy、RSS、connections/worlds与24h+ soak |
| WOC-SVC-P1-072 | 无packaged release gate | clean artifact在目标OS/service/container运行，证据绑定BuildSet并纳入release admission |

## 12. P2：维护性与可演进性

| ID | 当前差距 | 改进方向 |
|---|---|---|
| WOC-SVC-P2-001 | `SERVER_TICK_NS`是裸`u64` | 使用validated fixed-step type与profile identity |
| WOC-SVC-P2-002 | 三组error只派生Debug | 实现stable code、Display/Error与source chain |
| WOC-SVC-P2-003 | dedupe key使用匿名tuple | 定义`IngressSequenceKey`并封装namespace/generation |
| WOC-SVC-P2-004 | movement actor key使用匿名tuple | 定义`ActorLeaseKey`，避免与entity ref语义漂移 |
| WOC-SVC-P2-005 | `ServerTickInputBatch`字段完全public | 通过validated constructor和只读访问维护不变量 |
| WOC-SVC-P2-006 | enqueue参数强制ownership Vec | service envelope表达reservation、source与batch metadata |
| WOC-SVC-P2-007 | canonical ordering只有私有函数 | contract文档、schema version与golden test公开记录 |
| WOC-SVC-P2-008 | diagnostic路径多次clone payload | immutable/shared batch buffer与budgeted diagnostic view |
| WOC-SVC-P2-009 | movement先clone再建batch再to_vec | validated batch转移ownership，避免重复全量复制 |
| WOC-SVC-P2-010 | `runtime()`/`runtime_mut()`暴露层次不对称 | service提供窄query/operation port |
| WOC-SVC-P2-011 | test helper重复手写command/movement | versioned fixture builder覆盖边界与invalid variants |
| WOC-SVC-P2-012 | 三个main重复identity reporter | 共享bootstrap parser，仍保留独立role composition |
| WOC-SVC-P2-013 | package description只写bootstrap | manifest描述实际artifact contract与capability |
| WOC-SVC-P2-014 | 角色没有局部README/runbook | 记录config、ports、storage、health、shutdown与failure policy |
| WOC-SVC-P2-015 | 无owner/sequence图 | 维护ingress-to-commit-to-publish与shutdown时序文档 |
| WOC-SVC-P2-016 | public service API无version策略 | schema/operation兼容、deprecation和migration policy显式化 |

## 13. Canonical Owner 与依赖边界

| Owner | 本篇职责 | 依赖/不得吸收 |
|---|---|---|
| `ServerProductHost` | artifact composition、startup、ready、loop、drain、terminal | 不实现gameplay reducer或wire codec |
| `ServerClockScheduler` | monotonic/virtual clock、fixed step、backlog/overload policy | 不拥有connection或world state |
| `ServerIngressCoordinator` | reservation、fairness、principal/session/world binding、queue | command codec/validation registry来自Runtime19 |
| `AuthorityCommitCoordinator` | prepared tick、journal、commit generation、outcome/publication fence | world reducer来自Runtime12-18，storage provider可插拔 |
| `ReplicationCoordinator` | per-connection baseline、interest、delta、ACK/resync | transport/security来自Runtime08E |
| `ServerPersistenceCoordinator` | journal/checkpoint/recovery/compaction/durability | 不重新定义world schema |
| `ServerOperationsHost` | health、admin、metrics、signals、maintenance与shutdown | diagnostic schema复用Runtime Interface/Tooling evidence |
| `BotProductHost` | policy/observation/action/episode、remote/local modes | 不复制server authority rules |
| `HeadlessSimulationHost` | deterministic job、snapshot IO、step/cancel/result | offscreen renderer仅作为显式optional capability |

依赖顺序为 App03/ProductHost与真实VM前置，Runtime19 schema/admission和Runtime08E transport并行收敛，再实现service ingress；Runtime12-18提供qualified world/content owner后才能建立commit/persistence；Tooling10/11消费真实runner receipt建立process、replay与performance证据。不得在`main.rs`里临时拼socket、sleep和JSON log来绕过这些owner。

## 14. 重构里程碑

### M0 · Source 与 Contract Recheck

- 修复并通过native workspace generator/compile基础门；
- 冻结本篇物理清单、App03/Runtime08E/12-19重叠边界；
- 建立role/capability、service operation、authority generation与budget schema。

### M1 · Role Artifact 与 Product Host

- server/bot/headless binary进入各自真实composition root；
- materialize engine/plugin/package BuildSet并发布starting/ready/terminal；
- CLI/config、exit code、signal、drain与packaged artifact闭环。

### M2 · Clock、Ingress 与 Fixed Schedule

- 实现monotonic/virtual clock、fixed phase graph和overload policy；
- transport decoder先取得budget reservation；
- principal/session/world/actor binding、durable replay window与fair queues进入admission。

### M3 · Recoverable Authority Commit

- prepared input在destructive dequeue前写入可恢复journal；
- VM candidate、outcome、snapshot、event与commit generation同代；
- fault/retry/abort/recovery与hot reload都有terminal receipt。

### M4 · Replication 与 Persistence

- 建立baseline/interest/delta/ACK/resync；
- journal/checkpoint/compaction/cold recovery与authority lease fencing闭环；
- crash window、network partition、disk full和stale writer测试通过。

### M5 · Bot 与 Headless Products

- Bot支持versioned policy、observation/action、seed/replay、episode与remote/local模式；
- Headless支持snapshot IO、bounded step、cancel、batch world与result artifact；
- 两者capability和artifact closure与Server明确分离。

### M6 · Operations 与 Qualification

- health/admin/security/metrics/runbook、service/container安装与rollback闭环；
- process journey、fuzz/property/concurrency、24h+ soak与capacity基线通过；
- 所有证据绑定同一BuildSet/schema/workload，方可提升role capability。

## 15. 产品资格门

| Gate | 必须满足 |
|---|---|
| WOC-SVC-G01 | server/bot/headless三个binary都进入真实runner，不再只打印identity |
| WOC-SVC-G02 | ready只在provider、package、world、listener/storage或job input资格完成后发布 |
| WOC-SVC-G03 | fixed clock、phase ordering、late input与overload policy有deterministic test |
| WOC-SVC-G04 | 所有ingress在分配/解码前取得bytes/work/memory reservation |
| WOC-SVC-G05 | principal/session/world/actor ownership与replay window在authority前验证 |
| WOC-SVC-G06 | VM fault不会丢失或重复提交batch，原始terminal receipt可恢复且不可被覆盖 |
| WOC-SVC-G07 | command outcome、snapshot、event、replication、journal共享commit generation |
| WOC-SVC-G08 | crash发生在每个commit阶段时，恢复结果与client ACK一致 |
| WOC-SVC-G09 | replication具baseline、interest、delta、budget、ACK与resync |
| WOC-SVC-G10 | checkpoint/journal checksum、compaction、retention、cold restore与schema migration通过 |
| WOC-SVC-G11 | authority lease/epoch拒绝stale writer与双主提交 |
| WOC-SVC-G12 | drain停止新admission并有界完成tick、publish、persist、connection close和provider unload |
| WOC-SVC-G13 | Bot observation/action/seed/episode/replay和权限合同可版本化验证 |
| WOC-SVC-G14 | Headless snapshot IO、max tick/deadline/cancel/result与batch isolation通过 |
| WOC-SVC-G15 | health/liveness能识别stalled tick，admin操作受RBAC与audit保护 |
| WOC-SVC-G16 | real ZrVM process journey覆盖start/connect/command/replicate/recover/drain/exit |
| WOC-SVC-G17 | fuzz/property/race覆盖codec-admission-queue-journal-recovery组合 |
| WOC-SVC-G18 | 24h+ soak与capacity证据记录tick p99、queue age、RSS、allocation、world/connection规模 |
| WOC-SVC-G19 | packaged artifact在目标OS/service/container clean环境运行并产生可追溯证据 |
| WOC-SVC-G20 | 未满足同硬件、同BuildSet、同world/load、同failure policy前，不宣称性能超过Unreal |

## 16. 状态与非修改声明

| 项目 | 状态 | 证据 |
|---|---|---|
| 物理清单 | review_complete | 3 manifests、5 production Rust、1 test逐文件读取；target图由metadata解析 |
| Finding | review_complete | 4 P0 / 72 P1 / 16 P2，ID在本篇唯一 |
| Owner/里程碑/Gate | design_complete | 9 owner、M0-M6、20项产品资格门 |
| Production implementation | pending | 本轮未修改任何WOC/engine源码、test、manifest或artifact |
| Dynamic validation | blocked_upstream | 复用既有native workspace 6 compile errors；source未变化，不重复失败lane |

本轮只新增审查文档与索引，不修改`woc_server`、`woc_bot`、`woc_headless`、`woc_runtime`、`woc_protocol`、Zr source、tests、manifest、lockfile或generated artifact。实现前必须重新读取最新源码并重跑M0；本篇的行号、命中数与容量只代表2026-08-16工作树证据，不能当成永久契约。

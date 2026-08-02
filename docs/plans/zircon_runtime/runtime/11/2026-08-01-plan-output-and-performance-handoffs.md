# Runtime 11 产出与性能交接归档

> 来源：[11-job-system-task-model.md](../11-job-system-task-model.md) 的直接产出与性能记录。2026-08-01 输出治理将这些记录迁入编号子目录；除 Markdown 相对链接按新目录层级重定位外，历史文字、命令与验证证据保持原样。本归档不代表 milestone 完成。

## 迁入产出记录

- 当前失败交接（`open / Editor full harness Runtime 三池与 asset worker 双预算`）：[`11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md`](../11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md)

- 当前切片（source cubemap direct-Rayon 旁路收编）：[`11/2026-07-16-source-cubemap-task-pool-cutover.md`](../11/2026-07-16-source-cubemap-task-pool-cutover.md)，状态 `runtime_11_source_cubemap_task_pool_cutover_static_and_managed_build_passed_focused_test_pending`。该切片只关闭未分类 direct-Rayon owner；上方 P0 双预算失败交接继续保持 open，父计划仍为 `in_progress`。

- 当前切片（MVP 任务诊断准确性）：[`11/2026-07-17-task-diagnostics-accuracy-current-source.md`](../11/2026-07-17-task-diagnostics-accuracy-current-source.md)，修复 [`11/failure-2026-07-17-task-diagnostics-accuracy.md`](../11/failure-2026-07-17-task-diagnostics-accuracy.md) 所列 detached panic、错误 main-thread 命名与 queue/active/lag/panic/cancel 盲区；父计划在 focused/current-source 验收完成前继续 `in_progress`。

- 当前最高优先失败切片（有界偏好持久化）：[`11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md`](../11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md)，状态 `resolving_failure`。Runtime11 的领域无关 bounded keyed I/O lane、Platform overlay/backend adapter、模块 shutdown ownership 与 WOC consumer hard cut 已实现；Fence prerequisite 在分配前纳入 retained-byte 接纳，连续 Fence 通过最近 Fence 链保持 O(N)，WOC fresh cold-read 由显式 backend read gate 确定性验证 Pending -> Ready。Frameworks05 静态契约 7/7、Runtime11 非 lock 契约 6/6 和 Rust 1.94.1 精确格式门通过，新增 structured nested-lock guard 按 TDD 精确命中唯一旧 lock RED。独立二审提出的源码 Important 均已前向修复，nested lock 由 coordinator 受管生成；最终 current-source 二审、受管 lock/编译/测试 receipt 与 failure return 完成前父计划继续 `in_progress`，不得标 accepted。

- 当前切片（M1 通用终态观察器，2026-07-19）：`JobHandle::on_terminal(...)`、锁外 one-shot delivery、handle-local panic count、dependency continuation ordering 与 focused tests 已实现；continuation 逐项 containment，panic-first 回归证明后续 combined barrier callback 与 observer 仍全部投递，再重抛首个 panic，不需要 production test hook。Rust/Python behavior inventory 与全部当前镜像统一为 27；受管 Cargo 与 successor 独立 review 仍是接受前置，且两个既有 Runtime 11 failure 保持 open。

- 2026-07-18 性能审计交接：`JobHandle::combine`最后依赖终结已从三次状态锁局部降为一次，但diagnostics准确性切片引入的`updates_in_flight/update_epoch`使每次scheduled/enqueued/started/terminal额外执行4次全局共享原子RMW。Runtime11须提供diagnostics off近零bookkeeping、on时worker-local/sharded或有界采样方案，并以1M no-op jobs及1/100/10k fan-in证明吞吐/queue-delay预算；见PERF-MVP-317和`docs/plans/performance/01/2026-07-17-task-system-static-review.md`。

- 2026-07-22 asset worker completion交接：产品唯一请求队列已按`workers * 2`有界且request诊断O(1)，但duplicate waiter不计capacity，completion channel无界，并按waiter深cloneTexture/Mesh payload；owner Drop还同步等待pending。Runtime11按PERF-MVP-498提供共享immutable result ticket、completion entry/bytes/age预算、cancel/deadline/shutdown合同，禁止每observer复制payload；见`11/failure-2026-07-22-asset-worker-shared-completion-backpressure.md`。

- 2026-07-22 asset watch job交接：PERF-MVP-501要求OS callback只发布轻量coalescing generation entry，full scan/import/resource prepare不得在watcher线程同步执行；Runtime11承接有界affected-closure prepare、cancel/supersede/deadline与shutdown drain，Runtime04独占watch truth和短commit。持续事件风暴不得通过私有线程或无界task队列旁路统一预算。

- 2026-07-22 asset importer/cook job交接：PERF-MVP-504要求glTF/OBJ/IBL/font/shader等只消费Runtime04 content/revision source ticket，并在统一bounded CPU/I/O pool完成single-flight parse/decode/cook；同content并发请求共享一个result，queue/in-flight按entry+bytes有界且支持cancel/supersede/shutdown。禁止importer内部同步重开path、direct Rayon或每格式私有线程。

- 2026-07-22 artifact I/O交接：Runtime11为PERF-MVP-506提供有entry/bytes/RSS上限的chunk encode/decode/read/write lane；大payload不得在caller/editor/render线程整块压缩、解压或`fs::write`，同content+schema generation single-flight并支持cancel/shutdown flush。记录queue age、chunk bytes、peak in-flight owners与blocked caller；写失败由Runtime04保留last-good manifest。

- 2026-07-22 VG cook job交接：Runtime11为PERF-MVP-509提供有triangle/bytes/RSS/in-flight上限的leaf bounds、cluster与page encode jobs，按stable ordinal合并且支持cancel/supersede/shutdown；不得在importer caller串行完成整mesh cook或各plugin建立私有pool。feature-off不排队，same content+config并发请求共享Runtime04 single-flight result。

- 2026-07-22 migration I/O交接：Runtime11为PERF-MVP-511/512并行唯一root inventory后的独立document parse/stage/hash，按file/bytes/RSS设硬上限并支持cancel；live commit、危险窗口前state durability与ordered fsync仍由Runtime04 transaction owner串行。禁止每文件detached task、无界全文件Vec或用并行乱序放宽crash合同；Editor10只消费progress/cancel ticket。

- 2026-07-22 zrpack streaming补充：PERF-MVP-513已删除reader初始化的per-asset payload复制/重复hash；Runtime11继续按PERF-MVP-449为Cook→Pack、delta apply与promotion verify提供有chunk/bytes/RSS上限的I/O lane。base/target/delta/rebuilt不得同时整包驻留，same content hash复用Runtime04/506 chunk ticket，cancel/resume只在atomic publish安全边界生效。

- 2026-07-22 texture decode/assembly补充：Runtime11按PERF-MVP-523把external cubemap、array/cube/lightmap、IBL与`.zcube` decode/encode纳入统一bounded CPU/I/O lane，按entry+source/output/scratch bytes限制in-flight；解析后的container/layout ticket只建一次，worker直接写最终chunk，支持cancel/supersede/shutdown。禁止per-face/mip detached任务、caller串行整块转换或私有pool。

- 2026-07-22 project/data/audio asset补充：PERF-MVP-525已删除project document文本中转与重复parse；Runtime11按527为sealed Data/material/model/scene generation提供有bytes/RSS上限的serialize/pretty/atomic-write lane，按528为长音频提供有界decode ring、prefetch/seek/cancel/shutdown。UI/caller/audio callback不得整块转换、等待I/O或绕过统一budget。

- 2026-07-22 plugin scene system调度补充：PERF-MVP-532要求每个World拥有独立plugin system callback state，schedule run路径不得获取跨World共享callback Mutex；Runtime11记录ready/active/queue-delay、callback wall与worker overlap，按SystemParam访问冲突调度，reload/unload用generation quiescence而非等待全局执行锁。不得用detached/private pool绕过统一预算。

- 2026-07-22 plugin catalog candidate build补充：PERF-MVP-537/538的大catalog discovery/reload必须以统一CPU lane执行single-flight candidate projection、diagnostics与compiled project plan，支持supersede/cancel/deadline；主线程只在frame/lifecycle安全点原子commit已完成generation。记录queue age/build wall/rows+edges/bytes与publish count，禁止每plugin detached task、N次全量rebuild或在editor toggle callback同步构建整registry。

- 2026-07-22 native discovery/hot-update job补充：PERF-MVP-539后仍有watcher增量manifest read/TOML parse、DLL open/entry与delta pack staging/promotion在caller同步执行。Runtime11提供按path+generation single-flight的bounded I/O/CPU candidate lane与cancel/supersede/deadline；root/live-host锁只短暂commit，主线程不得同步完成整root report、pack rebuild或逐plugin load。记录queue age、read/parse/open/entry wall、bytes/RSS和last-good age。

- 2026-07-22 export materialize补充：PERF-MVP-547已完成package inventory早停、ZIP单次file walk与borrowed export-row index，但generated/native文件仍由caller串行无条件覆盖。Runtime11联动Plugins09/Editor15让同一export generation的changed file pairs进入有entry/read/write/scratch bytes上限的I/O lane，按stable path顺序commit report；unchanged write/copy=0，cancel/失败不发布成功generation。参考Unreal AutomationTool bounded copy，但并行度必须服从Zircon统一预算，禁止默认64线程或每阶段私有pool。

- 2026-07-22 EditorJobSystem背压补充：现有priority/category/dependency准入索引、类别配额与progress coalesce继续复用runtime scheduler；Runtime11/Editor14为submit、lifecycle、progress三类队列定义entry+bytes+oldest-age硬预算和shutdown drain。稳定label/spec/result使用共享owner，event state批量提交，主线程只按count/time/age泵；terminal事件必须保留，不得以无界内存换顺序或另建editor私有线程池。

- 2026-07-22 Play snapshot/output补充：PERF-MVP-550把World→DynamicScene→pretty JSON与atomic snapshot write/spawn纳入有source/output/scratch bytes、cancel/deadline的CPU+I/O ticket，主线程只commit generation；PERF-MVP-552把stdout/stderr接blocking-I/O owner，限制line/queue bytes与count/time/age。禁止per-Play无预算reader thread、单行无界Vec或controller/active锁内serialize/fsync/join/cleanup。

- 2026-07-22 Editor asset import single-flight补充：PERF-MVP-555要求Editor09按UUID+source/import generation
  合并watch/digest/manual等价请求并共享typed ticket；Runtime11/Editor14让重复observer、pending admission与completion
  分别服从entry+bytes+oldest-age预算，actual Runtime04 import每generation至多一次。禁止以同URI mutex串行一百万个
  重复job、扩大无界queue或建立asset私有worker规避统一调度。

- 2026-07-22 script build/export inventory补充：Runtime11为PERF-MVP-557提供Editor13 generation-keyed
  compile single-flight、first-event deadline及request entry/bytes/age预算；为PERF-MVP-071提供export cache
  persistence ticket，限制encoded/write bytes与deadline并在显式commit执行，`ExportGenerationInventory::drop`
  不得做clone/pretty encode/write/fsync。现有64KiB streaming hash保留，禁止换成后台whole-file Vec。

- 2026-07-22 export output durability补充：Runtime11为PERF-MVP-558提供按artifact bytes/deadline有界的flush/fsync/manifest atomic-commit ticket，stdout/stderr/manifest可合并durability barrier但不得丢terminal或完整log；reader/caller不串行等待三次sync，禁止用私有无界I/O线程掩盖慢盘。

- 2026-07-30 terminal/timer调度补充：PERF-MVP-585要求`JobState::publish_terminal`不在完成worker无预算同步展开全部continuation/observer，深chain用bounded trampoline避免递归栈，宽fan-out/慢observer走统一affinity lane；`TaskTimer`线程只发布到期ticket，不直接串行执行未知时长callback。以1/100/10k chain/fan-out、0/1/100ms observer/callback、same-deadline timer记录stack depth、completion-thread wall、ready age和lateness。低核三池oversubscription沿Bevy策略保持证据门，WPR未证明有害前不改线程分配，但报告必须区分逻辑预算与实际worker总数。

- 2026-07-30 settings persistence交接：Runtime11为PERF-MVP-590提供共享bounded atomic-persistence ticket，按scope/key+generation latest coalesce并限制queue entry/value/encoded bytes、oldest age与in-flight writer；序列化/写/fsync/rename不在UI caller执行，flush/shutdown有显式terminal与durability/error/retry语义。唯一registry/generation、typed slot、no-op event与read-your-write由Editor17持有；Runtime11不得建立settings私有pool、复制完整registry或用无界后台整库clone。keys `1/1K/100K`、same-key `1/1K/1M`、value `0/1KiB/1MiB`、filesystem `0/10ms/2s`、writers `1/16`要求UI filesystem wall=0、queued full-registry payload=0、queue/RSS硬有界且crash old/new、cancel/flush/shutdown通过。

- 2026-07-30 editor project F0交接：PERF-MVP-075/100/499要求同一generation的project scan/import、Editor09 meta/reference/catalog candidate、recent validation/open promotion、workspace/settings/scene detail与native discovery/load进入共享bounded CPU/I/O ticket，支持single-flight、cancel/supersede/deadline和shutdown。UI caller只commit ready/last-good handle；不得为project、catalog、recent或plugin另建私有pool/无界queue。assets `1/1K/100K`、recent `0/1/8/1K`、plugins `0/1/100/1K`记录queue entry/bytes/age、worker overlap、caller wall与first-frame p95；证据见`../../performance/01/2026-07-30-editor-ui-host-startup-project-current-review.md`。


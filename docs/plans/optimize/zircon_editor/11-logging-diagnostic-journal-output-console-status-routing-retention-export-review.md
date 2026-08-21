---
related_code:
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/context
  - zircon_editor/src/core/script_build
  - zircon_editor/src/core/play
  - zircon_editor/src/core/document
  - zircon_editor/src/core/notifications
  - zircon_editor/src/ui/activity
  - zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot.rs
  - zircon_editor/src/ui/retained_host/console_output.rs
  - zircon_editor/src/ui/retained_host/console_output
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/console_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/console_output.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/console.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/console.rs
  - zircon_editor/assets/ui/editor/console.zui
  - zircon_editor/assets/ui/editor/host/console_body.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDeviceRedirector.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Logging/TokenizedMessage.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/IMessageLogListing.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/Model/MessageLogListingModel.h
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Public/OutputLogSettings.h
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Public/OutputLogModule.h
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Private/SOutputLog.cpp
  - dev/godot/editor/editor_log.h
  - dev/godot/editor/editor_log.cpp
  - dev/bevy/crates/bevy_log/src/lib.rs
  - dev/Fyrox/fyrox-core/src/log.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/MessageManager.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/VFXErrorManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 11 · Logging、Diagnostic Journal、Output Console、Status Routing、Retention 与 Export 工程化差距

## 1. 结论

Zircon Editor的logging core已经不是空壳：它有受entry数和估算bytes双重约束的内存store、永不因clear复用的单调sequence、typed source/severity/jump、事件队列的backpressure/resync、可重入sink测试，以及项目级rolling file。Console也有真实retained template、severity/source过滤、follow-tail、固定行高窗口裁剪和jump action。这些基础值得保留。

但当前系统还不是工程级诊断基础设施。它同时存在`EditorLogService`、`tracing`、`EditorConsoleHistory/status line`、runtime diagnostics snapshot和通知系统等多条平行通道，没有进程级record authority、统一schema、可靠异步fan-out或产品级query surface。最严重的三个断点是：

1. 每条日志都在全局emission mutex内同步创建目录、检查metadata、打开文件、写入并`flush`；调用方随后还同步drain事件sink。日志风暴、慢盘、杀毒扫描或message-bus背压可以直接冻结Editor UI、job worker或Play控制路径，日志本身改变被测系统性能。
2. 生产Editor没有注册`EditorTopic::log()` consumer；普通record的零subscriber发布仍被映射为`Delivered`，只有resync分支会返回`NotConfigured`。同时317处status写入、少量`tracing`、runtime diagnostics和绝大多数插件/import路径不进入Console，导致“成功投递”与用户实际看见的记录完全脱节。
3. rolling file没有总容量/年龄配额、session/process/build identity、跨进程协调、崩溃flush或健康状态；配置阶段不探测I/O，首次append才失败，而绝大多数caller丢弃`persistence_error`。启动和project-open早期日志又发生在file sink配置之前。磁盘可无界增长，关键失败也可无声丢失。

本报告记录3个P0、57个P1、12个P2，给出M0-M6重构路线与28个验收门。没有修改生产代码。上一轮同一工作树的`zircon_editor --lib`测试编译已在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；结论来自逐文件静态调用链、现有测试源码和参考引擎源码，不得描述成动态测试通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| logging core | 13 / 1,889 / 59,995 | E3：config、entry/source/severity/jump、record/store、rolling file、service/filter/error及16项test attributes；fingerprint `4c94960e...a08aa932` |
| logging产品接入 | 63 / 12,694 / 464,755 | E3：builder、message sink、project/play/build/host producers、activity console、retained projection与两个Console资产；fingerprint `93c33665...f32ecab` |
| status/tracing绕行面 | 131 / 13,268 / 492,332 | E2 inventory、代表链E3：生产`set_status_line(_with_level)`与`tracing::`调用集合；fingerprint `ca481c55...d1a49c` |
| selected combined scope | 192 / 23,899 / 873,142 | 当前工作树去重集合；fingerprint `4225cab0...d37a0b` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256；它只标识本轮阅读集合，不是ABI、schema、event identity或兼容性hash。

product集合由生产文件中`EditorLogService/LogEntry/LogSource/activity_log/ConsoleOutputSnapshot/ConsoleMessageFilter/ConsoleSourceFilter`引用闭包构成，并显式加入Console ZUI资产；排除dedicated test路径。绕行集合由生产`set_status_line(_with_level)`和`tracing::`调用文件构成。集合有交集，combined已去重。

### 2.2 在途文件与验证隔离

成文时`ui/workbench/snapshot/data/console_output_snapshot.rs`只有import reorder，`ui/retained_host/console_output.rs`只有函数签名换行；二者属于当前工作树其他在途改动，本报告没有回退、格式化、暂存或提交。logging core和主要projection未显示修改。因工作树持续变化且动态测试仍有已知编译阻断，实施前必须重取源码、fingerprint、test inventory和真实窗口证据，故`source_recheck_required=true`。

证据等级：

- E3：`emit -> store -> rolling file -> event queue/sink`、builder配置、message bus零subscriber行为、project sink lifecycle、Console snapshot/filter/jump/clear链逐文件闭环。
- E3：13个logging core文件和全部16个test attributes逐项阅读，0 ignored。
- E2：131个status/tracing绕行文件inventory、生产subscriber/producer/command absence search和参考实现对照。
- 未覆盖：真实Editor窗口压力、慢盘/只读盘/磁盘满、进程崩溃、跨进程同时写、读屏、百万记录查询、remote/headless transport和重启恢复。

### 2.3 本轮追踪的生产链

1. `EditorBuilder`创建默认`EditorLogService`，安装一个`EditorMessageLogEventSink`；消息编码为私有JSON topic `zircon.editor.log.recorded.v1/resync.v1`。
2. producer调用`emit`；service在同一emission mutex中先push内存store，再同步append rolling file。
3. emission mutex释放后，原调用线程继续同步drain bounded event queue并调用唯一sink；reentrant emit依靠“先解锁再callback”避免自锁。
4. message bus无subscriber时返回空的成功report；普通record sink把零delivered/零error映射为`Delivered`，resync分支却返回`NotConfigured`。
5. production没有注册log topic consumer；Console snapshot不是靠topic增量更新，而是在Editor/chrome snapshot路径重新扫描store。
6. `activity_log_console_output`按filter clone全量store并拼接text/level/jump，随后`ConsoleOutputSnapshot::activity`只保留最后256行。
7. retained Console根据行文本生成最多512个row node，paint阶段再按18px固定行高裁剪；jump action按当前record sequence回查store。
8. `Clear`同时清内存log store和隐藏legacy console history，不清disk；project close关闭file sink，早期open日志在sink启用前发生。

## 3. 已有工程基础，重构时必须保留

### 3.1 有界内存和稳定sequence

- `EditorLogStore`同时约束record count和estimated bytes，默认2,048条/4 MiB；单条message限制8 KiB。
- record sequence使用checked increment，`clear`不复用旧sequence，降低旧jump误命中新record的风险。
- store snapshot、filter和record lookup均在统一mutex保护下，不暴露内部可变容器。

### 3.2 Typed source、severity与jump种子

- source至少区分Editor、Runtime、Play、Plugin、Import和ScriptBuild，Play携`PlayInstanceId`，plugin保留detail。
- severity不是靠显示字符串保存；jump至少分Asset和ScriptLocation，script build可以把file/line/column传到record。
- `LogEntry`与`LogRecord`分开，store负责赋sequence，caller不能伪造已提交record identity。

### 3.3 Backpressure、resync与可重入测试

- event queue同时有entry和estimated byte上限，溢出后进入resync而非继续无界堆积。
- sink delivery发生在emission mutex外，现有测试验证reentrant emit不会因同一锁死锁。
- event delivery report显式区分delivered/backpressured/resync，已经具备升级为真实delivery receipt的接口种子。

### 3.4 Console并非纯静态占位

- Workbench Console有All/Error/Warning/Info和source控制、Clear、scroll、follow-tail、jump action。
- line projection限制最大节点数，paint只遍历可见行加overscan；稳定尺寸避免文本变化推动整个pane布局。
- script build diagnostics有generation/request/step cursor，能按编译批次顺序发出typed location。

这些基础应进入新的process-wide Diagnostic Router和queryable Journal，不能在其旁边再加另一套“更完整日志”。

## 4. P0：热路径阻塞、交付真实性与持久化可靠性

### E-LOG-P0-01 · 每条日志在全局锁内同步flush磁盘，日志风暴可以冻结产品

`EditorLogService::emit`持有全局`emission` mutex时调用store push和`RollingFileLogSink::append`。append每次都会检查/创建目录，循环读取segment metadata，按需打开append handle，`write_all`并`flush`。这不只是“写日志有一点开销”：任意UI action、asset/import callback、Play process reader或job completion只要emit，就会把其线程延迟绑定到文件系统；其他producer同时被同一mutex串行阻塞。杀毒扫描、网络盘、磁盘满、目录权限变化和长路径错误都会放大为Editor卡顿。

mutex释放后，caller还要同步drain event queue并执行唯一sink；慢subscriber同样把延迟反压到producer。当前queue只缓冲“重入或并发时尚未被某个caller取走的record”，并不是独立consumer worker。目标必须是：producer只做有界、不可阻塞的MPSC admission和必要的emergency fallback；文件、UI、telemetry各由独立worker批量消费，拥有deadline、batch、flush policy、backpressure和shutdown fence。critical/fatal路径另设明确同步panic spool，而不是让所有info都支付崩溃日志成本。

### E-LOG-P0-02 · 没有进程级诊断authority，事件显示为Delivered但生产中无人接收

builder安装`EditorMessageLogEventSink`，但production注册的Editor topic只有scene inspection等通道；`EditorTopic::log()`订阅只出现在builder测试。message bus对零subscriber返回空成功dispatch report，普通record `publish`先判断backpressure为空并映射为`Delivered`；同一sink的resync分支反而会检查delivered为空并返回`NotConfigured`。所以普通`LogWriteReport.event_delivery=Delivered`可以精确地表示“发给了零个消费者”。Console也不通过这个event更新，而依赖未来某次全snapshot重新扫描。

与此同时，生产存在数百处status-line写入、另一小组`tracing::`调用、runtime独立diagnostics snapshot、notification center journal规划和零散child-process字符串输出。Plugin与Import虽有`LogSource`枚举，却没有真实producer adoption。故障可能只在status bar短暂出现、只进入未安装subscriber的tracing layer、只留在runtime snapshot，或在下一帧被覆盖。目标必须建立唯一`DiagnosticRecord`权威和process-wide router：tracing、runtime、plugin/import/build/play、status摘要与notification投递都共享identity/correlation；每个sink返回“被谁接收、落到哪个cursor、是否丢弃/降级”，零subscriber绝不能叫Delivered。

### E-LOG-P0-03 · 日志文件无总量治理且持久化失败被静默丢弃，既可耗尽磁盘又保不住事故证据

rolling file只按单segment byte阈值换文件，文件名只有epoch day和segment。没有目录总bytes、最大年龄、文件数、压缩、GC、session/process/build identity、manifest、跨进程锁或atomic ownership。首条record甚至允许超过segment limit，因为rotation只在`current_size > 0`时触发。项目反复使用会永久积累`.zircon/logs`，任意工程的ignore policy也不受Editor保证。

反向可靠性同样缺失：配置file sink只构造path，不探测create/open/write；project-open diagnostic在host完成file-sink配置前发出；startup、close后和配置失败日志只留内存。append错误只是`LogWriteReport.persistence_error: String`，大多数caller用`let _ = logs.emit(...)`丢弃。没有health state、operator banner、retry/backoff、emergency file、shutdown drain或panic/crash fence。目标必须把持久化变成受配额的versioned journal sink，显式报告durable cursor和degraded state，并在启动早期使用session spool后原子挂接到project journal。

## 5. P1：schema、routing、store、persistence与Console产品缺口

### 5.1 Record schema、identity与上下文

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-LOG-P1-01 | severity只有Info/Warning/Error，缺Trace/Debug/Critical/Fatal/Performance/Audit等工程语义。 | 分离diagnostic severity、outcome、verbosity和presentation tone；未知等级有forward-compatible映射。 |
| E-LOG-P1-02 | 每条记录强制`timestamp_frame: u64`，frame 0被用作非frame事件占位。 | optional frame/engine tick，加独立wall-clock和monotonic timestamp，明确clock domain。 |
| E-LOG-P1-03 | 没有process/thread/task/span/request/session identity。 | typed execution context，支持thread、job、span、request、session和parent correlation。 |
| E-LOG-P1-04 | 没有stable event code、schema version或producer contract。 | `DiagnosticCode` + payload schema/version，message只作为本地化/显示层。 |
| E-LOG-P1-05 | 除message和单jump外没有structured fields，查询只能解析文本。 | bounded typed fields/tokens，字段具privacy、display和index policy。 |
| E-LOG-P1-06 | source只有六个粗粒度channel，无法表达renderer/compiler/asset subsystem/category。 | owner + subsystem + category正交identity，支持稳定过滤与quota。 |
| E-LOG-P1-07 | plugin detail是任意非空字符串，没有package ID、generation或owner lease。 | capability-bound plugin producer，绑定package/version/generation，卸载后不能继续发布。 |
| E-LOG-P1-08 | project/document/asset/entity/node/job/play process上下文缺失。 | structured subject/context IDs和cross-record correlation graph。 |
| E-LOG-P1-09 | 一条record最多一个jump，无法组合asset、file、node、docs和fix。 | tokenized message与0..N typed actions；action复用Command08授权/provenance。 |
| E-LOG-P1-10 | jump仅验证非空；路径未canonicalize，line/column可为0，target生命周期未知。 | normalized target、1-based bounds、owner generation和执行时revalidation。 |

### 5.2 Ingestion、routing与producer adoption

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-LOG-P1-11 | `tracing`没有桥接到Editor journal，生产也未发现统一`tracing_subscriber`安装。 | process startup安装唯一subscriber/layer，转换span/event到Diagnostic Router并保留target。 |
| E-LOG-P1-12 | `set_status_line(_with_level)`形成隐藏legacy history；真实snapshot随后用activity log覆盖Console。 | status是journal query的短摘要sink，不再维护第二份不可见history。 |
| E-LOG-P1-13 | runtime diagnostics是另一套当前snapshot，没有历史、统一cursor或Editor correlation。 | runtime通过versioned bridge发布同一record schema；snapshot diagnostics作为provider state单独建模。 |
| E-LOG-P1-14 | plugin manager/lifecycle错误未系统进入Plugin channel。 | plugin load/admission/start/stop/crash有稳定codes、owner context和health listing。 |
| E-LOG-P1-15 | Import channel在生产没有producer。 | import/reimport/thumbnail/DDC pipeline按operation/asset/generation发布records。 |
| E-LOG-P1-16 | Play stdout/stderr通过字符串前缀判severity，stderr/output都只映射Warning且没有Error。 | child protocol携stream、level、timestamp、process/session和structured payload；raw bytes有encoding/truncation policy。 |
| E-LOG-P1-17 | save/export/autosave/render/UI等大量error只写status。 | 建立adoption matrix和lint/architecture test：用户可见失败必须产生durable record。 |
| E-LOG-P1-18 | service只有一个可替换event sink；file sink又是特殊内嵌路径。 | 多sink registry，每sink独立queue、policy、health、cursor与lifecycle。 |
| E-LOG-P1-19 | production没有log topic subscriber或可靠Console invalidation。 | Console订阅typed delta/cursor；断连后按journal cursor resync。 |
| E-LOG-P1-20 | 普通record零subscriber dispatch被报告为Delivered，resync却返回NotConfigured。 | recorded/resync统一delivery state machine，区分NoConsumer/Accepted/Persisted/Displayed/Dropped并携sink identity。 |
| E-LOG-P1-21 | recorded/resync使用builder内私有手写JSON schema。 | 公共versioned DTO/generated codec，设字段/bytes/depth预算和兼容性测试。 |

### 5.3 Store、query、cursor与delivery semantics

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-LOG-P1-22 | store按FIFO severity-blind eviction；debug storm可挤掉最后一条fatal。 | per-severity/owner reserve与retention policy，terminal/high severity受保护且eviction可观测。 |
| E-LOG-P1-23 | memory cap使用手工estimated bytes，不计Arc/VecDeque/allocator/index/sink副本。 | 实测/保守accounting、arena或segmented storage，并公开resident high-water。 |
| E-LOG-P1-24 | dropped只累计总数，没有gap marker、severity/source范围或first/last sequence。 | journal写入typed loss record，包含reason、range、counts和affected dimensions。 |
| E-LOG-P1-25 | snapshot clone全部records，没有page/cursor/tail/query budget。 | immutable segmented journal + cursor/window query，支持deadline/cancel/max bytes。 |
| E-LOG-P1-26 | `record(sequence)`线性扫描。 | sequence -> segment/offset索引，O(1)或O(log n) lookup并有benchmark门。 |
| E-LOG-P1-27 | sequence只在当前service实例内唯一，没有session epoch。 | `RecordId { session_id, sequence }`，重启、project switch和多进程不冲突。 |
| E-LOG-P1-28 | Clear直接删除内存authority但磁盘仍保留，没有clear marker或view/session语义。 | `ClearView`、`ArchiveSession`、`DeleteRetainedData`分型并经过权限/审计。 |
| E-LOG-P1-29 | resync只提供`through_sequence`；若store已evict，consumer无法证明缺口。 | receipt携oldest/newest/session epoch和loss ranges；cursor expiry显式返回。 |
| E-LOG-P1-30 | queue byte统计仍是record估算，不含序列化payload、sink wrapper和consumer副本。 | 每lane准确budget和global admission cap，压力指标进入diagnostics。 |
| E-LOG-P1-31 | replace sink没有detach barrier；pending records会被新旧sink语义混接。 | sink registration generation、drain/abort policy和atomic cutover receipt。 |
| E-LOG-P1-32 | sink callback同步且没有panic isolation、deadline或熔断。 | worker隔离、catch boundary、deadline/backoff/circuit breaker和emergency diagnostic path。 |

### 5.4 Persistence、retention与operations

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-LOG-P1-33 | 每条record执行目录/metadata/open/write/flush，支持P0热路径阻塞。 | 长寿命writer、batch/interval/critical flush策略和有界MPSC；producer latency设硬门。 |
| E-LOG-P1-34 | segment只用epoch day命名，record line自身没有wall time。 | session manifest记录start/end/build/process/project；record保存UTC和monotonic offset。 |
| E-LOG-P1-35 | 首record可超过max segment。 | admission前按encoded bytes校验；oversize payload截断/sidecar并产生明确marker。 |
| E-LOG-P1-36 | 没有总bytes、age、file count、GC或compression。 | project/global quota、age tier、压缩、low-disk response和可审计GC。 |
| E-LOG-P1-37 | 文件无session/process/build ID且无cross-process locking。 | per-process session spool + manifest/lease，merge由明确coordinator执行。 |
| E-LOG-P1-38 | ad-hoc `key=value`文本没有版本/parser；只转义反斜杠和CR/LF。 | versioned JSONL或binary journal，canonical encoding、checksum、length和recovery scan。 |
| E-LOG-P1-39 | 没有durable cursor、shutdown drain、panic flush或crash spool。 | flush fence、deadline、panic-safe emergency buffer及Crash报告关联；超时明确标记。 |
| E-LOG-P1-40 | 配置file sink不做I/O probe，失败延迟到第一条日志。 | configure返回health/lease并执行非破坏probe；状态变化在UI/operator surface可见。 |
| E-LOG-P1-41 | persistence error只回传字符串，绝大多数caller丢弃。 | service-owned health state和once/coalesced alert；caller无需递归“记录日志失败”。 |
| E-LOG-P1-42 | 没有redaction、privacy、绝对路径、token/PII或source-control策略。 | field-level sensitivity、default redaction、export scrub、project ignore与安全审计。 |

### 5.5 Console query、presentation、interaction与accessibility

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-LOG-P1-43 | `All`和`Info`都映射minimum Info；`Warning`又包含Error，标签语义与实际不一致。 | 独立severity toggles或明确`minimum severity`控件；合同、文案和查询一致。 |
| E-LOG-P1-44 | activity counts在source/severity过滤后计算；legacy history保留总计，两条语义不一致。 | query response同时给global、filtered、visible和evicted counts，单一authority。 |
| E-LOG-P1-45 | severity/source都是单选，不能组合多个channel或独立开关。 | multi-select facet、exclude/include、reset和可序列化filter state。 |
| E-LOG-P1-46 | 没有search、regex/expression、category、plugin、event code、time/span filter。 | incremental indexed query、debounce/cancel、highlight和明确query budget。 |
| E-LOG-P1-47 | 每次projection先clone/格式化最多2048条，再截最后256条。 | Console直接请求visible/tail window；delta只materialize新增/变化records。 |
| E-LOG-P1-48 | 256行tail cap静默发生，没有omitted/gap/pagination提示。 | `total/window/lost` metadata、load older/newer和cursor expiry row。 |
| E-LOG-P1-49 | node ID使用可见索引`ConsoleOutputLine####`，插入/filter会让identity漂移。 | 以stable record ID/revision reconcile row、focus、selection和actions。 |
| E-LOG-P1-50 | 固定18px且message `elide`，长行无法wrap或水平查看，结构化详情不可见。 | wrap/horizontal modes、details panel、token layout和可测量virtual row height。 |
| E-LOG-P1-51 | 无select/copy/copy all/save/export/open log folder/context menu。 | 完整选择与Clipboard/Export surface，导出走redaction和bounded streaming。 |
| E-LOG-P1-52 | 无pause、follow-tail toggle、bookmark、pin或回到最新记录动作。 | 明确live/follow/paused状态，new-record badge和稳定scroll anchor。 |
| E-LOG-P1-53 | Clear销毁内存authority，不是清视图；disk仍在，用户语义不可预测。 | 分离Clear View、Clear Filter、Archive Session和Delete Data，破坏性动作需确认。 |
| E-LOG-P1-54 | Console不显示record timestamp，也没有absolute/relative/frame模式。 | 可切换时间列，排序/复制/导出保留精度和clock domain。 |
| E-LOG-P1-55 | ScriptLocation jump只open asset并写“Opened at line/column”状态，不真正移动caret。 | typed navigation等待document ready，设置caret/selection并返回可验证receipt。 |
| E-LOG-P1-56 | record被evict后jump action只写status error；action ID没有稳定incident fallback。 | action绑定durable record/subject和revision；失效时仍可打开session file或解释retention结果。 |
| E-LOG-P1-57 | Console缺完整keyboard selection/copy/filter/focus contract、accessible row semantics和live announcement。 | 定义UIA/读屏role、severity/count announcement、快捷键、focus restore和持久化filter state。 |

## 6. P2：产品完整性与维护性

| ID | 当前差距 | 建议处理 |
|---|---|---|
| E-LOG-P2-01 | Console标签、source名、错误和jump反馈存在直接英文字符串。 | 接入typed localization key/args，diagnostic原文与UI chrome分型。 |
| E-LOG-P2-02 | 没有字体、行距、颜色/高对比度和timestamp display偏好。 | 由Editor settings保存可访问presentation profile，主题token而非硬编码色。 |
| E-LOG-P2-03 | source/severity仅靠短文本标签，scan密度和区分度有限。 | 使用一致icon/badge/color，但颜色不作为唯一编码。 |
| E-LOG-P2-04 | filter不能保存preset或在project/session间选择恢复策略。 | named presets和scope明确的settings migration。 |
| E-LOG-P2-05 | Console没有命令输入/history/completion；产品定位也未决定它是Output Log还是Terminal。 | 先明确产品边界；若引入命令，复用Command08权限和completion，不混入shell。 |
| E-LOG-P2-06 | 重复记录不会collapse/group，storm只靠FIFO淘汰。 | event code + normalized fields聚合count/first/latest，可展开原始occurrences。 |
| E-LOG-P2-07 | 所有来源挤在一个listing，没有按Build/Import/Play/Plugin保存page。 | 支持named listings/tabs和session pages，同时保留全局统一query。 |
| E-LOG-P2-08 | message只有纯文本与单jump，没有inline rich tokens或fix-it。 | tokenized text、docs/url/asset/node/fix actions，unknown token安全降级为text。 |
| E-LOG-P2-09 | 无bookmark、annotation或issue关联。 | local annotation作为独立overlay，不修改immutable producer record。 |
| E-LOG-P2-10 | 无JSONL/CSV/support bundle等正式export格式和schema。 | versioned streaming export、manifest、redaction summary和size estimate。 |
| E-LOG-P2-11 | 没有headless/remote/mobile viewer合同。 | router提供相同cursor/query协议，surface按capability降级。 |
| E-LOG-P2-12 | 用户没有storage health、占用、retention policy和打开目录入口。 | Settings/Diagnostics页展示quota、writer health、last durable cursor与清理动作。 |

## 7. 参考引擎对照与适用边界

| 参考 | 仓内可验证能力 | Zircon应吸收的原则 | 不应照搬/不可推断 |
|---|---|---|---|
| Unreal OutputDeviceRedirector | 多output devices、secondary-thread buffering、backlog、异步flush、dedicated primary logging thread、fence和panic/crash路径。 | producer与slow sink解耦；多sink有明确thread/flush/lifecycle合同；崩溃路径独立。 | 头文件不能证明所有平台实现都无阻塞，也不应直接复制全局singleton API。 |
| Unreal Tokenized Message / Message Log | severity、rich token、action/asset/file/url/fix、listing/page/filter/selection/event、export selected/all、duplicate处理和旧page retention。 | event journal与当前provider diagnostics分层；可跳转/修复对象必须typed，listing/query有稳定page。 | Message Log不等于高吞吐trace storage；Zircon仍需自己的bytes/quota/schema和plugin capability。 |
| Unreal Output Log | 独立severity/category/text过滤、settings持久化、timestamp/font/wrap、clear-on-PIE、focus/scroll、drawer/tab和suspend/resume。 | Console必须是可查询产品surface，不是字符串dump；filter/follow/display state有合同。 | 不把控制台命令输入自动等同于shell权限，也不照搬Slate widget结构。 |
| Godot EditorLog | 跨线程error handler经message queue进入UI、默认10k line limit、独立severity toggle/count、search、duplicate collapse、file link、selection/context menu、follow、持久化filters和accessibility name。 | 真实UI工作流至少包括搜索、复制、折叠、follow、链接、配置恢复和线程边界。 | Godot line-oriented RichTextLabel不是Zircon长期typed journal的目标数据结构。 |
| Bevy Log | process-wide tracing subscriber、LogTracer、EnvFilter target/module过滤、custom/fmt layers、trace到error、panic SpanTrace及Chrome/Tracy layers。 | 统一Rust producer path，保留span/target/context，并可组合性能/格式化sink。 | Bevy插件初始化模型不直接解决Editor多project持久化和interactive Message Log。 |
| Fyrox Log | listener fan-out、relative time、level enable、stdout/file、one-shot dedupe和测试verify helpers。 | listener/sink分离、level policy和可测日志合同可作最低基线。 | 其全局mutex/简单文本文件不是本文性能与恢复上限。 |
| Unity Graphics diagnostics | ShaderGraph按provider/node持有消息并选择性clear；VFX区分Compilation/Invalidate origin、model owner、dirty/scheduled regeneration和PerfWarning。 | “当前错误集合”必须按provider/subject/generation replace/invalidate，不能当append-only event log。 | 仓内Graphics包不是完整Unity Editor全局日志系统，不能据此虚构其持久化或console能力。 |

结论不是把所有参考API并在一起。Zircon需要组合Unreal的异步fan-out和tokenized listing、Godot的完整Console工作流、Bevy的process-wide tracing、Unity Graphics的provider generation，再用自己的session/build identity、plugin capability、bounded journal和跨进程host协议收敛。

## 8. 生命周期必须分型

当前实现把不同生命周期都叫“log”。重构必须先定义四种对象，避免一个store承担互相矛盾的语义：

| 类型 | 语义 | 更新方式 | 典型UI |
|---|---|---|---|
| Event Record | 已发生且不可改写的事实，如build step失败、process退出。 | append-only；可由correction record补充，不能原地篡改。 | Output Log、timeline、support bundle。 |
| Provider Diagnostic | 某subject当前仍有效的问题，如shader node error。 | owner + subject + generation原子replace/invalidate。 | Message Log、Inspector badge、Problems pane。 |
| Status Summary | 当前操作的短暂摘要，不是事实authority。 | 新摘要覆盖旧摘要；必须能跳转到record/job。 | status bar。 |
| Notification Delivery | 某record是否需要toast/badge/OS提示的投递状态。 | policy驱动delivered/suppressed/ack/dismiss receipt。 | toast、Notification Center。 |

进度属于Job09 authority；trace span属于execution context；metric属于diagnostic/telemetry采样。它们可以生成event或summary，但不应复制成互相独立的字符串历史。

## 9. 目标架构

```text
tracing / Runtime / Editor / Plugin / Child Process / Provider
                           |
                           v
              Diagnostic Ingress (bounded MPSC)
        schema + owner lease + session + sequence + context
                           |
                           v
                  Diagnostic Router
           /               |                 \
          v                v                  v
 Event Journal      Provider State       Status Projection
 segmented/query    owner+generation      latest summary only
     |     |               |                  |
     |     +---------------+------------------+
     |                     |
     v                     v
 File Writer       Console / Problems / Notifications / Headless
 batch+quota       cursor query + typed actions + delivery receipts
```

关键约束：

- ingress的常规路径不得等待磁盘、UI或网络；有界失败返回typed receipt并触发预留的loss marker。
- journal sequence由单writer或明确sharded ordering contract分配，identity包含session epoch。
- file/UI/telemetry sink各有worker、queue、deadline、health和cutover generation，慢sink不能拖住其他sink。
- Console只消费query/delta，不复制完整文本authority；filter/count/export都在同一query engine上。
- provider diagnostic与event journal共享schema和action token，但生命周期分别是replace/invalidate与append-only。
- status bar与toast只引用record/job/diagnostic ID；关闭UI不删除事实。
- critical crash path使用预分配emergency spool和明确flush fence，不在panic中依赖普通async worker成功。

## 10. 分层重构路线

### M0 · 封闭三项P0并停止扩大旧协议

1. 把rolling append移出emission mutex和producer线程，先建立有界writer queue、batch flush和health state。
2. 修正零subscriber为`NoConsumer`；生产Console建立真实typed subscriber/invalidation或明确删除无效bus sink。
3. file sink配置时probe目录/文件，append失败进入service-owned degraded state并在UI显示一次聚合告警。
4. 增加目录总bytes/age/file count临时配额和启动GC；首record oversize必须受控。
5. 修正Console filter语义和256行静默截断，不再为旧字符串字段增加新功能。

### M1 · 定义统一record schema、session identity与ingress

1. 定义`DiagnosticRecordId/Code/Severity/Verbosity/Owner/Subject/Context/Token/Action`和版本策略。
2. process启动即创建session/build/process manifest；project切换是context变化，不重置进程identity。
3. 建立有界MPSC ingress、admission receipt、loss reserve和per-owner/rate/severity policy。
4. 安装统一tracing layer，将Editor/runtime/plugin/build/play producer逐步迁移。

### M2 · Segmented journal、query engine与异步sink

1. immutable segments按entry/bytes切分，sequence索引支持tail/page/filter/lookup。
2. cursor返回oldest/newest/loss range/session epoch；eviction写typed marker。
3. sink registry支持独立worker、queue、health、deadline、detach barrier和shutdown fence。
4. memory/file/stderr/support-bundle sink共用schema，不重复字符串格式化authority。

### M3 · Durable persistence、恢复与隐私治理

1. versioned journal + checksum/length + manifest，支持partial-tail recovery和durable cursor。
2. 配置global/project/session quota、age tier、compression、low-disk降级和GC receipt。
3. session spool覆盖启动/无project阶段，project attach不丢早期records；多进程使用独立lease。
4. redaction、secret/PII/path policy贯穿producer、disk、copy和export；Crash bundle引用相同session/build ID。

### M4 · Console与Problems产品surface

1. Console使用windowed query和delta，支持search、multi-filter、timestamp、wrap、selection/copy/export和follow/pause。
2. stable record ID维持row identity、scroll anchor、selection、focus和actions。
3. 单独建立provider Problems/Message Log listing，按provider/subject/generation replace/invalidate。
4. ScriptLocation真正导航caret；asset/node/job/plugin token通过typed command执行并返回receipt。

### M5 · 全产品adoption与插件/child protocol

1. 对131个status/tracing绕行文件分类迁移，禁止error只留status。
2. import/save/build/export/play/render/plugin/Hub bridge逐域定义stable codes和context。
3. child process输出升级为versioned framed protocol；raw stdout/stderr作为兼容fallback并有encoding/line/byte budget。
4. plugin获得scope-bound publisher、quota和schema/action manifest；unload撤销provider state并保留event历史。

### M6 · 压力、故障、可访问性与发布验收

1. 压测百万records、100并发producer、慢盘、慢UI、sink crash、plugin flood和cursor eviction。
2. 记录emit p50/p95/p99、alloc、queue high-water、writer throughput、query latency和paint rows。
3. 故障注入磁盘满、permission revoke、partial write、process kill、panic、shutdown timeout和session recovery。
4. Windows优先完成真实Editor截图、键盘、UIA/读屏、200%缩放与large project数据集验收。

## 11. 验收门

1. 常规Info/Warning/Error emit不执行文件I/O、UI callback或网络调用；producer p99绑定硬件/构建profile并进入required benchmark。
2. 100个并发producer与慢盘同时运行时无全局长锁、死锁、UI frame hitch或无界queue。
3. critical/fatal路径可请求durable fence；超时返回明确receipt，不伪装为已持久化。
4. 每个record有session epoch和sequence；project切换、重启、多Editor进程不会identity冲突。
5. 零consumer返回`NoConsumer`；Accepted、Persisted、Displayed、Dropped状态不能互相冒充。
6. tracing、Editor、runtime、plugin/import、build和Play child record可在同一query按correlation筛选。
7. Plugin/Import channel各至少有真实生产producer和端到端行为测试，不靠枚举存在宣称完成。
8. status bar中的每个error都可跳到durable record；状态覆盖不删除历史。
9. debug storm不会淘汰策略保留范围内的Error/Critical；所有eviction产生可查询loss marker。
10. queue/accounting同时限制entry、encoded bytes和resident high-water；恶意8 KiB消息storm受per-owner quota约束。
11. cursor能检测session变更、window eviction和loss range；resync不会把缺口伪装成连续记录。
12. sequence lookup在百万记录数据集满足O(1)/O(log n)与延迟门，不做线性扫描。
13. sink替换/卸载有generation和drain/abort receipt；pending record不会无声发给错误owner。
14. 任一sink panic、hang或持续失败不会阻塞ingress和其他sink，并进入可观测degraded state。
15. 首次配置file sink即验证可写；运行期permission/low-disk变化在一次有界时间内反映到health UI。
16. 日志目录严格遵守bytes/age/file count配额；GC/压缩不删除策略保护的当前事故session。
17. partial-tail、进程kill和系统重启后journal恢复到最后完整record，并报告丢失/截断范围。
18. startup、project-open前、project-close后与panic阶段records都绑定同一session manifest并可查询。
19. 多Editor/Hub/runtime进程同时记录不会覆盖或混写segment；merge保留process identity和ordering limits。
20. secret/token/PII/绝对路径按field policy在disk、Clipboard、export和support bundle中一致脱敏。
21. Console的All、独立severity toggle和minimum-level模式文案与结果严格一致；counts说明统计范围。
22. 10万/100万记录Console只物化可见window；filter/search可取消，稳定帧不clone全journal。
23. tail cap、eviction和query truncation都有可见marker、总数和加载动作，不静默隐藏旧记录。
24. filter变化和新record插入不破坏selected record、keyboard focus或scroll anchor；follow/pause状态明确。
25. 用户可选择/copy/copy all/export/open log；大导出streaming、有进度/cancel、redaction和size budget。
26. ScriptLocation action等待document ready后真实设置caret/selection；失效target给出typed结果而非只写status。
27. provider generation更新会原子移除已修复diagnostics，不误删新generation，也不把当前错误当append-only历史。
28. Windows真实Editor通过鼠标、纯键盘、UI Automation/读屏和200%缩放；timestamp、长行、token和error announcement无重叠、截断歧义或焦点陷阱。

## 12. 与相邻报告的所有权

- Editor10拥有toast/Notification Center delivery、read/dismiss/action history；本文拥有它们引用的durable diagnostic record和delivery receipt基础。
- Editor09拥有job/progress/cancel/terminal authority；本文记录job事件和correlation，不复制job state machine。
- Editor08拥有command identity、authorization、provenance和remote policy；Console/token/fix action必须复用。
- Editor06拥有plugin lifecycle/settings；本文拥有scope-bound diagnostic publisher、quota、provider invalidation和unload日志。
- Editor07拥有Play process/session/child supervision；本文拥有其versioned diagnostic stream和Console query。
- Runtime diagnostics报告拥有runtime metrics/snapshot内部采集；本文拥有跨runtime/editor的record bridge和产品surface。
- Tooling07/10拥有Crash/evidence与test architecture；本文提供session/build/durable cursor，不能代替minidump、symbolication或ValidationSet。

## 13. 本轮未实施内容

本轮只做review和计划记录，没有修改logging core、message bus、tracing subscriber、runtime diagnostics、Console、status routing、rolling files、plugin SDK、child protocol或tests。三个P0均由生产调用链闭环，但慢盘、压力、崩溃恢复、真实窗口、键盘/UIA和跨进程结果仍必须在实施阶段生成新证据。旧`EditorConsoleHistory`、私有JSON topic和pipe/text projection不应作为长期兼容层；新typed schema和journal稳定后应硬切并删除双authority。

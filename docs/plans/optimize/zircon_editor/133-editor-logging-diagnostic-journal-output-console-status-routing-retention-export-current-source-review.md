---
related_code:
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/builder
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/editor_event/workbench/menu_action.rs
  - zircon_editor/src/core/script_build/diagnostics_sink.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/ui/activity/view.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/status.rs
  - zircon_editor/src/ui/layouts/views/console.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/console.rs
  - zircon_editor/src/ui/workbench/activity_log_console_projection.rs
  - zircon_editor/src/ui/workbench/state/console_history.rs
  - zircon_editor/src/ui/workbench/state/console_history
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/console_output_snapshot
  - zircon_editor/src/ui/retained_host/console_output.rs
  - zircon_editor/src/ui/retained_host/console_output
  - zircon_editor/src/ui/retained_host/app/activity_log_jump.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/console.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/console.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/console_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/console_projection
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/console_output.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/console_output
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/console.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics/observability.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics/observability.rs
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
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/121-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-current-source-review.md
  - docs/plans/optimize/zircon_editor/122-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/132-editor-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-current-source-review.md
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
refreshes: docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 133 - Editor Logging、Diagnostic Journal、Output Console、Status Routing、Retention 与 Export 当前源码复核

## 1. 结论

Editor11识别的三项P0仍然成立，但部分旧证据已经被当前实现改变。rolling file现在缓存当天segment的打开句柄，`record(sequence)`改为retained-window offset查询，Console改为直接请求最多256条tail、使用不可变分块generation和delta、稳定record sequence、虚拟化slot及可见行paint；Runtime task diagnostics、model import、Play backend和Script Build也开始进入`EditorLogService`。这些是应保留的工程基础，不能在重构中退回全量字符串拼接或每条重新打开文件。

系统仍不是工程级诊断基础设施。`emit`在全局emission mutex内同步`write_all + flush`，随后由producer线程同步drain唯一event sink；普通record在零订阅者时仍返回`Delivered`。`EditorLogService`、`tracing`、130个status/tracing绕行文件、隐藏`EditorConsoleHistory`、notification和若干标准错误输出仍是平行通道。journal没有session epoch、cursor/page/loss range、总磁盘配额、durable cursor、health state、redaction或crash spool；Console也没有搜索、组合过滤、选择/复制/导出、pause/follow控制、timestamp和真正的script caret导航。

当前还存在一个必须明确删除或接线的临时产品实现：`WorkbenchExtensionConsoleDiagnosticsWorkspace`默认collapsed，表格内容硬编码为`Session_12_10`、`Missing transient view`和`Null object path`；filter/clear/row action只返回固定status/output反馈字符串，不查询或修改日志authority。它不能作为“Diagnostics Workspace已实现”的证据。

本轮保留Editor11原有3个P0、57个P1和12个P2，并新增`E-LOG-P1-58`追踪硬编码Diagnostics Workspace。当前状态为：P0 `3 Open / 0 Partial / 0 Closed`，P1 `50 Open / 5 Partial / 3 Closed`，P2 `12 Open / 0 Partial / 0 Closed`。本轮只做current-source review，没有修改生产代码。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 集合 | 文件 / 行 / 非空行 / bytes | tests / ignored | fingerprint |
|---|---:|---:|---|
| logging core | 17 / 2,526 / 2,269 / 81,293 | 27 / 3 | `50e7299ce76751b197ecdd64a7003c80469402a9012a64c159c9f3691938f432` |
| product projection与接入 | 46 / 13,068 / 12,058 / 493,161 | 104 / 1 | `033c0e653ad0c32d9d03d3a041163db2de66eb7ae60334e4640a01b916a9e67e` |
| status/tracing绕行面 | 130 / 14,851 / 13,859 / 557,532 | 81 / 1 | `878e83640a2fbd8e2062058515dacd4220b6cd3ba1964671ab60eef8e87808af` |
| selected source union | 184 / 27,692 / 25,641 / 1,023,537 | 187 / 5 | `0d4e4296b2086913cd173ffdf126860ee899c2db3dceb351e2969fe4b071d878` |
| reference engines | 13 / 6,460 / 5,496 / 213,966 | 0 / 0 | `ecd3b8e2c398893697ce1927c28d970108f79959b17398db9be97bac4274a918` |
| plan sources | 12 / 3,998 / 3,188 / 411,203 | 0 / 0 | `143d99385f34d86c9e2cb77de001d082924a6d06e152ffd520b2eacc930631a4` |
| total evidence union | 209 / 38,150 / 34,325 / 1,648,706 | 187 / 5 | `b6f7b7622af41dfe84f7ad44ad75fbb69616eac71f74642858382ec8d6eec313` |

fingerprint按相对路径排序，对`path + NUL + per-file SHA-256 + LF`清单再做SHA-256。它标识本轮阅读集合，不是ABI、schema或资产兼容hash。product集合由frontmatter所列当前生产/测试链展开；status/tracing集合排除dedicated `tests/`、`test.rs`和`tests.rs`后，收集直接出现`set_status_line(`、`set_status_line_with_level(`或`tracing::`的生产Rust文件，当前为130个文件、350处匹配。selected source union已去重。

### 2.2 在途源码与证据等级

成文时`core/logging`、`core/context/builder.rs`、`ui/host/editor_activity_log.rs`等存在共享工作树修改，`core/context/builder/event_sinks.rs`、`core/logging/runtime_task_diagnostics`和`ui/workbench/activity_log_console_projection.rs`仍是未跟踪新增路径。本报告读取的是这些当前文件，没有回退、格式化、暂存或提交它们。源码仍可能继续变化，因此`source_recheck_required=true`；实施前必须重取fingerprint、测试和真实窗口证据。

- E3：`emit -> store -> rolling sink -> event queue/sink`、clear/resync、project sink配置、Runtime task pump、Import/Play/Script producer完整调用链。
- E3：Activity Console tail query、generation/delta、virtualized projection/paint/scroll、stable jump action和两个真实Console资产。
- E3：硬编码Console Diagnostics扩展资产、binding/navigation和fixed feedback实现，以及13个指定参考文件。
- E2：130文件/350处status/tracing绕行清单；未发现生产`EditorTopic::log()`订阅者，Console当前直接查询store。
- 未覆盖：真实窗口键盘/读屏/UIA、慢盘/磁盘满/权限撤销、进程崩溃、跨进程写、百万记录查询、remote/headless和重启恢复。本轮没有运行Cargo，因为只写review文档且共享生产源码在途。

## 3. 当前实现中应保留的基础

1. `EditorLogStore`同时受entry数和estimated bytes约束，message限制8 KiB，sequence checked increment且clear后不复用。
2. source至少区分Editor、Runtime、Play、Plugin、Import和ScriptBuild；Play携instance，jump分Asset和ScriptLocation。
3. event queue有entry/byte上限、backpressure、resync marker和失败重试；测试覆盖可重入sink、并发顺序、慢sink和clear resync。
4. rolling sink缓存当前day/segment/file/bytes，稳定segment不再每条执行create-dir/metadata/open；rotation和受管性能证据已有测试。
5. Runtime task bridge保留source cursor、dropped count、source-changed和has-more，并在retained-host tick实际pump；Import、Play和Script Build已有真实producer种子。
6. Console使用`snapshot_tail(256)`，按record sequence建立line/action identity，64行分块generation支持append/trim复用，projection只重绑定变化slot，paint只访问可见行加overscan。
7. Clear、filter、source和jump action已经走command/event/retained-host链，不是只存在于静态资产。

这些基础必须被吸收到process-wide Diagnostic Router、Event Journal和Provider Diagnostic Store中；不能以现有局部优化为理由继续保留多个字符串authority。

## 4. P0复核

### E-LOG-P0-01 - Open - 全局锁内同步flush且producer线程同步推进sink

`EditorLogService::emit`仍先锁全局`emission`，在锁内push store并调用`RollingFileLogSink::append`。当前sink已缓存打开句柄，这是实质改善；但每条仍在producer线程执行clock读取、文本格式化、state mutex、`write_all`和`flush`。慢盘、杀毒、网络工程目录、磁盘满或权限变化仍会阻塞所有producer，并把UI、job、Runtime task pump和Play output drain串行到同一锁上。

释放emission锁后，最先入队的caller仍同步执行`dispatch_pending_events()`并直接调用sink；bounded queue只保护并发/重入积压，不是独立consumer worker。目标必须是有界MPSC ingress、独立file/UI/telemetry worker、batch/interval/critical flush policy、deadline、shutdown fence和panic-safe emergency spool。普通Info路径不得等待磁盘、UI callback或网络。

### E-LOG-P0-02 - Open - 诊断authority仍分裂且零consumer仍被报告Delivered

Builder安装`EditorMessageLogEventSink`，但普通record分支只检查error/drop/backpressure；`delivered()`为空时仍映射为`Delivered`，而resync分支才会返回`NotConfigured`。当前生产Console直接查询canonical memory store，所以旧报告“用户完全看不到record”的说法已不准确；但生产仍未发现log topic subscriber，recorded event的delivery receipt继续说谎，event sink本身也没有承担Console invalidation。

更大的问题仍是authority分裂：130个文件直接写status/tracing，status同时维护不可见的`EditorConsoleHistory`，`tracing`没有统一subscriber/layer接入Editor journal，notification、provider diagnostics和部分直接stderr各自保存或丢弃信息。必须建立唯一versioned ingress/router，使event、provider-current-state、status summary和notification delivery分型但共享record/session/owner/correlation identity；零consumer必须是`NoConsumer`，不能是`Delivered`。

### E-LOG-P0-03 - Open - 持久化无总量治理、durability和可操作health

rolling file仍只有单segment byte阈值和`editor-{epoch_day}-{segment}.log`。没有目录总bytes、age/file-count、compression/GC、session/process/build/project manifest、cross-process lease、checksum/length、partial-tail recovery或durable cursor。首record仍可超过segment上限，因为空文件无条件接纳该line。

配置阶段只构造sink，不probe create/open/write；append错误仅进入当前`LogWriteReport.persistence_error`字符串，大多数caller直接丢弃。service只保存configuration error，不聚合append failure、retry/backoff、last durable sequence或degraded duration。启动早期和无project阶段没有session spool，Clear只删除内存而disk继续存在。必须建立受配额的versioned journal sink、service-owned health、early spool、atomic project attach、shutdown/crash fence和operator-visible恢复动作。

## 5. P1差距状态

### 5.1 Record schema、identity与上下文

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-LOG-P1-01 | Open | severity仍只有Info/Warning/Error；分离severity、verbosity、outcome与presentation tone，并保留未知等级。 |
| E-LOG-P1-02 | Open | 每条强制`timestamp_frame`且大量非frame事件填0；增加UTC、monotonic offset和optional frame/clock domain。 |
| E-LOG-P1-03 | Open | 无process/thread/task/span/request/session identity；建立typed execution context和parent correlation。 |
| E-LOG-P1-04 | Open | 无stable diagnostic code、payload schema/version；message不能继续承担机器合同。 |
| E-LOG-P1-05 | Open | 只有纯文本和单jump；增加bounded typed fields/tokens及index/privacy/display policy。 |
| E-LOG-P1-06 | Open | 六个粗channel不能表达subsystem/category；拆分owner、subsystem、category和operation。 |
| E-LOG-P1-07 | Open | Plugin source仍是任意字符串且生产无owner lease；绑定package/version/generation publisher capability。 |
| E-LOG-P1-08 | Open | 缺project/document/asset/entity/node/job/play-process上下文和跨record关联。 |
| E-LOG-P1-09 | Open | 一条record最多一个jump；支持0..N asset/file/node/docs/fix command tokens及授权receipt。 |
| E-LOG-P1-10 | Open | jump只验证非空，line/column可0且path未canonicalize；执行前重验target generation和1-based bounds。 |

### 5.2 Ingestion、routing与producer adoption

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-LOG-P1-11 | Open | `tracing`仍未桥接到Editor journal；process startup安装唯一subscriber/layer并保留target/span/fields。 |
| E-LOG-P1-12 | Open | status仍维护第二份256行`EditorConsoleHistory`，真实Console又从log store投影；status只能是journal query的短摘要。 |
| E-LOG-P1-13 | Partial | Runtime task source现有cursor/gap/dropped bridge并每tick接入log；但被压成字符串、frame固定0，其他runtime diagnostics仍未统一schema。 |
| E-LOG-P1-14 | Open | Plugin lifecycle/admission/crash未系统进入Plugin channel；建立owner-scoped health listing和stable codes。 |
| E-LOG-P1-15 | Partial | model import commit已进入Import channel并携generation/count/jump；完整import/reimport/thumbnail/DDC失败链仍未接入。 |
| E-LOG-P1-16 | Open | Play capture已有entry/byte/time budget和loss/truncation计数，但仍靠`process.*`字符串前缀推断stream/severity，stderr统一Warning；升级versioned framed child protocol。 |
| E-LOG-P1-17 | Open | save/export/autosave/render/UI等大量失败仍只写status；用adoption matrix和architecture test禁止error-only-status。 |
| E-LOG-P1-18 | Open | service只有一个可替换event sink，file sink仍是特殊内嵌字段；改为每sink独立queue/policy/health/cursor/lifecycle。 |
| E-LOG-P1-19 | Partial | Console现在可靠读取canonical store并复用append generation；仍无typed event invalidation、journal cursor或断连resync合同。 |
| E-LOG-P1-20 | Open | 普通record零subscriber仍是Delivered，resync却是NotConfigured；统一NoConsumer/Accepted/Persisted/Displayed/Dropped状态。 |
| E-LOG-P1-21 | Open | recorded/resync仍是builder私有JSON schema；定义公共versioned DTO、size/depth预算和兼容测试。 |

### 5.3 Store、query、cursor与delivery semantics

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-LOG-P1-22 | Open | FIFO severity-blind eviction可让低价值storm挤掉关键失败；增加owner/severity reserve、policy和eviction reason。 |
| E-LOG-P1-23 | Open | retained bytes仍是手工估算，不含Arc/VecDeque/allocator/index/sink副本；公开保守resident accounting/high-water。 |
| E-LOG-P1-24 | Open | dropped只有累计数；写typed loss marker，记录sequence range、reason、source和severity counts。 |
| E-LOG-P1-25 | Partial | `snapshot_tail(filter,max)`避免materialize全store；仍需page/cursor/max-bytes/deadline/cancel和immutable segment query。 |
| E-LOG-P1-26 | Closed | `record(sequence)`已用front sequence计算VecDeque offset并校验sequence；回归测试检查源码不再`.iter().find`。 |
| E-LOG-P1-27 | Open | sequence仍只在service实例内唯一；使用`RecordId { session_id, sequence }`覆盖重启和多进程。 |
| E-LOG-P1-28 | Open | Clear删除memory authority但disk保留且无marker；分型Clear View、Archive Session、Delete Data并审计。 |
| E-LOG-P1-29 | Open | resync只有`through_sequence`；返回session epoch、oldest/newest、loss ranges和cursor-expired。 |
| E-LOG-P1-30 | Open | event queue bytes仍按entry估算；每lane使用encoded/retained预算和global admission cap。 |
| E-LOG-P1-31 | Open | replace sink无detach barrier，pending可跨generation；定义drain/abort/cutover receipt。 |
| E-LOG-P1-32 | Open | callback无panic isolation、deadline、backoff或circuit breaker；sink必须由worker监督。 |

### 5.4 Persistence、retention与operations

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-LOG-P1-33 | Partial | 当前segment已复用目录/metadata/open控制路径；每条`write_all + flush`仍同步发生在全局锁内。迁移长寿命writer和batch flush。 |
| E-LOG-P1-34 | Open | 文件仅epoch day命名且record无wall time；增加session manifest与UTC/monotonic timestamp。 |
| E-LOG-P1-35 | Open | 首record仍可超过max segment；按encoded bytes admission，oversize截断/sidecar并写marker。 |
| E-LOG-P1-36 | Open | 无总bytes、age、file count、GC、compression和low-disk策略。 |
| E-LOG-P1-37 | Open | 无session/process/build ID和cross-process lease；使用per-process spool与明确merge coordinator。 |
| E-LOG-P1-38 | Open | `key=value` line无版本/checksum/parser，只转义反斜杠和CR/LF；改为可恢复versioned journal encoding。 |
| E-LOG-P1-39 | Open | 无durable cursor、shutdown drain、panic flush或crash spool；提供deadline和超时receipt。 |
| E-LOG-P1-40 | Open | configure不probe I/O；返回lease/health并做非破坏create/open/write验证。 |
| E-LOG-P1-41 | Open | append error仍只回传给caller；service聚合last error/count/since/retry并once/coalesce告警。 |
| E-LOG-P1-42 | Open | 无redaction、PII/token/path/source-control policy；field sensitivity贯穿disk/copy/export/support bundle。 |

### 5.5 Console query、presentation、interaction与accessibility

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-LOG-P1-43 | Open | Activity的All与Info都等于minimum Info，Warning包含Error；改为独立toggle或明确标注minimum severity。 |
| E-LOG-P1-44 | Open | counts只统计过滤后tail，legacy history又统计另一authority；query同时返回global/filtered/visible/evicted counts。 |
| E-LOG-P1-45 | Open | severity/source仍是单选；提供include/exclude multi-facet和可序列化filter state。 |
| E-LOG-P1-46 | Open | 真实Console无search/regex/category/plugin/code/time/span query；建立indexed incremental query、cancel和highlight。 |
| E-LOG-P1-47 | Closed | Activity projection已直接请求256条filtered tail，稳定generation复用，append只materialize新增records；不再先clone/format 2,048条。 |
| E-LOG-P1-48 | Open | 256条cap仍静默，无total/omitted/lost/page cursor；显示gap并支持load older/newer。 |
| E-LOG-P1-49 | Closed | source/action identity已使用record sequence；virtual slot以stable slot/source ID重绑定，不再把可见索引当record identity。 |
| E-LOG-P1-50 | Open | 行仍固定18px且elide；增加wrap/horizontal/details模式和可测量virtual row height。 |
| E-LOG-P1-51 | Open | 无selection/copy/copy-all/save/export/open-folder/context menu；导出必须streaming、bounded和redacted。 |
| E-LOG-P1-52 | Open | 无显式pause/follow toggle/bookmark/pin/back-to-latest；建立live state、new-record badge和scroll anchor。 |
| E-LOG-P1-53 | Open | Clear仍销毁memory authority且disk保留；分离view/session/data动作并为破坏性删除确认。 |
| E-LOG-P1-54 | Open | row只显示sequence/frame/source，没有wall/relative timestamp模式。 |
| E-LOG-P1-55 | Open | ScriptLocation仍只open asset并写status，未等待document ready或移动caret/selection。 |
| E-LOG-P1-56 | Open | record evict后jump只返回status error；绑定durable ID/revision并解释retention或回退session file。 |
| E-LOG-P1-57 | Open | 缺keyboard selection/copy/filter/focus、accessible row semantics、live announcement和filter persistence。 |
| E-LOG-P1-58 | Open | Console Diagnostics扩展页是collapsed preview scaffold：硬编码session/rows/counters，filter/clear/selection只替换固定feedback字符串。删除假数据，或接入同一query/counter/report authority并标记真实loading/empty/error。 |

## 6. P2差距状态

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-LOG-P2-01 | Open | Console chrome、source、错误和jump反馈仍有直接英文；使用typed localization key/args，保留原始diagnostic text。 |
| E-LOG-P2-02 | Open | 无字体、行距、对比度、color和timestamp偏好；保存可访问presentation profile。 |
| E-LOG-P2-03 | Open | row主要靠`[Info]/[Warning]/[Error]`短文本；使用一致icon/badge/color但颜色不作唯一编码。 |
| E-LOG-P2-04 | Open | filter无named preset、scope或迁移；支持global/project/session策略。 |
| E-LOG-P2-05 | Open | Console是否包含命令输入尚未形成产品边界；如引入必须复用Command130权限/completion，不能变成任意shell。 |
| E-LOG-P2-06 | Open | event不collapse/group，storm只靠FIFO；按code+normalized fields聚合count/first/latest并可展开。 |
| E-LOG-P2-07 | Open | 所有来源挤在一个listing；支持Build/Import/Play/Plugin named listings/pages并保留全局query。 |
| E-LOG-P2-08 | Open | 只有纯文本与单jump；增加docs/url/asset/node/fix rich tokens和未知token降级。 |
| E-LOG-P2-09 | Open | 无bookmark、annotation或issue关联；作为独立overlay，不修改immutable producer record。 |
| E-LOG-P2-10 | Open | 无正式JSONL/CSV/support-bundle schema；提供versioned streaming export、manifest和size estimate。 |
| E-LOG-P2-11 | Open | 无headless/remote/mobile viewer合同；复用同一cursor/query协议并按capability降级。 |
| E-LOG-P2-12 | Open | 用户看不到storage占用、quota、writer health和last durable cursor；进入Settings/Diagnostics操作面。 |

## 7. 参考引擎对照与适用边界

| 参考 | 仓内可验证能力 | Zircon应吸收的原则 | 不应照搬/不可推断 |
|---|---|---|---|
| Unreal OutputDeviceRedirector | 多output device、secondary-thread buffering、backlog、dedicated primary logging thread、async flush option、fence和panic-safe output path。 | producer与慢sink解耦；多sink有thread/flush/lifecycle合同，crash路径独立。 | 头文件不能证明所有平台实现都无阻塞，不复制全局singleton形态。 |
| Unreal Tokenized Message / Message Log | severity、URL/asset/docs/action/fix token、selection、filtered messages、export selected/all、page和duplicate处理。 | event journal与provider listing分层；action必须typed，listing有稳定page/selection。 | Message Log不是高吞吐trace storage，仍需Zircon自己的quota/schema/capability。 |
| Unreal Output Log | text/category/severity filter、settings持久化、timestamp、font、wrap、clear-on-PIE、scroll/focus、drawer/tab和suspend/resume。 | Console是queryable product surface，filter/follow/display state必须可恢复。 | 不把console input等同任意shell权限，不复制Slate结构。 |
| Godot EditorLog | error handler接入、10k line limit、独立type toggle/count、search、duplicate collapse、file link、selection、follow、持久化filter和accessibility name。 | 搜索、复制/选择、折叠、follow、链接、配置恢复和读屏是最低完整工作流。 | RichTextLabel line store不是长期typed journal架构。 |
| Bevy Log | process-wide tracing subscriber、LogTracer、EnvFilter target/module过滤、custom/fmt layers、SpanTrace及Chrome/Tracy/OS layers。 | Rust producer统一走tracing并保留span/target/context，性能和格式化sink可组合。 | Bevy plugin初始化不解决Editor多project journal和Message Log产品面。 |
| Fyrox Log | listener fan-out、relative time、level、stdout/file、one-shot dedupe和verify helper。 | listener/sink分离、level policy和可测合同是最低基线。 | 全局mutex和同步flush不是性能/恢复上限。 |
| Unity Graphics diagnostics | ShaderGraph按provider/node保存并选择性clear；VFX区分Compilation/Invalidate origin、model owner、dirty/scheduled regeneration和PerfWarning。 | provider current diagnostics必须按owner/subject/generation replace/invalidate，不能混成append-only log。 | Graphics包不是完整Unity Editor日志系统，不能推断其全局持久化能力。 |

Zircon应组合Unreal的异步fan-out和tokenized listing、Godot的Console工作流、Bevy的process-wide tracing、Unity Graphics的provider generation，再以自己的session/build identity、plugin capability、bounded journal和跨进程协议收敛。参考实现是能力证据，不是API照抄清单。

## 8. 生命周期与目标所有权

| 对象 | 语义 | 更新方式 | Owner / UI |
|---|---|---|---|
| Event Record | 已发生、不可原地改写的事实。 | append-only；修正另发record。 | Event Journal / Output Console、timeline、support bundle。 |
| Provider Diagnostic | 某subject当前仍有效的问题。 | owner + subject + generation原子replace/invalidate。 | Provider Store / Problems、Inspector badge。 |
| Status Summary | 当前操作的短摘要，不是事实authority。 | 新摘要覆盖旧摘要，引用record/job。 | Status Projection / status bar。 |
| Notification Delivery | record是否需要toast/badge/OS提示。 | policy驱动delivery/suppress/ack/dismiss receipt。 | Notification Router / toast、Center。 |

```text
tracing / Runtime / Editor / Plugin / Import / Build / Play / Provider
                               |
                               v
                 Diagnostic Ingress (bounded MPSC)
        schema + owner lease + session + context + admission receipt
                               |
                               v
                     Diagnostic Router
             /                 |                 \
            v                  v                  v
      Event Journal       Provider Store      Status Projection
      segments/cursor     owner+generation    latest summary only
         |      |               |                  |
         |      +---------------+------------------+
         |                      |
         v                      v
   File/Crash Sink      Console / Problems / Notifications / Headless
   batch+quota+health   query delta + typed actions + delivery receipts
```

关键约束：ingress常规路径不等待磁盘/UI/network；journal ID包含session epoch；每个sink有独立queue/worker/health/cutover generation；Console只消费query/delta；provider state与event history生命周期分离；status/toast只引用事实ID；critical crash path使用预分配emergency spool。

## 9. 分层重构路线

### M0 - 封闭三项P0并删除虚假产品状态

1. rolling append和event callback移出producer/global lock，建立有界writer lane、batch flush和health。
2. 修正零subscriber为NoConsumer；决定删除无效bus sink或让Console用typed invalidation/cursor订阅。
3. configure阶段probe I/O，append failure进入service-owned degraded state并一次聚合提示。
4. 增加目录总bytes/age/file-count配额、启动GC和oversize admission。
5. Console显示256条截断/gap；删除或接线硬编码Console Diagnostics workspace。

### M1 - 统一record schema、session identity与ingress

1. 定义RecordId/Code/Severity/Verbosity/Owner/Subject/Context/Token/Action和版本策略。
2. process启动创建session/build/process manifest；project切换只改变context。
3. 建立bounded MPSC、admission receipt、loss reserve和per-owner/rate/severity policy。
4. 安装统一tracing layer，逐步迁移runtime/plugin/import/build/play producer。

### M2 - Segmented journal、query engine与异步sink

1. immutable segments按entry/bytes切分，sequence index支持tail/page/filter/lookup。
2. cursor返回oldest/newest/loss range/session epoch；eviction写typed marker。
3. sink registry支持独立worker、queue、deadline、health、detach barrier和shutdown fence。
4. memory/file/stderr/support-bundle共享schema，避免重复字符串authority。

### M3 - Durable persistence、恢复与隐私

1. versioned journal + checksum/length + manifest，支持partial-tail recovery和durable cursor。
2. 配置global/project/session quota、compression、low-disk降级和GC receipt。
3. session spool覆盖startup/no-project阶段，project attach不丢早期record；多进程使用lease。
4. redaction/secret/PII/path policy贯穿producer、disk、copy、export和support bundle。

### M4 - Console、Problems与Diagnostics产品面

1. Console采用windowed query，支持search、多选facet、timestamp、wrap、selection/copy/export和follow/pause。
2. stable RecordId维持row identity、scroll anchor、selection、focus和actions。
3. 建立Provider Problems listing，按owner/subject/generation replace/invalidate。
4. ScriptLocation等待document ready并移动caret；action通过typed command返回receipt。
5. Diagnostics workspace只显示真实query/counter/report，明确loading/empty/degraded状态。

### M5 - 全产品adoption与插件/child protocol

1. 对130个status/tracing绕行文件分类迁移，禁止用户可见error只留status。
2. import/save/build/export/play/render/plugin/Hub逐域定义stable code和context。
3. child output升级versioned framed protocol；raw stdout/stderr只作有encoding/line/byte budget的fallback。
4. plugin获得scope-bound publisher、quota和schema/action manifest，unload撤销current state但保留event history。

### M6 - 压力、故障、可访问性与发布验收

1. 压测百万records、并发producer、慢盘/慢UI、sink panic、plugin flood和cursor eviction。
2. 记录emit p50/p95/p99、alloc、queue high-water、writer throughput、query latency和paint rows。
3. 注入disk full、permission revoke、partial write、process kill、panic、shutdown timeout和recovery。
4. Windows优先完成真实Editor键盘、UIA/读屏、200%缩放和large-project数据集验收。

## 10. 验收门

1. 常规Info/Warning/Error emit不执行文件I/O、UI callback或network；producer p99有release硬门。
2. file、UI、telemetry任一sink阻塞或panic不阻塞其他producer/sink。
3. 零consumer返回NoConsumer；Accepted/Persisted/Displayed/Dropped均携sink identity/cursor。
4. ingress达到entry/byte/rate上限时返回typed admission receipt并保留loss marker预算。
5. record schema有version、stable code、owner、session、UTC/monotonic clock和optional frame。
6. session重启、project切换和多进程不会产生RecordId冲突。
7. tail/page/filter query支持max entries、max bytes、deadline/cancel和cursor。
8. eviction/resync返回oldest/newest、gap range、reason及affected counts。
9. `record(id)`保持O(1)或O(log n)并有大规模benchmark。
10. Clear View、Archive Session和Delete Data语义/权限/审计彼此独立。
11. sink register/replace/detach有generation、drain/abort policy和cutover receipt。
12. file writer批量写/flush并提供last durable cursor、queue high-water和degraded state。
13. journal目录有bytes/age/file-count quota、compression/GC和low-disk策略。
14. journal encoding有checksum/length，partial tail可恢复且损坏有明确marker。
15. configure阶段发现不可写目录；runtime failure聚合告警并retry/backoff。
16. startup/no-project record进入session spool，project attach不丢失或重复。
17. shutdown、panic和crash路径有deadline/fence/emergency spool及未完成receipt。
18. secret/PII/path字段在disk、copy、export和support bundle中遵循同一redaction policy。
19. process-wide tracing subscriber保留target/span/fields并进入同一router。
20. Runtime task gap/dropped信息保持typed字段，不依赖解析显示字符串。
21. Plugin/Import/Play/Build每域至少有成功、失败、取消/丢失和owner teardown测试。
22. Console severity/source是组合facet，文案与精确查询语义一致。
23. Console显示total/visible/omitted/lost并可load older/newer。
24. Console支持search、selection、copy、streaming export、timestamp、wrap和details。
25. pause/follow/new-record badge保持scroll anchor，不因append/filter丢失selection/focus。
26. ScriptLocation实际移动caret/selection并返回document generation receipt。
27. Provider Problems按owner/subject/generation replace/invalidate，旧generation不能复活。
28. shipping Diagnostics workspace没有硬编码session/row/counter；所有action连接真实authority。
29. Console与Problems具keyboard、UIA/读屏role、severity/count announcement和focus restore。
30. Windows真实Editor通过慢盘、storm、filter/scroll/jump/export、200%缩放和读屏验收。

## 11. 与相邻报告的所有权

- Editor121拥有message bus topic/subscriber/inbox/backpressure通用合同；本报告拥有diagnostic delivery语义和是否继续使用log topic。
- Editor122拥有Editor event journal/replay通用生命周期；本报告拥有diagnostic record/provider schema与Console query。
- Editor130拥有Command、keymap、remote automation和action capability；本报告只定义diagnostic token引用这些command。
- Editor131拥有Job admission/progress/cancel/shutdown；本报告保存job diagnostic event和status projection。
- Editor132拥有toast/decision/progress Notification Center；本报告只提供可被通知引用的durable diagnostic identity和delivery input。
- Runtime task/process diagnostics的runtime source authority由相应`zircon_runtime`计划拥有；本报告拥有Editor bridge、journal和产品呈现。

## 12. 本轮未实施内容

本轮没有修改Rust生产代码、ZUI资产、Cargo manifest、测试或参考引擎源码；没有执行format、Cargo、真实窗口、磁盘故障或性能测试。交付物仅为当前源码差距账本、目标架构、重构顺序和验收门。实施必须从M0开始，先封闭producer阻塞、虚假delivery、持久化health和硬编码Diagnostics产品状态，不能继续在旧字符串协议上叠加功能。

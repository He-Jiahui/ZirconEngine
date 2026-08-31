---
related_code:
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/plugin_diagnostics.rs
  - zircon_runtime_interface/src/plugin_events.rs
  - zircon_runtime_interface/src/script_diagnostics
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling
  - zircon_runtime/src/runtime_diagnostics
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter
  - zircon_runtime_host/src/foreign_output
  - zircon_editor/src/core/gateway/session/profile.rs
  - zircon_editor/src/core/gateway/session/plugin_events.rs
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/script_build/diagnostics_sink.rs
  - zircon_editor/src/ui/retained_host/app/profiling
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CpuProfilerTrace.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CountersTrace.h
  - dev/UnrealEngine/Engine/Source/Developer/TraceServices/Public/TraceServices/Model/AnalysisSession.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/godot/main/performance.h
  - dev/godot/editor/debugger/editor_profiler.h
  - dev/godot/core/extension/gdextension_interface.json
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/editor/src/stats.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 04 · Profiling、Plugin Event、Script Diagnostic、Manifest 与 Crate Ownership 收口审查

## 1. 结论

本轮补齐 `zircon_runtime_interface` 前三份报告没有独立闭合的最后一组公共类型：profiling/diagnostics 快照与控制、native plugin host table/module manifest/event callback、dynamic plugin-event mirror、script/plugin diagnostic，以及顶层 re-export ownership。结论不是这些模块“完全不可用”。Profile recorder 已有 bounded ring 和 retention counters；plugin-event mirror 已有 producer queue event/byte cap、单页 event/encoded-byte cap、sequence/backlog age，以及 App/Editor shared consumer budget；native V4 system registration也已经加入explicit access plan、thread affinity和host-side policy。

但当前接口仍存在三项必须先冻结的直接风险。

第一，`ProfileCaptureConfig` 将 `max_frames/max_spans/max_counters` 定义为任意 `usize`，`normalized()` 只把0替换为default，不设上限；dynamic `ProfileControlRequest` 可以直接提交这些值。Recorder虽然不预分配全部容量，却会在capture期间持续增长到调用方指定上限，随后 `snapshot()` 深拷贝全部String/rows，再完整JSON编码，host的16 MiB consumer cap发生在producer完成之后。这是可跨DLL触发的CPU/内存/暂停资源耗尽合同。

第二，同一请求可控制 `output_root` 与 `session_id`。Export以 `PathBuf::from(output_root).join(sanitize_session_id)` 建目录并覆盖六个固定文件；`output_root`可为任意绝对/相对路径，session sanitizer又允许`.`，精确`..`仍是父目录。调用方可让Runtime在进程权限范围内创建目录并覆盖`timeline.zrtrace.json`、`summary.md`等文件。Profiler export必须由host-owned artifact service分配目录，Runtime只能返回流/页或写入pre-opened capability，不得接受路径字符串。

第三，native host table把 diagnostics `emit/metric` 函数指针广告为`Some`，实现却无条件返回`Ok`并丢弃全部内容。相邻的spawn/asset/event callbacks至少返回`UnsupportedVersion`，diagnostics则制造“已经记录”的假成功。它会让插件错误、指标和审计信息永久消失，且第一方contract tests可能只看到status绿色。必须在暴露table前实现typed sink，或将slot设为None/明确Unsupported，禁止silent success。

其余差距集中在身份与时间。`ProfileSnapshot`没有capture ID/generation、process/thread、clock domain/base time、source build或completeness；Editor当前只能通过span ID偏移和数组拼接合并不可比较的两个`Instant`时间轴。新增的`recorder_retention: Vec<_>`虽然保留每个ring统计，却不含recorder/source identity，只能靠vector位置猜来源。`RuntimeDiagnosticsSnapshot`又嵌入完整ProfileSnapshot，并把`selected_model_resource_id/selected_material_resource_id`描述成当前场景canonical Cube，产品示例假设已经进入稳定公共DTO。

插件面同样仍是部分可用table而非capability-negotiated contract。V3/V4 host table同时广告register、spawn、asset、event、bridge、diagnostics；实际只有system/component registration和bridge有生产实现。module target modes/capabilities是未规定codec的byte slice，event type“stable hash”没有algorithm/collision registry，callback event time是无clock-domain的`f32`。dynamic plugin-event page虽已显著改善boundedness，但queue overflow先丢事件、下一次drain只给非结构化错误，成功batch没有dropped/overflow/resync marker，consumer无法证明连续性。

本轮登记3项P0、60项P1、12项P2，均为`pending`。修复不是再增加一套telemetry struct；应建立Observation Session/Clock/Metric Registry、host-owned Artifact Service、generated Native ABI capability table和共享Diagnostic Envelope。完成本文后，`zircon_runtime_interface`四轮首审的主要物理域已经覆盖，但全crate完成仍取决于前三份报告与本文共同通过compatibility、安全、budget和cross-process资格门。

## 2. 审查边界与证据

### 2.1 物理范围

| 集合 | 文件 / 物理行 / bytes | 证据等级与边界 |
|---|---:|---|
| selected interface closing scope | 8 / 1,998 / 74,510 | E3：profiling、manifest、plugin ABI/diagnostic/event、script diagnostic、dynamic event DTO与lib exports |
| selected interface tests | 10 inline test attributes / 0 ignored | E3静态阅读：profile legacy serde、runtime diagnostics、plugin event shape、script location round-trip |
| Runtime profiling/diagnostics chain | recorder/control/export/dynamic response | E3：config admission、ring增长、snapshot深拷贝、filesystem export、foreign allocation |
| Runtime native plugin adapter | registration policy、bridge、stub callbacks | E3：实际host table slot与返回status，不从interface声明推测完成度 |
| Runtime plugin-event mirror | scene producer queue -> dynamic page encoder | E3：64/128 KiB page、16K/64 MiB queue、sequence、failure与backlog |
| Host/Editor consumers | foreign output、profile merge、gateway、bounded pump | E3：16 MiB profile consumer、256 KiB/64 event page、跨进程merge与event continuity |

selected interface source fingerprint为`7e03068352be1072caa74d77a9e0ae0b593238defe5be1ce0d3eb5e63df65005`。算法仍为相对路径排序、逐文件SHA-256，再对`path<TAB>hash<LF>`清单取SHA-256。成文时`lib.rs`、`profiling.rs`、`runtime_api/plugin_event_mirror.rs`已有其他在途修改；本文按当前工作树取证，不修改或回退它们，实施前必须重取fingerprint。

### 2.2 Owner 与去重边界

1. Interface 01拥有通用ABI table、buffer/status/handle、build set、thread/callback和foreign allocation安全；本文只拥有profile/plugin/event的domain contract。
2. Interface 02拥有通用Schema/Identity/Diagnostic/Persistence基础；本文要求这些剩余family接入，而不另建registry。
3. Interface 03拥有UI diagnostic/status与operation/host-output收敛；本文处理非UI observation/plugin/script family和全crate收口。
4. Runtime 03拥有DiagnosticStore、CPU recorder算法、config和export implementation；本文拥有跨DLL request/snapshot/clock/budget/artifact wire。
5. Editor 25拥有Performance Timeline、Runtime Diagnostics、Telemetry Dashboard产品和cross-process presentation；本文拥有使它可正确合并的source/clock/capture协议。
6. Plugins 01拥有package/SDK/native loader/admission/distribution和false shell产品；本文拥有`plugin_api.rs`/manifest/event/diagnostic ABI shape。native diagnostics stub的实现gate由两者共同验收。
7. Editor 11拥有日志journal/retention/search/export UI；Tooling 07拥有benchmark/capture/symbol/crash evidence。本文只提供结构化、可关联、可预算的输入合同。

### 2.3 纵向调用链

本轮逐项追踪：

1. JSON ProfileControlRequest -> dynamic FFI decode -> global recorder control -> snapshot/export -> JSON owned allocation -> Runtime Host/App/Editor decode；
2. ProfileCaptureConfig -> recorder ring retention -> snapshot -> hotspot/UI hotspot分析 -> fixed-name filesystem artifacts；
3. Editor recorder snapshot + child Runtime snapshot -> span ID remap ->数组/retention拼接 -> Timeline/hotspot；
4. RuntimeDiagnosticsSnapshot producer -> nested diagnostic series/history/profile -> host item/byte budget -> Editor pane projection；
5. native plugin entry -> V3/V4 host table -> system/component/asset/event/bridge/diagnostic callbacks ->实际adapter结果；
6. scene typed event -> bounded mirror queue -> dynamic delivery batch -> shared foreign-output policy -> Editor bounded pump；
7. script build diagnostics与Editor plugin registration diagnostics的producer/consumer；
8. Unreal/Godot/Bevy/Fyrox/Unity Graphics 的trace session、metric identity、plugin ABI generation与debug lifecycle参考边界。

### 2.4 动态证据边界

本轮没有重复不可达的Cargo lane。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断；当前selected interface文件又有3个在途修改。静态证据足以证明路径拼接、无上限config、stub返回值和缺失字段，但不能证明修复后的性能、跨进程时间同步或ABI兼容。

### 2.5 参考实现给出的最低基线

- Unreal Trace以analysis session、timeline/thread/process和trace event identity组织数据；CPU scope/counter前端不是把两个进程相对零点数组直接拼接。Plugin/Module descriptor又明确区分module loading/target/compatibility contract。
- Godot Performance使用注册monitor与debugger profiler协议区分数据owner，GDExtension interface由JSON/schema/header generator维护；这比Rust/C双方手写不对称table更接近可发布ABI。
- Bevy `DiagnosticPath`、measurement history和plugin lifecycle适合参考进程内registry；它没有稳定DLL承诺，因此不能直接作为跨语言wire依据。
- Fyrox动态plugin和Editor stats展示了plugin lifetime与本地统计owner；同样不能替代Zircon自己的版本化跨DLL协议。
- Unity Graphics `ProfilingScope`和`DebugManager`提供注册、作用域、panel/data reset生命周期；本文只借用其owner/lifecycle原则，不把Unity debug UI外推为远程telemetry系统。

## 3. 已有可保留基础

1. recorder frames/spans/counters均有ring cap和written/overwritten/retained/oldest/newest统计，已修复早期完全静默截断的一部分问题。
2. V4 native system registration独立于V1，包含access list、thread affinity、size/ABI header，并由host验证component/resource/capability与冲突。
3. native bridge已有generation-qualified host context、close-and-wait与dense method table，不能因为相邻stub而全部推倒。
4. plugin-event scene queue已有event count/payload bytes双预算，单payload、单page和wire encoded size都有producer-side检查。
5. dynamic delivery携带subscription、play session、per-subscription sequence、remaining backlog和oldest age；Editor pump也有bounded drain和backpressure观察。
6. shared foreign-output policy统一App/Editor的ProfileResponse与PluginEvents consumer限制，并统计accepted/rejected/decode time。
7. ScriptDiagnostic至少保留code/module和point location，RegistrationDiagnostic至少有code/plugin ID；可以迁移到共享envelope，而非退回纯字符串。

## 4. P0：先冻结资源、文件系统与假成功

### P0-01 · ProfileControl允许无上限ring与完整snapshot，consumer cap无法阻止producer资源耗尽

`max_frames/max_spans/max_counters`是任意`usize`，只对0做default；capture期间VecDeque可增长到该上限，snapshot深拷贝全部rows/strings，Runtime再一次性JSON编码。必须在FFI admission设固定宽度hard maxima、总byte/strings预算、capture duration/deadline和paged snapshot；超过限制在启动capture前拒绝。

### P0-02 · Profile export接受调用方路径，`..`可越出root并覆盖固定文件

`output_root`完全可控，sanitizer保留`.`，`session_id == ".."`使join定位父目录；六个固定文件用`fs::write`直接覆盖。必须删除wire中的filesystem path，改由host Artifact Service创建attempt directory或传入受限directory capability，使用staging/manifest/digest/atomic publish。

### P0-03 · Native diagnostics slots返回Ok但丢弃所有diagnostic与metric

V3/V4 table把emit/metric设为Some；`native_host_diagnostics_emit_v1`和`metric_v1`无条件`ZrStatus::ok()`。必须立即fail-close：未实现时slot为None或返回Unsupported；实现后走typed bounded sink并生成receipt/metrics。禁止任何测试把当前Ok当成能力证明。

## 5. P1：工程级公共合同差距

### 5.1 Observation session、control 与 capture config

### P1-01 · profile control JSON没有wire envelope或schema fingerprint

`ProfileControlRequest/Response`依赖Rust enum/field spelling。接入Schema Catalog，发布profile family ID、version、reader/writer range、capabilities、budgets和unknown command policy。

### P1-02 · request没有request/correlation ID

Start/Stop/Snapshot/Export无法去重或关联迟到response。增加request ID、observation session ID、attempt与idempotency key。

### P1-03 · request没有deadline、cancel或priority

全量snapshot/export会同步阻塞调用。control只负责提交bounded job，poll/cancel/harvest复用完整operation语义或专用stream protocol。

### P1-04 · request没有明确target process/runtime session

global recorder与dynamic session handle并非同一identity。请求必须指定process/session/recorder owner，拒绝跨session误控。

### P1-05 · `config`可附在任何command上且被静默忽略

Stop/Snapshot/Reset/Export携带config不会报错。为每个command定义独立payload，未知/多余字段按closed transport policy拒绝。

### P1-06 · `normalized()`把非法输入静默改成默认

空session/root、零capacity和非正budget被改写，调用方不知道实际配置。admission应返回validated effective config与normalization warnings；越界值明确拒绝。

### P1-07 · `frame_budget_ms`不拒绝NaN/Inf

`NaN <= 0.0`为false，会穿过normalized，随后JSON serialize/比较/热点分析产生错误或失败。所有浮点输入与sample必须finite并验证合理范围。

### P1-08 · capacity使用`usize`进入持久/transport DTO

它随data model变化。wire使用有界u32/u64，转换到usize前执行checked conversion并记录effective budget。

### P1-09 · capture config没有总内存/字符串/持续时间预算

三个row count不能限制每row String大小、总byte或长时间capture。加入max bytes、max string bytes、max duration、max producers和sampling policy。

### P1-10 · `include_perfetto`把capture与artifact格式耦合

capture应产生中立trace stream，export profile选择格式/压缩/符号化。把format selection移到host artifact/export request。

### P1-11 · session ID没有validated grammar与稳定identity

当前既用作显示名又参与路径。分离opaque ObservationSessionId、display label和artifact slug；ID由owner生成而非调用方任意字符串。

### P1-12 · control response状态是`"ok"/"error"`字符串

没有unknown-safe status、retryability或error code。改为tagged outcome和共享diagnostic envelope。

### 5.2 Snapshot、clock、metric 与 report

### P1-13 · ProfileSnapshot没有capture ID或generation

Start/Reset后的旧scope、旧snapshot和新capture无法区分。每个sample/response绑定capture generation，merge前验证一致性。

### P1-14 · snapshot没有producer process/session/build identity

Editor/Runtime数组拼接不能证明来源。加入process instance、runtime session、build set、host role与recorder ID。

### P1-15 · timestamp没有clock domain、base time或frequency

`start_us/timestamp_us`只相对各进程Instant零点。定义clock ID、epoch/base、tick frequency和sync uncertainty；不同clock未经mapping不得合并。

### P1-16 · span/frame/counter没有OS thread/task identity

`stream`字符串被误当Perfetto tid。记录process/thread/fiber/task IDs和display stream，线程复用必须有lifetime generation。

### P1-17 · span ID只在单recorder内唯一却没有owner qualifier

Editor通过最大ID偏移修补冲突，saturating add还可碰撞。使用`{recorder_id, capture_generation, local_id}`，parent引用同样qualified。

### P1-18 · frame index没有source和frame domain

Editor frame、Runtime frame、render frame和fixed tick可能同号不同义。使用FrameDomainId/sequence，并明确关联而非假定相等。

### P1-19 · recorder retention entry没有recorder identity

`Vec<ProfileRecorderRetentionSnapshot>`声称合并后分别保留，却只能靠顺序猜Editor/Runtime。条目加入producer/recorder/capture ID和sample family schema。

### P1-20 · retention sequence没有附到实际sample

aggregate oldest/newest无法识别数组中的具体gap或乱序。每个sample带sequence，snapshot声明range、dropped和completeness。

### P1-21 · snapshot没有page/cursor/truncation/digest

一次性Vec无法流式消费或验证多页一致性。返回immutable snapshot token和bounded pages，携带next cursor、complete、usage与digest。

### P1-22 · ProfileControlResponse可形成任意optional payload组合

error可携带snapshot、ok可无对应command结果，constructor不能约束deserialize。每个command使用tagged response variant和typed result。

### P1-23 · response同时携带snapshot与多个派生report，造成重复放大

Export response会复制raw snapshot、hotspots、counter/UI reports和file list。返回artifact receipt与按需report references，避免同一数据多次编码。

### P1-24 · RuntimeDiagnosticsSnapshot再次嵌入完整ProfileSnapshot

诊断query成本和profile ring大小被强绑定，profile还可能不是同一采集时刻。改用ObservationSnapshotId引用或独立page，并提供atomic capture barrier/consistency说明。

### P1-25 · RuntimeDiagnosticsSnapshot没有统一observation timestamp/generation

frame_index不足以证明render/input/reload/profile来自同一时刻。增加capture window、per-provider generation、staleness与partial provider status。

### P1-26 · selected model/material字段把canonical Cube示例写入公共合同

字段注释直接描述loaded scene canonical Cube，不是通用runtime诊断。移出稳定DTO，改为namespaced provider metric/resource observation。

### P1-27 · render adapter/device identity没有privacy与redaction policy

设备名、项目identity、scene URI和resource IDs可能敏感。transport标注privacy class并按principal/redaction profile输出。

### P1-28 · diagnostic series仍由任意path/unit/tag字符串隐式定义

这延续Runtime 03的cardinality问题。建立Metric Registry，stable metric ID绑定value kind、unit、aggregation、owner、privacy和cardinality limits。

### P1-29 · measurement只有frame index和f64 value

缺timestamp、sequence、validity、source和quality。记录typed value/sample time并拒绝NaN/Inf；unsupported/missing不是0。

### P1-30 · current/smoothed/min/max没有window语义

调用方不知道lifetime、retained window还是临时clone。字段绑定aggregation policy/window/range和sample count，或由query动态选择。

### P1-31 · series history在DTO层无独立预算

producer内部默认64不等于wire保证，任意构造/legacy decode可携带无限history。schema profile必须验证series count、history count、tags和bytes。

### P1-32 · scene reload diagnostics使用`usize`与大量互相重叠计数

它没有attempt/generation/source revision，`skipped`和分项可不一致。改为fixed-width typed outcome counters + invariant-checked receipt。

### P1-33 · hotspot report不绑定输入snapshot digest

无法证明report来自哪个capture或是否截断。加入analysis version、input digest/range、retention completeness和producer build。

### P1-34 · hotspot key依赖stream/category/name/path字符串

跨producer同名会错误聚合。使用registry metric/span IDs和source qualifier，display fields不参与identity。

### P1-35 · hints与UI alerts是自由文本

缺stable rule code、severity、evidence samples、threshold、fix-it和localization。建立versioned analysis rule result。

### P1-36 · `UiScenarioHotspot`把大量Editor实现计数固化在公共Runtime接口

每新增内部counter都向巨型struct追加`serde(default)`字段。改为registry-driven typed metric set和versioned scenario definition，Editor特有schema不应污染核心ABI crate。

### P1-37 · profile artifact文件名成为interface常量但没有manifest

常量不能证明同一attempt、格式version或完整集合。artifact service发布manifest、per-file digest/size/schema、source/build/capture和atomic completion marker。

### P1-38 · response file/export_dir是任意host路径字符串

跨机器/容器不可移植且泄露路径。返回ArtifactId/URI和receipt，实际路径只存在host/tooling本地层。

### P1-39 · 16 MiB/65,536 item consumer budget发生在producer完成后

shared policy值得保留，但需要producer page budgets对称化；超限不应先深clone/encode再fuse session。

### P1-40 · decode-time budget不能抢占已阻塞JSON decode

250 ms只在decode返回后检查elapsed。大payload采用streaming parser/page codec或可中断worker，文档不能声称hard deadline。

### 5.3 Native plugin/module ABI

### P1-41 · module descriptor没有`size_bytes`

只有abi_version，无法安全扩展或检测short/oversized struct。新版本采用header `{abi,size,reserved}`和generated layout assertions。

### P1-42 · target modes与capabilities是未定义codec的byte slices

接口不说明JSON、CSV、NUL列表还是其他格式，也无item/string cap。改为counted typed arrays或schema-bound canonical bytes，并由generated header定义。

### P1-43 · module kind使用raw u32却没有unknown policy

host与SDK必须通过generated enum decoder保存/拒绝unknown，不能把未知值默认Runtime。

### P1-44 · entry report没有size、build identity或compatibility fingerprint

plugin_id/manifest/modules/api指针不足以证明artifact与engine build匹配。加入plugin package/artifact digest、SDK/interface/build set和signed admission receipt；Plugins 01拥有装载执行。

### P1-45 · nested host subtables没有各自ABI/size/capability header

顶层V4升级会为任一子服务复制整张table。每个service table独立version/size/capability，host只暴露实际支持的slots。

### P1-46 · host table以非空函数指针广告Unsupported服务

spawn/asset/event callbacks均为Some但返回UnsupportedVersion。slot availability必须与capability一致；未实现服务不应伪装可调用。

### P1-47 · EventTypeId stable hash没有算法/version/collision registry

plugin同时提供namespace/name/hash，三者可矛盾。host根据versioned canonical identity计算并验证，catalog处理collision/redirect/tombstone。

### P1-48 · native event emit/drain没有schema revision、sequence和producer budget合同

raw pointer/len与caller-suppliedtype不足以形成可恢复event stream。定义event descriptor、payload schema、max bytes/rate、sequence/overflow与ack。

### P1-49 · native diagnostic/metric签名过于贫乏

emit只有target/message，metric只有path/value/unit；缺code/severity/span/tags/correlation/privacy，且f64可非finite。替换为bounded typed envelope和Metric Registry ID。

### P1-50 · native system invoke的返回status被Runtime丢弃

registration closure执行`let _ = invoke(...)`，系统失败不会进入scheduler/diagnostic/disable policy。Runtime必须读取status并产生typed failure、owner quarantine或retry/disable decision。

### P1-51 · `user_data: u64`没有析构callback或lifetime token

插件分配的context只能依赖unload自清理，partial registration/failure难以回收。加入host-trackedregistration token和explicit destroy/cancel callback。

### P1-52 · component schema只是raw bytes

没有schema ID/version/fingerprint、storage compatibility或migration。接入Reflection/Schema Registry，host在registration前验证并生成typed component contract。

### P1-53 · plugin state snapshot没有schema/size negotiation/digest

save写入caller buffer，restore接收raw bytes，没有required-size query、state version、plugin artifact/source generation或migration receipt。建立versioned state artifact和bounded two-phase sizing/streaming。

### P1-54 · callback event time使用`f32 seconds`

长会话精度快速下降且无clock domain/epoch。使用fixed-width ticks + clock ID或明确frame/subframe时间合同。

### P1-55 · callback request没有correlation、deadline与owner generation

handler_id/event_id/source_path均为字符串，无法识别迟到callback或reload后的旧handler。加入plugin/module/handler generation、request ID、deadline和typed source identity。

### P1-56 · callback result只有嵌套ZrStatus

无法表达consumed/deferred/retry/side effects或output。定义tagged callback outcome和receipt；status carrier lifetime仍回指Interface 01。

### 5.4 Dynamic plugin-event stream 与 diagnostics

### P1-57 · dynamic subscribe仍使用raw event/payload schema字符串

应引用Event Registry descriptor ID/version/fingerprint，subscribe admission返回effective descriptor与capabilities。

### P1-58 · subscription handle没有显式generation/epoch

虽然FFI调用同时带session handle且Runtime单调分配，wire handle本身仍只是u64。采用qualified handle并定义restart/overflow/lease semantics。

### P1-59 · overflow/drop不进入成功batch continuity合同

producer overflow会丢新事件并缓存一次failure；下一次drain返回非结构化错误，后续batch继续sequence且被丢事件从未分配sequence。加入dropped range/count、overflow generation和`ResyncRequired`，consumer不得静默继续。

### P1-60 · script/plugin diagnostics仍是两套孤立小协议

severity、code、plugin/module/path/message分别定义；script location只有未声明base/encoding的单点，line/column可为0，无range/related/fix-it/build/correlation/budget/privacy。统一迁到Interface 02/03 Diagnostic Envelope，并保留domain extensions。

## 6. P2：主链中一并收敛

### P2-01 · ProfileControl命令命名混合capture、query与export

按Observation Session、Snapshot Query、Artifact Export拆service，减少一枚enum承担所有生命周期。

### P2-02 · 默认16.67 ms不是精确refresh/frame policy

frame budget由产品/target/present mode配置并记录来源，不以全局常量暗示60 Hz绝对真值。

### P2-03 · profile默认output root属于Tooling策略

从interface移除`target/zircon-profiles`路径常量，交由Artifact Service/provider。

### P2-04 · snapshot中`active/feature_enabled`布尔值语义重叠

改为typed availability/capture state和reason，避免merge时OR产生伪状态。

### P2-05 · oldest pending age缺观察时刻

duration本身可保留，但batch应带producer monotonic observation timestamp/clock ID，使consumer能计算传输延迟。

### P2-06 · plugin delivery PartialEq每次重解析RawValue并使用expect

测试/比较可用canonical bytes或预计算digest，避免大payload重复parse；它不是生产continuity修复。

### P2-07 · play session ID仍是裸u64

迁到qualified session epoch/ID，与RuntimeSessionHandle和Editor PlaySession identity统一。

### P2-08 · RegistrationDiagnostic缺stage与package/artifact identity

作为P1-60迁移的一部分，保留registration stage和selection/admission/build context。

### P2-09 · ScriptSourceLocation path/line/column命名过于宽松

使用validated source URI、0/1-based明确range和UTF-8/UTF-16 column unit。

### P2-10 · 顶层lib.rs重导出所有profile/UI/plugin细节

按stable ABI、transport profile和internal model拆prelude，减少误用与semver blast radius。

### P2-11 · host_output旧目录仍留在interface树中

当前目录为空且lib未声明；确认无并发owner后删除空路径/历史脚本假设，唯一实现保留在`zircon_runtime_host`。

### P2-12 · tests偏重serde round-trip与shape

补充invalid finite/path/budget、clock/source merge、overflow/resync、stub capability和old/new ABI矩阵，不把可序列化当作协议完成。

## 7. 目标架构

### 7.1 Observation 与 artifact 分层

```text
Metric/Span Registry
  stable IDs + owner + unit/type + privacy + cardinality budget
             |
             v
Observation Session
  process/session/build + capture generation + clock domain/mapping
             |
             v
Bounded Recorder Pages
  sequences + dropped ranges + cursor + completeness + digest
             |
             +--> Analysis Service -> versioned findings
             +--> Host Artifact Service -> staged manifest + atomic publish
             +--> Editor Timeline -> only mapped clocks may merge
```

Runtime不接收文件路径。Host创建artifact attempt并决定本地路径、权限、配额和保留；Runtime提供bounded page/stream或受限writer capability。跨进程merge必须先建立clock mapping与uncertainty，不可比较时以separate lanes展示。

### 7.2 Generated native ABI

以IDL/schema生成Rust host、Rust SDK、C/C++ header、JSON schema、layout snapshots和docs。每张service subtable独立header/version/size/capability；host只将已实现且已授权的slot设为Some。所有callback输入有fixed-width size、per-field cap、owner generation、deadline和correlation；输出使用typed outcome/receipt。

### 7.3 Event 与 diagnostic continuity

Event Registry绑定event/schema/owner/version/budget。subscription带session epoch与generation，page带first/last sequence、dropped range、overflow/resync、producer timestamp和cursor。Diagnostic Envelope统一script/plugin/UI/runtime/host code、severity、source range、build/source/artifact、correlation、privacy、budget和truncation，message只是presentation字段。

## 8. 现有实现处置

| 当前实现 | 处置 |
|---|---|
| ProfileCaptureConfig/Control | 发布bounded V2；V1只作trusted legacy adapter并禁用path export |
| ProfileSnapshot | 迁为source/clock/capture-qualified page；保留现有row字段作为首版payload |
| retention counters | 保留统计，补recorder identity与per-sample sequence |
| RuntimeDiagnosticsSnapshot | 拆provider pages；删除canonical Cube字段，profile改引用 |
| UiScenarioHotspot | 迁到Editor scenario schema/Metric Registry，不再扩核心巨型struct |
| filesystem export | 移到host/tooling Artifact Service，建立manifest/digest/atomic publish |
| V3/V4 host API | 冻结兼容；新增generated service tables并按实际capability暴露slot |
| diagnostics stubs | 立即None/Unsupported，随后接typed sink；禁止Ok丢弃 |
| scene event queue/page | 保留双预算/sequence/backlog，补overflow/dropped/resync/epoch |
| Script/RegistrationDiagnostic | 迁到共享envelope，保留旧decoder support window |
| lib.rs re-exports | 按profile/ABI模块分组，internal types不再顶层公开 |

## 9. 分阶段重构

### M0 · 封闭三项P0

1. Profile request admission加入fixed maxima、total bytes/duration/string limits；过大config在capture启动前拒绝。
2. 禁用dynamic path export和`..` session slug；Runtime只返回snapshot/page，host分配artifact目录。
3. diagnostics slots未实现时设None/Unsupported并加contract test，不能返回Ok。

### M1 · Observation identity 与 clock

1. 定义ObservationSessionId、RecorderId、CaptureGeneration、Process/Thread/Task、ClockId/Mapping。
2. 所有sample和snapshot绑定source/build/session/clock与sequence。
3. Editor merge只接受相同capture或有显式clock mapping的数据。

### M2 · Metric/Span Registry

1. 注册stable metric/span IDs、unit/type/aggregation/privacy/cardinality。
2. 拒绝unknown/unbounded dynamic path或在受限extension namespace隔离。
3. 生成producer/consumer schema和analysis rule IDs。

### M3 · Bounded snapshot/page

1. control改为request envelope + async/cancel/deadline。
2. snapshot改page/cursor/digest/completeness/dropped range。
3. RuntimeDiagnostics按provider分页并记录staleness/partial failure。

### M4 · Host Artifact Service

1. host创建attempt directory、quota、ACL和retention policy。
2. Runtime以stream/pre-opened capability输出，host生成manifest/digests。
3. staging validated reread后atomic publish，失败可恢复/清理。

### M5 · Analysis contract

1. hotspot/report绑定input digest、analysis version与completeness。
2. hints/alerts改typed finding、threshold/evidence/fix-it。
3. UI scenario metrics从巨型struct迁入registry-driven schema。

### M6 · Native ABI generation与capability truth

1. 由IDL生成VNext host/service tables、SDK/header/layout tests。
2. service subtable独立version/size，slot只在实现且授权时暴露。
3. system invoke status进入scheduler failure/quarantine policy。

### M7 · Plugin event continuity

1. Event Registry替代raw event/schema strings与caller hash。
2. subscription/page加入epoch/generation/sequence range/drop/resync。
3. overflow、payload-too-large、serialize失败均变成typed stream state。

### M8 · Diagnostic convergence

1. script/plugin/native metric/diagnostic迁移到共享envelope/registry。
2. 加source range/build/artifact/correlation/privacy/truncation。
3. Editor journal和plugin manager消费typed records，不解析message。

### M9 · Compatibility hard cutover

1. V1/V3/V4与新table建立old/new host-plugin矩阵和sunset telemetry。
2. Profile/diagnostic旧JSON保存golden corpus与明确adapter support window。
3. 顶层re-export与internal modules按owner收缩。

### M10 · 故障、规模与跨进程资格

1. child process运行恶意config/path/NaN/huge strings/overflow和DLL callback faults。
2. 多process、多thread、clock skew/drift/restart下验证timeline与event continuity。
3. 结果绑定source/build/command/duration/artifact并进入Tooling 07 evidence。

## 10. 产品资格门

1. 任意profile capacity超过hard max在分配/record前返回typed LimitExceeded。
2. max count以内的极长字符串仍受总byte/string budget限制，RSS与pause保持阈值内。
3. NaN/Inf/负budget、zero/overflow capacity和unknown command均确定拒绝。
4. dynamic Runtime无法根据request字符串选择任意filesystem路径。
5. `session_id=".."`、absolute/rooted/mixed-separator/device path corpus均不能越界或覆盖非attempt文件。
6. export crash发生在每个write/flush/rename点后均可恢复，未完成attempt不冒充完成artifact。
7. Profile snapshot每页有source/capture/clock/sequence/cursor/digest/completeness。
8. 两个未映射的Instant clock不能被合并为单timeline；UI明确分lane或报unsupported。
9. clock mapping记录offset/uncertainty/drift，超阈值数据不用于跨进程duration比较。
10. process/thread/task IDs在重启和线程复用后不碰撞。
11. recorder retention可按recorder ID解释，sample sequence gap与dropped count一致。
12. profile consumer断开时producer memory仍bounded，snapshot可cancel且不先完整materialize。
13. RuntimeDiagnostics provider部分失败/过期会显示partial/stale，不回落成看似完整快照。
14. canonical Cube字段从稳定contract迁出且旧reader有明确adapter结果。
15. Metric Registry拒绝duplicate ID、unit/type冲突和超cardinality producer。
16. 非finite metric/sample不能进入ring、JSON、analysis或Editor UI。
17. hotspot finding可追溯到input snapshot digest、rule version和evidence range。
18. artifact manifest列出全部文件schema/version/digest/size，stale旧文件不混入本次结果。
19. generated Rust/C header在x86_64/aarch64支持矩阵layout一致，`usize` debt有明确拒绝或迁移。
20. host只为implemented+authorized native service暴露non-null slot。
21. diagnostics未实现时插件收到Unsupported；实现后每条Ok都能在typed sink按correlation查询。
22. diagnostic/metric payload超预算、非finite或bad UTF-8确定失败且不污染sink。
23. native system callback失败进入typed scheduler result，不能被`let _`吞掉。
24. plugin reload/unload等待in-flight callback，user context和registration token恰好释放一次。
25. EventTypeId在host catalog唯一，name/hash矛盾和collision admission失败。
26. plugin-event queue overflow产生dropped range和ResyncRequired；后续成功batch不掩盖gap。
27. sequence rollover、session restart和subscription handle reuse全部可检测。
28. event page始终同时满足producer payload、encoded bytes、item和consumer decode预算。
29. ScriptDiagnostic source range明确base/encoding，0/invalid位置被拒绝或标Unknown。
30. script/plugin/UI/native diagnostics共享code namespace且重复owner在build时失败。
31. old/new profile、diagnostic、plugin host/SDK compatibility corpus无未审批漂移。
32. 所有通过结果绑定source fingerprint、build set、commands、exit code、duration与artifact；未执行test binary不得标绿。

## 11. 验证说明

本文只修改审查文档和索引，不修改production Rust、manifest、tests或参考源码。本轮没有运行新动态测试；既有Editor编译阻断与3个selected在途文件要求实施前重新验证。文档自身需要通过frontmatter path、P0/P1/P2编号、里程碑/资格门、LF/UTF-8、link与`git diff --check`检查。

## 12. 审查决策

`zircon_runtime_interface`当前不能把profiling称为transport-safe，不能把native diagnostics称为可用，也不能把plugin-event sequence称为无缺口。保留现有ring retention、V4 access policy、bridge lifetime、event queue/page双预算和shared consumer policy；停止扩大path-bearing profile control、巨型UI hotspot struct、手写不对称host table和孤立diagnostic类型。

本文3项P0、60项P1、12项P2均为`pending`。Interface四份报告共同构成首轮公共边界baseline；下一步不再按文件零散追加 DTO，而应做全crate owner/schema/ABI manifest总表，并按M0依赖顺序开始修复或继续扫描尚未覆盖的其他crate。

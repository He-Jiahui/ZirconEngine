# `zircon_runtime_interface` Profiling / Plugin Event / Script Diagnostic / Native Manifest 公共合同当前源码复核

> report_id: `Interface13`  
> kind: `current-source-review`  
> canonical_source: [04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md](04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md)  
> review_scope: profiling control/snapshot/export，native plugin manifest/host tables/callback，dynamic plugin-event mirror，以及 script/plugin diagnostic 从 Interface 到 Runtime、Runtime Host、App、Editor 的生产消费者。  
> baseline: 当前工作树 `29dfa4a73de5dbc1a4eebe793b50db844c3db93e`（2026-08-29）；未提交与未跟踪源码只作为当前观察点，不视为已交付能力。  
> interface_contract_fingerprint: `ed0b93074f0aa89737d8ded815360f306d12a2bb09c91bcfe6a3fbfee02783eb`  
> focused_consumer_fingerprint: `a7062f2660467526899d4b25ce7f4f825e1c9d99e824530e71effc5c1ef30db2`  
> reference_fingerprint: `2c805aa50836390d3104a0a1254ff6e2a005453143246eeb201b65ee094b84ce`  
> status: review-only；不修改 production Rust，不运行 Cargo、Runtime DLL、Editor、GPU、跨进程、跨平台、fault、fuzz、scale、soak 或动态 benchmark。Tooling/Rust 迁移按用户要求排除；未查询、轮询、等待或实时跟踪协调器。

## 1. 结论

Interface04 的架构判断仍成立，但当前源码已经出现若干可保留的工程底座，不能再把全部问题写成“尚未实现”：profile session basename 已拒绝空名、`.`、`..`、分隔符和 Windows 保留名并追加稳定 hash；profile recorder 已记录 written/overwritten/oldest/newest retention；event mirror 已有 producer-side JSON byte/depth/deadline、16,384 events/64 MiB queue、64 deliveries/128 KiB payload page、256 KiB encoded page、sequence preflight 和 prepare/commit/rollback；Editor event consumer 已有 256 events/4 ms 总预算、每消费者 64 events、1 MiB retained bytes、round-robin、慢回调统计和 quarantine；Runtime Host 已有 per-output budget/fuse/typed metrics；native bridge context 已改成 generation handle、atomic snapshot、callback lease 和 close-and-wait；script build diagnostic 已能按 build generation/request/step 去重并投影到 EditorLog。

这些底座仍没有形成工程级公共合同：

1. `ProfileCaptureConfig` 的三个 `usize` capacity 仍由 caller 决定，没有 hard max、总 bytes、字符串或持续时间 admission；producer 先构造/克隆完整 snapshot 和多个派生 report，16 MiB/65,536 items consumer policy 发生在后段。
2. export 只修复了 session basename。`output_root` 仍是 caller path，Runtime 同步 `create_dir_all`、pretty JSON materialize 和固定文件名 `fs::write`，没有 host-owned attempt、quota、manifest、digest、staging、fsync 或 atomic publish。
3. native V4 host table 仍把 spawn/asset/event/diagnostics slots 设为 `Some`；其中 diagnostics callback 直接返回 `Ok` 并丢弃输入，spawn/asset/event 则返回 `UnsupportedVersion`。capability advertisement 与真实服务不一致。
4. public native descriptor/subtable 仍是手写、目标宽度相关的 Rust/C layout；module descriptor 无 `size_bytes`，nested tables 无各自 header，entry report 无 size/build/fingerprint，target modes/capabilities/schema 仍是未定义 raw bytes。
5. public plugin-event subscription 仍是裸 `u64`；成功 batch 只有 deliveries、remaining count 和 oldest age，没有 session epoch、generation、first/last sequence、dropped range、overflow/resync、producer clock 或 observation time。Runtime queue overflow 只在下次 drain 返回一次错误；App consumer 又把 backlog metadata 丢弃为 `Vec<Delivery>`。
6. profiling snapshot 没有 process/session/build/capture/clock/thread/task/page/digest identity。Editor 仍通过“当前最大 span ID + offset”合并两个 recorder，并拼接 session string；无法证明 ID、clock 或 retention owner 可比较。
7. script/plugin/native diagnostics 仍是彼此分离的小协议。script build 有 EditorLog bridge，但 plugin registration 主要停留在 catalog/pane DTO；native diagnostic/metric sink 仍是假成功，三者没有共享 code registry、source range、build/artifact/correlation/privacy/budget/truncation。

本轮对 Interface04 的 75 个旧条目重判：P0 为 **2 Open / 1 Partial**，P1 为 **51 Open / 9 Partial / 0 Closed**，P2 为 **9 Open / 2 Partial / 1 Closed**。32 项产品资格门为 **25 Fail / 7 Partial / 0 Pass**。本轮没有新增唯一 P0/P1/P2；报告数增加 1，canonical finding 总数不重复增加。Runtime156 已拥有 process-global recorder、observation mutation 与 config authority 等当前新增问题，Interface12/09 已拥有 shared Host output 断链，本文只回填 Interface04 的当前状态。

## 2. 审查边界与证据

### 2.1 物理范围

| 选择集 | files / lines / bytes / test attrs | dirty 状态 | fingerprint |
|---|---:|---:|---|
| Interface 直接合同文件 | **12 / 2,436 / 92,361 / 21** | **5 tracked modified / 3 untracked** | `ed0b93074f0aa89737d8ded815360f306d12a2bb09c91bcfe6a3fbfee02783eb` |
| focused Runtime/Host/App/Editor 生产消费者 | **101 / 24,164 / 856,736 / 175** | **49 tracked modified / 21 untracked** | `a7062f2660467526899d4b25ce7f4f825e1c9d99e824530e71effc5c1ef30db2` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics 参考选择 | **14 / 13,593 / 492,322 / 1** | n/a | `2c805aa50836390d3104a0a1254ff6e2a005453143246eeb201b65ee094b84ce` |

Interface 选择为 `lib.rs`、`manifest.rs`、`plugin_api.rs`、`plugin_diagnostics.rs`、`plugin_events.rs`、`profiling.rs`、`profiling/session_path.rs`、`runtime_api/session/plugin_event_mirror.rs`、`script_diagnostics` 三文件和 `tests/plugin_api_contracts.rs`。当前计数为 75 个顶层 `pub struct/enum/type/trait`、86 个 serde default 命中、5 个含 `usize` 的文件和 4 个含 raw pointer 的文件。计数只冻结物理变更面，不等于全部声明都应成为稳定 wire。

focused consumer 选择包含：

- Runtime profiling 与 runtime diagnostics 目录全量，dynamic API 的 exports/frame/diagnostics/profile FFI/event mirror，scene event mirror 目录全量；
- native loader 的 ABI declaration、behavior call、host callback、loaded callback owner，以及 bridge/context/registration/diagnostic slots；
- Runtime Host `foreign_output` 全量；
- Editor session gateway、runtime event consumer、script build、retained-host profiling 全量，以及 plugin capability catalog；
- App runtime library profile/event consumer和两个产品 diagnostics caller。

指纹算法为：规范化相对路径排序，逐文件 SHA-256，拼接 `path\0sha256\n` 后再取 SHA-256；未跟踪文件纳入当前观察指纹。它能证明本文读到哪一版源码，不能证明 clean checkout、编译或产品行为。

### 2.2 纵向调用链

本轮逐符号追踪了以下链：

1. `ProfileControlRequest` -> dynamic FFI bounded JSON decode -> process-global recorder/control -> full snapshot/report/export -> bounded response encode -> Host/App/Editor decode -> Editor snapshot merge；
2. native module/entry/host table -> registration scope advertisement -> context generation/lease -> system/component/event/diagnostic callbacks -> scheduler/catalog/diagnostic consumer；
3. scene typed event observer -> producer-side bounded JSON queue -> dynamic session pending page -> encoded batch commit/rollback -> Host/App decode -> Editor bounded consumer callback；
4. `ScriptDiagnostic` -> build completion -> generation/request/step cursor -> EditorLog，以及 `RegistrationDiagnostic` -> plugin catalog/capability pane；
5. profile artifact basename -> caller-controlled output root -> Runtime filesystem writes。

### 2.3 工作树与结构审计边界

选择集中存在大量未提交/未跟踪源码。本文不修改或回退这些代码，也不把 source-presence 写成已验证交付。`audit_runtime_structure.py --json` 分别在 30 秒和 60 秒的有界尝试中超时，没有生成结构审计结果；因此本文不宣称 touched crate 已完成 `IEntry/IManager/IDriver/IPlugin` 收敛，而改用上述定点生产调用链继续复核。clean-checkout source reproducibility 仍由 Interface10 `I10-P0-01` 唯一拥有，不在此重复登记。

### 2.4 本地参考源码

| 引擎 | 固定版本 / 选择 | 本轮用于建立的最低基线 |
|---|---|---|
| Unreal | `29dfa4a73de5dbc1a4eebe793b50db844c3db93e`；Cpu/Counters Trace、AnalysisSession、Plugin/Module Descriptor 5 文件 | trace spec 与 sample 分离，counter 有稳定 ID/type/display hint，analysis provider 有 read/edit scope；plugin/module descriptor 有文件/插件版本、module loading phase、target/platform/program allow/deny 与实际配置判断。 |
| Godot | `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`；Performance、EditorProfiler、GDExtension interface 3 文件 | monitor type、signature、frame metric和custom monitor modification time显式；GDExtension 以机器可读 type/function清单记录 `since`、`deprecated`、replacement和初始化/析构约束。 |
| Bevy | `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`；Diagnostic、Plugin 2 文件 | diagnostic path、measurement time、unit suffix、finite handling、history length与EMA语义显式；plugin lifecycle拆为build/ready/finish/cleanup并有唯一性规则。 |
| Fyrox | `8d815db36494f1badb347547dfc7094bf4fbbdf8`；dylib、Editor stats 2 文件 | dylib loader明确警告Rust ABI跨编译器不安全，保持library owner并用Loaded/Unloaded状态管理reload；stats UI只是renderer统计presentation，不冒充稳定transport。 |
| Unity Graphics | `a7e4c051d256a781ab362c64316b125a1e104694`；ProfilingScope、DebugManager 2 文件 | command buffer/profiling sampler scope与Debug UI panel/data register/unregister/reset生命周期分开；Graphics镜像不代表Unity完整Profiler或Plugin ABI，本文不据此外推闭源能力。 |

## 3. 当前可保留底座

1. `profile_session_basename()` 将输入限制为单一、96-byte、ASCII、hash-qualified basename；空名、`.`、`..`、分隔符、`CON/COM1/LPT9` 和 lossy collision 都有测试。
2. profile recorder 的 ring retention 已记录 capacity、written、overwritten、retained、oldest/newest sequence；`frame_budget_ms` 也会把 non-finite 值归一化，避免 NaN/Inf 继续传播。
3. event mirror producer 在序列化阶段限制单 payload 128 KiB、nesting和处理时间；subscription queue限制16,384条/64 MiB，页面限制64条/128 KiB，wire限制256 KiB。
4. dynamic event drain使用pending page和显式commit/rollback；sequence到达`u64::MAX`前预检可用窗口，单个确定性坏payload不会永久阻塞后续事件。
5. Editor event consumer有总/单consumer/time/retained-byte预算、pending-first、round-robin、sequence验证、stale gateway identity检查、panic quarantine与backlog observation。
6. Runtime Host foreign output统一执行encoded bytes、items、decode elapsed检查，按Profile/Event等kind保留accepted/rejected/call/blocked/bytes/decode metrics。
7. native callback owner已有generation handle、Arc pin、atomic directory、lease和close-and-wait；V4 system access/affinity校验与V3 conservative access可迁移。
8. script build diagnostic sink能拒绝stale completion、避免重复投影并输出EditorLog jump；这证明共享日志消费路径可实现，但尚未统一诊断协议。

## 4. P0 旧条目当前重判

| ID | 状态 | 当前源码证据与结论 |
|---|---|---|
| P0-01 | Open | `max_frames/max_spans/max_counters: usize` 仍没有hard max或总bytes/string/duration admission；`normalized()`只把0变1。caller可要求极大ring，snapshot仍全量clone，Host后置cap无法阻止producer分配和停顿。 |
| P0-02 | Partial | session basename traversal/collision已修复；但`output_root`仍由caller传入，Runtime同步创建目录并覆盖固定JSON/Perfetto文件，没有host-owned attempt、quota、manifest、digest、staging或durability。 |
| P0-03 | Open | V4 registration scope把diagnostic emit/metric暴露为`Some`，`native_host_diagnostics_emit_v1`与`metric_v1`仍直接返回`ZrStatus::ok()`且不保存任何输入。 |

## 5. P1 旧条目当前重判

状态语义：`Closed` 表示当前源码消除了旧命题；`Partial` 表示存在可迁移底座，但原公共合同仍不满足；`Open` 表示旧命题在当前生产链中仍直接成立。

### 5.1 Profile control、identity 与资源

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-01 | Open | profile control仍直接serde JSON，没有wire envelope、schema ID/fingerprint、reader range或unknown policy。 |
| P1-02 | Open | request仍无request/correlation/attempt ID。 |
| P1-03 | Open | request仍无deadline、cancel、priority；control/export/snapshot为同步调用。 |
| P1-04 | Open | request未限定target process/runtime session；per-session ABI实际控制process-global recorder。 |
| P1-05 | Open | optional `config`仍可附在所有command；非Start路径可静默忽略。 |
| P1-06 | Open | `normalized()`仍把0 capacity、空root和非法budget静默替换，caller得不到defaulted/rejected receipt。 |
| P1-07 | Partial | non-finite或非正`frame_budget_ms`现在会被发现并改为默认，避免NaN/Inf进入snapshot；仍不是typed拒绝。 |
| P1-08 | Open | capacity与多个diagnostic count继续使用`usize`进入serde/transport shape。 |
| P1-09 | Open | config仍无总memory、sample string、diagnostic、duration和CPU预算。 |
| P1-10 | Open | `include_perfetto`继续把capture policy与artifact格式耦合。 |
| P1-11 | Partial | export basename已有validated portable grammar/hash；public `session_id`本身仍是任意String，未成为stable observation identity。 |
| P1-12 | Open | response status仍是`String`，由`"ok"/"error"`约定语义。 |

### 5.2 Snapshot、metric 与 analysis

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-13 | Partial | Runtime内部capture state已有epoch并用于部分异步producer拒绝旧代；public snapshot/sample无capture ID/generation，普通scope/frame token仍不能跨reset证明归属。 |
| P1-14 | Open | snapshot没有producer process/runtime session/build/artifact identity。 |
| P1-15 | Open | timestamps没有clock ID、base/frequency、mapping或uncertainty。 |
| P1-16 | Open | span/frame/counter没有OS thread、fiber/task/queue identity。 |
| P1-17 | Open | span ID仍只在单recorder分配，DTO没有recorder owner；Editor只能做最大ID偏移。 |
| P1-18 | Open | frame index没有source、frame domain、tick/commit identity。 |
| P1-19 | Open | retention entry仍没有recorder ID；Editor合并时只是append两个匿名entry。 |
| P1-20 | Open | retention统计有oldest/newest sequence，实际frame/span/counter row仍不携带sequence。 |
| P1-21 | Open | snapshot仍无page/cursor/truncation/completeness/digest。 |
| P1-22 | Open | response仍可组合任意optional snapshot/diagnostics/hotspot/report/receipt/files状态。 |
| P1-23 | Open |同一response仍可携带snapshot、三类派生report和files，增加clone、JSON与Host decode放大。 |
| P1-24 | Open | RuntimeDiagnosticsSnapshot继续再次嵌入完整ProfileSnapshot。 |
| P1-25 | Open | RuntimeDiagnosticsSnapshot无统一observation time/generation/provider freshness/partial failure。 |
| P1-26 | Open | selected model/material/resource字段仍把canonical Cube示例写入公共合同。 |
| P1-27 | Open | render adapter/device identity仍无privacy/redaction/export policy。 |
| P1-28 | Open | diagnostic series继续由任意path/unit/tag字符串隐式定义。 |
| P1-29 | Open | measurement仍只有frame index与f64 value，无sequence/clock/quality/source。 |
| P1-30 | Open | current/smoothed/min/max仍无window、aggregation、missing/nonfinite语义。 |
| P1-31 | Open | series history在DTO层无独立page/byte/cardinality budget；Host item cap是后置总量。 |
| P1-32 | Open | scene reload diagnostics仍使用`usize`和多组重叠计数，没有状态机/receipt invariant。 |
| P1-33 | Open | hotspot report仍不绑定input snapshot digest、analysis version或completeness。 |
| P1-34 | Open | hotspot identity继续依赖stream/category/name/path字符串。 |
| P1-35 | Open | hints/alerts仍是自由文本，缺typed rule/threshold/evidence/fix-it。 |
| P1-36 | Open | `UiScenarioHotspot`仍把大量Editor实现计数固化进Runtime公共接口。 |
| P1-37 | Open | profile文件名是interface常量，但没有artifact manifest/schema/digest/size。 |
| P1-38 | Open | response `files/export_dir`仍是任意host path String，泄漏部署布局且无capability。 |
| P1-39 | Partial | Runtime encoder和Host consumer都有16 MiB/65,536 item等限制；recorder、runtime diagnostics和report仍在encode前完整materialize/clone。 |
| P1-40 | Open | decode elapsed只在`serde_json`返回后检查，不能抢占阻塞或高CPU decode。 |

### 5.3 Native manifest、ABI 与 lifecycle

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-41 | Open | `ZrPluginModuleDescriptorV1`仍无`size_bytes`。 |
| P1-42 | Open | target modes/capabilities仍是无codec/version/count/ownership说明的`ZrByteSlice`。 |
| P1-43 | Open | module kind仍是raw `u32`，没有unknown/preserve/reject policy。 |
| P1-44 | Open | entry report仍无size、plugin/build identity、SDK/compiler/layout fingerprint和compatibility receipt。 |
| P1-45 | Open | ECS/asset/event/bridge/diagnostics nested subtable仍无各自ABI/size/capability header。 |
| P1-46 | Open | V4把spawn/asset/event设为non-null却返回Unsupported，diagnostics更返回假Ok；slot advertisement不可信。 |
| P1-47 | Open | EventTypeId stable hash仍无算法/version/owner/collision registry；namespace/name/hash矛盾无统一admission。 |
| P1-48 | Open | native event emit/drain自身仍是stub，签名也没有schema revision、sequence、producer budget、receipt或continuity。dynamic event mirror进展不能替代native host服务。 |
| P1-49 | Open | native diagnostic/metric签名仍缺severity/code/source/correlation/unit/type/timestamp/privacy/budget。 |
| P1-50 | Open | V1/V4 native system closure继续`let _ = invoke(...)`，plugin failure不进入scheduler/quarantine result。 |
| P1-51 | Partial | generation context、Arc pin、callback lease和close-and-wait已明显改善DLL/context生命期；`user_data: u64`仍无显式destructor/lifetime token和exactly-once release contract。 |
| P1-52 | Open | component schema仍是raw bytes；当前registration甚至只消费type/display name并丢弃schema。 |
| P1-53 | Open | plugin state snapshot仍无schema revision、size negotiation、digest、migration、transaction或max bytes。 |
| P1-54 | Open | callback event time仍为`f32 seconds`，无clock/domain/precision policy。 |
| P1-55 | Partial | callback dispatch现在可由library generation owner和lease保护；public request仍无correlation、deadline、cancel、owner/session generation。 |
| P1-56 | Open | callback result仍只有嵌套`ZrStatus`，没有handled/retry/partial/output/side-effect receipt。 |

### 5.4 Dynamic plugin event continuity

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-57 | Open | subscribe仍使用raw `event_id/payload_schema` String；只限制request bytes，未绑定Event Registry ID/revision/owner。 |
| P1-58 | Partial | scene内部handle为slot+generation，Editor也绑定gateway identity与consumer generation；公开ABI handle仍只是非0 `u64`，session restart/reuse语义未发布。 |
| P1-59 | Partial | queue/page/sequence/backlog与overflow error是真实进展；成功batch仍无dropped range、overflow state、cursor、ack或ResyncRequired，App还丢弃remaining/age。 |

### 5.5 Diagnostic convergence

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-60 | Partial | script build diagnostic现在有generation-aware EditorLog投影；RegistrationDiagnostic仍只用于catalog/pane，native sink仍假Ok，三类DTO/Severity/identity未统一。 |

## 6. P2 旧条目当前重判

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P2-01 | Open | ProfileControl enum仍混合capture lifecycle、snapshot query、diagnostics query和filesystem export。 |
| P2-02 | Open | 默认16.67 ms仍不是精确refresh rational或显式产品frame policy。 |
| P2-03 | Open | `target/zircon-profiles`默认root仍属于Host/Tooling artifact策略。 |
| P2-04 | Open | snapshot的`active/feature_enabled`仍可形成重叠或矛盾布尔组合。 |
| P2-05 | Partial | Editor在接收backlog时记录本地`Instant`并计算observation age；wire中的oldest pending age仍没有producer clock或observation timestamp。 |
| P2-06 | Open | delivery `PartialEq`仍把两个RawValue重解析为Value并使用`expect`。 |
| P2-07 | Open | play session ID在event delivery中仍是裸`u64`。 |
| P2-08 | Open | RegistrationDiagnostic仍缺stage、package/module/artifact/build identity。 |
| P2-09 | Open | ScriptSourceLocation仍未定义line/column base、encoding、range/end或Unknown位置。 |
| P2-10 | Open | `lib.rs`继续顶层重导出profile/plugin/script细节，internal/stable profile没有分层。 |
| P2-11 | Closed | `src/host_output`目前为空目录且没有module/file/public export；旧源合同已从Interface树移除。应在后续目录清理中删除空壳，但不再构成公共API。 |
| P2-12 | Partial | 当前新增session basename、event budgets/rollback/sequence、consumer pump、native generation/lease与diagnostic projection测试；仍缺generated C/cross-arch/old-new/fault/fuzz/scale产品矩阵。 |

## 7. 当前关键断链

### 7.1 Profile admission 与 snapshot transport 仍颠倒

正确顺序应是先由Host创建capture attempt和资源budget，再让producer以bounded pages写入。当前顺序仍是caller提交任意capacity/path，Runtime创建ring、全量clone、构造派生report，最后encoder/Host才检查16 MiB与item数。后置限制只能保护consumer，不能保护Runtime内存、锁持有时间和帧延迟。

### 7.2 Profile identity 不足以合并

Editor `merge_profile_snapshot()` 用现有最大span ID作为offset，随后append frames/spans/counters/retention并拼接session string。没有recorder/process/capture/clock identity时，offset不能解决parent source、thread lane、clock epoch、retention owner或restart collision；正确行为应是未映射clock分lane显示，而不是制造一条伪统一timeline。

### 7.3 Native capability advertisement 不可信

V4 registration scope同时暴露真实registration、Unsupported stubs和Ok-discard diagnostics。plugin无法通过table shape区分implemented、authorized、temporarily unavailable；Host也无法审计“每个Ok进入哪个sink”。下一版必须由generated service header与capability negotiation决定non-null slots，兼容表只作有期限adapter。

### 7.4 Event continuity 只在进程内部部分存在

Runtime内部有generation handle、bounded queue、sequence和pending page，Editor内部有gateway generation、pending budget与quarantine；这些身份没有跨ABI。overflow error被消费后，后续成功batch无法说明此前drop了多少、应从哪里resync。App把batch压成`Vec<Delivery>`又丢失remaining/age，说明同一公共合同在两个consumer中语义不同。

### 7.5 Diagnostic bridge 只接通一条支线

Script build sink证明按generation/request/step去重并投影EditorLog可行；plugin catalog diagnostic仍是小型capability报告，native diagnostic/metric完全丢弃。目标必须是一个共享、bounded、generation-qualified envelope，再由EditorLog、Plugin Manager、telemetry和artifact导出分别投影。

## 8. 目标合同

| profile / owner | 权威内容 | 禁止继续暴露 |
|---|---|---|
| `ObservationControlV2` / Runtime diagnostics service | request/correlation/deadline/cancel/target、hard admission、typed outcome | 任意capacity/path和String status |
| `ObservationPageV2` / recorder owner | process/session/build/capture/recorder/clock/thread/task、sequence/cursor/drop/digest/completeness | 匿名完整Vec snapshot与最大ID偏移merge |
| `MetricSpanRegistryV1` / schema registry | stable ID、owner、unit/type/aggregation/privacy/cardinality | 任意path/category/name/tag定义身份 |
| `ArtifactAttemptV1` / Host artifact service | root capability、quota、staging、manifest/digest/fsync/atomic publish/retention | Runtime解释caller filesystem path |
| `NativeServiceTableVNext` / generated ABI owner | per-table version/size/capability、fixed widths、implemented+authorized slots、layout corpus | 手写nested tables与Some/Unsupported/Ok-discard |
| `PluginEventStreamV2` / Runtime event registry | schema owner/revision、epoch/generation、sequence range、drop/overflow/resync、ack/cursor | raw strings、裸handle与无gap成功batch |
| `DiagnosticEnvelopeV2` / diagnostics core | code/severity/stage/source/build/artifact/correlation/privacy/budget/truncation/cause | script/plugin/native三套孤立DTO和自由文本身份 |

## 9. 重构顺序

### M0 · 立即封闭三项P0

- profile config加入编译期hard maxima、checked total bytes/string/duration admission；在分配ring前返回typed LimitExceeded。
- dynamic Runtime禁用caller-selected export root；只返回bounded observation pages，由Host分配artifact attempt。
- diagnostics slots未实现时必须为None/Unsupported；接入sink后每个Ok都必须可按correlation查询。

### M1 · Observation identity 与 clock

- 定义Process/RuntimeSession/Build/Capture/Recorder/Clock/Thread/Task identity和mapping uncertainty。
- sample带recorder/sequence/clock；snapshot/page带capture generation和source digest。
- Editor只合并同capture且clock可映射的数据；否则分lane并显示unsupported mapping。

### M2 · Registry 与 bounded pages

- 建立Metric/Span Registry，固定unit/type/aggregation/privacy/cardinality和extension namespace。
- control改为async request/receipt/cancel；snapshot改cursor page、drop range、completeness、digest。
- RuntimeDiagnostics provider分别分页并报告fresh/stale/partial/error，不再嵌套完整profile。

### M3 · Host Artifact Service

- Host创建attempt、quota、ACL和retention；Runtime只获得受限writer或stream capability。
- manifest列schema/version/digest/size/source；staging reread验证后atomic publish。
- 对write/flush/fsync/rename各故障点建立child-process恢复矩阵。

### M4 · Generated native ABI

- 从IDL生成Rust Host/SDK、C/C++ header、JSON schema、layout snapshot和docs。
- 每张service subtable独立version/size/capability；所有count/size固定宽度且有per-field cap。
- callback request加入owner generation/correlation/deadline，result进入scheduler/quarantine/diagnostic receipt。

### M5 · Plugin event stream V2

- Event Registry替代raw event/schema strings和caller hash。
- subscription/page加入session epoch、generation、first/last sequence、dropped range、overflow/resync、producer time和cursor。
- Host、App、Editor使用同一batch语义；不得在App适配层丢失backlog/continuity字段。

### M6 · Diagnostic convergence

- script/plugin/native/UI/runtime diagnostic迁入共享envelope/code registry。
- sink统一执行count/bytes/per-code/dedup/suppression/privacy/truncation budget。
- EditorLog与Plugin Manager只做typed projection，不解析message恢复身份。

### M7 · Compatibility 与发布资格

- 冻结V1/V3/V4与VNext old/new host-plugin矩阵、support window和sunset telemetry。
- profile/event/diagnostic保存golden corpus、unknown/skew/malformed/fuzz/fault/scale测试。
- 在clean checkout、真实DLL、x86_64/aarch64、Windows/Linux和cross-language consumer上生成绑定BuildSet的资格artifact。

## 10. 32 项产品资格门当前状态

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01-G02 | Fail | profile capacity无hard max，总bytes/string/duration admission不存在。 |
| G03 | Partial | non-finite budget会归一化；仍不typed拒绝，zero/overflow/unknown组合也未闭合。 |
| G04 | Fail | Runtime仍根据request中的output root选择filesystem路径。 |
| G05 | Partial | session basename corpus已安全；caller root与固定文件覆盖仍存在。 |
| G06 | Fail | export无staging/flush/fsync/rename恢复和完成manifest。 |
| G07 | Fail | snapshot无source/capture/clock/page/cursor/digest/completeness。 |
| G08-G10 | Fail | Editor仍无clock mapping，process/thread/task/restart identity也不存在。 |
| G11 | Fail | retention无recorder ID，sample无sequence。 |
| G12 | Partial | event producer memory有界；profile producer仍全量materialize且不可cancel。 |
| G13-G18 | Fail | diagnostics provider freshness、canonical field移除、registry、nonfinite sample、analysis digest和artifact manifest均未完成。 |
| G19 | Fail | ABI仍含`usize`且无generated C/cross-arch qualification。 |
| G20 | Fail | Host继续为unsupported/假成功服务暴露non-null slot。 |
| G21-G23 | Fail | diagnostics Ok不可查询，payload/bad UTF-8/nonfinite sink规则缺失，system status仍被吞。 |
| G24 | Partial | generation owner、lease、close-and-wait存在；user context/token exactly-once release和完整reload fault资格缺失。 |
| G25 | Fail | EventTypeId hash算法/collision catalog不存在。 |
| G26 | Partial | overflow会产生一次typed drain error；成功batch仍无dropped range/ResyncRequired。 |
| G27 | Partial | sequence rollover可检测，public epoch/generation与handle reuse仍不可检测。 |
| G28 | Partial | event producer/page/encoded/item限制已对齐；decode deadline后置且未做动态压力资格。 |
| G29-G31 | Fail | source range base/encoding、共享diagnostic namespace、old/new compatibility corpus缺失。 |
| G32 | Fail | 本轮只有静态review，没有绑定BuildSet的动态qualification artifact。 |

合计：**25 Fail / 7 Partial / 0 Pass**。

## 11. 验证说明

本轮没有运行Cargo，因为目标是当前源码公共合同review，且工作树包含共享未提交与未跟踪实现；编译通过也不能证明资源、ABI、continuity或artifact资格。完成的验证包括：12个Interface合同文件物理扫描、101个focused producer/consumer文件逐符号追踪、公开声明/serde/usize/raw pointer/test与dirty计数、三组内容指纹、五套本地参考源码核对、Interface04 75项旧finding重判，以及报告编号/链接/状态/索引的静态检查。

结构审计脚本两次有界超时，结果记为unavailable而不是Pass。任何实施阶段必须重新取指纹，并补M0-M7对应的真实DLL、cross-language、fault、scale与artifact证据。当前`Partial`只表示存在可复用底座，不表示性能、兼容性或表现已达到或超过Unreal。

## 12. 审查决策

`zircon_runtime_interface` 当前不能把profiling称为producer-safe transport，不能把native diagnostics称为可用，不能把native host table称为capability-truthful，也不能把plugin-event sequence称为无缺口。允许保留session basename、retention accounting、event producer/page budgets、commit/rollback、Editor bounded pump、Host metrics、native generation lease和script EditorLog projection；禁止继续扩大path-bearing profile control、匿名完整snapshot、最大ID偏移merge、手写不对称host table、Some/Unsupported/Ok-discard slot、裸event handle和孤立diagnostic DTO。

下一实施切片必须从M0 admission/artifact ownership/diagnostic truth开始；在三项P0关闭前，不应增加新的profile report字段、native service slot或依赖当前event batch连续性的产品功能。

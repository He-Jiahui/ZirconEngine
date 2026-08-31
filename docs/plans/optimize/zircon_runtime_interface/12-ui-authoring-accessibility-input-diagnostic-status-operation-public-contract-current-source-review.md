# `zircon_runtime_interface` UI authoring / accessibility / input / diagnostic / operation 公共合同当前源码复核

> report_id: `Interface12`  
> kind: `current-source-review`  
> canonical_source: [03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md](03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md)  
> review_scope: `zircon_runtime_interface::ui`，以及 Runtime、Runtime Host、App、Editor 中 authoring、compiled package、runtime tree、reflection/control、accessibility、input side effect、diagnostic 和 operation 的生产消费者。  
> baseline: 当前工作树 `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f`（2026-08-29）；未提交源码只作为当前观察点，不视为已交付能力。  
> interface_ui_fingerprint: `fdddb3e4de4bee990865d22131af044544ba3ec9710c0274042f92c8cbfe9e1c`  
> focused_consumer_fingerprint: `57b869daa35c98e4409ffc970171033172f7ad2bf4d6c00e049ae4c8bb25a986`  
> reference_fingerprint: `6d4eed0a7c10ce1941c51bf1d8b9c40cf2acae14759225e0c2dfe2c54344a12b`  
> status: review-only；不修改 production Rust，不运行 Cargo、Editor、Runtime DLL、GPU、跨进程、跨平台、fault、fuzz、scale、soak 或动态 benchmark。Tooling/Rust 迁移按用户要求排除；未查询、轮询、等待或实时跟踪协调器。

## 1. 结论

Interface03 的核心判断仍然成立，但不能继续概括为“全部未实施”。当前源码已经补入若干真实底座：v2 graph compiler 使用迭代 DFS 并拒绝 cycle 与跨父挂载；`UiSurfaceFrame` 具备 layout/render/hit/focus/pipeline/window 分域 generation；accessibility producer 在 snapshot build 和 JSON encode 两阶段执行 16 MiB、65,536 items、nesting 与 250 ms 限制；input effect 有原子回滚、applied/rejected 列表和部分 text/number/clipboard receipt；operation service 有固定布局 poll、task/byte admission、deadline、worker/apply、terminal TTL、cancel 和 harvest；Runtime Host 有 typed metrics snapshot、per-kind budget、decode accounting 与 session fuse。

这些进展仍没有把公共合同闭合：

1. Editor 保存 `.zui` 仍经过 v2 -> legacy projection -> v2 rebuild。上一份 v2 文档只帮助保留 `pixel_snapping`、`state` 与 component `default_classes`；`repeat` 和 node `slots` 仍固定丢弃，legacy projection 仍把 focus/navigation/picking/a11y/widget 与 component API version 置空，全文仍由 `toml::to_string_pretty` 重写。P0 数据损失只获得局部缓解。
2. accessibility 快照自身没有 generation/epoch，action request 也没有 observed snapshot token。动态路径仍以 surface 顺序投影 node ID，并按当前 surface vector 反解；`generation_hint` 没有被读取。producer budget 能防止一部分资源耗尽，但不能防止 stale action 命中新实例。
3. Runtime 暴露 accessibility capture/action ABI，却没有 App/Editor accessibility tree consumer；Host 的 `RuntimeForeignOutputKind` 和 policy 仍只有七类，不含 accessibility。App 对通用 `UiHost`/`UiAction` 只做按 2 的幂次限频的“unhandled”日志，跨边界 side effect 没有执行 receipt。
4. reflection/control 仍以 path、裸 u64、`serde_json::Value`、`Option` 和 `Ack` 表达查询与变更；diff 无 base/new revision，subscription 使用 unbounded crossbeam channel，无 lease、overflow、ack、resync。
5. operation 内部状态机已明显工程化，但 public submit/result 仍是 `operation_id: String + JSON`，API table 仍无 cancel entry，Cancelled/Expired 也不能被 harvest outcome 表达。内部 cancel 能力不能替代跨 DLL 合同。
6. UI public surface 从旧报告的 642 个公开声明增长到 745 个；`#[serde(default)]` 命中从 431 增至 665，仍为 0 个 `deny_unknown_fields`。当前 dirty/untracked source 使 clean checkout 可复现性继续由 Interface10 `I10-P0-01` 阻断，本报告不重复登记。

本轮对 Interface03 的 87 个旧条目重判：P0 为 **1 Open / 2 Partial**，P1 为 **60 Open / 12 Partial / 0 Closed**，P2 为 **10 Open / 2 Partial / 0 Closed**。32 项产品资格门为 **19 Fail / 13 Partial / 0 Pass**。本轮没有新增唯一 P0/P1/P2；报告数增加 1，canonical finding 总数不重复增加。

## 2. 审查边界与证据

### 2.1 物理范围

| 选择集 | files / lines / bytes / test attrs | dirty 状态 | fingerprint |
|---|---:|---:|---|
| `zircon_runtime_interface/src/ui/**/*.rs` 全量 | **248 / 32,932 / 1,056,937 / 83** | **69 tracked modified / 17 untracked** | `fdddb3e4de4bee990865d22131af044544ba3ec9710c0274042f92c8cbfe9e1c` |
| focused Runtime/Host/App/Editor 生产消费者 | **463 / 92,811 / 3,231,858 / 461** | **196 tracked modified / 121 untracked** | `57b869daa35c98e4409ffc970171033172f7ad2bf4d6c00e049ae4c8bb25a986` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics 参考选择 | **20 / 15,151 / 539,471 / 30** | n/a | `6d4eed0a7c10ce1941c51bf1d8b9c40cf2acae14759225e0c2dfe2c54344a12b` |

focused consumer 选择包含：

- Runtime `ui/accessibility`、`ui/v2`、`ui/template/asset`、`ui/event_ui`、`ui/surface/input`、`asset/assets/ui`、`operation`，以及 dynamic frame/session 的 accessibility 与 host-output 路径；
- `zircon_runtime_host/src/foreign_output` 全量；
- Editor `ui/asset_editor` 全量和 session frame/operation/output/protocol gateway；
- App `runtime_entry_app/host_requests` 全量。

Interface UI 当前静态压力为 745 个 `pub struct/enum/type/trait`、665 个 `#[serde(...default...)]` 命中（104 个文件）、0 个 `deny_unknown_fields`、13 个 `toml::Value` 文件、53 个 `usize` 文件和 50 个名称含 Diagnostic/Status/Report/Result 的公开 family。计数用于冻结变更面，不等于所有声明都应成为稳定 wire。

指纹算法为：规范化相对路径排序，逐文件 SHA-256，拼接 `path\0sha256\n` 后再取 SHA-256；未跟踪文件纳入观察指纹。它能证明本文读到哪一版源码，不能证明 clean checkout、编译或产品行为。

### 2.2 纵向调用链

本轮逐符号追踪了以下链，而不是只搜索类型名：

1. `UiTemplateDocument` / `UiAssetDocument` / `UiV2AssetDocument` -> Runtime 三组 loader/migrator -> v2 compiler/arena/package/cache -> Runtime surface；
2. Editor source buffer -> v2 parse -> legacy projection -> edit -> previous-v2-assisted rebuild -> pretty TOML -> durable write/source revision；
3. retained `UiTree` -> surface frame publication/domain generations -> reflection/debug snapshots -> control mutation/diff/subscription；
4. ABI accessibility request -> bounded surface rebuild/extract/globalize/encode -> owned JSON -> action decode -> global ID split -> fresh current snapshot -> dispatch result；
5. input metadata -> route/reply/effect transaction -> Runtime side effects -> typed host request -> Host foreign-output page -> App routing/unhandled sink；
6. operation submit JSON -> raw admission -> handler registry -> worker preparation -> owner apply -> fixed poll -> harvest JSON -> Editor Host bounded decode；
7. UI diagnostics/debug snapshot -> Editor reflector/export，以及 Host typed metrics -> presentation diagnostic line。

### 2.3 工作树边界

Interface UI 选择中 17 个文件未跟踪，focused consumer 中 121 个文件未跟踪。当前代码可能是其他正在进行的工作；本文不修改、不回退，也不把它们写成已发布能力。公开 source clean-checkout 复现问题已由 Interface10 `I10-P0-01` 唯一登记；在该项关闭前，本文所有 Partial 都必须在 clean checkout 重新取指纹和复核。

### 2.4 本地参考源码

| 引擎 | 固定版本 / 选择 | 本轮用于建立的最低基线 |
|---|---|---|
| Unreal | 当前根提交 `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f`；Slate/UMG 5 文件 | `FSlateAccessibleWidget` 有 application-unique ID、parent/children/update owner，并按 window/property/text 等 capability 拆接口；`FReply` 只提交 capture/focus/navigation intent，由 application 执行；Widget Blueprint generated class、WidgetTree、runtime instance 分层。 |
| Godot | `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`；Input/Control/ResourceLoader/Error 5 文件 | typed InputEvent 层次、Control 传播/accept、ResourceFormatLoader 的识别/依赖/UID owner，以及稳定 Error 与 file/function/line context。 |
| Bevy | `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`；a11y/UI/diagnostic 4 文件 | AccessKit/ECS 生命周期投影，diagnostic path、measurement/history/suffix 与明确保留策略；进程内 ECS component 不应直接冒充稳定 DLL wire。 |
| Fyrox | `8d815db36494f1badb347547dfc7094bf4fbbdf8`；UI message/pool handle 3 文件 | `UiMessage` 的 destination/direction/handled/flags 与 index+generation handle；路由目标和对象代际不能压成可复用裸整数。 |
| Unity Graphics | `a7e4c051d256a781ab362c64316b125a1e104694`；Debug/Migration 3 文件 | Debug panel/data register-remove-reset lifecycle；持久 Version 与连续 migration。Graphics 镜像不是 Unity UI Toolkit 源码，不据此外推闭源 UI 能力。 |

## 3. 当前可保留底座

1. `UiV2DocumentCompiler` 已改用显式 `Enter/Exit` 栈，拒绝 cycle、missing child 和不同 parent 的重复挂载；node handle 也限制在 u32 容量。
2. compiled asset header/cache 已记录 source/compiler/package version、root/import/resource/component fingerprint，并在 cache 命中时重验 header 与 key。
3. `UiTreeNodes` 的 mutable entry point 会登记 dirty source，paint-order cursor 避免每次插入全表扫描；node reincarnation 在同一 live tree 内利用单调 paint order 区分 remove/reinsert。
4. `UiSurfaceFrameDomainGenerations` 已把 layout/render/hit/focus/pipeline/window 的变化分开发布，Runtime 有对应 authority tests。
5. accessibility build 在 producer 侧计 encoded bytes/items/nesting/deadline，dynamic encoder再次执行同一 interface limit；extract/action 对 role、hidden/missing target、text/range/widget行为已有大量 focused tests。
6. input reply 统一承载 effect intent，Runtime 对复合 effect 捕获 mutation snapshot并原子回滚；secure text result redaction、text constraint、number input 与 clipboard receipt 是正确方向。
7. operation service 已有 max tasks/retained bytes、raw request reservation、deadline、worker panic/channel loss、owner apply、terminal TTL、cancel/harvest race保护；poll carrier为 allocation-free fixed layout。
8. Host foreign-output state 已有 typed counters/snapshot、per-kind decode policy、release cleanup、protocol fuse 与 metrics；文本行已经只是一个 presentation adapter，但还缺 session/build transport identity。

## 4. P0 旧条目当前重判

| ID | 状态 | 当前源码证据与结论 |
|---|---|---|
| P0-01 | Partial | `last_valid_v2_document` 让 rebuild 保留 `pixel_snapping`、`state`、`default_classes`，并有 source revision 持久化检查；但 `repeat: None`、`slots: BTreeMap::new()` 仍是硬编码，focus/navigation/picking/a11y/widget/API version 仍无法通过 legacy projection，unknown syntax/comment/trivia 也被 pretty serializer删除。数据损失仍可发生。 |
| P0-02 | Partial | accessibility snapshot build/encode已有 producer limit；authoring仍以 `read_to_string`、`toml::from_str` 读取无界 source，三条 loader authority重复，recursive legacy tree、TOML node/string/map、compile/package没有共享 admission/cancel。原问题只关闭了 accessibility 的一部分。 |
| P0-03 | Open | snapshot/action没有 generation/epoch/observed token；`generation_hint` 未读。global node ID仍由 surface顺序投影，action按当前surface vector反解并重新取当前snapshot；`UiAccessibilityActionResult`未作为ABI response，stale/recycled/overflow仍不能结构化证明。 |

## 5. P1 旧条目当前重判

状态语义：`Closed` 表示当前源码消除了旧命题；`Partial` 表示存在可复用底座，但原合同仍不满足；`Open` 表示旧命题在当前生产链中仍直接成立。

### 5.1 公共层职责与版本面

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-01 | Open | interface UI仍同时拥有 source syntax、mutable tree、layout、ECS、dispatch、render/debug和transport DTO。 |
| P1-02 | Open | public声明增至745个，仍无family-level stability/owner/profile/deprecation manifest。 |
| P1-03 | Open | authoring、compiled artifact、live tree、debug snapshot和JSON wire继续复用serde public shape。 |
| P1-04 | Open | 除局部asset/package version外，control/accessibility/input/debug没有统一schema ID、fingerprint、reader range和capability set。 |
| P1-05 | Open | `serde(default)` 增至665处；缺字段仍可变成空ID、0、false或首个业务状态。 |
| P1-06 | Open | 0个`deny_unknown_fields`，authoring也没有unknown-field/CST preservation。 |
| P1-07 | Open | 大量closed enum依赖Rust variant spelling，缺`Unknown(raw)`与negotiation。 |
| P1-08 | Open | 53个UI文件含`usize`；a11y selection、debug counts、effect/trace indices仍进入serde shape。 |
| P1-09 | Open | public fields仍可构造矛盾结果；`UiInvocationResult`的value/error组合未改为tagged outcome。 |
| P1-10 | Open | string path/tree/action/property与裸u64 runtime ID仍没有source/compiled/runtime/transport分层。 |
| P1-11 | Open | 多个ID derive Default生成0或空字符串，跨边界decode没有统一invalid placeholder拒绝。 |
| P1-12 | Open | 未生成UI contract manifest、symbol/schema digest、compatibility window、budget和golden corpus清单。 |

### 5.2 Authoring、编译与迁移

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-13 | Open | recursive `UiTemplateNode` 和legacy template loader仍有生产consumer，没有read-only freeze/sunset。 |
| P1-14 | Open | `UiAssetDocument`仍是Editor编辑投影，`UiV2AssetDocument`仍是source/preview/compiler入口；没有唯一canonical semantic IR。 |
| P1-15 | Open | v2 loader仍只接受恰好version 2，没有连续migration graph、loss receipt或downgrade policy。 |
| P1-16 | Open | `ui/v2/loader.rs` 与 `asset/assets/ui/document_loader.rs` 仍重复parse/version/profile validation。 |
| P1-17 | Open | props/state/layout/slots/params/tokens/style仍大量使用任意`toml::Value`。 |
| P1-18 | Open | node/component/slot/class/selector/binding/resource/action继续使用raw String，没有stable rename identity/redirect transaction。 |
| P1-19 | Open | parse后只保留semantic DTO；没有source span、comment/trivia、raw token与syntax anchor。 |
| P1-20 | Partial | v2 compiler的迭代DFS已拒绝cycle和跨父挂载；但只验证root/component root可达子图，未证明全部nodes有唯一owner，orphan/full-document policy仍缺失。 |
| P1-21 | Open | recursive legacy tree无node/depth/stack admission，parse和helper仍可先消耗资源。 |
| P1-22 | Open | authoring string/map/list/value没有跨read/parse/validate/compile/package累计的统一budget。 |
| P1-23 | Open | Editor保存仍调用`toml::to_string_pretty`，不是lossless syntax patch/transaction receipt。 |
| P1-24 | Open | projection API只返回document/error，没有retained/dropped/defaulted/renamed字段的machine-readable loss report。 |
| P1-25 | Partial | package/cache已有import/resource/component fingerprints和dependency entries；仍缺font/icon/plugin等完整closure、optional policy、BuildSet和稳定ID/version。 |
| P1-26 | Open | artifact/package仍由Rust serde/TOML shape驱动，没有独立language-neutral section/layout/endianness/unknown-section specification。 |
| P1-27 | Open | artifact与validation report仍可在普通struct中组合，类型系统未保证error-free report才可publish。 |
| P1-28 | Partial | Editor source buffer有revision并在durable write后检查expected revision，cache也绑定source fingerprint；compile/publish仍没有统一document revision/CAS/attempt与stale result rejection contract。 |

### 5.3 Runtime tree、reflection 与 control

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-29 | Open | `UiTree` 仍derive Deserialize且fields public，可恢复duplicate root、missing parent、cycle、slot mismatch等非法live graph。 |
| P1-30 | Open | `insert_root` 遇重复node仍直接return；`insert_child`则返回`DuplicateNode`。 |
| P1-31 | Open | layout generation、pending mutation/layout sets与paint cursor使用`serde(skip)`；restore后观察/dirty语义仍由隐式重建决定。 |
| P1-32 | Partial | frame新增六个domain generations并按实际domain变化推进；`UiSurfaceFrame`仍一次携带arranged/render/hit/focus/window/pipeline/debug mega snapshot，没有按需page/budget。 |
| P1-33 | Open | `UiReflectionSnapshot`与`UiReflectorSnapshot`仍由Runtime/Editor两条生产消费链使用。 |
| P1-34 | Open | reflection snapshot/diff仍没有tree epoch、base/new revision、sequence、overflow/resync或digest。 |
| P1-35 | Open | patch继续用`UiNodePath + BTreeMap<String, JSON>`，没有stable property ID、CAS、principal或transaction ID。 |
| P1-36 | Open | property/action descriptor仍缺稳定schema/type identity；`UiValueType::Any`和JSON可绕过。 |
| P1-37 | Open | remote权限仍是`callable_from_remote: bool`，没有principal/capability/scope/policy revision。 |
| P1-38 | Open | `UiControlRequest`仍没有request/correlation、deadline、cancel、caller或schema envelope。 |
| P1-39 | Open | SetProperty/CallAction仍不携带tree/document/instance generation或expected revision。 |
| P1-40 | Open |成功写入和unsubscribe仍返回空`Ack`，不能证明before/new revision、change set和side effects。 |
| P1-41 | Open | query使用`Option`表示not-found，无法区分permission、stale、partial和unknown tree/property。 |
| P1-42 | Open | `UiInvocationResult`仍能构造value+error、两者皆无等矛盾状态。 |
| P1-43 | Open | subscription ID仍是裸u64；Runtime使用`crossbeam_channel::unbounded()`，没有lease、容量、overflow、ack和resync。 |

### 5.4 Accessibility、input 与 side effects

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-44 | Open | `capture_accessibility_tree`不读取`request.generation_hint`，每次resize/rebuild/capture。 |
| P1-45 | Open | App/Editor生产代码没有accessibility tree capture/update/remove/focus/action consumer；AccessKit adapter只存在Runtime内部。 |
| P1-46 | Open | Host output kind/policy/item counter没有Accessibility；该输出绕过统一consumer fuse/metrics family。 |
| P1-47 | Partial | producer已有总量/bytes/depth/time限制；snapshot仍是单个全树Vec，没有page/delta/continuation/completeness/truncation contract。 |
| P1-48 | Open | role/state/action/relation仍是closed enum+optional field集合，没有platform capability/version/unknown preservation。 |
| P1-49 | Open | `UiA11yTextSelection`仍使用`usize`，跨目标字宽和UTF adapter合同未固定。 |
| P1-50 | Open | typed `UiAccessibilityActionResult`仍无ABI producer/consumer；Runtime把status编码进diagnostic note并向动态caller只返回handled/ZrStatus。 |
| P1-51 | Open | accessibility diagnostics仍附在每次全树snapshot，没有独立stream/dedup/suppression/page。 |
| P1-52 | Open | global ID仍依赖surface顺序与bit projection，path仍格式化`surface-{index}:...`；协议未发布overflow/namespace/lifetime。 |
| P1-53 | Partial | effect transaction、secure-text redaction和若干receipt是真实进展；public result仍复制完整event、reply、route、notes、events和binding reports，缺统一hot-path trace budget。 |
| P1-54 | Open | pointer/navigation仍有平行invocation/result/diagnostic模型，未投影为统一dispatch outcome。 |
| P1-55 | Partial | input metadata已含user/device/window/surface/pointer/source/sequence/monotonic micros；ID仍可默认0，focus/capture没有owner epoch、device reconnect和surface generation admission。 |
| P1-56 | Partial | IME/clipboard/cursor有部分Host执行，generic host request也带viewport/surface/input sequence/request/effect index；App对pointer lock/popup/tooltip/link等`UiHost`仍只记unhandled日志，且没有accepted/denied/failed receipt。 |

### 5.5 Diagnostic、operation 与 host output

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-57 | Open |名称匹配family增至50个，仍没有共享diagnostic/status/report/result envelope。 |
| P1-58 | Open | typed enum、raw String、message-only error继续并存，没有namespaced code registry。 |
| P1-59 | Open | accessibility/binding/invalidation/localization/resource/schema各有severity enum，debug severity仍是`Option<String>`。 |
| P1-60 | Open | authoring diagnostic没有统一file/line/column/semantic address，runtime diagnostic没有artifact/build/generation identity。 |
| P1-61 | Partial | accessibility/foreign output和部分App warning已有限制或对数抑制；多数UI report Vec仍无总bytes/per-code/dedup/suppressed/truncated marker。 |
| P1-62 | Open | parse/compile/load/dispatch/host路径没有统一correlation、attempt、parent cause和operation identity。 |
| P1-63 | Open | `UiDebugEventRecord`仍用raw `kind/summary` String，overlay severity也是String。 |
| P1-64 | Open | debug snapshot仍是完整tree/render/hit/overdraw/event集合，没有principal、redaction policy、page、filter receipt与payload budget。 |
| P1-65 | Open | debug capture只有`captured_at_millis`，没有clock ID/epoch/frequency；input monotonic字段也没有跨进程clock identity。 |
| P1-66 | Open | public operation submit仍是任意String ID与JSON payload，没有descriptor/schema/capability/idempotency。 |
| P1-67 | Open | handle仍只验证非0，不绑定session/service epoch/generation；跨restart/reuse语义未发布。 |
| P1-68 | Partial | Runtime service内部已有cancel并测试queued/preparing/ready边界；public API table和Editor gateway仍没有cancel函数/ack，所以caller不能使用。 |
| P1-69 | Open | harvest outcome仍只有Succeeded/Failed；Cancelled/Expired在service层返回error，终态原因不能作为结果receipt传输。 |
| P1-70 | Open | progress仍只有completed/total/detail value，没有unit/stage/time/estimated/heartbeat/monotonic contract。 |
| P1-71 | Partial | operation request/result已有retained/output byte与item限制；输出仍是任意JSON，没有schema、descriptor revision、build/source/attempt/digest receipt。 |
| P1-72 | Partial | Host已有typed `RuntimeForeignOutputMetricsSnapshot`和per-kind counters，`diagnostic_line()`已是presentation；snapshot仍未绑定session/build，也没有versioned transport/event aggregation和decode non-preemption说明。 |

## 6. P2 旧条目当前重判

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P2-01 | Open | legacy/v1/v2/current/template命名仍不能表达read-only、authoring、compiled、runtime与sunset。 |
| P2-02 | Open | `UiTreeId(pub String)`、`UiNodePath(pub String)`仍公开内部表示。 |
| P2-03 | Open | root/child mutation error策略仍不一致。 |
| P2-04 | Open | accessibility `snapshot.node()`仍线性扫描`Vec`。 |
| P2-05 | Open | dynamic globalization仍把surface index写入path并依赖容器顺序。 |
| P2-06 | Open | debug/report仍有大量含义模糊的bool bundle。 |
| P2-07 | Open | `UiValueType::Any`继续作为公开远程属性类型。 |
| P2-08 | Open | debug schema version仍位于capture context，不是顶层可协商envelope。 |
| P2-09 | Open | closed enum Default仍常选择Declared/Mouse/Lines/ReplaceActive等业务状态。 |
| P2-10 | Partial | UI已拆到248个文件，多个旧大文件被模块化；contract owner与稳定profile仍未据此收敛，颗粒仍不等于边界。 |
| P2-11 | Open | public注释仍使用snapshot/generation/stable等词，但跨版本/一致性资格不足。 |
| P2-12 | Partial | 当前测试已新增cycle/multi-parent、producer budget、transaction rollback、domain generation、secure redaction和operation race；仍有大量source-contains/shape/same-version serde测试，真实产品不变量覆盖不足。 |

## 7. 当前关键断链

### 7.1 Authoring authority仍是有损投影

当前Editor把v2 source保存权交给legacy `UiAssetDocument`编辑模型。`previous`参数不是lossless syntax owner，只能从旧semantic DTO按node/component key拣回少数字段；删除/重命名、unknown field、comment、formatting、orphan node以及未显式保留的v2字段仍会消失。目标必须是：lossless TOML CST拥有source bytes与unknown syntax，canonical semantic IR拥有typed schema，legacy UI只提交field transaction，不能重建全文。

### 7.2 Producer budget只覆盖了accessibility的后半程

accessibility snapshot/extract/encode已经有一致limit，这是应保留的实现；但read/parse/validate/compile/package仍先物化无界String/TOML/recursive DTO。`UiZuiAssetLoader`和asset loader还复制同一profile validator。共享budget必须从文件读取开始累计bytes/nodes/depth/strings/maps/diagnostics/work units，并在每阶段携带deadline/cancel/usage receipt，不能在每层重新获得满额预算。

### 7.3 Accessibility身份与Host消费者同时缺失

即使producer完全有界，当前action仍无法证明目标来自caller观察的snapshot。目标协议需要 `{session_epoch, tree_id, snapshot_generation, stable_node_key, instance_generation}`，并由平台bridge保存最近accepted generation。capture必须支持NotModified/page/delta/remove/focus；action执行必须compare-to-snapshot，返回Accepted/Unsupported/StaleTarget/Denied/Failed typed result。App/Editor需要真实AccessKit/platform consumer，Host registry必须把accessibility纳入同一budget/fuse/metrics owner。

### 7.4 Reflection/control是无版本的进程内对象模型

`UiEventManager`的direct query优化和owned fanout优化不改变协议语义。unbounded subscription、path+JSON mutation、空Ack和无revision diff意味着丢包、并发writer、tree替换和权限边界都不可证明。应建立generation-bound immutable reflection snapshot、sequence delta、bounded subscription lease和typed mutation receipt；Editor08 principal/capability gateway只接收该v2协议。

### 7.5 Input transaction没有跨Host transaction

Runtime能回滚内存effect，但Host request一旦输出，App仍可能不执行、部分执行或只记录日志；Runtime没有ack回流，也不能据此修正focus/capture/popup/link状态。需要Host side-effect transaction ID、request ordinal、gesture/principal/capability、accepted/denied/failed/too-late receipt，以及Runtime reconcile/compensation规则。安全敏感payload必须按privacy class输出，不能出现在通用Debug。

### 7.6 Operation ABI落后于内部service

内部cancel/deadline/harvest基础应保留，但public ABI必须增加typed descriptor registry、cancel request/ack、所有terminal outcome、qualified handle和schema-bound result。否则跨DLL caller只能轮询一个内部已能取消、但外部无法控制也无法收获Cancelled/Expired原因的状态机。

## 8. 目标合同

| profile / owner | 权威内容 | 禁止继续暴露 |
|---|---|---|
| `UiAuthoringSourceVn` / Authoring service | lossless CST、source revision、semantic IR、stable element IDs、migration/loss receipt | Editor通过legacy DTO重写全文 |
| `UiCompiledPackageVn` / Compiler service | deterministic sections、dependency closure、schema/build/source digests、bounded reader | Rust serde/TOML struct作为未发布wire |
| `UiRuntimeTree` / Runtime UI | validated immutable topology + controlled mutation、qualified live handles、domain generations | 普通Deserialize直接恢复live tree |
| `UiReflectionProtocolVn` / Runtime+Editor gateway | snapshot token、page/delta/sequence、typed schema、CAS mutation、bounded lease | path+JSON、Option not-found、空Ack、unbounded channel |
| `UiAccessibilityProtocolVn` / Runtime+Platform bridge | generation-qualified tree/action、page/delta、platform capability matrix、typed result | surface index bit packing、ignored generation hint |
| `UiInputOutcomeVn` / Runtime UI | bounded route trace、atomic local effects、privacy-aware receipt | 完整原事件和无界notes作为默认结果 |
| `UiHostSideEffectVn` / App/Host | capability/gesture admission、execution receipt、reconcile | 只记unhandled日志或假定intent已生效 |
| `UiDiagnosticEnvelopeVn` / Diagnostic core | code/severity/span/build/source/correlation/privacy/budget/cause | 50套互不兼容family与raw strings |
| `RuntimeOperationVn` / Operation registry | descriptor/schema、qualified handle、submit/poll/cancel/harvest、全终态receipt | String+JSON无schema与不可取消ABI |

## 9. 重构顺序

### M0 · source与写入止损

- 先关闭Interface10 clean-checkout source reproducibility阻断，并生成UI public owner/profile manifest。
- 对包含v2-only、unknown、comment、orphan的文件禁止legacy save；输出blocking loss report与recoverable backup。
- Gate：打开-不修改-保存对完整corpus byte或semantic-lossless；任何未声明loss阻断publish。

### M1 · bounded authoring与唯一compiler authority

- 引入共享`UiAuthoringAdmissionBudget`，覆盖read/parse/validate/migrate/compile/package。
- 合并两份v2 loader/profile validator；legacy/v1/v2只作为bounded adapter进入canonical IR。
- Gate：100 MiB、百万node、10,000层、超长string/map在固定RSS/CPU/stack/deadline内fail-close并返回usage receipt。

### M2 · package与runtime tree边界

- 定义language-neutral compiled package section与dependency closure；artifact publish绑定source revision、BuildSet和expected cache generation。
- Runtime只能从validated package构造tree；live tree serde硬切为snapshot+validator。
- Gate：deterministic bytes/digest、unknown section矩阵、cycle/orphan/multi-parent/slot、stale publish与restore corpus通过。

### M3 · accessibility与reflection v2

- 先发布qualified snapshot/action identity，再建立App/Editor platform bridge与Host output registry。
- 合并reflection schema，增加page/delta/base-new revision/sequence/overflow/resync与bounded lease。
- Gate：surface reorder、node remove/recreate、Runtime restart、丢包/乱序/重复、subscriber overflow全部返回stale/resync，绝不静默继续。

### M4 · input/Host transaction

- 统一pointer/navigation/text/accessibility outcome，默认关闭或采样高成本trace。
- Host集中执行focus/capture/pointer lock/popup/tooltip/link/clipboard/IME并返回typed receipt。
- Gate：multi-window/multi-seat/device reconnect/gesture/capability/partial failure与compensation矩阵通过。

### M5 · diagnostic与operation contract

- 将50套family映射到共享envelope/code registry；collector加入bytes/count/per-code/dedup/suppression/privacy。
- 注册typed operation descriptor/input/output schema，扩展ABI cancel和全terminal harvest result。
- Gate：百万重复错误有界；operation cancel/too-late/apply race、Cancelled/Expired harvest、schema/build/source receipt通过。

### M6 · 发布资格

- 在clean checkout运行old/new reader-writer、真实Runtime DLL、App/Editor platform bridge、Windows/Linux、cross-language、fuzz/fault/scale/soak和性能矩阵。
- 每项结果绑定source fingerprint、BuildSet、command、duration、exit code与artifact；只编译到test binary不得标Pass。

## 10. 32项产品资格门当前状态

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01-G03 | Fail | v2-only/unknown/comment仍不lossless；无migration loss graph；authoring大输入/深度无producer admission。 |
| G04-G05 | Partial | accessibility有deadline/usage限制，compiled cache有fingerprint/deterministic起点；未覆盖全authoring与独立wire。 |
| G06-G07 | Fail | unknown package section协议不存在；live tree仍可普通serde恢复。 |
| G08 | Partial | cycle/multi-parent已拒绝；orphan、stable code、source span未闭合。 |
| G09-G10 | Fail | stale action与local-ID/surface projection未修复。 |
| G11 | Partial | snapshot总量有界；无page/delta/truncated/completeness。 |
| G12 | Fail | App/Editor真实accessibility platform bridge不存在。 |
| G13 | Partial | AccessKit/text range有局部转换校验；无Windows/Linux bridge golden matrix。 |
| G14-G18 | Fail | reflection无sequence/resync/CAS/principal/receipt/bounded subscription。 |
| G19-G21 | Partial | metadata、部分Host执行、transaction/redaction存在；multi-seat/reconnect、完整receipt和10K hot-path资格缺失。 |
| G22-G24 | Fail | 50套diagnostic未统一span/code/budget；百万错误仍无全局suppression contract。 |
| G25 | Partial | secure text result和App日志有局部redaction；统一privacy export policy缺失。 |
| G26-G27 | Fail | public ABI不能cancel；Cancelled/Expired不能harvest为typed result。 |
| G28-G31 | Partial | operation/foreign output有producer/consumer budget、typed metrics和fault基础；schema receipt、accessibility kind、session/build聚合、完整恢复矩阵缺失。 |
| G32 | Fail | 本轮只有静态review，没有绑定BuildSet的动态qualification artifact。 |

合计：**19 Fail / 13 Partial / 0 Pass**。

## 11. 验证说明

本轮没有运行Cargo，因为目标是当前源码公共合同review，且工作树包含大量共享未提交/未跟踪实现；重复编译不能证明这些公共协议已达到产品资格。完成的验证仅包括：全量Interface UI物理扫描、focused生产消费者逐符号追踪、公开/serde/usize/TOML/diagnostic family计数、dirty/untracked计数、三组内容指纹、五套本地参考源码核对，以及报告编号/状态/链接/索引的静态检查。

任何实现阶段都必须重新取指纹，并补充M0-M6对应动态证据。当前`Partial`只表示代码中存在可复用底座，不表示用户数据安全、ABI兼容、性能或产品表现已经优于Unreal。

## 12. 审查决策

`zircon_runtime_interface` 的UI公共面当前不能被标记为稳定工程合同。允许保留v2迭代graph validator、compiled fingerprint/cache、tree dirty/incarnation、surface domain generation、accessibility producer budget、input effect transaction、operation service与Host metrics；禁止继续扩大三套authoring写入模型、无世代accessibility action、path+JSON remote mutation、unbounded subscription、无回执Host side effect和String+JSON operation ABI。

下一实施切片只能从M0止损和M1 bounded canonical authoring入口开始；在lossless save与producer admission完成前，不应继续增加新的`.zui`字段或让legacy projection获得更多写权限。

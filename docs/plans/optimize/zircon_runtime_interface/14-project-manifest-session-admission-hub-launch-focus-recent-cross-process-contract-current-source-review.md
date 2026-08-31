# `zircon_runtime_interface` Project Manifest / Session Admission / Hub Launch / Focus / Recent 跨进程合同当前源码复核

> report_id: `Interface14`  
> kind: `current-source-review`  
> canonical_source: [06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md](06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md)  
> canonical_product_owner: [../zircon_editor/172-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md](../zircon_editor/172-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md)  
> review_scope: manifest partial probe / identity / migration / engine compatibility，ProjectLaunchIntent，session admission record / OS lease / heartbeat，Hub child launch / Ready mailbox，generation-qualified focus request / ack，以及 Hub/Editor shared recent-project projection。  
> baseline: 当前工作树 `f2df7ed2100a771881a3b7222b726789b0b40abd`（2026-08-29）；未提交与未跟踪源码纳入观察指纹，但不视为该提交已经交付的能力。  
> interface_contract_fingerprint: `09996d220be23ad5bbf3891f55de7234c0b746e8a8749f8b101e7b5f0b0cca4a`  
> focused_consumer_fingerprint: `eedd31d6ff37b3b342130cb76e6df0e14d8ca26cb59c08cf72c39e15d0b85174`  
> deduplicated_current_source_fingerprint: `fa65e90dcaeb865f3382d24731e4164066f304bd3a139e8f05557c9d0a879fc6`  
> reference_fingerprint: `10e80eb62d797b0090c412edb50992c443c2bf2bf9d108e6c6b02bc3198e2923`  
> status: review-only；不修改 production Rust，不运行 Cargo、Hub、Editor、真实双进程、kill/crash、跨平台、ACL、fuzz、scale、soak 或动态 benchmark。Tooling/Rust 迁移按用户要求排除；未查询、轮询、等待或实时跟踪协调器。

## 1. 结论

Interface06 的总体架构判断仍成立，但当前工作树已经补入一批不能忽略的工程底座：Editor preflight 有 4 MiB manifest 上限、canonical descriptor、ProjectGuid、BLAKE3 manifest digest、`ProjectIdentity` 和 admission 前 revalidation；`ProjectLaunchIntent` 有 schema version、operation ID、source、Normal/Safe/Recovery profile 与 validated deserialize；session record 已携带 BuildSet、operation、lifecycle、checked epoch、generation 和 heartbeat，OS lease 仍是独立权威；生产 retained-host tick 确实驱动 heartbeat；Hub 只把 OS lease 仍存续且 record 为 `Ready` 的实例判为可聚焦；focus 已改为按 instance/generation/sequence/request ID 排队，有 deadline、4 KiB输入、32项队列、claim、typed ack，并只在原生窗口真正获得焦点后报告 `Focused`；Ready mailbox 等到 focus watcher 绑定与 first present；recent store 有 256 KiB上限、revision/CAS、logical clock、tombstone、deadline/cancel、quarantine/rebuild、file sync、Unix directory sync和Windows write-through replace，且 recent failure 已从 project activation commit gate 移到 post-Ready projection。

这些进展仍没有组成一个工程级跨进程事务：

1. Hub 仍把 summary 可解析命名为 `ProjectValidation::Valid`；summary 本身字段公开并直接 `Deserialize`，只返回显示字段，不声明 deferred/ignored sections、reader policy或migration steps。Editor有带digest的preflight receipt，Hub summary probe却仍可无界读取完整文件。
2. `ProjectLaunchIntent`、session admission record、Ready mailbox、focus request、recent entry各自拥有一部分身份，却没有一个共同 envelope 同时绑定 operation、ProjectIdentity、manifest digest、BuildSet、process creation identity、admission epoch、session generation与deadline。
3. session record 先写成 `Ready`，随后才提交 activation ledger 的 `Session` effect；Hub probe在这个窗口已可把进程当作focus target。若ledger commit失败，Editor再尝试写`RecoveryRequired`，但此前的跨进程观察没有共同commit receipt。
4. `HubEditorReadyReceiptV1::after_first_present()`一次性构造固定五项全集；milestone是标签集合，不是逐阶段签名/摘要。它不携带operation、project、manifest、BuildSet、request hash或deadline，Hub也只验证child PID。
5. focus进展真实，但`HubEditorFocusSignalV1`和ack仍是public字段直接Deserialize；Hub publisher会自行`create_dir_all`，清理时先无界读取请求，目录计数忽略部分I/O错误，malformed/mismatch claim被永久留在private filename，launch mailbox仍无byte limit、claim、ack、删除或retention。
6. recent并发与durability明显改善，但`HubRecentProjectV1`仍嵌套可绕过parser的public summary和`PathBuf`，身份仍是lossy lexical path；存储路径无product/profile/BuildSet namespace，quarantine文件无retention。更关键的是filesystem lock、repair、quarantine、atomic replace和环境变量storage root全部进入了`zircon_runtime_interface`，违反schema/codec-only边界。
7. 版本仍是mailbox/focus/recent共享的exact V1，没有独立schema family、capability negotiation、support window、golden corpus或N/N-1真实binary矩阵。ACL/reparse/symlink、process creation epoch、monotonic heartbeat observation、统一telemetry和crash-point资格仍为空白。

本轮对 Interface06 的 56 个 P1 与 14 个 P2 逐项重判：P1 为 **20 Open / 25 Partial / 11 Closed**，P2 为 **8 Open / 6 Partial / 0 Closed**。36项产品资格门为 **14 Fail / 14 Partial / 8 Pass**。本轮没有新增唯一P0/P1/P2；报告数增加1，canonical finding总数不重复增加。Editor172/Editor51继续唯一拥有项目启动、activation与close产品状态机的五项P0；本文只刷新Interface06组合合同状态。

## 2. 审查边界与证据

### 2.1 物理范围

| 选择集 | files / lines / bytes / test attrs / ignored | dirty 状态 | fingerprint |
|---|---:|---:|---|
| Interface 直接合同 | **79 / 5,892 / 202,953 / 47 / 1** | **19 tracked modified / 47 untracked** | `09996d220be23ad5bbf3891f55de7234c0b746e8a8749f8b101e7b5f0b0cca4a` |
| focused App/Runtime/Editor/Hub 消费者 | **108 / 16,906 / 609,490 / 191 / 4** | **49 tracked modified / 37 untracked** | `eedd31d6ff37b3b342130cb76e6df0e14d8ca26cb59c08cf72c39e15d0b85174` |
| 去重当前源码 | **187 / 22,798 / 812,443 / 238 / 5** | **68 tracked modified / 84 untracked** | `fa65e90dcaeb865f3382d24731e4164066f304bd3a139e8f05557c9d0a879fc6` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics 参考 | **18 / 12,678 / 447,122** | n/a | `10e80eb62d797b0090c412edb50992c443c2bf2bf9d108e6c6b02bc3198e2923` |

Interface选择包含`hub_protocol`全量、manifest summary、session lock、activation operation ID、engine compatibility、canonical descriptor、manifest digest、ProjectGuid/ProjectIdentity、ProjectLaunchIntent和Runtime BuildSet合同。focused选择沿App CLI launch args、Runtime full manifest、Editor project authority/preflight/session guard/activation ledger/Hub link/startup/retained app，以及Hub editor focus/handshake/launch/projects/runtime action全链。

指纹算法为：规范化相对路径排序，逐文件SHA-256，再以`path<TAB>sha256`和LF拼接后取SHA-256；未跟踪文件纳入当前观察指纹。它能证明本文审查的工作树内容，不能证明clean checkout、编译或产品行为。当前`HEAD`与此前复核基线不同，后续实施必须重新取指纹。

### 2.2 纵向调用链

本轮逐符号追踪了以下路径：

1. Hub recent row -> summary `Valid` probe -> source engine选择 -> `ProjectLaunchIntent` JSON CLI参数 -> child spawn ->固定10秒mailbox poll -> PID-bound Ready；
2. App/Editor launch intent -> canonical root -> bounded full preflight -> ProjectIdentity/digest/engine compatibility/migration decision -> session admission/OS lease -> activation ledger -> session Ready -> focus binding -> first-present mailbox；
3. 第二次Hub open -> OS lease probe -> admission lifecycle classify -> generation-qualified focus publish -> Editor claim/validate -> native window attention -> `Focused/Rejected*` ack -> Hub成功状态；
4. Editor close -> session `Closing` -> runtime/plugin/document teardown -> lock removal，失败时转`RecoveryRequired`并保留lease；
5. Hub/Editor recent projection -> shared Interface store lease -> bounded read -> revision/CAS/tombstone merge -> quarantine/rebuild -> synced atomic replace。

### 2.3 当前工作树与结构审计边界

选择集中有84个未跟踪文件，说明多个关键类型尚未形成可从当前提交重建的交付面。本文不回退或修改这些源码，也不把source presence写成发布完成。此前对runtime interface convergence结构审计脚本的30秒与60秒有界尝试都超时，没有结果；本轮不再等待该阻塞，而以定点生产调用链继续review。没有结构审计结果时，本文不宣称Interface/Editor/Hub entry/manager/driver/plugin边界已经收敛。

### 2.4 本地参考源码

| 引擎 | 本轮适用事实 | 不外推 |
|---|---|---|
| Unreal | ProjectDescriptor显式file version、engine association、modules/plugins/target platforms；ProjectManager分离load与status；Project Browser提供Open a Copy/Convert in-place/Skip/Cancel；SingleInstanceMutex有取消等待和abandoned处理。 | 不复制大型类层次，也不把Unreal UI分支直接当Zircon wire schema。 |
| Godot | project list区分missing、different/unknown version与unsupported feature；Project Manager有scan、清理missing和recovery入口。 | 不把Godot单进程project manager当跨DLL/跨进程协议。 |
| Bevy | plugin生命周期分成Adding、Ready、Finished、Cleaned，Ready由所有plugin共同满足。 | 只用于组合式milestone语义，不替代project admission。 |
| Fyrox | manager持有child/command queue，项目界面暴露engine version和upgrade入口。 | 其较轻recent/launch实现不是Zircon工程上限。 |
| Unity Graphics | versionable/migratable asset有显式version，HDRP asset/plugin material各自维护migration version。 | 本地Graphics镜像不含Unity Hub跨进程实现，launch/focus/recent为N/A。 |

这些参考共同支持“可解析、可显示、可迁移、可在当前BuildSet打开、已激活、已首帧、可交互”必须是不同事实；它们不能证明Zircon当前性能、稳定性或表现达到或超过Unreal。

## 3. 当前可保留底座

1. 保留4 MiB Editor preflight read cap、manifest digest、canonical descriptor、ProjectGuid、ProjectIdentity和admission前digest revalidation。
2. 保留directional engine compatibility与Normal/Safe/Recovery composition profile，但不能把semver compatible升级为完整admission。
3. 保留ProjectLaunchIntent的private validated value、operation ID与source/profile；path wire后续替换为qualified locator。
4. 保留OS lease与record分离、lifecycle state、checked epoch、BuildSet、operation、session generation和production heartbeat owner。
5. 保留Hub对Pending/Ready/RecoveryRequired的分类；只有OS lease与Ready同时成立才允许focus。
6. 保留first-present之后发布Ready、PID绑定、path-redacted categorical failure；milestone必须改为逐阶段evidence而不是固定全集。
7. 保留focus per-request path、generation/sequence/deadline、4 KiB cap、claim、native-owner ack、bounded pending queue和retire stale rejection。
8. 保留recent的revision/CAS、logical clock、tombstone、bounded lease、corruption disposition、quarantine/rebuild和durable replacement机制，但将store迁出Interface。
9. 保留recent作为post-Ready非阻断投影；任何写回错误只产生deferred diagnostic，不回滚已提交project session。
10. 保留typed error/disposition方向，删除剩余依赖自由字符串恢复状态的调用链。

## 4. Canonical P0 路由

Interface14不新增P0，也不重复计数以下Editor canonical阻断：

| Canonical finding | 当前状态 | Interface14只负责 |
|---|---|---|
| `E-PROJ-P0-01` admission/compatibility批准前不得加载project-derived code | Open | ProjectIdentity、compatibility与request/receipt schema。 |
| `E-PROJ-P0-02` activation rollback/close失败不得静默释放guard | Open | terminal lifecycle与recovery disposition。 |
| `E-PROJ-P0-03` Claimed/Activating不得被解释为Ready | Open | phase/generation schema；Hub当前分类进展保留。 |
| `E-PROJ-P0-04` focus必须随session rebind并以owner ack完成 | Open | request/ack schema；当前focus链为部分底座。 |
| `E-PROJ-P0-05` Engine/BuildSet/migration/Safe/Recovery必须在产品层决策 | Open | typed compatibility/admission receipt，不拥有产品UI。 |

当前新增的“session record先Ready、ledger Session后commit”窗口由Editor172/Editor51继续canonical拥有；本文不得再创建一个Interface P0副本。

## 5. P1 旧条目当前重判

状态语义：`Closed`表示当前源码消除了旧命题；`Partial`表示出现可迁移底座，但原合同仍不成立；`Open`表示旧命题仍直接成立。

### 5.1 Manifest partial probe、identity 与 migration

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-001 | Open | `ProjectManifestSummary`仍public fields + direct `Deserialize`；recent/fixtures可绕过trim、semver、asset-root与migration parser invariants。 |
| RI-PROJ-P1-002 | Open | Hub `ProjectValidation::Valid`仍只调用summary parser；Runtime full manifest、plugin/script/settings/export与BuildSet未验证。 |
| RI-PROJ-P1-003 | Open | SummaryDocument虽读取asset roots/settings/library version，却在`into_summary`丢弃，receipt不声明validated/deferred/ignored/unsupported sections。 |
| RI-PROJ-P1-004 | Partial | Editor preflight已有canonical descriptor与manifest digest；通用summary结果仍无source path/size/reader policy/time，Hub也不绑定同一receipt。 |
| RI-PROJ-P1-005 | Partial | Editor已有migration decision/plan/action类型；Interface `Loaded`仍只暴露`migrated_from`，没有step、lossy、backup和writeback receipt。 |
| RI-PROJ-P1-006 | Partial | Editor preflight在读取前执行4 MiB cap；Hub summary probe和public string/bytes parser仍全量读/parse，无depth/item/string/path预算。 |
| RI-PROJ-P1-007 | Partial | `ProjectIdentity`已组合canonical descriptor、nonnil ProjectGuid与manifest digest，并进入Editor preflight；session/Ready/focus/recent未共同引用它。 |

### 5.2 Version、capability 与 validated wire

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-008 | Open | mailbox、focus、recent继续共享`HubProtocolVersionV1`与`HUB_PROTOCOL_VERSION_V1=1`。 |
| RI-PROJ-P1-009 | Open | Hub CLI、mailbox marker和ProjectLaunchIntent只接受exact version，没有reader/writer range、preserve/ignore policy或deprecation window。 |
| RI-PROJ-P1-010 | Open | Launch没有required/optional capabilities，Hub无法预知first-present receipt、safe mode、focus ack或schema支持。 |
| RI-PROJ-P1-011 | Open | 测试仍是同source tree roundtrip/shape；没有schema fingerprint、canonical corpus或old/new binary reader/writer matrix。 |
| RI-PROJ-P1-012 | Partial | Ready receipt、ProjectLaunchIntent、ProjectGuid/digest等使用private/custom validated deserialize；summary、focus signal/ack、mailbox、recent registry仍可先构造无效值，session record也未拒绝PID 0与heartbeat 0。 |
| RI-PROJ-P1-013 | Open | ProjectLaunchTarget、CanonicalDescriptorIdentity、Ready path scope和recent entry仍直接使用`PathBuf`或本机path语义进入wire/storage identity。 |
| RI-PROJ-P1-014 | Partial | Engine compatibility已有Compatible/Newer/Older/Incompatible typed direction；Hub schema/BuildSet/capability incompatibility仍是exact reject或自由字符串。 |

### 5.3 Session admission、liveness 与 terminal lifecycle

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-015 | Open | Hub先短暂打开/关闭OS lease handle，再单独读record；session可在两步之间退出或换代，仍没有lease-bound snapshot/epoch observation。 |
| RI-PROJ-P1-016 | Closed | Hub现在把live record分类为Pending/Ready/RecoveryRequired，只有`Ready + generation`可focus；旧的裸`Active`结论已删除。 |
| RI-PROJ-P1-017 | Partial | record有checked epoch和session generation；新claim仍从epoch 1开始，没有随机admission epoch或process creation identity，ABA/PID reuse未闭合。 |
| RI-PROJ-P1-018 | Closed | EditorManager持有10秒节奏的production heartbeat状态，retained-host tick和wake deadline真实驱动refresh，degraded后停止延长。 |
| RI-PROJ-P1-019 | Open | persisted heartbeat仍只有wall-clock millis，没有producer clock ID、monotonic sequence、observation time、skew/suspend/late disposition。 |
| RI-PROJ-P1-020 | Partial | V2 key=value codec严格拒绝duplicate/unknown并验证BuildSet/operation/lifecycle；仍接受无界`&str`，无byte cap、checksum、canonical schema或bounded decoder。 |
| RI-PROJ-P1-021 | Partial | Closing与RecoveryRequired已有持久phase，release错误会保留recovery；正常close删除record，没有Closed/cleanup receipt、retention或外部ack。 |

### 5.4 Hub launch、Ready 与 mailbox

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-022 | Partial | ProjectLaunchIntent已有schema/operation/source/profile/target；Hub session/protocol仍是旁路CLI参数，Ready未绑定operation/ProjectIdentity/BuildSet/request hash。 |
| RI-PROJ-P1-023 | Partial | Ready receipt列出五个milestone并等到first present；`after_first_present()`直接制造固定全集，未分别证明preflight/admission/activation/window/interactive，也没有逐阶段receipt。 |
| RI-PROJ-P1-024 | Open | Outcome仍只有Ready/Failed；无Pending、Progress、RetryAfter、cancel accepted、revoked或Closed结果。 |
| RI-PROJ-P1-025 | Open | Hub固定250 ms poll和10秒timeout；request没有caller deadline/cancel，timeout后也不监督/终止/查询child terminal状态。 |
| RI-PROJ-P1-026 | Partial | focus有claim/ack；launch mailbox仍直接read，不claim、不ack、不remove，duplicate/stale mailbox和reader crash没有确定状态。 |
| RI-PROJ-P1-027 | Partial | focus request/ack各限制4 KiB；launch mailbox仍无byte cap，focus过期清理会在cap前读取整文件。 |
| RI-PROJ-P1-028 | Partial | focus path helper已共享；Hub与Editor仍各自实现publisher/atomic write，handshake path/read/write也分散在两端，语义可继续漂移。 |

### 5.5 Focus request / ack

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-029 | Closed | 请求路径包含target instance、sequence与UUID request ID；并发请求不再覆盖单一固定mailbox。 |
| RI-PROJ-P1-030 | Closed | typed ack已有Focused/Expired/TargetMismatch/InboxFull/Stale，Hub等待精确request-bound ack后才成功。 |
| RI-PROJ-P1-031 | Closed | watcher把完整request传入bounded acknowledgement bridge，直到native `Focused(true)`才回写对应ack；旧token丢弃路径已消失。 |
| RI-PROJ-P1-032 | Open | malformed、oversize或target-mismatch请求在rename claim后返回错误，private claim永久保留；没有bounded quarantine/repair/cleanup owner。 |
| RI-PROJ-P1-033 | Closed | request有非零deadline、10秒TTL、expired rejection和generation验证；旧请求不会被无条件消费为Focused。 |
| RI-PROJ-P1-034 | Open | Hub publisher的atomic writer仍`create_dir_all`目标inbox；publisher能制造看似已绑定的namespace，缺owner-created binding receipt/ACL验证。 |
| RI-PROJ-P1-035 | Partial | UUID request ID、sequence、generation与ack match提供单次幂等基础；没有durable consumed set、duplicate disposition、Hub restart sequence epoch或replay history。 |

### 5.6 Recent projection 与 conflict

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-036 | Open | `HubRecentProjectV1`仍嵌套public serde summary；registry validate只检查path非空，无法证明summary来自manifest parser。 |
| RI-PROJ-P1-037 | Partial | revision、CAS和logical timestamp已存在；same-key/same-timestamp merge仍以manifest name择胜，缺writer ID与explicit conflict receipt。 |
| RI-PROJ-P1-038 | Closed | remove先写bounded tombstone，record推进logical clock并清tombstone，CAS重放Hub delta；旧writer静默复活与无tombstone截断命题已消除。 |
| RI-PROJ-P1-039 | Closed | Windows/Unix writer lease均支持deadline、nonblocking和cancellation；Editor post-Ready使用try-now，Hub background最多等待250 ms。 |
| RI-PROJ-P1-040 | Closed | Hub与Editor都调用同一个`HubRecentProjectsStore` transaction contract，不再各自维护第二套load/lock/write实现。 |
| RI-PROJ-P1-041 | Closed | corrupt/oversize load降级为空projection，mutation在同一lease下quarantine后rebuild；不再直接阻断Hub/Editor启动。仍缺last-known-good，但旧hard-fail命题已关闭。 |
| RI-PROJ-P1-042 | Closed | recent写回在session/ledger commit后执行，函数返回已提交value；失败只记录Deferred diagnostic，不回滚project activation。 |

### 5.7 Owner、storage、durability 与 lineage

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-043 | Partial | Editor `ProjectPaths`和preflight canonical descriptor建立adapter起点；wire/recent/mutex key仍混用raw/lossy path，ProjectIdentity未贯穿。 |
| RI-PROJ-P1-044 | Partial | intent有profile/source，session record有principal/BuildSet；shared recent root和project `.zircon/hub`仍未按product/channel/profile/BuildSet/test instance隔离。 |
| RI-PROJ-P1-045 | Open | Interface现在直接拥有filesystem I/O、OS mutex/flock、deadline policy、quarantine、atomic replace、HOME/USERPROFILE storage root；schema/codec crate吸收业务store更严重。 |
| RI-PROJ-P1-046 | Partial | recent writer执行file sync、Unix parent sync和Windows write-through replace，session guard另有durability枚举；handshake/focus receipt仍不声明Published/DurableLocal等级，Windows首次rename与网络FS保证未定义。 |
| RI-PROJ-P1-047 | Partial | recent和focus已实现部分write/claim/ack/rebuild步骤；三种协议没有共同crash-point taxonomy、visible state、repair receipt或fault matrix。 |
| RI-PROJ-P1-048 | Partial | recent entries/tombstones、focus pending queue与request TTL有cap；handshake、ack、private claims、quarantine/temp文件仍可累积且无namespace quota owner。 |
| RI-PROJ-P1-049 | Partial | LaunchOperationId贯穿intent、session record和activation ledger；Ready/focus/recent/Hub action history未全部引用同一operation，审计链仍断裂。 |

### 5.8 Security、observability 与 qualification

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P1-050 | Open | mailbox/recent目录没有owner-only permission contract、symlink/reparse拒绝、parent identity pinning或cross-user threat model。 |
| RI-PROJ-P1-051 | Partial | Ready receipt移除project path，failure改为categorical code；多数Hub/Editor errors仍打印完整path/instance，session token与project name没有统一sensitivity metadata。 |
| RI-PROJ-P1-052 | Partial | operation ID、Hub session token、focus request ID与session generation分别存在；没有统一correlation envelope贯穿日志、timeout、recent conflict和cleanup。 |
| RI-PROJ-P1-053 | Open | 没有startup phase latency、timeout cause、focus ack latency、stale mailbox、recent repair/lease wait和session ABA的统一metrics/census。 |
| RI-PROJ-P1-054 | Open | 测试以constructor/shape/filesystem happy path为主；没有双Hub/双Editor、kill/PID reuse、reader/writer crash、suspend/resume和replay动态矩阵。 |
| RI-PROJ-P1-055 | Partial | Editor manifest、focus和recent已有部分bytes/item/deadline预算；session codec/mailbox/deep JSON/path仍不完整，也没有fuzz/property/OOM隔离。 |
| RI-PROJ-P1-056 | Open | 没有N/N-1 Hub/Editor真实binary artifact、support window、upgrade/downgrade harness或旧reader/writer corpus。 |

## 6. P2 旧条目当前重判

| ID | 状态 | 当前复核结论 |
|---|---|---|
| RI-PROJ-P2-001 | Open | V1仍散落在type、constant、filename和wire version命名中，没有统一schema catalog。 |
| RI-PROJ-P2-002 | Partial | focus/recent path helper已集中；handshake mailbox、session lock和staging/quarantine path policy仍分散。 |
| RI-PROJ-P2-003 | Open | instance ID继续以`String`跨record/focus/Ready流动，validator还存在不同grammar/length约束。 |
| RI-PROJ-P2-004 | Open | HubError、session record error和store diagnostics大量依赖自由文本，调用方无法稳定分类/本地化。 |
| RI-PROJ-P2-005 | Open | key=value session codec的字段顺序与canonical encoding仍由手写`format!`隐式决定。 |
| RI-PROJ-P2-006 | Partial | recent 8条与64 tombstone已命名并有“rebuildable projection”注释；容量仍不是product/profile policy或可观测quota。 |
| RI-PROJ-P2-007 | Open | Runtime atomic writer、Hub focus writer、recent UUID temp、focus claim和corrupt quarantine继续使用多套命名/cleanup格式。 |
| RI-PROJ-P2-008 | Partial | Windows/Unix lease实现有同域单测和typed policy；尚无统一OS adapter conformance、network FS/reparse/permission矩阵。 |
| RI-PROJ-P2-009 | Partial | ProjectIdentity/Ready/intent等已区分validated value；summary/focus/ack/mailbox/recent仍把wire DTO与domain value混在public re-export。 |
| RI-PROJ-P2-010 | Partial | 新类型已有较多单位与identity注释；public path/time/sequence/capacity/sensitivity范围仍不统一。 |
| RI-PROJ-P2-011 | Open | 测试继续重复手写JSON、summary、temporary root与session fixtures，没有versioned corpus manifest/builder。 |
| RI-PROJ-P2-012 | Partial | startup failure与focus ack已有稳定枚举；manifest/session/store/Hub action仍主要输出自由文本，未形成统一code catalog。 |
| RI-PROJ-P2-013 | Open | `hub_recent_projects_path()`直接读HOME/USERPROFILE，缺失时回退相对cwd `.zircon`，没有typed storage unavailable。 |
| RI-PROJ-P2-014 | Open | crate注释宣称DTO层不做I/O，但recent store实际拥有完整业务存储；没有architecture test固定Interface非业务owner原则。 |

## 7. 当前关键断链

### 7.1 Summary `Valid` 与 Editor preflight 是两套事实

Hub直接`fs::read`后调用summary parser并返回`Valid`；Editor则canonicalize root、限定4 MiB、计算digest、读取full manifest、生成composition/migration/ProjectIdentity并在lease内revalidate。Hub在spawn前仍不能说明它基于哪一个descriptor hash，也不能说明完整manifest或BuildSet可打开。目标必须把Hub状态改为`PartialProbe`，只允许Editor/Runtime preflight receipt发布Open/Migrate/Safe/Reject结论。

### 7.2 Admission Ready 与 activation ledger commit 非原子

`guard.commit_ready()`先持久化`Ready + generation`，`ledger.commit(Session)`随后执行。Hub probe只看OS lease与record，因此可在两步之间发布focus；若ledger commit失败，Editor只能补写RecoveryRequired。正确合同应由一个commit coordinator生成包含operation、ProjectIdentity、BuildSet、ledger digest与session generation的ReadyCommit receipt，外部消费者只观察已完成commit marker。

### 7.3 Ready milestone 是固定声明，不是证据累积

`after_first_present()`直接插入五个required enum。当前调用位置确实晚于UI创建、focus binding和first-present callback，但DTO不能证明每项事实由谁、何时、基于哪个operation产生。milestone应成为带producer、sequence、time、evidence digest和required capability的append/commit record；Hub成功应验证共同request hash，而不是仅核对child PID。

### 7.4 Focus完成语义进步，但namespace与recovery未闭合

per-request file和native-owner ack已解决覆盖与假Focused的核心问题。剩余风险在边界：publisher自建目录，sequence只在进程内单调，public serde绕过validator，过期清理在size cap前读取，计数忽略I/O失败，malformed claim无回收，ack/requests无统一retention。下一版应由Editor发布generation-qualified inbox binding receipt，Hub只向该capability写入；receiver对所有terminal输入都返回typed disposition并交给bounded quarantine owner。

### 7.5 Recent transaction正确性提升，但owner位置错误

shared store已能避免多数双writer lost update，并提供真实durability底座；问题不是“再写一套store”，而是把现有transaction service迁到Hub/共享host service，Interface只保留operation/projection schema和validated codec。迁移时必须保留CAS、tombstone、quarantine和durability测试，不得退回旧的Hub/Editor双实现。

## 8. 目标合同与owner

| 合同 / owner | 必须证明 | 禁止继续依赖 |
|---|---|---|
| `ProjectDescriptorProbeV2` / Runtime manifest service | source identity、digest、reader policy、validated/deferred sections、budget、migration receipt | public summary字段与Hub `Valid` |
| `ProjectLaunchRequestV2` / Hub/App issuer | operation、locator、profile、required capabilities、deadline、nonce/hash | 分散CLI session/protocol参数 |
| `ProjectAdmissionCommitV2` / Editor session owner | ProjectIdentity、BuildSet、process creation、admission epoch、ledger digest、session generation | OS lease或record单独冒充Ready |
| `EditorStartupReceiptV2` / Editor host | request hash、逐milestone evidence、window/first-present/interactive、terminal/cancel | 固定全集milestone与PID-only验证 |
| `FocusInboxBindingV2` / Editor window owner | target generation、namespace capability、quota、expiry、ACL | publisher自行创建目录 |
| `FocusRequest/AckV2` / Hub + Editor | writer epoch、sequence、deadline、dedupe、terminal disposition、cleanup receipt | public invalid DTO与永久private claim |
| `RecentProjectOperationV2` / Hub recent service | ProjectId、writer/revision、upsert/tombstone、bounded transaction、repair/durability | lossy path identity和Interface filesystem store |
| `CrossProcessCorrelationV1` / diagnostics service | operation/admission/session/build/request IDs、privacy、metrics、retention | message字符串恢复身份 |

Owner边界保持：Interface只拥有schema、validated value、bounded codec与compatibility disposition；Runtime拥有full manifest/migration truth；Editor拥有preflight消费、admission、activation、heartbeat、focus receiver和Ready evidence；Hub拥有engine/BuildSet resolver、child supervision、request issuer、ack consumer与recent store；App只组合entry intent和host wiring。

## 9. 重构顺序

### M0 · 固定schema family与owner

- 为descriptor probe、launch、admission、startup、focus和recent分别建立SchemaId、support window、budget、privacy与golden corpus。
- 将recent store从Interface迁到Hub/共享host service，保持单一transaction owner和现有CAS/durability语义。
- 把Editor172/Interface02/Hub01/Editor02 canonical findings映射到唯一owner，禁止重复P0。

### M1 · Manifest probe与compatibility receipt

- summary wire、validated display value、full preflight receipt分层；Hub只显示PartialProbe。
- receipt加入descriptor digest/size/reader policy/validated/deferred sections和完整migration steps。
- engine/BuildSet/provider/trust/migration组合为Open/MigrateCopy/MigrateInPlace/Safe/Recovery/Reject typed decision。

### M2 · Admission commit

- 引入随机admission epoch、process creation identity、monotonic heartbeat sequence与observation time。
- session record绑定ProjectIdentity/BuildSet/operation/ledger digest；Ready只通过single commit marker发布。
- Closing/Closed/RecoveryRequired产生terminal receipt和cleanup disposition。

### M3 · Launch与startup receipt

- 合并intent、Hub session、protocol、capabilities和deadline为单一request envelope。
- milestones由阶段owner逐项签发并最终commit；支持progress/cancel/revocation/terminal。
- mailbox执行bounded claim/ack/remove/retention，Hub持续supervise child而不是10秒后失联。

### M4 · Focus inbox与request/ack V2

- Editor发布inbox binding capability；Hub不能自行创建目标namespace。
- writer epoch + generation + sequence + request ID形成稳定identity；duplicate/replay返回typed disposition。
- malformed/oversize/mismatch进入bounded quarantine并有repair/cleanup owner。

### M5 · Recent operation service

- 用ProjectId与writer identity替换lossy path key，保留revision/CAS/tombstone/logical ordering。
- storage root按product/profile/BuildSet/test instance隔离；quarantine/temp/lock有quota与retention。
- 将current store的file sync、directory sync、write-through replace和corruption tests迁入owner crate。

### M6 · Security、fault与observability

- owner-only ACL、symlink/reparse/parent replacement fail-closed；字段带sensitivity/redaction policy。
- write/flush/rename/read/claim/ack/cleanup逐crash point建立可见状态与repair receipt。
- startup/focus/recent/session指标按统一correlation context聚合。

### M7 · Compatibility与产品资格

- 真实N/N-1/N+1允许矩阵、旧binary/golden corpus、unknown field与upgrade/downgrade测试。
- 双Hub/双Editor、PID reuse、kill、suspend、publisher race、replay、network/permission动态矩阵。
- qualification artifact绑定source fingerprint、BuildSet、platform和test corpus；通过前不得宣称Unreal等价或更优。

## 10. 36项产品资格门当前状态

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01 | Partial | 本文冻结187个当前source path与fingerprint，但尚无自动drift gate，且84个文件untracked。 |
| G02 | Pass | 18个本地参考文件、适用性与指纹可重建，未引用不存在的Unity Hub源码。 |
| G03 | Fail | summary/focus/ack/mailbox/recent仍可由wire直接形成无效public value。 |
| G04 | Fail | partial probe不列validated/deferred/ignored/unsupported sections。 |
| G05 | Partial | Editor preflight以digest/ProjectIdentity revalidate；Hub probe/spawn不绑定该receipt。 |
| G06 | Partial | Editor有4 MiB cap，其他manifest/codec/mailbox depth/item/string/path预算未闭合。 |
| G07 | Partial | migration decision/plan存在；step/lossy/backup/writeback receipt不完整。 |
| G08 | Partial | directional semver和Safe/Recovery profile存在；完整feature/BuildSet/provider decision fixture缺失。 |
| G09 | Fail | mailbox/focus/recent仍共享exact V1，无独立SchemaId/window/corpus。 |
| G10 | Fail | 没有真实N/N-1/N+1 binary matrix。 |
| G11 | Partial | operation、BuildSet、session token和generation分散存在；Launch/Ready不共同绑定project/deadline/hash。 |
| G12 | Partial | receipt列五项milestone；缺preflight/admission/activation逐项evidence，全集由单constructor制造。 |
| G13 | Fail | timeout/cancel没有terminal receipt，Hub不监督超时child。 |
| G14 | Partial | focus有bounded claim/ack；launch mailbox与malformed/stale/oversize cleanup不确定。 |
| G15 | Fail | OS lease与record不是同一epoch snapshot，probe/read TOCTOU仍在。 |
| G16 | Fail | PID/instance没有OS process creation epoch，PID reuse资格缺失。 |
| G17 | Partial | production heartbeat存在；无monotonic sequence、clock/skew/suspend/late disposition。 |
| G18 | Pass | Hub不会把Claimed/Activating/Closing解释为Ready。 |
| G19 | Partial | mailbox等到first present；session record在ledger commit和first present前已对外Ready。 |
| G20 | Pass | focus按target generation/sequence/request ID排队，不再last-writer overwrite。 |
| G21 | Pass | Hub收到精确request-bound `Focused` ack后才报告FocusedExisting。 |
| G22 | Partial | expired/stale/full有稳定ack；malformed/mismatch/duplicate/replay仍无统一terminal disposition。 |
| G23 | Fail | malformed claim没有bounded quarantine和cleanup owner。 |
| G24 | Partial | recent有revision/CAS/tombstone且双writer不静默覆盖；仍无ProjectId/writer identity。 |
| G25 | Pass | recent lease支持deadline/cancel/nonblocking，UI/activation不无限等待。 |
| G26 | Partial | corrupt/oversize可quarantine/rebuild且不阻断startup；没有last-known-good和retention。 |
| G27 | Pass | recent write失败不回滚成功project activation。 |
| G28 | Pass | Hub与Editor使用同一recent store transaction、temp/durability/repair语义。owner crate仍需迁移。 |
| G29 | Fail | storage namespace未隔离product/profile/BuildSet/test instance。 |
| G30 | Fail | ACL、symlink/reparse和parent replacement没有fail-closed contract。 |
| G31 | Partial | Ready/failure已部分redact；path/token/project字段无统一sensitivity策略。 |
| G32 | Fail | 没有write/flush/rename/read/claim/ack/cleanup crash-point矩阵。 |
| G33 | Fail | 双进程、PID reuse、kill、suspend、race和replay动态测试未执行。 |
| G34 | Fail | arbitrary bytes/deep JSON/duplicate key/Unicode/path/oversize fuzz未建立。 |
| G35 | Fail | 没有可按operation/admission/session generation聚合全阶段的telemetry。 |
| G36 | Pass | 本报告已完成链接、编号、状态、计数、索引和`git diff --check`静态验证。 |

合计：**14 Fail / 14 Partial / 8 Pass**。

## 11. 验证说明

本轮没有运行Cargo，因为任务是当前源码合同review，且选择集有68个tracked修改和84个untracked文件；编译通过也不能证明跨进程原子性、PID identity、ACL、crash consistency或version skew。完成的静态验证包括：79个Interface直接合同文件与108个focused consumer文件的物理冻结、238个test属性计数、manifest/preflight/admission/Ready/focus/recent逐符号调用链、18个本地参考文件复核、Interface06全部70个旧条目重判，以及报告链接/状态/计数/索引检查。

结构审计结果因两次有界超时不可用，不记为Pass。后续实施必须重取指纹，并在clean checkout、真实Hub+Editor、Windows/Linux、kill/fault、ACL/reparse、fuzz与old/new binary上产生绑定BuildSet的动态证据。当前`Closed`只说明旧命题在这份工作树源码中被消除，不证明提交可重建或产品资格完成。

## 12. 审查决策

`zircon_runtime_interface`当前可以保留ProjectIdentity/digest、typed intent/compatibility、lifecycle/generation、production heartbeat、first-present mailbox、generation-qualified focus ack和revisioned recent transaction；不能把Hub summary称为完整项目Valid，不能把session record Ready称为activation事务已提交，不能把固定milestone集合称为逐阶段证据，不能把recent lexical path称为ProjectId，也不能继续让Interface成为filesystem/OS-lock/recovery store owner。

下一实施切片应先完成M0 owner/schema freeze与M2 admission commit，再迁移recent store owner；不得在当前V1上继续增加可选字段来掩盖共同identity和commit缺失。在G09-G17、G29-G35关闭前，不应宣称该跨进程链达到或超过Unreal工程级别。

# `zircon_runtime_interface` serialization / project / resource / reflection / world-sync 当前源码复核

> report_id: `Interface10`
> kind: `current-source-review`
> review_scope: `zircon_runtime_interface` 的 serialization、project、resource、reflect、world_sync，以及它们在 Runtime、Editor、App、Host 中的生产消费者。
> baseline: 当前工作树 `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f`（2026-08-29）；本报告不把未提交改动当作已交付能力。
> source_fingerprint: `3dcbb4ea632ef0157e1c9023547cfbb7e4da340941bfd769836907f3c1867b5f`
> reference_fingerprint: `e7bb7d4d70d5b24e86632ececc09468fac3d5161ae0e3e3a361708c8f10d7363`
> status: review-only；本轮不修改 production Rust，不运行 Cargo/Editor/GPU/跨进程/跨平台动态测试。

## 1. 结论

这轮不是对历史报告的文字复制，而是按当前源码逐文件复核 Interface02 所覆盖的五个公共数据域。当前 interface 选择集为 156 个文件、13,581 行、417,420 bytes、133 个测试属性；对应的 Unreal、Godot、Bevy、Fyrox、Unity Graphics 参考选择为 16 个文件、6,804 行、390,050 bytes、44 个测试属性。选择集指纹和参考提交见第 2 节。

当前实现已经有一些可以保留的工程化底座：

- binary wire 已固定 magic、little-endian/varint/reject-trailing、容器/深度/节点/字符串和 64 MiB body 限制，并有 `BinaryNode` 顺序的 golden bytes 测试。
- `VersionedSchema`、显式 migration chain、future-version 拒绝、manifest v1-v3 迁移、strict BLAKE3 manifest digest、non-nil `ProjectGuid`、launch idempotency operation ID 已形成可演进方向。
- `PersistedAssetReference` 已区分 Project asset 与 Builtin locator；退休 asset-ref 迁移对 material/model/scene 有 exact-shape helper；session lock 已有 Windows mutex/Unix flock、严格 codec、生命周期和 residual recovery。
- `TypeRegistry` 已有 full/short path 索引、短名歧义检测、重复 full path 拒绝、component/resource adapter、VM descriptor 校验和内部 `schema_catalog_generation`；world query bounded path 已有 item/encoded-bytes/depth/time 检查；invalidation dirty token 已按有序索引生成。

这些底座尚未组成可发布的公共合同。当前最高风险是：

1. `stable_uuid`、locator、world query 等身份/传输核心仍有跨平台或资源耗尽缺口；其中旧算法迁移和 catalog/redirect 仍未闭合。
2. interface crate 的新增 Project Identity/compatibility/launch/build-script 代码有 31 个未跟踪文件，且 44 个 tracked 文件被修改；干净检出无法复现这些公开模块，这是新增 P0。
3. session admission record 没有绑定 `ProjectIdentity`/project GUID/manifest digest；反射 schema response 没有公开 registry generation；world-sync 分页没有可验证的 continuation/completeness/ack/resync 语义。
4. 动态 DLL 使用 bounded query，Editor 进程内 gateway 仍使用 unbounded `LevelSystem::query_world`，同一 DTO 的安全语义依赖部署模式。

本轮选定账本共 66 项：P0 为 2 Open、3 Partial；P1 为 37 Open、16 Partial、1 Closed；P2 为 6 Open、1 Closed。这里的 61 项是 Interface02 的五域旧条目重判，另有 5 项以 `I10-*` 前缀登记的新差距（1 个 P0、4 个 P1），不与 Interface08/09 或 Interface06/07 重复计数。

## 2. 审查边界与证据

### 2.1 Zircon 物理范围

| 选择 | files | lines | bytes | test attrs | status |
|---|---:|---:|---:|---:|---|
| `zircon_runtime_interface` Cargo/build + serialization/project/resource/reflect/world_sync | 156 | 13,581 | 417,420 | 133 | 44 modified / 31 untracked status entries |
| Runtime/Editor/App/Host 生产消费者（逐符号追踪） | 重点抽查 `World::query_world*`、`TypeRegistry`、reflection gateway、asset migration、preflight/session、invalidation projection | n/a | n/a | n/a | 作为语义证据，不并入主指纹 |

主指纹算法：按规范化相对路径排序，逐文件 SHA-256，拼接 `path\0sha256\n` 后再 SHA-256。未跟踪文件纳入指纹；这保证报告不会把“文件存在”误写成“已提交”。

### 2.2 参考源码

| 引擎 | 固定提交 | 重点文件/语义 |
|---|---|---|
| Unreal | 工作树无 git commit；以当前 `dev/UnrealEngine` 文件内容为准 | `CustomVersion.h/.cpp`：GUID、version registration、Missing/Older/Newer/Invalid；`SoftObjectPath.h`：asset/subpath/redirect；`PackagePath.h`：mounted/local package identity |
| Godot | `8c7e6c5877a78e8e61ea4fd42673219a9091dca7` | `resource_uid.*`：UID/path 双向表、INVALID、添加/删除；`resource_format_binary.h`：内部/外部资源表；`class_db.h`：类型注册与命名空间 |
| Bevy | `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af` | `bevy_asset/id.rs`：typed stable/untyped ID；`path.rs`：source/path/label、fallible parse；`type_registry.rs`：依赖注册、短名歧义、反射 schema |
| Fyrox | `8d815db36494f1badb347547dfc7094bf4fbbdf8` | visitor version/magic/tagged binary、resource manager UUID/path/collision/move/reload |
| Unity Graphics | `a7e4c051d256a781ab362c64316b125a1e104694` | ProbeVolume migration 版本链；HDRP `AssetVersion` 子资源/版本与迁移归属 |

本地参考共读取 16 个文件，未使用网络资料。参考实现用于建立持久身份、版本迁移、类型注册、资源表和可恢复同步的工程基线，不据此声称 Zircon 已达到 Unreal 或 Unity 的性能。

### 2.3 动态证据边界

本轮是 review-only。没有运行 Cargo、真实 runtime DLL、Editor UI、跨进程 session、GPU、filesystem crash、fault injection、fuzz、100k/1M world scale、跨平台 byte corpus 或性能 benchmark。下文的“Pass/Partial/Open”是静态合同判断；任何 P0/P1 关闭都必须补充相应动态门禁。

Tooling 目录按用户要求排除，Tooling/Rust 迁移问题另行处理；Host foreign-output 的 ownership/admission 细节回指 Interface09，本报告只检查其消费的 DTO 和 project/resource 语义。

## 3. 当前纵向链与可保留底座

### 3.1 Serialization

入口是 `serialization::load_versioned`、`VersionedSchema`、text envelope/canonical writer 和 binary envelope/wire。当前 production `VersionedSchema` 实现包括 `TransactionJournal`、Settings/Layout/Workspace documents、`DynamicScene`、`ReflectedJsonDocument`、`ExportPreset` 等；generic binary caller 仍为零，实际生产文件格式 owner 尚未收口。

保留：schema version/future rejection、migration validation、binary magic/version/limits、tagged node order golden test、canonical map key ordering、直接 current binary decode 的优化方向。

### 3.2 Project

`ProjectManifestSummary` 当前格式 v3，支持 v1->v2、v2->v3 迁移；Editor preflight 对 legacy/migrated receipt fail-closed，并要求显式 migration 后重跑。`ProjectGuid`、manifest BLAKE3 digest、`ProjectIdentity`、engine compatibility、launch intent/idempotency API 已出现在工作树，但后者多数未跟踪。

session lock record v2 包含 principal、BuildSet ID、operation ID、lifecycle、checked epoch、session generation、heartbeat；Editor 使用 OS lease，Hub 通过相同 canonical-root mutex helper 竞争。

仍有三个边界：manifest summary 只做部分字段验证；`RelPath` 是 lexical path 而非物理 filesystem proof；session record 还没有 project identity/digest 绑定。

### 3.3 Resource

`stable_uuid_from_components` 使用 BLAKE3 derive key、framed components、algorithm version 1、UUID v8/RFC variant 和固定跨平台向量。`ResourceLocator` 做 slash normalization、scheme/package/label 分解；`ResourceHandle<T>` wire 只存 `ResourceId`；`ResourceEvent` 带 revision。`PersistedAssetReference` 的 Project/Builtin 分层是当前应统一的持久引用模型。

仍缺 legacy `DefaultHasher` ID 的 catalog/redirect/tombstone 迁移、跨平台 locator grammar、typed handle provenance、强不变量 `ResourceRecord`/`AssetReference` constructor 和可恢复 event stream。

### 3.4 Reflection

公共 DTO 是 `ReflectTypePath`、`ReflectObjectAddress`、`ReflectFieldInfo`、`ReflectNumericRange`、`ReflectTypeRegistration`、`ReflectedValue`、read/write/schema request/response。Runtime `TypeRegistry` 维护 registrations、short path map、ambiguous short paths、内部 catalog generation，并为 component/resource/VM 提供 adapter 和 descriptor validation。

仍缺 DTO 反序列化后的不变量复核、field stable ID/slot、write CAS/revision/permission/correlation、依赖闭包与对外 schema generation。`ReflectSchemaResponse` 只返回 `registrations`，导致远程消费者无法把 schema 快照钉在一个 catalog 世代上。

### 3.5 World Sync

公共 query 只有 Components、Hierarchy、InspectionFields、TransformSnapshot 四类；结果已带 generation，transform 带 replacement epoch。Runtime bounded path 在构建过程中检查 item/encoded bytes/nesting/time；dynamic DLL invalidation 使用 output page/pending queue。

但 `World::query_world`/Editor gateway 仍无界，DTO 没有 page/cursor/snapshot/cancel；invalidation batch 只有 generation/dirty/facts，拆页后的同世代片段没有 part/continuation/completeness/ack/resync，`WatchToken` 只有 non-zero u64（Editor 的 qualified wrapper 不是 interface wire invariant）。

### 3.6 逐文件证据锚点

| 物理文件 | 本轮检查的关键路径 | 结论锚点 |
|---|---|---|
| `zircon_runtime_interface/src/serialization/{load.rs,write.rs,payload_header.rs,versioned_schema.rs}` | envelope detection、schema/version validation、migration dispatch、writer API | schema/version 基础成立；profile/integrity/atomic ownership 未闭合 |
| `zircon_runtime_interface/src/serialization/binary/{wire.rs,encode.rs,decode.rs,value/*}` | magic、bincode options、container/depth/node limits、current/legacy decode | bounded wire 成立；多重物化与历史 corpus 缺失 |
| `zircon_runtime_interface/src/serialization/text/{canonical_writer.rs,canonical_spool.rs,canonical_map_key.rs}` | BTreeMap ordering、TempSpool、duplicate key、writer limits | deterministic order 有；spool quota/journal/recovery 无 |
| `zircon_runtime_interface/src/project/manifest_summary/{summary.rs,parse.rs,migration.rs}` | v1-v3 migration、name/default_scene/roots/settings/library validation | migration/future rejection 有；partial validation policy 与 full manifest owner 无 |
| `zircon_runtime_interface/src/project/{rel_path/*,project_name/*,template_pack/*}` | lexical path, Windows name rules, embedded templates/render | validators 分裂，template identity/pack digest 无 |
| `zircon_runtime_interface/src/project/{project_identity.rs,project_guid.rs,manifest_digest.rs,project_launch_intent.rs,activation_operation_id/*,engine_compatibility/*}` | canonical descriptor、GUID/digest、launch idempotency、compatibility | 逻辑已出现但为 untracked，触发 I10-P0-01 |
| `zircon_runtime_interface/src/project/session_lock/{record.rs,codec.rs,identity.rs,mod.rs}` | lifecycle、heartbeat、OS lease identity、strict codec、recovery | v2 lease 有；identity/digest binding 无 |
| `zircon_runtime_interface/src/project/persisted_asset_reference.rs` 与 `asset_ref/*` | Project/Builtin/runtime-only scheme、custom Deserialize、retired migration | 分层和 exact-shape helper 有；两套 AssetRef authority 并存 |
| `zircon_runtime_interface/src/resource/{stable_uuid.rs,locator.rs,resource_id.rs,resource_event.rs,resource_record.rs,resource_handle.rs}` | UUID framing/version、locator parse、ID/handle/event/record invariants | 基础 identity/revision 有；catalog/redirect/tombstone/stream contract 无 |
| `zircon_runtime_interface/src/reflect/{zr_reflect.rs,object_address.rs,field_info.rs,editor_hint.rs,reflected_value.rs,read_write.rs,schema.rs}` | public DTO constructors/Deserialize、field/value/write/schema response | DTO 可绕过校验；response 缺 catalog generation/fingerprint |
| `zircon_runtime/src/scene/reflect/{type_registry.rs,world_reflection.rs}` | registration indices、short-name ambiguity、adapter、catalog generation、schema projection | 内部 registry 有 generation；公开 response 丢失 generation |
| `zircon_runtime_interface/src/world_sync/{query.rs,invalidation.rs}` | query variants/results、generation、watch token、batch fields | generation 已补齐；page/snapshot/ack/resync 不存在 |
| `zircon_runtime/src/scene/inspection/snapshot.rs` | unbounded/bounded query、budget counters、deadline、replacement epoch | 两种 service 语义分裂，bounded 无 cursor |
| `zircon_runtime/src/scene/dynamic_scene/session/*` 与 `zircon_editor/src/ui/retained_host/app/assets/refresh/*` | invalidation page/pending queue、Editor projection/generation checks | producer 可分页，消费者无法证明同世代完整性 |

上述文件均以当前工作树内容为准；`git status --untracked-files=all` 的 44/31 状态和主指纹在第 2 节冻结。未把仅测试 fixture、Tooling 或 reference 代码误算为生产 owner。

## 4. 差距账本：Interface02 旧条目当前重判

状态含义：`Closed` 表示当前源码已消除原命题；`Partial` 表示有真实底座但仍不满足原合同；`Open` 表示原差距仍成立。旧条目的完整问题描述保留在 [02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md](02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md)，本表给出当前证据和路由，避免旧报告被误读为 current。

### 4.1 P0

| ID | 当前状态 | 当前源码证据 / 复核结论 |
|---|---|---|
| P0-01 | Partial | `stable_uuid.rs` 已固定 BLAKE3/framing/version/vector，并显式 UUID v8/RFC variant；但旧 `DefaultHasher` 持久 ID 的迁移、redirect/catalog/tombstone 和 persisted algorithm version 仍不存在。 |
| P0-02 | Open | canonical object 使用 `TempSpool`，每个 value 可打开一个 temp file；只有 64 MiB value/body 约束，没有 file-count、spool-byte、depth、attempt journal、stale cleanup 或 crash recovery。 |
| P0-03 | Partial | `ResourceLocator` 已 slash-normalize 并拒绝部分 Windows prefix；但仍调用宿主 `Path::components`/`to_string_lossy`，Windows/Unix grammar、Unicode、case、package/label 规则不一致，不能作为跨平台 identity。 |
| P0-04 | Partial | bounded Runtime path 已有 item/encoded/depth/time budget；Editor in-process gateway 仍走无界 `LevelSystem::query_world`，且 DTO 无 snapshot/pagination/resume，不能保证大 world 不阻塞或可恢复。 |

### 4.2 P1（serialization/project/resource/reflect/world_sync）

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P1-01 | Open | `SchemaId` 仍是任意 `Cow<'static,str>` string，缺 grammar/namespace/collision registry。 |
| P1-02 | Open | `PayloadHeader` 只有 schema id/version，缺 profile、engine/build、encoding、integrity、producer identity。 |
| P1-03 | Open | migration 仍是每个类型的函数指针列表，无集中 catalog、构建期全链证明和 owner metadata。 |
| P1-04 | Open | unwrapped text 仍统一解释为 schema version 0，legacy shape 与真正 v0 未区分。 |
| P1-05 | Open | `$zircon` envelope 保留 key 与 domain key 冲突/转义规则未形成完整协议。 |
| P1-06 | Open | binary write 仍可能同时物化 serde JSON、canonical value、BinaryValue、body/final Vec，多份峰值受限但未 single-pass。 |
| P1-07 | Open | current binary decode 的一般路径仍先物化完整 `BinaryValue` 树；typed direct decode 只覆盖部分调用。 |
| P1-08 | Open | generic binary 代码规模大但没有 production archive/asset format owner 和跨 crate consumer。 |
| P1-09 | Partial | wire magic/version/limits、little-endian/varint/reject-trailing 和 node order golden test 已存在；仍无独立 wire specification、跨版本 corpus 和非 Rust reader。 |
| P1-10 | Open | public `write_canonical_text_to` 明确调用无界 writer，大小责任只在注释/调用方。 |
| P1-11 | Open | canonical spool 没有 save-attempt identity、journal、quota、stale cleanup、crash recovery。 |
| P1-12 | Open | serialization API 不拥有 durable atomic commit；文件替换/sync 由各 domain 自行实现。 |
| P1-13 | Partial | 当前 golden/contract tests 覆盖同一源码版本；没有保存历史 reader/writer compatibility corpus。 |
| P1-14 | Partial | manifest digest 有 BLAKE3 且 preflight 使用 exact bytes；一般 payload/canonical document 没有完整 integrity/source/build binding。 |
| P1-15 | Partial | `RelPath::join_to` 做 lexical join；没有 physical resolve/symlink/reparse proof，文档仍过度宣称不能逃逸 root。 |
| P1-16 | Open | `RelPath` 未定义 filename reserved chars、长度、Unicode normalization、case 和 package boundary。 |
| P1-17 | Open | manifest summary 只检查 `name.trim()`，没有复用 `validate_project_name` 的 Windows reserved/trailing/forbidden 规则。 |
| P1-18 | Partial | `default_scene` 要求非空 trim；仍是 raw String，没有 scene ResourceId/locator/存在性和 project-root contract。 |
| P1-19 | Open | `library_version` 解析后明确丢弃，不能参与 compatibility/cook/recovery。 |
| P1-20 | Partial | roots 做 duplicate 和 O(n²) overlap 检查；缺数量/总字节/深度/单路径预算及 canonical trie/index。 |
| P1-21 | Partial | preflight 明确是 data-only partial probe 并对 legacy fail-closed；DTO 没有 deferred validation receipt/unknown field policy。 |
| P1-22 | Open | templates 仍以 `include_bytes!` 嵌入 interface crate，产品 pack 与稳定 DTO ownership 混合。 |
| P1-23 | Open | template pack 缺 pack ID/version/content digest/engine range/manifest，`ProjectTemplateId` 是 closed enum。 |
| P1-24 | Open | template render 仍只 trim name，绕过完整 `ProjectName` validator。 |
| P1-25 | Partial | exact-shape retired asset-ref helper 已供 material/model/scene migration 使用；DynamicScene/ReflectedJson 仍走任意 JSON 递归全局改写。 |
| P1-26 | Partial | Editor/Hub 生产路径已 canonicalize 后竞争同一 OS lease；底层 helper 仍信任调用方已 canonical，且新 producer 尚未全部 tracked。 |
| P1-27 | Partial | v2 record、lifecycle、OS lease、residual recovery 已成立；record 未绑定 ProjectIdentity/digest，session generation 仍非跨进程持久世代。 |
| P1-28 | Open | locator 没有统一 byte/Unicode/package/label grammar、大小和 reserved policy。 |
| P1-29 | Open | `ResourceHandle<T>` wire 仍只保存 ResourceId，缺 kind/type schema、locator provenance、generation/resolve result。 |
| P1-30 | Open | `ResourceKind` 是 closed enum 且 serde 依赖 Rust variant spelling，插件/未知 kind 无兼容策略。 |
| P1-31 | Open | `ResourceRecord` public fields 可构造空/矛盾 id、kind、locator、state、revision。 |
| P1-32 | Open | `AssetReference::new` 不验证 UUID 与 locator 的对应关系。 |
| P1-33 | Partial | `PersistedAssetReference` 已分 Project/Builtin 并覆盖大量生产文档；`project::AssetRef` 与 `resource::AssetReference` 仍是两套 API。 |
| P1-34 | Open | 没有公共 identity catalog、collision decision、redirect、tombstone、rebuild/repair receipt；runtime shader redirect 不等价于资源 identity redirect。 |
| P1-35 | Partial | `ResourceEvent` 有 revision 和 previous_locator；无 stream sequence、attempt、overflow/gap、ack、snapshot/resync。 |
| P1-36 | Open | `ReflectTypePath` derived Deserialize 可绕过 validated constructor，type path grammar 仅 trim/nonempty。 |
| P1-37 | Open | `ReflectObjectAddress` derived Deserialize 可绕过 constructor；EntityId 仍裸 u64、无 generation。 |
| P1-38 | Partial | registry 校验 plugin consistency 并保存 plugin id；DTO 仍在 type path 与 registration 重复存储 plugin identity。 |
| P1-39 | Open | registration 的 serialization strategy 与 serializable bool 仍可矛盾，未由单一 enum invariant 派生。 |
| P1-40 | Partial | Runtime `TypeRegistry` 已检查 field name 非空/唯一、declared value type 和 default type；公共 DTO 反序列化仍能绕过。 |
| P1-41 | Open | `ReflectNumericRange::new` 不检查 finite、min<=max、step>0；Deserialize 亦无验证。 |
| P1-42 | Open | `ReflectedValue` 是无预算递归 List/Map/JSON 树，接口没有节点/深度/字节限制。 |
| P1-43 | Partial | Runtime 内部 adapter 有 field slots；wire `ReflectWriteRequest` 仍使用字符串 field_name，缺稳定 field ID/slot version。 |
| P1-44 | Partial | Editor 周边有 generation/transaction 逻辑；公开 direct `World::reflect_write` 仍无 revision/CAS、permission、transaction/correlation。 |
| P1-45 | Partial | Runtime registry 已有 full/short path、ambiguity、adapter 和内部 generation；没有 dependency closure，也没有公开 catalog generation/fingerprint。 |
| P1-46 | Open | world query filter/select/type name 仍是任意 String，缺 grammar、canonicalization、declared schema binding。 |
| P1-47 | Closed | Components/Hierarchy/Inspection/Transform 结果现已携带 generation；需保留跨域 golden coverage。 |
| P1-48 | Partial | `u64::MAX` sentinel 仍绕过 NotModified；replacement epoch 只在 transform DTO，缺统一 epoch/resync。 |
| P1-49 | Partial | Editor 有 `QualifiedWatchToken<GatewaySessionIdentity>` 包装；interface wire token 仍 raw non-zero u64，不能证明 session/epoch。 |
| P1-50 | Partial | dynamic invalidation 已有 pending queue、generation、ordered dirty tokens 和输出页 budget；DTO 仍无 sequence range、part/continuation、overflow、ack/resync。 |

### 4.3 P2（本轮五域相关）

| ID | 状态 | 当前复核结论 |
|---|---|---|
| P2-01 | Closed | stable UUID 已显式设置 custom UUID v8 与 RFC variant。 |
| P2-02 | Open | manifest format/content/engine/library 版本仍有命名和职责混淆。 |
| P2-03 | Open | canonical map duplicate key 仍 last-wins，缺 domain-specific reject/merge policy。 |
| P2-04 | Open | locator 仍可能经 `to_string_lossy` 丢失非 UTF-8 信息。 |
| P2-05 | Open | closed serde enum 普遍依赖 Rust variant spelling；未知值无法保留/转发。 |
| P2-08 | Open | interface 顶层 re-export 过宽，serialization/project/resource/reflect/world-sync owner 边界难以机械检查。 |
| P2-09 | Open | 文档仍使用 Stable/portable/cannot escape 等强承诺，和当前 caller-asserted 语义不符。 |

Interface02 的 P1-51/P1-52（Hub mailbox/recent）、P1-53/P1-54（Export）、P1-55（math）、P1-56（diagnostic/contribution）以及 P2-06/P2-07/P2-10 分别路由到 Interface06、Interface04、Runtime153 或 Interface09；本报告不重复重判。

## 5. 本轮新增差距

| ID | severity/status | owner boundary | 影响 |
|---|---|---|---|
| I10-P0-01 | P0 / Open | interface source/release reproducibility | 新增公共 module/build source 未 tracked，clean checkout 无法复现 |
| I10-P1-01 | P1 / Open | resource identity catalog | stable identity algorithm version/legacy redirect 未持久化 |
| I10-P1-02 | P1 / Open | project session admission | lease record 不携带已预检 ProjectIdentity/manifest digest |
| I10-P1-03 | P1 / Open | reflection schema transport | response 丢失 catalog generation/fingerprint，无法检测 stale schema |
| I10-P1-04 | P1 / Open | world-sync transport | 同世代分页缺 continuation/completeness/ack/resync，丢页不可证明 |

### I10-P0-01 · 新增 Project Identity/launch/build contract 未进入可复现 source

`zircon_runtime_interface/src/project/mod.rs` 当前声明并 re-export `canonical_descriptor_identity`、`engine_compatibility`、`manifest_digest`、`project_guid`、`project_identity`、`project_launch_intent`、`activation_operation_id`；`zircon_runtime_interface/build.rs` 也存在，`Cargo.toml` 已加入 build dependency。但这 31 个 Rust 文件是 untracked，44 个 tracked 文件被修改。基于 clean checkout 的 Cargo source 不含这些模块，因而 `ProjectIdentity`、manifest digest、launch operation ID 的公共 ABI/serde shape 无法由仓库复现，使用者也无法知道哪些改动已经通过 review。

影响：Editor preflight 的 identity 计算、Hub launch intent、session lock 的新边界在当前工作树可见，但 release/package/CI/其他开发者检出会缺模块或得到不同 schema。任何基于此 identity 的 lock、focus、recent、asset reference 结论都不可签发。

重构要求：先把 interface crate 的 tracked/untracked 状态纳入 source manifest；为每个公共 DTO 生成 schema manifest、module owner、serde compatibility corpus 和 build-set digest；clean checkout、增量 checkout、缺文件、旧二进制 ABI 均必须 fail-closed。没有这些证据，不能将新 API 标为 Partial/Closed。

### I10-P1-01 · stable identity 算法版本未随 persisted identity/canonical catalog 持久化

`STABLE_UUID_ALGORITHM_VERSION = 1` 被加入 hash 输入，但 `ResourceId`/`AssetUuid`/`AssetReference` wire 没有携带算法版本、origin namespace 或 catalog epoch。旧 `DefaultHasher` 生成的 ID 也没有迁移表、redirect、collision decision 或 tombstone。算法升级会让同一 locator 产生新 ID，而运行时无法区分“重命名”“迁移”“资源删除”或“两个算法得到的碰撞”。

重构要求：定义 `StableIdentityDescriptor { algorithm, version, namespace, canonical_locator, id }`，在资源 catalog 和所有持久引用旁保存；迁移工具生成 old->new redirect/tombstone/collision receipt，读取端只允许显式 migration policy；跨平台 vectors 必须覆盖 old/new algorithm、Unicode、case、package、label 和 path separator。

### I10-P1-02 · session admission 没有绑定已预检 ProjectIdentity

`ProjectSessionAdmissionRecordV1` 当前保存 principal、BuildSet ID、operation ID、lifecycle、checked epoch、session generation、heartbeat，但没有 `ProjectIdentity`（canonical descriptor + ProjectGuid + manifest digest）。Editor preflight 计算出的 identity 在 admission record 中不可见，Hub/Editor focus/recent 消费者无法证明 lease 仍然对应同一 manifest bytes。OS mutex 解决了同一 canonical root 的并发，却没有解决 root 被替换、GUID 被重写或 manifest 在 preflight 后变更的问题。

重构要求：record v3 必须内嵌 identity digest/manifest digest、descriptor canonical form、preflight receipt id 和 build-set digest；每次 heartbeat/ready/focus 都验证 identity；发现 manifest/GUID/root 改变时进入 RecoveryRequired，禁止静默复用旧 lease。跨进程恢复测试必须覆盖文件替换、GUID 改写、symlink/reparse、进程崩溃和旧 record。

### I10-P1-03 · Reflection schema response 隐藏 catalog generation/fingerprint

Runtime `TypeRegistry` 在 registration/upsert/removal 时递增 `schema_catalog_generation`，dynamic-scene compile 也用该 generation 做 stale 检查；但 `ReflectSchemaResponse` 只有 `registrations: Vec<ReflectTypeRegistration>`。远程 Editor 取得 schema 后，插件注册/移除发生时无法检测 snapshot 已过期，也不能把 field read/write 与同一个 schema 版本绑定。内部已有的 generation 因此没有成为公共合同。

重构要求：`ReflectSchemaResponse` 增加 `catalog_generation`、schema fingerprint、registry source/build set、dependency closure 和 pagination cursor；`ReflectRead/Write` 携带 expected catalog generation，stale 时返回 typed error；fingerprint 必须由 canonical registration order、field IDs/types/default metadata 和 plugin identity 计算，而不是 Vec 传输顺序的偶然结果。

### I10-P1-04 · world-sync output page 无法证明同世代结果是否完整

dynamic DLL 的 invalidation path 允许按 item/bytes/depth/time 产生多页并保留 pending queue；`InvalidationBatch` 只有 `generation`、`dirty`、`facts`。一个 generation 被拆成多个 batch 时，没有 `page_index`、`page_count`/`continuation_token`、`snapshot_id`、`is_complete` 或 checksum；Editor 目前仅禁止 generation regression，因此重复同世代、丢失中间页、跨重连复用旧页都无法由 DTO 区分。

重构要求：定义 `WorldSyncSnapshotId`、`InvalidationStreamSequence` 和 page envelope（stream epoch、generation range、page ordinal、continuation、complete/overflow/resync reason、item/byte counts、checksum）；消费端必须 ack，检测 gap/duplicate/epoch mismatch 时请求 snapshot；bounded query 与 invalidation 必须共享同一 snapshot/epoch，不能由部署模式决定是否有界。

## 6. 目标架构与重构内容

### 6.1 Schema Catalog 与 persistence profiles

建立独立 `SchemaCatalog` owner，条目至少包含 schema id grammar、domain、current version、migration chain digest、encoding profiles、compatibility window、owner crate、integrity policy。`VersionedSchema` impl 只提供类型描述和 typed migration；catalog 在 build time 生成并在 runtime 校验。按用途拆分：

| profile | 用途 | 必须具备 |
|---|---|---|
| `Document` | project/layout/scene/asset source | schema/version、canonical text、integrity、atomic commit receipt、unknown-field policy |
| `Archive` | session/journal/recovery | binary wire version、bounded decode/encode、attempt id、journal、fsync/repair |
| `Transport` | DLL/Editor/Hub DTO | ABI/API/schema 分层版本、size/reserved、cursor/ack、build/identity binding |
| `Cache` | derived catalog/artifact | source digest、build set、staleness、rebuild policy，禁止当作 source of truth |

binary writer/reader 要求 single-pass 或可证明的 peak-memory budget；canonical spool 要有 bounded temp-file pool、attempt namespace、quota、cleanup journal、crash recovery。公共 writer 不得再暴露无界 `write_*_to`。

### 6.2 Project identity、manifest 与 session admission

把 `ProjectName`、`RelPath`、`ProjectGuid`、`ProjectManifestDigest`、engine compatibility 和 `CanonicalDescriptorIdentity` 组装成不可伪造 `ProjectIdentity`。manifest parse 分为 syntax、partial preflight、full validation、migration receipt 四阶段；所有阶段输出 source bytes digest 和 unknown-field/deferred-validation policy。

session lease 只接受 `ProjectIdentity`，record v3 把 identity、BuildSet、preflight receipt、operation id、session generation 和 lifecycle 作为一体；canonical root 由 owner 物理解析并记录 reparse/symlink policy。Hub focus/recent/mailbox 必须消费同一 identity，而不是各自用 path/name merge。

### 6.3 Resource identity catalog

统一 `project::AssetRef` 和 `resource::AssetReference` 的 authority，保留 Project/Builtin/RuntimeOnly 三个显式 provenance。资源 catalog 记录：typed `ResourceId`、kind stable ID、locator grammar version、canonical locator、algorithm version、source digest、revision、redirect/tombstone、collision decision 和 package ownership。

所有 mutation 通过私有构造/validated builder；`ResourceRecord` 不再暴露可制造矛盾状态的 public fields。event stream 增加 sequence/epoch/attempt、snapshot cursor、overflow/ack/resync；ResourceHandle wire 携带 kind/schema/provenance/generation，并可明确 unresolved/redirect/tombstone。

### 6.4 Reflection registry

定义统一 `TypePath`/`FieldId` grammar 和 plugin identity source，registration 中只保留一个 authority；serialization strategy 由 enum 派生 `serializable`，不允许双写矛盾。field metadata 在 registration 时生成稳定 slot/field ID，兼容变更通过 alias/tombstone。

公开 `ReflectSchemaResponse` 时返回 catalog generation/fingerprint/dependencies/page cursor。read/write 必须携带 expected object revision、schema generation、principal/capability、operation id；冲突返回 typed stale/permission/error receipt。`ReflectedValue` 和 numeric ranges 使用节点/深度/字节/finite budgets，derived Deserialize 后统一走 validator。

### 6.5 World sync

统一 query 和 invalidation 的 `SnapshotEnvelope`：world replacement epoch、snapshot id、schema/catalog generation、stream sequence、page cursor、budget receipt、complete/overflow/resync reason。所有结果都要能回答“来自哪个 immutable snapshot、是否完整、下一页是什么、如何恢复”。

Components/Hierarchy/Inspection/Transform 保留现有 typed variants，但 filter/select 改为 parsed canonical query AST；单 entity 超限返回 item-level rejection，不能构造全 world 后才失败。Editor、dynamic DLL、Host 必须调用同一 bounded service，不允许 product mode 分叉。

## 7. 分阶段实施路线与验收 Gate

### M0 · 可复现源码与契约清单

- 追踪 interface 当前 31 个未跟踪文件，生成 module/schema/owner manifest 和 clean-checkout CI。
- 对 `I10-P0-01` 建立 baseline digest、source status、ABI/schema diff；任何 untracked public module 阻断发布。
- Gate M0：干净检出可构建 interface；缺失/新增/重命名公共文件会被 manifest 检出；tracked/untracked 数量为零差异。

### M1 · Schema catalog 与 bounded persistence

- 迁移 SchemaId grammar、catalog、profile header、migration digest 和历史 compatibility corpus。
- 把 canonical spool/atomic commit/cleanup journal 收口到 persistence service；加入 file-count、disk-byte、depth、attempt 和 crash repair。
- Gate M1：历史 corpus byte/read/write roundtrip；future/unknown/corrupt/trailing/duplicate-key matrix；peak memory、temp handles、disk quota、kill-at-each-stage 证据。

### M2 · Project identity 与 lease v3

- 统一 ProjectName/RelPath/full manifest validation；物理 canonical descriptor 输出 reparse/symlink policy。
- record v3 绑定 ProjectIdentity、manifest/build digest 和 preflight receipt；Hub/Editor/Runtime lock/focus/recent 只消费 v3。
- Gate M2：root alias、manifest replace、GUID replace、symlink/reparse、crash/restart、two-process race、old record recovery 全部 fail-closed。

### M3 · Resource catalog 与迁移

- 发布 stable identity descriptor、algorithm-versioned IDs、legacy redirect/tombstone/collision registry。
- 合并两套 AssetRef，封装 ResourceRecord/Handle/Event；补 locator UTF-8 grammar 和 cross-platform vectors。
- Gate M3：旧 ID 全量迁移、rename/move/delete/restore、collision、unknown kind、Unicode/case/separator、snapshot/resync、duplicate event replay。

### M4 · Reflection catalog 与 typed mutation

- 生成 canonical registration/dependency catalog、schema fingerprint/generation/page；field ID/slot/alias/tombstone。
- read/write 引入 object revision/CAS、schema generation、principal/capability、correlation/receipt；统一 DTO validation/budgets。
- Gate M4：plugin add/remove、ambiguous short name、schema stale、field rename/alias、CAS conflict、permission、deep value/NaN/oversize fuzz。

### M5 · Snapshot world sync 与统一 bounded service

- 定义 snapshot/epoch/sequence/page/ack/resync envelope；query 与 invalidation 共用 immutable snapshot service。
- 移除 Editor unbounded gateway；所有模式共享 item/bytes/depth/time budget、cancel/deadline 和 continuation。
- Gate M5：100k/1M entity、deep reflection、single oversized item、world replacement、page loss/dup/reorder、reconnect/gap/overflow、deadline cancellation。

### M6 · 跨引擎/跨语言/性能资格

- 以 Unreal custom version/soft path、Godot UID/resource table、Bevy typed AssetId/TypeRegistry、Fyrox visitor/resource manager、Unity migration version 为对照补齐设计审计。
- 生成非 Rust reader/writer、ABI/schema manifest、历史 corpus、property/fuzz 和跨平台 byte vectors；执行 cold/warm/large-world/soak/fault benchmark。
- Gate M6：所有 P0 Closed、所有 Interface10 P1 有证据；无“注释承诺代替执行”、无 unbounded public writer/query、无未绑定 identity 的 admission/redirect/event。

## 8. 后续路由、排除项与完成定义

### 路由

- Runtime DLL ABI、foreign allocation、output owner/admission/fuse：以 Interface09 为准；本报告只消费其 bounded DTO 语义。
- Hub mailbox、recent、focus、project lifecycle cross-process：Interface06；真实 DLL、cross-language、skew/corpus/fuzz：Interface07。
- Runtime support crates、resource manager/RHI/WGPU 下层 owner：Runtime153。
- Editor Prefab/Scene Snapshot/World outliner 等上层产品语义：对应 Editor 当前源码报告；不在 interface 侧重复登记产品功能缺失。

### 排除

Tooling/IDE/Rust migration 不在本轮；没有因为用户要求“先 review”而改写实现。未查询、轮询、等待或实时跟踪协调器；共享工作树其他会话的改动均按现状记录，不做回滚。

### Interface10 完成定义

只有满足以下条件，Interface10 才能从 review-only 转为 implementation-ready：

1. `I10-P0-01` 通过 clean-checkout/source-manifest gate，所有公共模块进入可复现提交。
2. P0-02/P0-03/P0-04 具备 quota、cross-platform grammar、snapshot/page/cancel 和 Editor/DLL 统一 service 的动态证据。
3. I10-P1-01..04 的 descriptor/catalog/lease/schema/page contract 已写入 versioned wire spec，并有历史 corpus、fault 和 scale 验收。
4. Interface02 旧条目的 Partial/Closed 状态在实现后重新 current-source review；不能用本报告的静态 Partial 推断生产已修复。

本轮没有声称 Zircon 性能或表现优于 Unreal/Unity；报告的结论是当前源码仍未形成可证明的工程级公共数据合同，后续应按 M0->M6 依赖顺序重构。

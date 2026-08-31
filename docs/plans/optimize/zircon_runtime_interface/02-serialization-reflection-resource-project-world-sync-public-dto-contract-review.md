---
related_code:
  - zircon_runtime_interface/src/serialization
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/reflect
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime_interface/src/hub_protocol
  - zircon_runtime_interface/src/export
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_runtime_interface/src/math.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/dynamic_api/session/world_sync.rs
  - zircon_editor/src/core/sync/watch_map.rs
  - zircon_editor/src/core/settings/io.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/04-reflection-derive-script-host-macros-schema-codegen-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Serialization/CustomVersion.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Serialization/CustomVersion.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Misc/PackagePath.h
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_uid.cpp
  - dev/godot/core/io/resource_format_binary.h
  - dev/godot/core/object/class_db.h
  - dev/bevy/crates/bevy_asset/src/id.rs
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/Fyrox/fyrox-core/src/visitor/reader/binary.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeVolume.Migration.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/AssetProcessors/AssetVersion.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 02 · Serialization、Reflection、Resource、Project、World Sync 与公共 DTO 工程化差距

## 1. 结论

`zircon_runtime_interface` 已经形成了一批值得保留的接口基础：versioned text 会拒绝 future version，migration chain 会检查连续性；binary reader 有 magic、版本、总字节、节点、entry、字符串和深度上限；canonical writer 拒绝非有限浮点并处理 short write；Project `RelPath` 会拒绝绝对路径和 `.`/`..`；Hub token 使用规范 UUID；world invalidation 会输出有序 dirty token。它并非只有临时 struct。

但这些局部防线尚未组成工程级持久契约。当前最危险的问题首先是身份算法本身不稳定：`stable_uuid.rs` 用 Rust `DefaultHasher` 生成 128 bit UUID，而标准库没有承诺该算法跨 Rust 版本、实现和平台稳定；该结果又被 `AssetUuid`、`ResourceId`、locator reference 和 Editor/插件生产路径当作持久身份。工具链升级可能把同一标签映射成另一资源，且当前没有 identity version、迁移表或 redirect catalog。

第二个直接可用性风险在 canonical text：为保证对象 key 排序，writer 会为对象的每个 value 创建并持续持有一个独立临时文件，直到整个对象输出完毕。64 MiB 文档上限限制的是字节，不限制对象 entry、递归深度、打开文件数或 spool 数；几十万个很短字段可先耗尽 Windows handle、磁盘或 inode。临时文件又散落在全局 temp 下，以 pid/counter 命名，只有 best-effort `Drop` 清理，没有 attempt-owned directory、journal 或 crash recovery。

第三、第四个 P0 来自资源路径和 world query。`ResourceLocator` 依赖宿主 `std::path::Path::components` 规范化字符串，同一反斜杠 locator 在 Windows 与 Unix 可得到不同 canonical text，而该 text 会直接进入 stable ID；`RelPath` 的“cannot escape root”也只是词法保证，不能防 symlink/junction/reparse point。world query 则在 runtime 生产路径遍历整个 world，为每个实体构造反射 JSON，收集全部 rows 后再跨 FFI 序列化；请求没有 page/cursor/max rows/max bytes/snapshot token/cancellation，`Rows` 响应甚至不携带 generation。consumer-side 1 MiB/16,384 item 在途限制发生在 producer 已完成工作之后，不能阻止 OOM 或长时间阻塞。

更深层的共同原因是缺少统一的 Schema/Identity/Type Registry。`SchemaId` 可反序列化任意字符串，四个生产 `VersionedSchema` 各自持有函数指针 migration list，没有 catalog、collision gate、兼容矩阵或 writer/reader support window；resource kind、reflection type path、field name、world component name、export artifact key 又分别定义自己的字符串身份。接口 crate 因而同时承担 generic archive、project template product data、resource DTO、reflection metadata、Hub protocol、export record 和 math aliases，却没有一套发布级 wire profile、预算和演进 authority。

本轮登记 4 项 P0、56 项 P1、10 项 P2。整改不应继续向现有 DTO 随意加 `String`/`Vec` 字段；先冻结并版本化 identity 和 path grammar，封住 spool/world producer 资源耗尽，再建立 Schema Catalog、持久化事务、Resource/Reflection Registry 和分页 world-sync。最后才收敛 Hub/export/diagnostic/editor contribution/math 等次级 DTO，并以 golden corpus、旧新 reader/writer 矩阵、跨 Windows/Linux identity vector、property/fuzz 和故障注入证明兼容性。

## 2. 审查边界与证据

### 2.1 物理范围

| 集合 | 文件 / 物理行 / bytes | 证据等级与边界 |
|---|---:|---|
| selected interface scope | 152 / 10,739 / 368,698 | E3：serialization、project、resource、reflect、world_sync；Hub/export/editor contribution/math 作为次级 DTO |
| selected production | 132 / 8,068 / 274,928 | E3：排除路径名为 tests/test.rs 的文件；仍包含 4 个 inline test attributes |
| selected dedicated tests | 20 / 2,671 / 93,770 | E2：108 个 test attributes；加 production inline 共 112 个 |
| serialization | 46 / 5,290 / 181,885 | E3：text/binary envelope、canonical writer/spool、migration、limits、load/write |
| project | 45 / 1,915 / 67,587 | E3：manifest、RelPath、AssetRef、template、session lock、retired migration |
| resource + reflect + world_sync | 33 / 1,953 / 63,956 | E3：identity/handle/record、type/field/value/address、query/watch/invalidation |
| Hub + export secondary DTO | 26 / 1,184 / 42,083 | E3 declarations；行为 owner 路由到 Hub 01 与 Tooling 03 |
| production `VersionedSchema` adopters | 4 | E3 absence proof：Editor settings、dynamic scene、reflected JSON、ExportPreset |
| production generic binary callers | 0 | E3 Git tracked source search；generic binary 目前只由 interface tests 使用 |

selected scope 指纹为 `cacb85cf3e816073beed52f4c8354668ad728fccc9a1949388b8d3c970e58202`。算法仍为相对路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。该指纹固定的是成文观察点，不是实施 baseline。

成文时 `zircon_runtime_interface/src/lib.rs` 有其他修改，整个 `zircon_runtime_interface/src/host_output/` 约 11 个文件仍未跟踪。后者正在集中 host/profile/operation/plugin/world output 的 consumer budget、metrics 与 session fuse，是正向在途工作；但它尚非 Git baseline，且仍继承 Runtime Interface 01 已登记的 foreign buffer/status unsafe 风险。本文不修改也不替在途实现背书，故 `source_recheck_required: true`。

### 2.2 纵向调用与持久化链

本轮逐项追踪了以下链，而非只阅读公开 struct：

1. `VersionedSchema` -> text/binary write -> canonical/spool/tree -> header -> load -> migration -> production settings/scene/preset caller；
2. project TOML -> manifest summary/migration -> `RelPath`/template pack -> Editor/Hub project workflow；
3. locator/label -> stable UUID -> `AssetUuid`/`ResourceId`/typed handle/record -> Editor asset manager 与插件 runtime；
4. reflect registration/field/value/address -> runtime type registry -> Editor inspector/script/tooling schema；
5. `WorldQuery` -> runtime whole-world inspection -> FFI JSON output -> Editor gateway/watch projection；
6. Hub mailbox/recent project 与 export preset/stage report 的 producer/consumer contract；
7. focused tests、source searches、参考引擎 registry/version/path/migration 实现与当前动态验证结果。

### 2.3 动态证据边界

本轮运行了两次 `cargo test -p zircon_runtime_interface --locked`，但都没有进入 test binary：

- 共享 `D:\ZirconBuilds\cargo-target` 在编译时缺失 `libserde-*.rlib/.rmeta`，rustc 收到指向不存在 artifact 的 extern path；
- 独立 `D:\ZirconBuilds\cargo-target-runtime-interface-review-20260816` 完成 library 编译后，dep-info 目标目录在构建中消失，随后 linker 找不到刚生成的 `libsemver-*.rlib`。

第二次只报告 4 个既有 UI unused/dead-code warning，未证明测试行为。两次结果均属于 build artifact lifecycle/isolation failure，路由到 Tooling 08/10；本文不能把 112 个 test attributes 记为通过，也没有运行跨平台、恶意 object entry、百万实体 world、symlink escape、golden archive 或 fuzz。

### 2.4 参考源码给出的基线

- Unreal `FCustomVersion` 用 GUID、friendly name、reference count 和可选 validator 组成全局 registry，并能比较 Missing/Newer/Older/Invalid；它说明版本身份、登记冲突和 load policy 应集中，而不是由四个 Rust impl 各自持有匿名函数表。
- Unreal `FSoftObjectPath` 将 top-level asset path 与 subobject path 分型，package path 又有独立 owner。可借鉴的是身份、package、subobject、重定向与加载语义分层，不是复制字符串格式。
- Godot `ResourceUID` 同时拥有规范文本、random/path ID、正反向表和持久 cache，并显式处理 invalid/unrecognized UID；binary resource format 则携带格式版本、external/internal resource 与 dependency 表。Zircon 需要同样明确的 ID registry 和 archive owner。
- Bevy `AssetId` 区分运行期 index/generation 与显式稳定 UUID，untyped ID 保留 type identity；`TypeRegistry` 维护 full path、short path 和 ambiguous-name set，并递归登记依赖。Zircon 当前 typed handle 丢失 wire kind、reflection 短名无冲突 owner。
- Fyrox Visitor 把 magic、format version、field tag 和树 IR 放在同一个 archive family，并给版本枚举规定不可删除/移动规则；其 reader 同样并非资源耗尽安全模板，但证明 generic archive 必须有格式 owner，而不是只依赖 serde crate 版本。
- Unity Graphics 的 migration 文件以枚举/asset subobject 保存逐资源版本，并由 asset processor 驱动升级。仓内只有 graphics package，不足以推断 Unity 全引擎序列化；本文仅采用“版本与资产同行、迁移有明确 owner”这一局部证据。

## 3. 已有可保留基础

1. text loader 有 64 MiB输入上限，binary 有 magic/version/body、2M nodes、1M entries、16 MiB string 和 depth 128 等边界；future schema version 被明确拒绝。
2. migration chain 会检查 duplicate、顺序、缺步和多余 step；current-version load 也验证 chain，避免坏迁移长期潜伏。
3. canonical text 统一 lexical key order、单 trailing newline、short-write 行为和非有限浮点拒绝；streaming sink API 避免为了最终写盘再创建完整 String。
4. project manifest migration 与 `deny_unknown_fields` 局部存在；`RelPath` 拒绝绝对/prefix/dot component，AssetRef 将稳定 GUID 与 path hint 分离。
5. ResourceLocator 区分 `res://` 与 `pkg://`，会消解词法 `.`/`..` 并拒绝向根外弹出；typed/untyped resource handle 的 Rust API 已具雏形。
6. reflection DTO 已表达 type kind、field metadata、editor hint、script visibility、read/write 和 plugin owner，比裸 JSON 完整；Editor contribution batch 会拒绝重复 `(kind,id)` 并规范排序。
7. world query 的 `deny_unknown_fields`、有序 row、generation short-circuit 和 invalidation dirty-token canonical fast path可以保留。
8. Hub session token 使用 UUID v4 canonical form，mailbox/focus filename 有 path validator，消费者使用原子发布；具体进程真实性问题已由 Hub 01 拥有。
9. ExportPreset 选择 hard-cut envelope，拒绝 unwrapped v0，并至少验证 profile、空 plugin ID 和重复 package；具体 Build/Cook/Pack 执行问题由 Tooling 03 拥有。
10. 在途 `host_output` 把多个 foreign output 的 bytes/items/decode metrics 与熔断集中到一个 owner；应把预算继续前移到 producer/request，而不是回退为每个调用点各写一套。

## 4. 差距清单

### 4.1 P0：先冻结身份并阻止生产端资源耗尽

#### P0-01 · `stable_uuid` 使用未承诺稳定的 `DefaultHasher` 生成持久资源身份

实现用两个 domain byte 调用 `std::collections::hash_map::DefaultHasher` 拼出 128 bit。标准库只提供 hash-table hasher API，不承诺其算法是跨版本 wire contract；输出也没有算法 ID、namespace version、RFC UUID version/variant 或 golden vector。`AssetUuid::from_stable_label`、`ResourceId::from_stable_label/from_locator` 和 `AssetReference::from_locator` 直接消费该值，生产调用遍布 Editor asset sync、PBR viewer 与插件 runtime。必须立即冻结新 identity 写入，定义显式 versioned BLAKE3/UUIDv5 类算法、domain separation、UTF-8/Unicode/path normalization 和 namespace ID；保存现有 toolchain 输出映射，提供 legacy ID -> canonical ID redirect/migration，跨 Windows/Linux/Rust 版本运行 golden vectors。

#### P0-02 · canonical object 每个 value 长期持有一个 temp file，64 MiB byte cap 不能阻止 handle/disk exhaustion

`CanonicalObject` 的 `BTreeMap<String, CanonicalObjectEntry>` 为每个 value 创建 `TempSpool`，而 `TempSpool` 持有 open `File` 直到对象 emission/drop。大量短字段可在远小于 64 MiB 时打开海量文件；递归对象还会叠加 spools。全局 temp 的 `zircon-canonical-{pid}-{counter}.tmp` 只有 32 次 collision retry 和 best-effort drop，没有 entry/depth/open-file/spool-byte budget、attempt directory 或 crash sweep。应先加硬 entry/depth/open-spool/total-spool-byte limit，再改为 bounded arena + sorted runs/外排或内存阈值切换；所有临时文件归属一次 save attempt 目录和 journal，commit/abort/crash recovery 有确定清理。

状态（2026-08-30，source implemented / managed Cargo pending）：canonical writer 已增加 128 层 nesting、16,384 object entries、8,191 attempt spill files、512 MiB 累计 spool work 四项 typed hard limit；spill file 上限由 `512 MiB / (64 KiB + 1)` 固化，避免以后调整计费路径时丢失文件数 authority。不超过 64 KiB 的 value 留在内存，大 value 才进入 attempt-owned directory，并在 value serialization 结束后关闭 writer。小字段对象的 spool file/open handle 数从每 entry 1 个降为 0；大 value 的 retained writer 数不再随 object entry 数增长。独立 F 盘 release `rustc` harness 覆盖 entry/depth/file-count/output-budget 拒绝、内存阈值、spill 关句柄、journal recovery evidence/rollback、96 KiB value canonical 等价与小对象资源报告，最新 `10/10` 通过。显式把 scratch root 固定到 F 盘后的 16,384 小字段准入/拒绝路径最新 21 samples：P50 `31,357,600 ns`，P95 `57,347,900 ns`，实际 `disk_spool_files=0`、`retained_open_spool_handles=0`；运行后 F 盘 scratch root 为 `0 files / 0 child directories`，系统临时目录 `zircon-canonical-*` 残留为 `0`。该状态不等于 P0-02 完成：managed crate gate、bounded arena/sorted runs 与 runtime-owned crash sweep 仍 pending；当前数据也不是整帧耗时或功耗验收。

结构复核（2026-08-30，journal/crash-recovery slice）：当前 encoder 只在第一个大 value spill 时惰性创建 attempt directory，正常 `Drop` 已能删除本 attempt；但目录身份只有 PID 与进程内 counter，底层 writer 无法可靠判定另一个进程是否仍活跃，也无法抵抗 PID reuse。仓库内 durable resource transaction 由 `zircon_runtime::core::resource::io::transaction` 的持久化 owner 执行 recovery，Unreal DDC `DerivedDataBuildWorker` 同样用 GUID 隔离 scratch，并由拥有者发布或删除。由此拒绝在 interface encoder 内加入 TTL/PID 推测式全局 sweep：该做法可能删除并发 Editor/cook/runtime 正在使用的目录。

本切片已建立恢复所需但不越权的证据层：attempt directory 创建后、任何 `value-*.tmp` 出现前，写入一次固定 magic/version、owner PID 与 attempt id 的小 journal；journal 创建失败则立即回滚整个 directory，normal drop 仍删除 journal 与 values。journal 不在 per-entry/per-chunk 热路径。F 盘隔离 I/O 微基准独立运行两次，每次 5 warmups、21 samples、每 sample 64 attempts：无 journal 的 attempt create/value create/remove P50 为 `2,760,779..3,637,754 ns`，增加 journal 后为 `5,228,206..6,994,015 ns`，绝对增加 `2,467,427..3,356,261 ns`；P95 分别为 `6,809,901..7,513,012 ns` 与 `10,247,712..13,073,276 ns`。这是每个发生 spill 的 canonical document 一次的 F 盘文件系统成本，不是每 value 成本，也不是全序列化/整帧/功耗数据。后续 crash sweep 必须由持有独占 scratch-root admission 与可靠 process-instance identity/liveness 的 runtime persistence service 调用，并对未知版本、缺失/损坏 journal fail closed；在该 owner 与 fault-injection gate 落地前，P0-02 的 crash sweep 不标完成。

#### P0-03 · `ResourceLocator` 的平台依赖规范化参与 stable ID，同一逻辑资源可跨目标分裂

locator 交给宿主 `Path::components`；Windows 把反斜杠视为 separator/prefix，Unix 可把它当普通字符。`to_string()` 后的 canonical locator 又进入 P0-01 的 ID 生成，因此跨平台 cook、Hub、Editor、CI 和 runtime 可能为同一输入产生不同 ID，或一端拒绝而另一端接受。必须用与 OS path API 无关的 UTF-8 grammar/parser，明确 separator、package/source/label、percent encoding、Unicode normalization、case policy、长度和 reserved chars；先 canonicalize 再 hash。跨平台 vector 必须 byte-for-byte 相等，旧 locator 写入需要迁移诊断而非静默重算。

#### P0-04 · world query 无分页、快照和 producer budget，会先完整构造/序列化整个 world

runtime `World::query_world` 调用 `node_records()`，为每个实体建立 fields、逐 selector 生成 `serde_json::Value/BTreeMap`，最后 collect 全部 rows；FFI 再完整序列化输出。DTO 没有 limit/page/cursor/max bytes/deadline/cancel，`Rows(Vec<_>)` 不带 generation/snapshot token，调用者甚至不能从首个结果建立版本锚点。在途 consumer cap 发生得太晚。应由 runtime 建 snapshot/page owner，在扫描和 JSON allocation 前验证 query cost/权限，按稳定 cursor 返回 `generation/snapshot_id/rows/next_cursor/truncated`；支持 cancellation、time slice、projection column IDs 和 `ResyncRequired`。百万实体/巨大反射值必须在 producer hard limit 内确定失败或分页，不能依赖 host OOM。

### 4.2 P1：发布前必须闭合的 schema、project、resource、reflection 与 DTO 合同

#### P1-01 · `SchemaId` 可反序列化任意字符串，没有 grammar、namespace 或 collision registry

constructor 是 `const &'static str`，但 Deserialize 直接接受任意 owned String。空白、超长、大小写/Unicode 变体和重复 ownership 都能进入 header。建立 generated Schema Catalog，规定反向域名/engine namespace、长度、ASCII/Unicode policy、owner、current/min reader/min writer 和 retired IDs；startup/CI 拒绝 collision。

状态（2026-08-30，grammar source implemented / catalog pending）：`SchemaId` 已收敛为单一 const/runtime validator，wire grammar 为 1-128 ASCII bytes、至少两个 dot-separated namespace segments、segment 以 lowercase letter 开始且仅允许 lowercase/digit/interior hyphen；空段、大小写、非 ASCII、非法字符、段首尾 hyphen 与超长输入均返回 typed `SchemaIdError`，serde 不再接受任意 owned String。当前 7 个 production `SchemaId::new` 均符合该 grammar，F 盘独立 release harness `4/4` 通过。对照 Unreal `FCustomVersionRegistration` 的 GUID key + version + friendly name + central duplicate-registration check，本切片仍缺 generated catalog、stable numeric/GUID identity、owner/current/min reader/min writer、retired IDs 和 collision gate，因此 P1-01 继续保持 in progress，managed crate gate pending。

#### P1-02 · `PayloadHeader` 只有 schema ID 与 u32 version

它不说明 wire format revision、encoding、compression、feature/profile、producer build、payload digest、required capability、flags 或 schema fingerprint。内容格式和 domain schema 应分离版本；未知 required flag 必须拒绝，optional flag 可保留，完整 header 进入 digest/receipt。

#### P1-03 · migration chain 是每个类型的匿名函数指针列表，没有集中登记和构建期校验

四个 production impl 各自返回静态 slice，只有 load 时才验证。引入 catalog registration/derive，构建时生成完整 migration graph、source/target version、lossiness、required context 和 owner；重复/缺步/循环/越界在 CI 失败，不等用户打开旧项目才发现。

#### P1-04 · generic text loader 将所有 unwrapped JSON 解释为 schema version 0

这会把“忘记 envelope”“其他 JSON 文件”与真正 legacy v0 混为一谈。是否接受 legacy 必须由 schema policy 显式声明，并给 sunset/version probe；新格式默认 hard-cut，legacy adapter 在独立入口输出 migration receipt。

状态（2026-08-30，default hard-cut source implemented / managed Cargo pending）：`load_versioned` 的 text policy 已从 implicit schema-zero 改为 Reject，缺少 `$zircon` 时在 payload materialization 前返回 typed `MissingTextEnvelope`；新增 `load_versioned_legacy_schema_zero` 作为唯一显式 legacy adapter。源码逐 schema 复核后，仅 `DynamicScene`（v0→v1→v2）与 `ReflectedJsonDocument`（v0→v1）路由到 legacy adapter；Settings/EditorWorkspace/LayoutPreset 各自已声明 v0 retired，继续使用 default hard-cut，ExportPreset 继续使用 envelope alias。F 盘 focused release harness 覆盖 default reject + explicit legacy accept 以及 ExportPreset 行为，`5/5` 通过。migration receipt 目前仍只有 `Loaded::migrated_from`，尚缺 sunset/catalog policy 与完整 receipt，因此 P1-04 保持 in progress。

#### P1-05 · `$zircon` envelope detection 与 domain object 的保留 key 没有完整规则

普通对象若恰有形似 header 的 `$zircon` 字段会被当成 envelope 并产生不同错误。catalog 应拥有 content-type probe 和 reserved namespace；domain schema 禁止/escape 保留 key，probe 只读取 bounded prefix 并给出明确 NotThisFormat/Corrupt/Unsupported 分类。

#### P1-06 · binary write 在 64 MiB 最终上限前创建多份完整表示

当前先完整走一次 `FiniteFloatGuard`，再 `serde_json::to_value`，转 flat `BinaryValue` nodes，bincode 到 body Vec，再复制到最终 envelope Vec。最终 bytes 有限不等于峰值内存有限。改成 schema-aware streaming encoder或两遍 size+write，整个 attempt 受内存/节点/字符串预算；OOM 不应成为正常拒绝机制。

#### P1-07 · current binary decode 仍先物化完整 `BinaryValue` 节点树

direct decode 避免再创建 `serde_json::Value` 是进步，但仍需完整 node vector，migration path还会转 JSON Value。archive owner 应提供 bounded cursor/visitor 与 typed field decode；迁移若必须 IR，应由 scratch budget 和 spill policy约束。

#### P1-08 · generic binary archive 约 6k 行，却没有 production consumer 或资产格式 owner

生产 `VersionedSchema` 只有四个且全部走 text；binary write/load 的非测试 caller 为零。不要把测试充分的 generic tree 直接宣布为 cook/asset binary format。先由 Asset Archive/Cook owner定义 random access、bulk data、dependency table、compression、endianness、alignment、streaming、schema和patch需求，再决定复用范围。

#### P1-09 · binary body 依赖 `bincode 1.x` 与内部 node layout，缺少独立 wire specification

magic/format version是好基础，但没有语言无关字段说明、canonical bytes、unknown tag policy、reserved range、decoder conformance corpus和跨 crate-version gate。发布格式必须由规范和golden bytes拥有，serde/bincode只是一个实现。

#### P1-10 · public `write_canonical_text_to` 明确无总大小限制，调用约束只存在于注释

runtime-owned archive caller可以把任意对象写入 Vec/文件。API应要求显式 `SerializationBudget`，不存在“unbounded”生产默认；可信离线工具也使用配置化高上限并记录实际用量。

状态（2026-08-30，source implemented / managed Cargo pending）：新增无 `Default` 的 caller-owned `SerializationBudget`，`write_canonical_text_to(value, sink, budget)` 和内部 canonical writer 都必须取得显式 `max_output_bytes`；production `write_canonical_text_unbounded` 已删除，源码扫描为 `0`。唯一 production caller `RuntimeSessionArchive` 将其既有 `limit_bytes` 同时传给 encoder 与 bounded sink，encoder 的 `OutputTooLarge` 统一映射为领域 `ArtifactTooLarge`，不再依赖下游 Vec 拒写后才发现超限。设计对照 Unreal `FArchive::ArMaxSerializeSize/GetMaxSerializeSize()` 的 archive-owned maximum；F 盘 release harness 包含 encoder-side 小预算拒绝并 `8/8` 通过。managed interface/runtime crate gate仍 pending，因此本项保持 in progress。

#### P1-11 · canonical spool 没有 save attempt identity、journal 与 stale cleanup

即使补上文件数限制，进程 crash 后 `Drop` 不执行，全局 temp 残留无法关联源文档或安全回收。持久化服务应创建私有 attempt dir，写 owner/started_at/source/build manifest，startup按 lease/process identity恢复或清理。

#### P1-12 · serialization API 不拥有 durable atomic commit

writer只写任意 sink，fsync、temp publish、parent-dir sync、backup、冲突检测和 rollback由每个调用方自选。对项目/资产设置应提供 PersistenceTransaction：expected revision、write temp、flush、validate reread、atomic replace、directory durability、receipt/recovery；纯 encoder 保持内部层。

#### P1-13 · 没有保存的 writer/reader compatibility corpus

112项当前源码测试不能证明新 reader 读旧 bytes、旧 reader按策略拒绝新 bytes、migration幂等或canonical bytes不漂移。每个schema保存最小/边界/历史golden artifact和manifest，矩阵覆盖 current-2/current-1/current、future、unknown flag与损坏输入。

#### P1-14 · canonical output 与 payload 没有完整性/来源绑定

header无digest，读取只验证结构。项目/资产/导出记录应由外层receipt携带algorithm+digest、producer build、schema fingerprint和source revision；校验失败进入quarantine，不直接覆盖可恢复版本。

#### P1-15 · `RelPath` 的“cannot escape root”只成立于词法字符串

`join_to`直接 `root.join`，不会检查已有symlink、junction、mount或Windows reparse point；写入/复制仍可落到root外。类型文档应改为“lexically relative”，真正I/O通过root capability/openat-like resolver逐段拒绝link/reparse，或在受控sandbox中解析最终file identity。

#### P1-16 · `RelPath` 未定义跨平台文件名合法性

它接受Windows reserved device names、colon、trailing dot/space、控制字符、不同Unicode normalization及过长component；Linux创建成功的项目可能无法在Windowscheckout/export。定义project-portable profile与platform-specific profile，错误指出component和目标平台，manifest声明兼容集合。

#### P1-17 · manifest summary 没有复用 `validate_project_name`

summary只检查trim后非空，而template和project-name模块另有约束。一个类型的合法项目名在不同入口不一致。引入validated `ProjectName` newtype并custom Deserialize，所有manifest/template/Hub入口只接收该类型。

#### P1-18 · `default_scene` 是只检查非空的 raw String

它未解析为 `AssetRef`/ResourceLocator，不限制绝对路径、scheme、类型或project ownership。manifest schema应使用强类型scene reference，验证存在性/资源kind在更高层完成，并区分missing、invalid、unresolved和outside-project。

#### P1-19 · `library_version` 被解析后明确丢弃

注释称其为asset-content schema，但summary只 `let _ = self.library_version`。Hub/Editor可接受不支持的内容版本，直到深处失败。定义engine/project/content三个独立compatibility range，open前给出migrate/read-only/reject决策。

#### P1-20 · manifest root列表无总量/输入预算，overlap检查为 O(n²)

大量roots会扩大解析、pairwise比较和后续scan成本。TOML入口先限bytes/depth/table/items，roots canonical sort后线性前缀检查；重复、嵌套、case-fold collision与filesystem alias均结构化报告。

优化记录（2026-08-30，baseline captured / implementation next）：current-source 等价 F 盘 release harness 对 4,096 个互不重叠 roots 执行 `BTreeSet` duplicate pass 加 pairwise overlap pass，共 8,386,560 pairs；21 samples 为 P50 `217,578,200 ns`、P95 `335,529,800 ns`。这证明主要成本是 O(n²) 结构，而非 `strip_prefix` 细节。对照 Unreal `FLongPackagePathsSingleton` 以 `TDirectoryTree` 持有 RootPath/ContentPath 层级查询并以 `TSet` 持有 mount identity；Zircon manifest 是一次性 admission，不需要复制常驻树，拟采用 component-lexicographic sort + adjacent prefix scan，把 duplicate/overlap validation 收敛为 O(n log n) time / O(n) borrowed-index space，并先加明确 root-count admission。该微基准不包含 TOML parse、filesystem alias、case-fold/Unicode 或整项目打开耗时。

状态（2026-08-30，algorithm/budget source implemented / managed Cargo pending）：manifest-summary owner 新增 4 MiB `MAX_PROJECT_MANIFEST_BYTES` 与 4,096 `MAX_PROJECT_ASSET_ROOTS` admission，duplicate/overlap 从 stringly `InvalidValue` 收敛为 typed errors。production validator 复制 borrowed root references、按 `/` components 排序并只扫描相邻项，正确捕获 raw string ordering 会漏掉的 `a`, `a-b`, `a/child`。同机交错 21-sample release 对比中，4,096 roots 的旧/新 P50 为 `133,777,200 / 313,800 ns`（`426.31x`，`-99.77%`），P95 为 `189,681,100 / 479,700 ns`（`395.42x`，`-99.75%`）；直接 include production `parse.rs`/`limits.rs` 的 F 盘 harness 对 component overlap、duplicate、root-count、document-byte budget `2/2` 通过。TOML depth/table/item budget 已在下述 follow-up 实现；尚缺 case-fold/Unicode collision、filesystem alias 与 managed crate gate，因此 P1-20 保持 in progress。本数据不是完整 manifest parse 或 project-open 端到端耗时。

后续架构复核与实现（2026-08-30，TOML complexity admission source implemented / managed Cargo pending）：current source 原先在 4 MiB byte gate 后直接物化 `toml::Value`，随后进入 JSON 投影与 migration，没有项目域的 container complexity contract。当前 workspace 的 `toml 1.1.2` 由 `toml_parser::RecursionGuard` 在 parser event 层限制 80 层，因此没有另写 TOML scanner；新增 `manifest_summary/admission.rs` 在结构化 parse 后、JSON 投影前用显式栈执行第二层 domain admission：最大 nesting depth 32、累计 table entries 16,384、累计 array items 65,536，超限返回 typed `max/found` error。遍历为 O(nodes) time / O(frontier) borrowed space，不递归、不复制 key/value。Unreal `FPackageName::IsValidTextForLongPackageName` 同样把 lexical admission 与 mounted identity 分离；Zircon 此切片只拥有 document complexity，case-fold/Unicode/filesystem alias 继续归 portable path 与 I/O authority。TDD red 先得到三个缺失常量的 `E0432`；随后 F 盘 release harness 直接 include production `admission.rs` 并覆盖边界深度、三种超限以及三次真实 TOML parse-to-admission，结果 `7/7`。两次独立进程、每次 5 次预热后的 21-sample 最大允许结构 scan：16,384-entry table P50 `135,900..161,600 ns`、P95 `264,800..468,500 ns`；65,536-item array P50 `539,500..544,600 ns`、P95 `600,600..603,800 ns`。这是 Value 后置 admission 的绝对扫描开销，不是完整 TOML parse、project-open 或功耗结论。focused rustfmt/diff check 通过，managed crate gate 未执行。

#### P1-21 · summary故意忽略完整manifest未知字段，却没有“部分验证”能力声明

Hub读取summary可能给用户“项目有效”的错觉，实际上plugin/platform/editor字段从未验证。返回 `ManifestProbe { core_status, deferred_sections, required_capabilities }`，UI不得把summary parse success等同完整admission。

#### P1-22 · product project template bytes 被 `include_bytes!` 嵌入稳定 interface crate

neutral DTO crate因此拥有模板内容、branding和目录结构；模板任何变化会使整个interface rebuild/contract package变化。移到ProjectTemplateProvider/package artifact，interface只定义versioned descriptor与render request/result。

#### P1-23 · template pack没有pack ID/version/content digest/engine range

rendered entries只是RelPath+bytes，无法证明模板来源、迁移或复现。每个pack需要immutable manifest、file digest/mode、parameter schema、min/max engine、license/provenance和signed receipt。

#### P1-24 · template render再次只 trim name，绕过ProjectName类型

同一无效名称可能被模板写进manifest后才在另一入口失败。render request先构造所有validated parameters，再生成到attempt dir，完整验证后原子publish。

#### P1-25 · retired asset-reference migration递归遍历任意 JSON且按形状全局改写

公开函数接受programmatically constructed `Value`，没有node/depth budget，极深结构可stack overflow；任何恰为 `{uuid,url}` 的domain object都会被改写。迁移必须由schema field path驱动，使用显式stack和budget，输出changed paths/diagnostics/receipt并保证二次执行幂等。

结构与性能复核（2026-08-30，source implemented / managed Cargo pending）：current source 已把匹配收紧为 exactly-two-field `{ uuid, url }`，并提供不遍历 container 的 single-reference primitive；asset migration 已在已知 TOML schema path 上使用该 primitive。仍在使用 whole-value walker 的 production owners 只有 DynamicScene v0 与 ReflectedJson v0。原 walker 递归调用并为所有 array/object 重新 collect container；F 盘 release allocation baseline 对 16,384 个两字段普通对象执行无变化遍历，5 warmups 后 21 samples 为 P50 `10,127,200 ns`、P95 `28,769,800 ns`，每次固定产生 `32,768` allocation calls / `14,024,704` allocated bytes，证明瓶颈来自结构性 container rebuild，不是 reference resolver。

新 walker 先以只读 iterator-frame state machine 完成 admission，再以可变 iterator-frame state machine 原地 rewrite；resolver 在完整 admission 前不会执行，算法为 O(nodes) time / O(depth) auxiliary space。标准 hard limits 为 depth 128、nodes 2,000,000、retired references 1,000,000，另有 caller-owned `RetiredAssetRefMigrationBudget` 入口；超限返回 typed `resource/max/found`。生产源码直连 F 盘 release harness 覆盖 depth/node/reference pre-resolution rejection、exact migration、二次幂等、lookalike、malformed exact shape 与 builtin 分流，最终 `8/8` 通过；runtime asset-migration 的 exhaustive error mapper 已增加明确 resource-limit issue arm，没有 wildcard suppression。两次同进程交错 old/new、每次 5 warmups + 21 samples：P50 `8,545,500 -> 7,538,100 ns`（`-11.79%`）与 `9,553,000 -> 6,957,900 ns`（`-27.17%`）；P95 `14,015,900 -> 12,488,700 ns`（`-10.90%`）与 `12,809,200 -> 10,413,300 ns`（`-18.70%`）。allocation calls `32,768 -> 4`、allocated bytes `14,024,704 -> 760`，均下降 `99.99%`。Unreal `FCoreRedirects::RedirectNameAndValuesUnderReadLock` 对 registered typed name/value rule 迭代到 fixed point 并检测冲突，不把任意 object shape 当成 schema；DynamicScene/ReflectedJson 的最终 schema-path rule、changed-path receipt 与 redirect conflict catalog 仍 pending，因此 P1-25 保持 in progress。

#### P1-26 · project session lock用调用方原始路径构造Windows mutex identity

路径alias、大小写、symlink/junction、短路径/长路径可能得到不同mutex，两个进程各自认为独占。lock owner需使用canonical project identity（manifest GUID + verified filesystem identity），lockfile/mutex/Hub recent/session统一复用。

#### P1-27 · session lock record是弱ad-hoc lease而不是可恢复session protocol

record只有pid、宽松instance string和heartbeat；不证明进程create time、host、project/build/session nonce，instance也无长度上限。line `key=value` codec另造格式且无version/checksum。采用versioned bounded record、OS process creation identity、lease epoch/expiry、owner nonce和atomic heartbeat；明确它不是数据锁，实际save仍需revision/CAS。

#### P1-28 · ResourceLocator缺少统一长度、Unicode、package/label grammar

即使完成P0跨平台parser，也要限制总长/component/label，定义package namespace/version/provider、percent encoding和case policy。解析错误必须保留offset/category，不能用lossy string conversion。

#### P1-29 · typed `ResourceHandle<T>` 的wire只保存 `ResourceId`

phantom marker被跳过，不同资源类型有相同serialized shape，反序列化无法验证kind；untyped handle反而携带kind。持久handle应编码stable resource kind/type ID和identity version，typed decode检查匹配；运行期slot/generation handle与持久asset identity分离。

#### P1-30 · `ResourceKind` 是隐式variant-name的closed enum

新增/重命名variant会改变serde wire，未知plugin kind无法保留。定义namespaced stable kind ID、numeric/string assignment、unknown retention、owner/plugin package和capability registry，删除靠Rust enum顺序/名称演进的隐式合同。

#### P1-31 · `ResourceRecord` 的public fields允许不一致provenance/state

空source/importer/config hash、zero importer version、重复dependency、Failed却有artifact、Ready却无artifact/有error都可构造。用validated builder/state-specific record封装invariant；hash使用typed algorithm digest，dependency canonicalize，状态转换附attempt和diagnostic receipt。

#### P1-32 · `AssetReference::new` 允许UUID与locator互相矛盾

`from_locator`只是helper，任意caller可提交不相关的两份authority。稳定ID应为唯一authority，locator只是可变hint并由registry解析；若采用content/path-derived ID，则算法/version必须在record中且验证一致。

#### P1-33 · `project::AssetRef` 与 `resource::AssetReference` 两套模型并存

一套强调稳定GUID+path hint，另一套常从locator派生UUID；转换和ownership未统一。按hard cutover选择统一public reference，定义legacy adapter/migration和禁止新写旧形态的lint/source gate。

#### P1-34 · 稳定identity没有catalog、collision处理、redirect或tombstone

即使替换hash算法，也需要注册时检测duplicate ID/different source、move/rename redirect、delete tombstone、import conflict和merge policy。Asset Registry拥有这些状态，DTO只携带不可变ID和可选hint。

#### P1-35 · resource event缺少sequence、revision、attempt与overflow语义

event kind/record不足以判断漏事件、乱序、旧import完成覆盖新attempt或队列溢出。增加registry epoch、monotonic sequence、resource revision、attempt ID、cause/correlation和`ResyncRequired`；consumer按revision幂等应用。

#### P1-36 · `ReflectTypePath` 的validated constructor可被derived Deserialize绕过

空白/空字符串和无效short path可直接反序列化；`with_module_path/with_plugin_id`也接受未验证字符串。使用custom Deserialize和统一TypeIdentity parser，明确crate/module/plugin/generic参数grammar和长度。

结构准入复核（2026-08-30，source implemented / managed Cargo pending）：原 current source 的 `new` 只检查两段文本 trim 后非空，derived Deserialize、`with_module_path`、`with_plugin_id` 三条入口均绕过同一语义；仓库现有 full path 同时存在 Rust `::` 与 VM `.` 两种 wire family，未发现 generic argument wire。对照 Unreal `FTopLevelAssetPath` 将 package/asset 拆成私有 component、所有 string/component constructor 都进入 `TrySetPath` 且解析失败时 reset/fail，Zircon 已收敛到一个 parser：full path 最大 512 bytes，只允许全 `::` 或全 `.`、禁止 mixed/empty segment；Rust family 每段为 ASCII identifier，VM family 的 namespace 段为 ASCII key token（与 canonical plugin ID 的数字/`-`能力兼容）、terminal type 段仍为 identifier。short path 最大 128 bytes、必须是 full path terminal segment；module path 最大 384 bytes、若存在必须是 full path 的完整 prefix；plugin id 最大 128 bytes、必须是 canonical lowercase ASCII key。generic 参数在当前 revision 明确拒绝，未来必须先引入 schema/versioned TypeIdentity grammar，不能让不同 consumer 自行解析。

调用面量化与 hard cut：`ReflectTypePath::new` 约 30 个 source call sites，`with_plugin_id/with_module_path` 13 个调用点，原 full/short/plugin public-field access 超过 70 处。复核确认生产 consumer 都是只读，只有两个 registry 故障注入测试直接写 plugin projection；因此四个字段现已私有化，读路径统一迁到 borrowed accessor，外部 `ReflectTypePath { .. }` struct literal 为 `0`。custom Deserialize 使用 `deny_unknown_fields` 并复用 constructor/parser，builders 改为 fallible；`ReflectTypeRegistration` 只在 nested path validation 成功后写 canonical plugin owner，顶层 duplicate 已由下述 P1-38 hard cut 删除。`ReflectObjectAddress` 也删除较弱的 non-empty-only validator。约 200 行纯 grammar 已提取到 `type_path/validation.rs` owner，DTO 根只保留构造/accessor/serde/builder 协调。focused crate tests覆盖 invalid wire/unknown field/mixed separator/leaf mismatch/module/plugin/roundtrip；旧 current-source F 盘 probe 已先复现非法 wire 被 derived Deserialize 接受的红灯，随后直接 include production validation owner 的 F 盘 release `rustc` harness `21/21` 通过，其中包含 numeric/`-` plugin namespace 与 terminal identifier 分离。serde 依赖缓存被外部清理，故 custom Deserialize 和 cross-crate accessor 的 current-source managed crate gate仍 pending。P1-36 的 string invariant source closure已完成；stable schema identity/alias/revision归 Tooling reflection identity，不能用本项替代。

#### P1-37 · `ReflectObjectAddress` 同样可绕过constructor，且entity只是裸u64

serde可构造空type path；component address没有world/session/epoch/generation，entity slot复用后可写错对象。地址包含session/world snapshot、entity generation和stable type ID，decode后统一validate。

#### P1-38 · type registration重复存储plugin identity

`type_path.plugin_id`与registration顶层`plugin_id`可在deserialize后分裂。只保留一个canonical owner key，其他投影由registry计算；registration admission验证package/version/capability。

状态（2026-08-30，source implemented / managed Cargo pending）：顶层 `ReflectTypeRegistration::plugin_id` 原 current source 只有 5 个直接 consumer；serialized registration 同时写顶层与 nested key。VM registry replacement 比较 duplicated registration field，`validate_vm_registration` 先读取顶层再与 nested 比较，VM package catalog 也连续验证两份相同 owner，说明第二份没有独立语义，只增加 split-brain 状态。Unreal `FAssetData` 的 identity 由 `PackageName + AssetName`/`FTopLevelAssetPath` component 构成，plugin/mount access 由 package/mount policy查询，不在 identity DTO 再存一份 plugin owner。Zircon 已 hard cut 选择 `ReflectTypePath::plugin_id` 为唯一 serialized owner：registration 顶层字段和全部 5 个 consumer 已删除，replacement/package validation 只读 canonical owner，registration serde `deny_unknown_fields` 明确拒绝旧双字段 wire，`with_plugin_id` 只更新 nested owner。VM registry 对 full/short/plugin 的重复 trim 检查也已删除，只保留 display-name 与 package-prefix/capability 语义 admission；源码 gate 的 registration-level plugin owner/direct multiline nested-field bypass 均为 `0`。focused wire test 已写入，managed serde/cross-crate gate pending。此项不把 `plugin_owned` bool 与 capability sum type 的 P1-39 一并伪装完成；package version/capability admission仍由 plugin package/registry owner负责。

#### P1-39 · `serialization` strategy与`serializable` bool可互相矛盾

类似的plugin_owned/plugin_id、is_component/is_resource、editor/remote/script visibility也可形成不可能组合。用sum type/validated capabilities表达状态，拒绝矛盾registration而非让consumer猜优先级。

结构准入复核（2026-08-30，source implemented / managed Cargo pending）：current source 中 `serializable=false` 与非 `None` strategy 被 runtime reflection macro 和 VM state migration 有意用作“内部具备 codec、但不允许进入持久化/迁移出口”的 admission policy；`editor_visible`、`remote_visible`、`script_visibility` 也分别由 inspector、remote schema 与 script catalog 独立消费。对照 Unreal `EPropertyFlags` 将 `CPF_Transient`/`CPF_SkipSerialization`、`CPF_Edit`、Blueprint/network access 保持为正交 flags，本项不会错误地把这些能力压成一个互斥枚举。真正不可表示的状态有两类：其一，registration-level `plugin_owned` 与 canonical `ReflectTypePath::plugin_id` 重复，12 个 source files/22 lines 的读写中没有第二种生产语义；其二，`is_component/is_resource` 两个 bool 在 13/10 个 files 中被所有 adapter 和 registry 当作互斥分类，registry 还必须显式拒绝 `(true, true)`。hard cut 已删除 `plugin_owned` wire/builder/derive attribute，使用 `is_plugin_owned()` 从 validated `type_path.plugin_id` 只读投影；`ReflectTypeRole::{Value, Component, Resource}` 单字段已替换双 bool，serde wire 只接受一个 `role`，旧三字段由 `deny_unknown_fields` 拒绝。derive 仍在 attribute admission 阶段拒绝 component/resource 同时声明，consumer 只读取 canonical role。focused interface wire 与 derive retired-attribute regression 已写入；27 个受影响 Rust 文件通过 parser-only rustfmt，旧 builder、registration raw bool/plugin projection 和双布尔字段声明 source gates 均为 `0`，scoped diff check 通过。serialization eligibility 与三条 visibility 保留为独立 policy；managed serde/derive/cross-crate gate pending。

#### P1-40 · `ReflectFieldInfo` 没有字段唯一性与类型一致性验证

空/重复field name、无效type path、default value类型不符、enum option重复、numeric range附在非数值字段均可进入schema。registry admission对整个type做原子validate，错误包含type/field/stable ID。

状态（2026-08-30，source implemented / managed Cargo pending）：已完成 current-source review，并写入 `review-2026-08-30-reflect-field-admission.md`。Runtime `TypeRegistry` 现在在任何 publication 前一次性验证每个字段：字段/枚举数量预算、canonical key/text、唯一性、有界声明语法、default representation、editor hint、numeric metadata、enum option 唯一性及 enum default membership；失败使用带 `type_path`/`field_name`/`reason` 的 `InvalidFieldRegistration`，整个 registration 保持原子拒绝。`DeclaredValueType` 由一个 owner 提供 general native 与 strict VM 两种 policy，拥有 256-byte/16-depth 限制和 checked depth；native 保留 `MeshRenderer` 明确声明的动态 `List`，VM 继续只接受 typed `List<T>`/`Map<String, T>`。VM register/sync 已从同一 schema 的两次 validation/descriptor construction 收敛为一次预检结果，identical upsert 不推进 catalog generation；derive 将 `Vec<T>` 递归推断为 `List<T>`，未知 element 不再降级为裸 `List`。F 盘 release parser harness 30/30 通过；21 个独立进程样本、每样本 1,000,000 次 `List<Map<String, List<Scalar>>>` strict parse 为 P50 `482.6 ns/parse`、P95 `516.2 ns/parse`（范围 `435.3..628.4`）。focused source gates 证明 world 不再自行构造第二份 VM descriptor，scoped rustfmt/diff-check 通过。stable field ID 属于 P1-43，递归 value budget 属于 P1-42，本项不伪造 vector slot 为稳定 identity；managed interface/runtime/derive tests 仍待受管 lane。

#### P1-41 · numeric range不验证finite、min <= max与step > 0

NaN、反向范围、零/负step会让Inspector和script各自兜底。newtype constructor/custom Deserialize强制finite和单位语义，允许open bound用显式Option，不用NaN sentinel。

状态（2026-08-30，source implemented / managed Cargo pending）：`ReflectNumericRange` 已从混合 `editor_hint.rs` 拆到独立 `numeric_range.rs` owner，字段私有，唯一 constructor 返回 typed `ReflectNumericRangeError`；custom Deserialize 复用同一 finite、`min <= max`、`step > 0` admission，open bound 继续使用 `Option`。RuntimeInterface current source 除测试外无旧 infallible constructor/public-field consumer；F 盘 release harness `1/1` 通过。range 与 field kind/type/default 的一致性仍归 P1-40 registry admission，故本状态不关闭 P1-40，也不替代 managed reflect contract gate。

#### P1-42 · `ReflectedValue` 是无预算递归树并可携带任意JSON

除通用JSON parser默认递归外没有type-level node/string/container budget；通过非JSON serializer还可能进入非有限float。所有remote/editor/script边界使用`ReflectValueBudget`和finite validator，大blob/array改成paged/bulk handle。

状态（2026-08-30，source implemented / managed Cargo and product evidence pending）：已完成 current-source、仓库既有 budget 与本地 Unreal `FArchive`/JSON stack 重审，并写入 `review-2026-08-30-reflected-value-budget.md`。RuntimeInterface 现在提供 caller-owned `ReflectValueBudget`、typed `ReflectValueValidationError` 和非递归 flat-work-stack validator；统一统计 tagged/embedded-JSON node、depth、Map/object key 与 payload UTF-8 bytes、单容器 entries，并拒绝 Scalar/Vec/Quaternion 非有限分量。Runtime 唯一 policy 为 depth 128、nodes 16,384、累计字符串 1 MiB、单容器 4,096；`TypeRegistry` default、`WorldReflection` read/fields/write、world-query inspection、dynamic JSON component admission、dynamic-scene capture/spawn、reflected JSON read/write、VM reflected object 与 schema default 均在 publication/mutation/serialization 前复用该 owner。旧递归 finite walker 已删除。F 盘生产源码 harness 3/3；1,153-node/6,784-string-byte mixed tree 的 21 个独立 release 进程样本为 P50 `37.3 us/validation`、P95 `54.2 us/validation`（P50 `32.3 ns/node`，范围 `25.7..113.9 us`）。scoped rustfmt 与 diff-check 通过。该数据不是产品 latency/RSS/power 或 Unreal 横向数据；outer DTO/envelope byte/item/time budget 与 paged/bulk handle 仍是独立 owner，managed interface/runtime tests 与产品 workload 证据待受管 lane。

#### P1-43 · field读写主要依赖字符串名称，没有稳定field ID

rename会变成delete+add，旧scene/script/editor state无法可靠迁移。schema codegen生成stable type/field/variant IDs、aliases和migration metadata；显示名与identity分离。

状态（2026-08-31，field public DTO/VM/editor journal/dynamic scene v3 source implemented / descriptor-type-variant identity and managed-product gates pending）：已完成当前 reflection/runtime/scene/VM/editor 字符串身份消费面、本地 Unreal `FPropertyTag`/`PropertyGuid`/redirect 路径与候选算法重审，详见 `review-2026-08-30-stable-reflect-field-identity.md`。RuntimeInterface 新增非 nil canonical `ReflectFieldId`，`ReflectFieldInfo` 强制携带 ID/current name/display name/aliases；native/script derive 生成显式可保留 identity key，拒绝未 trim key。RuntimeInterface schema catalog 原子拒绝 ID/name/alias collision，并拥有唯一 adaptive immutable ID-to-slot index：`<=512` 为 sorted binary，`>512` 为 hash；Runtime `TypeRegistry` 只保留 adapter projection，并同步 catalog 的 register/VM replacement/remove/clear。Public `ReflectReadRequest`/`ReflectWriteRequest` 直接携带 ID，`ReflectFieldValue` 只把 current name 保留为诊断 metadata；WorldReflection component/resource 单字段与批量枚举统一调用 dense slot adapter。VM reflected state 已硬切为 `VmStateFieldValue { field_id, value }` 和默认 V3，删除 `VmStateFieldRename`。Editor command 在 capture 时解析 ID，journal/apply/undo 只消费 ID。Dynamic scene v3 保存 stable ID，显式 v2 importer 从历史 type/name 生成初始 ID，capture/load 采用 schema-order fast path 和 catalog fallback，并在 mutation 前拒绝 unknown/duplicate ID。最终结构 owner 为 `field_index.rs`、schema catalog 与 Runtime adapter projection。F 盘 direct-production-source gate 覆盖 0/1/16/512/513/4096 字段；最新 15-sample source benchmark 中 512 字段 P50 从 `3034.09` 降至 `23.60 ns/probe`（`128.6x`），4096 字段从 `20721.64` 降至 `72.44 ns/probe`（`286.0x`）。descriptor-only plugin 的 `ComponentPropertyDescriptor` 尚无显式 field ID，受 Runtime42 活跃 owner 阻塞；generated type/variant ID 仍 pending。快照 `2422`、manifest `fc579de87232fdc876330d0e31e270ba304bb7763bdd61931a960d7ef03aedb8` 的四组 Windows release managed tickets 已排队：`3701010c374d4a6281ec42715ea4488a`、`b28e2941ebad4b03b46254834cdbaa15`、`c2ab992d9af84cfc8f016f3c41322630`、`efe405cfaae046b680b8cfb15e41cf1f`；排队不等于通过。该 harness 不是完整产品/功耗结论；P1-43 保持进行中，managed Cargo 与产品 workload/RSS/power 待受管 lane。

#### P1-44 · reflection write没有revision/CAS、transaction、permission和correlation

远程Editor可能覆盖runtime新值，多字段修改无法原子提交，plugin/script visibility也不是authorization。定义`ReflectEditTransaction`：object revision、expected field revisions、typed patches、permission/capability、validation、undo token、correlation和structured conflict。

#### P1-45 · interface没有权威 Reflection Registry 与依赖闭包

Tooling 04已确认derive/script/runtime多条authority；DTO本身也无registration set fingerprint、duplicate full path、short-name ambiguity和field ID collision gate。建立单一generated schema IR/registry，像Bevy一样登记依赖并显式保留ambiguous short names，runtime/editor/script都消费同一snapshot。

状态（2026-08-30）：`catalog_and_runtime_projection_source_foundation_implemented /
generated_dependency_persistence_managed_product_validation_pending`。RuntimeInterface 已拥有 bounded
catalog admission、全局 field ID collision gate、scoped legacy alias、dependency closure/order、
versioned BLAKE3 fingerprint 与受验证 snapshot；Runtime `TypeRegistry` 已删除重复 short/ambiguous/
field-slot owner，schema response 携带全 catalog fingerprint，dynamic-scene descriptor batch 在当前
catalog clone 上预检。derive/script dependency edge、public value/request stable-ID wire、scene/VM legacy
import、managed Windows 与产品 profile 仍开放；详见
`review-2026-08-30-reflection-schema-catalog-redirect-authority.md`。

#### P1-46 · world query type name/filter/select没有validate或canonicalize

空字符串、重复selector、同一type同时with/without和超长列表均可进入runtime并放大比较成本。请求decode即转换为validated QueryPlan，使用stable type ID、dedupe/sort、矛盾检测、column projection和cost estimate。

#### P1-47 · `Rows` 响应不携带generation

只有`NotModified`带generation；首次或stale查询返回rows时caller无法仅从结果建立权威generation。每页都返回snapshot generation/ID、page ordinal和complete/truncated状态；NotModified也引用同一protocol header。

#### P1-48 · generation用`u64::MAX` sentinel规避饱和，却没有epoch/resync协议

达到MAX后永远返回Rows只是局部避免错误命中，并没有新epoch、client reset或测试路径。采用随机/monotonic world epoch + revision，wrap/restore/load触发明确`ResyncRequired`，不要保留永久慢路径sentinel。

#### P1-49 · `WatchToken` 只有nonzero u64，不绑定session/epoch/generation

跨session stale token、runtime重启后复用和错误gateway传递难以诊断。opaque token至少含session epoch/generation或由session-local registry验证，wire错误保留raw token与expected session；关闭session时批量revoke。

#### P1-50 · invalidation没有sequence range、gap、overflow、ack或resync marker

batch只有generation/dirty/facts。队列丢帧或consumer断连后无法证明连续，dirty token再规范也不能恢复。协议加入first/last sequence、previous cursor、overflow/truncated、snapshot baseline、ack和`ResyncRequired`，生产队列受bytes/items/age预算。

#### P1-51 · Hub mailbox内容没有绑定session token、expected project/build与process nonce

token只存在于filename/path外部，Ready payload内的pid/project也未形成不可伪造关联；具体spawn验证由Hub 01拥有。DTO应自带protocol/session/build/project/launch nonce和child creation identity，consumer同时校验内容与路径上下文。

#### P1-52 · recent-project identity与merge缺少文件身份、revision和tombstone

lexical path key和ASCII case fold不足以处理symlink/UNC/volume identity；timestamp/name tie-break无法表达删除或并发操作。使用project GUID+verified filesystem identity，操作日志带revision/device/op/tombstone，merge输出冲突而不是静默覆盖；行为整改归Hub 01。

#### P1-53 · ExportPreset validation远未覆盖实际语义，strict load又完整解析两次

features key/value、filters、scene/keep duplicates、customized conflicts、server/client约束和package naming均未验证；先解析`StrictPresetDocument<Value>`再generic load造成双倍解析/分配。由schema probe一次解析typed envelope，调用generated validator并输出all diagnostics；执行消费问题继续归Tooling 03。

状态（2026-08-30，single-materialization source implemented / semantic validation pending）：serialization owner 新增 `load_versioned_envelope` 与 typed `LoadError::MissingTextEnvelope`，复用既有 borrowed `RawValue` probe；`load_export_preset` 删除 `StrictPresetDocument<Value>`、header DTO 和第一次完整 payload materialization，schema/future-version/public envelope error class及 unknown-field fail-closed 行为保留。F 盘 release focused harness 行为 `4/4` 通过。4,194,477-byte preset、21 samples 微基准中，旧/新 P50 为 `206,887,400 / 189,994,400 ns`（`-8.17%`），P95 为 `353,533,400 / 294,738,200 ns`（`-16.63%`），allocation bytes P50 为 `8,390,031 / 4,194,355`（`-50.01%`），allocation calls P50 为 `16 / 5`（`-68.75%`）。这是 loader 局部结构证据，不是整 crate、整帧或功耗验收；features/filter/scene/keep/customized/server-client/package grammar 与 generated all-diagnostics validator仍 pending，P1-53 继续 in progress。

#### P1-54 · export digest/report不是自描述、可审计的stage receipt

`ExportDigest`声称algorithm-neutral，verifier却无法知道算法/domain；artifact key/locator是raw string且digest optional。pipeline report无schema/build/source/attempt/toolchain/timestamps/duration/required-stage closure，duplicate stage时`record()`静默取第一项。改为algorithm-tagged digest、typed artifact locator和versioned immutable stage receipts，拒绝duplicate/missing required stage。

#### P1-55 · public math contract直接暴露glam类型且不保证有效变换

type aliases把glam版本/serde表示变成public wire依赖；Transform反序列化可含NaN、非单位quat和奇异scale。`perspective`不检查finite或far>near，`inverse`不报告singular，`looking_at`的重合eye/target或共线up会产生退化basis。定义wire-owned scalar/vector/quat/transform schema与validated conversion，所有可能失败的构造返回Result/Option和结构化原因。

#### P1-56 · diagnostics与Editor contribution仍是无界字符串集合，不是可关联的公共协议

script/resource/plugin diagnostics缺少稳定code registry、severity domain、origin/build/attempt/correlation、validated span、related/fix-it和truncation；source line/column可为0。Contribution虽然检查duplicate与排序，但package/id/title/path/schema字符串无统一grammar，batch limit不是payload字段，也无owner version/capability/dependency/permission。两者应消费Schema/Identity Registry并受bytes/items/string budget，unknown contribution可保留但不能默认激活。

### 4.3 P2：在主重构中一并收敛

#### P2-01 · UUID bytes未显式设置RFC version/variant

若最终继续使用UUID载体，应输出规范version/variant并在文本/二进制间round-trip；若只是128-bit ID，应改名避免伪装成标准UUID。

#### P2-02 · manifest format/content/engine版本命名容易混淆

统一成 `container_format_version`、`project_schema_version`、`content_schema_set` 和 `engine_compatibility`，诊断不得只说“version mismatch”。

#### P2-03 · duplicate canonical map key采用last-wins但缺少domain policy

writer能观察自定义serializer重复key并替换前值；持久schema通常应拒绝duplicate。将last-wins只留给明确兼容adapter，默认canonical writer返回duplicate-key错误。

#### P2-04 · ResourceLocator使用lossy OS string conversion

`to_string_lossy`可把不同非UTF-8输入压成同一字符串。public locator只接受UTF-8 grammar，OS path转换失败必须显式报告。

#### P2-05 · closed serde enum普遍依赖Rust variant spelling

除ResourceKind外，export stage/status、world fact、reflection kind等也应由catalog分配稳定wire value与unknown policy，重命名不能直接改持久bytes。

#### P2-06 · focus signal target constructor与path validator分离

可构造无效instance后在落盘时才失败。使用validated InstanceId newtype，filename只是其投影。

#### P2-07 · Hub storage在HOME缺失时静默退回相对`.zircon/hub`

相对cwd会改变协议位置并可能写入项目。storage resolver应要求显式platform data root或返回错误，测试使用injected temp root。

#### P2-08 · interface crate顶层重导出过宽，owner边界难以辨认

serialization、product template、Hub、export、UI、math和FFI一起发布，任何变动扩大编译/兼容blast radius。按wire family拆分crate/package或至少feature-independent modules与versioned facade，禁止互相include product assets。

#### P2-09 · public文档使用“Stable”“portable”“cannot escape”等过强承诺

在identity算法、跨平台grammar、filesystem capability和兼容矩阵完成前，应改为准确的“versioned internal DTO under development”及明确支持边界。

#### P2-10 · 在途 `host_output` 的消费预算与正式DTO预算可能形成双重authority

保留其metrics/fuse实现，但最终limit来自握手/QueryPlan/Schema Catalog，consumer只核验producer遵约。未跟踪源码在实施前必须重新读取，避免报告常量被复制成第三套truth。

## 5. 目标架构

### 5.1 三个注册表与一个持久化服务

| Owner | 负责 | 不负责 |
|---|---|---|
| Schema Catalog | schema/wire ID、版本范围、migration graph、limits、golden corpus、fingerprint | 业务对象存储位置 |
| Identity/Asset Registry | stable asset/resource/project ID、kind、locator hint、redirect/tombstone、collision、revision | OS文件遍历细节 |
| Reflection Registry | stable type/field/variant ID、dependency、capability、ambiguity、serializer/editor/script adapters | Editor widget状态 |
| Persistence Service | attempt、budget、temp/journal、validate、CAS、fsync/atomic publish、receipt/recovery | domain migration语义本身 |

这些owner共享generated IDs和Build Set manifest，但不能合并成一个万能global map。project/template/export/world DTO通过它们获得identity和schema，runtime业务仍由各domain crate拥有。

### 5.2 Archive profile分层

```text
SchemaEnvelope
  wire_family + wire_version
  schema_id + schema_version + schema_fingerprint
  producer_build + required_capabilities
  limits/flags/compression
  payload_length + algorithm-tagged digest

Profiles
  AuthoringText   -> canonical UTF-8, bounded, diffable
  RuntimeArchive  -> typed binary, dependency table, bulk/streaming
  TransportPage   -> bounded page/cursor/snapshot/deadline
  Receipt         -> immutable inputs/outputs/provenance/result
```

generic serde实现可以服务profile，但profile定义格式，不能反过来让serde/bincode/glam crate版本成为事实规范。

### 5.3 World sync目标协议

query admission先把字符串DTO编译为validated `QueryPlan`，解析stable type/field IDs、权限和预算。runtime创建短寿命snapshot并返回page：`snapshot_id/world_epoch/generation/page/rows/next_cursor/complete/usage`。watch stream携带sequence、ack、overflow/resync和session epoch；consumer检测gap后丢弃incremental state并从snapshot恢复。

## 6. 分阶段重构与验收 Gate

### M0 · 冻结P0并建立迁移清单

1. 禁止新增`DefaultHasher` stable ID写入，导出所有legacy label/locator -> ID mapping和调用点。
2. canonical writer增加entry/depth/open spool/total temp bytes硬限制和attempt目录；恶意对象以typed `LimitExceeded`失败。
3. ResourceLocator切到OS无关parser，保留legacy parser只读adapter；建立Windows/Linux golden vectors。
4. world query先加producer max entities/components/bytes/deadline和cancel，超过限制拒绝；随后进入分页实现。

Gate：同一vector跨target byte相等；legacy ID可解析/redirect且不静默重算；百万短字段不会超过固定handle/temp预算；百万实体query不完整materialize且不OOM。

### M1 · Schema Catalog与wire profile

1. 为四个production schema及project/template/export/world DTO分配owner、ID、current/min reader/min writer、wire profile和budget。
2. 生成migration graph、collision检查、schema fingerprint、unknown flag策略和语言无关规范。
3. 删除默认unwrapped-v0；需要legacy的schema显式登记adapter和sunset。
4. 保存text/binary golden corpus与corruption/future fixtures。

Gate：重复SchemaId/build-time缺步直接失败；所有production payload可列出catalog entry；current reader兼容矩阵和canonical byte snapshot在CI稳定。

### M2 · Persistence Service与Project契约

1. 实现attempt directory、journal、byte/node/temp budget、validated reread、CAS、atomic replace、durability和recovery。
2. 引入ProjectName、ProjectId、portable RelPath、typed default scene和engine/content compatibility。
3. TemplateProvider从interface移出，发布immutable signed pack manifest。
4. session lock统一project/filesystem/process identity，并与save revision/CAS区分。

Gate：kill-before/after flush/rename/dir-sync均可恢复；symlink/junction escape被拒绝；Linux生成的portable project可在Windows验证；不支持content version在open前明确拒绝或迁移。

### M3 · Identity/Resource/Reflection Registry

1. 发布versioned identity algorithm与legacy redirect catalog，注册collision/tombstone/revision。
2. 统一AssetRef，分离persistent ID、runtime slot/generation和locator hint；typed wire验证resource kind。
3. 由schema IR生成type/field/variant IDs和Reflect registration，集中验证依赖、矛盾、short-name ambiguity。
4. reflection edit使用revision/CAS/transaction/permission与undo receipt。

Gate：rename/move不改稳定ID；wrong typed handle decode失败；duplicate type/field/plugin owner admission失败；并发Inspector/runtime写入产生conflict而非lost update。

### M4 · 分页World Sync与可恢复Watch

1. QueryPlan admission做type解析、dedupe、矛盾检查、cost和authorization。
2. 实现snapshot page/cursor、producer byte/time budget和cancellation；所有Rows带world epoch/generation。
3. watch batch加入session epoch、sequence、ack、overflow/truncated和ResyncRequired。
4. Editor gateway只维护bounded page/watch state，gap或runtime restart触发snapshot rebuild。

Gate：多页结果稳定无重复/遗漏；snapshot过期给确定错误；取消在deadline内生效；故意丢batch后consumer可检测并恢复；consumer cap不再是producer唯一防线。

### M5 · Hub、Export、Diagnostics、Contribution与Math收敛

1. Hub mailbox/recent identity复用Project/Session identity与revision operation log。
2. ExportPreset一次typed parse并完整validate；stage report变成source/build/attempt-bound receipt。
3. diagnostics使用code/span/correlation/fix-it/budget；contribution使用validated package/type IDs和capabilities。
4. math wire类型脱离glam，所有transform/projection/view conversion验证finite、normalization和singularity。

Gate：伪造/旧Hub ready无法匹配launch nonce；duplicate/missing export stage拒绝；超长diagnostic/contribution受控截断；NaN/degenerate transform不能进入持久/跨界DTO。

### M6 · 兼容、安全与故障矩阵

1. 运行old/new reader-writer、Windows/Linux、Rust toolchain、debug/release、endianness/data-model支持矩阵。
2. 对parser/migration/canonical/object entry/reflect value/query page做property与fuzz，OOM/stack/handle/disk故障在child process隔离。
3. 验证每个schema的round-trip、canonical determinism、migration idempotence和source/build receipt。
4. 将`zircon_runtime_interface`测试纳入source/build-bound ValidationSet，并修复共享target artifact lifecycle后再宣称通过。

Gate：golden corpus无未审批漂移；fuzz corpus无panic/stack overflow/unbounded temp；跨平台identity完全一致；构建缓存失败与产品测试失败可在result协议中明确分类。

## 7. 本轮完成定义与剩余队列

本文完成的是 serialization/project/resource/reflect/world-sync 与次级公共 DTO 的首轮 E3 静态审查，不是修复完成，也不是整个 interface crate 完成。4 P0、56 P1、10 P2 均为 `pending`；两次Cargo验证均因artifact lifecycle失败而未执行测试。

Runtime Interface 01继续拥有runtime DLL FFI/foreign ownership；Plugins 01拥有native plugin SDK/admission；Runtime 04与Editor 04拥有asset运行/authoring行为；Editor 05与Tooling 04拥有Inspector和schema codegen；Hub 01与Tooling 03拥有Hub/export执行链。后续interface队列应复核仍未独立闭合的UI/public authoring DTO版本、diagnostic/status统一以及在途host output最终形态；实施任何一项前重新取指纹并确认`lib.rs`/`host_output`的并发修改。

本轮没有修改production Rust、manifest、模板、测试或参考源码。

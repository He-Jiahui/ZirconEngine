---
related_code:
  - zircon_runtime/src/core/resource
  - zircon_runtime/src/asset/facade
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/pipeline
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/pack
  - zircon_runtime_interface/src/resource
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/frameworks/01/2026-08-15-m1-project-generation-durable-transaction-review.md
  - docs/plans/zircon_runtime/frameworks/02/fixed-2026-07-14-stale-subasset-reference-repair.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
reference_engines:
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StreamableManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/IO/IoDispatcher.h
  - dev/UnrealEngine/Engine/Source/Runtime/PakFile
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume
---

# 04 · Core Resource、Asset 与 Serialization 工程化差距

## 1. 结论

Zircon 的资产底座已经不再是简单的 demo 实现。当前 `ResourceManager` 有原子 mutation batch、revision、管理/就绪投影代际和有界事件流；importer registry 使用不可变 COW generation，并支持完整后缀、优先级和 capability；`AssetWorkerPool` 已有 single-flight、有界 waiter/completion、取消和过期；watch dispatch 也有 byte/entry budget、coalescing 与 reconciliation fallback。artifact store v4 使用 staging、内容寻址压缩 chunk、manifest 上限、解压上限和有界共享 chunk residency。这些能力应保留，不能再按旧计划把它们描述成“无界队列”或“一次构造完整压缩 Vec”。

但产品主链仍没有形成工程级资产系统。最关键的断层是：泛型 handle 只编码宽泛 `ResourceKind`，精确类型要等到 payload downcast 才发现；canonical `load_*_asset` 同步读取、解压、反序列化并深 clone 大对象，几乎没有生产调用持有 `ResourceLease`；成熟的异步 worker 没有接入 canonical residency；依赖环会被聚合为 `Loaded`；artifact manifest 没有绑定 source/importer/config/platform/toolchain identity；物理压缩 chunk 不是可独立请求的 mip/LOD/scene-cell；production watch publish 会先丢弃 last-good payload；缺失 subasset label 会静默退回父资产；`.zrpack` 是全内存批处理文件且没有 runtime mount、签名或崩溃恢复事务。

本轮登记 11 项 P1 和 1 项 P2，没有新增 P0。安装器崩溃窗口和未认证 native hot update 是严重发布边界，但当前并无证据表明它已成为默认远程自动更新路径，暂按 P1；一旦产品允许不可信来源触发该链，应由 Host/Plugin owner 上调并先于内容功能处理。

## 2. 审查边界与物理覆盖

### 2.1 已读范围

- 对 `zircon_runtime/src/asset` 全树建立物理清单：551 个 Rust 文件，其中 production 377、tests 174；production 约 54,106 行，tests 约 33,789 行，当前共有 866 个 `#[test]`。本轮逐层扫描入口和调用点，并深读 identity、facade、project loading、worker、watch、importer registry/schema、artifact store、reference resolver、pack/delta/install 主链。
- 对 `zircon_runtime/src/core/resource` 的 57 个 Rust 文件、约 9,381 行完整建立清单，深读 authority、registry、mutation/commit、payload/lease、runtime slot、readiness/management generation、event stream、atomic/durable I/O transaction 及相关并发/事务测试。
- 核对所有 production `load_model/mesh/material/texture/shader/scene/font_asset` 与 `acquire_*` 调用点，确认 load API 在 renderer/streamer/UI 等路径中的实际使用形态，而不是只根据 facade 声明判断。
- 核对既有 Runtime04 的开放 failure、artifact semantic section 记录、lazy residency 记录和 2026-07-14 stale subasset “fixed” 记录；本文只纠正 current source 与既有结论的冲突，不接管其实现 owner。
- 读取 Bevy typed handle/load state、Fyrox resource loader future/type UUID、Godot threaded load/cache mode、Unreal streamable/IoDispatcher/AssetRegistry/Pak，以及 Unity Graphics probe-volume semantic streaming 对应源码。

### 2.2 明确未覆盖

- 本篇不逐格式判定 glTF/OBJ/texture/font/shader importer 的格式规范完整度，也不评估 meshopt、SDF、IBL 等算法质量；它们进入 renderer/streaming 与对应内容格式专篇。
- scene asset 的实体/component schema 本轮只作为序列化 payload 和依赖图证据，world mutation、entity identity、prefab、rollback、determinism 归下一篇 `05`。
- native plugin 包装与动态库激活只追踪到 asset hot-update 入口。ABI、签名信任根、代码卸载和 host rollback 的最终 owner 属于后续 Host/Plugin 审查。
- 没有用静态代码宣称当前 artifact/pack 吞吐优于或劣于参考引擎；本篇只登记缺少的流式契约和必须执行的规模测量。

## 3. 当前实现闭环

### 3.1 Resource authority、mutation 与 projection

`ResourceManager` 用一个 `RwLock<ResourceAuthority>` 共同拥有 registry、management projection、payload、runtime slot 和 readiness，并以单独 commit mutex 串行发布。`ResourceMutationBatch` 会先 prepare 再 apply，成功后推进 revision/generation；event stream 有 4,096 entry、4 MiB 和 60 秒窗口，并报告 gap、dropped、coalesced。新引入的 management/readiness generation 以 `Arc` 投影向 reader 发布，已经比逐查询重建整张表更接近正确方向。

资源记录只含 `ResourceId`、宽泛 `ResourceKind`、URI、revision、依赖 id 和部分导入元数据。runtime slot 维护 `Unloaded/Loading/Loaded/Reloading/Failed` 等状态，`ResourceLease` 可以计数并让旧 payload 在最后 lease 释放后回收；但是这些能力没有成为上层产品加载的默认所有权模型。

### 3.2 Import、worker、watch 与 artifact

`AssetImporterRegistryGeneration` 是不可变索引，按 importer id、extension/full suffix、priority 和 capability 做确定性选择，同 matcher + priority 冲突会被拒绝。`AssetWorkerPool` 将 I/O 交给 task pool，并限制 active request、waiter、completion 与保留时间；完成 payload 共享而不是每个 waiter 复制。watch dispatch 是单 worker 的有界流，溢出后合并并触发 reconciliation，失败会广播 `AssetWatchError`。

artifact store v4 将 bincode payload 写入 zstd stream，再切成 64 KiB 内容寻址的压缩 chunk；manifest 与 chunk 有大小/hash/corruption 检查，读端共享压缩 chunk residency 有 byte budget。这个实现解决了“压缩输出全部留在一个 Vec”和无界重复压缩 chunk 缓存，但读端仍必须按顺序拼接一个 zstd frame 并反序列化完整 `ImportedAsset`。

### 3.3 Project loading、reference 与 pack

`ProjectAssetManager::ensure_resident` 在 residency stripe lock 内同步查 registry、打开 artifact、读取所有 chunk、zstd 解压并 bincode 反序列化；确认 project generation 没变化后，把 payload 写入 resource manager。`load_typed<T: Clone>` 随后取得 `Arc<T>` 并深 clone 成按值返回。异步 worker 与这条 canonical residency 链并未连接。

reference resolver 同时使用 GUID、路径和 subasset label；当 exact label 不存在时，`entry_by_hint` 会退回 base entry。pack writer/reader/delta/installer 则以完整 `Vec<u8>` 为主要边界，每个输入资产目前对应一个 u32-sized chunk；export trim 读取独立 manifest 的 roots/dependencies，而不是直接消费权威 registry/cook graph。代码树中没有发现 `.zrpack`/`pack://` runtime mount resolver 的 production 消费者。

## 4. 差距清单

### P1-1：泛型 handle 没有持久化精确资产类型，类型安全只在 payload downcast 时补救

**证据**

- `asset/facade/handle.rs` 的 `Handle<TAsset>` 只序列化 `ResourceId`，转换到 `ResourceHandle<TAsset::Marker>`；interface handle 也只有 id 与 `PhantomData`。
- `UntypedResourceHandle` 只保存 id 和宽泛 `ResourceKind`，typed conversion 只检查 kind。`ResourceRecord` 和 artifact manifest 同样没有稳定的 `AssetTypeId/SchemaId`。
- 多个不同 payload 共用 marker：`TextureAsset`/`UiIconAsset`，多代 UI style/layout/widget 类型分别共享同一 marker。`load_imported_asset` 因而对同一个 kind 用 `.or_else` 逐个探测 variant。
- `Assets::get` 最终用 Rust `TypeId` downcast，所以错误不会产生内存不安全，但反序列化和 handle conversion 会过早宣称成功，直到加载后才返回 `None`。

**后果与目标契约**

重命名、插件边界、cook cache、网络/场景引用和长期存档都不能依赖进程内 `TypeId` 或大类 marker。目标是注册稳定的 `AssetTypeId + SchemaId`，由 typed codec registry 绑定 Rust payload；handle、resource record、dependency edge、artifact/cook manifest 和错误 DTO 均携带精确类型。可序列化的 `Copy` soft handle 可以保留，但 `typed()` 必须验证精确 type id；迁移旧数据时显式产生 type migration report，不能继续 probe variant。

Bevy 的 strong/untyped handle 保留 exact `TypeId` 并在转换时校验，Fyrox loader 以 data type UUID 选择加载器。Zircon 不应直接持久化 Rust `TypeId`，但应吸收“类型身份在 payload 之前就可验证”的契约。

### P1-2：产品加载深 clone 大资产，lease 与实际 payload 寿命脱节

`Asset` trait 要求 `Clone`；`ProjectAssetManager::load_typed` 在确保驻留后把 `Arc<T>` 内部对象 clone 返回。Mesh 拥有 attribute/index/morph/skin/SDF/virtual geometry 的多组 Vec/Map，Texture 拥有 RGBA/container bytes，Model/Scene 也拥有大型 Vec。当前 production 可找到约 50 个 `load_*_asset` 调用点，却没有专用 `acquire_*_asset` 消费者；resource facade 外只有通用 `Assets<T>::acquire` 暴露 lease。

同时 `get` 返回的 `Arc` 不参与 lease 计数，调用者可长期持有 payload，而 residency accounting 显示零；按值 clone 又脱离资源 revision、reload 和回收状态。这既制造内存/带宽成本，也使“谁保证当前版本存活”无法审计。

目标是 `AssetSnapshot<T> { strong version lease, Arc<T>, resource revision, content generation }` 成为运行时读取默认值；soft/weak handle 只负责身份，不能隐式保证驻留。最后一个版本 lease 释放后才允许 eviction，reload 后旧 lease 固定旧版本，新 acquire 观察新版本。迁移完所有 production 调用点后删除按值 `load_typed<T: Clone>`，大型 asset 不再因 facade 契约被迫实现深 Clone。

### P1-3：canonical residency 是调用线程同步全量加载，成熟 worker 没有接入产品主链

`ensure_resident` 在 stripe lock 内完成 artifact open/read、顺序 zstd decode、bincode deserialize 和 payload publication。它没有返回可共享 request handle，也没有 priority、deadline、progress、stage、cancel、admission 或失败 backoff；重复失败的帧路径可以再次发起同样工作。project generation 的二次确认能阻止过时结果 publish，但不能阻止被浪费的 I/O/CPU，也不能让 render/UI 选择 fallback。

这不是“没有异步设施”：`AssetWorkerPool` 已经具备 single-flight、有界等待者、取消和完成预算。真正差距是 worker result 与 `ResourceRuntimeSlot/ResourceLease` 不属于一个 state machine。

目标建立 `AssetLoadRequest/AssetLoadFuture`：按 resource + exact type + artifact/cook key single-flight，明确 Queued → ReadingSections → Decoding → Validating → Publishing → Ready/Failed/Cancelled；request 有 priority/deadline/progress/cancel，slot 有失败分类、重试退避和 last-good。同步 API 只能是工具线程上的显式 `wait` wrapper，帧/render submission 路径由结构守卫禁止调用。Bevy pending task/load state、Godot threaded token/status 和 Unreal `FStreamableHandle` 的 priority/cancel/progress 提供了适配证据。

### P1-4：依赖图只存裸 id，环路被明确聚合为 `Loaded`

`ResourceRecord.dependency_ids` 不区分 required/optional、runtime/editor、hard/soft、expected type、版本约束或来源。readiness projection 递归计算 aggregate；再次访问 cycle node 时返回 direct/recursive `Loaded` 并 hash cycle edge。现有 cycle 测试只验证报告终止和浅层 row，实际接受 texture ↔ shader 环为 loaded，而不是把环作为结构错误。

因此缺失依赖会 Failed，循环依赖却可能健康，深链还承担递归栈风险；cook trim、reload invalidation 和 streaming priority 也无法从裸 id 推断正确策略。

目标在 candidate registry commit 前用有界迭代遍历和 SCC/topological analysis 验证图，edge 至少包含 category、required、expected type/schema、minimum revision/version 和 provenance；结果显式区分 Cycle、Missing、TypeMismatch、VersionMismatch、OptionalUnavailable。runtime readiness 消费已验证的 generation，不在查询时递归发现结构。Unreal AssetRegistry 的 dependency category/query 可作大型内容图参考，但 Zircon 应保留更严格的 typed edge 和不可变 generation。

### P1-5：artifact 自身完整，但没有绑定产生它的 registry/cook identity

`ArtifactManifest` 只有 schema version、宽泛 kind、resource revision、content hash、raw/compressed size 和 chunk rows；没有 resource id、exact type/schema、source hash、importer id/version、config hash、platform/profile 或 toolchain/cook key。`ResourceRecord` 已保存部分 source/importer/config 元数据，但 `PreparedProjectArtifactRead` 只捕获路径、URI 和 asset id，read 时没有把 expected descriptor 传给 store。

`ensure_resident` 会在读取后确认 registry row 没在并发中变化，却不会证明同一 URI 上内部合法的 artifact 正是该 row 的产物。`LibraryCacheKey(source_hash, importer_version, config_hash)` 存在于定义/测试，但没有成为 production read/publish 的唯一 identity，当前 cache 判定是分散的。

目标建立版本化 `CookArtifactKey`，至少覆盖 resource id、exact type/schema、source hash、importer id+version、normalized config hash、target platform/profile、toolchain/cook schema；manifest 记录该 key 与 section table，registry candidate 只引用已验证 manifest hash。`open(expected_descriptor)` 在解码前拒绝 stale/type/platform mismatch，发布 transaction 原子绑定 registry generation 与 artifact generation。

### P1-6：64 KiB 压缩 chunk 是物理存储切片，不是 mesh/texture/scene 的语义流式单元

artifact v4 的 chunk 是一个 zstd frame 的连续字节段。标准读取创建顺序 `ChunkReader` 和单个 decoder，最后仍反序列化完整 `ImportedAsset`，raw 上限可到 2 GiB。共享 residency 只约束压缩 chunk bytes，不约束 decoded working set、最终 CPU payload 或 GPU upload budget；任何一个 mip/LOD 需要先读完整前缀并解码整对象。

目标 manifest 提供可独立寻址和校验的 semantic sections：metadata、mesh LOD/cluster、texture mip/face、animation clip/range、audio page、scene cell/component column 等。每节独立 codec、offset/hash/alignment、dependency 和 CPU/GPU size；scheduler 可发 range request、并行 decode、取消未需要 section，并在 decoded/GPU budget 下逐帧 admission。Unity Graphics probe volume 以 cell offset/element count 组织 streamable asset，并用 AsyncReadManager 的多 ReadCommand、cancel/status、staging/GPU buffer 和逐帧 cell budget，证明“语义 section + 调度预算”比任意压缩切片更接近目标。

### P1-7：production hot reload 没有使用 reload state machine，成功发布先丢弃 last-good

resource 层具备 `start_reload/fail_reload`、`Reloading/Error` 和 lease 保留语义，但 asset production 子树没有调用这些 transition；它们只在测试中出现。watch 成功后通过 `upsert_lazy` 发布 registry，`manager/commit.rs` 在 ready record revision 变化时移除 payload并把 runtime 设为 `Unloaded`。于是旧可用 payload 先消失，下一次产品访问再同步加载新 artifact。

watch import 失败则只增加 diagnostics 并广播 `AssetWatchError`，没有让资源 slot 记录失败 candidate、backoff 和 last-good。现有 M3 对 Loaded → Reloading → Loaded/Failed 的目标描述是正确的，但 production chain 尚未实现，不能因 API/测试存在而标为完成。

目标是每个 resource slot 持有 active version 与 candidate version：watch/import 启动异步 candidate build，验证 exact type/dependencies/artifact 后一次原子 publish generation；现有 lease 固定旧 version，新 acquire 切新 version。失败只销毁 candidate，active 继续服务，并保留 structured failure、source revision 和 retry policy。禁止用 `upsert_lazy` 表示已准备好的热重载 publish。

### P1-8：缺失 subasset label 会静默退回父资产，修复 GUID 时改变了语义目标

`reference_resolver::entry_by_hint` 先找 exact labeled entry，再执行 `.or(base_entry)`。现有测试明确期望 stale `#MissingMesh` 最终解析为父 asset UUID 且 `resolved.sub() == None`；2026-07-14 的 fixed 记录把它视为 dangling repair 成功。

这会把“同一资产中的 mesh/clip/material 子对象不存在”改写成“引用整个父资产”。GUID/path 漂移可以自动修复，语义 subasset 消失则必须报错；两者不能共用 fallback，否则场景能加载却绑定到错误对象，错误还可能进入重新保存后的永久数据。

目标区分 stale GUID + exact subasset still exists、renamed subasset with explicit redirect、missing subasset 和 ambiguous candidate。只有 exact label/type 可证明时才修 GUID/path；label rename 必须由 importer 产生版本化 redirect/migration。缺失 label 返回 typed dangling error 和候选列表，绝不退回父资产。该 finding 明确重开并纠正既有 fixed 计划的验收语义。

### P1-9：通用 importer source-schema migration 契约没有转换数据，只是占位接口

`AssetSchemaMigrator::migrate_source_schema` 只收到 source version，没有 bytes、document 或 typed intermediate；`StaticAssetSchemaMigrator` 只比较 current/minimum version并返回“migrated”摘要。production 中没有该 trait 的实际调用或实现。项目 zmeta 已有另一条真实 commandlet migration 链，因此差距不是“所有迁移都不存在”，而是 generic importer contract 不能履行它宣称的 source transformation。

目标 registry 为每个 exact asset type/importer 注册连续的 `N -> N+1` deterministic transform，输入受限 document/bytes、输出新 document、diagnostics 和 migration provenance；支持 dry-run、批量报告、取消、备份/rollback 和 deprecated range policy。缺失任一中间 step 必须失败，artifact key 和最终 schema id 记录完整 migration chain。迁移测试使用历史 golden corpus 和 idempotence/determinism，不以返回摘要文字验收。

### P1-10：`.zrpack` 是全内存构建/读取容器，没有接入 runtime asset source 或权威 cook graph

`ZrPackInputAsset` 持有完整 bytes，writer 收集并构建完整输出 Vec；每个资产当前是一个 u32-sized chunk。reader 持有整个 pack Vec，open 时校验全部 chunk，`read_asset` 再复制 Vec；delta apply materialize 全部目标资产后重建完整 pack。staging/export 还会 `fs::read` base/delta/所有输入并为 deterministic double-run、delta verification 做额外 clone。

pack manifest 只有 path/hash/size，没有 exact type/schema、platform/cook/bundle/section/dependency。trim 的 roots/dependencies 来自另一个可手写 JSON manifest，而不是权威 resource registry generation。asset pack/export/tests 之外没有 `ZrPackReader`、`.zrpack` 或 `pack://` 的 runtime resolver/mount consumer；目前它是离线批工具，不是产品 streaming data source。

目标 cook 从 validated registry graph 生成 platform-specific bundle manifest；reader 以文件/IoDispatcher/mmap/range I/O 为边界，不要求完整 pack 进内存。mount/unmount 发布 generation，支持 base + signed overlay/delta、semantic section lookup、priority/cancel 和 bounded cache；ResourceIo 按 URI scheme 路由 pack source。大包验收至少覆盖 1/10/100 GiB、随机 section read、overlay、mount race、corruption 和内存峰值。

### P1-11：pack 只有内容 hash，没有发布信任、崩溃恢复事务和 native hot-update 绑定

pack 校验 BLAKE3 content hash，但 manifest/chunk 没有 signature、public key id、revocation/trust policy、encryption/compression policy。installer 使用 read/write/copy/rename/remove，未见 owner lock、WAL/journal、file+directory sync；promotion 先把 installed 改名为 backup，再把 staged 改名为 installed，中间崩溃会没有 active generation。copy fallback 直接写 installed，restore error 可被忽略，receipt 又在 promotion 后单独写，无法充当恢复日志；并发 installer 也可能竞争。

hot update 入口先安装 pack，随后从请求中另一个 `export_root` 搜索并 reload native plugins。插件并不是从 pack 加载，因此不能说 pack 已直接执行代码；但两者也没有 authenticated generation/candidate hash 绑定，一个 hash-valid unsigned pack operation 后可以加载与其无关的 native export tree。

目标是 signed manifest/section、trusted key policy、platform/cook identity 和可轮换/revoke 的 key id；安装采用 owner lock、durable journal、staging validation、fsync 和单一 active-generation pointer，启动时可确定恢复或回滚。native hot update 请求必须绑定已认证 pack generation、精确 plugin package/hash/ABI 和 candidate activation transaction；安全 owner 最终由 Host/Plugin 专篇承接。

### P2-1：单个 ResourceAuthority 写锁承担图投影与 payload 状态，规模上限尚无证据

atomic authority 简化了当前正确性，但 commit/readiness reverse closure、registry mutation、payload/runtime slot 和管理投影共享同一锁。readiness 重算会在 publication critical section 中遍历依赖并构造投影；资产数、依赖 fan-out、并发 acquire/reload 增长时，写锁 hold time 和 reader tail latency 可能成为瓶颈。当前未找到覆盖 10K/100K/1M 资源及高 fan-out reload 的受管 benchmark，所以这里登记“必须测量”，不宣称已发生性能失败。

目标先以 workload benchmark 得到 lock hold/wait、commit p95/p99、projection allocation、reader stall 和 retained generations；随后将 candidate graph/build 放到锁外，锁内只做 revision check + immutable generation pointer swap。payload/version shard 可按 resource id 分片，但 registry、readiness 和 active payload 必须以同一个 publication generation 提供可解释的一致性，不能为了分锁重新引入撕裂读。

## 5. 参考引擎证据与适用边界

| 参考 | 已核对机制 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| Bevy asset | strong handle 用 `Arc<StrongHandle>` 保持生命周期，untyped handle 保留 exact `TypeId`；server info 跟踪 pending task、loading/failed dependencies | exact type 在加载前可验证、strong/weak identity、共享 pending request 和明确 load state | Bevy 的进程内 TypeId 不适合作为 Zircon 长期存档/跨动态库稳定 id |
| Fyrox resource loader | loader 暴露 exact data type UUID，并返回 boxed async loader future，包含平台转换入口 | 稳定 type UUID、异步 loader contract、平台派生数据边界 | 不直接复制其资源模型或序列化格式 |
| Godot ResourceLoader | threaded request/token 可复用，提供 status/progress 与 cache mode，加载依赖有明确 API | canonical async request、progress、cache/reuse policy、错误可查询 | 全局 singleton/cache 与动态 Variant 风格不必进入 Rust typed core |
| Unreal Streamable/IoDispatcher/AssetRegistry | request 有 priority/cancel/progress/combined handle；I/O request 可取消；registry 查询依赖类别与可用性 | request handle、优先级与取消、内容图分类、安装/可用性状态 | 不复制同步 load 历史旁路和 UObject/package 兼容债务 |
| Unreal Pak/AssetManager | pak 有签名、加密、压缩块、mount/async；AssetManager 管 primary type、bundle、chunk 和 cook rules | trusted bundle、mount generation、platform cook/chunk rule | Pak 格式本身不是目标；Zircon 应围绕 semantic section 和 content address 设计 |
| Unity Graphics probe streaming | streamable asset 记录 cell offset/count，AsyncReadManager 批量 range read，有 cancel/status、staging/GPU buffer 与逐帧 cell budget | semantic sections、多 range async I/O、CPU/GPU admission budget | 该源码只证明 Graphics probe volume 链，不代表 Unity 闭源通用资产系统 |

## 6. 目标架构与所有权

### 6.1 Identity 与 registry

`AssetTypeRegistry` 是唯一 exact type/schema/codec/migration owner；`ResourceRegistryGeneration` 保存 typed dependency graph、source/cook identity 与 artifact manifest hash。soft handle 只持稳定 id + exact type，strong snapshot 额外持版本 lease。所有 unknown type/schema 先于 payload decode 失败。

### 6.2 Load、residency 与 version

`AssetLoadCoordinator` 以 cook key single-flight 调度 I/O/decode/validation，返回共享 request handle；`ResourceVersionSlot` 同时容纳 active 和 candidate，发布只交换 immutable version pointer。CPU decoded、compressed cache、GPU upload/residency 分别有 entry/byte/time budget，并通过统一 pressure signal 进行 admission/eviction。

### 6.3 Cook、artifact 与 bundle

import/migration 生成规范化中间数据，cook 根据 target profile 产出独立 semantic sections；artifact manifest 绑定全部输入 identity、section table、dependency 和 hash。bundle 只是多个已验证 artifact section 的部署索引，不重新发明身份；runtime mount source 支持 file/range/overlay 和异步读取。

### 6.4 Reload、reference 与发布事务

watch 只产生 source invalidation；coordinator 构建 candidate graph/artifact/version，验证成功后一次发布 registry + active version generation。reference repair 只能保持 exact semantic target，rename 由显式 redirect/migration 完成。pack/plugin update 使用签名信任和 durable generation transaction，失败保留 last-good content/code。

## 7. 硬切边界

1. 引入 stable exact type/schema 后，删除只按 `ResourceKind` 成功转换 typed handle 的路径，不保留 variant probing shim。
2. 完成产品调用迁移后删除按值 clone 的 `load_typed/load_*_asset`；帧线程同步等待由结构测试禁止。
3. 删除 readiness 对 cycle 返回 Loaded 的特殊分支；非法 graph 不能进入 published generation。
4. artifact store read 必须接收 expected cook descriptor；没有完整 key 的旧 artifact 只允许显式离线迁移/重建，不在运行时猜测。
5. hot reload 停止用 `upsert_lazy` 直接替换已加载 revision，全部走 candidate-version publish。
6. 删除 missing subasset → base asset fallback，并纠正依赖该行为的测试/fixture/fixed 记录。
7. 删除“迁移只返回摘要”的通用 schema API，所有 migrator 必须实际转换并登记 provenance。
8. `.zrpack` reader 改为 I/O source 后，移除 whole-pack Vec production API；旧格式只保留离线转换工具。
9. unsigned/无 journal 的 pack 不允许进入 production mount/hot update；native candidate 必须和 authenticated update generation 绑定。

## 8. 测试先行重构里程碑

| 里程碑 | 先写失败测试 | 实现范围 | 退出条件 |
|---|---|---|---|
| M0 · Baseline | 精确记录 1K/100K asset、加载/clone、artifact/pack memory、reload stall | 建 workload、trace、工作区指纹和 owner map | 可复现现状，所有结论有 source fingerprint |
| M1 · Exact identity | 同 kind 不同 type handle 转换、旧 schema、插件 type collision | `AssetTypeId/SchemaId` registry、typed record/handle/codec | 错类型在 I/O 前失败，跨进程 id 稳定 |
| M2 · Graph | cycle、missing/optional、type/version mismatch、100K 深/宽图 | typed edge、SCC/topology、immutable graph generation | 非法 candidate 不发布，查询无递归栈风险 |
| M3 · Async residency | single-flight、cancel、priority inversion、deadline、failure storm | coordinator 接入 worker、slot 与 version lease | 产品热路径无同步 decode/clone，失败有 backoff |
| M4 · Artifact/cook | stale URI、wrong platform/type/schema/toolchain、corrupt section | cook key、expected read descriptor、semantic section manifest | 任一 identity mismatch 在 publish 前拒绝 |
| M5 · Streaming | mip/LOD/cell range、budget pressure、cancel during decode/upload | range I/O、独立 codec、CPU/GPU admission/eviction | 请求单 section 不读取/解码整资产 |
| M6 · Reload/migration | candidate fail、旧 lease、missing subasset、历史 corpus | active/candidate publish、redirect、step migrator | 失败保留 last-good，修复不改变语义 target |
| M7 · Bundle/update | 100 GiB sparse read、overlay、并发 install、各 crash point、bad signature | runtime mount、signed bundle、journal/active generation | bounded RSS，可恢复原 generation，无未认证 code 关联 |
| M8 · Product closure | editor import → play → watch reload → cook/package → cold launch | 收敛 app/editor/render/plugin consumer | 不存在旧 clone/sync/probe/fallback API 调用 |

## 9. 验收矩阵

| 维度 | 必须覆盖的场景 | 关键断言 |
|---|---|---|
| Unit | exact type/schema、cook key、section hash、edge validation、migration step | deterministic；错误分类稳定；无隐式 fallback |
| Transaction | registry+artifact publish、active/candidate swap、bundle active pointer | 全成或全不成；old generation 始终可读 |
| Concurrency | 1/8/64 loader、同 key single-flight、cancel/reload/acquire race | 无重复 publish、死锁、版本撕裂和 waiter 泄漏 |
| Fault injection | open/read/decode/validate/write/fsync/rename/receipt 每阶段失败或 panic | last-good 保留；启动恢复确定；错误可查询 |
| Scale | 1K/100K/1M record，高 fan-out/depth，1/10/100 GiB bundle | 有界栈/RSS/锁等待；尾延迟满足预算 |
| Streaming | 单 mip/LOD/cell、快速视点移动、budget pressure、device upload failure | 只触及必要 section；可取消；fallback 可预测 |
| Migration/reference | 每个历史 schema、label rename/delete、GUID/path move、ambiguous candidate | semantic identity 不变；缺失不改指父资产 |
| Security/update | bad/expired/revoked key、manifest substitution、export-root mismatch、并发 installer | 未认证 generation/code 永不激活 |
| Product | editor import/reimport、runtime cold/warm load、hot reload、pack mount/update/rollback | 全链共享同一 identity/state/diagnostics |

性能记录必须包含 source fingerprint、target profile、存储设备、CPU/GPU、worker 数、cache state、raw samples 和 p50/p95/p99。至少测量：同步旧路径与 async 新路径的 frame stall；1 MiB/1 GiB 资产单 section/全量读取；100K 依赖 commit；1/10/100 GiB pack open/random read；clone 前后 CPU bytes/allocation/RSS；reload 期间 last-good 可用率。没有同条件数据不得声称“优于 Unreal/Unity”。

## 10. 既有计划纠正与 owner

1. `runtime/04-asset-pipeline-alignment.md` 继续是实现 owner。其 worker、watch、importer registry、artifact v4 和投影 generation 的已落地部分应保留；旧文档里关于无界请求、whole compressed Vec 或 importer ambiguity 的描述，实施前必须按 current source 复核。
2. Runtime04 对“handle 保持 Copy id、lease 显式存在”的方向只完成了一半：soft handle 可以 Copy，但 loaded payload 必须默认返回 versioned strong lease。当前约 50 个 product load 调用和零专用 acquire consumer 证明不能把 lease API 存在当作迁移完成。
3. Runtime04 M3 的 Reloading/Failed/last-good 目标仍开放；production watch 使用 `upsert_lazy` 并没有接入 `start_reload/fail_reload`，对应验收必须重开。
4. artifact semantic-section failure 已正确承认 v4 chunk 只是单 zstd frame 的物理切片；其 owner 与 render streaming 计划保持原处，本篇提供跨 asset/cook/mount 的上游 contract。
5. `frameworks/02/fixed-2026-07-14-stale-subasset-reference-repair.md` 的 missing-label → parent fallback 验收语义必须重开并撤销；GUID/path repair 不能改变 subasset identity。
6. durable file transaction successor 正由会话 `frameworks01-m1-durable-file-transaction-successor-r6-20260815` 修改。pack installer 应复用最终 canonical owner lock/journal/fsync/recovery，而不是在 pack 子树再造第二套；本文不改其工作区。
7. native plugin trust/ABI/rollback 的最终 owner 属于后续 Plugin/Host 编号计划；本篇拥有 artifact/bundle identity 和安装 generation，禁止两边复制同一 finding。

## 11. 工作区复核标记

本轮 resource/asset 尤其 `core/resource/io`、artifact、management/readiness generation 正与其他会话的大量未提交修改重叠。本文按 2026-08-15 current source 取证，状态为 `recheck_required`。实施 M0/M1/M4/M7 前必须重新读取 handle、record、ensure_resident、artifact manifest/store、resource commit、reference resolver、pack reader/writer/install 和 hot-update 调用链；只有对应失败测试、产品调用迁移和规模/故障验收同时通过，才能关闭 finding。新增 DTO、API 名称或独立单测数量不构成工程化完成证据。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|

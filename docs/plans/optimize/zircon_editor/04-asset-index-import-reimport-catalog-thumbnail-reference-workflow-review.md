---
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/core/plugin/extension_materialization.rs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/retained_host/app/assets
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/registry
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/EditorFramework/AssetImportData.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/AutoReimport/AutoReimportManager.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/ThumbnailRendering/ThumbnailManager.h
  - dev/godot/editor/file_system/editor_file_system.h
  - dev/godot/editor/import/editor_import_plugin.h
  - dev/Fyrox/editor/src/asset
  - dev/bevy/crates/bevy_asset/src/processor
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 04 · Asset Index、Import/Reimport、Catalog、Thumbnail 与 Reference Workflow 工程化差距

## 1. 结论

当前资产基础并非旧计划所描述的“尚无 registry / GUID / sidecar / thumbnail job”。Runtime 已有 `AssetRegistryIndex`、UUID/path/type/tag/dependency/referencer 查询、`.zmeta` v7、importer/config/source digest、targeted scan/import与资源发布；Editor 也已有可保留的类型注册表、代际 catalog、按键合并的有界事件 mailbox、每帧 drain 预算、preview job token和工程关闭时的代际失效。

真正的问题是这些基础没有收敛成一条产品资产管线。仓内同时存在三套资产视图：Runtime authoritative registry、完全没有production caller的 `EditorAssetIndex`、以及产品实际使用的 `DefaultEditorAssetManager` catalog/reference graph。与 `EditorAssetIndex` 配套的 `EditorAssetImportFlow` 也没有production caller；UI、animation、layout、scene与model工作流继续直接、同步调用 `AssetManager::import_asset/reimport_all`。因此编排层已有的generation coalescing、并发 admission、job状态、取消与reason收集并未保护真实用户操作。

即使单独看这套未接入的flow，成功结果也只清除 `Importing`，不会替换runtime registry、重新摄取 `.zmeta` 或使index进入 `Ready`；其测试明确断言成功后仍是 `Stale`。产品catalog则从Runtime `ProjectManager`重新复制一份完整source generation和editor DTO，再重建folder/details/reference graph。增量路径仍会克隆两张全量map，所有catalog构建都在调用 `refresh_from_runtime_project` 的线程上完成；background job只覆盖缩略图生成。

本切片没有新增能够独立证明“立即覆盖或删除用户源数据”的P0；相关save/delete/close数据完整性已归入02报告。本报告记录0个新增P0、30个P1、8个P2。没有运行Cargo、Editor、真实watcher风暴、百万资产catalog、importer crash、磁盘满、源控制、GPU thumbnail或跨版本reimport测试；性能判断只基于同步路径、复制次数和缺失预算，不宣称已经完成与Unreal/Unity的同机benchmark。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core asset orchestration clean production | 23 / 3,301 | E3：type registry、source authority、Editor index、import flow；fingerprint `608291d8...aeb91` |
| editor asset manager production | 44 / 4,168 | E3：catalog/source generation、reference graph、preview cache/scheduler、change stream；fingerprint `76a825ff...734e3` |
| product integration focused clean set | 15 / 2,900 | E3：project、scene、UI asset、animation、layout、model import、event refresh；fingerprint `0f7de5a1...51d3` |
| runtime registry/meta support focused set | 8 / 2,493 | E3：`.zmeta`、registry query、full/targeted scan/import；fingerprint `b29e679d...8fe9` |
| core index/import dedicated tests | 3 / 1,219 | E2：28个test attributes；未运行 |
| wider focused asset tests | 70 test attributes | E1-E2：包含type registry、index、flow与host契约；未运行 |

fingerprint按相对路径排序，将 `path + NUL + per-file SHA-256` 再计算SHA-256。它只标识本轮阅读集合，不是兼容ID，也不能代替构建和产品行为验证。

### 2.2 在途文件隔离

成文时 `zircon_editor/src/core/asset/dirty/error.rs`、`dirty/mod.rs` 及其两个测试文件有其他Session或用户修改。它们已由02 Document/Save报告拥有，本轮不把这些文件纳入clean fingerprint，也不复述其在途实现。

Runtime importer ingest中的若干texture/font/glTF/mesh/shader文件以及project asset manager的其他管理路径也处于在途修改；本报告不据此评价各格式的解码质量，只审查干净的Editor编排、catalog和它们调用到的stable registry/meta合同。实施前必须重新读取active owner，尤其是Runtime import transaction与watch event发布顺序。

### 2.3 本轮追踪的产品链

1. project open/save/scene create -> `AssetManager::import_asset` -> runtime project generation -> `EditorAssetManager::refresh_from_runtime_project` -> immutable catalog generation。
2. UI asset/animation/layout/model authoring -> source write/stage -> direct import/reimport -> asset/resource/editor change streams -> retained host refresh plan。
3. `AssetWatchEvent` -> `EditorAssetIndex::apply_watch_events` -> `EditorAssetImportFlow::submit` -> Editor job -> runtime import backend；该链只存在于core与测试，未进入产品。
4. runtime `ResourceRecord`/`.zmeta`/artifact -> editor `AssetCatalogRecord` -> direct-reference extraction -> `ReferenceGraph` -> folder/catalog/details DTO。
5. visible asset rows -> `PreviewScheduler` -> background thumbnail job -> PNG cache -> preview change mailbox -> paint-only refresh。
6. plugin contribution -> `AssetImporterDescriptor` / `AssetTypeContribution` -> extension snapshots与shell projection；没有连接到Runtime importer选择或真实Import动作。

## 3. 已有工程基础，重构时必须保留

### 3.1 Runtime registry与sidecar

- `AssetRegistryIndex` 已明确标记authoritative project registry，并维护UUID、path、asset id、source、dependency path和reverse referencer索引。
- query按稳定顺序返回type/filter/dependency/referencer结果，targeted更新能移除旧source entries并重建相关关系。
- `.zmeta` 已包含UUID、URL、asset kind、source unit、included files、artifact locator、importer id/version、raw import settings、config hash、source digest、schema migration、dependencies、tags和subassets。
- full/targeted scan会基于source/importer/config变化决定重导，并将Runtime resource publication与dependency closure纳入同一代际。

这些事实纠正Editor09/10中早期“metadata/index尚未存在”的描述。目标不是新建第四套database，而是把Runtime registry提升为Editor query/projection的唯一来源。

### 3.2 Editor type、source与toolkit primitive

- `AssetTypeRegistry` 校验字段owner、重复模板/命令、空字段和operation binding，并生成稳定creation menu generation。
- builtin和plugin asset type使用同一贡献模型；toolkit route保存project-stable locator，不长期保存机器本地path。
- `AssetSourceAuthority` 已区分project/package/builtin/library/derived/transient，并把 `ProjectOnly` 写权限限制到 `res://`。
- extension snapshot能按capability过滤asset importer/type contributions，shell有generation cache，避免每次绘制重新物化注册表。

### 3.3 Import flow并发primitive

- `EditorAssetImportFlow` 以 `(uuid, uri, source_digest)` 作为generation key，同代请求合并、不同代请求按UUID mutex group串行。
- admission默认限制4,096 flights、4 MiB估算保留量和5分钟oldest age；completed result会按容量/年龄回收。
- job panic被捕获，lease Drop会完成取消清理，取消在调用backend前后检查，重复reason按稳定集合合并。
- stale generation在admission前会重新校验，不会把旧URI/digest直接当作当前资产继续提交。

这些primitive本身可复用，但必须先接入产品并补上非阻塞提交与成功commit合同。

### 3.4 Catalog generation与事件背压

- Editor catalog使用immutable `Arc<EditorAssetCatalogGeneration>` 发布，catalog revision与publish epoch分离；单个preview更新可以共享其余Arc索引。
- project close会使source-sync epoch失效、发布更新的空catalog，并让旧preview token不能提交。
- editor change mailbox每订阅者最多512个key，重复key保留最新；溢出收敛为一次CatalogChanged，而不是无限保留每个asset事件。
- retained host每个stream最多drain 256项/600微秒，总体预留2毫秒，并记录pending count、queue age、drain time；32毫秒quiet period、250毫秒max deferral与4,096项accumulator构成可观测背压。

### 3.5 Preview generation safety

- preview任务按UUID mutex group串行，scheduler限制64个in-flight，并用job token、catalog revision、Arc row identity、source hash和meta path共同验证currentness。
- source image decode和placeholder生成在background EditorJob中执行；旧project或旧source完成不会覆盖新catalog。
- 完成、取消和submit失败均有admission release路径，`PreviewAdmissionAvailable` 能唤醒消费者重试可见项。

## 4. 权威分裂：当前数据流不是单一资产系统

| 层 | 当前owner | 自己持有的重复状态 | 产品是否使用 |
|---|---|---|---|
| Runtime | `ProjectManager` + `AssetRegistryIndex` + `ResourceRegistry` | UUID/path/type/tags/dependencies/referencers/import status | 是，真实import与resource publication |
| Core Editor index | `EditorAssetIndex` | runtime registry Arc、meta projection、dirty/importing UUID、pending paths、revision | 否，只在core/test和re-export出现 |
| Product Editor catalog | `DefaultEditorAssetManager` | ProjectManager clone、catalog maps、locator map、source generation、reference graph、folder/details DTO、preview state | 是，Asset Browser/Details/Preview使用 |

目标owner应是：Runtime asset database负责身份、source/import product、依赖和代际；Editor只持有按需求编译的immutable presentation/query generation，以及真正属于Editor的selection、viewport visibility、thumbnail queue和authoring session。`EditorAssetIndex` 与 `DefaultEditorAssetManager` 不能继续各自从sidecar/runtime数据推导一套状态机。

## 5. P1：产品化前必须闭合的架构与工作流

### P1-01 · 三套资产真值并存，Editor index与产品catalog互不相认

`EditorAssetIndex` 包装 `Arc<AssetRegistryIndex>`，另存meta、dirty/importing和pending path；`DefaultEditorAssetManager` 不使用它，而是从 `ProjectManager::registry()` 再建 `AssetCatalogRecord`、locator map、source generation和reference graph。任何修复若只更新其中一套，另一套仍会漂移。必须定义一个generation-bound `EditorAssetProjectionService`，直接消费Runtime registry delta；删除未接入的平行状态或把它迁移为该service内部的单一实现。

### P1-02 · `EditorAssetImportFlow` 在production没有任何caller

全仓production引用只有 `core/asset/mod.rs` re-export；所有submit/request/ticket行为只在模块测试中出现。真实UI直接调用 `AssetManager::import_asset/reimport_all`，因此coalescing、admission、job progress、panic containment、reason聚合和generation校验都只是未部署primitive。M0必须先做唯一 `AssetImportCoordinator` gateway，并禁止Editor产品层直接取得可写 `AssetManager` import入口。

### P1-03 · flow成功后不会提交新registry/meta，测试明确仍为Stale

backend返回 `Option<AssetStatusRecord>` 后，job只形成result并让lease清除 `Importing`。它不调用 `replace_runtime_registry`、不摄取新 `.zmeta`、不清dirty，也不发布catalog generation。`successful_import_uses_runtime_backend_and_clears_importing_state` 最终断言 `EditorAssetImportState::Stale`。成功合同必须返回 committed runtime generation/delta，由coordinator原子发布registry、resource和editor projection；若只完成CPU产物但未发布，应是 `ProducedAwaitingCommit`，不能叫成功。

### P1-04 · `status=None` 被当作成功完成

Runtime backend的签名允许“没有active project”时返回None。scene catalog caller会显式把None转为错误，但通用flow把它封装进成功result、清除Importing并进入completed cache。统一合同必须区分 `NoActiveProject`、`NoMatchingAsset`、`UpToDate`、`Imported`、`Failed`；不存在的project/asset不能依赖每个caller自行补判。

### P1-05 · 名为submit的API可以无限阻塞调用线程

UUID lifecycle处于Starting/Clearing时，`ImportFlowSharedState::reserve` 在Condvar上等待；合并到尚未发布admission的flight时，`wait_admission` 也在Condvar上等待。`EditorAssetImportTicket::wait` 再提供无deadline阻塞。真实Editor接入后，这些入口很容易卡UI线程。submit必须只做有界、非阻塞admission并立即返回Pending/Backpressured；等待只能发生在async worker或显式deadline API。

### P1-06 · 取消无法中断实际import，进度只有0/1与1/1

job只在同步 `backend.import` 前后检查cancel；解码、依赖扫描、artifact写入和registry commit期间无法响应。进度也只在开始/结束各报一次，无法表达scan、decode、derive、write、publish或当前文件。Runtime importer必须接受cooperative cancellation、phase/item/byte progress与deadline；不可取消的commit段需要短小、可恢复并明确显示。

### P1-07 · oldest-flight熔断没有hung job隔离与恢复

任一active flight超过5分钟后，新flight会收到 `OldestFlightAgeExceeded`，但系统不取消、隔离、重启worker或生成incident；一个卡死的第三方importer可使整个coordinator持续拒绝新工作。需要per-importer watchdog、process/worker isolation、quarantine、operator cancel和可持久化failure record，不能只把年龄当admission拒绝条件。

### P1-08 · 真实保存路径会静默丢弃导入错误

UI asset external effect、node operation、UI asset canonical save、animation save等路径使用 `let _ = import_asset(...)`；source写入已成功后，调用方继续刷新/同步或返回save成功。用户看到文档已保存，却不知道Runtime artifact/catalog可能仍旧。持久化成功后的import failure不应伪装成save失败，但必须形成 `SavedButProjectionFailed` 复合终态、可见diagnostic、retry action与dirty-derived状态。

### P1-09 · project save只记录post-persist错误，没有repair queue

`post_persist_project_save_sync` 正确避免把durable scene save翻回失败，但错误只写log后返回成功，没有在asset workspace标记out-of-date、创建repair job或阻止后续运行旧artifact。应保留“source已安全提交”的语义，同时将导入/目录/watcher阶段写入持久化reconciliation queue；重启后也能继续修复，而不是依赖用户查看日志。

### P1-10 · UI asset删除后用全量reimport，失败仍被忽略

`RemoveAssetSource` 先删除源文件，再直接 `reimport_all` 并丢弃结果。即使command未来可恢复source，Runtime registry、资源和Editor catalog在失败期间仍可能保留幽灵条目。删除/rename必须走source transaction：预检referencers与write authority，stage到trash，提交targeted registry delta，失败回滚source与registry；全量reimport不能作为删除协议。

### P1-11 · model import是多步部分提交，没有统一rollback manifest

产品先stage model source，再导root model，然后循环导derived animation assets，最后解析material/resource并向world添加mesh。中途任何derived import或world command失败，先前source和artifacts已留下，没有统一receipt说明哪些步骤完成、哪些可回滚。需要import transaction manifest、deterministic derived product set、all-or-nothing registry publication和独立的“导入后实例化”undo command。

### P1-12 · 产品导入调用仍是同步API，可能占用UI/host线程

scene create catalog、project save、layout save、asset editor、animation editor与retained host model/material路径直接执行Runtime import。即使Runtime内部部分工作并行，caller仍等待完整结果，且没有统一deadline。所有产品入口应提交后台coordinator job；只有短暂的source/registry commit允许在受控线程同步，UI通过generation/event观察完成。

### P1-13 · Watch与DigestMismatch只是枚举值，没有产品auto-reimport owner

flow定义Watch/DigestMismatch/Manual reason，`EditorAssetIndex`也能应用watch events和保留unknown path，但没有production wiring。真实产品监听AssetManager发布后的asset changes，这更像“导入完成后的通知”，不是“source变化 -> 去抖 -> 决定重导 -> commit”的Editor控制面。需要唯一watch owner处理self-write suppression、rename pairing、quiet period、source control、dirty open document冲突与import settings变化。

### P1-14 · plugin `AssetImporterDescriptor`只可查询，不能驱动真实导入

descriptor有id、display name、operation、extensions、output type、priority和capabilities；shell能按extension查询，但production没有caller，Runtime importer registry也不消费它。第三方插件注册的importer因此只能成为catalog metadata，不能接管文件识别、settings、执行或产物发布。必须定义Editor discovery descriptor到Runtime processor implementation的安全binding，包括version、schema、capability、sandbox和deterministic product contract。

### P1-15 · import settings虽然存在于`.zmeta`，Editor没有schema/默认值/验证/undo

Runtime sidecar使用裸 `toml::Table`，Editor type/importer descriptor都没有字段schema、条件可见性、范围、单位、preset、migration或validation callback。仓内没有产品Import Settings inspector/reimport action；dirty tests只把字符串effect id当作外部副作用。需要typed settings schema、stable field ids、default provenance、multi-edit、undo、config hash canonicalization和旧schema迁移。

### P1-16 · Asset type与Importer是两个没有一致性约束的注册表

`AssetTypeDefinition`描述runtime kind、写策略、presentation、thumbnail、toolkit、creation templates和context commands；Importer descriptor另存extension/output type/operation。两者没有约束某output type必须存在、runtime kind匹配、settings schema可用、thumbnail provider版本随import product失效。应发布一个编译后的Asset Class generation，统一source recognizer、processor、output schema、editor toolkit、preview provider和capability诊断。

### P1-17 · plugin catalog允许贡献批次部分成功

`apply_contributions` 会保留accepted contribution、收集rejected errors并递增generation；plugin `build_editor_extensions` 把错误转成diagnostic但仍返回部分物化的asset type catalog。另一条shell materialization路径却在首个错误时返回Err。相同插件在不同consumer可能呈现半安装状态。插件enable/disable与catalog发布必须是代际原子操作：要么完整贡献通过并一次发布，要么保留上一代并将新代标记Failed。

### P1-18 · catalog refresh在调用线程复制完整source generation

`EditorAssetProjectSourceGeneration::capture` 每次clone project manifest、package registry和全部 `ResourceRecord` 到HashMap；`delta_since` 再clone added/modified/removed/renamed记录并排序。即使最终unchanged，也已经支付O(N) clone/compare。Runtime应发布immutable registry generation与typed delta，Editor只保留Arc/sequence cursor；lag时请求一次snapshot，不自行比较整个database。

### P1-19 · “增量”catalog仍克隆两张全量map并全局重建projection

非metadata变化路径首先clone `catalog_by_uuid` 与 `uuid_by_locator`，patch少量记录后仍重建preview scheduler、reference graph、folder records、全部asset details和immutable indexes。成本仍接近O(N+E)，峰值同时保留旧新多份字符串/meta。需要persistent/structural-sharing maps、delta-aware folder/reference updates，以及按需details generation；增量不能只缩小读取artifact的集合。

### P1-20 · full projection为每个Ready asset加载artifact并手写提取引用

`project_catalog_record` 对每个root asset读取 `.zmeta`；Ready时还 `load_artifact_by_id`，再由Editor match `ImportedAsset` variants提取直接引用。大型project打开或metadata变化会把catalog discovery升级成payload I/O。依赖应来自Runtime registry的authoritative metadata，无需加载artifact；新增asset类型也不应要求Editor再维护一份handwritten extractor。

### P1-21 · Editor `ReferenceGraph`重复Runtime reverse index

Runtime `AssetRegistryIndex` 已维护dependencies、dependency paths和referencers；Editor仍从artifact引用重建outgoing/incoming HashMap<HashSet>。两套算法的覆盖范围不同，可能对同一资产给出不同referencer结果。Editor details必须查询同一registry generation；presentation层只缓存已排序view，不再拥有第二套依赖真值。

### P1-22 · reference解析以locator回退掩盖UUID冲突

`ReferenceGraph::rebuild` 与details projection先按UUID找，失败后按locator寻找已知资产，并把返回UUID替换成locator命中的当前UUID。迁移期这有可用性价值，但会把“稳定GUID已断裂、路径恰好被另一个资产占用”显示成有效引用。应区分ResolvedByUuid、StalePath、MissingUuid、PathOccupiedByDifferentUuid与LegacyPathOnly，只有显式migration/repair能改写引用。

### P1-23 · shader IDE环境写入被耦合在catalog发布临界链

full projection或shader delta会在持有source-sync gate后同步执行 `write_shader_ide_env_for_project`；失败会阻止新的Editor catalog提交，即使Runtime registry已经更新。IDE辅助文件属于derived side effect，应在catalog commit后单独排队、原子写入并可重试；不能决定Asset Browser是否看到最新代。

### P1-24 · preview provider没有真正的可执行渲染分派

`generate_preview_artifact` 只支持SourceImage缩放、CPU placeholder和Icon占位；`ThumbnailProviderDescriptor::Operation` 直接返回“must be dispatched”错误，但产品没有operation host接入。mesh/material/prefab/scene/font/audio等资产没有renderer-owned thumbnail scene、camera/light preset、waveform/glyph或type-specific provider。需要可注册provider接口、隔离的preview world/render queue和严格GPU/CPU预算。

### P1-25 · thumbnail cache key不足以表达provider与渲染配置版本

路径只由UUID和source hash构成。thumbnail provider实现、engine build、shader/material pipeline、color management、theme、preview scene、尺寸/quality变化都不会失效旧PNG。应使用content-addressed derived key：source/import product digest + provider id/version + renderer/schema/build/config fingerprint，并保存receipt用于诊断与回收。

### P1-26 · preview先写文件、后验证currentness，且写入非原子

任务先decode/render并直接保存最终PNG，再检查cancel、catalog revision、Arc row、source hash和最新meta。stale job虽然不会发布catalog，却已消耗I/O并可能留下无引用文件；进程崩溃可留下截断PNG。应写到job-scoped temp，currentness/receipt校验后atomic rename；失效任务删除temp，启动时scavenge孤儿与partial artifacts。

### P1-27 · preview状态同时存在sidecar和内存，但完成不提交sidecar

projection从 `.zmeta.preview_state` 初始化；job完成后加载meta、只把修改后的document放回 `AssetCatalogRecord`，没有save sidecar。进程重启后状态回到旧值，Error/Ready语义与磁盘不一致。必须二选一：若preview是纯derived cache，移出source sidecar并由receipt/database拥有；若sidecar是合同，就用generation CAS原子提交，不能只改内存副本。

### P1-28 · preview admission只有容量，没有队列、优先级和自动重试策略

`PreviewScheduler` 保存visible set，但只在caller逐UUID请求时尝试；64项满后返回None，依赖 `PreviewAdmissionAvailable` 触发外部再次遍历可见资产。没有viewport距离、selected/hovered优先级、fairness、dedup queue、retry/backoff或per-provider cost。失败时还设置dirty=false，不会自动重试。应让scheduler拥有有界priority queue和失败政策，UI只发布visibility/selection demand。

### P1-29 · event drain有预算，昂贵的catalog commit没有预算或后台阶段

每帧事件读取的600微秒/stream与2毫秒总体设计值得保留，但一旦accumulator提交且有asset changes，`refresh_from_runtime_project` 会在同一调用路径执行O(N) capture/delta和可能的全量projection/artifact I/O，然后才应用refresh plan。输入预算不能保护后续同步重建。需要后台prepare + 主线程常数时间generation swap，并记录prepare/commit耗时、bytes与lag。

### P1-30 · 缺少工程级批量导入、冲突与恢复控制面

当前没有统一Import/Reimport/Cancel/Retry/Show Log/Reveal Source/Reset Settings/Apply to Selection工作台，也没有source-control checkout、外部修改与未保存document冲突、worker crash恢复、bad importer quarantine、per-project import history或determinism verification。工程目标不能只暴露同步函数；需要可观察的Import Queue、typed incident、持久化receipt和headless commandlet使用同一coordinator。

## 6. P2：应在主架构收敛时一并清理

### P2-01 · `EditorAssetIndex::rows()`每次分配完整Vec

Runtime `entries()`先分配排序Vec，Editor再map collect另一Vec。若保留该API，应由immutable generation持有稳定有序slice或提供iterator/page cursor；不要让高频Asset Browser读取隐式O(N)分配。

### P2-02 · Editor DTO把typed身份重复转换为String

内部已有 `AssetUuid/AssetId/AssetUri/PathBuf`，public catalog/details又保存String副本，并额外构建String key HashMap。跨ABI/serialization边界可以有wire DTO，但进程内查询应保留typed compact ids和共享string arena，防止重复解析与内存膨胀。

### P2-03 · 单个preview更新仍复制完整assets/details Arc数组

`updated_asset` 为替换一个row重新collect整个assets和details slice。比深clone好，但高频thumbnail完成仍是O(N) Arc clone。可用chunked immutable vector、slot generation或独立preview-state table，让单项更新接近O(log N)/O(1)。

### P2-04 · 多个generation/token计数器使用wrapping或saturating

Editor index revision和import identities使用wrapping；catalog revision/publish epoch使用saturating。极端长生命周期会产生ABA或永远不再推进。统一使用nonzero epoch + checked rollover/restart generation，并在测试中覆盖边界，不能让不同子系统各自选择溢出语义。

### P2-05 · state读写锁poison多数用`expect`终止Editor

change stream与import state会恢复poisoned lock，`DefaultEditorAssetManager` 的大量RwLock读取/写入却 `expect("editor asset state lock poisoned")`。一次preview/provider panic若发生在持锁区，后续catalog读取可持续panic。应缩短临界区、禁止插件代码持锁执行，并将poison转换为可重建projection/incident。

### P2-06 · preview cache path接受未经约束的variant

thumbnail当前传source hash，但 `PreviewArtifactKey.variant` 是public String，`path_for` 直接拼到文件名。未来provider/插件可引入分隔符、保留名或超长路径。缓存key必须使用固定hex digest和独立display metadata，不允许自由文本进入物理路径。

### P2-07 · import reason只存去重集合，没有来源、次数与时间

Watch/DigestMismatch/Manual合并后只剩BTreeSet，无法说明哪次文件事件触发、多少次被coalesce、等待多久或哪个caller请求。应保留有界聚合统计：first/last timestamp、count、source watcher/caller、latest paths/settings generation，避免为了诊断保存无界事件列表。

### P2-08 · 多处source mtime/diagnostic使用弱类型默认值

missing mtime被投影成0，diagnostics降为message String，Preview/Import状态缺phase/error code/owner。应使用Option/typed diagnostic id/severity/remediation和monotonic timestamps；UI格式化放在presentation层，不以空串或0承载“未知”。

## 7. 目标架构：一条可提交、可恢复、可观察的资产管线

### 7.1 单一权威与代际发布

```text
Source Watch / Manual Request / Settings Commit
                    |
                    v
          Asset Import Coordinator
      recognize -> plan -> execute -> validate
                    |
           staged products + receipt
                    |
                    v
        Runtime Asset Database Commit
   registry + resources + dependencies + events
                    |
          immutable generation / delta
                    |
                    v
      Editor Presentation Projection
  folders + query pages + details + preview demand
```

禁止的结构是：Editor watcher更新一套index，Runtime importer更新另一套registry，Asset Browser再从ProjectManager重建第三套catalog。唯一commit owner必须给出generation、delta、receipt和terminal outcome；Editor只消费，不猜测。

### 7.2 Import job状态机

至少需要以下可观察状态：`Queued -> Recognizing -> WaitingForDependencies -> Processing -> StagingProducts -> Validating -> Committing -> Published`，以及 `CancelledBeforeCommit / FailedRecoverable / FailedQuarantined / Superseded / SavedSourceAwaitingRepair`。每个状态带phase progress、bytes/items、importer id/version、source/settings digest、deadline和incident id。

commit前所有产物写入隔离staging；commit以短事务替换artifact receipt、sidecar/asset database generation和resource projection。取消只允许在commit前；commit后失败进入reconciliation，不能回报一个模糊Err让caller猜测磁盘处于何种状态。

### 7.3 Query与presentation边界

- Runtime发布Arc registry generation和typed delta cursor；lagged consumer获取新snapshot。
- folder/type/tag/source/reference search由索引服务分页查询，不为每次UI刷新复制整个catalog。
- selected/visible rows才物化details；其余使用compact row record。
- reverse references直接使用Runtime authoritative graph，并保留resolved/stale/missing分类。
- Editor plugin只能提供schema、provider和operation，不直接修改内部map或sidecar。

### 7.4 Derived data与thumbnail

thumbnail、shader IDE文件和其他Editor cache均使用content-addressed key、receipt、atomic publish、disk byte budget、LRU/retention与startup scavenger。Preview scheduler拥有需求优先级和provider成本；renderer thumbnail使用隔离world、固定camera/light/color management，并能在headless验证相同输入产生允许范围内的结果。

## 8. 重构顺序

### M0 · 封闭产品旁路与错误终态

1. 建立 `AssetImportCoordinator` facade，替换Editor中所有直接 `import_asset/reimport_all` 写调用。
2. 让submit严格非阻塞；引入typed outcome与 `SavedSourceAwaitingRepair`，所有吞错路径发布可见incident/retry。
3. 连接watch/manual/settings/save/model/scene入口，保留现有generation key、mutex group与有界admission primitive。
4. 明确成功commit：Runtime generation/delta已发布后才进入Published；`None`不再代表成功。

### M1 · 合并三套资产视图

1. Runtime发布immutable registry generation与delta；Editor停止clone整个 `ProjectManager` source generation。
2. 将 `EditorAssetIndex` 与 `DefaultEditorAssetManager` 合并为一个projection owner，移除重复dirty/importing/reference真值。
3. 直接消费Runtime dependency/referencer索引；details按需物化，folder/reference使用delta更新。
4. 保留catalog Arc generation、close invalidation、bounded mailbox和frame drain budgets。

### M2 · 工程Importer与Settings协议

1. 编译统一Asset Class generation：recognizer、processor implementation、output schema、settings schema、toolkit、preview provider。
2. plugin contribution整批验证/整批发布；绑定runtime implementation id/version/capability/sandbox。
3. 实现Import Settings inspector、preset、multi-edit、undo、schema migration、config hash与reimport preview。
4. 引入process/worker isolation、watchdog、cancel、quarantine和determinism receipt。

### M3 · Derived Data与大工程性能

1. thumbnail/IDE/cache使用content-addressed receipt、atomic rename、disk budget与retention。
2. preview demand scheduler支持selected/hovered/visible priority、provider cost、retry/backoff和GPU queue budget。
3. catalog prepare在background完成，主线程只做generation swap；query分页、字符串intern和structural sharing。
4. 建立10万/100万资产、watch storm、mass reimport、cold/warm cache与memory ceiling基线。

### M4 · 产品工作台与headless一致性

1. Import Queue展示phase/progress/cancel/retry/log/incident/quarantine。
2. Asset Browser提供references、source/artifact receipt、settings、stale reason与repair action。
3. commandlet/CI和Editor使用同一coordinator与receipt schema，支持deterministic rebuild和cache miss解释。
4. source control、rename/delete、external modification和dirty document冲突进入统一transaction/prompt policy。

## 9. 验收门

1. production search中Editor产品层不再直接调用可写 `AssetManager::import_asset/reimport_all`；仅coordinator adapter可调用。
2. submit在blocked UUID/admission场景下于固定微秒预算内返回Pending/Backpressured，不等待Condvar。
3. 成功ticket必对应可查询的committed runtime generation；Editor index/catalog在同一delta后进入Ready。
4. no project、missing asset、up-to-date、imported、cancelled、superseded、failed具有不同typed outcome。
5. UI asset/animation/project save后导入失败，source保持durable、document终态明确、UI出现repair action，重启后仍能恢复。
6. model + derived animations在任一步故障时不会发布部分registry generation；staging可清理或继续。
7. watcher能合并save storm、配对rename、抑制self-write，并在dirty open document冲突时停在用户决策前。
8. plugin importer缺operation/runtime implementation/settings schema时整批拒绝，上一代catalog继续可用。
9. import settings修改可undo/redo，config hash稳定，旧schema能迁移，未知字段不会静默删除。
10. catalog单资产delta不会clone全部ResourceRecord、两张全量map或重建全部details/reference graph。
11. 10万资产catalog后台prepare期间UI frame p99保持目标预算，主线程commit为常数级swap。
12. reverse reference结果与Runtime registry同代一致；GUID断裂与locator fallback显示为不同诊断。
13. shader IDE文件生成失败不阻止Asset Browser看到最新catalog，并可独立重试。
14. mesh/material/scene/font/audio等preview走注册provider；Operation descriptor不再在产品路径直接报未分派。
15. stale/cancelled preview不会写最终cache文件；process-kill只留下可识别temp，重启scavenger会清理。
16. provider/version/color/scene配置变化会导致新derived key；磁盘cache受byte budget和retention约束。
17. preview满载时selected/hovered优先，滚动离屏任务可降级/取消，无饥饿且失败遵循backoff。
18. importer hang/crash不会阻塞整个Editor；watchdog终止worker、生成incident并允许其他importer继续。
19. commandlet与Editor对相同source/settings/importer版本产生相同receipt和dependency graph。
20. Windows长路径、Unicode、case collision、read-only/source-control、磁盘满、rename/delete race均有故障注入测试。

## 10. 参考源码校准

### Unreal Engine

- `IAssetRegistry` 同时提供异步/同步scan、modified-file rescan、priority、completion、batch events、dependencies/referencers；Zircon不应在Editor另建引用图来补Runtime registry。
- `UAssetImportData` 保存source file metadata，`UAutoReimportManager` 明确拥有monitored directories与self-change ignore，说明auto-reimport需要独立控制面而非把watch reason放进未接入enum。
- `UThumbnailManager` 支持按class注册renderer、shared thumbnail pool和dirty event；Zircon当前SourceImage/placeholder不足以承担复杂资产预览。

### Godot

- `EditorFileSystem` 同时跟踪scanning/importing、scan actions、sources changed、reimport file/group和filesystem change notification，source discovery与reimport状态由一个Editor owner协调。
- `EditorImportPlugin` 暴露import options、visibility、order、importer name和真正import入口；Zircon descriptor只有operation/extension，缺settings和execution contract。

### Fyrox

- Fyrox Editor asset模块把browser item、selection、dependency、creator、selector和preview cache组合成明确产品工作流；它是Editor UX与resource manager协作的参考，不应被用作Zircon三套database的理由。
- preview cache展示资源类型驱动的异步预览需求；Zircon可参考其产品闭环，但目标仍需更严格的generation、receipt、预算和GPU隔离。

### Bevy

- Bevy `AssetProcessor` 与meta明确区分Load/Process/Ignore、loader/process settings、processed hash/full hash和process dependencies，用于判断source或依赖是否变化。
- Bevy没有完整工程Editor Asset Browser，因此只校准processor/meta/cache invalidation primitive，不把缺少Editor控制面当作Zircon可省略该层的依据。

### Unity Graphics

- Graphics仓内 `AssetReimportUtils` 使用AssetDatabase批量editing scope、进度条和finally StopAssetEditing；具体LUT/importer代码展示postprocess与derived asset更新。
- 该仓不是Unity Editor AssetDatabase完整源码，本报告只把它作为SRP/package importer consumer证据，不推断闭源GUID database、worker isolation或thumbnail internals。

## 11. 对现有Editor09/10计划的纠正

- “需要建立typed AssetTypeRegistry、GUID sidecar、Runtime registry、thumbnail job”已部分完成，后续不得重复创建平行实现。
- M2不应继续把 `EditorAssetIndex + EditorAssetImportFlow` 当成已进入产品的资产管线；它们目前只有测试可达，成功后也不收敛为Ready。
- 产品实际Asset Browser数据源是 `DefaultEditorAssetManager`，它与Runtime registry重复projection/reference graph；必须先合并authority，再扩充UI。
- `.zmeta` 已有import settings/importer/config/source digest，但缺Editor schema与事务工作流；问题不是“没有字段”，而是“裸表没有工程控制面”。
- bounded mailbox、frame drain budget、catalog generation和preview currentness已经存在，应保留并扩展；性能重构不能退回无界channel或可变共享DTO。
- Editor10的reference management应直接建立在Runtime registry generation上，不再通过加载每个artifact和handwritten `ImportedAsset` match重建引用。

## 12. 未验证项与下一轮衔接

本轮没有验证具体texture/font/glTF/mesh/shader importer算法、artifact原子提交和Runtime watcher线程，因为相关实现有在途修改且Runtime资产纵向报告已覆盖其合同。实施M0前必须以当时working tree重新追踪Runtime import commit、AssetChange发布和resource generation顺序。

下一份Editor报告转向Inspector/property authoring：reflection schema、multi-object edit、component topology、custom drawer、validation、units/ranges、asset picker、transaction/preview与large-selection性能。Asset Import Settings inspector与本报告的schema/coordinator共享基础，不能再建立独立property系统。

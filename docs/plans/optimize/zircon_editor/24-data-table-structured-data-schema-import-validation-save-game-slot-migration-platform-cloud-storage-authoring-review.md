---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_runtime/src/asset/assets/data.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/platform/preferences
  - zircon_plugins/asset_importers/data
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Factories/DataTableFactory.cpp
  - dev/UnrealEngine/Engine/Source/Editor/DataTableEditor/Private/DataTableEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DataTable.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SaveGameSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SaveGameSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameFramework/SaveGame.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/AsyncActionHandleSaveGame.h
  - dev/godot/core/io/resource_saver.h
  - dev/godot/core/io/resource_saver.cpp
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/file_access.cpp
  - dev/godot/core/io/config_file.h
  - dev/godot/core/io/config_file.cpp
  - dev/godot/core/io/json.h
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_ecs/src/world/reflect.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 24 · Data Table / Structured Data / Schema / Import / Validation / SaveGame / Slot / Migration / Platform / Cloud Storage Authoring 工程化差距

## 1. 结论

Zircon已经拥有三组值得保留的底层能力，但目前没有形成Data Table或SaveGame产品。第一，`DataAsset`及builtin/optional importer能够把Text、TOML、JSON、YAML和XML导入为文本与`serde_json::Value`；第二，artifact store已有content addressing、chunk、Zstd、校验和原子提交；第三，DynamicScene session archive已有slot、metadata、validation、retention、preview、writer与原子文件路径，platform preferences也有带容量/权限错误的原子写入。这些都不是占位代码，不能因产品入口失真而整体重写。

然而当前Data Table与Save Data Workbench完全由固定字符串构成。`DT_Items`、`Schema_Item`、128 rows、512 refs、Potion_Health、AutoSave_01、SaveData v4、LZ4和Cloud Sync queued都写在ZUI、route或feedback中；字段提交只修改retained control，Save/Load/Validate没有document、provider、job、artifact或runtime acknowledgement。更严重的是，builtin registry只给`ResourceKind::Data`一个placeholder thumbnail，不提供factory/toolkit；`ImportedAsset::Data`又被reference analysis明确归入空引用。用户在界面看到的表格、schema、版本和引用数量在当前产品中没有可持久化对象。

本轮确认五项P0。其一，两份Workbench正在把fixture冒充产品事实。其二，通用`DataAsset`没有row schema、stable row key、field type、default、migration、reference/localization语义和typed runtime accessor，因此不是Data Table。其三，运行时没有面向player/profile/platform user的SaveGame service，两份按钮不能建立存档产品。其四，DynamicScene session archive虽然规模巨大，但没有产品caller，且`World::clone`/serde硬编码builtin component map，直接复用会静默遗漏plugin或任意typed component；它不能被接线后改名为SaveGame。其五，结构化数据导入没有source bytes、depth、node、alias、CPU或allocation预算，XML递归投影还丢失mixed-content顺序、namespace identity、comment和processing instruction，深输入可阻塞或击穿Editor。

参考源码说明了产品边界。Unreal `UDataTable`把RowStruct、RowMap、typed `FindRow<T>`、import policy、serialize/reimport与change delegate放在同一合同中，DataTable Editor围绕transaction、add/delete/rename、spreadsheet copy/paste和reimport工作；SaveGame则由platform service、platform user、slot、async save/load/delete、class/version header和用户可调用async action构成。Godot的ResourceSaver、`user://` FileAccess、ConfigFile和JSON提供可组合的资源/用户存储底座，但并不自动等价于完整SaveGame。Fyrox Visitor和Bevy scene/reflect同样是versioned serialization或world projection基础，不是玩家存档产品。本地Unity Graphics没有同级DataTable/SaveGame权威源码，本文不推测其闭源行为。

本轮登记5项P0、60项P1、12项P2和32个验收门。实施顺序必须先关闭静态假成功和无预算导入，冻结DataTable与SaveGame各自的schema/identity/ownership，再建立真实DataTable document/toolkit/compiler/runtime accessor和SaveGame platform service/migration/load transaction，最后接入cloud、cook、规模资格并删除重复authority。Runtime05继续拥有World/DynamicScene底层修复；本篇拥有Data authoring与SaveGame产品边界，不把session archive API膨胀重复登记为Editor实现。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes | 证据等级 |
|---|---:|---:|---|
| Editor Workbench、registry与routes | 21 / 6,205 / 295,470 | 7 | E3：两份ZUI、入口、field mutation、feedback、navigation、template binding、asset registry/reference analysis逐分支 |
| Data importer、model与artifact | 18 / 4,239 / 158,621 | 14 | E3：builtin/optional importer、DataAsset、cache payload、artifact store与load facade |
| Save/archive/platform anchors | 13 / 3,212 / 116,395 | 10 | E2/E3：World snapshot、level、archive artifact/writer/slot/manifest及preference atomic file focused handoff |
| focused tests | 5 / 862 / 30,219 | 21 | E3静态阅读：builtin Data、artifact material、session archive/single-slot和preference quota/failure |
| selected combined scope | 57 / 14,518 / 600,705 | 52 | 当前工作树fingerprint `a83a2e1843a81286955e5768fdb5f4b6767a3dfb0277197a3de28c1bc17e53d6`；0 ignored，3个在途文件 |

行数为物理文本行。fingerprint按相对路径排序，对每个选定文件计算SHA-256，再对`path<TAB>hash<LF>`清单计算SHA-256。范围内已有3个非本轮修改：`workbench_preview_actions.rs`、`render_asset_vfx.rs`和`runtime_state.rs`；本轮按当前工作树取证，不吸收、不回退。实施前必须重算fingerprint并复核route/binding终态。

DynamicScene session本体约565个Rust文件、9,399行、547个public function与367个unique public name，已由Runtime05完整登记。本轮没有把全部文件重复计入57文件focused scope，只复核SaveGame复用会经过的artifact、manifest、writer、slot和World snapshot锚点；这避免以重复统计制造扫描深度。

### 2.2 动态证据边界

本轮没有运行新的Cargo、Editor窗口、深层YAML/XML fuzz、DataTable大表、断电恢复、平台用户、主机quota、云冲突或真实游戏Save/Load测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断；相关源码没有改变到足以越过该编译门，本轮没有重复相同lane。52个test attribute是静态inventory，不是通过数；它们证明局部parser、artifact和atomic primitive行为，不能证明DataTable/SaveGame产品存在。

### 2.3 参考边界

- Unreal DataTable用于定义typed row schema、row identity、import/reimport、transaction与runtime lookup的产品下限，不要求复制UObject/UScriptStruct实现。
- Unreal SaveGame用于定义platform user/slot/async service、version/class header和user-facing action边界，不代表其默认实现自动满足所有cloud、encryption或跨版本迁移要求。
- Godot用于区分resource serialization、user-data file access和product SaveGame；其基础API是参考，不是降低Zircon产品门槛的理由。
- Fyrox Visitor用于判断版本化序列化、region和兼容读取基础；Bevy scene/reflect用于判断world projection/type registration基础。二者都不替代SaveGame参与策略、slot服务和迁移目录。
- 本地Unity Graphics只覆盖graphics package，不含完整Data Table或SaveGame源码。本文不以外部印象补齐不可验证功能。

## 3. 必须保留的真实基础

1. 保留`DataAsset`统一承载source URI、format、text和canonical value的基础角色，但把opaque document与typed DataTable artifact分层。
2. 保留builtin TOML/JSON/Text importer和optional YAML/XML plugin的注册、typed error与测试框架；收敛重复owner和输入预算，而不是重写plugin框架。
3. 保留artifact store的content addressing、chunk、Zstd、hash verification、2 GiB hard safety cap和atomic commit；为DataTable另加更严格的产品预算。
4. 保留asset manager generation/load facade、cache payload与import coordinator，DataTable compiler通过同一generation receipt接入。
5. 保留DynamicScene archive的sealed artifact、manifest、metadata/tag、slot、validation、preview、retention和atomic writer，作为可选SaveGame payload contributor或checkpoint基础。
6. 保留`RuntimeSessionArchiveWriter`的admission/worker方向，但收敛565文件的组合式facade并补cancel、deadline和terminal receipt。
7. 保留platform preferences `stage_atomic_write -> commit -> fsync parent`以及Denied/CapacityExceeded/CorruptBackend/TransientIo错误语义，抽取为platform storage primitive。
8. 保留focused parser/artifact/archive/preference tests作为回归基础，新增schema、migration、failure injection、platform/cloud和scale测试。
9. 保留Workbench稳定control/route identity；完成产品provider后令其只投影真实session，未完成时显示Unavailable。
10. 保留Runtime05对World/DynamicScene schema、snapshot和restore的所有权；SaveGame通过显式participant合同消费它，不复制或绕开底层修复。

## 4. 目标架构

```text
StructuredDataProduct
  -> DataAssetRepository(source_revision, import_generation)
  -> DataTableSchemaRegistry
       -> schema_id/version/fingerprint
       -> stable field ids, types, defaults, constraints, references, localization
       -> row key policy + migration graph
  -> DataTableDocument
       -> lossless source mapping + typed rows + unknown fields
       -> transaction/undo/clipboard/multi-edit/import diff
  -> DataTableCompiler
       -> validation diagnostics + dependency manifest
       -> immutable cooked column/row artifact + generation receipt
  -> RuntimeDataTableHandle<T>
       -> typed lookup/query/hot-reload snapshot

SaveGameProduct
  -> SaveGameSchemaRegistry + ParticipantRegistry
       -> stable type ids/versions/fingerprints/migrations
       -> capture/apply phase, ownership, unknown-data policy
  -> SaveGameCoordinator
       -> player/profile/platform user + slot identity
       -> async capture -> validate -> encode -> protect -> atomic commit
       -> async read -> verify -> migrate -> stage -> transactional apply
  -> PlatformSaveStorage
       -> enumerate/read/write/delete/quota/free-space/lifecycle deadlines
       -> local generations/backups + cloud etag/conflict/offline journal
  -> SaveGameAuthoringProvider
       -> schema/participant/migration/slot diagnostics
       -> no gameplay payload editing in the Editor Workbench
```

DataTable和SaveGame必须是两个独立产品边界。`asset_id + schema_id + schema_version + source_revision + import_generation + artifact_generation`贯穿DataTable；`game_id + platform_user_id + profile_id + slot_id + save_generation + schema_catalog_fingerprint + build_id`贯穿SaveGame。DynamicScene archive只能作为带版本与参与策略的payload contributor，不能成为整个SaveGame identity。

## 5. P0：先关闭假产品、静默丢数据与输入击穿

### P0-1：Data Table与Save Data Workbench把fixture和control mutation冒充产品状态

两份ZUI硬编码`DT_Items`、`Schema_Item`、Localization、Potion_Health、Sword_01、Armor_Heavy、Debug_Item、128 rows、2 warnings、512 refs，以及AutoSave_01、Manual_03、Cloud_02、SaveData v4、LZ4和Queued local sample only。`data_production.rs`、`runtime_state.rs`和generic `module_field_edit.rs`只写固定feedback或修改control的`value/value_text`，没有domain provider、document revision、job、artifact、storage result或runtime acknowledgement。现有route测试也没有证明业务状态来自产品owner。

这会令用户相信数据已保存、迁移已运行或cloud已排队，实际进程外没有任何变化。必须立即把所有成功文案改为可证明的provider state；产品未实现前按钮disabled并显示`Unavailable(reason, missing_capability)`。完成后Workbench只能投影真实DataTableDocument和SaveGameCoordinator，不允许继续保留fixture fallback。

### P0-2：Data Table资产与运行时合同不存在，静态行编辑没有可提交对象

`DataAsset`只有URI、五类format、完整文本和通用JSON value，没有row schema identity/version/fingerprint、stable row key、field type/default/constraint、reference/localization、migration或unknown-field保存合同。builtin asset registry只给`ResourceKind::Data` placeholder thumbnail，`builtin_toolkit`只识别三类UI asset；没有Data factory、template、document toolkit或compiler。全仓产品侧也没有typed table consumer，`load_data_asset`没有形成Gameplay/Editor query路径。

必须新增明确的`DataTableAsset`/`DataTableSchema`/cooked artifact，而不是在`serde_json::Value`上继续堆字符串约定。创建、打开、编辑、验证、保存、reimport、cook和runtime lookup共享stable schema/row/field identity。通用DataAsset继续服务opaque config/document；只有声明schema的输入才能进入DataTable Editor。

### P0-3：运行时没有SaveGame产品服务，Workbench的slot/schema/cloud声明均无authority

仓内没有面向player/profile/platform user的SaveGame public service、slot repository、participant registry、schema catalog、migration planner、autosave scheduler、load transaction或cloud provider。搜索到的“session archive”服务于DynamicScene session，platform preferences服务于小型key-value设置；两者都没有Gameplay SaveGame语义。Save/Load按钮只返回固定`schema v4`和local sample文案。

必须先冻结`ISaveGameService`、`IPlatformSaveStorage`、participant与schema/migration合同，再接UI。API至少覆盖enumerate/exists/read/write/delete/rename/copy、async cancellation/progress、player/profile/slot identity、quota、terminal receipt和typed error。没有service receipt时Editor或游戏UI不得报告保存、载入或云同步成功。

### P0-4：DynamicScene session archive不能直接改名或接线为SaveGame

Runtime05已确认session archive约565文件、547个public function，但除tests/structure/reexport外没有真实app/editor/runtime产品consumer。更关键的是，`World::clone`与serde硬编码builtin component maps，会静默遗漏任意plugin/typed component；`LevelSystem::snapshot()`及archive capture经过该路径。组件/资源又没有稳定per-type schema ID/version/fingerprint/migration chain，capture同步扫描并输出pretty JSON。直接将Save Data按钮接到archive会制造“成功保存但恢复后丢状态”的最高风险。

必须先修复Runtime05拥有的reflective snapshot/restore与type schema，再以`SaveGameParticipant`显式声明哪些world/entity/component state进入玩家存档。archive的artifact/writer/retention可复用，但其组合式public API要收敛为typed request；SaveGame必须另有player/slot/build/schema/protection/cloud envelope和transactional load。

### P0-5：结构化数据导入无产品预算，XML递归canonicalization有损且可击穿Editor

`AssetImportContext::source_text()`克隆全部source bytes，source loader使用`fs::read`，TOML/JSON/YAML/XML importer再构造完整DOM/value并把原始text与canonical JSON同时存入artifact。未找到Data输入bytes、depth、node、scalar、alias、CPU、allocation或diagnostic预算。XML helper递归遍历element，trim并汇总text、把attribute local name作为map key，导致mixed-content顺序、namespace区分、comment和processing instruction丢失；深树还可能栈溢出。2 GiB artifact hard cap发生得太晚，也远高于交互式authoring预算。

必须在读取前和parse/canonicalize阶段执行分层budget，所有parser采用深度/节点/别名/字符串限制、cancellation和structured diagnostic。XML若作为opaque Data必须保留原语义或声明受限映射；不能把有损JSON视图当可逆canonical source。DataTable导入应使用schema-aware streaming CSV/JSON等路径，拒绝超预算输入并保持旧generation有效。

## 6. P1：Data Schema、Identity 与 Runtime Contract

### P1-1：缺少稳定DataTable schema identity

定义project-scoped或package-scoped `schema_id`、semantic version、fingerprint和owner；重命名显示名不得改变identity，冲突与owner卸载必须产生typed diagnostic。

### P1-2：field只有JSON key，没有stable field identity

每个字段需要stable ID、name/alias、type、nullable/default、deprecated/replacement和source span；rename通过alias/migration处理，不能被当作delete+add。

### P1-3：row key策略没有合同

支持显式primary key、复合key或generated stable ID，规定case/Unicode/normalization、duplicate handling和rename redirect；显示顺序不能成为identity。

### P1-4：类型系统被压平到`serde_json::Value`

schema至少表达bool、signed/unsigned integer、float、decimal policy、string/name/text、enum/flags、vector/color、asset/entity/tag reference、array/map/optional/struct，并定义范围与转换规则。

### P1-5：default、nullable、missing和explicit null混为一谈

semantic model必须区分absent、defaulted、explicit null、invalid和unknown，import/export/patch/runtime lookup保持同一语义，避免版本升级时默认值被烘焙成旧值。

### P1-6：enum、tag和受控词表没有registry linkage

字段应引用versioned enum/tag registry，Editor提供合法值、deprecated值和migration，compiler拒绝不存在或owner已卸载的值。

### P1-7：Data reference没有typed locator和dependency identity

asset、row、localization、tag或其他table引用必须是typed reference，带target kind/schema与soft/hard policy；裸字符串只能作为源表示，不能成为runtime合同。

### P1-8：未知字段/行没有forward-compatible保存策略

Editor应保存unknown field/value/source span并显示read-only diagnostic；旧版本工具不得在普通Save时静默删除新版本数据。

### P1-9：schema evolution没有兼容性分类

建立additive、default-changing、rename、type widening/narrowing、key change和breaking分类，生成impact report与required migration，禁止只增加`version 12`字符串。

### P1-10：runtime没有typed、generation-safe DataTable handle

提供`DataTableHandle<T>`或等价schema-qualified接口，返回immutable generation snapshot、typed lookup error和hot-reload transition；不能让游戏系统反复查询通用JSON并自行cast。

## 7. P1：Data Table Editor、Document 与协作工作流

### P1-11：`ResourceKind::Data`没有真实toolkit或factory

建立builtin或first-party单一owner，提供Create Data Table、选择schema/template、stable asset ID、atomic write/import/open闭环；opaque Data则进入安全只读source viewer。

### P1-12：没有transactional table document

DataTableDocument持有base/source/artifact revision、dirty/history、selection、validation和import link；cell/row/schema mutation经Editor02 transaction与undo/redo，不直接改control文本。

### P1-13：缺少基本row增删改名与duplicate操作

实现add/delete/rename/duplicate，维护stable key、引用影响、selection和单事务undo；删除被引用row必须给出dependency decision而非静默成功。

### P1-14：缺少spreadsheet级copy/paste与范围编辑

支持typed rectangular selection、header mapping、clipboard MIME/version、parse preview、partial error与single undo group；跨schema粘贴必须显式映射。

### P1-15：缺少sort/filter/search与虚拟化owner

排序和过滤只改变projection，不改变source row order；大表用virtualized rows/columns、stable selection和cancelable query，不能按控件数量线性构建完整UI。

### P1-16：缺少multi-cell/bulk edit与公式化转换

批量set/clear/fill/replace/convert需要preview、affected count、validation delta和可撤销patch；表达式运行在有预算sandbox，不能执行任意脚本。

### P1-17：details面板没有typed property editor

按schema生成enum、reference、localization、numeric range、array/struct等编辑器，显示default/inherited/invalid/unknown状态并支持Reset。

### P1-18：schema修改与row data修改没有分离的决策流程

schema edit先生成兼容性/迁移preview，再以cross-document transaction更新数据、references和cook；普通cell编辑不能意外改变schema。

### P1-19：外部修改与多实例缺少revision conflict

Save必须compare base source revision；支持reload、three-way merge、save copy和显式force。多实例共享document owner或通过revision广播，不能last writer silently wins。

### P1-20：没有row-level diff、review与source-control语义

提供schema-aware diff，按stable row/field ID显示add/delete/rename/value change，支持large table summary和conflict resolution；不要只比较pretty JSON行。

## 8. P1：Import、Validation、Artifact 与 Reference Graph

### P1-21：产品表格没有CSV/TSV/spreadsheet导入导出

实现RFC/locale明确的CSV/TSV parser、encoding/delimiter/header/key policy、schema mapping preview和error rows；导出必须稳定、可重导且不丢类型。

### P1-22：builtin与optional plugin重复拥有TOML/JSON authority

当前plugin priority 100会在安装后覆盖builtin，导致相同文件的importer/error/canonicalization随插件可用性变化。必须有单一selection policy、implementation version和compatibility tests。

### P1-23：YAML alias/tag/merge语义缺少安全与兼容合同

声明是否支持alias、merge key、custom tag和duplicate key，限制alias expansion和depth；unsupported语义要带span拒绝，不能静默折叠。

### P1-24：XML canonical JSON没有namespace与mixed-content模型

若保留XML映射，必须保存qualified name、namespace URI、attribute identity、ordered text/element nodes与unsupported node诊断；否则把XML定位为opaque read-only Data。

### P1-25：TOML/JSON数字与时间类型转换可能丢语义

定义integer范围、float NaN/Inf、datetime、duplicate key、object order与precision policy；canonical artifact应使用typed value，不以JSON最小公分母吞掉源类型。

### P1-26：import source和canonical value双份常驻缺少memory accounting

大输入会同时持有bytes、String、parser DOM、JSON value和serialized artifact。建立phase peak预算、streaming/spill、source retention policy和telemetry，避免交互进程峰值放大。

### P1-27：validation没有规则registry、source span与增量执行

规则声明owner、schema/version、severity、affected path、fix action和cost；cell edit只重跑受影响规则，完整validation进入background job并带generation。

### P1-28：`ImportedAsset::Data`被reference analysis明确视为无引用

schema-aware compiler必须输出hard/soft asset、row、localization和tag dependency manifest，供rename/delete/cook/reimport使用；Workbench的512 refs只能来自该manifest。

### P1-29：Data artifact没有schema/fingerprint/dependency header

cooked artifact记录format version、schema ID/version/fingerprint、source/importer/compiler generation、dependency hashes、row count、index layout和endianness/target信息，load时严格验证。

### P1-30：没有面向runtime访问模式的布局与索引策略

按profile选择row-oriented、columnar、key hash/index、localized strip或chunked streaming，给出lookup/build/memory基线；不能把完整JSON tree直接作为最终运行时表示。

## 9. P1：SaveGame Schema、Participant 与 Migration

### P1-31：缺少SaveGame envelope identity

定义magic、format version、game/project ID、build/content/mod set、platform user/profile/slot、save generation、timestamp和schema catalog fingerprint；载入前即可做兼容判断。

### P1-32：缺少stable participant/type identity

Gameplay subsystem、world component、plugin和script state通过registry声明stable type ID、owner/version/fingerprint；Rust type name或当前module path不能充当持久identity。

### P1-33：缺少显式capture参与策略

默认不保存任意World。participant声明capture phase、scope、dependencies、required/optional、snapshot budget和privacy class，未知owner卸载时执行明确策略。

### P1-34：缺少分阶段restore与事务边界

Load必须先read/verify/migrate到staging，建立或切换目标world，在dependency order中apply，全部成功后才publish；失败回滚到原world/last-good state。

### P1-35：migration只存在静态`v3 to v4`文案

建立per-type和envelope migration graph，验证无缺边/歧义/cycle，支持dry run、成本预算、backup和typed report；迁移函数必须deterministic且可测试历史fixture。

### P1-36：unknown participant data没有保留与降级策略

对missing optional plugin/mod可选择opaque preserve、disable-load或drop-with-consent；required gameplay state缺失必须拒绝，不能当作空默认继续游戏。

### P1-37：build、DLC、mod和plugin兼容性没有catalog

envelope记录内容集合及版本，load planner区分compatible、migratable、missing optional和incompatible；Editor提供诊断而不是只有一个schema下拉框。

### P1-38：脚本状态没有稳定schema与安全边界

脚本module声明save schema、field IDs、migration和capability；禁止任意VM heap/closure/native handle自动序列化，恢复代码运行在受限phase。

### P1-39：引用对象、实体和资源的重绑定规则未定义

区分persistent entity ID、session entity、asset ID、soft locator和external account ID，载入时分阶段resolve并报告dangling references，禁止直接保存临时handle。

### P1-40：determinism、random state和time state没有保存策略

明确simulation tick、RNG stream、timer/cooldown、clock/domain和pending deterministic commands的capture/apply顺序；否则恢复后立即产生不可复现分歧。

## 10. P1：Platform Storage、Lifecycle、Protection 与 Cloud

### P1-41：没有platform user、profile与controller mapping

存档请求使用稳定PlatformUserId/ProfileId，不由Editor固定`UserIndex 0`或路径猜测；登录切换、guest、sign-out和多controller状态进入service生命周期。

### P1-42：slot identity、显示metadata与物理路径混在一起

slot用opaque ID，display name、playtime、map、difficulty、thumbnail、created/updated time和manual/autosave类别进入versioned metadata；平台路径由storage backend决定。

### P1-43：API缺少完整async/cancel/progress/terminal receipt

capture、encode、write、cloud和load/apply各有阶段进度、deadline、cancellation acknowledgement与exactly-one terminal state；主线程不做阻塞I/O或完整world serialization。

### P1-44：autosave缺少scheduler、coalescing和checkpoint policy

按game state/lifecycle触发、debounce/coalesce、限制并发与写放大，保留最近成功generation；退出/suspend deadline到期时返回明确未完成状态。

### P1-45：quota、free-space和平台限制没有产品前置检查

在capture/encode前查询slot/count/bytes/filename/atomic replace能力，估算最终与temporary空间；CapacityExceeded提供可恢复动作，不把偏好存储的4096路径cache当存档quota。

### P1-46：crash-consistency、backup和last-known-good策略不完整

复用atomic primitive，增加双generation或journal、fsync policy、校验后promote、stale temp recovery和bounded backup retention；故障注入覆盖每个write/flush/rename点。

### P1-47：compression只是界面中的固定LZ4字符串

envelope声明algorithm/version/dictionary/chunk和uncompressed size，先执行bomb/ratio预算再解压；按平台/数据profile选择，不在UI中伪造固定算法。

### P1-48：encryption、authentication、tamper和privacy policy缺失

区分机密性、完整性、反作弊与用户隐私目标，key来自platform secure storage或账户服务，记录algorithm/key epoch；禁止自制crypto或把checksum称为防篡改。

### P1-49：cloud sync没有etag、generation和冲突模型

provider以base etag/local generation做conditional upload，支持offline journal、retry/backoff、duplicate suppression和multi-device conflict；绝不能把`Queued local sample only`算作cloud成功。

### P1-50：cloud冲突没有可解释merge/choose流程

默认保留local/remote双副本及metadata，按slot策略选择newest、manual decision或schema-aware merge；Gameplay payload通常不能逐JSON字段盲合并。

## 11. P1：Authority、性能、诊断、测试与产品资格

### P1-51：DataTable与SaveGame共用一个“Data”外观会混淆ownership

asset type、runtime service和Workbench分成Structured Data/Data Table/SaveGame Diagnostics，使用不同capability、document和权限；SaveGame槽不是project asset。

### P1-52：Static Workbench与未来真实toolkit存在双入口风险

Data Table按钮应打开canonical asset toolkit，Save Data应打开service diagnostic/slot inspector；extension页面只能是同一provider的projection，不复制document或fixture。

### P1-53：route/feedback没有request ID、generation或provenance

所有操作返回typed receipt，包含request/job/document/storage generation、actor、origin、status和diagnostic IDs；过期结果不能覆盖新选择或新slot。

### P1-54：Data import/cook和Save capture缺少resource admission

接入Editor09/runtime task authority，声明CPU、memory、I/O、temporary disk和platform exclusivity；大表、world capture、compression与cloud不能无限并发。

### P1-55：大表规模没有性能资格

建立10K/100K/1M rows按目标产品分层的open/filter/edit/validate/save/cook/lookup时间和peak memory预算，超档进入streaming/read-only模式并明确展示。

### P1-56：大存档规模没有帧预算与内存资格

capture分片或copy-on-write，限制单帧工作、peak bytes、compression latency和temporary disk；在实际游戏负载下证明autosave不造成可感知卡顿。

### P1-57：diagnostic缺少稳定code、path和修复动作

Data使用schema/row/field/source span，Save使用slot/participant/type/migration/storage stage；severity、retryability、user action和privacy redaction由统一registry定义。

### P1-58：测试主要证明parser或静态route，不证明产品闭环

新增create-open-edit-undo-save-reimport-cook-runtime lookup，以及capture-write-enumerate-read-migrate-apply-delete端到端测试；静态反馈断言不能作为feature测试。

### P1-59：缺少历史fixture与跨版本兼容矩阵

每个released schema/importer/save format保留golden fixture，当前版本必须读取或明确拒绝；migration CI覆盖N-2/N-1、missing plugin、platform endian/locale和corrupt/truncated输入。

### P1-60：manifest maturity与界面承诺没有资格门

`experimental/stable/complete`只能由required tests、budgets、platform matrix和product caller证据驱动。Data/Save surface未通过门禁前必须显示Experimental/Unavailable，不得硬编码成功。

## 12. P2：完整性、诊断与维护性

### P2-1：Data格式与DataTable产品术语混用

文档和API区分opaque structured document、schema-bound table、runtime config和SaveGame payload，避免所有JSON/TOML都被称为Data Table。

### P2-2：schema/row/field/slot ID仍可能散落为裸字符串

集中typed ID、parse/validation和display conversion，减少route、manifest、artifact与runtime之间的拼写漂移。

### P2-3：importer capability与priority缺少生成清单

构建时生成extension/format/priority/implementation version矩阵，并检查duplicate authority和平台差异。

### P2-4：artifact中source text保留策略缺少可配置性

区分Editor source cache、diagnostic excerpt和runtime cooked payload；release artifact不默认携带完整原文或隐私字段。

### P2-5：validation/diagnostic列表缺少统一虚拟化与分页

大表错误和存档participant报告使用bounded journal、query、group和pagination，不在UI构建全部row。

### P2-6：save slot thumbnail可能引入隐式render依赖

thumbnail capture独立job、明确generation/尺寸/格式/失败降级，不阻塞核心save commit，也不令无图槽失败。

### P2-7：路径、文件名和显示名的Unicode/case策略未集中

platform backend负责合法化和冲突检测，stable slot/schema identity不依赖host path normalization。

### P2-8：时间戳不能作为唯一冲突排序依据

使用monotonic generation/etag与设备identity，wall clock只用于展示；处理时钟漂移和离线设备。

### P2-9：telemetry缺少内容与隐私边界

只记录size、latency、stage、code和匿名schema/participant ID；row values、player state、paths和cloud tokens默认不进入日志。

### P2-10：archive facade的组合爆炸提高维护成本

Runtime05负责把path/loaded/source/named/selected/metadata/retention组合收敛为request structs和少量service方法，SaveGame不再复制一层组合API。

### P2-11：文档与source之间没有自动漂移检查

用结构测试验证报告引用路径、Workbench fixture、public SaveGame symbols和Data toolkit状态；产品变化后提示重新审查P0而不是让报告静默过时。

### P2-12：参考引擎能力容易被错误等同

文档持续标明serialization foundation、resource storage、DataTable和SaveGame product的不同层级；不以“Bevy/Fyrox能序列化场景”证明Zircon已有玩家存档。

## 13. 当前静态第二Authority清单

| Surface / handler | 固定或伪业务状态 | 当前真实效果 | 目标owner |
|---|---|---|---|
| Data Table Workspace | `DT_Items`、`Schema_Item`、Localization | ZUI常量 | DataTableDocument |
| Data Table rows | Potion/Sword/Armor/Debug Item与数值 | ZUI常量 | virtualized row provider |
| Data validation | 128 rows、2 warnings、512 refs | feedback常量 | validator + dependency manifest |
| Data details | Potion_Health、Gameplay Item、version 12 | control value mutation | typed row/schema transaction |
| Save Data Workspace | AutoSave_01、Manual_03、Cloud_02 | ZUI常量 | SaveGame slot provider |
| Save sections | PlayerState、Inventory、QuestLog、DebugSlot | ZUI常量 | participant/schema report |
| Save validation | 6 slots、2 migrations、1 warning | feedback常量 | migration planner/storage query |
| Save schema/compression | SaveData v4/v3/Legacy、LZ4 | control value mutation | envelope/migration/storage policy |
| Cloud state | `Queued local sample only` | 固定字符串 | cloud provider receipt |
| Open/Save/Load/Validate routes | 固定opened/queued/success文案 | 不触发domain operation | typed request/job terminal receipt |

任何保留上述值作为demo的需求都必须移到独立sample project或test fixture，不得留在默认Editor产品面。开发模式也不能在真实按钮失败时自动回退fixture。

## 14. 分层重构里程碑

### M0：冻结真实性、输入预算与基线

把两份Workbench切为Unavailable/real-provider-only；冻结57文件fingerprint、P0复现、parser corpus、artifact/archive primitive和历史fixture。为Data source设bytes/depth/node/alias/time/memory预算并先封堵XML递归击穿。

### M1：Data Schema与Artifact合同

建立stable schema/row/field/reference IDs、typed value、unknown preservation、compatibility classification、migration registry和versioned cooked artifact header。收敛builtin/plugin重复importer authority。

### M2：DataTable Document与Factory/Toolkit

实现create/open、transaction/undo、row/cell/schema editing、selection、virtualization、clipboard、source conflict和atomic Save/Reimport。opaque Data保持安全source viewer。

### M3：Import、Validation、Reference与Cook

实现CSV/TSV及schema mapping preview、bounded parser、incremental validator、dependency manifest、cook job和generation receipt；接入rename/delete/package/cook链。

### M4：Runtime Typed Data Access

实现immutable DataTable generation、typed handle/lookup/index、hot reload和memory/latency profiling；选定Gameplay系统作为首个真实consumer并删除JSON自行cast路径。

### M5：SaveGame Envelope、Schema与Participant

冻结service API、stable participant/type IDs、capture/apply phases、migration graph、unknown policy和build/mod catalog。Runtime05先修复World arbitrary component snapshot/restore。

### M6：Platform Storage与Transactional Save/Load

实现platform user/profile/slot、async enumerate/read/write/delete、quota、atomic generations/backups、capture/encode/protect pipeline和staged transactional load/rollback。

### M7：Lifecycle、Autosave、Protection与Cloud

接入suspend/quit deadline、autosave coalescing、compression envelope、secure key/protection policy、cloud etag/offline journal/conflict resolution和privacy redaction。

### M8：Editor Diagnostics与产品接线

Data Workbench只打开canonical toolkit；Save Workbench只投影真实schema/participant/slot/migration/storage/cloud provider。所有action展示request generation与terminal receipt。

### M9：规模、故障、跨版本与Authority硬收敛

通过32门、历史fixture、parser fuzz、断电/磁盘满/权限/cloud race、百万行与大存档资格；删除fixture/fixed success、重复importer和无caller组合facade，更新maturity声明。

## 15. 验收门禁

1. Data Table可从默认产品创建、打开、编辑、保存、关闭、重开，row/field identity和typed values不变。
2. Opaque Data与schema-bound DataTable明确区分，未知格式不会被误开为可编辑表格。
3. schema/row/field rename通过stable ID与migration保持引用，不产生delete+add数据损失。
4. missing、default、explicit null、invalid和unknown在import、Editor、cook、runtime中语义一致。
5. add/delete/rename/duplicate、multi-cell edit和clipboard均为单一可撤销transaction。
6. 外部修改和多实例Save通过base revision检测，普通Save不能覆盖新磁盘revision。
7. CSV/TSV/TOML/JSON/YAML/XML对声明的语义有明确支持/拒绝矩阵和source-span diagnostic。
8. 深层XML、YAML alias bomb、超大scalar和超预算source被有界拒绝，旧artifact保持可用。
9. import phase peak memory、CPU和temporary bytes满足设定预算，cancellation可终止未提交generation。
10. builtin/optional importer在同一输入上有稳定selection与implementation version，安装插件不静默改变语义。
11. validator支持增量与完整模式，diagnostic稳定指向schema/row/field/source span。
12. Data reference manifest驱动rename/delete/cook，界面引用数与manifest一致。
13. cooked DataTable含schema/importer/compiler/dependency generation并拒绝不兼容load。
14. typed runtime lookup没有通用JSON cast，hot reload以immutable generation切换。
15. 目标大表规模下open/filter/edit/validate/save/cook/lookup与peak memory全部达标。
16. SaveGame service按platform user/profile/slot完成enumerate/exists/write/read/delete并返回typed terminal receipt。
17. Save envelope在读取payload前验证magic、format、game/build/catalog、sizes与protection metadata。
18. 每个participant拥有stable type ID/version/fingerprint、capture/apply phase和required/optional policy。
19. 任意plugin/typed component不会因`World::clone`硬编码而在成功存档中静默遗漏。
20. per-type/envelope migration覆盖历史fixture，缺边、歧义、cycle和超预算均在apply前拒绝。
21. unknown optional participant可按策略opaque preserve，missing required participant阻止load。
22. Load在staging完成verify/migrate/resolve，apply失败回滚并保留原world/last-good slot。
23. persistent entity/asset/soft reference重绑定有typed report，不恢复临时process handle。
24. autosave不阻塞目标帧预算，重复请求coalesce且始终保留最近成功generation。
25. 磁盘满、权限拒绝、写/flush/rename任一点失败都不损坏last-known-good slot。
26. quota/free-space/temporary space在capture前后校验，失败提供可恢复动作和准确stage。
27. compression有algorithm/version/uncompressed-size与bomb预算，界面不再固定宣称LZ4。
28. encryption/authentication使用平台或审计过的provider/key lifecycle，日志不泄漏payload/key/token。
29. cloud upload使用etag/base generation，离线重试幂等，多设备冲突保留双方且不会last-writer-wins静默覆盖。
30. suspend/sign-out/quit deadline下每个请求都有completed/cancelled/deferred/failed终态，不留下假成功UI。
31. DataTable与SaveGame端到端、历史兼容、corruption、fuzz、failure injection和scale lanes成为required CI evidence。
32. 默认Workbench不含任何固定业务计数、row/slot、migration、compression、cloud或success fallback；所有显示值可追溯到provider generation。

## 16. 禁止的临时修补

- 禁止只给`ResourceKind::Data`挂一个JSON文本编辑器就宣称Data Table完成。
- 禁止把schema、row key、field type、version继续藏在约定JSON key中而无stable identity。
- 禁止把CSV一次性split(',')，必须使用符合quoted/newline/encoding策略的parser。
- 禁止通过调高2 GiB artifact cap解决交互式import预算；cap必须在读取和parse前生效。
- 禁止继续把有损XML-to-JSON结果称为canonical或可逆表示。
- 禁止仅把Save Data route接到DynamicScene archive并重命名为SaveGame。
- 禁止默认序列化整个World、VM heap或plugin state；参与者必须显式注册并有版本。
- 禁止以Rust type name、`TypeId`、module path或临时entity handle作为持久schema identity。
- 禁止同步在游戏/UI线程capture全world、压缩、写盘或云上传。
- 禁止先删除旧slot再写新slot；必须提交新generation后再更新current pointer。
- 禁止用checksum冒充encryption/tamper protection，或自行设计未审计crypto。
- 禁止用wall-clock newest作为唯一cloud冲突规则，也禁止静默last writer wins。
- 禁止在storage未返回terminal receipt前显示Save/Load/Cloud成功。
- 禁止保留静态fixture作为真实provider失败时的fallback。
- 禁止以新增静态route断言替代create-edit-cook-runtime和capture-write-load-apply闭环测试。

## 17. 本轮产出边界

本轮只新增审查文档与覆盖索引，不修改production Editor/runtime/plugin源码或tests，不创建DataTable/SaveGame临时实现。报告基于57个focused文件的E3静态阅读、Runtime05既有565文件session archive证据和本地参考源码；选定范围source fingerprint为`a83a2e1843a81286955e5768fdb5f4b6767a3dfb0277197a3de28c1bc17e53d6`，其中3个文件在途，实施前必须重取。

动态验证仍受既有`zircon_editor --lib`编译失败阻断：617.2秒后239个错误、122个warning。本轮没有重复不能抵达产品行为的相同lane，也没有声称任何动态测试通过。后续实施必须按M0-M9依赖顺序推进，先交付真实schema/transaction/storage边界，再允许Data Table或Save Data surface恢复可执行状态。

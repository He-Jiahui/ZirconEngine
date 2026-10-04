---
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_runtime/src/asset/mutation
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/relocation.rs
  - zircon_runtime/src/asset/registry/query.rs
  - zircon_runtime/src/asset/registry/relocation.rs
base_reports:
  - docs/plans/performance/01/2026-08-16-editor-core-asset-save-registry-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetTools.cpp
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetRenameManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/ObjectTools.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
doc_type: implementation-evidence
status: static_current_m0_complete_structural_cutover_required_dynamic_blocked
---

# Editor core Asset currentness、refactor审查与builtin base M0（2026-08-23）

## 当前冻结

- `zircon_editor/src/core/asset/**`：**38/38 Rust文件、8,273 physical lines、269,504 bytes、64 tests**。
- ordered workspace-relative path + NUL + raw bytes + NUL SHA256：
  `70e951c8efef89a2cc612a145a3fb9e842c100e791b4489995867c6337e12387`。
- 8月16日35文件报告已逐文件复核；`7a20f921b`把当时已审的Save All job adapter等工作区内容
  写入历史，`08094b9b9`只改变四个dirty文件的格式。当前新增3个`refactor/**`文件和root export
  已完整读取，并沿UI manager、Runtime mutation/registry/relocation生产链复核。
- 用户并行拥有`core/asset/mod.rs`、`core/asset/refactor/**`及大量Runtime asset mutation/relocation
  变化，本轮不修改它们。本轮唯一源码M0位于原本clean的`type_registry/builtin.rs`。
- 没有current-source可执行文件，managed Cargo会话不可执行。本文不声明CPU、latency、RSS、power、
  throughput、算法最优或引擎数值对齐；Asset仍留在动态未验收范围。

## 38/38逐文件复核

| 文件 | current-source结果 |
|---|---|
| `dirty/error.rs` | typed错误；本轮仅rustfmt。 |
| `dirty/external_effect_id.rs` | construction-time canonicalization和borrowed lookup；无新热点。 |
| `dirty/mod.rs` | 导出dirty/save batch/job adapter；本轮仅格式/公开面变化。 |
| `dirty/registry.rs` | 4,096 journal与generation token应保留；reset caller仍clone完整map，per-effect clear仍逐次锁/发布。 |
| `dirty/save_batch.rs` | 完整preflight后持有全batch owned intents；apply按generation安全，但仍完整sort/copy/materialize。 |
| `dirty/save_batch/tests.rs` | 4项覆盖partial/stale/cancel；无大payload/RSS/UI wait门。 |
| `dirty/save_job_adapter.rs` | 原子reserve与64-ticket poll预算正确；仍一次materialize全部intent/job/ticket，pending estimate不约束running serialized/result bytes，且无产品caller。 |
| `dirty/save_job_adapter/tests.rs` | 8项覆盖admission、mutex、failure、cancel、poll budget；fixture payload很小且busy-yield等待。 |
| `dirty/tests.rs` | 16项覆盖10K journal/cursor/race；缺clone bytes、lock p95与一次document commit计数。 |
| `import_flow/error.rs` | typed submit错误，无独立热点。 |
| `import_flow/flight.rs` | exact generation single-flight正确；Condvar admission/result仍可阻塞caller，terminal结果/status被observer复制。 |
| `import_flow/job.rs` | 共享job system与panic cleanup正确；每job仍格式化URI progress。 |
| `import_flow/mod.rs` | entry/estimated-byte/age限制存在；public blocking wait仍是UI集成风险。 |
| `import_flow/state.rs` | generation/UUID串行化正确；mutex identity格式化并扫描active UUID，timestamp bucket vector removal为线性。 |
| `import_flow/submit.rs` | 避免index/state/backend重叠锁；observer和UUID transition仍同步等待，request被clone。 |
| `import_flow/tests.rs` | 11项覆盖retry/duplicate/progress/panic/shutdown；无UI submit latency与distinct-UUID scale门。 |
| `import_flow/tests/concurrency.rs` | 5项覆盖admission/lifecycle races；测试本身使用blocking handoff。 |
| `index.rs` | 借用Runtime registry record正确；`rows()`仍全collect+path sort，unknown watch side state可增长，不能成为第二Browser truth。 |
| `index/tests.rs` | 12项覆盖projection/watch delta；无stable-query allocation和unknown storm预算。 |
| `mod.rs` | 当前新增refactor与save DTO exports；不执行算法。 |
| `refactor/delete.rs` | 新增Runtime topology投影；在Editor state读锁内把target和全部referencers再clone一次，缺generation token/bytes/page。 |
| `refactor/mod.rs` | 新refactor facade；明确filesystem mutation留给Runtime，边界方向正确。 |
| `refactor/tests.rs` | 5项覆盖missing/read-only/subasset/referencer order；无1M referencer、clone/sort/allocation/lock budget。 |
| `source_authority.rs` | locator分类线性于输入长度，提供写策略；无frame rebuild。 |
| `toolkit_route.rs` | 小型borrowed route DTO；无独立热点。 |
| `type_registry/asset_type_id.rs` | construction-time验证和borrowed string lookup；无query owned-ID。 |
| `type_registry/builtin.rs` | 本轮缓存一次已验证26-type base；后续`with_builtins`不再重建/校验26个contribution，但仍deep-clone独立registry。 |
| `type_registry/context_command.rs` | construction-time normalize；owned payload计入generation即可。 |
| `type_registry/contribution.rs` | owned delta；上层candidate validation仍clone/reapply完整existing+candidate集合。 |
| `type_registry/creation_template.rs` | construction-time sort/dedup；default document bytes必须进入retained预算。 |
| `type_registry/definition.rs` | materialized owned definition，borrowed query；无stable-frame rebuild。 |
| `type_registry/error.rs` | diagnostics only。 |
| `type_registry/mod.rs` | declarations/re-exports only。 |
| `type_registry/presentation.rs` | compact validated DTO；无独立热点。 |
| `type_registry/registry.rs` | batch publish和Arc menu generation正确；mutable registry clone仍复制26个definition/owner maps。 |
| `type_registry/registry/batch.rs` | touched collection各sort一次；owner strings仍复制，extension candidate仍应一次batch而非逐entry重放。 |
| `type_registry/thumbnail_provider.rs` | descriptor/palette only；无decode/upload。 |
| `type_registry/toolkit.rs` | construction-time descriptor；无独立热点。 |

## 当前P0/P1结构瓶颈

### P0：Asset仍有Runtime与Editor两份完整Project authority

`EditorAssetState.project`保存可clone的`ProjectManager`。新delete preflight持有state `RwLock`读锁并在该
aggregate上查询、collect、sort和clone；Runtime relocation又在generation read下执行
`active_project.clone()`，在clone candidate上prepare，提交后Editor调用`refresh_from_runtime_project`
重建完整投影。正确链应是：

`RuntimeAssetGeneration -> immutable topology query/ticket -> durable mutation -> affected delta -> Editor projection`

Editor不保存第二个完整manager，不在owner lock内排序/分配，不以full refresh补偿mutation。

### P0：delete/relocation preflight没有可晋升的generation ticket

Runtime `AssetMutationDeletePreflight`明确要求commit时重新执行，因为结果不保留generation或source
reservation。Editor随后又复制target/referencers。大referencer集合因此会在admission和commit重复查询、
排序与复制；预览也不能安全晋升。Runtime04应返回
`AssetMutationTicket { project_generation, topology_generation, source_identity, compact affected IDs }`，
commit只复验generation/mutation evidence，stale时显式拒绝。

### P1：referencer query重复排序且比较器分配

`AssetRegistryIndex::get_referencers_by_uuid`从已有reverse `HashSet`复制后按`ToString::to_string`
排序；delete preflight随后按locator再次排序，并在tie break中每次比较都格式化两个UUID字符串。
relocation preflight也使用同一字符串tie break。复杂度为`O(R log R)`且比较阶段产生`O(R log R)`
临时字符串形状。稳定顺序应由generation拥有的ordered index/slot提供，或只在最终可见page按
allocation-free typed UUID bytes比较；内部mutation无需先做一次UUID string sort。

### P0：Save All移动了线程，没有完成非阻塞transaction

adapter原子reserve是进展，但仍完整clone/materialize batch，worker通过任意executor回调document
authority，running payload/result不受pending estimate约束，且产品路径未连接。UI若submit后wait，
仍只是调度间接层。必须与explicit save/autosave共用一个streaming durable coordinator、document lane、
真实payload resident bound和generation-checked completion receipt。

### P1：Import/Index仍保留blocking与第二投影风险

Import single-flight/limits值得保留，但Condvar和同步observer会卡caller；distinct UUID identity/order路径
可退化。`EditorAssetIndex::rows()`全量collect/sort，若接入Browser会与现有catalog generation形成双truth。
Runtime04应发布stable ordered slots和affected UUID delta，Editor只处理affected+visible page。

### P1：Plugin asset-type materialization仍复制完整base

本轮M0只消除静态base的重复构建/校验。每个mutable registry仍clone 26个definition和FieldOwners，
plugin candidate validation仍可能重放existing+candidate。最终需要`Arc<ImmutableBuiltinBase> + overlay delta +
one capability generation batch`，而不是在M0上继续堆缓存。

## 已落地M0：共享一次已验证builtin base

`builtin_registry_base()`以`OnceLock<Result<AssetTypeRegistry, AssetTypeRegistryError>>`保存一次构建结果；
`builtin_asset_type_definition`直接借用该base，`with_builtins`通过`builtin_registry()`获得独立clone。
错误结果同样缓存并clone，调用签名和独立可变语义不变。

静态量化：首次build仍为26 contributions；后续每次`with_builtins()`的contribution构建/验证/apply
**26 -> 0**。本M0不宣称registry clone bytes=0，也不把内建类型没有触发的creation-menu编译计入收益。
source contract已在旧实现上RED、在当前实现上GREEN。

## Unreal主参考依据

- `AssetRegistryState.cpp:2252-2275`从`CachedDependsNodes`直接查node并追加referencers；依赖图是
  registry owner维护的索引，不由Editor临时重建。Zircon应保留Runtime reverse index并提供generation view。
- `AssetRenameManager.cpp:387-511`把find referencers、package status、load referencing packages、
  checkout/read-only和rename分成显式阶段；`:810-870`还按package缓存一次registry referencer查询。
  `:1738-2049`区分perform与save referencing packages。这支持prepared mutation ticket、共享查询和
  durable phase telemetry；UE的同步modal/全package加载不是Zircon主线程目标。
- `ObjectTools.cpp:316-460,2767-3204`为delete显式收集内存referencers并让用户确认；它说明delete
  不能只看路径存在性。Zircon的MVP可以先fail-closed阻止referenced delete，但必须按generation复验。
- `AssetTools.cpp:1429-1472`在AssetTools owner中注册并保留asset type actions及lookup，而不是每次请求
  重建静态类型集合。这支持共享已验证builtin base；Zircon仍需保留plugin generation与独立overlay。
- `FileHelpers.cpp`与`PackageAutoSaver.cpp`的dirty set、prepare/save/post-save阶段继续约束Save All。
  它们不是复制UE package/global wait模型的依据。

## Hard-cut顺序

1. Runtime04用immutable generation store统一registry/catalog/project authority；Editor删除完整
   `ProjectManager`副本和`current_project_snapshot`类API。
2. Runtime04为delete/rename/move提供typed mutation ticket、ordered affected IDs和一次topology query；
   commit复验generation，禁止admission与commit重复全排序/全clone。
3. reverse index返回borrowed/stable slot view或paged generation；内部顺序使用allocation-free typed key，
   UI只对visible page生成字符串。
4. Frameworks01/Runtime11执行唯一durable source/meta/registry/resource transaction，含write/fsync/replace/
   restart recovery；Editor03只提交undoable intent并消费terminal receipt。
5. Editor09从affected delta更新catalog/details/preview/reference surfaces；mutation后full refresh=0。
6. Editor14/Runtime11把explicit save、Save All、autosave统一为bounded streaming coordinator；UI wait=0，
   running+queued+result bytes均受约束。
7. Editor12/Plugins01发布immutable builtin base + plugin overlay generation，一次batch materialize；
   删除逐candidate完整base clone/replay兼容路径。

## 验收矩阵

| 门 | 输入 | 必须采集 | 接受条件 |
|---|---|---|---|
| topology | assets/referencers `1/100/10K/1M`，visible `0/50/1K` | reverse visits、sorts/comparisons、formatted strings、cloned bytes、lock p95 | one query/generation；internal string allocation=0；work near affected+visible；owner lock不含sort/clone |
| mutation | delete/rename/move、companions `0/1K`、stale/fault each phase | ticket reuse/reject、project clones、fs/meta/registry/resource phases、restart outcome | aggregate clone=0；full refresh=0；one durable owner；stale apply=0；UUID identity不变 |
| save | docs `1/16/1K/16K`，payload `1KiB/64MiB/1GiB`，stall `0/10s` | queued/running/result bytes、UI wait、lane、cancel/deadline、generation commits | whole-batch materialization=0；UI wait/I/O=0；resident hard bound；one commit/document |
| import/index | duplicate/distinct UUID `1/10K/1M`，rows `1/100K` | submit block、identity scans、sort/clone bytes、stable projection work | caller block=0；same generation job/parse=1；stable rebuild/sort=0；Browser one truth |
| type registry | reloads `1/100/10K`，types/templates `26/10K` | base builds/validations/cloned bytes、overlay work、publishes | base build=1/process；stable base work=0；one overlay batch/publish；final clone bytes measured/bounded |
| product | cold/warm F1/F4 save/import/delete/rename，至少31可比样本 | WPR CPU/File I/O/waits/locks/CSwitch/RSS/power，typed phases | p50/p95/p99/CI/effect size；无current binary不填数值；RenderDoc仅验收thumbnail/Browser可见GPU parity |

## 静态验证

- Asset current 38/38文件已按base report、post-base commits、current diff和新增文件逐一reconcile。
- `rustfmt --edition 2021 --check`：38/38 GREEN。
- focused Python 4模块执行10 tests：9通过、1 error。error为
  `test_editor09_asset_type_registry_batch_contract.py`读取已删除的
  `zircon_editor/src/core/editor_plugin.rs`；这是validator owner漂移，不是产品断言失败，本切片不恢复旧文件。
- 新builtin base source contract：1/1 GREEN，旧实现已先确认RED。
- scoped `git diff --check`：GREEN，仅有checkout行尾转换warning。
- Rust/Cargo、allocation/RSS scale、WPR/xperf、power、current product与RenderDoc未运行；无动态接受结论。


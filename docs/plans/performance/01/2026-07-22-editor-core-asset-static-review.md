---
related_code:
  - zircon_editor/src/core/asset
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/bevy/crates/bevy_asset/src/assets.rs
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
tests:
  - zircon_editor/src/core/asset/dirty/tests.rs::dirty_snapshot_stores_each_external_effect_id_once
  - zircon_editor/src/core/asset/index/tests.rs::runtime_registry_replacement_resolves_pending_paths_without_cloning_path_keys
  - zircon_editor/src/tests/editor_asset_type_registry/materialization.rs::registry_delta_path_has_no_entry_clone_or_per_delta_full_sort
  - current-source Windows Cargo and F1/F4 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core asset逐文件性能静态审查（2026-07-22）

## 范围与覆盖

已逐文件阅读`zircon_editor/src/core/asset`生产 **24/24**，并完整阅读`dirty/tests.rs`与`index/tests.rs`；`import_flow/tests.rs`仍待后续测试批，故该目录当前为 **26/27**。`zircon_editor/src/core`累计完成 **214/257**，剩余43个继续留在`pending.md`。受管Cargo lane由`runtime09-ui-timer-frame-visible-deadline-leaf-20260722`预约，本轮没有运行raw Cargo，也没有把未执行的产品/RenderDoc验证写成通过。

## 已确认的性能形状

- Dirty snapshot过去同时持有`Vec<DirtyExternalEffectId>`和以同一ID为key的`BTreeMap`。本轮改为sorted ID Vec与平行revision Vec，revision查询使用binary search，公开排序和compare-and-clear语义不变。整批snapshot仍复制全部effect map、逐document进入Editor03 history并在generation抖动时最多8轮重试，登记PERF-MVP-554。
- `EditorAssetIndex::replace_runtime_registry`过去先clone所有已解析pending path，再逐项第二次hash remove。本轮用`HashSet::retain`原地转移命中的UUID，成功路径path clone=0。`rows()`仍令runtime registry每次collect+path sort，registry replacement仍全量校验projection；回链PERF-MVP-500并登记556。
- Import flow通过同URI mutex group串行任务，却不single-flight/coalesce；重复watch/digest/manual request都会进入EditorJobSystem。submit为原子URI→UUID/importing标记同时持state/index两锁，job label和两次progress还格式化URI；登记PERF-MVP-555，必须由Editor09/14与Runtime04统一准入，而不是在UI另建队列。
- AssetTypeRegistry既有validate-then-commit已删除完整entry clone，本轮进一步让existing lookup直接借用contribution key，新definition才复制一次key；单条delta保留binary ordered insert，多条template/command delta一次extend+sort，避免K次Vec移位。既有clone-on-augment failure仍因current-source Cargo/规模证据未完成而保持open。
- `DirtyRegistry`、`EditorAssetIndex`、`ImportFlowState`均使用单owner锁；foreign import backend在job中锁外执行，未发现将asset decode/I/O放入这些mutex的新增路径。风险主要是全量projection、重复任务准入与跨两锁提交，而非伪造更多线程。

## 参考引擎核对

- Godot `EditorFileSystem`以FileInfo、pending scan/change与targeted `update_files`维护编辑器资产视图；Zircon应让runtime generation提供稳定ordered slots/affected set，Editor只维护瞬态bitset，不在每个Browser查询重排全registry。
- Bevy `Assets`在唯一collection内排队typed `AssetEvent`，`AssetServer`明确在spawn可能阻塞的任务前释放`AssetInfos`锁；Zircon同样应把import backend/job提交放在短reservation提交之外，并让重复generation共享ticket。
- Unreal AssetRegistry提供compiled filter/index查询，dirty package保存有集中owner；Zircon的dirty/save与asset query应消费batch generation和索引，不让每个consumer重复扫描或保存自己的dirty真源。

## 本轮直接止损与动态验收

直接止损四组：effect ID双owner→单owner；pending path reconcile删除clone+二次remove；existing asset-type lookup删除key clone；multi-entry contribution按批合并。对应源码RED→GREEN守卫、scoped rustfmt与`git diff --check`通过；托管Cargo因CPU lane reservation未运行。

动态门以assets/documents/requests **1/1k/100k**、duplicate import **1/1M**、stable/1% change、consumer 30/60/120Hz记录rows collect/sort、effect/map clone bytes、history/index/state锁wait+hold、jobs submitted/merged、queue entries/bytes/oldest age、URI String bytes、RSS与p95。必须满足stable generation全量build/sort=0、每effect ID owner=1、同UUID/source generation实际import≤1、所有队列bytes硬有界，并通过watch rename/remove、meta atomicity、import cancel/panic/path migration、dirty undo/redo/save token、F1/F4产品操作；完成current-source Cargo和产品trace前不进入`review.md`。

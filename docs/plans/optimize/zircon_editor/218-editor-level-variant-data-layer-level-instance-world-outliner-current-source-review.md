---
title: Editor Level、Variant、Data Layer、Level Instance 与 World Outliner 当前源码复审
category: zircon_editor
report_id: Editor218
review_date: 2026-08-29
baseline_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
verification_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
canonical_owner: Editor41
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/115-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
  - docs/plans/optimize/zircon_editor/162-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zc-runtime-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/128-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-current-source-review.md
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
related_handoffs:
  - docs/plans/zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md
  - docs/plans/zircon_editor/editor/10/failure-2026-07-17-project-open-repeated-manager-scan.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/hierarchy.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/world/project_io/prefab.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_plugins/prefab_tools
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition/DataLayer
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance
  - dev/UnrealEngine/Engine/Plugins/Enterprise/VariantManagerContent
  - dev/UnrealEngine/Engine/Plugins/Enterprise/VariantManager
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/Fyrox/editor/src/world
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/BatchLayers.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/RenderingLayerUtils.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor218 · Level / Variant / Data Layer / Level Instance / World Outliner 当前源码复审

## 1. 结论

Editor162 之后没有出现足以改变 canonical 状态的产品闭环。Level Variant 仍是 233 行固定 ZUI，含 19 条 route，并继续发布 Vehicle_Showcase、Variant_Red、18 overrides、2 conflicts 与 Preview/Apply queued 文本；真实 Variant source、binding、capture、compiler、artifact、preview scope、apply operation 和 runtime consumer 仍为零。Data Layer 与 Level Instance 仍没有独立 ResourceKind、source asset、world-local owner、runtime authority、revision、state machine、cook artifact 或正式 Editor session。

Prefab 的真实进展仍限于 metadata preservation：Scene/World/正式 document codec 可以逐字段 roundtrip prefab_instance，插件也明确拒绝在 importer、graph authority 和 transaction factory 安装前提供 Create/Open/Apply/Revert/Break。核心 DTO 仍是 AssetReference + local transform + entity_path/property_path 字符串 + serde_json::Value；runtime importer 仍为 DiagnosticOnly。ReflectObjectAddress 只提供 component/resource object address，field_name 仍为字符串，且 Prefab/Variant 没有消费它；这不是 source-local stable ID、typed collection selector、declared type 或 schema fingerprint。

Hierarchy 有真实但局部的工程化底座：world/selection generation fence、rename sparse delta、固定 10 个 physical controls、viewport-bounded paint、ASCII 无分配大小写匹配和 projection profiling 都应保留。但 topology change 仍强制 full reflow；filter 每次仍同步 O(N) 扫描并 clone 可见行；最终 SceneNodeData 仍只有 id/name/depth/selected；context menu 仍是固定字符串，未找到四个 SceneNode action 的 production executor。没有 typed item/provider/mode/column、folder、unloaded descriptor、DataLayer/LevelInstance item、query generation/cancel/LKG 或 100k/1M qualification。

Editor41 的 canonical finding 总数保持不变，本轮状态仍为：**P0：3 Open / 1 Partial / 1 Closed；P1：60 Open / 9 Partial / 1 Closed；P2：12 Open；Gates：30 Fail / 2 Pass**。没有同内容、同画质、同硬件、同平台、同故障语义的动态证据支持本域性能或表现达到、接近或优于 Unreal；局部代码更少、ASCII filter 更省分配或只绘制可见行都不能构成领先证明。

## 2. 审查范围、统计与 currentness

统计读取当前 working-tree 物理内容。行数为物理行；tests/ignored 只统计精确 Rust test/ignore attribute。fingerprint 按 lowercase repository-relative path 排序，将 path、NUL、lowercase file SHA-256、LF 拼接后再取 SHA-256。选择集用于复现本报告，不代表参考仓库全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Level Variant 产品面 | **5 / 2,686 / 2,639 / 131,486 / 1 / 0** | a5e59844fd08a4af68cee5fd936cbaa78abaf44fc97c739d992e46463003a697 |
| Prefab / Instance / Scene IO | **21 / 3,012 / 2,759 / 112,198 / 13 / 0** | 72c6df55dc7da302b83aec41c4a3453ba1f40ef6db09552f3a1496c0ae3e6a07 |
| Hierarchy / Outliner / world sync | **50 / 7,844 / 7,084 / 270,000 / 83 / 0** | baafe0e07384b06f49f1ea934b07215147a7e91b0ba00c00e3ad3433d95813a3 |
| Zircon selected union | **76 / 13,542 / 12,482 / 513,684 / 97 / 0** | 95908390b4a4eab26d98eb46d8346225fc890043376ebcaba986acbae7b402a9 |
| Unreal / Godot / Fyrox / Bevy / Unity Graphics selected | **31 / 18,258 / 15,502 / 670,221 / 2 / 0** | 329ed6a1c3f4a9b427693fdb02728dabe78c998938c10445aed9cb3c5a18c76b |
| All selected | **107 / 31,800 / 27,984 / 1,183,905 / 99 / 0** | 9cd435d79fa9ae1577e3a751d6e9ea54dcb58ae5f59f0830542926e0c1f942dc |

- baseline 与 verification HEAD 均为 f660cfa9f3f84bff0903e4564ff1af4d065aee73；共享工作树含大量用户与其他 Session 的在途修改，本报告读取物理文件，不回退、不覆盖，也不把未完成改动写成已集成能力。
- 四个 production roots 当前有 16,908 个已跟踪、2,366 个未跟踪且物理存在的 Rust/TOML/ZUI/Zr 文件，共 19,274 个。DataLayerAsset、DataLayerInstance、DataLayerSource、LevelInstanceId、LevelInstanceSource、WorldFragment、WorldOutliner、OutlinerItemProvider、OutlinerColumnProvider、VariantSetAsset、VariantBindingSource、WorldPartitionCellDescriptor 十二个精确合同全部零命中。
- ResourceKind 仍为 26 个 variant，包含 Prefab，不包含 VariantSet、DataLayer、LevelInstance 或 WorldFragment。
- Level Variant ZUI 仍为 233 行、19 条 route；对应 feedback 继续固定 queued 结果。精确产品合同为零，因此 route、preview-action allowlist 和 template binding 只能证明 UI wiring，不能证明业务执行。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；open handoff 只作为静态边界，不阻塞本轮 review。Tooling 按用户要求排除。
- 本轮没有修改 production、tests、Cargo、ABI 或参考源码，也没有运行 Cargo、Editor、save/reopen、cook、fault、scale、soak、跨平台或跨引擎 benchmark。两个 Pass 都来自当前静态源码与既有测试断言，不等于本轮动态 qualification。

## 3. Editor162 之后的 current-source 重判

| 主题 | 当前源码事实 | canonical 影响 |
|---|---|---|
| Variant 产品面 | 233 行、19 routes、固定 Vehicle_Showcase/Variant_Red/18/2；Preview/Apply 仍只写 queued feedback | P0-01、P1-31 至 P1-40 与 G01-G06 保持 Open/Fail |
| Data Layer / Level Instance | 十二个精确合同零命中；ResourceKind 仍只有 Prefab | P0-02、P1-03/P1-07、P1-11 至 P1-25 与 G07-G12/G15-G19 保持 Open/Fail |
| Prefab preservation | Prefab metadata 正式 codec/World roundtrip 仍成立 | P0-03 Closed、P1-30/P1-69 Partial、G13 Pass 保持 |
| Prefab authority | override 仍为两条字符串 path + JSON；importer 仍 DiagnosticOnly；插件 README 主动隐藏危险 operation | P0-04、P1-26 至 P1-29 与 G14-G19 保持 Open/Fail |
| Reflection foundation | ReflectObjectAddress 只限定 component/resource object；field_name 仍是字符串，无 source object ID、selector、declared type、schema fingerprint | 不把 Runtime99l 底座误记成 Variant/Instance typed address 产品 |
| World sync row | WorldHierarchyRow 有 entity/parent/depth/display/kind/hash/active/children，但最终 SceneNodeData 只保留 id/name/depth/selected | P0-05 维持 Partial；P1-41 至 P1-46 保持 Open |
| Sparse update | rename/selection 可 O(Delta) patch；新增/删除/移动/重排仍发 requires_hierarchy_reflow | P1-53 Open；P1-56/P1-59 Partial；G26 Fail |
| Filter | ASCII 匹配避免逐行 lowercase allocation，并新增 source/match/ancestor/visible counters；仍同步全量扫描、parent index、visible clone | P1-54/P1-55/P1-60 与 G28/G29 保持 Open/Fail |
| Context menu | SceneNode 固定 Open/Rename/Duplicate/Delete 字符串；production 搜索只有 provider/tests，没有对应 executor | P1-50 Open，G24 Fail |
| Bounded paint | 固定 10 controls 与 10k viewport-bound test 仍成立 | P1-51 Closed，G27 Pass 保持 |

## 4. 四条产品纵链事实

| 产品链 | 当前源码事实 | 判定 |
|---|---|---|
| VariantSetSource -> compiler -> VariantArtifact -> Preview/Apply -> runtime switch | 只有固定 ZUI、navigation、preview allowlist、template binding 与 fixed feedback | Open |
| DataLayerAsset -> membership/state authority -> registry/cook artifact | 没有领域类型；render_layer_mask、active_in_hierarchy 与 ResourceKind::Data 均是不同语义 | Open |
| LevelInstanceSource -> load/edit/rebase -> provenance artifact -> World adapter | Prefab metadata preservation 已有；source identity、instantiate/lifecycle/provenance/rebase/edit/cook 均无 | Partial foundation，产品 Open |
| WorldOutlinerModel -> typed providers -> bounded truthful projection -> commands | hierarchy artifact、generation fence、rename/reparent transaction、bounded paint 已有；typed providers、semantic rows、range delta、indexed query、truthful actions与规模资格无 | Partial foundation，产品 Open |

## 5. P0：必须先阻断的产品与 authority 断路

| ID | 状态 | 当前事实与必须重构内容 |
|---|---|---|
| P0-01 | Open | 固定 Variant workspace 仍以 native Preview/Apply queued 状态冒充真实结果。真实 provider 未接入前必须隐藏、显式标 Fixture/Unavailable 或返回 typed unsupported；不得继续发布业务数字。 |
| P0-02 | Open | 仍缺 Data Layer 产品；禁止把 render_layer_mask、active、runtime parent、ResourceKind::Data 或 Scene hierarchy 重命名为 Data Layer/World Outliner。 |
| P0-03 | Closed | Scene/World 与正式 document codec 已无损保留 prefab_instance，现有 exact-equality tests 覆盖 asset reference、local transform 和 overrides。后续 schema/rebase 风险由 P1-26/P1-27/P1-69 持有。 |
| P0-04 | Open | Variant Apply、Prefab override Apply、Level Instance Commit 仍无 stable identity、expected revision、完整 preflight、atomic rollback 与 durable receipt；这些入口不得修改共享 asset/live World。 |
| P0-05 | Partial | bounded physical controls、可见区 paint 与真实 rename/reparent 已存在；typed item/owner/editability/load/lock/visibility、真实扩展 action、range delta、query/cancel 和 100k/1M qualification 仍缺失，不能宣称工程级 Outliner。 |

## 6. P1：工程化重构主线

### 6.1 Identity、schema 与 revision 基础（P1-01 至 P1-10）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-01 | Open | 定义跨 World/session 不碰撞的 world-qualified object ID；当前 u64 entity ID 无 owner namespace。 |
| P1-02 | Partial | Scene entity ID 在单一 source 内可稳定参与 rename/reparent，但没有 WorldFragment/source namespace、migration 与跨实例身份；建立 source-local stable ID。 |
| P1-03 | Open | 定义 LevelInstanceId、parent instance、source revision 和 generation 合同。 |
| P1-04 | Open | 生成 source-object 与 instance-object 双向 provenance artifact，禁止运行时猜路径。 |
| P1-05 | Open | 建立可持久化 typed reflected property address；现有 ReflectObjectAddress + string field_name 不满足 selector/type/schema 要求。 |
| P1-06 | Open | address 必须含 component、field、collection selector、declared type 与 schema fingerprint。 |
| P1-07 | Open | 为 VariantSet、DataLayer、WorldFragment/LevelInstance 增加独立 ResourceKind、marker、importer 和 owner。 |
| P1-08 | Open | 为新 authoring asset 定义版本化 schema、migration、unknown-field 与 downgrade policy。 |
| P1-09 | Partial | hierarchy 已带 world generation/selection revision；Variant/DataLayer/Instance request、delta、receipt 仍无 owner/request ID/source revision。 |
| P1-10 | Open | 用 lint、roundtrip 和 rename tests 固化 display name/path 与 stable identity 分离。 |

### 6.2 Data Layer（P1-11 至 P1-20）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-11 | Open | 实现版本化 DataLayerAsset 与 world-local DataLayerInstance，二者职责不可合并。 |
| P1-12 | Open | 实现 many-to-many membership record、批量 assignment、owner validation 和 undo receipt。 |
| P1-13 | Open | 实现 runtime/private、client-only/server-only/both filter policy，不复用 render layer。 |
| P1-14 | Open | 实现 Unloaded/Loaded/Activated requested state 机及明确 transition failure。 |
| P1-15 | Open | 计算 parent-aware effective state，拒绝 cycle、跨 owner 与 type-invalid hierarchy。 |
| P1-16 | Open | 分离 shared initial editor state、per-user loaded/visible state 与 runtime replicated state。 |
| P1-17 | Open | 将 current layer creation context 接入 actor/entity creation、duplicate、paste 与 undo/redo。 |
| P1-18 | Open | 定义 server/client write authority、replication、event、expected generation 与 stale policy。 |
| P1-19 | Open | 为 unloaded descriptor 保留 layer membership、owner、folder、instance 和 Outliner 字段。 |
| P1-20 | Open | 产出 partition/cook registry、membership artifact、initial state、dependency 与 stable diagnostics。 |

### 6.3 Level Instance / Prefab（P1-21 至 P1-30）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-21 | Partial | Prefab DTO/插件边界已明确不冒充 Level Instance，但共同 foundation 与两个独立产品 owner 尚未设计完成；禁止 type alias。 |
| P1-22 | Open | 建立版本化 WorldFragment/LevelInstance source asset 与 stable local object IDs。 |
| P1-23 | Open | 实现 instance register/load/wait/unload/fail/stale 状态机和 terminal receipt。 |
| P1-24 | Open | 接入 dependency readiness、priority、memory/I/O budget、cancel、retry 与 shutdown drain。 |
| P1-25 | Open | 实现 nested ancestry、source/instance loop detection 和完整 chain diagnostic。 |
| P1-26 | Open | 以 typed address + source object ID 替换 Prefab 的字符串 entity/property path 与 raw JSON。 |
| P1-27 | Open | 实现 base/source/instance three-way rebase，并分类 clean/applied/conflict/orphan/type mismatch。 |
| P1-28 | Open | 实现隔离 edit session、current instance、dirty、source revision lock、commit 与 discard。 |
| P1-29 | Open | 实现 create-from-selection、move-to-instance、break、pivot、bounds 的 atomic transaction。 |
| P1-30 | Partial | Scene IO preservation 和 runtime component descriptor 已闭合；importer backend、实例化、World lifecycle、provenance、reload/rebase 仍无。 |

### 6.4 Level Variant（P1-31 至 P1-40）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-31 | Open | 实现 VariantSet source document、importer、compiler/artifact 与 asset editor session。 |
| P1-32 | Open | 为 set/variant/binding/capture/function call 分配 stable ID 与 deterministic ordering。 |
| P1-33 | Open | 实现 loaded/unloaded 都可诊断的 object binding resolver，保留 last-known label 但不以 label 定位。 |
| P1-34 | Open | 建立 reflection-driven property capture registry、capturability/read-only/side-effect policy。 |
| P1-35 | Open | 实现 typed recorded-value codec、schema migration、custom type adapter 和 byte/category contract。 |
| P1-36 | Open | 实现 Record transaction、current-vs-recorded comparison、dirty 与 undo/redo。 |
| P1-37 | Open | 实现可撤销 Preview scope、异常恢复、Variant 切换与多 viewport 一致性。 |
| P1-38 | Open | 实现 Apply 全量 resolve/preflight、atomic/best-effort policy、rollback 与 durable receipt。 |
| P1-39 | Open | 实现 set/variant/binding/capture 的 create/remove/move/duplicate/merge 与冲突诊断。 |
| P1-40 | Open | 产出 immutable cooked switching artifact，并公开 source/artifact revision 与 runtime install receipt。 |

### 6.5 World Outliner 产品模型（P1-41 至 P1-50）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-41 | Open | 定义 typed Outliner item ID 与 World/Level/Folder/Entity/Component union，禁止 pane row index 充当身份。 |
| P1-42 | Open | 增加 Descriptor/DataLayer/LevelInstance item 与稳定 namespace，支持 loaded/unloaded 同一身份。 |
| P1-43 | Open | 建立 hierarchy provider，拥有 parent/children/materialization/delta，而不是直接读取一份全量 Scene Vec。 |
| P1-44 | Open | 建立 mode 接口，拥有 selection、context、drag/drop、rename/delete、folder 和 editability policy。 |
| P1-45 | Open | 建立 column registry，支持 cell/search/sort/action、source-control/load/error columns 与 plugin contribution。 |
| P1-46 | Open | 建立 type/tag/component/layer/level/instance/state filter grammar 与 typed match reason。 |
| P1-47 | Open | 将 actor folder 组织层与 runtime transform parent 分离，drop target 必须声明动作语义。 |
| P1-48 | Open | 实现 expand/collapse、expand all、reveal、breadcrumb 与 workspace persistence；expanded = has_children 不是状态。 |
| P1-49 | Open | 实现 pin/sort/group/only-selected/hidden/locked 与 versioned filter preset。 |
| P1-50 | Open | 动态构造 typed context menu，只显示已安装 executor，并接真实 command/transaction/receipt。 |

### 6.6 Outliner 增量与规模（P1-51 至 P1-60）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-51 | Closed | Scene tree 物理控件固定 10，native paint 只处理可见范围；现有 10k test 证明容量不随 logical count 增长。 |
| P1-52 | Open | provider 必须按 expansion lazy materialize children，并保留轻量 subtree aggregate；当前 full model 常驻。 |
| P1-53 | Open | topology delta 必须支持 insert/remove/move/reorder range 与 ancestor patch；当前 topology change 强制 reflow。 |
| P1-54 | Open | filter/sort 使用 query generation、cancel、deadline 与 last-known-good projection。 |
| P1-55 | Open | 建立 name/type/tag/layer 索引；ASCII fast path 和 O(N) profiling 不等于索引。 |
| P1-56 | Partial | world generation 与 selection revision fence 已存在；需扩展到全部 typed provider、source revision 和 request ID。 |
| P1-57 | Partial | reparent 已检查 cycle/static parent 并整笔拒绝；仍缺 owner/instance/layer/folder/lock/editability 全量 preflight。 |
| P1-58 | Partial | 多选 reparent 已用单一 atomic transaction/undo；大批量 selection/layer/reparent 仍无 chunk、progress、cancel 和 rollback budget。 |
| P1-59 | Partial | sparse update 下 entity ID、selection、rename 与 scroll offset 有基础；没有 expansion path、typed anchor 和跨 provider 稳定恢复。 |
| P1-60 | Open | 建立 100k loaded / 1M descriptor 的 memory、query、scroll、paint、input、churn、cancel 预算与 receipts。 |

### 6.7 文档、事务、作业、cook 与证据（P1-61 至 P1-70）

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| P1-61 | Open | Variant/DataLayer/LevelInstance 编辑接入 document dirty/save/autosave/recovery。 |
| P1-62 | Open | 跨 Scene/source asset commit 接入可恢复 multi-document transaction coordinator。 |
| P1-63 | Open | load/rebase/audit/filter/cook 接入 Editor job admission、cancel、progress、deadline 与 shutdown。 |
| P1-64 | Open | 定义 stable diagnostic code、owner/item/property、related asset、chain 与 fix action。 |
| P1-65 | Open | 建立 source-control/dirty/error/load/visibility/lock 的真实 provider columns。 |
| P1-66 | Open | cook 拒绝 missing source、instance cycle、orphan override、invalid layer、unresolved capture。 |
| P1-67 | Open | 建立新 asset 的 roundtrip、migration、unknown-field、deterministic artifact 与 skew tests。 |
| P1-68 | Open | 建立 Variant capture/apply/rollback、DataLayer authority、LevelInstance lifecycle 的完整矩阵。 |
| P1-69 | Partial | Prefab link 的 Scene/World/正式 codec exact roundtrip 已有；source reload、rename/reparent resolution、rebase regression 仍无。 |
| P1-70 | Open | 建立多 provider、unloaded descriptor、context action 与 100k/1M 端到端 Outliner qualification。 |

## 7. P2：主线完成后的扩展

| ID | 状态 | 后续扩展 |
|---|---|---|
| P2-01 | Open | Variant thumbnail、director/function call、remote preview 与 multi-user conflict UI。 |
| P2-02 | Open | Variant composition、inheritance、parameterized variants 与 batch render/export。 |
| P2-03 | Open | Data Layer external content bundle 与跨项目可挂载 layer package。 |
| P2-04 | Open | Data Layer runtime debugging、network authority timeline 与 state heatmap。 |
| P2-05 | Open | Level Instance per-type property merge policy plugin 与 adapter registry。 |
| P2-06 | Open | Level Instance HLOD、world partition container 与 distributed cook integration。 |
| P2-07 | Open | multi-user Level Instance edit lease、review、merge 与 changelist workflow。 |
| P2-08 | Open | Outliner custom grouping、saved collections、smart folders 与 bookmarks。 |
| P2-09 | Open | remote PIE/server world comparison 与 cross-world selection bridge。 |
| P2-10 | Open | background indexed query、million-item paging 与 GPU-assisted visualization。 |
| P2-11 | Open | Variant/DataLayer/Instance Python/commandlet automation 与 headless validation API。 |
| P2-12 | Open | 跨 Variant/DataLayer/Instance/PCG/Sequencer 的统一 provenance/diff browser。 |

## 8. 五套参考源码的合同差异

| 参考 | 已读取的关键合同 | Zircon 必须吸收的工程原则 | 不得误用 |
|---|---|---|---|
| Unreal | DataLayerAsset 与 world-local DataLayerInstance 分离；requested/effective Unloaded/Loaded/Activated、parent validation、runtime/private、client/server authority、ActorDesc resolution 与 per-user editor state 由 manager/WorldDataLayers 持有。LevelInstance 有 ID、register/load/unload、loop check、edit 与 override policy。Variant 有 binding/property capture/value/switch。SceneOutliner 将 item/hierarchy/mode/column 和 unloaded ActorDesc 分开。 | 分离 source policy、world instance、runtime authority、editor projection；mutation 经过 owner/revision/preflight/receipt；Outliner 是可扩展策略系统。 | 不机械翻译 Unreal 类名；display/property segment 仍需升级为 Zircon stable field identity 和 schema fingerprint。 |
| Godot | PackedScene/SceneState 真实 pack/instantiate ownership、editable instance、dependency、connection 与 recovery；SceneTreeDock 将 instantiate/reparent/duplicate/replace/save branch 纳入 undo/redo，并做 cycle/foreign-owner 检查。 | Scene source ownership、editable boundary、instance generation、save/reopen 与 authoring command 必须同时存在。 | Godot NodePath 只能作为 UI/兼容输入，不能成为最终 override authority key。 |
| Fyrox | WorldViewerDataProvider 分离 root/children/parent/name/selection/validate；drop 通过 CommandGroup 组合 link、child position 与 selection；context action 有稳定 UUID。 | 复用 Zircon transaction 方向，但提升 provider identity、drop validation、selection receipt 和 command registration。 | Fyrox viewer 主要面向 loaded graph，不覆盖 DataLayer、unloaded descriptor 或百万级 paging。 |
| Bevy | ScenePatch 注册 dependency，resolve 为 ResolvedSceneRoot，再 spawn/apply；cached scene 有 copy-on-write 和明确错误。 | LevelInstance/Variant artifact 在 resolve 后不可变；dependency readiness、queued/waiting/failed/cancel 与 source revision 必须可观察。 | ScenePatch 是 runtime dependency/apply 参考，不等于完整 Editor commit、three-way rebase 或 World Outliner。 |
| Unity Graphics | BatchLayer 与 RenderingLayerUtils 只定义 GPU draw filtering、mask bit/format 和 renderer event。 | 在架构与 lint 中永久隔离 render filtering 与 Data Layer authoring/runtime ownership。 | 绝不能把 rendering layer mask 包装为 Data Layer；它没有 source asset、membership、load state、network authority 或 cook registry。 |

## 9. 目标架构与 owner 边界

目标保持四条独立纵链，公共部分只共享 identity、schema/reflection、artifact、transaction、job、diagnostic 与 world-sync primitives：

1. VariantSetSource -> VariantCompiler -> VariantArtifact -> PreviewScope/ApplyOperation -> RuntimeVariantService。
2. DataLayerAsset -> MembershipCompiler -> DataLayerRegistry -> AuthorityBoundStateSnapshot -> Partition/Cook adapter。
3. LevelInstanceSource -> InstanceCompiler/Artifact -> InstanceRegistry -> Load/Edit/Rebase Service -> World adapter。
4. WorldOutlinerModel -> typed HierarchyProvider/Mode/Column/Filter -> bounded Projection -> Editor command/receipt。

Editor41 继续拥有四类 authoring 产品、Outliner projection 与交互闭环；Runtime99zc 拥有 Prefab compiler/artifact/instance registry/rebase，Runtime99j 拥有 Scene/World lifecycle 与 project IO，Runtime99k 拥有 transform hierarchy/activation，Runtime99l 拥有 reflection/schema/property address。Editor128/129 分别持有 clipboard/drag payload 与通用 search/query；Editor138 持有 World Partition 大域；Editor181 持有通用 Scene hierarchy projection。跨 owner 实施必须先冻结公共合同和依赖顺序，不得复制第二套类型或状态机。

Runtime99zc 与 Runtime99j 对 prefab_instance 被 World 清除的旧描述已过时；Editor162/218 只取代这一条窄 currentness。它们关于 importer conflict、字符串 override、无 compiler/artifact/instance registry/provenance/rebase/network/save 产品闭环的主裁决仍有效。

## 10. 依赖有序里程碑

| Milestone | 当前状态 | 交付与退出条件 |
|---|---|---|
| M0 | Partial | 移除 Variant 固定成功文案；Prefab link 无损或 fail closed。Prefab 半项已完成，Variant 真相门仍失败。 |
| M1 | Pending | identity、generation、typed address、schema/migration ADR 与 tests 冻结。 |
| M2 | Partial | Scene/Prefab preservation 已完成；instantiate、provenance、source reload 与 save/reopen 产品闭环未完成。 |
| M3 | Pending | Data Layer asset/membership/state/authority/cook registry 完成。 |
| M4 | Pending | Level Instance register/load/unload/fail/cancel/retry/shutdown 与 loop detection 完成。 |
| M5 | Pending | isolated edit、commit/discard、three-way rebase、break/pivot/bounds 完成。 |
| M6 | Pending | Variant source/binding/capture/typed value/Record/migration 完成。 |
| M7 | Pending | Preview/Apply preflight、rollback、receipt 与 runtime artifact 完成。 |
| M8 | Partial | hierarchy artifact 可复用；typed Outliner model/provider/mode/column/filter/unloaded descriptor 未完成。 |
| M9 | Partial | rename/reparent transaction 可复用；folder/expansion/context/layer/instance interactions 未完成。 |
| M10 | Partial | bounded physical row 已完成；range delta、indexed jobs/cancel、100k/1M、cook/diagnostics 未完成。 |
| M11 | Pending | fixture 删除、字符串 key hard cutover、文档/capability/UI/runtime 一致与 reference recheck。 |

M0-M2 必须先于任何 Apply/Commit 成功文案；M3-M7 依赖 M1 公共 identity；M8 先吸收现有 hierarchy artifact 与 bounded paint，再由 M9/M10 增加语义和规模。Partial 只表示可复用基础存在，不允许开放产品 capability。

## 11. G01-G32 验收门重判

| Gate | 状态 | 当前证据 / 通过条件 |
|---|---|---|
| G01 | Fail | production 无独立 VariantSet/DataLayer/LevelInstance 类型、asset kind、owner 与公开合同。 |
| G02 | Fail | Vehicle_Showcase/18 overrides/2 conflicts 仍作为默认真实状态。 |
| G03 | Fail | Variant Preview/Apply 未触达真实 operation、revision、transaction 或 receipt。 |
| G04 | Fail | 无 binding/property 全量 resolve 和 atomic apply failure test。 |
| G05 | Fail | 无 Preview scope、切换/undo/play/异常恢复。 |
| G06 | Fail | 无 Variant source roundtrip/migration/stable ID/typed value/unknown-field tests。 |
| G07 | Fail | 无 Data Layer requested/effective 与三态 table test。 |
| G08 | Fail | 无 parent/cycle/private/runtime/client/server 组合 validator。 |
| G09 | Fail | 无 per-user layer state 与 shared source 隔离。 |
| G10 | Fail | 无 runtime layer server/client authority 与 stale generation policy。 |
| G11 | Fail | 无保存 layer/level/folder/instance/provenance 的 unloaded descriptor。 |
| G12 | Fail | 无 layer registry/membership/initial-state cook artifact。 |
| G13 | Pass | Scene/World/runtime-extension/正式 codec 对 prefab_instance 有逐字段相等测试；本轮未动态执行。 |
| G14 | Fail | override 仍用字符串路径，rename/reparent 后无法由 stable source ID 解析。 |
| G15 | Fail | 无 nested Level Instance 注册/加载/cook loop chain diagnostic。 |
| G16 | Fail | 无 dependency missing/cancel/load/unload failure 的实体、owner、provenance 泄漏矩阵。 |
| G17 | Fail | 无 source reload 与 clean/applied/conflict/orphan/type mismatch rebase。 |
| G18 | Fail | 无 edit commit revision preflight/atomic save/rollback/discard。 |
| G19 | Fail | 无 create/move/break/pivot 的 undo/save/reopen/crash recovery。 |
| G20 | Fail | 无跨 World/PIE/unloaded/nested instance 的 typed item identity。 |
| G21 | Fail | 无可注册卸载的 mode/column/filter/hierarchy provider 产品合同。 |
| G22 | Fail | runtime reparent 可用，但不存在独立 actor folder 与 typed drop semantic qualification。 |
| G23 | Fail | 无真实 expansion、reveal、breadcrumb、filter restore 与 workspace persistence。 |
| G24 | Fail | 右键菜单仍固定字符串，未证明所有可见 action 都有 production executor。 |
| G25 | Fail | filter 没有 hidden selected count、跨 owner 限制与 authoritative-selection receipt。 |
| G26 | Fail | topology change 仍要求 full hierarchy reflow，无 insert/remove/move/reorder range patch。 |
| G27 | Pass | scene tree 物理控件固定 10，paint visible range 有 10k node viewport-bound test；仅关闭 node-count gate。 |
| G28 | Fail | full model/reflow/filter 仍 O(N)，没有 100k scroll/paint/input/query 帧预算证据。 |
| G29 | Fail | 无 1M descriptor、indexed query、cancel、LKG、memory/latency qualification。 |
| G30 | Fail | hierarchy 只有部分 generation fence；Variant/DataLayer/Instance 异步结果无 owner/request/source generation。 |
| G31 | Fail | Windows dynamic、migration golden、cook、crash recovery 与性能 qualification 未通过。 |
| G32 | Fail | Variant fixture、菜单、capability、UI 与 runtime artifact 完成状态仍不一致。 |

## 12. 禁止的临时实现

1. 禁止把固定业务数字、queued 文本、disabled importer、descriptor、菜单项或 ZUI route 计为功能完成。
2. 禁止用 display name、UI path、entity row index、裸 u64、字符串 property path 或 JSON value 作为跨 source/instance authority identity。
3. 禁止把 render layer、active flag、runtime transform parent、Scene row 或 ResourceKind::Data 改名为 Data Layer。
4. 禁止把 Prefab metadata preservation 宣称为 instantiate、Level Instance、rebase、propagation 或 cook 完成。
5. 禁止让 visible menu action 在 executor 缺失时仍显示，也禁止 no-op/queued feedback 返回成功。
6. 禁止在 topology change 时隐藏 full reflow、在 O(N) filter 上只补 profiling 后宣称 indexed query。
7. 禁止用 bounded physical controls 证明 100k/1M logical model、query、memory、selection 或 mutation 已达标。
8. 禁止复制 Runtime99j/99k/99l/99zc、Editor128/129/138/181 的 owner；公共合同先冻结再接 adapter。
9. 禁止以参考引擎类名数量或局部代码更少证明架构、性能或表现领先。

## 13. 本轮验证与后续实施入口

- 静态读取 76 个 Zircon selected 文件、31 个参考文件，并对 19,274 个 production Rust/TOML/ZUI/Zr 物理文件执行精确合同复核。
- 十二个目标合同零命中；Variant 固定事实、Prefab 字符串/JSON DTO、DiagnosticOnly importer、SceneNodeData 四字段、topology full reflow、同步 O(N) filter 与固定菜单均重新定位到当前物理源码。
- P0/P1/P2/Gate 表逐项保留 Editor41 ID；Editor218 只刷新 currentness，不重复增加 canonical totals。
- 未运行 Cargo 或动态产品矩阵，因此实现状态保持 pending；下一实施入口严格从 M0 真相门与 M1 identity/schema ADR 开始。
- 当前工作树不是 clean baseline；后续实施前必须重扫 source drift，不得拿本报告 fingerprint 代替 build receipt。

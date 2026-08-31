# Frameworks01 结构性 successor 准入复核（2026-08-23）

## 状态

- `research_complete`
- `static_current_source_validation_complete`
- `implementation_not_started`
- `implementation_waiting_active_consumer_owner_transfer`
- `profiling_not_started`
- `current_owner_transfer_required`
- `managed_rust_validation_blocked_by_foreign_attribution`

本记录只固化结构判断、owner 路由和实施/测量门，不声称任何生产优化、性能收益或
功耗收敛已经完成。当前 Frameworks01 exact4 的 Rust 不可变副本仍被 Runtime74 五份
未接受 UI binding blob 阻断；Frameworks01 不吸收或改写这些 foreign 路径。

## 准入原则

1. 先恢复 Client/Editor 的可用资产导入闭环，再处理不影响功能的类型所有权收束，最后
   删除没有生产 producer 的 IK 队列债务。
2. 每项 hard cut 必须在同一 source snapshot 内删除旧 owner、alias、re-export、fallback
   和全部旧 consumer；不允许用兼容层分期保留双 owner。
3. 先证明真实产品路径存在，再做性能优化。只有 source complexity 推导时，必须明确写成
   渐进上界，不能当成 profile、功耗或引擎对标数据。
4. 生产改动只能由对应 numbered plan 的 active owner 在 audited ownership transfer 后执行。

## A. UI document importer 与 `asset -> ui` 反向边

### Current-source 结论

- `zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs` 直接调用
  `crate::ui::v2::UiZuiAssetLoader`，是当前 production dependency audit 中唯一明确的
  `asset -> ui` 反向边。`tools.tests.test_frameworks_05_asset_ui_boundary` 当前因此为 2/3，
  ZUI retired-importer guard 也因该文件仍存在而失败。
- `zircon_plugin_ui_document_importer_runtime` 已提供 `.zui` descriptor、解析实现和
  `plugin_registration()`；Runtime typed importer test 已要求 `AssetImporter::default()` 不再
  处理 `.zui`，只允许插件 fixture 解析。因此 Asset fallback 与既有测试契约彼此冲突。
- `RuntimePluginId::UiDocumentImporter` 和 builtin catalog row 已存在，但
  `zircon_first_party_runtime_catalog` 没有对应 optional dependency、feature 或 provider branch。
  `zircon_app` 的 `target-client`/`target-editor-host` 也没有链接该 provider。
- Runtime profile preset 只默认选择 `ui`，未选择 `ui_document_importer`。App provider projection
  读取 raw `EntryConfig.project_plugins`，而 module selection 使用
  `manifest_with_mode_baseline`；同一个启动过程因此存在两套 effective selection 语义。

### 结构决策

1. 以现有 `manifest_with_mode_baseline(target, override)` 作为唯一 effective plugin-selection
   算法；App provider projection、builtin module selection、editor project-open projection 和
   export plan 不得再各自重建 baseline/override 规则。
2. 为 UI importer 建立独立的 first-party catalog feature 和 App feature，并在
   Client/Editor 产品 feature 中链接；不要为了一个 importer 强制链接整个 base plugin bundle。
3. ClientRuntime 与 EditorHost 的 UI-capable baseline 默认选择
   `RuntimePluginId::UiDocumentImporter`。显式 project override 必须通过同一 merge 函数处理，
   不得绕过 baseline 或生成第二份 selection authority。
4. 先证明默认启动和 editor project-open 均把 provider registration 注入
   `AssetImporterRegistry`，再删除 `import_ui_v2_asset.rs`、ingest module registration 和
   `zircon.builtin.ui.zui` descriptor。删除后不得留下 forwarding module 或 builtin fallback。
5. 增加 hard-cut guard：production Asset source 不得引用 Runtime UI implementation；默认
   Client/Editor 的 `.zui` 路由必须来自 package id `ui_document_importer`，且 provider 未编译时
   必须给出明确 unavailable diagnostic，而不是静默回退。

### 算法与测量门

- Selection 合并规模为 baseline 条目数 `B` 与 project selection 数 `S` 的小集合操作；当前
  问题是 authority 分叉，不是这个循环的 CPU 热点。不得先微调 HashSet/Vec。
- correctness GREEN 后再采集 cold startup 和 project-open：provider resolution wall time、
  manifest clone bytes、registration count、`.zui` 首次 import wall time/bytes 和重复 open 的
  cache hit。要求 provider registration 恰为 1、fallback registration 为 0、asset-to-ui edge 为 0。
- Unreal 对照以 `UWidgetBlueprintFactory`/UMGEditor 的 editor/import owner 依赖 AssetTools/
  AssetRegistry 为准：UI authoring importer 位于高层工具/插件侧，Asset core 不反向依赖 UI
  implementation。该证据只决定依赖方向，不提供 Zircon 的耗时阈值。

### Owner 路由

- Frameworks05：Asset/Core dependency guard 与 fallback hard cut。
- Plugins UI document importer / first-party catalog owner：provider linkage 和 registration。
- App/Editor composition owner：effective manifest 与 project-open 注入。

## B. `ShaderModuleSourceBinding` 所有权

### Current-source 结论

- `ShaderModuleSourceBinding` 目前定义在
  `plugin/package_manifest/plugin_shader_permutation_manifest.rs`，但字段全部是 render-time
  source binding：owner id、import path、owned source、content hash 和 diagnostic origin。
- 直接类型名已被 graphics prepared resource、resource streamer 和 shader module registry
  使用；兼容别名 `PluginShaderModuleSource` 又贯穿 builtin assembly、extension registry、
  native loader、runtime plugin registration 和 render framework。影响面不止直接 import 文件。
- 文件明确保留 `pub type PluginShaderModuleSource = ShaderModuleSourceBinding` 的
  "Compatibility name"。这违反本轮 hard-cut 要求，也让 Graphics 长期依赖 Plugin manifest owner。

### 结构决策

1. 在 `core/framework/render/shader/` 建立 folder-backed `ShaderModuleSourceBinding` owner；
   serializable `PluginShaderModuleManifest` 继续留在 Plugin package manifest。
2. Plugin/native loader 只负责把 package-relative manifest 投影为 Core render binding；Graphics、
   builtin assembly 和 render framework 全部消费 Core 类型。
3. 在同一次迁移中删除 `PluginShaderModuleSource` alias 和 Plugin 根重导出，迁移全部直接/别名
   consumer，并增加 `graphics -> plugin` forbidden-edge guard。禁止临时双定义或桥接 trait。
4. 迁移前生成 current use-graph union 和 pre/post blob fingerprint；不能只按当前直接类型名的
   consumer 数估 scope。

### 算法与测量门

- 这是所有权/DAG 修复，不预设运行时性能收益。构造仍需只 hash 一次 source，并共享
  `Arc<str>`；迁移不得引入 source clone 或 render-time file IO。
- 验收测量 source registration count、bytes hashed、source clone bytes、module lookup P50/P95/P99
  和 shader assembly cache hit。结构迁移前后 workload 必须相同，alias 删除本身不作为性能收益。
- Unreal 以 RenderCore 的 shader source mapping、compiler definition 和 shader library owners 为
  主要参考：插件/模块提供输入，RenderCore 拥有运行时 shader source contract。该对照用于 owner
  边界，不用于宣称数值接近 Unreal。

### Owner 路由

- Frameworks01：Core/Graphics/Plugin DAG 与 hard-cut guard。
- Shader03/Render owner：binding 行为、hash/assembly/cache product tests。

### 2026-08-25 执行状态

- `git grep --untracked` 的 current use-graph union 覆盖 27 个精确 Rust blob；ownership
  transfer-preview 指纹为
  `a3d617de198dc859ce2fac004dbe619c4786b3afca2da9c26f18b7fac52c55d5`。
- 其中 26 个 blob eligible；唯一不可转移项是
  `graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs`，其
  source owner `mvp00-current-source-convergence-r2-01a00797-20260818` 仍为 `active`，
  blocking reason 为 `source_owner_executable`。
- Frameworks01 未 apply transfer、未改写任一 Shader consumer，也未建立临时 Core/Plugin
  双 owner。待该精确 consumer 释放或完成协调器 ownership transfer 后，按上述 27-file union
  重新确认 current hash，再原子执行本节 hard cut。

## C. 全局 IK command queue

### Current-source 结论

- Runtime 与 first-party animation plugin 各自维护一份
  `Mutex<HashMap<WorldHandle, WorldIkCommandQueue>>`、4,096 command limit、replacement epoch 和
  `mem::take(...).partition(...)` drain；两份 manager/service identity 重复。
- tracked production source 有 drain consumer，但 `queue_ik_command` 的调用全部是测试，当前产品
  producer 数为 0。现有 ignored benchmark 只比较线性 membership 与 borrowed `BTreeSet` lookup，
  没有覆盖 map/mutex、World lock、partition allocation、IK application、frame time 或 energy。
- 每个 ticked Level 即使没有 command 也会通过 `entry(world).or_default()` 留下空 slot；Level handle
  单调增长且没有 unload callback。内存下界为 O(L) map state，上界可保留 O(L * 4096) command slots。
  n 条 command、d 个 deferred entity 的 membership 为 O(n log d)，但全部 World 仍通过一个全局锁。

### 结构决策

1. 当前 producer 为 0，所以 MVP 先 hard-cut global queue API、两份 queue state 和重复 Runtime
   manager/module registration；不要优化一个没有产品输入的容器。
2. TwoBoneIK、LookAt 和后续 skeletal control 应成为 animation graph/evaluation instance 的直接
   input/output，并复用 graph-local output storage。
3. 只有新的外部 producer contract 被产品 trace 证明后，才允许引入 replacement inbox；该 inbox
   必须 Level-lifecycle-owned、bounded、随 Level 移除，并先测 contention/deferred ratio 再选容器。
4. Unreal `FAnimNode_TwoBoneIK::EvaluateSkeletalControl_AnyThread` 与
   `FAnimNode_SkeletalControlBase` 的可复用 `BoneTransforms` 支持 graph/evaluation-local ownership，
   不支持 Zircon 当前 process-wide manager queue；参考证据不等于性能数据。

### 性能门

- zero-command：0 map insertion、warmup 后 0 queue allocation、0 global queue lock acquisition。
- 如未来存在 producer：1/4/16 Worlds，0/256/4,096 commands，0/50/100% deferred，epoch rollover
  和 concurrent producers；记录 enqueue/drain lock wait/hold P50/P95/P99、allocations/bytes、
  commands retained/admitted、animation tick CPU、frame time、RSS 与 Windows ETW/WPA energy。
- 只有同 workload 的 before/after capture 才能声称瓶颈消失；membership microbenchmark 不能用于
  端到端或功耗结论。

### Owner 路由

- Runtime08C / Plugins04：product trace、重复 manager hard cut 与 graph-local IK contract。
- Frameworks01：neutral animation contract 和 `scene -> animation` boundary guard 保持不回退。

## 实施顺序与退出条件

1. UI importer effective-selection + provider linkage + fallback hard cut，形成 Client/Editor `.zui`
   最小可用闭环。
2. Shader source binding 原子 owner 迁移，消除 Plugin compatibility alias 和 Graphics 反向边。
3. IK global queue dead-capability hard cut；仅在真实 producer 出现后重开 inbox 设计。

每项进入实现前必须重新获取 current ownership matrix、租约与 blob hash。任何 active foreign owner、
baseline drift 或 hash 漂移都必须先协调 transfer；不得借本记录扩大当前 r9 exact scope。三项分别取得
production compile、focused/upward tests、fresh dependency audit、独立复审和 coordinator milestone
commit 后，才可把状态从 `implementation_not_started` 提升。

## 2026-08-23 current-source 验证记录

- 在 HEAD `68edcd71042de817a74d4ad70efc07cfe2c72bfa` 上，fresh
  `python -B -m unittest tools.tests.test_frameworks_01_runtime_error_owner_boundary
  tools.tests.test_frameworks_02_core_error_single_source -v` 为 2/2 GREEN，测试本体耗时
  56.395 秒。
- fresh `python -B -m unittest tools.tests.test_frameworks_01_scene_animation_boundary -v`
  为 9/9 GREEN，测试本体耗时 43.246 秒；这只证明 current source 的 neutral identity、
  replacement epoch、backpressure 和 hard-cut source-shape 合同，不替代 Rust product gate。
- fresh `python -B -m unittest tools.tests.test_frameworks_03_server_feature_boundary -v`
  为 14/14 GREEN，测试本体耗时 3.743 秒，覆盖 server feature closure、physical RHI split 和
  dotted-key reverse-dependency detection。
- fresh `python -B tools/runtime_domain_dependency_audit.py --repo-root .` 成功，结果为
  2,749 production references / 72 domain edges；其中 `asset -> ui = 1`、
  `graphics -> plugin = 12`，与本记录 A/B 的结构性 owner 判断一致。
- 第一份 exact4 materialization request `b4592532f00a4e0f8c51d5096fdb91ac`
  （job `423dc385d9934db18af21304e621ae3f`）在 `closure_planning` 阶段因
  `validation_copy_external_source_missing` 终止。第二份 request
  `bab76dfbabb94babb64c1e08ca12b7d0`（job
  `7fb51dc6a5834bcbbd655c14f86de263`）已把 `E:\Git\zr_vm` 固定到 commit
  `160b6591260ee00295d538e91e95d56e7c7022a3`，source hash
  `9fe456b56dca4d04068eb2650ef55d37877714abfe6c35592f8acfeb8e6e6961`，但在
  `materialization_prepare` 因同五份 Runtime74 UI binding blob 的
  `validation_copy_baseline_drift` 终止。两次均未生成 input manifest、未启动 Cargo，
  所以只构成 current-source attribution 阻塞证据，不构成 neutral RHI RED。
- 本记录自身 `git diff --check` GREEN；初次落盘 SHA-256 为
  `65161ff1c8dfa44619034e214a17675e0b8b7b795f22168253b1b2bf21c2019e`。本节写入后必须
  重新 attribute 最终 hash，旧 hash 不得用于 closeout snapshot。

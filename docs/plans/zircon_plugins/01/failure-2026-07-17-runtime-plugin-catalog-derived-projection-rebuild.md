---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: runtime-plugin-catalog-derived-projection-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
tests:
  - runtime plugin catalog projection build-count regression
  - 1/100/1000 feature graph and 1/100/10000 candidate transaction scaling regressions
  - project completion/report/extension byte-and-order parity matrix
---

# Plugins01：runtime plugin catalog 派生投影重复重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP plugin runtime-plugin-catalog/builtin-catalog 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：package/module/feature/provider/dependency identity 与稳定顺序属于 catalog generation 的统一派生状态，completion、report、extension、lifecycle 各自缓存会产生多个失效 authority。

## 失败现象与复现证据

`runtime_plugin_catalog` 87/87 个 child Rust 文件已逐文件静态通读。`feature_definition_map` 在 `feature_manifest_for_selection` 每次查询、project feature completion 和 feature dependency report 中分别重建；即使 project extension path 已直接去掉一次 completed-manifest 二次 completion，completion 与 report 仍各构建一次完整 feature definition map。

project selection default completion 以 registrations×selections 互扫并返回 owned selection clone；owner feature completion 以 selections×definitions 查找，再以 definitions×selections 补 external provider。feature dependency fixed-point 反复全扫 pending，成功/立即失败时 `Vec::remove(index)` 触发移位；cycle 判断又以 missing capabilities×all definitions 扫描。available feature merge 对每 feature 扫 feature registrations，registration match 再扫 manifest selections/features。

bridge dependency closure 从每个 package root 做 DFS；module→provider lifecycle lookup 会对每个 registration 再调用一次 provider module scan并分配 module-name Vec。catalog builtin/profile consumers 还会重复构造整个 descriptor/catalog。上述行为都位于 bootstrap/editor change/export/hot-reload generation，而非稳定 frame，但会放大 MVP 启动和编辑器插件交互延迟。

## 最低共享层根因

`RuntimePluginCatalog` 只保存 owned registration Vec，没有与 mutation generation 绑定的 immutable derived projection。身份索引、稳定顺序、feature graph、provider/module map 与 diagnostics dependency graph由各 consumer 临时重建，无法统一预算或精确失效。

## 架构修复验收

- 每个 catalog generation 至多构建一次 ordered derived projection，覆盖 package id、runtime module→provider、feature/provider definition、selection default、feature registration 与 capability dependency graph。
- completion、single-feature lookup、dependency report、extension merge、bridge closure/lifecycle 共用 projection；`register/register_feature/hot reload` 只使下一代精确重建一次。
- feature availability 使用有序图/入度或等价 work queue，1/100/1000 feature 的总访问为 O(V+E)，不得在 pending Vec 中反复 remove/全扫。
- 首次声明与 manifest 原始顺序继续决定 selection/report/diagnostic/extension 顺序；byte/order parity 全通过。
- 产品启动、editor plugin toggle/hot reload 与 export trace 记录 projection build count、wall、allocation；确认 projection 不进入 frame/tick。

## 禁止临时方案

- 不得给 completion/report/lookup/lifecycle 各加独立 memoization。
- 不得用无序容器迭代改变可观察顺序，或在 cache hit 时跳过 diagnostics。
- 不得把 builtin catalog 设为永不失效的全局静态并忽略 native/hot-reload generation。
- 不得把注册期问题描述成逐帧热点；优先级由 MVP 启动/editor interaction 规模证据决定。

## 修复结果与回传

当前状态：`independent_review_returned_important_3 / implementation_rework_in_progress / managed_validation_not_authorized`。

### 已完成项目

- `RuntimePluginCatalog` 已持有与 catalog generation 绑定的 ordered derived projection，package/feature registration、definition、runtime module/provider、target capability 与 bridge dependent 查询共用同一代索引。
- feature resolution 已改为稳定顺序 ready-set/work-queue，并保留原 manifest/selection 诊断次序；projection 提供 build-count、entry count、string bytes 与 build wall 指标。
- project completion、extension report/merge、bridge lifecycle 与 builtin catalog consumers 已迁移到共享 projection；不再各自重建 feature definition/provider 查找表。
- `feef12b0258748eda07e3c630d732585` 暴露的 `feature_blocking.rs` `E0505` 已在最低 owner 修复：终局 cycle 检测只构建一次 owned `HashSet<String>`，随后可按值消费 pending 状态；复杂度保持 O(V+E)，无额外 catalog 扫描或顺序变化。
- 15:49 的共享格式化/投影更新改写 20 个 manifest 输入后，旧 snapshot 由 owner 释放；当前完整 134-file catalog/builtin source manifest fingerprint 为 `42e3828de67f025a527cb5eb12eb491083e44a0cbe5fc3eee6cfbc0401f551f9`。既有 scoped `rustfmt --check`、`git diff --check` 与结构预算检查通过，fresh gate 将覆盖合并后的当前字节。
- 早期 manifest 的 Critical `0` / Important `0` 结论已被后续 exact55 复审取代，不得用于当前源码验收；
  R6 修复后仍需在 fresh GREEN 前登记新的最终复审。

### 待完成项目

- 修复前 reservations `3f5829dd820d42fdbe0ccae4e22dc247`、`f002395f97fb4190a3d66602f77c275a` 已由 owner 释放，避免 source-manifest stale；fresh canonical reservation `010630b4f96848a28febb4c9fb09b8db` 已按合并后的当前源码重新登记并等待 FIFO。
- focused catalog GREEN 后仍需 runtime/plugin broad gate、最终独立 review、failure fixed return 与 coordinator milestone atomic commit。

该 fresh reservation 未绑定 job 并已 expired；下述更新的 managed binary 证据取代它。

### 2026-07-22 current-source focused 证据

managed job `93f88e221e244b93b176afa90a07cdff` 保留的 test binary（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）执行完整
`runtime_plugin_catalog::derived_projection::tests` 过滤组为
`7 passed / 0 failed / 4303 filtered`，覆盖单代 projection build、register generation 推进、
O(V+E) feature resolution 以及 completion/report/extension JSON byte-order parity。Runtime/plugin broad gate、
最终复审、failure return 与 milestone review 仍待完成，因此保持 `open`。

`tests::plugin_extensions` broad gate 进一步发现 rendering generated manifest 已声明
`volumetric_fog` / `oit` / `light_cookies` / `irradiance_volumes` / `planar_reflections` /
`subsurface_scattering` 六个已存在的 runtime/editor feature crate，但
`RENDERING_FEATURE_ROWS` 仍只有旧 9 项，破坏了 manifest 单源和 catalog completion。现已按
generated manifest 顺序把六项加入 Rust descriptor authority，并把 manifest parity expected 更新为
15 项。静态比对确认 Rust row IDs 与 `rendering/plugin.toml` `15/15` 字节顺序一致；
fresh compile/focused/broad 复验前不声明 current-source GREEN。

### 2026-07-22 registration与project generation增量证据

性能审计继续逐文件阅读当前`runtime_plugin_catalog/**`生产84/84、`builtin_catalog/**`44/44、descriptor 13/13、feature registration report 8/8和registration report生产13/13。现有单代projection与O(V+E) feature ready-set保持；新增两项局部止损：bridge provider reload借用registry export并复用replacement引用批次（PERF-MVP-535），feature capability在最终owned index/set插入前保持borrowed（PERF-MVP-536）。源码RED→GREEN、rustfmt与diff检查已通过，current-source Cargo因受管CPU lane被其他Session预约仍待执行。

本failure不得因已有projection而提前关闭，新增剩余验收如下：

- `register`/`register_feature`及hot reload多项变更必须经candidate transaction一次验证、一次projection+diagnostics build、一次generation publish；N项变更不得发布N代或重扫N次全catalog（PERF-MVP-537）。
- `builtin()`/`package_manifests()`不得因查询深clone完整registration/manifest/diagnostics；project completion、dependency report与runtime extension merge必须在catalog generation+canonical project fingerprint+target上共用一个`CompiledProjectPluginPlan`/frozen extension generation（PERF-MVP-538，并与PERF-MVP-533同一authority）。
- stable重复请求的catalog/report/manifest/registry deep-clone bytes与plan build count必须为0；toggle/reload只发布一个changed generation，失败维持last-good，旧代由in-flight `Arc`安全延寿。
- 1/100/10k plugins/features/modules/interfaces/contributions记录build count、rows/edges visited、clone bytes、alloc/RSS/wall；F0启动、F4插件页/toggle/hot reload产品trace与broad Cargo完成后再执行最终复审和fixed return。

### 2026-07-30 frozen project-plan 实施状态

- `plugins01-catalog-project-plan-r1-20260730` 已取得 catalog project/extension、feature dependency、
  project manifest、catalog feature test 与本 handoff 的精确 lease。
- 当前 `register_reports_batch` 已一次 candidate batch 发布一代 projection；但 `project.rs`、
  `feature_dependencies.rs` 和 `runtime_extensions.rs` 仍分别调用 manifest completion，且
  `runtime_extension_report_for_project` 每次新建 registry。当前源码没有
  `CompiledProjectPluginPlan` 或等价的 catalog-generation + canonical project fingerprint authority。
- `CompiledProjectPluginPlan` 已以结构化无分配 manifest fingerprint、源 manifest 等值复核、target 和 catalog
  generation 的本地 immutable authority 缓存 completed manifest、feature report 与 extension report；相同请求复用一个 plan，且
  `publish_candidate_generation` 在新一代 projection/diagnostics 发布后清空 cache。metrics/regression 已断言
  stable 请求只建一次、batch registration 后下一请求只建一个新 plan。
- `CompiledProjectPluginPlan` 现持有冻结的 `Arc<RuntimeExtensionCatalogReport>`；
  `runtime_extensions_for_project` 在 cache hit 仅复制 `Arc`，不再深 clone registry。动态 session 的
  `LinkedRuntimePluginPlan` 持有同一 report generation，并经只读 `registry` facade 完成模块、scene hook
  与 world extension 安装，未再移出或复制完整 registry。
- project-plan cache 已从 `RefCell` 收敛为 `Mutex` + `AtomicU64`：相同 manifest/target 只由一个建造者发布，
  catalog clone 使用独立空 cache，避免跨 catalog generation 共享可变 cache。新增回归契约断言稳定请求
  `Arc::ptr_eq`、generation publish 后旧在飞 snapshot 继续有效且新请求获得新 snapshot，并保证 catalog
  仍为 `Send + Sync`。
- scoped Rust `1.94.1` formatting 与 `git diff --check` 已通过。用户指定的 validation copy
  `5945e3ef29d74bd69602adca02e243b5` 属于外部 session；本 session 的 read-only status 请求被 coordinator
  以 `validation_copy_foreign_session` 拒绝，因此没有重新 materialize、cleanup 或启动 Cargo。该副本及其
  owner 的 FIFO 运行结果只能作为外部 descriptor 证据；本次 current source 仍需由 coordinator-managed
  Windows Cargo 验证类型、diagnostic byte/order parity、generation failover、snapshot sharing 与
  runtime/plugin broad。handoff 保持 `open`，不得提前 fixed return。

### 2026-07-30 current-source world projection ownership repair

- accessor recovery 的 managed job `284761d5165d4661b4a08290f4a2fb28`（default-feature
  `profile_availability_projection`）已自然 release，终态 `exit 101`。在执行任何目标测试前，唯一
  current-source error 为 `dynamic_api/session/construction.rs` 的 E0596：共享
  `Arc<RuntimeExtensionCatalogReport>` 不能借出可变 `registry` 来调用 `apply_to_world`。
- 根因是 project extension report 在 catalog generation 构建期已经 `finalize()`；world 注册必须从该
  immutable registry 派生每个 runtime session 的 `WorldRuntimeExtensionPlan`，而不是修改缓存 report。
  `construction.rs` 现在先调用 `world_runtime_extension_plan()`，随后把该只读 plan 应用到新 World；report
  和 registry 均不再移出、深 clone 或经 `Arc` 可变访问。
- 当前 consumer 文件的 Rust `1.94.1` scoped `rustfmt --check` 与 `git diff --check` 已通过。两次 managed
  acquire dry-run 分别返回 CPU lane 已为其他 session 保留和 coordinator timeout；没有创建 job、重试/清理
  外部 validation copy，亦未把静态检查记为 Cargo GREEN。下一步仍是 FIFO 许可下的 fresh current-source
  incremental compile，之后再执行 catalog/runtime broad 与 failure return。

### 2026-07-30 exact55 独立复审返工

- `plugins01-catalog-atomic-update-r5-20260730` 的 exact55 immutable manifest 已由独立 reviewer 在开始、结束和
  二次结束三个原子点复算一致：HEAD `77b3b857f82ff121c9727ffabfc8ae65ec9e3357`，55 paths，5 deleted，
  aggregate 8191 bytes，fingerprint
  `2e7cb643e813d5459284f5510485d6c6a97050ad0f5104757802da613aa102d4`。评审结论为
  Critical `0` / Important `3` / Minor `0`，因此该 manifest 明确不得进入 managed acceptance。
- Important 1：事件队列版 feature fixed-point 必须恢复旧首遍声明顺序语义。早到的 available provider 可以影响
  后续 immediate blocker；已经 immediate-blocked 的行不得被后到 provider 改写，也不得进入 unresolved provider
  集合制造假 cycle；immediate block 必须先于终局 unresolved block 输出。
- Important 2：candidate transaction 的 replace/remove 不得按 mutation 重扫完整 Vec。R6 将在首次对应域 mutation
  时一次构建 identity index，以 stable slot+tombstone 执行全部操作，最后一次 materialize；空 transaction 不 clone
  任一 registration row，1/100/10k 规模必须证明 indexed rows 与 catalog size 同阶而非 mutations x catalog。
- Important 3：`CompiledProjectPluginPlan` 必须同时冻结共享 completed manifest、feature report 与 extension report。
  cache hit 不得再次序列化 manifest 或深 clone report；canonical fingerprint 改为结构化无分配计算并以源 manifest
  等值复核，公开 completion/report consumer 硬切到 `Arc` snapshot，不保留 owned compatibility overload。
- successor `plugins01-catalog-atomic-update-r6-20260730` 已接管扩展后的 exact71 code/doc lease，并保持
  `resolving_failure`。当前仅因 foreign full-lib Cargo compile 保持 Rust 输入冻结；期间继续完成 TDD、consumer 和
  performance counter 设计，不把队列等待当成 blocked。三项 Important 修复、fresh immutable manifest、独立
  `C0/I0/M0` 复审、managed focused+broad GREEN、fixed return 与 coordinator atomic commit 全部完成前，本 failure
  保持 `open`。

### 2026-07-30 R6 current-source compile boundary

- source-raced diagnostic job `d191f9ea6be24ac1b9fb3861bc843e94` 仅暴露三个已删除的一参数
  `complete_project_manifest` 测试调用；调用方已全部硬切为显式 `RuntimeTargetMode`，未恢复 overload 或 alias。
- Text09 full-lib job `eb62a140fead4bfe9848c4d751f8a0d5` 在 R6 最后写入前十秒启动，因此不得作为 acceptance；它随后以当前
  Catalog API 完成整库编译并运行目标测试 `1 passed / 0 failed`、`exit 0`、无 live PID，可作为当前编译边界的
  diagnostic evidence。fresh exact71 review 与本 owner 的 managed focused/broad gate 仍是验收前置。
- 首遍声明顺序修复后，正向依赖链会在 initial scan 直接完成而不进入 ready queue；规模回归已把该计数锁定为
  `pushes=0/pops=0`，并在反向边用例锁定唯一受影响 waiter 的 `pushes=1/pops=1`，避免用旧队列语义制造假 RED。

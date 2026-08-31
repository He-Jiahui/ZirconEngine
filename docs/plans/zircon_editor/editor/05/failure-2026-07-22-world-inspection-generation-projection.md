---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-inspection-generation-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/mod.rs
tests:
  - cargo test -p zircon_runtime --lib inspection --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib editing::editor_projection --locked --jobs 1 -- --nocapture --test-threads=1
---

# Editor05：World inspection generation projection交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene inspection 5/5逐Rust文件性能审查，PERF-MVP-456
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：Editor05拥有Hierarchy/Inspector/viewport edit-mode projection与F4启用边界；Editor02共同拥有generation/delta消息，Runtime07提供World change projection。
- 生命周期键：`world-inspection-generation-projection`

## 失败现象与复现证据

Runtime每次inspection全量`node_records()` project/clone/sort，重建多份hierarchy/hash容器；selected fields全扫TypeRegistry并复制metadata/value。editor test-only consumer再构造第二套owned hierarchy/inspector DTO，`build_stats`第二次`node_records()`全场扫描。`generation/subtree_hash`当前只是输出字段，没有阻止stable generation重建。

本轮只删除focus第二遍、parent BTreeMap、field-name临时clone并预分配基础容器；这些局部止损不允许直接解除editor consumer的`cfg(test)`门。

## 最低共享层根因

World没有发布按hierarchy/name/active/reflection generation封存的inspection artifact/delta；Editor05各consumer也没有共享同一projection owner。Runtime与Editor两套owned DTO让cache放在哪一侧都会形成第二份truth。

## 架构修复验收

- Runtime07按world hierarchy/name/active/type/component generation发布immutable inspection row/field artifact与added/changed/removed delta；subtree hash只重算changed row到ancestor chain。
- Editor05 Hierarchy、Inspector、viewport stats共享同一artifact；stable generation零producer build/scan/clone，selection-only只切field projection，不重建hierarchy。
- Editor02传递generation/delta与backpressure，不按idle frame重复请求全量snapshot；consumer落后时按generation合并并可显式请求一次resync。
- 删除editor第二套完整runtime DTO复制或把它收敛为borrowed/Arc view；解除`cfg(test)`必须由F4产品trace与Cargo门共同批准。
- 1/1k/10k/100k nodes、depth 1/64/5k、types/components 1/100/10k及stable/rename/reparent/selection/field edit记录node/type/subtree visits、clone bytes、build/delta/resync count、queue age与p95；工作随dirty范围而非total scene或frame count增长。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅在Editor缓存完整WorldInspection而没有精确Runtime generation/delta；禁止Runtime/Editor各持一份独立authoritative hierarchy。
- 禁止用固定帧节流掩盖全量重建；变更延迟/合并必须有generation与最大age语义。
- 禁止在未完成规模/Cargo/F4验收前直接移除`cfg(test)`。

## 性能调查与结构性优化计划（2026-08-22）

状态：`架构调研完成 / Windows profiling baseline pending / 不得先行优化`。当前
`profile_scope!("editor", "hierarchy", "filter_projection")` 是可用的采样边界，但没有
profiling editor binary；此前受管 `zircon_editor --profile profiling` job 在依赖下载期被
协调器回收，未产出可比较的 CPU、allocation 或功耗数据。因此下列结构判断是源码证据，不
把它写成已测量的性能结论。

### 已确认的结构性风险

- runtime 的 `World::inspection_artifact()` 已按 generation 复用同一 `Arc`，它是 hierarchy
  rows/delta 的唯一 runtime authority；筛选不能回迁到 runtime 或创建第二份 hierarchy truth。
- editor 的 `filtered_hierarchy_entries` 每次创建 parent-index、included bitmap 和新的
  `Vec<WorldInspectionHierarchyRow>`；row clone 同时复制 name/kind strings。一个 UI recompute
  会在 presentation 和 pointer-surface 两条路径分别调用该函数，筛选结果不是共享产品。
- filter 激活时 `scene_hierarchy_refresh` 放弃 sparse fragment 并要求 authoritative reflow；这
  保证正确性，却把 query 和 generation 更新耦合为 full filtered projection。当前算法不应在
  没有采样数据时被误改成局部字符串技巧或 fixed-frame throttle。

### 参考系统与采用边界

- Unreal `SceneOutliner` 将 `SearchBoxFilter`、filter collection 与 tree refresh 分开；输入框
  的延迟通知和 filter changed 均只触发 outliner view refresh（`SSceneOutliner.cpp`），不把
  search state 写入 world。
- Godot `SceneTreeEditor::_update_filter_helper` 遍历已存在的 tree items，并保留命中节点的
  ancestor visibility（`editor/scene/scene_tree_editor.cpp`）；`SceneTreeDock` 仅把 query
  交给 scene-tree view（`editor/docks/scene_tree_dock.cpp`）。
- Zircon 的落点保持 editor-owned derived view：runtime 继续发布 immutable rows/generation/
  delta，`zircon_editor::ui::retained_host` 拥有 query、matching 和 view cache。这比复制 Unreal
  的 retained C++ item graph 更符合当前 Rust `SceneEntries(Arc<[Row]>)` 边界。

### 待实现的通用产品（在 baseline 后）

1. 在 `ui/workbench/snapshot/data/scene_entry/` 建立非物化的
   `HierarchyFilterProjection`：保存 source generation/identity、normalized query 与可见 row
   indices；它引用同一个 `Arc<[WorldInspectionHierarchyRow]>`，不 clone rows/string，也不成为
   runtime authority。
2. retained host 为 `(source generation, normalized query)` 缓存一个 projection；presentation、
   pointer layout 与 hierarchy reflow 必须消费同一份 projection。query 或 source generation
   改变时只失效该 cache，selection-only update 不重扫 hierarchy。
3. 先以一次 O(N + matched-ancestor-links) projection 保证正确性；generation change 时最多
   重新计算一次。只有测量证明它仍越过预算时，才以 runtime delta 的 changed subtrees 驱动
   增量更新。不得预先加入 timer、frame throttle、第二份 rows 或 feature-specific branch。
4. 将 row-consuming APIs 从连续 slice 假设收敛为 projection iterator/row accessor，避免为了
   `Deref<[Row]>` 兼容而重新 materialize filtered rows；模块仍遵守 owner-leaf + thin facade
   规范，不把 cache、matching 和 retained host lifecycle 堆回一个文件。

### 采样设计与准入门

Windows profiling run 必须使用 `tools/ui-profile-capture.ps1` 的 hierarchy fixture，输出仅放在
`E:` 或 `F:` managed target/profile root。每个场景采集 warm-up 后不少于 30 次：1k、5k、10k、
100k rows；flat、depth 64、depth 5k；blank、zero-match、single deep match、all-match query；
stable generation、rename/reparent generation、selection-only update。记录
`filter_projection` CPU p50/p95、allocation bytes/count、rows scanned/matched/ancestor kept、
projection cache hit/miss、presentation/pointer reuse count、full reflow count、frame time and CPU
package power when the profiler exposes it。

现有 capture 的 `hierarchy_scroll` 只发送 wheel interaction，不能触发 text filter；它不可作为
本项的 proxy。先在 profile tool 增加 `hierarchy_filter` 场景及 geometry-bound text input（不得使用
固定坐标或手工操作），由该场景 materialize hierarchy fixture、提交 deterministic query sequence
并导出本节 counters。该 profiling-tool 前置不改变 editor 筛选算法。

允许开始实现的条件：先得到 current-source profiling binary 和上述 baseline；若 p95 或 allocation
显示该 projection 是目标路径的主导项，按本节的 shared projection design 实施并以同一矩阵比较。
若不是主导项，保留结构证据并把优化转向实际热点。实现后不得宣称与 Unreal 的功耗或耗时相当，
除非相同场景、硬件和采样方法已有量化对照。

## 修复结果与回传

Open state: `source repair complete; Runtime->Editor boundary, scale, and F4 product validation pending`; no pass is claimed.

- Runtime now publishes one immutable `Arc<WorldInspectionArtifact>` per world generation, reuses
  the hierarchy allocation for stable and field-only generations, and retains only the current
  primary-selection field artifact. Its diagnostics expose hierarchy and focused-field build work.
- The production Editor host retains the runtime artifacts only long enough to publish a
  generation/delta message; Hierarchy and viewport projections consume the shared row allocation
  instead of reconstructing an authoritative DTO copy.
- Current source and focused tests cover stable-generation reuse, field-only deltas, selection
  changes, removal, and the application-level Hierarchy -> Inspector -> transaction -> save ->
  reopen flow. The declared Cargo boundary, 1/1k/10k/100k scale batch, and F4 product trace remain
  required before this handoff can return as `fixed-*`.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-08-23 +08:00 | `profiling_trace_export_hardened / contract_validation_complete / managed_product_capture_pending` | 新增 `ui-profile-hierarchy-filter-metrics.ps1`，从实际 `editor/hierarchy/filter_projection` span 导出 p50/p95，并汇总 `projection_invocation`、source rows、name matches、ancestor links、visible rows 五项 Rust counter 为 `hierarchy_filter_metrics.json`。严格 `hierarchy_filter` gate 要求至少一个 span 和全部五项 counter；合法零匹配、零 ancestor、零 visible row 样本保留，不会被误当缺失。复核发现 blank query fast path 原先遗漏全部 counter，现已与普通路径共用单次记录：source/name-match/visible 均为全量行、ancestor 为 0，保持返回结果与算法不变。capture 编排、source-bound manifest 与 output-contract fixture 均已纳入 helper，工具清单为 11 项。后续复审又将指标汇总收敛到唯一的有限、非负输入过滤：缺失、空、NaN、Infinity 或负值不再被 PowerShell 转换为伪零样本。 | 专项 Pester 为 6/6：完整 trace 放行、缺 visible-row counter 拒绝、缺失 trace 值拒绝、非有限或负值拒绝、capture/manifest 接线及 11 项工具集均通过；blank-query Rust source regression、PowerShell AST、rustfmt、source contract、scoped diff 检查通过。完整 `ui-profile-capture-output-contract` 在桌面 60 秒上限内先发现旧工具计数 `10`（实际应为 `11`），已将其修为 11 并显式断言新 helper；修复后整套尚无终态，不能记作全套通过。2026-08-23 新受管 Cargo `623131f38d5346468e1d39f1c768429f` 已下载依赖并启动 `proc-macro2`、`quote`、`unicode-ident`、`syn` 编译，随后因 `cargo_process_tree_alive` 进入 `finish_blocked`；进程树确认退出后已由 coordinator 释放，job `exit_code` 仍为 `null`，没有 test output，故不能记作 Cargo 通过或失败。此切片不改筛选算法、不产生 CPU/allocation/frame/GPU/功耗基线；恢复 profiling editor binary 后按本文件 1k/5k/10k/100k 与 30+ 次矩阵采样。 |
| 2026-08-22 +08:00 | `architecture_review_and_instrumentation_complete / profiling_baseline_pending` | 复核 runtime immutable artifact、retained-host filter、presentation/pointer consumers 及 Unreal/Godot scene tree，形成 shared projection 的 owner、算法、模块边界和准入采样矩阵；为下一次 trace 加入 source rows、name matches、ancestor links、visible rows 与 invocation telemetry。 | telemetry 的 rustfmt、diff 和生产源码契约已通过；没有 profiling editor binary、Cargo test 或可比较的 CPU/allocation/power 数据。任何 hierarchy filter 实现优化须先完成本记录的 Windows managed baseline。 |
| 2026-08-23 +08:00 | `profiling_input_contract_audited / hierarchy_filter_scenario_pending` | 已确认资产控件 `WorkbenchSceneSearchField` 由 host 编辑入口 `HierarchySearchQuery` 接收 query，geometry artifact 对外的唯一可命中 ID 是 `template.left.HierarchySearchQuery`；现有 scenario registry 只有 `hierarchy_scroll`，其自动化器只发送 pointer/wheel，不能以受控文本驱动筛选。 | 不使用固定坐标、手工输入或 scroll proxy 伪造 baseline。后续 scenario 必须从当次 `ui_profile_geometry.json` 定位搜索框，发出确定性 Unicode 查询序列，并在 interaction evidence 中记录 control id、query 与 geometry refresh；然后才能按既定 1k 至 100k、30+ 次矩阵采样。 |
| 2026-08-23 +08:00 | `profiling_input_infrastructure_complete / managed_product_capture_pending` | 新增 `hierarchy_filter` profile scenario；它复用 hierarchy logical-node fixture，严格从当次 geometry 选择唯一的 `template.left.HierarchySearchQuery`，点击后先以 Windows `SendInput` 发送 Ctrl+A/Backspace 清空已有文本，再注入配置 query 的 UTF-16 code units。P/Invoke 由当前进程架构对应的 `INPUT` ABI 大小测试约束；manifest 绑定输入模块与 query，evidence gate 必须同时确认 target provenance、精确 query、reset event 数、请求/发送 code-unit 数和交互后的 fresh geometry epoch。 | `tools/tests/ui-profile-hierarchy-filter-input.Tests.ps1` 5/5、`tools/tests/ui-profile-capture-output-contract.Tests.ps1` 42/42、PowerShell AST parse、source contract 与 `git diff --check` 通过。此前 coordinator register 错误已不再复现，dry run 可分配 F: ephemeral lane；本次 `hierarchy_filter` focused Cargo 的 stdout 被桌面 64 秒外层时限截断，后续进程与 lane 已退出/清理但无法取得 exit code，故不声明 Cargo 通过或失败。尚无 profiling editor binary，以及 CPU、allocation、frame 或 power 数据；恢复可观测的 managed run 后按本文件定义的 1k/5k/10k/100k、30+ runs 矩阵采样，baseline 前禁止改写 hierarchy filter 算法。 |

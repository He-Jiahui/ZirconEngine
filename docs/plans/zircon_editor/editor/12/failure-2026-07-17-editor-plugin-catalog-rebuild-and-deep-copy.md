---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-plugin-catalog-rebuild-and-deep-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/manager/lifecycle_replacement.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/native.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs
tests:
  - editor plugin catalog generation build-count regression
  - plugin-manager recompute package/capability clone-count regression
  - plugin enable-disable/hot-reload ordering parity
---

# Editor12：editor plugin catalog 重建与 owned projection 复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：editor core plugin/catalog 与直接 UI 调用方静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：runtime/editor catalog generation、插件 UI projection 与 hot-reload invalidation 必须由 Editor12 单一管理。

## 失败现象与复现证据

plugin manager 的 `editor_plugin_catalog()` 从 runtime package manifests 重新构造 builtin editor catalog；多个 status/enablement/project/manifest-completion consumer 随后反复调用 `package_manifests()` 与 `capabilities_for_package()`，分别深 clone manifests/capability strings。`from_descriptors` 对每个 descriptor 线性扫描 runtime manifests；`editor_extensions()` 每调用一次都重建 extension/asset registry并 clone 所有 descriptor/contribution。调用链位于插件管理 UI 的 recompute/enablement 路径，当前没有 generation cache 或 build-count 证据。

extension 注册本身也随已注册数量重复工作：每个 plugin 先 clone 完整 `EditorCommandRegistry` 做事务候选，分别五次重扫既有 drawer/menu/component/template/importer ids，重建 available-operation set，并从 builtins 重放全部 prior asset contributions 做冲突验证。批量 bootstrap 因此存在 registrations×existing-catalog 的二次增长候选。

native plugin 管理 UI 的只读 status 路径调用 `NativePluginLoader.load_discovered_all`，会重新递归发现并实际加载 runtime/editor 动态库、执行 descriptor/entry 路径，然后又重建 builtin status 与临时 runtime catalog。native-aware enablement 先 discover 一次取 package，随后 manifest completion 内再次 discover；feature/packaging/target-mode 操作也各自重建 catalog/completed manifest。也就是说，稳定面板刷新和单次 toggle 都可能重复文件系统扫描、manifest parse、foreign library load/callback 与全量 projection。

Retained host 451-file 审查确认这条只读路径位于可见 pane 的 slow recompute：`module_plugins_pane_data` 每次重新解析 project root/`zircon-project.toml`，随后调用 `native_plugin_status_report` 并完整 materialize plugin rows、capabilities/features/diagnostics。visibility gate 只避免隐藏 pane，无法让 unchanged visible pane 命中 generation cache。

## 最低共享层根因

editor plugin catalog 不是由 runtime/editor plugin generation 持有的稳定投影；API 主要返回 owned collections，迫使 UI consumer 重建 catalog 或深 clone manifests/capabilities/extensions。

## 架构修复验收

- runtime/editor plugin generation 只构建一次 immutable ordered editor catalog projection，UI consumers 共享借用/`Arc` rows。
- package id、capability、extension 与 asset contribution 建有序索引；lookup 近 O(1)/O(logN)，不返回全量 owned clone。
- register/enable/disable/hot reload 精确递增 generation；单次 UI recompute build count 为 0（未变）或 1（变更）。
- batch registration 使用一次 staging generation/projection 与一次原子 publish；失败仍不泄漏 commands/views/consumers，不能每 plugin clone 完整 registry。
- 只读 status/report 只消费 live-host/catalog generation，不触发 dynamic library load、entry callback 或文件系统 discovery；explicit refresh 每 generation 至多发现/加载一次。
- 1/100/1000 plugin benchmark 记录 clone bytes、build count 和 recompute p95；manifest/extension/diagnostic 顺序与 lifecycle 语义等价。
- unchanged 可见 Module/Plugin pane 的 project manifest read/parse、discovery/library load/entry callback、catalog/status/row build count 全为 0；显式 refresh 或 lifecycle generation 每代至多一次。

## 禁止临时方案

- 不得在每个 UI panel 各缓存一份 catalog。
- 不得用无序 map 迭代改变 builtin/registration/diagnostic 顺序。
- 不得让借用跨越 catalog generation 更新而悬垂。

## 修复结果与回传

Open state: Editor12 已建立 generation-owned immutable catalog snapshot、projection 与 extension
report；仍待 source-bound Cargo、独立复审、native refresh publication 和 1/100/1000 scale 证据，
不得回传 `fixed-*`。

当前 native status 依赖：Editor12 已将稳定状态读取从
`NativePluginLoader.load_discovered_all` 降为 discovery authority 的 last-good report 投影，因而不再在 pane
recompute 中加载 DLL 或执行 entry callback；但 cold root 的 `discover()` 仍会调度并等待 refresh，且会物化 owned
report。该剩余根因已由 Plugins01 的
[`native-plugin-discovery-recursive-rescan`](../../../zircon_plugins/01/failure-2026-07-17-native-plugin-discovery-recursive-rescan.md)
覆盖。该 owner 必须提供不调度、返回已发布 `Arc<NativePluginDiscoverySnapshot>`（或明确
`refresh-pending`）的公开读面；随后 Editor12 才能移除这次中间 `discover()` 调用并完成 1/100/1000 stable-read
验收。本 failure 保持 `open`，不得把无 DLL-load 当作完整 native refresh acceptance。

2026-07-22 current-source补充：descriptor→runtime manifest已由双线性find改为一次first-wins borrowed index，extension/menu/operation validation也删除临时集合与双查表；但`EditorPluginCatalog`仍同时保存per-plugin registry和deep-cloned merged registry，mutation即全量merge。按PERF-MVP-538让Runtime/Editor共享frozen extension generation前，本failure保持open。

## 2026-07-27 架构收束设计

- **所有权与目录**：`zircon_editor::core::plugin` 是 catalog generation、原生候选发现和扩展物化的唯一 owner；按本计划 M1 直接把现有 `editor_plugin.rs`、`editor_plugin_sdk/` 与 `editor_plugin_catalog_gen.rs` 迁入该目录，删除旧根路径，不保留 re-export 或 UI 私有副本。`ui::host::EditorManager` 只持有该 core owner 的 facade，retained pane 只消费不可变 snapshot。
- **稳定读面**：新增 `EditorPluginCatalogStore`，在每个已发布 generation 唯一持有一个 `Arc<EditorPluginCatalogSnapshot>`。snapshot 包含固定排序的 package/extension/diagnostic rows、package-id 与 capability 的有序索引、已物化的 extension catalog，以及 generation/fingerprint；查询返回借用切片或 clone `Arc`，不再通过 `package_manifests()`、`capabilities()` 或 UI row 重建返回全量 owned collection。
- **变更面**：builtin 初始化、批量 register/enable/disable、native discovery 结果和 hot-reload 都先构造候选 snapshot；通过全部 descriptor、capability、extension 和 lifecycle 校验后一次 publish 并递增 generation。失败候选只更新受控 diagnostics/progress，不得污染 last-good snapshot 或已注册贡献。
- **I/O 边界**：`refresh_native_catalog(project_root)` 是唯一允许文件系统 discovery、动态库 load 或 entry callback 的命令面；稳定 `plugin_status`/pane recompute 只读取当前 generation。首次尚无 project snapshot 时返回明确的 `refresh-pending`/last-good 状态，由 Runtime11 有界任务层执行刷新，不能在 UI 刷新线程同步补做 I/O。
- **生命周期与并发**：UI、commandlet、export 和 enablement 持有 `Arc` snapshot，旧 generation 在最后一个 reader 释放后才回收；更新不让借用跨 generation 悬垂。刷新、重新加载和启停按一次 mutation batch publish，禁止逐插件重建/merge 或按 panel 建第二份 truth。
- **参考证据**：Unreal `IPluginManager.h` 的 `RefreshPluginsList`、稳定 `FindPlugin`/`GetDiscoveredPlugins` 查询以及 phase event，和 `PluginManager.cpp` 的集中 discovery/merge，支持将昂贵发现放在明确 refresh 命令而非只读查询；`RapidJsonPluginLoadingTests.cpp` 覆盖 descriptor 解析边界。Fyrox `editor/src/plugin.rs` 将 plugin lifecycle 归 editor owner，支持 Zircon 的 core plugin owner + host facade 分层。Zircon 的 VM 插件契约仍优先于两者：不引入 Rust dynamic-object 长期兼容语义。
- **待验证矩阵**：同 generation 连续 1/100/1000 次 pane recompute 的 catalog/status/row build、manifest parse、discovery/load/callback 与 clone bytes 均为 0；显式 refresh/enable/disable/hot reload 每 generation 至多一次 publish；失败候选、重复 refresh、读者持有旧 Arc、无 native 项目和 1k/10k manifest 需覆盖顺序、last-good 和资源回收。

## 产出记录与时间

### 2026-07-27

- 状态：`resolving_failure`；本 failure 继续保持 `open`，尚无受管 Cargo、独立 review 或 fixed return。
- 完成项目：确认 builtin catalog 重建与 native status 同步 I/O 的双重根因；完成 core owner、immutable snapshot、显式 refresh、原子 publish 与 hard-cut 目录迁移设计。
- 证据：`zircon_editor/src/ui/host/editor_manager_plugins_export/{mod.rs,status/native.rs}`、`zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs` 的当前调用链；`dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h`、`Private/PluginManager.cpp`、`Private/Tests/RapidJsonPluginLoadingTests.cpp` 与 `dev/Fyrox/editor/src/plugin.rs`。
- 完成项目：已将 `core/editor_plugin.rs`、`core/editor_plugin_catalog_gen.rs` 与 `core/editor_plugin_sdk/` 硬迁移为 `core/plugin/{descriptor,projection,registration,catalog,capability_report,extension_catalog_report,extension_materialization,sdk}`；`core/mod.rs` 和所有直接调用方改为 `core::plugin`，旧路径和空 SDK 目录均已删除，无 re-export bridge。
- 静态验证：`python tools/tests/test_editor12_plugin_catalog_projection_contract.py` 为 `3/3`；新 `core/plugin` leaf 的 `rustfmt --check` 全部通过；scoped `git diff --check` 通过。更大既有 caller 文件的批量 rustfmt 检查受本地 30 秒窗口截断，不能计为格式验收；尚待 source-bound 受管 Cargo、独立 review、native refresh/snapshot 行为与 scale evidence。

### 2026-07-27 12:02 CST

- 状态：`resolving_failure`，本 failure 保持 `open`；没有把静态检查误记为 Cargo 或 fixed return。
- 完成项目：core owner 已新增 `EditorPluginCatalogSnapshot` 与 `EditorPluginCatalogStore`，以 `RwLock<Arc<...>>` 原子发布 generation；snapshot 在构造期建立 package/capability 索引，`EditorManager` 和 builtin status/enablement/project/manifest-completion consumer 改为读取该共享 snapshot，而非每个 consumer 重建 builtin catalog。
- 静态验证：`python tools/tests/test_editor12_plugin_catalog_store_contract.py` 为 `3/3`；它覆盖 store 的 Arc/RwLock 发布面、snapshot 索引和 host consumer 无 `EditorPluginCatalog::builtin` 重建路径。该结果是共享树静态诊断，不替代 source-bound Cargo。
- 失败交接：native discovery/load 仍在四个 UI 请求链同步执行，已建立并导入 Runtime11 open node `926663`（`native-plugin-discovery-bounded-refresh-publication`）。Runtime11 负责有界刷新、取消、generation-bound immutable publication；Frameworks04 保留现有 native loader/projection 所有权；Editor12 在该合同完成前不得添加本地扫描或线程兜底。
- 后续：建立单一不可变 Editor12 源清单后，依 FIFO 运行受管 Cargo、独立 review 和 1/100/1000 plugin 行为/scale 证据；随后才能将本 failure 返回为 `fixed-*`。

### 2026-07-27 12:10 CST

- 状态：`resolving_failure`，仍为 `open`；本次仅修复 store 内部并发 publish 语义，尚未开始 Cargo。
- 完成项目：`EditorPluginCatalogStore::publish` 原先在写锁外读取 generation，两个发布者可能各自生成同一代次。现将 generation 读取、snapshot 构造和 `Arc` 槽位替换放入同一 `RwLock` 写临界区，保证每次已发布 snapshot 的 generation 严格单调。
- 测试：先在 `test_editor12_plugin_catalog_store_contract.py` 加入写锁原子性断言，旧实现得到预期 `1` 项失败；修复后该守卫为 `4/4`，`rustfmt --check zircon_editor/src/core/plugin/catalog_store.rs` 与 scoped `git diff --check` 通过。
- 后续优化边界：如需要将候选 snapshot 构造移出写锁，必须由拥有 `catalog_snapshot.rs` 的精确会话把 candidate 与 generation-assignment 合同一并改造；不得重新引入锁外 generation 读取。

### 2026-07-27 12:16 CST

- 状态：`resolving_failure`，保持 `open`；当前 FIFO 有外部 Text01 Cargo，Editor12 未申请或启动 Cargo。
- 完成项目：`catalog_store.rs` 增加 `concurrent_publishes_assign_distinct_monotonic_generations`，四个同步发布者必须得到 `2..=5` 的唯一 generation，最终 snapshot 为 `5`。
- 静态验证：并发行为锚已写入源文件；`test_editor12_plugin_catalog_store_contract.py` 仍为 `4/4`，store 文件 `rustfmt --check` 与 scoped diff 均通过。
- 待验证：该 Rust 单元测试尚待单一不可变源清单下的受管 `cargo test -p zircon_editor --lib ... --locked` 编译和执行；在此之前不宣称运行时并发验收。

### 2026-07-27 12:38 CST

- 状态：`resolving_failure`，保持 `open`；本条仅记录 Editor12 core catalog 的静态收束与新 source-bound 输入，不构成 Cargo、独立 review 或 `fixed-*` 回传。
- 完成项目：`EditorPluginCatalogSnapshot` 将原先名实不符的 package-to-capabilities `capability_index` 拆分为 `capabilities_by_package` 与 `packages_by_capability`。后者用 `BTreeMap` 并对 package id 排序去重，新增 `packages_for_capability` 借用查询，补齐计划要求的 capability-to-package 有序索引，未在 UI 侧引入副本或本地缓存。
- 测试：新增 Rust 行为测试，覆盖共享 capability 返回按 package id 排序的两个 package 与未知 capability 的空切片；`python tools/tests/test_editor12_plugin_catalog_store_contract.py` 为 `6/6`，`rustfmt --check` 与 scoped `git diff --check` 通过。Rust 测试尚未运行，不能将该静态结果计为运行时验收。
- 受管验证准备：已归因并物化 validation-copy `30e3db7d1f3040ce8106fbbf25de64ed`，不可变输入哈希为 `8ed95e7c404b58006ae05f5654f25c00a1ade975fb443ea35030ea19bc5e939d`，精确清单仅含 `catalog_snapshot.rs` 与 `catalog_store.rs`。当前 coordinator 仍有 Text01 Cargo 和受控动作执行；Editor12 未启动 Cargo，待 FIFO 空闲后仅通过该 copy-run 执行。

### 2026-07-27 12:55 CST

- 状态：`resolving_failure`，保持 `open`；最终四路径输入已冻结，仍未运行 Cargo、独立 review 或 `fixed-*` 回传。
- 完成项目：build 生成物从 `editor_plugin_catalog_gen.rs` 硬切换为 `plugin_catalog_generated.rs`，adapter include 与 build writer 同步更新。旧根模块已删除后不再保留其生成文件名，生成内容、条目排序和运行期接口均未改变。
- 测试：`catalog_store.rs` 新增发布后旧 snapshot `Arc` 仍可读取的回归测试；静态合同扩展为硬切生成物名检查，`python tools/tests/test_editor12_plugin_catalog_store_contract.py` 为 `7/7`，四个输入 `rustfmt --check` 与 scoped `git diff --check` 通过。全部 Rust 测试仍待受管执行。
- 受管验证准备：自有两个精确 scope 已经审计合并为 union validation Session `editor12-plugin-catalog-unified-validation-r1-20260727`；validation-copy `dd31015df58243b29a3c75a770fa26cf` 已物化，输入哈希 `e875bb0f795a60d1771a66d82bf7860c3e060b65a642ce18696accb68f1c9533`，清单为 `zircon_editor/build.rs`、`catalog_gen.rs`、`catalog_snapshot.rs`、`catalog_store.rs`。在 FIFO 空闲前不得改动该四路径或启动未绑定的 Cargo。

### 2026-07-27 13:06 CST

- 状态：`resolving_failure`，保持 `open`；仍无 Rust Cargo 终态、独立 review 或 `fixed-*` 回传。
- 失败记录：通用 validation-copy `dd31015df58243b29a3c75a770fa26cf` 的 `run` 以 `could not find Cargo.toml` 终态。经 `workspace_copy.py` 调用链审计，根因是该 API 只复制显式 path，不能用于 Cargo 闭包；该结果不是 Editor12 编译失败，也不创建 Coordinator01 代码 failure。
- 失败记录：改用 `materialize-cargo` 后，job `a86d649633764cfc8e9548fac7027e1b` 在 `closure_planning` 以 `validation_copy_external_source_missing` 终态，未启动 Cargo。该依赖归入既有 Coordinator01 `validation-copy-zr-vm-external-source-pin` 合同，Editor12 不得以共享工作树或未固定 sibling 源绕过。
- 后续受管操作：已固定 `E:\Git\zr_vm` commit `503fb72163cd20ddf32a38f8a330083712f5d648`，`mountPath: zr_vm`，并包含 `zr_vm_rust_binding` 与 `_sys` 两个 crate roots。新 cargo-copy `f8726b18912e49d7a74dcb10051f3006` 已接受、external source hash 为 `f4d20ea8a8ebcec4a7bf89ac293208c2479d2b35a7ab380407be0af3e0fc17f6`，当前物化 18,032 个 repository inputs；仅在其转为 `materialized` 后才能发起同一命令的受管 run。
- 失败交接：closure 已持久化但无 hash/无 typed terminal 的后续非终态已交接至 `docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-27-validation-copy-cargo-materialization-nonterminal.md`。Editor12 保留当前 job，不清理、不重建、不绕过 coordinator；Coordinator01 修复并返回后，才从同一上层 command 重放。

### 2026-07-27 14:59 CST

- 状态：`resolving_failure`，failure 保持 `open`；没有将静态检查或未启动的 Cargo 记为运行时通过。
- 完成项目：`EditorPluginCatalogSnapshot` 现在在 generation publish 的构造期物化
  `Arc<EditorExtensionCatalogReport>`，并以借用 `&Arc` 返回。稳定 UI/commandlet 读取不再首次触发
  `EditorPluginCatalog` 的 `OnceLock` extension materialization；旧 reader 仍持有完整 immutable
  snapshot，新 publish 才会构造新 extension report。
- TDD 与静态验证：先在 `test_editor12_plugin_catalog_store_contract.py` 增加 snapshot-owned extension
  guard，旧实现如预期失败（9 项中 1 项）；实现后该脚本为 `9/9`，
  `rustfmt --check zircon_editor/src/core/plugin/catalog_snapshot.rs` 通过。该证据不替代 Rust 编译或
  行为 benchmark。
- 验收阻塞记录：受管 job `f8726b18912e49d7a74dcb10051f3006` 已在 coordinator materialization
  prepare 终态失败，未创建 Cargo run；其 source baseline attribution 问题继续由
  `failure-2026-07-27-validation-copy-cargo-materialization-nonterminal.md` 处理。native discovery
  的 explicit refresh/publication 仍由 Runtime11 合同所有，Editor12 不在 UI read path 增加本地 I/O。

### 2026-07-27 15:21 CST

- 状态：`resolving_failure`，保持 `open`；隔离基础层已实现，但未将其误记为 cdylib end-to-end
  验收或 fixed return。
- 完成项目：新增 `core/plugin/isolation.rs`，统一捕获 extension registration、runtime event consumer
  discovery 与 lifecycle callback 的 error/panic，并产出带 package id、operation 的 host diagnostic。
  registration 先填充 candidate `EditorExtensionRegistry`，仅完整成功才发布，失败/ panic 不泄漏部分
  contribution；snapshot 构建 faulted package index，`EditorPluginManager` 的 published row 据此进入
  `Faulted`。
- 测试：新增 Rust unit coverage（callback error、callback panic、拒绝注册后 Faulted manager entry）；
  `test_editor12_plugin_manager_contract.py` 为 `4/4`，catalog contract 为 `9/9`，相关 `rustfmt --check`
  和 scoped `git diff --check` 通过。Rust unit 尚未由 Cargo 执行。
- 后续：保持 coordinator materialization failure handoff 为当前 source-bound gate；待其 fixed return
  后，必须在 immutable snapshot 上执行受管 Cargo、独立 review，再继续 cdylib DTO、revoke 和 1/100/1000
  scale evidence。

### 2026-07-27 15:30 CST

- 状态：`resolving_failure`，保持 `open`；M1 state machine 现已显式编码，但 lifecycle executor
  接线与 Cargo 行为验收仍未完成。
- 完成项目：`EditorPluginState` 增加 `Revoking`，`can_transition_to` 定义从发现、校验、加载、激活、
  回收、停用和故障恢复的唯一合法边。`EditorPluginManager::transition_state` 以原子 snapshot publish
  拒绝未知插件和非法跳跃；`set_enabled` 在内部依次求取合法状态，不能从 `Active` 直接绕过
  `Revoking` 到 `Disabled`，也不能将 `Faulted` 直接标成 `Active`。
- 测试：新增 Rust state matrix 与 illegal-skip regression；`test_editor12_plugin_manager_contract.py` 为
  `5/5`，catalog contract 为 `9/9`，全量 Editor12 changed plugin Rust 文件 `rustfmt --check` 通过。
  Rust regression 尚待 source-bound Cargo 执行。

### 2026-07-27 15:41 CST

- 状态：`resolving_failure`，保持 `open`；三源 metadata 已由 manager 承接，实际项目/native
  discovery I/O、受管 Cargo、独立 review 和 scale evidence 仍待完成。
- 完成项目：新增 `EditorPluginDiscovery`，把 package id、source 与 loading phase 作为一次性上游
  输入；`new_with_discoveries` 和 `publish_catalog_with_discoveries` 在 mutation 前校验重复/未知 package，
  与 catalog snapshot 在同一 generation 发布。已有 package 未提供新 input 时保留旧 source/phase，避免
  stable read 重新发现或改变排序。
- 测试：新增 Rust initial-generation 与 publish-generation source/phase regression；manager static
  contract 为 `6/6`、catalog contract 为 `9/9`，相关 `rustfmt --check` 通过。Rust tests 尚未执行，不能
  作为三源 discovery 或热加载行为的运行时证据。

### 2026-07-28 23:xx CST

- 状态：`resolving_failure`，保持 `open`；本条仅同步当前精确 scope 的静态重验，不把它写成 Cargo、独立复审、fixed return 或 commit。
- 完成项目：catalog snapshot/store、lifecycle replacement、canonical projection 和 commandlet consumer 继续共享同一 immutable generation owner；`run_plugin_list_commandlet` 读取 manager snapshot projection，不恢复 descriptor-local 或 commandlet-local catalog truth。
- 静态验证：`test_editor12_plugin_catalog_store_contract.py` 为 `10/10`、`test_editor12_plugin_catalog_projection_contract.py` 为 `3/3`、`test_editor12_plugin_manager_contract.py` 为 `20/20`；精确文件 `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。
- 下游失败交接：Coordinator01 的 `validation-copy-cargo-materialization-nonterminal` 仍是唯一 source-bound gate。当前 copy `416b041cd7524ae6a983f8801bf9bcfc` 固定 18033-path closure 但仍为 `materializing`、无 input hash/无 Cargo run；Editor12 不清理、重建或绕过它，待下游 fixed return 后再以一个 immutable source snapshot 运行受管 Cargo 与独立复审。

### 2026-07-30

- 状态：`resolving_failure`，本 failure 继续保持 `open`；本条记录一次受管 source-manifest 归因失败，不构成 Rust Cargo、独立 review、`fixed-*` 回传或 commit。
- 失败记录：新核心验证 copy `e79f0604caae42edab842afb02f743b5` 已被 coordinator 接受，但在 `overlay_ownership` 终态失败，错误为 `validation_copy_overlay_not_owned`，路径为 `zircon_editor/build.rs`；没有 input manifest hash、Cargo job、Cargo run 或测试结果。
- 根因与处理：精确 lease 只保护写入，并不为 validation-copy 持久化当前内容哈希。core hard-cut 需要的 36 个文件已无冲突续租，并通过 `baseline attribute` 请求 `569d0f0bdf7042b2b9f61c0892a3efb4` 登记为 Editor12 当前归因；后续必须新建 immutable copy，旧 job 不得重放或作为验收证据。该边界由 Coordinator01 validation-copy/attribution 合同与 Editor12 source-manifest 接线共同处理。

### 2026-07-30 review recovery correction

- 状态：`resolving_failure`；本条记录独立 review 暴露并已进入代码/静态回归的 Editor12 恢复缺口，不构成 Cargo GREEN、`fixed-*` 回传或 commit。
- 失败记录：多 package 替换在后一个旧实例的 `Disabled` 回调失败时，旧的 preflight 会拒绝下一次 replacement；若先用 enablement facade 重试，条目会落为 `Disabled`，无法再完成该实例的 `Unloaded` 或激活其候选。该路径在首次多 package 回归中未被覆盖。
- 修复：mutable `EditorPluginCatalog` 已 hard-cut 为 crate-private，外部构造改由 `EditorPluginManager::from_plugins`/`from_descriptors` 承接，初始 catalog generation 固定由 manager 发布为 `1`。replacement transaction 不再绕过失败的 `Disabled`：它重试该 stage，继续 `Unloaded`，保留已清理条目的 `Revoking` 状态，完整旧批次回收后才将所有候选重置并激活。
- 测试：新增 multi-plugin initial-generation 与 `Disabled`-failure multi-package replacement recovery Rust 回归；静态合同 `test_editor12_plugin_catalog_store_contract.py` 为 `11/11`、`test_editor12_plugin_manager_contract.py` 为 `22/22`、`test_editor12_plugin_admission_contract.py` 为 `5/5`；精确 `rustfmt --edition 2024 --config skip_children=true --check` 与 scoped `git diff --check` 通过。Rust tests 尚未由 Cargo 执行。
- 独立复审：修正测试 fixture 的阶段定位后，复审结果为 Critical/Important/Minor = `0/0/0`；确认 `Revoking` 与 enabled-`Faulted` 均重新进入 retirement，`Disabled` 重试后完成 `Unloaded`，候选统一 reset、activate、hot-reload。
- 受管归因：本轮 9 个 Rust source/test 路径已续租并通过 `baseline attribute` request `7f1e12e54ca644849c8f4347f26e03c6` 登记当前 hash；该操作不代表 validation copy 或 Cargo 已启动。
- 后续：已 materialized 的 copy `ebd72da0d6bf46109270c0250fdca37a` 在本次源修改前创建，不能重用。Coordinator01 必须先完成 `failure-2026-07-30-validation-copy-source-hash-canonicalization.md` 的 canonicalization return；随后 Editor12 重新 baseline-attribution、materialize immutable copy、创建新的 canonical reservation，执行 focused Cargo 和独立复审。

### 2026-07-30 native editor contribution fault reporting correction

- 状态：`resolving_failure`，本 failure 继续保持 `open`；本条只记录 M2 native contribution isolation 的恢复，不构成 Rust Cargo、独立 review、`fixed-*` 回传或 commit。
- 失败记录：selected native editor package 的 load 或 serialized contribution materialization 失败会在 `NativeEditorContributionMaterialization` 中清空 contribution 并记录 diagnostic，但 host projection 以 `require_usable_native_entry` 直接过滤该 package，导致 manager/panel 看不到 diagnostics，也不能按既有 diagnostics-to-`Faulted` 规则发布状态。
- 修复：`native_editor_registration_reports_from_load_report` 现在保留 selected package 的 registration；未取得 usable native entry 时注入 host diagnostic，同时保持空 contribution registry。原有 registration `is_success() == false` 边界负责将该 package 置为 `Faulted`，未新增 UI cache、fallback、兼容 alias 或 plugin-specific 分支。
- 测试与后续：新增 `selected_native_load_failure_remains_visible_to_the_plugin_manager` Rust 回归和 native activation source contract；全量 `tools/tests/test_editor12_*.py` 共 19 个静态契约通过，精确 `rustfmt --edition 2024 --config skip_children=true --check`、scoped `git diff --check` 通过。Rust regression 尚未运行；仍等待 Coordinator01 `failure-2026-07-30-validation-copy-source-hash-canonicalization.md` return 后，以新的 immutable copy 执行受管 Cargo 与独立复审。

### 2026-07-30 project-scoped native registration publication

- 状态：`resolving_failure`，本 failure 继续保持 `open`；本条记录 M1 项目范围 manager 接线与静态验证，不构成 Rust Cargo GREEN、独立 review、`fixed-*` 回传或 commit。
- 完成项目：`EditorManager` 不再持有全局 `builtin_shared()` manager，而是构造自己的 builtin 基线；项目打开在 document 事件前加载 selected native editor reports，并通过 manager 的持锁 project publication 替换上一项目的 `Project` rows。关闭或打开回滚会清理 project rows，且先发布 document close 再传播清理错误。headless commandlet 仍是唯一保留 process-wide builtin read model 的消费者。
- 生命周期与所有权：catalog 对没有 Rust `EditorPluginHandle` 的 native host registration 记录 host-owned lifecycle event，不再错误报告“no manager-owned lifecycle handle”。同 package id 的 project report 重发也被视为新 host lifecycle instance，先回收旧行再在新 catalog generation 记录 `Loaded/Enabled`，不会保留脱离 registration report 的 `Active` 状态。完整 publication transaction 在独立 `manager/publication.rs`，project replacement 在 `manager/project_registration.rs`；builtin/manifest rows 的 active-phase retraction 保护保持不变，Project rows 可随项目关闭被撤换。
- 测试：新增 project-native replace/clear 与 host lifecycle Rust regression；更新 admission、catalog-store、manager static contracts 以验证新的叶模块和非全局 host owner。全量 19 个 `tools/tests/test_editor12_*.py`、相关 `rustfmt --edition 2024 --config skip_children=true --check` 与 scoped `git diff --check` 通过。
- 后续受管验证：尚未创建本次代码的 Cargo job 或 run。必须等待 Coordinator01 `failure-2026-07-30-validation-copy-source-hash-canonicalization.md` return，并从当前 hash 物化新的 immutable source copy 后运行 focused Rust test、独立复审与 fixed return；旧 copy/run 一律不得重用。

### 2026-07-30 project lifecycle stable status snapshot

- 状态：`resolving_failure`，本 failure 继续保持 `open`；此项只完成稳定读取的 I/O hard-cut，尚无 source-bound Cargo、独立 review、scale evidence、`fixed-*` 回传或 commit。
- 完成项目：`EditorManager` 现在持有 `Mutex<Option<Arc<ProjectPluginStatusSnapshot>>>`。项目打开显式取得一次 `NativePluginLoadReport`，同一报告依次完成 native manifest、selected registration 和 status projection，并在 manager manifest publication 成功后发布 host-owned status snapshot；项目关闭和打开回滚先清除该 snapshot。retained Module/Plugin pane 只 clone 该 `Arc`，不再解析 project root/`zircon-project.toml`，也不再调用 native discovery/load。
- 静态验证：先加入 snapshot contract 得到预期 RED（缺少 `project_snapshot.rs`）；实现后 `python tools/tests/test_editor12_plugin_manager_contract.py` 为 `24/24`，精确 Rust 文件 `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。该结果不替代 Rust 编译或运行时行为验收。
- 剩余范围：pane 行投影仍会为 UI view model 物化字符串，尚未证明 1/100/1000 clone bytes 为零；native refresh 的非调度 published discovery read 仍属于 Plugins01，且新的 immutable source-copy Cargo gate 仍受 Coordinator01 canonicalization failure 约束。两项完成后才可进行完整 scale/Cargo/review/fixed return。

### 2026-07-30 retained pane Arc-identity projection cache

- 状态：`resolving_failure`，本 failure 继续保持 `open`；本条完成同一 status snapshot 的 UI row/diagnostic 稳定读取复用，不构成 Cargo GREEN、独立 review、scale evidence、`fixed-*` 回传或 commit。
- 完成项目：retained host 新增 `ModulePluginPaneProjectionCache`，只保存 manager-owned `Arc<EditorPluginStatusReport>` 与由其派生的 `ModulePluginsPaneViewData`。相同 `Arc` 以 `Arc::ptr_eq` 命中并 clone 轻量 pane handle；不同 `Arc` 才借用 status report 生成 rows/diagnostics 并替换缓存。缓存不解析文件、不执行 discovery/load/callback、不保存可变 plugin truth，项目 snapshot 变更自然失效。
- 测试：先将 cache identity、host construction 与 pane hit/store 约束加入 static contract，缺少 `cache.rs` 时预期 RED；实现后 `python tools/tests/test_editor12_plugin_manager_contract.py` 为 `24/24`。cache 内含 same-Arc hit / successor-Arc miss Rust regression，尚待受管 Cargo 执行；精确 `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。
- 剩余范围：尚未测量 1/100/1000 plugin 的 clone bytes/build count/p95，也未验证 close/reopen、native load failure、reader 持有旧 Arc 的运行时行为；仍须等待 Coordinator01 canonical immutable-copy gate 后执行 focused Cargo、独立 review 和 scale evidence，才可回传本 failure。

### 2026-07-30 native-aware mutation snapshot publication

- 状态：`resolving_failure`，本 failure 继续保持 `open`；本条修复 mutation 后 retained pane 复用陈旧 status `Arc` 的回归，不构成 Rust Cargo、独立 review、scale evidence、`fixed-*` 回传或 commit。
- 失败记录：project-open-only publication 会让 native-aware enable/feature/packaging/target-mode action 修改 manager 或 project manifest 后仍保留旧 status snapshot；Arc-identity cache 会正确命中该过期投影，因而 UI 无法反映已提交的 action。
- 修复：`publish_project_plugin_status_from_load_report` 成为唯一 mutation publication helper。native-aware enable、feature toggle/dependency 和 packaging/target-mode 操作都复用本操作已取得的 `NativePluginLoadReport`，完成 manifest 后仅在成功路径发布 successor status `Arc`；失败路径不发布，保留 last-good snapshot。未在 retained UI 添加 refresh 或重复 discovery。
- 测试：static contract 先因三个 mutation owner 缺少 publication helper 预期 RED；实现后 `python tools/tests/test_editor12_plugin_manager_contract.py` 为 `24/24`。`native_plugins.rs` Rust regression 现在断言 dependency/feature mutation 换代 status Arc，并从最终 native selection mutation 读取已发布 snapshot；受管 Cargo 尚未运行。精确 `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。
- 剩余范围：仍缺 1/100/1000 scale 计数、close/reopen/native-load-error/old-reader 的运行时验证与 Coordinator01 source-bound copy；该 failure 不得据此 fixed return。

### 2026-07-30 Performance01 core plugin current-source supplement

- 状态：`resolving_failure / static_complete / dynamic_pending`；本条只同步当前35/35个`core/plugin/**` Rust文件（6,318行、51 tests）的性能静态证据，不构成Cargo、scale、独立review、`fixed-*`回传或commit。
- 已成立边界：manager/catalog stable read共享`Arc` snapshot，package/registration/capability/panel projection索引在generation构建，unchanged loading phase/project manifest返回同一snapshot；旧“稳定pane每读重建catalog”描述不得继续使用。
- PERF-MVP-538剩余根因：external lifecycle event也会clone完整`EditorPluginCatalog`（包含extension registries与全部lifecycle history），随后重建manifest/capability/index/panel projection；每个manager generation又无条件`build_active_extensions`，重clone所有extension descriptor、builtin asset types与diagnostics。native batch在outer candidate和`materialize_serialized_contribution_batch`内双clone registry；initial admission还多次owned manifest并递归DFS。
- 目标结构：把immutable structural catalog/compiled extension owner与bounded lifecycle state分离，active extensions按`{catalog_generation, active_set_generation}`共享；成功非结构event不得换structural generation或重建projection/registry。project/native batches在一个candidate transaction内只物化一次；admission借用manifest并改iterative indexed graph。保持last-good、one publish、old-reader quiescence与phase/revoke/rollback。
- PERF-MVP-594联动：bridge和manager当前形成两层锁内callback；manager还让routine成功event永久扩张history。按bounded page、短锁active-handle snapshot、锁外affinity dispatch、generation-checked commit与entry+bytes+age bounded audit收束；不得以私有线程池或丢lossless terminal规避。
- 验收矩阵：plugins/contributions/history `0/1/100/1K/10K`、messages `0/1/64/4,096`、callback `0/1/16ms/10s`、batches `1/100`、dependency depth `1/1K/100K`、threads `1/16`、reload/unload/error/stale completion。记录catalog/manager/projection/extension builds、clone bytes、callback/mutation lock wait+hold、queue/audit bytes+age、stack/RSS和F0/F4 p95；当前尚无上述动态证据。

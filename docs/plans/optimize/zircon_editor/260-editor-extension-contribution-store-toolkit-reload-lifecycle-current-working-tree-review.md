---
title: Editor Extension、Contribution Store、Toolkit 与 Reload Lifecycle 当前工作树复审
category: zircon_editor
report_id: Editor260
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/259-editor-plugin-provider-catalog-current-working-tree-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/199-runtime-plugin-profile-catalog-provider-resolution-current-working-tree-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
related_code:
  - zircon_editor/src/core/plugin
  - zircon_editor/src/core/extension
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/core/runtime_event_consumer/host/contribution_lifecycle.rs
  - zircon_editor/src/core/tools
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/editor/plugins
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/Fyrox/editor/src/plugins
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor260 · Extension / Contribution Store / Reload Lifecycle 差距

## 1. 结论

当前 Editor 已经有一组值得保留的底层结构：不可变的 catalog snapshot、ContributionStore 的按 ticket 撤销、Toolkit 的 save lease、runtime event consumer 的 active generation、插件生命周期阶段以及 registration gate。这些结构说明项目已经开始处理并发、快照和资源所有权，而不是单纯的临时全局 Vec。

但是这些结构没有形成一个可提交、可回滚、可审计的 Editor provider transaction。catalog admission 只检查已发布节点之间的环，缺失依赖会被测试明确视为合法；没有 runtime manifest 的 editor descriptor 会自动生成 standalone manifest；生命周期失败写入 registration 的 lifecycle report，却不进入 catalog diagnostics 或 snapshot fault set；extension materialization 遇到重复贡献会继续返回部分 registry，同时 manager entry 仍可为 Active；撤销时又先改变 View、runtime consumer 和 tool scheduler，最后才提交 ContributionStore，任一步失败都会留下跨管理器的半完成状态。

能力语义也没有贯穿这条链路。ContributionStore 的 IndexedContribution 支持按 CapabilitySet 过滤，但 manager snapshot 的 build_active_extensions 直接把 Active registration 的 raw extensions 全量物化，因此同一个 contribution 在 Store 查询和 active extension projection 中可能得出不同的可见性。Toolkit 和 AssetTypeContribution 还缺少稳定的 plugin owner、provider generation 与 source digest，任意 caller 可以注册看起来合法的 toolkit。

本轮冻结 79 个 Rust 文件、17,783 行、16,146 个非空行、626,887 bytes、173 个测试声明、0 个 ignored marker。范围包括 plugin catalog/admission/snapshot/manager/lifecycle、extension registry 与 ContributionStore、document/asset toolkit、Editor Host contribution registration、runtime event consumer retirement 及其测试。范围不包括 Tooling 优化，也不把已有 Workbench UI substrate 当成 provider lifecycle 的实现。当前登记 5 项 P0（5 Open）、34 项 P1（30 Open / 4 Partial / 0 Closed）、10 项 P2（10 Open），以及 28 道资格门（25 Fail / 3 Partial / 0 Pass）。

## 2. 物理扫描范围与证据

### 2.1 选择集

逐文件读取：

- zircon_editor/src/core/plugin 下 36 个 Rust 文件，覆盖 admission、catalog、snapshot、manager、discovery、registration、materialization、lifecycle bridge、isolation 和 tests。
- zircon_editor/src/core/extension 下 40 个 Rust 文件，覆盖 EditorExtensionRegistry、ContributionBatch/Store、snapshot/index、toolkit registry、save authority、layout 和 lifecycle tests。
- zircon_editor/src/core/editor_extension.rs、zircon_editor/src/ui/host/editor_extension_registration.rs。
- zircon_editor/src/core/runtime_event_consumer/host/contribution_lifecycle.rs。

选择集指标为 79 文件、17,783 行、16,146 非空行、626,887 bytes、173 个测试声明。计数包含生产代码和同目录测试，目的是同时审计 implementation 与其自证边界；没有把未编译 cfg 分支从风险中扣除。

### 2.2 直接证据

- core/plugin/admission.rs:39-67 建立 dependencies_by_package 后只调用 find_dependency_cycle；未知依赖不会报错。admission.rs:168-173 的测试名称就是 ignores_dependencies_outside_the_published_catalog，并断言 Ok。
- core/plugin/catalog.rs:56-93 在 runtime manifest 找不到时调用 plugin.descriptor().standalone_package_manifest()。Editor descriptor 因而可以在没有对应 runtime package 的情况下进入 catalog。
- core/plugin/catalog.rs:139-161 记录 lifecycle event 时只把未知 package diagnostic 加到 catalog；正常 plugin 的 lifecycle report 写回 registration，但 catalog diagnostics 不同步扩展。catalog.rs:189-192 的 is_package_faulted 只看 registration.is_success()。
- core/plugin/registration.rs:84-126 把 lifecycle failure 记录到 lifecycle、failed_lifecycle_stages 和返回 report；registration.rs:137-140 的 is_success 仍只判断 registration diagnostics，生命周期失败不会改变健康判定。
- core/plugin/catalog_snapshot.rs:43-51 由 registration.is_success() 生成 faulted_packages，因此 snapshot 不会把 Loaded/Enabled/Disabled/Unloaded callback failure 作为 fault。
- core/plugin/extension_materialization.rs:9-156 对每项 registry registration error 只追加字符串 diagnostic，继续物化其它 contribution，最后返回部分 registry。sequence 还在 172 行用 saturating_add。
- core/plugin/manager/snapshot.rs:93-110 从 Active entries 的 registration.extensions 直接调用 build_editor_extensions，没有通过 ContributionStore 的 required_capabilities 过滤。
- core/plugin/manager/state.rs:13-34 只有 Discovered、Validated、Loading、Active、Revoking、Disabled、Faulted；331-365 以 Loaded、Enabled 两次 callback 直接进入 Active，没有 Ready、Quiescing、Unloading 或 shutdown acknowledgement。
- core/plugin/manager/publication.rs:26-84 在最终 publish 前先 retire replaced active entries、激活新 entry 和发送 HotReloaded；失败时只能重新发布旧 catalog 与已经改变的 entry。
- core/plugin/manager/project_selection.rs:75-101 将 manifest 中不属于当前 editor package id 的选择刻意忽略；unknown editor selection 没有 typed receipt。
- core/plugin/catalog_store.rs:31-51 以 saturating_add 计算 next generation，却用 assert 检查精确的 next generation；饱和后会从可诊断错误退化为 panic。
- core/extension/store/model/contribution_store.rs:169-412、415-495、498-571 对 generation 和 next_ticket 都使用 saturating_add；ContributionTicket 没有 owner/provider generation。
- core/extension/store/model/snapshot.rs 的 IndexedContribution 具备 required_capabilities 过滤能力，但只有读取该 snapshot 的 caller 才能得到这个结果，manager active_extensions 没有共享该判定。
- core/extension/toolkit/registry.rs:119-149 允许任何 caller 以 DocumentToolkitDescriptor 注册 toolkit；descriptor 没有 plugin owner、provider generation、source digest 或 capability admission。
- core/extension/toolkit/registry.rs:161-188 的 close 只等待 active_saves，未等待 provider job、command、viewport 或 runtime consumer 的 quiescence。
- ui/host/editor_extension_registration.rs:156-265 先 clone/revoke ContributionStore，随后依次 retire views、runtime consumers、tools，最后才 commit shell；中途错误会留下已退休外部资源和仍存在的 owner/store 状态。
- core/runtime_event_consumer/host/contribution_lifecycle.rs:33-107 在 active consumer callback 失败后仍提交 registry/quarantine/user-disabled/cursor 状态，错误只放在 cleanup_error，外层 revoke 仍可能继续执行其它副作用。
- core/plugin/manager.rs:40-41、127-133 用 process-wide OnceLock 保存 builtin manager；多个 project/session 无法拥有独立的 builtin lifecycle authority。

### 2.3 参考引擎对照

- Unreal PluginManager 的模块发现、依赖排序、load phase、enabled 状态和 shutdown/unload 是同一管理面；Zircon 将 manifest catalog、manager entry、extension registry、runtime consumer 和 tools 分散在多个可独立改变的对象中。
- Godot GDExtensionManager 与 editor plugins 把加载、实例、版本/接口检查和退出边界作为 extension 生命周期的一部分；Zircon 的 native row 与 Rust plugin 使用不同 identity，且没有统一 unload receipt。
- Bevy PluginGroup 在 build 前检查重复插件并保持确定性插入顺序；Zircon 的 ContributionStore 有 duplicate check，但 catalog materialization 的 conflict 只变成排序后的字符串诊断，不能阻止 Active 状态。
- Fyrox editor plugin trait 暴露 start/update/exit 等宿主回调，插件实例由 editor 生命周期持有；Zircon 只有 Loaded/Enabled/Disabled/Unloaded 等事件记录，没有 initialize/ready/quiesce 的状态机和 acknowledgement。
- Unity Graphics 的 editor package 将 importer、graph editor、preview 和 domain reload 视为同一 package/product contract；Zircon 的 ZUI template、operation factory、asset/toolkit 和 runtime provider 没有共同 generation/source map。

## 3. 当前装配流与断点

EditorPluginDescriptor + runtime manifest
  -> EditorPluginCatalog.from_descriptors
  -> CatalogSnapshot / ManagerSnapshot
  -> Active registration.extensions
  -> build_editor_extensions (partial registry allowed)
  -> Editor Host contribution registration
  -> ContributionStore / commands / scene modes / overlays / runtime consumers / tools
  -> revoke or hot reload

这条路径缺少五个工程级 owner：

1. EditorDependencyClosure：同时验证 editor package、runtime package、capability、target、native ABI 和 Cargo feature。
2. EditorPluginHealth：把注册、生命周期、materialization、runtime consumer、toolkit 和 command 结果聚合成一个 typed health record。
3. EditorGeneration：为 catalog、manager、contribution、ZUI、toolkit、runtime consumer 和 native instance 提供同一 generation/instance token。
4. EditorAdmissionTransaction：所有 external manager 先 prepare，再一次性 commit；失败保留 last-good generation。
5. EditorRetirementReceipt：quiesce、cancel acknowledgement、revoke、dispose、unload 和 late-publish fence 都有终态证明。

## 4. P0 阻断项

| ID | 阻断 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED-EL-01 | Editor catalog 可以接受缺失的 editor/runtime 依赖 | admission 只查环，unknown dependency 测试断言 Ok；catalog 对缺失 runtime manifest 自动生成 standalone manifest | 在 publish 前解析完整依赖闭包；缺失、target 不符、runtime/editor manifest 不配对必须返回 typed fatal decision |
| ED-EL-02 | 生命周期失败不会进入 catalog/snapshot 健康状态 | registration.is_success 只看 registration diagnostics；catalog snapshot faulted_packages 也只按该值生成 | 为每个 package 建立聚合 health，lifecycle failure、native failure、materialization failure 都必须使 generation 进入 Faulted 或阻止 Active |
| ED-EL-03 | Active plugin 可以拥有部分物化的 extension registry | extension_materialization 遇到 duplicate/invalid contribution 只积累字符串并返回 partial registry；manager entry 仍可为 Active | 将 contribution materialization 改为全量 preflight + atomic commit；任何 required contribution 错误都拒绝该 package 并保留 last-good registry |
| ED-EL-04 | revoke/hot reload 跨 manager 不是原子事务 | Host revoke 先 retire view/runtime consumer/tools，最后才提交 ContributionStore；中途错误会造成外部副作用和 owner/store 分叉 | 引入跨 manager prepare/commit/rollback transaction，所有 manager 以同一 owner generation 提交，失败恢复旧快照并产生 terminal receipt |
| ED-EL-05 | required capability 在 active projection 中被绕过 | ContributionStore IndexedContribution 支持 CapabilitySet 过滤；manager snapshot 却直接物化 Active registration.extensions raw data | 统一 CapabilityResolver；catalog、manager snapshot、materializer、Host registry 和执行 command 使用同一 resolved capability set，并拒绝未知 capability |

## 5. P1 重构账本

| ID | 差距 | 目标验收 |
|---|---|---|
| ED-EL-06 | catalog admission 没有缺失依赖错误类型 | 增加 MissingDependency、TargetMismatch、RuntimePairMismatch 等 typed error，包含 depender、dependency、source 和 repair hint |
| ED-EL-07 | standalone_package_manifest 掩盖 runtime/editor 配对错误 | descriptor-only、editor-only、runtime-backed 三种 policy 显式声明；runtime-backed 无配对 manifest 时不能自动补造 |
| ED-EL-08 | project selection 静默忽略未知 package | 每个 selection 生成 selected/disabled/unknown/retired/role-mismatch receipt；required unknown 在 resolve 阶段失败 |
| ED-EL-09 | catalog diagnostics 与 lifecycle report 分裂 | diagnostics 只允许从 immutable health reducer 生成，生命周期事件发布后 snapshot 的 health 与 manager state 必须一致 |
| ED-EL-10 | 生命周期状态缺少 initialize/ready/quiesce/unloading/shutdown | 为插件和 native provider 增加显式状态、合法转换、超时和恢复路径，禁止 Active 直接跳到 Disabled |
| ED-EL-11 | lifecycle report 只有字符串和 Vec stage | 记录 event id、sequence、generation、duration、error code、callback owner 和 acknowledgement |
| ED-EL-12 | panic isolation 丢失 action/stack/source chain | run_editor_plugin_boundary 返回结构化 panic/fault，携带 crate、symbol、stage、backtrace policy 和是否可重试 |
| ED-EL-13 | callback 执行是同步无预算调用 | lifecycle callback 使用 bounded execution budget、cancellation token 和 deadline；超时进入 Faulted/Quiescing，而不是无限占用 manager lock |
| ED-EL-14 | hot reload 没有资源 quiescence lease | reload 先冻结 command、jobs、watchers、toolkit saves、viewport overlays 和 runtime consumers，再交换 provider generation |
| ED-EL-15 | catalog/store/manager 使用多个 generation 时钟 | 由 EngineGenerationAuthority 分配 checked generation；所有 snapshot、receipt 和 journal entry 引用同一 generation |
| ED-EL-16 | saturating generation/ticket 会隐藏耗尽 | catalog、ContributionStore、diagnostic sequence、manager generation 改成 checked_add；耗尽返回 typed GenerationExhausted，禁止 assert/panic |
| ED-EL-17 | builtin manager 为进程级 OnceLock | builtin provider 绑定 EditorSessionId/ProjectId；多个打开项目必须有独立 lifecycle、contribution store 和 shutdown |
| ED-EL-18 | Arc ptr identity 不能代表 native/provider 实例 | 为 Rust/native provider 分配稳定 ProviderInstanceId 和 load token；replacement/reload 按 token 判断同一实例 |
| ED-EL-19 | catalog snapshot 没有 provider provenance | registration 记录 source kind、crate/artifact version、Cargo features、manifest fingerprint、ABI、target、build revision |
| ED-EL-20 | materializer 以字符串 sequence 处理冲突 | conflict 记录 contribution id、owner、priority、dependency order 和 resolution policy；required conflict 直接 fail |
| ED-EL-21 | materializer 的 partial registry 没有 last-good 指针 | EditorExtensionCatalogReport 同时保存 candidate、last_good_generation 和 typed diagnostics；失败不覆盖可执行 registry |
| ED-EL-22 | manager snapshot 每次发布都全量重建扩展 | 按 catalog generation、active package set、capability set 和 extension digest 缓存 materialization；变更 package 才增量重建 |
| ED-EL-23 | raw registration.extensions 与 ContributionStore 有两份事实源 | Active extension projection 只读取已提交 ContributionSnapshot，禁止从 registration raw Vec 旁路物化 |
| ED-EL-24 | ContributionStore source 只有字符串 namespace | ContributionSource 绑定 package id、provider instance、generation、manifest digest 和 trust decision，撤销按完整 owner token |
| ED-EL-25 | required_capabilities 没有 catalog capability admission | contribute/replace 前解析 capabilities_by_package；不存在的 capability、禁用 capability 和 target 不符必须拒绝 |
| ED-EL-26 | ticket/revoke 缺少 stale generation 防护 | ContributionTicket 携带 owner generation 和 store epoch；旧 owner 的 revoke、replace、command execute 必须返回 StaleGeneration |
| ED-EL-27 | bounded change journal 只能 reset，不能恢复原因 | journal 记录 snapshot digest、parent generation、owner delta 和 reset reason；跨窗口 consumer 可验证重放或获取完整 snapshot |
| ED-EL-28 | toolkit descriptor 没有 provider owner/asset contract | DocumentToolkitDescriptor、AssetToolkitDescriptor 和 AssetTypeContribution 绑定 owner token、asset type schema、runtime kind、capabilities 和 disposal |
| ED-EL-29 | 任意 caller 可以注册 toolkit | ToolkitRegistry 只接受 AdmissionTransaction 产生的 lease；重复 document/instance、owner mismatch 和 capability mismatch 在 prepare 阶段拒绝 |
| ED-EL-30 | toolkit close 只检查 active_saves | close/unregister 等待 save、job、command、preview、viewport 和 runtime consumer leases；超时返回可恢复 close receipt |
| ED-EL-31 | command/menu path 仍是弱类型字符串 | command descriptor 必须绑定 operation factory、typed input/output、document/world scope、undo/redo、cancel 和 owner generation |
| ED-EL-32 | ZUI template 通过同名和 plugins:// 前缀隐式绑定 | 生成 template/view/source map，记录 template revision、provider symbol、operation route、asset schema 和 migration version |
| ED-EL-33 | runtime consumer cleanup error 仍提交部分退休状态 | 设计明确的 cleanup policy：可重试错误保持 Quiescing，强制错误进入 Faulted quarantine；不能以 cleanup_error 字符串代替状态 |
| ED-EL-34 | revoke 外部副作用顺序不可回滚 | View、scene mode、overlay、runtime consumer、tools、commands、ContributionStore 使用同一 prepared retirement bundle，提交前不改变 live manager |
| ED-EL-35 | scene/overlay/runtime consumer 各自拥有 commit | 建立 CompositeEditorRetirement，提供 prepare hash、commit receipt、rollback receipt 和 late-publish fence |
| ED-EL-36 | lifecycle replacement 失败后旧 entry 已被修改 | replacement 采用 two-generation overlap 或 last-good handoff；旧 provider 只有在新 provider ready 且所有 lease 转移后才卸载 |
| ED-EL-37 | lifecycle stage 与 package health 无统一 reducer | 用单一状态 reducer 消费 registration、lifecycle、materialization、toolkit、native 和 runtime event 结果，禁止 caller 各自推断 Active |
| ED-EL-38 | 序列化跳过多数 runtime-only registry 字段 | 为 UI template source、pane provider、localization、scene mode、viewport provider、operation factory 制定 restore/rebind contract 和失效迁移 |
| ED-EL-39 | 测试只覆盖局部 Store/Toolkit，不覆盖跨 manager 故障 | 生成 provider matrix，覆盖 missing dependency、capability mismatch、panic、partial materialization、revoke failure、reload timeout、session isolation、generation exhaustion 和 100+ provider 压测 |

## 6. P2 产品化与性能

P2 共 10 项：1) Provider Health/Generation inspector；2) admission dependency graph viewer；3) lifecycle waterfall 与 callback budget；4) contribution owner/provenance browser；5) hot-reload impact preview；6) toolkit/save/job lease dashboard；7) last-good 与 failed receipt 导出；8) capability-to-contribution matrix；9) catalog/materialization cache telemetry；10) 100+ provider、10k contribution、长时间 reload/undo/save soak 基准。所有界面和 telemetry 必须消费 typed health/generation/receipt，不能从 Active 数量或静态菜单文案推断成功。

## 7. 资格门

| Gate | 当前 | 必须证明 |
|---|---|---|
| G1 dependency closure | Fail | editor/runtime/native/Cargo/target 依赖闭包完整且缺失会阻止 publish |
| G2 manifest pairing | Fail | runtime-backed editor package 不再由 standalone fallback 掩盖 |
| G3 no silent selection drop | Fail | unknown、retired、uncompiled、role mismatch 都有 receipt |
| G4 lifecycle health propagation | Fail | callback failure 在 catalog、snapshot、manager entry 和 UI health 中一致 |
| G5 typed lifecycle stages | Fail | initialize/ready/quiesce/unload/shutdown 有合法状态和 terminal receipt |
| G6 panic/error isolation | Fail | panic、timeout、native error 有 typed source chain 且不污染其它 provider |
| G7 quiesce before reload | Fail | commands/jobs/saves/watchers/overlays/consumers 都收到 cancel acknowledgement |
| G8 generation exhaustion | Fail | 所有 generation/ticket/sequence 使用 checked overflow handling |
| G9 session isolation | Fail | 两个 project/session 不共享 builtin manager 或 contribution state |
| G10 provider identity | Fail | Rust/native replacement 使用稳定 instance/load token |
| G11 materialization atomicity | Fail | 任一 required contribution conflict 不会发布 partial active registry |
| G12 capability consistency | Fail | catalog、manager snapshot、Store、Host registration 和 execution 共享同一 resolver |
| G13 raw projection ban | Fail | active extension 只能来自 committed ContributionSnapshot |
| G14 deterministic conflict policy | Partial | duplicate error 存在，但 owner/priority/resolution receipt 不完整 |
| G15 toolkit ownership | Fail | toolkit/asset type 注册必须由 provider lease 产生并可撤销 |
| G16 close/save barrier | Fail | provider close 等待 save/job/preview/viewport/runtime leases |
| G17 typed operation contract | Fail | command/menu 有 factory、typed I/O、undo/redo、cancel 和 error |
| G18 ZUI source map | Fail | template、view、callback、operation、asset schema 双向可追溯 |
| G19 runtime consumer retirement | Fail | cleanup error 不会在未决状态下提交 live registry |
| G20 composite revoke transaction | Fail | view/scene/overlay/consumer/tools/commands/store 一致提交或恢复 |
| G21 replacement handoff | Fail | 新 generation ready 前旧 generation 不会被提前卸载 |
| G22 health reducer | Fail | package health 由单一 reducer 产生，不由 caller 猜测 |
| G23 snapshot cache correctness | Fail | cache key 包含 active set、capability、digest、generation，失败保留 last-good |
| G24 journal replay | Partial | bounded journal 可 reset，但跨窗口完整 provenance/replay 不足 |
| G25 persistence/restore | Fail | runtime-only contribution 在重启后可验证 rebind 或显式失效迁移 |
| G26 fault injection | Fail | 每个 external manager 的 prepare/commit/rollback 都有故障测试 |
| G27 scale/soak budget | Fail | 100+ provider、10k contribution、长时 reload/save/undo 有 P95/P99 预算 |
| G28 release/security provenance | Fail | provider/artifact/ABI/trust/source digest 和 terminal receipt 随 editor bundle 发布 |

## 8. 后续重构顺序

先实现 DependencyClosure、ProviderHealth 和统一 GenerationAuthority，关闭 ED-EL-01/02/05 的事实源分裂；随后把 extension materialization 与 Host registration 合并为 AdmissionTransaction，处理 ED-EL-03/04/20-23/34-37；再补齐 Toolkit/Asset/Operation/ZUI 的 owner 与 persistence contract，最后做 native reload、fault injection 和规模基准。实现阶段每次只推进一个 generation boundary，并以 last-good snapshot、typed receipt 和跨 manager rollback 测试作为合入条件。

## 9. 验证记录

本轮为 review-only，没有修改 zircon_editor 或生产 Rust 代码，没有运行 Cargo、Editor、UI automation、native unload 或压力测试。已完成路径存在性、行数/非空行/字节数、test marker、Markdown frontmatter 与 git diff --check 静态检查；未查询、轮询、等待或实时跟踪协调器。实现前必须重新读取相关源码并重算本报告指标。

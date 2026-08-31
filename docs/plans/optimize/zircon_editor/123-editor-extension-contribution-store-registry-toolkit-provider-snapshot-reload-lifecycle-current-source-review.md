---
title: Editor Extension、Contribution Store、Registry、Toolkit、Provider、Snapshot、Reload 与 Lifecycle 当前源码复核
category: zircon_editor
report_id: Editor123
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor50
refreshes:
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
related_code:
  - zircon_editor/src/core/extension
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/core/plugin
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_app/src/entry/entry_runner/editor.rs
tests:
  - zircon_editor/src/core/extension/store/tests.rs
  - zircon_editor/src/core/extension/toolkit/tests
  - zircon_editor/src/core/plugin/manager/tests
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/tests/editor_plugin_sdk.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration
  - zircon_editor/src/tests/host/manager/document_toolkit_lifecycle.rs
plan_sources:
  - docs/zircon_editor/core/plugin.md
  - docs/plans/performance/01/2026-07-30-editor-core-editor-extension-current-review.md
  - docs/plans/performance/01/2026-08-15-editor-extension-contribution-overlay-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-plugin-catalog-lifecycle-current-architecture-review.md
  - docs/plans/zircon_editor/editor/02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md
  - docs/plans/zircon_editor/editor/06/failure-2026-07-28-plugin-contribution-ticket-revoke-contract.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Features/ModularFeatures.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenus.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenuOwner.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Toolkits/AssetEditorToolkit.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/editor_data.cpp
  - dev/Fyrox/editor/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Tools/MaterialUpgrader/MaterialUpgraderRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ICoreRenderPipelinePreferencesProvider.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 123 · Editor Extension / Contribution Store / Registry / Toolkit / Provider / Snapshot / Reload / Lifecycle 工程化差距

## 1. 结论

当前扩展代码已经有真实的局部底座：`EditorPluginManagerSnapshot` 按 generation 保存 catalog、phase、state 与 `active_extensions`；`ContributionStore` 以 immutable `Arc<ContributionSnapshot>` 发布、多 family ticket 记录、namespace 校验和 bounded change journal；scene mode 与 viewport overlay 有局部 callback boundary、fault/quarantine；`DocumentToolkitRegistry` 有 checked DocumentId、save/close lease、不可变 descriptor snapshot 和锁外 save I/O。这些实现可以保留，但不能据此声称已有 Unreal/Godot 级别的扩展运行时。

当前最严重的断路是两套激活权威不相交。manager snapshot 的 `active_extensions()` 只有测试、SDK 断言和状态投影读取；`run_editor_with_config()` 仍遍历 `editor_plugin_registrations`，由 `register_editor_plugin_registration()` 直接把 command、view、scene mode、overlay、ContributionStore 和 runtime consumer 装入活跃 Host。项目 native registration 进入 manager 后没有对应的 Workbench reconciler。于是 manager 报告 Disabled/Faulted 时旧 callback 仍可执行，manager 报告 Active 时贡献也可能从未可见。

第二个断路是生命周期。`ContributionStore::revoke()` 没有 production caller，`OwnedContribution` 只有 owner string 与 ticket；command/view/scene/overlay/runtime consumer 没有同一个 owner-generation 的 mount lease、quiesce、reconcile 和 leak census。project close、disable、reload 只能替换 manager rows，不能证明旧 UI 入口、trait object、snapshot reader、异步 job 已经退休。

第三个断路是安全边界。ContributionStore 只在 batch 级保存 capability；asset importer、scene mode、overlay 和 inspector 的 item capability 由不同路径手写，存在绕过禁用门的组合。overlay/scene mode有局部 unwind 捕获，但 Inspector `can_handle/build/validate`、field-editor factory、pane-data `snapshot()`仍可能在 shell/world 锁或 reflection refresh 内调用外来代码；Rust unwind 也不能处理 hang、abort、FFI UB 和无限分配。

第四个断路是跨 registry 原子性。注册过程先 prepare scene/overlay、再构造 command registry、再安装 view、最后 contribute store；任何后段失败都没有统一 rollback receipt。`DocumentToolkitRegistry::register()` 在锁外取得 descriptor 尚可，但 `publish_snapshot()` 仍在 mutex 内遍历 entry descriptor，close/clear 也把 map mutation、snapshot 发布和 trait object drop 混在一个临界区，无法作为可重入、可故障的产品协议。

本轮冻结当前 HEAD `590376671b8745a0d230304c94432857c669bfbd`、epoch 524。frontmatter 的 49 个证据根展开并去重为 139 个物理文件：Zircon source/test 113 / 24,661 lines / 22,444 non-empty / 864,684 bytes / 255 test attributes，reference 14 / 16,849 / 14,257 / 638,005 / 22，plan/docs 12 / 3,175 / 2,401 / 309,251 / 2，union 139 / 44,685 / 39,102 / 1,811,940 / 279。source fingerprint `9f270c006aa4df12272c749c75483bd906e11208b87be72a3c34f773d8e4d4a8`，reference `8a89883c699b574b9b21705977d6ae26b3ebbc2c64a8acbbe329d271501aec5f`，docs `72ac1ff500a09ef9a2f0a6eb596415e57a3601d0b192cf5ec7d1684efa2abde4`，union `a681602d3c45b2bf36d70b1492d82727f78e817004996a592cf6aa1c7ed84350`。本报告登记 5 个 P0、60 个 P1、15 个 P2、40 个资格门；只写 review，不修改生产代码。

## 2. 参考基线与当前证据

Unreal 的 `IModularFeatures`、`FModuleManager`、ToolMenus owner 和 AssetEditorToolkit 把模块状态、owner、菜单注册和 toolkit 生命周期分开，并要求卸载前撤销 owner 资源。Godot `EditorPlugin`、`EditorNode`、`EditorData` 将插件实例、dock/inspector/undo 等 editor-owned 状态集中在 editor 生命周期内。Fyrox 的 plugin trait 与 editor command/message 路径提供显式插件入口，Bevy `Plugin`/`App` 以构建阶段和 schedule 明确插入点，Unity Graphics 的 provider/registry模式强调可发现、可排序、可替换的 provider 合同。

当前 Zircon 可保留的对应关系是：manager catalog≈模块发现，ContributionSnapshot≈只读目录，scene/overlay isolation≈callback 防线，DocumentToolkit lease≈文档级互斥。但尚缺一条从 package admission 到 mounted generation、所有 family receipt、fault/quiesce、revoke/unload 的闭合链；不能把 descriptor 存在、单元测试通过或固定 Workbench 数据当作扩展产品完成。

## 3. 当前调用链与断点

```text
Project/App RunConfig registrations
  -> RetainedHost startup loop
  -> register_editor_plugin_registration()
  -> prepare runtime consumer / scene mode / overlay
  -> mutate command + view registries
  -> ContributionStore.contribute()
  -> visible callbacks and UI
  -X-> manager desired-state reconciler

Catalog/discovery/phase/enablement
  -> EditorPluginManagerSnapshot.active_extensions()
  -> tests/status projections
  -X-> Workbench mount, revoke, quiesce

Disable / project close / reload
  -> manager row replacement
  -X-> command/view/scene/overlay/store/runtime consumer retirement
```

`register_editor_extension_owned()` 的局部候选构造和错误检查是有效基础，但它不能提供跨 family 的 commit id：scene mode 与 overlay 已 prepare 后，command/view/store 仍可能失败；runtime consumer 直到 store 成功后才 install，反向撤销顺序未定义。`active_extensions` 由 `build_editor_extensions()` 构造出 registry、asset types 和 diagnostics，却没有把结果转换成 Host 可执行 mount plan。

## 4. P0：正确性与生命周期

### **P0-01** · manager active snapshot 没有 Workbench production consumer

`manager/snapshot.rs` 只按 `EditorPluginState::Active` 构建 `active_extensions`；当前实际产品启动在 `ui/retained_host/app.rs` 直接消费 `EditorHostRunConfig::editor_plugin_registrations`。二者没有共同 generation，也没有 desired/mounted 差异计算。manager 的状态可以和实际可见扩展永久不一致。

目标是唯一 `EditorExtensionReconciler`：订阅 manager generation，生成完整 desired set，比较当前 mounted owner-generation，准备所有 family 后一次 publish；App 只能提交 catalog/registration input，不得直接安装永久 callback。

### **P0-02** · 没有统一 revoke、quiesce 和 unload fence

`ContributionStore::revoke()`只有 store 测试调用；production 没有按 owner 查找并撤销 command、view、scene mode、overlay、template、asset、runtime consumer 的路径。旧 `Arc<dyn ...>`、snapshot reader、异步 job 和 active scene mode 没有共同 fence，Plugins01/Editor06 无法证明 binary 卸载安全。

目标是每次 mount 返回 `ExtensionMountLease`，记录所有 family receipt；unmount 先关闭 admission，再等待 callback/job/snapshot reader，按逆依赖撤销 UI、commands、providers、store 和 runtime consumers，发布 terminal generation、leak census 与 operator receipt。

### **P0-03** · capability admission 在 family 之间不一致

ContributionStore 的 `IndexedContribution` 只携带 batch capability；asset importer query、scene mode registry、overlay registry、menu-to-command 绑定各自再解释 item capability。尤其 scene mode 注册/创建没有统一 capability 检查，overlay 只检查 provider item，容易在 owner 被禁用后仍执行 callback。

目标是 admission 时编译 `ActivationRequirement + FeatureRequirement + PrincipalPermission` 为不可变 `EffectiveCapabilityPredicate`，所有 snapshot query、factory、callback 入口都校验同一 predicate 与 owner generation。

### **P0-04** · 外部 callback 仍可在锁内或无隔离执行

Inspector customization 的 `can_handle`、reflection refresh 中的 build/field-editor factory、pane-data `snapshot()`没有和 overlay/scene mode 相同的 boundary、deadline、cancel、output budget、quarantine。shell/world 锁内重入可死锁，panic 可破坏主循环，hang 无法恢复。

目标是 `EditorExtensionCallbackSupervisor`：锁内只冻结 immutable input 和 callback plan，锁外执行并记录 invocation id、affinity、deadline、panic/fault、预算和 generation；stale result 丢弃，连续 fault 进入隔离。

### **P0-05** · DocumentToolkitRegistry 的发布与 drop 仍混在 mutex 临界区

`RegistryState::publish_snapshot()`在 registry mutex 内遍历 descriptor；register 先写两个 map 再 publish，clear/commit_close 又在相同锁域内重建 snapshot 并 drop trait-object 容器。descriptor panic、重入或 callback drop 可能留下 map 与 snapshot 不一致；Host 的 DirtyRegistry 也没有共享 lifecycle commit。

目标是注册时锁外取得并验证 owned descriptor，锁内只处理纯数据 candidate，Arc/drop 移到锁外；Toolkit、Dirty、document close/save 通过一个 lifecycle transaction 提交，callback fault 转成 typed terminal receipt。

## 5. P1：激活、owner 与跨 registry 事务

| ID | 当前差距 | 必须重构 |
|---|---|---|
| **P1-01** | active_extensions 只被 tests/status 读取。 | Reconciler 订阅 manager generation 并发布 desired/mounted receipt。 |
| **P1-02** | startup direct registration 绕过 phase、enablement、fault。 | App 只提交输入，mount 由 manager 决定。 |
| **P1-03** | project native report 只替换 manager rows。 | project generation 生成完整 mount/revoke diff。 |
| **P1-04** | OwnedContribution 只有 owner string+ticket。 | 使用 package/build/plugin/instance generation。 |
| **P1-05** | direct caller 统一名为 editor.extension.direct。 | builtin/direct 也分配稳定 ModuleId 与实例代。 |
| **P1-06** | Builtin source 折叠所有 builtin owner。 | source 携 module、package、build 和 generation。 |
| **P1-07** | command/view/scene/overlay/store 没有共同 commit id。 | prepare plan 覆盖所有 family，单一 commit receipt。 |
| **P1-08** | 后段失败没有跨 family rollback。 | candidate 全部可逆，commit 失败保持旧代可见。 |
| **P1-09** | runtime consumer 在 store 成功后另行安装。 | consumer 纳入同一个 owner mount plan。 |
| **P1-10** | 只有 template/pane 支持局部替换。 | reload 以 owner generation 整批替换。 |
| **P1-11** | materialization 可能部分 family 成功。 | package generation all-or-nothing publish。 |
| **P1-12** | manager schema 漏 settings、pane、overlay、factory、field editor。 | exhaustive contribution schema 让新 family 编译期接入。 |
| **P1-13** | ticket ownership 无完整 family receipt。 | MountReceipt 列出每个 stable contribution id。 |
| **P1-14** | revoke 不等待 snapshot/route/callback reader。 | reader lease 归零后才返回 Quiesced。 |
| **P1-15** | disable/reload 没有 drain deadline 或 leak census。 | 明确 Drain/Cancel/Quarantine/Restart terminal policy。 |

## 6. P1：schema、capability 与 admission

| ID | 当前差距 | 必须重构 |
|---|---|---|
| **P1-16** | snapshot 只保存 batch capability。 | 每项保存 effective predicate 与 policy generation。 |
| **P1-17** | importer item capability 不进入 query。 | importer visible/executable 使用同一 predicate。 |
| **P1-18** | scene mode 不检查 capability。 | register/create/push/update 都检查 owner generation。 |
| **P1-19** | overlay 漏 batch/plugin activation capability。 | preparation 取并集，撤销时立即 quiesce。 |
| **P1-20** | capability 是任意 String，无 namespace/bytes 预算。 | canonical CapabilityId、长度和总量门。 |
| **P1-21** | activation 与 feature requirement 混用。 | 拆成 activation、feature、principal 三类。 |
| **P1-22** | PluginContributionId 无 version/build/principal。 | ExtensionOwnerRef 绑定 package/build/generation/trust。 |
| **P1-23** | ticket/generation saturating_add 后语义不明。 | checked exhaustion 进入 terminal degraded。 |
| **P1-24** | descriptor 只做 nonblank 检查。 | 字段、整批 count/depth/bytes/locale 预算。 |
| **P1-25** | zui template 主要检查 suffix。 | AssetRef/ArtifactId canonical path 与 owner 校验。 |
| **P1-26** | scene-mode prepare 会执行有副作用 factory。 | declarative probe 与 commit 后实例化分离。 |
| **P1-27** | graph node/pin/type/schema cross-reference 不闭合。 | GraphSchemaCompiler 输出 typed schema artifact。 |
| **P1-28** | timeline descriptor 缺 clock/binding compatibility。 | 编译 TrackTypeSchema 并接 Editor14。 |
| **P1-29** | Inspector first-match 无 priority/specificity。 | typed predicate、priority、owner tie-break，歧义 fail-close。 |
| **P1-30** | registry 与 batch 重复校验，family 会漂移。 | 单一 schema builder 生成 SDK/validation/index。 |

## 7. P1：产品消费、callback 与 fault domain

| ID | 当前差距 | 必须重构 |
|---|---|---|
| **P1-31** | drawer contribution 无真实 production reader。 | 接入 drawer host，否则标记 Unavailable。 |
| **P1-32** | settings page 未形成 shared store 到 UI 闭环。 | Editor12 提供 settings provider/UI mount。 |
| **P1-33** | graph/timeline descriptor 没有 open/edit/save/compile consumer。 | domain owner 接入完整产品链。 |
| **P1-34** | executable command registry 与 snapshot family 分裂。 | command mount lease 与 manifest 同代拥有。 |
| **P1-35** | field editor 无完整 catalog/owner/priority 注册。 | SDK schema 显式加入 factory 与 callback policy。 |
| **P1-36** | active_extensions 的 product assertion 缺失。 | 用 manager-to-workbench integration test 取代 status 断言。 |
| **P1-37** | pane-data snapshot 串行、无 timeout/cancel/budget。 | supervisor 分帧采集并返回 partial/stale/fault receipt。 |
| **P1-38** | Inspector can_handle 可能在 shell mutex 内执行。 | 锁外匹配 immutable target descriptor。 |
| **P1-39** | Inspector build/surface/validate 无 invocation identity。 | 每阶段携 CallbackInvocationId 并进入 quarantine。 |
| **P1-40** | field-editor factory 未声明 thread/reentry/cost。 | CallbackPolicy 成为注册必填字段。 |
| **P1-41** | catch_unwind 不能处理 hang、abort、FFI UB。 | 高风险 native callback 用可终止 worker/进程隔离。 |
| **P1-42** | overlay 输出无 primitive/string/path/time 预算。 | provider 级 count/bytes/time gate 与降级。 |
| **P1-43** | overlay fault 无 generation reset/probe。 | quarantine 绑定代，repair 后只创建新代。 |
| **P1-44** | active scene mode 不随 owner disable 自动退出。 | 先 exit/pop、等待 fence，再撤销 factory。 |
| **P1-45** | callback 普遍没有 affinity/cancel/deadline/reentry。 | schema 强制声明并由 supervisor 执行。 |

## 8. P1：store、snapshot、toolkit 与文档生命周期

| ID | 当前差距 | 必须重构 |
|---|---|---|
| **P1-46** | 每次 contribute/revoke clone 全 family map。 | 以 100/1k/10k 基准决定 persistent/chunked index。 |
| **P1-47** | ticket 保留整 batch，与 snapshot 重复持有 callback。 | retention 只留撤销所需 delta，lease 单独拥有 callback root。 |
| **P1-48** | 空 batch 仍推进 generation/journal。 | no-op 返回 Unchanged，不污染观察者。 |
| **P1-49** | journal 只有固定 count，无 bytes/age/criticality。 | count+bytes+age budget 与 ResetRequired。 |
| **P1-50** | delta 只有 kind/ticket/count，没有 exact ids。 | bounded changed-id page、before/after generation/digest。 |
| **P1-51** | 多数 snapshot family 不暴露 source/ticket。 | 每项保存 OwnerRef、ContributionId、MountGeneration。 |
| **P1-52** | query 没有 page/bytes/deadline，caller 全量 collect。 | indexed bounded query 与 family projection。 |
| **P1-53** | toolkit snapshot 每次重建 descriptor array。 | entry 保存 owned descriptor，纯数据增量发布。 |
| **P1-54** | mutex poison 直接 into_inner，无 invariant receipt。 | 恢复后校验双 map/snapshot，失败停止 admission。 |
| **P1-55** | direct toolkit save panic 逃出 API。 | registry boundary 返回 typed ToolkitFault。 |
| **P1-56** | autosave_source_path 不持 save lease。 | 在同一 document revision lease 冻结 source identity。 |
| **P1-57** | close lease 只阻止 active save，不阻止 pane/provider callback。 | document close 纳入 callback/job fence。 |
| **P1-58** | snapshot generation 与 manager/plugin generation 未绑定。 | DocumentToolkitSnapshot 携 owner/project generation。 |
| **P1-59** | clear 先切 map 再 drop retired entries，缺失败 receipt。 | two-phase retire，返回每个 descriptor 的 terminal status。 |
| **P1-60** | ContributionChange 不能表达 project/reload/owner cause。 | typed lifecycle cause、causal parent、continuation cursor。 |

## 9. P2：可观测性、性能与维护风险

1. **P2-01**：registry snapshot 缺 schema version 与 migration policy；加入 versioned manifest。
2. **P2-02**：diagnostics 只有字符串；改为 code、severity、owner、path、generation、remediation。
3. **P2-03**：duplicate/namespace 错误未返回 candidate digest；补 deterministic rejection receipt。
4. **P2-04**：BTreeMap 顺序被隐式当 priority；增加显式 priority/specificity 字段。
5. **P2-05**：owner label 与 security principal 混用；分离展示名、package id、trust identity。
6. **P2-06**：plugin manager fault 没有用户可执行 repair action；提供 retry/quarantine/restart 建议。
7. **P2-07**：scene/overlay fault 只有最后一条错误；保留 bounded fault history 和采样指标。
8. **P2-08**：ContributionStore 缺高水位和 bytes 计数；增加 live/retired allocation telemetry。
9. **P2-09**：snapshot reader lifetime 无 profiling；增加 generation hold-time histogram。
10. **P2-10**：toolkit save latency 与 callback fault 未接入统一诊断总线；发布 typed events。
11. **P2-11**：插件 API 没有 declared thread affinity；SDK lint 和 registration rejection 必须补齐。
12. **P2-12**：测试集中于 happy-path/duplicate，缺 reload/close/panic/reentry/scale matrix。
13. **P2-13**：没有跨平台 native unload、ABI、long-running leak 资格测试；交由 Plugins01 组合验证。
14. **P2-14**：固定 direct owner 使多插件调试难以归因；所有 builtin 也要使用稳定 instance id。
15. **P2-15**：文档将 descriptor/store 存在写成 feature available；availability 必须来自 mounted receipt。

## 10. 40 个资格门

以下资格门在实现前全部保持 Open，任何一个失败都不能把扩展系统标记为 production-ready：

`EXT-GATE-01` package admission、`EXT-GATE-02` manifest schema、`EXT-GATE-03` build identity、`EXT-GATE-04` trust principal、`EXT-GATE-05` loading phase、`EXT-GATE-06` capability predicate、`EXT-GATE-07` desired set、`EXT-GATE-08` mounted set、`EXT-GATE-09` owner generation、`EXT-GATE-10` cross-family prepare、`EXT-GATE-11` candidate digest、`EXT-GATE-12` atomic publish、`EXT-GATE-13` rollback、`EXT-GATE-14` command lease、`EXT-GATE-15` view lease、`EXT-GATE-16` scene-mode lease、`EXT-GATE-17` overlay lease、`EXT-GATE-18` store ticket、`EXT-GATE-19` runtime-consumer lease、`EXT-GATE-20` template replacement、`EXT-GATE-21` reader fence、`EXT-GATE-22` callback fence、`EXT-GATE-23` job cancellation、`EXT-GATE-24` quiesce deadline、`EXT-GATE-25` revoke receipt、`EXT-GATE-26` leak census、`EXT-GATE-27` Inspector isolation、`EXT-GATE-28` field-editor isolation、`EXT-GATE-29` pane-data isolation、`EXT-GATE-30` native process boundary、`EXT-GATE-31` overlay budget、`EXT-GATE-32` scene-mode fault policy、`EXT-GATE-33` toolkit save lease、`EXT-GATE-34` toolkit close lease、`EXT-GATE-35` dirty lifecycle commit、`EXT-GATE-36` project replacement、`EXT-GATE-37` reload repair、`EXT-GATE-38` snapshot cursor/resync、`EXT-GATE-39` scale benchmark、`EXT-GATE-40` cross-platform unload。

## 11. 重构顺序与验收

1. **Owner 与 admission**：先定义 `ExtensionOwnerRef`、package/build/principal、capability predicate、loading phase 和 desired set；禁止新增 direct permanent registration。
2. **统一 mount transaction**：实现跨 command/view/scene/overlay/store/runtime consumer 的 candidate、commit id、rollback 和 `ExtensionMountLease`，再把 manager active snapshot 接到 Workbench reconciler。
3. **Callback supervisor**：将 Inspector、field editor、pane-data、scene mode、overlay 全部迁移到统一 boundary，补 deadline、cancel、预算、quarantine 和 generation CAS。
4. **Retirement**：实现 disable/project close/reload 的 admission close、reader/job/callback fence、逆依赖 revoke、leak census 和 terminal receipt；Plugins01 只在 receipt 完成后允许 unload。
5. **Toolkit/Dirty 合并**：descriptor 锁外冻结，纯数据 snapshot，DocumentLifecycle transaction 统一 toolkit、dirty、autosave、close 和 project replacement。
6. **真实产品消费**：为 drawer/settings/graph/timeline/asset/scene/overlay 接入真实 host；没有 consumer 的 contribution 必须显示 Unavailable，不得用 descriptor 或 queued 文案冒充。
7. **资格验证**：做 1/10/100/1k/10k owner 与 contribution 基准，reload/disable/close/panic/reentry/transport fault、多窗口和 native unload 矩阵；动态结果必须附 commit、generation、receipt 与 leak census。

## 12. 本轮边界

本轮仅静态读取当前 Zircon 与 `dev/UnrealEngine`、`dev/godot`、`dev/Fyrox`、`dev/bevy`、`dev/Graphics` 参考源码，刷新证据和重构要求；没有运行 Cargo、GUI、插件 reload、panic/fault injection、native unload、跨进程或规模 benchmark。旧 Editor50 的 P0/P1/P2 结论全部仍为 Open，但本报告纠正了 current HEAD、路径范围与统计。Editor06 继续拥有 discovery/enablement/live-reload 流程，Editor08 拥有 command/remote admission，Editor05 拥有 Inspector/property 语义，Editor02 拥有 document/save/recovery，Plugins01 拥有 native ABI/unload；Editor123 只拥有这些 domain 如何以 owner-generation mount、quiesce、revoke 和 reconcile 收敛为一个扩展 runtime contract。

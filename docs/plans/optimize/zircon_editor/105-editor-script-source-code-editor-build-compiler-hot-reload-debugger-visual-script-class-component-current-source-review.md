---
title: Editor Script Source、Code Editor、Build、Compiler、Hot Reload、Debugger、Visual Script、Class 与 Component 当前源码复核
category: zircon_editor
report_id: Editor105
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor31
refreshes:
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
related_code:
  - zircon_editor/src/core/script_build
  - zircon_editor/src/core/logging/jump.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_runtime_interface/src/script_diagnostics/mod.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_source.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_record.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_state.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/script/vm/module/script_module.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_slot_lifecycle.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/generated_manifest.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/runtime_projection.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src
  - zircon_app/Cargo.toml
  - examples/woc/zircon-project.toml
  - examples/woc/scripts/woc_game/plugin.toml
tests:
  - zircon_editor/src/core/script_build/tests.rs
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/state_migration.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/reflection.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/performance/01/2026-08-16-editor-core-script-build-generation-current-architecture-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Blueprint.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Public/BlueprintEditor.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Public/BlueprintCompilationManager.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintCompilationManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload
  - dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding
  - dev/godot/editor/script/script_editor_plugin.h
  - dev/godot/editor/script/script_editor_plugin.cpp
  - dev/godot/core/object/script_language.h
  - dev/godot/core/debugger/script_debugger.h
  - dev/Fyrox/editor/src/plugins/inspector/editors/script.rs
  - dev/Fyrox/editor/src/lib.rs
reference_sources:
  - E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - E:/Git/zr_vm/zr_vm_language_server
  - E:/Git/zr_vm/zr_vm_lib_debug
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 60 open
  p2: 12 open
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor31/105 · Script Source、Code Editor、Build、Compiler、Hot Reload、Debugger、Visual Script、Class 与 Component 当前源码复核

## 1. 结论

Zircon Runtime 的脚本底层不是空壳。`vm_plugin_package_discovery` 有深度、entry、manifest bytes、path bytes、wall-time 和 cancellation 预算；`VmPluginManager` 有 backend family、slot 与 generation；`HotReloadCoordinator` 有状态快照、schema migration、reflection prepare、activation、commit/rollback；`zr_vm_language` 运行时插件有 host module、reflection table、call-site、生命周期 export 和真实 project backend。这些是可保留的工程底座。

Editor 端却没有把它们组成产品闭环。`ScriptBuildOrchestrator` 仍是孤立的 typed 状态机：有 300 ms debounce、1,000 ms first-event cap、20 path/64 KiB admission、active + one queued request 和 `Watch < Command < Play` promotion，但 `CompileModules`、`ValidateLedger`、`RefreshBindings` 没有 production executor、watch owner、shared job、compiler adapter、artifact receipt、install receipt 或 Play waiter。它把 request id 直接当 generation，失败时还会清理 queued request 和新 watch path。

Runtime 启动则反过来在 `load_startup_scripts()` 同步 discover package，再在 `load_discovered_package()` 内直接编译并启动 session。这绕过 Editor build generation，形成第二 compiler authority；Editor 的 build 状态不会约束 Client、Server、Cook 或 Play 使用的 artifact。

默认装配同样不可信。`zircon_app/Cargo.toml` 的 `target-client` 与 `target-editor-host` 都只开通 generic `script`，没有 `first-party-zr-vm-language-runtime-plugin` 或 `backend-zr-vm`；WOC manifest 却要求 `backend = "zr_vm:project"`。缺 backend 时 selector 可解析，直到 load 才报告 unavailable，项目 required 与默认 profile 因而不相容。

Editor 没有 Script Source/Script Module/Script Class/Script Component/Visual Script 资源、代码文档、LSP client、symbol index、rename、breakpoint、debug session 或 profile pane。唯一显式命令是 `Show Script Build Logs`；诊断 jump 最终只发 `OpenAsset(path)`，打开后把行列写进 status line，不把文档光标定位到 source range。

Scene binding 只是 `package/module/enabled/update/fixed_update + BTreeMap<String, JSON>`，World 加载后整体编码成动态 `script.bindings`。它缺 stable class/component/field id、字段类型/默认值/可见性/约束、schema version、redirect、instance override provenance 和 typed Inspector contract。已有 VM package state migration 不能替代每个 scene component 的字段迁移。

`E:/Git/zr_vm` 已经有 parser/AOT/CLI、LSP 的 diagnostics/completion/hover/definition/reference/rename/semantic tokens/inlay/code actions/formatting，以及 debug library 的 line/function/data breakpoint、condition/logpoint、stack/evaluate/snapshot/profile。Zircon 没有 versioned adapter 暴露它们；不得另造 regex parser 或静态 debug route。

因此目标是唯一链：`ScriptWorkspace + transactional Source/VisualScript Documents -> shared semantic compiler -> immutable ScriptArtifactSet -> qualified InstallReceipt -> VM slot generation`。文本、图、Class/Component、Editor Build、Play、Cook、Hot Reload 和 Debug 必须共享 source/artifact/install identity。

## 2. 当前物理范围与证据

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与说明 |
|---|---:|---|
| Zircon Editor/Runtime/App/WOC selected | **72 / 13,067 / 11,913 / 484,179 / 109 / 0** | 当前工作树递归选择；fingerprint `ba1a4c908f8776ffa4969f9e52afde4dd434a2c6a3aa0f1d990f896235a623be` |
| Unreal/Godot/Fyrox/HotReload reference | **118 / 35,850 / 29,671 / 1,222,906 / 0 / 0** | Blueprint/Kismet/Compilation/HotReload/LiveCoding、Godot Script/Debugger、Fyrox script inspector；fingerprint `5160abe3e33330604f5eeeed027b79f42f30038195601f6903bae5806f1a219e` |
| 去重 union | **190 / 48,917 / 41,584 / 1,707,085 / 109 / 0** | 两组路径不重叠；fingerprint `18e5ad21a11cf5f523547dcb6cf6d84c1e110035d44a164c5daf1378431d60b2` |

当前 WOC `scripts/woc_game` 有 **817 个 `.zr`、247,582 行、9,978,430 bytes、354 个 `.zrp`、37 个 `.zro`**。`plugin.toml` 声明 `zr_vm:project`、`preserve_state`、64/128 MiB memory policy 和 `main` entry；`zircon-project.toml` 只声明 package roots/startup packages。规模和 37 个局部 `.zro` 不等于统一 artifact set、dependency manifest、cook receipt 或默认 profile 可运行。

逐层事实：

1. `ScriptBuildRequest` 固定三步，generation 由 request id 构造；没有 source revision、module dependency snapshot、compiler/backend identity、artifact digest 或 install generation。
2. `ScriptBuildOrchestrator::complete` 在失败/取消时丢弃 queued request 与 pending watch paths；source 事实可能比失败 build 更新，却被状态机删除。
3. `ScriptBuildDiagnosticsSink` 逐条同步格式化并调用 `EditorLogService`；`ScriptDiagnostic` 只有 severity/code/module/message 与可选 path/line/column，没有 source revision、range、related info、fix-it、symbol、artifact 或 backend。
4. 全仓 production caller 只有 script-build 自身、测试和日志跳转；没有 watch、Build command、Play、job executor、VM compile adapter 或 commandlet 调用者。
5. `ResourceKind` 现有资产枚举没有 Script Source、Script Module、Script Class、Script Component、Visual Script、Debug Map 或 Script Artifact。
6. first-party Editor catalog 当前注册 Navigation/Neural 等已有 provider，没有 ZrVM language/editor provider；runtime catalog 的 ZrVM provider通过 feature 选择，不是默认产品能力。
7. `load_startup_scripts` 同步 discover/filter/load，`load_discovered_package` 在 runtime 线程直接 compile/open workspace/start session；没有“qualified artifact required”或 source generation fence。
8. `HotReloadCoordinator` 的 save/migrate/prepare/activate/rollback 是真实底座，但 production source 没有从 Editor build 或 file watch 触发 `hot_reload_discovered_slot` 的产品路径。
9. VM state blob 以 JSON 保存 package state；`stateSchema` 是 package export，不是 Scene Script Component 字段 schema、default diff 或 override migration。
10. Scene IO 将 binding properties 放入动态组件，runtime 再按字符串和 scalar 索引访问；没有 typed ECS registration、class identity、field redirect 或 inspector transaction。
11. `LogJump::script_location` 保存 path/line/column，但 host 成功只发 `OpenAsset(path)` 并写 status；没有 SourceDocument revision/range navigation。
12. `zr_vm_language/runtime` 有 real backend/host/reflection tests，但没有把 compiler structured diagnostics、debug transport、LSP lifecycle 或 artifact manifest 发布到 Zircon Editor。
13. `target-client`、`target-editor-host` 都不含 `backend-zr-vm`；generic `script` 会让配置看似可用，缺 provider 的错误延迟到 package load。

## 3. 参考引擎对照

- Unreal `UBlueprint`、`FBlueprintEditor` 与 `BlueprintCompilationManager` 将 source graph、generated/skeleton class、dependency queue、compile result、reinstance、debug object 和 graph navigation 分开；Zircon 应吸收其 identity/queue/receipt 边界，而不是复制 UObject 全局扫描。
- Unreal HotReload/LiveCoding 证明 compile、load patch、reinstance、completion 是不同阶段；Zircon 更适合 immutable artifact + slot generation + safe-point commit，Editor 不能直接替换 VM instance。
- Godot `ScriptEditor`/`ScriptLanguage`/`ScriptDebugger` 提供多文档、unsaved/save-all、reload、breakpoint、stack/locals/evaluate、profiling 和语言扩展接口；Zircon 当前只有通用 asset open 与 log filter。
- Fyrox 把 Script 类型选择、反射 Inspector、外部 IDE 打开、build queue、child process output 和 play-after-build 连成 authoring workflow；这至少是当前字符串 binding 的产品下限。
- Bevy 本地范围没有 first-party script Editor，不能作为缺失能力的辩护；Unity Graphics 参考集不含 Unity Editor scripting/Visual Scripting，本报告不把它们伪装成证据。
- `E:/Git/zr_vm` 是直接语言依赖，应以 revisioned LSP/debug/compiler adapter 接入，不另写简化语义层。

## 4. Owner 边界与目标链

| 领域 | 唯一 owner | Editor105 必须消费/提供 |
|---|---|---|
| document/dirty/save/recovery | Editor02 | ScriptWorkspace、SourceDocument、external conflict、revision |
| asset/reference/dependency | Editor04 | Script/VisualScript/Class/Component stable asset identity |
| reflection/property | Runtime reflection + Editor05 | compiled class schema、typed field patch、override provenance |
| build/jobs/diagnostics | Editor09 + Editor11 + Tooling03 | bounded compile job、diagnostic pages、artifact/cook receipt |
| language semantics | ZrVM owner + versioned adapter | compiler/LSP/debug schema、source ranges、debug map |
| runtime install/reload | Runtime07 | qualified artifact、slot generation、state migration、rollback |
| Play/session | Editor07 | build-before-run、target/session identity、attach receipt |

## 5. P0：先关闭伪闭环

| ID | 当前差异 | 必须重构 |
|---|---|---|
| P0-1 | required `zr_vm:project` 与默认 Client/Editor feature 不相容 | profile preflight、provider capability receipt、缺 backend 在打开项目之前失败 |
| P0-2 | Editor build 状态机无 caller，Runtime startup 同步重复 compile | 唯一 SharedBuild/Artifact owner；Runtime 只接受 qualified artifact |
| P0-3 | 无 Script Source 产品，diagnostic jump 只是假定位 | SourceDocument + real code editor + revision/range navigation |
| P0-4 | JSON `script.bindings` 不是 Class/Component authoring contract | typed class/component schema、stable field id、migration、Inspector transaction |
| P0-5 | Visual Script、LSP、Debugger 完全未接入 | versioned ZrVM adapter；text/graph 共用 compiler/artifact/debug map |

## 6. P1：Workspace、Language Service 与 Source

| ID | 差异 | ID | 差异 |
|---|---|---|---|
| P1-01 | project scripts 无 workspace id/schema | P1-02 | package/module/path/name 混作 identity |
| P1-03 | `.zr` 不在 asset catalog | P1-04 | 无 transactional source document |
| P1-05 | encoding/line map 合同缺失 | P1-06 | 无 LSP process/library lifecycle owner |
| P1-07 | LSP sync 未接 Editor02 revision | P1-08 | completion/hover 无 reflection generation |
| P1-09 | definition/reference/rename 无跨资产事务 | P1-10 | fix-it/code action 无安全模型 |
| P1-11 | generated/source 边界未定义 | P1-12 | symbol/search index 无规模预算 |
| P1-13 | request id 冒充 source generation | P1-14 | build 失败删除 source/watch 事实 |
| P1-15 | changed path 不是 dependency plan | P1-16 | `CompileModules` 无 executor |
| P1-17 | `ValidateLedger` 只是状态名 | P1-18 | `RefreshBindings` 只是状态名 |
| P1-19 | compiler 无 structured diagnostics | P1-20 | diagnostic ingress 逐条同步无 page budget |
| P1-21 | 散落 `.zro` 无 artifact manifest | P1-22 | `.zri`/debug map 生命周期不清 |
| P1-23 | artifact/cache/清理无 owner | P1-24 | Play build-before-run 未闭合 |
| P1-25 | Cook/headless 无唯一入口 | P1-26 | startup 同步 discovery/compile |
| P1-27 | startup 与 Editor build 双 authority | P1-28 | hot reload 无 production trigger |
| P1-29 | slot 依赖 package name 无 session id | P1-30 | package state ≠ component instance state |
| P1-31 | state migration 无 Editor preflight | P1-32 | rollback 无产品 projection |
| P1-33 | lifecycle export 只靠可选名称 | P1-34 | VM 并发靠全局 mutex |
| P1-35 | backend unavailable 延迟到 load | P1-36 | 无 Script Class asset identity |
| P1-37 | 无 Script Component asset identity | P1-38 | field 无 stable id |
| P1-39 | field type/default/constraint 缺失 | P1-40 | default 与 instance override 不可区分 |
| P1-41 | visibility/permission 缺失 | P1-42 | update/fixed flags 过于粗糙 |
| P1-43 | dynamic binding 绕过 typed ECS | P1-44 | required sibling/conflict 规则缺失 |
| P1-45 | class/component graph 不进 cook/rename | P1-46 | 无 Visual Script document/schema |
| P1-47 | Visual Script 不得另建简化 runtime | P1-48 | node catalog 无 typed provider contract |
| P1-49 | pin inference/conversion 缺失 | P1-50 | control/data flow/cycle 语义缺失 |
| P1-51 | graph refactor/semantic diff 缺失 | P1-52 | graph debug mapping 缺失 |
| P1-53 | 无 Script Workspace/editor toolkit | P1-54 | 无真实 Build 命令/状态 projection |
| P1-55 | 无 breakpoint 存储/rebind | P1-56 | 无 debug session lifecycle |
| P1-57 | 无 stack/locals/watch/evaluate | P1-58 | 无 profiling/coverage integration |
| P1-59 | 无多 target/session/package debug model | P1-60 | 无 fault/scale/performance/migration qualification |

## 7. P2 与 Gate

P2 全部 Open：language backend plugin SDK、text/graph round-trip、live value/time-travel、remote compile/cache、dependency registry、sandbox visualization、deterministic replay、semantic merge、script budget annotation、Interp/Binary/AOT parity、cross-engine authoring benchmark、supply-chain signing。

32 个 Gate 当前为 **32 Fail / 0 Partial / 0 Pass**。最低门禁必须证明：source/artifact/install/request identity 全链可追溯；817 个 WOC module 的 full/incremental build 产出完整 manifest；UI 线程无 compile/process wait/file I/O；stale diagnostic、stale install、reload failure、external edit、LSP crash/restart、debug detach 都有唯一 terminal receipt；Client/Server/Cook/Editor 使用同一 compiler/artifact；Class/Component field rename/migrate 不丢 instance override。

## 8. 分层重构顺序

1. **M0 Truthfulness**：统一 feature profile；项目 required provider 在 preflight 明确失败；冻结 ZrVM adapter revision，删除“generic script 即可运行”的假设。
2. **M1 Workspace/Source**：在 Editor02 document/transaction owner 下引入 ScriptWorkspace、SourceDocument、VisualScriptDocument、asset identity、LSP lifecycle、real range navigation。
3. **M2 Shared Build/Artifact**：把 ScriptBuildOrchestrator 接入 Editor09 job authority；分离 source revision、compile generation、artifact digest、install generation，建立 immutable content-addressed ArtifactSet 与 bounded diagnostics。
4. **M3 Class/Component/Scene**：由 reflection + Editor05 提供 typed class/component schema、stable field ids、default/override provenance、legacy `script.bindings` migration 和 cook/reference graph。
5. **M4 Runtime Install/Reload**：Runtime07 只接 qualified artifact；为 session/slot/target 建 identity，保留 HotReloadCoordinator 的 prepare/commit/rollback，分别迁移 package state 与 component state。
6. **M5 Visual Script**：typed graph/node/pin schema、provider catalog、cycle/effect/type validation、semantic diff；文本和图 lowering 到同一 module/interface/artifact。
7. **M6 Debugger/Profiler**：接 ZrVM debug adapter，提供 attach/session/thread/frame/value/evaluate/breakpoint/profile/coverage，并以 artifact debug map 处理 stale source。
8. **M7 Play/Cook/Scale**：Editor07、Tooling03、multi-PIE/client/server、fault injection、LSP restart、artifact corruption、WOC 817-file cold/warm/incremental/soak 资格。

## 9. 禁止临时修补

- 不得在 Console 旁加多行文本框就宣称 Script Editor 完成。
- 不得用 regex、扩展名或 stderr 字符串重写 ZrVM parser/LSP/debug 语义。
- 不得让 Editor 与 Runtime 各自编译后比较时间戳；不得继续用 request id 同时表示 source/artifact/install generation。
- 不得在 build 失败时清空更新的 watch/source 事实，或把逐条 log emission 当诊断分页。
- 不得给 JSON map 加几个 hardcoded Inspector 字段就命名为 Script Component。
- 不得把 Visual Script 做成独立解释器、独立类型系统或 shipping-only graph runner。
- 不得先画断点按钮，再用静态 route/fixed feedback 冒充 debugger；不得绕过 HotReloadCoordinator 直接替换 VM instance。
- 不得把 817 `.zr`、37 `.zro` 或局部测试数量当作默认 Editor/Client 可运行证明。

## 10. 验证边界

本轮完成当前工作树递归枚举、脚本 build/runtime/catalog/scene binding/hot-reload/feature graph 逐层阅读，并核对 Unreal、Godot、Fyrox、HotReload/LiveCoding 与外部 ZrVM 能力；没有修改 Rust/C/TOML 生产实现。没有运行 Cargo、ZrVM native build、LSP/debug protocol、WOC full compile、Play、Cook、hot reload、fault、scale 或 UI 动态测试。`source_recheck_required: true` 表示共享 dirty worktree 与外部 ZrVM 在途状态；实施前必须重算 72 文件 manifest、外部 adapter revision、feature profile 和 startup order。
